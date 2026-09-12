pub mod html;
pub mod most_read;
pub mod news;
pub mod on_this_day;
pub mod types;

pub use html::{parse_onthisday_event, parse_story_html, strip_html_tags, wrap_story_spans};
pub use news::{get_ongoing_links, get_recent_deaths_links, render_news_modal};
pub use on_this_day::{get_otd_tab_at, render_on_this_day_modal, today_date_str};
pub use types::{
    CachedWrappedItem, DailyFeedCache, DailyFeedKind, DailyFeedModalState, FeedEntry, OnThisDayTab,
    ParsedStory, SpanStyle, StyledChunk,
};

use crate::app::App;
use crate::theme;
use crate::ui::modals::utils::{centered_rect, render_modal_frame_at};
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_daily_feed_modal_area(container_rect: Rect, kind: DailyFeedKind) -> Rect {
    match kind {
        DailyFeedKind::MostRead => centered_rect(50, 57, container_rect),
        DailyFeedKind::OnThisDay => centered_rect(75, 69, container_rect),
        DailyFeedKind::News => centered_rect(75, 65, container_rect),
    }
}

pub fn get_feed_entries(app: &App, kind: DailyFeedKind) -> Vec<FeedEntry> {
    let feed = match &app.daily_feed {
        Some(f) => f,
        None => return Vec::new(),
    };

    match kind {
        DailyFeedKind::News => {
            let mut entries = Vec::new();
            for item in &feed.news {
                if let Some(first_link) = item.links.first() {
                    let summary = item.story.as_deref().unwrap_or("");
                    let clean_story = strip_html_tags(summary);
                    let target = first_link.title.clone();
                    entries.push(FeedEntry {
                        title: if !clean_story.is_empty() {
                            clean_story
                        } else {
                            first_link.display_title().to_string()
                        },
                        target_article: target,
                        suffix: None,
                    });
                }
            }
            if !feed.ongoing.is_empty() {
                let first_target = feed
                    .ongoing
                    .first()
                    .map(|o| o.target.clone())
                    .unwrap_or_default();
                entries.push(FeedEntry {
                    title: "Ongoing".to_string(),
                    target_article: first_target,
                    suffix: None,
                });
            }
            if !feed.recent_deaths.is_empty() {
                let first_target = feed
                    .recent_deaths
                    .first()
                    .map(|d| d.target.clone())
                    .unwrap_or_default();
                entries.push(FeedEntry {
                    title: "Recent deaths".to_string(),
                    target_article: first_target,
                    suffix: None,
                });
            }
            entries
        }
        DailyFeedKind::OnThisDay => {
            let otd_tab = app
                .modals
                .daily_feed_modal
                .as_ref()
                .map(|m| m.otd_tab)
                .unwrap_or_default();
            let events_slice: &[crate::api::daily_feed::OnThisDayEvent] =
                if let Some(archive) = &feed.onthisday_all {
                    match otd_tab {
                        OnThisDayTab::Events => {
                            if !archive.events.is_empty() {
                                &archive.events
                            } else {
                                &feed.onthisday
                            }
                        }
                        OnThisDayTab::Births => &archive.births,
                        OnThisDayTab::Deaths => &archive.deaths,
                        OnThisDayTab::Holidays => &archive.holidays,
                    }
                } else {
                    &feed.onthisday
                };

            let mut entries = Vec::new();
            for event in events_slice {
                let target = event
                    .pages
                    .first()
                    .map(|p| p.title.clone())
                    .unwrap_or_default();
                if target.is_empty() {
                    continue;
                }
                let year_str = match event.year {
                    Some(y) if y < 0 => format!("{} BC", y.abs()),
                    Some(y) => format!("{}", y),
                    None => "Holiday".to_string(),
                };
                let clean_text = strip_html_tags(&event.text);
                let display = format!("[ {} ]  {}", year_str, clean_text);
                entries.push(FeedEntry {
                    title: display,
                    target_article: target,
                    suffix: None,
                });
            }
            entries
        }
        DailyFeedKind::MostRead => {
            let mut entries = Vec::new();
            if let Some(payload) = &feed.mostread {
                for (idx, item) in payload.articles.iter().take(25).enumerate() {
                    let rank_str = format!("{}. ", idx + 1);
                    let views_str = item.views.map(crate::api::stats::format_metric);
                    let display = format!("{}{}", rank_str, item.display_title());
                    entries.push(FeedEntry {
                        title: display,
                        target_article: item.title.clone(),
                        suffix: views_str.map(|v| format!("  {} views", v)),
                    });
                }
            }
            entries
        }
    }
}

