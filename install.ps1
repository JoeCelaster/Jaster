#Requires -Version 5.1
$ErrorActionPreference = 'Stop'

# PowerShell 5.1 still defaults to TLS 1.0 on some builds, which GitHub refuses.
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$tmp = Join-Path $env:TEMP ("jaster-" + [guid]::NewGuid())
New-Item -ItemType Directory -Force -Path $tmp | Out-Null

Write-Host "📦 Downloading Jaster..."

# -UseBasicParsing keeps this working where Internet Explorer is absent.
Invoke-WebRequest -UseBasicParsing `
    -Uri 'https://github.com/JoeCelaster/Jaster/releases/latest/download/jaster-windows-x86_64.zip' `
    -OutFile (Join-Path $tmp 'jaster.zip')

Expand-Archive -Path (Join-Path $tmp 'jaster.zip') -DestinationPath $tmp -Force

# Run the inner installer as script text rather than invoking the file, so a
# restrictive execution policy does not block it.
$payload   = Join-Path $tmp 'jaster'
$installer = Join-Path $payload 'install.ps1'
& ([scriptblock]::Create((Get-Content -Raw $installer))) -Root $payload

Remove-Item $tmp -Recurse -Force -ErrorAction SilentlyContinue
