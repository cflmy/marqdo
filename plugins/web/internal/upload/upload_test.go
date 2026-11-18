package upload

import (
	"os"
	"path/filepath"
	"testing"
)

func TestValidateAllowReject(t *testing.T) {
	types := []any{
		map[string]any{"类型": "image/png", "扩展名": "png"},
		map[string]any{"类型": "text/plain", "扩展名": "txt"},
	}
	ok := Validate("a.png", "image/png", 100, 1000, types)
	if ok["ok"] != true {
		t.Fatalf("expected ok: %#v", ok)
	}
	bad := Validate("a.exe", "application/octet-stream", 100, 1000, types)
	if bad["ok"] != false {
		t.Fatalf("expected reject: %#v", bad)
	}
	big := Validate("a.png", "image/png", 9000, 1000, types)
	if big["ok"] != false {
		t.Fatalf("expected size reject: %#v", big)
	}
}

func TestSaveRoundtrip(t *testing.T) {
	dir := t.TempDir()
	src := filepath.Join(dir, "sample.txt")
	if err := os.WriteFile(src, []byte("hello-upload"), 0o644); err != nil {
		t.Fatal(err)
	}
	url := "file:" + filepath.Join(dir, "blobs")
	prefix := "smoke/"
	ct := "text/plain"
	saved, err := Save(url, nil, src, &ct, &prefix)
	if err != nil {
		t.Fatal(err)
	}
	if saved["ok"] != true {
		t.Fatalf("save: %#v", saved)
	}
	key, _ := saved["key"].(string)
	if key == "" {
		t.Fatal("missing key")
	}
}
