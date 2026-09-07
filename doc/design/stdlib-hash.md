# `lib/hash` — 摘要与 HMAC（Mid M3）

| | |
|---|---|
| 状态 | **已落地（M3）** |
| 日期 | 2026-09-07 |
| 相关 | [stdlib-mid.md](stdlib-mid.md) · [stdlib-secrets.md](stdlib-secrets.md) |
| 导入 | `lib/hash.mq.md` · `lib/哈希.mq.md` |
| Host | `host_hash_*`（`sha2` / `sha1` / `md-5` / `hmac`） |

---

## 1. API

| 英文 | 中文 | 形参 | 结果 |
|------|------|------|------|
| `sha256` | `哈希256` | `text` | 小写 hex |
| `sha1` | `哈希1` | `text` | 小写 hex |
| `md5` | `md5` | `text` | 小写 hex |
| `hmac_sha256` | `密钥哈希256` | `key`, `text` | 小写 hex |

输入按 **UTF-8 字节**摘要。文档标明：用于完整性标识 / 缓存键，**不是**密码存储（无 Argon2/bcrypt）。

## 2. 刻意不做

密码哈希、流式大文件哈希 API、非 UTF-8 二进制类型。
