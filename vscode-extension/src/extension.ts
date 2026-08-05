import * as vscode from "vscode";
import { LiveServer, configureCliResolution, runMarqdo, workspaceCwd } from "./cli";
import { DiagnosticBag } from "./diagnostics";
import {
  checkInstallStatus,
  installFromGitHub,
  promptIfNeeded,
  resetSkipPrompt,
  resolveCliPath,
  resolveLibEnv,
} from "./install";

let output: vscode.OutputChannel;
let diagnostics: DiagnosticBag;
let viewServer: LiveServer;
let debugServer: LiveServer;
let extContext: vscode.ExtensionContext;

export function activate(context: vscode.ExtensionContext): void {
  extContext = context;
  output = vscode.window.createOutputChannel("Marqdo");
  diagnostics = new DiagnosticBag();
  viewServer = new LiveServer(output, "view");
  debugServer = new LiveServer(output, "debug");

  configureCliResolution(
    context,
    () => resolveCliPath(context),
    () => resolveLibEnv(context)
  );

  context.subscriptions.push(
    output,
    diagnostics.disposable,
    vscode.commands.registerCommand("marqdo.run", () => runActiveFile()),
    vscode.commands.registerCommand("marqdo.view", () => startView()),
    vscode.commands.registerCommand("marqdo.debug", () => startDebug()),
    vscode.commands.registerCommand("marqdo.debug.stop", () => stopDebug()),
    vscode.commands.registerCommand("marqdo.view.stop", () => stopView()),
    vscode.commands.registerCommand("marqdo.catalog", () => runCatalog()),
    vscode.commands.registerCommand("marqdo.showOutput", () => output.show(true)),
    vscode.commands.registerCommand("marqdo.installCli", () => installCli()),
    vscode.commands.registerCommand("marqdo.checkCli", () => checkCli())
  );

  context.subscriptions.push(
    vscode.workspace.onDidSaveTextDocument((doc) => {
      if (doc.languageId === "marqdo" && autoDiagnose()) {
        void diagnoseDocument(doc);
      }
    })
  );

  void promptIfNeeded(context, output);
}

export function deactivate(): void {
  void viewServer?.stop();
  void debugServer?.stop();
}

function autoDiagnose(): boolean {
  return vscode.workspace.getConfiguration("marqdo").get<boolean>("diagnoseOnSave") ?? false;
}

async function installCli(): Promise<void> {
  await resetSkipPrompt(extContext);
  const status = checkInstallStatus(extContext);
  try {
    const mode = status.suggestedMode ?? "bundle";
    await installFromGitHub(extContext, output, mode);
  } catch (e) {
    const err = e instanceof Error ? e.message : String(e);
    output.appendLine(`[install] ${err}`);
    const open = await vscode.window.showErrorMessage(`Marqdo install failed: ${err}`, "Open Releases");
    if (open) {
      await vscode.env.openExternal(vscode.Uri.parse("https://github.com/cflmy/marqdo/releases"));
    }
  }
}

async function checkCli(): Promise<void> {
  const status = checkInstallStatus(extContext);
  output.appendLine(`[check] ${status.detail}`);
  if (status.cliOk && status.versionOk && status.libOk) {
    void vscode.window.showInformationMessage(
      `Marqdo ready: ${status.parsedVersion ?? status.version} (≥ ${status.requiredVersion}), stdlib OK`
    );
    return;
  }
  const choice = await vscode.window.showWarningMessage(
    status.detail,
    "Install / Repair",
    "Open Releases"
  );
  if (choice === "Install / Repair") {
    await installCli();
  } else if (choice === "Open Releases") {
    await vscode.env.openExternal(vscode.Uri.parse("https://github.com/cflmy/marqdo/releases"));
  }
}

function activeMqDoc(): vscode.TextDocument | undefined {
  const doc = vscode.window.activeTextEditor?.document;
  if (!doc) {
    return undefined;
  }
  if (doc.languageId === "marqdo" || doc.fileName.endsWith(".mq.md")) {
    return doc;
  }
  return undefined;
}

async function ensureCliOrOfferInstall(): Promise<boolean> {
  const status = checkInstallStatus(extContext);
  if (status.cliOk && status.versionOk && status.libOk) {
    return true;
  }
  if (status.cliOk && status.versionOk && !status.libOk) {
    const choice = await vscode.window.showWarningMessage(
      "Standard library (lib/) is missing. Continue anyway, or install stdlib?",
      "Install stdlib",
      "Continue",
      "Cancel"
    );
    if (choice === "Install stdlib") {
      await installCli();
      return true;
    }
    if (choice === "Continue") {
      return true;
    }
    return false;
  }
  const label = !status.cliOk
    ? "Marqdo CLI not found. Install the official release (includes stdlib)?"
    : `Marqdo CLI is below ${status.requiredVersion}. Upgrade now?`;
  const choice = await vscode.window.showWarningMessage(label, "Install", "Cancel");
  if (choice === "Install") {
    await installCli();
    const again = checkInstallStatus(extContext);
    return again.cliOk && again.versionOk;
  }
  return false;
}

function resolveTargetPath(): string | undefined {
  const doc = activeMqDoc();
  const folder = vscode.workspace.workspaceFolders?.[0];
  return doc?.uri.fsPath ?? folder?.uri.fsPath;
}

