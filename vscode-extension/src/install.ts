import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";
import * as os from "os";
import { spawnSync } from "child_process";
import { probeMarqdo } from "./cli";

const REPO = "cflmy/marqdo";
const SKIP_KEY = "marqdo.skipInstallPrompt";
const MANAGED_DIR = "cli";

export type InstallStatus = {
  cliOk: boolean;
  libOk: boolean;
  version?: string;
  cliPath?: string;
  libRoot?: string;
  detail: string;
};

type GhAsset = { name: string; browser_download_url: string };
type GhRelease = {
  tag_name: string;
  html_url: string;
  assets: GhAsset[];
};

function cfg() {
  return vscode.workspace.getConfiguration("marqdo");
}

export function managedRoot(context: vscode.ExtensionContext): string {
  return path.join(context.globalStorageUri.fsPath, MANAGED_DIR);
}

export function managedExePath(context: vscode.ExtensionContext): string {
  const name = process.platform === "win32" ? "marqdo.exe" : "marqdo";
  return path.join(managedRoot(context), name);
}

export function managedLibRoot(context: vscode.ExtensionContext): string {
  // Release zip layout: marqdo.exe + lib/…  → MARQDO_LIB can be the dir that contains lib/, or lib itself.
  // load.rs: root.join(remainder) and root.join("lib").join(remainder)
  return managedRoot(context);
}

/** Prefer explicit setting, then managed install, then PATH. */
export function resolveCliPath(context?: vscode.ExtensionContext): string {
  const setting = cfg().get<string>("cliPath")?.trim() || "marqdo";
  if (setting !== "marqdo") {
    return setting;
  }
  if (context) {
    const managed = managedExePath(context);
    if (fs.existsSync(managed)) {
      return managed;
    }
  }
  const stored = cfg().get<string>("managedCliPath")?.trim();
  if (stored && fs.existsSync(stored)) {
    return stored;
  }
  return "marqdo";
}

export function resolveLibEnv(context?: vscode.ExtensionContext): string | undefined {
  const setting = cfg().get<string>("libPath")?.trim();
  if (setting) {
    return setting;
  }
  if (context) {
    const root = managedLibRoot(context);
    if (fs.existsSync(path.join(root, "lib")) || looksLikeLibRoot(root)) {
      return root;
    }
  }
  return undefined;
}

function looksLikeLibRoot(root: string): boolean {
  return (
    fs.existsSync(path.join(root, "math.mq.md")) ||
    fs.existsSync(path.join(root, "text.mq.md")) ||
    fs.existsSync(path.join(root, "lib", "math.mq.md"))
  );
}

function findExeDir(cliPath: string): string | undefined {
  if (cliPath.includes(path.sep) || cliPath.includes("/") || /\.exe$/i.test(cliPath)) {
    if (fs.existsSync(cliPath)) {
      return path.dirname(cliPath);
    }
  }
  // Resolve via where/which
  const cmd = process.platform === "win32" ? "where" : "which";
  const r = spawnSync(cmd, [cliPath === "marqdo" && process.platform === "win32" ? "marqdo.exe" : cliPath], {
    encoding: "utf8",
    shell: true,
  });
  const first = (r.stdout || "")
    .split(/\r?\n/)
    .map((s) => s.trim())
    .find((s) => s.length > 0);
  if (first && fs.existsSync(first)) {
    return path.dirname(first);
  }
  return undefined;
}

function stdlibPresent(cliPath: string, context?: vscode.ExtensionContext): { ok: boolean; root?: string } {
  const envLib = process.env.MARQDO_LIB;
  if (envLib && looksLikeLibRoot(envLib)) {
    return { ok: true, root: envLib };
  }
  const configured = resolveLibEnv(context);
  if (configured && looksLikeLibRoot(configured)) {
    return { ok: true, root: configured };
  }
  const dir = findExeDir(cliPath);
  if (dir) {
    if (looksLikeLibRoot(dir) || looksLikeLibRoot(path.join(dir, "lib"))) {
      return { ok: true, root: dir };
    }
  }
  // Workspace ./lib
  const ws = vscode.workspace.workspaceFolders?.[0]?.uri.fsPath;
  if (ws && looksLikeLibRoot(path.join(ws, "lib"))) {
    return { ok: true, root: path.join(ws, "lib") };
  }
  return { ok: false };
}

export function checkInstallStatus(context: vscode.ExtensionContext): InstallStatus {
  const cliPath = resolveCliPath(context);
  const probe = probeMarqdo(cliPath);
  if (!probe.ok) {
    return {
      cliOk: false,
      libOk: false,
      cliPath,
      detail: probe.detail,
    };
  }
  const lib = stdlibPresent(cliPath, context);
  return {
    cliOk: true,
    libOk: lib.ok,
    version: probe.version,
    cliPath,
    libRoot: lib.root,
    detail: lib.ok
      ? `CLI ${probe.version}; stdlib at ${lib.root}`
      : `CLI ${probe.version}; stdlib missing (no lib/ next to binary, no MARQDO_LIB)`,
  };
}

