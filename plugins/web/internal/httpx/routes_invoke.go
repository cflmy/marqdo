package httpx

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/marqdo/marqdo/plugins/web/internal/plugin"
)

// legacySSEBridgeMu serializes the process-global MARQDO_EVENT_FORWARD /
// MARQDO_CANCEL_TOKEN bridge for the legacy_serial path only. Isolated execute
// never acquires this lock (ADR 0009).
var legacySSEBridgeMu sync.Mutex

// legacySSEBridgeActive is 1 while a holder has mutated process env for the bridge.
var legacySSEBridgeActive atomic.Int32

func tryAcquireLegacySSEBridge() bool {
	return legacySSEBridgeMu.TryLock()
}

func releaseLegacySSEBridge() {
	legacySSEBridgeActive.Store(0)
	legacySSEBridgeMu.Unlock()
}

type invokeRoute struct {
	Method     string
	FnPath     string
	BodyMode   string
	ReturnMode string
	Class      string
	Path       string
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
	class := strings.ToLower(strOpt(m, "class", ""))
	if class == "" {
		class = strings.ToLower(strOpt(m, "类别", ""))
	}
	return invokeRoute{
		Method:     method,
		FnPath:     fn,
		BodyMode:   body,
		ReturnMode: ret,
		Class:      class,
	}
}

