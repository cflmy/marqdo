# browser-media（路线 F）

作者**零业务 JS**；`.mq.md` 用 **GFM 表格 + `lib/browser`**（禁止 `json.set` 链）。

| 能力 | 演示 |
|------|------|
| read_file | 文本 / DataURL |
| canvas | `@` 指令表 |
| audio | beep（Web Audio） |
| observe | Intersection + Resize |
| drag/drop | `data-drag` → `drop_text` |

```bash
cargo build -p marqdo --release
./target/release/marqdo wasm build -o examples/browser-media
cd examples/browser-media && python3 -m http.server 8767
```

写法约定见 [.cursor/skills/marqdo-dev/SKILL.md](../../.cursor/skills/marqdo-dev/SKILL.md)。
路线图：[browser-wasm-f.md](../../doc/roadmap/browser-wasm-f.md)。
