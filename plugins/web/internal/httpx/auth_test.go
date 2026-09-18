package httpx

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/auth"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

func TestGateMatchesAndRBAC(t *testing.T) {
	session.Configure(session.Config{TTLSec: 3600})
	session.Reset(3600)

	gates := []gate{{
		path:      "/write",
		roles:     []string{"admin"},
		matchMode: gateMatchPrefix,
		onDeny:    onDenyRedirect,
		exclude:   []string{"/admin/login"},
	}}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /write", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusOK)
	})
	handler := withRBAC(mux, gates, "/admin/login")

	req := httptest.NewRequest(http.MethodGet, "/write", nil)
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)
	if rec.Code != http.StatusSeeOther {
		t.Fatalf("anonymous /write: got %d", rec.Code)
	}
	loc := rec.Header().Get("Location")
	if loc != "/admin/login?next=/write" {
		t.Fatalf("location=%q", loc)
	}

	res := auth.Login("admin", "secret", []any{
		map[string]any{"username": "admin", "password": "secret", "role": "admin"},
	}, 3600)
	sid, _ := res["session_id"].(string)
	req2 := httptest.NewRequest(http.MethodGet, "/write", nil)
	req2.Header.Set("Cookie", "marqdo_sid="+sid)
	rec2 := httptest.NewRecorder()
	handler.ServeHTTP(rec2, req2)
	if rec2.Code != http.StatusOK {
		t.Fatalf("authed /write: got %d", rec2.Code)
	}
}

func TestWithNavAuth(t *testing.T) {
	session.Configure(session.Config{TTLSec: 3600})
	session.Reset(3600)
	page := map[string]any{}
	withNavAuth(page, "")
	if page["_logged_in"] != false {
		t.Fatalf("expected logged out")
	}
	res := auth.Login("u", "p", []any{
		map[string]any{"username": "u", "password": "p"},
	}, 3600)
	sid, _ := res["session_id"].(string)
	withNavAuth(page, "marqdo_sid="+sid)
	if page["_logged_in"] != true {
		t.Fatalf("expected logged in")
	}
	if page["_nav_user"] != "u" {
		t.Fatalf("nav user=%v", page["_nav_user"])
	}
}

func TestLogoutPathExcludedFromGate(t *testing.T) {
	gates := []gate{{
		path:        "/admin",
		permissions: []string{"desk:access"},
		matchMode:   gateMatchPrefix,
		onDeny:      onDenyRedirect,
		exclude:     []string{"/admin/login"},
	}}
	mux := http.NewServeMux()
	mux.HandleFunc("GET /admin/logout", func(w http.ResponseWriter, r *http.Request) {
		w.WriteHeader(http.StatusNoContent)
	})
	handler := withRBAC(mux, gates, "/login")

	req := httptest.NewRequest(http.MethodGet, "/admin/logout", nil)
	rec := httptest.NewRecorder()
	handler.ServeHTTP(rec, req)
	if rec.Code != http.StatusNoContent {
		t.Fatalf("visitor /admin/logout should reach handler, got %d loc=%s", rec.Code, rec.Header().Get("Location"))
	}
}
