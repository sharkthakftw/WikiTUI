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

pub struct ModalStateRegistry {
    pub search_modal: SearchModalState,
    pub onboarding: OnboardingModalState,
    pub lists_modal: ListsModalState,
    pub settings_modal: SettingsModalState,
    pub categories_modal: CategoriesModalState,
    pub daily_feed_modal: Option<crate::ui::modals::DailyFeedModalState>,
    pub command_palette: crate::app::types::CommandPaletteState,
    pub qr_modal: Option<crate::app::types::QrModalState>,
    pub image_modal: Option<crate::app::types::ImageModalState>,
    pub link_peek: Option<crate::app::types::LinkPeekState>,
    pub confirm_action: Option<ConfirmAction>,
}

pub struct Workspace {
    pub tabs: Vec<Tab>,
    pub active_tab_idx: usize,
    pub prev_tab_idx: Option<usize>,
    pub closed_tabs_stack: ClosedTabsHistory,
    pub waiting_for_split_cmd: bool,
    pub zen_mode: bool,
    pub tab_bar_cache: Option<crate::ui::tab_bar::TabBarCache>,
    pub scroll_drag: Option<crate::mouse::ScrollDragTarget>,
    pub(crate) next_pane_id: usize,
}

pub struct UserDataStores {
    pub saved_lists: crate::saved_lists::SavedListsStore,
    pub recent_articles: Vec<crate::app::recent::RecentArticleEntry>,
    pub summary_cache: std::collections::HashMap<String, (Option<String>, Option<String>)>,
    pub feed: crate::feed::FeedState,
    pub config: ConfigManager,
}

pub struct App {
    pub running: bool,
    pub input_mode: InputMode,
    pub status_message: StatusMessageState,
    pub wiki_stats: crate::api::WikiStatistics,
    pub daily_feed: Option<crate::api::DailyFeed>,
    pub pending_open_tfa: bool,
    pub launch_quote_idx: usize,
    pub audio_player: crate::audio::AudioPlayer,
    pub graphics: GraphicsState,
    pub(crate) network: NetworkDispatcher,

    pub modals: ModalStateRegistry,
    pub workspace: Workspace,
    pub user_data: UserDataStores,
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
            input_mode: InputMode::Normal,
            status_message: StatusMessageState::default(),
            wiki_stats: crate::api::WikiStatistics::default(),
            daily_feed: cached_feed,
            pending_open_tfa: false,
            launch_quote_idx: quote_idx,
            audio_player: crate::audio::AudioPlayer::new(),
            graphics: GraphicsState::default(),
            network: NetworkDispatcher::new(cmd_tx),

            workspace: Workspace {
                tabs: Vec::new(),
                active_tab_idx: 0,
                prev_tab_idx: None,
                closed_tabs_stack: ClosedTabsHistory::default(),
                waiting_for_split_cmd: false,
                zen_mode: false,
                tab_bar_cache: None,
                scroll_drag: None,
                next_pane_id: 1,
            },

            modals: ModalStateRegistry {
                search_modal: SearchModalState::default(),
                onboarding: OnboardingModalState::default(),
                lists_modal: ListsModalState::default(),
                settings_modal: SettingsModalState::default(),
                categories_modal: CategoriesModalState::default(),
                daily_feed_modal: None,
                command_palette: crate::app::types::CommandPaletteState::default(),
                qr_modal: None,
                image_modal: None,
                link_peek: None,
                confirm_action: None,
            },

