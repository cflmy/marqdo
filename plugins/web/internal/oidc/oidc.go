// Package oidc implements OAuth 2.0 authorization-code + OIDC userinfo for ext/web.
package oidc

import (
	"crypto/rand"
	"crypto/sha256"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strings"
	"time"
)

// Config is the relying-party settings (CFLMY IdP or any OIDC provider).
type Config struct {
	Issuer       string
	ClientID     string
	ClientSecret string
	RedirectURI  string
	// RedirectOrigins is an optional comma-separated allowlist of origins
	// (e.g. "https://qdqc.com,https://www.qdqc.com") used when RedirectURI is a path.
	RedirectOrigins string
	Scopes          string
	CallbackPath    string
	// AuthorizeURL / TokenURL / UserInfoURL override discovery when set.
	AuthorizeURL string
	TokenURL     string
	UserInfoURL  string
}

// Enabled reports whether the minimum OIDC fields are present.
func (c Config) Enabled() bool {
	return strings.TrimSpace(c.ClientID) != "" &&
		strings.TrimSpace(c.ClientSecret) != "" &&
		strings.TrimSpace(c.RedirectURI) != "" &&
		(strings.TrimSpace(c.Issuer) != "" || strings.TrimSpace(c.AuthorizeURL) != "")
}

// IsPathRedirect reports whether RedirectURI is a path (resolved per request Host).
func (c Config) IsPathRedirect() bool {
	s := strings.TrimSpace(c.RedirectURI)
	if s == "" {
		return false
	}
	if strings.HasPrefix(s, "/") {
		return true
	}
	u, err := url.Parse(s)
	if err != nil {
		return false
	}
	// scheme-relative or bare path without host
	return u.Scheme == "" && u.Host == "" && u.Path != ""
}

// Callback reports the local path (default /oidc/callback).
func (c Config) Callback() string {
	p := strings.TrimSpace(c.CallbackPath)
	if p == "" {
		ru := strings.TrimSpace(c.RedirectURI)
		if c.IsPathRedirect() {
			if !strings.HasPrefix(ru, "/") {
				ru = "/" + ru
			}
			return ru
		}
		if u, err := url.Parse(ru); err == nil && u.Path != "" {
			return u.Path
		}
		return "/oidc/callback"
	}
	if !strings.HasPrefix(p, "/") {
		p = "/" + p
	}
	return p
}

// PublicOrigin derives scheme://host from a login/callback request.
// Prefers X-Forwarded-Proto + X-Forwarded-Host (first value), else Host + TLS.
func PublicOrigin(r *http.Request) (string, error) {
	if r == nil {
		return "", fmt.Errorf("oidc: nil request")
	}
	scheme := strings.TrimSpace(r.Header.Get("X-Forwarded-Proto"))
	if i := strings.IndexByte(scheme, ','); i >= 0 {
		scheme = strings.TrimSpace(scheme[:i])
	}
	if scheme == "" {
		if r.TLS != nil {
			scheme = "https"
		} else {
			scheme = "http"
		}
	}
	host := strings.TrimSpace(r.Header.Get("X-Forwarded-Host"))
	if i := strings.IndexByte(host, ','); i >= 0 {
		host = strings.TrimSpace(host[:i])
	}
	if host == "" {
		host = strings.TrimSpace(r.Host)
	}
	if host == "" {
		return "", fmt.Errorf("oidc: empty Host")
	}
	return scheme + "://" + host, nil
}

// ResolveRedirectURI returns the absolute redirect_uri for this request.
// Absolute RedirectURI is returned unchanged; path RedirectURI is joined to the request origin.
func (c Config) ResolveRedirectURI(r *http.Request) (string, error) {
	ru := strings.TrimSpace(c.RedirectURI)
	if ru == "" {
		return "", fmt.Errorf("oidc: empty redirect_uri")
	}
	if !c.IsPathRedirect() {
		return ru, nil
	}
	origin, err := PublicOrigin(r)
	if err != nil {
		return "", err
	}
	path := ru
	if !strings.HasPrefix(path, "/") {
		path = "/" + path
	}
	abs := origin + path
	if err := c.checkOriginAllowlist(origin); err != nil {
		return "", err
	}
	return abs, nil
}

