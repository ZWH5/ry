#!/usr/bin/env python3
"""
CSS Selector Testing Script for Douban Book Scraping
Validates that CSS selectors can correctly extract book metadata
"""

from html.parser import HTMLParser
from html import unescape
import re
import sys
from pathlib import Path


def extract_with_css(html_content: str) -> dict:
    """
    Mimics the Rust CSS selector extraction using regex and HTML parsing
    """
    result = {}
    
    # Extract title - span[property='v:itemreviewed']
    title_match = re.search(r"<span property=['\"]v:itemreviewed['\"]>([^<]+)</span>", html_content)
    if title_match:
        result['title'] = title_match.group(1).strip()
    else:
        # Fallback: h1 span
        title_match = re.search(r"<h1[^>]*>\s*<span[^>]*>([^<]+)</span>", html_content)
        if title_match:
            result['title'] = title_match.group(1).strip()
    
    # Extract image - a.nbg > img src
    img_match = re.search(r"<a class=['\"]nbg['\"][^>]*>.*?<img[^>]*src=['\"]([^'\"]+)['\"]", html_content, re.DOTALL)
    if img_match:
        result['image'] = img_match.group(1)
    
    # Extract metadata labels with their values
    # Looking for <span class="pl">LABEL</span>...:VALUE pattern
    
    # Extract authors
    author_pattern = r"<span class=['\"]pl['\"]>作者</span>\s*<a[^>]*href=['\"][^'\"]*author[^'\"]*['\"]>([^<]+)</a>"
    authors = re.findall(author_pattern, html_content)
    if authors:
        result['authors'] = [a.strip() for a in authors]
    
    # Extract publisher 
    pub_pattern = r"<span class=['\"]pl['\"]>出版社</span>\s*:\s*<a[^>]*>([^<]+)</a>"
    pub_match = re.search(pub_pattern, html_content)
    if pub_match:
        result['publisher'] = pub_match.group(1).strip()
    
    # Extract publication year
    year_pattern = r"<span class=['\"]pl['\"]>出版年</span>\s*:\s*([^\n<]+)"
    year_match = re.search(year_pattern, html_content)
    if year_match:
        result['pubdate'] = year_match.group(1).strip()
    
    # Extract pages
    pages_pattern = r"<span class=['\"]pl['\"]>页数</span>\s*:\s*(\d+)"
    pages_match = re.search(pages_pattern, html_content)
    if pages_match:
        result['pages'] = int(pages_match.group(1))
    
    # Extract ISBN
    isbn_pattern = r"<span class=['\"]pl['\"]>ISBN</span>\s*:\s*(\d+(?:\-\d+)*)"
    isbn_match = re.search(isbn_pattern, html_content)
    if isbn_match:
        result['isbn'] = isbn_match.group(1).strip()
    
    return result


def test_html_file(filepath: str) -> None:
    """Test CSS extraction on a HTML file"""
    path = Path(filepath)
    
    if not path.exists():
        print(f"❌ File not found: {filepath}")
        sys.exit(1)
    
    print(f"\n📖 Testing: {path.name}")
    print(f"   Size: {path.stat().st_size:,} bytes")
    print("-" * 60)
    
    with open(path, 'r', encoding='utf-8') as f:
        html_content = f.read()
    
    data = extract_with_css(html_content)
    
    # Display results
    if not data:
        print("❌ No data extracted!")
        return
    
    print(f"✓ Title: {data.get('title', 'N/A')}")
    print(f"✓ Authors: {', '.join(data.get('authors', ['N/A']))}")
    print(f"✓ Publisher: {data.get('publisher', 'N/A')}")
    print(f"✓ Publication Year: {data.get('pubdate', 'N/A')}")
    print(f"✓ Pages: {data.get('pages', 'N/A')}")
    print(f"✓ ISBN: {data.get('isbn', 'N/A')}")
    print(f"✓ Image: {data.get('image', 'N/A')[:60]}...")
    
    # Validation checks
    print("\n✅ Validation Results:")
    checks = [
        ("Title extracted", bool(data.get('title'))),
        ("Authors extracted", bool(data.get('authors'))),
        ("Publisher extracted", bool(data.get('publisher'))),
        ("Publication year extracted", bool(data.get('pubdate'))),
        ("Image URL extracted", bool(data.get('image'))),
    ]
    
    passed = sum(1 for _, result in checks if result)
    for check_name, result in checks:
        status = "✓" if result else "✗"
        print(f"  {status} {check_name}")
    
    print(f"\nPassed: {passed}/{len(checks)}")


if __name__ == "__main__":
    # Test the downloaded file
    import os
    print(f"Current directory: {os.getcwd()}")
    print(f"Files in directory: {os.listdir('.')[:20]}\n")
    
    files_to_test = ["test_1003078_huozhe.html", "huozhe_book.html", "cangdi_book.html"]
    for f in files_to_test:
        if os.path.exists(f):
            test_html_file(f)
        else:
            print(f"⏭️  Skipping {f} (not found)")
