@echo off
rem Double-click to upload the ready-made Windows and Linux files from the dist folder
rem to the newest release that already exists on GitHub (for example one made on the website).
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0packaging\publish-release.ps1" latest
echo.
pause
