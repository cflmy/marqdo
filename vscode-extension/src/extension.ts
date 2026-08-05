import * as vscode from "vscode";

/**
 * Marqdo VS Code extension entry.
 * Keep this host thin: language contribution lives in package.json / TextMate;
 * run / diagnostics / DAP come later via the marqdo CLI.
 */
export function activate(_context: vscode.ExtensionContext): void {
  // P0: grammar + language id are declarative.
  // P1+: register commands that spawn `marqdo`.
}

export function deactivate(): void {}
