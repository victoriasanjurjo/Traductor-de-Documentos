# Script de Empaquetado y Firma Digital MSIX - Traductor de Documentos - Inglés a Español (v1.0.0)
# Desarrolladora: Victoria Sanjurjo (CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B)
$ErrorActionPreference = "Stop"

$ProjectRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$ReleaseExe  = Join-Path $ProjectRoot "src-tauri\target\release\traductor-documentos-en-es.exe"
$MsixDir     = Join-Path $ProjectRoot "packaging\msix"
$LayoutDir   = Join-Path $MsixDir "layout"
$OutputsDir  = Join-Path $ProjectRoot "outputs"

if (-not (Test-Path $ReleaseExe)) {
    throw "No se encontró el binario compilado en $ReleaseExe. Ejecute primero cargo build --release."
}

if (Test-Path $LayoutDir) {
    Remove-Item -Recurse -Force $LayoutDir
}
New-Item -ItemType Directory -Force -Path $LayoutDir, (Join-Path $LayoutDir "Assets"), $OutputsDir | Out-Null

# 1. Copiar ejecutable, manifiesto y recursos visuales al layout MSIX
Copy-Item -Force $ReleaseExe (Join-Path $LayoutDir "traductor-documentos-en-es.exe")
Copy-Item -Force (Join-Path $MsixDir "AppxManifest.xml") (Join-Path $LayoutDir "AppxManifest.xml")
Copy-Item -Force (Join-Path $MsixDir "Assets\*.png") (Join-Path $LayoutDir "Assets\")

# 2. Localizar herramientas oficiales del Windows 10/11 SDK (makeappx.exe y signtool.exe)
$MakeAppx = Get-ChildItem -Path "C:\Program Files (x86)\Windows Kits\10\bin" -Recurse -Filter "makeappx.exe" -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match "\\x64\\" } |
    Sort-Object FullName -Descending |
    Select-Object -First 1 -ExpandProperty FullName

$SignTool = Get-ChildItem -Path "C:\Program Files (x86)\Windows Kits\10\bin" -Recurse -Filter "signtool.exe" -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match "\\x64\\" } |
    Sort-Object FullName -Descending |
    Select-Object -First 1 -ExpandProperty FullName

if (-not $MakeAppx -or -not $SignTool) {
    throw "No se encontraron makeappx.exe o signtool.exe en Windows Kits."
}

Write-Host "Usando MakeAppx: $MakeAppx"
Write-Host "Usando SignTool: $SignTool"

# 3. Empaquetar el directorio layout en un archivo .msix
$MsixFile = Join-Path $OutputsDir "TraductorDeDocumentos_InglesAEspanol_1.0.0.0_x64.msix"
if (Test-Path $MsixFile) { Remove-Item -Force $MsixFile }

& $MakeAppx pack /d $LayoutDir /p $MsixFile /nv /o
if ($LASTEXITCODE -ne 0) {
    throw "Error al ejecutar makeappx.exe pack (código $LASTEXITCODE)"
}

# 4. Generar o usar Certificado de Desarrollo para Victoria Sanjurjo (CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B)
$Subject = "CN=CF220D35-0C1C-408C-BCB2-E85AA65BC95B"
$Cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object { $_.Subject -eq $Subject } | Select-Object -First 1
if (-not $Cert) {
    Write-Host "Generando certificado autofirmado de firma de código ($Subject)..."
    $Cert = New-SelfSignedCertificate `
        -Type Custom `
        -Subject $Subject `
        -KeyUsage DigitalSignature `
        -FriendlyName "Victoria Sanjurjo - Traductor de Documentos MSIX Cert" `
        -CertStoreLocation "Cert:\CurrentUser\My" `
        -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")
}

$PfxPasswordPlain = "TraductorDocs2026!"
$PfxPassword = ConvertTo-SecureString -String $PfxPasswordPlain -Force -AsPlainText
$PfxPath = Join-Path $MsixDir "VictoriaSanjurjo_MSIX.pfx"
$CerPath = Join-Path $OutputsDir "VictoriaSanjurjo_MSIX.cer"

Export-PfxCertificate -Cert $Cert -FilePath $PfxPath -Password $PfxPassword | Out-Null
Export-Certificate -Cert $Cert -FilePath $CerPath | Out-Null

# 5. Firmar digitalmente el paquete MSIX con SHA256
& $SignTool sign /fd SHA256 /a /f $PfxPath /p $PfxPasswordPlain $MsixFile
if ($LASTEXITCODE -ne 0) {
    throw "Error al firmar el paquete MSIX con signtool.exe (código $LASTEXITCODE)"
}

# Copiar también una referencia directa en la raíz del proyecto para acceso inmediato
$RootMsixCopy = Join-Path $ProjectRoot "TraductorDeDocumentos_InglesAEspanol_1.0.0.0_x64.msix"
Copy-Item -Force $MsixFile $RootMsixCopy

Write-Host "================================================================="
Write-Host "PAQUETE MSIX CREADO Y FIRMADO EXITOSAMENTE:"
Write-Host " -> $MsixFile"
Write-Host " -> $RootMsixCopy"
Write-Host " -> Certificado Público: $CerPath"
Write-Host "================================================================="
