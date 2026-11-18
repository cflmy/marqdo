## 层归属（必填，见 doc/design/layers.md）

- [ ] Language（`src/lex` `src/parse` `src/ast` `src/interp` `src/diagnostics` / 契约）
- [ ] Runtime（`lib/` `plugins/` `crates/` catalog / view / debug / builtin）
- [ ] Document（`*.mq.md` `skills/` `ext/` `public/` `examples/`）

## 核心表面（见 doc/design/core-surface.md）

- [ ] 不涉及核心标记
- [ ] 涉及核心标记：ADR 编号 ______；core-surface.md 与 `CORE_CONSTRUCTS` 已同步（CI 守卫）

## 验收

- [ ] 相关验收（机器可验证）全绿
- [ ] 全量回归绿（`cargo test` / `go test ./...` 视触及面）
