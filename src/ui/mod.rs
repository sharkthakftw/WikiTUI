pub mod feed;
pub mod launch_screen;
pub mod modals;
pub mod pane_view;
pub mod status_bar;
pub mod tab_bar;
pub mod utils;

pub use utils::{truncate_to_width, truncate_with_ellipsis};

use crate::app::{App, InputMode};
use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

pub const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

pub fn current_spinner_frame() -> &'static str {
    let elapsed_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let frame_idx = ((elapsed_ms / 80) % (SPINNER_FRAMES.len() as u128)) as usize;
    SPINNER_FRAMES[frame_idx]
}

pub fn compute_zen_area(size: ratatui::layout::Rect) -> ratatui::layout::Rect {
    modals::centered_rect(80, 90, size)
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.size();

    if app.user_data.feed.active {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(size);

        let feed_area = chunks[0];
        let status_area = chunks[1];

        feed::render_feed_view(f, &app.user_data.feed, feed_area, app.user_data.config.ui.rounded_borders);
        status_bar::render(f, app, status_area);
    } else if app.workspace.zen_mode {
        let zen_area = compute_zen_area(size);
        pane_view::render_single_active_pane(f, app, zen_area);
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(size);

        let tab_bar_area = chunks[0];
        let main_area = chunks[1];
        let status_area = chunks[2];

        tab_bar::render(f, app, tab_bar_area);
        status_bar::render(f, app, status_area);
        pane_view::render_panes(f, app, main_area);
    }

    if app.input_mode == InputMode::Help {
        modals::render_help_modal(f, app, size);
    }

    if app.input_mode == InputMode::Search
        || app.input_mode == InputMode::RenameList
        || app.input_mode == InputMode::CreateNewList
        || app.input_mode == InputMode::SleepTimerPrompt
    {
        modals::render_search_modal(f, app, size);
    }

    if app.input_mode == InputMode::CategoryOnboarding {
        modals::render_category_onboarding_modal(f, app, size);
    }

    if app.input_mode == InputMode::SaveToList {
        modals::render_save_to_list_modal(f, app, size);
    }

    if app.input_mode == InputMode::SavedListsViewer {
        modals::render_saved_lists_viewer_modal(f, app, size);
    }

    if app.input_mode == InputMode::Confirm {
        modals::render_confirm_modal(f, app, size);
    }

    if app.input_mode == InputMode::Settings {
        modals::render_settings_modal(f, app, size);
    }

    if app.input_mode == InputMode::Categories {
        modals::render_categories_modal(f, app, size);
    }

    if app.input_mode == InputMode::DailyFeedModal {
        modals::render_daily_feed_modal(f, app, size);
    }

    if app.input_mode == InputMode::CommandPalette {
        modals::render_palette_modal(f, app, size);
    }

    if app.input_mode == InputMode::QrModal {
        modals::render_qr_modal(f, app, size);
    }

    let is_modal_open = (app.input_mode != InputMode::Normal
        && app.input_mode != InputMode::LocalSearch
        && app.input_mode != InputMode::LinkPeek)
        || app.workspace.tabs.iter().any(|t| t.panes.iter().any(|p| p.show_toc))
        || app.modals.daily_feed_modal.is_some();

    if is_modal_open {
        app.graphics.pending_image_renders.clear();
    } else if let Some(peek) = &app.modals.link_peek {
        let peek_area = modals::compute_link_peek_area(peek, size);
        app.graphics.pending_image_renders.retain(|task| {
            task.screen_x >= peek_area.x + peek_area.width
                || task.screen_x + task.cols <= peek_area.x
                || task.screen_y >= peek_area.y + peek_area.height
                || task.screen_y + task.rows <= peek_area.y
        });
    }

    if app.input_mode == InputMode::ImageModal {
        modals::render_image_modal(f, app, size);
    }

    if app.input_mode == InputMode::LinkPeek {
        modals::render_link_peek(f, app, size);
    }
}
