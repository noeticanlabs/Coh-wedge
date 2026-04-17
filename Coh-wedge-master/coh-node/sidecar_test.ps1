$BaseUrl = "http://127.0.0.1:3030"

function Test-Health {
    Write-Host "Testing Health Check..." -ForegroundColor Cyan
    Invoke-RestMethod -Uri "$BaseUrl/health" -Method Get
}

function Test-VerifyMicro {
    Write-Host "`nTesting /v1/verify-micro (Valid)..." -ForegroundColor Cyan
    $Receipt = @{
        schema_id = "coh.receipt.micro.v1"
        version = "1.0.0"
        object_id = "agent.workflow.test"
        canon_profile_hash = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
        policy_hash = "0" * 64
        step_index = 0
        state_hash_prev = "1" * 64
        state_hash_next = "2" * 64
        chain_digest_prev = "0" * 64
        chain_digest_next = "76114b520738a80e18048c2a37734c97b17d6f7e06f94393bbce7949bb87381c"
        metrics = @{
            v_pre = "100"
            v_post = "80"
            spend = "20"
            defect = "0"
        }
    }
    $Resp = Invoke-RestMethod -Uri "$BaseUrl/v1/verify-micro" -Method Post -Body ($Receipt | ConvertTo-Json) -ContentType "application/json"
    $Resp | ConvertTo-Json -Depth 5
}

function Test-VerifyMicro-Invalid {
    Write-Host "`nTesting /v1/verify-micro (Invalid - E003 Policy Violation)..." -ForegroundColor Cyan
    $Receipt = @{
        schema_id = "coh.receipt.micro.v1"
        version = "1.0.0"
        object_id = "agent.workflow.test"
        canon_profile_hash = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
        policy_hash = "0" * 64
        step_index = 0
        state_hash_prev = "1" * 64
        state_hash_next = "2" * 64
        chain_digest_prev = "0" * 64
        chain_digest_next = "0" * 64
        metrics = @{
            v_pre = "100"
            v_post = "200" # VIOLATION: post > pre
            spend = "0"
            defect = "0"
        }
    }
    $Resp = Invoke-RestMethod -Uri "$BaseUrl/v1/verify-micro" -Method Post -Body ($Receipt | ConvertTo-Json) -ContentType "application/json"
    $Resp | ConvertTo-Json -Depth 5
}

function Test-Execute-Verified {
    Write-Host "`nTesting /v1/execute-verified (Gatekeeper)..." -ForegroundColor Cyan
    $Payload = @{
        receipt = @{
            schema_id = "coh.receipt.micro.v1"
            version = "1.0.0"
            object_id = "agent.workflow.test"
            canon_profile_hash = "4fb5a33116a4e393ad7900f0744e8ec5d1b7a2d67d71003666d628d7a1cded09"
            policy_hash = "0" * 64
            step_index = 0
            state_hash_prev = "1" * 64
            state_hash_next = "2" * 64
            chain_digest_prev = "0" * 64
            chain_digest_next = "76114b520738a80e18048c2a37734c97b17d6f7e06f94393bbce7949bb87381c"
            metrics = @{ v_pre = "100"; v_post = "80"; spend = "20"; defect = "0" }
        }
        action = @{
            cmd = "TRANSFER"
            amount = 20
        }
    }
    $Resp = Invoke-RestMethod -Uri "$BaseUrl/v1/execute-verified" -Method Post -Body ($Payload | ConvertTo-Json -Depth 5) -ContentType "application/json"
    $Resp | ConvertTo-Json -Depth 5
}

Write-Host "Coh Sidecar Verification Suite" -ForegroundColor Yellow
Write-Host "-------------------------------" -ForegroundColor Yellow
Write-Host "Note: Ensure sidecar is running (cargo run -p coh-sidecar)" -ForegroundColor Gray

Test-Health
Test-VerifyMicro
Test-VerifyMicro-Invalid
Test-Execute-Verified
