# Classic markup migration card (not default)

Use this card **only** when editing files that declare `markup: classic` / `标记: 经典`, or when the CLI is run with `--legacy-markup`. New documents must use Occlusion (see SKILL.md CORE SYNTAX).

| Classic (legacy) | Occlusion / current |
|---|---|
| `[key](map)` index read | `access(map, "key")` / `取(map, "key")` |
| `- [item](coll)` foreach by index form | `- [描述](遍历 coll 为 item)` |
| Teach `**…**` as “the return of print” | Print with `> print`; `**…**` is return/code expression |
| Default link = index | Default `[描述](payload)` = occlusion; `[]` never executes |

Do not place Classic rules in the front of the default Skill pack. After migration, remove `markup: classic` and verify with `marqdo run` / `marqdo check`.
