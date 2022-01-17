use lazy_static::lazy_static;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct AndroidApp {
    pub name: String,
    pub id: String,
    pub notes: String,
    pub delete: bool,
}

pub fn parse_lma_text(text: String) -> Vec<AndroidApp> {
    lazy_static! {
        static ref LMA_APP_RE: Regex = Regex::new(r"([^\n]+)\n\t([\S]+)").unwrap();
    }
    LMA_APP_RE.captures_iter(&text).map(|cap| AndroidApp {
        name: cap[1].to_string(),
        id: cap[2].to_string(),
        notes: String::new(),
        delete: false,
    }).collect()
}

pub async fn get_icon(id: String) -> Option<image::RgbaImage> {
    let url = format!("https://play.google.com/store/apps/details?id={}", id);
    let img_url = {
        let html = reqwest::get(url).await.ok()?.text().await.ok()?;
        let document = scraper::Html::parse_document(&*html);
        document.select(&scraper::Selector::parse(".T75of.sHb2Xb").ok()?).next()?
            .value().attr("src")?.to_string()
    };
    let bytes = reqwest::get(img_url).await.ok()?.bytes().await.ok()?;
    Some(image::load_from_memory(&*bytes).ok()?.to_rgba8())
}