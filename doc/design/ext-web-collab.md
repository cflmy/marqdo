# Collaborative document MVP

| | |
|---|---|
| 状态 | **Accepted · MVP (LWW)** |
| 日期 | 2026-09-17 |

## Approach

Not a full CRDT. Peers sync plain text over `route_ws mode=room` with:

- Client → server: `DOC\\t<body>`
- Server fan-out (existing G-WS1/2)
- Apply remote if body ≠ local; `applying` flag suppresses echo
- Presence JSON on `room.presence` (`presence=True`)

## Demo

`examples/web-collab-doc/` — open two browsers on the same room.
