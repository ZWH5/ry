#!/bin/bash

# 测试豆瓣API搜刮功能
echo "Testing Douban API for book search..."

# 测试搜索功能
echo ""
echo "=== Test 1: Search for '三体' ==="
curl -s "https://api.douban.com/book/search?q=三体&count=5&start=0" | jq '.'

echo ""
echo "=== Test 2: Search for '活着' ==="
curl -s "https://api.douban.com/book/search?q=活着&count=5&start=0" | jq '.'

echo ""
echo "=== Test 3: Get specific book details ==="
# 从搜索结果中获取第一本书的ID，然后获取详情
BOOK_ID=$(curl -s "https://api.douban.com/book/search?q=三体&count=1&start=0" | jq -r '.books[0].id')
if [ ! -z "$BOOK_ID" ]; then
    echo "Getting details for book ID: $BOOK_ID"
    curl -s "https://api.douban.com/book/$BOOK_ID" | jq '.'
else
    echo "Could not find book ID"
fi

echo ""
echo "Testing completed!"
