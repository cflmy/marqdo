# 官方扩展：`ext/web` 动态网站

| | |
|---|---|
| 状态 | **Accepted · W0–W7 + P3 完结**（SQLite/Postgres 作者面：页面装配 / db CRUD+`where` / form / 嵌入主区 / `app.route` / 路由 `/_part` / admin UI / RBAC / 上传相册 / SEO·RSS·sitemap；Postgres 等见 §9） |
| 日期 | 2026-08-11 |
| 相关 | [markdown-mapping.md](markdown-mapping.md) · [module-namespace.md](module-namespace.md) · [objects.md](objects.md) · [ext-abi.md](ext-abi.md) · [ext-cli.md](ext-cli.md) · [stdlib-i18n.md](stdlib-i18n.md) |
| 安装（目标） | `marqdo ext add web`（中英：`web` / `网页`） |
| 本文目的 | 锁定**作者面**与**类 API**；指导实现 |

---

## 0. 一句话

`ext/web` 用 **GFM 表格 + Marqdo 类方法** 描述动态网站：表仍是字典/列表嵌套；类方法把表装配成可渲染页面。  
**不改 Marqdo 核心语法**；扩展库存在的理由，正是让「文档即代码」在网站场景下成立。

---

## 1. 为何要做网络扩展库

### 1.1 问题

若没有专用扩展，作者只能：

- 手写 JSON 袋、`json.set` 拼装组件/样式；或  
- 要求改语言（让表格单元格求值、点号取值等）。

二者都破坏 Marqdo 的「代码即文档」。

### 1.2 答案

在 **`ext/web/web.mq.md`（英文）** 与 **`ext/web/网页.mq.md`（中文）** 中各提供一套 **`#` 类**：

- 作者只写**表** + **少量实例方法调用**；  
- 表单元格保持**字面量**；  
- **寻址字符串**由类方法解析；  
- **导入英文文件 → 只用英文 API；导入中文文件 → 只用中文 API**（见 [stdlib-i18n.md](stdlib-i18n.md)）。

### 1.3 硬约束（评审锁定）

| # | 约束 |
|---|------|
| C1 | **禁止**为 web 改动 Marqdo 核心：单元格求值、import、点号值读取等保持不变 |
| C2 | 作者面**禁止** `json.parse` / `json.set` 袋胶水、手写 part JSON、手写 assemble 袋 |
| C3 | 能力以 **`#` 类 + `##` 方法** 暴露；**英文库文件只用英文名，中文库文件只用中文名**（见 [stdlib-i18n.md](stdlib-i18n.md)）；同一 `.mq.md` 内禁止中英 API 混排 |
| C4 | 首页在 **`index.mq.md`**：页面表在此；可复用片段进 `components/`；主体可就地写 |
| C5 | **旧有网络扩展库全部废弃**：`plugins/web`、`ext/web/**`（含现网示例/gold 中依赖旧 API 者）评审通过后**删除或清空后重写**；禁止在旧代码上打补丁演进 |
| C6 | `# db` / `# 数据库` 必须提供完整**增删改查**；列表查询英文方法名为 **`select`**（中文 **`查询`**）；后台 `/admin` 与表单提交走同一套写库 API |
| C7 | 用户输入经 `# form` / `# 表单`：字段表 + 校验表；**服务端校验必须在写库前执行**（见 §5.5） |

---

## 2. 作者心智与目录

```text
myapp/
  index.mq.md              # 入口 + 首页：页面表 +（可选）就地主体表 + 类方法 + listen
  pages/                   # 子页：同形的页面表（导出布局表或由入口再装配）
  components/              # 可复用组件：导出与路径同名的 ##，体为 |属性|值|样式|
  styles/                  # 样式模块：每个样式一个 ##，体为 |属性|值|
  db/                      # 库：schema 表 +（可选）open/种子
  data/                    # sqlite 等运行时数据（gitignore）
```

