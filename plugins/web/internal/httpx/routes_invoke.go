package httpx

import (
	"bufio"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"strings"
	"sync"
	"time"

	"github.com/marqdo/marqdo/plugins/web/internal/plugin"
)

type invokeRoute struct {
	Method     string
	FnPath     string
	BodyMode   string
	ReturnMode string
}

func parseInvokeRoute(v any) invokeRoute {
	m, _ := v.(map[string]any)
	method := strings.ToUpper(strOpt(m, "method", "POST"))
	body := strings.ToLower(strOpt(m, "body", "json"))
	ret := strings.ToLower(strOpt(m, "return", "json"))
	fn := strOpt(m, "fn", "")
	if fn == "" {
		fn = strOpt(m, "function", "")
	}
	return invokeRoute{
		Method:     method,
		FnPath:     fn,
		BodyMode:   body,
		ReturnMode: ret,
	}
}

func (st *state) mountInvokeRoutes(mux *http.ServeMux) {
	for _, key := range sortedKeys(st.invokeRoutes) {
		cfg := parseInvokeRoute(st.invokeRoutes[key])
		path := routePathKey(key)
		method := cfg.Method
		if method == "" {
			method = http.MethodPost
		}
		pattern := method + " " + path
		routeCfg := cfg
		mux.HandleFunc(pattern, func(w http.ResponseWriter, r *http.Request) {
			if r.Method != method {
				http.Error(w, "method not allowed", http.StatusMethodNotAllowed)
				return
			}
			st.handleInvoke(w, r, routeCfg)
		})
	}
}

func (st *state) handleInvoke(w http.ResponseWriter, r *http.Request, route invokeRoute) {
	queryMap := queryToMap(r.URL.RawQuery)

	var payload map[string]any
	switch route.BodyMode {
	case "query":
		payload = queryMap
	case "raw":
		body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 8<<20))
		if err != nil {
			jsonErr(w, http.StatusBadRequest, fmt.Sprintf("body: %v", err))
			return
		}
		payload = map[string]any{"payload": string(body)}
	case "form":
		body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 8<<20))
		if err != nil {
			jsonErr(w, http.StatusBadRequest, fmt.Sprintf("body: %v", err))
			return
		}
		payload = formToMap(body)
	default:
		body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 8<<20))
		if err != nil {
			jsonErr(w, http.StatusBadRequest, fmt.Sprintf("body: %v", err))
			return
		}
		if len(body) == 0 {
			payload = queryMap
		} else {
			var raw map[string]any
			if err := json.Unmarshal(body, &raw); err != nil {
				jsonErr(w, http.StatusBadRequest, fmt.Sprintf("json: %v", err))
				return
			}
			if raw == nil {
				payload = queryMap
			} else {
				for k, v := range queryMap {
					if _, exists := raw[k]; !exists {
						raw[k] = v
					}
				}
				payload = raw
			}
		}
	}

	// Bind common auth / client headers for OpenAI-compatible and API-key surfaces.
	if auth := strings.TrimSpace(r.Header.Get("Authorization")); auth != "" {
		if _, exists := payload["authorization"]; !exists {
			payload["authorization"] = auth
		}
	}
	if key := strings.TrimSpace(r.Header.Get("X-Api-Key")); key != "" {
		if _, exists := payload["api_key"]; !exists {
			payload["api_key"] = key
		}
	}

	args := invokeArgsFromPayload(payload)

	// M02: live SSE — stream host_stream_publish events while CallLibPath runs.
	// return=sse-live|sse_live uses MARQDO_EVENT_FORWARD NDJSON bridge (request-local).
	// Also auto-upgrade json routes when the OpenAI-compatible body asks stream:true
	// so token deltas flush before the lib call returns (Q06 / workbench).
	liveSSE := route.ReturnMode == "sse-live" || route.ReturnMode == "sse_live"
	if !liveSSE && payloadWantsStream(payload) {
		liveSSE = true
	}
	if liveSSE {
		st.handleInvokeLiveSSE(w, r, route, args)
		return
	}

	value, err := plugin.CallLibPath(route.FnPath, args)
	if err != nil {
		code := http.StatusInternalServerError
		msg := err.Error()
		if strings.Contains(msg, "interpreter busy") {
			w.Header().Set("Retry-After", "2")
			jsonErr(w, http.StatusServiceUnavailable, msg)
			return
		}
		if strings.Contains(msg, "unknown library") ||
			strings.Contains(msg, "call_lib_path: need") ||
			strings.Contains(msg, "not allowed") ||
			strings.Contains(msg, "no site") ||
			strings.Contains(msg, "no active host") {
			code = http.StatusBadRequest
		}
		if strings.Contains(msg, "unauthorized") || strings.Contains(msg, "Unauthorized") {
			code = http.StatusUnauthorized
		}
		jsonErr(w, code, msg)
		return
	}

	// OpenAI-compatible handlers may return `{sse: "..."}` to force event-stream
	// even when the route's declared return mode is json.
	if m, ok := value.(map[string]any); ok {
		if s, ok := m["sse"].(string); ok && strings.TrimSpace(s) != "" {
			writeSSE(w, m)
			return
		}
	}

	switch route.ReturnMode {
	case "text", "html", "text/html":
		if value == nil {
			jsonErr(w, http.StatusInternalServerError, "null result")
			return
		}
		text := fmt.Sprint(value)
		if s, ok := value.(string); ok {
			text = s
		}
		ct := "text/plain; charset=utf-8"
		if route.ReturnMode == "html" || route.ReturnMode == "text/html" {
			ct = "text/html; charset=utf-8"
		}
		w.Header().Set("Content-Type", ct)
		w.WriteHeader(http.StatusOK)
		_, _ = io.WriteString(w, text)
	case "sse", "event-stream", "text/event-stream":
		writeSSE(w, value)
	case "status":
		if value == nil {
			jsonErr(w, http.StatusInternalServerError, "null result")
			return
		}
		w.WriteHeader(http.StatusNoContent)
	default:
		code, body, err := jsonWrapInvokeValue(value)
		if err != nil {
			jsonErr(w, code, err.Error())
			return
		}
		writeJSON(w, code, body)
	}
}