function rustTargetTriple(): string | undefined {
  const { platform, arch } = process;
  if (platform === "win32" && arch === "x64") {
    return "x86_64-pc-windows-msvc";
  }
  if (platform === "linux" && arch === "x64") {
    return "x86_64-unknown-linux-gnu";
  }
  if (platform === "darwin" && arch === "arm64") {
    return "aarch64-apple-darwin";
  }
  if (platform === "darwin" && arch === "x64") {
    return "x86_64-apple-darwin";
  }
  return undefined;
}

async function fetchLatestRelease(): Promise<GhRelease> {
  const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
    headers: {
      Accept: "application/vnd.github+json",
      "User-Agent": "marqdo-vscode-extension",
    },
  });
  if (!res.ok) {
    throw new Error(`GitHub releases/latest HTTP ${res.status}`);
  }
  return (await res.json()) as GhRelease;
}

async function downloadToFile(url: string, dest: string, onProgress?: (msg: string) => void): Promise<void> {
  onProgress?.(`Downloading ${path.basename(dest)}…`);
  const res = await fetch(url, {
    headers: { "User-Agent": "marqdo-vscode-extension", Accept: "application/octet-stream" },
    redirect: "follow",
  });
  if (!res.ok) {
    throw new Error(`Download failed HTTP ${res.status}: ${url}`);
  }
  const buf = Buffer.from(await res.arrayBuffer());
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.writeFileSync(dest, buf);
}

function extractZip(zipPath: string, destDir: string): void {
  fs.mkdirSync(destDir, { recursive: true });
  if (process.platform === "win32") {
    const ps = `Expand-Archive -LiteralPath '${zipPath.replace(/'/g, "''")}' -DestinationPath '${destDir.replace(/'/g, "''")}' -Force`;
    const r = spawnSync("powershell", ["-NoProfile", "-Command", ps], { encoding: "utf8" });
    if (r.status !== 0) {
      throw new Error(r.stderr || r.stdout || `Expand-Archive exit ${r.status}`);
    }
    return;
  }
  const r = spawnSync("unzip", ["-o", zipPath, "-d", destDir], { encoding: "utf8" });
  if (r.status !== 0) {
    throw new Error(r.stderr || r.stdout || `unzip exit ${r.status}`);
  }
}

function pickBundleAsset(release: GhRelease, triple: string): GhAsset | undefined {
  const zipName = `marqdo-${release.tag_name.replace(/^v/, "")}-${triple}.zip`;
  return (
    release.assets.find((a) => a.name === zipName) ||
    release.assets.find(
      (a) => a.name.endsWith(`-${triple}.zip`) && !a.name.includes("stdlib")
    )
  );
}

function pickStdlibAsset(release: GhRelease): GhAsset | undefined {
  return release.assets.find((a) => a.name.includes("stdlib") && a.name.endsWith(".zip"));
}

async function applyManagedPaths(context: vscode.ExtensionContext, exe: string): Promise<void> {
  await cfg().update("cliPath", exe, vscode.ConfigurationTarget.Global);
  await cfg().update("managedCliPath", exe, vscode.ConfigurationTarget.Global);
  await cfg().update("libPath", managedLibRoot(context), vscode.ConfigurationTarget.Global);
}

/**
 * Download recommended release bundle (exe + lib/) into extension globalStorage.
 */
