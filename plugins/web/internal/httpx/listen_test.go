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
	"github.com/marqdo/marqdo/plugins/web/internal/middleware"
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
	if cc := rec.Header().Get("Cache-Control"); !strings.Contains(cc, "max-age=") {
		t.Fatalf("static cache-control=%q", cc)
	}
	b, _ := io.ReadAll(rec.Result().Body)
	if string(b) != "hello-static" {
		// httptest body is already in rec.Body
		if rec.Body.String() != "hello-static" {
			t.Fatalf("static body=%q", rec.Body.String())
		}
	}
}

func TestNewHandlerIconsRedirectSitemap404(t *testing.T) {
	dir := t.TempDir()
	iconPath := filepath.Join(dir, "favicon.png")
	if err := os.WriteFile(iconPath, []byte("PNG"), 0o644); err != nil {
		t.Fatal(err)
	}

	page := map[string]any{"title": "W7", "intro": "<p>home</p>"}
	nf := map[string]any{"title": "Not Found Page", "intro": "<p>missing</p>"}
	a := app.New(map[string]any{"page": page})
	a, err := app.Icons(a, []any{
		map[string]any{"path": iconPath, "rel": "icon", "type": "image/png", "url": "/favicon.ico"},
	})
	if err != nil {
		t.Fatal(err)
	}
	a, err = app.Redirect(a, "/legacy", "/", true)
	if err != nil {
		t.Fatal(err)
	}
	a, err = app.ErrorPage(a, 404, nf)
	if err != nil {
		t.Fatal(err)
	}
	items := []any{map[string]any{"loc": "/"}, map[string]any{"loc": "/about"}}
	a, err = app.Sitemap(a, "/sitemap.xml", "http://example.com", "", "path", 100, items)
	if err != nil {
		t.Fatal(err)
	}
	a, err = app.Robots(a, "", "http://example.com/sitemap.xml")
	if err != nil {
		t.Fatal(err)
	}
	a, err = middleware.Configure(a, map[string]any{"cache_control": "public, max-age=120"})
	if err != nil {
		t.Fatal(err)
	}

	h, err := httpx.NewHandler(a, dir)
	if err != nil {
		t.Fatal(err)
	}

	rec := httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/favicon.ico", nil))
	if rec.Code != 200 || !strings.Contains(rec.Header().Get("Content-Type"), "image/") {
		t.Fatalf("favicon code=%d ct=%q", rec.Code, rec.Header().Get("Content-Type"))
	}

	rec = httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/legacy", nil))
	if rec.Code != http.StatusMovedPermanently {
		t.Fatalf("redirect code=%d", rec.Code)
	}

	rec = httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/sitemap.xml", nil))
	if rec.Code != 200 || !strings.Contains(rec.Body.String(), "<urlset") {
		t.Fatalf("sitemap code=%d body=%q", rec.Code, rec.Body.String())
	}

	rec = httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/robots.txt", nil))
	if rec.Code != 200 || !strings.Contains(rec.Body.String(), "Sitemap:") {
		t.Fatalf("robots code=%d body=%q", rec.Code, rec.Body.String())
	}

	rec = httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/missing", nil))
	if rec.Code != 404 || !strings.Contains(rec.Body.String(), "Not Found Page") {
		t.Fatalf("404 code=%d body=%q", rec.Code, rec.Body.String())
	}

	rec = httptest.NewRecorder()
	h.ServeHTTP(rec, httptest.NewRequest(http.MethodGet, "/", nil))
	if !strings.Contains(rec.Header().Get("Cache-Control"), "max-age=120") {
		t.Fatalf("cache-control=%q", rec.Header().Get("Cache-Control"))
	}
	if !strings.Contains(rec.Body.String(), `rel="icon"`) {
		t.Fatalf("home missing icon link: %s", rec.Body.String())
	}
}
