use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_category_onboarding_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            let total = crate::feed::profile::POPULAR_CATEGORIES.len();
            app.modals.onboarding.cursor_idx = (app.modals.onboarding.cursor_idx + 1) % total;
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            let total = crate::feed::profile::POPULAR_CATEGORIES.len();
            if app.modals.onboarding.cursor_idx == 0 {
                app.modals.onboarding.cursor_idx = total.saturating_sub(1);
            } else {
                app.modals.onboarding.cursor_idx -= 1;
            }
        }
        KeyCode::Char(' ') => {
            if let Some(val) = app
                .modals
                .onboarding
                .selected
                .get_mut(app.modals.onboarding.cursor_idx)
            {
                *val = !*val;
            }
        }
        KeyCode::Enter => {
            app.submit_category_onboarding();
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
            app.toggle_feed_mode();
        }
        _ => {}
    }
}
