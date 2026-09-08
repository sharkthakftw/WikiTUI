pub mod article;
pub mod category;
pub mod daily_feed;
pub mod feed;
pub mod images;
pub mod random;
pub mod search;
pub mod shorten;
pub mod stats;
pub mod summary;
pub mod updates;

pub use daily_feed::DailyFeed;
pub use stats::WikiStatistics;

use crate::feed::algorithm::FeedItem;
use std::sync::mpsc::{Receiver, Sender};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiError {
    Network(String),
    Parse(String),
    Wikipedia(String),
    NotFound(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::Network(msg) => write!(f, "network error: {}", msg),
            ApiError::Parse(msg) => write!(f, "parse error: {}", msg),
            ApiError::Wikipedia(msg) => write!(f, "Wikipedia error: {}", msg),
            ApiError::NotFound(msg) => write!(f, "not found: {}", msg),
        }
    }
}

impl std::error::Error for ApiError {}

impl From<ureq::Error> for ApiError {
    fn from(err: ureq::Error) -> Self {
        ApiError::Network(err.to_string())
    }
}

pub(crate) fn send_request_json<T: serde::de::DeserializeOwned>(
    req: ureq::Request,
    timeout_secs: u64,
) -> Result<T, ApiError> {
    let resp = req
        .timeout(std::time::Duration::from_secs(timeout_secs.max(1)))
        .call()
        .map_err(|e| ApiError::Network(e.to_string()))?;
    resp.into_json().map_err(|e| ApiError::Parse(e.to_string()))
}

#[derive(Debug, Clone)]
pub struct SearchResultItem {
    pub title: String,
    pub snippet: String,
    pub has_audio: bool,
}

pub enum NetworkCommand {
    Search {
        request_id: u64,
        pane_id: usize,
        query: String,
        limit: usize,
        timeout: u64,
    },
    FetchArticle {
        request_id: u64,
        pane_id: usize,
        title: String,
        timeout: u64,
        offline_cache: bool,
        cache_lifetime: u64,
    },
    FetchRandomArticle {
        request_id: u64,
        pane_id: usize,
        timeout: u64,
        offline_cache: bool,
        cache_lifetime: u64,
    },
    FetchFeedBatch {
        timeout: u64,
    },
    FetchDailyFeed {
        timeout: u64,
        offline_cache: bool,
    },
    FetchStats {
        timeout: u64,
    },
    CheckForUpdates {
        timeout: u64,
    },
    FetchImage {
        url: String,
        timeout: u64,
    },
    FetchCategoryMembers {
        category: String,
        limit: usize,
        timeout: u64,
    },
    DecodeHalfblockImage {
        url: String,
        path: std::path::PathBuf,
        cols: usize,
        rows: usize,
        filter: crate::config::HalfblockFilter,
    },
    PredecodeKittyImage {
        path: std::path::PathBuf,
    },
    ShortenUrl {
        url: String,
        timeout: u64,
    },
    FetchSummary {
        title: String,
        timeout: u64,
    },
}

#[derive(Debug)]
pub enum NetworkEvent {
    SearchResult {
        request_id: u64,
        pane_id: usize,
        query: String,
        results: Vec<SearchResultItem>,
    },
    ArticleResult {
        request_id: u64,
        pane_id: usize,
        title: String,
        content: String,
    },
    FeedBatchLoaded {
        items: Vec<FeedItem>,
    },
    DailyFeedLoaded(Box<DailyFeed>),
    StatsLoaded(WikiStatistics),
    UpdateCheckResult {
        latest_tag: Result<String, String>,
    },
    ImageLoaded {
        url: String,
        path: std::path::PathBuf,
    },
    HalfblockImageDecoded {
        url: String,
        cols: usize,
        rows: usize,
        lines: Vec<ratatui::text::Line<'static>>,
    },
    CategoryMembersLoaded {
        category: String,
        members: Vec<String>,
    },
    UrlShortened {
        original_url: String,
        short_url: String,
    },
    SummaryLoaded {
        title: String,
        description: Option<String>,
        extract: Option<String>,
    },
    Error {
        request_id: u64,
        pane_id: usize,
        error: ApiError,
    },
}

enum ImageTask {
    DecodeHalfblock {
        url: String,
        path: std::path::PathBuf,
        cols: usize,
        rows: usize,
        filter: crate::config::HalfblockFilter,
    },
    PredecodeKitty {
        path: std::path::PathBuf,
    },
}

