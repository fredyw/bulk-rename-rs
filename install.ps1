$ErrorActionPreference = 'Stop'

$Repo = "fredyw/bmv"
$BinaryName = "bmv.exe"
$InstallDir = Join-Path $HOME ".bmv\bin"
$ArtifactName = "bmv-windows-x86_64.exe"

if (-not (Test-Path $InstallDir)) {
    New-Item -Path $InstallDir -ItemType Directory | Out-Null
}

Write-Host "Fetching latest release version for $Repo..." -ForegroundColor Cyan
$Release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
$Version = $Release.tag_name

if (-not $Version) {
    Write-Error "Could not find any releases for $Repo."
    exit 1
}

$DownloadUrl = "https://github.com/$Repo/releases/download/$Version/$ArtifactName"
$DestPath = Join-Path $InstallDir $BinaryName

Write-Host "Downloading $BinaryName $Version to $DestPath..." -ForegroundColor Cyan
Invoke-WebRequest -Uri $DownloadUrl -OutFile $DestPath

Write-Host "Successfully installed $BinaryName $Version!" -ForegroundColor Green

# Check if the directory is in the Path
$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    Write-Host "`nWarning: $InstallDir is not in your Path." -ForegroundColor Yellow
    Write-Host "Adding it now..." -ForegroundColor Cyan
    $NewPath = "$UserPath;$InstallDir"
    [System.Environment]::SetEnvironmentVariable("Path", $NewPath, "User")
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "Done! You may need to restart your terminal to use '$($BinaryName.Replace('.exe',''))'." -ForegroundColor Green
} else {
    Write-Host "`n$BinaryName is ready to use!" -ForegroundColor Green
}
