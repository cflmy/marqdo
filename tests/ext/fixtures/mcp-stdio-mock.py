#!/usr/bin/env python3
"""Minimal stdio MCP server for offline client goldens."""
import json
import sys

def read_msg():
    line = sys.stdin.readline()
    if not line:
        return None
    line = line.strip()
    if not line:
        return read_msg()
    return json.loads(line)

def write_msg(msg):
    sys.stdout.write(json.dumps(msg, separators=(",", ":")) + "\n")
    sys.stdout.flush()

TOOLS = [
    {
        "name": "echo",
        "description": "echo arguments",
        "inputSchema": {"type": "object", "properties": {}, "additionalProperties": True},
    }
]

while True:
    msg = read_msg()
    if msg is None:
        break
    mid = msg.get("id")
    method = msg.get("method") or ""
    params = msg.get("params") or {}
    if mid is None and str(method).startswith("notifications/"):
        continue
    if method == "initialize":
        write_msg({
            "jsonrpc": "2.0",
            "id": mid,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "mock", "version": "0.1.0"},
            },
        })
    elif method in ("tools/list", "list_tools"):
        write_msg({"jsonrpc": "2.0", "id": mid, "result": {"tools": TOOLS}})
    elif method in ("tools/call", "call_tool"):
        name = params.get("name") or ""
        args = params.get("arguments") or {}
        if name != "echo":
            write_msg({
                "jsonrpc": "2.0",
                "id": mid,
                "error": {"code": -32000, "message": f"unknown tool {name}"},
            })
        else:
            write_msg({
                "jsonrpc": "2.0",
                "id": mid,
                "result": {
                    "content": [{"type": "text", "text": json.dumps(args, separators=(",", ":"))}],
                    "isError": False,
                },
            })
    elif method == "ping":
        write_msg({"jsonrpc": "2.0", "id": mid, "result": {}})
    else:
        write_msg({
            "jsonrpc": "2.0",
            "id": mid,
            "error": {"code": -32601, "message": f"method not found: {method}"},
        })