| 表种 | 列 | 一句话 |
|------|-----|--------|
| **页面表** | `组件` \| `样式` | 本页用哪些组件、各套哪套样式 |
| **组件 / 主体表** | `属性` \| `值` \| `样式` | 前端属性 ↔ 取值（库字段或字面量）↔ 样式名 |
| **样式表** | `属性` \| `值` | CSS 声明 |
| **库结构表** | `字段` \| `类型` \| `可空` | schema |

---

## 3. 表内寻址约定（单元格仍是字符串）

单元格**不求值**。下列写法是 **web 扩展约定的路径文本**，由 `# 页面` / `# 样式` / `# 数据库` 的方法解析。

### 3.1 反引号与裸名

| 含义 | 写法 | 例 |
|------|------|-----|
| 要「去导入树 / 定义里解析」的名字 | 反引号 `` `名` `` | `` `nav` ``、`` `title` ``、`` `topnav` ``、`` `articles` ``（表名） |
| 字面量（文案、URL、CSS 属性名/值、SQL 类型、**列名**） | **裸**或 `"…"` | `首页`、`/about`、`font-size`、`integer`、`title`（作为列名时） |

### 3.2 路径形态（锁定）

| 场景 | 形态 | 例 | 方法内语义 |
|------|------|-----|------------|
| 引用导入模块的导出 | `模块.`导出`` | `nav.`nav``、`shell.`topnav`` | 在**站点入口模块**的 import 树上调用 `模块.导出`（零参 `##`），取回表 Value |
| 引用库字段 | `模块.`表`.列` | `articles.`articles`.title` | 模块名 = 命名空间（对应导入的 db 文件）；`` `表` `` = 库表名；**列名裸写**（列不是变量）→ 运行时绑定 `表.列` |
| 前端属性名 | `` `title` `` | 组件表「属性」列 | 解析为字段名 `title` |
| 样式引用 | 同「模块.导出」或纯样式名 | `shell.`card-title`` | 取回 `|属性|值|` 表并编译为 CSS 类 |

**刻意不做的事：**

- 不把「组件变量名」魔法等同于「库表名」；  
- 不要求作者写 `json.set` 才能把 `nav` 放进袋；  
- 不把 `articles.`articles`.title` 做成语言级表达式。

### 3.3 与模块命名空间的关系

- 库名来自 frontmatter 导入（茎名或 `as`），**裸名**，见 [module-namespace.md](module-namespace.md)。  
- `nav.`nav`` 中：前段 `nav` = 导入库名；`` `nav` `` = 该库上的 `## nav` 导出。  
- 组件/样式文件应导出 **与路径第二段同名的 `##`**，直接返回表（不要 `## bag` / `## bind` 袋）。

---

## 4. 规范示例：`index.mq.md`（作者面唯一样板）

**语言与导入绑定（锁定，纠正手写混用）：**

| 导入 | 库名（默认茎） | 类 / 方法 / 形参 |
|------|----------------|------------------|
| `import web:ext/web/web.mq.md` | `web` | **仅英文**：`page` · `compose_components` · `db` · `app` · `components=` … |
| `导入 网页:ext/web/网页.mq.md` | `网页` | **仅中文**：`页面` · `组件装配` · `数据库` · `应用` · `组件=` … |

禁止：`import web:ext/web/web.mq.md` 后写 `web.页面` / `` `页`.组件装配 ``（英文库混中文成员）。  
禁止：在 `web.mq.md` 文件内定义中文 `## 组件装配` 别名。中英对等靠**两个文件**，不靠同一文件双份导出。

### 4.1 英文面（导入 `web.mq.md`）

