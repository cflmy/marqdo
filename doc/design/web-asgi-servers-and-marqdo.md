# 调研：Marqdo `ext/web` 能否对接 Daphne / 业界应用服务器

| | |
|---|---|
| 状态 | **调研笔记** |
| 日期 | 2026-08-29 |
| 范围 | Daphne、Uvicorn、Gunicorn、Hypercorn 等 ASGI/WSGI 服务器；对照 Marqdo `plugins/web`（axum） |
| 相关 | [web-net-capabilities.md](web-net-capabilities.md) · [ext-web.md](ext-web.md) · [ext-web-net.md](ext-web-net.md) |
| 触发 | `app.listen` 已能本地起站；问「能否像业界一样用 Daphne 等服务器」 |

---

## 0. 一句话结论

**不能把 Marqdo 站点「塞进」Daphne / Uvicorn / Gunicorn 当 Python ASGI/WSGI 应用跑。**

Marqdo 的 HTTP 服务**已经是**应用服务器：`plugins/web` 内嵌 **axum + tokio**，由 `` `app`.listen `` / `` `应用`.监听 `` 绑定端口。这在 Rust 生态里的角色，约等于 Python 里的 **Uvicorn/Daphne**，而不是 Django/Flask「应用代码」。

业界生产部署要支持的，通常不是「换成 Daphne」，而是：

1. **反代 + TLS**（Nginx / Caddy / Traefik）→ 已文档锁定；  
2. **进程监督**（systemd / Docker / k8s）→ 运维层，与框架无关；  
3. （可选）多进程 / 多副本水平扩展 → 需会话外置等，见下文。

---

## 1. Daphne「等」服务器是什么

| 名称 | 协议面 | 典型搭档 | 角色 |
|------|--------|----------|------|
| **Daphne** | ASGI | Django Channels | Python 异步/WebSocket 参考服务器 |
| **Uvicorn** | ASGI | FastAPI、Django ASGI | 高性能 ASGI（常作 Gunicorn worker） |
| **Gunicorn** | 主要为 WSGI | Django/Flask | 进程管理 + worker；**不能**直接跑 ASGI/WebSocket |
| **Hypercorn** | ASGI | 需 HTTP/2/3 时 | ASGI 另一实现 |
| **Granian** | ASGI（Rust 实现） | Python 应用 | 用 Rust 加速 Python ASGI |

共同点：它们都假设应用暴露 **Python ASGI/WSGI callable**（`application` / `asgi.py`）。服务器负责 accept、协议解析、生命周期；**业务在 Python 进程里**。

2026 常见生产组合示例：

```text
Internet → Nginx/Caddy (TLS) → Uvicorn/Daphne (ASGI) → Django/FastAPI 应用
```

---

## 2. Marqdo 今天怎么起服务

```text
.mq.md（ext/web 作者面）
    → ABI：web_listen / 路由装配
    → plugins/web：组装 axum::Router
    → tokio TcpListener + axum::serve   ← 这里就是「应用服务器」
```

关键事实（实现）：

- `plugins/web/src/http.rs`：`TcpListener::bind` + `axum::serve`；  
- 依赖：`axum` 0.8、`tokio` multi-thread；含 WS、multipart、中间件层；  
- HTTPS：设计上 **进程内 TLS 不作为默认**；生产用反代终止 TLS + `cookie_secure`（见 [web-net-capabilities.md](web-net-capabilities.md) W7）。

因此：

| 问法 | 答案 |
|------|------|
| 能否 `daphne marqdo_app:app`？ | **否** — 没有 Python ASGI 入口 |
| 能否 `gunicorn -k uvicorn.workers…`？ | **否** — 同上 |
| 能否用 Nginx 反代到 `127.0.0.1:18085`？ | **是** — 推荐生产路径 |
| Marqdo 有没有「业界级」HTTP 栈？ | **有** — axum/hyper 即 Rust 侧业界默认之一 |

---

## 3. 分层对照（避免角色错位）

| 层级 | Python 典型 | Marqdo 对应 |
|------|-------------|-------------|
| 应用 / 路由 / 模板 / DB | Django / FastAPI | `ext/web` + `plugins/web` 装配 |
| 应用服务器（accept / HTTP / WS） | Daphne / Uvicorn | **内嵌 axum::serve** |
| 反代 / TLS / 静态边缘 | Nginx / Caddy | **外置反代**（文档已写） |
| 进程管理 | systemd / Gunicorn master | systemd / Docker / k8s |