// AllowedOrigins returns trimmed non-empty origins from RedirectOrigins.
func (c Config) AllowedOrigins() []string {
	raw := strings.TrimSpace(c.RedirectOrigins)
	if raw == "" {
		return nil
	}
	var out []string
	for _, p := range strings.Split(raw, ",") {
		p = strings.TrimRight(strings.TrimSpace(p), "/")
		if p != "" {
			out = append(out, p)
		}
	}
	return out
}

func (c Config) checkOriginAllowlist(origin string) error {
	allowed := c.AllowedOrigins()
	if len(allowed) == 0 {
		return nil
	}
	origin = strings.TrimRight(strings.TrimSpace(origin), "/")
	for _, a := range allowed {
		if strings.EqualFold(a, origin) {
			return nil
		}
	}
	return fmt.Errorf("oidc: origin %q not in redirect_origins", origin)
}

type discoveryDoc struct {
	AuthorizationEndpoint string `json:"authorization_endpoint"`
	TokenEndpoint         string `json:"token_endpoint"`
	UserinfoEndpoint      string `json:"userinfo_endpoint"`
}

// ResolveEndpoints fills Authorize/Token/UserInfo from discovery when missing.
func (c *Config) ResolveEndpoints(client *http.Client) error {
	if c.AuthorizeURL != "" && c.TokenURL != "" && c.UserInfoURL != "" {
		return nil
	}
	issuer := strings.TrimRight(strings.TrimSpace(c.Issuer), "/")
	if issuer == "" {
		return fmt.Errorf("oidc: issuer or explicit endpoints required")
	}
	if client == nil {
		client = &http.Client{Timeout: 15 * time.Second}
	}
	discURL := issuer + "/.well-known/openid-configuration"
	res, err := client.Get(discURL)
	if err != nil {
		return fmt.Errorf("oidc discovery: %w", err)
	}
	defer res.Body.Close()
	body, err := io.ReadAll(io.LimitReader(res.Body, 1<<20))
	if err != nil {
		return err
	}
	if res.StatusCode >= 300 {
		return fmt.Errorf("oidc discovery HTTP %d: %s", res.StatusCode, truncate(string(body), 200))
	}
	var doc discoveryDoc
	if err := json.Unmarshal(body, &doc); err != nil {
		return fmt.Errorf("oidc discovery json: %w", err)
	}
	if c.AuthorizeURL == "" {
		c.AuthorizeURL = doc.AuthorizationEndpoint
	}
	if c.TokenURL == "" {
		c.TokenURL = doc.TokenEndpoint
	}
	if c.UserInfoURL == "" {
		c.UserInfoURL = doc.UserinfoEndpoint
	}
	if c.AuthorizeURL == "" || c.TokenURL == "" || c.UserInfoURL == "" {
		return fmt.Errorf("oidc: incomplete discovery endpoints")
	}
	return nil
}

// ScopesCSV returns space-separated scopes (default openid profile email).
func (c Config) ScopesCSV() string {
	s := strings.TrimSpace(c.Scopes)
	if s == "" {
		return "openid profile email"
	}
	return s
}

// RandomURLString returns a URL-safe random string of n bytes (base64url, no pad).
func RandomURLString(n int) (string, error) {
	b := make([]byte, n)
	if _, err := rand.Read(b); err != nil {
		return "", err
	}
	return base64.RawURLEncoding.EncodeToString(b), nil
}

// PKCEChallengeS256 returns code_challenge for a verifier.
func PKCEChallengeS256(verifier string) string {
	sum := sha256.Sum256([]byte(verifier))
	return base64.RawURLEncoding.EncodeToString(sum[:])
}

