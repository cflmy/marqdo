package style_test

import (
	"strings"
	"testing"

	"github.com/marqdo/marqdo/plugins/web/internal/style"
)

func TestStyleSimple(t *testing.T) {
	css, err := style.Style(map[string]any{
		"name": "card",
		"table": []any{
			map[string]any{"选择器": ".card", "属性": "color", "值": "navy"},
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(css, ".card {") || !strings.Contains(css, "color: navy;") {
		t.Fatalf("%q", css)
	}
}

func TestStyleStrictKeepsQuotedSlash(t *testing.T) {
	css, err := style.Style(map[string]any{
		"name":   "t",
		"strict": true,
		"table": []any{
			map[string]any{"选择器": ".box", "属性": "grid-column", "值": "1 / 5"},
		},
	})
	if err != nil {
		t.Fatal(err)
	}
	if !strings.Contains(css, "1 / 5") {
		t.Fatalf("missing quoted value: %q", css)
	}
}

func TestStyleStrictAllowsNumeric(t *testing.T) {
	css, err := style.Style(map[string]any{
		"name":   "x",
		"strict": true,
		"table": []any{
			map[string]any{"选择器": ".x", "属性": "z-index", "值": float64(20)},
			map[string]any{"选择器": ".y", "属性": "opacity", "值": float64(0.5)},
			map[string]any{"选择器": ".z", "属性": "font-weight", "值": float64(700)},
		},
	})
	if err != nil {
		t.Fatalf("numeric CSS must not be suspicious: %v", err)
	}
	if !strings.Contains(css, "z-index: 20") || !strings.Contains(css, "opacity: 0.5") {
		t.Fatalf("%q", css)
	}
}

func TestStyleStrictRejectsBoolean(t *testing.T) {
	_, err := style.Style(map[string]any{
		"name":   "x",
		"strict": true,
		"table": []any{
			map[string]any{"选择器": ".x", "属性": "display", "值": true},
		},
	})
	if err == nil || !strings.Contains(err.Error(), "suspicious") {
		t.Fatalf("err=%v", err)
	}
}
