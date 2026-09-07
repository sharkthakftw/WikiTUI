use std::fs;
use std::path::Path;
use std::time::Duration;

pub fn evict_expired_cache(lifetime_hours: u64) {
    if lifetime_hours == 0 {
        return;
    }

    let max_age = Duration::from_secs(lifetime_hours.saturating_mul(3600));
    let cache_dir = crate::paths::cache_dir();
    let articles_dir = cache_dir.join("articles");
    let images_dir = crate::paths::image_cache_dir();
    let audio_dir = crate::paths::audio_cache_dir();

    clean_directory(&articles_dir, max_age, |path| {
        path.extension().and_then(|e| e.to_str()) == Some("html")
    });

    clean_directory(&images_dir, max_age, |_| true);

    clean_directory(&cache_dir, max_age, |path| {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            name.starts_with("feed_") && name.ends_with(".json")
        } else {
            false
        }
    });

    let daily_feed_dir = cache_dir.join("daily_feed");
    if let Ok(entries) = fs::read_dir(&daily_feed_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed > max_age {
                                let _ = fs::remove_dir_all(&path);
                            }
                        }
                    }
                }
            }
        }
    }

    clean_directory(&audio_dir, max_age, |path| {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            name != "durations.json" && name != "positions.json"
        } else {
            false
        }
    });
}

fn clean_directory<F>(dir: &Path, max_age: Duration, filter: F)
where
    F: Fn(&Path) -> bool,
{
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && filter(&path) {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(modified) = metadata.modified() {
                        if let Ok(elapsed) = modified.elapsed() {
                            if elapsed > max_age {
                                let _ = fs::remove_file(&path);
                            }
                        }
                    }
                }
            }
        }
    }
}
