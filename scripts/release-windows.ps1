# Build Windows release assets and upload them to an existing GitHub Release tag.
# Usage (from repo root):
#   powershell -File ./scripts/release-windows.ps1
#   powershell -File ./scripts/release-windows.ps1 -Tag v0.0.4
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
New-Item -ItemType Directory -Force -Path $dist | Out-Null

Write-Host "Building release binary for $Tag ..."
cargo build --release -q

$exeName = "marqdo-$ver-$target.exe"
$zipName = "marqdo-$ver-$target.zip"
$exePath = Join-Path $dist $exeName
$zipPath = Join-Path $dist $zipName
Copy-Item ".\target\release\marqdo.exe" $exePath -Force
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path $exePath -DestinationPath $zipPath -Force

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

foreach ($file in @($exeName, $zipName)) {
    $path = Join-Path $dist $file
    # Replace existing asset with same name
    $existing = $rel.assets | Where-Object { $_.name -eq $file }
    if ($existing) {
        Invoke-RestMethod -Method Delete -Uri $existing.url -Headers $headers | Out-Null
    }
    $uri = "https://uploads.github.com/repos/cflmy/marqdo/releases/$($rel.id)/assets?name=$([uri]::EscapeDataString($file))"
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $ctype = if ($file.EndsWith(".zip")) { "application/zip" } else { "application/octet-stream" }
    $asset = Invoke-RestMethod -Method Post -Uri $uri -Headers $headers -ContentType $ctype -Body $bytes
    Write-Host "  $($asset.browser_download_url)"
}

Write-Host "OK"
