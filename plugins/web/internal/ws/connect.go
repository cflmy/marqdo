package ws

import (
	"fmt"
	"net/http"
	"strings"
	"time"

	"github.com/gorilla/websocket"
)

// Connect dials url, sends one text message, collects text replies until close or timeout.
// Mirrors Rust ws::connect; errors are returned as {ok:false, error}.
func Connect(url, message string, headers map[string]any, timeoutSec uint64) map[string]any {
	if !(strings.HasPrefix(url, "ws://") || strings.HasPrefix(url, "wss://")) {
		return map[string]any{"ok": false, "error": "ws url must start with ws:// or wss://"}
	}
	if timeoutSec < 1 {
		timeoutSec = 1
	}
	timeout := time.Duration(timeoutSec) * time.Second
	hdr := http.Header{}
	for k, v := range headersFromValue(headers) {
		hdr.Set(k, v)
	}
	dialer := websocket.Dialer{HandshakeTimeout: timeout}
	conn, _, err := dialer.Dial(url, hdr)
	if err != nil {
		return map[string]any{"ok": false, "error": fmt.Sprintf("ws connect: %v", err)}
	}
	defer conn.Close()

	if err := conn.SetWriteDeadline(time.Now().Add(timeout)); err != nil {
		return map[string]any{"ok": false, "error": fmt.Sprintf("ws send: %v", err)}
	}
	if err := conn.WriteMessage(websocket.TextMessage, []byte(message)); err != nil {
		return map[string]any{"ok": false, "error": fmt.Sprintf("ws send: %v", err)}
	}

	var messages []string
	deadline := time.Now().Add(timeout)
	for {
		if err := conn.SetReadDeadline(deadline); err != nil {
			break
		}
		mt, data, err := conn.ReadMessage()
		if err != nil {
			break
		}
		if mt == websocket.TextMessage {
			messages = append(messages, string(data))
		}
		if mt == websocket.CloseMessage {
			break
		}
	}
	return map[string]any{"ok": true, "messages": toAnyList(messages)}
}

func headersFromValue(headers map[string]any) map[string]string {
	out := map[string]string{}
	if headers == nil {
		return out
	}
	for k, v := range headers {
		switch t := v.(type) {
		case string:
			out[k] = t
		case nil:
			out[k] = ""
		default:
			out[k] = fmt.Sprint(t)
		}
	}
	return out
}

func toAnyList(ss []string) []any {
	out := make([]any, len(ss))
	for i, s := range ss {
		out[i] = s
	}
	return out
}
