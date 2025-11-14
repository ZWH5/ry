# 测试搜刮三体
$url = "https://book.douban.com/j/search?search_text=%E4%B8%89%E4%BD%93&start=0"
$headers = @{
    'User-Agent' = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/120.0.0.0 Safari/537.36'
    'Referer' = 'https://book.douban.com/'
    'Accept-Language' = 'zh-CN,zh;q=0.9'
}

Write-Host "🔍 测试搜刮豆瓣: 三体" -ForegroundColor Green
Write-Host "═════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "📡 URL: $url" -ForegroundColor Yellow
Write-Host ""

$start = Get-Date
try {
    $response = Invoke-WebRequest -Uri $url -Headers $headers -TimeoutSec 15 -UseBasicParsing
    $elapsed = (Get-Date) - $start
    
    Write-Host "✅ 请求成功" -ForegroundColor Green
    Write-Host "HTTP状态: $($response.StatusCode)" -ForegroundColor Green
    Write-Host "响应时间: $($elapsed.TotalSeconds)s" -ForegroundColor Cyan
    Write-Host "数据大小: $($response.Content.Length) bytes" -ForegroundColor Cyan
    Write-Host ""
    
    # 解析JSON
    $data = $response.Content | ConvertFrom-Json
    
    Write-Host "📊 搜索结果统计" -ForegroundColor Yellow
    Write-Host "─────────────────────────────────────" 
    Write-Host "总数: $($data.total)" -ForegroundColor Cyan
    Write-Host "返回条数: $($data.items.Count)" -ForegroundColor Cyan
    Write-Host "错误信息: $($data.error_info // '无')" -ForegroundColor $(if ($data.error_info) { 'Red' } else { 'Green' })
    Write-Host ""
    
    if ($data.items.Count -gt 0) {
        Write-Host "📚 搜索结果 (前3条)" -ForegroundColor Yellow
        Write-Host "─────────────────────────────────────"
        $data.items[0..([Math]::Min(2, $data.items.Count-1))] | ForEach-Object {
            Write-Host ""
            Write-Host "• $($_.title)" -ForegroundColor Cyan
            Write-Host "  作者: $($_.author)" -ForegroundColor Gray
            Write-Host "  评分: $($_.rate)" -ForegroundColor Gray
            Write-Host "  出版社: $($_.publisher)" -ForegroundColor Gray
        }
    } else {
        Write-Host "❌ 没有返回结果" -ForegroundColor Red
        if ($data.error_info) {
            Write-Host "原因: $($data.error_info)" -ForegroundColor Yellow
        }
    }
    
} catch {
    $elapsed = (Get-Date) - $start
    Write-Host "❌ 请求失败" -ForegroundColor Red
    Write-Host "耗时: $($elapsed.TotalSeconds)s" -ForegroundColor Yellow
    Write-Host "错误: $($_.Exception.Message)" -ForegroundColor Red
}

Write-Host ""
Write-Host "═════════════════════════════════════════" -ForegroundColor Cyan
Write-Host "✅ 测试完成" -ForegroundColor Green
