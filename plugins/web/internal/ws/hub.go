package ws

import (
	"sync"
)

const hubCap = 256

type hub struct {
	mu   sync.Mutex
	room map[string]*broadcastRoom
}

type broadcastRoom struct {
	mu   sync.Mutex
	subs map[chan string]struct{}
}

var globalHub = &hub{room: map[string]*broadcastRoom{}}

func subscribe(path string) chan string {
	ch := make(chan string, hubCap)
	globalHub.mu.Lock()
	room := globalHub.room[path]
	if room == nil {
		room = &broadcastRoom{subs: map[chan string]struct{}{}}
		globalHub.room[path] = room
	}
	room.mu.Lock()
	room.subs[ch] = struct{}{}
	room.mu.Unlock()
	globalHub.mu.Unlock()
	return ch
}

func unsubscribe(path string, ch chan string) {
	globalHub.mu.Lock()
	room := globalHub.room[path]
	globalHub.mu.Unlock()
	if room == nil {
		return
	}
	room.mu.Lock()
	delete(room.subs, ch)
	empty := len(room.subs) == 0
	room.mu.Unlock()
	if !empty {
		return
	}
	// A room can be derived from a URL path. Remove empty rooms so a stream of
	// one-off room names cannot leave metadata allocated for the process lifetime.
	globalHub.mu.Lock()
	if globalHub.room[path] == room {
		room.mu.Lock()
		if len(room.subs) == 0 {
			delete(globalHub.room, path)
		}
		room.mu.Unlock()
	}
	globalHub.mu.Unlock()
}

// JoinRoom subscribes to a named room (G-WS1). Same backing store as path broadcast.
func JoinRoom(room string) chan string {
	return subscribe(room)
}

// LeaveRoom unsubscribes from a named room (G-WS1).
func LeaveRoom(room string, ch chan string) {
	unsubscribe(room, ch)
}

// PublishRoom fans out text to all subscribers in a named room (G-WS1).
func PublishRoom(room string, text string) {
	Publish(room, text)
}

// Publish fans out text to all subscribers on path (Rust ws_hub::publish).
func Publish(path string, text string) {
	globalHub.mu.Lock()
	room := globalHub.room[path]
	globalHub.mu.Unlock()
	if room == nil {
		return
	}
	room.mu.Lock()
	for ch := range room.subs {
		select {
		case ch <- text:
		default:
		}
	}
	room.mu.Unlock()
}
