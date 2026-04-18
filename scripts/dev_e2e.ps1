$ErrorActionPreference = 'Stop'

Write-Host "[dev_e2e] Starting Sidecar..." -ForegroundColor Cyan

$sidecar = Start-Process powershell -ArgumentList @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', 'scripts/sidecar_up.ps1') -PassThru

try {
    Write-Host "[dev_e2e] Waiting for /health..." -ForegroundColor Cyan
    $ok = $false
    for ($i = 0; $i -lt 120; $i++) {
        try {
            $r = Invoke-WebRequest -UseBasicParsing http://127.0.0.1:3030/health -TimeoutSec 2
            if ($r.StatusCode -eq 200) { $ok = $true; break }
        }
        catch {}
        Start-Sleep -Seconds 1
    }
    if (-not $ok) { throw "Sidecar not ready after 120s" }

    Write-Host "[dev_e2e] Running APE demo against Sidecar..." -ForegroundColor Cyan
    powershell -NoProfile -ExecutionPolicy Bypass -File scripts/ape_demo_sidecar.ps1 -SidecarUrl "http://127.0.0.1:3030" -Seed 42 -Action "transfer_100_tokens"

    $artifact = Join-Path (Resolve-Path ape) 'output/demo_e2e.json'
    if (Test-Path $artifact) {
        Write-Host "[dev_e2e] SUCCESS: Artifact: $artifact" -ForegroundColor Green
    }
    else {
        throw "[dev_e2e] Artifact missing: ape/output/demo_e2e.json"
    }
}
finally {
    if ($sidecar -and !$sidecar.HasExited) {
        Write-Host "[dev_e2e] Stopping Sidecar..." -ForegroundColor Yellow
        Stop-Process -Id $sidecar.Id -Force -ErrorAction SilentlyContinue
    }
}
