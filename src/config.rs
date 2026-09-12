use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Config {
    pub general: GeneralConfig,
    pub reader: ReaderConfig,
    pub ui: UiConfig,
    pub search: SearchConfig,
    pub network: NetworkConfig,
    pub input: InputConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct InputConfig {
    pub mouse_support: bool,
    pub scroll_speed: usize,
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            mouse_support: true,
            scroll_speed: 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HintMode {
    #[default]
    Semantic,
    Numbered,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GeneralConfig {
    pub liked_readonly: bool,
    pub auto_restore_session: bool,
    pub confirm_quit: bool,
    pub hint_mode: HintMode,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            liked_readonly: true,
            auto_restore_session: false,
            confirm_quit: true,
            hint_mode: HintMode::Semantic,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    pub rounded_borders: bool,
    pub icons: bool,
    pub scroll_indicator: bool,
    pub stats: bool,
    pub dim_inactive_panes: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            rounded_borders: false,
            icons: true,
            scroll_indicator: true,
            stats: true,
            dim_inactive_panes: false,
        }
    }
}

impl UiConfig {
    pub fn border_type(&self) -> ratatui::widgets::BorderType {
        crate::theme::border_type(self.rounded_borders)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageProtocol {
    #[default]
    Auto,
    Kitty,
    Halfblocks,
    Off,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HalfblockFilter {
    Nearest,
    Triangle,
    #[serde(rename = "catmullrom")]
    #[default]
    Catmullrom,
    Gaussian,
    Lanczos3,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ReaderConfig {
    pub scroll_lines: usize,
    pub underline_links: bool,
    pub show_footnotes: bool,
    pub show_external_links: bool,
    pub toc_section_numbers: bool,
    pub heading_marker: bool,
    pub code_line_numbers: bool,
    pub show_images: bool,
    pub image_protocol: ImageProtocol,
    pub halfblock_filter: HalfblockFilter,
    pub max_image_height: usize,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self {
            scroll_lines: 1,
            underline_links: false,
            show_footnotes: true,
            show_external_links: true,
            toc_section_numbers: true,
            heading_marker: true,
            code_line_numbers: true,
            show_images: true,
            image_protocol: ImageProtocol::Auto,
            halfblock_filter: HalfblockFilter::Catmullrom,
            max_image_height: 25,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    pub limit: usize,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self { limit: 20 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct NetworkConfig {
    pub timeout: u64,
    pub offline_cache: bool,
    pub cache_lifetime: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            timeout: 10,
            offline_cache: true,
            cache_lifetime: 24,
        }
    }
}

use std::time::SystemTime;

impl Config {
    pub fn config_path() -> PathBuf {
        crate::paths::config_dir().join("config.toml")
    }

    pub fn get_modified_time() -> Option<SystemTime> {
        let path = Self::config_path();
        fs::metadata(&path).ok().and_then(|m| m.modified().ok())
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path)
            && let Ok(config) = toml::from_str::<Config>(&content) {
                return config;
            }
        let config = Self::default();
        config.save();
        config
    }

    pub fn reload_if_changed(&mut self, last_mtime: &mut Option<SystemTime>) -> bool {
        let current_mtime = Self::get_modified_time();
        if current_mtime.is_some() && current_mtime != *last_mtime {
            *last_mtime = current_mtime;
            let path = Self::config_path();
            if let Ok(content) = fs::read_to_string(&path)
                && let Ok(new_config) = toml::from_str::<Config>(&content) {
                    *self = new_config;
                    return true;
                }
        }
        false
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = fs::write(path, content);
        }
    }
}
