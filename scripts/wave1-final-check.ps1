# Wave 1 Final Verification Script (PowerShell)

Write-Host "============================================" -ForegroundColor Cyan
Write-Host "  Wave 1 Final Verification" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan

$errors = 0
$warnings = 0

# 1. ビルド確認
Write-Host "`n1. Building release binary..." -ForegroundColor Green
try {
    cargo build --release 2>&1 | Out-Null
    if ($LASTEXITCODE -eq 0) {
        Write-Host "   ✓ Release build successful" -ForegroundColor Green
    } else {
        Write-Host "   ✗ Release build failed" -ForegroundColor Red
        $errors += 1
    }
} catch {
    Write-Host "   ✗ Build error: $_" -ForegroundColor Red
    $errors += 1
}

# 2. テスト実行
Write-Host "`n2. Running all tests..." -ForegroundColor Green
try {
    $testOutput = cargo test 2>&1
    if ($LASTEXITCODE -eq 0) {
        # Count passed tests
        $passedTests = ([regex]::Matches($testOutput, "test.*ok")).Count
        Write-Host "   ✓ All tests passed ($passedTests tests)" -ForegroundColor Green
    } else {
        Write-Host "   ✗ Some tests failed" -ForegroundColor Red
        Write-Host $testOutput
        $errors += 1
    }
} catch {
    Write-Host "   ✗ Test error: $_" -ForegroundColor Red
    $errors += 1
}

# 3. バイナリサイズ確認
Write-Host "`n3. Checking binary size..." -ForegroundColor Green
$binaryPath = "target\release\opencode_poc.exe"
if (Test-Path $binaryPath) {
    $sizeBytes = (Get-Item $binaryPath).Length
    $sizeMB = [math]::Round($sizeBytes / 1MB, 2)

    Write-Host "   Binary size: $sizeMB MB" -ForegroundColor Yellow
    if ($sizeMB -lt 20) {
        Write-Host "   ✓ Size is within acceptable range" -ForegroundColor Green
    } else {
        Write-Host "   ⚠ Size is larger than expected" -ForegroundColor Yellow
        $warnings += 1
    }
} else {
    Write-Host "   ✗ Binary not found at $binaryPath" -ForegroundColor Red
    $errors += 1
}

# 4. Cargo.toml 確認
Write-Host "`n4. Checking Cargo.toml..." -ForegroundColor Green
$cargoToml = Get-Content "Cargo.toml"
if ($cargoToml -match "profile\.release") {
    Write-Host "   ✓ Release profile configured" -ForegroundColor Green
} else {
    Write-Host "   ⚠ Release profile not found (may not be critical)" -ForegroundColor Yellow
    $warnings += 1
}

# 5. ドキュメント確認
Write-Host "`n5. Checking documentation..." -ForegroundColor Green
$requiredDocs = @(
    "docs\API_SPECIFICATION.md",
    "docs\SETUP_GUIDE.md",
    "DEPLOYMENT.md"
)

$missingDocs = 0
foreach ($doc in $requiredDocs) {
    if (Test-Path $doc) {
        Write-Host "   ✓ Found: $doc" -ForegroundColor Green
    } else {
        Write-Host "   ✗ Missing: $doc" -ForegroundColor Red
        $missingDocs += 1
    }
}

if ($missingDocs -gt 0) {
    $errors += $missingDocs
}

# 6. ロードテストスクリプト確認
Write-Host "`n6. Checking load test scripts..." -ForegroundColor Green
$loadTestScripts = @(
    "tests\load\k6-auth-load.js",
    "tests\load\k6-file-upload.js",
    "tests\load\run-load-tests.ps1"
)

foreach ($script in $loadTestScripts) {
    if (Test-Path $script) {
        Write-Host "   ✓ Found: $script" -ForegroundColor Green
    } else {
        Write-Host "   ✗ Missing: $script" -ForegroundColor Red
        $errors += 1
    }
}

# 7. E2E テストスクリプト確認
Write-Host "`n7. Checking E2E test scripts..." -ForegroundColor Green
if (Test-Path "tests\e2e\api-flow.test.ts") {
    Write-Host "   ✓ Found: tests\e2e\api-flow.test.ts" -ForegroundColor Green
} else {
    Write-Host "   ✗ Missing: tests\e2e\api-flow.test.ts" -ForegroundColor Red
    $errors += 1
}

# サマリー
Write-Host "`n============================================" -ForegroundColor Cyan
Write-Host "  Verification Summary" -ForegroundColor Cyan
Write-Host "============================================" -ForegroundColor Cyan

if ($errors -eq 0) {
    Write-Host "`n✓ All checks passed!" -ForegroundColor Green
    Write-Host "`nWave 1 Status: READY FOR RELEASE" -ForegroundColor Green
    Write-Host "`nNext Steps:" -ForegroundColor Yellow
    Write-Host "  1. Run load tests: .\tests\load\run-load-tests.ps1" -ForegroundColor Yellow
    Write-Host "  2. Run E2E tests: cargo test --test e2e" -ForegroundColor Yellow
    Write-Host "  3. Review DEPLOYMENT.md for production deployment" -ForegroundColor Yellow
    exit 0
} else {
    Write-Host "`n✗ Verification failed with $errors error(s) and $warnings warning(s)" -ForegroundColor Red
    Write-Host "`nPlease fix the errors above before proceeding." -ForegroundColor Red
    exit 1
}
