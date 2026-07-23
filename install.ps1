# pitu PowerShell Installer for Windows
# Usage: iex (irm https://raw.githubusercontent.com/sah-rohit/pitu/main/install.ps1)

$ErrorActionPreference = "Stop"

Write-Host "📷 Building/Installing pitu CLI Image Workbench on Windows..." -ForegroundColor Cyan

$InstallDir = "$env:USERPROFILE\.local\bin"
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$ExeTarget = Join-Path $InstallDir "pitu.exe"

# If cargo is installed, build release binary
if (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "⚡ Compiling release binary with Cargo..." -ForegroundColor Yellow
    cargo build --release
    Copy-Item ".\target\release\pitu.exe" -Destination $ExeTarget -Force
} else {
    Write-Error "Cargo is required to build from source. Please install Rust from https://rustup.rs"
    exit 1
}

# Ensure InstallDir is in User PATH
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "⚙️ Adding $InstallDir to User PATH Environment Variable..." -ForegroundColor Yellow
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
    $env:PATH = "$env:PATH;$InstallDir"
}

Write-Host "✨ Installation Complete! Restart your terminal and run 'pitu'." -ForegroundColor Green