pub fn render_daily_feed_modal(f: &mut Frame, app: &App, size: Rect) {
    let state = match &app.modals.daily_feed_modal {
        Some(s) => s,
        None => return,
    };

    let (icon, title_text) = match state.kind {
        DailyFeedKind::News => (
            if app.user_data.config.ui.icons {
                "󰋫"
            } else {
                ""
            },
            "in the news".to_string(),
        ),
        DailyFeedKind::OnThisDay => (
            if app.user_data.config.ui.icons {
                "󰃭"
            } else {
                ""
            },
            format!("on this day · {}", today_date_str()),
        ),
        DailyFeedKind::MostRead => (
            if app.user_data.config.ui.icons {
                "󰄬"
            } else {
                ""
            },
            "most read".to_string(),
        ),
    };
    let accent_color = theme::BLUE;

    let modal_area = compute_daily_feed_modal_area(size, state.kind);
    let modal_block = render_modal_frame_at(
        f,
        modal_area,
        icon,
        &title_text,
        accent_color,
        app.user_data.config.ui.rounded_borders,
    );

    if app.daily_feed.is_none() {
        let vertical_offset = (modal_area.height.saturating_sub(2) / 2) as usize;
        let mut lines = Vec::new();
        for _ in 0..vertical_offset {
            lines.push(Line::from(""));
        }
        let spinner = crate::ui::current_spinner_frame();
        lines.push(Line::from(vec![
            Span::styled(
                format!("{} ", spinner),
                Style::default().fg(theme::BLUE).bold(),
            ),
            Span::styled(
                "fetching daily feed from Wikipedia...",
                Style::default().fg(theme::BEIGE).bold(),
            ),
        ]));
        let loading_p = Paragraph::new(lines)
            .block(modal_block)
            .alignment(Alignment::Center);
        f.render_widget(loading_p, modal_area);
        return;
    }

    let entries = get_feed_entries(app, state.kind);
    let total = entries.len();
    let selected_idx = state.cursor_idx.min(total.saturating_sub(1));

    match state.kind {
        DailyFeedKind::News => {
            news::render_news_modal(
                f,
                app,
                modal_area,
                modal_block,
                selected_idx,
                state.link_idx,
            );
        }
        DailyFeedKind::OnThisDay => {
            on_this_day::render_on_this_day_modal(
                f,
                app,
                modal_area,
                modal_block,
                selected_idx,
                state.link_idx,
                state.otd_tab,
            );
        }
        DailyFeedKind::MostRead => {
            most_read::render_most_read_modal(
                f,
                app,
                &entries,
                modal_area,
                modal_block,
                selected_idx,
            );
        }
    }
}

