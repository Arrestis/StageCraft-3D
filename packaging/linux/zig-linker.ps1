# Linker shim: lets Rust on Windows link the Linux (x86_64, glibc 2.28+) build with `zig cc`.
# Used by packaging\build-linux-from-windows.ps1 (through zig-linker.cmd). ZIG must point at zig.exe.
$zig = $env:ZIG
if (-not $zig) { Write-Error "ZIG is not set"; exit 1 }

function Expand-Args([string[]]$list) {
    foreach ($a in $list) {
        if ($a.StartsWith("@") -and (Test-Path -LiteralPath $a.Substring(1))) {
            # GNU-style response file: one argument per line, backslash escapes.
            foreach ($line in [System.IO.File]::ReadAllLines($a.Substring(1))) {
                if ($line.Length -eq 0) { continue }
                $sb = New-Object System.Text.StringBuilder
                for ($i = 0; $i -lt $line.Length; $i++) {
                    $c = $line[$i]
                    if ($c -eq '\' -and $i + 1 -lt $line.Length) { $i++; [void]$sb.Append($line[$i]) }
                    elseif ($c -eq '"') { }
                    else { [void]$sb.Append($c) }
                }
                $sb.ToString()
            }
        } else { $a }
    }
}

$out = New-Object System.Collections.Generic.List[string]
foreach ($a in (Expand-Args $args)) {
    if ($a -eq "-lgcc_s") { $out.Add("-lunwind"); continue }
    if ($a.StartsWith("-B") -or $a.StartsWith("-fuse-ld=") -or $a.StartsWith("--target=") -or $a -eq "-no-pie") { continue }
    if ($a -eq "-Wl,--fix-cortex-a53-843419") { continue }
    if ($a -eq "-Wl,--no-undefined-version" -or $a.StartsWith("-Wl,-z,") -and $a.Contains("nostart-stop-gc")) { continue }
    $out.Add($a)
}

# Pass everything through a response file (paths contain spaces; command lines get long).
$rsp = [System.IO.Path]::GetTempFileName()
$lines = foreach ($a in $out) { '"' + ($a -replace '\\', '/') + '"' }
[System.IO.File]::WriteAllLines($rsp, $lines)
# ZIG_TARGET picks the platform: x86_64-linux-gnu.2.28 (default) or aarch64-linux-gnu.2.28.
$target = if ($env:ZIG_TARGET) { $env:ZIG_TARGET } else { "x86_64-linux-gnu.2.28" }
& $zig cc -target $target "@$rsp"
$code = $LASTEXITCODE
Remove-Item -LiteralPath $rsp -ErrorAction SilentlyContinue
exit $code
