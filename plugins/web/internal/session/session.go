// Package session ports in-memory / SQLite session store from the Rust web plugin.
package session

import (
	"context"
	"crypto/rand"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"strings"
	"sync"
	"time"

	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/redis/go-redis/v9"
)

const (
	sessionTable = "_marqdo_sessions"
	csrfKey      = "_csrf"
	defaultTTL   = 3600
)

// Config controls session backend.
type Config struct {
	DBURL        string // sqlite:… or legacy SQL session table
	RedisURL     string // redis:// or rediss:// (G-SESS1)
	TTLSec       uint64
	CookieSecure bool
}

var (
	cfgMu sync.Mutex
	cfg   = Config{TTLSec: defaultTTL}

	memMu sync.Mutex
	mem   = map[string]memRec{}
	memTTL uint64 = defaultTTL

	redisMu     sync.Mutex
	redisClient *redis.Client
	redisURL    string
)

type memRec struct {
	exp  uint64
	data map[string]any
}

func nowSecs() uint64 {
	return uint64(time.Now().Unix())
}

func cloneData(m map[string]any) map[string]any {
	out := make(map[string]any, len(m))
	for k, v := range m {
		out[k] = v
	}
	return out
}

func isRedisURL(url string) bool {
	u := strings.ToLower(strings.TrimSpace(url))
	return strings.HasPrefix(u, "redis://") || strings.HasPrefix(u, "rediss://")
}

func isSQLiteURL(url string) bool {
	u := strings.TrimSpace(url)
	return u != "" && !isRedisURL(u)
}

// Configure sets session backend before listen or offline ABI calls.
// RedisURL or a redis:// DBURL selects the Redis backend; sqlite DBURL keeps SQL table storage.
func Configure(c Config) {
	ttl := c.TTLSec
	if ttl == 0 {
		ttl = defaultTTL
	}
	c.TTLSec = ttl
	if c.RedisURL == "" && isRedisURL(c.DBURL) {
		c.RedisURL = strings.TrimSpace(c.DBURL)
		c.DBURL = ""
	}
	if isSQLiteURL(c.DBURL) {
		_ = ensureSessionTable(c.DBURL)
	}
	if c.RedisURL != "" {
		_ = openRedis(c.RedisURL)
	} else {
		closeRedis()
	}
	cfgMu.Lock()
	cfg = c
	cfgMu.Unlock()
}

func openRedis(url string) error {
	redisMu.Lock()
	defer redisMu.Unlock()
	if redisClient != nil && redisURL == url {
		return nil
	}
	if redisClient != nil {
		_ = redisClient.Close()
		redisClient = nil
	}
	opts, err := redis.ParseURL(url)
	if err != nil {
		return err
	}
	redisClient = redis.NewClient(opts)
	redisURL = url
	return nil
}

func closeRedis() {
	redisMu.Lock()
	defer redisMu.Unlock()
	if redisClient != nil {
		_ = redisClient.Close()
		redisClient = nil
		redisURL = ""
	}
}

// CloseRedis releases the Redis client (plugin shutdown / tests).
func CloseRedis() {
	closeRedis()
}

func redisKey(id string) string {
	return "marqdo:session:" + id
}

type redisBlob struct {
	Data map[string]any `json:"data"`
}

func loadRedis(id string) (map[string]any, bool) {
	client := redisClientForRead()
	if client == nil {
		return nil, false
	}
	ctx := context.Background()
	raw, err := client.Get(ctx, redisKey(id)).Result()
	if err == redis.Nil || err != nil {
		return nil, false
	}
	var blob redisBlob
	if err := json.Unmarshal([]byte(raw), &blob); err != nil {
		return nil, false
	}
	if blob.Data == nil {
		blob.Data = map[string]any{}
	}
	return blob.Data, true
}

func saveRedis(id string, ttl uint64, data map[string]any) bool {
	client := redisClientForRead()
	if client == nil {
		return false
	}
	raw, err := json.Marshal(redisBlob{Data: cloneData(data)})
	if err != nil {
		return false
	}
	ctx := context.Background()
	if ttl == 0 {
		ttl = defaultTTL
	}
	if err := client.SetEx(ctx, redisKey(id), raw, time.Duration(ttl)*time.Second).Err(); err != nil {
		return false
	}
	return true
}

func deleteRedis(id string) {
	client := redisClientForRead()
	if client == nil {
		return
	}
	ctx := context.Background()
	_, _ = client.Del(ctx, redisKey(id)).Result()
}

func redisClientForRead() *redis.Client {
	redisMu.Lock()
	defer redisMu.Unlock()
	return redisClient
}

