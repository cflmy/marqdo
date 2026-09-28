#!/usr/bin/env bash
# Artifact blog smoke (ADR 0007). Usage: bash examples/marqdo-blog/smoke.sh
# Expects serve.mq.md already listening on BASE (default 18081).
set -uo pipefail
BASE="${BASE:-http://127.0.0.1:18081}"
PASS=0; FAIL=0
ok()   { echo "  ok: $1"; PASS=$((PASS+1)); }
bad()  { echo "  FAIL: $1"; FAIL=$((FAIL+1)); }

echo "== Document home =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/")
[ "$code" = "200" ] && ok "GET / -> $code" || bad "GET / -> $code"
curl -s "$BASE/" | grep -q '<title>Marqdo 博客</title>' && ok "title" || bad "title"
curl -s "$BASE/" | grep -q 'class="card"' && ok "data_source cards" || bad "cards missing"
curl -s "$BASE/" | grep -q 'Hello Marqdo' && ok "seed title" || bad "seed title missing"

echo "== Static =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/static/live.js")
[ "$code" = "200" ] && ok "GET /static/live.js -> $code" || bad "static -> $code"

echo "== Dynamic Document =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/post/hello")
[ "$code" = "200" ] && ok "GET /post/hello -> $code" || bad "post detail -> $code"
curl -s "$BASE/post/hello" | grep -q 'article-title' && ok "article title" || bad "article title missing"

echo "== ZH Document =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/about")
[ "$code" = "200" ] && ok "GET /about -> $code" || bad "about -> $code"
curl -s "$BASE/about" | grep -q '关于 Marqdo' && ok "about body" || bad "about body missing"

echo "== Endpoint =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/api/ping")
[ "$code" = "200" ] && ok "GET /api/ping -> $code" || bad "ping -> $code"
curl -s "$BASE/api/ping" | grep -q 'marqdo-blog' && ok "ping json" || bad "ping json missing"
code=$(curl -s -o /tmp/mq_blog_echo.json -w "%{http_code}" \
  -H 'Content-Type: application/json' -d '{"message":"hi"}' "$BASE/api/echo")
[ "$code" = "200" ] && ok "POST /api/echo -> $code" || bad "echo -> $code"
grep -q '"message":"hi"' /tmp/mq_blog_echo.json && ok "echo body" || bad "echo body missing"

echo
echo "通过 $PASS / $((PASS+FAIL))"
[ "$FAIL" = "0" ] || exit 1
