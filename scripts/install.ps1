$ErrorActionPreference = "Stop"

$Repo = if ($env:BYI_INSTALL_REPO) { $env:BYI_INSTALL_REPO } else { "wbytts/byi" }
$InstallDir = if ($env:BYI_INSTALL_DIR) { $env:BYI_INSTALL_DIR } else { Join-Path $env:USERPROFILE ".local\bin" }
$BinaryName = "byi.exe"

function Get-ByiTarget {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture

    # 平台名必须和 GitHub Release workflow 产物后缀保持一致。
    switch ($arch) {
        "X64" { return "x86_64-pc-windows-msvc" }
        "Arm64" { return "aarch64-pc-windows-msvc" }
        default { throw "不支持的架构: $arch" }
    }
}

$target = Get-ByiTarget
$asset = "byi-$target.zip"
$url = "https://github.com/$Repo/releases/latest/download/$asset"
$tmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
$archive = Join-Path $tmpDir $asset

New-Item -ItemType Directory -Path $tmpDir | Out-Null
New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null

try {
    Write-Host "下载 $url"
    Invoke-WebRequest -Uri $url -OutFile $archive

    # 解压后只安装单个 CLI 二进制，不修改用户 PATH。
    Expand-Archive -Path $archive -DestinationPath $tmpDir -Force
    Copy-Item -Path (Join-Path $tmpDir $BinaryName) -Destination (Join-Path $InstallDir $BinaryName) -Force

    Write-Host "安装完成: $(Join-Path $InstallDir $BinaryName)"
    Write-Host "如果命令不可用，请确认 $InstallDir 已加入 PATH。"
}
finally {
    Remove-Item -Path $tmpDir -Recurse -Force -ErrorAction SilentlyContinue
}
