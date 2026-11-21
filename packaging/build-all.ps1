# Builds every Windows and Linux release file into dist\ on this Windows PC, then copies the
# finished files and release notes into releases\v<version>\ . Commit that folder and click
# "Push origin" in GitHub Desktop: GitHub then creates the release and adds the Mac version.
# Run from the project folder:  powershell -ExecutionPolicy Bypass -File packaging\build-all.ps1
$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
& (Join-Path $PSScriptRoot "build-windows.ps1")
& (Join-Path $PSScriptRoot "build-linux-from-windows.ps1")
& (Join-Path $PSScriptRoot "stage-release.ps1")
