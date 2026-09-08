use crate::app::{App, InputMode, PaneContent};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_categories_mode(app: &mut App, key: KeyEvent) {
    let (categories_count, current_category, current_article) = {
        let pane = app.active_pane();
        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            let total_cats = parsed_doc.categories.len();
            let selected_cat = parsed_doc
                .categories
                .get(
                    app.modals.categories_modal
                        .cursor_idx
                        .min(total_cats.saturating_sub(1)),
                )
                .cloned();
            let selected_art = selected_cat
                .as_ref()
                .and_then(|c| app.modals.categories_modal.cached_members.get(c))
                .and_then(|m| {
                    m.get(
                        app.modals.categories_modal
                            .article_cursor_idx
                            .min(m.len().saturating_sub(1)),
                    )
                    .cloned()
                });
            (total_cats, selected_cat, selected_art)
        } else {
            (0, None, None)
        }
    };

    match key.code {
        KeyCode::Tab | KeyCode::BackTab => {
            app.modals.categories_modal.focus_right = !app.modals.categories_modal.focus_right;
        }
        KeyCode::Left | KeyCode::Char('h') if key.modifiers.is_empty() => {
            app.modals.categories_modal.focus_right = false;
        }
        KeyCode::Right | KeyCode::Char('l') if key.modifiers.is_empty() => {
            app.modals.categories_modal.focus_right = true;
        }
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => {
            if app.modals.categories_modal.focus_right {
                if let Some(ref cat) = current_category {
                    if let Some(members) = app.modals.categories_modal.cached_members.get(cat) {
                        if !members.is_empty() {
                            app.modals.categories_modal.article_cursor_idx =
                                (app.modals.categories_modal.article_cursor_idx + 1) % members.len();
                        }
                    }
                }
            } else if categories_count > 0 {
                app.modals.categories_modal.cursor_idx =
                    (app.modals.categories_modal.cursor_idx + 1) % categories_count;
                app.modals.categories_modal.article_cursor_idx = 0;
                let pane = app.active_pane();
                if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                    if let Some(cat) = parsed_doc.categories.get(app.modals.categories_modal.cursor_idx) {
                        let cat = cat.clone();
                        app.fetch_category_members_if_needed(&cat);
                    }
                }
            }
        }
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() => {
            if app.modals.categories_modal.focus_right {
                if let Some(ref cat) = current_category {
                    if let Some(members) = app.modals.categories_modal.cached_members.get(cat) {
                        if !members.is_empty() {
                            if app.modals.categories_modal.article_cursor_idx == 0 {
                                app.modals.categories_modal.article_cursor_idx =
                                    members.len().saturating_sub(1);
                            } else {
                                app.modals.categories_modal.article_cursor_idx -= 1;
                            }
                        }
                    }
                }
            } else if categories_count > 0 {
                if app.modals.categories_modal.cursor_idx == 0 {
                    app.modals.categories_modal.cursor_idx = categories_count.saturating_sub(1);
                } else {
                    app.modals.categories_modal.cursor_idx -= 1;
                }
                app.modals.categories_modal.article_cursor_idx = 0;
                let pane = app.active_pane();
                if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                    if let Some(cat) = parsed_doc.categories.get(app.modals.categories_modal.cursor_idx) {
                        let cat = cat.clone();
                        app.fetch_category_members_if_needed(&cat);
                    }
                }
            }
        }
        KeyCode::Enter | KeyCode::Char('o') if key.modifiers.is_empty() => {
            if app.modals.categories_modal.focus_right {
                if let Some(article_title) = current_article {
                    app.input_mode = InputMode::Normal;
                    app.open_article(&article_title);
                }
            } else {
                app.modals.categories_modal.focus_right = true;
            }
        }
        KeyCode::Char('t') if key.modifiers.is_empty() => {
            if app.modals.categories_modal.focus_right {
                if let Some(article_title) = current_article {
                    app.input_mode = InputMode::Normal;
                    app.new_tab();
                    app.open_article(&article_title);
                }
            }
        }
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => {
            if app.modals.categories_modal.focus_right {
                if let Some(article_title) = current_article {
                    app.input_mode = InputMode::Normal;
                    app.new_tab();
                    app.open_article(&article_title);
                }
            }
        }
        KeyCode::Char('y') => {
            let copy_url = if app.modals.categories_modal.focus_right {
                current_article.map(|title| {
                    format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"))
                })
            } else {
                current_category.map(|cat| {
                    format!(
                        "https://en.wikipedia.org/wiki/Category:{}",
                        cat.replace(' ', "_")
                    )
                })
            };

            if let Some(url) = copy_url {
                if crate::clipboard::copy_to_clipboard(&url) {
                    app.set_status_message(format!("copied link: {}", url));
                } else {
                    app.set_status_message("failed to copy (no clipboard backend found)");
                }
            }
        }
        KeyCode::Char('c') | KeyCode::Esc | KeyCode::Char('q') => {
            app.input_mode = InputMode::Normal;
        }
        _ => {}
    }
}
