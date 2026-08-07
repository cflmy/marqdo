# Build all Marqdo v0.1.2+ release assets and optionally upload to GitHub Releases.
# Usage (repo root):
#   powershell -File ./scripts/release-full.ps1
#   powershell -File ./scripts/release-full.ps1 -Tag v0.1.2 -Upload
#   powershell -File ./scripts/release-full.ps1 -SkipPublic
param(
    [string]$Tag = "",
    [switch]$Upload,
    [switch]$DeployPages,
    [switch]$SkipPublic,
    [switch]$SkipVsix
)

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent
Set-Location $Root

function Invoke-Git {
    param([Parameter(ValueFromRemainingArguments = $true)][string[]]$GitArgs)
    $prev = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    & git @GitArgs 2>&1 | Out-Null
    $code = $LASTEXITCODE
    $ErrorActionPreference = $prev
    if ($code -ne 0) { throw "git $($GitArgs -join ' ') failed (exit $code)" }
}

if (-not $Tag) {
    $Tag = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"').Matches.Groups[1].Value
    $Tag = "v$Tag"
}
$ver = $Tag.TrimStart("v")
$target = "x86_64-pc-windows-msvc"
$dist = Join-Path $Root "dist"
New-Item -ItemType Directory -Force -Path $dist | Out-Null

$exeName = "marqdo-$ver-$target.exe"
$zipName = "marqdo-$ver-$target.zip"
$stdlibName = "marqdo-$ver-stdlib.zip"
$sourceName = "marqdo-$ver-source.zip"
$vsixName = "marqdo-$ver.vsix"
$publicName = "marqdo-$ver-public.zip"

Write-Host "=== Marqdo release $Tag ==="

Write-Host "[1/6] cargo build --release"
cargo build --release -q

Write-Host "[2/6] binary + bundle + stdlib zips"
$exePath = Join-Path $dist $exeName
Copy-Item ".\target\release\marqdo.exe" $exePath -Force

$stage = Join-Path $dist "stage-bundle"
if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
New-Item -ItemType Directory -Force -Path $stage | Out-Null
Copy-Item ".\target\release\marqdo.exe" (Join-Path $stage "marqdo.exe") -Force
Copy-Item ".\lib" (Join-Path $stage "lib") -Recurse -Force
$zipPath = Join-Path $dist $zipName
if (Test-Path $zipPath) { Remove-Item $zipPath -Force }
Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $zipPath -Force

$stageLib = Join-Path $dist "stage-stdlib"
if (Test-Path $stageLib) { Remove-Item $stageLib -Recurse -Force }
New-Item -ItemType Directory -Force -Path $stageLib | Out-Null
Copy-Item ".\lib" (Join-Path $stageLib "lib") -Recurse -Force
$stdlibPath = Join-Path $dist $stdlibName
if (Test-Path $stdlibPath) { Remove-Item $stdlibPath -Force }
Compress-Archive -Path (Join-Path $stageLib "*") -DestinationPath $stdlibPath -Force

Write-Host "[3/6] source archive (main @ HEAD, no .git)"
$sourcePath = Join-Path $dist $sourceName
if (Test-Path $sourcePath) { Remove-Item $sourcePath -Force }
git archive --format=zip -o $sourcePath HEAD
if ($LASTEXITCODE -ne 0) { throw "git archive failed" }

if (-not $SkipVsix) {
    Write-Host "[4/6] VSIX (branch vscode-extension)"
    Invoke-Git fetch origin vscode-extension
    if (Test-Path "vscode-extension") { Remove-Item "vscode-extension" -Recurse -Force }
    Invoke-Git checkout origin/vscode-extension -- vscode-extension
    Push-Location vscode-extension
    npm ci --silent
    npm run compile
    $vsixPath = Join-Path $dist $vsixName
    if (Test-Path $vsixPath) { Remove-Item $vsixPath -Force }
    npx --yes @vscode/vsce package --no-dependencies -o $vsixPath
    Pop-Location
    Remove-Item "vscode-extension" -Recurse -Force -ErrorAction SilentlyContinue
} else {
    Write-Host "[4/6] VSIX skipped"
}

if (-not $SkipPublic) {
    Write-Host "[5/6] public static site"
    & .\target\release\marqdo.exe view output public -o public
    if (-not (Test-Path "public\index.html")) { throw "public/index.html missing after view output" }
    $publicPath = Join-Path $dist $publicName
    if (Test-Path $publicPath) { Remove-Item $publicPath -Force }
    Compress-Archive -Path "public\*" -DestinationPath $publicPath -Force
} else {
    Write-Host "[5/6] public skipped"
}

Write-Host "[6/6] dist summary"
Get-ChildItem $dist -File | ForEach-Object { Write-Host ("  {0,12:N0} bytes  {1}" -f $_.Length, $_.Name) }

