# `lib/table` 集合加厚 + `lib/cli`（Mid M5）

| | |
|---|---|
| 状态 | **已落地（M5）** |
| 日期 | 2026-09-07 |
| 相关 | [stdlib-mid.md](stdlib-mid.md) · [roadmap/stdlib-mid.md](../roadmap/stdlib-mid.md) |
| 集合 | 加厚现有 `lib/table` / `lib/表`（不另开 `listx`） |
| CLI | `lib/cli.mq.md` / `lib/命令行.mq.md` |

---

## 1. 集合新增

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `sort` | `排序` | `list` | 升序列表（int/num/text 优先；其它按显示文本） |
| `sort_by` | `按键排序` | `list`, `key` | 对 **map 列表**按键值排序 |
| `unique` | `去重` | `list` | 保序去重 |
| `chunk` | `分块` | `list`, `size` | 列表的列表 |
| `zip` | `拉链` | `a`, `b` | `[[a0,b0], …]`（取较短长度） |
| `flatten` | `展平` | `list` | 一层展平 |

`reverse` 已有。

## 2. CLI

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `parse` | `解析` | 可选 `args` | map：`--k v` / `--k=v` → 文本；裸 `--flag` → `True`；位置参 → 键 `_` 列表 |

`args` 缺省时用 `sys.args`（经宿主 `argv`）。

## 3. 刻意不做

稳定自定义比较器、惰性迭代器、完整 argparse 互斥组。
