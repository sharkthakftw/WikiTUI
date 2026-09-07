pub mod audio_ctrl;
pub mod events;
pub mod feed_ctrl;
pub mod history;
pub mod layout_mgr;
pub mod modal_ctrl;
pub mod navigation;
pub mod network;
pub mod pane;
pub mod recent;
pub mod search;
pub mod settings;
pub mod tab;
pub mod types;

pub use pane::{LocalMatch, Pane, PaneContent, TextSelection};
pub use settings::SettingItem;
pub use tab::Tab;
pub use types::{
    is_article_link, CategoriesModalState, ClosedTabState, ClosedTabsHistory, ConfigManager,
    ConfirmAction, GraphicsState, ImageRenderTask, InputMode, ListsModalState, NetworkDispatcher,
    OnboardingModalState, SearchModalState, SettingsModalState, StatusMessageState,
};

use crate::api::NetworkCommand;
use std::sync::mpsc::Sender;

pub struct App {
    pub running: bool,
    pub tabs: Vec<Tab>,
    pub active_tab_idx: usize,
    pub prev_tab_idx: Option<usize>,
    pub input_mode: InputMode,
    pub search_modal: SearchModalState,
    pub waiting_for_split_cmd: bool,
    pub zen_mode: bool,

    pub feed: crate::feed::FeedState,
    pub onboarding: OnboardingModalState,

    pub saved_lists: crate::saved_lists::SavedListsStore,
    pub lists_modal: ListsModalState,
    pub confirm_action: Option<ConfirmAction>,
    pub config: ConfigManager,
    pub settings_modal: SettingsModalState,
    pub categories_modal: CategoriesModalState,
    pub closed_tabs_stack: ClosedTabsHistory,
    pub status_message: StatusMessageState,
    pub wiki_stats: crate::api::WikiStatistics,
    pub daily_feed: Option<crate::api::DailyFeed>,
    pub daily_feed_modal: Option<crate::ui::modals::DailyFeedModalState>,
    pub pending_open_tfa: bool,
    pub recent_articles: Vec<crate::app::recent::RecentArticleEntry>,
    pub launch_quote_idx: usize,
    pub scroll_drag: Option<crate::mouse::ScrollDragTarget>,
    pub audio_player: crate::audio::AudioPlayer,
    pub command_palette: crate::app::types::CommandPaletteState,
    pub qr_modal: Option<crate::app::types::QrModalState>,
    pub image_modal: Option<crate::app::types::ImageModalState>,
    pub link_peek: Option<crate::app::types::LinkPeekState>,
    pub summary_cache: std::collections::HashMap<String, (Option<String>, Option<String>)>,
    pub graphics: GraphicsState,

    pub(crate) next_pane_id: usize,
    pub(crate) network: NetworkDispatcher,
}