export async function installFromGitHub(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel,
  mode: "bundle" | "stdlib" = "bundle"
): Promise<InstallStatus> {
  const triple = rustTargetTriple();
  output.show(true);

  return vscode.window.withProgress(
    {
      location: vscode.ProgressLocation.Notification,
      title: "Marqdo",
      cancellable: false,
    },
    async (progress) => {
      progress.report({ message: "Fetching latest release…" });
      output.appendLine(`Fetching https://github.com/${REPO}/releases/latest`);
      const release = await fetchLatestRelease();
      output.appendLine(`Latest: ${release.tag_name}`);

      const destRoot = managedRoot(context);
      fs.mkdirSync(destRoot, { recursive: true });
      const tmp = path.join(os.tmpdir(), `marqdo-ext-${Date.now()}`);
      fs.mkdirSync(tmp, { recursive: true });

      try {
        if (mode === "bundle") {
          if (!triple) {
            throw new Error(
              `No prebuilt bundle for ${process.platform}/${process.arch}. Open ${release.html_url} or build from source.`
            );
          }
          const asset = pickBundleAsset(release, triple);
          if (!asset) {
            throw new Error(
              `No asset for ${triple} in ${release.tag_name}. Available: ${release.assets
                .map((a) => a.name)
                .join(", ") || "(none)"}. See ${release.html_url}`
            );
          }
          const zipPath = path.join(tmp, asset.name);
          progress.report({ message: `Downloading ${asset.name}…` });
          await downloadToFile(asset.browser_download_url, zipPath, (m) => output.appendLine(m));
          progress.report({ message: "Extracting (CLI + stdlib)…" });
          // Clear previous managed install
          fs.rmSync(destRoot, { recursive: true, force: true });
          fs.mkdirSync(destRoot, { recursive: true });
          extractZip(zipPath, destRoot);
          output.appendLine(`Extracted to ${destRoot}`);
        } else {
          const asset = pickStdlibAsset(release);
          if (!asset) {
            throw new Error(`No stdlib zip in ${release.tag_name}. See ${release.html_url}`);
          }
          const zipPath = path.join(tmp, asset.name);
          progress.report({ message: `Downloading ${asset.name}…` });
          await downloadToFile(asset.browser_download_url, zipPath, (m) => output.appendLine(m));
          progress.report({ message: "Extracting stdlib…" });
          extractZip(zipPath, destRoot);
          output.appendLine(`Stdlib extracted to ${destRoot}`);
        }

        const exe = managedExePath(context);
        if (mode === "bundle" && !fs.existsSync(exe)) {
          // Some zips might nest a folder
          const nested = findFileRecursive(destRoot, process.platform === "win32" ? "marqdo.exe" : "marqdo");
          if (nested) {
            // Flatten: move exe+lib up if needed is complex; just point cliPath at nested
            await cfg().update("cliPath", nested, vscode.ConfigurationTarget.Global);
            await cfg().update("managedCliPath", nested, vscode.ConfigurationTarget.Global);
            const libCandidate = path.dirname(nested);
            await cfg().update("libPath", libCandidate, vscode.ConfigurationTarget.Global);
          } else {
            throw new Error(`Extracted archive but ${exe} not found`);
          }
        } else if (mode === "bundle") {
          await applyManagedPaths(context, exe);
        } else {
          await cfg().update("libPath", managedLibRoot(context), vscode.ConfigurationTarget.Global);
        }

        // chmod +x on unix
        if (process.platform !== "win32" && fs.existsSync(exe)) {
          fs.chmodSync(exe, 0o755);
        }

        const status = checkInstallStatus(context);
        output.appendLine(status.detail);
        if (status.cliOk && status.libOk) {
          void vscode.window.showInformationMessage(
            `Marqdo installed (${status.version}). Stdlib ready.`
          );
        } else if (status.cliOk && !status.libOk) {
          void vscode.window.showWarningMessage(
            `Marqdo CLI ok but stdlib still missing. Try “Marqdo: Install / Repair CLI”.`
          );
        }
        return status;
      } finally {
        fs.rmSync(tmp, { recursive: true, force: true });
      }
    }
  );
}

function findFileRecursive(root: string, name: string): string | undefined {
  const stack = [root];
  while (stack.length) {
    const dir = stack.pop()!;
    let entries: fs.Dirent[];
    try {
      entries = fs.readdirSync(dir, { withFileTypes: true });
    } catch {
      continue;
    }
    for (const e of entries) {
      const p = path.join(dir, e.name);
      if (e.isDirectory()) {
        stack.push(p);
      } else if (e.name === name) {
        return p;
      }
    }
  }
  return undefined;
}

export async function promptIfNeeded(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel
): Promise<void> {
  if (cfg().get<boolean>("autoInstall.checkOnStartup") === false) {
    return;
  }
  if (context.globalState.get<boolean>(SKIP_KEY)) {
    return;
  }

  const status = checkInstallStatus(context);
  output.appendLine(`[install] ${status.detail}`);

  if (status.cliOk && status.libOk) {
    return;
  }

  let message: string;
  let primary: string;
  if (!status.cliOk) {
    message =
      "Marqdo CLI not found. Download the official release (includes standard library) into the extension?";
    primary = "Install CLI + stdlib";
  } else {
    message =
      "Marqdo CLI found, but the standard library (lib/) is missing. Download stdlib into the extension?";
    primary = "Install stdlib";
  }

  const choice = await vscode.window.showWarningMessage(
    message,
    primary,
    "Open Releases",
    "Don't ask again"
  );

  if (choice === primary) {
    try {
      await installFromGitHub(context, output, status.cliOk ? "stdlib" : "bundle");
    } catch (e) {
      const err = e instanceof Error ? e.message : String(e);
      output.appendLine(`[install] ${err}`);
      const open = await vscode.window.showErrorMessage(`Marqdo install failed: ${err}`, "Open Releases");
      if (open) {
        await vscode.env.openExternal(vscode.Uri.parse(`https://github.com/${REPO}/releases`));
      }
    }
  } else if (choice === "Open Releases") {
    await vscode.env.openExternal(vscode.Uri.parse(`https://github.com/${REPO}/releases`));
  } else if (choice === "Don't ask again") {
    await context.globalState.update(SKIP_KEY, true);
  }
}

export async function resetSkipPrompt(context: vscode.ExtensionContext): Promise<void> {
  await context.globalState.update(SKIP_KEY, undefined);
}
