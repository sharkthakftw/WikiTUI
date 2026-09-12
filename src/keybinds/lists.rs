use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_save_to_list_mode(app: &mut App, key: KeyEvent) {
    let custom_lists: Vec<_> = app
        .user_data
        .saved_lists
        .lists
        .iter()
        .filter(|l| l.id != "liked")
        .cloned()
        .collect();
    let total = custom_lists.len() + 1;

    match key.code {
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            app.modals.lists_modal.save_cursor_idx =
                (app.modals.lists_modal.save_cursor_idx + 1) % total;
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            if app.modals.lists_modal.save_cursor_idx == 0 {
                app.modals.lists_modal.save_cursor_idx = total.saturating_sub(1);
            } else {
                app.modals.lists_modal.save_cursor_idx -= 1;
            }
        }
        KeyCode::Char(' ') | KeyCode::Enter => {
            if app.modals.lists_modal.save_cursor_idx < custom_lists.len() {
                let list_id = custom_lists[app.modals.lists_modal.save_cursor_idx]
                    .id
                    .clone();
                let target_title = app.modals.lists_modal.target_title.clone();
                let added = app
                    .user_data
                    .saved_lists
                    .toggle_article_in_list(&list_id, &target_title);
                app.mark_active_article_read();
                app.record_article_saved(&target_title, added);
            } else {
                app.modals.search_modal.input.clear();
                app.modals.search_modal.cursor_pos = 0;
                app.modals.lists_modal.create_return_mode = InputMode::SaveToList;
                app.input_mode = InputMode::CreateNewList;
            }
        }
        KeyCode::Char('n') => {
            app.modals.search_modal.input.clear();
            app.modals.search_modal.cursor_pos = 0;
            app.modals.lists_modal.create_return_mode = InputMode::SaveToList;
            app.input_mode = InputMode::CreateNewList;
        }
        KeyCode::Esc => {
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}

pub fn handle_create_new_list_mode(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('w') | KeyCode::Char('h') | KeyCode::Backspace => {
                app.delete_word_left();
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
            app.submit_create_new_list();
        }
        KeyCode::Esc => {
            app.modals.search_modal.input.clear();
            app.modals.search_modal.cursor_pos = 0;
            app.input_mode = app.modals.lists_modal.create_return_mode.clone();
        }
        _ => {}
    }
}

