# Deploy public/ static site to branch gh-pages.
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

if (-not (Test-Path "public\index.html")) {
    Write-Host "Building public site..."
    cargo build --release -q
    & .\target\release\marqdo.exe view output public -o public
}

$staging = Join-Path $Root "dist\gh-pages-publish"
if (Test-Path $staging) {
    $prev = $ErrorActionPreference
    $ErrorActionPreference = "Continue"
    git worktree remove $staging -f 2>&1 | Out-Null
    $ErrorActionPreference = $prev
    Remove-Item $staging -Recurse -Force -ErrorAction SilentlyContinue
}

Invoke-Git fetch origin gh-pages
Invoke-Git worktree add -B gh-pages $staging origin/gh-pages

Get-ChildItem $staging -Force | Where-Object { $_.Name -ne ".git" } | Remove-Item -Recurse -Force
Copy-Item -Recurse (Join-Path $Root "public\*") $staging -Force

Set-Location $staging
Invoke-Git add -A
$status = git status --porcelain
if ($status) {
    Invoke-Git commit -m "Deploy public site for v0.1.2"
    Invoke-Git push origin gh-pages
    Write-Host "Pushed gh-pages"
} else {
    Write-Host "gh-pages unchanged"
}

Set-Location $Root
$prev = $ErrorActionPreference
$ErrorActionPreference = "Continue"
git worktree remove $staging -f 2>&1 | Out-Null
$ErrorActionPreference = $prev
