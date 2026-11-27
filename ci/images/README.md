# Shared Marqdo CI images

The ecosystem publishes exactly these reusable runtime images to the Gitea
container registry, per released CLI version:

| Image | Contents | Consumers |
| --- | --- | --- |
| `git.cflmy.cn/cflmy/marqdo-ci-shell:3.0.0-rc.5` | Ubuntu 24.04 (glibc 2.39), git/curl/python3/build tools, released CLI + Bundle (`MARQDO_BIN`/`MARQDO_STDLIB_DIR` preset) | standard library, extension, conformance and reconcile lanes |
| `git.cflmy.cn/cflmy/marqdo-ci-rust:3.0.0-rc.5` | shell image + pinned Rust 1.100 toolchain (rustfmt/clippy) + warm cargo cache | Design/Preview and Rust tool lanes |
| `git.cflmy.cn/cflmy/marqdo-ci-web:3.0.0-rc.5` (planned) | Playwright Chromium/Node + released CLI/Bundle + packaged preview web Tool | Web domain browser acceptance |

Rules:

- Tags are immutable and tied to one released CLI version; do not move a tag.
  A new CLI release publishes new tags; consumers update in reviewed PRs.
- Images are public for anonymous pull; CI never needs registry push credentials.
- Component lanes contain only their own `scripts/*` entry command. No per-repo
  image variance, apt installs, Rust toolchain downloads or CLI rebuilds.
- Core CLI keeps its specialized Rust/Windows-cross gates; the root ecosystem
  candidate keeps its Playwright/rustup image because it must build consumers and
  run Docker archive checks. These two exceptions are documented in the
  repository conventions, not copied per component.
- The images are CI infrastructure, not product releases; their provenance is
  recorded here and in `docs/evidence/`.
