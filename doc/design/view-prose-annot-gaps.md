# View：叙述面标注缺口（prose annot gaps）

| | |
|---|---|
| 状态 | **Active** |
| 日期 | 2026-10-15 |
| 相关 | [view-document-annotations.md](view-document-annotations.md) · [view-static-interactive.md](view-static-interactive.md) · [view-frame-selection.md](view-frame-selection.md) · [view-execution-trace.md](view-execution-trace.md) |

> **定位**：对既有文档面设计的**缺口专章**——写清用户可见的四类问题、根因（文件:行）与验收口径。不改写 ADR；不发明作者侧标注 DSL。

---

## 1. 为何「改了却几乎没变化」

| 原因 | 说明 |
|------|------|
| 错二进制 | PATH 上的 `marqdo` 可能仍是旧版（如 1.3.0）；改动在源码 / `./target/debug/marqdo`。验收必须用 `cargo build && ./target/debug/marqdo view …` |
| Chip 被溢出藏掉 | 同 band 堆多帧 return/print/who；`MAX_VISIBLE_ANNOTS=4` 把当前帧的 `who` 打成 `annot-chip-overflow`（`display:none`）；帧切换不重算溢出 |
| L-leader 几何错位 | `initAnnotLeaders` 里 `y0=0` 只在 `.annot-band` 顶画短折，**连不到**上方 underline；窄屏或 chip>4 时整段关掉 leaders |
| 旧静态快照 | gh-pages / 旧 `view output` 不会自动刷新 |

---

## 2. 目标视觉语法（既有，不新造）

锚定 [view-document-annotations.md](view-document-annotations.md) §3：

```
文本段…… `变量` …… **语句** …… *返回*
              ━━━━━
                │
                └──→ 结果标注 chip
```

- underline：`mark.mq-anchor.mq-{decl|code|return}`
- band：段落下方 `.annot-band` → leaders + chips
- L-leader：CSS `border-left` + `border-bottom`（禁止 SVG）
- 公式/代码：[view-static-interactive.md](view-static-interactive.md) G3 **一层主视觉**；卡下不叠 Bind dump
- 帧：[view-frame-selection.md](view-frame-selection.md) **源码常显、只滤 chip**；默认末帧

---

## 3. 管线

```mermaid
flowchart LR
  CLI["marqdo view / view output"] --> build["build_file_view"]
  build --> cap["run_file_capture"]
  cap --> annot["build_view_annots"]
  annot --> doc["document::render_module_document"]
  doc --> page["html.rs page_file"]
  page --> js["initAnnotLeaders + initCallPickers"]
```

| 步骤 | 位置 |
|------|------|
| CLI / 导出 | `src/main.rs` · `src/view/mod.rs` · `src/view/output.rs` |
| Capture + Call 事件 | `src/interp.rs` |
| Annot 组装 | `src/view/annot.rs` `build_view_annots` |
| 文档面 HTML | `src/view/document.rs` |
| 卡片 chrome | `src/view/render.rs` `render_*_assign_card` |
| CSS / leader JS | `src/view/html.rs` |

---

## 4. 四条用户问题 → 根因 → 验收

复现源：`public/00-welcome.mq.md` · `public/00-欢迎.mq.md`  
对照金样：`tests/view/document-annot-demo.mq.md` · `tests/markup-v03/inc-prose.mq.md`

### P1 — 叙述与代码同段时信息不显示

| | |
|---|---|
| 现象 | underline 在，chip / 连线弱或无 |
| 根因 | `html.rs` `initAnnotLeaders`：`y0=0` 相对 band；`document.rs` `MAX_VISIBLE_ANNOTS` 把当前帧 chip 溢出；PATH 旧 CLI |
| 目标 | 同 `inc-prose`：underline + L-leader（从锚点底到 chip）+ 可见 chip |
| 验收 | `./target/debug/marqdo view public/00-welcome.mq.md`：主文 `**greet…**` 与 `## greet` 体内均有可见 chip 与接到 underline 的 leader（宽屏） |

