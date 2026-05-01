$ErrorActionPreference = "Stop"

Write-Host "Checking Rust installation..." -ForegroundColor Cyan

if (-not (Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "Rust not found. Installing rustup..." -ForegroundColor Yellow

    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"

    Start-Process -FilePath "$env:TEMP\rustup-init.exe" -ArgumentList "-y" -Wait
} else {
    Write-Host "Rust already installed." -ForegroundColor Green
}

Write-Host "Ensuring toolchain components..." -ForegroundColor Cyan

rustup component add rustfmt clippy

Write-Host "Setup complete." -ForegroundColor Green