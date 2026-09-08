use crate::app::{App, InputMode, PaneContent, SettingItem};
use ratatui::layout::Rect;

pub fn handle_scroll(
    app: &mut App,
    delta: i32,
    col: u16,
    row: u16,
    term_width: u16,
    term_height: u16,
) {
    let size = Rect::new(0, 0, term_width, term_height);

    if handle_modal_scroll(app, delta, col, row, size) {
        return;
    }

    handle_workspace_scroll(app, delta, col, row, term_width, term_height);
}

fn handle_modal_scroll(app: &mut App, delta: i32, col: u16, row: u16, size: Rect) -> bool {
    if app.user_data.feed.active {
        if delta < 0 {
            app.user_data.feed.prev_post();
        } else {
            app.user_data.feed.next_post();
            app.maybe_fetch_feed_batch();
        }
        return true;
    }

    if app.active_pane().toc_focused {
        if delta < 0 {
            app.select_prev_toc_item();
        } else {
            app.select_next_toc_item();
        }
        return true;
    }

    match &app.input_mode {
        InputMode::Settings => {
            let total = SettingItem::ALL.len();
            if total > 0 {
                if delta < 0 {
                    app.modals.settings_modal.cursor_idx =
                        if app.modals.settings_modal.cursor_idx == 0 {
                            total - 1
                        } else {
                            app.modals.settings_modal.cursor_idx - 1
                        };
                } else {
                    app.modals.settings_modal.cursor_idx =
                        (app.modals.settings_modal.cursor_idx + 1) % total;
                }
            }
            true
        }
        InputMode::ImageModal => true,
        InputMode::SaveToList => {
            let count = app
                .user_data
                .saved_lists
                .lists
                .iter()
                .filter(|l| l.id != "liked")
                .count()
                + 1;
            if count > 0 {
                if delta < 0 {
                    app.modals.lists_modal.save_cursor_idx =
                        if app.modals.lists_modal.save_cursor_idx == 0 {
                            count - 1
                        } else {
                            app.modals.lists_modal.save_cursor_idx - 1
                        };
                } else {
                    app.modals.lists_modal.save_cursor_idx =
                        (app.modals.lists_modal.save_cursor_idx + 1) % count;
                }
            }
            true
        }
        InputMode::SavedListsViewer => {
            let (_container_area, left_area, right_area) =
                crate::ui::modals::lists::compute_saved_lists_viewer_areas(size);

            if col >= right_area.x
                && col < right_area.x + right_area.width
                && row >= right_area.y
                && row < right_area.y + right_area.height
            {
                app.modals.lists_modal.viewer_focus_right = true;
            } else if col >= left_area.x
                && col < left_area.x + left_area.width
                && row >= left_area.y
                && row < left_area.y + left_area.height
            {
                app.modals.lists_modal.viewer_focus_right = false;
            }

            let lists_count = app.user_data.saved_lists.lists.len();
            let current_articles_count = app
                .user_data
                .saved_lists
                .lists
                .get(app.modals.lists_modal.viewer_list_idx)
                .map(|l| l.articles.len())
                .unwrap_or(0);

            if app.modals.lists_modal.viewer_focus_right {
                if current_articles_count > 0 {
                    if delta < 0 {
                        app.modals.lists_modal.viewer_article_idx =
                            app.modals.lists_modal.viewer_article_idx.saturating_sub(1);
                    } else {
                        app.modals.lists_modal.viewer_article_idx =
                            (app.modals.lists_modal.viewer_article_idx + 1)
                                .min(current_articles_count - 1);
                    }
                }
            } else if lists_count > 0 {
                if delta < 0 {
                    app.modals.lists_modal.viewer_list_idx =
                        app.modals.lists_modal.viewer_list_idx.saturating_sub(1);
                } else {
                    app.modals.lists_modal.viewer_list_idx =
                        (app.modals.lists_modal.viewer_list_idx + 1).min(lists_count - 1);
                }
                app.modals.lists_modal.viewer_article_idx = 0;
            }
            true
        }
        InputMode::CategoryOnboarding => {
            let total = crate::feed::profile::POPULAR_CATEGORIES.len();
            if total > 0 {
                if delta < 0 {
                    app.modals.onboarding.cursor_idx = if app.modals.onboarding.cursor_idx == 0 {
                        total - 1
                    } else {
                        app.modals.onboarding.cursor_idx - 1
                    };
                } else {
                    app.modals.onboarding.cursor_idx =
                        (app.modals.onboarding.cursor_idx + 1) % total;
                }
            }
            true
        }
        InputMode::Categories => {
            let total = match &app.active_pane().content {
                PaneContent::ArticleText { parsed_doc, .. } => parsed_doc.categories.len(),
                _ => 0,
            };
            if total > 0 {
                if delta < 0 {
                    app.modals.categories_modal.cursor_idx =
                        if app.modals.categories_modal.cursor_idx == 0 {
                            total - 1
                        } else {
                            app.modals.categories_modal.cursor_idx - 1
                        };
                } else {
                    app.modals.categories_modal.cursor_idx =
                        (app.modals.categories_modal.cursor_idx + 1) % total;
                }
            }
            true
        }
        InputMode::DailyFeedModal => {
            if let Some(modal) = &mut app.modals.daily_feed_modal {
                if delta < 0 {
                    modal.scroll = modal.scroll.saturating_sub(2);
                } else {
                    modal.scroll = modal.scroll.saturating_add(2);
                }
            }
            true
        }
        _ => false,
    }
}

fn handle_workspace_scroll(
    app: &mut App,
    delta: i32,
    col: u16,
    row: u16,
    term_width: u16,
    term_height: u16,
) {
    if matches!(app.input_mode, InputMode::Normal | InputMode::LocalSearch) {
        if !app.workspace.zen_mode && row >= 1 && row < term_height.saturating_sub(1) {
            let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
            let tab = app.active_tab_mut();
            let rects = tab.layout_root.compute_rects(main_rect);
            if let Some(&(pane_idx, _)) = rects.iter().find(|(_, r)| {
                col >= r.x && col < r.x + r.width && row >= r.y && row < r.y + r.height
            }) {
                tab.active_pane_idx = pane_idx;
            }
        }
        let speed = app.user_data.config.input.scroll_speed.max(1);
        if delta < 0 {
            app.scroll_up_lines(speed, term_height);
        } else {
            app.scroll_down_lines(speed, term_height);
        }
    }
}
