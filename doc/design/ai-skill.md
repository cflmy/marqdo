# Marqdo AI Skill

| | |
|---|---|
| 状态 | 已提供 |
| 日期 | 2026-08-05（初版）· 2026-08-28（ext/web 完结复核 · ext/quantum Q8） |
| 路径 | [`skills/marqdo/`](../../skills/marqdo/)（权威）；Cursor 镜像 [`.cursor/skills/marqdo/`](../../.cursor/skills/marqdo/) |

---

## 目的

让接入方（Cursor Agent、其它支持 Agent Skills 的模型宿主）在**加载本 skill 后**即可按 v0.2 语法正确编写 / 修改 `.mq.md`，并在使用 **`ext/web` 动态站** 或 **`ext/quantum` 电路/可视化** 时遵循官方作者面（不混中英 API、不写 JSON 袋胶水；量子绘图须引用 `kind=`/`theme=` 字符串）。

## 内容

| 文件 | 用途 |
|------|------|
| `SKILL.md` | 必读：硬规则、标记表、**ext/web**、**ext/quantum（Q7/Q8）**、工作流、反模式 |
| `reference.md` | 按需：CLI、stdlib/ext 索引、web/quantum API 摘要 |
| `examples.md` | 按需：可复制样例（§13 最小站点 · §14 纠缠实验） |

## 使用

- **本仓库 Cursor**：项目 skill 已在 `.cursor/skills/marqdo/`；描述含 `.mq.md` / Marqdo 触发词，可自动选用。  
- **其它 AI**：将 `skills/marqdo/` 整目录作为 skill 包导入，或把 `SKILL.md` 贴进系统/项目说明并允许模型按需打开 `reference.md` / `examples.md`。  
- **校验**：生成后执行 `marqdo run <file>.mq.md`，按 `path:line:col` 诊断迭代。

语法真相仍以 [markdown-mapping.md](markdown-mapping.md) 与 [keywords.md](keywords.md) 为准；skill 是面向模型的压缩操作手册。
