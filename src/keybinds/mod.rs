pub mod categories;
pub mod confirm;
pub mod daily_feed;
pub mod help;
pub mod image;
pub mod lists;
pub mod normal;
pub mod onboarding;
pub mod palette;
pub mod peek;
pub mod qr;
pub mod search;
pub mod settings;
pub mod sleep_timer;

use crate::app::{App, InputMode};
use crossterm::event::KeyEvent;

pub fn handle_key_event(app: &mut App, key: KeyEvent, term_width: u16, term_height: u16) {
    match &app.input_mode {
        InputMode::CategoryOnboarding => onboarding::handle_category_onboarding_mode(app, key),
        InputMode::SaveToList => lists::handle_save_to_list_mode(app, key),
        InputMode::CreateNewList => lists::handle_create_new_list_mode(app, key),
        InputMode::SavedListsViewer => lists::handle_saved_lists_viewer_mode(app, key),
        InputMode::RenameList => lists::handle_rename_list_mode(app, key),
        InputMode::Confirm => confirm::handle_confirm_mode(app, key),
        InputMode::Settings => settings::handle_settings_mode(app, key),
        InputMode::Categories => categories::handle_categories_mode(app, key),
        InputMode::DailyFeedModal => daily_feed::handle_daily_feed_mode(app, key),
        InputMode::CommandPalette => palette::handle_palette_mode(app, key),
        InputMode::QrModal => qr::handle_qr_mode(app, key),
        InputMode::ImageModal => image::handle_image_modal_mode(app, key),
        InputMode::LinkPeek => peek::handle_link_peek_mode(app, key, term_height),
        InputMode::Help => help::handle_help_mode(app, key),
        InputMode::LocalSearch => search::handle_local_search_mode(app, key, term_height),
        InputMode::Search => search::handle_search_mode(app, key),
        InputMode::SleepTimerPrompt => sleep_timer::handle_sleep_timer_mode(app, key),
        InputMode::Normal => normal::handle_normal_mode(app, key, term_width, term_height),
    }
}
