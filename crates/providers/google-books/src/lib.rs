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
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use traits::MediaProvider;

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
        
        // 选择搜索结果容器
        let item_selector = Selector::parse("div.subject-item").map_err(|_| anyhow!("Failed to parse selector"))?;
        let title_selector = Selector::parse("h2 a").map_err(|_| anyhow!("Failed to parse title selector"))?;
        let image_selector = Selector::parse("img.cover").map_err(|_| anyhow!("Failed to parse image selector"))?;
        let info_selector = Selector::parse("p.info").map_err(|_| anyhow!("Failed to parse info selector"))?;
        
        let mut books = Vec::new();
        let mut count = 0;
        
        for item in document.select(&item_selector) {
            let mut book = DoubanBook {
                id: String::new(),
                title: String::new(),
                image: None,
                author: None,
                publisher: None,
                pubdate: None,
                pages: None,
                summary: None,
            };

            // 提取书名和ID
            if let Some(title_elem) = item.select(&title_selector).next() {
                if let Some(text) = title_elem.inner_html().lines().next() {
                    book.title = text.trim().to_string();
                }
                if let Some(href) = title_elem.value().attr("href") {
                    // 从URL中提取ID: /subject/1007241/
                    if let Some(id) = href.split('/').nth(2) {
                        book.id = id.to_string();
                    }
                }
            }

            // 提取图片
            if let Some(img_elem) = item.select(&image_selector).next() {
                if let Some(src) = img_elem.value().attr("src") {
                    book.image = Some(src.to_string());
                }
            }

            // 提取作者和出版信息
            if let Some(info_elem) = item.select(&info_selector).next() {
                let info_text = info_elem.inner_html();
                // 简单的信息提取
                let lines: Vec<&str> = info_text.split("<br>").collect();
                if let Some(first_line) = lines.first() {
                    // 第一行通常是作者信息
                    let author_str = html_escape::decode_html_entities(first_line).to_string();
                    if !author_str.is_empty() {
                        book.author = Some(vec![author_str]);
                    }
                }
            }

            if !book.id.is_empty() && !book.title.is_empty() {
                books.push(book);
                count += 1;
            }
        }

        Ok((books, count as u64))
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

        // 提取书名
        let title_selector = Selector::parse("h1 span").map_err(|_| anyhow!("Failed to parse selector"))?;
        if let Some(elem) = document.select(&title_selector).next() {
            book.title = elem.inner_html().trim().to_string();
        }

        // 提取封面图片
        let image_selector = Selector::parse("a.nbg img").map_err(|_| anyhow!("Failed to parse image selector"))?;
        if let Some(elem) = document.select(&image_selector).next() {
            if let Some(src) = elem.value().attr("src") {
                book.image = Some(src.to_string());
            }
        }

        // 提取详细信息 (作者、出版社、出版日期等)
        let info_selector = Selector::parse("div#info").map_err(|_| anyhow!("Failed to parse info selector"))?;
        if let Some(info_elem) = document.select(&info_selector).next() {
            let info_text = info_elem.inner_html();
            
            // 解析作者
            if let Some(author_start) = info_text.find("作者</span>") {
                if let Some(author_end) = info_text[author_start..].find("</a>") {
                    let author_section = &info_text[author_start..author_start + author_end + 4];
                    let author_names: Vec<String> = author_section
                        .split("</a>")
                        .filter_map(|part| {
                            if let Some(start) = part.rfind('>') {
                                let name = &part[start + 1..];
                                if !name.is_empty() && name.len() < 100 {
                                    return Some(html_escape::decode_html_entities(name).to_string());
                                }
                            }
                            None
                        })
                        .collect();
                    if !author_names.is_empty() {
                        book.author = Some(author_names);
                    }
                }
            }

            // 解析出版社
            if let Some(pub_start) = info_text.find("出版社</span>") {
                if let Some(pub_end) = info_text[pub_start..].find("</a>") {
                    let pub_section = &info_text[pub_start..pub_start + pub_end];
                    if let Some(start) = pub_section.rfind('>') {
                        let publisher = &pub_section[start + 1..];
                        book.publisher = Some(html_escape::decode_html_entities(publisher).to_string());
                    }
                }
            }

            // 解析出版日期
            if let Some(date_start) = info_text.find("出版年</span>") {
                if let Some(date_end) = info_text[date_start..].find("</") {
                    let date_section = &info_text[date_start + 12..date_start + date_end];
                    if let Some(start) = date_section.rfind('>') {
                        let pubdate = &date_section[start + 1..];
                        book.pubdate = Some(pubdate.trim().to_string());
                    }
                }
            }

            // 解析页数
            if let Some(page_start) = info_text.find("页数</span>") {
                if let Some(page_end) = info_text[page_start..].find("</") {
                    let page_section = &info_text[page_start + 10..page_start + page_end];
                    if let Some(start) = page_section.rfind('>') {
                        let pages_str = &page_section[start + 1..];
                        if let Ok(pages) = pages_str.trim().parse::<i32>() {
                            book.pages = Some(pages);
                        }
                    }
                }
            }
        }

        // 提取书籍简介
        let summary_selector = Selector::parse("div.intro p").map_err(|_| anyhow!("Failed to parse summary selector"))?;
        if let Some(elem) = document.select(&summary_selector).next() {
            let summary = elem.inner_html();
            let decoded = html_escape::decode_html_entities(&summary).to_string();
            if !decoded.is_empty() {
                book.summary = Some(decoded);
            }
        }

        Ok(self.douban_book_to_metadata_details(book, identifier.to_string()))
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
