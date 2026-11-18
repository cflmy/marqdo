# Build Windows release assets (exe, bundle zip, stdlib zip, VSIX) and upload
# to an existing GitHub Release tag.
# Usage (from repo root):
#   powershell -File ./scripts/release-windows.ps1
#   powershell -File ./scripts/release-windows.ps1 -Tag v0.1.2
param(
    [string]$Tag = ""
)

$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)

if (-not $Tag) {
    $Tag = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"').Matches.Groups[1].Value
    $Tag = "v$Tag"
}
$ver = $Tag.TrimStart("v")
$target = "x86_64-pc-windows-msvc"
$dist = Join-Path (Get-Location) "dist"
$stage = Join-Path $dist "stage-bundle"
$stageLib = Join-Path $dist "stage-stdlib"
New-Item -ItemType Directory -Force -Path $dist | Out-Null

Write-Host "Building release binary for $Tag ..."
cargo build --release -q

$exeName = "marqdo-$ver-$target.exe"
$zipName = "marqdo-$ver-$target.zip"
$stdlibName = "marqdo-$ver-stdlib.zip"
$vsixName = "marqdo-$ver.vsix"
$exePath = Join-Path $dist $exeName
$zipPath = Join-Path $dist $zipName
$stdlibPath = Join-Path $dist $stdlibName
$vsixPath = Join-Path $dist $vsixName

Copy-Item ".\target\release\marqdo.exe" $exePath -Force

Write-Host "Building VS Code extension $vsixName (from branch vscode-extension) ..."
git fetch origin vscode-extension 2>$null
git checkout origin/vscode-extension -- vscode-extension
Push-Location vscode-extension
npm ci --silent
npm run compile
npx @vscode/vsce package --no-dependencies -o $vsixPath
Pop-Location

# Bundle: marqdo.exe + lib/ (optional disk override)
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Force -Path $stage | Out-Null
Copy-Item ".\target\release\marqdo.exe" (Join-Path $stage "marqdo.exe") -Force
Copy-Item ".\lib" (Join-Path $stage "lib") -Recurse -Force
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $zipPath -Force

# Stdlib-only zip (override / mirror)
if (Test-Path $stageLib) { Remove-Item $stageLib -Recurse -Force }
New-Item -ItemType Directory -Force -Path $stageLib | Out-Null
Copy-Item ".\lib" (Join-Path $stageLib "lib") -Recurse -Force
if (Test-Path $stdlibPath) { Remove-Item $stdlibPath -Force }
Compress-Archive -Path (Join-Path $stageLib "*") -DestinationPath $stdlibPath -Force

Write-Host "Packaged:"
Write-Host "  $exePath  (CLI with embedded stdlib)"
Write-Host "  $zipPath  (marqdo.exe + lib/)"
Write-Host "  $stdlibPath  (lib/ only)"
Write-Host "  $vsixPath  (VS Code extension)"

$raw = ("protocol=https`nhost=github.com`n`n" | git credential fill 2>$null) -join "`n"
$user = ([regex]::Match($raw, '(?m)^username=(.+)$')).Groups[1].Value
$token = ([regex]::Match($raw, '(?m)^password=(.+)$')).Groups[1].Value
if (-not $token) { throw "No GitHub credentials from git credential fill; run gh auth login or configure git credentials." }
$pair = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("${user}:${token}"))
$headers = @{
    Authorization = "Basic $pair"
    Accept        = "application/vnd.github+json"
    "User-Agent"  = "marqdo-release-windows"
}

$rel = Invoke-RestMethod -Uri "https://api.github.com/repos/cflmy/marqdo/releases/tags/$Tag" -Headers $headers
Write-Host "Uploading to release $($rel.html_url)"

$notes = @"
## Marqdo $Tag

### Downloads

| Asset | Contents |
|-------|----------|
| ``$exeName`` | CLI with **embedded** official stdlib (``lib/*.mq.md``) |
| ``$zipName`` | ``marqdo.exe`` + ``lib/`` (optional disk override) |
| ``$stdlibName`` | Standard library only — unpack to override embedded lib |
| ``$vsixName`` | **VS Code / Cursor extension** — built from branch ``vscode-extension`` only (see doc/design/vscode-extension-commit.md) |

### Highlights
- **Embedded stdlib**: standalone ``.exe`` imports ``lib/…`` without a separate zip
- **writeback** + **subtask** (file / function / foreign concurrency)
- **v0.2 syntax**: ``+`param` ``, ``1.`` branches, backtick identifiers, quoted strings
- ``marqdo version --check`` compares with GitHub latest release

### Quick start

``````text
marqdo --version
marqdo version --check
marqdo run public/00-welcome.mq.md
marqdo view public
``````

See [CHANGELOG.md](https://github.com/cflmy/marqdo/blob/main/CHANGELOG.md).
"@

$body = @{
    tag_name   = $Tag
    name       = "Marqdo $Tag"
    body       = $notes
    draft      = $false
    prerelease = $false
} | ConvertTo-Json -Depth 5
Invoke-RestMethod -Method Patch -Uri "https://api.github.com/repos/cflmy/marqdo/releases/$($rel.id)" -Headers $headers -ContentType "application/json; charset=utf-8" -Body ([Text.Encoding]::UTF8.GetBytes($body)) | Out-Null

$rel = Invoke-RestMethod -Uri "https://api.github.com/repos/cflmy/marqdo/releases/$($rel.id)" -Headers $headers

foreach ($file in @($exeName, $zipName, $stdlibName, $vsixName)) {
    $path = Join-Path $dist $file
    $existing = $rel.assets | Where-Object { $_.name -eq $file }
    if ($existing) {
        Invoke-RestMethod -Method Delete -Uri $existing.url -Headers $headers | Out-Null
        Write-Host "  deleted old $file"
    }
    $uri = "https://uploads.github.com/repos/cflmy/marqdo/releases/$($rel.id)/assets?name=$([uri]::EscapeDataString($file))"
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $ctype = if ($file.EndsWith(".zip")) { "application/zip" } elseif ($file.EndsWith(".vsix")) { "application/vsix" } else { "application/octet-stream" }
    $asset = Invoke-RestMethod -Method Post -Uri $uri -Headers $headers -ContentType $ctype -Body $bytes
    Write-Host "  $($asset.browser_download_url)"
}

Write-Host "OK"
