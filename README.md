# Marqdo 生态集成

Marqdo 以 Markdown 编写可执行程序与可验证知识。这个仓库固定一组组件提交，维护组件责任、跨仓库验收和发行组合。

## 使用与开发

- 安装语言、CLI 和默认标准库，请使用 [Marqdo CLI 的官方发行](https://git.cflmy.cn/cflmy/marqdo-cli/releases)。各工具和扩展在各自仓库提供安装说明。
- 修改语言、标准库、领域扩展或工具，请先在 [组件目录](components.toml) 找到唯一责任仓库。开发和发布在该组件内独立进行。
- 验证生态组合，请初始化此仓库的子模块。Git tree 的 gitlink 固定参与验收的源码提交。

```sh
git clone https://git.cflmy.cn/cflmy/marqdo.git
cd marqdo
git submodule update --init
python3 scripts/check-layout.py --checkouts
```

## 组件与维护入口

| 目录 | 内容 |
| --- | --- |
| [cli](cli) | 语言实现、宿主原语和公共协议 |
| [tools](tools) | Knowledge、Prove、View、Lint，各自独立仓库 |
| [tests/conformance](tests/conformance) | 独立公共契约测试 |
| 标准库与扩展 | 九个 `marqdo-std-*`、十九个 `marqdo-ext-*` 已预建空仓，接入状态见迁移台账 |

- [官网与用户文档](https://git.cflmy.cn/cflmy/marqdo-site) 负责站点内容、静态构建与托管。
- [VS Code 扩展](https://git.cflmy.cn/cflmy/marqdo-vscode) 负责编辑器集成与 VSIX 发布。

[架构和归属](docs/ARCHITECTURE.md) · [开发、验收和发布](docs/DEVELOPMENT.md) · [迁移台账](docs/MIGRATION.md) · [贡献者和 Agent 指引](AGENTS.md)

v2 实现、历史文档和示例可从 Git 历史查阅。[v2.9.1 历史树](https://git.cflmy.cn/cflmy/marqdo/src/tag/v2.9.1)
