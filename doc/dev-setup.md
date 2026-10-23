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
| GCC / Binutils | 原生依赖编译与链接 | `cc --version` / `ld --version` |
| Go | ≥ 1.22（仅构建 `plugins/web` 与 web 扩展测试需要） | `go version` |

安装 Rust（若尚未安装）：

- 官方：[https://rustup.rs/](https://rustup.rs/)
- Windows：下载 `rustup-init.exe`，按提示安装后**重新打开终端**

本仓库可选 `rust-toolchain.toml`（钉 1.81.0）。若本机无 `rustup`、但已有匹配的 `rustc`/`cargo`，也可直接开发。

**不需要：** Flex、Bison、LLVM。Node/npm 只在开发 VS Code 扩展时需要；核心 Rust 开发不依赖它们。

本仓库提供了固定 Rust 版本：`rust-toolchain.toml` 使用 Rust 1.81.0，并启用
`rustfmt` / `clippy`。在受限开发容器中，推荐先加载仓库环境脚本：

```bash
source scripts/dev-env.sh
```

脚本会优先使用 `target/toolchain/` 下的工具链，普通开发机则回退到 PATH 中已有的工具，
并把 Cargo、Go 缓存和临时目录放在工作区内。

---

## 2. 获取代码与构建

```bash
git clone https://github.com/cflmy/marqdo.git
cd marqdo

cargo build
cargo test
cargo run -- run tests/structure/hello.mq.md
```

构建 Go web 插件（完整扩展开发需要）：

```bash
bash scripts/build-web-plugin.sh
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

只验证 Rust 核心时可运行 `cargo test --locked --lib`；包含全部扩展的 gold 测试需要 Go，
并建议单线程运行以避免共享临时夹具互相影响：

```bash
cargo test --locked --test gold -- --test-threads=1
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

## 3.5 Git 推送（遇网络失败时使用代理）

直连 `git push` 偶尔会报 `RPC failed; curl 35 Recv failure: Connection reset by peer` 或 `curl 56`（远端挂断，常见于数据包稍大时）。此时**通过本机代理 7890 推送**：

```bash
# Linux/macOS（本机 clash 监听 127.0.0.1:7890）
git -c http.proxy=http://127.0.0.1:7890 -c http.postBuffer=524288000 push origin main
```

要点：

- 代理可用性先探测：`curl -x http://127.0.0.1:7890 -o /dev/null -w "%{http_code}\n" https://github.com`，返回 `200` 再推。
- `http.postBuffer=524288000`（500 MB）用于规避推送大包被断开（`curl 56`）；两个 `-c` 可叠加。
- 该命令**一次性生效**，不写死仓库配置。若想长期默认走代理，可执行：
  ```bash
  git config http.proxy http://127.0.0.1:7890
  git config http.postBuffer 524288000
  ```
- 若是别的代理端口，把 `7890` 换成实际端口（如 Windows 示例里 `$env:HTTPS_PROXY` 的同款端口）。

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
