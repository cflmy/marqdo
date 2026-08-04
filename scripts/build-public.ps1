# Generate HTML next to public/*.mq.md via the Marqdo interpreter.
$ErrorActionPreference = "Stop"
Set-Location (Split-Path $PSScriptRoot -Parent)
cargo build --release -q
& .\target\release\marqdo.exe view output public -o public
Write-Host "OK → public/index.html + public/pages/  (sources: public/**/*.mq.md)"
