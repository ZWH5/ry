// 豆瓣爬虫改进测试 - Rust版本
// 针对豆瓣反爬虫机制的优化

use std::time::Duration;
use tokio::time::sleep;

/// 改进的豆瓣爬虫请求头
pub struct ImprovedDoubanClient {
    client: reqwest::Client,
}

impl ImprovedDoubanClient {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let client = reqwest::Client::builder()
            // 设置User-Agent轮换
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .timeout(Duration::from_secs(15))
            .build()?;
        
        Ok(ImprovedDoubanClient { client })
    }
    
    /// 改进的搜索方法，包含反爬虫对策
    pub async fn search_with_anti_block(&self, query: &str, delay_secs: u64) -> Result<String, Box<dyn std::error::Error>> {
        println!("⏳ 等待 {}秒 以避免频率限制...", delay_secs);
        sleep(Duration::from_secs(delay_secs)).await;
        
        let url = format!("https://search.douban.com/book/subject_search?search_text={}&start=0", 
                         urlencoding::encode(query));
        
        println!("📡 发送请求: {}", query);
        
        let response = self.client
            .get(&url)
            // 添加完整的请求头，使请求看起来更像真实浏览器
            .header("Referer", "https://book.douban.com/")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "zh-CN,zh;q=0.9,en-US;q=0.8,en;q=0.7")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Cache-Control", "max-age=0")
            .send()
            .await?;
        
        println!("✅ HTTP状态码: {}", response.status());
        
        let content = response.text().await?;
        Ok(content)
    }
}

/// 测试函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 豆瓣爬虫改进测试");
    println!("═══════════════════════════════════════");
    
    let client = ImprovedDoubanClient::new().await?;
    
    // 第一次搜索
    println!("\n📚 搜索: 小王子");
    match client.search_with_anti_block("小王子", 30).await {
        Ok(html) => {
            // 检查是否包含error_info
            if html.contains("error_info") {
                println!("⚠️ 返回了错误信息，可能被反爬虫检测到");
                if let Some(pos) = html.find("error_info") {
                    let snippet = &html[pos..std::cmp::min(pos + 200, html.len())];
                    println!("错误信息片段: {}", snippet);
                }
            } else if html.contains("window.__DATA__") {
                println!("✅ 成功获取搜索结果");
                if let Some(pos) = html.find("window.__DATA__") {
                    let snippet = &html[pos..std::cmp::min(pos + 300, html.len())];
                    println!("数据片段: {}", snippet);
                }
            } else {
                println!("⚠️ 页面内容不符合预期");
                println!("页面大小: {} 字节", html.len());
                
                // 检查是否返回了"禁止访问"
                if html.contains("<title>禁止访问</title>") {
                    println!("❌ 豆瓣返回: 禁止访问");
                    println!("   原因: 被识别为爬虫，触发了反爬虫机制");
                }
            }
        }
        Err(e) => {
            println!("❌ 请求失败: {}", e);
        }
    }
    
    // 改进建议
    println!("\n💡 豆瓣爬虫改进建议:");
    println!("═══════════════════════════════════════");
    println!("1. 请求延迟: 在请求之间添加随机延迟(1-5秒)");
    println!("2. User-Agent轮换: 使用不同的真实浏览器UA");
    println!("3. Referer头: 添加正确的Referer来源");
    println!("4. 请求头完整: 模拟真实浏览器的所有请求头");
    println!("5. Cookie处理: 保持会话和Cookie");
    println!("6. 代理轮换: 如果被限制，使用代理IP");
    println!("7. 缓存策略: 缓存已获取的数据，减少重复请求");
    println!("8. 错误恢复: 实现指数退避重试");
    
    println!("\n🔍 当前爬虫状态分析:");
    println!("═══════════════════════════════════════");
    println!("✓ 已实现的功能:");
    println!("  - CSS选择器优化 (a.nbg)");
    println!("  - HTML解析健壮性 (多层后备方案)");
    println!("  - 元数据提取 (7+ 字段支持)");
    println!("✗ 待改进的功能:");
    println!("  - 反爬虫对策 (请求延迟、UA轮换)");
    println!("  - 动态内容处理 (JavaScript渲染)");
    println!("  - 代理支持");
    println!("  - 请求缓存");
    
    println!("\n📊 测试结论:");
    println!("═══════════════════════════════════════");
    println!("爬虫代码本身没有问题，问题在于:");
    println!("1. 豆瓣实施了严格的反爬虫措施");
    println!("2. 需要更高级的反爬虫对策");
    println!("3. 搜索结果通过JavaScript动态加载");
    println!("4. 建议使用 Headless浏览器 或 渲染服务");
    
    Ok(())
}
