# Release task reference

Read [the executable release entry](index.mq.md). Version and mode come from the user; existing authorization is retained. Every task has its own input/return tables, effects, failure boundaries and recovery behavior.

| Need | Actual callable source |
|---|---|
| Inspect versions and checkout | local.mq.md inspect |
| Derive mode obligations | index.mq.md plan |
| Author local source/notes | ../marqdo-dev/files.mq.md read, patch, write or apply |
| Prepare versions/editor/docs | local.mq.md bump, extension and docs |
| Commit reviewed files/push refs | local.mq.md commit, push and tag |
| Observe exact-source CI/edit reviewed notes | github.mq.md status and notes |
| Verify GitHub bytes and package roles | assets.mq.md download |
| Publish verified R2 packs/check public parity | assets.mq.md publish and status |
| Build signed noble source/check actual apt | ubuntu.mq.md ship and status |
| Require every release obligation complete | index.mq.md finish |

A failed process, hook, HTTP response or pending CI remains visible. Inspect Result.ok before value. Use --require-ok when an external automation must stop on a failed Result; observational status may truthfully return complete=False. finish turns incomplete publication into a failed completion gate and retains its evidence.

Git retries use the documented proxy then direct route. A reviewed SSH/HK route can be supplied through MARQDO_RELEASE_GIT_REMOTE; no jump program or credential value belongs in a model argument. Existing tags and failed stages remain available for inspection. Do not force, retag or reset to make a check green. If sudo, GPG or dput needs human interaction, report the exact failure and give the concrete ubuntu.mq.md ship call for the already authorized version. An accepted orig archive must retain its original bytes.

Credentials stay in existing private stores. Do not publish from reading this note, deploy gh-pages without instruction, or call an asset list proof of green CI and apt publication.
