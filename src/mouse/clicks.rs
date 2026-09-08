use super::scrollbar::active_pane_rect;
use crate::app::{App, InputMode, PaneContent};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[inline]
fn rect_contains(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
}

pub fn handle_left_click(
    app: &mut App,
    col: u16,
    row: u16,
    term_width: u16,
    term_height: u16,
    alt: bool,
) {
    let size = Rect::new(0, 0, term_width, term_height);

    if handle_modal_left_click(app, col, row, size, term_width, term_height, alt) {
        return;
    }

    handle_workspace_left_click(app, col, row, term_width, term_height, alt);
}

pub fn handle_middle_click(app: &mut App, col: u16, row: u16, term_width: u16, term_height: u16) {
    let size = Rect::new(0, 0, term_width, term_height);

    if row == 0 {
        if let Some(tab_idx) = crate::ui::tab_bar::get_tab_at_col(app, term_width, col) {
            app.close_tab(tab_idx);
        }
        return;
    }

    if app.user_data.feed.active {
        if let Some(item) = app.user_data.feed.current_item().cloned() {
            app.open_article_in_background_tab(&item.title);
        }
        return;
    }

    if app.input_mode == InputMode::DailyFeedModal {
        if let Some((_, _, target)) = crate::ui::modals::get_daily_feed_item_at(app, col, row, size)
        {
            app.open_article_in_background_tab(&target);
        }
        return;
    }

    if app.input_mode != InputMode::Normal {
        return;
    }

    if app.workspace.zen_mode {
        let zen_rect = crate::ui::compute_zen_area(Rect::new(0, 0, term_width, term_height));
        if rect_contains(zen_rect, col, row) {
            let pane = app.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                if let Some(link_idx) = crate::ui::pane_view::get_link_at_coord(
                    parsed_doc,
                    pane.scroll_offset,
                    zen_rect,
                    col,
                    row,
                ) {
                    pane.selected_link_idx = Some(link_idx);
                    app.activate_selected_in_background_tab();
                }
            }
        }
        return;
    }

    if row >= 1 && row < term_height.saturating_sub(1) {
        let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
        let tab = app.active_tab_mut();
        let rects = tab.layout_root.compute_rects(main_rect);

        for (pane_idx, rect) in rects {
            if rect_contains(rect, col, row) {
                tab.active_pane_idx = pane_idx;
                let pane = &mut tab.panes[pane_idx];
                match &pane.content {
                    PaneContent::SearchResults { items, .. } => {
                        let inner_y = rect.y + 1;
                        if row >= inner_y && row < rect.y + rect.height.saturating_sub(1) {
                            let row_in_pane = (row - inner_y) as usize;
                            let clicked_line = pane.scroll_offset + row_in_pane;
                            let inner_width = (rect.width as usize).saturating_sub(4);
                            if let Some(item_idx) = crate::ui::pane_view::get_search_result_at_line(
                                items,
                                pane.selected_idx,
                                inner_width,
                                clicked_line,
                            ) {
                                pane.selected_idx = item_idx;
                                let title = items[item_idx].title.clone();
                                app.open_article_in_background_tab(&title);
                            }
                        }
                    }
                    PaneContent::Empty => {
                        let recent_articles = app.get_continue_reading_articles();
                        let inner_height = (rect.height as usize).saturating_sub(2);
                        let show_recent = !recent_articles.is_empty()
                            && inner_height >= (crate::ui::launch_screen::LOGO.len() + 8);

                        if show_recent {
                            let displayed_count = recent_articles.len().min(7);
                            let total_content_height =
                                crate::ui::launch_screen::LOGO.len() + 4 + displayed_count + 2;
                            let v_pad = inner_height.saturating_sub(total_content_height) / 2;
                            let start_row = rect.y
                                + 1
                                + (v_pad as u16)
                                + (crate::ui::launch_screen::LOGO.len() as u16)
                                + 6;

                            if row >= start_row && row < start_row + (displayed_count as u16) {
                                let idx = (row - start_row) as usize;
                                if idx < recent_articles.len() {
                                    let title = recent_articles[idx].clone();
                                    app.open_article_in_background_tab(&title);
                                }
                            }
                        }
                    }
                    PaneContent::ArticleText { parsed_doc, .. } => {
                        if let Some(link_idx) = crate::ui::pane_view::get_link_at_coord(
                            parsed_doc,
                            pane.scroll_offset,
                            rect,
                            col,
                            row,
                        ) {
                            pane.selected_link_idx = Some(link_idx);
                            app.activate_selected_in_background_tab();
                        }
                    }
                    _ => {}
                }
                break;
            }
        }
    }
}

