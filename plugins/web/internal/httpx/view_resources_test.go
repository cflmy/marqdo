package httpx

import (
	"net/http"
	"net/http/httptest"
	"net/url"
	"path/filepath"
	"regexp"
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/app"
	"github.com/marqdo/marqdo/plugins/web/internal/artifact"
	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/form"
	"github.com/marqdo/marqdo/plugins/web/internal/session"
)

func resourceNode(resource map[string]any) map[string]any {
	return map[string]any{"type": "resource", "value": resource, "slot": "content"}
}

func TestViewResourcesObserveRequestsFormsAndNavigation(t *testing.T) {
	uri := "sqlite:" + filepath.Join(t.TempDir(), "view.db")
	fields := []any{
		map[string]any{"name": "id", "type": "integer", "nullable": false},
		map[string]any{"name": "title", "type": "text", "nullable": false},
		map[string]any{"name": "slug", "type": "text", "nullable": true},
		map[string]any{"name": "summary", "type": "text", "nullable": true},
		map[string]any{"name": "body", "type": "text", "nullable": true},
	}
	if _, err := db.Init(uri, "articles", fields); err != nil {
		t.Fatal(err)
	}
	if _, err := db.Insert(uri, "articles", map[string]any{
		"title": "Original <title>", "slug": "first", "summary": "Short summary", "body": "Full article body",
	}); err != nil {
		t.Fatal(err)
	}
	query := map[string]any{"_type": "data_query", "source": "articles", "limit": float64(50), "link_prefix": "/post/"}
	navigation := map[string]any{"_type": "view_navigation", "region": "nav", "items": []any{
		map[string]any{"label": "Guest login", "href": "/admin/login", "when": "guest"},
		map[string]any{"label": "Private desk", "href": "/admin", "when": "auth"},
	}}
	page := map[string]any{"title": "View resources", "artifact_nodes": []any{
		resourceNode(navigation), resourceNode(query),
		resourceNode(map[string]any{"_type": "form_ref", "id": "article"}),
	}}
	bag := app.New(map[string]any{"page": page, "db": map[string]any{"url": uri}})
	var err error
	bag, err = app.Auth(bag, []any{map[string]any{"username": "admin", "password": "secret", "role": "admin"}}, nil)
	if err != nil {
		t.Fatal(err)
	}
	selected := form.SetFields(form.New("articles", "insert", ""), fields[1:2])
	selected = form.SetRules(selected, []any{map[string]any{"field": "title", "rule": "required", "message": "Title required"}})
	selected = form.SetLabels(selected, "Publish", "Cancel", "/")
	bag, err = app.MountForm(bag, "article", selected)
	if err != nil {
		t.Fatal(err)
	}
	detailed := map[string]any{"_type": "data_query", "source": "articles", "detail": true,
		"where": map[string]any{"slug": "{slug}"}}
	bag, err = app.Route(bag, "/post/{slug}", map[string]any{"title": "Article", "artifact_nodes": []any{resourceNode(detailed)}})
	if err != nil {
		t.Fatal(err)
	}
	handler, err := NewHandler(bag, "")
	if err != nil {
		t.Fatal(err)
	}
	call := func(method, path, body, cookie string) *httptest.ResponseRecorder {
		request := httptest.NewRequest(method, path, strings.NewReader(body))
		if body != "" {
			request.Header.Set("Content-Type", "application/x-www-form-urlencoded")
		}
		if cookie != "" {
			request.Header.Set("Cookie", cookie)
		}
		recorder := httptest.NewRecorder()
		handler.ServeHTTP(recorder, request)
		return recorder
	}
	initial := call("GET", "/", "", "")
	if initial.Code != http.StatusOK {
		t.Fatalf("initial: %d %s", initial.Code, initial.Body.String())
	}
	text := initial.Body.String()
	if !strings.Contains(text, "Guest login") || strings.Contains(text, "Private desk") ||
		!strings.Contains(text, "Short summary") || strings.Contains(text, "Full article body") ||
		!strings.Contains(text, "Original &lt;title&gt;") {
		t.Fatal(text)
	}
	cookies := initial.Result().Cookies()
	if len(cookies) == 0 {
		t.Fatal("rendered form has no actual session cookie")
	}
	cookie := cookies[0].Name + "=" + cookies[0].Value
	match := regexp.MustCompile(`name="_csrf" value="([^"]+)"`).FindStringSubmatch(text)
	if len(match) != 2 {
		t.Fatal("resource form omitted its CSRF token")
	}
	token := match[1]
	bad := call("POST", "/_form/article", "title=Rejected&_csrf=wrong", cookie)
	if bad.Code != http.StatusForbidden {
		t.Fatalf("bad csrf accepted: %d %s", bad.Code, bad.Body.String())
	}
	invalid := call("POST", "/_form/article", url.Values{"_csrf": {token}, "title": {""}}.Encode(), cookie)
	if invalid.Code != http.StatusOK || !strings.Contains(invalid.Body.String(), "Title required") ||
		!strings.Contains(invalid.Body.String(), `name="_csrf" value="`+token+`"`) {
		t.Fatalf("validation retry: %d %s", invalid.Code, invalid.Body.String())
	}
	valid := call("POST", "/_form/article", url.Values{"_csrf": {token}, "title": {"Inserted after startup"}, "_mq_return": {"/new"}}.Encode(), cookie)
	if valid.Code != http.StatusSeeOther || valid.Header().Get("Location") != "/" {
		t.Fatalf("submit: %d %s", valid.Code, valid.Body.String())
	}
	current := call("GET", "/", "", cookie)
	if !strings.Contains(current.Body.String(), "Inserted after startup") {
		t.Fatal("query cached startup rows")
	}
	fragment := call("GET", "/_part/main", "", cookie)
	if fragment.Code != http.StatusOK || !strings.Contains(fragment.Body.String(), "Inserted after startup") ||
		!strings.Contains(fragment.Body.String(), `name="_csrf" value="`+token+`"`) ||
		strings.Contains(fragment.Body.String(), "<!DOCTYPE") {
		t.Fatalf("request fragment lost context: %d %s", fragment.Code, fragment.Body.String())
	}
	unknown := call("GET", "/_part/not-declared", "", cookie)
	if unknown.Code != http.StatusNotFound {
		t.Fatalf("unknown View slot: %d", unknown.Code)
	}

	detail := call("GET", "/post/first?slug=other", "", cookie)
	if detail.Code != http.StatusOK || !strings.Contains(detail.Body.String(), "Full article body") ||
		strings.Contains(detail.Body.String(), "Short summary") {
		t.Fatal(detail.Body.String())
	}
	if detailed["where"].(map[string]any)["slug"] != "{slug}" {
		t.Fatal("request mutated retained Resource")
	}
	sid, ok := session.IDFromCookie(cookie)
	if !ok || !session.SetValue(sid, "username", "admin") {
		t.Fatal("could not establish fixture session")
	}
	logged := call("GET", "/", "", cookie)
	if strings.Contains(logged.Body.String(), "Guest login") || !strings.Contains(logged.Body.String(), "Private desk") {
		t.Fatal(logged.Body.String())
	}
}