// AuthorizeRedirect builds the browser redirect to the IdP authorize endpoint.
// redirectURI must be the absolute URI used for this login (same value later for token exchange).
func (c Config) AuthorizeRedirect(state, codeChallenge, redirectURI string) (string, error) {
	if c.AuthorizeURL == "" {
		return "", fmt.Errorf("oidc: authorize URL not resolved")
	}
	redirectURI = strings.TrimSpace(redirectURI)
	if redirectURI == "" {
		return "", fmt.Errorf("oidc: empty redirect_uri")
	}
	q := url.Values{}
	q.Set("response_type", "code")
	q.Set("client_id", c.ClientID)
	q.Set("redirect_uri", redirectURI)
	q.Set("scope", c.ScopesCSV())
	q.Set("state", state)
	if codeChallenge != "" {
		q.Set("code_challenge", codeChallenge)
		q.Set("code_challenge_method", "S256")
	}
	sep := "?"
	if strings.Contains(c.AuthorizeURL, "?") {
		sep = "&"
	}
	return c.AuthorizeURL + sep + q.Encode(), nil
}

// TokenResponse is the OAuth token endpoint JSON.
type TokenResponse struct {
	AccessToken  string `json:"access_token"`
	TokenType    string `json:"token_type"`
	ExpiresIn    int    `json:"expires_in"`
	RefreshToken string `json:"refresh_token"`
	IDToken      string `json:"id_token"`
	Scope        string `json:"scope"`
	Error        string `json:"error"`
	ErrorDesc    string `json:"error_description"`
}

// ExchangeCode swaps authorization code for tokens (client_secret_post).
// redirectURI must match the absolute URI sent in AuthorizeRedirect.
func (c Config) ExchangeCode(client *http.Client, code, codeVerifier, redirectURI string) (*TokenResponse, error) {
	if c.TokenURL == "" {
		return nil, fmt.Errorf("oidc: token URL not resolved")
	}
	redirectURI = strings.TrimSpace(redirectURI)
	if redirectURI == "" {
		return nil, fmt.Errorf("oidc: empty redirect_uri")
	}
	if client == nil {
		client = &http.Client{Timeout: 15 * time.Second}
	}
	form := url.Values{}
	form.Set("grant_type", "authorization_code")
	form.Set("code", code)
	form.Set("redirect_uri", redirectURI)
	form.Set("client_id", c.ClientID)
	form.Set("client_secret", c.ClientSecret)
	if codeVerifier != "" {
		form.Set("code_verifier", codeVerifier)
	}
	req, err := http.NewRequest(http.MethodPost, c.TokenURL, strings.NewReader(form.Encode()))
	if err != nil {
		return nil, err
	}
	req.Header.Set("Content-Type", "application/x-www-form-urlencoded")
	req.Header.Set("Accept", "application/json")
	res, err := client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("oidc token: %w", err)
	}
	defer res.Body.Close()
	body, err := io.ReadAll(io.LimitReader(res.Body, 1<<20))
	if err != nil {
		return nil, err
	}
	var tok TokenResponse
	if err := json.Unmarshal(body, &tok); err != nil {
		return nil, fmt.Errorf("oidc token json: %w (%s)", err, truncate(string(body), 200))
	}
	if tok.Error != "" {
		return nil, fmt.Errorf("oidc token: %s (%s)", tok.Error, tok.ErrorDesc)
	}
	if tok.AccessToken == "" {
		return nil, fmt.Errorf("oidc token: empty access_token (HTTP %d)", res.StatusCode)
	}
	return &tok, nil
}

// UserInfo is the subset we need from /oauth2/userinfo (CFLMY + standard OIDC).
type UserInfo struct {
	Sub               string `json:"sub"`
	Email             string `json:"email"`
	PreferredUsername string `json:"preferred_username"`
	Name              string `json:"name"`
	Picture           string `json:"picture"`
	IsAdmin           bool   `json:"is_admin"`
	AdminRole         string `json:"admin_role"`
}

