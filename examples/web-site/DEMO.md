# Demo walkthrough (`examples/web-site`)

Aligned with The Web Conference **Demo** expectations: implemented system,
**hands-on** interaction, clear venue script, reproducible from the repo.

## Run

```bash
./scripts/build-web-plugin.sh
marqdo run examples/web-site/index.mq.md
```

Open http://127.0.0.1:18081/

## Venue script (≈3 minutes)

| Step | Action | What reviewers should notice |
|------|--------|------------------------------|
| 1 | Home | Claim badges + steps; **8 seed cards** with summaries |
| 2 | Click a card | `/post/{slug}` detail from the same table (`detail=True`) |
| 3 | Open `index.mq.md` beside the browser | Narrative = program; field/rule/route/intro tables |
| 4 | `/new` empty title → submit | Server validation from **rule table** (no author JS) |
| 5 | `/new` valid title+slug → submit | Row appears on Home and Admin |
| 6 | `/admin` | List + create form — table-declared admin, not an SPA |
| 7 | Terminal check | `find examples/web-site \( -name '*.js' -o -name '*.css' \)` → **0** |

## Authoring contract

| Surface | Source |
|---------|--------|
| Routes / forms / admin | GFM tables in `index.mq.md` |
| Styles | `styles/shell.mq.md` → `web.make_style` (ink / wine / gold; `shell_css=off`) |
| Intro | `| front | back | style |` → `page.compose_intro` (same bind as main) |
| Fonts | `page.head` table → Google Fonts (Noto Serif SC + Ma Shan Zheng) |
| Author `.js` / `.css` | **None** |

## Video

Same beats as above; English captions preferred. See thesis
`www-2027-demo/VIDEO.md` for length / upload tips.
