# test_server.ps1
Write-Host "Testing HTTP Server..." -ForegroundColor Cyan

$endpoints = @(
    @{Path="/help"; Expected="commands"},
    @{Path="/status"; Expected="status"},
    @{Path="/fibonacci?num=10"; Expected="55"},
    @{Path="/reverse?text=hello"; Expected="olleh"},
    @{Path="/toupper?text=hello"; Expected="HELLO"},
    @{Path="/timestamp"; Expected="timestamp"}
)

foreach ($endpoint in $endpoints) {
    try {
        $response = Invoke-WebRequest -Uri "http://localhost:8080$($endpoint.Path)" -UseBasicParsing
        $content = $response.Content
        
        if ($content -match $endpoint.Expected) {
            Write-Host "[OK] $($endpoint.Path)" -ForegroundColor Green
        } else {
            Write-Host "[FAIL] $($endpoint.Path) - Expected: $($endpoint.Expected)" -ForegroundColor Red
        }
    } catch {
        Write-Host "[ERROR] $($endpoint.Path) - $($_.Exception.Message)" -ForegroundColor Red
    }
}

Write-Host "`nDone!" -ForegroundColor Cyan