func usingRedis() bool {
	c := getCfg()
	return c.RedisURL != ""
}

func getCfg() Config {
	cfgMu.Lock()
	defer cfgMu.Unlock()
	return cfg
}

// Reset clears in-memory sessions (tests / each listen).
func Reset(ttlSec uint64) {
	ttl := ttlSec
	if ttl == 0 {
		ttl = defaultTTL
	}
	cfgMu.Lock()
	cfg.TTLSec = ttl
	if cfg.DBURL == "" {
		// keep cookie_secure / db as-is when already configured
	}
	cfgMu.Unlock()
	memMu.Lock()
	mem = map[string]memRec{}
	memTTL = ttl
	memMu.Unlock()
}

func ensureSessionTable(url string) error {
	create := fmt.Sprintf(`
CREATE TABLE IF NOT EXISTS "%s" (
  id TEXT PRIMARY KEY,
  expires_at INTEGER NOT NULL,
  data TEXT NOT NULL
)`, sessionTable)
	if _, err := db.Exec(url, create, nil); err != nil {
		return err
	}
	idx := fmt.Sprintf(`CREATE INDEX IF NOT EXISTS idx_marqdo_sessions_exp ON "%s"(expires_at)`, sessionTable)
	_, err := db.Exec(url, idx, nil)
	return err
}

func pruneMem() {
	now := nowSecs()
	for id, rec := range mem {
		if rec.exp <= now {
			delete(mem, id)
		}
	}
}

func pruneSQL(url string) {
	now := int64(nowSecs())
	_, _ = db.Exec(url, fmt.Sprintf(`DELETE FROM "%s" WHERE expires_at <= ?`, sessionTable), []any{now})
}

func secureRandomID() (string, error) {
	b := make([]byte, 32)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return hex.EncodeToString(b), nil
}

func randomCSRF() (string, error) {
	b := make([]byte, 24)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return hex.EncodeToString(b), nil
}

func ensureCSRF(data map[string]any) (string, error) {
	if v, ok := data[csrfKey].(string); ok && v != "" {
		return v, nil
	}
	t, err := randomCSRF()
	if err != nil {
		return "", err
	}
	data[csrfKey] = t
	return t, nil
}

func loadSQL(url, id string) (uint64, map[string]any, bool) {
	q, err := db.Query(url, fmt.Sprintf(`SELECT expires_at, data FROM "%s" WHERE id = ?`, sessionTable), []any{id})
	if err != nil {
		return 0, nil, false
	}
	rows, _ := q["rows"].([]any)
	if len(rows) == 0 {
		return 0, nil, false
	}
	row, ok := rows[0].(map[string]any)
	if !ok {
		return 0, nil, false
	}
	exp := uint64(asInt64(row["expires_at"]))
	dataS, _ := row["data"].(string)
	data := map[string]any{}
	_ = json.Unmarshal([]byte(dataS), &data)
	if data == nil {
		data = map[string]any{}
	}
	return exp, data, true
}

func asInt64(v any) int64 {
	switch t := v.(type) {
	case int64:
		return t
	case float64:
		return int64(t)
	case int:
		return int64(t)
	default:
		return 0
	}
}

func saveSQL(url, id string, exp uint64, data map[string]any) error {
	raw, err := json.Marshal(data)
	if err != nil {
		return err
	}
	_, err = db.Exec(url, fmt.Sprintf(`
INSERT INTO "%s" (id, expires_at, data) VALUES (?, ?, ?)
ON CONFLICT (id) DO UPDATE SET expires_at = excluded.expires_at, data = excluded.data`, sessionTable),
		[]any{id, int64(exp), string(raw)})
	return err
}

func deleteSQL(url, id string) {
	_, _ = db.Exec(url, fmt.Sprintf(`DELETE FROM "%s" WHERE id = ?`, sessionTable), []any{id})
}

func touchExpiry(ttl uint64) uint64 {
	return nowSecs() + ttl
}

func getRecord(id string) (map[string]any, bool) {
	c := getCfg()
	now := nowSecs()
	if usingRedis() {
		data, ok := loadRedis(id)
		if !ok {
			return nil, false
		}
		return cloneData(data), true
	}
	if c.DBURL != "" {
		pruneSQL(c.DBURL)
		exp, data, ok := loadSQL(c.DBURL, id)
		if !ok {
			return nil, false
		}
		if exp <= now {
			deleteSQL(c.DBURL, id)
			return nil, false
		}
		return data, true
	}
	memMu.Lock()
	defer memMu.Unlock()
	pruneMem()
	rec, ok := mem[id]
	if !ok {
		return nil, false
	}
	if rec.exp <= now {
		delete(mem, id)
		return nil, false
	}
	return cloneData(rec.data), true
}

