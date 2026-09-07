use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_link_peek_mode(app: &mut App, key: KeyEvent, term_height: u16) {
    match key.code {
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => {
            app.activate_selected_in_background_tab();
            app.close_link_peek();
        }
        KeyCode::Enter => {
            app.activate_selected(term_height);
            app.close_link_peek();
        }
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('p') => {
            app.close_link_peek();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            app.close_link_peek();
            app.select_next_item(term_height);
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.close_link_peek();
            app.select_prev_item(term_height);
        }
        _ => {
            app.close_link_peek();
        }
    }
}
