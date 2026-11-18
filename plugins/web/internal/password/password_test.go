package password_test

import (
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/password"
)

func TestHashAndVerifyRoundtrip(t *testing.T) {
	hash, err := password.Hash("secret")
	if err != nil {
		t.Fatal(err)
	}
	if !password.IsHashed(hash) {
		t.Fatalf("not PHC: %q", hash)
	}
	if !strings.HasPrefix(hash, "$argon2id$") {
		t.Fatalf("want argon2id prefix, got %q", hash)
	}
	if !password.Verify("secret", hash) {
		t.Fatal("verify secret failed")
	}
	if password.Verify("wrong", hash) {
		t.Fatal("wrong password should fail")
	}
	out, err := password.HashResult("secret")
	if err != nil {
		t.Fatal(err)
	}
	if out["hash"] == nil || out["hash"] == "" {
		t.Fatalf("HashResult shape: %v", out)
	}
}

func TestLegacyPlaintext(t *testing.T) {
	if !password.Verify("secret", "secret") {
		t.Fatal("plaintext equality")
	}
	if password.Verify("other", "secret") {
		t.Fatal("plaintext mismatch")
	}
	if password.Verify("", "secret") || password.Verify("secret", "") {
		t.Fatal("empty should fail")
	}
}

func TestEmptyHashError(t *testing.T) {
	if _, err := password.Hash(""); err == nil {
		t.Fatal("expected error")
	}
}
