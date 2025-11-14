#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
测试搜刮豆瓣 - 搜索"三体"
验证反爬虫机制效果
"""

import requests
import json
import time
from datetime import datetime

def test_douban_search(query: str, max_retries: int = 3) -> dict:
    """
    测试搜刮豆瓣图书
    
    Args:
        query: 搜索关键词
        max_retries: 最大重试次数
        
    Returns:
        搜索结果字典
    """
    
    url = "https://book.douban.com/j/search"
    
    # 真实浏览器User-Agent列表
    user_agents = [
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/119.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.1 Safari/605.1.15",
    ]
    
    params = {
        "search_text": query,
        "start": 0,
        "cat": "1001",
    }
    
    headers = {
        "Referer": "https://book.douban.com/",
        "Accept-Language": "zh-CN,zh;q=0.9",
        "Accept-Encoding": "gzip, deflate, br",
        "Cache-Control": "max-age=0",
        "DNT": "1",
        "Upgrade-Insecure-Requests": "1",
        "Connection": "keep-alive",
    }
    
    print(f"🔍 开始搜刮豆瓣图书: '{query}'")
    print(f"⏰ 时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print("─" * 60)
    
    for attempt in range(max_retries):
        try:
            # 轮换User-Agent
            ua_index = attempt % len(user_agents)
            headers["User-Agent"] = user_agents[ua_index]
            
            print(f"\n📡 请求 #{attempt + 1}/{max_retries}")
            print(f"   User-Agent: {user_agents[ua_index][:50]}...")
            
            # 添加延迟 (反爬虫措施)
            if attempt > 0:
                delay = 2 ** attempt  # 指数退避: 2s, 4s, 8s
                print(f"   ⏳ 延迟 {delay} 秒...")
                time.sleep(delay)
            
            start_time = time.time()
            response = requests.get(url, params=params, headers=headers, timeout=10)
            elapsed = time.time() - start_time
            
            print(f"   ✓ HTTP {response.status_code} (耗时: {elapsed:.2f}s)")
            
            if response.status_code == 200:
                data = response.json()
                print(f"   📊 数据大小: {len(response.text)} 字节")
                
                if "error_info" in data:
                    error = data["error_info"]
                    print(f"   ⚠️  错误: {error}")
                    
                    if "搜索访问太频繁" in error:
                        print(f"   💤 被限流，{attempt + 1 < max_retries and '准备重试...' or '已达最大重试次数'}")
                        if attempt + 1 < max_retries:
                            continue
                    else:
                        return data
                else:
                    print(f"   ✅ 搜索成功!")
                    return data
            else:
                print(f"   ❌ HTTP错误 {response.status_code}")
        
        except requests.exceptions.Timeout:
            print(f"   ❌ 超时 (10秒)")
            if attempt + 1 < max_retries:
                continue
        except requests.exceptions.ConnectionError as e:
            print(f"   ❌ 连接错误: {str(e)[:50]}")
            if attempt + 1 < max_retries:
                continue
        except json.JSONDecodeError as e:
            print(f"   ❌ JSON解析错误: {str(e)[:50]}")
            print(f"   响应内容: {response.text[:200]}")
            if attempt + 1 < max_retries:
                continue
        except Exception as e:
            print(f"   ❌ 错误: {str(e)[:100]}")
            if attempt + 1 < max_retries:
                continue
    
    print(f"\n❌ 所有重试都失败了")
    return {"error": "all_retries_failed"}


def main():
    # 测试搜刮"三体"
    result = test_douban_search("三体")
    
    print("\n" + "=" * 60)
    print("📋 搜索结果")
    print("=" * 60)
    
    if "error" in result:
        print(f"❌ 错误: {result.get('error_info', result.get('error'))}")
    else:
        total = result.get("total", 0)
        items = result.get("items", [])
        error_info = result.get("error_info", "无")
        
        print(f"总结果数: {total}")
        print(f"返回条数: {len(items)}")
        print(f"错误信息: {error_info}")
        
        if items:
            print("\n📚 前5条结果:")
            print("─" * 60)
            for i, item in enumerate(items[:5], 1):
                print(f"\n{i}. {item.get('title', '未知')}")
                print(f"   作者: {item.get('author', '未知')}")
                print(f"   出版社: {item.get('publisher', '未知')}")
                print(f"   评分: {item.get('rate', 'N/A')}")
                print(f"   ID: {item.get('id', 'N/A')}")
        else:
            print("\n❌ 没有返回任何结果")
    
    print("\n" + "=" * 60)
    print(f"✅ 测试完成 (时间: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')})")
    print("=" * 60)
    
    # 保存原始响应用于分析
    with open("test_santi_response.json", "w", encoding="utf-8") as f:
        json.dump(result, f, ensure_ascii=False, indent=2)
    print("✓ 原始响应已保存到 test_santi_response.json")


if __name__ == "__main__":
    main()
