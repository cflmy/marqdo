// Package cache ports memory and Redis cache backends from the Rust web plugin (W-G9).
package cache

import (
	"context"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/redis/go-redis/v9"
)

type entry struct {
	value string
	exp   *uint64 // absolute expiry unix secs; nil = no TTL
}

type memStore struct {
	data map[string]entry
}

type backend struct {
	mem      *memStore
	redisURL string
}

var (
	handlesMu sync.Mutex
	handles   = map[string]*backend{}
)

func now() uint64 {
	return uint64(time.Now().Unix())
}

func isMemory(url string) bool {
	u := strings.ToLower(strings.TrimSpace(url))
	return u == "" || u == "memory:" || u == "memory://" || strings.HasPrefix(u, "memory:")
}

func isRedis(url string) bool {
	u := strings.ToLower(strings.TrimSpace(url))
	return strings.HasPrefix(u, "redis://") || strings.HasPrefix(u, "rediss://")
}

// Open creates or reuses a cache handle. Returns {_type, url}.
func Open(url string) (map[string]any, error) {
	url = strings.TrimSpace(url)
	if url == "" {
		url = "memory:"
	}
	if !isMemory(url) && !isRedis(url) {
		return nil, fmt.Errorf("cache url must be `memory:` or `redis://…`, got `%s`", url)
	}
	handlesMu.Lock()
	defer handlesMu.Unlock()
	if _, ok := handles[url]; !ok {
		var b backend
		if isMemory(url) {
			b.mem = &memStore{data: map[string]entry{}}
		} else {
			b.redisURL = url
		}
		handles[url] = &b
	}
	return map[string]any{"_type": "cache", "url": url}, nil
}

func pruneMem(store *memStore) {
	t := now()
	for k, e := range store.data {
		if e.exp != nil && *e.exp <= t {
			delete(store.data, k)
		}
	}
}

func withBackend(url string, f func(*backend) (map[string]any, error)) (map[string]any, error) {
	handlesMu.Lock()
	defer handlesMu.Unlock()
	b, ok := handles[url]
	if !ok {
		return nil, fmt.Errorf("cache `%s` not opened — call cache.new first", url)
	}
	return f(b)
}

// Get returns {ok:true,value} on hit or {ok:false} on miss.
func Get(url, key string) (map[string]any, error) {
	return withBackend(url, func(b *backend) (map[string]any, error) {
		if b.mem != nil {
			pruneMem(b.mem)
			if e, ok := b.mem.data[key]; ok {
				return map[string]any{"ok": true, "value": e.value}, nil
			}
			return map[string]any{"ok": false}, nil
		}
		return redisGet(b.redisURL, key)
	})
}

// Set stores a value with optional TTL seconds (0/nil = no expiry).
func Set(url, key, value string, ttlSec *uint64) (map[string]any, error) {
	return withBackend(url, func(b *backend) (map[string]any, error) {
		if b.mem != nil {
			pruneMem(b.mem)
			var exp *uint64
			if ttlSec != nil && *ttlSec > 0 {
				t := now() + *ttlSec
				exp = &t
			}
			b.mem.data[key] = entry{value: value, exp: exp}
			return map[string]any{"ok": true}, nil
		}
		return redisSet(b.redisURL, key, value, ttlSec)
	})
}

// Del removes a key. Returns {ok:bool} (true if key existed).
func Del(url, key string) (map[string]any, error) {
	return withBackend(url, func(b *backend) (map[string]any, error) {
		if b.mem != nil {
			ok := false
			if _, exists := b.mem.data[key]; exists {
				delete(b.mem.data, key)
				ok = true
			}
			return map[string]any{"ok": ok}, nil
		}
		return redisDel(b.redisURL, key)
	})
}

// Exists returns {ok:bool} — ok is true when the key is present.
func Exists(url, key string) (map[string]any, error) {
	return withBackend(url, func(b *backend) (map[string]any, error) {
		if b.mem != nil {
			pruneMem(b.mem)
			_, ok := b.mem.data[key]
			return map[string]any{"ok": ok}, nil
		}
		return redisExists(b.redisURL, key)
	})
}

// TTL returns remaining seconds: -1 no expiry, -2 missing key.
func TTL(url, key string) (map[string]any, error) {
	return withBackend(url, func(b *backend) (map[string]any, error) {
		if b.mem != nil {
			pruneMem(b.mem)
			if e, ok := b.mem.data[key]; ok {
				var left int64 = -1
				if e.exp != nil {
					left = int64(*e.exp - now())
					if left < 0 {
						left = 0
					}
				}
				return map[string]any{"ok": true, "ttl": left}, nil
			}
			return map[string]any{"ok": false, "ttl": -2}, nil
		}
		return redisTTL(b.redisURL, key)
	})
}

// Reset clears all handles (tests).
func Reset() {
	handlesMu.Lock()
	defer handlesMu.Unlock()
	handles = map[string]*backend{}
}

func redisClient(url string) (*redis.Client, error) {
	opts, err := redis.ParseURL(url)
	if err != nil {
		return nil, err
	}
	return redis.NewClient(opts), nil
}

func redisGet(url, key string) (map[string]any, error) {
	client, err := redisClient(url)
	if err != nil {
		return nil, err
	}
	defer client.Close()
	ctx := context.Background()
	v, err := client.Get(ctx, key).Result()
	if err == redis.Nil {
		return map[string]any{"ok": false}, nil
	}
	if err != nil {
		return nil, err
	}
	return map[string]any{"ok": true, "value": v}, nil
}

func redisSet(url, key, value string, ttlSec *uint64) (map[string]any, error) {
	client, err := redisClient(url)
	if err != nil {
		return nil, err
	}
	defer client.Close()
	ctx := context.Background()
	if ttlSec != nil && *ttlSec > 0 {
		if err := client.SetEx(ctx, key, value, time.Duration(*ttlSec)*time.Second).Err(); err != nil {
			return nil, err
		}
	} else {
		if err := client.Set(ctx, key, value, 0).Err(); err != nil {
			return nil, err
		}
	}
	return map[string]any{"ok": true}, nil
}

func redisDel(url, key string) (map[string]any, error) {
	client, err := redisClient(url)
	if err != nil {
		return nil, err
	}
	defer client.Close()
	ctx := context.Background()
	n, err := client.Del(ctx, key).Result()
	if err != nil {
		return nil, err
	}
	return map[string]any{"ok": n > 0}, nil
}

func redisExists(url, key string) (map[string]any, error) {
	client, err := redisClient(url)
	if err != nil {
		return nil, err
	}
	defer client.Close()
	ctx := context.Background()
	n, err := client.Exists(ctx, key).Result()
	if err != nil {
		return nil, err
	}
	return map[string]any{"ok": n > 0}, nil
}

func redisTTL(url, key string) (map[string]any, error) {
	client, err := redisClient(url)
	if err != nil {
		return nil, err
	}
	defer client.Close()
	ctx := context.Background()
	t, err := client.TTL(ctx, key).Result()
	if err != nil {
		return nil, err
	}
	secs := int64(t.Seconds())
	return map[string]any{"ok": secs >= -1, "ttl": secs}, nil
}
