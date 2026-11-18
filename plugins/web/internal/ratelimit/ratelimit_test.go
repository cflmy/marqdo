package ratelimit_test

import (
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/ratelimit"
)

func TestLocksAfterMaxFails(t *testing.T) {
	ratelimit.Reset()
	ip, user := "127.0.0.1", "admin"
	for i := 0; i < 5; i++ {
		ratelimit.RecordFailure(ip, user)
	}
	if err := ratelimit.Check(ip, user); err == nil {
		t.Fatal("expected lockout")
	}
	ratelimit.ClearSuccess(ip, user)
	if err := ratelimit.Check(ip, user); err != nil {
		t.Fatalf("cleared: %v", err)
	}
}
