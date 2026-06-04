# Load tests runner (PowerShell)
Write-Host "Starting load tests..." -ForegroundColor Green

# Check if k6 is installed
try {
    k6 --version | Out-Null
} catch {
    Write-Host "ERROR: k6 is not installed. Install from https://k6.io/docs/getting-started/installation/" -ForegroundColor Red
    exit 1
}

# Test file fixture preparation
if (-not (Test-Path "tests/fixtures/test-file.bin")) {
    Write-Host "Creating test fixture..." -ForegroundColor Yellow
    [byte[]]$bytes = 1..1024 | ForEach-Object { [byte]$_ % 256 }
    [System.IO.File]::WriteAllBytes("tests/fixtures/test-file.bin", $bytes)
}

# Auth API load test
Write-Host "Running auth API load test..." -ForegroundColor Cyan
k6 run --summary-export=auth-load-results.json tests/load/k6-auth-load.js
if ($LASTEXITCODE -ne 0) {
    Write-Host "Auth load test failed!" -ForegroundColor Red
    exit 1
}

# File upload load test
Write-Host "Running file upload load test..." -ForegroundColor Cyan
k6 run --summary-export=file-load-results.json tests/load/k6-file-upload.js
if ($LASTEXITCODE -ne 0) {
    Write-Host "File upload load test failed!" -ForegroundColor Red
    exit 1
}

Write-Host "Load tests completed. Results:" -ForegroundColor Green
Write-Host "  - Auth results: auth-load-results.json"
Write-Host "  - File upload results: file-load-results.json"