pub fn handle_saved_lists_viewer_mode(app: &mut App, key: KeyEvent) {
    let lists_count = app.user_data.saved_lists.lists.len();
    let current_articles_count = app
        .user_data
        .saved_lists
        .lists
        .get(app.modals.lists_modal.viewer_list_idx)
        .map(|l| l.articles.len())
        .unwrap_or(0);

    match key.code {
        KeyCode::Left | KeyCode::Char('h') => {
            app.modals.lists_modal.viewer_focus_right = false;
        }
        KeyCode::Right | KeyCode::Char('l') => {
            if current_articles_count > 0 {
                app.modals.lists_modal.viewer_focus_right = true;
            }
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            if app.modals.lists_modal.viewer_focus_right {
                if current_articles_count > 0 {
                    app.modals.lists_modal.viewer_article_idx =
                        (app.modals.lists_modal.viewer_article_idx + 1) % current_articles_count;
                }
            } else if lists_count > 0 {
                app.modals.lists_modal.viewer_list_idx =
                    (app.modals.lists_modal.viewer_list_idx + 1) % lists_count;
                app.modals.lists_modal.viewer_article_idx = 0;
            }
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            if app.modals.lists_modal.viewer_focus_right {
                if current_articles_count > 0 {
                    if app.modals.lists_modal.viewer_article_idx == 0 {
                        app.modals.lists_modal.viewer_article_idx =
                            current_articles_count.saturating_sub(1);
                    } else {
                        app.modals.lists_modal.viewer_article_idx -= 1;
                    }
                }
            } else if lists_count > 0 {
                if app.modals.lists_modal.viewer_list_idx == 0 {
                    app.modals.lists_modal.viewer_list_idx = lists_count.saturating_sub(1);
                } else {
                    app.modals.lists_modal.viewer_list_idx -= 1;
                }
                app.modals.lists_modal.viewer_article_idx = 0;
            }
        }
        KeyCode::Enter => {
            if app.modals.lists_modal.viewer_focus_right
                && let Some(title) = app
                    .user_data
                    .saved_lists
                    .lists
                    .get(app.modals.lists_modal.viewer_list_idx)
                    .and_then(|l| l.articles.get(app.modals.lists_modal.viewer_article_idx))
                    .cloned()
                {
                    app.input_mode = InputMode::Normal;
                    if key
                        .modifiers
                        .intersects(KeyModifiers::ALT | KeyModifiers::META)
                    {
                        app.new_tab();
                    }
                    app.open_article(&title);
                }
        }
        KeyCode::Char('t') => {
            if app.modals.lists_modal.viewer_focus_right
                && let Some(title) = app
                    .user_data
                    .saved_lists
                    .lists
                    .get(app.modals.lists_modal.viewer_list_idx)
                    .and_then(|l| l.articles.get(app.modals.lists_modal.viewer_article_idx))
                    .cloned()
                {
                    app.input_mode = InputMode::Normal;
                    app.new_tab();
                    app.open_article(&title);
                }
        }
        KeyCode::Char('d') | KeyCode::Delete => {
            if app.modals.lists_modal.viewer_focus_right {
                if let Some(list) = app
                    .user_data
                    .saved_lists
                    .lists
                    .get(app.modals.lists_modal.viewer_list_idx)
                    && (!app.user_data.config.general.liked_readonly || list.id != "liked")
                    && let Some(art) =
                        list.articles.get(app.modals.lists_modal.viewer_article_idx)
                {
                    app.modals.confirm_action =
                        Some(crate::app::ConfirmAction::DeleteArticle {
                            list_id: list.id.clone(),
                            title: art.clone(),
                        });
                    app.input_mode = InputMode::Confirm;
                }
            } else if let Some(list) = app
                .user_data
                .saved_lists
                .lists
                .get(app.modals.lists_modal.viewer_list_idx)
                && list.id != "liked" {
                    app.modals.confirm_action = Some(crate::app::ConfirmAction::DeleteList {
                        list_id: list.id.clone(),
                        title: list.name.clone(),
                    });
                    app.input_mode = InputMode::Confirm;
                }
        }
        KeyCode::Char('r') => {
            if !app.modals.lists_modal.viewer_focus_right
                && let Some(list) = app
                    .user_data
                    .saved_lists
                    .lists
                    .get(app.modals.lists_modal.viewer_list_idx)
                    && list.id != "liked" {
                        app.modals.lists_modal.rename_list_id = list.id.clone();
                        app.modals.search_modal.input = list.name.clone();
                        app.modals.search_modal.cursor_pos = list.name.chars().count();
                        app.input_mode = InputMode::RenameList;
                    }
        }
        KeyCode::Char('n') => {
            app.modals.lists_modal.target_title.clear();
            app.modals.search_modal.input.clear();
            app.modals.search_modal.cursor_pos = 0;
            app.modals.lists_modal.create_return_mode = InputMode::SavedListsViewer;
            app.input_mode = InputMode::CreateNewList;
        }
        KeyCode::Char('M') | KeyCode::Esc | KeyCode::Char('q') => {
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}

pub fn handle_rename_list_mode(app: &mut App, key: KeyEvent) {
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('w') | KeyCode::Char('h') | KeyCode::Backspace => {
                app.delete_word_left();
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
            let new_name = app.modals.search_modal.input.trim().to_string();
            if !new_name.is_empty() {
                let list_id = app.modals.lists_modal.rename_list_id.clone();
                if app.user_data.saved_lists.rename_list(&list_id, &new_name) {
                    app.set_status_message(format!("renamed list to '{}'", new_name));
                }
            }
            app.modals.search_modal.input.clear();
            app.modals.search_modal.cursor_pos = 0;
            app.input_mode = InputMode::SavedListsViewer;
        }
        KeyCode::Esc => {
            app.modals.search_modal.input.clear();
            app.modals.search_modal.cursor_pos = 0;
            app.input_mode = InputMode::SavedListsViewer;
        }
        _ => {}
    }
}
