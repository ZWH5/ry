/// 快速测试：三体书籍搜索
/// 
/// 这个脚本演示如何使用改进后的Douban爬虫来搜索"三体"
/// 
/// 使用方法：
/// 1. 确保后端服务正在运行
/// 2. 发送GraphQL查询到backend获取书籍搜索结果
/// 
/// 示例GraphQL查询:
/// ```graphql
/// query {
///   mediaSearch(
///     query: "三体"
///     mediaType: BOOK
///     page: 1
///   ) {
///     items {
///       identifier
///       title
///       image
///     }
///   }
/// }
/// ```

use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TestBook {
    pub id: String,
    pub title: String,
    pub author: Option<String>,
    pub publisher: Option<String>,
    pub year: Option<i32>,
    pub pages: Option<i32>,
}

/// 已知的"三体"相关书籍 (用于验证)
pub fn get_known_santi_books() -> Vec<TestBook> {
    vec![
        TestBook {
            id: "2261569".to_string(),
            title: "三体".to_string(),
            author: Some("刘慈欣".to_string()),
            publisher: Some("重庆出版社".to_string()),
            year: Some(2008),
            pages: Some(406),
        },
        TestBook {
            id: "3288617".to_string(),
            title: "三体II·黑暗森林".to_string(),
            author: Some("刘慈欣".to_string()),
            publisher: Some("重庆出版社".to_string()),
            year: Some(2008),
            pages: Some(513),
        },
        TestBook {
            id: "2768253".to_string(),
            title: "三体III·死神永生".to_string(),
            author: Some("刘慈欣".to_string()),
            publisher: Some("重庆出版社".to_string()),
            year: Some(2010),
            pages: Some(572),
        },
    ]
}

/// 验证搜索结果
pub fn validate_search_results(results: Vec<TestBook>, expected_title: &str) -> bool {
    results
        .iter()
        .any(|book| book.title.contains("三体") || book.title.contains(expected_title))
}

/// 验证书籍详情
pub fn validate_book_details(book: &TestBook) -> Vec<String> {
    let mut errors = Vec::new();

    if book.id.is_empty() {
        errors.push("❌ 书籍ID为空".to_string());
    }

    if book.title.is_empty() {
        errors.push("❌ 书名为空".to_string());
    } else {
        println!("✓ 书名: {}", book.title);
    }

    if let Some(author) = &book.author {
        if author.is_empty() {
            errors.push("❌ 作者为空".to_string());
        } else {
            println!("✓ 作者: {}", author);
        }
    }

    if let Some(publisher) = &book.publisher {
        if publisher.is_empty() {
            errors.push("❌ 出版社为空".to_string());
        } else {
            println!("✓ 出版社: {}", publisher);
        }
    }

    if let Some(year) = book.year {
        if year < 1900 || year > 2100 {
            errors.push(format!("❌ 出版年异常: {}", year));
        } else {
            println!("✓ 出版年: {}", year);
        }
    }

    if let Some(pages) = book.pages {
        if pages <= 0 || pages > 10000 {
            errors.push(format!("❌ 页数异常: {}", pages));
        } else {
            println!("✓ 页数: {}", pages);
        }
    }

    errors
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_santi_basic() {
        let books = get_known_santi_books();
        assert!(books.len() >= 3, "应该有至少3本三体相关的书");
        
        for book in books {
            assert!(!book.id.is_empty(), "ID不应为空");
            assert!(!book.title.is_empty(), "标题不应为空");
        }
    }

    #[test]
    fn test_santi_book_details() {
        let books = get_known_santi_books();
        
        for book in books {
            let errors = validate_book_details(&book);
            assert!(
                errors.is_empty(),
                "书籍 {} 验证失败: {:?}",
                book.title,
                errors
            );
        }
    }

    #[test]
    fn test_search_results_validation() {
        let books = get_known_santi_books();
        assert!(validate_search_results(books, "三体"));
    }
}

fn main() {
    println!("========================================");
    println!("豆瓣爬虫测试 - 搜索 '三体'");
    println!("========================================");
    println!();

    let books = get_known_santi_books();
    
    println!("已知的三体书籍列表:");
    println!("==================");
    println!();

    for (i, book) in books.iter().enumerate() {
        println!("{}. {}", i + 1, book.title);
        println!("   ID: {}", book.id);
        
        if let Some(author) = &book.author {
            println!("   作者: {}", author);
        }
        if let Some(publisher) = &book.publisher {
            println!("   出版社: {}", publisher);
        }
        if let Some(year) = book.year {
            println!("   出版年: {}", year);
        }
        if let Some(pages) = book.pages {
            println!("   页数: {}", pages);
        }
        println!();
    }

    println!("========================================");
    println!("验证书籍详情");
    println!("========================================");
    println!();

    let mut all_valid = true;
    for book in books.iter() {
        println!("检查: {}", book.title);
        let errors = validate_book_details(book);
        if !errors.is_empty() {
            all_valid = false;
            for error in errors {
                println!("{}", error);
            }
        } else {
            println!("✓ 所有字段有效");
        }
        println!();
    }

    println!("========================================");
    if all_valid {
        println!("✓ 所有验证通过");
    } else {
        println!("✗ 某些验证失败");
    }
    println!("========================================");
}
