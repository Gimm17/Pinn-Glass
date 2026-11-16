# PinnGlass Windows Installer
# Installs PinnGlass to %LocalAppData%\Programs\PinnGlass without needing Admin rights

$ErrorActionPreference = "Stop"

$sourceExe = "$PSScriptRoot\target\release\pinnglass.exe"
$sourceIcon = "$PSScriptRoot\app.ico"

if (-not (Test-Path $sourceExe)) {
    Write-Error "Binary not found at $sourceExe. Please run 'cargo build --release' first."
}

$installDir = "$env:LOCALAPPDATA\Programs\PinnGlass"
$targetExe = "$installDir\pinnglass.exe"
$targetIcon = "$installDir\app.ico"

Write-Host "==========================================" -ForegroundColor Cyan
Write-Host "    Installing PinnGlass Image Viewer     " -ForegroundColor Cyan
Write-Host "==========================================" -ForegroundColor Cyan

# 1. Create target directory
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# 2. Copy binaries and assets
Write-Host "[1/6] Copying files to $installDir..." -ForegroundColor Yellow
Copy-Item -Path $sourceExe -Destination $targetExe -Force
if (Test-Path $sourceIcon) {
    Copy-Item -Path $sourceIcon -Destination $targetIcon -Force
}

# 3. Create Windows Shortcuts (Start Menu & Desktop)
Write-Host "[2/6] Creating Start Menu & Desktop shortcuts..." -ForegroundColor Yellow
$wsh = New-Object -ComObject WScript.Shell

# Start Menu Shortcut
$startMenuDir = "$env:APPDATA\Microsoft\Windows\Start Menu\Programs"
$startMenuShortcut = "$startMenuDir\PinnGlass.lnk"
$shortcut = $wsh.CreateShortcut($startMenuShortcut)
$shortcut.TargetPath = $targetExe
$shortcut.WorkingDirectory = $installDir
$shortcut.Description = "PinnGlass - Ultra-Fast Image Viewer"
$shortcut.IconLocation = "$targetExe,0"
$shortcut.Save()

# Desktop Shortcut
$desktopDir = [System.Environment]::GetFolderPath('Desktop')
$desktopShortcut = "$desktopDir\PinnGlass.lnk"
$shortcut = $wsh.CreateShortcut($desktopShortcut)
$shortcut.TargetPath = $targetExe
$shortcut.WorkingDirectory = $installDir
$shortcut.Description = "PinnGlass - Ultra-Fast Image Viewer"
$shortcut.IconLocation = "$targetExe,0"
$shortcut.Save()

# 4. Register in Windows Registry (User-level HKCU, no Admin needed)
Write-Host "[3/6] Registering in Windows Registry..." -ForegroundColor Yellow

# App Paths
$appPathsKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\App Paths\pinnglass.exe"
New-Item -Path $appPathsKey -Force | Out-Null
Set-ItemProperty -Path $appPathsKey -Name "(Default)" -Value $targetExe
Set-ItemProperty -Path $appPathsKey -Name "Path" -Value $installDir

# Applications registration
$appKey = "HKCU:\Software\Classes\Applications\pinnglass.exe"
$appOpenKey = "$appKey\shell\open\command"
New-Item -Path $appOpenKey -Force | Out-Null
Set-ItemProperty -Path $appKey -Name "(Default)" -Value "PinnGlass"
Set-ItemProperty -Path $appKey -Name "FriendlyAppName" -Value "PinnGlass"
Set-ItemProperty -Path $appKey -Name "ApplicationCompany" -Value "PinnGlass"
Set-ItemProperty -Path $appOpenKey -Name "(Default)" -Value "`"$targetExe`" `"%1`""

$iconKey = "$appKey\DefaultIcon"
if (-not (Test-Path $iconKey)) { New-Item -Path $iconKey -Force | Out-Null }
Set-ItemProperty -Path $iconKey -Name "(Default)" -Value "`"$targetIcon`",0"

# Fix Windows Shell MuiCache (Friendly Name in 'Open With')
$muiCacheKey = "HKCU:\Software\Classes\Local Settings\Software\Microsoft\Windows\Shell\MuiCache"
Set-ItemProperty -Path $muiCacheKey -Name "$targetExe.FriendlyAppName" -Value "PinnGlass" -Force
Set-ItemProperty -Path $muiCacheKey -Name "$targetExe.ApplicationCompany" -Value "PinnGlass" -Force

$supportedTypesKey = "$appKey\SupportedTypes"
New-Item -Path $supportedTypesKey -Force | Out-Null
$extensions = @(".jpg", ".jpeg", ".jpe", ".jfif", ".png", ".webp", ".gif", ".bmp", ".dib", ".ico", ".tiff", ".tif", ".hdr", ".svg", ".svgz")
foreach ($ext in $extensions) {
    Set-ItemProperty -Path $supportedTypesKey -Name $ext -Value ""
}

# 5. Add "Open with PinnGlass" Context Menu for all images in Windows Explorer
Write-Host "[4/6] Adding 'Open with PinnGlass' to Windows Explorer context menu..." -ForegroundColor Yellow
$imageAssocKey = "HKCU:\Software\Classes\SystemFileAssociations\image\shell\PinnGlass"
$imageAssocCmd = "$imageAssocKey\command"
New-Item -Path $imageAssocCmd -Force | Out-Null
Set-ItemProperty -Path $imageAssocKey -Name "(Default)" -Value "Open with PinnGlass"
Set-ItemProperty -Path $imageAssocKey -Name "Icon" -Value "`"$targetExe`",0"
Set-ItemProperty -Path $imageAssocCmd -Name "(Default)" -Value "`"$targetExe`" `"%1`""