pub fn run_worker(cmd_rx: Receiver<NetworkCommand>, ev_tx: Sender<NetworkEvent>) {
    let agent: std::sync::Arc<ureq::Agent> = std::sync::Arc::new(
        ureq::builder()
            .user_agent(concat!(
                "wikid/",
                env!("CARGO_PKG_VERSION"),
                " (https://github.com/sharkthakftw/wikid)"
            ))
            .build(),
    );

    let (img_tx, img_rx) = std::sync::mpsc::channel::<ImageTask>();
    let img_rx = std::sync::Arc::new(std::sync::Mutex::new(img_rx));
    let pool_size = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(2)
        .clamp(1, 4);

    for _ in 0..pool_size {
        let img_rx = img_rx.clone();
        let ev_tx = ev_tx.clone();
        std::thread::spawn(move || loop {
            let task = {
                let rx = match img_rx.lock() {
                    Ok(guard) => guard,
                    Err(_) => break,
                };
                match rx.recv() {
                    Ok(task) => task,
                    Err(_) => break,
                }
            };
            match task {
                ImageTask::DecodeHalfblock {
                    url,
                    path,
                    cols,
                    rows,
                    filter,
                } => {
                    if let Ok(bytes) = std::fs::read(&path) {
                        if let Some(lines) =
                            crate::graphics::halfblocks::render_halfblock_image_from_bytes(
                                &bytes, cols, rows, filter,
                            )
                        {
                            let _ = ev_tx.send(NetworkEvent::HalfblockImageDecoded {
                                url,
                                cols,
                                rows,
                                lines,
                            });
                        }
                    }
                }
                ImageTask::PredecodeKitty { path } => {
                    crate::graphics::kitty::predecode_kitty_image(&path);
                }
            }
        });
    }

    while let Ok(cmd) = cmd_rx.recv() {
        match cmd {
            NetworkCommand::DecodeHalfblockImage {
                url,
                path,
                cols,
                rows,
                filter,
            } => {
                let _ = img_tx.send(ImageTask::DecodeHalfblock {
                    url,
                    path,
                    cols,
                    rows,
                    filter,
                });
            }
            NetworkCommand::PredecodeKittyImage { path } => {
                let _ = img_tx.send(ImageTask::PredecodeKitty { path });
            }
            cmd => {
                let agent = agent.clone();
                let ev_tx = ev_tx.clone();

                std::thread::spawn(move || match cmd {
                    NetworkCommand::Search {
                        request_id,
                        pane_id,
                        query,
                        limit,
                        timeout,
                    } => match search::search_wikipedia(&agent, &query, limit, timeout) {
                        Ok(results) => {
                            let _ = ev_tx.send(NetworkEvent::SearchResult {
                                request_id,
                                pane_id,
                                query,
                                results,
                            });
                        }
                        Err(err) => {
                            let _ = ev_tx.send(NetworkEvent::Error {
                                request_id,
                                pane_id,
                                error: err,
                            });
                        }
                    },
                    NetworkCommand::FetchArticle {
                        request_id,
                        pane_id,
                        title,
                        timeout,
                        offline_cache,
                        cache_lifetime,
                    } => {
                        match article::fetch_article_wikipedia(
                            &agent,
                            &title,
                            timeout,
                            offline_cache,
                            cache_lifetime,
                        ) {
                            Ok(content) => {
                                let _ = ev_tx.send(NetworkEvent::ArticleResult {
                                    request_id,
                                    pane_id,
                                    title,
                                    content,
                                });
                            }
                            Err(err) => {
                                let _ = ev_tx.send(NetworkEvent::Error {
                                    request_id,
                                    pane_id,
                                    error: err,
                                });
                            }
                        }
                    }
                    NetworkCommand::FetchRandomArticle {
                        request_id,
                        pane_id,
                        timeout,
                        offline_cache,
                        cache_lifetime,
                    } => {
                        match random::fetch_random_article(
                            &agent,
                            timeout,
                            offline_cache,
                            cache_lifetime,
                        ) {
                            Ok((title, content)) => {
                                let _ = ev_tx.send(NetworkEvent::ArticleResult {
                                    request_id,
                                    pane_id,
                                    title,
                                    content,
                                });
                            }
                            Err(err) => {
                                let _ = ev_tx.send(NetworkEvent::Error {
                                    request_id,
                                    pane_id,
                                    error: err,
                                });
                            }
                        }
                    }
                    NetworkCommand::FetchFeedBatch { timeout } => {
                        if let Ok(items) = feed::fetch_feed_batch(&agent, timeout) {
                            let _ = ev_tx.send(NetworkEvent::FeedBatchLoaded { items });
                        }
                    }
                    NetworkCommand::FetchDailyFeed {
                        timeout,
                        offline_cache,
                    } => {
                        if let Ok(feed) =
                            daily_feed::fetch_daily_feed(&agent, timeout, offline_cache)
                        {
                            let _ = ev_tx.send(NetworkEvent::DailyFeedLoaded(Box::new(feed)));
                        }
                    }
                    NetworkCommand::FetchStats { timeout } => {
                        if let Ok(statistics) = stats::fetch_wiki_statistics(&agent, timeout) {
                            let _ = ev_tx.send(NetworkEvent::StatsLoaded(statistics));
                        }
                    }
                    NetworkCommand::CheckForUpdates { timeout } => {
                        let res = updates::check_latest_release(&agent, timeout)
                            .map_err(|e| e.to_string());
                        let _ = ev_tx.send(NetworkEvent::UpdateCheckResult { latest_tag: res });
                    }
                    NetworkCommand::FetchImage { url, timeout } => {
                        if let Ok(path) = images::fetch_and_cache_image(&agent, &url, timeout) {
                            let _ = ev_tx.send(NetworkEvent::ImageLoaded { url, path });
                        }
                    }
                    NetworkCommand::FetchCategoryMembers {
                        category,
                        limit,
                        timeout,
                    } => {
                        if let Ok(members) =
                            category::fetch_category_members(&agent, &category, limit, timeout)
                        {
                            let _ = ev_tx
                                .send(NetworkEvent::CategoryMembersLoaded { category, members });
                        }
                    }
                    NetworkCommand::ShortenUrl { url, timeout } => {
                        if let Ok(short_url) = shorten::shorten_url(&agent, &url, timeout) {
                            let _ = ev_tx.send(NetworkEvent::UrlShortened {
                                original_url: url,
                                short_url,
                            });
                        }
                    }
                    NetworkCommand::FetchSummary { title, timeout } => {
                        if let Ok((res_title, description, extract)) =
                            summary::fetch_summary(&agent, &title, timeout)
                        {
                            let _ = ev_tx.send(NetworkEvent::SummaryLoaded {
                                title: res_title,
                                description,
                                extract,
                            });
                        }
                    }
                    _ => {}
                });
            }
        }
    }
}
