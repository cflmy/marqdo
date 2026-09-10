#!/usr/bin/env bash
# 暗恋见君 Marqdo 切片冒烟（需先启动 index.mq.md）。
# 用法：bash examples/anlian-mq/smoke.sh
set -uo pipefail
BASE="http://127.0.0.1:18180"
PASS=0; FAIL=0
ok()  { echo "  ok: $1"; PASS=$((PASS+1)); }
bad() { echo "  FAIL: $1"; FAIL=$((FAIL+1)); }

echo "== 首页 / 列表 =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/")
[ "$code" = "200" ] && ok "GET /" || bad "GET / -> $code"
curl -s "$BASE/" | grep -q '暗恋见君' && ok "品牌标题" || bad "品牌标题"
curl -s "$BASE/" | grep -q 'class="hero"' && ok "Hero 区" || bad "Hero 区"
curl -s "$BASE/" | grep -q 'bg-hero-640\|heroImageFloat\|hero-art' && ok "Hero 主图/动效类" || bad "Hero 主图/动效类"
curl -s -o /dev/null -w "%{http_code}" "$BASE/static/bg-body-960.webp" | grep -q 200 && ok "背景图静态资源" || bad "背景图静态资源"
curl -s "$BASE/" | grep -q '@keyframes pulse\|animation: pulse\|animation:pulse' && ok "pulse 动效 CSS" || bad "pulse 动效 CSS"
curl -s "$BASE/posts" | grep -q 'welcome\|欢迎' && ok "帖子列表" || bad "帖子列表"
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/post/welcome")
[ "$code" = "200" ] && ok "帖子详情" || bad "帖子详情 -> $code"
curl -s "$BASE/news" | grep -q 'migrate\|迁往\|Marqdo' && ok "新闻列表" || bad "新闻列表"

echo "== 站长 =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/sitemap.xml")
[ "$code" = "200" ] && ok "sitemap" || bad "sitemap -> $code"
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/robots.txt")
[ "$code" = "200" ] && ok "robots" || bad "robots -> $code"

echo "== 门禁 =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/write")
[ "$code" = "303" ] || [ "$code" = "302" ] && ok "未登录 /write 重定向" || bad "/write -> $code"

echo "== 登录 =="
rm -f /tmp/anlian_mq_hdr.txt
curl -s -D /tmp/anlian_mq_hdr.txt -o /dev/null -d "username=admin&password=anlian" "$BASE/admin/login"
sid=$(grep -i "set-cookie" /tmp/anlian_mq_hdr.txt | sed 's/.*marqdo_sid=\([^;]*\).*/\1/' | head -1)
[ -n "$sid" ] && ok "会话 cookie" || bad "无 cookie"
code=$(curl -s -b "marqdo_sid=$sid" -o /dev/null -w "%{http_code}" "$BASE/write")
[ "$code" = "200" ] && ok "登录后 /write" || bad "登录后 /write -> $code"

echo "== 聊天 WS =="
wsok=$(python3 -c "
import socket, base64, os
s = socket.create_connection(('127.0.0.1', 18180), timeout=3)
key = base64.b64encode(os.urandom(16)).decode()
s.sendall((f'GET /chat/ws HTTP/1.1\r\nHost: x\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n').encode())
r = s.recv(1024).decode(errors='replace')
print('OK' if '101' in r.split('\r\n')[0] else 'NO')
s.close()
" 2>/dev/null)
[ "$wsok" = "OK" ] && ok "WebSocket 握手" || bad "WebSocket $wsok"

echo
echo "通过 $PASS / $((PASS+FAIL))"
[ "$FAIL" = "0" ] || exit 1
