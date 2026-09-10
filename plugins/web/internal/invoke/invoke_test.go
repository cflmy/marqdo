package invoke_test

import (
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/invoke"
	"github.com/marqdo/marqdo/plugins/web/internal/middleware"
)

func TestRoutesFromTable(t *testing.T) {
	tbl := map[string]any{
		"path":   []any{"/api/echo"},
		"method": []any{"POST"},
		"fn":     []any{"demo.echo"},
		"body":   []any{"json"},
		"return": []any{"json"},
	}
	routes := invoke.RoutesFromTable(tbl)
	echo, ok := routes["api/echo"].(map[string]any)
	if !ok {
		t.Fatalf("routes=%v", routes)
	}
	if echo["fn"] != "demo.echo" {
		t.Fatalf("fn=%v", echo["fn"])
	}
	if echo["method"] != "POST" {
		t.Fatalf("method=%v", echo["method"])
	}
}

func TestRegisterInvokeRoute(t *testing.T) {
	a := app.New(nil)
	out, err := invoke.Register(a, invoke.Options{
		Path:   "/api/ping",
		Method: "GET",
		Fn:     "demo.ping",
		Body:   "query",
	})
	if err != nil {
		t.Fatal(err)
	}
	routes := out["invoke_routes"].(map[string]any)
	ping, ok := routes["api/ping"].(map[string]any)
	if !ok {
		t.Fatalf("routes=%v", routes)
	}
	if ping["method"] != "GET" {
		t.Fatalf("method=%v", ping["method"])
	}
	if ping["body"] != "query" {
		t.Fatalf("body=%v", ping["body"])
	}
}

func TestRegisterRejectsInvalidFn(t *testing.T) {
	_, err := invoke.Register(app.New(nil), invoke.Options{
		Path: "/api/x",
		Fn:   "not-valid",
	})
	if err == nil || !strings.Contains(err.Error(), "lib.member") {
		t.Fatalf("want lib.member error, got %v", err)
	}
}

func TestConfigureMergesInvokeTable(t *testing.T) {
	a := app.New(nil)
	invokeTbl := map[string]any{
		"path":   []any{"/api/echo"},
		"method": []any{"POST"},
		"fn":     []any{"demo.echo"},
		"body":   []any{"json"},
		"return": []any{"json"},
	}
	out, err := middleware.Configure(a, map[string]any{"invoke": invokeTbl})
	if err != nil {
		t.Fatal(err)
	}
	routes := out["invoke_routes"].(map[string]any)
	if routes["api/echo"] == nil {
		t.Fatalf("invoke_routes=%v", routes)
	}
}