pub fn get_daily_feed_item_at(
    app: &App,
    col: u16,
    row: u16,
    size: Rect,
) -> Option<(usize, usize, String)> {
    let feed = app.daily_feed.as_ref()?;
    let modal = app.modals.daily_feed_modal.as_ref()?;
    let modal_area = compute_daily_feed_modal_area(size, modal.kind);

    let inner_x = modal_area.x + 1;
    let inner_w = modal_area.width.saturating_sub(2);
    let inner_y = modal_area.y + 1;
    let inner_h = modal_area.height.saturating_sub(2);

    if col < inner_x || col >= inner_x + inner_w || row < inner_y || row >= inner_y + inner_h {
        return None;
    }

    match modal.kind {
        DailyFeedKind::MostRead => {
            let entries = get_feed_entries(app, DailyFeedKind::MostRead);
            let scroll = modal
                .scroll
                .min(entries.len().saturating_sub(inner_h as usize));
            let clicked_idx = (row - inner_y) as usize + scroll;
            entries
                .get(clicked_idx)
                .map(|e| (clicked_idx, 0, e.target_article.clone()))
        }
        DailyFeedKind::News => {
            let cache_ref = modal.cache.borrow();
            let mut line_offsets = Vec::new();
            let mut line_count = 0;

            for item in &cache_ref.news_wrapped {
                line_offsets.push(line_count);
                line_count += item.wrapped_lines.len() + 1;
            }

            let separator_start = line_count;
            let has_ongoing_or_deaths = !feed.ongoing.is_empty() || !feed.recent_deaths.is_empty();
            if has_ongoing_or_deaths {
                line_count += 2;
            }
            let separator_end = line_count;

            let ongoing_row = feed.news.len();
            let ongoing_offset = if !feed.ongoing.is_empty() {
                line_offsets.push(line_count);
                let off = line_count;
                line_count += 2;
                Some(off)
            } else {
                None
            };

            let deaths_row = feed.news.len() + (if !feed.ongoing.is_empty() { 1 } else { 0 });
            let deaths_offset = if !feed.recent_deaths.is_empty() {
                line_offsets.push(line_count);
                let off = line_count;
                line_count += 1;
                Some(off)
            } else {
                None
            };

            let scroll = modal
                .scroll
                .min(line_count.saturating_sub(inner_h as usize));
            let clicked_line = (row - inner_y) as usize + scroll;

            if has_ongoing_or_deaths
                && clicked_line >= separator_start
                && clicked_line < separator_end
            {
                return None;
            }

            for (item_idx, item) in cache_ref.news_wrapped.iter().enumerate() {
                let start_line = line_offsets[item_idx];
                let end_line = start_line + item.wrapped_lines.len();
                if clicked_line >= start_line && clicked_line < end_line {
                    let line_in_item = clicked_line - start_line;
                    if let Some(words) = item.wrapped_lines.get(line_in_item) {
                        let mut cur_x = inner_x + 3;
                        for (text, style) in words {
                            let span_w =
                                unicode_width::UnicodeWidthStr::width(text.as_str()) as u16;
                            if col >= cur_x && col < cur_x + span_w {
                                match style {
                                    SpanStyle::Link {
                                        link_idx: l_idx,
                                        title,
                                    }
                                    | SpanStyle::BoldLink {
                                        link_idx: l_idx,
                                        title,
                                    } => {
                                        return Some((item_idx, *l_idx, title.clone()));
                                    }
                                    _ => {}
                                }
                            }
                            cur_x += span_w;
                        }
                    }
                    return item.links.first().map(|l| (item_idx, 0, l.clone()));
                }
            }

            if let Some(off) = ongoing_offset
                && clicked_line == off {
                    let mut cur_x = inner_x + 12;
                    let mut link_counter = 0;
                    for (idx, og) in feed.ongoing.iter().enumerate() {
                        if idx > 0 {
                            cur_x += 3;
                        }
                        let og_l_idx = link_counter;
                        link_counter += 1;
                        let og_w =
                            unicode_width::UnicodeWidthStr::width(og.display.as_str()) as u16;
                        if col >= cur_x && col < cur_x + og_w {
                            return Some((ongoing_row, og_l_idx, og.target.clone()));
                        }
                        cur_x += og_w;
                        for (sub_target, sub_display) in &og.sub_events {
                            cur_x += 2;
                            let sub_l_idx = link_counter;
                            link_counter += 1;
                            let sub_w =
                                unicode_width::UnicodeWidthStr::width(sub_display.as_str()) as u16;
                            if col >= cur_x && col < cur_x + sub_w {
                                return Some((ongoing_row, sub_l_idx, sub_target.clone()));
                            }
                            cur_x += sub_w + 1;
                        }
                    }
                    return feed
                        .ongoing
                        .first()
                        .map(|o| (ongoing_row, 0, o.target.clone()));
                }

            if let Some(off) = deaths_offset
                && clicked_line == off {
                    let mut cur_x = inner_x + 18;
                    for (d_idx, death) in feed.recent_deaths.iter().enumerate() {
                        if idx_offset_matches(d_idx) {
                            cur_x += 3;
                        }
                        let death_w =
                            unicode_width::UnicodeWidthStr::width(death.name.as_str()) as u16;
                        if col >= cur_x && col < cur_x + death_w {
                            return Some((deaths_row, d_idx, death.target.clone()));
                        }
                        cur_x += death_w;
                    }
                    return feed
                        .recent_deaths
                        .first()
                        .map(|d| (deaths_row, 0, d.target.clone()));
                }

            None
        }
        DailyFeedKind::OnThisDay => {
            let cache_ref = modal.cache.borrow();
            let mut line_offsets = Vec::new();
            let mut line_count = 3;

            for item in &cache_ref.otd_wrapped {
                line_offsets.push(line_count);
                line_count += item.wrapped_lines.len() + 1;
            }

            let otd_inner_h = modal_area.height.saturating_sub(2) as usize;
            let scroll = modal.scroll.min(line_count.saturating_sub(otd_inner_h));
            let clicked_line = (row.saturating_sub(inner_y) as usize) + scroll;

            if clicked_line < 3 {
                return None;
            }

            for (item_idx, item) in cache_ref.otd_wrapped.iter().enumerate() {
                let start_line = line_offsets[item_idx];
                let end_line = start_line + item.wrapped_lines.len();
                if clicked_line >= start_line && clicked_line < end_line {
                    let line_in_item = clicked_line - start_line;
                    if let Some(words) = item.wrapped_lines.get(line_in_item) {
                        let badge_len = format!("[ {} ] {}", item.year_str, item.elapsed_str)
                            .chars()
                            .count();
                        let prefix_w = (3 + badge_len) as u16;
                        let mut cur_x = inner_x + prefix_w;
                        for (text, style) in words {
                            let span_w =
                                unicode_width::UnicodeWidthStr::width(text.as_str()) as u16;
                            if col >= cur_x && col < cur_x + span_w {
                                match style {
                                    SpanStyle::Link {
                                        link_idx: l_idx,
                                        title,
                                    }
                                    | SpanStyle::BoldLink {
                                        link_idx: l_idx,
                                        title,
                                    } => {
                                        return Some((item_idx, *l_idx, title.clone()));
                                    }
                                    _ => {}
                                }
                            }
                            cur_x += span_w;
                        }
                    }
                    return item.links.first().map(|l| (item_idx, 0, l.clone()));
                }
            }

            None
        }
    }
}

