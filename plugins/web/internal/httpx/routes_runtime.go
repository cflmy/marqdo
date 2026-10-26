package httpx

import (
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strings"
	"time"

	"github.com/marqdo/marqdo/plugins/web/internal/plugin"
)

// System runtime control routes (M-R3 / M-R4). Cancel and event replay do not
// enter the Marqdo interpreter lock.
func (st *state) mountRuntimeRoutes(mux *http.ServeMux) {
	mux.HandleFunc("POST /_runtime/tasks", st.handleRuntimeAdmit)
	mux.HandleFunc("GET /_runtime/tasks/{id}", st.handleRuntimeGet)
	mux.HandleFunc("POST /_runtime/tasks/{id}/cancel", st.handleRuntimeCancel)
	mux.HandleFunc("GET /_runtime/tasks/{id}/events", st.handleRuntimeEvents)
}

func runtimeQuery(op string, extra map[string]any) (map[string]any, error) {
	payload := map[string]any{"op": op}
	for k, v := range extra {
		payload[k] = v
	}
	raw, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}
	out, err := plugin.Query("runtime", string(raw))
	if err != nil {
		return nil, err
	}
	var m map[string]any
	if err := json.Unmarshal([]byte(out), &m); err != nil {
		return nil, err
	}
	return m, nil
}

func (st *state) handleRuntimeAdmit(w http.ResponseWriter, r *http.Request) {
	body, err := io.ReadAll(http.MaxBytesReader(w, r.Body, 1<<20))
	if err != nil {
		jsonErr(w, http.StatusBadRequest, err.Error())
		return
	}
	var req map[string]any
	if len(body) > 0 {
		if err := json.Unmarshal(body, &req); err != nil {
			jsonErr(w, http.StatusBadRequest, fmt.Sprintf("json: %v", err))
			return
		}
	}
	if req == nil {
		req = map[string]any{}
	}
	taskID, _ := req["task_id"].(string)
	if taskID == "" {
		taskID = fmt.Sprintf("task_%d", time.Now().UnixNano())
		req["task_id"] = taskID
	}
	if _, ok := req["idempotency_key"]; !ok {
		req["idempotency_key"] = taskID
	}
	if _, ok := req["class"]; !ok {
		req["class"] = "execute"
	}
	m, err := runtimeQuery("admit", req)
	if err != nil {
		if strings.Contains(err.Error(), "conflict") || strings.Contains(fmt.Sprint(m["error"]), "conflict") {
			jsonErr(w, http.StatusConflict, err.Error())
			return
		}
		jsonErr(w, http.StatusInternalServerError, err.Error())
		return
	}
	if ok, _ := m["ok"].(bool); !ok {
		jsonErr(w, http.StatusConflict, fmt.Sprint(m["error"]))
		return
	}
	w.Header().Set("Location", "/_runtime/tasks/"+taskID)
	writeJSON(w, http.StatusAccepted, m)
}

func (st *state) handleRuntimeGet(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	m, err := runtimeQuery("get", map[string]any{"task_id": id})
	if err != nil {
		jsonErr(w, http.StatusInternalServerError, err.Error())
		return
	}
	if st, _ := m["status"].(float64); st == 404 || fmt.Sprint(m["error"]) == "not_found" {
		jsonErr(w, http.StatusNotFound, "task not found")
		return
	}
	writeJSON(w, http.StatusOK, m)
}

func (st *state) handleRuntimeCancel(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	m, err := runtimeQuery("cancel", map[string]any{"task_id": id})
	if err != nil {
		jsonErr(w, http.StatusInternalServerError, err.Error())
		return
	}
	if st, _ := m["status"].(float64); st == 404 || fmt.Sprint(m["error"]) == "not_found" {
		jsonErr(w, http.StatusNotFound, "task not found")
		return
	}
	writeJSON(w, http.StatusOK, m)
}

func (st *state) handleRuntimeEvents(w http.ResponseWriter, r *http.Request) {
	id := r.PathValue("id")
	after := int64(0)
	if s := r.URL.Query().Get("after_seq"); s != "" {
		fmt.Sscanf(s, "%d", &after)
	}
	if last := strings.TrimSpace(r.Header.Get("Last-Event-ID")); last != "" {
		// Accept "task:seq" or bare seq.
		if i := strings.LastIndexByte(last, ':'); i >= 0 {
			fmt.Sscanf(last[i+1:], "%d", &after)
		} else {
			fmt.Sscanf(last, "%d", &after)
		}
	}
	wantSSE := strings.Contains(r.Header.Get("Accept"), "text/event-stream") ||
		r.URL.Query().Get("stream") == "1"

	if !wantSSE {
		m, err := runtimeQuery("events", map[string]any{"task_id": id, "after_seq": after})
		if err != nil {
			jsonErr(w, http.StatusInternalServerError, err.Error())
			return
		}
		if st, _ := m["status"].(float64); st == 404 || fmt.Sprint(m["error"]) == "not_found" {
			jsonErr(w, http.StatusNotFound, "task not found")
			return
		}
		writeJSON(w, http.StatusOK, m)
		return
	}

	w.Header().Set("Content-Type", "text/event-stream; charset=utf-8")
	w.Header().Set("Cache-Control", "no-cache")
	w.Header().Set("Connection", "keep-alive")
	w.WriteHeader(http.StatusOK)
	flusher, _ := w.(http.Flusher)
	cursor := after
	ticker := time.NewTicker(25 * time.Millisecond)
	defer ticker.Stop()
	heartbeat := time.NewTicker(15 * time.Second)
	defer heartbeat.Stop()
	ctx := r.Context()
	for {
		select {
		case <-ctx.Done():
			return
		case <-heartbeat.C:
			_, _ = io.WriteString(w, ": ping\n\n")
			if flusher != nil {
				flusher.Flush()
			}
		case <-ticker.C:
			m, err := runtimeQuery("events", map[string]any{"task_id": id, "after_seq": cursor})
			if err != nil {
				continue
			}
			if st, _ := m["status"].(float64); st == 404 {
				_, _ = io.WriteString(w, "event: error\ndata: {\"error\":\"not_found\"}\n\n")
				if flusher != nil {
					flusher.Flush()
				}
				return
			}
			evs, _ := m["events"].([]any)
			for _, raw := range evs {
				b, err := json.Marshal(raw)
				if err != nil {
					continue
				}
				var obj map[string]any
				_ = json.Unmarshal(b, &obj)
				eid := ""
				if s, ok := obj["event_id"].(string); ok {
					eid = s
				}
				if seq, ok := obj["seq"].(float64); ok {
					cursor = int64(seq)
				}
				etype, _ := obj["type"].(string)
				if etype == "" {
					etype = "message"
				}
				if eid != "" {
					_, _ = io.WriteString(w, "id: "+eid+"\n")
				}
				_, _ = io.WriteString(w, "event: "+etype+"\ndata: "+string(b)+"\n\n")
				if flusher != nil {
					flusher.Flush()
				}
			}
		}
	}
}
