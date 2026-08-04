# -*- coding: utf-8 -*-
"""Create a GitHub release from Cargo.toml version and upload Windows binaries."""
from __future__ import annotations

import json
import re
import subprocess
import urllib.error
import urllib.request
from pathlib import Path

REPO = "cflmy/marqdo"


def crate_version() -> str:
    text = Path("Cargo.toml").read_text(encoding="utf-8")
    m = re.search(r'(?m)^version\s*=\s*"([^"]+)"', text)
    if not m:
        raise SystemExit("version not found in Cargo.toml")
    return m.group(1)


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
    ver = crate_version()
    tag = f"v{ver}"
    stem = f"marqdo-{ver}-x86_64-pc-windows-msvc"
    assets = [
        Path(f"target/dist/{stem}.exe"),
        Path(f"target/dist/{stem}.zip"),
    ]
    for p in assets:
        if not p.exists():
            raise SystemExit(f"missing asset: {p}")

    notes = f"""## Marqdo {tag}

### Highlights
- Function body end via `---` / `***` or empty `****` return
- Blank-line paragraph comments (lex + view rendering)
- Brand logo / favicon from s3.cflmy.cn
- Welcome docs and call-arg spaced named values

### Assets
- `{stem}.exe` — Windows x64 binary
- `{stem}.zip` — same binary zipped

```text
marqdo --version
marqdo run public/00-welcome.mq.md
marqdo view public
```
"""

    token = git_token()

    existing = None
    try:
        _, existing = api(
            "GET",
            f"https://api.github.com/repos/{REPO}/releases/tags/{tag}",
            token,
        )
    except SystemExit as e:
        if "404" not in str(e):
            raise

    if existing:
        release_id = existing["id"]
        upload_url = existing["upload_url"].split("{", 1)[0]
        print(f"reuse release id={release_id}")
    else:
        payload = json.dumps(
            {
                "tag_name": tag,
                "name": tag,
                "body": notes,
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

    _, rel = api(
        "GET",
        f"https://api.github.com/repos/{REPO}/releases/{release_id}",
        token,
    )
    want = {p.name for p in assets}
    for asset in rel.get("assets", []):
        if asset["name"] in want:
            api(
                "DELETE",
                f"https://api.github.com/repos/{REPO}/releases/assets/{asset['id']}",
                token,
            )
            print(f"deleted old asset {asset['name']}")

    for path in assets:
        data = path.read_bytes()
        url = f"{upload_url}?name={path.name}"
        ctype = "application/zip" if path.suffix == ".zip" else "application/octet-stream"
        _, uploaded = api("POST", url, token, data=data, content_type=ctype)
        print(f"uploaded {path.name} -> {uploaded.get('browser_download_url')}")

    print(f"OK https://github.com/{REPO}/releases/tag/{tag}")


if __name__ == "__main__":
    main()
