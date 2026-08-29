#!/usr/bin/env bash
# 冒烟测试：marqdo-blog 全功能验证。
# 用法：bash examples/marqdo-blog/smoke.sh
set -uo pipefail
BASE="http://127.0.0.1:18085"
PASS=0; FAIL=0
ok()   { echo "  ok: $1"; PASS=$((PASS+1)); }
bad()  { echo "  FAIL: $1"; FAIL=$((FAIL+1)); }

echo "== 首页 =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/")
[ "$code" = "200" ] && ok "GET / -> $code" || bad "GET / -> $code"
curl -s "$BASE/" | grep -q '<title>Marqdo 博客</title>' && ok "title" || bad "title"
curl -s "$BASE/" | grep -q 'class="card"' && ok "卡片渲染" || bad "卡片缺失"
curl -s "$BASE/" | grep -q 'aside.side' && ok "主题 CSS 注入" || bad "主题 CSS 缺失"
curl -s "$BASE/" | grep -q '@media (max-width: 860px)' && ok "响应式媒体查询" || bad "媒体查询缺失"

echo "== 静态 =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/static/live.js")
[ "$code" = "200" ] && ok "GET /static/live.js -> $code" || bad "static -> $code"
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/favicon.ico")
[ "$code" = "200" ] && ok "GET /favicon.ico -> $code" || bad "favicon -> $code"
curl -s "$BASE/" | grep -q 'rel="icon"' && ok "head icon link" || bad "head icon 缺失"
curl -s "$BASE/" | grep -q 'mq-images' && ok "图片装配 logo" || bad "mq-images 缺失"

echo "== 动态路由 =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/post/hello-marqdo")
[ "$code" = "200" ] && ok "GET /post/hello-marqdo -> $code" || bad "post detail -> $code"
curl -s "$BASE/post/hello-marqdo" | grep -q 'article-title' && ok "文章详情渲染" || bad "文章详情缺失"

echo "== 标签 =="
curl -s "$BASE/tags" | grep -q 'href="/tag/marqdo"' && ok "标签归档链接" || bad "标签链接缺失"
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/tag/web")
[ "$code" = "200" ] && ok "GET /tag/web -> $code" || bad "tag page -> $code"

echo "== 鉴权门禁 =="
code=$(curl -s -o /dev/null -w "%{http_code}" "$BASE/admin")
[ "$code" = "303" ] && ok "未登录 /admin -> $code (重定向登录)" || bad "未登录 /admin -> $code"

echo "== 登录 =="
rm -f /tmp/mq_smoke_cookies.txt /tmp/mq_smoke_headers.txt
curl -s -D /tmp/mq_smoke_headers.txt -o /dev/null -d "username=admin&password=marqdo" "$BASE/admin/login"
sid=$(grep -i "set-cookie" /tmp/mq_smoke_headers.txt | sed 's/.*marqdo_sid=\([^;]*\).*/\1/')
[ -n "$sid" ] && ok "登录获取会话 cookie" || bad "登录未返回 cookie"
code=$(curl -s -b "marqdo_sid=$sid" -o /dev/null -w "%{http_code}" "$BASE/admin")
[ "$code" = "200" ] && ok "带会话访问 /admin -> $code" || bad "带会话 /admin -> $code"

echo "== 发布文章 (CRUD) =="
code=$(curl -s -b "marqdo_sid=$sid" -o /dev/null -w "%{http_code}" "$BASE/admin/posts/new")
[ "$code" = "200" ] && ok "GET /admin/posts/new -> $code" || bad "new form -> $code"

echo "== WebSocket =="
wsok=$(python3 -c "
import socket, base64, os
s = socket.create_connection(('127.0.0.1', 18085), timeout=3)
key = base64.b64encode(os.urandom(16)).decode()
s.sendall((f'GET /live HTTP/1.1\r\nHost: x\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {key}\r\nSec-WebSocket-Version: 13\r\n\r\n').encode())
r = s.recv(1024).decode(errors='replace')
if '101' not in r.split('\r\n')[0]:
    print('NO'); s.close(); raise SystemExit
mask = os.urandom(4); payload = b'ping'
m = bytes(b ^ mask[i%4] for i, b in enumerate(payload))
s.sendall(bytes([0x81, 0x80|len(payload)]) + mask + m)
d = s.recv(1024)
print('OK' if b'ping' in d else 'NO')
s.close()
" 2>/dev/null)
[ "$wsok" = "OK" ] && ok "WebSocket /live 回显" || bad "WebSocket 回显 $wsok"

echo
echo "通过 $PASS / $((PASS+FAIL))"
[ "$FAIL" = "0" ] || exit 1
