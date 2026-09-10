package apikey_test

import (
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/apikey"
)

func TestHashFormula(t *testing.T) {
	got := apikey.Hash("pep", "secret-key")
	want := apikey.Hash("pep", "secret-key")
	if got != want || got == "" {
		t.Fatalf("hash=%q", got)
	}
	if got == apikey.Hash("pep", "other") {
		t.Fatal("different keys should differ")
	}
}

func TestCheckMatchAndScopes(t *testing.T) {
	pepper := "site-pepper"
	key := "live-key-1"
	digest := apikey.Hash(pepper, key)
	keys := []any{
		map[string]any{
			"hash":   digest,
			"scopes": "read,write",
		},
	}
	out := apikey.Check("", "Bearer "+key, pepper, keys)
	if out["ok"] != true {
		t.Fatalf("match: %v", out)
	}
	scopes, _ := out["scopes"].([]any)
	if len(scopes) != 2 || scopes[0] != "read" || scopes[1] != "write" {
		t.Fatalf("scopes=%v", scopes)
	}
}

func TestCheckMiss(t *testing.T) {
	keys := []any{
		map[string]any{"hash": apikey.Hash("p", "stored"), "scopes": "admin"},
	}
	out := apikey.Check("wrong", "", "p", keys)
	if out["ok"] != false {
		t.Fatalf("expected miss: %v", out)
	}
}

func TestCheckKeyParam(t *testing.T) {
	pepper := "p"
	key := "direct"
	digest := apikey.Hash(pepper, key)
	keys := []any{map[string]any{"hash": digest, "scopes": "api"}}
	out := apikey.Check(key, "", pepper, keys)
	if out["ok"] != true {
		t.Fatalf("%v", out)
	}
}
