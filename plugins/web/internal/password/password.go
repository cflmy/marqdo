// Package password ports argon2id hashing from the Rust web plugin.
package password

import (
	"crypto/rand"
	"crypto/subtle"
	"encoding/base64"
	"fmt"
	"strconv"
	"strings"

	"golang.org/x/crypto/argon2"
)

// Defaults match Rust argon2::Argon2::default() (argon2id, m=19456, t=2, p=1).
const (
	timeCost    = 2
	memoryKiB   = 19456
	parallelism = 1
	keyLen      = 32
	saltLen     = 16
)

// IsHashed reports whether stored looks like an argon2 PHC string.
func IsHashed(stored string) bool {
	return strings.HasPrefix(stored, "$argon2")
}

// Hash returns an argon2id PHC string for plaintext password.
func Hash(password string) (string, error) {
	if password == "" {
		return "", fmt.Errorf("password must not be empty")
	}
	salt := make([]byte, saltLen)
	if _, err := rand.Read(salt); err != nil {
		return "", fmt.Errorf("getrandom: %w", err)
	}
	sum := argon2.IDKey([]byte(password), salt, timeCost, memoryKiB, uint8(parallelism), keyLen)
	b64Salt := base64.RawStdEncoding.EncodeToString(salt)
	b64Hash := base64.RawStdEncoding.EncodeToString(sum)
	return fmt.Sprintf("$argon2id$v=19$m=%d,t=%d,p=%d$%s$%s",
		memoryKiB, timeCost, parallelism, b64Salt, b64Hash), nil
}

// HashResult is the ABI shape for web_password_hash: {"hash":"…"}.
func HashResult(password string) (map[string]any, error) {
	h, err := Hash(password)
	if err != nil {
		return nil, err
	}
	return map[string]any{"hash": h}, nil
}

// Verify checks plaintext against an argon2 PHC hash, or legacy plaintext equality
// (dev / gold tests).
func Verify(password, encoded string) bool {
	if password == "" || encoded == "" {
		return false
	}
	if IsHashed(encoded) {
		return verifyArgon2(password, encoded)
	}
	return password == encoded
}

func verifyArgon2(password, encoded string) bool {
	// $argon2id$v=19$m=19456,t=2,p=1$salt$hash
	parts := strings.Split(encoded, "$")
	// "", "argon2id", "v=19", "m=…,t=…,p=…", salt, hash
	if len(parts) != 6 {
		return false
	}
	if parts[1] != "argon2id" && parts[1] != "argon2i" && parts[1] != "argon2d" {
		return false
	}
	var m uint32
	var t, p uint32
	for _, kv := range strings.Split(parts[3], ",") {
		k, v, ok := strings.Cut(kv, "=")
		if !ok {
			return false
		}
		n, err := strconv.ParseUint(v, 10, 32)
		if err != nil {
			return false
		}
		switch k {
		case "m":
			m = uint32(n)
		case "t":
			t = uint32(n)
		case "p":
			p = uint32(n)
		}
	}
	if m == 0 || t == 0 || p == 0 {
		return false
	}
	salt, err := base64.RawStdEncoding.DecodeString(parts[4])
	if err != nil {
		salt, err = base64.StdEncoding.DecodeString(parts[4])
		if err != nil {
			return false
		}
	}
	want, err := base64.RawStdEncoding.DecodeString(parts[5])
	if err != nil {
		want, err = base64.StdEncoding.DecodeString(parts[5])
		if err != nil {
			return false
		}
	}
	var got []byte
	switch parts[1] {
	case "argon2id":
		got = argon2.IDKey([]byte(password), salt, t, m, uint8(p), uint32(len(want)))
	case "argon2i":
		got = argon2.Key([]byte(password), salt, t, m, uint8(p), uint32(len(want)))
	default:
		return false
	}
	return subtle.ConstantTimeCompare(got, want) == 1
}
