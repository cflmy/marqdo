# 开发环境配置（Rust 参考解释器）

| | |
|---|---|
| 状态 | M0 |
| 相关 | [dependencies.md](design/dependencies.md) · [interpreter.md](roadmap/interpreter.md) |

---

## 1. 必备软件

| 软件 | 要求 | 检查 |
|------|------|------|
| Rust + Cargo | stable，建议 ≥ 1.81（本机验证过 1.81） | `rustc --version` / `cargo --version` |
| Git | 任意近期版本 | `git --version` |

安装 Rust（若尚未安装）：

- 官方：[https://rustup.rs/](https://rustup.rs/)
- Windows：下载 `rustup-init.exe`，按提示安装后**重新打开终端**

本仓库可选 `rust-toolchain.toml`（钉 1.81.0）。若本机无 `rustup`、但已有匹配的 `rustc`/`cargo`，也可直接开发。

**不需要：** Flex、Bison、LLVM、Node/npm（实现栈为 Rust）。

---

## 2. 获取代码与构建

```bash
git clone https://github.com/cflmy/marqdo.git
cd marqdo

cargo build
cargo test
cargo run -- run tests/structure/hello.mq.md
```

当前 M0：`run` 会读入文件并明确报错「evaluation not implemented」（脚手架，非静默假成功）。

> 若 `cargo build` 报 `edition2024` / 过新 crate：仓库已对 `clap` 等做兼容钉扎（见 `Cargo.toml`）；请使用附带的 `Cargo.lock`，勿盲目 `cargo update` 到最新 major。

---

## 3. 推荐组件

```bash
rustup component add rustfmt clippy   # 若使用 rustup
cargo fmt
cargo clippy --all-targets
```

代理下拉 crate（可选，与 git 推送类似）：

```bash
# Windows PowerShell 示例
$env:HTTPS_PROXY="http://127.0.0.1:7890"
$env:HTTP_PROXY="http://127.0.0.1:7890"
cargo build
```

或配置 `%USERPROFILE%\.cargo\config.toml`：

```toml
[http]
proxy = "http://127.0.0.1:7890"
```

---

## 4. 目录约定

| 路径 | 含义 |
|------|------|
| `src/lex` | 行分类 / 词法（自研） |
| `src/parse` | 递归下降语法 |
| `src/diagnostics` | 诊断 |
| `tests/` | 金样例 + 集成测试（`structure` / `keywords` / `errors`） |
| `public/` | 用户可执行文档（无 errors） |
| `spike/` | 旧 Python 风险探测，非运行时依赖 |
| `doc/` | 设计与路线图 |

---

## 5. 下一里程碑

见 [interpreter.md](roadmap/interpreter.md)：M1 词法·行分类与最小 AST。
