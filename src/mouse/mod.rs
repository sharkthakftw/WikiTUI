pub mod clicks;
pub mod scroll;
pub mod scrollbar;
pub mod selection;
pub mod types;

pub use clicks::{handle_left_click, handle_middle_click, handle_mouse_move};
pub use scroll::handle_scroll;
pub use scrollbar::{active_pane_rect, handle_scrollbar_down, handle_scrollbar_drag};
pub use selection::{handle_selection_down, handle_selection_drag, handle_selection_up};
pub use types::ScrollDragTarget;

use crate::app::App;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent, term_width: u16, term_height: u16) {
    match mouse.kind {
        MouseEventKind::ScrollUp => {
            handle_scroll(app, -1, mouse.column, mouse.row, term_width, term_height);
        }
        MouseEventKind::ScrollDown => {
            handle_scroll(app, 1, mouse.column, mouse.row, term_width, term_height);
        }
        MouseEventKind::Down(MouseButton::Left) => {
            if !handle_scrollbar_down(app, mouse.column, mouse.row, term_width, term_height) {
                let alt = mouse
                    .modifiers
                    .contains(crossterm::event::KeyModifiers::ALT);
                let _ =
                    handle_selection_down(app, mouse.column, mouse.row, term_width, term_height);
                handle_left_click(app, mouse.column, mouse.row, term_width, term_height, alt);
            }
        }
        MouseEventKind::Down(MouseButton::Middle) => {
            handle_middle_click(app, mouse.column, mouse.row, term_width, term_height);
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if app.scroll_drag.is_some() {
                handle_scrollbar_drag(app, mouse.row, term_width, term_height);
            } else {
                handle_selection_drag(app, mouse.column, mouse.row, term_width, term_height);
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            app.scroll_drag = None;
            handle_selection_up(app);
        }
        MouseEventKind::Moved => {
            let ctrl = mouse
                .modifiers
                .contains(crossterm::event::KeyModifiers::CONTROL);
            handle_mouse_move(app, mouse.column, mouse.row, term_width, term_height, ctrl);
        }
        _ => {}
    }
}
