# logging utils
function Info {
    param (
        [string]$message
    )
    Write-Host "Info " -NoNewline -ForegroundColor Blue
    Write-Host $message
}
function Warning {
    param (
        [string]$message
    )
    Write-Host "Warning " -NoNewline -ForegroundColor Yellow
    Write-Host $message -NoNewline
}
function Err {
    param (
        [string]$message
    )
    Write-Host "Error " -NoNewline -ForegroundColor Red
    Write-Host $message
}
function Success {
    param (
        [string]$message
    )
    Write-Host "Success " -NoNewline -ForegroundColor Green
    Write-Host $message
}

# define target path
$baseDir = Join-Path -Path $env:USERPROFILE -ChildPath "ept"
$toolchainsDir = Join-Path -Path $baseDir -ChildPath "toolchain"
$tempDir = Join-Path -Path $baseDir -ChildPath "temp/_script_"
$zipPath = Join-Path -Path $tempDir -ChildPath "latest.zip"

# toolchain download address
$zipUrl = "https://registry.edgeless.top/api/ept/latest"

# check if toolchain dir exists
if (Test-Path -Path $toolchainsDir) {
    Warning("The directory '$toolchainsDir' already exists. Do you want to delete and recreate it? (y/n)")
    $confirmation = Read-Host
    if ($confirmation.ToLower() -eq 'y') {
        Remove-Item -Path $toolchainsDir -Recurse -Force
    }
    else {
        Err("Operation aborted by user.")
        exit 1
    }
}

# create workshops
Info("Creating ept directory at '$baseDir'...")
New-Item -Path $toolchainsDir -ItemType Directory -Force | Out-Null
New-Item -Path $tempDir -ItemType Directory -Force | Out-Null

# download and extract
try {
    Info("Downloading ept toolchain...")
    Invoke-WebRequest -Uri $zipUrl -OutFile $zipPath -ErrorAction Stop
    
    Info("Extracting ept toolchain...")
    Expand-Archive -Path $zipPath -DestinationPath $toolchainsDir
} catch {
    Err("Failed to download and extract ept toolchain")
    exit 2
}

# add toolchain to PATH
$currentPath = [Environment]::GetEnvironmentVariable("PATH", [EnvironmentVariableTarget]::User)
if ($currentPath -notlike "*$toolchainsDir*") {
    $currentPath += ";$toolchainsDir"
    [Environment]::SetEnvironmentVariable("PATH", $currentPath, [EnvironmentVariableTarget]::User)
    Info("'$toolchainsDir' has been added to the user PATH")
}

# add official mirror
$eptExecutablePath = Join-Path -Path $toolchainsDir -ChildPath "ept.exe"
if (Test-Path -Path $eptExecutablePath) {
   Info("Adding official mirror...")
    & $eptExecutablePath mirror add https://registry.edgeless.top
} else {
    Err("ept executable not found in the extracted files")
    exit 2
}

Success("ept has been installed to current user, restart the shell then try 'ept help'")