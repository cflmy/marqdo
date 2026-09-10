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
	room.mu.Unlock()
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
