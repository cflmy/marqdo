package render

import (
	"fmt"
	"path/filepath"
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/compose"
	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

func TestAnlianPostsResolve(t *testing.T) {
	url := "sqlite:" + filepath.Join(t.TempDir(), "anlian.db")
	fields := []any{
		map[string]any{"name": "title", "type": "text", "nullable": false},
		map[string]any{"name": "summary", "type": "text"},
		map[string]any{"name": "board", "type": "text"},
		map[string]any{"name": "slug", "type": "text"},
	}
	if _, err := db.Init(url, "posts", fields); err != nil {
		t.Fatal(err)
	}
	rows := []any{map[string]any{"title": "欢迎", "summary": "Portable test fixture", "board": "demo", "slug": "welcome"}}
	if _, err := db.Insert(url, "posts", rows); err != nil {
		t.Fatal(err)
	}
	card := map[string]any{
		"属性": []any{"title", "body", "meta", "href"},
		"值":  []any{"posts.title", "posts.summary", "posts.board", "posts.slug"},
		"样式": []any{"", "", "", ""},
	}
	page, err := compose.ComposeMain(map[string]any{"title": "帖子", "intro": "<h1>帖子</h1>"}, card, func(path string) (any, error) {
		return nil, fmt.Errorf("no lib %s", path)
	})
	if err != nil {
		t.Fatal(err)
	}
	pm := page.(map[string]any)
	binds, _ := table.AsBind(pm["main"]).([]any)
	tn, ok := table.BindTableName(binds)
	t.Logf("table=%v BindTableName=%q ok=%v main=%v", pm["table"], tn, ok, pm["main"])
	out, err := db.Select(url, tn, 200, db.SelectOpts{Order: "-created_at"})
	t.Logf("select err=%v rows=%v", err, out["rows"])
	intro, items, _ := resolveMain(pm, url)
	t.Logf("intro=%q items=%d", intro, len(items))
	html := RenderPage(pm, url, "")
	if !strings.Contains(html, "欢迎") {
		t.Fatalf("missing 欢迎 in html:\n%s", html)
	}
}
