package tenant_test

import (
	"net/http/httptest"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/tenant"
)

func TestResolvePathQuery(t *testing.T) {
	cfg := tenant.Config{Mode: "path", Param: "t", Column: "tenant_id"}
	r := httptest.NewRequest("GET", "/api/items?t=acme", nil)
	if got := tenant.Resolve(r, cfg); got != "acme" {
		t.Fatalf("got %q", got)
	}
}

func TestResolvePathPrefix(t *testing.T) {
	cfg := tenant.Config{Mode: "path", Param: "t"}
	r := httptest.NewRequest("GET", "/t/beta/items", nil)
	if got := tenant.Resolve(r, cfg); got != "beta" {
		t.Fatalf("got %q", got)
	}
}

func TestResolveHeader(t *testing.T) {
	cfg := tenant.Config{Mode: "header", Param: "X-Tenant-Id"}
	r := httptest.NewRequest("GET", "/", nil)
	r.Header.Set("X-Tenant-Id", "Gamma")
	if got := tenant.Resolve(r, cfg); got != "gamma" {
		t.Fatalf("got %q", got)
	}
}

func TestMergeWhereAndStamp(t *testing.T) {
	w := tenant.MergeWhere(nil, "tenant_id", "acme")
	arr, ok := w.([]any)
	if !ok || len(arr) != 1 {
		t.Fatalf("where=%v", w)
	}
	row := tenant.StampRow(map[string]any{"title": "x"}, "tenant_id", "acme")
	if row["tenant_id"] != "acme" {
		t.Fatalf("row=%v", row)
	}
}

func TestConfigure(t *testing.T) {
	out, err := tenant.Configure(map[string]any{}, map[string]any{
		"mode": "path", "param": "org", "default_scope": true,
	})
	if err != nil {
		t.Fatal(err)
	}
	cfg := tenant.FromBag(out)
	if cfg.Mode != "path" || cfg.Param != "org" || !cfg.DefaultScope {
		t.Fatalf("%+v", cfg)
	}
}
