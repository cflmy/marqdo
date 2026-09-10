package ws

import (
	"testing"
	"time"
)

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

func readOne(ch chan string) string {
	select {
	case s := <-ch:
		return s
	case <-time.After(time.Second):
		return ""
	}
}
