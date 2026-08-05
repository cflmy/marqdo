# Build Windows release assets (exe, bundle zip with lib/, stdlib zip) and upload
# to an existing GitHub Release tag.
# Usage (from repo root):
#   powershell -File ./scripts/release-windows.ps1
#   powershell -File ./scripts/release-windows.ps1 -Tag v0.1.0
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
$exePath = Join-Path $dist $exeName
$zipPath = Join-Path $dist $zipName
$stdlibPath = Join-Path $dist $stdlibName

Copy-Item ".\target\release\marqdo.exe" $exePath -Force

# Bundle: marqdo.exe + lib/ (stdlib resolves next to the binary)
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Force -Path $stage | Out-Null
Copy-Item ".\target\release\marqdo.exe" (Join-Path $stage "marqdo.exe") -Force
Copy-Item ".\lib" (Join-Path $stage "lib") -Recurse -Force
Get-ChildItem -Path (Join-Path $stage "lib") -Recurse -Filter "*.mq.md" | Out-Null
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $zipPath -Force

# Stdlib-only zip (for users who already have the exe)
if (Test-Path $stageLib) { Remove-Item $stageLib -Recurse -Force }
New-Item -ItemType Directory -Force -Path $stageLib | Out-Null
Copy-Item ".\lib" (Join-Path $stageLib "lib") -Recurse -Force
if (Test-Path $stdlibPath) { Remove-Item $stdlibPath -Force }
Compress-Archive -Path (Join-Path $stageLib "*") -DestinationPath $stdlibPath -Force

Write-Host "Packaged:"
Write-Host "  $exePath  (binary only — no stdlib)"
Write-Host "  $zipPath  (marqdo.exe + lib/)"
Write-Host "  $stdlibPath  (lib/ only)"

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

# Refresh release notes
$notes = @"
## Marqdo $Tag

### Downloads

| Asset | Contents |
|-------|----------|
| ``$exeName`` | **Executable only** — does **not** include the standard library |
| ``$zipName`` | **Recommended**: ``marqdo.exe`` + ``lib/`` (place them together; ``lib/`` is resolved next to the binary) |
| ``$stdlibName`` | Standard library only (``lib/*.mq.md``) — unpack next to an existing ``marqdo.exe``, or set ``MARQDO_LIB`` |

> **Important:** the standalone ``.exe`` does **not** ship ``lib/``. Importing ``lib/text.mq.md`` etc. needs the bundle zip or the stdlib zip beside the binary (or ``MARQDO_LIB``).

### Highlights
- Code as docs as knowledge: ``.mq.md`` is narrative + program
- ``marqdo debug``: breakpoints / step / locals; ``marqdo view`` stays docs-only
- View: function outline + search; wider layout
- Roadmap: deeper OKF-aligned catalog for human–AI knowledge bases

``````text
marqdo --version
marqdo run public/00-welcome.mq.md
marqdo view public
marqdo debug public
``````
"@

$body = @{
    tag_name   = $Tag
    name       = $Tag
    body       = $notes
    draft      = $false
    prerelease = $false
} | ConvertTo-Json -Depth 5
Invoke-RestMethod -Method Patch -Uri "https://api.github.com/repos/cflmy/marqdo/releases/$($rel.id)" -Headers $headers -ContentType "application/json; charset=utf-8" -Body ([Text.Encoding]::UTF8.GetBytes($body)) | Out-Null

# Re-fetch assets list after patch
$rel = Invoke-RestMethod -Uri "https://api.github.com/repos/cflmy/marqdo/releases/$($rel.id)" -Headers $headers

foreach ($file in @($exeName, $zipName, $stdlibName)) {
    $path = Join-Path $dist $file
    $existing = $rel.assets | Where-Object { $_.name -eq $file }
    if ($existing) {
        Invoke-RestMethod -Method Delete -Uri $existing.url -Headers $headers | Out-Null
        Write-Host "  deleted old $file"
    }
    $uri = "https://uploads.github.com/repos/cflmy/marqdo/releases/$($rel.id)/assets?name=$([uri]::EscapeDataString($file))"
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $ctype = if ($file.EndsWith(".zip")) { "application/zip" } else { "application/octet-stream" }
    $asset = Invoke-RestMethod -Method Post -Uri $uri -Headers $headers -ContentType $ctype -Body $bytes
    Write-Host "  $($asset.browser_download_url)"
}

Write-Host "OK"
