# Marqdo executable reference

The maintained programming rules and runnable examples live in [the executable programming guide](../../doc/library-development.mq.md). Read its parameter and return contracts, then call the actual source heading. [The development entry](../marqdo-dev/index.mq.md) provides native file operations and verification tasks; [the release entry](../marqdo-release/index.mq.md) provides publication workflows.

Use [the recursive Markdown web example](../../examples/web-layout-handbook/index.mq.md) and [the quantum workflow](../../examples/quantum-entanglement/index.mq.md) for domain development. Their real source is the manual. [Classic migration context](migration-classic.md) remains for explicitly historical syntax; new programs use readable occlusion and access(collection, key). Do not copy a second fenced program from this discovery note.

## Theorem tables (`marqdo prove`, CLI 2.8.0+)

A knowledge function may carry a theorem table before its first executable
statement. Place it right under the parameter list:

| 定理 | 结论 | 前提 | 策略 |
|---|---|---|---|
| 大额必经人工 | 包含(轨迹, "工单.创建") | 金额 > 10000 | auto |

- `结论` grammar: `结果.ok = True/False`, `包含(轨迹, "操作名")`,
  `长度(轨迹) ⋈ n`, `参数 ⋈ 常量`, combined with 且/或/非. `轨迹` names the
  operation trace by the link's display text; `结果` is the returned value.
- `前提` restricts the claim's scope (same comparison grammar). Keep it a
  single-variable linear condition.
- `策略` is `auto` (generated proof script) or `none` (theorem generated but
  not proven — honest `unverified`).

Judge with `marqdo prove 文件.mq.md --json`. Statuses, strongest first:
`proved` (Lean confirms) > `checked` (offline symbolic execution closed the
path — advisory, never a proof) > `unverified` (outside the provable subset)
> `rejected` (subset check refused) > `unproved` (counterexample found, e.g.
`金额 = 10001`) > `error`. Without a Lean toolchain the run stays honest at
`checked`/`unverified` and exits 0; `--require-lean` makes unjudged statuses
exit 2 for CI. Never weaken a failing claim to make it pass — fix the claim
or the code, and let `proof.divergence` stop the line when the offline
judger and Lean disagree. Full contract: [design/prove-cli.md](../../doc/design/prove-cli.md).
