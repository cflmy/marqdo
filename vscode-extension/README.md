# VS Code 扩展（分支工作区）

| | |
|---|---|
| 状态 | **P0 高亮 + P1 Run/Problems + 轻量 Debug 入口**（伴随 Marqdo **v0.1.0**） |
| 日期 | 2026-08-05 |
| 语言仓 | [`cflmy/marqdo`](https://github.com/cflmy/marqdo) `main` |
| 扩展仓形态 | **同仓库专用分支** `vscode-extension`（本目录）；`main` **不跟踪**本目录 |

---

## 1. 与 `public` / `gh-pages` 的类比

| | `public/` 用户站 | `vscode-extension/` |
|--|------------------|---------------------|
| **main** | 跟踪可执行 `.mq.md` 源 | **忽略**本目录（不进 `main` 历史） |
| **专用分支** | `gh-pages`：生成 HTML | **`vscode-extension`**：扩展源码与规范 |
| **消费者** | 浏览器 / Pages | VS Code / Open VSX / Marketplace（日后） |

开发扩展时：

```bash
git fetch origin
git checkout vscode-extension
cd vscode-extension
npm install
```

回到语言本体：

```bash
git checkout main
```

可选：`git worktree add ../marqdo-vscode vscode-extension`，语言与扩展并行目录开发。

---

## 2. 目标（对齐 v0.1.0 分发）

v0.1.0 已是「可解释 + view + debug CLI + catalog」的阶段性稳定小版本。扩展的第一阶段目标是 **编辑体验**，而不是一次做满 IDE：

| 优先级 | 能力 | 说明 |
|--------|------|------|
| **P0** | `.mq.md` 语言登记 + 基础语法高亮 | TextMate（已落地） |
| **P0** | 语言配置 | 括号 / 自动成对（已落地） |
| **P1** | 任务 / 命令 | `Marqdo: Run File` → `marqdo run`（已落地） |
| **P1** | 问题面板 | CLI `path:line:col` → Problems（已落地） |
| **P2** | 轻量语言特性 | 大纲（`#` 函数）、跳转到定义（同文件 / 导入） |
| **P3** | DAP / 调试 | 现已：`Marqdo: Open Debugger` 拉起 CLI debug 页；日后 DAP |
| **P3** | OKF / catalog | `Marqdo: Generate Catalog`（已落地轻量入口） |

原则：**扩展是 Marqdo CLI 的薄宿主**；语义以解释器与 [markdown-mapping.md](../doc/design/markdown-mapping.md) 为准，扩展不发明第二套语法。

---

## 3. 目录约定（本分支）

```
vscode-extension/
  README.md                 # 本文件：分支模型与目标
  DEVELOPMENT.md            # 日常开发流程、脚本、验收
  ARCHITECTURE.md           # 进程边界、与 CLI 的契约
  package.json              # 扩展清单（Marketplace id 待定）
  tsconfig.json
  .vscode/                  # F5：Run Extension
  .vscodeignore
  src/
    extension.ts            # activate：命令 / 诊断
    cli.ts                  # spawn marqdo
    diagnostics.ts          # path:line:col → Problems
  syntaxes/                 # TextMate 语法（P0）
    marqdo.tmLanguage.json
  language-configuration.json
```

生成物（`out/`、`node_modules/`、`*.vsix`）不入库；见本目录 `.gitignore`。

---

## 4. 命名与发布（待拍板）

| 项 | 建议（可改） |
|----|--------------|
| 扩展显示名 | Marqdo |
| `publisher` | `cflmy`（或组织账号） |
| `name`（包名） | `marqdo` |
| 语言 id | `marqdo` |
| 文件关联 | `*.mq.md` |

发布渠道：先私有 / Open VSX，再 VS Marketplace。版本号与语言仓 **不必锁死同一 semver**，但发行说明应写明「需要 Marqdo CLI ≥ 0.1.0」。

---

## 5. 文档索引（语言仓 `main`）

在 `main` 上仅保留指针，避免扩展实现泄漏进语言历史：

- [doc/design/vscode-extension.md](https://github.com/cflmy/marqdo/blob/main/doc/design/vscode-extension.md)（`main` 上的分支说明）
- 语法：[markdown-mapping.md](https://github.com/cflmy/marqdo/blob/main/doc/design/markdown-mapping.md)
- 诊断：[pipeline-debug.md](https://github.com/cflmy/marqdo/blob/main/doc/design/pipeline-debug.md)
- CLI debug：[view-debug.md](https://github.com/cflmy/marqdo/blob/main/doc/design/view-debug.md)
- Catalog / OKF：[catalog-cli.md](https://github.com/cflmy/marqdo/blob/main/doc/design/catalog-cli.md)

---

## 6. 一句话

**语言在 `main`；编辑器宿主在 `vscode-extension` 分支。先高亮与 run/诊断，再 LSP/DAP；始终以 CLI 为真相源。**
