# VS Code 扩展（分支指针）

| | |
|---|---|
| 状态 | **已定分支模型** |
| 代码分支 | **`vscode-extension`**（同仓库） |
| 目录 | 工作区 `vscode-extension/`（**仅在该分支跟踪；`main` 在 `.gitignore` 忽略**） |
| 提交约定 | **[vscode-extension-commit.md](vscode-extension-commit.md)**（必读，避免误提交到 `main`） |

---

## 分工（类比 `public` / `gh-pages`）

| 分支 | 内容 |
|------|------|
| `main` | Marqdo 语言 / CLI / `public/*.mq.md`；**不**存放扩展实现 |
| `vscode-extension` | VS Code 扩展源码、TextMate 语法、VSIX 发版 |
| `gh-pages` | 用户站生成 HTML |

```bash
git checkout vscode-extension
cd vscode-extension && npm install
```

规范入口（在 **`vscode-extension` 分支**上阅读）：

- `vscode-extension/README.md` — 目标与目录  
- `vscode-extension/DEVELOPMENT.md` — 日常流程  
- `vscode-extension/ARCHITECTURE.md` — 与 CLI 边界  

扩展是 CLI 的薄宿主；语法真相仍是 [markdown-mapping.md](markdown-mapping.md)。
