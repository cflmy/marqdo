package httpx

import (
	"net/http"

	"github.com/gorilla/websocket"
	"github.com/marqdo/marqdo/plugins/web/internal/ws"
)

var wsUpgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool { return true },
}

func (st *state) mountWSRoutes(mux *http.ServeMux) {
	for _, path := range sortedKeys(st.wsRoutes) {
		spec := ws.ParseRouteSpec(st.wsRoutes[path]) // mode + optional room_key
		routePath := path
		mux.HandleFunc("GET "+routePath, func(w http.ResponseWriter, r *http.Request) {
			st.handleWS(w, r, routePath, spec)
		})
	}
}

func (st *state) handleWS(w http.ResponseWriter, r *http.Request, path string, spec ws.RouteSpec) {
	conn, err := wsUpgrader.Upgrade(w, r, nil)
	if err != nil {
		return
	}
	defer conn.Close()

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
			room = spec.RoomKey
		}
		ch := ws.JoinRoom(room)
		defer ws.LeaveRoom(room, ch)

		done := make(chan struct{})
		go func() {
			defer close(done)
			for {
				mt, data, err := conn.ReadMessage()
				if err != nil {
					return
				}
				if mt == websocket.TextMessage {
					ws.PublishRoom(room, string(data))
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
			}
		}
	default:
		return
	}
}
