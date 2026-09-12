use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

pub use crate::feed::categories::POPULAR_CATEGORIES;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedProfile {
    pub version: u32,
    #[serde(default)]
    pub has_onboarded: bool,
    #[serde(default)]
    pub selected_categories: Vec<String>,
    pub total_likes: u32,
    pub total_seen: u32,
    pub category_scores: HashMap<String, i32>,
    pub liked_articles: HashSet<String>,
    pub seen_articles: HashSet<String>,
}

impl Default for FeedProfile {
    fn default() -> Self {
        let mut category_scores = HashMap::new();
        category_scores.insert("given names".to_string(), -1000);
        category_scores.insert("surnames".to_string(), -1000);
        category_scores.insert("disambiguation pages".to_string(), -1000);

        Self {
            version: 1,
            has_onboarded: false,
            selected_categories: Vec::new(),
            total_likes: 0,
            total_seen: 0,
            category_scores,
            liked_articles: HashSet::new(),
            seen_articles: HashSet::new(),
        }
    }
}

impl FeedProfile {
    pub fn profile_path() -> PathBuf {
        crate::paths::config_dir().join("feed_profile.json")
    }

    pub fn load() -> Self {
        let path = Self::profile_path();
        let loaded = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<FeedProfile>(&content).ok());

        if let Some(mut profile) = loaded {
            let mut normalized = HashMap::with_capacity(profile.category_scores.len());
            for (k, v) in profile.category_scores {
                normalized.insert(k.to_lowercase(), v.max(0));
            }
            profile.category_scores = normalized;
            return profile;
        }
        let profile = Self::default();
        profile.save();
        profile
    }

    pub fn save(&self) {
        let path = Self::profile_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(pretty_json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, pretty_json);
        }
    }

    pub fn reset(&mut self) {
        *self = Self::default();
        self.save();
    }

    pub fn record_engagement(&mut self, categories: &[String], points: i32) {
        for cat in categories {
            let cat_lower = cat.to_lowercase();
            let current = self.category_scores.entry(cat_lower).or_insert(0);
            *current = (*current + points).max(0);
        }
    }

    pub fn mark_liked(&mut self, title: &str, categories: &[String]) -> bool {
        let is_liked = if self.liked_articles.contains(title) {
            self.liked_articles.remove(title);
            self.record_engagement(categories, -50);
            if self.total_likes > 0 {
                self.total_likes -= 1;
            }
            false
        } else {
            self.liked_articles.insert(title.to_string());
            self.record_engagement(categories, 50);
            self.total_likes += 1;
            true
        };
        self.save();
        is_liked
    }

    pub fn mark_seen(&mut self, title: &str) {
        if !self.seen_articles.contains(title) {
            self.seen_articles.insert(title.to_string());
            self.total_seen += 1;
            self.save();
        }
    }
}

fn stem_word(w: &str) -> &str {
    if let Some(s) = w.strip_suffix("ies") {
        s
    } else if let Some(s) = w.strip_suffix("IES") {
        s
    } else if let Some(s) = w.strip_suffix("es") {
        s
    } else if let Some(s) = w.strip_suffix("ES") {
        s
    } else if let Some(s) = w.strip_suffix(|c| c == 's' || c == 'S') {
        s
    } else if let Some(s) = w.strip_suffix(|c| c == 'y' || c == 'Y') {
        s
    } else {
        w
    }
}