            user_data: UserDataStores {
                saved_lists: crate::saved_lists::SavedListsStore::load(),
                recent_articles: Self::load_recent_articles(),
                summary_cache: std::collections::HashMap::new(),
                feed: crate::feed::FeedState::new(),
                config: ConfigManager::new(config),
            },
        };
        app.user_data
            .saved_lists
            .sync_liked_articles(&mut app.user_data.feed.profile.liked_articles);
        if app.user_data.config.general.auto_restore_session {
            if let Some(session) = crate::session::SessionState::load() {
                app.restore_session(session);
            }
        }
        if app.workspace.tabs.is_empty() {
            app.workspace.tabs.push(Tab::new("home".to_string(), 0));
        }
        app
    }

    pub fn check_config_sync(&mut self) {
        self.user_data.config.check_sync();
    }

    pub fn save_session(&self) {
        crate::session::SessionState::save_app_session(self);
    }

    pub fn restore_session(&mut self, state: crate::session::SessionState) {
        state.restore_to_app(self);
    }

    pub fn quit(&mut self) {
        if self.user_data.config.general.confirm_quit {
            self.modals.confirm_action = Some(ConfirmAction::Quit);
            self.input_mode = InputMode::Confirm;
        } else {
            self.save_session();
            self.running = false;
        }
    }

    pub fn toggle_zen_mode(&mut self) {
        self.workspace.zen_mode = !self.workspace.zen_mode;
    }

    pub fn open_command_palette(&mut self) {
        self.input_mode = InputMode::CommandPalette;
        self.modals.command_palette.query.clear();
        self.modals.command_palette.selected_idx = 0;
    }

    pub fn active_tab(&self) -> &Tab {
        let idx = self
            .workspace
            .active_tab_idx
            .min(self.workspace.tabs.len().saturating_sub(1));
        &self.workspace.tabs[idx]
    }

    pub fn active_tab_mut(&mut self) -> &mut Tab {
        if self.workspace.tabs.is_empty() {
            self.workspace.tabs.push(Tab::new("home".to_string(), 0));
        }
        if self.workspace.active_tab_idx >= self.workspace.tabs.len() {
            self.workspace.active_tab_idx = self.workspace.tabs.len() - 1;
        }
        let idx = self.workspace.active_tab_idx;
        &mut self.workspace.tabs[idx]
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
        self.user_data.config.reader.show_images = !self.user_data.config.reader.show_images;
        let status = if self.user_data.config.reader.show_images {
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
                if self.user_data.feed.active {
                    if let Some(item) = self.user_data.feed.current_item() {
                        (
                            item.title.clone(),
                            format!(
                                "https://en.wikipedia.org/wiki/{}",
                                item.title.replace(' ', "_")
                            ),
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
            self.modals.qr_modal = Some(crate::app::types::QrModalState {
                title,
                full_url: url.clone(),
                short_url: None,
                matrix,
            });
            self.input_mode = InputMode::QrModal;
            self.network.send(crate::api::NetworkCommand::ShortenUrl {
                url,
                timeout: self.user_data.config.network.timeout,
            });
        } else {
            self.set_status_message("failed to generate qr code");
        }
    }

    pub fn close_qr_modal(&mut self) {
        self.modals.qr_modal = None;
        self.input_mode = InputMode::Normal;
    }

    pub fn open_image_modal(
        &mut self,
        url: String,
        alt: Option<String>,
        caption: Option<String>,
        path: Option<std::path::PathBuf>,
    ) {
        self.modals.image_modal = Some(crate::app::types::ImageModalState {
            url,
            alt,
            caption,
            path,
        });
        self.input_mode = InputMode::ImageModal;
    }

    pub fn close_image_modal(&mut self) {
        self.modals.image_modal = None;
        self.input_mode = InputMode::Normal;
    }

    pub fn open_link_peek(
        &mut self,
        title: String,
        raw_target: String,
        anchor_x: u16,
        anchor_y: u16,
    ) {
        if raw_target.starts_with("#cite_note")
            || raw_target.starts_with("#cite_ref")
            || raw_target.starts_with("cite_note")
            || raw_target.starts_with("cite_ref")
            || title.starts_with("#cite_note")
            || title.starts_with("cite_note")
        {
            return;
        }

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
            self.modals.link_peek = Some(crate::app::types::LinkPeekState {
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
        if let Some((desc, extract)) = self.user_data.summary_cache.get(&clean_title) {
            self.modals.link_peek = Some(crate::app::types::LinkPeekState {
                title: clean_title,
                raw_target,
                description: desc.clone(),
                extract: extract.clone(),
                is_loading: false,
                anchor_x,
                anchor_y,
            });
        } else {
            self.modals.link_peek = Some(crate::app::types::LinkPeekState {
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
        self.modals.link_peek = None;
        if self.input_mode == InputMode::LinkPeek {
            self.input_mode = InputMode::Normal;
        }
    }

    pub fn send_fetch_summary(&self, title: String) {
        self.network.send(crate::api::NetworkCommand::FetchSummary {
            title,
            timeout: self.user_data.config.network.timeout,
        });
    }
}