// handleInvokeLiveSSE streams RuntimeEvent-shaped lines while the lib call runs.
// Client disconnect sets MARQDO_CANCEL + cancel token file so the llm plugin can stop.
func (st *state) handleInvokeLiveSSE(w http.ResponseWriter, r *http.Request, route invokeRoute, args map[string]any) {
	dir := os.TempDir()
	fwd, err := os.CreateTemp(dir, "mq-evt-*.ndjson")
	if err != nil {
		jsonErr(w, http.StatusInternalServerError, fmt.Sprintf("event forward: %v", err))
		return
	}
	fwdPath := fwd.Name()
	_ = fwd.Close()
	defer os.Remove(fwdPath)

	prev := os.Getenv("MARQDO_EVENT_FORWARD")
	_ = os.Setenv("MARQDO_EVENT_FORWARD", fwdPath)
	cancelFile, cancelErr := os.CreateTemp(dir, "mq-cancel-*.tok")
	cancelPath := ""
	if cancelErr == nil {
		cancelPath = cancelFile.Name()
		_ = cancelFile.Close()
		_ = os.Setenv("MARQDO_CANCEL_TOKEN", cancelPath)
	}
	defer func() {
		if prev == "" {
			_ = os.Unsetenv("MARQDO_EVENT_FORWARD")
		} else {
			_ = os.Setenv("MARQDO_EVENT_FORWARD", prev)
		}
		_ = os.Unsetenv("MARQDO_CANCEL_TOKEN")
		_ = os.Unsetenv("MARQDO_CANCEL")
		if cancelPath != "" {
			_ = os.Remove(cancelPath)
		}
	}()

	w.Header().Set("Content-Type", "text/event-stream; charset=utf-8")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	w.Header().Set("X-Accel-Buffering", "no")
	w.WriteHeader(http.StatusOK)
	flusher, _ := w.(http.Flusher)
	writeEvent := func(payload any) {
		b, err := json.Marshal(payload)
		if err != nil {
			return
		}
		_, _ = io.WriteString(w, "data: "+string(b)+"\n\n")
		if flusher != nil {
			flusher.Flush()
		}
	}
	writeEvent(map[string]any{
		"schema":         "RuntimeEventV1",
		"schema_version": 1,
		"type":           "run.started",
		"payload":        map[string]any{"fn": route.FnPath, "bridge": "sse-live"},
	})

	type callResult struct {
		value any
		err   error
	}
	done := make(chan callResult, 1)
	go func() {
		v, e := plugin.CallLibPath(route.FnPath, args)
		if e != nil && strings.Contains(e.Error(), "interpreter busy") {
			// Another invoke still holds the lock — surface as failed run, not hang.
			e = fmt.Errorf("interpreter busy: try again shortly")
		}
		done <- callResult{value: v, err: e}
	}()

	var mu sync.Mutex
	offset := int64(0)
	seq := uint64(1)
	poll := time.NewTicker(25 * time.Millisecond)
	defer poll.Stop()

	ctx := r.Context()
	for {
		select {
		case <-ctx.Done():
			// Propagate cancel to in-flight LLM/plugin via token file + env.
			_ = os.Setenv("MARQDO_CANCEL", "1")
			if cancelPath != "" {
				_ = os.WriteFile(cancelPath, []byte("cancelled"), 0o600)
			}
			writeEvent(map[string]any{
				"schema": "RuntimeEventV1", "schema_version": 1, "type": "run.interrupted",
				"payload": map[string]any{"reason": "client_disconnect", "cancel_propagated": cancelPath != ""},
			})
			return
		case res := <-done:
			// Drain remaining forward lines once.
			mu.Lock()
			offset = drainForward(fwdPath, offset, func(line string) {
				seq++
				writeEvent(normalizeLiveSSELine(seq, line))
			})
			mu.Unlock()
			if res.err != nil {
				writeEvent(map[string]any{
					"schema": "RuntimeEventV1", "schema_version": 1, "type": "run.failed",
					"payload": map[string]any{"error": res.err.Error()},
				})
			} else {
				writeEvent(map[string]any{
					"schema": "RuntimeEventV1", "schema_version": 1, "type": "run.completed",
					"payload": map[string]any{"result": res.value},
				})
			}
			_, _ = io.WriteString(w, "data: [DONE]\n\n")
			if flusher != nil {
				flusher.Flush()
			}
			return
		case <-poll.C:
			mu.Lock()
			offset = drainForward(fwdPath, offset, func(line string) {
				seq++
				writeEvent(normalizeLiveSSELine(seq, line))
			})
			mu.Unlock()
		}
	}
}

