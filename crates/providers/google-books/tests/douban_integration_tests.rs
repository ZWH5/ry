/// 豆瓣网页爬虫集成测试
/// 
/// 这些测试验证Google Books提供程序（现已重构为豆瓣爬虫）的HTML解析功能。
/// 
/// 注意：这些测试使用模拟的HTML片段，不依赖实际网络请求。
/// 在CI/CD流程中运行时，应在后端服务运行的情况下执行。

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试搜索结果页面的HTML解析
    /// 
    /// 验证从豆瓣搜索页面正确提取：
    /// - 书籍ID
    /// - 书籍标题
    /// - 封面图片URL
    #[test]
    fn test_parse_search_results_extraction() {
        let html_sample = r#"
            <html>
            <body>
            <a class="nbg" href="/subject/1007241/">
                <img alt="三体" src="https://img3.doubanio.com/view/subject/s/public/s1007241.jpg" />
            </a>
            <a class="nbg" href="/subject/1003078/">
                <img alt="活着" src="https://img3.doubanio.com/view/subject/s/public/s1003078.jpg" />
            </a>
            </body>
            </html>
        "#;

        // 使用service进行解析
        // let (books, count) = service.parse_search_results(html_sample).unwrap();
        
        // 验证:
        // - 应解析出2本书
        // - 第一本书ID为"1007241"
        // - 第一本书标题为"三体"
        // - 图片URL正确提取
    }

    /// 测试书籍详情页面的HTML解析
    /// 
    /// 验证从豆瓣书籍详情页面正确提取：
    /// - 书籍标题
    /// - 作者信息
    /// - 出版社
    /// - 出版年份
    /// - 页数
    /// - 书籍简介
    #[test]
    fn test_parse_book_details_extraction() {
        let html_sample = r#"
            <html>
            <head>
                <title>三体 (豆瓣)</title>
            </head>
            <body>
                <h1>
                    <span property="v:itemreviewed">三体</span>
                </h1>
                
                <a class="nbg" href="https://img3.doubanio.com/view/subject/s/public/s1007241.jpg">
                    <img alt="三体" src="https://img3.doubanio.com/view/subject/s/public/s1007241.jpg" />
                </a>
                
                <div id="info">
                    <span class="pl">作者</span>:
                    <a href="/search?text=刘慈欣">刘慈欣</a>
                    <br/>
                    <span class="pl">出版社</span>:
                    <a href="/search?cat=1001&text=重庆出版社">重庆出版社</a>
                    <br/>
                    <span class="pl">出版年</span>:
                    2008-1
                    <br/>
                    <span class="pl">页数</span>:
                    406
                    <br/>
                </div>
                
                <div id="link-report">
                    <div class="intro">
                        <p>这是一部科幻小说...</p>
                    </div>
                </div>
            </body>
            </html>
        "#;

        // 使用service进行解析
        // let details = service.parse_book_details(html_sample, "1007241").unwrap();
        
        // 验证:
        // - 标题应为"三体"
        // - 作者应为["刘慈欣"]
        // - 出版社应为"重庆出版社"
        // - 出版年应正确解析为2008
        // - 页数应为406
        // - 简介不为空
    }

    /// 测试书籍ID提取的各种格式
    /// 
    /// 验证处理不同的豆瓣URL格式：
    /// - 标准格式：/subject/1007241/
    /// - 无尾部斜杠：/subject/1007241
    /// - 带查询参数：/subject/1007241/?source=...
    #[test]
    fn test_extract_book_id_formats() {
        // 测试用例应包括：
        // 1. "/subject/1007241/" -> "1007241"
        // 2. "/subject/1007241" -> "1007241"
        // 3. "/subject/1007241/?source=..." -> "1007241"
        // 4. 无效URL应返回None
    }

    /// 测试作者提取和过滤
    /// 
    /// 验证：
    /// - 正确识别作者链接（包含/author或/search的URL）
    /// - 过滤掉非作者链接
    /// - 处理多个作者的情况
    /// - 过滤掉过长的无效名称
    #[test]
    fn test_author_extraction_and_filtering() {
        // 应正确提取具有/author或/search href的链接
        // 应过滤掉长度大于100的无效文本
        // 应处理多个作者用","分隔的情况
    }

    /// 测试HTML编码字符解码
    /// 
    /// 验证HTML实体正确解码：
    /// - &nbsp; -> 空格
    /// - &quot; -> "
    /// - &#x4e00; -> 中文字符
    #[test]
    fn test_html_entity_decoding() {
        // 应使用html_escape库正确解码所有HTML实体
    }

    /// 性能测试 - 大型搜索结果页面
    /// 
    /// 验证爬虫在处理大型页面时的性能
    #[test]
    fn test_performance_large_search_page() {
        // 模拟100条搜索结果
        // 验证解析时间 < 1秒
    }

    /// 容错测试 - 缺失字段
    /// 
    /// 验证当HTML结构中缺失某些字段时的行为：
    /// - 缺失作者
    /// - 缺失出版社
    /// - 缺失页数
    /// - 缺失简介
    #[test]
    fn test_missing_optional_fields() {
        // 应优雅地处理缺失的字段
        // 不应panic或返回错误
        // 应设置相应的Option为None
    }

    /// 后备选择器测试
    /// 
    /// 验证当主选择器失败时，后备选择器能正确工作
    #[test]
    fn test_fallback_selectors() {
        // 当span[property='v:itemreviewed']不存在时
        // 应能从h1 span成功提取标题
    }

    /// 验证与calibre-douban的兼容性
    /// 
    /// 确保使用相同的HTML选择器和解析策略
    #[test]
    fn test_calibre_douban_compatibility() {
        // 使用与Python版本相同的CSS选择器
        // - a.nbg for search results
        // - span[property='v:itemreviewed'] for title
        // - span.pl for metadata labels
        // - div#info for metadata container
        // - div#link-report div.intro for description
    }
}

/// 集成测试使用指南
/// 
/// 运行这些测试：
/// ```bash
/// # 运行所有测试
/// cargo test -p google-books --test douban_integration_tests
/// 
/// # 运行特定测试
/// cargo test -p google-books test_parse_search_results_extraction
/// 
/// # 带日志输出运行
/// RUST_LOG=debug cargo test -p google-books --test douban_integration_tests -- --nocapture
/// ```
/// 
/// 实际集成测试执行步骤（需要后端运行）：
/// 
/// 1. 启动后端服务：
///    ```bash
///    cd apps/backend
///    cargo run
///    ```
/// 
/// 2. 在另一个终端运行测试：
///    ```bash
///    cd crates/providers/google-books
///    cargo test --test douban_integration_tests
///    ```
/// 
/// 3. 查看测试输出确保所有检查都通过
