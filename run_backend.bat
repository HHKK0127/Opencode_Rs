@echo off
cd C:\Drive\Cargo\OpenCode_Rs

set JWT_SECRET=test_secret_key_32_bytes_exactly
set DATABASE_URL=sqlite:///C:/Drive/Cargo/OpenCode_Rs/poc_test.db
set ENVIRONMENT=development
set TEST_USER_PASSWORD=testpassword

del /F /Q poc_test.db 2>nul

echo ===== OpenCode Backend Server =====
.\target\release\opencode_poc.exe

pause
