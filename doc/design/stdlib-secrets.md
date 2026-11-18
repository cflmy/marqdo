# `lib/secrets` — 安全随机令牌（Mid M3）

| | |
|---|---|
| 状态 | **已落地（M3）** |
| 日期 | 2026-09-07 |
| 相关 | [stdlib-hash.md](stdlib-hash.md) · [stdlib-mid.md](stdlib-mid.md) |
| 导入 | `lib/secrets.mq.md` · `lib/机密.mq.md` |
| Host | `host_secrets_*`（`getrandom`） |

---

## 1. API

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `token_hex` | `十六进制令牌` | 可选 `n`（字节数，默认 16） | 小写 hex，长度 `2*n` |
| `token_urlsafe` | `网址安全令牌` | 可选 `n`（字节数，默认 16） | URL-safe Base64（无填充） |

**不要**用 `math.random` / `math.seed` 生成令牌。

## 2. 刻意不做

密钥派生、加密/解密、密码哈希。
