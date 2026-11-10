package httpx

import (
	"encoding/json"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/form"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

func TestInspectionUsesActualRegistrationsAfterResourceBindingsWithoutExecutingThem(t *testing.T) {
	filename := filepath.Join(t.TempDir(), "must-not-create.db")
	bag := app.New(map[string]any{"page": map[string]any{"title": "Home"},
		"db": map[string]any{"url": "sqlite:" + filename}, "admin": true})
	bag["artifact_declared"] = []any{map[string]any{"method": "GET", "path": "/", "kind": "web", "file": "index.mq.md"}}
	// A cached observation is deliberately stale and must never be returned.
	bag["inspect"] = map[string]any{"declared": []any{map[string]any{"path": "/stale"}}}
	var err error
	bag, err = app.Route(bag, "/post/{id}", map[string]any{"title": "Post"})
	if err != nil {
		t.Fatal(err)
	}
	before, err := InspectApp(bag)
	if err != nil {
		t.Fatal(err)
	}
	bag, err = app.Auth(bag, []any{map[string]any{"username": "fixture", "password": "never-return-this-secret"}}, map[string]any{"login_path": "/members/login"})
	if err != nil {
		t.Fatal(err)
	}
	bag, err = app.MountForm(bag, "article", form.New("articles", "insert", ""))
	if err != nil {
		t.Fatal(err)
	}
	bag["ws_routes"] = map[string]any{"/live/{id}": map[string]any{"mode": "echo"}}
	bag["middleware"] = map[string]any{"json_routes": map[string]any{"api/items": map[string]any{"table": "items", "method": "GET"}}}
	bag["artifact_routes"] = map[string]any{"GET /api/ping": map[string]any{"method": "GET", "path": "/api/ping", "file": "/never-run.mq.md"}}
	bag["static_dir"] = "."
	bag["static_mount"] = "/assets"
	session.Configure(session.Config{TTLSec: 3600})
	session.Reset(3600)
	sid := session.NewID(3600)
	session.SetValue(sid, "inspection_marker", "retained")
	report, err := InspectApp(bag)
	if err != nil {
		t.Fatal(err)
	}
	if report["scope"] != "registered" {
		t.Fatal(report)
	}
	type route struct{ method, path string }
	observed := map[route]string{}
	for _, group := range []string{"declared", "system"} {
		for _, row := range report[group].([]map[string]any) {
			key := route{row["method"].(string), row["path"].(string)}
			if _, seen := observed[key]; seen {
				t.Fatalf("duplicate registered route: %#v", key)
			}
			observed[key] = group
		}
	}
	for _, key := range []route{{"GET", "/"}, {"GET", "/post/{id}"}, {"GET", "/live/{id}"}, {"GET", "/api/items"}, {"GET", "/api/ping"}} {
		if observed[key] != "declared" {
			t.Fatalf("missing current Resource declaration %#v: %#v", key, report)
		}
	}
	for _, key := range []route{{"POST", "/_md"}, {"GET", "/_form/{id}"}, {"POST", "/_form/{id}"},
		{"GET", "/post/{id}/_part/{part}"}, {"GET", "/members/login"}, {"POST", "/members/login"},
		{"GET", "/admin/login"}, {"GET", "/admin"}, {"GET", "/admin/"}, {"GET|HEAD", "/assets/"}} {
		if observed[key] != "system" {
			t.Fatalf("missing actual system route %#v: %#v", key, report)
		}
	}
	encoded, _ := json.Marshal(report)
	if strings.Contains(string(encoded), "never-return-this-secret") || strings.Contains(string(encoded), "/stale") {
		t.Fatalf("inspection leaked unrelated app state: %s", encoded)
	}
	if value, ok := session.GetValue(sid, "inspection_marker"); !ok || value != "retained" {
		t.Fatal("inspection reset runtime sessions")
	}
	if _, err := os.Stat(filename); !os.IsNotExist(err) {
		t.Fatalf("inspection opened a database: %v", err)
	}
	prior, _ := json.Marshal(before)
	if strings.Contains(string(prior), "/live/") {
		t.Fatal("inspection mutated an earlier returned observation")
	}
}

func TestInspectionRejectsActualRouteConflictsWithoutStartingTheServer(t *testing.T) {
	bag := app.New(map[string]any{"page": map[string]any{}})
	bag["routes"] = map[string]any{"/_md": map[string]any{}}
	bag["artifact_routes"] = map[string]any{"POST /_md": map[string]any{"method": "POST", "path": "/_md", "file": "never-run.mq.md"}}
	if _, err := InspectApp(bag); err == nil || !strings.Contains(err.Error(), "invalid route registration") {
		t.Fatalf("conflicting routes were advertised as usable: %v", err)
	}
}
