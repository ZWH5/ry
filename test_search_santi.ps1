# 测试豆瓣爬虫搜索功能 - 搜索"三体"

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "Douban Book Scraper Test - Search for '三体'" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

# 搜索URL
$SEARCH_URL = "https://search.douban.com/book/subject_search?search_text=%E4%B8%89%E4%BD%93&start=0"

Write-Host "测试URL: $SEARCH_URL" -ForegroundColor Green
Write-Host ""
Write-Host "下载HTML页面..."
Write-Host ""

try {
    # 下载搜索结果页面
    $response = Invoke-WebRequest -Uri $SEARCH_URL `
        -Headers @{
            "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
        } `
        -ErrorAction Stop

    $html = $response.Content
    $html | Out-File -FilePath "santi_search.html" -Encoding UTF8

    $fileSize = (Get-Item "santi_search.html").Length
    Write-Host "✓ 页面下载成功 ($fileSize bytes)" -ForegroundColor Green
    Write-Host ""

    # 提取搜索结果
    Write-Host "搜索结果分析:" -ForegroundColor Yellow
    Write-Host "=============" -ForegroundColor Yellow
    Write-Host ""

    # 查找所有a.nbg链接
    $bookMatches = [regex]::Matches($html, '<a class="nbg" href="([^"]*)">')
    $bookCount = $bookMatches.Count

    Write-Host "发现 $bookCount 本书" -ForegroundColor Green
    Write-Host ""

    Write-Host "前5本书的信息:" -ForegroundColor Yellow
    Write-Host "------------" -ForegroundColor Yellow
    Write-Host ""

    for ($i = 0; $i -lt [Math]::Min(5, $bookMatches.Count); $i++) {
        $url = $bookMatches[$i].Groups[1].Value
        $bookIdMatch = [regex]::Match($url, '/subject/(\d+)')
        if ($bookIdMatch.Success) {
            $bookId = $bookIdMatch.Groups[1].Value
            Write-Host "  书籍 $($i+1):"
            Write-Host "    ID: $bookId"
            Write-Host "    URL: $url"
            Write-Host ""
        }
    }

    Write-Host "✓ 搜索测试完成" -ForegroundColor Green

} catch {
    Write-Host "✗ 页面下载失败: $_" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "现在测试获取第一本书的详情..." -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

# 提取第一本书的ID
if ($bookMatches.Count -gt 0) {
    $firstUrl = $bookMatches[0].Groups[1].Value
    $firstBookIdMatch = [regex]::Match($firstUrl, '/subject/(\d+)')
    
    if ($firstBookIdMatch.Success) {
        $firstBookId = $firstBookIdMatch.Groups[1].Value
        
        Write-Host "第一本书ID: $firstBookId" -ForegroundColor Green
        Write-Host ""

        $DETAIL_URL = "https://book.douban.com/subject/$firstBookId/"
        Write-Host "详情页面URL: $DETAIL_URL" -ForegroundColor Green
        Write-Host ""

        Write-Host "下载详情页面..."

        try {
            $detailResponse = Invoke-WebRequest -Uri $DETAIL_URL `
                -Headers @{
                    "User-Agent" = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
                } `
                -ErrorAction Stop

            $detailHtml = $detailResponse.Content
            $detailHtml | Out-File -FilePath "santi_detail_${firstBookId}.html" -Encoding UTF8

            Write-Host "✓ 详情页面下载成功" -ForegroundColor Green
            Write-Host ""

            # 提取书名
            $titleMatch = [regex]::Match($detailHtml, '<span property="v:itemreviewed">([^<]*)</span>')
            $title = if ($titleMatch.Success) { $titleMatch.Groups[1].Value } else { "未找到" }

            # 提取作者
            $authorsMatches = [regex]::Matches($detailHtml, '<span class="pl">作者</span>.*?<a href="/author/[^"]*">([^<]*)</a>', [System.Text.RegularExpressions.RegexOptions]::Singleline)
            $authors = if ($authorsMatches.Count -gt 0) { 
                ($authorsMatches | ForEach-Object { $_.Groups[1].Value }) -join "," 
            } else { 
                "未找到" 
            }

            # 提取出版社
            $publisherMatch = [regex]::Match($detailHtml, '<span class="pl">出版社</span>\s*:\s*<a[^>]*>([^<]*)</a>', [System.Text.RegularExpressions.RegexOptions]::Singleline)
            $publisher = if ($publisherMatch.Success) { $publisherMatch.Groups[1].Value } else { "未找到" }

            # 提取出版年
            $pubdateMatch = [regex]::Match($detailHtml, '<span class="pl">出版年</span>\s*:\s*([^\n<]*)', [System.Text.RegularExpressions.RegexOptions]::Singleline)
            $pubdate = if ($pubdateMatch.Success) { $pubdateMatch.Groups[1].Value.Trim() } else { "未找到" }

            # 提取页数
            $pagesMatch = [regex]::Match($detailHtml, '<span class="pl">页数</span>\s*:\s*(\d+)', [System.Text.RegularExpressions.RegexOptions]::Singleline)
            $pages = if ($pagesMatch.Success) { $pagesMatch.Groups[1].Value } else { "未找到" }

            Write-Host "提取的书籍信息:" -ForegroundColor Yellow
            Write-Host "==============" -ForegroundColor Yellow
            Write-Host "ID: $firstBookId" -ForegroundColor White
            Write-Host "书名: $title" -ForegroundColor White
            Write-Host "作者: $authors" -ForegroundColor White
            Write-Host "出版社: $publisher" -ForegroundColor White
            Write-Host "出版年: $pubdate" -ForegroundColor White
            Write-Host "页数: $pages" -ForegroundColor White
            Write-Host ""
            Write-Host "✓ 详情页面解析完成" -ForegroundColor Green

        } catch {
            Write-Host "✗ 详情页面下载失败: $_" -ForegroundColor Red
        }

    } else {
        Write-Host "✗ 未能提取书籍ID" -ForegroundColor Red
    }
} else {
    Write-Host "✗ 搜索结果为空" -ForegroundColor Red
}

Write-Host ""
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "测试完成！" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "生成的HTML文件:" -ForegroundColor Green
Get-Item santi_*.html -ErrorAction SilentlyContinue | ForEach-Object {
    Write-Host "  - $($_.Name) ($($_.Length) bytes)"
}