```markdown
---
title: web-site
description: Home = page table; main may be inline; class methods assemble.
import web:ext/web/web.mq.md
import shell:styles/shell.mq.md
import nav:components/nav.mq.md
import side:components/side.mq.md
import foot:components/foot.mq.md
import articles:db/articles.mq.md
import db:db/index.mq.md
---

# main

`home` =

| 组件 | 样式 |
|------|------|
| nav.`nav` | shell.`topnav` |
| side.`side` | shell.`side-panel` |
| foot.`foot` | |

Main content is not a reusable component, so it is authored here.

`index` =

| 属性 | 值 | 样式 |
|------|-----|------|
| `title` | articles.`articles`.title | shell.`card-title` |
| `body` | articles.`articles`.body | shell.`card-body` |

*`store` = > db.open *
*`page` = > web.page title="Marqdo Web Site" intro="<h1>Web Site</h1>" *
*`page` = > `page`.compose_components components=`home` *
*`page` = > `page`.compose_main main=`index` *
*`app` = > web.app page=`page` db=`store` admin=True host=127.0.0.1 port=18081 *
> `app`.listen
```

说明：表头列名可用中文（`组件`/`样式`/`属性`/`值`）——那是**表约定词汇**，不是库成员名；库成员必须与导入语言一致。

### 4.2 中文面（导入 `网页.mq.md`）

```markdown
---
title: 站点
导入 网页:ext/web/网页.mq.md
import shell:styles/shell.mq.md
import nav:components/nav.mq.md
import side:components/side.mq.md
import foot:components/foot.mq.md
import articles:db/articles.mq.md
import db:db/index.mq.md
---

# main

`首页` =

| 组件 | 样式 |
|------|------|
| nav.`nav` | shell.`topnav` |
| side.`side` | shell.`side-panel` |
| foot.`foot` | |

主体不是可复用组件，因此就地撰写。

`index` =

| 属性 | 值 | 样式 |
|------|-----|------|
| `title` | articles.`articles`.title | shell.`card-title` |
| `body` | articles.`articles`.body | shell.`card-body` |

*`库` = > db.open *
*`页` = > 网页.页面 标题="站点" 引言="<h1>站点</h1>" *
*`页` = > `页`.组件装配 组件=`首页` *
*`页` = > `页`.主体装配 主体=`index` *
*`应用` = > 网页.应用 页面=`页` 数据库=`库` 后台=True 主机=127.0.0.1 端口=18081 *
> `应用`.监听
```

**允许出现在 index 的非表内容：** frontmatter 导入、开库一行、构造页面/应用并装配、`listen` / `监听`。  
**不允许：** `json.*` 袋、手写 `/_part` 注册、中英 API 混用。

---

## 5. 类 API（分文件 · 分语言）

依据 [stdlib-i18n.md](stdlib-i18n.md)：**导入哪个文件，就用哪套名字。**  
`ext/web/web.mq.md` = 全英文；`ext/web/网页.mq.md` = 全中文。中文实现可委托插件同一套原语，或薄封装英文库，但**对外导出的 `#` / `##` / 形参名必须是中文**。

### 5.1 页面 · `page` / `页面`

**英文（`web.mq.md`）：**

```markdown
# page
    + `title`=…
    + `intro`=…

## compose_components
    + `components`     # page table |组件|样式|

## compose_main
    + `main`           # bind table |属性|值|样式|
```

**中文（`网页.mq.md`）：**

```markdown
# 页面
    + `标题`=…
    + `引言`=…

## 组件装配
    + `组件`

## 主体装配
    + `主体`
```

| 行为 | 说明 |
|------|------|
| compose_components / 组件装配 | 解析组件·样式路径 → 填 nav/sidebar/footer → CSS → 登记 part |
| compose_main / 主体装配 | 解析库字段与样式路径 → 填 main → 登记 part（如 id=`index`） |

**槽位推断：** 由组件导出名决定（`nav`→顶栏，`side`→侧栏，`foot`→底栏；其余→main）。

### 5.2 样式 · `style` / `样式`

**英文：** `# style` · `## process` · `+ style=` / `name=` / `path=`  
**中文：** `# 样式` · `## 样式处理` · `+ 样式=` / `名=` / `路径=`  