func (st *state) mountInvokeRoutes(mux *http.ServeMux) {
	for _, key := range sortedKeys(st.invokeRoutes) {
		cfg := parseInvokeRoute(st.invokeRoutes[key])
		path := routePathKey(key)
		cfg.Path = path
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
	if last := strings.TrimSpace(r.Header.Get("Last-Event-ID")); last != "" {
		if _, exists := payload["last_event_id"]; !exists {
			payload["last_event_id"] = last
		}
	}
	// Merge Go ServeMux path wildcards (e.g. /turns/{id}/events → id / turn_id).
	if route.Path != "" {
		pathVals := pathValuesOf(r, route.Path)
		for k, v := range pathVals {
			if _, exists := payload[k]; !exists {
				payload[k] = v
			}
		}
		if id, ok := pathVals["id"]; ok {
			if _, exists := payload["turn_id"]; !exists {
				payload["turn_id"] = id
			}
		}
	}

	args := invokeArgsFromPayload(payload)

	liveSSE := route.ReturnMode == "sse-live" || route.ReturnMode == "sse_live"
	if !liveSSE && payloadWantsStream(payload) {
		liveSSE = true
	}

	class := resolveInvokeClass(route, liveSSE)
	switch class {
	case "execute", "short", "read", "control":
		// Isolated workers: do not take parent INVOKE_LOCK (ADR 0009).
		// control/read/short use separate admission slots; cancel/events also
		// available lock-free via /_runtime/*.
		st.handleInvokeIsolated(w, r, route, args, liveSSE, class)
		return
	default:
		// legacy_serial: parent CallLibPath (ADR 0008). Never isolate undeclared non-stream.
		st.handleInvokeParent(w, r, route, args)
	}
}

func resolveInvokeClass(route invokeRoute, liveSSE bool) string {
	c := strings.ToLower(strings.TrimSpace(route.Class))
	switch c {
	case "execute", "short", "read", "control":
		return c
	case "legacy", "legacy_serial":
		return "legacy_serial"
	}
	if liveSSE {
		return "execute"
	}
	return "legacy_serial"
}

func (st *state) handleInvokeParent(w http.ResponseWriter, r *http.Request, route invokeRoute, args map[string]any) {
	_ = r
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

// handleInvokeIsolated runs admits on a Supervisor worker (no parent
// CallLibPath / Setenv / legacySSEBridgeMu). admissionClass is execute|short|read|control.
func (st *state) handleInvokeIsolated(w http.ResponseWriter, r *http.Request, route invokeRoute, args map[string]any, liveSSE bool, admissionClass string) {
	if admissionClass == "" {
		admissionClass = "execute"
	}
	taskID := fmt.Sprintf("exec_%d", time.Now().UnixNano())
	adm, err := runtimeQuery("admit", map[string]any{
		"task_id":         taskID,
		"idempotency_key": taskID,
		"class":           admissionClass,
		"payload":         map[string]any{"fn": route.FnPath},
	})
	if err != nil {
		jsonErr(w, http.StatusInternalServerError, fmt.Sprintf("admit: %v", err))
		return
	}
	if ok, _ := adm["ok"].(bool); !ok {
		status := http.StatusServiceUnavailable
		if s, ok := adm["status"].(float64); ok && s >= 400 {
			status = int(s)
		}
		w.Header().Set("Retry-After", "2")
		jsonErr(w, status, fmt.Sprint(adm["error"]))
		return
	}
	cancelTok, _ := adm["cancel_token"].(string)
	eventPath, _ := adm["event_path"].(string)

	entryModule, fsRoot, err := resolveSiteEntry(st)
	if err != nil {
		jsonErr(w, http.StatusInternalServerError, err.Error())
		return
	}

	argsFile, err := materializeArgsJSON(args)
	if err != nil {
		jsonErr(w, http.StatusInternalServerError, fmt.Sprintf("args: %v", err))
		return
	}
	defer os.Remove(argsFile)

	disp, err := runtimeQuery("dispatch", map[string]any{
		"task_id": taskID,
		"entry": map[string]any{
			"module":   entryModule,
			"function": route.FnPath,
		},
		"fs_root":   fsRoot,
		"args_json": argsFile,
	})
	if err != nil {
		_, _ = runtimeQuery("cancel", map[string]any{"task_id": taskID})
		jsonErr(w, http.StatusInternalServerError, fmt.Sprintf("dispatch: %v", err))
		return
	}
	if ok, _ := disp["ok"].(bool); !ok {
		status := http.StatusTooManyRequests
		if s, ok := disp["status"].(float64); ok && s >= 400 {
			status = int(s)
		}
		if ra, ok := disp["Retry-After"]; ok {
			w.Header().Set("Retry-After", fmt.Sprint(ra))
		} else {
			w.Header().Set("Retry-After", "2")
		}
		jsonErr(w, status, fmt.Sprint(disp["error"]))
		return
	}
	if ep, ok := disp["event_path"].(string); ok && ep != "" {
		eventPath = ep
	}
	if tok, ok := disp["cancel_token"].(string); ok && tok != "" {
		cancelTok = tok
	}
	emitResult, _ := disp["emit_result"].(string)

	if liveSSE {
		st.streamIsolatedSSE(w, r, route, taskID, eventPath, cancelTok, emitResult)
		return
	}
	st.waitIsolatedJSON(w, r, route, taskID, eventPath, cancelTok, emitResult)
}

// cancelOnDisconnect reports whether the client opted into legacy
// disconnect=cancel (header X-Marqdo-Cancel-On-Disconnect: 1|true).
// Default is false: subscription close must not cancel the task (ADR 0009 / design §7.2).
func cancelOnDisconnect(r *http.Request) bool {
	v := strings.TrimSpace(strings.ToLower(r.Header.Get("X-Marqdo-Cancel-On-Disconnect")))
	return v == "1" || v == "true" || v == "yes"
}

func (st *state) streamIsolatedSSE(w http.ResponseWriter, r *http.Request, route invokeRoute, taskID, eventPath, cancelTok, emitResult string) {
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
		"payload": map[string]any{
			"fn":      route.FnPath,
			"bridge":  "isolated",
			"task_id": taskID,
		},
	})

	offset := int64(0)
	carry := ""
	seq := uint64(1)
	poll := time.NewTicker(25 * time.Millisecond)
	defer poll.Stop()
	ctx := r.Context()

	finish := func(clientGone bool) {
		offset, carry = drainForwardCarry(eventPath, offset, carry, func(line string) {
			seq++
			writeEvent(normalizeLiveSSELine(seq, line))
		})
		if !clientGone {
			result := readEmitResult(emitResult)
			if result == nil {
				if g, err := runtimeQuery("get", map[string]any{"task_id": taskID}); err == nil {
					if t, ok := g["task"].(map[string]any); ok {
						result = t["result"]
					}
				}
			}
			writeEvent(map[string]any{
				"schema": "RuntimeEventV1", "schema_version": 1, "type": "run.completed",
				"payload": map[string]any{"result": result, "task_id": taskID},
			})
			_, _ = io.WriteString(w, "data: [DONE]\n\n")
			if flusher != nil {
				flusher.Flush()
			}
		}
	}

	doCancelOnDisconnect := cancelOnDisconnect(r)

	for {
		select {
		case <-ctx.Done():
			// Default: observer gone ≠ cancel. Task keeps running; client may
			// reattach via GET /_runtime/tasks/{id}/events (Last-Event-ID).
			if doCancelOnDisconnect {
				_, _ = runtimeQuery("cancel", map[string]any{"task_id": taskID})
				_ = writeCancelToken(cancelTok)
				writeEvent(map[string]any{
					"schema": "RuntimeEventV1", "schema_version": 1, "type": "run.interrupted",
					"payload": map[string]any{
						"reason":            "client_disconnect",
						"cancel_propagated": cancelTok != "",
						"status":            "stopping",
						"task_id":           taskID,
					},
				})
				deadline := time.Now().Add(3 * time.Second)
				for time.Now().Before(deadline) {
					if workerDone(taskID, emitResult) {
						break
					}
					time.Sleep(50 * time.Millisecond)
				}
			} else {
				writeEvent(map[string]any{
					"schema": "RuntimeEventV1", "schema_version": 1, "type": "run.interrupted",
					"payload": map[string]any{
						"reason":            "client_disconnect",
						"cancel_propagated": false,
						"status":            "observer_closed",
						"task_id":           taskID,
						"resume":            "/_runtime/tasks/" + taskID + "/events",
					},
				})
			}
			finish(true)
			return
		case <-poll.C:
			offset, carry = drainForwardCarry(eventPath, offset, carry, func(line string) {
				seq++
				writeEvent(normalizeLiveSSELine(seq, line))
			})
			if workerDone(taskID, emitResult) {
				finish(false)
				return
			}
		}
	}
}

