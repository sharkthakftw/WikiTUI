use crate::app::App;
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_image_modal_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Enter => {
            app.close_image_modal();
        }
        KeyCode::Char('y') => {
            if let Some(modal) = &app.modals.image_modal {
                let url = modal.url.clone();
                crate::clipboard::copy_to_clipboard(&url);
                app.set_status_message("copied image url to clipboard");
            }
        }
        _ => {}
    }
}
