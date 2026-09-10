// Package ratelimit ports login attempt throttling from the Rust web plugin.
package ratelimit

import (
	"fmt"
	"sync"
	"time"
)

const (
	maxFails   = 5
	windowSec  = 900 // 15 minutes
	lockMsg    = "Too many failed login attempts. Try again in about 15 minutes."
)

type window struct {
	count uint32
	start int64
}

var (
	mu       sync.Mutex
	attempts map[string]*window
)

func nowSecs() int64 {
	return time.Now().Unix()
}

func key(ip, username string) string {
	return ip + "|" + username
}

// Reset clears attempt counters (between listens / tests).
func Reset() {
	mu.Lock()
	defer mu.Unlock()
	attempts = map[string]*window{}
}

func ensureMap() {
	if attempts == nil {
		attempts = map[string]*window{}
	}
}

// Check returns an error when the client is temporarily locked out.
func Check(ip, username string) error {
	k := key(ip, username)
	now := nowSecs()
	mu.Lock()
	defer mu.Unlock()
	ensureMap()
	w, ok := attempts[k]
	if !ok {
		return nil
	}
	if now-w.start < windowSec && w.count >= maxFails {
		return fmt.Errorf("%s", lockMsg)
	}
	if now-w.start >= windowSec {
		delete(attempts, k)
	}
	return nil
}

// RecordFailure increments the failure counter for IP+username.
func RecordFailure(ip, username string) {
	k := key(ip, username)
	now := nowSecs()
	mu.Lock()
	defer mu.Unlock()
	ensureMap()
	w, ok := attempts[k]
	if !ok {
		attempts[k] = &window{count: 1, start: now}
		return
	}
	if now-w.start >= windowSec {
		w.count = 0
		w.start = now
	}
	if w.count < ^uint32(0) {
		w.count++
	}
}

// ClearSuccess removes the counter after a successful login.
func ClearSuccess(ip, username string) {
	k := key(ip, username)
	mu.Lock()
	defer mu.Unlock()
	ensureMap()
	delete(attempts, k)
}
