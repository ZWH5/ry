#!/bin/bash

# 测试豆瓣爬虫搜索功能 - 搜索"三体"

echo "========================================="
echo "Douban Book Scraper Test - Search for '三体'"
echo "========================================="
echo ""

# 搜索URL
SEARCH_URL="https://search.douban.com/book/subject_search?search_text=%E4%B8%89%E4%BD%93&start=0"

echo "测试URL: $SEARCH_URL"
echo ""
echo "下载HTML页面..."
echo ""

# 下载搜索结果页面
curl -s \
  -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36" \
  "$SEARCH_URL" > santi_search.html

if [ -f santi_search.html ]; then
  echo "✓ 页面下载成功 ($(wc -c < santi_search.html) bytes)"
  echo ""
  
  # 提取搜索结果（使用grep查找a.nbg链接）
  echo "搜索结果分析:"
  echo "=============="
  echo ""
  
  # 查找所有a.nbg链接
  BOOK_COUNT=$(grep -o 'class="nbg"' santi_search.html | wc -l)
  echo "发现 $BOOK_COUNT 本书"
  echo ""
  
  # 提取前5本书的信息
  echo "前5本书的信息:"
  echo "------------"
  
  grep -o '<a class="nbg" href="[^"]*">' santi_search.html | head -5 | while read -r line; do
    URL=$(echo "$line" | grep -o 'href="[^"]*"' | cut -d'"' -f2)
    BOOK_ID=$(echo "$URL" | grep -o '/subject/[0-9]*' | cut -d'/' -f3)
    echo "  ID: $BOOK_ID"
    echo "  URL: $URL"
  done
  
  echo ""
  echo "✓ 搜索测试完成"
  
else
  echo "✗ 页面下载失败"
  exit 1
fi

echo ""
echo "========================================="
echo "现在测试获取第一本书的详情..."
echo "========================================="
echo ""

# 提取第一本书的ID
FIRST_BOOK_ID=$(grep -o '<a class="nbg" href="[^"]*">' santi_search.html | head -1 | grep -o '/subject/[0-9]*' | cut -d'/' -f3)

if [ ! -z "$FIRST_BOOK_ID" ]; then
  echo "第一本书ID: $FIRST_BOOK_ID"
  echo ""
  
  DETAIL_URL="https://book.douban.com/subject/$FIRST_BOOK_ID/"
  echo "详情页面URL: $DETAIL_URL"
  echo ""
  
  echo "下载详情页面..."
  curl -s \
    -H "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36" \
    "$DETAIL_URL" > santi_detail_${FIRST_BOOK_ID}.html
  
  if [ -f santi_detail_${FIRST_BOOK_ID}.html ]; then
    echo "✓ 详情页面下载成功"
    echo ""
    
    # 提取书名
    TITLE=$(grep -o '<span property="v:itemreviewed">[^<]*</span>' santi_detail_${FIRST_BOOK_ID}.html | head -1 | sed 's/<[^>]*>//g')
    
    # 提取作者
    AUTHORS=$(grep -A 5 '<span class="pl">作者</span>' santi_detail_${FIRST_BOOK_ID}.html | grep -o '<a href="/author/[^"]*">[^<]*</a>' | sed 's/<[^>]*>//g' | head -3 | tr '\n' ',' | sed 's/,$//')
    
    # 提取出版社
    PUBLISHER=$(grep -A 2 '<span class="pl">出版社</span>' santi_detail_${FIRST_BOOK_ID}.html | tail -1 | sed 's/<[^>]*>//g' | xargs)
    
    # 提取出版年
    PUBDATE=$(grep -A 2 '<span class="pl">出版年</span>' santi_detail_${FIRST_BOOK_ID}.html | tail -1 | sed 's/<[^>]*>//g' | xargs)
    
    # 提取页数
    PAGES=$(grep -A 2 '<span class="pl">页数</span>' santi_detail_${FIRST_BOOK_ID}.html | tail -1 | sed 's/<[^>]*>//g' | xargs)
    
    echo "提取的书籍信息:"
    echo "=============="
    echo "ID: $FIRST_BOOK_ID"
    echo "书名: $TITLE"
    echo "作者: $AUTHORS"
    echo "出版社: $PUBLISHER"
    echo "出版年: $PUBDATE"
    echo "页数: $PAGES"
    echo ""
    echo "✓ 详情页面解析完成"
    
  else
    echo "✗ 详情页面下载失败"
  fi
else
  echo "✗ 未能提取书籍ID"
fi

echo ""
echo "========================================="
echo "测试完成！"
echo "========================================="
echo ""
echo "生成的HTML文件:"
ls -lh santi_*.html 2>/dev/null || echo "  (无HTML文件)"
