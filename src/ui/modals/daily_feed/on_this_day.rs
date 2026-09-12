use super::html::{parse_onthisday_event, wrap_story_spans};
use super::types::{OnThisDayTab, SpanStyle};
use crate::app::App;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

const MONTH_NAMES: [&str; 12] = [
    "january",
    "february",
    "march",
    "april",
    "may",
    "june",
    "july",
    "august",
    "september",
    "october",
    "november",
    "december",
];

pub fn today_date_str() -> String {
    let (_y, m, d) = crate::api::daily_feed::utc_today();
    let month_str = MONTH_NAMES
        .get(m.saturating_sub(1) as usize)
        .copied()
        .unwrap_or("");
    format!("{} {}", month_str, d)
}

pub fn render_on_this_day_modal(
    f: &mut Frame,
    app: &App,
    modal_area: Rect,
    modal_block: Block,
    selected_idx: usize,
    link_idx: usize,
    otd_tab: OnThisDayTab,
) {
    let feed = match &app.daily_feed {
        Some(f) => f,
        None => return,
    };

    let archive = feed.onthisday_all.as_ref();
    let events_slice: &[crate::api::daily_feed::OnThisDayEvent] = if let Some(arch) = archive {
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

    let avail_w = (modal_area.width as usize).saturating_sub(4);

    let dummy_cache;
    let cache_ref = if let Some(modal_state) = &app.modals.daily_feed_modal {
        let mut cache = modal_state.cache.borrow_mut();
        let parsed_list = match otd_tab {
            OnThisDayTab::Events => &mut cache.otd_events_parsed,
            OnThisDayTab::Births => &mut cache.otd_births_parsed,
            OnThisDayTab::Deaths => &mut cache.otd_deaths_parsed,
            OnThisDayTab::Holidays => &mut cache.otd_holidays_parsed,
        };

        if parsed_list.len() != events_slice.len() {
            *parsed_list = events_slice
                .iter()
                .map(|ev| {
                    let (chunks, links) = parse_onthisday_event(&ev.text, &ev.pages);
                    crate::ui::modals::daily_feed::ParsedStory { chunks, links }
                })
                .collect();
            cache.otd_cached_width = 0;
        }

        if cache.otd_cached_tab != Some(otd_tab) || cache.otd_cached_width != avail_w {
            let current_year = crate::api::daily_feed::utc_today().0 as i32;
            let parsed_list = match otd_tab {
                OnThisDayTab::Events => &cache.otd_events_parsed,
                OnThisDayTab::Births => &cache.otd_births_parsed,
                OnThisDayTab::Deaths => &cache.otd_deaths_parsed,
                OnThisDayTab::Holidays => &cache.otd_holidays_parsed,
            };

            cache.otd_wrapped = events_slice
                .iter()
                .zip(parsed_list.iter())
                .map(|(event, parsed)| {
                    let (year_str, elapsed_str) = match event.year {
                        Some(y) if y < 0 => {
                            let yrs = current_year + y.abs();
                            (format!("{} BC", y.abs()), format!("({} yrs ago) ", yrs))
                        }
                        Some(y) => {
                            let yrs = current_year - y;
                            if yrs == 0 {
                                (format!("{}", y), "(this year) ".to_string())
                            } else {
                                (format!("{}", y), format!("({} yrs ago) ", yrs))
                            }
                        }
                        None => ("Holiday".to_string(), String::new()),
                    };
                    let badge_prefix = format!("[ {} ] {}", year_str, elapsed_str);
                    let badge_len = badge_prefix.chars().count();
                    let text_w = avail_w.saturating_sub(badge_len + 3);
                    let wrapped = wrap_story_spans(&parsed.chunks, text_w + 3);

                    crate::ui::modals::daily_feed::CachedWrappedItem {
                        links: parsed.links.clone(),
                        wrapped_lines: wrapped,
                        year_str,
                        elapsed_str,
                    }
                })
                .collect();
            cache.otd_cached_tab = Some(otd_tab);
            cache.otd_cached_width = avail_w;
        }

        drop(cache);
        modal_state.cache.borrow()
    } else {
        let current_year = crate::api::daily_feed::utc_today().0 as i32;
        dummy_cache = std::cell::RefCell::new(crate::ui::modals::daily_feed::DailyFeedCache {
            otd_wrapped: events_slice
                .iter()
                .map(|event| {
                    let (chunks, links) = parse_onthisday_event(&event.text, &event.pages);
                    let (year_str, elapsed_str) = match event.year {
                        Some(y) if y < 0 => {
                            let yrs = current_year + y.abs();
                            (format!("{} BC", y.abs()), format!("({} yrs ago) ", yrs))
                        }
                        Some(y) => {
                            let yrs = current_year - y;
                            if yrs == 0 {
                                (format!("{}", y), "(this year) ".to_string())
                            } else {
                                (format!("{}", y), format!("({} yrs ago) ", yrs))
                            }
                        }
                        None => ("Holiday".to_string(), String::new()),
                    };
                    let badge_prefix = format!("[ {} ] {}", year_str, elapsed_str);
                    let badge_len = badge_prefix.chars().count();
                    let text_w = avail_w.saturating_sub(badge_len + 3);
                    let wrapped = wrap_story_spans(&chunks, text_w + 3);

                    crate::ui::modals::daily_feed::CachedWrappedItem {
                        links,
                        wrapped_lines: wrapped,
                        year_str,
                        elapsed_str,
                    }
                })
                .collect(),
            ..Default::default()
        });
        dummy_cache.borrow()
    };

    let selected_event = events_slice.get(selected_idx);
    let focused_page = selected_event.and_then(|ev| {
        let links = &cache_ref.otd_wrapped.get(selected_idx)?.links;
        let active_link_idx = link_idx.min(links.len().saturating_sub(1));
        let target = links.get(active_link_idx)?;
        ev.pages
            .iter()
            .find(|p| &p.title == target || p.display_title() == *target)
            .or_else(|| ev.pages.first())
    });

    let mut modal_block = modal_block;
    if let Some(page) = focused_page
        && let Some(desc) = page.description.as_deref().filter(|d| !d.is_empty()) {
            let icon = if app.user_data.config.ui.icons {
                "󰋼 "
            } else {
                ""
            };
            let icon_w = unicode_width::UnicodeWidthStr::width(icon);
            let prefix_w = 1 + icon_w + 2;

            if avail_w > prefix_w + 4 {
                let title = page.display_title();
                let title_w = unicode_width::UnicodeWidthStr::width(title.as_str());

                if prefix_w + title_w >= avail_w {
                    let max_title_w = avail_w.saturating_sub(prefix_w);
                    let clean_title = crate::ui::truncate_to_width(&title, max_title_w);
                    let footer_line = Line::from(vec![Span::styled(
                        format!(" {}{}: ", icon, clean_title),
                        Style::default().fg(theme::BLUE).bold(),
                    )]);
                    modal_block = modal_block.title_bottom(footer_line);
                } else {
                    let max_desc_w = avail_w.saturating_sub(prefix_w + title_w + 1);
                    let clean_desc = if max_desc_w > 3 {
                        Some(crate::ui::truncate_to_width(desc, max_desc_w))
                    } else {
                        None
                    };

                    let mut spans = vec![Span::styled(
                        format!(" {}{}:", icon, title),
                        Style::default().fg(theme::BLUE).bold(),
                    )];
                    if let Some(cd) = clean_desc {
                        spans.push(Span::styled(
                            format!(" {} ", cd),
                            Style::default().fg(theme::GREY).italic(),
                        ));
                    }
                    modal_block = modal_block.title_bottom(Line::from(spans));
                }
            }
        }

    let mut lines = Vec::new();
    let avail_w = (modal_area.width as usize).saturating_sub(4);

    let (ev_count, b_count, d_count, h_count) = if let Some(arch) = archive {
        (
            if !arch.events.is_empty() {
                arch.events.len()
            } else {
                feed.onthisday.len()
            },
            arch.births.len(),
            arch.deaths.len(),
            arch.holidays.len(),
        )
    } else {
        (feed.onthisday.len(), 0, 0, 0)
    };

    let tabs = [
        (OnThisDayTab::Events, "1", "Events", ev_count),
        (OnThisDayTab::Births, "2", "Births", b_count),
        (OnThisDayTab::Deaths, "3", "Deaths", d_count),
        (OnThisDayTab::Holidays, "4", "Holidays", h_count),
    ];

    let mut tab_spans = vec![Span::raw("  ")];
    for (i, (t, num, label, count)) in tabs.iter().enumerate() {
        if i > 0 {
            tab_spans.push(Span::styled("   ", Style::default().fg(theme::DARK_GREY)));
        }
        let is_active = otd_tab == *t;
        if is_active {
            tab_spans.push(Span::styled(
                format!("[{}] {} ({})", num, label, count),
                Style::default().fg(theme::BLUE).bold(),
            ));
        } else {
            tab_spans.push(Span::styled(
                format!("[{}] ", num),
                Style::default().fg(theme::DARK_GREY),
            ));
            tab_spans.push(Span::styled(
                format!("{} ({})", label, count),
                Style::default().fg(theme::GREY),
            ));
        }
    }
    lines.push(Line::from(tab_spans));
    let div_w = avail_w.saturating_sub(2);
    lines.push(Line::from(vec![
        Span::raw("  "),
        Span::styled("─".repeat(div_w), Style::default().fg(theme::DARK_GREY)),
    ]));
    lines.push(Line::from(""));

    let mut line_offsets = Vec::new();
    if events_slice.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  no entries available in this category.",
            Style::default().fg(theme::GREY).italic(),
        )]));
    } else {
        for (idx, item) in cache_ref.otd_wrapped.iter().enumerate() {
            line_offsets.push(lines.len());
            let is_selected = idx == selected_idx;
            let active_link_idx = if is_selected {
                link_idx.min(item.links.len().saturating_sub(1))
            } else {
                0
            };
            let badge_prefix = format!("[ {} ] {}", item.year_str, item.elapsed_str);
            let badge_len = badge_prefix.chars().count();

            for (line_idx, line_words) in item.wrapped_lines.iter().enumerate() {
                let mut spans = Vec::new();
                if line_idx == 0 {
                    let prefix = if is_selected { " ▶ " } else { "   " };
                    let prefix_style = if is_selected {
                        Style::default().fg(theme::BLUE).bold()
                    } else {
                        Style::default().fg(theme::GREY)
                    };
                    spans.push(Span::styled(prefix, prefix_style));
                    spans.push(Span::styled("[ ", Style::default().fg(theme::DARK_GREY)));
                    spans.push(Span::styled(
                        item.year_str.clone(),
                        Style::default().fg(theme::BLUE).bold(),
                    ));
                    spans.push(Span::styled(" ] ", Style::default().fg(theme::DARK_GREY)));
                    if !item.elapsed_str.is_empty() {
                        spans.push(Span::styled(
                            item.elapsed_str.clone(),
                            Style::default().fg(theme::GREY).italic(),
                        ));
                    }
                } else {
                    let pad_len = 3 + badge_len;
                    spans.push(Span::raw(" ".repeat(pad_len)));
                }

                for (text, style) in line_words {
                    let span_style = match style {
                        SpanStyle::Normal => {
                            if is_selected {
                                Style::default().fg(theme::FG).bold()
                            } else {
                                Style::default().fg(theme::FG)
                            }
                        }
                        SpanStyle::Bold => Style::default().fg(theme::FG).bold(),
                        SpanStyle::Italic => Style::default().fg(theme::GREY).italic(),
                        SpanStyle::Link {
                            link_idx: l_idx, ..
                        } => {
                            if is_selected && *l_idx == active_link_idx {
                                Style::default()
                                    .fg(theme::VIOLET)
                                    .bold()
                                    .add_modifier(Modifier::UNDERLINED)
                            } else if app.user_data.config.reader.underline_links {
                                Style::default()
                                    .fg(theme::BLUE)
                                    .add_modifier(Modifier::UNDERLINED)
                            } else {
                                Style::default().fg(theme::BLUE)
                            }
                        }
                        SpanStyle::BoldLink {
                            link_idx: l_idx, ..
                        } => {
                            if is_selected && *l_idx == active_link_idx {
                                Style::default()
                                    .fg(theme::VIOLET)
                                    .bold()
                                    .add_modifier(Modifier::UNDERLINED)
                            } else if app.user_data.config.reader.underline_links {
                                Style::default()
                                    .fg(theme::BLUE)
                                    .bold()
                                    .add_modifier(Modifier::UNDERLINED)
                            } else {
                                Style::default().fg(theme::BLUE).bold()
                            }
                        }
                    };
                    spans.push(Span::styled(text.clone(), span_style));
                }
                lines.push(Line::from(spans));
            }
            lines.push(Line::from(""));
        }
    }

    let inner_height = modal_area.height.saturating_sub(2) as usize;
    let total_lines = lines.len();
    let scroll = app
        .modals
        .daily_feed_modal
        .as_ref()
        .map(|m| m.scroll)
        .unwrap_or(0)
        .min(total_lines.saturating_sub(inner_height));

    let p = Paragraph::new(lines)
        .block(modal_block)
        .scroll((scroll as u16, 0));
    f.render_widget(p, modal_area);
}

