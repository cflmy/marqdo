// Package apikey implements Bearer API key verification (G-API1).
//
// Hash formula: sha256 hex of pepper + ":" + key (documented for anlian apikeys).
package apikey

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"strings"

	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

var (
	hashKeys   = []string{"hash", "哈希", "key_hash", "密钥哈希"}
	scopeKeys  = []string{"scopes", "scope", "作用域", "权限"}
)

// Hash returns the stored digest for pepper+key (sha256 hex of "pepper:key").
func Hash(pepper, key string) string {
	sum := sha256.Sum256([]byte(pepper + ":" + key))
	return hex.EncodeToString(sum[:])
}

// BearerToken strips a Bearer prefix from an Authorization header value.
func BearerToken(authorization string) string {
	s := strings.TrimSpace(authorization)
	if s == "" {
		return ""
	}
	const prefix = "Bearer "
	if len(s) > len(prefix) && strings.EqualFold(s[:len(prefix)], prefix) {
		return strings.TrimSpace(s[len(prefix):])
	}
	return s
}

// Check verifies a raw key or Authorization header against a GFM keys table.
// Args: key or authorization, pepper, keys (rows with hash + scopes).
// Returns {"ok":true,"scopes":[...]} or {"ok":false}.
func Check(key, authorization, pepper string, keys any) map[string]any {
	raw := strings.TrimSpace(key)
	if raw == "" {
		raw = BearerToken(authorization)
	}
	if raw == "" || pepper == "" {
		return map[string]any{"ok": false}
	}
	digest := Hash(pepper, raw)
	rowsAny := table.AsRows(keys)
	rows, ok := rowsAny.([]any)
	if !ok {
		return map[string]any{"ok": false}
	}
	for _, row := range rows {
		m, ok := row.(map[string]any)
		if !ok {
			continue
		}
		stored := pickCell(m, hashKeys)
		if stored == "" || !strings.EqualFold(stored, digest) {
			continue
		}
		return map[string]any{"ok": true, "scopes": parseScopes(pickCell(m, scopeKeys))}
	}
	return map[string]any{"ok": false}
}

func pickCell(m map[string]any, keys []string) string {
	for _, k := range keys {
		if v, ok := m[k]; ok && v != nil {
			return strings.TrimSpace(fmt.Sprint(v))
		}
	}
	return ""
}

func parseScopes(raw string) []any {
	raw = strings.TrimSpace(raw)
	if raw == "" {
		return []any{}
	}
	parts := strings.Split(raw, ",")
	out := make([]any, 0, len(parts))
	for _, p := range parts {
		p = strings.TrimSpace(p)
		if p != "" {
			out = append(out, p)
		}
	}
	return out
}
