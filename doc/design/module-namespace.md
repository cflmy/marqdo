# 模块命名空间（import / 点号寻址）

| | |
|---|---|
| 状态 | **Accepted；统一 `import bind:target` 已锁定** |
| 日期 | 2026-08-13 |
| 相关 | [stdlib-modules.md](stdlib-modules.md) · [stdlib-i18n.md](stdlib-i18n.md) · [markdown-mapping.md](markdown-mapping.md) · [objects.md](objects.md) · [call-arguments.md](call-arguments.md) · [ext-agent.md](ext-agent.md) |

---

## 1. 动机

加载器曾把每个导入文件的顶层 `#` / `##` **扁平合并**进同一全局函数表；同名则**后导入静默覆盖**。

后果：

- `lib/json` 的 `parse` 与 `lib/time` 的 `parse` 不能共存。
- 官方 ext / 多库编排无法按大型语言习惯书写 `time.parse` / `json.parse`。
- 旧 frontmatter `import path:path.mq.md` 对 Markdown / 编辑器不友好，且与调用标记 `>` 抢外形。

**结论**：模块命名空间 + 点号寻址；frontmatter 导入统一为可读的 `import bind:target`。

---

## 2. 目标与非目标

### 2.1 目标

1. **导入绑定库名**：导入一个 `.mq.md` 得到一个**库（模块）绑定**，成员不倒进全局。
2. **点号寻址调用**：`库.成员`、`库.对象.成员`；`.` 表示定义树寻址。
3. **库名不是变量**：路径段为**裸标识符**，不加反引号。
4. **短名与文件导入同形**：`import fmt:time.format`，废除独立 `use` 关键字。
5. **中英对等**：`import` / `导入`。

### 2.2 非目标

- 第三方包管理 / 版本求解。
- 旧 `> path.mq.md` / `> use` 双轨兼容。
- `` `time`.parse ``（库名加反引号）。
- `import fmt:time.*` 通配。

---

## 3. 核心语义

### 3.1 模块 / 库

| 内容 | 归属 |
|------|------|
| 顶层 `##` 自由函数 | 库根下的**函数** |
| 顶层 `#` 对象 | 库根下的**类型（对象）** |
| 类型下的 `##` 方法 | 该对象节点下的**成员函数** |
| 被导入文件的 `# main` | **不**作为入口执行 |

### 3.2 导入句法（frontmatter）

仍只出现在文件头成对 `---` 内。**调用**仍用 `> print`（不变）。

| 含义 | 英文 | 中文 |
|------|------|------|
| 导入文件 | `import utils:utils.mq.md` | `导入 utils:utils.mq.md` |
| 标准库 | `import json:lib/json.mq.md` | `导入 json:lib/json.mq.md` |
| 短名（成员） | `import fmt:time.format` | `导入 fmt:时间.格式化` |

规则：

- **绑定名必填**，在 `:` 左侧（裸名，无反引号）。
- `:` 右侧 `target`：
  - 以 `.mq.md` 结尾 → **文件导入**；
  - 否则为 **点号路径**（≥2 段），把该成员绑为本地短名（须先有对应文件导入）。
- 同一文件内绑定名不得冲突（文件库名与短名共用命名空间）→ 硬错误。
- 旧写法 `> ….mq.md`、`> … as …`、`> use …`、`> 使用 …` → **硬错误**，并提示新句法。

### 3.3 废除的旧规则

不再支持：

- 省略绑定名时用文件名茎作库名；
- 导入侧 `as` / `作为`；
- 独立关键字 `use` / `使用`。

### 3.4 调用解析顺序

1. **点号路径 callee**（§4）→ 按段在导入库 / 类型树上寻址。  
2. **当前文件**顶层 `#` / `##` 与 frontmatter **短名绑定**。  
3. **L0 内置**。  
4. **插件注册名**。  
5. 否则 → 未知 callee。

---

## 4. 点号寻址（核心外形）

### 4.1 文法

```text
path_callee ::= ident ('.' ident)+
```

### 4.2 二段：`库.成员`

