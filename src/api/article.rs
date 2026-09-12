use serde::Deserialize;
use std::path::PathBuf;

#[derive(Deserialize)]
struct WikiParseText {
    #[serde(rename = "*")]
    html: Option<String>,
}

#[derive(Deserialize)]
struct WikiParseCategoriesHtml {
    #[serde(rename = "*")]
    html: Option<String>,
}

#[derive(Deserialize)]
struct WikiParseObject {
    text: Option<WikiParseText>,
    categorieshtml: Option<WikiParseCategoriesHtml>,
}

#[derive(Deserialize)]
struct WikiError {
    info: Option<String>,
}

#[derive(Deserialize)]
struct WikiParseResponse {
    parse: Option<WikiParseObject>,
    error: Option<WikiError>,
}

pub fn cache_dir() -> PathBuf {
    crate::paths::cache_dir().join("articles")
}

fn fnv1a_hash(s: &str) -> u64 {
    s.bytes().fold(0xcbf29ce484222325, |h, b| {
        (h ^ (b as u64)).wrapping_mul(0x100000001b3)
    })
}

pub fn normalize_title(title: &str) -> String {
    crate::parser::url_decode(title)
        .replace('_', " ")
        .trim()
        .to_string()
}

pub fn cache_file_path(title: &str) -> PathBuf {
    let normalized = normalize_title(title);
    let safe_name: String = normalized
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect();

    let hash = fnv1a_hash(&normalized);
    cache_dir().join(format!("{}_{:016x}.html", safe_name, hash))
}

pub fn get_cached_article(title: &str, lifetime_hours: u64) -> Option<String> {
    let path = cache_file_path(title);
    if let Ok(metadata) = std::fs::metadata(&path)
        && let Ok(modified) = metadata.modified()
            && let Ok(elapsed) = modified.elapsed() {
                if elapsed.as_secs() <= lifetime_hours.saturating_mul(3600) {
                    if let Ok(content) = std::fs::read_to_string(&path)
                        && content.contains("catlinks") {
                            return Some(content);
                        }
                } else {
                    let _ = std::fs::remove_file(&path);
                }
            }
    None
}

pub fn save_cached_article(title: &str, html: &str) {
    let dir = cache_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = cache_file_path(title);
    let _ = std::fs::write(&path, html);
}

pub fn fetch_article_wikipedia(
    agent: &ureq::Agent,
    title: &str,
    timeout_secs: u64,
    offline_cache: bool,
    cache_lifetime: u64,
) -> Result<String, super::ApiError> {
    let normalized_title = normalize_title(title);
    if offline_cache
        && let Some(cached_html) = get_cached_article(&normalized_title, cache_lifetime) {
            return Ok(cached_html);
        }

    let url = "https://en.wikipedia.org/w/api.php";
    let res = agent
        .get(url)
        .timeout(std::time::Duration::from_secs(timeout_secs.max(1)))
        .query("action", "parse")
        .query("page", &normalized_title)
        .query("prop", "text|categorieshtml")
        .query("format", "json")
        .query("disableeditsection", "1")
        .query("disabletoc", "1")
        .query("redirects", "1")
        .call();

    match res {
        Ok(response) => {
            let parse_resp: WikiParseResponse = response
                .into_json()
                .map_err(|e| super::ApiError::Parse(e.to_string()))?;

            if let Some(err) = parse_resp.error
                && let Some(info) = err.info {
                    return Err(super::ApiError::Wikipedia(info));
                }

            let parse_obj = parse_resp.parse;
            let body_html = parse_obj
                .as_ref()
                .and_then(|p| p.text.as_ref())
                .and_then(|t| t.html.as_deref())
                .unwrap_or("");
            let cat_html = parse_obj
                .as_ref()
                .and_then(|p| p.categorieshtml.as_ref())
                .and_then(|c| c.html.as_deref())
                .unwrap_or("");

            if body_html.trim().is_empty() {
                return Err(super::ApiError::NotFound(
                    "article HTML content not found".to_string(),
                ));
            }

            let combined_html = if cat_html.trim().is_empty() {
                body_html.to_string()
            } else {
                format!("{}\n{}", body_html, cat_html)
            };

            if offline_cache {
                save_cached_article(&normalized_title, &combined_html);
            }
            Ok(combined_html)
        }
        Err(err) => {
            if offline_cache {
                let path = cache_file_path(&normalized_title);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    return Ok(content);
                }
            }
            Err(super::ApiError::Network(err.to_string()))
        }
    }
}
