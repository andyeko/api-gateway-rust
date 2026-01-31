param(
    [switch]$Gateway,
    [switch]$Auth,
    [switch]$Admin,
    [switch]$WithFront
)

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = Split-Path -Parent $root

if (-not ($Gateway -or $Auth -or $Admin)) {
    $Gateway = $true
    $Auth = $true
    $Admin = $true
}

if ($Gateway) {
    Start-Process -WorkingDirectory $root -NoNewWindow -FilePath "cargo" -ArgumentList "run -p apisentinel-gateway"
}
if ($Auth) {
    Start-Process -WorkingDirectory $root -NoNewWindow -FilePath "cargo" -ArgumentList "run -p apisentinel-auth"
}
if ($Admin) {
    Start-Process -WorkingDirectory $root -NoNewWindow -FilePath "cargo" -ArgumentList "run -p apisentinel-admin"
}

if ($WithFront) {
    $frontDir = Join-Path $root "front"
    if (Test-Path $frontDir) {
        Start-Process -WorkingDirectory $frontDir -NoNewWindow -Wait -FilePath "npm.cmd" -ArgumentList "install"
        Start-Process -WorkingDirectory $frontDir -NoNewWindow -FilePath "npm.cmd" -ArgumentList "run dev"
        Write-Host "Frontend started on http://localhost:5173"
    } else {
        Write-Host "Front directory not found: $frontDir"
    }
}

Write-Host "Services started. Use Task Manager or 'Stop-Process' to stop them."
