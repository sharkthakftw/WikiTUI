use crate::app::{App, InputMode};
use crate::ui::modals::{
    get_feed_entries, get_ongoing_links, get_recent_deaths_links, parse_onthisday_event,
    parse_story_html, DailyFeedKind, OnThisDayTab,
};
use crossterm::event::{KeyCode, KeyEvent};

fn get_modal_row_links(app: &App, kind: DailyFeedKind, cursor_idx: usize) -> Vec<String> {
    let feed = match &app.daily_feed {
        Some(f) => f,
        None => return Vec::new(),
    };
    match kind {
        DailyFeedKind::News => {
            if cursor_idx < feed.news.len()
                && let Some(item) = feed.news.get(cursor_idx) {
                    let raw = item.story.as_deref().unwrap_or("");
                    let (_, links) = parse_story_html(raw);
                    return links;
                }
            let ongoing_row = feed.news.len();
            if !feed.ongoing.is_empty() && cursor_idx == ongoing_row {
                return get_ongoing_links(&feed.ongoing)
                    .into_iter()
                    .map(|(target, _)| target)
                    .collect();
            }
            let deaths_row = feed.news.len() + (if !feed.ongoing.is_empty() { 1 } else { 0 });
            if !feed.recent_deaths.is_empty() && cursor_idx == deaths_row {
                return get_recent_deaths_links(&feed.recent_deaths)
                    .into_iter()
                    .map(|(target, _)| target)
                    .collect();
            }
            Vec::new()
        }
        DailyFeedKind::OnThisDay => {
            let otd_tab = app
                .modals
                .daily_feed_modal
                .as_ref()
                .map(|m| m.otd_tab)
                .unwrap_or_default();
            let events_slice: &[crate::api::daily_feed::OnThisDayEvent] =
                if let Some(arch) = &feed.onthisday_all {
                    match otd_tab {
                        OnThisDayTab::Events => {
                            if !arch.events.is_empty() {
                                &arch.events
                            } else {
                                &feed.onthisday
                            }
                        }
                        OnThisDayTab::Births => &arch.births,
                        OnThisDayTab::Deaths => &arch.deaths,
                        OnThisDayTab::Holidays => &arch.holidays,
                    }
                } else {
                    &feed.onthisday
                };
            if let Some(event) = events_slice.get(cursor_idx) {
                let (_, links) = parse_onthisday_event(&event.text, &event.pages);
                return links;
            }
            Vec::new()
        }
        DailyFeedKind::MostRead => Vec::new(),
    }
}

