# Makes the app icons from assets\icon-source.png (the Stagecraft 3D artwork):
#   assets\icon.png (512 px), assets\icon-256.png, assets\icon-1024.png and assets\icon.ico (16-256 px).
# The artwork's rounded square is cut out with transparent corners.
# Run from the project folder:  powershell -ExecutionPolicy Bypass -File packaging\make-icon.ps1
param([string]$Source = (Join-Path $PSScriptRoot "..\assets\icon-source.png"),
      [string]$Out = (Join-Path $PSScriptRoot "..\assets"))

Add-Type -AssemblyName System.Drawing
$src = [System.Drawing.Bitmap]::FromFile((Resolve-Path $Source).Path)
$W = $src.Width; $H = $src.Height

# Find the rounded square: scan the middle row/column for the first non-black pixel.
function Test-Ink($c) { ($c.R + $c.G + $c.B) -gt 90 }
$midY = [int]($H / 2); $midX = [int]($W / 2)
$left = 0; while ($left -lt $W -and -not (Test-Ink $src.GetPixel($left, $midY))) { $left++ }
$right = $W - 1; while ($right -gt 0 -and -not (Test-Ink $src.GetPixel($right, $midY))) { $right-- }
$top = 0; while ($top -lt $H -and -not (Test-Ink $src.GetPixel($midX, $top))) { $top++ }
$bottom = $H - 1; while ($bottom -gt 0 -and -not (Test-Ink $src.GetPixel($midX, $bottom))) { $bottom-- }
$side = [Math]::Max($right - $left, $bottom - $top)
Write-Host "Artwork square: x $left-$right, y $top-$bottom"

function New-Icon([int]$S) {
    $bmp = New-Object System.Drawing.Bitmap $S, $S, ([System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.SmoothingMode = 'AntiAlias'
    $g.InterpolationMode = 'HighQualityBicubic'
    $g.PixelOffsetMode = 'HighQuality'
    $g.Clear([System.Drawing.Color]::Transparent)
    $pad = [Math]::Max(0.0, $S * 0.02)
    $inner = $S - 2 * $pad
    $scale = $inner / $side
    # Brush that paints the artwork so its square lands on [pad, pad+inner].
    $brush = New-Object System.Drawing.TextureBrush $src
    $brush.WrapMode = 'Clamp'
    $m = New-Object System.Drawing.Drawing2D.Matrix
    $m.Translate([float]$pad, [float]$pad)
    $m.Scale([float]$scale, [float]$scale)
    $m.Translate([float](-$left), [float](-$top))
    $brush.Transform = $m
    $r = [float]($inner * 0.19)
    $x0 = [float]$pad; $y0 = [float]$pad; $x1 = [float]($pad + $inner); $y1 = [float]($pad + $inner)
    $path = New-Object System.Drawing.Drawing2D.GraphicsPath
    $path.AddArc($x0, $y0, 2 * $r, 2 * $r, 180, 90)
    $path.AddArc($x1 - 2 * $r, $y0, 2 * $r, 2 * $r, 270, 90)
    $path.AddArc($x1 - 2 * $r, $y1 - 2 * $r, 2 * $r, 2 * $r, 0, 90)
    $path.AddArc($x0, $y1 - 2 * $r, 2 * $r, 2 * $r, 90, 90)
    $path.CloseFigure()
    $g.FillPath($brush, $path)
    $g.Dispose()
    return $bmp
}

$png = [System.Drawing.Imaging.ImageFormat]::Png
(New-Icon 1024).Save((Join-Path $Out "icon-1024.png"), $png)
(New-Icon 512).Save((Join-Path $Out "icon.png"), $png)
(New-Icon 256).Save((Join-Path $Out "icon-256.png"), $png)

# .ico with PNG-compressed images (Windows Vista and later).
$sizes = 16, 20, 24, 32, 40, 48, 64, 128, 256
$pngs = foreach ($s in $sizes) {
    $ms = New-Object System.IO.MemoryStream
    (New-Icon $s).Save($ms, $png)
    , $ms.ToArray()
}
$fs = [System.IO.File]::Create((Join-Path $Out "icon.ico"))
$bw = New-Object System.IO.BinaryWriter $fs
$bw.Write([UInt16]0); $bw.Write([UInt16]1); $bw.Write([UInt16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
    $s = $sizes[$i]
    $d = if ($s -ge 256) { 0 } else { $s }
    $bw.Write([byte]$d); $bw.Write([byte]$d); $bw.Write([byte]0); $bw.Write([byte]0)
    $bw.Write([UInt16]1); $bw.Write([UInt16]32)
    $bw.Write([UInt32]$pngs[$i].Length); $bw.Write([UInt32]$offset)
    $offset += $pngs[$i].Length
}
foreach ($p in $pngs) { $bw.Write($p) }
$bw.Close()
$src.Dispose()
Write-Host "Icons written to $Out"
