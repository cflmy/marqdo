import * as vscode from "vscode";
import { spawn, spawnSync, type ChildProcessWithoutNullStreams } from "child_process";
import type { ExtensionContext } from "vscode";

export interface CliResult {
  code: number | null;
  stdout: string;
  stderr: string;
}

let resolveCli: (() => string) | undefined;
let resolveLib: (() => string | undefined) | undefined;

/** Wire install-aware path resolution from extension activate. */
export function configureCliResolution(
  _context: ExtensionContext,
  cli: () => string,
  lib: () => string | undefined
): void {
  resolveCli = cli;
  resolveLib = lib;
}

function configuredPath(): string {
  return (
    vscode.workspace.getConfiguration("marqdo").get<string>("cliPath")?.trim() ||
    "marqdo"
  );
}

export function resolveMarqdoCli(): string {
  if (resolveCli) {
    return resolveCli();
  }
  return configuredPath();
}

function spawnEnv(): NodeJS.ProcessEnv {
  const env = { ...process.env };
  const lib = resolveLib?.();
  if (lib) {
    env.MARQDO_LIB = lib;
  }
  return env;
}

export function workspaceCwd(prefer?: vscode.Uri): string {
  if (prefer) {
    const folder = vscode.workspace.getWorkspaceFolder(prefer);
    if (folder) {
      return folder.uri.fsPath;
    }
    return vscode.Uri.joinPath(prefer, "..").fsPath;
  }
  return vscode.workspace.workspaceFolders?.[0]?.uri.fsPath ?? process.cwd();
}

export function runMarqdo(
  args: string[],
  options: { cwd: string; timeoutMs?: number }
): Promise<CliResult> {
  const cli = resolveMarqdoCli();
  return new Promise((resolve) => {
    const child = spawn(cli, args, {
      cwd: options.cwd,
      shell: process.platform === "win32",
      env: spawnEnv(),
    });
    let stdout = "";
    let stderr = "";
    let settled = false;
    const finish = (code: number | null) => {
      if (settled) {
        return;
      }
      settled = true;
      resolve({ code, stdout, stderr });
    };
    const timer =
      options.timeoutMs && options.timeoutMs > 0
        ? setTimeout(() => {
            child.kill();
            finish(null);
          }, options.timeoutMs)
        : undefined;
    child.stdout.on("data", (d: Buffer) => {
      stdout += d.toString();
    });
    child.stderr.on("data", (d: Buffer) => {
      stderr += d.toString();
    });
    child.on("error", (err) => {
      if (timer) {
        clearTimeout(timer);
      }
      stderr += err.message;
      finish(1);
    });
    child.on("close", (code) => {
      if (timer) {
        clearTimeout(timer);
      }
      finish(code);
    });
  });
}

export function probeMarqdo(
  cliOverride?: string
): { ok: boolean; version?: string; detail: string } {
  const cli = cliOverride ?? resolveMarqdoCli();
  try {
    const r = spawnSync(cli, ["--version"], {
      encoding: "utf8",
      shell: process.platform === "win32",
      timeout: 8000,
      env: spawnEnv(),
    });
    const out = `${r.stdout ?? ""}${r.stderr ?? ""}`.trim();
    if (r.error) {
      return { ok: false, detail: r.error.message };
    }
    if (r.status !== 0) {
      return { ok: false, detail: out || `exit ${r.status}` };
    }
    return { ok: true, version: out, detail: out };
  } catch (e) {
    return { ok: false, detail: e instanceof Error ? e.message : String(e) };
  }
}

export class LiveServer {
  private proc: ChildProcessWithoutNullStreams | null = null;
  private output: vscode.OutputChannel;
  private label: string;

  constructor(output: vscode.OutputChannel, label: string) {
    this.output = output;
    this.label = label;
  }

  get running(): boolean {
    return this.proc !== null && this.proc.exitCode === null;
  }

  async start(
    subcommand: "view" | "debug",
    targetPath: string,
    host: string,
    port: number
  ): Promise<string> {
    if (this.running) {
      await this.stop();
    }
    const cli = resolveMarqdoCli();
    const cwd = workspaceCwd(vscode.Uri.file(targetPath));
    this.output.appendLine(
      `$ ${cli} ${subcommand} ${targetPath} --host ${host} --port ${port} --no-open`
    );
    this.proc = spawn(
      cli,
      [subcommand, targetPath, "--host", host, "--port", String(port), "--no-open"],
      {
        cwd,
        shell: process.platform === "win32",
        env: spawnEnv(),
      }
    );
    const url = `http://${host}:${port}/`;
    this.proc.stdout.on("data", (d: Buffer) => this.output.append(d.toString()));
    this.proc.stderr.on("data", (d: Buffer) => this.output.append(d.toString()));
    this.proc.on("exit", (code) => {
      this.output.appendLine(`[${this.label}] exited (${code})`);
      this.proc = null;
    });
    this.proc.on("error", (err) => {
      this.output.appendLine(`[${this.label}] ${err.message}`);
      this.proc = null;
    });
    await new Promise((r) => setTimeout(r, 600));
    return url;
  }

  async stop(): Promise<void> {
    if (!this.proc) {
      return;
    }
    const p = this.proc;
    this.proc = null;
    p.kill();
    await new Promise((r) => setTimeout(r, 200));
  }
}
