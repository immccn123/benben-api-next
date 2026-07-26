#!/usr/bin/env pwsh
$ErrorActionPreference = 'Stop'
Set-Location -LiteralPath $PSScriptRoot

$projectRoot = (Resolve-Path "$PSScriptRoot\..\..\..").Path
$envFile = "$PSScriptRoot\..\.env"

Write-Host "=== SSO Setup ===" -ForegroundColor Cyan

Write-Host "[1/2] Building..." -ForegroundColor Yellow
Push-Location $projectRoot
cargo build --bin keygen -p auth 2>&1 | Out-Null
if ($LASTEXITCODE -ne 0) { Write-Host "Build failed" -ForegroundColor Red; Pop-Location; exit 1 }
Pop-Location

Write-Host "[2/2] Generating server keys..." -ForegroundColor Yellow
Push-Location $projectRoot
$out = & cargo run --bin keygen -p auth -- server 2>&1 | Out-String
Pop-Location

$lines = @{}
foreach ($line in ($out -split "`n")) {
    if ($line -match '^(AUTH_\w+)=(.*)') { $lines[$matches[1]] = $matches[2] }
}

@"
# === SSO Configuration ===
# Generated: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')

AUTH_BASE_URL=http://127.0.0.1:3000
AUTH_LISTEN_ADDR=127.0.0.1:3000

AUTH_CLIENT_VERIFY_KEY=$($lines['AUTH_CLIENT_VERIFY_KEY'])
AUTH_JWT_SIGN_KEY=$($lines['AUTH_JWT_SIGN_KEY'])
AUTH_JWT_VERIFY_KEY=$($lines['AUTH_JWT_VERIFY_KEY'])

CPOAUTH_BASE_URL=
CPOAUTH_CLIENT_ID=
CPOAUTH_CLIENT_SECRET=
CPOAUTH_REDIRECT_URI=
"@ | Out-File -FilePath $envFile -Encoding utf8

Write-Host ""
Write-Host ".env written. Fill in CPOAUTH_* and run:" -ForegroundColor Green
Write-Host "  cargo run --bin keygen -- app -c my-app --callback-url https://my.app/callback"
Write-Host "  cargo run -p auth"
