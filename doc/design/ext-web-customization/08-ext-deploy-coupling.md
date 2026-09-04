# 解决 8：扩展与部署耦合

| | |
|---|---|
| 问题 | [研究 §8](../../research/ext-web-customization-limits.md#8-扩展与部署耦合) |
| 优先级 | **P3** · 文档 + 发布流程 |
| 触点 | `marqdo ext add web` · ABI 版本 · Docker/发布说明 · [ext-cli.md](../ext-cli.md) · [marqdo-release](../../../.cursor/skills/marqdo-release/SKILL.md) |

---

## 1. 目标

1. 站点作者**不必 fork 插件**才能完成 P0/P1 定制（把能力收进官方参数，见 01–05）。
2. 版本耦合可预期：changelog / 扩展清单标明「web 插件需 ≥ x.y.z」。
3. 升级路径文档化：二进制、`.marqdo-ext`、Docker 镜像三者对齐检查。

---

## 2. 作者 / 运维约定（草案）

| 约定 | 说明 |
|------|------|
| 定制优先走 `.mq.md` 开关 | `admin_prefix`、`shell_css`、`layout`、`gate`… 进官方 API 后，禁止文档再教「改 plugins/web」 |
| 兼容性矩阵 | 在 [web-net-capabilities.md](../web-net-capabilities.md) 或本目录维护「Marqdo ↔ libweb 最低版本」表 |
| `marqdo ext add web` | 安装后打印当前插件路径与版本；与 CLI 版本不符时警告 |
| Docker | 示例 Dockerfile 注释：升级需同时换 `marqdo` 与 `ext add` 产物 |
| 样式引号等行为变更 | 写入 CHANGELOG **Breaking / migration** 小节（如 0.3.2 T5） |

---

## 3. 工程改动点

| 改动 | 说明 |
|------|------|
| 插件/Crate 版本可见 | `web_listen` 或 `ext list` 显示 web ABI/包版本 |
| CI | 金样使用仓库内 `plugins/web`；发布技能检查 VSIX/CLI/插件三件套 |
| 研究→设计→路线图 | C0–C4 进 [ext-web roadmap](../../roadmap/ext-web.md)，避免只活在私有 fork |

---

## 4. 兼容策略

- 不承诺插件源码级稳定 API；承诺 **`.mq.md` 作者 API** 在小版本尽量兼容，破坏性变更进 CHANGELOG。
- Fork 插件仅作为「官方尚未交付」的临时手段，并在 issue/路线图留归还项。

---

## 5. 验收

| 场景 | 期望 |
|------|------|
| 升级说明文档 | 运维按清单升级后 web 金样通过 |
| C0–C2 落地 | 自研后台场景**无需**改 `plugins/web` 源码 |
| `ext list` / 文档 | 能核对版本 |

---

## 6. 过渡期 → 预编译路径（进行中）

**目标：** 用户 `marqdo ext add web` **不必**本机 Rust。

| 交付 | 说明 |
|------|------|
| Release `marqdo-*-native-*.zip` | Windows / Linux 预编译 `web`/`agent`/`quantum` |
| Windows portable zip | 内含 `ext/` + `ext/native/*.dll` |
| `ext add` 下载回退 | 本地无 `.so`/`.dll` 时从 GitHub Release 拉取（`MARQDO_EXT_NO_DOWNLOAD=1` 可关） |

仍保留开发者 `cargo build -p marqdo_plugin_*` 路径。详见 [ext-cli.md](../ext-cli.md)。
