@echo off
set PATH=%USERPROFILE%\.cargo\bin;%PATH%
cargo build --release
if %errorlevel% neq 0 (
    echo Build failed!
    exit /b 1
)
if not exist dist mkdir dist
copy /Y target\release\snapvault.exe dist\Jtu.exe >nul
echo Build OK: dist\Jtu.exe