把 Daphne 接到 Marqdo，相当于要求「用 Python ASGI 服务器去托管 Rust axum 应用」——协议面不匹配，没有现成插槽。

---

## 4. 若目标是「生产级部署」，已支持什么 / 还差什么

### 4.1 已支持（或已锁定）

| 能力 | 状态 |
|------|------|
| 本地 / 内网 `listen` | ✅ |
| HTTP + WebSocket | ✅（axum） |
| 反代终止 HTTPS | ✅ 文档路径 |
| 访问日志、限速、CORS、安全头等 | ✅ W1–W7 |
| 会话可 SQLite 持久化 | ✅（单机多启需注意） |

### 4.2 与「Daphne 集群」对等时仍需注意

| 主题 | 说明 |
|------|------|
| **多进程副本** | 每进程各自 `listen` 时，内存会话 / 内存 cache / 内存 WS hub **不共享**；多副本需 sticky session，或会话/缓存外置（Redis 等，驱动面已有雏形） |
| **优雅重启 / 零停机** | axum 可接 signal；未做成一等作者 API（运维可用 systemd + 反代） |
| **HTTP/2 末端** | 多由反代提供；进程内 HTTP/1.1 足够常见场景 |
| **与 Python 生态混部** | 可同机 Nginx 分流：`/api/py` → Uvicorn，`/` → Marqdo；**不是**共用一个 Daphne |

### 4.3 不建议的路线

1. **实现完整 ASGI 适配器让 Daphne 加载 Marqdo**  
   成本高、收益低：等于在 Python 进程里再调一层 FFI/子进程，丢掉 axum 直驱优势。  
2. **把 axum 拆掉、改写为「纯应用、外挂任意服务器」**  
   Rust 没有统一 ASGI；外挂只能是自定义协议或「只当 CGI」，与当前架构相反。

### 4.4 可选增强（若要对标「可换服务器」的体验）

| 方向 | 含义 | 优先级建议 |
|------|------|------------|
| **A. 部署文档 + 示例 Compose** | Nginx/Caddy + `marqdo run` + systemd | **高**（满足绝大多数「业界部署」诉求） |
| **B. listen 运维旋钮** | workers 说明、graceful shutdown、Unix socket | 中 |
| **C. 多副本清单** | 会话/WS/cache 外置检查表 | 中 |
| **D. 反向：Marqdo 作边缘、反代到既有 ASGI** | 已可用 HTTP 客户端/路由外置服务 | 按需 |
| **E. ASGI bridge** | 明确 **非目标** | 不做 |

---

## 5. 推荐生产拓扑（等价于「Nginx + Daphne」）

```text
客户端
  │ HTTPS
  ▼
Nginx / Caddy / Traefik     ← TLS、压缩、限流、静态可旁路
  │ HTTP proxy_pass
  ▼
marqdo run index.mq.md      ← 内部 axum::serve（角色 ≈ Uvicorn/Daphne）
  （host=127.0.0.1 port=18085，cookie_secure=True）
```

要点：

- Marqdo **只绑回环**；对外只暴露反代。  
- WebSocket：反代需 `Upgrade` / `Connection` 正确转发（与反代 Daphne 相同注意点）。  
- 多机：共享 DB（或 Postgres 驱动）、共享会话存储；WS 广播跨机需外置 pub/sub（当前 hub 为进程内）。

---

## 6. 结论与建议

| 问题 | 结论 |
|------|------|
| 能支持 Daphne 吗？ | **不能直接托管**；协议是 Python ASGI，Marqdo 是 Rust axum 自托管。 |
| 能支持「业界服务器」吗？ | **能**：用 **Nginx/Caddy 等业界反代** 挂在 `listen` 前面；应用服务器层已由 axum 承担。 |
| 缺的是换 Daphne，还是缺部署体验？ | 主要是 **部署文档 / Compose / 多副本约定**，不是再嵌一套 ASGI 运行时。 |

**建议下一刀（若产品化）：** 写一份 `examples/` 或 `doc/design` 下的「反代部署」最小样例（Caddyfile / nginx.conf + `cookie_secure`），而不是调研 ASGI bridge。

---

## 7. 参考

- 本仓：[web-net-capabilities.md](web-net-capabilities.md)（HTTPS 反代锁定）· [ext-web.md](ext-web.md)  
- Daphne：Django Channels 参考 ASGI 服务器  
- Uvicorn / Gunicorn worker：当代 Python ASGI 生产主流组合  
- axum：Tokio 生态 HTTP 应用框架（Rust 侧与 Uvicorn 同层）
