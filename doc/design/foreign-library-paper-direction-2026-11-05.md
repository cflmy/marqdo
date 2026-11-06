# 外联库论文的研究方向

作者于 2026-11-05 将论文主线明确为：**叙述本身是程序，外部代码是值**。
Marqdo 的外联库用于在这一宿主模型中组合既有语言的计算能力，统一说明、控制流、
代码复用、状态与显式数据交换。论文应先解释 Notebook/文学编程中的实际设计问题，
再给出解决方案、可核验的收益、形式化依据和应用实验。

后续实现工作围绕绑定的代码值、参数化调用、持久会话、复制值和文档计划展开。
生命周期、值域和重放机制为这个统一设计提供保证；不能将重放检查器重新提升成
取代外联库与叙述式编程的论文主题。不得把未测的人因优势或未证明的实现正确性
写成既成事实。

完整方向文件在论文工作区：

- 方向（当前）：`MarqdoThesis/PLDI/design/V7-NARRATIVE-FIRST-DIRECTION-2026-11-05.md`
- v6 方向（历史）：`MarqdoThesis/PLDI/design/V6-NARRATIVE-FIRST-DIRECTION-2026-11-05.md`
- 证据映射：`MarqdoThesis/PLDI/design/V6-EVIDENCE-MAP-2026-11-05.md`
- 章节与页预算：`MarqdoThesis/PLDI/design/V6-OUTLINE-2026-11-05.md`

当前稿位于 `MarqdoThesis/PLDI/paper-v7`；v6 保留，历史 v5 及其证据保留。方向文件取代了
此前把论文缩成“外联会话值 + 因果重放”的 N3 路线：重放是支撑机制，不是主线。

2026-11-05 交付：`MarqdoThesis/PLDI/paper-v7/main.pdf`，正文（含 AI Use）20 页。
v6 保留为上一版尝试；更早的 v6 草稿保留在 `paper-v6-draft-2026-11-05/`。