# ProgID for Default Apps
$progIdKey = "HKCU:\Software\Classes\PinnGlass.Image"
$progIdOpen = "$progIdKey\shell\open\command"
$progIdIcon = "$progIdKey\DefaultIcon"
New-Item -Path $progIdOpen -Force | Out-Null
New-Item -Path $progIdIcon -Force | Out-Null
Set-ItemProperty -Path $progIdKey -Name "(Default)" -Value "PinnGlass Image File"
Set-ItemProperty -Path $progIdIcon -Name "(Default)" -Value "`"$targetExe`",0"
Set-ItemProperty -Path $progIdOpen -Name "(Default)" -Value "`"$targetExe`" `"%1`""

# Capabilities for Windows 10/11 "Default Apps" Settings
$capKey = "HKCU:\Software\PinnGlass\Capabilities"
$capAssoc = "$capKey\FileAssociations"
New-Item -Path $capAssoc -Force | Out-Null
Set-ItemProperty -Path $capKey -Name "ApplicationDescription" -Value "Ultra-fast, lightweight GPU-accelerated image viewer in Rust"
Set-ItemProperty -Path $capKey -Name "ApplicationName" -Value "PinnGlass"
foreach ($ext in $extensions) {
    Set-ItemProperty -Path $capAssoc -Name $ext -Value "PinnGlass.Image"
}

$regAppsKey = "HKCU:\Software\RegisteredApplications"
if (-not (Test-Path $regAppsKey)) {
    New-Item -Path $regAppsKey -Force | Out-Null
}
Set-ItemProperty -Path $regAppsKey -Name "PinnGlass" -Value "Software\PinnGlass\Capabilities"

# 6. Create Windows Settings "Installed Apps" Entry (Add/Remove Programs)
Write-Host "[5/6] Registering in Windows Installed Apps..." -ForegroundColor Yellow
$uninstallKey = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\PinnGlass"
New-Item -Path $uninstallKey -Force | Out-Null
Set-ItemProperty -Path $uninstallKey -Name "DisplayName" -Value "PinnGlass Image Viewer"
Set-ItemProperty -Path $uninstallKey -Name "DisplayVersion" -Value "1.0.0"
Set-ItemProperty -Path $uninstallKey -Name "Publisher" -Value "PinnGlass"
Set-ItemProperty -Path $uninstallKey -Name "DisplayIcon" -Value "`"$targetExe`",0"
Set-ItemProperty -Path $uninstallKey -Name "InstallLocation" -Value $installDir
Set-ItemProperty -Path $uninstallKey -Name "UninstallString" -Value "powershell.exe -ExecutionPolicy Bypass -File `"$installDir\uninstall.ps1`""

# 7. Create Uninstaller script in target directory
Write-Host "[6/6] Generating clean uninstaller..." -ForegroundColor Yellow
$uninstallerScript = @'
# PinnGlass Uninstaller
Write-Host "Uninstalling PinnGlass..." -ForegroundColor Yellow

# Remove Shortcuts
Remove-Item "$env:APPDATA\Microsoft\Windows\Start Menu\Programs\PinnGlass.lnk" -ErrorAction SilentlyContinue
Remove-Item "$([System.Environment]::GetFolderPath('Desktop'))\PinnGlass.lnk" -ErrorAction SilentlyContinue

# Remove Registry Entries
Remove-Item -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\App Paths\pinnglass.exe" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\Classes\Applications\pinnglass.exe" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\Classes\SystemFileAssociations\image\shell\PinnGlass" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\Classes\PinnGlass.Image" -Recurse -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\PinnGlass" -Recurse -ErrorAction SilentlyContinue
Remove-ItemProperty -Path "HKCU:\Software\RegisteredApplications" -Name "PinnGlass" -ErrorAction SilentlyContinue
Remove-Item -Path "HKCU:\Software\Microsoft\Windows\CurrentVersion\Uninstall\PinnGlass" -Recurse -ErrorAction SilentlyContinue

Write-Host "Registry entries and shortcuts removed." -ForegroundColor Green
Write-Host "You can now safely delete the directory: $PSScriptRoot"
'@
Set-Content -Path "$installDir\uninstall.ps1" -Value $uninstallerScript -Force

Write-Host ""
Write-Host "==========================================" -ForegroundColor Green
Write-Host "   PinnGlass Successfully Installed!      " -ForegroundColor Green
Write-Host "==========================================" -ForegroundColor Green
Write-Host "Location: $targetExe"
Write-Host "Start Menu: Shortcut created (type 'PinnGlass' in Start)"
Write-Host "Desktop: Shortcut created"
Write-Host "Context Menu: 'Open with PinnGlass' active on image files"
Write-Host ""
