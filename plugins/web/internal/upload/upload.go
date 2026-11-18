// Package upload ports media validation and offline save (W-G6).
package upload

import (
	"crypto/rand"
	"encoding/hex"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"

	"github.com/marqdo/marqdo/plugins/web/internal/storage"
)

// Validate checks size / MIME / extension against optional allowlist table.
func Validate(filename, contentType string, size, maxBytes uint64, types any) map[string]any {
	if size > maxBytes {
		return map[string]any{
			"ok":    false,
			"error": fmt.Sprintf("file too large: %d > %d", size, maxBytes),
		}
	}
	name := strings.TrimSpace(filename)
	if name == "" {
		return map[string]any{"ok": false, "error": "empty filename"}
	}
	if strings.Contains(name, "..") || strings.Contains(name, "/") || strings.Contains(name, `\`) {
		return map[string]any{"ok": false, "error": "unsafe filename"}
	}
	ct := strings.ToLower(strings.TrimSpace(contentType))
	if i := strings.IndexByte(ct, ';'); i >= 0 {
		ct = strings.TrimSpace(ct[:i])
	}
	ext := ""
	if i := strings.LastIndexByte(name, '.'); i >= 0 && i < len(name)-1 {
		ext = strings.ToLower(name[i+1:])
	}
	if types != nil && !typesAllow(types, ct, ext) {
		return map[string]any{
			"ok":    false,
			"error": fmt.Sprintf("type not allowed: content_type=%s ext=%s", ct, ext),
		}
	}
	outCT := ct
	if outCT == "" {
		outCT = "application/octet-stream"
	}
	return map[string]any{
		"ok":           true,
		"filename":     name,
		"content_type": outCT,
		"size":         float64(size),
		"ext":          ext,
	}
}

func typesAllow(types any, contentType, ext string) bool {
	if types == nil {
		return true
	}
	switch t := types.(type) {
	case string:
		s := strings.TrimSpace(t)
		if s == "" {
			return true
		}
		for _, p := range strings.Split(s, ",") {
			p = strings.ToLower(strings.TrimSpace(p))
			if p != "" && p == contentType {
				return true
			}
		}
		return false
	case []any:
		if len(t) == 0 {
			return true
		}
		for _, row := range t {
			if rowAllows(row, contentType, ext) {
				return true
			}
		}
		return false
	case map[string]any:
		colMime := first(t, "类型", "type", "mime")
		colExt := first(t, "扩展名", "ext", "extension")
		if ma, ok := colMime.([]any); ok {
			var exts []string
			switch ev := colExt.(type) {
			case []any:
				for _, x := range ev {
					exts = append(exts, strings.ToLower(cellStr(x)))
				}
			case string:
				exts = []string{strings.ToLower(ev)}
			}
			if len(ma) == 0 {
				return true
			}
			for i, mimeV := range ma {
				mime := strings.ToLower(strings.TrimSpace(cellStr(mimeV)))
				rowExt := ""
				if i < len(exts) {
					rowExt = exts[i]
				}
				mimeOK := mime == "" || mime == contentType
				extOK := rowExt == "" || extMatches(rowExt, ext)
				if mimeOK && extOK {
					return true
				}
			}
			return false
		}
		if colMime != nil || colExt != nil {
			return rowAllows(t, contentType, ext)
		}
		if len(t) == 0 {
			return true
		}
		for mime, extsV := range t {
			exts := extList(extsV)
			mimeOK := strings.TrimSpace(mime) == "" || strings.EqualFold(mime, contentType)
			extOK := exts == "" || extMatches(exts, ext)
			if mimeOK && extOK {
				return true
			}
		}
		return false
	default:
		return true
	}
}

func rowAllows(row any, contentType, ext string) bool {
	switch t := row.(type) {
	case string:
		return strings.EqualFold(strings.TrimSpace(t), contentType)
	case map[string]any:
		mime := strings.ToLower(strings.TrimSpace(firstStr(t, "类型", "type", "mime")))
		exts := strings.ToLower(strings.TrimSpace(firstStr(t, "扩展名", "ext", "extension")))
		mimeOK := mime == "" || mime == contentType
		extOK := exts == "" || extMatches(exts, ext)
		return mimeOK && extOK
	default:
		return false
	}
}

func extMatches(exts, ext string) bool {
	for _, e := range strings.Split(exts, ",") {
		e = strings.TrimSpace(strings.TrimPrefix(e, "."))
		if e == ext {
			return true
		}
	}
	return false
}

func extList(v any) string {
	switch t := v.(type) {
	case string:
		return strings.ToLower(strings.TrimSpace(t))
	case []any:
		parts := make([]string, 0, len(t))
		for _, x := range t {
			if s, ok := x.(string); ok {
				parts = append(parts, s)
			}
		}
		return strings.ToLower(strings.Join(parts, ","))
	default:
		return ""
	}
}

func first(m map[string]any, keys ...string) any {
	for _, k := range keys {
		if v, ok := m[k]; ok {
			return v
		}
	}
	return nil
}

func firstStr(m map[string]any, keys ...string) string {
	return cellStr(first(m, keys...))
}

func cellStr(v any) string {
	switch t := v.(type) {
	case nil:
		return ""
	case string:
		return t
	case float64:
		if t == float64(int64(t)) {
			return fmt.Sprintf("%d", int64(t))
		}
		return fmt.Sprintf("%v", t)
	case bool:
		if t {
			return "true"
		}
		return "false"
	default:
		return fmt.Sprint(t)
	}
}

// MakeKey builds prefix + timestamp-random + sanitized basename.
func MakeKey(prefix, filename string) (string, error) {
	base := filename
	if i := strings.LastIndexAny(filename, `/\`); i >= 0 {
		base = filename[i+1:]
	}
	base = strings.TrimSpace(base)
	if base == "" {
		return "", fmt.Errorf("empty filename")
	}
	var safe strings.Builder
	for _, c := range base {
		if (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') || (c >= '0' && c <= '9') ||
			c == '.' || c == '-' || c == '_' {
			safe.WriteRune(c)
		} else {
			safe.WriteRune('_')
		}
	}
	s := safe.String()
	if strings.Contains(s, "..") {
		return "", fmt.Errorf("unsafe filename")
	}
	pref := strings.ReplaceAll(strings.TrimPrefix(strings.TrimSpace(prefix), "/"), `\`, "/")
	if pref != "" && !strings.HasSuffix(pref, "/") {
		pref += "/"
	}
	if strings.Contains(pref, "..") {
		return "", fmt.Errorf("prefix must not contain `..`")
	}
	rnd := make([]byte, 4)
	if _, err := rand.Read(rnd); err != nil {
		return "", err
	}
	return fmt.Sprintf("%s%d-%s-%s", pref, time.Now().Unix(), hex.EncodeToString(rnd), s), nil
}

// Save reads local path bytes into storage under key (or auto key).
func Save(storageURL string, key *string, path string, contentType, prefix *string) (map[string]any, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return nil, fmt.Errorf("read `%s`: %w", path, err)
	}
	filename := filepath.Base(path)
	ct := "application/octet-stream"
	if contentType != nil && strings.TrimSpace(*contentType) != "" {
		ct = strings.TrimSpace(*contentType)
	}
	objectKey := ""
	if key != nil && strings.TrimSpace(*key) != "" {
		objectKey = *key
	} else {
		p := "uploads/"
		if prefix != nil && strings.TrimSpace(*prefix) != "" {
			p = *prefix
		}
		objectKey, err = MakeKey(p, filename)
		if err != nil {
			return nil, err
		}
	}
	return storage.PutBytes(storageURL, objectKey, data, ct)
}