async function openUrl(url: string, settingKey: "viewOpen" | "debugOpen"): Promise<void> {
  const open =
    vscode.workspace.getConfiguration("marqdo").get<string>(settingKey) ?? "external";
  if (open === "simpleBrowser") {
    await vscode.commands.executeCommand("simpleBrowser.show", url);
  } else {
    await vscode.env.openExternal(vscode.Uri.parse(url));
  }
}

async function runActiveFile(): Promise<void> {
  if (!(await ensureCliOrOfferInstall())) {
    return;
  }
  const doc = activeMqDoc();
  if (!doc) {
    void vscode.window.showWarningMessage("Open a .mq.md file to run.");
    return;
  }
  if (doc.isDirty) {
    await doc.save();
  }
  const cwd = workspaceCwd(doc.uri);
  output.show(true);
  output.appendLine(`$ marqdo run ${doc.uri.fsPath}`);
  const result = await runMarqdo(["run", doc.uri.fsPath], {
    cwd,
    timeoutMs: vscode.workspace.getConfiguration("marqdo").get<number>("runTimeoutMs") ?? 60000,
  });
  if (result.stdout.trim()) {
    output.appendLine(result.stdout.trimEnd());
  }
  if (result.stderr.trim()) {
    output.appendLine(result.stderr.trimEnd());
  }
  if (result.code === 0) {
    diagnostics.clear(doc.uri);
    void vscode.window.setStatusBarMessage("Marqdo: run ok", 3000);
  } else {
    diagnostics.setFromCli(result.stderr || result.stdout, doc.uri, cwd);
    void vscode.window.showErrorMessage("Marqdo: run failed (see Problems / Output)");
  }
}

async function diagnoseDocument(doc: vscode.TextDocument): Promise<void> {
  const cwd = workspaceCwd(doc.uri);
  const result = await runMarqdo(["run", doc.uri.fsPath], {
    cwd,
    timeoutMs: vscode.workspace.getConfiguration("marqdo").get<number>("runTimeoutMs") ?? 60000,
  });
  if (result.code === 0) {
    diagnostics.clear(doc.uri);
  } else {
    diagnostics.setFromCli(result.stderr || result.stdout, doc.uri, cwd);
  }
}

async function startView(): Promise<void> {
  if (!(await ensureCliOrOfferInstall())) {
    return;
  }
  const target = resolveTargetPath();
  if (!target) {
    void vscode.window.showWarningMessage("Open a folder or .mq.md file for view.");
    return;
  }
  const cfg = vscode.workspace.getConfiguration("marqdo");
  const host = cfg.get<string>("viewHost") ?? "127.0.0.1";
  const port = cfg.get<number>("viewPort") ?? 7429;
  output.show(true);
  try {
    const url = await viewServer.start("view", target, host, port);
    await openUrl(url, "viewOpen");
    void vscode.window.setStatusBarMessage(`Marqdo view: ${url}`, 5000);
  } catch (e) {
    void vscode.window.showErrorMessage(
      `Marqdo view failed: ${e instanceof Error ? e.message : String(e)}`
    );
  }
}

async function startDebug(): Promise<void> {
  if (!(await ensureCliOrOfferInstall())) {
    return;
  }
  const target = resolveTargetPath();
  if (!target) {
    void vscode.window.showWarningMessage("Open a folder or .mq.md file to debug.");
    return;
  }
  const cfg = vscode.workspace.getConfiguration("marqdo");
  const host = cfg.get<string>("debugHost") ?? "127.0.0.1";
  const port = cfg.get<number>("debugPort") ?? 7430;
  output.show(true);
  try {
    const url = await debugServer.start("debug", target, host, port);
    await openUrl(url, "debugOpen");
    void vscode.window.setStatusBarMessage(`Marqdo debug: ${url}`, 5000);
  } catch (e) {
    void vscode.window.showErrorMessage(
      `Marqdo debug failed: ${e instanceof Error ? e.message : String(e)}`
    );
  }
}

async function stopView(): Promise<void> {
  await viewServer.stop();
  void vscode.window.setStatusBarMessage("Marqdo view stopped", 3000);
}

async function stopDebug(): Promise<void> {
  await debugServer.stop();
  void vscode.window.setStatusBarMessage("Marqdo debug stopped", 3000);
}

async function runCatalog(): Promise<void> {
  if (!(await ensureCliOrOfferInstall())) {
    return;
  }
  const folder = vscode.workspace.workspaceFolders?.[0];
  if (!folder) {
    void vscode.window.showWarningMessage("Open a workspace folder for catalog.");
    return;
  }
  const outDir =
    vscode.workspace.getConfiguration("marqdo").get<string>("catalogOut") ?? ".marqdo";
  output.show(true);
  output.appendLine(`$ marqdo catalog ${folder.uri.fsPath} -o ${outDir}`);
  const result = await runMarqdo(["catalog", folder.uri.fsPath, "-o", outDir], {
    cwd: folder.uri.fsPath,
    timeoutMs: 120000,
  });
  if (result.stdout.trim()) {
    output.appendLine(result.stdout.trimEnd());
  }
  if (result.stderr.trim()) {
    output.appendLine(result.stderr.trimEnd());
  }
  if (result.code === 0) {
    const index = vscode.Uri.joinPath(folder.uri, outDir, "index.md");
    try {
      const doc = await vscode.workspace.openTextDocument(index);
      await vscode.window.showTextDocument(doc, { preview: true });
    } catch {
      void vscode.window.showInformationMessage(`Marqdo catalog written to ${outDir}`);
    }
  } else {
    void vscode.window.showErrorMessage("Marqdo catalog failed (see Output)");
  }
}
