param(
  [string]$CohHostAddress = "127.0.0.1",
  [int]$Port = 3030
)

$ErrorActionPreference = 'Stop'

Write-Host "[sidecar_up] Starting Sidecar on ${CohHostAddress}:$Port..." -ForegroundColor Cyan

$env:COH_HOST = $CohHostAddress
$env:COH_PORT = "$Port"

Push-Location coh-node
try {
  if (-not (Test-Path target_sidecar)) { New-Item -ItemType Directory -Path target_sidecar | Out-Null }
  $env:CARGO_TARGET_DIR = (Resolve-Path target_sidecar)
  cargo run -j 1 -p coh-sidecar | Write-Host
} finally {
  Pop-Location
}
