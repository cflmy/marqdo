package httpx

import (
	"os"
	"path/filepath"
	"testing"
)

func TestDrainForwardReadsIncrementalLines(t *testing.T) {
	dir := t.TempDir()
	path := filepath.Join(dir, "evt.ndjson")
	if err := os.WriteFile(path, []byte("{\"type\":\"a\"}\n{\"type\":\"b\"}\n"), 0o600); err != nil {
		t.Fatal(err)
	}
	var lines []string
	off := drainForward(path, 0, func(line string) {
		lines = append(lines, line)
	})
	if len(lines) != 2 {
		t.Fatalf("got %d lines %#v", len(lines), lines)
	}
	if off <= 0 {
		t.Fatalf("offset=%d", off)
	}
	// Append after first drain — second call with offset must only see new line.
	f, err := os.OpenFile(path, os.O_APPEND|os.O_WRONLY, 0o600)
	if err != nil {
		t.Fatal(err)
	}
	if _, err := f.WriteString("{\"type\":\"c\"}\n"); err != nil {
		t.Fatal(err)
	}
	_ = f.Close()
	var more []string
	off2 := drainForward(path, off, func(line string) {
		more = append(more, line)
	})
	if len(more) != 1 || more[0] != "{\"type\":\"c\"}" {
		t.Fatalf("incremental got %#v off=%d→%d", more, off, off2)
	}
}
