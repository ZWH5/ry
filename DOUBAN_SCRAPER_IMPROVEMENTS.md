# 豆瓣网页爬虫改进文档

## 概述

通过参考calibre-douban项目的BeautifulSoup实现，对Rust版本的豆瓣书籍网页爬虫进行了系统改进，大幅提升了HTML解析的鲁棒性和可靠性。

## 主要改进

### 1. 搜索结果解析 (`parse_search_results`)

**改进前的问题：**
- 使用较为复杂的CSS选择器链 (`div.subject-item` → `h2 a` → `img.cover`)
- 对豆瓣前端变化敏感

**改进方案：**
- 采用Python版本的直接选择器：`a.nbg`（搜索结果链接）
- 从链接的`alt`属性提取书名
- 从图片的`src`属性提取封面URL
- 更可靠地从URL中提取书籍ID

```rust
// 与calibre-douban保持一致的选择器
if let Ok(item_selector) = Selector::parse("a.nbg") {
    for item in document.select(&item_selector) {
        // 提取ID、书名、图片
    }
}
```

### 2. 书籍详情解析 (`parse_book_details`)

**改进前的问题：**
- 使用字符串查找(`find()`)和手动截断处理HTML
- 容易因为HTML结构变化而失败
- 没有后备选择器

**改进方案：**
- 实现多层选择器策略
- 首先尝试标准选择器 `span[property='v:itemreviewed']`
- 失败时尝试后备选择器 `h1 span`
- 使用结构化CSS选择器代替字符串操作

```rust
// 主选择器
if let Ok(selector) = Selector::parse("span[property='v:itemreviewed']") {
    if let Some(elem) = document.select(&selector).next() {
        book.title = elem.inner_html().trim().to_string();
    }
}

// 后备选择器
if book.title.is_empty() {
    if let Ok(selector) = Selector::parse("h1 span") {
        // 尝试备用方案
    }
}
```

### 3. 元数据提取函数 (`parse_book_info`)

**新增功能：**
- 结构化提取所有书籍元数据
- 支持提取作者、出版社、出版年、页数等
- 智能处理副标题合并
- 过滤无效的作者链接

```rust
fn parse_book_info(&self, document: &Html, book: &mut DoubanBook) {
    if let Ok(selector) = Selector::parse("span.pl") {
        for element in document.select(&selector) {
            let text = self.get_element_text(&element);
            
            if text.starts_with("作者") {
                // 从父元素中提取作者链接
                // 过滤 /author 或 /search URL
            } else if text.starts_with("出版社") {
                // 提取出版社信息
            } else if text.starts_with("出版年") {
                // 提取出版年份
            }
            // ... 更多字段
        }
    }
}
```

### 4. 文本提取辅助函数

**新增 `get_element_text()`:**
- 安全提取元素内所有文本
- 自动过滤空白符
- 处理嵌套元素

**新增 `get_tail_text()`:**
- 获取元素后面的文本内容（类似Python的`get_tail()`)
- 处理兄弟节点的文本和元素
- 跳过分隔符（如`|`)

```rust
fn get_element_text(&self, element: &ElementRef) -> String {
    element
        .text()
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join("")
}

fn get_tail_text(&self, element: &ElementRef) -> Option<String> {
    // 获取元素后面的文本内容
    if let Some(mut next_sibling) = element.next_sibling() {
        // 处理后续文本节点
    }
    None
}
```

### 5. 书籍ID提取 (`extract_book_id`)

**改进方案：**
- 支持两种URL格式：`/subject/123/` 和 `/subject/123`
- 使用`find()`和切片而不是正则表达式（性能更好）
- 明确注释说明URL格式

```rust
fn extract_book_id(&self, href: &str) -> Option<String> {
    if let Some(start) = href.find("/subject/") {
        let after_subject = &href[start + 9..];
        if let Some(end) = after_subject.find('/') {
            return Some(after_subject[..end].to_string());
        } else {
            return Some(after_subject.to_string());
        }
    }
    None
}
```

### 6. 错误处理改进

- 所有Selector::parse()都使用`if let Ok()`而非`?`，防止单一失败导致整个请求失败
- 使用`Option`处理可能缺失的字段
- 优雅降级而不是panic

## 与calibre-douban的对应关系

| Python (calibre-douban) | Rust (Ryot) | 功能 |
|---|---|---|
| `load_book_urls_new()` - `a.nbg` | `parse_search_results()` | 搜索结果列表解析 |
| `parse_book()` - `span[property='v:itemreviewed']` | `parse_book_details()` | 书籍详情解析 |
| `parse_book()` - `span.pl` 循环 | `parse_book_info()` | 元数据提取 |
| `get_text()` | `get_element_text()` | 元素文本提取 |
| `get_tail()` | `get_tail_text()` | 尾部文本提取 |
| `author_filter()` | 内联过滤逻辑 | 作者链接过滤 |

## 鲁棒性特性

### 1. 选择器容错
- 使用`if let Ok()`处理失败的选择器
- 提供后备选择器方案

### 2. 字段验证
- 检查字符串是否为空
- 检查文本长度（防止提取垃圾数据）
- 只在有效数据时更新字段

### 3. 边界处理
- 处理URL末尾可能没有`/`的情况
- 处理遗漏的字段（使用`Option`）
- 处理HTML编码字符（使用`html_escape`)

## 编译验证

所有改进已通过Rust编译检查：
- 无编译错误
- 类型安全
- 符合项目代码风格

## 使用示例

```rust
let service = GoogleBooksService::new(&config).await?;

// 搜索书籍
let results = service
    .metadata_search(1, "三体", false, &None)
    .await?;

// 获取书籍详情
let details = service
    .metadata_details("2261569")
    .await?;
```

## 未来可能的改进

1. **重试机制：** 添加自动重试逻辑处理网络临时问题
2. **缓存：** 实现搜索结果缓存以减少网络请求
3. **配置化选择器：** 支持通过配置文件更新CSS选择器
4. **性能优化：** 使用并发请求处理多本书籍详情查询
5. **日志记录：** 添加详细的调试日志用于问题诊断

## 参考资源

- 原项目参考：[calibre-douban](https://github.com/fugary/calibre-douban)
- HTML解析库：[scraper crate](https://docs.rs/scraper/)
- BeautifulSoup文档对应的Rust实现模式

## 提交记录

- 提交ID：`dfa1ee7f`
- 标题：Improve Douban web scraper with robust HTML parsing
- 文件：`crates/providers/google-books/src/lib.rs`
- 统计：158 insertions(+), 118 deletions(-)
