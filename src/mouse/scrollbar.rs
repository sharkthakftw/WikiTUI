use super::types::ScrollDragTarget;
use crate::app::{App, InputMode, PaneContent};
use ratatui::layout::Rect;

pub fn active_pane_rect(app: &App, term_width: u16, term_height: u16) -> Rect {
    let size = Rect::new(0, 0, term_width, term_height);
    let main_rect = if app.workspace.zen_mode {
        crate::ui::compute_zen_area(size)
    } else {
        Rect::new(0, 1, term_width, term_height.saturating_sub(2))
    };
    let tab = app.active_tab();
    let rects = tab.layout_root.compute_rects(main_rect);
    rects
        .iter()
        .find(|(idx, _)| *idx == tab.active_pane_idx)
        .map(|(_, r)| *r)
        .unwrap_or(main_rect)
}

pub fn handle_scrollbar_down(
    app: &mut App,
    col: u16,
    row: u16,
    term_width: u16,
    term_height: u16,
) -> bool {
    let size = Rect::new(0, 0, term_width, term_height);

    if app.active_pane().toc_focused {
        let container_rect = active_pane_rect(app, term_width, term_height);
        let toc_area = crate::ui::modals::compute_toc_modal_area(container_rect);
        if col == toc_area.x + toc_area.width.saturating_sub(1)
            && row > toc_area.y
            && row < toc_area.y + toc_area.height.saturating_sub(1)
        {
            let pane = app.active_pane_mut();
            let mut dragged = false;
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                let total = parsed_doc.headings.len();
                let visible_rows = (toc_area.height.saturating_sub(2)) as usize;
                if total > visible_rows && visible_rows > 0 {
                    let rel_y = (row - (toc_area.y + 1)) as usize;
                    let target_idx = if visible_rows > 1 {
                        (rel_y * (total - 1)) / (visible_rows - 1)
                    } else {
                        0
                    };
                    pane.selected_toc_idx = Some(target_idx.min(total - 1));
                    dragged = true;
                }
            }
            if dragged {
                app.workspace.scroll_drag = Some(ScrollDragTarget::Toc);
                return true;
            }
        }
        return false;
    }

    if app.input_mode == InputMode::SavedListsViewer {
        let (_container_area, left_area, right_area) =
            crate::ui::modals::lists::compute_saved_lists_viewer_areas(size);

        if col == left_area.x + left_area.width.saturating_sub(1)
            && row > left_area.y
            && row < left_area.y + left_area.height.saturating_sub(1)
        {
            let total = app.user_data.saved_lists.lists.len();
            let visible_rows = (left_area.height.saturating_sub(2)) as usize;
            if total > visible_rows && visible_rows > 0 {
                app.workspace.scroll_drag = Some(ScrollDragTarget::SavedLists(false));
                let rel_y = (row - (left_area.y + 1)) as usize;
                let target_idx = if visible_rows > 1 {
                    (rel_y * (total - 1)) / (visible_rows - 1)
                } else {
                    0
                };
                app.modals.lists_modal.viewer_list_idx = target_idx.min(total - 1);
                app.modals.lists_modal.viewer_article_idx = 0;
                app.modals.lists_modal.viewer_focus_right = false;
                return true;
            }
        }

        if col == right_area.x + right_area.width.saturating_sub(1)
            && row > right_area.y
            && row < right_area.y + right_area.height.saturating_sub(1)
            && let Some(list) = app
                .user_data
                .saved_lists
                .lists
                .get(app.modals.lists_modal.viewer_list_idx)
            {
                let total = list.articles.len();
                let visible_rows = (right_area.height.saturating_sub(2)) as usize;
                if total > visible_rows && visible_rows > 0 {
                    app.workspace.scroll_drag = Some(ScrollDragTarget::SavedLists(true));
                    let rel_y = (row - (right_area.y + 1)) as usize;
                    let target_idx = if visible_rows > 1 {
                        (rel_y * (total - 1)) / (visible_rows - 1)
                    } else {
                        0
                    };
                    app.modals.lists_modal.viewer_article_idx = target_idx.min(total - 1);
                    app.modals.lists_modal.viewer_focus_right = true;
                    return true;
                }
            }
        return false;
    }

    if app.input_mode != InputMode::Normal
        || app.workspace.zen_mode
        || !app.user_data.config.ui.scroll_indicator
    {
        return false;
    }

    if row >= 1 && row < term_height.saturating_sub(1) {
        let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
        let tab = app.active_tab_mut();
        let rects = tab.layout_root.compute_rects(main_rect);

        let mut dragged_pane = None;
        for (pane_idx, rect) in rects {
            if col == rect.x + rect.width.saturating_sub(1)
                && row > rect.y
                && row < rect.y + rect.height.saturating_sub(1)
            {
                let pane = &mut tab.panes[pane_idx];
                let track_height = (rect.height.saturating_sub(2)) as usize;
                let rel_y = (row - (rect.y + 1)) as usize;

                match &pane.content {
                    PaneContent::ArticleText { parsed_doc, .. } => {
                        let total_lines = parsed_doc.lines.len();
                        let viewport = pane.viewport_height.max(1);
                        if total_lines > viewport && track_height > 1 {
                            tab.active_pane_idx = pane_idx;
                            let max_scroll = total_lines.saturating_sub(viewport);
                            let target_scroll = (rel_y * max_scroll) / (track_height - 1);
                            pane.scroll_offset = target_scroll.min(max_scroll);
                            dragged_pane = Some(pane_idx);
                        }
                    }
                    PaneContent::SearchResults { items, .. } => {
                        let inner_width = (rect.width as usize).saturating_sub(4);
                        let counts = crate::ui::pane_view::compute_search_result_lines_count(
                            items,
                            pane.selected_idx,
                            inner_width,
                        );
                        let total_lines: usize = counts.iter().sum();
                        let viewport = pane.viewport_height.max(1);
                        if total_lines > viewport && track_height > 1 {
                            tab.active_pane_idx = pane_idx;
                            let max_scroll = total_lines.saturating_sub(viewport);
                            let target_scroll = (rel_y * max_scroll) / (track_height - 1);
                            pane.scroll_offset = target_scroll.min(max_scroll);
                            dragged_pane = Some(pane_idx);
                        }
                    }
                    _ => {}
                }
                break;
            }
        }
        if let Some(pane_idx) = dragged_pane {
            app.workspace.scroll_drag = Some(ScrollDragTarget::Pane(pane_idx));
            return true;
        }
    }

    false
}

