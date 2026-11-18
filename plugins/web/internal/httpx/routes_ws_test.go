package httpx

import (
	"net/http"
	"net/http/httptest"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/ws"
)

func TestHandleWSRejectsUnauthenticatedHandshake(t *testing.T) {
	st := &state{}
	r := httptest.NewRequest("GET", "http://app.example/live", nil)
	w := httptest.NewRecorder()
	st.handleWS(w, r, "/live", ws.RouteSpec{Mode: ws.ModeBroadcast, RequireAuth: true})
	if w.Code != http.StatusUnauthorized {
		t.Fatalf("unauthenticated ws handshake code=%d, want 401", w.Code)
	}
}

func TestHandleWSAllowsPublicRouteThroughAuthGate(t *testing.T) {
	st := &state{}
	r := httptest.NewRequest("GET", "http://app.example/live", nil)
	w := httptest.NewRecorder()
	// Public route: must not 401. Upgrade then fails on the recorder (no
	// Hijacker), which is fine — only the auth gate is under test.
	st.handleWS(w, r, "/live", ws.RouteSpec{Mode: ws.ModeBroadcast, RequireAuth: false})
	if w.Code == http.StatusUnauthorized {
		t.Fatalf("public ws route unexpectedly requires authentication")
	}
}

func TestSameOriginWebSocketHandshake(t *testing.T) {
	cases := []struct {
		name   string
		origin string
		want   bool
	}{
		{name: "non-browser client", want: true},
		{name: "same origin", origin: "https://app.example", want: true},
		{name: "different origin", origin: "https://evil.example", want: false},
		{name: "malformed origin", origin: "://bad", want: false},
	}
	for _, tc := range cases {
		t.Run(tc.name, func(t *testing.T) {
			r := httptest.NewRequest("GET", "http://app.example/live", nil)
			if tc.origin != "" {
				r.Header.Set("Origin", tc.origin)
			}
			if got := sameOrigin(r); got != tc.want {
				t.Fatalf("sameOrigin(%q)=%v, want %v", tc.origin, got, tc.want)
			}
		})
	}
}