pub fn handle_daily_feed_mode(app: &mut App, key: KeyEvent) {
    let state = match &app.modals.daily_feed_modal {
        Some(s) => s.clone(),
        None => {
            app.input_mode = InputMode::Normal;
            return;
        }
    };

    let entries = get_feed_entries(app, state.kind);
    let total = entries.len();

    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.close_daily_feed_modal();
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if total > 0 {
                let kind = state.kind;
                if let Some(modal) = &mut app.modals.daily_feed_modal
                    && modal.cursor_idx + 1 < total {
                        modal.cursor_idx += 1;
                        modal.link_idx = 0;
                    }
                let target_line = crate::ui::modals::get_modal_item_line_offset(
                    app,
                    kind,
                    app.modals
                        .daily_feed_modal
                        .as_ref()
                        .map(|m| m.cursor_idx)
                        .unwrap_or(0),
                );
                if let Some(modal) = &mut app.modals.daily_feed_modal
                    && target_line >= modal.scroll + 12 {
                        modal.scroll = target_line.saturating_sub(8);
                    }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if total > 0 {
                let kind = state.kind;
                if let Some(modal) = &mut app.modals.daily_feed_modal
                    && modal.cursor_idx > 0 {
                        modal.cursor_idx -= 1;
                        modal.link_idx = 0;
                    }
                let target_line = crate::ui::modals::get_modal_item_line_offset(
                    app,
                    kind,
                    app.modals
                        .daily_feed_modal
                        .as_ref()
                        .map(|m| m.cursor_idx)
                        .unwrap_or(0),
                );
                if let Some(modal) = &mut app.modals.daily_feed_modal
                    && target_line < modal.scroll {
                        modal.scroll = target_line;
                    }
            }
        }
        KeyCode::Char('g') | KeyCode::Home => {
            if let Some(modal) = &mut app.modals.daily_feed_modal {
                modal.cursor_idx = 0;
                modal.link_idx = 0;
                modal.scroll = 0;
            }
        }
        KeyCode::Char('G') | KeyCode::End => {
            if total > 0 {
                let kind = state.kind;
                if let Some(modal) = &mut app.modals.daily_feed_modal {
                    modal.cursor_idx = total.saturating_sub(1);
                    modal.link_idx = 0;
                }
                let target_line = crate::ui::modals::get_modal_item_line_offset(
                    app,
                    kind,
                    app.modals
                        .daily_feed_modal
                        .as_ref()
                        .map(|m| m.cursor_idx)
                        .unwrap_or(0),
                );
                if let Some(modal) = &mut app.modals.daily_feed_modal {
                    modal.scroll = target_line.saturating_sub(6);
                }
            }
        }
        KeyCode::Tab | KeyCode::Char('l') | KeyCode::Right => {
            if state.kind == DailyFeedKind::News || state.kind == DailyFeedKind::OnThisDay {
                let links = get_modal_row_links(app, state.kind, state.cursor_idx);
                let total_links = links.len();
                if total_links > 0
                    && let Some(modal) = &mut app.modals.daily_feed_modal {
                        modal.link_idx = (modal.link_idx + 1) % total_links;
                    }
            }
        }
        KeyCode::BackTab | KeyCode::Char('h') | KeyCode::Left => {
            if state.kind == DailyFeedKind::News || state.kind == DailyFeedKind::OnThisDay {
                let links = get_modal_row_links(app, state.kind, state.cursor_idx);
                let total_links = links.len();
                if total_links > 0
                    && let Some(modal) = &mut app.modals.daily_feed_modal {
                        modal.link_idx = if modal.link_idx == 0 {
                            total_links - 1
                        } else {
                            modal.link_idx - 1
                        };
                    }
            }
        }
        KeyCode::Enter => {
            let target =
                if state.kind == DailyFeedKind::News || state.kind == DailyFeedKind::OnThisDay {
                    let links = get_modal_row_links(app, state.kind, state.cursor_idx);
                    links
                        .get(state.link_idx)
                        .cloned()
                        .or_else(|| links.first().cloned())
                } else {
                    entries
                        .get(state.cursor_idx)
                        .map(|e| e.target_article.clone())
                };

            if let Some(target) = target.or_else(|| {
                entries
                    .get(state.cursor_idx)
                    .map(|e| e.target_article.clone())
            })
                && !target.is_empty() {
                    app.close_daily_feed_modal();
                    app.open_article(&target);
                }
        }
        KeyCode::Char('t') => {
            let target =
                if state.kind == DailyFeedKind::News || state.kind == DailyFeedKind::OnThisDay {
                    let links = get_modal_row_links(app, state.kind, state.cursor_idx);
                    links
                        .get(state.link_idx)
                        .cloned()
                        .or_else(|| links.first().cloned())
                } else {
                    entries
                        .get(state.cursor_idx)
                        .map(|e| e.target_article.clone())
                };

            if let Some(target) = target.or_else(|| {
                entries
                    .get(state.cursor_idx)
                    .map(|e| e.target_article.clone())
            })
                && !target.is_empty() {
                    app.close_daily_feed_modal();
                    if !matches!(app.active_pane().content, crate::app::PaneContent::Empty) {
                        app.new_tab();
                    }
                    app.open_article(&target);
                }
        }
        KeyCode::Char('1') if state.kind == DailyFeedKind::OnThisDay => {
            if let Some(modal) = &mut app.modals.daily_feed_modal {
                modal.otd_tab = OnThisDayTab::Events;
                modal.cursor_idx = 0;
                modal.link_idx = 0;
                modal.scroll = 0;
            }
        }
        KeyCode::Char('2') if state.kind == DailyFeedKind::OnThisDay => {
            if let Some(modal) = &mut app.modals.daily_feed_modal {
                modal.otd_tab = OnThisDayTab::Births;
                modal.cursor_idx = 0;
                modal.link_idx = 0;
                modal.scroll = 0;
            }
        }
        KeyCode::Char('3') if state.kind == DailyFeedKind::OnThisDay => {
            if let Some(modal) = &mut app.modals.daily_feed_modal {
                modal.otd_tab = OnThisDayTab::Deaths;
                modal.cursor_idx = 0;
                modal.link_idx = 0;
                modal.scroll = 0;
            }
        }
        KeyCode::Char('4') if state.kind == DailyFeedKind::OnThisDay => {
            if let Some(modal) = &mut app.modals.daily_feed_modal {
                modal.otd_tab = OnThisDayTab::Holidays;
                modal.cursor_idx = 0;
                modal.link_idx = 0;
                modal.scroll = 0;
            }
        }
        KeyCode::Char(']') if state.kind == DailyFeedKind::OnThisDay => {
            if let Some(modal) = &mut app.modals.daily_feed_modal {
                modal.otd_tab = match modal.otd_tab {
                    OnThisDayTab::Events => OnThisDayTab::Births,
                    OnThisDayTab::Births => OnThisDayTab::Deaths,
                    OnThisDayTab::Deaths => OnThisDayTab::Holidays,
                    OnThisDayTab::Holidays => OnThisDayTab::Events,
                };
                modal.cursor_idx = 0;
                modal.link_idx = 0;
                modal.scroll = 0;
            }
        }
        KeyCode::Char('[') if state.kind == DailyFeedKind::OnThisDay => {
            if let Some(modal) = &mut app.modals.daily_feed_modal {
                modal.otd_tab = match modal.otd_tab {
                    OnThisDayTab::Events => OnThisDayTab::Holidays,
                    OnThisDayTab::Births => OnThisDayTab::Events,
                    OnThisDayTab::Deaths => OnThisDayTab::Births,
                    OnThisDayTab::Holidays => OnThisDayTab::Deaths,
                };
                modal.cursor_idx = 0;
                modal.link_idx = 0;
                modal.scroll = 0;
            }
        }
        _ => {}
    }
}
