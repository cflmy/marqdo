#!/usr/bin/env bash
# 暗恋见君 Marqdo 切片冒烟（需先启动 index.mq.md）。
set -uo pipefail
BASE="http://127.0.0.1:18180"
PASS=0; FAIL=0
ok()  { echo "  ok: $1"; PASS=$((PASS+1)); }
bad() { echo "  FAIL: $1"; FAIL=$((FAIL+1)); }

echo "== 首页 =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/")
[ "$code" = "200" ] && ok "GET /" || bad "GET / -> $code"
curl -s "$BASE/" | grep -q '暗恋见君' && ok "品牌" || bad "品牌"
curl -s "$BASE/" | grep -q 'class="hero"' && ok "Hero" || bad "Hero"
curl -s "$BASE/" | grep -q 'notice-title\|公告信息' && ok "公告" || bad "公告"
curl -s "$BASE/" | grep -q 'news-timeline\|新闻动态' && ok "新闻时间线" || bad "新闻时间线"
curl -s "$BASE/" | grep -q 'home-posts\|论坛交流' && ok "帖子区" || bad "帖子区"
curl -s "$BASE/" | grep -q 'bg-body-960\|/static/css/base.css' && ok "原站 CSS/背景" || bad "原站 CSS/背景"
curl -s -o /dev/null -w "%{http_code}" "$BASE/static/css/base.css" | grep -q 200 && ok "base.css" || bad "base.css"

echo "== 列表 / 详情 =="
curl -s "$BASE/post" | grep -q 'welcome\|欢迎\|论坛交流' && ok "帖子列表" || bad "帖子列表"
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/post/welcome")
[ "$code" = "200" ] && ok "帖子详情" || bad "帖子详情 -> $code"
curl -s "$BASE/news" | grep -q 'migrate\|迁往\|新闻' && ok "新闻列表" || bad "新闻列表"

echo "== 片段 API =="
curl -s "$BASE/post/ten" | grep -q 'home-post-card\|欢迎' && ok "/post/ten" || bad "/post/ten"
curl -s "$BASE/news/five" | grep -q 'news-timeline\|迁往\|站点' && ok "/news/five" || bad "/news/five"

echo "== 其它页 =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/topics")
[ "$code" = "200" ] && ok "专题" || bad "专题 -> $code"
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/chat")
[ "$code" = "200" ] && ok "聊天列表" || bad "聊天列表 -> $code"
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/guide/automation-api")
[ "$code" = "200" ] && ok "API 指南" || bad "API 指南 -> $code"
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/sitemap.xml")
[ "$code" = "200" ] && ok "sitemap" || bad "sitemap -> $code"

echo "== 门禁 / 登录 =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/post/public")
[ "$code" = "303" ] || [ "$code" = "302" ] && ok "未登录发帖重定向" || bad "发帖 -> $code"
rm -f /tmp/anlian_mq_hdr.txt
curl -s -D /tmp/anlian_mq_hdr.txt -o /dev/null -d "username=admin&password=anlian" "$BASE/accounts/login"
sid=$(grep -i "set-cookie" /tmp/anlian_mq_hdr.txt | sed 's/.*marqdo_sid=\([^;]*\).*/\1/' | head -1)
[ -n "$sid" ] && ok "会话 cookie" || bad "无 cookie"
code=$(curl -s -b "marqdo_sid=$sid" -o /dev/null -w "%{http_code}" "$BASE/post/public")
[ "$code" = "200" ] && ok "登录后发帖" || bad "登录后发帖 -> $code"

echo "== WS =="
wsok=$(python3 -c "
import socket, base64, os
s = socket.create_connection(('127.0.0.1', 18180), timeout=3)
key = base64.b64encode(os.urandom(16)).decode()
s.sendall((f'GET /chat/ws HTTP/1.1\r\nHost: x\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n').encode())
r = s.recv(1024).decode(errors='replace')
print('OK' if '101' in r.split('\r\n')[0] else 'NO')
s.close()
" 2>/dev/null)
[ "$wsok" = "OK" ] && ok "WebSocket" || bad "WebSocket $wsok"

echo
echo "通过 $PASS / $((PASS+FAIL))"
[ "$FAIL" = "0" ] || exit 1