// normalizeLiveSSELine maps Marqdo EventBus frames (LLM delta/text, workbench
// envelopes) onto RuntimeEventV1 so chat UIs see message.delta.payload.delta
// while tokens still arrive before run.completed.
func normalizeLiveSSELine(seq uint64, line string) map[string]any {
	base := map[string]any{
		"schema": "RuntimeEventV1", "schema_version": 1, "seq": seq,
	}
	var obj map[string]any
	if err := json.Unmarshal([]byte(line), &obj); err != nil || obj == nil {
		base["type"] = "message.delta"
		base["payload"] = map[string]any{"raw": line}
		return base
	}
	// Already workbench-shaped.
	if t, _ := obj["type"].(string); t == "message.delta" || t == "turn.started" || t == "model.resolved" || t == "usage.updated" || t == "turn.completed" || t == "turn.failed" {
		for k, v := range obj {
			base[k] = v
		}
		base["seq"] = seq
		return base
	}
	// LLM plugin / net SSE: {type:delta|plan|reasoning, text:...}
	if t, _ := obj["type"].(string); t == "delta" || t == "plan" || t == "reasoning" {
		text, _ := obj["text"].(string)
		base["type"] = "message.delta"
		base["payload"] = map[string]any{"delta": text, "kind": t}
		return base
	}
	// Plan process cards — keep as message.delta for back-compat, but never
	// promote `result` into payload.delta (that caused full-answer duplication).
	if t, _ := obj["type"].(string); t == "decompose" || t == "decision" || t == "await" || t == "round" || t == "done" || t == "tool_start" || t == "tool_end" || t == "plan_act" {
		base["type"] = "message.delta"
		base["payload"] = obj
		return base
	}
	if text, ok := obj["text"].(string); ok && text != "" {
		base["type"] = "message.delta"
		base["payload"] = map[string]any{"delta": text}
		return base
	}
	base["type"] = "message.delta"
	base["payload"] = obj
	return base
}

func drainForward(path string, offset int64, onLine func(string)) int64 {
	f, err := os.Open(path)
	if err != nil {
		return offset
	}
	defer f.Close()
	if _, err := f.Seek(offset, io.SeekStart); err != nil {
		return offset
	}
	sc := bufio.NewScanner(f)
	sc.Buffer(make([]byte, 0, 64*1024), 1024*1024)
	for sc.Scan() {
		line := strings.TrimSpace(sc.Text())
		offset += int64(len(sc.Bytes()) + 1)
		if line == "" {
			continue
		}
		onLine(line)
	}
	return offset
}
