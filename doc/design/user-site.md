# 用户静态文档站（public / gh-pages）

| | |
|---|---|
| 状态 | 已定 |
| 日期 | 2026-08-04 |
| 相关 | [view.md](view.md) · 源目录 [`public/`](../../public/) |

---

## 1. 分工

| 路径 | 是否入库 | 职责 |
|------|----------|------|
| `doc/` | 是 | 开发者设计 / ADR / 路线图（单数 `doc`，勿再造 `docs/`） |
| `public/**/*.mq.md` | 是 | 用户可读、可执行、可下载的介绍（结构 / 关键字 / 特性迭代）；**双语**：拉丁文件名 = 英文，中文文件名 = 中文编程，同目录并列 |
| `public/index.html`、`public/pages/` | **否**（ignore） | `marqdo view output` 生成的 HTML |
| `tests/{structure,keywords,errors}/` | 是 | 金样例与失败夹具（含 errors） |
| 远程分支 `gh-pages` | 发布产物 | 发布整个 `public/`（源文件 + 生成 HTML） |

**禁止**把 `tests/errors/` 编进用户站。

---

## 2. 生成

```bash
cargo run --release -- view output public -o public
# 或
./scripts/build-public.sh
powershell -File ./scripts/build-public.ps1
```

产物写在源旁：

```
public/
  *.mq.md / structure/ / …   # 入库源
  index.html                 # 生成（ignore）
  pages/…                    # 生成（ignore）
```

---

## 3. 发布分支

将构建后的 `public/` 推到 **`gh-pages`**。CI：`.github/workflows/pages.yml`。

仓库 Settings → Pages → Source = Deploy from branch → `gh-pages` / root。

---

## 4. 验收

1. `public/` 下每个 `.mq.md`（除纯工具模块外）`marqdo run` 退出码 0。  
2. `view output public -o public` 后首页为欢迎页，侧栏无 errors。  
3. 主分支跟踪 `public/**/*.mq.md`；不提交 `public/index.html` / `public/pages/`。