页面装配可内部调用；作者一般不必在 index 显式调。

### 5.3 数据库 · `db` / `数据库`

完整**增删改查**；页面绑定、种子、`/admin`、表单提交共用。

**英文（`web.mq.md`）：**

```markdown
# db
    + `url`=…

## init
    + `name`
    + `fields`       # or table= schema

## insert
    + `table`
    + `rows`

## select
    + `table`
    + `where`=None   # optional simple filters
    + `limit`=200

## get
    + `table`
    + `id`

## update
    + `table`
    + `id`
    + `row`

## delete
    + `table`
    + `id`

## exec
    + `sql`
    + `args`=None    # escape hatch: raw SQL (not everyday authoring)
```

**中文（`网页.mq.md`）：**

```markdown
# 数据库
    + `地址`=…

## 初始化
    + `数据库表`     # 或 名= + 字段=

## 插入
    + `表`
    + `行`           # 或 多行=

## 查询
    + `表`
    + `条件`=None
    + `上限`=200

## 获取
    + `表`
    + `id`

## 更新
    + `表`
    + `id`
    + `行`

## 删除
    + `表`
    + `id`

## 执行
    + `sql`
    + `参数`=None
```

| 中文 | 英文 | 职责 |
|------|------|------|
| 初始化 | `init` | 建表；登记表名 |
| 插入 | `insert` | 增 |
| 查询 | `select` | 查列表（可带简单条件 / limit） |
| 获取 | `get` | 按主键一行 |
| 更新 | `update` | 改 |
| 删除 | `delete` | 删 |
| 执行 | `exec` | 可选原始 SQL（进阶） |

`table=` / `表=` 接受表名或句柄；行数据优先 GFM 表。  
**命名锁定：** 列表查询用 `select` / `查询`，**不用** `all`；原始 SQL 用 `exec` / `执行`，避免与 `select` 混淆。

### 5.4 应用 · `app` / `应用`

**英文：** `# app` · `## route` · `## static` · `## listen` · 形参 `page` `db` `admin` `host` `port`  
**中文：** `# 应用` · `## 路由` · `## 静态` · `## 监听` · 形参 `页面` `数据库` `后台` `主机` `端口`

`listen` / `监听`：加载插件、注入 `db_url`、按页面 part 提供 `/_part/{id}`（首页）与 `{path}/_part/{id}`（`app.route` 页）、可选 `static` 目录挂载、启动 HTTP。作者不必手写 part 注册。

`static` / `静态`：`dir=`（或 `目录=`）+ 可选 `mount=`（默认 `/static`）；`listen` 时用 `tower-http` 提供只读文件服务。

### 5.5 表单 · `form` / `表单`（规划锁定）

用户输入也走「表 + 类方法」，与页面/库同一心智；**提交最终调用 `# db` 的 insert/update**，不另开旁路。

**英文（`web.mq.md`）：**

```markdown
# form
    + `table`=None      # bound db table name (optional until submit)
    + `action`=insert   # insert | update
    + `id`=None         # required when action=update

## fields
    + `fields`          # |字段|标签|类型|必填|默认|…

## validate
    + `rules`=None      # |字段|规则|消息|…  (optional extra rules)
    + `data`            # submitted row / map

## render
    # → HTML form (or form descriptor for page slot)

## submit
    + `data`
    + `db`              # db handle
```

**中文（`网页.mq.md`）：**

```markdown
# 表单
    + `表`=None
    + `动作`=插入        # 插入 | 更新
    + `id`=None

## 字段
    + `字段`

## 校验
    + `规则`=None
    + `数据`

## 渲染

## 提交
    + `数据`
    + `数据库`
```

#### 5.5.1 字段表（作者面）

```markdown
`文章表单` =

| 字段 | 标签 | 类型 | 必填 | 默认 |
|------|------|------|------|------|
| title | 标题 | text | true | |
| body | 正文 | textarea | false | |
```

