pub mod categories;
pub mod daily_feed;
pub mod help;
pub mod image;
pub mod lists;
pub mod onboarding;
pub mod palette;
pub mod peek;
pub mod qr;
pub mod search;
pub mod settings;
pub mod toc;
pub mod utils;

pub use categories::{
    compute_categories_modal_areas, get_category_item_at, render_categories_modal,
};
pub use daily_feed::{
    compute_daily_feed_modal_area, get_daily_feed_item_at, get_daily_feed_link_at,
    get_feed_entries, get_modal_item_line_offset, get_ongoing_links, get_otd_tab_at,
    get_recent_deaths_links, parse_onthisday_event, parse_story_html, render_daily_feed_modal,
    DailyFeedCache, DailyFeedKind, DailyFeedModalState, FeedEntry, OnThisDayTab,
};
pub use help::{compute_help_modal_area, render_help_modal};
pub use image::{compute_image_modal_area, render_image_modal};
pub use lists::{
    compute_confirm_modal_area, compute_save_to_list_modal_area, compute_saved_lists_viewer_areas,
    render_confirm_modal, render_save_to_list_modal, render_saved_lists_viewer_modal,
};
pub use onboarding::{compute_onboarding_modal_area, render_category_onboarding_modal};
pub use palette::{compute_palette_modal_area, render_palette_modal};
pub use peek::render_link_peek;
pub use qr::{compute_qr_modal_area, render_qr_modal};
pub use search::{compute_search_modal_area, render_search_modal};
pub use settings::{compute_settings_modal_area, render_settings_modal};
pub use toc::{compute_toc_modal_area, render_toc_modal};
pub use utils::{centered_rect, compute_centered_scroll};
