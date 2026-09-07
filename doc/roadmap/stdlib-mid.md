# 路线图：中层标准库（数个版本）

| | |
|---|---|
| 状态 | **M1–M4 已落地 · M5–M6 规划中** |
| 日期 | 2026-09-07 |
| 设计 | [stdlib-mid.md](../design/stdlib-mid.md) |
| 相关 | [stdlib-modules.md](../design/stdlib-modules.md) · [next-phase.md](next-phase.md) · [ext-cli.md](../design/ext-cli.md) |

> **不预标 SemVer。** 每一波随当时产品发版合入；下文用 **M1…M6** 波次名。每波须可单独演示、可回滚。

---

## 0. 总序

```text
M1  文本与编解码地基     re + encoding          ← 最先解锁「无插件写库」
M2  路径与文件中层       path + fs 加厚
M3  完整性与机密         hash + secrets
M4  表格数据与文本加厚   csv + text++
M5  集合与 CLI 糖        table/listx + cli
M6  可观测与标识         log + uuid（+ 可选 yaml）
```

原则：

1. **先原语、后糖**：正则 / 编解码 / 路径先于 CLI 糖与日志格式。  
2. **每波必须有金样 + public 短页**；无文档不算完成。  
3. **并行约束**：不阻塞 `ext/linalg` 等已定领域工作；但新官方能力评审优先问 Mid。  
4. **插件冻结规则**：自 M1 起，官方新增「通用」ABI 插件默认 **拒绝**，除非设计文档论证无法用 host+L1。

---

## 1. 现状缺口（开工依据）

| 能力 | 现状 | 后果 |
|------|------|------|
| 正则 | 无 | 校验、替换、简单解析只能 foreign 或插件 |
| base64 / hex | 无（仅有 `url_encode`） | 令牌、摘要、嵌二进制文本困难 |
| 路径代数 | `fs` 只做 I/O | 拼接靠字符串，跨平台易错 |
| 哈希 / HMAC | 无 | 校验、缓存键、签名只能外联 |
| CSV | 无 | 数据脚本推向外联 Python |
| `lib/text` | 3 个包装函数 | 「有 text 库」名不副实 |
| 安全随机 | 仅 `math.random` | 易误用于 token |
| 结构化日志 / flags | 无 | 每个脚本自造 |

对照：Go 日常高频包（`regexp`、`encoding/*`、`crypto`、`path/filepath`、`slices`）与 Python「难写对才进 stdlib」——正是 M1–M5 的选型来源；**不**把 HTTP 服务端、数据库驱动塞进 Mid（见设计文 §3.2）。

---

## 2. 波次明细

### M1 — 正则与编解码（最高优先） — **done**

| 交付 | 说明 |
|------|------|
| `lib/re.mq.md` / `lib/正则.mq.md` | `is_match` / `find` / `find_all` / `replace` / `split` |
| `lib/encoding.mq.md` / `lib/编码.mq.md` | `base64_*` / `hex_*` |
| Host | `host_re_*`（`regex`）、`host_encoding_*`（无第三方编解码 crate） |
| 金样 | `tests/lib/re-smoke` · `正则-烟测` · `encoding-smoke` · `编码-烟测` |
| 文档 | [stdlib-re.md](../design/stdlib-re.md) · [stdlib-encoding.md](../design/stdlib-encoding.md) · public `08-re-encoding` |
| 验收 | 无插件：邮箱式校验 + base64/hex 往返 |

**刻意不做：** 回溯引用极致兼容、替换回调、完整 PCRE 方言文档化——先 RE2/ Rust `regex` 语义并写清。

---

### M2 — 路径与文件中层 — **done**

| 交付 | 说明 |
|------|------|
| `lib/path.mq.md` / `lib/路径.mq.md` | `join` / `split` / `file_name` / `parent` / `extension` / `normalize` / `is_absolute` |
| `lib/fs` 加厚 | `copy_file` / `move` / `make_temp`（文本边界；无 `read_bytes`） |
| Host | `host_path_*`、`host_copy_file` / `host_move` / `host_make_temp` |
| 金样 | `path-smoke` · `路径-烟测` · `fs-copy-move` · `文件-复制` |
| 文档 | [stdlib-path.md](../design/stdlib-path.md) · [stdlib-fs-mid.md](../design/stdlib-fs-mid.md) · public `09-path-fs` |
| 验收 | 规范化 + 沙箱内复制/移动 |

**刻意不做：** 完整 `shutil`、监视 `inotify`、权限 ACL 全家桶。

---

### M3 — 哈希与机密 — **done**

| 交付 | 说明 |
|------|------|
| `lib/hash.mq.md` / `lib/哈希.mq.md` | `sha256` / `sha1` / `md5` / `hmac_sha256` |
| `lib/secrets.mq.md` / `lib/机密.mq.md` | `token_hex` / `token_urlsafe` |
| Host | `host_hash_*`、`host_secrets_*`（`sha2`/`sha1`/`md-5`/`hmac`/`getrandom`） |
| 金样 | `hash-smoke` · `哈希-烟测` · `secrets-smoke` · `机密-烟测` |
| 文档 | [stdlib-hash.md](../design/stdlib-hash.md) · [stdlib-secrets.md](../design/stdlib-secrets.md) · public `10-hash-secrets` |
| 验收 | 已知向量 + 令牌长度 |

