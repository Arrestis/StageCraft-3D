# Builds the Linux versions of Stagecraft 3D on Windows (no Linux machine needed) into dist\ ,
# for PCs (x86_64) and 64-bit ARM (aarch64: Raspberry Pi 4/5, ARM laptops and servers):
#   Stagecraft3D-<ver>-linux-<arch>.tar.gz and .AppImage   (any distribution)
#   stagecraft-3d_<ver>_<amd64|arm64>.deb                  (Debian, Ubuntu, Mint, Raspberry Pi OS)
#   stagecraft-3d-<ver>-1.<arch>.rpm                       (Fedora, openSUSE, RHEL)
#   stagecraft-3d-<ver>-1-<arch>.pkg.tar.zst               (Arch, Manjaro, EndeavourOS)
# The program is linked with Zig (downloaded once into .tools\) for glibc 2.28+ (Ubuntu 20.04+,
# Debian 10+, Fedora 29+ ...), then packaged by packaging\linux-packager.
# Needs: Rust targets x86_64-unknown-linux-gnu and aarch64-unknown-linux-gnu.
# Run from the project folder:  powershell -ExecutionPolicy Bypass -File packaging\build-linux-from-windows.ps1
param([string]$Dist = "dist")

$ErrorActionPreference = "Stop"
$root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Set-Location $root
$version = (Select-String -Path Cargo.toml -Pattern '^version\s*=\s*"([^"]+)"').Matches[0].Groups[1].Value
$tools = Join-Path $root ".tools"
New-Item -ItemType Directory -Force $tools | Out-Null
$ProgressPreference = "SilentlyContinue"

$zig = Join-Path $tools "zig-windows-x86_64-0.13.0\zig.exe"
if (-not (Test-Path $zig)) {
    Write-Host "Downloading Zig (linker for Linux)..."
    Invoke-WebRequest "https://ziglang.org/download/0.13.0/zig-windows-x86_64-0.13.0.zip" -OutFile "$tools\zig.zip" -UseBasicParsing
    Expand-Archive "$tools\zig.zip" -DestinationPath $tools
}

Write-Host "== Linux packaging tool"
$env:CARGO_TARGET_DIR = Join-Path $root "target\packager"
cargo build --release --manifest-path packaging\linux-packager\Cargo.toml
if ($LASTEXITCODE -ne 0) { throw "packager build failed" }
Remove-Item Env:CARGO_TARGET_DIR
$packager = Join-Path $root "target\packager\release\linux-packager.exe"
New-Item -ItemType Directory -Force $Dist | Out-Null

$env:ZIG = $zig
foreach ($arch in "x86_64", "aarch64") {
    $triple = "$arch-unknown-linux-gnu"
    Write-Host "== Linux $arch"
    $runtime = Join-Path $tools "runtime-$arch"
    if (-not (Test-Path $runtime)) {
        Write-Host "Downloading the AppImage runtime for $arch..."
        Invoke-WebRequest "https://github.com/AppImage/type2-runtime/releases/download/continuous/runtime-$arch" -OutFile $runtime -UseBasicParsing
    }
    $env:ZIG_TARGET = "$arch-linux-gnu.2.28"
    $linkerVar = "CARGO_TARGET_" + $triple.ToUpper().Replace("-", "_") + "_LINKER"
    Set-Item "Env:$linkerVar" (Join-Path $root "packaging\linux\zig-linker.cmd")
    cargo build --release --target $triple
    if ($LASTEXITCODE -ne 0) { throw "Linux $arch build failed" }
    & $packager --bin "target\$triple\release\stagecraft_3d" --project $root --version $version --arch $arch --out $Dist --runtime $runtime
    if ($LASTEXITCODE -ne 0) { throw "packaging failed for $arch" }
}
Get-ChildItem $Dist -Filter "*$version*" | Where-Object { $_.Name -match "linux|\.deb|\.rpm|pkg\.tar" } | Format-Table Name, Length