func putRecord(id string, exp uint64, data map[string]any) bool {
	c := getCfg()
	if usingRedis() {
		ttl := c.TTLSec
		if ttl == 0 {
			ttl = defaultTTL
		}
		if exp > nowSecs() {
			ttl = exp - nowSecs()
		}
		return saveRedis(id, ttl, data)
	}
	if c.DBURL != "" {
		return saveSQL(c.DBURL, id, exp, data) == nil
	}
	memMu.Lock()
	defer memMu.Unlock()
	mem[id] = memRec{exp: exp, data: cloneData(data)}
	return true
}

// NewID creates a session and returns its id (optional ttl; 0 = configured default).
func NewID(ttlSec uint64) string {
	c := getCfg()
	ttl := ttlSec
	if ttl == 0 {
		ttl = c.TTLSec
	}
	if ttl == 0 {
		ttl = defaultTTL
	}
	if ttlSec > 0 {
		cfgMu.Lock()
		cfg.TTLSec = ttl
		cfgMu.Unlock()
	}
	id, err := secureRandomID()
	if err != nil {
		id = fmt.Sprintf("fallback-%d", nowSecs())
	}
	data := map[string]any{}
	_, _ = ensureCSRF(data)
	exp := touchExpiry(ttl)
	if usingRedis() {
		ttl := ttl
		_ = saveRedis(id, ttl, data)
	} else if c.DBURL != "" {
		pruneSQL(c.DBURL)
		_ = saveSQL(c.DBURL, id, exp, data)
	} else {
		memMu.Lock()
		memTTL = ttl
		pruneMem()
		mem[id] = memRec{exp: exp, data: data}
		memMu.Unlock()
	}
	return id
}

// New is ABI shape web_session_new → {"id":"…"}.
func New(ttlSec uint64) map[string]any {
	return map[string]any{"id": NewID(ttlSec)}
}

// Set stores a key (sliding expiry). ABI: {"ok":bool}.
func Set(id, key string, value any) map[string]any {
	return map[string]any{"ok": SetValue(id, key, value)}
}

// SetValue is the low-level setter.
func SetValue(id, key string, value any) bool {
	data, ok := getRecord(id)
	if !ok {
		return false
	}
	data[key] = value
	return putRecord(id, touchExpiry(getCfg().TTLSec), data)
}

// Get is ABI shape: {"ok":true,"value":…} or {"ok":false}.
func Get(id, key string) map[string]any {
	v, ok := GetValue(id, key)
	if !ok {
		return map[string]any{"ok": false}
	}
	return map[string]any{"ok": true, "value": v}
}

// GetValue returns the value and whether it existed (sliding expiry on hit).
func GetValue(id, key string) (any, bool) {
	data, ok := getRecord(id)
	if !ok {
		return nil, false
	}
	v, exists := data[key]
	if !exists {
		return nil, false
	}
	putRecord(id, touchExpiry(getCfg().TTLSec), data)
	return v, true
}

// Del removes a key. ABI: {"ok":bool}.
func Del(id, key string) map[string]any {
	return map[string]any{"ok": DelKey(id, key)}
}

// DelKey is the low-level deleter.
func DelKey(id, key string) bool {
	data, ok := getRecord(id)
	if !ok {
		return false
	}
	if _, exists := data[key]; !exists {
		return false
	}
	delete(data, key)
	return putRecord(id, touchExpiry(getCfg().TTLSec), data)
}

// Destroy removes a session. ABI: {"ok":bool}.
func Destroy(id string) map[string]any {
	return map[string]any{"ok": DestroyID(id)}
}

// DestroyID is the low-level destroyer.
func DestroyID(id string) bool {
	c := getCfg()
	if usingRedis() {
		deleteRedis(id)
		return true
	}
	if c.DBURL != "" {
		deleteSQL(c.DBURL, id)
		return true
	}
	memMu.Lock()
	defer memMu.Unlock()
	_, ok := mem[id]
	if ok {
		delete(mem, id)
	}
	return ok
}

// CSRFFor returns (creating if needed) the CSRF token for a session.
func CSRFFor(id string) (string, bool) {
	data, ok := getRecord(id)
	if !ok {
		return "", false
	}
	tok, err := ensureCSRF(data)
	if err != nil {
		return "", false
	}
	putRecord(id, touchExpiry(getCfg().TTLSec), data)
	return tok, true
}

