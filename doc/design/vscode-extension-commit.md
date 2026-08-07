# VS Code 扩展 — 分支与提交约定

| | |
|---|---|
| 状态 | **强制约定**（勿在 `main` 提交扩展源码） |
| 扩展分支 | **`vscode-extension`** |
| 设计说明 | [vscode-extension.md](vscode-extension.md) |

---

## 1. 原则（必读）

| 分支 | 是否跟踪 `vscode-extension/` |
|------|-------------------------------|
| **`main`** | **否** — `.gitignore` 含 `/vscode-extension/` |
| **`vscode-extension`** | **是** — 扩展唯一源码所在分支 |

**禁止**在 `main` 上：

- 删除 `.gitignore` 里的 `/vscode-extension/`
- `git add vscode-extension/`
- 把扩展目录「合并进 main」以便发版

发版时从 **`vscode-extension` 分支检出**扩展目录打 VSIX（见 §4），**不是**把扩展提交进 `main`。

---

## 2. 日常开发（扩展）

```bash
git fetch origin
git checkout vscode-extension
cd vscode-extension && npm ci && npm run compile
```

改语法、CLI 检测、扩展版本 → **只在 `vscode-extension` 分支提交**：

```bash
git add vscode-extension/
git commit -m "vscode-extension: <简要说明>"
git push origin vscode-extension
```

---

## 3. 日常开发（CLI / 语言）

在 **`main`** 上改 `src/`、`lib/`、`public/`、`tests/` 等；**不要**顺带提交 `vscode-extension/`。

若本地为了调试 checkout 过扩展目录，工作区里可以有 `vscode-extension/` 文件夹，但应被 ignore，且 **`git status` 不应出现**该目录下文件。

---

## 4. 发版（CLI tag + VSIX）

1. **`main`**：打 tag（如 `v0.1.2`），CI 构建 `marqdo.exe` 等。
2. **VSIX**：Release 工作流在 tag 构建时执行：
   ```bash
   git fetch origin vscode-extension
   git checkout origin/vscode-extension -- vscode-extension
   cd vscode-extension && npm ci && npm run compile && npx @vscode/vsce package
   ```
3. 本地脚本 `scripts/release-windows.ps1` 同样假定：先 `git checkout origin/vscode-extension -- vscode-extension`（或已在 `vscode-extension` 分支），再打 VSIX。

扩展版本号与最低 CLI 要求写在 **`vscode-extension/package.json`** 与分支 README，**不要**在 `main` 的 CHANGELOG 里冒充已合并扩展源码。

---

## 5. 给 Agent / 协作者

执行「更新 VS Code 插件」类任务时：

1. **切到或基于 `vscode-extension` 分支**改 `vscode-extension/**`
2. **不要**移除 `main` 上 `.gitignore` 的 `/vscode-extension/`
3. **不要**在 `main` 的 release 提交里包含 `vscode-extension/package-lock.json` 等大文件
4. 仅需在 `main` 更新时：改 `doc/design/vscode-extension.md`、Release 说明、**本文件** — 指针文档即可

---

## 6. 历史说明

v0.1.2 准备阶段曾误将 `vscode-extension/` 并入 `main`（并删除 ignore），导致推送体积过大。已恢复：**扩展仅 `vscode-extension` 分支**；`main` 只保留本约定与设计指针。
