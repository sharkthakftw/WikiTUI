use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_sleep_timer_mode(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('w') | KeyCode::Char('h') | KeyCode::Backspace => {
                app.delete_word_left();
                return;
            }
            KeyCode::Char('c') => {
                app.search_modal.input.clear();
                app.search_modal.cursor_pos = 0;
                app.input_mode = InputMode::Normal;
                return;
            }
            _ => {}
        }
    }
    match key.code {
        KeyCode::Char(c) => {
            app.type_search_char(c);
        }
        KeyCode::Backspace => {
            app.backspace_search_char();
        }
        KeyCode::Delete => {
            app.delete_search_char();
        }
        KeyCode::Left => {
            app.move_search_cursor_left();
        }
        KeyCode::Right => {
            app.move_search_cursor_right();
        }
        KeyCode::Home => {
            app.move_search_cursor_home();
        }
        KeyCode::End => {
            app.move_search_cursor_end();
        }
        KeyCode::Enter => {
            app.submit_sleep_timer_prompt();
        }
        KeyCode::Esc => {
            app.search_modal.input.clear();
            app.search_modal.cursor_pos = 0;
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}
