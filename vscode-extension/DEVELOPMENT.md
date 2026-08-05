# 开发流程

## 环境

- Node.js 20+（LTS）
- VS Code 1.85+（`engines.vscode` 在 `package.json` 收紧）
- 本机已安装 **Marqdo CLI ≥ 0.1.0**（`marqdo --version`），用于 Run / catalog 集成

## 首次

```bash
git checkout vscode-extension
cd vscode-extension
npm install
```

按 F5（「运行扩展」）打开 Extension Development Host，打开任意 `.mq.md` 验证语言 id / 高亮。

## 日常脚本（约定）

| npm script | 用途 |
|------------|------|
| `npm run compile` | `tsc -p .` → `out/` |
| `npm run watch` | 监视编译 |
| `npm run lint` | 日后 ESLint（先可空） |
| `npm run package` | `vsce package` 打 `.vsix`（发布前） |

尚未加入的脚本：落地 `package.json` 时按上表补齐，勿引入与 Marqdo 语言仓抢戏的重型框架。

## 改语法 / 高亮

1. 对照语言仓 `doc/design/markdown-mapping.md`（以 `main` 为准）。  
2. 改 `syntaxes/*.tmLanguage.json`（或等价）。  
3. 用 `tests/` 与 `public/` 下真实 `.mq.md` 肉眼验收（Extension Host 打开语言仓 worktree）。

## 改 CLI 集成

- 子进程调用 `marqdo`，cwd = 工作区根或文件目录（与 `marqdo run` 导入/`lib/` 解析一致）。  
- 解析 stderr 的 `path:line:col: message` 写入 `DiagnosticCollection`。  
- **禁止**在扩展内重新实现解释器。

## 提交与推送

只在 **`vscode-extension` 分支** 提交本目录：

```bash
git status   # 确认在 vscode-extension
git add vscode-extension
git commit -m "…"
git push -u origin vscode-extension
```

不要把本目录合并进 `main`（`main` 的 `.gitignore` 会忽略它）。

## 验收清单（P0）

- [ ] `.mq.md` 打开后 Language Mode = Marqdo  
- [ ] `#` / `*` / `**` / `>` / 表格 / 注释行有可区分着色  
- [ ] 函数体内 `---` / `***`、空返回 `****` 不与普通 Markdown 粗体完全糊成一团（尽力）  
- [ ] F5 开发宿主可重复加载，无激活期未捕获异常  
