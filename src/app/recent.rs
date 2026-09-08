use super::App;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RecentArticleEntry {
    pub title: String,
    pub timestamp: u64,
}

impl App {
    pub fn recent_articles_file_path() -> PathBuf {
        crate::paths::config_dir().join("recent_articles.json")
    }

    pub fn load_recent_articles() -> Vec<RecentArticleEntry> {
        let path = Self::recent_articles_file_path();
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        if let Ok(entries) = serde_json::from_str::<Vec<RecentArticleEntry>>(&content) {
            entries
                .into_iter()
                .map(|mut e| {
                    e.title = e.title.replace('_', " ");
                    e
                })
                .collect()
        } else if let Ok(strings) = serde_json::from_str::<Vec<String>>(&content) {
            strings
                .into_iter()
                .map(|t| RecentArticleEntry {
                    title: t.replace('_', " "),
                    timestamp: 0,
                })
                .collect()
        } else {
            Vec::new()
        }
    }

    pub fn save_recent_articles(&self) {
        let path = Self::recent_articles_file_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&self.user_data.recent_articles) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn record_recent_article(&mut self, title: &str) {
        let clean = title.trim().replace('_', " ");
        let lower = clean.to_lowercase();
        if clean.is_empty() || lower.starts_with("category:") || lower.starts_with("portal:") {
            return;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        self.user_data.recent_articles.retain(|e| e.title != clean);
        self.user_data.recent_articles.insert(
            0,
            RecentArticleEntry {
                title: clean,
                timestamp: now,
            },
        );
        if self.user_data.recent_articles.len() > 10 {
            self.user_data.recent_articles.truncate(10);
        }
        self.save_recent_articles();
    }

    pub fn get_continue_reading_articles(&self) -> Vec<String> {
        let filtered: Vec<String> = self
            .user_data
            .recent_articles
            .iter()
            .filter(|e| {
                let lower = e.title.to_lowercase();
                !lower.starts_with("category:") && !lower.starts_with("portal:")
            })
            .map(|e| e.title.clone())
            .collect();

        if !filtered.is_empty() {
            return filtered;
        }

        let mut seen = HashSet::new();
        let mut list = Vec::with_capacity(10);
        for l in &self.user_data.saved_lists.lists {
            for a in l.articles.iter().rev() {
                let lower = a.to_lowercase();
                if !lower.starts_with("category:")
                    && !lower.starts_with("portal:")
                    && seen.insert(a.as_str())
                {
                    list.push(a.clone());
                    if list.len() >= 10 {
                        return list;
                    }
                }
            }
        }
        list
    }

    pub fn get_continue_reading_with_timestamps(&self) -> Vec<(String, Option<u64>)> {
        let filtered: Vec<(String, Option<u64>)> = self
            .user_data
            .recent_articles
            .iter()
            .filter(|e| {
                let lower = e.title.to_lowercase();
                !lower.starts_with("category:") && !lower.starts_with("portal:")
            })
            .map(|e| {
                let ts = if e.timestamp > 0 {
                    Some(e.timestamp)
                } else {
                    None
                };
                (e.title.clone(), ts)
            })
            .collect();

        if !filtered.is_empty() {
            return filtered;
        }

        let mut seen = HashSet::new();
        let mut list = Vec::with_capacity(10);
        for l in &self.user_data.saved_lists.lists {
            for a in l.articles.iter().rev() {
                let lower = a.to_lowercase();
                if !lower.starts_with("category:")
                    && !lower.starts_with("portal:")
                    && seen.insert(a.as_str())
                {
                    list.push((a.clone(), None));
                    if list.len() >= 10 {
                        return list;
                    }
                }
            }
        }
        list
    }
}

pub fn format_relative_time(timestamp: u64) -> String {
    if timestamp == 0 {
        return String::new();
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if now <= timestamp {
        return "just now".to_string();
    }
    let diff = now - timestamp;
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        let mins = diff / 60;
        format!("{}m ago", mins)
    } else if diff < 86400 {
        let hours = diff / 3600;
        format!("{}h ago", hours)
    } else if diff < 86400 * 7 {
        let days = diff / 86400;
        if days == 1 {
            "yesterday".to_string()
        } else {
            format!("{}d ago", days)
        }
    } else if diff < 86400 * 30 {
        let weeks = diff / (86400 * 7);
        format!("{}w ago", weeks)
    } else {
        let months = diff / (86400 * 30);
        format!("{}mo ago", months)
    }
}

#[derive(Clone, Debug)]
pub struct SemanticHint {
    pub key: char,
    pub char_idx: usize,
    pub title: String,
}

impl App {
    pub fn current_continue_reading_hints(&self) -> Vec<(String, Option<SemanticHint>)> {
        let recents = self.get_continue_reading_with_timestamps();
        let displayed_count = recents.len().min(7);
        let titles: Vec<String> = recents
            .into_iter()
            .take(displayed_count)
            .map(|(t, _)| t)
            .collect();
        let hints = compute_semantic_hints(&titles);
        titles.into_iter().zip(hints).collect()
    }

    pub fn find_semantic_hint_article(&self, key: char) -> Option<String> {
        let target_c = key.to_ascii_lowercase();
        self.current_continue_reading_hints()
            .into_iter()
            .find(|(_, h)| h.as_ref().map(|x| x.key) == Some(target_c))
            .map(|(t, _)| t)
    }
}

pub fn compute_semantic_hints(titles: &[String]) -> Vec<Option<SemanticHint>> {
    const RESERVED_KEYS: &[char] = &[
        'f', 'n', 'd', 't', 'r', 'q', 'z', 'a', 'F', 'N', 'D', 'T', 'R', 'Q', 'Z', 'A', ':', '?',
        ',', '/', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0',
    ];

    let mut used_keys = std::collections::HashSet::new();
    let mut hints = Vec::with_capacity(titles.len());

    for title in titles {
        let mut assigned = None;

        let mut prev_is_space = true;
        for (idx, ch) in title.char_indices() {
            let is_word_start = prev_is_space && (ch.is_alphanumeric() || ch == '.');
            prev_is_space = ch.is_whitespace() || ch == '(' || ch == '[' || ch == '-';
            if is_word_start && !ch.is_whitespace() {
                let lower = ch.to_ascii_lowercase();
                if !RESERVED_KEYS.contains(&lower) && used_keys.insert(lower) {
                    assigned = Some(SemanticHint {
                        key: lower,
                        char_idx: idx,
                        title: title.clone(),
                    });
                    break;
                }
            }
        }

        if assigned.is_none() {
            for (idx, ch) in title.char_indices() {
                if !ch.is_whitespace() {
                    let lower = ch.to_ascii_lowercase();
                    if !RESERVED_KEYS.contains(&lower) && used_keys.insert(lower) {
                        assigned = Some(SemanticHint {
                            key: lower,
                            char_idx: idx,
                            title: title.clone(),
                        });
                        break;
                    }
                }
            }
        }

        hints.push(assigned);
    }

    hints
}
