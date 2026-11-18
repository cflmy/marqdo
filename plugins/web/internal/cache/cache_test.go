package cache

import (
	"testing"
	"time"
)

func TestMemoryRoundtrip(t *testing.T) {
	Reset()
	url := "memory:test-cache"
	if _, err := Open(url); err != nil {
		t.Fatal(err)
	}
	ttl := uint64(60)
	if _, err := Set(url, "a", "1", &ttl); err != nil {
		t.Fatal(err)
	}
	g, err := Get(url, "a")
	if err != nil {
		t.Fatal(err)
	}
	if g["ok"] != true {
		t.Fatalf("get ok: %v", g["ok"])
	}
	if g["value"] != "1" {
		t.Fatalf("value: %v", g["value"])
	}
	ex, err := Exists(url, "a")
	if err != nil {
		t.Fatal(err)
	}
	if ex["ok"] != true {
		t.Fatalf("exists: %v", ex["ok"])
	}
	if _, err := Del(url, "a"); err != nil {
		t.Fatal(err)
	}
	g, err = Get(url, "a")
	if err != nil {
		t.Fatal(err)
	}
	if g["ok"] != false {
		t.Fatalf("get after del: %v", g["ok"])
	}
	ex, err = Exists(url, "a")
	if err != nil {
		t.Fatal(err)
	}
	if ex["ok"] != false {
		t.Fatalf("exists after del: %v", ex["ok"])
	}
}

func TestMemoryTTLExpiry(t *testing.T) {
	Reset()
	url := "memory:ttl-test"
	if _, err := Open(url); err != nil {
		t.Fatal(err)
	}
	ttl := uint64(1)
	if _, err := Set(url, "k", "v", &ttl); err != nil {
		t.Fatal(err)
	}
	tt, err := TTL(url, "k")
	if err != nil {
		t.Fatal(err)
	}
	if tt["ok"] != true {
		t.Fatalf("ttl ok: %v", tt["ok"])
	}
	ttlVal, ok := tt["ttl"].(int64)
	if !ok {
		// JSON roundtrip may use float64 in some paths; accept both.
		if f, ok := tt["ttl"].(float64); ok {
			ttlVal = int64(f)
		} else {
			t.Fatalf("ttl type: %T", tt["ttl"])
		}
	}
	if ttlVal < 0 || ttlVal > 1 {
		t.Fatalf("ttl seconds: %d", ttlVal)
	}
	time.Sleep(1100 * time.Millisecond)
	g, err := Get(url, "k")
	if err != nil {
		t.Fatal(err)
	}
	if g["ok"] != false {
		t.Fatalf("expected expired miss, got %v", g)
	}
}

func TestMemoryDelReturnsOk(t *testing.T) {
	Reset()
	url := "memory:del-test"
	if _, err := Open(url); err != nil {
		t.Fatal(err)
	}
	d, err := Del(url, "missing")
	if err != nil {
		t.Fatal(err)
	}
	if d["ok"] != false {
		t.Fatalf("del missing: %v", d["ok"])
	}
	if _, err := Set(url, "x", "y", nil); err != nil {
		t.Fatal(err)
	}
	d, err = Del(url, "x")
	if err != nil {
		t.Fatal(err)
	}
	if d["ok"] != true {
		t.Fatalf("del existing: %v", d["ok"])
	}
}

func TestRedisOpenOnly(t *testing.T) {
	Reset()
	out, err := Open("redis://127.0.0.1:6379/0")
	if err != nil {
		t.Fatal(err)
	}
	if out["_type"] != "cache" {
		t.Fatalf("type: %v", out["_type"])
	}
}
