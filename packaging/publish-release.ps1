# Publishes Stagecraft 3D on GitHub as a Release with ready-to-run files (no compiling on GitHub
# for Windows and Linux). Easiest: double-click "Publish release.cmd" in the project folder.
#
#  1. Sends your latest commits to GitHub.
#  2. Creates the release v<version> (version from Cargo.toml) and uploads every finished file in
#     dist\ for that version: Windows exe / portable zip / MSI (64 and 32 bit) and Linux AppImage /
#     tar.gz / deb / rpm. Build them first with packaging\build-all.ps1.
#  3. GitHub then builds the Mac version (it can only be made on a Mac) and adds it to the same
#     release automatically (workflow "Add Mac version to release").
# To add the files to a release that already exists (for example one made on the GitHub
# website), give its tag:  "Publish release.cmd" v01
# Needs the GitHub CLI signed in (gh auth login).
param([string]$Tag = "")
# Errors are checked by exit code below (Windows PowerShell turns git/gh progress text into errors otherwise).
$ErrorActionPreference = "Continue"
Set-Location (Join-Path $PSScriptRoot "..")

$version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
$repo = (gh repo view --json nameWithOwner -q .nameWithOwner)
if (-not $repo) { Write-Host "GitHub CLI is not signed in: run 'gh auth login' first." -ForegroundColor Red; exit 1 }
if ($Tag -eq "latest") {
    $Tag = (gh release view --repo $repo --json tagName -q .tagName)
    if (-not $Tag) { Write-Host "There is no release on GitHub yet: use 'Publish release.cmd'." -ForegroundColor Red; exit 1 }
}
$tag = if ($Tag) { $Tag } else { "v$version" }

$files = @(Get-ChildItem dist -File -ErrorAction SilentlyContinue | Where-Object { $_.Name -like "*$version*" } | ForEach-Object FullName)
Write-Host ""
Write-Host "Stagecraft 3D - publishing release $tag to $repo" -ForegroundColor Cyan
if ($files.Count -eq 0) { Write-Host "No finished files for $version in dist\. Run packaging\build-all.ps1 first." -ForegroundColor Red; exit 1 }
$files | ForEach-Object { Write-Host "   $(Split-Path $_ -Leaf)" }
Write-Host ""

Write-Host "1/3  Sending your latest changes to GitHub..."
git push origin HEAD
if ($LASTEXITCODE -ne 0) { Write-Host "Could not push. Open GitHub Desktop, pull, then try again." -ForegroundColor Red; exit 1 }

gh release view $tag --repo $repo *> $null
$exists = ($LASTEXITCODE -eq 0)
if ($exists) {
    Write-Host "2/3  Release $tag already exists on GitHub: adding the files to it."
} else {
    Write-Host "2/3  Marking this version as $tag..."
    git tag -f -a $tag -m "Stagecraft 3D $version" | Out-Null
    git push -f origin $tag
    if ($LASTEXITCODE -ne 0) { Write-Host "Could not send the tag $tag." -ForegroundColor Red; exit 1 }
}

$notes = Join-Path ([System.IO.Path]::GetTempPath()) "stagecraft-release-notes.md"
$utf8 = New-Object System.Text.UTF8Encoding $false
$text = [System.IO.File]::ReadAllText((Resolve-Path packaging\release-notes.md).Path, $utf8).Replace("@VERSION@", $version)
[System.IO.File]::WriteAllText($notes, $text, $utf8)

Write-Host "3/3  Uploading $($files.Count) files (this can take a few minutes)..."
if ($exists) {
    gh release upload $tag @files --repo $repo --clobber
} else {
    gh release create $tag @files --repo $repo --verify-tag --title "Stagecraft 3D $tag" --notes-file $notes
}
if ($LASTEXITCODE -ne 0) { Write-Host "Upload failed." -ForegroundColor Red; exit 1 }

Write-Host ""
Write-Host "Done! Release $tag is on GitHub. The Mac version is added automatically in about 10 minutes." -ForegroundColor Green
Start-Process "https://github.com/$repo/releases/tag/$tag"
