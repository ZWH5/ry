#!/usr/bin/env python3
"""
Douban Book Scraper Test - 小王子 Search
测试豆瓣网页爬虫搜索功能
"""

import requests
import re
from html.parser import HTMLParser
from urllib.parse import urlencode, quote

class BookParser(HTMLParser):
    """解析豆瓣搜索结果页面"""
    
    def __init__(self):
        super().__init__()
        self.books = []
        self.current_book = {}
        self.in_link = False
        self.in_image = False
        
    def handle_starttag(self, tag, attrs):
        attrs_dict = dict(attrs)
        
        # 捕获搜索结果链接
        if tag == 'a' and 'class' in attrs_dict and 'nbg' in attrs_dict.get('class', ''):
            self.in_link = True
            if 'href' in attrs_dict:
                href = attrs_dict['href']
                # 从URL中提取书籍ID: /subject/1234567/
                match = re.search(r'/subject/(\d+)', href)
                if match:
                    self.current_book = {'id': match.group(1), 'url': href}
        
        # 捕获图片
        if tag == 'img' and self.in_link:
            self.in_image = True
            if 'alt' in attrs_dict:
                self.current_book['title'] = attrs_dict['alt']
            if 'src' in attrs_dict:
                self.current_book['image'] = attrs_dict['src']
    
    def handle_endtag(self, tag):
        if tag == 'a' and self.in_link:
            self.in_link = False
            if self.current_book.get('id') and self.current_book.get('title'):
                self.books.append(self.current_book)
                self.current_book = {}

def search_douban_books(query, start=0):
    """搜索豆瓣书籍"""
    
    url = "https://search.douban.com/book/subject_search"
    params = {
        'search_text': query,
        'start': start
    }
    
    headers = {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
    }
    
    print(f"\n📚 搜索书籍: {query}")
    print(f"📍 URL: {url}?{urlencode(params)}")
    print(f"⏳ 正在获取...")
    
    try:
        response = requests.get(url, params=params, headers=headers, timeout=10)
        response.encoding = 'utf-8'
        
        print(f"✅ HTTP状态码: {response.status_code}")
        
        if response.status_code == 200:
            # 解析HTML
            parser = BookParser()
            parser.feed(response.text)
            
            print(f"\n📊 搜索结果统计:")
            print(f"   找到 {len(parser.books)} 本书")
            
            if parser.books:
                print(f"\n📖 搜索结果:")
                print("=" * 80)
                
                for idx, book in enumerate(parser.books[:10], 1):  # 显示前10本
                    print(f"\n{idx}. {book.get('title', 'N/A')}")
                    print(f"   ID: {book.get('id')}")
                    print(f"   URL: {book.get('url')}")
                    print(f"   Image: {book.get('image', 'N/A')[:60]}...")
                
                print("\n" + "=" * 80)
                
                # 尝试获取第一本书的详情
                if parser.books:
                    first_book_id = parser.books[0]['id']
                    print(f"\n🔍 尝试获取第一本书详情: {first_book_id}")
                    get_book_details(first_book_id)
            else:
                print("❌ 未找到搜索结果")
        else:
            print(f"❌ HTTP错误: {response.status_code}")
            
    except Exception as e:
        print(f"❌ 错误: {e}")

def get_book_details(book_id):
    """获取书籍详细信息"""
    
    url = f"https://book.douban.com/subject/{book_id}/"
    
    headers = {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36'
    }
    
    print(f"📍 详情页URL: {url}")
    print(f"⏳ 正在获取详情...")
    
    try:
        response = requests.get(url, headers=headers, timeout=10)
        response.encoding = 'utf-8'
        
        print(f"✅ HTTP状态码: {response.status_code}")
        
        if response.status_code == 200:
            html = response.text
            
            # 提取标题
            title_match = re.search(r'<span property="v:itemreviewed">([^<]+)</span>', html)
            title = title_match.group(1).strip() if title_match else "N/A"
            
            # 提取作者
            authors = []
            author_pattern = r'<span class="pl">作者</span>.*?</div>'
            author_section = re.search(author_pattern, html, re.DOTALL)
            if author_section:
                author_links = re.findall(r'>([^<]+)</a>', author_section.group(0))
                # 过滤出实际作者（去掉"更多"等文本）
                authors = [a.strip() for a in author_links if a.strip() and len(a.strip()) < 50]
            
            # 提取出版社
            publisher_match = re.search(r'<span class="pl">出版社</span>:\s*<a[^>]*>([^<]+)</a>', html)
            publisher = publisher_match.group(1).strip() if publisher_match else "N/A"
            
            # 提取出版年
            pubdate_match = re.search(r'<span class="pl">出版年</span>:\s*([^\s<]+)', html)
            pubdate = pubdate_match.group(1).strip() if pubdate_match else "N/A"
            
            # 提取页数
            pages_match = re.search(r'<span class="pl">页数</span>:\s*(\d+)', html)
            pages = pages_match.group(1) if pages_match else "N/A"
            
            # 提取简介
            intro_match = re.search(r'<div id="link-report"><div class="intro">\s*<p>([^<]+)</p>', html)
            intro = intro_match.group(1).strip()[:100] if intro_match else "N/A"
            
            print(f"\n📕 书籍详情:")
            print("=" * 80)
            print(f"标题: {title}")
            print(f"作者: {', '.join(authors) if authors else 'N/A'}")
            print(f"出版社: {publisher}")
            print(f"出版年: {pubdate}")
            print(f"页数: {pages}")
            print(f"简介: {intro}...")
            print("=" * 80)
            
        else:
            print(f"❌ HTTP错误: {response.status_code}")
            
    except Exception as e:
        print(f"❌ 错误: {e}")

if __name__ == '__main__':
    # 测试搜索 小王子
    search_douban_books('小王子')
