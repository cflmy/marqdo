package middleware_test

import (
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/middleware"
)

func TestConfigureCORSOrigins(t *testing.T) {
	cors := map[string]any{
		"允许来源": []any{"https://a.example", "https://b.example"},
		"方法":   []any{"GET,POST", "GET"},
		"头":    []any{"Content-Type,Authorization", "Content-Type"},
		"暴露头":  []any{"X-Total", ""},
		"凭证":   []any{"true", ""},
	}
	sec := map[string]any{
		"头": []any{"X-Frame-Options", "X-Content-Type-Options", "Content-Security-Policy"},
		"值": []any{"DENY", "nosniff", "default-src 'self'"},
	}
	api := map[string]any{
		"路径": []any{"/api/posts", "/api/publish"},
		"方法": []any{"GET", "POST"},
		"表":  []any{"articles", "articles"},
		"条件": []any{"", ""},
		"排序": []any{"-created_at", ""},
		"上限": []any{"20", "10"},
	}

	a := app.New(map[string]any{"page": map[string]any{"title": "mw"}})
	out, err := middleware.Configure(a, map[string]any{
		"cors":       cors,
		"security":   sec,
		"compress":   true,
		"body_limit": float64(1048576),
		"json":       api,
	})
	if err != nil {
		t.Fatal(err)
	}
	mw, _ := out["middleware"].(map[string]any)
	if mw == nil {
		t.Fatal("missing middleware")
	}
	cobj, _ := mw["cors"].(map[string]any)
	origins, _ := cobj["allow_origins"].([]any)
	if len(origins) < 1 || origins[0] != "https://a.example" {
		t.Fatalf("allow_origins=%v", origins)
	}
	methods, _ := cobj["methods"].([]any)
	if len(methods) < 2 || methods[1] != "POST" {
		t.Fatalf("methods=%v", methods)
	}
	expose, _ := cobj["expose_headers"].([]any)
	if len(expose) < 1 || expose[0] != "X-Total" {
		t.Fatalf("expose=%v", expose)
	}
	if cred, _ := cobj["credentials"].(bool); !cred {
		t.Fatal("credentials want true")
	}

	secmap, _ := mw["security"].(map[string]any)
	if secmap["X-Frame-Options"] != "DENY" {
		t.Fatalf("security frame=%v", secmap["X-Frame-Options"])
	}
	if mw["compress"] != true {
		t.Fatalf("compress=%v", mw["compress"])
	}
	if mw["body_limit"] != float64(1048576) {
		t.Fatalf("body_limit=%v", mw["body_limit"])
	}
	jmap, _ := mw["json_routes"].(map[string]any)
	posts, _ := jmap["api/posts"].(map[string]any)
	if posts["method"] != "GET" || posts["table"] != "articles" {
		t.Fatalf("posts=%v", posts)
	}
	if posts["order"] != "-created_at" || posts["limit"] != float64(20) {
		t.Fatalf("posts order/limit=%v", posts)
	}
	pub, _ := jmap["api/publish"].(map[string]any)
	if pub["method"] != "POST" {
		t.Fatalf("publish=%v", pub)
	}
}

func TestParseReadsBag(t *testing.T) {
	a := map[string]any{
		"middleware": map[string]any{
			"cors": map[string]any{
				"allow_origins": []any{"https://a.example"},
				"methods":       []any{"GET"},
				"credentials":   true,
			},
			"compress":   true,
			"body_limit": float64(1000),
			"json_routes": map[string]any{
				"api/posts": map[string]any{"method": "GET", "table": "articles", "limit": float64(10)},
			},
		},
	}
	cfg := middleware.Parse(a)
	if cfg.CORS == nil || len(cfg.CORS.AllowOrigins) != 1 {
		t.Fatalf("cors=%v", cfg.CORS)
	}
	if !cfg.Compress || cfg.BodyLimit == nil || *cfg.BodyLimit != 1000 {
		t.Fatalf("cfg=%+v", cfg)
	}
	if len(cfg.JSONRoutes) != 1 || cfg.JSONRoutes[0].Path != "/api/posts" {
		t.Fatalf("json routes=%v", cfg.JSONRoutes)
	}
}