func (st *state) waitIsolatedJSON(w http.ResponseWriter, r *http.Request, route invokeRoute, taskID, eventPath, cancelTok, emitResult string) {
	_ = eventPath
	poll := time.NewTicker(25 * time.Millisecond)
	defer poll.Stop()
	ctx := r.Context()
	for {
		select {
		case <-ctx.Done():
			_, _ = runtimeQuery("cancel", map[string]any{"task_id": taskID})
			_ = writeCancelToken(cancelTok)
			jsonErr(w, http.StatusRequestTimeout, "client disconnected")
			return
		case <-poll.C:
			if !workerDone(taskID, emitResult) {
				continue
			}
			value := readEmitResult(emitResult)
			if value == nil {
				jsonErr(w, http.StatusInternalServerError, "worker produced no result")
				return
			}
			if m, ok := value.(map[string]any); ok {
				if s, ok := m["sse"].(string); ok && strings.TrimSpace(s) != "" {
					writeSSE(w, m)
					return
				}
			}
			switch route.ReturnMode {
			case "text", "html", "text/html":
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
			case "status":
				w.WriteHeader(http.StatusNoContent)
			default:
				code, body, err := jsonWrapInvokeValue(value)
				if err != nil {
					jsonErr(w, code, err.Error())
					return
				}
				writeJSON(w, code, body)
			}
			return
		}
	}
}

func workerDone(taskID, emitResult string) bool {
	if emitResult != "" {
		if st, err := os.Stat(emitResult); err == nil && st.Size() > 0 {
			return true
		}
	}
	// Sibling of event file written by Supervisor even if emit_result was omitted.
	if taskID != "" {
		if dir := os.Getenv("MARQDO_RUNTIME_DIR"); dir != "" {
			p := filepath.Join(dir, "events", taskID+".result.json")
			if st, err := os.Stat(p); err == nil && st.Size() > 0 {
				return true
			}
		}
	}
	g, err := runtimeQuery("get", map[string]any{"task_id": taskID})
	if err != nil {
		return false
	}
	if _, ok := g["exit_code"]; ok {
		return true
	}
	if t, ok := g["task"].(map[string]any); ok {
		switch fmt.Sprint(t["state"]) {
		case "succeeded", "failed", "cancelled", "interrupted":
			return true
		}
	}
	return false
}

func readEmitResult(path string) any {
	if path == "" {
		return nil
	}
	b, err := os.ReadFile(path)
	if err != nil || len(b) == 0 {
		return nil
	}
	var v any
	if err := json.Unmarshal(b, &v); err != nil {
		return string(b)
	}
	return v
}