| 列 | 含义 |
|----|------|
| `字段` | 对应库列名（裸名或 `` `列` ``，与 schema 对齐） |
| `标签` | 展示文案（字面量） |
| `类型` | 控件/值类型：`text` `textarea` `number` `email` `url` `checkbox` `select` … |
| `必填` | 是否必填 |
| `默认` | 可选默认值 |

#### 5.5.2 校验表（作者面）

```markdown
`文章校验` =

| 字段 | 规则 | 消息 |
|------|------|------|
| title | required | 标题不能为空 |
| title | max:120 | 标题过长 |
| body | max:8000 | 正文过长 |
```

| 规则（初版） | 含义 |
|--------------|------|
| `required` | 非空 |
| `min:N` / `max:N` | 字符串长度或数值范围 |
| `email` / `url` | 格式 |
| `match:字段` | 与另一字段相等（如确认密码） |
| `in:a,b,c` | 枚举 |

schema 上的「可空 / 类型」在 **submit 时自动并入校验**（不必重复写 `required`，但校验表可覆盖消息）。

#### 5.5.3 行为流程

```text
render  → 按字段表出 HTML（或嵌入 page 主区）
用户填写 → POST /_form/{id} 或同页提交
validate → 合并 schema + 校验表；失败则带回字段级错误、不写库
submit  → action=insert → db.insert；action=update → db.update
        → 成功后刷新槽位或跳转（由 form 选项约定）
```

| 层 | 校验 |
|----|------|
| **服务端（必须）** | `validate` 在写库前执行；唯一可信边界 |
| **浏览器（可选增强）** | 同源规则可生成 `required`/`maxlength` 等 HTML 属性；**不能**替代服务端 |

`/admin` 的新建/编辑页 = 内置 form：字段来自 schema，校验来自类型/可空；与站点自定义 form **同一套** `validate` + `db.insert`/`update`。

#### 5.5.4 与页面的关系

- 表单可以是独立路由页，或页面表里的一个「组件」导出（仍用 `|属性|值|样式|` 描述展示，提交走 form 类）。  
- **`page.compose_form id=… form=…`**：把 form 嵌入该页主区；`listen` 从首页与路由页收集表单并挂载 `/_form/{id}`（`app.mount_form` 仍可用）。校验失败时若表单有所属页，则带回该页壳重渲染。

### 5.6 文件职责

| 文件 | 内容 |
|------|------|
| `ext/web/web.mq.md` | 英文 `# page` `# style` `# db` `# form` `# app` |
| `ext/web/网页.mq.md` | 中文 `# 页面` `# 样式` `# 数据库` `# 表单` `# 应用`（可包装英文实现，**不得**把中文名写进 `web.mq.md`） |

---

## 6. 模块导出形状（配合寻址）

### 6.1 组件 `components/nav.mq.md`

```markdown
## nav

`nav` =

| 属性 | 值 | 样式 |
|------|-----|------|
| `title` | `nav`.`title` | |
| `href` | `nav`.`href` | |

**`nav`**
```

说明：组件**内部**绑定仍可用 `` `表`.`列` ``（表名+列名皆引用）。与 index 主体里「模块命名空间 + 裸列名」的 `articles.`articles`.title` 分工：

- **跨文件、带导入命名空间** → `模块.`表`.列`；  
- **组件文件内、表已明确** → `` `表`.`列` `` 即可。

（若评审希望全站统一为一种形态，可在评审意见中二选一；默认允许上述两种。）

### 6.2 样式 `styles/shell.mq.md`

每个样式一个 `##`，**禁止** `## bag` + `json.set`：

```markdown
## topnav
… |属性|值| …
**`topnav`**

## card-title
…
```

### 6.3 库 `db/articles.mq.md` / `db/index.mq.md`

- `articles`：`## schema`（或同名导出）返回结构表；  
- `index`：`## open` → 确保插件、建库、init、幂等种子，返回 `# 数据库` 句柄。

