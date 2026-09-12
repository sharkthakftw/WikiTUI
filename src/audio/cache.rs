use std::collections::HashMap;
use std::path::PathBuf;

fn load_json_map(filename: &str) -> HashMap<String, u64> {
    let path = crate::paths::audio_cache_dir().join(filename);
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_json_map(filename: &str, map: &HashMap<String, u64>) {
    let dir = crate::paths::audio_cache_dir();
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join(filename);
    if let Ok(json) = serde_json::to_string_pretty(map) {
        let _ = std::fs::write(&path, json);
    }
}

pub fn get_cached_duration(url: &str) -> Option<u64> {
    load_json_map("durations.json").get(url).copied()
}

pub fn save_cached_duration(url: &str, duration_secs: u64) {
    let mut map = load_json_map("durations.json");
    map.insert(url.to_string(), duration_secs);
    save_json_map("durations.json", &map);
}

pub fn get_saved_position(url: &str) -> Option<u64> {
    load_json_map("positions.json").get(url).copied()
}

pub fn save_position(url: &str, elapsed_secs: u64) {
    let mut map = load_json_map("positions.json");
    map.insert(url.to_string(), elapsed_secs);
    save_json_map("positions.json", &map);
}

pub fn clear_position(url: &str) {
    let mut map = load_json_map("positions.json");
    if map.remove(url).is_some() {
        save_json_map("positions.json", &map);
    }
}

pub fn extract_wikimedia_file_title(url: &str) -> Option<String> {
    let idx = url.rfind('/')?;
    let mut name = &url[idx + 1..];
    if let Some(q) = name.find('?') {
        name = &name[..q];
    }
    let base = if name.ends_with(".mp3") && name.contains(".ogg") {
        if let Some(ogg_pos) = name.find(".ogg") {
            &name[..ogg_pos + 4]
        } else {
            name
        }
    } else {
        name
    };

    if base.is_empty() {
        None
    } else {
        Some(base.to_string())
    }
}

pub fn safe_audio_filename(url: &str) -> String {
    let base = extract_wikimedia_file_title(url).unwrap_or_else(|| "track.ogg".to_string());
    let sanitized: String = base
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if sanitized.is_empty() {
        "track.ogg".to_string()
    } else {
        sanitized
    }
}

pub fn get_cached_audio_path(url: &str) -> Option<PathBuf> {
    let filename = safe_audio_filename(url);
    let path = crate::paths::audio_cache_dir().join(&filename);
    if path.exists()
        && std::fs::metadata(&path)
            .map(|m| m.len() > 1024)
            .unwrap_or(false)
    {
        Some(path)
    } else {
        None
    }
}

pub fn spawn_background_audio_download(url: &str) {
    let url_str = url.to_string();
    let filename = safe_audio_filename(url);
    let dir = crate::paths::audio_cache_dir();
    let final_path = dir.join(&filename);
    if final_path.exists() {
        return;
    }

    std::thread::spawn(move || {
        let _ = std::fs::create_dir_all(&dir);
        let part_path = dir.join(format!("{}.part", filename));
        const USER_AGENT: &str = concat!(
            "wikid/",
            env!("CARGO_PKG_VERSION"),
            " (https://github.com/sharkthakftw/wikid)"
        );

        if let Ok(resp) = ureq::get(&url_str).set("User-Agent", USER_AGENT).call()
            && let Ok(mut file) = std::fs::File::create(&part_path) {
                let mut reader = resp.into_reader();
                if std::io::copy(&mut reader, &mut file).is_ok() {
                    let _ = std::fs::rename(&part_path, &final_path);
                } else {
                    let _ = std::fs::remove_file(&part_path);
                }
            }
    });
}