func TestViewResourceValidationDoesNotQueryAndFailuresRemainVisible(t *testing.T) {
	query := map[string]any{"_type": "data_query", "source": "missing"}
	if _, err := artifact.RenderNodesWithResources([]any{resourceNode(query)}, ValidateViewResource); err != nil {
		t.Fatalf("startup unexpectedly queried missing DB: %v", err)
	}
	for _, resource := range []map[string]any{
		{"_type": "invented"}, {"_type": "data_query", "source": "items", "limit": 1.5},
		{"_type": "data_query", "source": "items", "link_prefix": "javascript:"},
		{"_type": "form_ref", "id": ""},
	} {
		if _, err := ValidateViewResource(resource); err == nil {
			t.Fatalf("bad resource accepted: %#v", resource)
		}
	}
	if _, err := artifact.RenderNodes([]any{resourceNode(query)}); err == nil {
		t.Fatal("offline render invented request context")
	}
	bag := app.New(map[string]any{"page": map[string]any{"artifact_nodes": []any{resourceNode(query)}}})
	handler, err := NewHandler(bag, "")
	if err != nil {
		t.Fatal(err)
	}
	response := httptest.NewRecorder()
	handler.ServeHTTP(response, httptest.NewRequest("GET", "/", nil))
	if response.Code != http.StatusInternalServerError || !strings.Contains(response.Body.String(), "no database") {
		t.Fatalf("missing backend became successful empty content: %d %s", response.Code, response.Body.String())
	}
}

func TestViewHeadResourcesServeInHeadAndNeverLeakIntoFragments(t *testing.T) {
	nodes := []any{
		map[string]any{"type": "resource", "value": map[string]any{"_type": "document_head", "meta": map[string]any{"key": "description", "value": "Actual SEO"}, "links": []any{map[string]any{"rel": "stylesheet", "href": "/theme.css"}}}},
		map[string]any{"type": "markdown", "slot": "content", "value": "# Real content"},
	}
	bag := app.New(map[string]any{"page": map[string]any{"title": "Head test", "artifact_nodes": nodes}})
	handler, err := NewHandler(bag, "")
	if err != nil {
		t.Fatal(err)
	}
	for _, path := range []string{"/", "/_part/main", "/_part/content"} {
		response := httptest.NewRecorder()
		handler.ServeHTTP(response, httptest.NewRequest("GET", path, nil))
		body := response.Body.String()
		if response.Code != http.StatusOK || !strings.Contains(body, "Real content") {
			t.Fatalf("%s %d %s", path, response.Code, body)
		}
		if path == "/" {
			head, rest, _ := strings.Cut(body, "</head>")
			if !strings.Contains(head, `<meta name="description" content="Actual SEO"`) || !strings.Contains(head, `href="/theme.css"`) || strings.Contains(rest, "Actual SEO") {
				t.Fatal(body)
			}
		} else if strings.Contains(body, "Actual SEO") || strings.Contains(body, "<head") || strings.Contains(body, "theme.css") {
			t.Fatal(body)
		}
	}
}
