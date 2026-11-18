---
name: marqdo-release
description: Prepare, publish and verify a user-authorized Marqdo full, cli or extension release through its executable mq.md handbook. Use for release versioning, immutable tags, matching CI gates, GitHub/R2 parity and Ubuntu PPA publication.
---

Read [the executable release entry](../marqdo-release/index.mq.md) and use its task routing. Version and mode come from the user; an already stated choice remains authorized. Inspection of an old release does not authorize republishing it. Never infer a new version, replace a tag, force-push or bypass hooks.

Use `marqdo call` on the actual handbook headings. The repository development binary is `target/debug/marqdo`; prepare its engineering plugin through [the development entry](../marqdo-dev/index.mq.md). File reading and authoring use native [file tools](../marqdo-dev/files.mq.md). No Python release orchestrator is required. New OS/service capabilities belong at explicit plugin or external-tool boundaries; version decisions, retries and publication order remain readable mq.md.

The entry preserves full/cli/ext deliverables, separate editor sources, clean-main tagging, bilingual authored notes, exact-commit Release and EK gates, both native platforms, GitHub/R2 identical bytes, VERSION-after-ZIP ordering and Ubuntu source/build/apt evidence. Every applicable obligation must be verified before declaring completion. Default full/cli work includes PPA; gh-pages deployment needs its own instruction.

Read-only status and verification are separate from publication. Grant effects stated by each task. Inspect Result.ok before value; use --require-ok when an external automation must stop on a failed Result. Credentials stay in their existing private stores. Interactive sudo/GPG/dput or a failed service request remains incomplete: report the concrete failure and the exact handbook recovery call, preserving local evidence.

Independent extension releases commit reviewed bilingual notes at doc/releases/ext-X.Y.Z.md before tagging. The [pack handbook](../../skills/marqdo-release/packs.mq.md) and extension-release.yml run native staging, both platform builds, exact-tag release creation and non-clobbering gh asset upload through marqdo call. An existing upload is inspected rather than replaced. Windows execution remains a CI obligation; a local Linux ZIP cannot prove it.
