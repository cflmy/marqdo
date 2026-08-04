"""ADR 0001 Spike acceptance: S1–S5."""

from __future__ import annotations

from marqdo_spike import (
    front_matter_imports,
    parse_tokens,
    scan_bullets,
    split_front_matter,
)


def test_s1_emphasis_vs_strong():
    """S1: *a* vs **a** yield distinct inline marks."""
    src = "hello *em* and **strong** end\n"
    tokens = parse_tokens(src)
    inlines = [t for t in tokens if t.type == "inline"]
    assert inlines, "expected inline token"
    children = inlines[0].children or []
    types = [c.type for c in children]
    assert "em_open" in types and "em_close" in types
    assert "strong_open" in types and "strong_close" in types
    assert any(c.type == "text" and c.content == "em" for c in children)
    assert any(c.type == "text" and c.content == "strong" for c in children)


def test_s2_bullet_minus_vs_plus_line_scan():
    """S2: line scan distinguishes '-' (loop) vs '+' (branch)."""
    src = """
# main

- loop item
+ branch item
"""
    bullets = scan_bullets(src)
    markers = [b.marker for b in bullets]
    assert "-" in markers
    assert "+" in markers
    assert bullets[0].marker == "-"
    assert bullets[1].marker == "+"


def test_s2_library_list_tokens_exist():
    """S2b: library at least parses both as lists (bullet char may need line scan)."""
    src = "- a\n\n+ b\n"
    tokens = parse_tokens(src)
    assert any(t.type == "bullet_list_open" for t in tokens)


def test_s3_single_column_table():
    """S3: GFM single-column table."""
    src = """
| 果 |
|----|
| 苹果 |
| 梨 |
"""
    tokens = parse_tokens(src)
    assert any(t.type == "table_open" for t in tokens)
    texts = [t.content for t in tokens if t.type == "inline"]
    flat = " ".join(texts)
    assert "果" in flat
    assert "苹果" in flat
    assert "梨" in flat


def test_s4_front_matter_vs_body_hr():
    """S4: file-level --- front matter vs body --- frame."""
    src = """---
title: demo
---

# main

---
+ `x` > 0
  > 输出 内容=正
---
"""
    fm, body = split_front_matter(src)
    assert fm is not None
    assert "title: demo" in fm
    assert body.lstrip().startswith("# main")
    assert "---" in body
    assert "+ `x` > 0" in body
    # Body still has its own ---; must not be eaten by FM splitter
    assert body.count("---") >= 2


def test_s5_front_matter_mq_imports():
    """S5: `> file.mq.md` lines inside front matter."""
    src = """---
title: main
> utils.mq.md
> lib/math.mq.md
---

# main
"""
    fm, body = split_front_matter(src)
    assert fm is not None
    imports = front_matter_imports(fm)
    assert imports == ["utils.mq.md", "lib/math.mq.md"]
    assert "# main" in body
