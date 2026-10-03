# 给已构建的 osu-lyric.exe 做 Authenticode 代码签名。
#
# 为什么需要这个脚本：Windows SmartScreen 的"发行者：发布者未知"只取决于 exe
# 有没有有效的代码签名证书。本项目当前**没有任何签名证书**，因此仅靠改源码
# 无法正规消除该提示（也不能靠伪造签名、关闭 Defender 或修改系统策略绕过）。
# 有了正式证书后，用这个脚本签名即可，不要把证书提交进仓库。
#
# 用法：
#   pwsh -File ./scripts/sign-windows.ps1 -ExePath ./dist/osu-lyric.exe `
#        -PfxPath C:\keys\codesign.pfx -PfxPassword '密码'
#
# 也支持环境变量（适合 CI，避免明文写在命令行里）：
#   OSU_LYRIC_PFX_PATH、OSU_LYRIC_PFX_PASSWORD、OSU_LYRIC_TIMESTAMP_URL
#
# 退出码：0 = 签名并验证通过；1 = 缺少证书/signtool 或签名失败（不会静默通过）。

[CmdletBinding()]
param(
    [string]$ExePath = "./dist/osu-lyric.exe",
    [string]$PfxPath = $env:OSU_LYRIC_PFX_PATH,
    [string]$PfxPassword = $env:OSU_LYRIC_PFX_PASSWORD,
    [string]$TimestampUrl = $env:OSU_LYRIC_TIMESTAMP_URL
)

$ErrorActionPreference = "Stop"

if ([string]::IsNullOrWhiteSpace($TimestampUrl)) {
    $TimestampUrl = "http://timestamp.digicert.com"
}

function Fail([string]$Message) {
    Write-Host "签名未完成：$Message" -ForegroundColor Yellow
    exit 1
}

# 1) 目标文件必须存在
if (-not (Test-Path -LiteralPath $ExePath)) {
    Fail "找不到待签名的文件：$ExePath（请先执行 just build）"
}
$exe = (Resolve-Path -LiteralPath $ExePath).Path

# 2) 证书必须由使用者提供 —— 脚本绝不生成自签名证书
if ([string]::IsNullOrWhiteSpace($PfxPath)) {
    Write-Host "未提供代码签名证书（-PfxPath 或 OSU_LYRIC_PFX_PATH）。" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "说明：SmartScreen 的『发行者：发布者未知』是因为 exe 未签名。" -ForegroundColor Yellow
    Write-Host "仅靠修改源码无法正规消除；需要一张受信任的代码签名证书：" -ForegroundColor Yellow
    Write-Host "  - EV 证书：通常能立即建立 SmartScreen 信誉；" -ForegroundColor Yellow
    Write-Host "  - OV 证书：需要一段时间的下载量累积后信誉才会建立。" -ForegroundColor Yellow
    Write-Host "本脚本不会生成假证书，也不会修改 Defender / SmartScreen 设置。" -ForegroundColor Yellow
    exit 1
}
if (-not (Test-Path -LiteralPath $PfxPath)) {
    Fail "找不到证书文件：$PfxPath"
}
$pfx = (Resolve-Path -LiteralPath $PfxPath).Path

# 3) 找到 signtool（Windows SDK 自带；不额外安装任何东西）
$signtoolCmd = Get-Command signtool.exe -ErrorAction SilentlyContinue
$signtool = $null
if ($signtoolCmd) { $signtool = $signtoolCmd.Source }
if (-not $signtool) {
    $kitsRoot = "${env:ProgramFiles(x86)}\Windows Kits\10\bin"
    if (Test-Path -LiteralPath $kitsRoot) {
        $candidates = Get-ChildItem $kitsRoot -Recurse -Filter "signtool.exe" -ErrorAction SilentlyContinue |
            Where-Object { $_.FullName -match '\\x64\\' } |
            Sort-Object FullName -Descending
        if ($candidates) { $signtool = ($candidates | Select-Object -First 1).FullName }
    }
}
if (-not $signtool) {
    Fail "找不到 signtool.exe。请安装 Windows SDK 的签名工具（Signing Tools for Desktop Apps）后重试。"
}
Write-Host "signtool: $signtool"
Write-Host "签名目标: $exe"

# 4) 签名（SHA-256 + RFC3161 时间戳；带时间戳后证书过期也不影响已发布文件的有效性）
$signArgs = @("sign", "/fd", "sha256", "/td", "sha256", "/tr", $TimestampUrl, "/f", $pfx)
if (-not [string]::IsNullOrEmpty($PfxPassword)) {
    $signArgs += @("/p", $PfxPassword)
}
$signArgs += $exe

& $signtool @signArgs
if ($LASTEXITCODE -ne 0) {
    Fail "signtool sign 失败（退出码 $LASTEXITCODE）"
}

# 5) 验证签名确实生效，再报告成功
& $signtool verify /pa /v $exe
if ($LASTEXITCODE -ne 0) {
    Fail "签名已写入但 verify 未通过（退出码 $LASTEXITCODE）"
}

$sig = Get-AuthenticodeSignature -LiteralPath $exe
Write-Host ""
Write-Host "签名完成：$($sig.Status) / 发布者：$($sig.SignerCertificate.Subject)" -ForegroundColor Green
exit 0