pub fn get_otd_tab_at(
    modal_area: Rect,
    col: u16,
    row: u16,
    feed: Option<&crate::api::DailyFeed>,
) -> Option<OnThisDayTab> {
    let tab_row = modal_area.y + 1;
    if row != tab_row {
        return None;
    }

    let feed = feed?;
    let archive = feed.onthisday_all.as_ref();
    let (ev_count, b_count, d_count, h_count) = if let Some(arch) = archive {
        (
            if !arch.events.is_empty() {
                arch.events.len()
            } else {
                feed.onthisday.len()
            },
            arch.births.len(),
            arch.deaths.len(),
            arch.holidays.len(),
        )
    } else {
        (feed.onthisday.len(), 0, 0, 0)
    };

    let tabs = [
        (OnThisDayTab::Events, "1", "Events", ev_count),
        (OnThisDayTab::Births, "2", "Births", b_count),
        (OnThisDayTab::Deaths, "3", "Deaths", d_count),
        (OnThisDayTab::Holidays, "4", "Holidays", h_count),
    ];

    let mut current_x = modal_area.x + 1 + 2;
    for (i, (tab_type, num, label, count)) in tabs.into_iter().enumerate() {
        if i > 0 {
            current_x += 3;
        }
        let tab_len = format!("[{}] {} ({})", num, label, count).chars().count() as u16;
        if col >= current_x && col < current_x + tab_len {
            return Some(tab_type);
        }
        current_x += tab_len;
    }

    None
}
