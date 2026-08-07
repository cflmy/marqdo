# Changelog

## v0.1.2 — 2026-08-07

### Added
- **Embedded standard library**: official `lib/*.mq.md` ships inside the `marqdo` binary; disk `lib/` and `MARQDO_LIB` still override.
- **`lib/writeback`** / **`lib/subtask`**: Jupyter-style writeback; concurrent subtasks (file / function / foreign).
- **Surface syntax v0.2**: `` + `param` `` parameters, `1.` ordered branches, backtick identifiers, quoted strings.
- **`marqdo version --check`**: compare installed CLI with latest GitHub release.
- **VS Code extension v0.0.6** (branch **`vscode-extension` only**): v0.2 grammar, update check — see [doc/design/vscode-extension-commit.md](doc/design/vscode-extension-commit.md)

### Changed
- Subtask `spawn` accepts `path=`, `fn=`, `code=`, or `lang=`+`source=` (not file-only).
- Release notes: standalone `.exe` includes stdlib; bundle/stdlib zips remain optional for overrides.

## v0.1.1

- v0.2 syntax migration, writeback/subtask v1 (file subprocess only), view input deferral, optional parameters.

## v0.1.0

- Initial public releases: tree + bytecode backends, `view` / `debug` / `catalog`, core stdlib, `ext/` installer.
