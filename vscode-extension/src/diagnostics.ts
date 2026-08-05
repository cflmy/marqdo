import * as vscode from "vscode";
import * as path from "path";

export function parseDiagnostics(
  text: string,
  fallbackUri: vscode.Uri,
  cwd: string
): vscode.Diagnostic[] {
  const out: vscode.Diagnostic[] = [];
  for (const raw of text.split(/\r?\n/)) {
    const line = raw.trim();
    if (!line) {
      continue;
    }
    const parsed = parseDiagLine(line);
    if (!parsed) {
      continue;
    }
    out.push(
      makeDiag(parsed.file, parsed.line, parsed.col, parsed.message, fallbackUri, cwd)
    );
  }
  return out;
}

function parseDiagLine(
  line: string
): { file: string; line: number; col: number; message: string } | undefined {
  // Windows: E:\path\to\file.mq.md:7:1: message
  const win = /^([A-Za-z]:[\\/][^:]*):(\d+):(\d+):\s*(.+)$/.exec(line);
  if (win) {
    return {
      file: win[1],
      line: Number(win[2]),
      col: Number(win[3]),
      message: win[4],
    };
  }
  // Unix / relative: path/file.mq.md:7:1: message
  const normal = /^(.+):(\d+):(\d+):\s*(.+)$/.exec(line);
  if (normal) {
    return {
      file: normal[1],
      line: Number(normal[2]),
      col: Number(normal[3]),
      message: normal[4],
    };
  }
  const spanOnly = /^(\d+):(\d+):\s*(.+)$/.exec(line);
  if (spanOnly) {
    return {
      file: "",
      line: Number(spanOnly[1]),
      col: Number(spanOnly[2]),
      message: spanOnly[3],
    };
  }
  return undefined;
}

function makeDiag(
  filePath: string,
  line1: number,
  col1: number,
  message: string,
  fallbackUri: vscode.Uri,
  cwd: string
): vscode.Diagnostic {
  const line = Math.max(0, line1 - 1);
  const col = Math.max(0, col1 - 1);
  const range = new vscode.Range(line, col, line, Math.max(col + 1, col));
  const diag = new vscode.Diagnostic(range, message, vscode.DiagnosticSeverity.Error);
  diag.source = "marqdo";
  (diag as vscode.Diagnostic & { targetPath?: string }).targetPath = resolvePath(
    filePath,
    fallbackUri,
    cwd
  );
  return diag;
}

function resolvePath(filePath: string, fallbackUri: vscode.Uri, cwd: string): string {
  if (!filePath) {
    return fallbackUri.fsPath;
  }
  if (path.isAbsolute(filePath)) {
    return filePath;
  }
  return path.resolve(cwd, filePath);
}

export class DiagnosticBag {
  private readonly collection: vscode.DiagnosticCollection;

  constructor() {
    this.collection = vscode.languages.createDiagnosticCollection("marqdo");
  }

  get disposable(): vscode.Disposable {
    return this.collection;
  }

  clear(uri?: vscode.Uri): void {
    if (uri) {
      this.collection.delete(uri);
    } else {
      this.collection.clear();
    }
  }

  setFromCli(text: string, fallbackUri: vscode.Uri, cwd: string): void {
    const parsed = parseDiagnostics(text, fallbackUri, cwd);
    const byFile = new Map<string, vscode.Diagnostic[]>();
    for (const d of parsed) {
      const target =
        (d as vscode.Diagnostic & { targetPath?: string }).targetPath ?? fallbackUri.fsPath;
      const list = byFile.get(target) ?? [];
      list.push(d);
      byFile.set(target, list);
    }
    if (byFile.size === 0) {
      const trimmed = text.trim();
      if (trimmed) {
        const range = new vscode.Range(0, 0, 0, 1);
        this.collection.set(fallbackUri, [
          new vscode.Diagnostic(
            range,
            trimmed.split(/\r?\n/)[0],
            vscode.DiagnosticSeverity.Error
          ),
        ]);
      } else {
        this.collection.delete(fallbackUri);
      }
      return;
    }
    for (const [file, diags] of byFile) {
      this.collection.set(vscode.Uri.file(file), diags);
    }
  }
}