if ($DeployPages) {
    Write-Host "Deploying gh-pages..."
    if (-not (Test-Path "public\index.html")) {
        & .\target\release\marqdo.exe view output public -o public
    }
    $pagesDir = Join-Path $dist "gh-pages-staging"
    if (Test-Path $pagesDir) { Remove-Item $pagesDir -Recurse -Force }
    New-Item -ItemType Directory -Force -Path $pagesDir | Out-Null
    Copy-Item -Recurse public\* $pagesDir -Force
    # peaceiris-style: only site artifacts on gh-pages
    git worktree add $pagesDir gh-pages 2>$null
    if ($LASTEXITCODE -ne 0) {
        git branch -D gh-pages 2>$null
        git checkout --orphan gh-pages-temp
        git reset --hard
        git checkout main
        git branch -D gh-pages-temp 2>$null
        git checkout -b gh-pages
        Copy-Item -Recurse (Join-Path $Root "public\*") . -Force
        git add -A
        git commit -m "Deploy public site for $Tag"
        git push -f origin gh-pages
        git checkout main
    } else {
        Write-Host "gh-pages worktree exists; copy public/ manually or use CI pages workflow"
    }
}

if ($Upload) {
    Write-Host "Uploading to GitHub Release $Tag ..."
    $assets = @($exeName, $zipName, $stdlibName, $sourceName)
    if (-not $SkipVsix) { $assets += $vsixName }
    if (-not $SkipPublic) { $assets += $publicName }

    $raw = ("protocol=https`nhost=github.com`n`n" | git credential fill 2>$null) -join "`n"
    $user = ([regex]::Match($raw, '(?m)^username=(.+)$')).Groups[1].Value
    $token = ([regex]::Match($raw, '(?m)^password=(.+)$')).Groups[1].Value
    if (-not $token) { throw "No GitHub token from git credential; run gh auth login" }
    $headers = @{
        Authorization = "Bearer $token"
        Accept        = "application/vnd.github+json"
        "User-Agent"  = "marqdo-release-full"
    }

    $notes = @"
## Marqdo $Tag

### Downloads

| Asset | Contents |
|-------|----------|
| ``$exeName`` | Windows CLI (**embedded** official ``lib/``) |
| ``$zipName`` | ``marqdo.exe`` + ``lib/`` (optional disk override) |
| ``$stdlibName`` | Standard library ``lib/*.mq.md`` only |
| ``$sourceName`` | Source tree snapshot (``git archive`` of ``main``) |
| ``$vsixName`` | VS Code / Cursor extension (from branch ``vscode-extension``) |
| ``$publicName`` | Static user docs site (also at [gh-pages](https://cflmy.github.io/marqdo/)) |

### Highlights
- Embedded stdlib, **writeback** + **subtask**, v0.2 syntax, ``marqdo version --check``
- Extension source: branch ``vscode-extension`` — [vscode-extension-commit.md](https://github.com/cflmy/marqdo/blob/main/doc/design/vscode-extension-commit.md)

See [CHANGELOG.md](https://github.com/cflmy/marqdo/blob/main/CHANGELOG.md).
"@

    try {
        $rel = Invoke-RestMethod -Uri "https://api.github.com/repos/cflmy/marqdo/releases/tags/$Tag" -Headers $headers
        $releaseId = $rel.id
        $uploadUrl = $rel.upload_url -replace "\{.*", ""
        Write-Host "Updating release $($rel.html_url)"
        $body = @{ tag_name = $Tag; name = "Marqdo $Tag"; body = $notes; draft = $false; prerelease = $false } | ConvertTo-Json -Depth 5
        Invoke-RestMethod -Method Patch -Uri "https://api.github.com/repos/cflmy/marqdo/releases/$releaseId" -Headers $headers -ContentType "application/json; charset=utf-8" -Body ([Text.Encoding]::UTF8.GetBytes($body)) | Out-Null
    } catch {
        Write-Host "Creating release $Tag ..."
        $body = @{ tag_name = $Tag; name = "Marqdo $Tag"; body = $notes; draft = $false; prerelease = $false } | ConvertTo-Json -Depth 5
        $rel = Invoke-RestMethod -Method Post -Uri "https://api.github.com/repos/cflmy/marqdo/releases" -Headers $headers -ContentType "application/json; charset=utf-8" -Body ([Text.Encoding]::UTF8.GetBytes($body))
        $releaseId = $rel.id
        $uploadUrl = $rel.upload_url -replace "\{.*", ""
    }

    $rel = Invoke-RestMethod -Uri "https://api.github.com/repos/cflmy/marqdo/releases/$releaseId" -Headers $headers
    foreach ($file in $assets) {
        $path = Join-Path $dist $file
        if (-not (Test-Path $path)) { Write-Warning "skip missing $file"; continue }
        $existing = $rel.assets | Where-Object { $_.name -eq $file }
        if ($existing) {
            Invoke-RestMethod -Method Delete -Uri "https://api.github.com/repos/cflmy/marqdo/releases/assets/$($existing.id)" -Headers $headers | Out-Null
        }
        $uri = "$uploadUrl`?name=$([uri]::EscapeDataString($file))"
        $bytes = [System.IO.File]::ReadAllBytes($path)
        $ctype = switch -Regex ($file) {
            '\.zip$' { "application/zip" }
            '\.vsix$' { "application/vsix" }
            default { "application/octet-stream" }
        }
        $asset = Invoke-RestMethod -Method Post -Uri $uri -Headers $headers -ContentType $ctype -Body $bytes
        Write-Host "  uploaded $($asset.browser_download_url)"
    }
    Write-Host "OK https://github.com/cflmy/marqdo/releases/tag/$Tag"
}

Write-Host "Done."
