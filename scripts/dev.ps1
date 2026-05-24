# Start Postgres (Docker), API, and dashboard for local development.
param(
    [switch]$SkipDocker
)

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot

if (-not $SkipDocker) {
    Push-Location $Root
    docker compose up -d
    Pop-Location
    Write-Host "Waiting for PostgreSQL..."
    Start-Sleep -Seconds 3
}

Write-Host "Run API:    cargo run -p netchronicle-api"
Write-Host "Run agent:  cargo run -p netchronicle-agent"
Write-Host "Dashboard:  cd apps/dashboard && npm run dev"
