package httpx

import (
	"net/http/httptest"
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

func TestGoMuxPattern(t *testing.T) {
	if got := goMuxPattern("/_media/{*key}"); got != "/_media/{key...}" {
		t.Fatalf("got %q", got)
	}
	if got := goMuxPattern("/post/{slug}"); got != "/post/{slug}" {
		t.Fatalf("got %q", got)
	}
}

func TestPathParamNamesIncludesID(t *testing.T) {
	got := pathParamNames("/desk/posts/{id}")
	if len(got) != 1 || got[0] != "id" {
		t.Fatalf("got %#v", got)
	}
	got = pathParamNames("/post/{slug}")
	if len(got) != 1 || got[0] != "slug" {
		t.Fatalf("got %#v", got)
	}
}

func TestResolveSessionReplacesStaleCookie(t *testing.T) {
	session.Configure(session.Config{TTLSec: 3600})
	session.Reset(3600)

	req := httptest.NewRequest("GET", "/login", nil)
	req.Header.Set("Cookie", "marqdo_sid="+strings.Repeat("ab", 32))
	sid, csrf, setCookie := resolveSession(req)
	if sid == "" || csrf == "" {
		t.Fatalf("expected fresh sid+csrf, got sid=%q csrf=%q", sid, csrf)
	}
	if setCookie == nil || !strings.Contains(*setCookie, "marqdo_sid=") {
		t.Fatalf("expected Set-Cookie for replacement session, got %#v", setCookie)
	}
	if !session.ValidateCSRF(sid, csrf) {
		t.Fatal("csrf should validate for new session")
	}

	// Live session should keep the same id without re-issuing cookie.
	req2 := httptest.NewRequest("GET", "/login", nil)
	req2.Header.Set("Cookie", "marqdo_sid="+sid)
	sid2, csrf2, setCookie2 := resolveSession(req2)
	if sid2 != sid || csrf2 != csrf {
		t.Fatalf("live session should be stable: %q/%q vs %q/%q", sid, csrf, sid2, csrf2)
	}
	if setCookie2 != nil {
		t.Fatalf("live session should not Set-Cookie again")
	}
}

