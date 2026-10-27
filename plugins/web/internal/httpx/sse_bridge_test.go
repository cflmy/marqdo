package httpx

import (
	"os"
	"path/filepath"
	"sync"
	"testing"
	"time"
)

// M-R0: second stream must fail before rewriting A's forward/cancel env.
func TestLegacySSEBridgeSingleFlight(t *testing.T) {
	if !tryAcquireLegacySSEBridge() {
		t.Fatal("expected first acquire to succeed")
	}
	fwdA := filepath.Join(t.TempDir(), "a.ndjson")
	tokA := filepath.Join(t.TempDir(), "a.tok")
	if err := os.WriteFile(fwdA, nil, 0o600); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(tokA, nil, 0o600); err != nil {
		t.Fatal(err)
	}
	t.Setenv("MARQDO_EVENT_FORWARD", fwdA)
	t.Setenv("MARQDO_CANCEL_TOKEN", tokA)
	legacySSEBridgeActive.Store(1)

	if tryAcquireLegacySSEBridge() {
		releaseLegacySSEBridge()
		t.Fatal("second acquire must fail while bridge held")
	}
	active, fwd, tok := legacySSEBridgeSnapshot()
	if !active {
		t.Fatal("bridge should remain active for holder A")
	}
	if fwd != fwdA || tok != tokA {
		t.Fatalf("rejected second stream rewrote env: fwd=%q tok=%q", fwd, tok)
	}

	releaseLegacySSEBridge()
	if tryAcquireLegacySSEBridge() {
		releaseLegacySSEBridge()
	} else {
		t.Fatal("bridge should be free after release")
	}
}

// M-R0: cancel-token file must not be deleted while bridge holder waits.
// (sse-live observer disconnect no longer cancels by default — see cancelOnDisconnect.)
func TestCancelTokenSurvivesUntilHolderExits(t *testing.T) {
	dir := t.TempDir()
	tok := filepath.Join(dir, "cancel.tok")
	if err := os.WriteFile(tok, nil, 0o600); err != nil {
		t.Fatal(err)
	}
	if err := writeCancelToken(tok); err != nil {
		t.Fatal(err)
	}
	body, err := os.ReadFile(tok)
	if err != nil {
		t.Fatalf("token must still exist after cancel mark: %v", err)
	}
	if string(body) != "cancelled" {
		t.Fatalf("body=%q", body)
	}
}

func TestDrainForwardCarryHalfFrames(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "evt.ndjson")
	if err := os.WriteFile(path, []byte(`{"t":"部`), 0o600); err != nil {
		t.Fatal(err)
	}
	var lines []string
	off, carry := drainForwardCarry(path, 0, "", func(line string) {
		lines = append(lines, line)
	})
	if len(lines) != 0 {
		t.Fatalf("incomplete UTF-8 frame must stay in carry, got %#v", lines)
	}
	f, err := os.OpenFile(path, os.O_APPEND|os.O_WRONLY, 0o600)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := f.WriteString("分\"}\n{\"t\":\"ok\"}\n"); err != nil {
		t.Fatal(err)
	}
	_ = f.Close()
	off, carry = drainForwardCarry(path, off, carry, func(line string) {
		lines = append(lines, line)
	})
	if len(lines) != 2 {
		t.Fatalf("got %d lines %#v carry=%q off=%d", len(lines), lines, carry, off)
	}
	if carry != "" {
		t.Fatalf("carry should be empty, got %q", carry)
	}
}

func TestDrainForwardCarryCRLFAndSplit(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "evt.ndjson")
	part1 := []byte("{\"a\":1}\r\n{\"b\":")
	if err := os.WriteFile(path, part1, 0o600); err != nil {
		t.Fatal(err)
	}
	var lines []string
	off, carry := drainForwardCarry(path, 0, "", func(line string) {
		lines = append(lines, line)
	})
	if len(lines) != 1 || lines[0] != `{"a":1}` {
		t.Fatalf("first frame %#v", lines)
	}
	f, err := os.OpenFile(path, os.O_APPEND|os.O_WRONLY, 0o600)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := f.WriteString("2}\n"); err != nil {
		t.Fatal(err)
	}
	_ = f.Close()
	_, carry = drainForwardCarry(path, off, carry, func(line string) {
		lines = append(lines, line)
	})
	if len(lines) != 2 || lines[1] != `{"b":2}` {
		t.Fatalf("split frame %#v carry=%q", lines, carry)
	}
}

func TestLegacySSEBridgeConcurrentReject(t *testing.T) {
	var wg sync.WaitGroup
	acquired := 0
	var mu sync.Mutex
	for i := 0; i < 8; i++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			if tryAcquireLegacySSEBridge() {
				mu.Lock()
				acquired++
				mu.Unlock()
				time.Sleep(20 * time.Millisecond)
				releaseLegacySSEBridge()
			}
		}()
	}
	wg.Wait()
	if acquired < 1 {
		t.Fatal("expected at least one acquirer")
	}
}
