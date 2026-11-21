# Builds every Windows version of Stagecraft 3D into dist\ . For each one:
#   Stagecraft3D-<ver>-<name>.exe            the program on its own
#   Stagecraft3D-<ver>-<name>-portable.zip   program + README + LICENSE, run from anywhere
#   Stagecraft3D-<ver>-<name>.msi            installer: Start menu + desktop shortcuts, uninstall
# Versions (<name>):
#   windows-x64        Windows 10/11, 64-bit (Intel/AMD)
#   windows-x86        Windows 10/11, 32-bit
#   windows-arm64      Windows 10/11 on ARM (Snapdragon / Surface Pro X / Copilot+ PCs)
#   windows7-x64       Windows 7, 8 and 8.1, 64-bit (draws with OpenGL)
#   windows7-x86       Windows 7, 8 and 8.1, 32-bit (draws with OpenGL)
#
# Needs: Rust stable with targets x86_64-pc-windows-msvc, i686-pc-windows-msvc and
#   aarch64-pc-windows-gnullvm; Rust nightly with rust-src (for the Windows 7 targets);
#   the WiX Toolset v5 (dotnet tool install --global wix --version 5.0.2).
#   llvm-mingw (ARM64 linker) is downloaded into .tools\ automatically.
# Run from the project folder:  powershell -ExecutionPolicy Bypass -File packaging\build-windows.ps1
param([string]$Dist = "dist", [string[]]$Only = @())

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $root
$version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
$Only = @($Only | ForEach-Object { $_ -split "," } | Where-Object { $_ })
New-Item -ItemType Directory -Force $Dist | Out-Null
$tools = Join-Path $root ".tools"
New-Item -ItemType Directory -Force $tools | Out-Null

# llvm-mingw for the ARM64 build (Microsoft's ARM64 tools are not needed).
$llvmName = "llvm-mingw-20261117-ucrt-x86_64"
$llvm = Join-Path $tools "$llvmName\bin"
function Get-LlvmMingw {
    if (-not (Test-Path (Join-Path $llvm "aarch64-w64-mingw32-clang.exe"))) {
        Write-Host "Downloading llvm-mingw (ARM64 linker)..."
        $ProgressPreference = "SilentlyContinue"
        Invoke-WebRequest "https://github.com/mstorsjo/llvm-mingw/releases/download/20261117/$llvmName.zip" -OutFile "$tools\llvm-mingw.zip" -UseBasicParsing
        Expand-Archive "$tools\llvm-mingw.zip" -DestinationPath $tools
    }
}

$builds = @(
    @{ Name = "windows-x64";   Triple = "x86_64-pc-windows-msvc";     WixArch = "x64";   Kind = "stable" },
    @{ Name = "windows-x86";   Triple = "i686-pc-windows-msvc";       WixArch = "x86";   Kind = "stable" },
    @{ Name = "windows-arm64"; Triple = "aarch64-pc-windows-gnullvm"; WixArch = "arm64"; Kind = "llvm" },
    @{ Name = "windows7-x64";  Triple = "x86_64-win7-windows-msvc";   WixArch = "x64";   Kind = "win7" },
    @{ Name = "windows7-x86";  Triple = "i686-win7-windows-msvc";     WixArch = "x86";   Kind = "win7" }
)
foreach ($b in $builds) {
    if ($Only.Count -gt 0 -and $Only -notcontains $b.Name) { continue }
    $name = $b.Name; $triple = $b.Triple
    Write-Host "== $name ($triple)"
    switch ($b.Kind) {
        "stable" { cargo build --release --target $triple }
        "llvm" {
            Get-LlvmMingw
            $savedPath = $env:PATH
            $env:PATH = "$llvm;$env:PATH"
            $env:CARGO_TARGET_AARCH64_PC_WINDOWS_GNULLVM_LINKER = "aarch64-w64-mingw32-clang"
            $env:CARGO_TARGET_AARCH64_PC_WINDOWS_GNULLVM_RUSTFLAGS = "-C target-feature=+crt-static"
            $env:CC_aarch64_pc_windows_gnullvm = "aarch64-w64-mingw32-clang"
            $env:AR_aarch64_pc_windows_gnullvm = "llvm-ar"
            cargo build --release --target $triple
            $env:PATH = $savedPath
        }
        "win7" { cargo +nightly build --release -Z build-std=std,panic_abort --target $triple --no-default-features }
    }
    if ($LASTEXITCODE -ne 0) { throw "cargo build failed for $triple" }
    $exe = "target\$triple\release\stagecraft_3d.exe"
    $base = "Stagecraft3D-$version-$name"

    Copy-Item $exe (Join-Path $Dist "$base.exe") -Force

    $stage = Join-Path ([System.IO.Path]::GetTempPath()) "stagecraft-$name-portable"
    if (Test-Path $stage) { Remove-Item $stage -Recurse -Force }
    New-Item -ItemType Directory $stage | Out-Null
    Copy-Item $exe (Join-Path $stage "Stagecraft3D.exe")
    Copy-Item README.md, LICENSE $stage
    $zip = Join-Path $Dist "$base-portable.zip"
    if (Test-Path $zip) { Remove-Item $zip -Force }
    Compress-Archive -Path (Join-Path $stage "*") -DestinationPath $zip

    wix build packaging\windows\stagecraft.wxs -arch $b.WixArch -d "Version=$version" -d "ExePath=$exe" -o (Join-Path $Dist "$base.msi")
    if ($LASTEXITCODE -ne 0) { throw "wix build failed for $name" }
    Remove-Item (Join-Path $Dist "$base.wixpdb") -ErrorAction SilentlyContinue
}
Get-ChildItem $Dist -Filter "*$version*windows*" | Format-Table Name, Length
