use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SavedList {
    pub id: String,
    pub name: String,
    pub articles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedListsStore {
    pub version: u32,
    pub lists: Vec<SavedList>,
    #[serde(skip)]
    pub cached_saved_set: HashSet<String>,
}

impl Default for SavedListsStore {
    fn default() -> Self {
        Self {
            version: 1,
            lists: vec![SavedList {
                id: "liked".to_string(),
                name: "Liked".to_string(),
                articles: Vec::new(),
            }],
            cached_saved_set: HashSet::new(),
        }
    }
}

impl SavedListsStore {
    pub fn file_path() -> PathBuf {
        crate::paths::config_dir().join("saved_articles.json")
    }

    pub fn rebuild_cache(&mut self) {
        self.cached_saved_set.clear();
        for list in &self.lists {
            for a in &list.articles {
                self.cached_saved_set.insert(a.trim().to_lowercase());
            }
        }
    }

    pub fn load() -> Self {
        let path = Self::file_path();
        let loaded = fs::read_to_string(&path)
            .ok()
            .and_then(|content| serde_json::from_str::<SavedListsStore>(&content).ok());

        if let Some(mut store) = loaded {
            store.ensure_list("liked", "Liked");
            store.rebuild_cache();
            return store;
        }
        let mut store = Self::default();
        store.save();
        store.rebuild_cache();
        store
    }

    pub fn save(&self) {
        let path = Self::file_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(pretty_json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, pretty_json);
        }
    }

    pub fn create_list(&mut self, name: &str) -> String {
        let clean_name = name.trim();
        if clean_name.is_empty() {
            return String::new();
        }
        let id = clean_name.to_lowercase().replace(' ', "_");

        if !self.lists.iter().any(|l| l.id == id) {
            self.lists.push(SavedList {
                id: id.clone(),
                name: clean_name.to_string(),
                articles: Vec::new(),
            });
            self.save();
            self.rebuild_cache();
        }
        id
    }

    pub fn delete_list(&mut self, list_id: &str) {
        if list_id == "liked" {
            return;
        }
        self.lists.retain(|l| l.id != list_id);
        self.save();
        self.rebuild_cache();
    }

    pub fn clear_list(&mut self, list_id: &str) {
        if let Some(list) = self.lists.iter_mut().find(|l| l.id == list_id) {
            list.articles.clear();
            self.save();
            self.rebuild_cache();
        }
    }

    pub fn toggle_article_in_list(&mut self, list_id: &str, title: &str) -> bool {
        let title_trimmed = title.trim();
        if title_trimmed.is_empty() {
            return false;
        }

        if let Some(list) = self.lists.iter_mut().find(|l| l.id == list_id) {
            let added = if let Some(idx) = list.articles.iter().position(|a| a == title_trimmed) {
                list.articles.remove(idx);
                false
            } else {
                list.articles.push(title_trimmed.to_string());
                true
            };
            self.save();
            self.rebuild_cache();
            return added;
        }
        false
    }

    pub fn remove_article_from_list(&mut self, list_id: &str, title: &str) -> bool {
        let title_trimmed = title.trim();
        if title_trimmed.is_empty() {
            return false;
        }

        if let Some(list) = self.lists.iter_mut().find(|l| l.id == list_id) {
            if let Some(idx) = list.articles.iter().position(|a| a == title_trimmed) {
                list.articles.remove(idx);
                self.save();
                self.rebuild_cache();
                return true;
            }
        }
        false
    }

    pub fn is_article_in_list(&self, list_id: &str, title: &str) -> bool {
        let title_trimmed = title.trim();
        if let Some(list) = self.lists.iter().find(|l| l.id == list_id) {
            list.articles.iter().any(|a| a == title_trimmed)
        } else {
            false
        }
    }

    pub fn is_article_saved_anywhere(&self, title: &str) -> bool {
        if self.cached_saved_set.is_empty() {
            return false;
        }
        let title_trimmed = title.trim();
        if title_trimmed.is_empty() {
            return false;
        }
        if title_trimmed.is_ascii() {
            if title_trimmed.len() <= 256 {
                let mut buf = [0u8; 256];
                let bytes = title_trimmed.as_bytes();
                for (i, &b) in bytes.iter().enumerate() {
                    buf[i] = b.to_ascii_lowercase();
                }
                if let Ok(s) = std::str::from_utf8(&buf[..bytes.len()]) {
                    return self.cached_saved_set.contains(s);
                }
            }
        } else {
            let mut buf = [0u8; 256];
            let mut cursor = 0;
            let mut fits = true;
            for c in title_trimmed.chars() {
                for lc in c.to_lowercase() {
                    let len = lc.len_utf8();
                    if cursor + len > buf.len() {
                        fits = false;
                        break;
                    }
                    lc.encode_utf8(&mut buf[cursor..]);
                    cursor += len;
                }
                if !fits {
                    break;
                }
            }
            if fits {
                if let Ok(s) = std::str::from_utf8(&buf[..cursor]) {
                    return self.cached_saved_set.contains(s);
                }
            }
        }
        self.cached_saved_set
            .contains(&title_trimmed.to_lowercase())
    }

    pub fn ensure_list(&mut self, id: &str, name: &str) {
        if let Some(list) = self.lists.iter_mut().find(|l| l.id == id) {
            if list.name != name {
                list.name = name.to_string();
                self.save();
            }
        } else {
            self.lists.push(SavedList {
                id: id.to_string(),
                name: name.to_string(),
                articles: Vec::new(),
            });
            self.save();
        }
    }

    pub fn rename_list(&mut self, list_id: &str, new_name: &str) -> bool {
        let trimmed = new_name.trim();
        if list_id == "liked" || trimmed.is_empty() {
            return false;
        }
        if let Some(list) = self.lists.iter_mut().find(|l| l.id == list_id) {
            list.name = trimmed.to_string();
            self.save();
            return true;
        }
        false
    }

    pub fn set_article_in_list(
        &mut self,
        list_id: &str,
        list_name: &str,
        title: &str,
        in_list: bool,
    ) {
        self.ensure_list(list_id, list_name);
        if let Some(list) = self.lists.iter_mut().find(|l| l.id == list_id) {
            let title_trimmed = title.trim();
            if in_list {
                if !list.articles.iter().any(|a| a == title_trimmed) {
                    list.articles.push(title_trimmed.to_string());
                    self.save();
                    self.rebuild_cache();
                }
            } else if let Some(idx) = list.articles.iter().position(|a| a == title_trimmed) {
                list.articles.remove(idx);
                self.save();
                self.rebuild_cache();
            }
        }
    }

    pub fn sync_liked_articles(&mut self, feed_liked: &mut HashSet<String>) {
        self.ensure_list("liked", "Liked");
        let mut modified = false;
        if let Some(list) = self.lists.iter_mut().find(|l| l.id == "liked") {
            for title in feed_liked.iter() {
                let title_trimmed = title.trim();
                if !title_trimmed.is_empty() && !list.articles.iter().any(|a| a == title_trimmed) {
                    list.articles.push(title_trimmed.to_string());
                    modified = true;
                }
            }
            for title in &list.articles {
                feed_liked.insert(title.clone());
            }
        }
        if modified {
            self.save();
            self.rebuild_cache();
        }
    }
}
