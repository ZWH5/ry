use anyhow::Result;
use async_trait::async_trait;
use chrono::Datelike;
use common_models::{EntityAssets, PersonSourceSpecifics, SearchDetails};
use common_utils::{PAGE_SIZE, compute_next_page, ryot_log};
use convert_case::{Case, Casing};
use dependent_models::MetadataSearchSourceSpecifics;
use dependent_models::{PersonDetails, SearchResults};
use enum_models::MediaSource;
use itertools::Itertools;
use media_models::{
    BookSpecifics, MetadataDetails, MetadataSearchItem, PartialMetadataPerson, PeopleSearchItem,
};
use traits::MediaProvider;

use crate::{
    client::URL,
    models::{
        AuthorLibrarySearchResponse, BookSearchItem, BookSearchResults, Description,
        EditionsResponse, MediaLibrarySearchResponse, MetadataDetailsAuthorResponse,
        MetadataDetailsBook, OpenlibraryService, PersonDetailsAuthor, DoubanBook, DoubanSearchResult,
    },
    utilities::{get_key, parse_date, parse_date_flexible},
};

#[async_trait]
impl MediaProvider for OpenlibraryService {
    async fn people_search(
        &self,
        page: u64,
        query: &str,
        _display_nsfw: bool,
        _source_specifics: &Option<PersonSourceSpecifics>,
    ) -> Result<SearchResults<PeopleSearchItem>> {
        let rsp = self
            .client
            .get(format!("{URL}/book/search?q=author:{}&count={}&start={}", 
                query, PAGE_SIZE, (page.saturating_sub(1) * PAGE_SIZE)))
            .send()
            .await?;
        let search: DoubanSearchResult = rsp.json().await?;
        let resp = search
            .books
            .into_iter()
            .filter_map(|book| {
                book.author.as_ref().and_then(|authors| {
                    authors.first().map(|author_name| PeopleSearchItem {
                        name: author_name.clone(),
                        identifier: author_name.clone(),
                        birth_year: None,
                        ..Default::default()
                    })
                })
            })
            .unique_by(|p| p.identifier.clone())
            .collect_vec();
        let data = SearchResults {
            items: resp,
            details: SearchDetails {
                total_items: search.total,
                next_page: compute_next_page(page, PAGE_SIZE, search.total),
            },
        };
        Ok(data)
    }

    async fn person_details(
        &self,
        identifier: &str,
        _source_specifics: &Option<PersonSourceSpecifics>,
    ) -> Result<PersonDetails> {
        let rsp = self
            .client
            .get(format!("{URL}/book/search?q=author:{}&count=1", identifier))
            .send()
            .await?;
        let data: DoubanSearchResult = rsp.json().await?;
        let book = data.books.first().ok_or(anyhow::anyhow!("Author not found"))?;
        
        Ok(PersonDetails {
            death_date: None,
            birth_date: None,
            description: book.summary.clone(),
            name: identifier.to_string(),
            source_url: Some(format!("https://book.douban.com/author/{}/", identifier)),
            assets: EntityAssets {
                remote_images: book.image.as_ref().map(|img| vec![img.clone()]).unwrap_or_default(),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn metadata_details(&self, identifier: &str) -> Result<MetadataDetails> {
        let rsp = self
            .client
            .get(format!("{URL}/book/{}", identifier))
            .send()
            .await?;
        let book_data: DoubanBook = rsp.json().await?;
        ryot_log!(debug, "Douban book response: {:?}", book_data);

        let mut people = vec![];
        for author_name in book_data.author.iter().flatten() {
            people.push(PartialMetadataPerson {
                role: "Author".to_owned(),
                identifier: author_name.clone(),
                source: MediaSource::Openlibrary,
                ..Default::default()
            });
        }

        let genres = book_data
            .tags
            .iter()
            .flatten()
            .map(|tag| tag.name.to_case(Case::Title))
            .collect_vec();

        let publish_year = book_data.pubdate.as_ref().and_then(|date| {
            parse_date_flexible(date).map(|d| d.year())
        });

        let remote_images = book_data.image.as_ref().map(|img| vec![img.clone()]).unwrap_or_default();

        Ok(MetadataDetails {
            people,
            genres,
            description: book_data.summary,
            title: book_data.title.clone(),
            publish_year,
            source_url: Some(format!("https://book.douban.com/subject/{}/", identifier)),
            book_specifics: Some(BookSpecifics {
                pages: book_data.pages,
                ..Default::default()
            }),
            assets: EntityAssets {
                remote_images,
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn metadata_search(
        &self,
        page: u64,
        query: &str,
        _display_nsfw: bool,
        _source_specifics: &Option<MetadataSearchSourceSpecifics>,
    ) -> Result<SearchResults<MetadataSearchItem>> {
        let rsp = self
            .client
            .get(format!("{URL}/book/search?q={}&count={}&start={}", 
                query, PAGE_SIZE, (page.saturating_sub(1) * PAGE_SIZE)))
            .send()
            .await?;
        let search: DoubanSearchResult = rsp.json().await?;
        let resp = search
            .books
            .iter()
            .map(|book| {
                let images = book.image.as_ref().map(|img| img.clone()).into_iter().collect_vec();
                BookSearchItem {
                    images,
                    title: book.title.clone(),
                    identifier: book.id.clone(),
                    publish_year: book.pubdate.as_ref().and_then(|date| {
                        parse_date_flexible(date).map(|d| d.year())
                    }),
                    author_names: book.author.clone().unwrap_or_default(),
                    ..Default::default()
                }
            })
            .collect_vec();
        let data = BookSearchResults {
            total: search.total,
            items: resp,
        };
        let next_page = compute_next_page(page, PAGE_SIZE, search.total);
        Ok(SearchResults {
            details: SearchDetails {
                next_page,
                total_items: data.total,
            },
            items: data
                .items
                .into_iter()
                .map(|b| MetadataSearchItem {
                    identifier: b.identifier,
                    title: b.title,
                    image: b.images.first().cloned(),
                    publish_year: b.publish_year,
                })
                .collect(),
        })
    }
}
