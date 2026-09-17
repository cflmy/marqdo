#!/usr/bin/env python3
"""Upload Marqdo extension packs to Cloudflare R2 (CDN: https://ext.marqdo.com).

No third-party deps (stdlib only) — AWS Signature V4 PutObject.

Credentials: ~/.marqdo/r2.env (chmod 600; never commit). Required:
  R2_ENDPOINT, R2_BUCKET, R2_ACCESS_KEY_ID, R2_SECRET_ACCESS_KEY
Optional: R2_PUBLIC_BASE, R2_REGION (default auto)

Usage:
  python3 scripts/upload-ext-r2.py --version 1.0.2 dist/marqdo-1.0.2-ext.zip \\
      dist/marqdo-1.0.2-native-x86_64-unknown-linux-gnu.zip
  python3 scripts/upload-ext-r2.py --version 1.0.2 --from-dir dist/
"""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import hmac
import os
import sys
import tempfile
import urllib.error
import urllib.request
from pathlib import Path
from urllib.parse import quote, urlparse


def load_env_file(path: Path) -> None:
    if not path.is_file():
        return
    for line in path.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, _, v = line.partition("=")
        os.environ.setdefault(k.strip(), v.strip().strip("'").strip('"'))


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def hmac_sha256(key: bytes, msg: str) -> bytes:
    return hmac.new(key, msg.encode("utf-8"), hashlib.sha256).digest()


def signing_key(secret: str, datestamp: str, region: str, service: str) -> bytes:
    k_date = hmac_sha256(("AWS4" + secret).encode("utf-8"), datestamp)
    k_region = hmac.new(k_date, region.encode("utf-8"), hashlib.sha256).digest()
    k_service = hmac.new(k_region, service.encode("utf-8"), hashlib.sha256).digest()
    return hmac.new(k_service, b"aws4_request", hashlib.sha256).digest()


def put_object(
    *,
    endpoint: str,
    bucket: str,
    key: str,
    body: bytes,
    content_type: str,
    access_key: str,
    secret_key: str,
    region: str,
) -> None:
    endpoint = endpoint.rstrip("/")
    host = urlparse(endpoint).netloc
    # Path-style: https://endpoint/bucket/key
    canonical_uri = "/" + bucket + "/" + quote(key, safe="/~")
    amz_date = dt.datetime.now(dt.timezone.utc).strftime("%Y%m%dT%H%M%SZ")
    datestamp = amz_date[:8]
    payload_hash = sha256_hex(body)
    canonical_headers = (
        f"content-type:{content_type}\n"
        f"host:{host}\n"
        f"x-amz-content-sha256:{payload_hash}\n"
        f"x-amz-date:{amz_date}\n"
    )
    signed_headers = "content-type;host;x-amz-content-sha256;x-amz-date"
    canonical_request = "\n".join(
        [
            "PUT",
            canonical_uri,
            "",
            canonical_headers,
            signed_headers,
            payload_hash,
        ]
    )
    credential_scope = f"{datestamp}/{region}/s3/aws4_request"
    string_to_sign = "\n".join(
        [
            "AWS4-HMAC-SHA256",
            amz_date,
            credential_scope,
            sha256_hex(canonical_request.encode("utf-8")),
        ]
    )
    sig = hmac.new(
        signing_key(secret_key, datestamp, region, "s3"),
        string_to_sign.encode("utf-8"),
        hashlib.sha256,
    ).hexdigest()
    auth = (
        f"AWS4-HMAC-SHA256 Credential={access_key}/{credential_scope}, "
        f"SignedHeaders={signed_headers}, Signature={sig}"
    )
    url = endpoint + canonical_uri
    req = urllib.request.Request(url, data=body, method="PUT")
    req.add_header("Content-Type", content_type)
    req.add_header("x-amz-content-sha256", payload_hash)
    req.add_header("x-amz-date", amz_date)
    req.add_header("Authorization", auth)
    try:
        with urllib.request.urlopen(req, timeout=300) as resp:
            if resp.status not in (200, 201):
                raise RuntimeError(f"PUT {key} HTTP {resp.status}")
    except urllib.error.HTTPError as e:
        detail = e.read().decode("utf-8", errors="replace")
        raise RuntimeError(f"PUT {key} HTTP {e.code}: {detail}") from e


def content_type_for(path: Path) -> str:
    if path.name == "VERSION" or path.suffix.lower() == ".txt":
        return "text/plain; charset=utf-8"
    if path.suffix.lower() == ".json":
        return "application/json"
    return "application/zip"


def upload_file(
    path: Path,
    key: str,
    *,
    endpoint: str,
    bucket: str,
    access_key: str,
    secret_key: str,
    region: str,
    public_base: str,
) -> None:
    body = path.read_bytes()
    print(f"upload {path.name} → s3://{bucket}/{key} ({len(body)} bytes)")
    put_object(
        endpoint=endpoint,
        bucket=bucket,
        key=key,
        body=body,
        content_type=content_type_for(path),
        access_key=access_key,
        secret_key=secret_key,
        region=region,
    )
    print(f"  public: {public_base.rstrip('/')}/{key}")


def main() -> int:
    load_env_file(Path.home() / ".marqdo" / "r2.env")
    load_env_file(Path(".env"))
    load_env_file(Path(".env.r2"))

    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--version", required=True, help="Extension pack SemVer (no v prefix)")
    ap.add_argument("--from-dir", type=Path, help="Upload all marqdo-{VER}-*.zip from dir")
    ap.add_argument("files", nargs="*", type=Path, help="Files to upload")
    ap.add_argument("--no-latest", action="store_true", help="Do not update latest/ pointers")
    args = ap.parse_args()

    ver = args.version.lstrip("v")
    for key in ("R2_ENDPOINT", "R2_BUCKET", "R2_ACCESS_KEY_ID", "R2_SECRET_ACCESS_KEY"):
        if not os.environ.get(key):
            print(
                f"missing {key}; write ~/.marqdo/r2.env (see .env.r2.example)",
                file=sys.stderr,
            )
            return 1

    endpoint = os.environ["R2_ENDPOINT"]
    bucket = os.environ["R2_BUCKET"]
    access_key = os.environ["R2_ACCESS_KEY_ID"]
    secret_key = os.environ["R2_SECRET_ACCESS_KEY"]
    region = os.environ.get("R2_REGION", "auto")
    public_base = os.environ.get("R2_PUBLIC_BASE", "https://ext.marqdo.com")

    files: list[Path] = [p for p in args.files if p.is_file()]
    if args.from_dir:
        files.extend(sorted(args.from_dir.glob(f"marqdo-{ver}-*.zip")))
    if not files:
        print("no files to upload", file=sys.stderr)
        return 1

    kw = dict(
        endpoint=endpoint,
        bucket=bucket,
        access_key=access_key,
        secret_key=secret_key,
        region=region,
        public_base=public_base,
    )
    for path in files:
        upload_file(path, f"v{ver}/{path.name}", **kw)
        if not args.no_latest:
            upload_file(path, f"latest/{path.name}", **kw)

    if not args.no_latest:
        with tempfile.NamedTemporaryFile("w", encoding="utf-8", delete=False) as tf:
            tf.write(ver + "\n")
            tmp = Path(tf.name)
        try:
            upload_file(tmp, "latest/VERSION", **kw)
            upload_file(tmp, f"v{ver}/VERSION", **kw)
        finally:
            tmp.unlink(missing_ok=True)

    print("done.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