fn category_matches(key: &str, cat: &str) -> bool {
    if key.eq_ignore_ascii_case(cat) {
        return true;
    }
    if key.contains(' ') {
        let key_lower = key.to_lowercase();
        let cat_lower = cat.to_lowercase();
        if let Some(pos) = cat_lower.find(&key_lower) {
            let before = if pos == 0 {
                true
            } else {
                !cat_lower[..pos]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric())
            };
            let end = pos + key_lower.len();
            let after = if end >= cat_lower.len() {
                true
            } else {
                !cat_lower[end..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphanumeric())
            };
            if before && after {
                return true;
            }
        }
        return false;
    }

    let key_stem = stem_word(key);
    for word in cat.split(|c: char| !c.is_alphanumeric()) {
        if word.is_empty() {
            continue;
        }
        if word.eq_ignore_ascii_case(key) {
            return true;
        }
        let word_stem = stem_word(word);
        if word_stem.len() >= 3 && key_stem.len() >= 3 && word_stem.eq_ignore_ascii_case(key_stem) {
            return true;
        }
        let w_char_count = word.chars().count();
        let k_char_count = key.chars().count();
        if w_char_count >= 6 && k_char_count >= 6 {
            let w_idx = word.char_indices().nth(6).map_or(word.len(), |(i, _)| i);
            let k_idx = key.char_indices().nth(6).map_or(key.len(), |(i, _)| i);
            if word[..w_idx].eq_ignore_ascii_case(&key[..k_idx]) {
                return true;
            }
        }
    }
    false
}

fn is_preset_category(cat: &str) -> bool {
    for (display_name, label, subcats) in POPULAR_CATEGORIES {
        if cat.eq_ignore_ascii_case(display_name) || cat.eq_ignore_ascii_case(label) {
            return true;
        }
        for subcat in *subcats {
            if cat.eq_ignore_ascii_case(subcat) {
                return true;
            }
        }
    }
    false
}

fn is_maintenance_category(cat: &str) -> bool {
    let lower = cat.to_lowercase();
    lower.contains("articles")
        || lower.contains("cs1")
        || lower.contains("use dmy")
        || lower.contains("use mdy")
        || lower.contains("stub")
        || lower.contains("disambiguation")
        || lower.contains("wikidata")
        || lower.contains("webarchive")
        || lower.contains("pages with")
        || lower.contains("wikipedia")
        || lower.contains("hidden categories")
}

impl FeedProfile {
    pub fn score_for_categories(&self, categories: &[String]) -> i32 {
        let mut score = 0;
        for cat in categories {
            for (key, &cat_score) in &self.category_scores {
                if cat_score > 0 && category_matches(key, cat) {
                    score += cat_score;
                }
            }
        }
        score
    }

    pub fn complete_onboarding(&mut self, chosen_indices: &[usize]) {
        self.selected_categories.clear();
        for &idx in chosen_indices {
            if let Some((_, label, subcats)) = POPULAR_CATEGORIES.get(idx) {
                self.selected_categories.push(label.to_string());
                for subcat in *subcats {
                    let current = self
                        .category_scores
                        .entry(subcat.to_lowercase())
                        .or_insert(0);
                    *current += 200;
                }
            }
        }
        self.has_onboarded = true;
        self.save();
    }

    pub fn get_active_subcategories(&self) -> Vec<String> {
        let mut subcats = Vec::new();
        for (display_name, label, subcat_list) in POPULAR_CATEGORIES {
            if self
                .selected_categories
                .iter()
                .any(|c| c == *label || c == *display_name)
            {
                for s in *subcat_list {
                    subcats.push(s.to_string());
                }
            }
        }
        if subcats.is_empty() {
            for (_, _, subcat_list) in POPULAR_CATEGORIES {
                for s in *subcat_list {
                    subcats.push(s.to_string());
                }
            }
        }
        subcats
    }

    pub fn get_organic_categories(&self) -> Vec<String> {
        let mut organic: Vec<(&String, i32)> = self
            .category_scores
            .iter()
            .filter(|&(cat, &score)| {
                score > 0
                    && !is_preset_category(cat)
                    && !is_maintenance_category(cat)
                    && cat.len() >= 3
            })
            .map(|(cat, &score)| (cat, score))
            .collect();

        organic.sort_by_key(|a| std::cmp::Reverse(a.1));
        organic
            .into_iter()
            .take(20)
            .map(|(cat, _)| cat.clone())
            .collect()
    }
}