### P2 — 函数内变量不显示

| | |
|---|---|
| 现象 | `` `who` `` 有浅蓝 underline，无值 chip |
| 根因 | bind chip 存在但带 `annot-chip-overflow`；`applyCallFrame` 只 `hidden` 非选中帧，不解除当前帧 overflow |
| 目标 | 选中调用帧时常显 `who` · `print` · `return` 各一枚 |
| 验收 | 切换「调用」下拉，三枚 chip 随帧变且默认可见（无需点 +N） |

### P3 — ` ```text ` 渲染不一致

| | |
|---|---|
| 现象 | 变成内联 `text marqdo ext add…`，无卡片 |
| 根因 | Comment 围栏若进 `flush_para` → `join(" ")` → pulldown 当 inline code；应走 `render_narrative_fence` |
| 目标 | 与 bound python 同 `.code-card` chrome（badge + `pre.code-body`），**无** Run/foot |
| 验收 | welcome install 段：`data-lang="text"` + 两行 `marqdo ext add`；无 `<code>text marqdo` |

### P4 — 函数引用 / 代码卡未渲染

| | |
|---|---|
| 现象 | `**greet who=…**` 无结果；或 `hi` python 只见头/Run、不见源码 |
| 根因 | 同行多 Call 未与 BoldCode zip；或 Run 挤在 head；静态假 Run；错看 Debug 路径 |
| 目标 | 主文两 greet 各有 call chip（stdout 摘要）；`hi` = `render_code_assign_card` head→body→foot；静态藏 Run |
| 验收 | 两枚 greet mark 均有非空 `data-annot-id`；`code-body` 含 `hello-from-python`；`MARQDO_VIEW_LIVE=false` 无 Run 按钮 |

### call_id 语义（写死）

| 位置 | call_id |
|------|---------|
| 主文 `**greet…**` 的 call chip | 入口帧（`# main` 的 call）；结果为 stdout/返回摘要 |
| `## greet` 体内 print / who / return | callee 的 CallFrame id；跟函数标题旁 picker |

---

## 5. 修复约定（实现红线）

1. **帧感知密度**：band 内按选中 `call_id` 计可见 chip；默认末帧；同帧最多 4 枚常显；溢出只藏非当前帧或多余 kind。切换后客户端重算 overflow + `layoutAll()`。
2. **Leader 容器**：提升到 `.doc-block`（prose + band）；`y0` = 锚点底相对 block；按锚点 x 排序再画线。
3. **卡片**：只复用 Browse `render_*_assign_card` / `render_narrative_fence` + `html.rs` `.code-*` / `.formula-card`；本波不改 Debug AST 路径（除非回归红）。
4. **静态 Run**：`MARQDO_VIEW_LIVE=false` 时隐藏 Run，不留禁用假按钮（对齐 view-static-interactive D5）。
5. **非目标**：不做 debug 断点；不发明作者标注 DSL；不用 SVG leader；不单独发版/推 gh-pages（除非另嘱）。

---

## 6. 验收命令

```bash
cargo build -q
./target/debug/marqdo version   # 须为当前工作树版本
./target/debug/marqdo view public/00-welcome.mq.md --no-open
./target/debug/marqdo view tests/view/document-annot-demo.mq.md --no-open
cargo test -q --lib view::
```

---

## 7. 修订历史

| 日期 | 说明 |
|------|------|
| 2026-10-15 | 初稿：四问题、错二进制/溢出/leader 根因、帧感知密度与 leader 几何约定 |
| 2026-10-15 | L-leader 拥挤/消失：同跨度多帧 chip **共用 annot id**；可见 chip 按锚点 x 重排；leader 挂 doc-block |
| 2026-10-15 | L-leader 黑线碎段：JS `border*: "1.5px solid"` 无色覆盖 class → 改为 kind 色写入 shorthand；chip SSR/DOM 按锚点 L→R |
| 2026-10-15 | L-leader 改为 SVG：描述底边中点 → 轨道 → 短竖线 → 值 chip 顶边中点；chip 间距加宽 |
