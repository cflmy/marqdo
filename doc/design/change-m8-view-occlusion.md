# 变更记录：M8 view 遮蔽呈现（+ 修一个真实回归）

| | |
|---|---|
| 状态 | **Implemented** |
| 日期 | 2026-10-22 |
| 来源 | [occlusion-syntax.md](occlusion-syntax.md) §13 M8「view 徽章/呈现」 |
| 改动 | `src/load.rs` · `src/view/{document,render,html,mod,debug_page}.rs` |
| 影响 | 只影响呈现；顺带修了 **view 无法解析遮蔽文件** 的回归 |

---

## 1. 先修回归：view 用错解析模式

`view` 内部多处直接用 `crate::parse::parse_source(source)` 重新解析文件，
而 per-file `markup:` 是在 `load` 层生效的（M7）。结果：

```
<div class="structure"><div class="err">parse/load error: expected ')' after link-index collection</div></div>
```

任何带 `markup: occlusion` 的文档在 view 里**结构区直接报错**，只能退化成源码视图。
用户看到的"粗体里的 `[描述](实现)` 没渲染"其实是这个错误的表象。

**修法**：`load::parse_source_auto(source)` —— 先读 frontmatter 的 `markup:`，
用 `with_markup` 包住 `parse_source`，view 的五处解析点全部改用它。

## 2. 遮蔽呈现

### 2.1 浏览视图（`document.rs`，`/file`）

```html
<div class="doc-line doc-occl" data-line="81">
  <span class="occl-pre">留存 =</span>          <!-- 变量名：灰、等宽 -->
  <span class="occl-label">客户留存率，模拟数据</span>  <!-- 描述：正文 -->
  <code class="occl-code">0.42</code>           <!-- 实现：代码片 -->
</div>
```

- 空载荷 → `<span class="occl-stub">桩</span>`（琥珀色徽章，title 说明"未接通或待补返回值"）
- 前缀里的反引号/`**` 都剥掉（与其它行 `mq-anchor` 的处理一致）
- 只在词法上切分，不改语义：解析器早已验证结构，这里只决定怎么画

### 2.2 卡片视图（`render.rs`，`/debug`）

按 AST 直接匹配 `Expr::Occluded`（`Assign` / `Expr` / `Return` 三种语句），
渲染成 `card occl-card`：徽章「遮蔽」+ 变量名 + 描述 + 代码片/桩徽章。

### 2.3 样式（`html.rs`）

```css
.doc-occl { display:flex; align-items:baseline; flex-wrap:wrap; gap:.4rem; }
.occl-pre { font-family:var(--mono); font-size:.86em; color:var(--muted); }
.occl-label { color:var(--ink); }
.occl-code { font-family:var(--mono); background:var(--fill); border:1px solid var(--line);
             border-radius:6px; padding:.02rem .38rem; }
.occl-stub { background:rgba(255,159,10,.18); color:#9a5b00; border-radius:999px; }
```

刻意**不加**虚线下划线：容易被读成链接/缩写。描述就是正文，靠代码片的边框与之区分。

## 3. 验证

| 项 | 结果 |
|----|------|
| 结构区解析错误 | **消失**（`/file` 页面 0 处 `parse/load error`） |
| 七个遮蔽行 | 全部渲染为 `doc-occl`：6 个代码片 + 1 个桩徽章 |
| 文本抽取 | `留存 = 客户留存率，模拟数据 [0.42]` / `待接 = 搜索接入，接口未接，待开发 [桩]` |
| 单元（4） | 裸赋值 / 粗体赋值 / 空载荷+嵌套 payload / 非遮蔽行不误判 |
| 全量 | `cargo test --release` **613 passed / 0 failed**（609 + 4） |

## 4. 未能做到的：截图

本机没有任何浏览器（`chromium`/`chrome`/`firefox` 均不存在），
`npm i puppeteer` 装上了包但 Chromium 下载被网络阻断（`storage.googleapis.com`
TLS 断连，两次重试均失败）。所以无法自动截图比对，改为：
DOM 结构断言 + 提供 `/.preview/occlusion.html` 方案对照页由作者目视确认。

## 5. 变更清单

| 文件 | 变更 |
|------|------|
| `src/load.rs` | 新增 `parse_source_auto`（按 frontmatter `markup:` 解析） |
| `src/view/mod.rs` | 3 处解析改 `parse_source_auto` |
| `src/view/debug_page.rs` | 1 处同上 |
| `src/view/document.rs` | `split_occlusion` / `match_delim` / `render_occlusion_line` + 4 个单测 |
| `src/view/render.rs` | `occl_card` + 三个 `Expr::Occluded` 语句分支 |
| `src/view/html.rs` | `.doc-occl` / `.occl-*` 样式 |
| `doc/design/change-m8-view-occlusion.md` | 本文件 |
