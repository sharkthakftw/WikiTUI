use crate::app::App;
use crate::config::Config;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingItem {
    LikedReadonly,
    AutoRestoreSession,
    ConfirmQuit,
    HintMode,
    RoundedBorders,
    Icons,
    ScrollIndicator,
    Stats,
    DimInactivePanes,
    HeadingMarker,
    ScrollLines,
    UnderlineLinks,
    ShowFootnotes,
    ShowExternalLinks,
    TocSectionNumbers,
    CodeLineNumbers,
    ShowImages,
    ImageProtocol,
    HalfblockFilter,
    SearchLimit,
    NetworkTimeout,
    OfflineCache,
    CacheLifetime,
    MouseSupport,
    ScrollSpeed,
}

impl SettingItem {
    pub const ALL: &'static [SettingItem] = &[
        SettingItem::LikedReadonly,
        SettingItem::AutoRestoreSession,
        SettingItem::ConfirmQuit,
        SettingItem::HintMode,
        SettingItem::RoundedBorders,
        SettingItem::Icons,
        SettingItem::ScrollIndicator,
        SettingItem::Stats,
        SettingItem::DimInactivePanes,
        SettingItem::HeadingMarker,
        SettingItem::ScrollLines,
        SettingItem::UnderlineLinks,
        SettingItem::ShowFootnotes,
        SettingItem::ShowExternalLinks,
        SettingItem::TocSectionNumbers,
        SettingItem::CodeLineNumbers,
        SettingItem::ShowImages,
        SettingItem::ImageProtocol,
        SettingItem::HalfblockFilter,
        SettingItem::SearchLimit,
        SettingItem::NetworkTimeout,
        SettingItem::OfflineCache,
        SettingItem::CacheLifetime,
        SettingItem::MouseSupport,
        SettingItem::ScrollSpeed,
    ];

    pub fn section(&self) -> &'static str {
        match self {
            SettingItem::LikedReadonly
            | SettingItem::AutoRestoreSession
            | SettingItem::ConfirmQuit
            | SettingItem::HintMode => "general",
            SettingItem::RoundedBorders
            | SettingItem::Icons
            | SettingItem::ScrollIndicator
            | SettingItem::Stats
            | SettingItem::DimInactivePanes => "ui",
            SettingItem::HeadingMarker
            | SettingItem::ScrollLines
            | SettingItem::UnderlineLinks
            | SettingItem::ShowFootnotes
            | SettingItem::ShowExternalLinks
            | SettingItem::TocSectionNumbers
            | SettingItem::CodeLineNumbers
            | SettingItem::ShowImages
            | SettingItem::ImageProtocol
            | SettingItem::HalfblockFilter => "reader",
            SettingItem::SearchLimit => "search",
            SettingItem::NetworkTimeout
            | SettingItem::OfflineCache
            | SettingItem::CacheLifetime => "network",
            SettingItem::MouseSupport | SettingItem::ScrollSpeed => "input",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            SettingItem::LikedReadonly => "liked list read-only",
            SettingItem::AutoRestoreSession => "auto-restore last session",
            SettingItem::ConfirmQuit => "confirm before quitting",
            SettingItem::HintMode => "continue reading hints",
            SettingItem::RoundedBorders => "rounded borders",
            SettingItem::Icons => "icons",
            SettingItem::ScrollIndicator => "scroll indicator",
            SettingItem::Stats => "wikipedia live stats",
            SettingItem::DimInactivePanes => "dim inactive split panes",
            SettingItem::HeadingMarker => "heading marker",
            SettingItem::ScrollLines => "scroll lines per step",
            SettingItem::UnderlineLinks => "underline links",
            SettingItem::ShowFootnotes => "show footnotes & citations",
            SettingItem::ShowExternalLinks => "show external links section",
            SettingItem::TocSectionNumbers => "toc section numbers",
            SettingItem::CodeLineNumbers => "code line numbers",
            SettingItem::ShowImages => "render images",
            SettingItem::ImageProtocol => "graphics protocol",
            SettingItem::HalfblockFilter => "halfblock filter",
            SettingItem::SearchLimit => "search results limit",
            SettingItem::NetworkTimeout => "request timeout",
            SettingItem::OfflineCache => "offline article cache",
            SettingItem::CacheLifetime => "cache lifetime",
            SettingItem::MouseSupport => "mouse support",
            SettingItem::ScrollSpeed => "mouse scroll speed",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            SettingItem::LikedReadonly => "prevent manual deletion of articles from liked list",
            SettingItem::AutoRestoreSession => "automatically restore last session on startup",
            SettingItem::ConfirmQuit => "prompt for confirmation when exiting wikid",
            SettingItem::HintMode => {
                "hint style for continue reading list (semantic, numbered, none)"
            }
            SettingItem::RoundedBorders => "use rounded border corners instead of sharp",
            SettingItem::Icons => "display nerd fonts",
            SettingItem::ScrollIndicator => {
                "display scrollbar track on right edge of content panes"
            }
            SettingItem::Stats => "display live wikipedia statistics on launch screen",
            SettingItem::DimInactivePanes => {
                "subtly dim unfocused panes in multi-pane splits"
            }
            SettingItem::HeadingMarker => "display colored bar marker (▍) before section headings",
            SettingItem::ScrollLines => "number of lines to scroll per j/k press (1-20)",
            SettingItem::UnderlineLinks => "display underlined modifier on article links",
            SettingItem::ShowFootnotes => "show inline reference numbers and references section",
            SettingItem::ShowExternalLinks => "show the external links section at the bottom",
            SettingItem::TocSectionNumbers => "display hierarchical numbers in table of contents",
            SettingItem::CodeLineNumbers => "display line numbers in code blocks",
            SettingItem::ShowImages => "render inline article images and diagrams",
            SettingItem::ImageProtocol => {
                "graphics rendering protocol (auto, kitty, halfblocks, off)"
            }
            SettingItem::HalfblockFilter => {
                "halfblock image resampling filter (nearest, triangle, catmullrom, gaussian, lanczos3)"
            }
            SettingItem::SearchLimit => "maximum number of search results to fetch (5-50)",
            SettingItem::NetworkTimeout => "network request timeout in seconds (2-60s)",
            SettingItem::OfflineCache => {
                "cache downloaded articles in ~/.cache/wikid for offline reading"
            }
            SettingItem::CacheLifetime => "hours before cached articles are re-downloaded (1-168h)",
            SettingItem::MouseSupport => "enable mouse clicks, tab switching, and scroll wheel",
            SettingItem::ScrollSpeed => "number of lines to scroll per mouse wheel tick (1-20)",
        }
    }
}

