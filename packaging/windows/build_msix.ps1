$ErrorActionPreference = "Stop"

$Root = (Resolve-Path "$PSScriptRoot\..\..").Path
$ReleaseDir = "C:\RustTarget\imageresizej\release"
$StageDir = "C:\RustTarget\imageresizej\stage"
$SparseDir = "$PSScriptRoot\sparse"
$AssetsDir = "$SparseDir\Assets"
$CertDir = "$PSScriptRoot\cert"
$PfxPath = "$CertDir\ImageResizerJ.pfx"
$CerPath = "$CertDir\ImageResizerJ.cer"
$CertPass = "ImageResizerJ-Dev-Sign"
$MsixPath = "$StageDir\ImageResizerJ.msix"

# 1. Find Windows SDK tools (makeappx.exe and signtool.exe)
$SdkBins = Get-ChildItem "C:\Program Files (x86)\Windows Kits\10\bin\*\x64\makeappx.exe" -ErrorAction SilentlyContinue |
    Sort-Object FullName -Descending
if (-not $SdkBins) {
    throw "makeappx.exe not found in Windows Kits. Install Windows 10/11 SDK in Visual Studio Installer."
}
$SdkDir = $SdkBins[0].DirectoryName
$MakeAppx = Join-Path $SdkDir "makeappx.exe"
$SignTool = Join-Path $SdkDir "signtool.exe"
Write-Host "Using Windows SDK: $SdkDir" -ForegroundColor Cyan

# 2. Generate required PNG icons for MSIX from assets\icon.ico
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

# 3. Create self-signed code signing certificate (once)
New-Item -ItemType Directory -Force $CertDir | Out-Null
if (-not (Test-Path $PfxPath)) {
    Write-Host "Creating self-signed certificate CN=ImageResizerJ..." -ForegroundColor Cyan
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
    Export-Certificate -Cert $cert -FilePath $CerPath | Out-Null
    Remove-Item "Cert:\CurrentUser\My\$($cert.Thumbprint)" -ErrorAction SilentlyContinue
}

# 4. Copy binaries and public cert into staging folder
New-Item -ItemType Directory -Force $StageDir | Out-Null
Copy-Item "$ReleaseDir\imageresizej.exe" $StageDir -Force
Copy-Item "$ReleaseDir\resizer-worker.exe" $StageDir -Force
Copy-Item "$ReleaseDir\resizer_win_shell.dll" $StageDir -Force
Copy-Item $CerPath $StageDir -Force

# 5. Pack and sign Sparse MSIX
Write-Host "Packing ImageResizerJ.msix..." -ForegroundColor Cyan
& $MakeAppx pack /o /d $SparseDir /nv /p $MsixPath
if ($LASTEXITCODE -ne 0) { throw "makeappx.exe failed" }

Write-Host "Signing ImageResizerJ.msix..." -ForegroundColor Cyan
& $SignTool sign /fd SHA256 /f $PfxPath /p $CertPass $MsixPath
if ($LASTEXITCODE -ne 0) { throw "signtool.exe failed" }

Write-Host "`nDone! All files are ready in: $StageDir" -ForegroundColor Green