**刻意不做：** 密码哈希（Argon2/bcrypt）、完整 TLS、证书管理——仍属服务端/安全产品。

---

### M4 — CSV 与文本加厚 — **done**

| 交付 | 说明 |
|------|------|
| `lib/csv.mq.md` / `lib/逗号表.mq.md` | `parse` / `stringify`（首行表头 → list of maps） |
| `lib/text` / `文本` 加厚 | `contains` / `starts_with` / `ends_with` / `replace` / `to_upper`/`to_lower` / `repeat` / `pad` |
| Host | `host_csv_*`、`host_text_*` |
| 金样 | `csv-smoke` · `逗号表-烟测` · `text-mid-smoke` · `文本-加厚-烟测` |
| 文档 | [stdlib-csv.md](../design/stdlib-csv.md) · [stdlib-text-mid.md](../design/stdlib-text-mid.md) · public `11-csv-text` |
| 验收 | CSV 往返 + 文本清洗小例 |

**可选同波：** 极简 `format`（`{name}` 替换），若与引用语法冲突则延后。

---

### M5 — 集合算法与 CLI

| 交付 | 说明 |
|------|------|
| `table` 或 `listx` | `sort` / `sort_by` / `unique` / `reverse`（已有 host 则接线）/ `chunk` / `zip` / `flatten` |
| `lib/cli.mq.md` / `lib/命令行.mq.md` | 从 `sys.args` 解析 `--key value` / `--flag`；纯 mq 优先 |
| 验收 | 一个「过滤 CSV + 排序 + 打印」脚本零插件；一个带 `--input` 的 CLI 例 |

**刻意不做：** 泛型迭代器协议、惰性流、完整 argparse 互斥组。

---

### M6 — 日志、UUID、收口

| 交付 | 说明 |
|------|------|
| `lib/log.mq.md` / `lib/日志.mq.md` | `debug`/`info`/`warn`/`error`；level 过滤；一行文本（可选 JSON 一行） |
| `lib/uuid.mq.md` / `lib/标识.mq.md` | `v4` |
| 文档收口 | 更新 `stdlib-modules.md` 总表；设计文状态 → **M1–M6 已落地**（按实际） |
| 可选 | `yaml` 只读子集——**默认不做**，除非 catalog 强需求且依赖可接受 |

---

## 3. 跨波次工程清单

每波共用：

- [ ] 中英 L1 对称（[stdlib-i18n.md](../design/stdlib-i18n.md)）  
- [ ] `host_*` 形参在 L1 **全量转发**（[stdlib-modules.md](../design/stdlib-modules.md) §2.1）  
- [ ] 双后端金样（tree + bytecode，若该 host 已接线）  
- [ ] WASM：标注 `host-only` 或实现 stub  
- [ ] CHANGELOG「标准库」条目  
- [ ] skill `examples.md` 增加 Mid 惯用写法（GFM 优先）  

宿主膨胀守卫：

- 新增 crate 须在 PR/设计中写清：**为何不能纯 mq**、**features 如何裁剪**、**许可证**。  
- 禁止借 Mid 把 `ext/web` 的服务端栈链进主二进制。

---

## 4. 与官方扩展的互动（减插件）

| 扩展 | Mid 落地后可做的减负 |
|------|----------------------|
| `ext/web` | 路由/校验用 `re`；ETag/缓存键用 `hash`；静态路径用 `path` |
| `ext/agent` | 工具参数校验、密钥材料用 `secrets`/`hash`（领域核仍插件） |
| `ext/ai/llm` | 头/体编解码走 `encoding` |
| `ext/linalg` / `quantum` | **不**迁入 Mid；继续插件；仅共用 `path`/`hash` 等通用面 |

跟踪方式：某波完成后，开短 issue/清单「ext X 可删除的原生辅助」，不强迫同波大改。

---

## 5. 建议人力切分（示意）

| 波次 | 大致体量 | 可演示产物 |
|------|----------|------------|
| M1 | 中 | 「正则替换 + base64 往返」用户页 |
| M2 | 中 | 「安全拼接路径写文件」 |
| M3 | 小–中 | 「下载内容 sha256 校验」 |
| M4 | 中 | 「CSV → 表 → 打印」 |
| M5 | 小–中 | 「零插件数据 CLI」 |
| M6 | 小 | 「带 log level 的脚本」 |

若只能开一条线：**M1 → M3 → M2 → M4 → M5 → M6**（完整性校验常比路径代数更刚需时，M3 可紧贴 M1）。

---

## 6. 完成定义（整条 Mid 叙事）

当 M1–M5 完成时，对外可宣称：

> Marqdo 自带中层标准库：正则、编解码、路径、哈希、CSV、加厚文本与集合/CLI 糖；**写通用脚本与多数扩展库不再需要原生插件。**  
> 插件保留给 web 运行时、智能体、量子与线性代数等**领域引擎**。

M6 为体验与可观测收口，不阻塞上述宣称。

---

## 7. 一句话

**按 M1→M6 把通用原语沉进 `lib/`，用数个发版周期换掉「事事插件」——领域 `ext/` 继续尖，语言本体变宽。**