func materializeArgsJSON(args map[string]any) (string, error) {
	f, err := os.CreateTemp("", "mq-args-*.json")
	if err != nil {
		return "", err
	}
	path := f.Name()
	enc := json.NewEncoder(f)
	if err := enc.Encode(args); err != nil {
		_ = f.Close()
		_ = os.Remove(path)
		return "", err
	}
	if err := f.Close(); err != nil {
		_ = os.Remove(path)
		return "", err
	}
	return path, nil
}

func resolveSiteEntry(st *state) (module string, fsRoot string, err error) {
	fsRoot = st.entryDir
	if raw, qerr := plugin.Query("entry_dir", "{}"); qerr == nil {
		var s string
		if json.Unmarshal([]byte(raw), &s) == nil && s != "" {
			fsRoot = s
		}
	}
	if raw, qerr := plugin.Query("entry_path", "{}"); qerr == nil {
		var s string
		if json.Unmarshal([]byte(raw), &s) == nil && s != "" {
			module = s
		}
	}
	if module == "" {
		// Fallback: common serve entry names under fs_root.
		for _, name := range []string{"serve.mq.md", "index.mq.md", "main.mq.md"} {
			p := filepath.Join(fsRoot, name)
			if st, e := os.Stat(p); e == nil && !st.IsDir() {
				module = p
				break
			}
		}
	}
	if module == "" {
		return "", "", fmt.Errorf("isolated execute: cannot resolve site entry module")
	}
	if !filepath.IsAbs(module) {
		abs, aerr := filepath.Abs(module)
		if aerr == nil {
			module = abs
		}
	}
	if fsRoot == "" {
		fsRoot = filepath.Dir(module)
	}
	if !filepath.IsAbs(fsRoot) {
		abs, aerr := filepath.Abs(fsRoot)
		if aerr == nil {
			fsRoot = abs
		}
	}
	return module, fsRoot, nil
}

// normalizeLiveSSELine maps Marqdo EventBus frames onto RuntimeEventV1.
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
	if t, _ := obj["type"].(string); t == "message.delta" || t == "turn.started" || t == "model.resolved" || t == "usage.updated" || t == "turn.completed" || t == "turn.failed" {
		for k, v := range obj {
			base[k] = v
		}
		base["seq"] = seq
		return base
	}
	if t, _ := obj["type"].(string); t == "delta" || t == "plan" || t == "reasoning" {
		text, _ := obj["text"].(string)
		base["type"] = "message.delta"
		base["payload"] = map[string]any{"delta": text, "kind": t}
		return base
	}
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
	off, _ := drainForwardCarry(path, offset, "", onLine)
	return off
}

const drainForwardMaxLine = 1024 * 1024

func drainForwardCarry(path string, offset int64, carry string, onLine func(string)) (int64, string) {
	f, err := os.Open(path)
	if err != nil {
		return offset, carry
	}
	defer f.Close()
	if _, err := f.Seek(offset, io.SeekStart); err != nil {
		return offset, carry
	}
	chunk, err := io.ReadAll(io.LimitReader(f, drainForwardMaxLine+1))
	if err != nil && err != io.EOF {
		onLine(fmt.Sprintf(`{"type":"protocol.error","error":%q}`, err.Error()))
		return offset, carry
	}
	if len(chunk) == 0 {
		return offset, carry
	}
	offset += int64(len(chunk))
	buf := carry + string(chunk)
	for {
		i := strings.IndexByte(buf, '\n')
		if i < 0 {
			if len(buf) > drainForwardMaxLine {
				onLine(`{"type":"protocol.error","error":"ndjson line exceeds 1MiB"}`)
				return offset, ""
			}
			return offset, buf
		}
		lineBytes := buf[:i]
		line := strings.TrimSpace(strings.TrimSuffix(lineBytes, "\r"))
		buf = buf[i+1:]
		if line == "" {
			continue
		}
		onLine(line)
	}
}

func legacySSEBridgeSnapshot() (active bool, forward string, cancelTok string) {
	return legacySSEBridgeActive.Load() != 0, os.Getenv("MARQDO_EVENT_FORWARD"), os.Getenv("MARQDO_CANCEL_TOKEN")
}

func writeCancelToken(path string) error {
	if path == "" {
		return nil
	}
	return os.WriteFile(path, []byte("cancelled"), 0o600)
}
