package httpx_test

import (
	"io"
	"net/http"
	"net/http/httptest"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/httpx"
)

func TestNewHandlerHomeRoutesStatic(t *testing.T) {
	dir := t.TempDir()
	pub := filepath.Join(dir, "public")
	if err := os.MkdirAll(pub, 0o755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(pub, "hi.txt"), []byte("hello-static"), 0o644); err != nil {
		t.Fatal(err)
	}

	page := map[string]any{"title": "Home", "intro": "<h1>Home</h1>"}
	a := app.New(map[string]any{"page": page})
	a, err := app.Static(a, pub, "/static")
	if err != nil {
		t.Fatal(err)
	}
	about := map[string]any{"title": "About", "intro": "<h1>About</h1>", "_route": "/about"}
	a, err = app.Route(a, "/about", about)
	if err != nil {
		t.Fatal(err)
	}

	h, err := httpx.NewHandler(a, "")
	if err != nil {
		t.Fatal(err)
	}

	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/", nil))
	if rec.Code != 200 || !strings.Contains(rec.Body.String(), "Home") {
		t.Fatalf("home code=%d body=%q", rec.Code, rec.Body.String())
	}

	rec = httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/about", nil))
	if rec.Code != 200 || !strings.Contains(rec.Body.String(), "About") {
		t.Fatalf("about code=%d body=%q", rec.Code, rec.Body.String())
	}

	rec = httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/static/hi.txt", nil))
	if rec.Code != 200 {
		t.Fatalf("static code=%d", rec.Code)
	}
	b, _ := io.ReadAll(rec.Result().Body)
	if string(b) != "hello-static" {
		// httptest body is already in rec.Body
		if rec.Body.String() != "hello-static" {
			t.Fatalf("static body=%q", rec.Body.String())
		}
	}
}
