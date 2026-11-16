# Set PinnGlass as Default Image Viewer in Windows
$exts = @(".jpg", ".jpeg", ".jpe", ".jfif", ".png", ".webp", ".gif", ".bmp", ".dib", ".ico", ".tiff", ".tif", ".hdr", ".svg", ".svgz")

foreach ($ext in $exts) {
    # 1. Classes association
    $classKey = "HKCU:\Software\Classes\$ext"
    if (-not (Test-Path $classKey)) { New-Item -Path $classKey -Force | Out-Null }
    Set-ItemProperty -Path $classKey -Name "(Default)" -Value "PinnGlass.Image" -Force
    
    $classOpenWith = "$classKey\OpenWithProgids"
    if (-not (Test-Path $classOpenWith)) { New-Item -Path $classOpenWith -Force | Out-Null }
    Set-ItemProperty -Path $classOpenWith -Name "PinnGlass.Image" -Value "" -Force

    # 2. Explorer FileExts association
    $fileExtsProgids = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\$ext\OpenWithProgids"
    if (-not (Test-Path $fileExtsProgids)) { New-Item -Path $fileExtsProgids -Force | Out-Null }
    Set-ItemProperty -Path $fileExtsProgids -Name "PinnGlass.Image" -Value ([byte[]]@()) -Force
    
    $fileExtsList = "HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\$ext\OpenWithList"
    if (-not (Test-Path $fileExtsList)) { New-Item -Path $fileExtsList -Force | Out-Null }
    Set-ItemProperty -Path $fileExtsList -Name "a" -Value "pinnglass.exe" -Force
    Set-ItemProperty -Path $fileExtsList -Name "MRUList" -Value "a" -Force
}

$code = @'
using System;
using System.Runtime.InteropServices;
public class ShellNotify {
    [DllImport("shell32.dll")]
    public static extern void SHChangeNotify(int wEventId, int uFlags, IntPtr dwItem1, IntPtr dwItem2);
}
'@
Add-Type -TypeDefinition $code -ErrorAction SilentlyContinue
[ShellNotify]::SHChangeNotify(0x08000000, 0x0000, [IntPtr]::Zero, [IntPtr]::Zero)

Write-Host "PinnGlass has been registered as the primary handler for all image formats!" -ForegroundColor Green
Write-Host "Opening Windows Default Apps settings..." -ForegroundColor Cyan
Start-Process "ms-settings:defaultapps"
