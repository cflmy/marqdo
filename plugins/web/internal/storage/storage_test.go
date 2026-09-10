package storage

import (
	"os"
	"path/filepath"
	"testing"
)

func TestFilePutGetListDelete(t *testing.T) {
	dir := t.TempDir()
	url := "file:" + dir
	if _, err := Open(url); err != nil {
		t.Fatal(err)
	}
	body := "hi"
	if _, err := Put(url, "a/hello.txt", &body, nil, strPtr("text/plain")); err != nil {
		t.Fatal(err)
	}
	got, err := Get(url, "a/hello.txt")
	if err != nil {
		t.Fatal(err)
	}
	if got["ok"] != true || got["body"] != "hi" {
		t.Fatalf("get: %#v", got)
	}
	ls, err := List(url, "a/")
	if err != nil {
		t.Fatal(err)
	}
	if ls["count"].(float64) != 1 {
		t.Fatalf("list: %#v", ls)
	}
	if _, err := Delete(url, "a/hello.txt"); err != nil {
		t.Fatal(err)
	}
	got2, _ := Get(url, "a/hello.txt")
	if got2["ok"] != false {
		t.Fatalf("expected missing after delete: %#v", got2)
	}
}

func TestPutFromPath(t *testing.T) {
	dir := t.TempDir()
	src := filepath.Join(dir, "src.txt")
	if err := os.WriteFile(src, []byte("hello-upload"), 0o644); err != nil {
		t.Fatal(err)
	}
	url := "file:" + filepath.Join(dir, "blobs")
	p := src
	if _, err := Put(url, "x.txt", nil, &p, strPtr("text/plain")); err != nil {
		t.Fatal(err)
	}
	got, err := Get(url, "x.txt")
	if err != nil {
		t.Fatal(err)
	}
	if got["body"] != "hello-upload" {
		t.Fatalf("body=%v", got["body"])
	}
}

func strPtr(s string) *string { return &s }