fn handle_modal_left_click(
    app: &mut App,
    col: u16,
    row: u16,
    size: Rect,
    term_width: u16,
    term_height: u16,
    alt: bool,
) -> bool {
    if app.user_data.feed.active {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(size);
        let inner_area = chunks[0];
        let card_area = crate::ui::feed::compute_feed_card_area(inner_area);

        if row == card_area.y + card_area.height.saturating_sub(1)
            && col >= card_area.x + card_area.width.saturating_sub(14)
        {
            app.toggle_feed_like();
        } else if let Some(item) = app.user_data.feed.current_item().cloned() {
            app.user_data.feed.active = false;
            if alt {
                app.new_tab();
            }
            app.open_article(&item.title);
        }
        return true;
    }

    if app.input_mode == InputMode::DailyFeedModal {
        if let Some(modal) = &mut app.modals.daily_feed_modal {
            let area = crate::ui::modals::compute_daily_feed_modal_area(size, modal.kind);
            if !rect_contains(area, col, row) {
                app.close_daily_feed_modal();
                return true;
            }

            if modal.kind == crate::ui::modals::DailyFeedKind::OnThisDay {
                if let Some(tab) =
                    crate::ui::modals::get_otd_tab_at(area, col, row, app.daily_feed.as_ref())
                {
                    if modal.otd_tab != tab {
                        modal.otd_tab = tab;
                        modal.cursor_idx = 0;
                        modal.link_idx = 0;
                        modal.scroll = 0;
                    }
                    return true;
                }
            }

            if let Some(target) = crate::ui::modals::get_daily_feed_link_at(app, col, row, size) {
                if !target.is_empty() {
                    app.close_daily_feed_modal();
                    if alt {
                        app.new_tab();
                    }
                    app.open_article(&target);
                    return true;
                }
            }
        }
        return true;
    }

    if app.input_mode == InputMode::Help {
        let help_area = crate::ui::modals::compute_help_modal_area(size);
        if !rect_contains(help_area, col, row) {
            app.input_mode = InputMode::Normal;
        }
        return true;
    }

    if app.input_mode == InputMode::QrModal {
        let qr_area = crate::ui::modals::compute_qr_modal_area(size);
        if !rect_contains(qr_area, col, row) {
            app.close_qr_modal();
        }
        return true;
    }

    if app.input_mode == InputMode::ImageModal {
        let image_area = crate::ui::modals::compute_image_modal_area(size);
        if !rect_contains(image_area, col, row) {
            app.close_image_modal();
        }
        return true;
    }

    if app.input_mode == InputMode::Search {
        let search_area = crate::ui::modals::search::compute_search_modal_area(size);
        if !rect_contains(search_area, col, row) {
            app.input_mode = InputMode::Normal;
        }
        return true;
    }

    if app.input_mode == InputMode::CreateNewList {
        let create_area = crate::ui::modals::compute_search_modal_area(size);
        if !rect_contains(create_area, col, row) {
            app.modals.search_modal.input.clear();
            app.modals.search_modal.cursor_pos = 0;
            app.input_mode = app.modals.lists_modal.create_return_mode.clone();
        }
        return true;
    }

    if app.input_mode == InputMode::SleepTimerPrompt {
        let prompt_area = crate::ui::modals::compute_search_modal_area(size);
        if !rect_contains(prompt_area, col, row) {
            app.modals.search_modal.input.clear();
            app.modals.search_modal.cursor_pos = 0;
            app.input_mode = InputMode::Normal;
        }
        return true;
    }

    if app.input_mode == InputMode::Settings {
        let area = crate::ui::modals::compute_settings_modal_area(size);
        let inner = Rect::new(
            area.x + 1,
            area.y + 1,
            area.width.saturating_sub(2),
            area.height.saturating_sub(2),
        );

        if !rect_contains(area, col, row) {
            app.input_mode = InputMode::Normal;
            return true;
        }

        if let Some((idx, item, val_start_x)) = crate::ui::modals::settings::get_setting_row_at(
            inner,
            row,
            app.modals.settings_modal.cursor_idx,
        ) {
            app.modals.settings_modal.cursor_idx = idx;
            let is_numeric = matches!(
                item,
                crate::app::SettingItem::ScrollLines
                    | crate::app::SettingItem::SearchLimit
                    | crate::app::SettingItem::NetworkTimeout
                    | crate::app::SettingItem::CacheLifetime
                    | crate::app::SettingItem::ScrollSpeed
            );
            if is_numeric {
                if col >= val_start_x {
                    let rel_col = col - val_start_x;
                    if rel_col <= 3 {
                        app.adjust_selected_setting(-1);
                    } else if rel_col >= 11 {
                        app.adjust_selected_setting(1);
                    } else {
                        app.adjust_selected_setting(0);
                    }
                }
            } else {
                app.adjust_selected_setting(0);
            }
        }
        return true;
    }

    if app.input_mode == InputMode::CategoryOnboarding {
        let area = crate::ui::modals::compute_onboarding_modal_area(size);
        if rect_contains(area, col, row) {
            match crate::ui::modals::onboarding::get_onboarding_row_at(area, row) {
                Some(crate::ui::modals::onboarding::OnboardingHit::Category(idx)) => {
                    app.modals.onboarding.cursor_idx = idx;
                    if let Some(val) = app.modals.onboarding.selected.get_mut(idx) {
                        *val = !*val;
                    }
                }
                Some(crate::ui::modals::onboarding::OnboardingHit::Submit) => {
                    app.submit_category_onboarding();
                }
                None => {}
            }
        } else {
            app.input_mode = InputMode::Normal;
        }
        return true;
    }

    if app.input_mode == InputMode::Categories {
        let (container_area, left_area, right_area) =
            crate::ui::modals::compute_categories_modal_areas(size);

        if rect_contains(container_area, col, row) {
            if rect_contains(left_area, col, row) {
                app.modals.categories_modal.focus_right = false;
                if let Some(clicked_cat_idx) =
                    crate::ui::modals::get_category_item_at(app, false, left_area, row)
                {
                    app.modals.categories_modal.cursor_idx = clicked_cat_idx;
                    app.modals.categories_modal.article_cursor_idx = 0;
                    let pane = app.active_pane();
                    if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                        if let Some(cat) = parsed_doc.categories.get(clicked_cat_idx) {
                            let cat = cat.clone();
                            app.fetch_category_members_if_needed(&cat);
                        }
                    }
                }
                return true;
            }

            if rect_contains(right_area, col, row) {
                app.modals.categories_modal.focus_right = true;
                if let Some(clicked_art_idx) =
                    crate::ui::modals::get_category_item_at(app, true, right_area, row)
                {
                    let pane = app.active_pane();
                    let target_title =
                        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                            let selected_cat_idx = app
                                .modals
                                .categories_modal
                                .cursor_idx
                                .min(parsed_doc.categories.len().saturating_sub(1));
                            parsed_doc.categories.get(selected_cat_idx).and_then(|cat| {
                                app.modals
                                    .categories_modal
                                    .cached_members
                                    .get(cat)
                                    .and_then(|members| members.get(clicked_art_idx).cloned())
                            })
                        } else {
                            None
                        };

                    if let Some(title) = target_title {
                        app.modals.categories_modal.article_cursor_idx = clicked_art_idx;
                        app.input_mode = InputMode::Normal;
                        if alt {
                            app.new_tab();
                        }
                        app.open_article(&title);
                    }
                }
                return true;
            }
        } else {
            app.input_mode = InputMode::Normal;
        }
        return true;
    }

    if app.input_mode == InputMode::SaveToList {
        let area = crate::ui::modals::compute_save_to_list_modal_area(size);
        if rect_contains(area, col, row) {
            match crate::ui::modals::lists::get_save_to_list_item_at(app, area, row) {
                Some(crate::ui::modals::lists::SaveToListHit::Toggle(idx)) => {
                    app.modals.lists_modal.save_cursor_idx = idx;
                    let custom_lists: Vec<_> = app
                        .user_data
                        .saved_lists
                        .lists
                        .iter()
                        .filter(|l| l.id != "liked")
                        .cloned()
                        .collect();
                    if let Some(list) = custom_lists.get(idx) {
                        let list_id = list.id.clone();
                        let target_title = app.modals.lists_modal.target_title.clone();
                        app.user_data
                            .saved_lists
                            .toggle_article_in_list(&list_id, &target_title);
                    }
                }
                Some(crate::ui::modals::lists::SaveToListHit::CreateNew) => {
                    let custom_lists_count = app
                        .user_data
                        .saved_lists
                        .lists
                        .iter()
                        .filter(|l| l.id != "liked")
                        .count();
                    app.modals.lists_modal.save_cursor_idx = custom_lists_count;
                    app.modals.search_modal.input.clear();
                    app.modals.search_modal.cursor_pos = 0;
                    app.modals.lists_modal.create_return_mode = InputMode::SaveToList;
                    app.input_mode = InputMode::CreateNewList;
                }
                None => {}
            }
        } else {
            app.input_mode = InputMode::Normal;
        }
        return true;
    }

    if app.input_mode == InputMode::Confirm {
        let area = crate::ui::modals::compute_confirm_modal_area(size);
        if rect_contains(area, col, row) {
            if let Some(c) = crate::ui::modals::lists::get_confirm_button_at(app, area, col, row) {
                crate::keybinds::confirm::handle_confirm_mode(
                    app,
                    crossterm::event::KeyEvent::new(
                        crossterm::event::KeyCode::Char(c),
                        crossterm::event::KeyModifiers::empty(),
                    ),
                );
            }
        } else {
            app.input_mode = InputMode::Normal;
            app.modals.confirm_action = None;
        }
        return true;
    }

    if app.active_pane().toc_focused {
        let container_rect = active_pane_rect(app, term_width, term_height);
        let toc_area = crate::ui::modals::compute_toc_modal_area(container_rect);
        if rect_contains(toc_area, col, row) {
            let pane = app.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                let current_scroll = pane.scroll_offset;
                let active_heading_idx = parsed_doc
                    .headings
                    .iter()
                    .rposition(|h| h.line_idx <= current_scroll)
                    .unwrap_or(0);
                let selected_idx = pane.selected_toc_idx.unwrap_or(active_heading_idx);

                if let Some(clicked_idx) = crate::ui::modals::toc::get_toc_heading_at(
                    parsed_doc,
                    selected_idx,
                    toc_area,
                    row,
                ) {
                    pane.selected_toc_idx = Some(clicked_idx);
                    app.activate_toc_selection(term_height);
                }
            }
        } else {
            app.active_pane_mut().toc_focused = false;
        }
        return true;
    }

    if app.input_mode == InputMode::SavedListsViewer {
        let (container_area, left_area, right_area) =
            crate::ui::modals::lists::compute_saved_lists_viewer_areas(size);

        if rect_contains(container_area, col, row) {
            if rect_contains(left_area, col, row) {
                app.modals.lists_modal.viewer_focus_right = false;
                if let Some(clicked_list_idx) =
                    crate::ui::modals::lists::get_saved_lists_viewer_item_at(
                        app, false, left_area, row,
                    )
                {
                    app.modals.lists_modal.viewer_list_idx = clicked_list_idx;
                    app.modals.lists_modal.viewer_article_idx = 0;
                }
                return true;
            }

            if rect_contains(right_area, col, row) {
                app.modals.lists_modal.viewer_focus_right = true;
                if let Some(clicked_art_idx) =
                    crate::ui::modals::lists::get_saved_lists_viewer_item_at(
                        app, true, right_area, row,
                    )
                {
                    if let Some(list) = app.user_data.saved_lists.lists.get(app.modals.lists_modal.viewer_list_idx) {
                        if clicked_art_idx < list.articles.len() {
                            app.modals.lists_modal.viewer_article_idx = clicked_art_idx;
                            let title = list.articles[clicked_art_idx].clone();
                            app.input_mode = InputMode::Normal;
                            if alt {
                                app.new_tab();
                            }
                            app.open_article(&title);
                        }
                    }
                }
                return true;
            }
        } else {
            app.input_mode = InputMode::Normal;
        }
        return true;
    }

    false
}

