use crate::app::{App, InputMode};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_confirm_mode(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Enter => match app.modals.confirm_action.take() {
            Some(crate::app::ConfirmAction::DeleteList { list_id, .. }) => {
                app.user_data.saved_lists.delete_list(&list_id);
                let lists_count = app.user_data.saved_lists.lists.len();
                app.modals.lists_modal.viewer_list_idx = app
                    .modals
                    .lists_modal
                    .viewer_list_idx
                    .min(lists_count.saturating_sub(1));
                app.modals.lists_modal.viewer_article_idx = 0;
                app.input_mode = InputMode::SavedListsViewer;
            }
            Some(crate::app::ConfirmAction::DeleteArticle { list_id, title }) => {
                app.user_data
                    .saved_lists
                    .remove_article_from_list(&list_id, &title);
                if list_id == "liked" {
                    app.user_data.feed.profile.liked_articles.remove(&title);
                    if app.user_data.feed.profile.total_likes > 0 {
                        app.user_data.feed.profile.total_likes -= 1;
                    }
                    app.user_data.feed.profile.save();
                    for item in &mut app.user_data.feed.items {
                        if item.title == title {
                            item.is_liked = false;
                        }
                    }
                }
                let articles_count = app
                    .user_data
                    .saved_lists
                    .lists
                    .get(app.modals.lists_modal.viewer_list_idx)
                    .map(|l| l.articles.len())
                    .unwrap_or(0);
                app.modals.lists_modal.viewer_article_idx = app
                    .modals
                    .lists_modal
                    .viewer_article_idx
                    .min(articles_count.saturating_sub(1));
                app.input_mode = InputMode::SavedListsViewer;
            }
            Some(crate::app::ConfirmAction::ResetFeed) => {
                app.reset_feed();
            }
            Some(crate::app::ConfirmAction::Quit) => {
                app.save_session();
                app.running = false;
            }
            None => {
                app.input_mode = InputMode::Normal;
            }
        },
        KeyCode::Char('n') | KeyCode::Esc => {
            let action = app.modals.confirm_action.take();
            if matches!(
                action,
                Some(crate::app::ConfirmAction::ResetFeed) | Some(crate::app::ConfirmAction::Quit)
            ) {
                app.input_mode = InputMode::Normal;
            } else {
                app.input_mode = InputMode::SavedListsViewer;
            }
        }
        _ => {}
    }
}
