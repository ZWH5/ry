use anyhow::Result;
use common_utils::get_base_http_client;

use crate::{
    models::{MetadataDetailsBook, OpenlibraryService},
    utilities::get_key,
};

pub static URL: &str = "https://api.douban.com";
pub static IMAGE_BASE_URL: &str = "https://img1.doubanio.com";

impl OpenlibraryService {
    pub async fn new(config: &config_definition::OpenlibraryConfig) -> Result<Self> {
        let client = get_base_http_client(None);
        Ok(Self {
            client,
            image_url: IMAGE_BASE_URL.to_owned(),
            image_size: config.cover_image_size.to_string(),
        })
    }

    pub fn get_book_cover_image_url(&self, image_url: &str) -> String {
        image_url.to_string()
    }

    pub fn get_author_cover_image_url(&self, image_url: &str) -> String {
        image_url.to_string()
    }

    pub fn get_cover_image_url(&self, _t: &str, _c: i64) -> String {
        String::new()
    }

    pub async fn id_from_isbn(&self, isbn: &str) -> Option<String> {
        self.client
            .get(format!("{URL}/book/search?q={isbn}&count=1"))
            .send()
            .await
            .ok()?
            .json::<MetadataDetailsBook>()
            .await
            .ok()
            .and_then(|data| data.books.and_then(|mut books| books.pop().map(|b| b.id)))
    }
}
