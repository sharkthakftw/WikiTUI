use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_palette_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.input_mode = InputMode::Normal;
        }
        KeyCode::Up | KeyCode::BackTab => {
            app.modals.command_palette.selected_idx = app.modals.command_palette.selected_idx.saturating_sub(1);
        }
        KeyCode::Char('p') | KeyCode::Char('k')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.modals.command_palette.selected_idx = app.modals.command_palette.selected_idx.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Tab => {
            let filtered_len = crate::palette::filter_commands(&app.modals.command_palette.query).len();
            if filtered_len > 0 {
                app.modals.command_palette.selected_idx =
                    (app.modals.command_palette.selected_idx + 1).min(filtered_len.saturating_sub(1));
            }
        }
        KeyCode::Char('n' | 'j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let filtered_len = crate::palette::filter_commands(&app.modals.command_palette.query).len();
            if filtered_len > 0 {
                app.modals.command_palette.selected_idx =
                    (app.modals.command_palette.selected_idx + 1).min(filtered_len.saturating_sub(1));
            }
        }
        KeyCode::Enter => {
            let filtered = crate::palette::filter_commands(&app.modals.command_palette.query);
            if let Some((cmd, _)) = filtered.get(app.modals.command_palette.selected_idx) {
                let action = cmd.execute;
                app.input_mode = InputMode::Normal;
                action(app);
            } else {
                let trimmed = app
                    .modals
                    .command_palette
                    .query
                    .trim()
                    .strip_prefix(':')
                    .unwrap_or(app.modals.command_palette.query.trim())
                    .trim();
                if let Some(arg) = trimmed.strip_prefix("sleep ") {
                    app.input_mode = InputMode::Normal;
                    let arg = arg.trim();
                    if arg == "off" {
                        app.cancel_audio_sleep_timer();
                    } else if let Some(secs) = crate::audio::parse_duration_to_secs(arg) {
                        app.set_audio_sleep_timer_secs(secs);
                    } else if let Ok(mins) = arg.parse::<u64>() {
                        app.set_audio_sleep_timer_secs(mins * 60);
                    } else {
                        app.set_status_message(format!("invalid sleep duration: {}", arg));
                    }
                }
            }
        }
        KeyCode::Backspace => {
            app.modals.command_palette.query.pop();
            app.modals.command_palette.selected_idx = 0;
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.modals.command_palette.query.clear();
            app.modals.command_palette.selected_idx = 0;
        }
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.modals.command_palette.query.push(c);
            app.modals.command_palette.selected_idx = 0;
        }
        _ => {}
    }
}
