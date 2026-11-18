package render

import (
	"strings"
	"testing"
)

func TestFormatMetaDate(t *testing.T) {
	if got := formatMetaDate("2026-09-17T13:41:04Z"); got != "2026-09-17" {
		t.Fatalf("got %q", got)
	}
	if got := formatMetaDate("plain"); got != "plain" {
		t.Fatalf("got %q", got)
	}
}

func TestTuneDropcapHTML(t *testing.T) {
	in := "<p>中文首段。</p><p>二段</p>"
	out := tuneDropcapHTML(in)
	if !strings.Contains(out, `class="has-dropcap"`) {
		t.Fatalf("expected dropcap: %s", out)
	}
	skip := "<p>$E=mc^2$</p>"
	if got := tuneDropcapHTML(skip); strings.Contains(got, "has-dropcap") {
		t.Fatalf("formula should skip: %s", got)
	}
}
