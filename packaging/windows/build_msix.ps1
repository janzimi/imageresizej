$ErrorActionPreference = "Stop"

$Root = (Resolve-Path "$PSScriptRoot\..\..").Path
$CargoMeta = cargo metadata --manifest-path "$Root\Cargo.toml" --no-deps --format-version 1 | ConvertFrom-Json
$TargetDir = $CargoMeta.target_directory
$ReleaseDir = Join-Path $TargetDir "release"
$StageDir = Join-Path $TargetDir "stage"

$SparseDir = "$PSScriptRoot\sparse"
$AssetsDir = "$SparseDir\Assets"
$PfxPath = if ($env:IMAGERESIZEJ_PFX) { $env:IMAGERESIZEJ_PFX } else { "C:\Keys\ImageResizerJ.pfx" }
$MsixPath = "$StageDir\ImageResizerJ.msix"

$SdkBins = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\makeappx.exe" -ErrorAction SilentlyContinue |
    Sort-Object FullName -Descending
if (-not $SdkBins) {
    throw "makeappx.exe not found in Windows Kits. Install Windows 10/11 SDK in Visual Studio Installer."
}
$SdkDir = $SdkBins[0].DirectoryName
$MakeAppx = Join-Path $SdkDir "makeappx.exe"
$SignTool = Join-Path $SdkDir "signtool.exe"
Write-Host "Using Windows SDK: $SdkDir" -ForegroundColor Cyan

$CertPass = $env:IMAGERESIZEJ_CERT_PASS
if (-not $CertPass) {
    $sec = Read-Host "Enter PFX certificate password" -AsSecureString
    $CertPass = (New-Object System.Net.NetworkCredential("", $sec)).Password
}

Add-Type -AssemblyName System.Drawing
$IconPath = Join-Path $Root "assets\icon.ico"
if (-not (Test-Path $IconPath)) {
    throw "Icon not found: $IconPath"
}
New-Item -ItemType Directory -Force $AssetsDir | Out-Null

function Save-IconPng([string]$SrcIco, [string]$OutPng, [int]$Size) {
    $ico = New-Object System.Drawing.Icon($SrcIco, $Size, $Size)
    $srcBmp = $ico.ToBitmap()
    $dstBmp = New-Object System.Drawing.Bitmap($Size, $Size)
    $g = [System.Drawing.Graphics]::FromImage($dstBmp)
    $g.InterpolationMode = [System.Drawing.Drawing2D.InterpolationMode]::HighQualityBicubic
    $g.Clear([System.Drawing.Color]::Transparent)
    $g.DrawImage($srcBmp, 0, 0, $Size, $Size)
    $dstBmp.Save($OutPng, [System.Drawing.Imaging.ImageFormat]::Png)
    $g.Dispose(); $dstBmp.Dispose(); $srcBmp.Dispose(); $ico.Dispose()
}

Save-IconPng $IconPath "$AssetsDir\StoreLogo.png" 50
Save-IconPng $IconPath "$AssetsDir\Square44x44Logo.png" 44
Save-IconPng $IconPath "$AssetsDir\Square150x150Logo.png" 150

$PfxDir = Split-Path -Parent $PfxPath
New-Item -ItemType Directory -Force $PfxDir | Out-Null
if (-not (Test-Path $PfxPath)) {
    Write-Host "Creating self-signed certificate CN=ImageResizerJ at $PfxPath..." -ForegroundColor Cyan
    $cert = New-SelfSignedCertificate `
        -Type Custom `
        -Subject "CN=ImageResizerJ" `
        -KeyUsage DigitalSignature `
        -FriendlyName "ImageResizerJ Code Signing" `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -NotAfter (Get-Date).AddYears(10) `
        -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")

    $secPass = ConvertTo-SecureString -String $CertPass -Force -AsPlainText
    Export-PfxCertificate -Cert $cert -FilePath $PfxPath -Password $secPass | Out-Null
    Remove-Item "Cert:\CurrentUser\My\$($cert.Thumbprint)" -ErrorAction SilentlyContinue
}

New-Item -ItemType Directory -Force $StageDir | Out-Null
Copy-Item "$ReleaseDir\imageresizej.exe" $StageDir -Force
Copy-Item "$ReleaseDir\resizer-worker.exe" $StageDir -Force
Copy-Item "$ReleaseDir\resizer_win_shell.dll" $StageDir -Force

$pfxCert = New-Object System.Security.Cryptography.X509Certificates.X509Certificate2($PfxPath, $CertPass)
[System.IO.File]::WriteAllBytes(
    "$StageDir\ImageResizerJ.cer",
    $pfxCert.Export([System.Security.Cryptography.X509Certificates.X509ContentType]::Cert)
)
$pfxCert.Dispose()

Write-Host "Packing ImageResizerJ.msix..." -ForegroundColor Cyan
& $MakeAppx pack /o /d $SparseDir /nv /p $MsixPath
if ($LASTEXITCODE -ne 0) { throw "makeappx.exe failed" }

Write-Host "Signing ImageResizerJ.msix..." -ForegroundColor Cyan
& $SignTool sign /fd SHA256 /f $PfxPath /p $CertPass $MsixPath
if ($LASTEXITCODE -ne 0) { throw "signtool.exe failed" }

Write-Host "`nDone! All files are ready in: $StageDir" -ForegroundColor Green