fn handle_workspace_left_click(
    app: &mut App,
    col: u16,
    row: u16,
    term_width: u16,
    term_height: u16,
    alt: bool,
) {
    if app.input_mode != InputMode::Normal {
        return;
    }

    if app.workspace.zen_mode {
        let zen_rect = crate::ui::compute_zen_area(Rect::new(0, 0, term_width, term_height));
        if rect_contains(zen_rect, col, row) {
            let pane = app.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                if let Some(link_idx) = crate::ui::pane_view::get_link_at_coord(
                    parsed_doc,
                    pane.scroll_offset,
                    zen_rect,
                    col,
                    row,
                ) {
                    pane.selected_link_idx = Some(link_idx);
                    if alt {
                        app.activate_selected_in_new_tab();
                    } else {
                        app.activate_selected(term_height);
                    }
                } else if let Some(img) = crate::ui::pane_view::get_image_at_coord(
                    parsed_doc,
                    pane.scroll_offset,
                    zen_rect,
                    col,
                    row,
                ) {
                    let path = pane
                        .loaded_images
                        .get(&img.url)
                        .cloned()
                        .or_else(|| crate::graphics::cache::get_cached_image_path(&img.url));
                    app.open_image_modal(img.url, img.alt, img.caption, path);
                }
            }
        }
        return;
    }

    if row == 0 {
        if let Some(tab_idx) = crate::ui::tab_bar::get_tab_at_col(app, term_width, col) {
            app.switch_to_tab(tab_idx);
        }
        return;
    }

    if row >= 1 && row < term_height.saturating_sub(1) {
        let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
        let tab = app.active_tab_mut();
        let rects = tab.layout_root.compute_rects(main_rect);

        for (pane_idx, rect) in rects {
            if rect_contains(rect, col, row) {
                tab.active_pane_idx = pane_idx;

                let pane = &mut tab.panes[pane_idx];
                match &pane.content {
                    PaneContent::SearchResults { items, .. } => {
                        let inner_y = rect.y + 1;
                        if row >= inner_y && row < rect.y + rect.height.saturating_sub(1) {
                            let row_in_pane = (row - inner_y) as usize;
                            let clicked_line = pane.scroll_offset + row_in_pane;
                            let inner_width = (rect.width as usize).saturating_sub(4);
                            if let Some(item_idx) = crate::ui::pane_view::get_search_result_at_line(
                                items,
                                pane.selected_idx,
                                inner_width,
                                clicked_line,
                            ) {
                                pane.selected_idx = item_idx;
                                let title = items[item_idx].title.clone();
                                if alt {
                                    app.new_tab();
                                }
                                app.open_article(&title);
                            }
                        }
                    }
                    PaneContent::Empty => {
                        let recent_articles = app.get_continue_reading_articles();
                        let inner_height = (rect.height as usize).saturating_sub(2);
                        let show_recent = !recent_articles.is_empty()
                            && inner_height >= (crate::ui::launch_screen::LOGO.len() + 8);

                        if show_recent {
                            let displayed_count = recent_articles.len().min(7);
                            let total_content_height =
                                crate::ui::launch_screen::LOGO.len() + 4 + displayed_count + 2;
                            let v_pad = inner_height.saturating_sub(total_content_height) / 2;
                            let start_row = rect.y
                                + 1
                                + (v_pad as u16)
                                + (crate::ui::launch_screen::LOGO.len() as u16)
                                + 6;

                            if row >= start_row && row < start_row + (displayed_count as u16) {
                                let idx = (row - start_row) as usize;
                                if idx < recent_articles.len() {
                                    let title = recent_articles[idx].clone();
                                    if alt {
                                        app.new_tab();
                                    }
                                    app.open_article(&title);
                                }
                            }
                        }
                    }
                    PaneContent::ArticleText { parsed_doc, .. } => {
                        if let Some(link_idx) = crate::ui::pane_view::get_link_at_coord(
                            parsed_doc,
                            pane.scroll_offset,
                            rect,
                            col,
                            row,
                        ) {
                            pane.selected_link_idx = Some(link_idx);
                            if alt {
                                app.activate_selected_in_new_tab();
                            } else {
                                app.activate_selected(term_height);
                            }
                        } else if let Some(img) = crate::ui::pane_view::get_image_at_coord(
                            parsed_doc,
                            pane.scroll_offset,
                            rect,
                            col,
                            row,
                        ) {
                            let path = pane.loaded_images.get(&img.url).cloned().or_else(|| {
                                crate::graphics::cache::get_cached_image_path(&img.url)
                            });
                            app.open_image_modal(img.url, img.alt, img.caption, path);
                        }
                    }
                    _ => {}
                }
                break;
            }
        }
    }
}