pub fn get_modal_item_line_offset(app: &App, kind: DailyFeedKind, cursor_idx: usize) -> usize {
    match kind {
        DailyFeedKind::MostRead => cursor_idx,
        DailyFeedKind::News => {
            let Some(modal) = &app.modals.daily_feed_modal else {
                return 0;
            };
            let cache_ref = modal.cache.borrow();
            let mut line_offsets = Vec::new();
            let mut line_count = 0;
            for item in &cache_ref.news_wrapped {
                line_offsets.push(line_count);
                line_count += item.wrapped_lines.len() + 1;
            }
            let feed = app.daily_feed.as_ref();
            let has_og = feed.map(|f| !f.ongoing.is_empty()).unwrap_or(false);
            let has_deaths = feed.map(|f| !f.recent_deaths.is_empty()).unwrap_or(false);
            if has_og || has_deaths {
                line_count += 2;
            }
            if has_og {
                line_offsets.push(line_count);
                line_count += 2;
            }
            if has_deaths {
                line_offsets.push(line_count);
            }
            line_offsets.get(cursor_idx).copied().unwrap_or(0)
        }
        DailyFeedKind::OnThisDay => {
            let Some(modal) = &app.modals.daily_feed_modal else {
                return 0;
            };
            let cache_ref = modal.cache.borrow();
            let mut line_offsets = Vec::new();
            let mut line_count = 3;
            for item in &cache_ref.otd_wrapped {
                line_offsets.push(line_count);
                line_count += item.wrapped_lines.len() + 1;
            }
            line_offsets.get(cursor_idx).copied().unwrap_or(0)
        }
    }
}
pub fn get_daily_feed_link_at(app: &App, col: u16, row: u16, size: Rect) -> Option<String> {
    get_daily_feed_item_at(app, col, row, size).map(|(_, _, target)| target)
}

fn idx_offset_matches(idx: usize) -> bool {
    idx > 0
}
