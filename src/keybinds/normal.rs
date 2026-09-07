use crate::app::App;
use crate::layout::SplitDirection;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_normal_mode(app: &mut App, key: KeyEvent, term_width: u16, term_height: u16) {
    if app.feed.active {
        match key.code {
            KeyCode::Esc | KeyCode::Char('F') | KeyCode::Char('q') => {
                app.feed.active = false;
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.feed.next_post();
                app.maybe_fetch_feed_batch();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.feed.prev_post();
            }
            KeyCode::Char('l') => {
                app.toggle_feed_like();
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                app.confirm_action = Some(crate::app::ConfirmAction::ResetFeed);
                app.input_mode = crate::app::InputMode::Confirm;
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => {
                if let Some(item) = app.feed.current_item().cloned() {
                    let cur_tab = app.active_tab_idx;
                    app.new_tab();
                    let pane_id = app.active_pane().id;
                    app.active_pane_mut().is_loading = true;
                    app.send_fetch_article(pane_id, item.title.clone());
                    app.active_tab_idx = cur_tab;
                    app.set_status_message(format!("opened '{}' in background tab", item.title));
                }
            }
            KeyCode::Enter => {
                if let Some(item) = app.feed.current_item().cloned() {
                    app.feed.active = false;
                    let pane_id = app.active_pane().id;
                    app.active_pane_mut().is_loading = true;
                    app.send_fetch_article(pane_id, item.title);
                }
            }
            KeyCode::Char('t') => {
                if let Some(item) = app.feed.current_item().cloned() {
                    app.feed.active = false;
                    app.new_tab();
                    let pane_id = app.active_pane().id;
                    app.active_pane_mut().is_loading = true;
                    app.send_fetch_article(pane_id, item.title);
                }
            }
            _ => {}
        }
    } else if app.active_pane().toc_focused {
        match key.code {
            KeyCode::Esc | KeyCode::Char('o') => {
                app.toggle_toc();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.select_next_toc_item();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.select_prev_toc_item();
            }
            KeyCode::Enter => {
                app.activate_toc_selection(term_height);
            }
            _ => {}
        }
    } else if app.waiting_for_split_cmd {
        app.waiting_for_split_cmd = false;
        match key.code {
            KeyCode::Char('v') => {
                app.split_active_pane(SplitDirection::Vertical);
            }
            KeyCode::Char('s') => {
                app.split_active_pane(SplitDirection::Horizontal);
            }
            KeyCode::Char('c') | KeyCode::Char('x') | KeyCode::Char('q') => {
                app.close_active_pane();
            }
            KeyCode::Char('h') => {
                app.navigate_panes('h', term_width, term_height);
            }
            KeyCode::Char('j') => {
                app.navigate_panes('j', term_width, term_height);
            }
            KeyCode::Char('k') => {
                app.navigate_panes('k', term_width, term_height);
            }
            KeyCode::Char('l') => {
                app.navigate_panes('l', term_width, term_height);
            }
            KeyCode::Char('H') => {
                app.move_pane('h', term_width, term_height);
            }
            KeyCode::Char('J') => {
                app.move_pane('j', term_width, term_height);
            }
            KeyCode::Char('K') => {
                app.move_pane('k', term_width, term_height);
            }
            KeyCode::Char('L') => {
                app.move_pane('l', term_width, term_height);
            }
            _ => {}
        }
    } else {
        if matches!(app.active_pane().content, crate::app::PaneContent::Empty)
            && (key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT)
        {
            if let KeyCode::Char(c) = key.code {
                match app.config.general.hint_mode {
                    crate::config::HintMode::Semantic => {
                        if let Some(title) = app.find_semantic_hint_article(c) {
                            app.open_article(&title);
                            return;
                        }
                    }
                    crate::config::HintMode::Numbered => {
                        if c.is_ascii_digit() && c != '0' && key.modifiers.is_empty() {
                            let idx = (c as usize) - ('1' as usize);
                            let recents = app.get_continue_reading_articles();
                            if let Some(title) = recents.get(idx) {
                                app.open_article(title);
                                return;
                            }
                        }
                    }
                    crate::config::HintMode::None => {}
                }
            }
        }

        match key.code {
            KeyCode::Esc => {
                app.active_pane_mut().selection.text_selection = None;
                app.clear_local_search();
            }
            KeyCode::Char('q') => {
                app.quit();
            }
            KeyCode::Char(':') => {
                app.open_command_palette();
            }
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.open_command_palette();
            }
            KeyCode::Char('p') => {
                let pane_rect =
                    crate::mouse::scrollbar::active_pane_rect(app, term_width, term_height);
                let link_info = if let crate::app::PaneContent::ArticleText { parsed_doc, .. } =
                    &app.active_pane().content
                {
                    let scroll = app.active_pane().scroll_offset;
                    let target_link_idx = app
                        .active_pane()
                        .selected_link_idx
                        .filter(|&idx| {
                            parsed_doc.links.get(idx).is_some_and(|l| !l.is_citation())
                        })
                        .or_else(|| {
                            parsed_doc.links.iter().position(|l| {
                                !l.is_citation()
                                    && l.span_indices
                                        .first()
                                        .is_some_and(|&(line, _)| line >= scroll && line < scroll + 20)
                            })
                        });

                    if let Some(idx) = target_link_idx {
                        if let Some(link) = parsed_doc.links.get(idx) {
                            let (line_idx, span_idx) =
                                link.span_indices.first().copied().unwrap_or((scroll, 0));
                            let col_offset: usize = parsed_doc
                                .lines
                                .get(line_idx)
                                .map(|l| {
                                    l.spans
                                        .iter()
                                        .take(span_idx)
                                        .map(|s| {
                                            unicode_width::UnicodeWidthStr::width(s.content.as_ref())
                                        })
                                        .sum()
                                })
                                .unwrap_or(0);

                            let row_offset = line_idx.saturating_sub(scroll);
                            Some((
                                idx,
                                link.title.clone(),
                                link.text.clone(),
                                col_offset,
                                row_offset,
                            ))
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some((idx, title, raw_target, col_offset, row_offset)) = link_info {
                    app.active_pane_mut().selected_link_idx = Some(idx);
                    let anchor_x = pane_rect.x + 1 + (col_offset as u16);
                    let anchor_y = pane_rect.y + 1 + (row_offset as u16);
                    app.open_link_peek(title, raw_target, anchor_x, anchor_y);
                }
            }
            KeyCode::Char('z') => {
                app.toggle_zen_mode();
            }
            KeyCode::Char('F') => {
                app.toggle_feed_mode();
            }
            KeyCode::Char('r') => {
                app.fetch_random_article();
            }
            KeyCode::Char('a') => {
                app.toggle_spoken_audio();
            }
            KeyCode::Char('A') => {
                app.stop_spoken_audio();
            }
            KeyCode::Char('>') => {
                app.seek_spoken_audio(10);
            }
            KeyCode::Char('<') => {
                app.seek_spoken_audio(-10);
            }
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.intra_jump_back(term_height);
            }
            KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.intra_jump_forward(term_height);
            }
            KeyCode::Char('o') => {
                app.toggle_toc();
            }
            KeyCode::Char('c') if key.modifiers.is_empty() => {
                app.toggle_categories_modal();
            }
            KeyCode::Char('m') => {
                app.open_save_to_list_modal();
            }
            KeyCode::Char('M') => {
                app.open_saved_lists_viewer();
            }
            KeyCode::Char('y') => {
                app.copy_focused_link();
            }
            KeyCode::Char('Y') => {
                app.copy_article_link();
            }
            KeyCode::Char('U') => {
                app.check_for_updates();
            }
            KeyCode::Char('?') => {
                app.toggle_help_popup();
            }
            KeyCode::Char(',') if key.modifiers.is_empty() => {
                app.input_mode = crate::app::InputMode::Settings;
            }
            KeyCode::Char('/') => {
                app.enter_local_search_mode();
            }
            KeyCode::Char('n') => {
                if matches!(app.active_pane().content, crate::app::PaneContent::Empty) {
                    app.open_daily_feed_modal(crate::ui::modals::DailyFeedKind::News);
                } else {
                    app.next_local_match(term_height);
                }
            }
            KeyCode::Char('d') => {
                if matches!(app.active_pane().content, crate::app::PaneContent::Empty) {
                    app.open_daily_feed_modal(crate::ui::modals::DailyFeedKind::OnThisDay);
                } else if key.modifiers.contains(KeyModifiers::CONTROL) {
                    app.scroll_page_down(term_height);
                }
            }
            KeyCode::Char('N') => {
                app.prev_local_match(term_height);
            }
            KeyCode::Char(']') => {
                app.jump_next_heading(term_height);
            }
            KeyCode::Char('[') => {
                app.jump_prev_heading(term_height);
            }
            KeyCode::Char('f') => {
                if matches!(app.active_pane().content, crate::app::PaneContent::Empty) {
                    if let Some(tfa) = app.daily_feed.as_ref().and_then(|f| f.tfa.as_ref()) {
                        let title = tfa.display_title();
                        app.open_article(&title);
                    } else {
                        app.pending_open_tfa = true;
                        app.active_pane_mut().is_loading = true;
                        app.send_fetch_daily_feed();
                    }
                } else {
                    app.scroll_page_down(term_height);
                }
            }
            KeyCode::Char('b') => {
                app.scroll_page_up(term_height);
            }
            KeyCode::Char('g') => {
                app.jump_to_top();
            }
            KeyCode::Char('G') => {
                app.jump_to_bottom(term_height);
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.enter_search_mode();
            }
            KeyCode::Char('i')
                if matches!(
                    app.active_pane().content,
                    crate::app::PaneContent::SearchResults { .. }
                ) =>
            {
                app.edit_search_mode();
            }
            KeyCode::Char('I') => {
                app.toggle_images();
            }
            KeyCode::Char('s') => {
                app.activate_selected_in_split(SplitDirection::Horizontal);
            }
            KeyCode::Char('v') => {
                app.activate_selected_in_split(SplitDirection::Vertical);
            }
            KeyCode::Char('%') => {
                app.toggle_alternate_tab();
            }
            KeyCode::Char('t')
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::META) =>
            {
                app.new_tab();
            }
            KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.waiting_for_split_cmd = true;
            }
            KeyCode::Char('=') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.resize_active_split(5);
            }
            KeyCode::Char('-') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.resize_active_split(-5);
            }
            KeyCode::Char('S') => {
                if app.active_tab().name == "home" {
                    if let Some(session) = crate::session::SessionState::load() {
                        session.restore_to_app(app);
                    }
                }
            }
            KeyCode::Char('H') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.history_back();
            }
            KeyCode::Char('L') if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.history_forward();
            }
            KeyCode::Char('h')
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::META) =>
            {
                app.prev_tab();
            }
            KeyCode::Char('l')
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::META) =>
            {
                app.next_tab();
            }
            KeyCode::Char(c @ '0'..='9')
                if key
                    .modifiers
                    .intersects(KeyModifiers::ALT | KeyModifiers::META) =>
            {
                let tab_idx = if c == '0' {
                    9
                } else {
                    (c as usize) - ('1' as usize)
                };
                app.switch_to_tab(tab_idx);
            }
            KeyCode::Char(c @ '0'..='9')
                if key.modifiers.is_empty()
                    && matches!(
                        app.active_pane().content,
                        crate::app::PaneContent::SearchResults { .. }
                    ) =>
            {
                app.activate_search_result_digit(c);
            }
            KeyCode::Char('x') => {
                app.close_active_pane();
            }
            KeyCode::Char('u') if key.modifiers.is_empty() => {
                app.reopen_last_closed();
            }
            KeyCode::Char('H' | 'h')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && (key.modifiers.contains(KeyModifiers::SHIFT)
                        || matches!(key.code, KeyCode::Char('H'))) =>
            {
                app.move_pane('h', term_width, term_height);
            }
            KeyCode::Char('L' | 'l')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && (key.modifiers.contains(KeyModifiers::SHIFT)
                        || matches!(key.code, KeyCode::Char('L'))) =>
            {
                app.move_pane('l', term_width, term_height);
            }
            KeyCode::Char('J' | 'j')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && (key.modifiers.contains(KeyModifiers::SHIFT)
                        || matches!(key.code, KeyCode::Char('J'))) =>
            {
                app.move_pane('j', term_width, term_height);
            }
            KeyCode::Char('K' | 'k')
                if key.modifiers.contains(KeyModifiers::CONTROL)
                    && (key.modifiers.contains(KeyModifiers::SHIFT)
                        || matches!(key.code, KeyCode::Char('K'))) =>
            {
                app.move_pane('k', term_width, term_height);
            }
            KeyCode::Char('h') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.navigate_panes('h', term_width, term_height);
            }
            KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.navigate_panes('l', term_width, term_height);
            }
            KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.navigate_panes('j', term_width, term_height);
            }
            KeyCode::Char('k') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.navigate_panes('k', term_width, term_height);
            }
            KeyCode::Tab => {
                app.focus_next_link();
            }
            KeyCode::BackTab => {
                app.focus_prev_link();
            }
            KeyCode::Char('j') | KeyCode::Down => {
                app.select_next_item(term_height);
            }
            KeyCode::Char('k') | KeyCode::Up => {
                app.select_prev_item(term_height);
            }
            KeyCode::Char('t') => {
                if matches!(app.active_pane().content, crate::app::PaneContent::Empty) {
                    app.open_daily_feed_modal(crate::ui::modals::DailyFeedKind::MostRead);
                } else {
                    app.activate_selected_in_new_tab();
                }
            }
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::ALT) => {
                app.activate_selected_in_background_tab();
            }
            KeyCode::Enter => {
                app.activate_selected(term_height);
            }
            _ => {}
        }
    }
}
