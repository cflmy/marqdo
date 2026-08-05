# 外联 / 胶水标准库（候选）

| | |
|---|---|
| 状态 | **草案 / 暂缓**（本波不实现；五库之后再议） |
| 日期 | 2026-08-05 |
| 相关 | [stdlib-modules.md](stdlib-modules.md) · [code-vs-comment.md](code-vs-comment.md) · [markdown-mapping.md](markdown-mapping.md) · [stdlib-math.md](stdlib-math.md) |
| 灵感 | Markdown \`\`\`lang 代码块；Marqdo 作编排层 |

---

## 1. 为什么值得做

[code-vs-comment.md](code-vs-comment.md) 已划出**外联面**：\`\`\`lang 是「其它语言」。若运行时能：

1. **收集**本文件（或导入图）中的围栏代码块；  
2. 按语言 **subprocess / 嵌入运行时**执行；  
3. 把 stdout / 约定返回值 **接回 Marqdo 值**；  

则 Marqdo 成为**文学化胶水语言**：结构、叙述、分支循环用 Marqdo，重活交给 Python/JS/Shell/SQL。

风险：安全（任意代码）、可复现（环境漂移）、view/CI（无解释器）——故为**候选**，默认关闭。

---

## 2. 与叙述面的边界

| 面 | 行为 |
|----|------|
| 叙述段里的 \`\`\` | 今日：注释；**外联库启用后仍可配置为「可调用」或「仅展示」** |
| 可执行面附近的 \`\`\` | 建议：**具名块**才可调用，避免误跑文档里的示例 |

推荐：**只有带名字的围栏**进入外联目录。

---

## 3. 命名围栏语法（草案）

三种候选（实现前锁定一种）：

### 3.1 信息串后缀（推荐试水）

````markdown
```python name=normalize
def normalize(s):
    return s.strip().lower()
```
````

或：

````markdown
```python#normalize
…
```
````

### 3.2 HTML 注释锚点

````markdown
<!-- foreign:normalize -->
```python
…
```
````

### 3.3 紧邻 Marqdo 声明

```markdown
*`normalize` = foreign_def lang=python *

```python
…
```
```

**倾向 3.1**：仍是合法 Markdown，多数渲染器忽略未知 info 字段；与「文档即源」一致。

未命名围栏：view 可语法高亮展示，**不可** `foreign_call`。

---

## 4. 导入与 API

| 导入 | API 语言 |
|------|----------|
| `> lib/foreign.mq.md` | 英文 |
| `> lib/外联.mq.md` | 中文 |

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `run` | `运行` | `name`, 可选 `stdin` / `名`, `标准输入` | 文本（stdout） |
| `run_lang` | `按语言运行` | `lang`, `source`, 可选 `stdin` | 文本 |
| `call` | `调用` | `name`, 可选 `args`（JSON 文本） | 文本 |
| `langs` | `语言表` | （无） | 已注册语言列表 |

`call` 约定（v1）：

- 宿主把 `args` 经环境变量或临时 JSON 文件传给子进程；  
- 被调代码打印**一行 JSON** 到 stdout 作为返回值；  
- 或退出码非 0 → Marqdo 诊断。

更紧的 Python 嵌入（`pyo3`）列为后期优化，**v1 只用子进程**，便于推理安全边界。

---

## 5. 语言支持矩阵（v1）

| 语言 | 探测 | 备注 |
|------|------|------|
| `python` / `python3` | `PATH` | 优先实现；服务科学计算与 [stdlib-math.md](stdlib-math.md) CAS 外置 |
| `node` / `javascript` | `PATH` | 第二 |
| `sh` / `bash` | Unix；Windows 可用 Git Bash 或跳过 | 高风险，默认更严 |
| `cmd` / `powershell` | Windows | 可选 |
| 其它 | 配置表 | `marqdo.toml` / 环境变量注册解释器路径 |

无解释器时：清晰诊断 `foreign language not available: python`，而非静默跳过。

---

## 6. 权限模型（硬性）

| 场景 | 默认 |
|------|------|
| `marqdo run` | **拒绝**，除非 `--allow-foreign` |
| `marqdo view` | 拒绝（或仅 `--allow-foreign` 的本地演示） |
| `view output` / CI 用户站 | **拒绝**（文档示例用录制输出或纯 Marqdo） |
| 金样例 | 用 `tests/foreign/` + 环境探测；无 Python 则 `ignore` |

附加约束（可组合）：

- `--foreign-lang python,node` 白名单；  
- 超时（默认 5–30s）；  
- 工作目录锁在文件所在根；  
- 无网络（与 `--allow-net` 分离；子进程环境可剥 `HTTP_PROXY` 等——尽力而为）。

---

## 7. 典型胶水故事

```markdown
---
title: Normalize names
> lib/foreign.mq.md
> lib/fs.mq.md
---

```python name=normalize
import sys
print(sys.stdin.read().strip().lower())
```

# main

*`raw` = > read_text path=names.txt *
*`clean` = > run name=normalize stdin=`raw` *
> print text=`clean`
```

中文轨：

```markdown
---
> lib/外联.mq.md
> lib/文件.mq.md
---
```

叙述说明用中文；围栏内语言仍是 Python——**外联块语言 ≠ Marqdo API 语言**。

---

## 8. 与数学库的关系

- 厚 CAS（SymPy）**优先**走外联，而不是塞进默认 `lib/math`。  
- 官方可提供可选示例：`examples/math-sympy.mq.md`（要求 Python + `--allow-foreign`）。  
- 数值与 SVG 仍留在数学库，保证无 Python 时可教可演示。

---

## 9. 是否定为「标准库」的决策标准

纳入默认 `lib/foreign` 当且仅当：

1. 权限模型在 CLI/view/CI 测全；  
2. 至少一种语言金样例在 CI 稳定（或正式 `#[ignore]` 策略）；  
3. 文档明确「胶水能力默认关闭，不是网页 RCE」。

若短期内无法保证安全叙事 → 放在 `lib/experimental/foreign` 或 feature `foreign`，用户站不放可点击「运行外联」的按钮。

---

## 10. 实现分期

| 期 | 内容 |
|----|------|
| F0 | 词法：收集命名围栏 → 模块表（尚不执行） |
| F1 | `run` + Python 子进程 + 权限旗标 |
| F2 | `call` JSON 约定；超时与 cwd 锁 |
| F3 | Node；view 展示块与「需 CLI 权限」提示 |
| F4 | 可选嵌入运行时、与 math-cas 示例合流 |

---

## 11. 一句话

**\`\`\`lang 是外联面的源，不是自动执行的陷阱；具名块 + 显式导入 + 默认拒绝权限，才能把 Marqdo 做成安全可讲的胶水语言。**