impl App {
    pub fn adjust_selected_setting(&mut self, delta: i32) {
        if let Some(item) = SettingItem::ALL
            .get(self.modals.settings_modal.cursor_idx)
            .copied()
        {
            match item {
                SettingItem::ScrollLines => {
                    let cur = self.user_data.config.reader.scroll_lines as i32;
                    let new_val = if delta == 0 {
                        if cur >= 20 {
                            1
                        } else {
                            cur + 1
                        }
                    } else {
                        (cur + delta).clamp(1, 20)
                    };
                    self.user_data.config.reader.scroll_lines = new_val as usize;
                }
                SettingItem::LikedReadonly => {
                    self.user_data.config.general.liked_readonly =
                        !self.user_data.config.general.liked_readonly;
                }
                SettingItem::AutoRestoreSession => {
                    self.user_data.config.general.auto_restore_session =
                        !self.user_data.config.general.auto_restore_session;
                }
                SettingItem::ConfirmQuit => {
                    self.user_data.config.general.confirm_quit =
                        !self.user_data.config.general.confirm_quit;
                }
                SettingItem::HintMode => {
                    self.user_data.config.general.hint_mode =
                        match self.user_data.config.general.hint_mode {
                            crate::config::HintMode::Semantic => {
                                if delta < 0 {
                                    crate::config::HintMode::None
                                } else {
                                    crate::config::HintMode::Numbered
                                }
                            }
                            crate::config::HintMode::Numbered => {
                                if delta < 0 {
                                    crate::config::HintMode::Semantic
                                } else {
                                    crate::config::HintMode::None
                                }
                            }
                            crate::config::HintMode::None => {
                                if delta < 0 {
                                    crate::config::HintMode::Numbered
                                } else {
                                    crate::config::HintMode::Semantic
                                }
                            }
                        };
                }
                SettingItem::RoundedBorders => {
                    self.user_data.config.ui.rounded_borders =
                        !self.user_data.config.ui.rounded_borders;
                }
                SettingItem::Icons => {
                    self.user_data.config.ui.icons = !self.user_data.config.ui.icons;
                }
                SettingItem::ScrollIndicator => {
                    self.user_data.config.ui.scroll_indicator =
                        !self.user_data.config.ui.scroll_indicator;
                }
                SettingItem::Stats => {
                    self.user_data.config.ui.stats = !self.user_data.config.ui.stats;
                }
                SettingItem::DimInactivePanes => {
                    self.user_data.config.ui.dim_inactive_panes =
                        !self.user_data.config.ui.dim_inactive_panes;
                }
                SettingItem::HeadingMarker => {
                    self.user_data.config.reader.heading_marker =
                        !self.user_data.config.reader.heading_marker;
                }
                SettingItem::UnderlineLinks => {
                    self.user_data.config.reader.underline_links =
                        !self.user_data.config.reader.underline_links;
                }
                SettingItem::ShowFootnotes => {
                    self.user_data.config.reader.show_footnotes =
                        !self.user_data.config.reader.show_footnotes;
                }
                SettingItem::ShowExternalLinks => {
                    self.user_data.config.reader.show_external_links =
                        !self.user_data.config.reader.show_external_links;
                }
                SettingItem::TocSectionNumbers => {
                    self.user_data.config.reader.toc_section_numbers =
                        !self.user_data.config.reader.toc_section_numbers;
                }
                SettingItem::CodeLineNumbers => {
                    self.user_data.config.reader.code_line_numbers =
                        !self.user_data.config.reader.code_line_numbers;
                }
                SettingItem::ShowImages => {
                    self.user_data.config.reader.show_images =
                        !self.user_data.config.reader.show_images;
                }
                SettingItem::ImageProtocol => {
                    self.user_data.config.reader.image_protocol =
                        match self.user_data.config.reader.image_protocol {
                            crate::config::ImageProtocol::Auto => {
                                if delta < 0 {
                                    crate::config::ImageProtocol::Off
                                } else {
                                    crate::config::ImageProtocol::Kitty
                                }
                            }
                            crate::config::ImageProtocol::Kitty => {
                                if delta < 0 {
                                    crate::config::ImageProtocol::Auto
                                } else {
                                    crate::config::ImageProtocol::Halfblocks
                                }
                            }
                            crate::config::ImageProtocol::Halfblocks => {
                                if delta < 0 {
                                    crate::config::ImageProtocol::Kitty
                                } else {
                                    crate::config::ImageProtocol::Off
                                }
                            }
                            crate::config::ImageProtocol::Off => {
                                if delta < 0 {
                                    crate::config::ImageProtocol::Halfblocks
                                } else {
                                    crate::config::ImageProtocol::Auto
                                }
                            }
                        };
                }
                SettingItem::HalfblockFilter => {
                    self.user_data.config.reader.halfblock_filter =
                        match self.user_data.config.reader.halfblock_filter {
                            crate::config::HalfblockFilter::Nearest => {
                                if delta < 0 {
                                    crate::config::HalfblockFilter::Lanczos3
                                } else {
                                    crate::config::HalfblockFilter::Triangle
                                }
                            }
                            crate::config::HalfblockFilter::Triangle => {
                                if delta < 0 {
                                    crate::config::HalfblockFilter::Nearest
                                } else {
                                    crate::config::HalfblockFilter::Catmullrom
                                }
                            }
                            crate::config::HalfblockFilter::Catmullrom => {
                                if delta < 0 {
                                    crate::config::HalfblockFilter::Triangle
                                } else {
                                    crate::config::HalfblockFilter::Gaussian
                                }
                            }
                            crate::config::HalfblockFilter::Gaussian => {
                                if delta < 0 {
                                    crate::config::HalfblockFilter::Catmullrom
                                } else {
                                    crate::config::HalfblockFilter::Lanczos3
                                }
                            }
                            crate::config::HalfblockFilter::Lanczos3 => {
                                if delta < 0 {
                                    crate::config::HalfblockFilter::Gaussian
                                } else {
                                    crate::config::HalfblockFilter::Nearest
                                }
                            }
                        };
                    for tab in &mut self.workspace.tabs {
                        for pane in &mut tab.panes {
                            pane.halfblock_cache.clear();
                        }
                    }
                }
                SettingItem::SearchLimit => {
                    let cur = self.user_data.config.search.limit as i32;
                    let step = 5;
                    let new_val = if delta == 0 {
                        if cur >= 50 {
                            5
                        } else {
                            cur + step
                        }
                    } else {
                        (cur + delta * step).clamp(5, 50)
                    };
                    self.user_data.config.search.limit = new_val as usize;
                }
                SettingItem::NetworkTimeout => {
                    let cur = self.user_data.config.network.timeout as i32;
                    let step = 2;
                    let new_val = if delta == 0 {
                        if cur >= 60 {
                            2
                        } else {
                            cur + step
                        }
                    } else {
                        (cur + delta * step).clamp(2, 60)
                    };
                    self.user_data.config.network.timeout = new_val as u64;
                }
                SettingItem::OfflineCache => {
                    self.user_data.config.network.offline_cache =
                        !self.user_data.config.network.offline_cache;
                }
                SettingItem::CacheLifetime => {
                    let cur = self.user_data.config.network.cache_lifetime as i32;
                    let step = 6;
                    let new_val = if delta == 0 {
                        if cur >= 168 {
                            1
                        } else {
                            cur + step
                        }
                    } else {
                        (cur + delta * step).clamp(1, 168)
                    };
                    self.user_data.config.network.cache_lifetime = new_val as u64;
                }
                SettingItem::MouseSupport => {
                    self.user_data.config.input.mouse_support =
                        !self.user_data.config.input.mouse_support;
                }
                SettingItem::ScrollSpeed => {
                    let cur = self.user_data.config.input.scroll_speed as i32;
                    let new_val = if delta == 0 {
                        if cur >= 20 {
                            1
                        } else {
                            cur + 1
                        }
                    } else {
                        (cur + delta).clamp(1, 20)
                    };
                    self.user_data.config.input.scroll_speed = new_val as usize;
                }
            }
            self.user_data.config.save();
            self.user_data.config.update_mtime();
        }
    }

