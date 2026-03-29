# Check if bun is in the PATH
if (Get-Command bun -ErrorAction SilentlyContinue) {
    bun scripts/build.js
} else {
    Write-Host "Error: Bun is not installed or not in PATH." -ForegroundColor Red
}