impl App {
    pub fn new(cmd_tx: Sender<NetworkCommand>) -> Self {
        let config = crate::config::Config::load();
        if config.ui.stats {
            let _ = cmd_tx.send(NetworkCommand::FetchStats {
                timeout: config.network.timeout,
            });
        }
        let (y, m, d) = crate::api::daily_feed::utc_today();
        let cached_feed = if config.network.offline_cache {
            crate::api::daily_feed::get_cached_daily_feed(y, m, d)
        } else {
            None
        };
        let cache_lifetime = config.network.cache_lifetime;
        if cache_lifetime > 0 {
            std::thread::spawn(move || {
                crate::cache::evict_expired_cache(cache_lifetime);
            });
        }
        let quote_idx = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as usize)
            .unwrap_or(0);
        let mut app = Self {
            running: true,
            tabs: Vec::new(),
            active_tab_idx: 0,
            prev_tab_idx: None,
            input_mode: InputMode::Normal,
            search_modal: SearchModalState::default(),
            waiting_for_split_cmd: false,
            zen_mode: false,
            feed: crate::feed::FeedState::new(),
            onboarding: OnboardingModalState::default(),

            saved_lists: crate::saved_lists::SavedListsStore::load(),
            lists_modal: ListsModalState::default(),
            confirm_action: None,
            config: ConfigManager::new(config),
            settings_modal: SettingsModalState::default(),
            categories_modal: CategoriesModalState::default(),
            closed_tabs_stack: ClosedTabsHistory::default(),
            status_message: StatusMessageState::default(),
            wiki_stats: crate::api::WikiStatistics::default(),
            daily_feed: cached_feed,
            daily_feed_modal: None,
            pending_open_tfa: false,
            recent_articles: Self::load_recent_articles(),
            launch_quote_idx: quote_idx,
            scroll_drag: None,
            audio_player: crate::audio::AudioPlayer::new(),
            command_palette: crate::app::types::CommandPaletteState::default(),
            qr_modal: None,
            image_modal: None,
            link_peek: None,
            summary_cache: std::collections::HashMap::new(),
            graphics: GraphicsState::default(),

            next_pane_id: 1,
            network: NetworkDispatcher::new(cmd_tx),
        };
        app.saved_lists
            .sync_liked_articles(&mut app.feed.profile.liked_articles);
        if app.config.general.auto_restore_session {
            if let Some(session) = crate::session::SessionState::load() {
                app.restore_session(session);
            }
        }
        if app.tabs.is_empty() {
            app.tabs.push(Tab::new("home".to_string(), 0));
        }
        app
    }

    pub fn check_config_sync(&mut self) {
        self.config.check_sync();
    }

    pub fn save_session(&self) {
        crate::session::SessionState::save_app_session(self);
    }

    pub fn restore_session(&mut self, state: crate::session::SessionState) {
        state.restore_to_app(self);
    }

    pub fn quit(&mut self) {
        if self.config.general.confirm_quit {
            self.confirm_action = Some(ConfirmAction::Quit);
            self.input_mode = InputMode::Confirm;
        } else {
            self.save_session();
            self.running = false;
        }
    }

    pub fn toggle_zen_mode(&mut self) {
        self.zen_mode = !self.zen_mode;
    }

    pub fn open_command_palette(&mut self) {
        self.input_mode = InputMode::CommandPalette;
        self.command_palette.query.clear();
        self.command_palette.selected_idx = 0;
    }

    pub fn active_tab(&self) -> &Tab {
        let idx = self.active_tab_idx.min(self.tabs.len().saturating_sub(1));
        &self.tabs[idx]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        if self.tabs.is_empty() {
            self.tabs.push(Tab::new("home".to_string(), 0));
        }
        if self.active_tab_idx >= self.tabs.len() {
            self.active_tab_idx = self.tabs.len() - 1;
        }
        let idx = self.active_tab_idx;
        &mut self.tabs[idx]
    }

    pub fn active_pane(&self) -> &Pane {
        let tab = self.active_tab();
        let idx = tab.active_pane_idx.min(tab.panes.len().saturating_sub(1));
        &tab.panes[idx]
    }

    pub fn active_pane_mut(&mut self) -> &mut Pane {
        let tab = self.active_tab_mut();
        if tab.panes.is_empty() {
            tab.panes.push(Pane::new(0));
        }
        if tab.active_pane_idx >= tab.panes.len() {
            tab.active_pane_idx = tab.panes.len() - 1;
        }
        let idx = tab.active_pane_idx;
        &mut tab.panes[idx]
    }

    pub fn toggle_images(&mut self) {
        self.config.reader.show_images = !self.config.reader.show_images;
        let status = if self.config.reader.show_images {
            "enabled"
        } else {
            "disabled"
        };
        self.set_status_message(format!("inline images {}", status));
    }

    pub fn open_qr_modal(&mut self) {
        let (title, url) = match &self.active_pane().content {
            crate::app::PaneContent::ArticleText { title, .. } => (
                title.clone(),
                format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_")),
            ),
            crate::app::PaneContent::SearchResults { query, .. } => (
                format!("search: {}", query),
                format!(
                    "https://en.wikipedia.org/wiki/Special:Search?search={}",
                    query.replace(' ', "_")
                ),
            ),
            _ => {
                if self.feed.active {
                    if let Some(item) = self.feed.current_item() {
                        (
                            item.title.clone(),
                            format!("https://en.wikipedia.org/wiki/{}", item.title.replace(' ', "_")),
                        )
                    } else {
                        self.set_status_message("no article to share");
                        return;
                    }
                } else {
                    self.set_status_message("no article to share");
                    return;
                }
            }
        };

        if let Ok(qrcode) = fast_qr::qr::QRBuilder::new(url.as_bytes()).build() {
            let size = qrcode.size;
            let mut matrix = vec![vec![false; size]; size];
            for y in 0..size {
                for x in 0..size {
                    matrix[y][x] = qrcode[y][x].value();
                }
            }
            self.qr_modal = Some(crate::app::types::QrModalState {
                title,
                full_url: url.clone(),
                short_url: None,
                matrix,
            });
            self.input_mode = InputMode::QrModal;
            self.network.send(crate::api::NetworkCommand::ShortenUrl {
                url,
                timeout: self.config.network.timeout,
            });
        } else {
            self.set_status_message("failed to generate qr code");
        }
    }

    pub fn close_qr_modal(&mut self) {
        self.qr_modal = None;
        self.input_mode = InputMode::Normal;
    }

    pub fn open_image_modal(
        &mut self,
        url: String,
        alt: Option<String>,
        caption: Option<String>,
        path: Option<std::path::PathBuf>,
    ) {
        self.image_modal = Some(crate::app::types::ImageModalState {
            url,
            alt,
            caption,
            path,
        });
        self.input_mode = InputMode::ImageModal;
    }

    pub fn close_image_modal(&mut self) {
        self.image_modal = None;
        self.input_mode = InputMode::Normal;
    }

    pub fn open_link_peek(
        &mut self,
        title: String,
        raw_target: String,
        anchor_x: u16,
        anchor_y: u16,
    ) {
        if raw_target.starts_with("http://")
            || raw_target.starts_with("https://")
            || raw_target.starts_with("//")
        {
            let desc = raw_target
                .split("://")
                .nth(1)
                .unwrap_or(&raw_target)
                .split('/')
                .next()
                .map(|s| s.to_string());
            self.link_peek = Some(crate::app::types::LinkPeekState {
                title: "External Link".to_string(),
                raw_target: raw_target.clone(),
                description: desc,
                extract: Some(raw_target),
                is_loading: false,
                anchor_x,
                anchor_y,
            });
            self.input_mode = InputMode::LinkPeek;
            return;
        }

        let clean_title = crate::parser::url_decode(&title)
            .replace('_', " ")
            .trim()
            .to_string();
        if let Some((desc, extract)) = self.summary_cache.get(&clean_title) {
            self.link_peek = Some(crate::app::types::LinkPeekState {
                title: clean_title,
                raw_target,
                description: desc.clone(),
                extract: extract.clone(),
                is_loading: false,
                anchor_x,
                anchor_y,
            });
        } else {
            self.link_peek = Some(crate::app::types::LinkPeekState {
                title: clean_title.clone(),
                raw_target,
                description: None,
                extract: None,
                is_loading: true,
                anchor_x,
                anchor_y,
            });
            self.send_fetch_summary(clean_title);
        }
        self.input_mode = InputMode::LinkPeek;
    }

    pub fn close_link_peek(&mut self) {
        self.link_peek = None;
        if self.input_mode == InputMode::LinkPeek {
            self.input_mode = InputMode::Normal;
        }
    }

    pub fn send_fetch_summary(&self, title: String) {
        self.network.send(crate::api::NetworkCommand::FetchSummary {
            title,
            timeout: self.config.network.timeout,
        });
    }
}
