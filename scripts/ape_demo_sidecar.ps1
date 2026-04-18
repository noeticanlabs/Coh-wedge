param(
    [string]$SidecarUrl = "http://127.0.0.1:3030",
    [int]$Seed = 42,
    [string]$Action = "transfer_100_tokens"
)

$ErrorActionPreference = 'Stop'

Write-Host "[ape_demo_sidecar] Running APE demo against $SidecarUrl (seed=$Seed, action=$Action)" -ForegroundColor Cyan

Push-Location ape
try {
    if (-not (Test-Path target_ape)) { New-Item -ItemType Directory -Path target_ape | Out-Null }
    $env:CARGO_TARGET_DIR = (Resolve-Path target_ape)
    cargo run -j 1 --release -- demo --mode sidecar --sidecar-url $SidecarUrl --seed $Seed --action $Action | Write-Host
    $artifact = Join-Path (Get-Location) 'output/demo_e2e.json'
    if (Test-Path $artifact) {
        Write-Host "[ape_demo_sidecar] Artifact: $artifact" -ForegroundColor Green
    }
    else {
        Write-Host "[ape_demo_sidecar] Artifact not found (expected output/demo_e2e.json)" -ForegroundColor Yellow
    }
}
finally {
    Pop-Location
}
