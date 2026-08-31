# ─────────────────────────────────────────────────────────────
# Deep IDE 图标修复脚本
# 问题：deepIDE.ico 透明信息可能只存在于 AND 掩码/或完全没有
#       （alpha 全 255 + 白底），打包进 exe 后 Windows 资源管理器
#       只认 alpha 通道，不透明区域会显示成纯色方块（黑色/白色）。
# 方案：按 XOR + AND 掩码解码出图像，再对"连通到边界的白色背景"
#       做洪泛填充抠成真透明（保留鲸鱼内部的白色元素），
#       重建为 256/128/64/48/32/16 多尺寸真 alpha ICO。
# 用法：pwsh tools/rebuild-icon.ps1 [ico路径] [-SourceIco 源图标路径]
# ─────────────────────────────────────────────────────────────
param(
  [string]$IcoPath = "src-tauri/icons/deepIDE.ico",
  [string]$SourceIco = ""
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

# 源文件与输出文件分开，防止脚本反复把自身输出当输入
if ([string]::IsNullOrEmpty($SourceIco)) { $SourceIco = $IcoPath }
$absIco = (Resolve-Path $IcoPath).Path
$absSrc = (Resolve-Path $SourceIco).Path
$b = [System.IO.File]::ReadAllBytes($absSrc)
if ($absSrc -eq $absIco) { Write-Host "WARN: source equals output" }
$count = [BitConverter]::ToUInt16($b, 4)
if ($count -lt 1) { throw "invalid ico: no entries" }

# 取第一个条目（假定为原始尺寸，本项目为 128x128）
$w = $b[6];    if ($w -eq 0) { $w = 256 }
$h = $b[7];    if ($h -eq 0) { $h = 256 }
$sz = [BitConverter]::ToUInt32($b, 6 + 8)
$off = [BitConverter]::ToUInt32($b, 6 + 12)
Write-Host "source entry: ${w}x${h}, size=$sz, offset=$off"

$blob = [byte[]]::new($sz)
[Array]::Copy($b, $off, $blob, 0, $sz)

$xorSize = $w * $h * 4
$xor = [byte[]]::new($xorSize)
[Array]::Copy($blob, 40, $xor, 0, $xorSize)
$rowStride = [int]([math]::Ceiling($w / 32) * 4)
$and = [byte[]]::new($rowStride * $h)
[Array]::Copy($blob, 40 + $xorSize, $and, 0, $and.Length)

# 1) 解码：XOR(自底向上 BGRA) + AND 掩码(bit=1 透明) → 位图
$src = New-Object System.Drawing.Bitmap($w, $h, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
for ($y = 0; $y -lt $h; $y++) {
  $srcRow = $h - 1 - $y
  for ($x = 0; $x -lt $w; $x++) {
    $p = $srcRow * $w * 4 + $x * 4
    $bl = $xor[$p]; $gr = $xor[$p + 1]; $rd = $xor[$p + 2]
    $idx = $y * $rowStride + [math]::Floor($x / 8)
    $maskBit = ($and[$idx] -shr (7 - ($x % 8))) -band 1
    $al = if ($maskBit -eq 1) { 0 } else { 255 }
    $src.SetPixel($x, $y, [System.Drawing.Color]::FromArgb($al, $rd, $gr, $bl))
  }
}

# 1.5) 白色背景抠透明：从图像四边做洪泛填充（连通到边界的近白色 → alpha=0），
#      这样鲸鱼内部的白色（眼睛等）与彩色文字都能保留
$alphaMap = [byte[]]::new($w * $h)
for ($i = 0; $i -lt $alphaMap.Length; $i++) { $alphaMap[$i] = 255 }
$queue = New-Object System.Collections.Queue
function Test-Bg([int]$x, [int]$y) {
  if ($x -lt 0 -or $y -lt 0 -or $x -ge $w -or $y -ge $h) { return $false }
  $c = $src.GetPixel($x, $y)
  return ($c.R -ge 235 -and $c.G -ge 235 -and $c.B -ge 235)
}
function Push-Bg([int]$x, [int]$y) {
  if ($x -lt 0 -or $y -lt 0 -or $x -ge $w -or $y -ge $h) { return }
  $id = $y * $w + $x
  if ($alphaMap[$id] -eq 255 -and (Test-Bg -x $x -y $y)) {
    $alphaMap[$id] = 0
    $queue.Enqueue([int]$id)
  }
}
for ($x = 0; $x -lt $w; $x++) { Push-Bg -x $x -y 0; Push-Bg -x $x -y ($h - 1) }
for ($y = 0; $y -lt $h; $y++) { Push-Bg -x 0 -y $y; Push-Bg -x ($w - 1) -y $y }
while ($queue.Count -gt 0) {
  $id = [int]$queue.Dequeue()
  $qx = $id % $w; $qy = [math]::Floor($id / $w)
  Push-Bg -x ($qx - 1) -y $qy
  Push-Bg -x ($qx + 1) -y $qy
  Push-Bg -x $qx -y ($qy - 1)
  Push-Bg -x $qx -y ($qy + 1)
}
for ($i = 0; $i -lt $alphaMap.Length; $i++) {
  if ($alphaMap[$i] -eq 0) {
    $x = $i % $w; $y = [math]::Floor($i / $w)
    $c = $src.GetPixel($x, $y)
    $src.SetPixel($x, $y, [System.Drawing.Color]::FromArgb(0, $c.R, $c.G, $c.B))
  }
}
$dbgZero = 0
for ($i = 0; $i -lt $alphaMap.Length; $i++) { if ($alphaMap[$i] -eq 0) { $dbgZero++ } }
Write-Host "white-keyed transparent pixels: $dbgZero / $($w * $h)"

# 2) 每种尺寸生成 BGRA(straight alpha) 的 ICONIMAGE
function New-IconImage([System.Drawing.Bitmap]$source, [int]$size) {
  $bmp = New-Object System.Drawing.Bitmap($size, $size, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::HighQuality
  $g.PixelOffsetMode = [System.Drawing.Drawing2D.PixelOffsetMode]::HighQuality
  $g.CompositingQuality = [System.Drawing.Drawing2D.CompositingQuality]::HighQuality
  $g.Clear([System.Drawing.Color]::Transparent)
  $g.DrawImage($source, 0, 0, $size, $size)
  $g.Dispose()

  $rect = New-Object System.Drawing.Rectangle(0, 0, $size, $size)
  $data = $bmp.LockBits($rect, [System.Drawing.Imaging.ImageLockMode]::ReadOnly, [System.Drawing.Imaging.PixelFormat]::Format32bppArgb)
  $pixels = [byte[]]::new($size * $size * 4)
  [System.Runtime.InteropServices.Marshal]::Copy($data.Scan0, $pixels, 0, $pixels.Length)
  $bmp.UnlockBits($data)
  $bmp.Dispose()

  # GDI+ 给出的是预乘 alpha，转为 straight alpha
  for ($i = 0; $i -lt $pixels.Length; $i += 4) {
    $a = $pixels[$i + 3]
    if ($a -eq 0) { $pixels[$i] = 0; $pixels[$i + 1] = 0; $pixels[$i + 2] = 0 }
    elseif ($a -lt 255) {
      $pixels[$i]     = [math]::Min(255, [math]::Round($pixels[$i]     * 255 / $a))
      $pixels[$i + 1] = [math]::Min(255, [math]::Round($pixels[$i + 1] * 255 / $a))
      $pixels[$i + 2] = [math]::Min(255, [math]::Round($pixels[$i + 2] * 255 / $a))
    }
  }

  # 转自底向上 DIB
  $xor = [byte[]]::new($pixels.Length)
  for ($y = 0; $y -lt $size; $y++) {
    [Array]::Copy($pixels, $y * $size * 4, $xor, ($size - 1 - $y) * $size * 4, $size * 4)
  }

  $bih = [byte[]]::new(40)
  [BitConverter]::GetBytes([int32]40).CopyTo($bih, 0)
  [BitConverter]::GetBytes([int32]$size).CopyTo($bih, 4)
  [BitConverter]::GetBytes([int32]($size * 2)).CopyTo($bih, 8)
  [BitConverter]::GetBytes([int16]1).CopyTo($bih, 12)
  [BitConverter]::GetBytes([int16]32).CopyTo($bih, 14)
  $andMask = [byte[]]::new([int]([math]::Ceiling($size / 32) * 4 * $size))  # 全 0：透明由 alpha 通道决定
  return ($bih + $xor + $andMask)
}

$sizes = @(256, 128, 64, 48, 32, 16)
$blobs = @()
foreach ($s in $sizes) {
  $blobOut = New-IconImage -source $src -size $s
  $blobs += ,$blobOut
  Write-Host "generated ${s}x${s}: $($blobOut.Length) bytes"
}
$src.Dispose()

# 3) 组装 ICO（全部 BMP 真 alpha 条目）
$ms = New-Object System.IO.MemoryStream
$bw = New-Object System.IO.BinaryWriter($ms)
$bw.Write([uint16]0); $bw.Write([uint16]1); $bw.Write([uint16]$sizes.Count)
$offset = 6 + 16 * $sizes.Count
for ($i = 0; $i -lt $sizes.Count; $i++) {
  $s = $sizes[$i]
  $dim = 0
  if ($s -lt 256) { $dim = $s }
  $bw.Write([byte]$dim)
  $bw.Write([byte]$dim)
  $bw.Write([byte]0); $bw.Write([byte]0)
  $bw.Write([uint16]1); $bw.Write([uint16]32)
  $bw.Write([uint32]$blobs[$i].Length); $bw.Write([uint32]$offset)
  $offset += $blobs[$i].Length
}
$bw.Flush()
foreach ($bOut in $blobs) { $ms.Write($bOut, 0, $bOut.Length) }
$newIco = $ms.ToArray()
$bw.Dispose(); $ms.Dispose()
[System.IO.File]::WriteAllBytes($absIco, $newIco)
Write-Host "written $absIco : $($newIco.Length) bytes, $($sizes.Count) entries"

# 4) 校验：alpha 通道有真实透明度
$chk = [System.IO.File]::ReadAllBytes($absIco)
$c2 = [BitConverter]::ToUInt16($chk, 4)
for ($i = 0; $i -lt $c2; $i++) {
  $o = 6 + $i * 16
  $sw = $chk[$o]; if ($sw -eq 0) { $sw = 256 }
  $sh = $chk[$o + 1]; if ($sh -eq 0) { $sh = 256 }
  $so = [BitConverter]::ToUInt32($chk, $o + 12)
  $ss = [BitConverter]::ToUInt32($chk, $o + 8)
  $bl = [byte[]]::new($ss)
  [Array]::Copy($chk, $so, $bl, 0, $ss)
  $alphaZero = 0; $total = 0
  for ($p = 40; $p -lt 40 + $sw * $sh * 4; $p += 4) {
    $total++
    if ($bl[$p + 3] -eq 0) { $alphaZero++ }
  }
  Write-Host ("entry ${i}: {0}x{1} alphaZero={2}/{3}" -f $sw, $sh, $alphaZero, $total)
}