pub fn handle_mouse_move(
    app: &mut App,
    col: u16,
    row: u16,
    term_width: u16,
    term_height: u16,
    ctrl: bool,
) {
    if app.input_mode == InputMode::DailyFeedModal {
        let size = Rect::new(0, 0, term_width, term_height);
        if let Some((item_idx, l_idx, _)) =
            crate::ui::modals::get_daily_feed_item_at(app, col, row, size)
        {
            if let Some(modal) = &mut app.modals.daily_feed_modal {
                modal.cursor_idx = item_idx;
                modal.link_idx = l_idx;
            }
        }
        return;
    }

    if app.input_mode != InputMode::Normal && app.input_mode != InputMode::LinkPeek {
        return;
    }

    let mut hovered_link = None;

    if app.workspace.zen_mode {
        let zen_rect = crate::ui::compute_zen_area(Rect::new(0, 0, term_width, term_height));
        if rect_contains(zen_rect, col, row) {
            let pane = app.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                if let Some(link_idx) = crate::ui::pane_view::get_link_at_coord(
                    parsed_doc,
                    pane.scroll_offset,
                    zen_rect,
                    col,
                    row,
                ) {
                    pane.selected_link_idx = Some(link_idx);
                    if let Some(link) = parsed_doc.links.get(link_idx) {
                        if !link.is_citation() {
                            hovered_link = Some((link.title.clone(), link.text.clone()));
                        }
                    }
                }
            }
        }
    } else if row >= 1 && row < term_height.saturating_sub(1) {
        let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
        let tab = app.active_tab_mut();
        let rects = tab.layout_root.compute_rects(main_rect);

        for (pane_idx, rect) in rects {
            if rect_contains(rect, col, row) {
                let pane = &mut tab.panes[pane_idx];
                if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                    if let Some(link_idx) = crate::ui::pane_view::get_link_at_coord(
                        parsed_doc,
                        pane.scroll_offset,
                        rect,
                        col,
                        row,
                    ) {
                        pane.selected_link_idx = Some(link_idx);
                        if let Some(link) = parsed_doc.links.get(link_idx) {
                            if !link.is_citation() {
                                hovered_link = Some((link.title.clone(), link.text.clone()));
                            }
                        }
                    }
                }
                break;
            }
        }
    }

    if ctrl {
        if let Some((title, raw_target)) = hovered_link {
            if app.modals.link_peek.as_ref().map(|p| &p.raw_target) != Some(&raw_target) {
                app.open_link_peek(title, raw_target, col, row);
            }
        } else if app.input_mode == InputMode::LinkPeek {
            app.close_link_peek();
        }
    } else if app.input_mode == InputMode::LinkPeek {
        app.close_link_peek();
    }
}
