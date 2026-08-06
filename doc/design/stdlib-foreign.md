# 外联 / 胶水标准库

| | |
|---|---|
| 状态 | **已落地（v1）** |
| 日期 | 2026-08-06 |
| 相关 | [stdlib-modules.md](stdlib-modules.md) · [stdlib-math.md](stdlib-math.md)（公式同构绑定） |

---

## 1. 原则

**导入即用**：`lib/foreign` / `lib/外联`。解释器来自用户本机（PATH / `set_cmd` / `MARQDO_FOREIGN_*`）。live view 可点 Run；静态导出展示源码，Run 禁用。

---

## 2. 绑定语法（与公式同构）

```markdown
`hi` =
```python
print("hello")
```

*`out` = > run code=`hi` *
```

得到运行时类型 **`code`**（`lang` + `source`）。未绑定的 \`\`\` 围栏仍是叙述注释。

曾用的 \`\`\`python name=hi **已移除**。

---

## 3. API

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `set_cmd` | `设置命令` | `lang`, `cmd` | None |
| `run` | `运行` | `code`（code 值） | 文本 stdout |
| `run_lang` | `按语言运行` | `lang`, `source` | 文本 |
| `langs` | `语言表` | | 列表 |

live view：Structure 中 code 卡片带命令框 + Run → `POST /api/foreign-run`。

---

## 4. 一句话

**`name` = + \`\`\`lang 围栏 → code 值；显式 run；本机解释器可配置。**
