# Copies the finished files for the current version from dist\ into releases\v<version>\ together
# with RELEASE-NOTES.md. Committing and pushing that folder (GitHub Desktop: Commit, then Push
# origin) makes GitHub publish the release (.github/workflows/publish-release.yml).
$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $root
$version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
$dest = Join-Path $root "releases\v$version"
New-Item -ItemType Directory -Force $dest | Out-Null

$files = @(Get-ChildItem dist -File | Where-Object { $_.Name -like "*$version*" -and $_.Name -notlike "*macos*" })
if ($files.Count -eq 0) { throw "No finished files for $version in dist\ - run packaging\build-all.ps1" }
foreach ($f in $files) { Copy-Item $f.FullName $dest -Force }

$utf8 = New-Object System.Text.UTF8Encoding $false
$notes = [System.IO.File]::ReadAllText((Join-Path $root "packaging\release-notes.md"), $utf8).Replace("@VERSION@", $version)
[System.IO.File]::WriteAllText((Join-Path $dest "RELEASE-NOTES.md"), $notes, $utf8)

Write-Host ""
Write-Host "Release v$version is ready in releases\v$version ($($files.Count) files)." -ForegroundColor Green
Write-Host "In GitHub Desktop: Commit to main, then Push origin. GitHub publishes the release."
Get-ChildItem $dest | Format-Table Name, Length
