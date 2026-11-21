@echo off
rem Double-click to publish this version of Stagecraft 3D as a GitHub Release.
rem Uploads the ready-made Windows and Linux files from the dist folder;
rem GitHub then adds the Mac version. Build the files first with packaging\build-all.ps1.
rem To add the files to a release that already exists, run:  "Publish release.cmd" v01
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0packaging\publish-release.ps1" %*
echo.
pause
