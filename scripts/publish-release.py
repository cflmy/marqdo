# -*- coding: utf-8 -*-
"""Create GitHub release v0.0.2 and upload Windows binaries using git credentials."""
from __future__ import annotations

import json
import subprocess
import urllib.error
import urllib.request
from pathlib import Path

REPO = "cflmy/marqdo"
TAG = "v0.0.2"
TITLE = "v0.0.2"
NOTES = """## Marqdo v0.0.2

User docs site, view polish, and project layout cleanup.

### Highlights
- User-facing executable docs in `public/`; static site on `gh-pages`
- Gold fixtures moved under `tests/{structure,keywords,errors}/`
- `marqdo view` / `view output` restyle; catalog CLI
- Default open first file; cleaner diagnostic paths on Windows

### Assets
- `marqdo-0.0.2-x86_64-pc-windows-msvc.exe` — Windows x64 binary
- `marqdo-0.0.2-x86_64-pc-windows-msvc.zip` — same binary zipped

```text
marqdo --version
marqdo run tests/structure/hello.mq.md
marqdo view public
```
"""

ASSETS = [
    Path("target/dist/marqdo-0.0.2-x86_64-pc-windows-msvc.exe"),
    Path("target/dist/marqdo-0.0.2-x86_64-pc-windows-msvc.zip"),
]


def git_token() -> str:
    proc = subprocess.run(
        ["git", "credential", "fill"],
        input="protocol=https\nhost=github.com\n\n",
        text=True,
        capture_output=True,
        check=True,
    )
    for line in proc.stdout.splitlines():
        if line.startswith("password="):
            return line[len("password=") :]
    raise SystemExit("no github password from git credential fill")


def api(method: str, url: str, token: str, data: bytes | None = None, content_type: str | None = None):
    req = urllib.request.Request(url, data=data, method=method)
    req.add_header("Accept", "application/vnd.github+json")
    req.add_header("Authorization", f"Bearer {token}")
    req.add_header("X-GitHub-Api-Version", "2022-11-28")
    req.add_header("User-Agent", "marqdo-release-script")
    if content_type:
        req.add_header("Content-Type", content_type)
    try:
        with urllib.request.urlopen(req) as resp:
            body = resp.read()
            return resp.status, json.loads(body.decode()) if body else {}
    except urllib.error.HTTPError as e:
        err = e.read().decode("utf-8", errors="replace")
        raise SystemExit(f"HTTP {e.code} {url}: {err}") from e


def main() -> None:
    for p in ASSETS:
        if not p.exists():
            raise SystemExit(f"missing asset: {p}")

    token = git_token()

    # If release already exists, reuse it; else create.
    status, existing = 0, None
    try:
        status, existing = api(
            "GET",
            f"https://api.github.com/repos/{REPO}/releases/tags/{TAG}",
            token,
        )
    except SystemExit as e:
        if "404" not in str(e):
            raise
        existing = None

    if existing:
        release_id = existing["id"]
        upload_url = existing["upload_url"].split("{", 1)[0]
        print(f"reuse release id={release_id}")
    else:
        payload = json.dumps(
            {
                "tag_name": TAG,
                "name": TITLE,
                "body": NOTES,
                "draft": False,
                "prerelease": False,
            }
        ).encode("utf-8")
        _, created = api(
            "POST",
            f"https://api.github.com/repos/{REPO}/releases",
            token,
            data=payload,
            content_type="application/json",
        )
        release_id = created["id"]
        upload_url = created["upload_url"].split("{", 1)[0]
        print(f"created release id={release_id}")

    # Delete existing assets with same names (idempotent re-upload)
    _, rel = api(
        "GET",
        f"https://api.github.com/repos/{REPO}/releases/{release_id}",
        token,
    )
    for asset in rel.get("assets", []):
        if asset["name"] in {p.name for p in ASSETS}:
            api(
                "DELETE",
                f"https://api.github.com/repos/{REPO}/releases/assets/{asset['id']}",
                token,
            )
            print(f"deleted old asset {asset['name']}")

    for path in ASSETS:
        data = path.read_bytes()
        url = f"{upload_url}?name={path.name}"
        ctype = "application/zip" if path.suffix == ".zip" else "application/octet-stream"
        _, uploaded = api("POST", url, token, data=data, content_type=ctype)
        print(f"uploaded {path.name} -> {uploaded.get('browser_download_url')}")

    print(f"OK https://github.com/{REPO}/releases/tag/{TAG}")


if __name__ == "__main__":
    main()