---

## 7. 运行时职责划分

```text
作者 .mq.md 表（字面量单元格）
        │
        ▼
  ext/web 类方法（Marqdo）
        │  解析路径；需要时经 ABI host_query
        │  调用入口模块 import 树上的 lib.member
        ▼
  plugins/web（HTTP / SQL / HTML 壳）
        │
        ▼
  浏览器：四区布局 + /_part 局部刷新 + /admin（可选）
```

| 层 | 做 | 不做 |
|----|----|------|
| Marqdo 核心 | 表→Value、类/方法、import | 不识别 web 路径语法 |
| `ext/web/*.mq.md` | 作者 API、中英面、委托插件 | 不直接 `host_*` |
| `plugins/web` | 路径解析辅助、装配结果、listen、SQL、渲染 | 不要求作者写袋 |

**实现要点（评审通过后再做）：**

1. ABI `host_query("call_lib_path")`：在**站点入口模块**上执行 `lib.member`；  
2. 插件函数由类方法调用（如 compose_components / compose_main）；  
3. `listen` 合并页面内 part，注入 `db_url`。

---

## 8. 全部废弃 · 从头开发（锁定）

**旧有网络扩展库内容一律作废**，包括但不限于：

| 范围 | 处理 |
|------|------|
| [`plugins/web/`](../../plugins/web/) | 评审通过后**删除或清空**，按本文重写 cdylib |
| [`ext/web/`](../../ext/web/)（`web.mq.md` / `网页.mq.md` / templates） | 同上，**不**在旧 L1 上打补丁 |
| 依赖旧 API 的示例（如当前 `examples/web-site`、`web-demo`、`man-write-site` 可跑胶水版） | 按 §4 规范重写或暂时移除 |
| 依赖旧 API 的 gold / 单测 | 删除后按新 API 重写 |
| 本文之前的袋式 / assemble / wire / `## bag` 等作者面 | 永久移出正典 |

**禁止**：在遗留代码上「修到能跑」再文档化。  
**允许**：实现时参考旧代码中的 HTTP/SQLite 技术经验，但**新树另起**，不以 ABI 表面或作者面兼容为目标。

历史摸索中明确不再回归的作者面模式：

- `json.parse` / `json.set` 拼 components/styles 袋；  
- 手写 `web.assemble` + 袋参数；  
- 手写 part JSON + `` `app`.part ``；  
- `|前端变量|后端数据库|` 填路由；  
- 「变量名即表名」魔法默认表；  
- 样式 `## bag`；掏空 index 只留 `wire.boot`。

---

## 9. 非本波次（可记入路线图）

- Postgres / Redis / S3 驱动细节（SQLite 已落地）；  
- 复杂权限与多租户；  
- 子页路由糖的更多约定（`# 应用.route` 已够用）。

---

## 10. 评审清单

请确认或批注：

1. **§3 路径**：`模块.`导出`` 与 `模块.`表`.列`（列裸名）是否定为唯一正典？组件内 `` `表`.`列` `` 是否保留？  
2. **§4 index**：开库是否允许单独一行 `` db.open ``，还是必须进一步藏进 `web.应用`？  
3. **§5 分文件**：英文 `web.mq.md` / 中文 `网页.mq.md` 是否严格分语言、禁止混排？  
4. **§5.3**：列表查询用 `select` / `查询`、原始 SQL 用 `exec` / `执行` 是否认可？  
5. **§5.5 表单/校验**：字段表、校验表、服务端必校、admin 共用是否认可？  
6. **§5 槽位**：仅由组件导出名推断，是否足够？  
7. **§8 废弃**：是否确认**全部推倒重来**、不兼容旧实现？  

评审通过后：冻结本文为 Accepted → **清空旧 web 扩展代码** → 按本文从零实现（骨架 → db CRUD/`select` → form/validate → 页面装配 → app listen → 规范示例 → gold）。
