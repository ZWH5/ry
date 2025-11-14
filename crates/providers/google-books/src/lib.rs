use anyhow::Result;
use async_trait::async_trait;
use common_models::{EntityAssets, SearchDetails};
use common_utils::get_base_http_client;
use common_utils::{PAGE_SIZE, compute_next_page, convert_date_to_year};
use convert_case::{Case, Casing};
use dependent_models::MetadataSearchSourceSpecifics;
use dependent_models::SearchResults;
use itertools::Itertools;
use media_models::{BookSpecifics, MetadataDetails, MetadataFreeCreator, MetadataSearchItem};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use traits::MediaProvider;

static URL: &str = "https://api.douban.com";

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
    #[serde(default)]
    pub tags: Option<Vec<DoubanTag>>,
    pub summary: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DoubanTag {
    pub name: String,
    pub count: Option<u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DoubanSearchResult {
    pub books: Vec<DoubanBook>,
    pub total: u64,
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
        let rsp = self
            .client
            .get(format!("{URL}/book/{identifier}"))
            .send()
            .await?;
        let book_data: DoubanBook = rsp.json().await?;
        Ok(self.douban_book_to_metadata_details(book_data, identifier.to_string()))
    }

    async fn metadata_search(
        &self,
        page: u64,
        query: &str,
        _display_nsfw: bool,
        _source_specifics: &Option<MetadataSearchSourceSpecifics>,
    ) -> Result<SearchResults<MetadataSearchItem>> {
        let start = (page.saturating_sub(1) * PAGE_SIZE) as u32;
        let rsp = self
            .client
            .get(format!("{URL}/book/search"))
            .query(&[
                ("q", query),
                ("count", &PAGE_SIZE.to_string()),
                ("start", &start.to_string()),
            ])
            .send()
            .await?;
        let search: DoubanSearchResult = rsp.json().await?;
        let resp = search
            .books
            .into_iter()
            .map(|b| {
                let image = b.image.clone();
                let publish_year = b.pubdate.as_ref().and_then(|d| parse_date_to_year(d));
                MetadataSearchItem {
                    title: b.title,
                    image,
                    publish_year,
                    identifier: b.id,
                }
            })
            .collect();
        let next_page = compute_next_page(page, PAGE_SIZE, search.total);
        Ok(SearchResults {
            items: resp,
            details: SearchDetails {
                next_page,
                total_items: search.total,
            },
        })
    }
}

impl GoogleBooksService {
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

        let genres = book
            .tags
            .unwrap_or_default()
            .into_iter()
            .map(|tag| tag.name.to_case(Case::Title))
            .unique()
            .collect();

        let assets = EntityAssets {
            remote_images,
            ..Default::default()
        };

        MetadataDetails {
            assets,
            title: book.title.clone(),
            description: book.summary,
            genres,
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
        let resp = self
            .client
            .get(format!("{URL}/book/search"))
            .query(&[("q", isbn)])
            .send()
            .await
            .ok()?;
        let search: DoubanSearchResult = resp.json().await.ok()?;
        Some(search.books.first()?.id.clone())
    }
}
