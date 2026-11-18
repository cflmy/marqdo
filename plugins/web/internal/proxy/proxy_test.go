package proxy_test

import (
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/middleware"
	"github.com/marqdo/marqdo/plugins/web/internal/proxy"
)

func TestRoutesFromTable(t *testing.T) {
	tbl := map[string]any{
		"path":             []any{"/proxy/chat"},
		"upstream":         []any{"$OPENAI_BASE_URL"},
		"stream":           []any{true},
		"strip_prefix":     []any{"/proxy"},
		"methods":          []any{"POST"},
		"headers_from_env": []any{"OPENAI_API_KEY=Authorization"},
		"timeout_ms":       []any{60000},
	}
	routes := proxy.RoutesFromTable(tbl)
	chat, ok := routes["proxy/chat"].(map[string]any)
	if !ok {
		t.Fatalf("routes=%v", routes)
	}
	if chat["upstream"] != "$OPENAI_BASE_URL" {
		t.Fatalf("upstream=%v", chat["upstream"])
	}
	if chat["stream"] != true {
		t.Fatalf("stream=%v", chat["stream"])
	}
	if chat["strip_prefix"] != "/proxy" {
		t.Fatalf("strip_prefix=%v", chat["strip_prefix"])
	}
	if chat["headers_from_env"] != "OPENAI_API_KEY=Authorization" {
		t.Fatalf("headers_from_env=%v", chat["headers_from_env"])
	}
	methods, _ := chat["methods"].([]any)
	if len(methods) != 1 || methods[0] != "POST" {
		t.Fatalf("methods=%v", methods)
	}
	if chat["timeout_ms"] != float64(60000) {
		t.Fatalf("timeout_ms=%v", chat["timeout_ms"])
	}
}

func TestRegisterProxyRoute(t *testing.T) {
	a := app.New(map[string]any{"page": map[string]any{"title": "p"}})
	out, err := proxy.Register(a, proxy.Options{
		Path:           "/v1/stream",
		Upstream:       "https://example.com/v1/chat",
		Stream:         true,
		HeadersFromEnv: "TOKEN=X-Token",
	})
	if err != nil {
		t.Fatal(err)
	}
	routes := out["proxy_routes"].(map[string]any)
	entry, ok := routes["v1/stream"].(map[string]any)
	if !ok {
		t.Fatalf("routes=%v", routes)
	}
	if entry["upstream"] != "https://example.com/v1/chat" {
		t.Fatalf("upstream=%v", entry["upstream"])
	}
	if entry["headers_from_env"] != "TOKEN=X-Token" {
		t.Fatalf("headers=%v", entry["headers_from_env"])
	}
}

func TestConfigureMergesProxyTable(t *testing.T) {
	a := app.New(nil)
	proxyTbl := map[string]any{
		"path":     []any{"/proxy/chat"},
		"upstream": []any{"$OPENAI_BASE_URL"},
		"stream":   []any{true},
	}
	out, err := middleware.Configure(a, map[string]any{"proxy": proxyTbl})
	if err != nil {
		t.Fatal(err)
	}
	routes := out["proxy_routes"].(map[string]any)
	if routes["proxy/chat"] == nil {
		t.Fatalf("proxy_routes=%v", routes)
	}
}

func TestRegisterRejectsEmptyUpstream(t *testing.T) {
	_, err := proxy.Register(app.New(nil), proxy.Options{Path: "/x", Upstream: "  "})
	if err == nil {
		t.Fatal("want error for empty upstream")
	}
}
