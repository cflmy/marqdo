package form_test

import (
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/form"
)

func TestValidateRequiredAndEmail(t *testing.T) {
	f := form.New("users", "insert", "")
	f = form.SetFields(f, []any{
		map[string]any{"字段": "email", "标签": "Email", "类型": "email", "必填": true},
		map[string]any{"字段": "name", "标签": "Name", "类型": "text", "必填": false},
	})
	f = form.SetRules(f, []any{
		map[string]any{"字段": "email", "规则": "email", "消息": "bad email"},
	})

	bad := form.Validate(f, nil, map[string]any{"email": "", "name": "x"})
	if ok, _ := bad["ok"].(bool); ok {
		t.Fatalf("expected required failure: %v", bad)
	}
	errs, _ := bad["errors"].([]any)
	if len(errs) == 0 {
		t.Fatal("expected errors")
	}

	badEmail := form.Validate(f, nil, map[string]any{"email": "not-an-email", "name": "x"})
	if ok, _ := badEmail["ok"].(bool); ok {
		t.Fatalf("expected email failure: %v", badEmail)
	}
	found := false
	for _, e := range badEmail["errors"].([]any) {
		m := e.(map[string]any)
		if m["field"] == "email" && m["message"] == "bad email" {
			found = true
		}
	}
	if !found {
		t.Fatalf("expected custom email message: %v", badEmail)
	}

	good := form.Validate(f, nil, map[string]any{"email": "a@b.co", "name": "x"})
	if ok, _ := good["ok"].(bool); !ok {
		t.Fatalf("expected ok: %v", good)
	}
}

func TestRenderContainsInputFields(t *testing.T) {
	f := form.New("articles", "insert", "")
	f = form.SetFields(f, []any{
		map[string]any{"字段": "title", "标签": "Title", "类型": "text", "必填": true},
		map[string]any{"字段": "body", "标签": "Body", "类型": "textarea"},
	})
	html := form.Render(f, "article", nil, nil, "")
	checks := []string{
		`name="title"`,
		`name="body"`,
		"<textarea",
		`action="/_form/article"`,
		"Title",
		"Body",
	}
	for _, want := range checks {
		if !strings.Contains(html, want) {
			t.Fatalf("missing %q in:\n%s", want, html)
		}
	}
}

func TestRenderMarkdownAndLabels(t *testing.T) {
	f := form.New("posts", "insert", "")
	f = form.SetFields(f, []any{
		map[string]any{"字段": "content", "标签": "正文", "类型": "markdown", "必填": true},
	})
	f = form.SetLabels(f, "发布文章", "返回列表", "/desk/posts")
	if f["redirect"] != "/desk/posts" || f["cancel_href"] != "/desk/posts" {
		t.Fatalf("labels should stamp redirect/cancel_href, got %#v %#v", f["redirect"], f["cancel_href"])
	}
	html := form.Render(f, "post", nil, nil, "")
	for _, want := range []string{
		`data-mq-field="markdown"`,
		`class="mq-markdown"`,
		">发布文章</button>",
		`href="/desk/posts"`,
		">返回列表</a>",
	} {
		if !strings.Contains(html, want) {
			t.Fatalf("missing %q in:\n%s", want, html)
		}
	}
}
