package httpx

import (
	"encoding/json"
	"fmt"
	"net/http"
	"strings"

	"github.com/gorilla/websocket"
	"github.com/marqdo/marqdo/plugins/web/internal/plugin"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
	"github.com/marqdo/marqdo/plugins/web/internal/ws"
)

var wsUpgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true },
}

func (st *state) mountWSRoutes(mux *http.ServeMux) {
	for _, path := range sortedKeys(st.wsRoutes) {
		spec := ws.ParseRouteSpec(st.wsRoutes[path]) // mode + optional room_key / on_message
		routePath := path
		pattern := routePath
		if strings.Contains(routePath, "{") {
			pattern = goMuxPattern(routePath)
		}
		mux.HandleFunc("GET "+pattern, func(w http.ResponseWriter, r *http.Request) {
			st.handleWS(w, r, routePath, spec)
		})
	}
}

func pathValuesOf(r *http.Request, routePath string) map[string]string {
	out := map[string]string{}
	// Collect {name} tokens from the route template.
	rest := routePath
	for {
		i := strings.Index(rest, "{")
		if i < 0 {
			break
		}
		rest = rest[i+1:]
		j := strings.Index(rest, "}")
		if j < 0 {
			break
		}
		name := rest[:j]
		name = strings.TrimPrefix(name, "*")
		if name != "" {
			if v := r.PathValue(name); v != "" {
				out[name] = v
			}
		}
		rest = rest[j+1:]
	}
	return out
}

func (st *state) handleWS(w http.ResponseWriter, r *http.Request, path string, spec ws.RouteSpec) {
	conn, err := wsUpgrader.Upgrade(w, r, nil)
	if err != nil {
		return
	}
	defer conn.Close()

	vals := pathValuesOf(r, path)
	switch spec.Mode {
	case ws.ModeEcho:
		for {
			mt, data, err := conn.ReadMessage()
			if err != nil {
				return
			}
			if mt == websocket.TextMessage {
				if err := conn.WriteMessage(websocket.TextMessage, data); err != nil {
					return
				}
			} else if mt == websocket.CloseMessage {
				return
			}
		}
	case ws.ModeDrain:
		for {
			mt, _, err := conn.ReadMessage()
			if err != nil || mt == websocket.CloseMessage {
				return
			}
		}
	case ws.ModeBroadcast, ws.ModeRoom:
		room := path
		if spec.RoomKey != "" {
			room = ws.ExpandTemplate(spec.RoomKey, vals)
		} else if len(vals) > 0 {
			room = ws.ExpandTemplate(path, vals)
		}
		user := session.UsernameFromCookie(r.Header.Get("Cookie"))
		if user == "" {
			user = "anon"
		}
		ch := ws.JoinRoom(room)
		defer ws.LeaveRoom(room, ch)

		presenceRoom := room + ".presence"
		var presenceCh chan string
		if spec.Presence {
			presenceCh = ws.JoinRoom(presenceRoom)
			defer ws.LeaveRoom(presenceRoom, presenceCh)
			ws.PublishRoom(presenceRoom, presenceJSON("join", user, room))
			defer ws.PublishRoom(presenceRoom, presenceJSON("leave", user, room))
		}

		done := make(chan struct{})
		go func() {
			defer close(done)
			for {
				mt, data, err := conn.ReadMessage()
				if err != nil {
					return
				}
				if mt == websocket.TextMessage {
					text := string(data)
					if spec.OnMessage != "" {
						out, err := plugin.CallLibPath(spec.OnMessage, map[string]any{
							"message": text,
							"room":    room,
							"path":    path,
							"user":    user,
						})
						if err != nil {
							_ = conn.WriteMessage(websocket.TextMessage, []byte(`{"error":`+jsonQuote(err.Error())+`}`))
							continue
						}
						skip, rewritten := interpretOnMessage(out, text)
						if skip {
							continue
						}
						text = rewritten
					}
					ws.PublishRoom(room, text)
				} else if mt == websocket.CloseMessage {
					return
				}
			}
		}()

		for {
			select {
			case <-done:
				return
			case text, ok := <-ch:
				if !ok {
					return
				}
				if err := conn.WriteMessage(websocket.TextMessage, []byte(text)); err != nil {
					return
				}
			case text, ok := <-presenceOrNil(presenceCh):
				if presenceCh == nil {
					continue
				}
				if !ok {
					return
				}
				if err := conn.WriteMessage(websocket.TextMessage, []byte(text)); err != nil {
					return
				}
			}
		}
	default:
		return
	}
}

func presenceOrNil(ch chan string) <-chan string {
	if ch == nil {
		return nil
	}
	return ch
}

func presenceJSON(event, user, room string) string {
	b, _ := json.Marshal(map[string]any{
		"type":  "presence",
		"event": event,
		"user":  user,
		"room":  room,
	})
	return string(b)
}

func jsonQuote(s string) string {
	b, _ := json.Marshal(s)
	return string(b)
}

func interpretOnMessage(out any, original string) (skip bool, text string) {
	text = original
	m, ok := out.(map[string]any)
	if !ok {
		if s, ok := out.(string); ok && s != "" {
			return false, s
		}
		return false, original
	}
	if v, ok := m["skip"]; ok && boolish(v) {
		return true, ""
	}
	if v, ok := m["drop"]; ok && boolish(v) {
		return true, ""
	}
	if v, ok := m["message"]; ok {
		text = fmt.Sprint(v)
	} else if v, ok := m["消息"]; ok {
		text = fmt.Sprint(v)
	} else if v, ok := m["text"]; ok {
		text = fmt.Sprint(v)
	}
	return false, text
}
