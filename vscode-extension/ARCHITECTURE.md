# 架构（规范）

## 进程边界

```
┌─────────────────────────────┐
│  VS Code Extension Host     │
│  · 语言配置 / TextMate      │
│  · 命令、任务、Problems     │
│  · （日后）LSP client       │
│  · （日后）DAP client       │
└─────────────┬───────────────┘
              │ spawn / stdio
              ▼
┌─────────────────────────────┐
│  marqdo CLI（已安装）         │
│  · run / catalog / debug    │
│  · 诊断文案与行号真相        │
└─────────────────────────────┘
```

扩展 **不嵌入** Rust 解释器；分发上用户安装 CLI（或日后可选打包 sidecar，仍属后置决策）。

## 语言 id

- VS Code `languageId`: `marqdo`
- 文件：`*.mq.md`（必须保留 `.md` 后缀以便通用预览；用 `filename` / `filenamePattern` 关联，避免抢全部 Markdown）

## 与 Markdown 扩展共存

`.mq.md` 同时是 Markdown。策略：

1. **默认**用 Marqdo 语言模式（执行语义优先）。  
2. 提供命令「以 Markdown 打开预览」调用内置 Markdown preview（不切换语言 id 也可预览时，优先不破坏高亮）。  
3. 不禁用用户其它 Markdown 扩展；冲突时文档说明如何 `files.associations`。

## 诊断契约

CLI 用户可见失败：

```text
path:line:col: message
```

扩展将相对路径解析到工作区 URI，`severity` 为 0-based。与语言仓 P0 诊断模型一致。

## 调试（P3）

- 短期：命令「在外部打开 `marqdo debug`」或 Simple Browser 打开本地 debug URL。  
- 中期：DAP 适配器进程包装现有 debug session API（`src/view/debug_api.rs` 语义），避免第三套断点模型。

## OKF / catalog（P3）

- 命令触发 `marqdo catalog <workspace> -o .marqdo`。  
- 生成物已在语言仓 `.gitignore`（`.marqdo/`）；扩展只负责调用与打开 `index.md` / `catalog.yaml`。

## 非目标（本阶段）

- 在扩展内实现完整树遍历 / 字节码  
- 替代 `marqdo view` 静态站  
- 强制用户登录或云服务  
