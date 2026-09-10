// Package storage ports file: blob storage from the Rust web plugin (W-G6).
package storage

import (
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"unicode/utf8"
)

func isFile(url string) bool {
	u := strings.ToLower(strings.TrimSpace(url))
	return strings.HasPrefix(u, "file:") || strings.HasPrefix(u, "file://")
}

func isS3(url string) bool {
	return strings.HasPrefix(strings.ToLower(strings.TrimSpace(url)), "s3://")
}

func fileRoot(url string) (string, error) {
	raw := strings.TrimSpace(url)
	raw = strings.TrimPrefix(raw, "file://")
	raw = strings.TrimPrefix(raw, "file:")
	if raw == "" {
		return "", nil
	}
	if err := os.MkdirAll(raw, 0o755); err != nil {
		return "", fmt.Errorf("storage mkdir: %w", err)
	}
	return raw, nil
}

func safeKey(key string) (string, error) {
	k := strings.TrimPrefix(strings.TrimSpace(key), "/")
	if k == "" {
		return "", fmt.Errorf("storage key is empty")
	}
	if strings.Contains(k, "..") {
		return "", fmt.Errorf("storage key must not contain `..`")
	}
	return strings.ReplaceAll(k, `\`, "/"), nil
}

func pathFor(root, key string) (string, error) {
	k, err := safeKey(key)
	if err != nil {
		return "", err
	}
	return filepath.Join(root, filepath.FromSlash(k)), nil
}

func ctypePath(dest string) string {
	return dest + ".ctype"
}

// Open validates and describes a storage backend.
func Open(url string) (map[string]any, error) {
	url = strings.TrimSpace(url)
	if isFile(url) {
		if _, err := fileRoot(url); err != nil {
			return nil, err
		}
		return map[string]any{
			"_type":   "storage",
			"url":     url,
			"backend": "file",
		}, nil
	}
	if isS3(url) {
		rest := strings.TrimPrefix(url, "s3://")
		rest = strings.TrimPrefix(rest, "S3://")
		bucket := rest
		if i := strings.IndexByte(bucket, '?'); i >= 0 {
			bucket = bucket[:i]
		}
		if bucket == "" {
			return nil, fmt.Errorf("s3 url missing bucket (s3://bucket?endpoint=…)")
		}
		return map[string]any{
			"_type":   "storage",
			"url":     url,
			"backend": "s3",
		}, nil
	}
	return nil, fmt.Errorf("storage url must be `file:…` or `s3://bucket?…`, got `%s`", url)
}

func loadBytes(body *string, path *string) ([]byte, error) {
	if path != nil && *path != "" {
		b, err := os.ReadFile(*path)
		if err != nil {
			return nil, fmt.Errorf("read path `%s`: %w", *path, err)
		}
		return b, nil
	}
	if body != nil {
		return []byte(*body), nil
	}
	return nil, fmt.Errorf("storage.put needs `body` or `path`")
}

// Put stores bytes from body or local path.
func Put(url, key string, body, path, contentType *string) (map[string]any, error) {
	bytes, err := loadBytes(body, path)
	if err != nil {
		return nil, err
	}
	ct := "application/octet-stream"
	if contentType != nil && strings.TrimSpace(*contentType) != "" {
		ct = strings.TrimSpace(*contentType)
	}
	return PutBytes(url, key, bytes, ct)
}

// PutBytes persists raw bytes (multipart upload / binary save).
func PutBytes(url, key string, data []byte, contentType string) (map[string]any, error) {
	ct := strings.TrimSpace(contentType)
	if ct == "" {
		ct = "application/octet-stream"
	}
	if isFile(url) {
		root, err := fileRoot(url)
		if err != nil {
			return nil, err
		}
		dest, err := pathFor(root, key)
		if err != nil {
			return nil, err
		}
		if parent := filepath.Dir(dest); parent != "" && parent != "." {
			if err := os.MkdirAll(parent, 0o755); err != nil {
				return nil, err
			}
		}
		if err := os.WriteFile(dest, data, 0o644); err != nil {
			return nil, err
		}
		_ = os.WriteFile(ctypePath(dest), []byte(ct), 0o644)
		return map[string]any{
			"ok":           true,
			"key":          key,
			"size":         float64(len(data)),
			"content_type": ct,
		}, nil
	}
	if isS3(url) {
		return nil, fmt.Errorf("s3 put not implemented in Go plugin yet")
	}
	return nil, fmt.Errorf("unknown storage backend")
}

// Get reads an object; JSON shape matches Rust gold.
func Get(url, key string) (map[string]any, error) {
	if isFile(url) {
		root, err := fileRoot(url)
		if err != nil {
			return nil, err
		}
		dest, err := pathFor(root, key)
		if err != nil {
			return nil, err
		}
		info, err := os.Stat(dest)
		if err != nil || info.IsDir() {
			return map[string]any{"ok": false}, nil
		}
		bytes, err := os.ReadFile(dest)
		if err != nil {
			return nil, err
		}
		ctBytes, err := os.ReadFile(ctypePath(dest))
		ct := "application/octet-stream"
		if err == nil {
			ct = string(ctBytes)
		}
		if utf8.Valid(bytes) {
			s := string(bytes)
			return map[string]any{
				"ok":           true,
				"key":          key,
				"body":         s,
				"content_type": ct,
				"size":         float64(len(s)),
			}, nil
		}
		return map[string]any{
			"ok":           true,
			"key":          key,
			"path":         dest,
			"content_type": ct,
			"size":         float64(len(bytes)),
			"binary":       true,
		}, nil
	}
	if isS3(url) {
		return nil, fmt.Errorf("s3 get not implemented in Go plugin yet")
	}
	return nil, fmt.Errorf("unknown storage backend")
}

// Delete removes an object if present.
func Delete(url, key string) (map[string]any, error) {
	if isFile(url) {
		root, err := fileRoot(url)
		if err != nil {
			return nil, err
		}
		dest, err := pathFor(root, key)
		if err != nil {
			return nil, err
		}
		info, err := os.Stat(dest)
		ok := false
		if err == nil && !info.IsDir() {
			if err := os.Remove(dest); err != nil {
				return nil, err
			}
			_ = os.Remove(ctypePath(dest))
			ok = true
		}
		return map[string]any{"ok": ok}, nil
	}
	if isS3(url) {
		return nil, fmt.Errorf("s3 delete not implemented in Go plugin yet")
	}
	return nil, fmt.Errorf("unknown storage backend")
}

// ReadBytes loads raw object bytes for HTTP download.
// Returns found=false when the object is missing.
func ReadBytes(url, key string) ([]byte, string, string, bool, error) {
	k, err := safeKey(key)
	if err != nil {
		return nil, "", "", false, err
	}
	filename := k
	if i := strings.LastIndexByte(k, '/'); i >= 0 && i < len(k)-1 {
		filename = k[i+1:]
	}
	if filename == "" {
		filename = "download"
	}
	if isFile(url) {
		root, err := fileRoot(url)
		if err != nil {
			return nil, "", "", false, err
		}
		dest, err := pathFor(root, k)
		if err != nil {
			return nil, "", "", false, err
		}
		info, err := os.Stat(dest)
		if err != nil || info.IsDir() {
			return nil, "", "", false, nil
		}
		bytes, err := os.ReadFile(dest)
		if err != nil {
			return nil, "", "", false, err
		}
		ct := "application/octet-stream"
		if ctBytes, err := os.ReadFile(ctypePath(dest)); err == nil {
			ct = string(ctBytes)
		}
		return bytes, ct, filename, true, nil
	}
	if isS3(url) {
		return nil, "", "", false, fmt.Errorf("s3 read not implemented in Go plugin yet")
	}
	return nil, "", "", false, fmt.Errorf("unknown storage backend")
}

// List returns keys under optional prefix.
func List(url, prefix string) (map[string]any, error) {
	prefix = strings.TrimPrefix(strings.TrimSpace(prefix), "/")
	if isFile(url) {
		root, err := fileRoot(url)
		if err != nil {
			return nil, err
		}
		var keys []string
		if err := walkKeys(root, root, prefix, &keys); err != nil {
			return nil, err
		}
		return map[string]any{
			"ok":    true,
			"keys":  keys,
			"count": float64(len(keys)),
		}, nil
	}
	if isS3(url) {
		return nil, fmt.Errorf("s3 list not implemented in Go plugin yet")
	}
	return nil, fmt.Errorf("unknown storage backend")
}

func walkKeys(root, dir, prefix string, out *[]string) error {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil
	}
	for _, ent := range entries {
		name := ent.Name()
		if strings.HasSuffix(name, ".ctype") {
			continue
		}
		path := filepath.Join(dir, name)
		if ent.IsDir() {
			if err := walkKeys(root, path, prefix, out); err != nil {
				return err
			}
			continue
		}
		rel, err := filepath.Rel(root, path)
		if err != nil {
			return err
		}
		rel = filepath.ToSlash(rel)
		if prefix == "" || strings.HasPrefix(rel, prefix) {
			*out = append(*out, rel)
		}
	}
	return nil
}
