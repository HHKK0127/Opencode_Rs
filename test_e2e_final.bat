@echo off
setlocal enabledelayedexpansion

set API_URL=http://127.0.0.1:8080
set USERNAME=testuser
set PASSWORD=testpassword

echo =========================================
echo Wave 5 Final E2E Test ^(Windows^)
echo =========================================
echo.

REM 1. ???????
echo 1^> ???????...
curl -s "%API_URL%/health"
echo.
echo.

REM 2. ????
echo 2^> ???? (testuser/testpassword)...
curl -s -X POST "%API_URL%/api/v1/auth/login" ^
  -H "Content-Type: application/json" ^
  -d "{\"username\":\"%USERNAME%\",\"password\":\"%PASSWORD%\"}"
echo.
echo.

echo =========================================
echo Wave 5 Test Completed
echo =========================================
pause
