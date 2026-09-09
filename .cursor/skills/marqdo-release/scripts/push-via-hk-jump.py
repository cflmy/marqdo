#!/usr/bin/env python3
"""Push / run git|gh through an HK SSH jump host (Paramiko HTTP CONNECT).

When this machine cannot reach GitHub reliably but can SSH to a host that can
(e.g. hk.cflmy.de), open a local HTTP CONNECT proxy over Paramiko and run the
remaining command with http(s).proxy pointed at it.

Why Paramiko: OpenSSH/`sshpass` has hung on password auth in this environment;
Paramiko password + direct-tcpip works.

Credentials (never commit):
  HK_SSH_PASSWORD          password string
  HK_SSH_PASSWORD_FILE     file containing password (mode 600 recommended)
  or SSH key via Paramiko agent/keys if password unset and key works

Optional env:
  HK_SSH_HOST   default hk.cflmy.de
  HK_SSH_USER   default root
  HK_JUMP_PORT  default 18081 (local CONNECT listen)

Usage:
  HK_SSH_PASSWORD_FILE=/tmp/hk.pw \\
    python3 .cursor/skills/marqdo-release/scripts/push-via-hk-jump.py -- \\
      git push origin main

  # keep proxy only (Ctrl+C to stop)
  ... push-via-hk-jump.py --serve

  # wrap gh
  ... push-via-hk-jump.py -- gh release view v0.3.9
"""
from __future__ import annotations

import os
import select
import socket
import socketserver
import subprocess
import sys
import threading
import time
from pathlib import Path

try:
    import paramiko
except ImportError as e:  # pragma: no cover
    raise SystemExit(
        "paramiko required: apt install python3-paramiko  (or pip install paramiko)"
    ) from e

ROOT = Path(__file__).resolve().parents[4]


class _Server(socketserver.ThreadingMixIn, socketserver.TCPServer):
    allow_reuse_address = True
    daemon_threads = True


def load_password() -> str | None:
    path = os.environ.get("HK_SSH_PASSWORD_FILE")
    if path:
        return Path(path).read_text(encoding="utf-8").strip()
    pw = os.environ.get("HK_SSH_PASSWORD")
    return pw.strip() if pw else None


def connect_hk() -> paramiko.SSHClient:
    host = os.environ.get("HK_SSH_HOST", "hk.cflmy.de")
    user = os.environ.get("HK_SSH_USER", "root")
    pw = load_password()
    client = paramiko.SSHClient()
    client.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    kwargs: dict = {
        "hostname": host,
        "username": user,
        "timeout": 20,
        "auth_timeout": 25,
        "banner_timeout": 25,
    }
    if pw:
        kwargs.update(password=pw, allow_agent=False, look_for_keys=False)
    else:
        kwargs.update(allow_agent=True, look_for_keys=True)
    print(f"== HK jump {user}@{host} ==", flush=True)
    client.connect(**kwargs)
    _, stdout, stderr = client.exec_command(
        "hostname; curl -sS --max-time 20 -o /dev/null -w gh=%{http_code} https://github.com/; echo"
    )
    out = stdout.read().decode()
    err = stderr.read().decode()
    print(out, end="" if out.endswith("\n") else "\n", flush=True)
    if err.strip():
        print(err, file=sys.stderr, flush=True)
    return client


def make_proxy(transport: paramiko.Transport, port: int) -> _Server:
    class Handler(socketserver.StreamRequestHandler):
        def handle(self) -> None:
            try:
                first = self.rfile.readline(65536)
                if not first:
                    return
                parts = first.decode("latin1", "replace").split()
                if len(parts) < 2 or parts[0].upper() != "CONNECT":
                    self.wfile.write(b"HTTP/1.1 405 Method Not Allowed\r\n\r\n")
                    return
                while True:
                    line = self.rfile.readline(65536)
                    if line in (b"\r\n", b"\n", b""):
                        break
                host, _, port_s = parts[1].partition(":")
                chan = transport.open_channel(
                    "direct-tcpip",
                    (host, int(port_s or "443")),
                    self.connection.getpeername(),
                )
                self.wfile.write(b"HTTP/1.1 200 Connection Established\r\n\r\n")
                sock = self.connection
                sock.setblocking(False)
                chan.setblocking(False)
                try:
                    while True:
                        r, _, x = select.select([sock, chan], [], [sock, chan], 120)
                        if x:
                            break
                        if sock in r:
                            data = sock.recv(65536)
                            if not data:
                                break
                            chan.sendall(data)
                        if chan in r:
                            data = chan.recv(65536)
                            if not data:
                                break
                            sock.sendall(data)
                finally:
                    try:
                        chan.close()
                    except Exception:
                        pass
            except Exception:
                pass

    return _Server(("127.0.0.1", port), Handler)


