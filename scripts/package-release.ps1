# 单文件 release 打包：pnpm build -> 组装 embed/ -> cargo build -> 产出 tosu-lyrics.exe
#
# 用法：pwsh -File scripts/package-release.ps1 [-FinalDir <输出目录>]
# 不传 -FinalDir 时输出到仓库上一级的 release/ 目录（不含任何个人机器路径）。
[CmdletBinding()]
param(
    # 留空则输出到仓库上一级的 release/ 目录（不含任何个人机器路径）
    [string]$FinalDir
)
$ErrorActionPreference = "Stop"
$repo = Split-Path -Parent $PSScriptRoot
$tp = Join-Path $repo "tosu-proxy"
$dist = Join-Path $tp "target\dist-package"
$embed = Join-Path $repo "embed"
# 默认值必须在这里求值：param() 的默认值在绑定阶段计算，那时 $PSScriptRoot 还不可靠
if ([string]::IsNullOrWhiteSpace($FinalDir)) {
    $FinalDir = Join-Path (Split-Path -Parent $repo) "release"
}
$finalDir = $FinalDir

Write-Host "[1/5] 前端构建" -ForegroundColor Magenta
Push-Location $repo
pnpm exec vite build --outDir "tosu-proxy/target/dist-package" --emptyOutDir
Pop-Location

Write-Host "[2/5] 组装 embed/" -ForegroundColor Magenta
if (Test-Path $embed) { Remove-Item $embed -Recurse -Force }
New-Item -ItemType Directory -Force -Path $embed | Out-Null
Copy-Item (Join-Path $dist "index.html") $embed -Force
Copy-Item (Join-Path $dist "assets") $embed -Recurse -Force
if (Test-Path (Join-Path $dist "osu.svg")) { Copy-Item (Join-Path $dist "osu.svg") $embed -Force }
Copy-Item (Join-Path $repo "static\LRC.otf") $embed -Force
Copy-Item (Join-Path $repo "static\tLRC.otf") $embed -Force
if (Test-Path (Join-Path $tp "lib\ffprobe.exe")) { Copy-Item (Join-Path $tp "lib\ffprobe.exe") (Join-Path $embed "ffprobe") -Force }
Write-Host ("      embed/ 文件数 = " + (Get-ChildItem $embed -Recurse -File | Measure-Object).Count)

Write-Host "[3/5] 后端 release 构建" -ForegroundColor Magenta
cargo build -r --bin osu-lyric --features=new --manifest-path (Join-Path $tp "Cargo.toml")

Write-Host "[4/5] 打包 tosu-lyrics.exe" -ForegroundColor Magenta
New-Item -ItemType Directory -Force -Path $finalDir | Out-Null
Copy-Item (Join-Path $tp "target\release\osu-lyric.exe") (Join-Path $finalDir "tosu-lyrics.exe") -Force

Write-Host "[5/5] 完成" -ForegroundColor Magenta
Get-Item (Join-Path $finalDir "tosu-lyrics.exe") | Select-Object FullName, Length