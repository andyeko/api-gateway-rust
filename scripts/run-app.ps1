param(
    [string[]]$Features = @("gateway"),
    [switch]$WithFront
)

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$root = Split-Path -Parent $root

$featureFlag = $Features -join ","

Write-Host "Running single app with features: $featureFlag"

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

cargo run -p apisentinel-app --features $featureFlag
