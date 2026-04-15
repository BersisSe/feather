# Test concurrent requests to the server
# Run this while the server is running: .\test_concurrent.ps1

Write-Host "Testing concurrent requests to /sync endpoint..."
Write-Host "Starting two requests simultaneously (should overlap, not wait for each other)"
Write-Host ""

$time_start = Get-Date

# Make two concurrent requests using Start-Job
$job1 = Start-Job -ScriptBlock {
    Write-Host "[Request 1] Starting at $(Get-Date -Format 'HH:mm:ss')"
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:5050/sync" -TimeoutSec 15
    Write-Host "[Request 1] Completed at $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Green
    return $response
}

$job2 = Start-Job -ScriptBlock {
    Start-Sleep -Milliseconds 100  # Small delay to ensure first request starts first
    Write-Host "[Request 2] Starting at $(Get-Date -Format 'HH:mm:ss')"
    $response = Invoke-WebRequest -Uri "http://127.0.0.1:5050/sync" -TimeoutSec 15
    Write-Host "[Request 2] Completed at $(Get-Date -Format 'HH:mm:ss')" -ForegroundColor Green
    return $response
}

# Wait for both to complete
$job1, $job2 | Wait-Job

$time_end = Get-Date
$total_time = ($time_end - $time_start).TotalSeconds

Write-Host ""
Write-Host "Total elapsed time: $($total_time)s" -ForegroundColor Cyan
Write-Host ""
Write-Host "If concurrent: ~5 seconds (both run in parallel)"
Write-Host "If sequential: ~10 seconds (one after another)"