def github_askpass() -> Path:
    """Write a one-shot GIT_ASKPASS that answers with github.com store creds."""
    out = subprocess.check_output(
        ["git", "credential", "fill"],
        input=b"protocol=https\nhost=github.com\n\n",
    ).decode()
    data = dict(line.split("=", 1) for line in out.splitlines() if "=" in line)
    user = data.get("username") or "x-access-token"
    password = data.get("password") or ""
    if not password:
        raise SystemExit("no github.com credential in git credential store")
    ask = Path("/tmp/marqdo-git-askpass-hk.py")
    ask.write_text(
        "#!/usr/bin/env python3\n"
        "import sys\n"
        f"u={user!r}; p={password!r}\n"
        "print(u if 'username' in sys.argv[-1].lower() else p)\n",
        encoding="utf-8",
    )
    ask.chmod(0o700)
    return ask


def run_with_proxy(proxy: str, argv: list[str]) -> int:
    env = os.environ.copy()
    for k in (
        "http_proxy",
        "https_proxy",
        "HTTP_PROXY",
        "HTTPS_PROXY",
        "ALL_PROXY",
        "all_proxy",
    ):
        env.pop(k, None)
    env["http_proxy"] = proxy
    env["https_proxy"] = proxy
    env["HTTP_PROXY"] = proxy
    env["HTTPS_PROXY"] = proxy

    cmd = list(argv)
    if cmd and cmd[0] == "git":
        ask = github_askpass()
        env["GIT_ASKPASS"] = str(ask)
        env["GIT_TERMINAL_PROMPT"] = "0"
        # Force this CONNECT proxy; clear gitconfig Clash 7890 for the session.
        cmd = [
            "git",
            "-c",
            f"http.proxy={proxy}",
            "-c",
            f"https.proxy={proxy}",
            "-c",
            "credential.helper=",
            "-c",
            "http.version=HTTP/1.1",
            "-c",
            "http.postBuffer=524288000",
            *cmd[1:],
        ]
        try:
            print("+", " ".join(cmd), flush=True)
            return subprocess.call(cmd, cwd=ROOT, env=env)
        finally:
            ask.unlink(missing_ok=True)

    if cmd and cmd[0] == "gh" and not env.get("GH_TOKEN"):
        # session token from git credential store
        out = subprocess.check_output(
            ["git", "credential", "fill"],
            input=b"protocol=https\nhost=github.com\n\n",
        ).decode()
        data = dict(line.split("=", 1) for line in out.splitlines() if "=" in line)
        if data.get("password"):
            env["GH_TOKEN"] = data["password"]

    print("+", " ".join(cmd), flush=True)
    return subprocess.call(cmd, cwd=ROOT, env=env)


def main(argv: list[str]) -> int:
    args = argv[1:]
    if not args or args[0] in ("-h", "--help"):
        print(__doc__)
        return 0 if args else 1

    serve_only = args == ["--serve"]
    if args[0] == "--":
        args = args[1:]
    if not serve_only and not args:
        print("missing command after --", file=sys.stderr)
        return 2

    port = int(os.environ.get("HK_JUMP_PORT", "18081"))
    client = connect_hk()
    transport = client.get_transport()
    assert transport is not None
    server = make_proxy(transport, port)
    threading.Thread(target=server.serve_forever, daemon=True).start()
    proxy = f"http://127.0.0.1:{port}"
    print(f"== CONNECT proxy {proxy} ==", flush=True)
    # smoke
    subprocess.call(
        [
            "curl",
            "-sS",
            "--connect-timeout",
            "10",
            "--max-time",
            "25",
            "--proxy",
            proxy,
            "-o",
            "/dev/null",
            "-w",
            "github=%{http_code}\\n",
            "https://github.com/",
        ]
    )

    try:
        if serve_only:
            print("serving; Ctrl+C to stop", flush=True)
            while transport.is_active():
                time.sleep(5)
            return 1
        return run_with_proxy(proxy, args)
    finally:
        server.shutdown()
        client.close()


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