// ValidateCSRF checks token against session.
func ValidateCSRF(id, token string) bool {
	if token == "" {
		return false
	}
	v, ok := GetValue(id, csrfKey)
	if !ok {
		return false
	}
	s, _ := v.(string)
	return s == token
}

// SessionCookie builds Set-Cookie for marqdo_sid.
// ttlSec 0 means the configured default (never Max-Age=0 — that clears cookies;
// use ClearCookie for logout).
func SessionCookie(id string, ttlSec uint64, secure bool) string {
	if ttlSec == 0 {
		ttlSec = getCfg().TTLSec
		if ttlSec == 0 {
			ttlSec = defaultTTL
		}
	}
	s := fmt.Sprintf("marqdo_sid=%s; HttpOnly; Path=/; SameSite=Lax; Max-Age=%d", id, ttlSec)
	if secure {
		s += "; Secure"
	}
	return s
}

// IssueCookie builds Set-Cookie using Configure'd TTL and CookieSecure.
func IssueCookie(id string) string {
	c := getCfg()
	return SessionCookie(id, c.TTLSec, c.CookieSecure)
}

// ClearCookie is the Set-Cookie value that expires marqdo_sid immediately.
func ClearCookie() string {
	return "marqdo_sid=; HttpOnly; Path=/; Max-Age=0; SameSite=Lax"
}

// IDFromCookie parses marqdo_sid from a Cookie header.
func IDFromCookie(cookieHeader string) (string, bool) {
	if cookieHeader == "" {
		return "", false
	}
	for _, part := range strings.Split(cookieHeader, ";") {
		p := strings.TrimSpace(part)
		if v, ok := strings.CutPrefix(p, "marqdo_sid="); ok {
			v = strings.Trim(v, `"`)
			if v != "" {
				return v, true
			}
		}
	}
	return "", false
}

// ParseRolesCSV splits "admin,author" into lowercased roles.
func ParseRolesCSV(raw string) []string {
	var out []string
	for _, part := range strings.Split(raw, ",") {
		part = strings.TrimSpace(part)
		if part != "" {
			out = append(out, strings.ToLower(part))
		}
	}
	return out
}

// RoleFromCookie returns session role (visitor when anonymous or missing).
func RoleFromCookie(cookieHeader string) string {
	sid, ok := IDFromCookie(cookieHeader)
	if !ok {
		return "visitor"
	}
	if v, ok := GetValue(sid, "role"); ok {
		if s, ok := v.(string); ok && s != "" {
			return strings.ToLower(s)
		}
	}
	if v, ok := GetValue(sid, "username"); ok {
		if s, ok := v.(string); ok && s != "" {
			return "admin"
		}
	}
	return "visitor"
}

// UsernameFromCookie returns the session username or empty string.
func UsernameFromCookie(cookieHeader string) string {
	sid, ok := IDFromCookie(cookieHeader)
	if !ok {
		return ""
	}
	if v, ok := GetValue(sid, "username"); ok {
		if s, ok := v.(string); ok {
			return s
		}
	}
	return ""
}

// RoleAllowed reports whether role is in allowed (empty allowed = all).
func RoleAllowed(role string, allowed []string) bool {
	if len(allowed) == 0 {
		return true
	}
	role = strings.ToLower(role)
	for _, a := range allowed {
		if strings.EqualFold(a, role) {
			return true
		}
	}
	return false
}

// PermissionsFromCookie returns session permission codes (empty if anonymous).
func PermissionsFromCookie(cookieHeader string) []string {
	sid, ok := IDFromCookie(cookieHeader)
	if !ok {
		return nil
	}
	return PermissionsFromSession(sid)
}

// PermissionsFromSession reads the permissions CSV from a session id.
func PermissionsFromSession(sid string) []string {
	if v, ok := GetValue(sid, "permissions"); ok {
		if s, ok := v.(string); ok && s != "" {
			return ParseRolesCSV(s)
		}
	}
	return nil
}

// SetPermissions stores permission codes on the session.
func SetPermissions(sid string, codes []string) {
	SetValue(sid, "permissions", strings.Join(codes, ","))
}

// PermissionAllowed reports whether the session holds any of needed.
func PermissionAllowed(held, needed []string) bool {
	if len(needed) == 0 {
		return true
	}
	set := map[string]struct{}{}
	for _, h := range held {
		h = strings.ToLower(strings.TrimSpace(h))
		if h == "*" {
			return true
		}
		if h != "" {
			set[h] = struct{}{}
		}
	}
	for _, n := range needed {
		n = strings.ToLower(strings.TrimSpace(n))
		if _, ok := set[n]; ok {
			return true
		}
	}
	return false
}
