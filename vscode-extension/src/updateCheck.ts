import * as vscode from "vscode";
import { compareSemver, parseCliVersion } from "./install";

const REPO = "cflmy/marqdo";
const LAST_CHECK_KEY = "marqdo.lastUpdateCheck";
const CHECK_INTERVAL_MS = 24 * 60 * 60 * 1000;

type GhRelease = {
  tag_name: string;
  html_url: string;
  assets: { name: string }[];
};

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

function extensionVersion(): string {
  return vscode.extensions.getExtension("cflmy.marqdo")?.packageJSON?.version ?? "0.0.0";
}

function pickVsixAsset(release: GhRelease): string | undefined {
  const mine = extensionVersion();
  const exact = release.assets.find((a) => a.name === `marqdo-${mine}.vsix`);
  if (exact) {
    return undefined;
  }
  return release.assets.find((a) => a.name.endsWith(".vsix"))?.name;
}

/** Notify when a newer CLI or extension release exists on GitHub. */
export async function checkForUpdates(
  context: vscode.ExtensionContext,
  output: vscode.OutputChannel,
  installedCliVersion?: string
): Promise<void> {
  if (vscode.workspace.getConfiguration("marqdo").get<boolean>("autoInstall.checkUpdates") === false) {
    return;
  }
  const last = context.globalState.get<number>(LAST_CHECK_KEY) ?? 0;
  if (Date.now() - last < CHECK_INTERVAL_MS) {
    return;
  }
  await context.globalState.update(LAST_CHECK_KEY, Date.now());

  try {
    const release = await fetchLatestRelease();
    const latestTag = release.tag_name.replace(/^v/, "");
    const extVer = extensionVersion();
    const messages: string[] = [];

    if (compareSemver(latestTag, extVer) > 0 || pickVsixAsset(release)) {
      messages.push(`extension ${extVer} → ${release.tag_name}`);
    }
    const parsedCli = installedCliVersion ? parseCliVersion(installedCliVersion) : undefined;
    if (parsedCli && compareSemver(latestTag, parsedCli) > 0) {
      messages.push(`CLI ${parsedCli} → ${release.tag_name}`);
    }
    if (messages.length === 0) {
      output.appendLine(`[update] up to date (${release.tag_name})`);
      return;
    }
    output.appendLine(`[update] newer release: ${messages.join("; ")}`);
    const choice = await vscode.window.showInformationMessage(
      `Marqdo ${release.tag_name} is available (${messages.join(", ")}).`,
      "Open Releases",
      "Later"
    );
    if (choice === "Open Releases") {
      await vscode.env.openExternal(vscode.Uri.parse(release.html_url));
    }
  } catch (e) {
    output.appendLine(
      `[update] check skipped: ${e instanceof Error ? e.message : String(e)}`
    );
  }
}
