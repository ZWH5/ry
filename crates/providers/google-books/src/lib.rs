use anyhow::{Result, anyhow};
use async_trait::async_trait;
use common_models::{EntityAssets, SearchDetails};
use common_utils::get_base_http_client;
use common_utils::{PAGE_SIZE, compute_next_page};
use convert_case::{Case, Casing};
use dependent_models::MetadataSearchSourceSpecifics;
use dependent_models::SearchResults;
use itertools::Itertools;
use media_models::{BookSpecifics, MetadataDetails, MetadataFreeCreator, MetadataSearchItem};
use reqwest::Client;
use scraper::{Html, Selector, element_ref::ElementRef};
use serde::{Deserialize, Serialize};
use traits::MediaProvider;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct GoogleBooksService {
    client: Client,
}

impl GoogleBooksService {
    pub async fn new(_config: &config_definition::GoogleBooksConfig) -> Result<Self> {
        let client = get_base_http_client(None);
        Ok(Self { client })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DoubanBook {
    pub id: String,
    pub title: String,
    pub image: Option<String>,
    pub author: Option<Vec<String>>,
    pub publisher: Option<String>,
    pub pubdate: Option<String>,
    pub pages: Option<i32>,
    pub summary: Option<String>,
}

fn parse_date_to_year(date_str: &str) -> Option<i32> {
    if date_str.len() >= 4 {
        date_str[..4].parse().ok()
    } else {
        None
    }
}

#[async_trait]
impl MediaProvider for GoogleBooksService {
    async fn metadata_details(&self, identifier: &str) -> Result<MetadataDetails> {
        let url = format!("https://book.douban.com/subject/{}/", identifier);
        let html = self.fetch_html(&url).await?;
        self.parse_book_details(&html, identifier)
    }

    async fn metadata_search(
        &self,
        page: u64,
        query: &str,
        _display_nsfw: bool,
        _source_specifics: &Option<MetadataSearchSourceSpecifics>,
    ) -> Result<SearchResults<MetadataSearchItem>> {
        let start = (page.saturating_sub(1) * PAGE_SIZE) as u32;
        let url = format!(
            "https://search.douban.com/book/subject_search?search_text={}&start={}",
            urlencoding::encode(query),
            start
        );
        
        let html = self.fetch_html(&url).await?;
        let (books, total) = self.parse_search_results(&html)?;
        
        let resp = books
            .into_iter()
            .map(|b| MetadataSearchItem {
                title: b.title,
                image: b.image,
                publish_year: b.pubdate.as_ref().and_then(|d| parse_date_to_year(d)),
                identifier: b.id,
            })
            .collect();

        let next_page = compute_next_page(page, PAGE_SIZE, total);
        Ok(SearchResults {
            items: resp,
            details: SearchDetails {
                next_page,
                total_items: total,
            },
        })
    }
}

impl GoogleBooksService {
    async fn fetch_html(&self, url: &str) -> Result<String> {
        let resp = self
            .client
            .get(url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .send()
            .await?;
        Ok(resp.text().await?)
    }

    fn parse_search_results(&self, html_content: &str) -> Result<(Vec<DoubanBook>, u64)> {
        let document = Html::parse_document(html_content);
        
        let mut books = Vec::new();
        
        // 选择搜索结果容器 - 与Python版本保持一致
        if let Ok(item_selector) = Selector::parse("a.nbg") {
            for item in document.select(&item_selector) {
                if let Some(href) = item.value().attr("href") {
                    // 从URL中提取ID和完整书籍链接
                    if let Some(book_id) = self.extract_book_id(href) {
                        let mut book = DoubanBook {
                            id: book_id,
                            title: String::new(),
                            image: None,
                            author: None,
                            publisher: None,
                            pubdate: None,
                            pages: None,
                            summary: None,
                        };

                        // 提取书名 - 从image的alt或title
                        if let Some(img) = item.select(&Selector::parse("img").unwrap()).next() {
                            if let Some(alt) = img.value().attr("alt") {
                                book.title = alt.trim().to_string();
                            }
                            if let Some(src) = img.value().attr("src") {
                                book.image = Some(src.to_string());
                            }
                        }

                        if !book.title.is_empty() {
                            books.push(book);
                        }
                    }
                }
            }
        }

        let count = books.len() as u64;
        Ok((books, count))
    }

    fn parse_book_details(&self, html_content: &str, identifier: &str) -> Result<MetadataDetails> {
        let document = Html::parse_document(html_content);
        
        let mut book = DoubanBook {
            id: identifier.to_string(),
            title: String::new(),
            image: None,
            author: None,
            publisher: None,
            pubdate: None,
            pages: None,
            summary: None,
        };

        // 提取书名 - 与Python版本一致
        if let Ok(selector) = Selector::parse("span[property='v:itemreviewed']") {
            if let Some(elem) = document.select(&selector).next() {
                book.title = elem.inner_html().trim().to_string();
            }
        }

        // 如果上面的方法失败，尝试 h1 span
        if book.title.is_empty() {
            if let Ok(selector) = Selector::parse("h1 span") {
                if let Some(elem) = document.select(&selector).next() {
                    book.title = elem.inner_html().trim().to_string();
                }
            }
        }

        // 提取封面图片 - 与Python版本保持一致（a.nbg > img）
        if let Ok(selector) = Selector::parse("a.nbg") {
            if let Some(elem) = document.select(&selector).next() {
                if let Ok(img_selector) = Selector::parse("img") {
                    if let Some(img) = elem.select(&img_selector).next() {
                        if let Some(src) = img.value().attr("src") {
                            book.image = Some(src.to_string());
                        }
                    }
                }
            }
        }

        // 提取详细信息 - 与Python版本保持一致
        self.parse_book_info(&document, &mut book);

        // 提取书籍简介
        if let Ok(selector) = Selector::parse("div#link-report div.intro") {
            let intros: Vec<_> = document.select(&selector).collect();
            if let Some(intro) = intros.last() {
                let summary = intro.inner_html();
                let decoded = html_escape::decode_html_entities(&summary).to_string();
                if !decoded.is_empty() && decoded.len() > 10 {
                    book.summary = Some(decoded);
                }
            }
        }

        Ok(self.douban_book_to_metadata_details(book, identifier.to_string()))
    }

    fn parse_book_info(&self, document: &Html, book: &mut DoubanBook) {
        if let Ok(selector) = Selector::parse("span.pl") {
            for element in document.select(&selector) {
                let text = self.get_element_text(&element);
                
                if text.starts_with("作者") {
                    // 从父元素中提取作者链接
                    if let Some(parent) = element.parent() {
                        if let Ok(link_sel) = Selector::parse("a") {
                            let authors: Vec<String> = parent
                                .select(&link_sel)
                                .filter_map(|a| {
                                    let href = a.value().attr("href").unwrap_or("");
                                    if href.contains("/author") || href.contains("/search") {
                                        Some(self.get_element_text(&a))
                                    } else {
                                        None
                                    }
                                })
                                .filter(|a| !a.is_empty() && a.len() < 100)
                                .collect();
                            if !authors.is_empty() {
                                book.author = Some(authors);
                            }
                        }
                    }
                } else if text.starts_with("出版社") {
                    if let Some(publisher) = self.get_tail_text(&element) {
                        book.publisher = Some(publisher);
                    }
                } else if text.starts_with("副标题") {
                    if let Some(subtitle) = self.get_tail_text(&element) {
                        book.title = format!("{}:{}", book.title, subtitle);
                    }
                } else if text.starts_with("出版年") {
                    if let Some(pubdate) = self.get_tail_text(&element) {
                        book.pubdate = Some(pubdate);
                    }
                } else if text.starts_with("页数") {
                    if let Some(pages_str) = self.get_tail_text(&element) {
                        if let Ok(pages) = pages_str.trim().parse::<i32>() {
                            book.pages = Some(pages);
                        }
                    }
                } else if text.starts_with("ISBN") {
                    // ISBN处理可以在这里添加（如果需要）
                    // book.isbn = Some(self.get_tail_text(&element).unwrap_or_default());
                }
            }
        }
    }

    fn get_element_text(&self, element: &ElementRef) -> String {
        element
            .text()
            .map(|t| t.trim())
            .filter(|t| !t.is_empty())
            .collect::<Vec<_>>()
            .join("")
    }

    fn get_tail_text(&self, element: &ElementRef) -> Option<String> {
        // 获取元素后面的文本内容（跳过标签）
        if let Some(mut next_sibling) = element.next_sibling() {
            loop {
                match next_sibling.value() {
                    scraper::node::Node::Text(text) => {
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() && trimmed != "|" {
                            return Some(trimmed);
                        }
                    }
                    scraper::node::Node::Element(_) => {
                        // 遇到元素，尝试获取其文本
                        if let Some(elem_ref) = ElementRef::wrap(next_sibling.clone()) {
                            let text = elem_ref.text().next()?.trim().to_string();
                            if !text.is_empty() {
                                return Some(text);
                            }
                        }
                        break;
                    }
                    _ => {}
                }
                next_sibling = next_sibling.next_sibling()?;
            }
        }
        None
    }

    fn extract_book_id(&self, href: &str) -> Option<String> {
        // 从豆瓣书籍URL中提取ID
        // 格式: /subject/1234567/ 或 /subject/1234567
        if let Some(start) = href.find("/subject/") {
            let after_subject = &href[start + 9..];
            if let Some(end) = after_subject.find('/') {
                return Some(after_subject[..end].to_string());
            } else {
                // URL末尾没有/
                return Some(after_subject.to_string());
            }
        }
        None
    }

    fn douban_book_to_metadata_details(
        &self,
        book: DoubanBook,
        identifier: String,
    ) -> MetadataDetails {
        let remote_images = book.image.as_ref().map(|img| vec![img.clone()]).unwrap_or_default();
        
        let mut creators = book
            .author
            .unwrap_or_default()
            .into_iter()
            .map(|a| MetadataFreeCreator {
                name: a,
                role: "Author".to_owned(),
            })
            .collect_vec();
        
        if let Some(p) = book.publisher {
            creators.push(MetadataFreeCreator {
                name: p,
                role: "Publisher".to_owned(),
            });
        }

        let assets = EntityAssets {
            remote_images,
            ..Default::default()
        };

        MetadataDetails {
            assets,
            title: book.title.clone(),
            description: book.summary,
            genres: vec![],
            creators,
            publish_year: book.pubdate.as_ref().and_then(|d| parse_date_to_year(d)),
            book_specifics: Some(BookSpecifics {
                pages: book.pages,
                ..Default::default()
            }),
            source_url: Some(format!(
                "https://book.douban.com/subject/{}/",
                identifier
            )),
            ..Default::default()
        }
    }

    /// Get a book's ID from its ISBN
    pub async fn id_from_isbn(&self, isbn: &str) -> Option<String> {
        let url = format!("https://search.douban.com/book/subject_search?search_text={}", isbn);
        let html = self.fetch_html(&url).await.ok()?;
        let (books, _) = self.parse_search_results(&html).ok()?;
        Some(books.first()?.id.clone())
    }
}