```markdown
---
import time:lib/time.mq.md
import json:lib/json.mq.md
---

# main

*t = > time.now_unix*
*s = > time.format unix=`t` pattern="%Y-%m-%d"*
*obj = > json.parse text={"a":1}*
*a = > json.get value=`obj` key="a"*
> print text=`s`
```

### 4.3 三段：`库.对象.成员`

```markdown
> agent.agent          # 库 → 类型：构造
```

实例方法须 `` > `var`.method ``；无接收者的 `库.类型.方法` → 硬错误。

### 4.4 变量 / 实例 vs 库路径（Python 统一命名空间）

| 写法 | 含义 |
|------|------|
| `time.parse` | 库路径（裸名，第一段非变量） |
| `` > `助手`.step `` | 独立 `>` 调用：实例方法（接收者反引号） |
| `*回复 = > 助手.step …*` | `*…*` 段内：若 `助手` 是变量 → 实例方法；否则 → 库路径 |

**统一规则**（与 Python 一致）：对裸点号 `a.b`，当 `a` 是当前作用域**局部变量**时解释为**方法调用**（`a` 为接收者）；否则解释为**库路径**（`lib.a.b`）。此规则同时应用于树遍历与字节码两后端。

- 在**独立 `>` 调用**中裸词 = 文本，故方法接收者仍须反引号：`` > `助手`.step ``。
- 在 **`*…*` / `**…**` 段内**裸词 = 变量，接收者不加反引号：`*回复 = 助手.step …*`。

### 4.5 短名示例

```markdown
---
import time:lib/time.mq.md
import fmt:time.format
---

# main

*t = > time.now_unix*
*s = > fmt unix=`t` pattern="%Y-%m-%d"*
```

### 4.6 应用侧完整示例

```markdown
---
import time:lib/time.mq.md
import writeback:lib/writeback.mq.md
import agent:ext/ai/agent.mq.md
import json:lib/json.mq.md
---

# main

*`助手` = > agent.agent model=… tools=… *
*`out` = > `助手`.step task=… *
*`body` = > json.stringify value=`out` *
> writeback.record value=`body` key=ok
```

---

## 5. 导出面

默认：库内顶层 `#` / `##` 均可被路径访问。本版不引入 `private`。

---

## 6. 与旧行为的关系（破坏性变更）

| 旧 | 新 |
|----|----|
| `import time:lib/time.mq.md` | `import time:lib/time.mq.md` |
| `import t`:lib/time.mq.md | `import t:lib/time.mq.md` |
| `import fmt`:time.format
| 茎名默认库名 | 绑定名始终显式 |
| 扁平合并进全局 | 仅库绑定 + 可选短名 |

不提供旧 `>` 导入过渡开关。

---

## 7. 对分层与 ext 的影响

| 层 | 影响 |
|----|------|
| L0 内置 | 仍为全局短名 |
| L1 / 官方 ext | 经库路径调用；**`ext/**/*.mq.md` 严禁 `host_*`** |
| ABI 插件 | 插件注册表 |

---

## 8. 实现要点

1. frontmatter 只解析 `import` / `导入` 行；右侧分流文件 / 成员。  
2. AST 仍用 `Import`（文件）与 `Use`（短名）存储，对外无 `use` 关键字。  
3. 金样例：`tests/structure/import/`、`tests/structure/ns/`。

| 阶段 | 内容 |
|------|------|
| M1+M2 | 点号路径 + 短名 — 已落地 |
| **Import 句法** | 统一 `import bind:target` — **本波次** |

---

## 9. 决议摘要

1. 导入 = `import bind:target`（`导入` 等价）。  
2. target = `.mq.md` 文件 **或** `库.成员` 点号路径。  
3. 调用以裸名点号路径寻址；实例方法用反引号接收者（仅独立 `>` 调用；`*…*`/`**…**` 段内接收者不加反引号）。  
4. 裸点号 `a.b`：`a` 为局部变量 → 方法调用，否则 → 库路径（Python 统一命名空间）。
