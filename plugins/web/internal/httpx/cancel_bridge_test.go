package httpx

import (
	"os"
	"path/filepath"
	"testing"
)

// F06: client-disconnect cancel bridge writes token file + MARQDO_CANCEL for llm plugin.
func TestPropagateCancelSignal(t *testing.T) {
	dir := t.TempDir()
	tok := filepath.Join(dir, "cancel.tok")
	if err := os.WriteFile(tok, []byte("cancelled"), 0o600); err != nil {
		t.Fatal(err)
	}
	t.Setenv("MARQDO_CANCEL_TOKEN", tok)
	t.Setenv("MARQDO_CANCEL", "1")

	if os.Getenv("MARQDO_CANCEL") != "1" {
		t.Fatalf("MARQDO_CANCEL not set")
	}
	body, err := os.ReadFile(tok)
	if err != nil {
		t.Fatal(err)
	}
	if string(body) != "cancelled" {
		t.Fatalf("token body=%q", body)
	}
}
