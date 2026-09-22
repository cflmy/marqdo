package ws

import (
	"testing"
	"time"
)

func TestNamedRoomJoinPublishLeave(t *testing.T) {
	room := "chat.room.42"
	ch := JoinRoom(room)
	ch2 := JoinRoom(room)
	PublishRoom(room, "hello-room")
	if got := readOne(ch); got != "hello-room" {
		t.Fatalf("ch1=%q", got)
	}
	if got := readOne(ch2); got != "hello-room" {
		t.Fatalf("ch2=%q", got)
	}
	LeaveRoom(room, ch)
	LeaveRoom(room, ch2)
}

func TestBroadcastFanout(t *testing.T) {
	path := "/t-fanout-go"
	ch := subscribe(path)
	ch2 := subscribe(path)
	Publish(path, "hi")
	if got := readOne(ch); got != "hi" {
		t.Fatalf("ch1=%q", got)
	}
	if got := readOne(ch2); got != "hi" {
		t.Fatalf("ch2=%q", got)
	}
	unsubscribe(path, ch)
	unsubscribe(path, ch2)
}

func TestUnsubscribeRemovesEmptyRoom(t *testing.T) {
	path := "/t-empty-room-gc"
	ch := subscribe(path)
	unsubscribe(path, ch)
	globalHub.mu.Lock()
	_, ok := globalHub.room[path]
	globalHub.mu.Unlock()
	if ok {
		t.Fatalf("empty room %q was not garbage-collected", path)
	}
}

func TestResubscribeDuringCleanupKeepsRoom(t *testing.T) {
	path := "/t-room-rejoin"
	ch := subscribe(path)
	ch2 := subscribe(path)
	unsubscribe(path, ch)
	unsubscribe(path, ch2)
	ch3 := subscribe(path)
	Publish(path, "still-alive")
	if got := readOne(ch3); got != "still-alive" {
		t.Fatalf("re-subscribed chan got %q", got)
	}
	unsubscribe(path, ch3)
	globalHub.mu.Lock()
	_, ok := globalHub.room[path]
	globalHub.mu.Unlock()
	if ok {
		t.Fatalf("empty room %q was not garbage-collected after rejoin", path)
	}
}

func readOne(ch chan string) string {
	select {
	case s := <-ch:
		return s
	case <-time.After(time.Second):
		return ""
	}
}
