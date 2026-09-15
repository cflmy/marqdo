package render

import (
	"bytes"
	"strings"

	"github.com/yuin/goldmark"
	"github.com/yuin/goldmark/extension"
	"github.com/yuin/goldmark/parser"
	"github.com/yuin/goldmark/renderer/html"
)

// markdownToHTML renders article body Markdown to HTML (GFM subset), matching
// the Rust web plugin's pulldown-cmark path.
func markdownToHTML(md string) string {
	md = strings.TrimSpace(md)
	if md == "" {
		return ""
	}
	gm := goldmark.New(
		goldmark.WithExtensions(
			extension.GFM,
			extension.Strikethrough,
		),
		goldmark.WithParserOptions(
			parser.WithAutoHeadingID(),
		),
		goldmark.WithRendererOptions(
			html.WithHardWraps(),
			html.WithXHTML(),
		),
	)
	var buf bytes.Buffer
	if err := gm.Convert([]byte(md), &buf); err != nil {
		return `<div class="article-body"><p class="article-p">` + esc(md) + `</p></div>`
	}
	return `<div class="article-body md">` + buf.String() + `</div>`
}
