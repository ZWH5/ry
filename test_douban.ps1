# PowerShell test script for Douban API
Write-Host "Testing Douban API for book search..." -ForegroundColor Green
Write-Host ""

# Test 1: Search for '三体'
Write-Host "=== Test 1: Search for '三体' ===" -ForegroundColor Cyan
$response1 = Invoke-WebRequest -Uri "https://api.douban.com/book/search?q=%E4%B8%89%E4%BD%93&count=5&start=0" -UseBasicParsing
Write-Host $response1.Content | ConvertFrom-Json | ConvertTo-Json -Depth 10

Write-Host ""

# Test 2: Search for '活着'
Write-Host "=== Test 2: Search for '活着' ===" -ForegroundColor Cyan
$response2 = Invoke-WebRequest -Uri "https://api.douban.com/book/search?q=%E6%B4%BB%E7%9D%80&count=5&start=0" -UseBasicParsing
Write-Host $response2.Content | ConvertFrom-Json | ConvertTo-Json -Depth 10

Write-Host ""
Write-Host "Testing completed!" -ForegroundColor Green
