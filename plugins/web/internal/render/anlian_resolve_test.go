package render

import (
	"fmt"
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/compose"
	"github.com/marqdo/marqdo/plugins/web/internal/db"
	"github.com/marqdo/marqdo/plugins/web/internal/table"
)

func TestAnlianPostsResolve(t *testing.T) {
	url := "sqlite:/home/cflmy/work/marqdo/examples/anlian-mq/data/anlian-mq.db"
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