    pub fn reset_selected_setting(&mut self) {
        if let Some(item) = SettingItem::ALL
            .get(self.modals.settings_modal.cursor_idx)
            .copied()
        {
            let default_config = Config::default();
            match item {
                SettingItem::LikedReadonly => {
                    self.user_data.config.general.liked_readonly =
                        default_config.general.liked_readonly;
                }
                SettingItem::AutoRestoreSession => {
                    self.user_data.config.general.auto_restore_session =
                        default_config.general.auto_restore_session;
                }
                SettingItem::ConfirmQuit => {
                    self.user_data.config.general.confirm_quit =
                        default_config.general.confirm_quit;
                }
                SettingItem::HintMode => {
                    self.user_data.config.general.hint_mode = default_config.general.hint_mode;
                }
                SettingItem::RoundedBorders => {
                    self.user_data.config.ui.rounded_borders = default_config.ui.rounded_borders;
                }
                SettingItem::Icons => {
                    self.user_data.config.ui.icons = default_config.ui.icons;
                }
                SettingItem::ScrollIndicator => {
                    self.user_data.config.ui.scroll_indicator = default_config.ui.scroll_indicator;
                }
                SettingItem::Stats => {
                    self.user_data.config.ui.stats = default_config.ui.stats;
                }
                SettingItem::DimInactivePanes => {
                    self.user_data.config.ui.dim_inactive_panes =
                        default_config.ui.dim_inactive_panes;
                }
                SettingItem::HeadingMarker => {
                    self.user_data.config.reader.heading_marker =
                        default_config.reader.heading_marker;
                }
                SettingItem::ScrollLines => {
                    self.user_data.config.reader.scroll_lines = default_config.reader.scroll_lines;
                }
                SettingItem::UnderlineLinks => {
                    self.user_data.config.reader.underline_links =
                        default_config.reader.underline_links;
                }
                SettingItem::ShowFootnotes => {
                    self.user_data.config.reader.show_footnotes =
                        default_config.reader.show_footnotes;
                }
                SettingItem::ShowExternalLinks => {
                    self.user_data.config.reader.show_external_links =
                        default_config.reader.show_external_links;
                }
                SettingItem::TocSectionNumbers => {
                    self.user_data.config.reader.toc_section_numbers =
                        default_config.reader.toc_section_numbers;
                }
                SettingItem::CodeLineNumbers => {
                    self.user_data.config.reader.code_line_numbers =
                        default_config.reader.code_line_numbers;
                }
                SettingItem::ShowImages => {
                    self.user_data.config.reader.show_images = default_config.reader.show_images;
                }
                SettingItem::ImageProtocol => {
                    self.user_data.config.reader.image_protocol =
                        default_config.reader.image_protocol;
                }
                SettingItem::HalfblockFilter => {
                    self.user_data.config.reader.halfblock_filter =
                        default_config.reader.halfblock_filter;
                    for tab in &mut self.workspace.tabs {
                        for pane in &mut tab.panes {
                            pane.halfblock_cache.clear();
                        }
                    }
                }
                SettingItem::SearchLimit => {
                    self.user_data.config.search.limit = default_config.search.limit;
                }
                SettingItem::NetworkTimeout => {
                    self.user_data.config.network.timeout = default_config.network.timeout;
                }
                SettingItem::OfflineCache => {
                    self.user_data.config.network.offline_cache =
                        default_config.network.offline_cache;
                }
                SettingItem::CacheLifetime => {
                    self.user_data.config.network.cache_lifetime =
                        default_config.network.cache_lifetime;
                }
                SettingItem::MouseSupport => {
                    self.user_data.config.input.mouse_support = default_config.input.mouse_support;
                }
                SettingItem::ScrollSpeed => {
                    self.user_data.config.input.scroll_speed = default_config.input.scroll_speed;
                }
            }
            self.user_data.config.save();
            self.user_data.config.update_mtime();
        }
    }

    pub fn reset_all_settings(&mut self) {
        self.user_data.config.current = Config::default();
        self.user_data.config.save();
        self.user_data.config.update_mtime();
    }
}