// FetchUserInfo calls the userinfo endpoint with a bearer access token.
func (c Config) FetchUserInfo(client *http.Client, accessToken string) (*UserInfo, error) {
	if c.UserInfoURL == "" {
		return nil, fmt.Errorf("oidc: userinfo URL not resolved")
	}
	if client == nil {
		client = &http.Client{Timeout: 15 * time.Second}
	}
	req, err := http.NewRequest(http.MethodGet, c.UserInfoURL, nil)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Authorization", "Bearer "+accessToken)
	req.Header.Set("Accept", "application/json")
	res, err := client.Do(req)
	if err != nil {
		return nil, fmt.Errorf("oidc userinfo: %w", err)
	}
	defer res.Body.Close()
	body, err := io.ReadAll(io.LimitReader(res.Body, 1<<20))
	if err != nil {
		return nil, err
	}
	if res.StatusCode >= 300 {
		return nil, fmt.Errorf("oidc userinfo HTTP %d: %s", res.StatusCode, truncate(string(body), 200))
	}
	var info UserInfo
	if err := json.Unmarshal(body, &info); err != nil {
		return nil, fmt.Errorf("oidc userinfo json: %w", err)
	}
	if info.Sub == "" {
		return nil, fmt.Errorf("oidc userinfo: missing sub")
	}
	return &info, nil
}

// LocalUsername picks a stable display username for sessions / RBAC.
func (u *UserInfo) LocalUsername() string {
	if u == nil {
		return ""
	}
	if s := strings.TrimSpace(u.PreferredUsername); s != "" {
		return s
	}
	if s := strings.TrimSpace(u.Email); s != "" {
		if i := strings.IndexByte(s, '@'); i > 0 {
			return s[:i]
		}
		return s
	}
	if s := strings.TrimSpace(u.Name); s != "" {
		return s
	}
	sub := strings.TrimSpace(u.Sub)
	if len(sub) > 32 {
		return sub[:32]
	}
	return sub
}

// IsIdPAdmin reports whether CFLMY IdP marks this user as an administrator.
func (u *UserInfo) IsIdPAdmin() bool {
	if u == nil {
		return false
	}
	if u.IsAdmin {
		return true
	}
	return strings.TrimSpace(u.AdminRole) != ""
}

// LocalRole maps IdP admin → site admin; otherwise defaultRole (member).
func (u *UserInfo) LocalRole(defaultRole string) string {
	if defaultRole == "" {
		defaultRole = "member"
	}
	if u.IsIdPAdmin() {
		return "admin"
	}
	return defaultRole
}

func truncate(s string, n int) string {
	if len(s) <= n {
		return s
	}
	return s[:n] + "…"
}

// ParseConfig reads oidc bag from app / auth options (EN + ZH keys).
func ParseConfig(m map[string]any) Config {
	if m == nil {
		return Config{}
	}
	if nested, ok := m["oidc"].(map[string]any); ok {
		m = nested
	}
	c := Config{
		Issuer:          firstStr(m, "issuer", "发行方", "issuer_url"),
		ClientID:        firstStr(m, "client_id", "客户端编号", "客户端id", "clientId"),
		ClientSecret:    firstStr(m, "client_secret", "客户端密钥", "clientSecret"),
		RedirectURI:     firstStr(m, "redirect_uri", "回调", "回调地址", "redirect"),
		RedirectOrigins: firstStr(m, "redirect_origins", "回调来源", "redirect_origin"),
		Scopes:          firstStr(m, "scopes", "范围", "scope"),
		CallbackPath:    firstStr(m, "callback_path", "回调路径"),
		AuthorizeURL:    firstStr(m, "authorize_url", "授权地址"),
		TokenURL:        firstStr(m, "token_url", "令牌地址"),
		UserInfoURL:     firstStr(m, "userinfo_url", "用户信息地址"),
	}
	return c
}

func firstStr(m map[string]any, keys ...string) string {
	for _, k := range keys {
		if v, ok := m[k]; ok {
			switch t := v.(type) {
			case string:
				if s := strings.TrimSpace(t); s != "" && !strings.EqualFold(s, "none") {
					return s
				}
			}
		}
	}
	return ""
}