pub fn handle_scrollbar_drag(app: &mut App, row: u16, term_width: u16, term_height: u16) {
    let Some(target) = app.workspace.scroll_drag else {
        return;
    };
    let size = Rect::new(0, 0, term_width, term_height);

    match target {
        ScrollDragTarget::Toc => {
            if app.active_pane().toc_focused {
                let container_rect = active_pane_rect(app, term_width, term_height);
                let toc_area = crate::ui::modals::compute_toc_modal_area(container_rect);
                let pane = app.active_pane_mut();
                if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                    let total = parsed_doc.headings.len();
                    let visible_rows = (toc_area.height.saturating_sub(2)) as usize;
                    if total > visible_rows && visible_rows > 1 {
                        let rel_y = row
                            .saturating_sub(toc_area.y + 1)
                            .min((visible_rows - 1) as u16)
                            as usize;
                        let target_idx = (rel_y * (total - 1)) / (visible_rows - 1);
                        pane.selected_toc_idx = Some(target_idx.min(total - 1));
                    }
                }
            }
        }
        ScrollDragTarget::SavedLists(is_right) => {
            if app.input_mode == InputMode::SavedListsViewer {
                let (_container_area, left_area, right_area) =
                    crate::ui::modals::lists::compute_saved_lists_viewer_areas(size);
                if is_right {
                    if let Some(list) = app
                        .user_data
                        .saved_lists
                        .lists
                        .get(app.modals.lists_modal.viewer_list_idx)
                    {
                        let total = list.articles.len();
                        let visible_rows = (right_area.height.saturating_sub(2)) as usize;
                        if total > visible_rows && visible_rows > 1 {
                            let rel_y = row
                                .saturating_sub(right_area.y + 1)
                                .min((visible_rows - 1) as u16)
                                as usize;
                            let target_idx = (rel_y * (total - 1)) / (visible_rows - 1);
                            app.modals.lists_modal.viewer_article_idx = target_idx.min(total - 1);
                        }
                    }
                } else {
                    let total = app.user_data.saved_lists.lists.len();
                    let visible_rows = (left_area.height.saturating_sub(2)) as usize;
                    if total > visible_rows && visible_rows > 1 {
                        let rel_y = row
                            .saturating_sub(left_area.y + 1)
                            .min((visible_rows - 1) as u16)
                            as usize;
                        let target_idx = (rel_y * (total - 1)) / (visible_rows - 1);
                        app.modals.lists_modal.viewer_list_idx = target_idx.min(total - 1);
                    }
                }
            }
        }
        ScrollDragTarget::Pane(pane_idx) => {
            if app.input_mode == InputMode::Normal
                && !app.workspace.zen_mode
                && app.user_data.config.ui.scroll_indicator
            {
                let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
                let tab = app.active_tab_mut();
                let rects = tab.layout_root.compute_rects(main_rect);

                if let Some((_, rect)) = rects.into_iter().find(|(idx, _)| *idx == pane_idx) {
                    let pane = &mut tab.panes[pane_idx];
                    let track_height = (rect.height.saturating_sub(2)) as usize;
                    if track_height > 1 {
                        let rel_y = row
                            .saturating_sub(rect.y + 1)
                            .min((track_height - 1) as u16)
                            as usize;

                        match &pane.content {
                            PaneContent::ArticleText { parsed_doc, .. } => {
                                let total_lines = parsed_doc.lines.len();
                                let viewport = pane.viewport_height.max(1);
                                if total_lines > viewport {
                                    let max_scroll = total_lines.saturating_sub(viewport);
                                    let target_scroll = (rel_y * max_scroll) / (track_height - 1);
                                    pane.scroll_offset = target_scroll.min(max_scroll);
                                }
                            }
                            PaneContent::SearchResults { items, .. } => {
                                let inner_width = (rect.width as usize).saturating_sub(4);
                                let counts =
                                    crate::ui::pane_view::compute_search_result_lines_count(
                                        items,
                                        pane.selected_idx,
                                        inner_width,
                                    );
                                case_compute_search_scroll(
                                    pane,
                                    &counts,
                                    inner_width,
                                    rel_y,
                                    track_height,
                                );
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}

fn case_compute_search_scroll(
    pane: &mut crate::app::Pane,
    counts: &[usize],
    _inner_width: usize,
    rel_y: usize,
    track_height: usize,
) {
    let total_lines: usize = counts.iter().sum();
    let viewport = pane.viewport_height.max(1);
    if total_lines > viewport {
        let max_scroll = total_lines.saturating_sub(viewport);
        let target_scroll = (rel_y * max_scroll) / (track_height - 1);
        pane.scroll_offset = target_scroll.min(max_scroll);
    }
}
