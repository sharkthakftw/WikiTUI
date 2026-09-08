use crate::app::{App, InputMode, SettingItem};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_settings_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char(',') => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let total = SettingItem::ALL.len();
            if total > 0 {
                app.modals.settings_modal.cursor_idx =
                    (app.modals.settings_modal.cursor_idx + 1) % total;
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let total = SettingItem::ALL.len();
            if total > 0 {
                app.modals.settings_modal.cursor_idx = if app.modals.settings_modal.cursor_idx == 0
                {
                    total - 1
                } else {
                    app.modals.settings_modal.cursor_idx - 1
                };
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            app.adjust_selected_setting(0);
        }
        KeyCode::Char('h') | KeyCode::Left => {
            app.adjust_selected_setting(-1);
        }
        KeyCode::Char('l') | KeyCode::Right => {
            app.adjust_selected_setting(1);
        }
        KeyCode::Char('r') => {
            app.reset_selected_setting();
        }
        KeyCode::Char('R') => {
            app.reset_all_settings();
        }
        _ => {}
    }
}
