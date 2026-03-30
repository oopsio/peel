# Peel Installation Script (Windows)
# Created for Peel Programming Language

$ErrorActionPreference = "Stop"

# --- Configurations ---
$BaseUrl = "https://raw.githubusercontent.com/oopsio/peel/master/built"
$InstallDir = "$HOME\.peel\bin"
$ProjectLogo = "Peel"

# --- UI Helpers ---
function Show-Header {
    Write-Host ""
    Write-Host "  P E E L " -ForegroundColor Yellow -BackgroundColor Black
    Write-Host "  The Peel Programming Language Installer" -ForegroundColor Cyan
    Write-Host "  ---------------------------------------"
}

function Show-Progress {
    param([string]$Activity, [int]$Percent)
    Write-Progress -Activity "Installing Peel" -Status $Activity -PercentComplete $Percent
}

function Write-Step {
    param([string]$Msg)
    Write-Host "  [+] $Msg" -ForegroundColor Gray
}

function Write-Success {
    param([string]$Msg)
    Write-Host "`n  [SUCCESS] $Msg" -ForegroundColor Green
}

function Write-Error {
    param([string]$Msg)
    Write-Host "`n  [ERROR] $Msg" -ForegroundColor Red
}

# --- Detection ---
function Get-BinaryName {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($arch -eq "AMD64") {
        return "peel-win32-x64.exe"
    } elseif ($arch -eq "x86") {
        return "peel-win32-ia32.exe"
    } else {
        throw "Unsupported architecture: $arch"
    }
}

# --- Main Logic ---
try {
    Show-Header
    
    # 1. Determine Binary
    $BinaryName = Get-BinaryName
    $DownloadUrl = "$BaseUrl/$BinaryName"
    Write-Step "Detected Architecture: $env:PROCESSOR_ARCHITECTURE"

    # 2. Setup Directory
    if (!(Test-Path $InstallDir)) {
        Write-Step "Creating installation directory: $InstallDir"
        New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
    }

    # 3. Download
    Write-Step "Downloading $BinaryName..."
    $DestPath = Join-Path $InstallDir "peel.exe"
    
    $WebClient = New-Object System.Net.WebClient
    $WebClient.DownloadFile($DownloadUrl, $DestPath)
    
    # 4. Update Path (User Scope)
    Write-Step "Updating environment variables..."
    $CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if ($CurrentPath -notlike "*$InstallDir*") {
        $NewPath = "$CurrentPath;$InstallDir"
        [Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
        $env:Path = "$env:Path;$InstallDir"
    }

    # 5. Finalize
    Write-Success "Peel has been successfully installed!"
    Write-Host "  Location: $DestPath"
    Write-Host "  Version: $(& $DestPath --version)"
    Write-Host "`n  Please RESTART your terminal to start using 'peel'." -ForegroundColor Yellow
} catch {
    Write-Error "Failed to install Peel: $_"
}
