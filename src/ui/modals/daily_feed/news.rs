use super::html::{parse_story_html, wrap_story_spans};
use super::types::SpanStyle;
use crate::app::App;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub fn get_ongoing_links(ongoing: &[crate::api::daily_feed::OngoingItem]) -> Vec<(String, String)> {
    ongoing
        .iter()
        .flat_map(|og| {
            std::iter::once((og.target.clone(), og.display.clone()))
                .chain(og.sub_events.iter().cloned())
        })
        .collect()
}

pub fn get_recent_deaths_links(
    deaths: &[crate::api::daily_feed::RecentDeathItem],
) -> Vec<(String, String)> {
    deaths
        .iter()
        .map(|d| (d.target.clone(), d.name.clone()))
        .collect()
}

pub fn render_news_modal(
    f: &mut Frame,
    app: &App,
    modal_area: Rect,
    modal_block: Block,
    selected_idx: usize,
    link_idx: usize,
) {
    let feed = match &app.daily_feed {
        Some(f) => f,
        None => return,
    };

    let mut lines = Vec::new();
    let mut line_offsets = Vec::new();
    if feed.news.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  no news stories available.",
            Style::default().fg(theme::GREY).italic(),
        )]));
    } else {
        let avail_w = (modal_area.width as usize).saturating_sub(4);
        let mut row_counter = 0;

        let dummy_cache;
        let cache_ref = if let Some(modal_state) = &app.modals.daily_feed_modal {
            let mut cache = modal_state.cache.borrow_mut();
            if cache.news_parsed.len() != feed.news.len() {
                cache.news_parsed = feed
                    .news
                    .iter()
                    .map(|item| {
                        let raw_story = item.story.as_deref().unwrap_or("");
                        let (chunks, links) = parse_story_html(raw_story);
                        crate::ui::modals::daily_feed::ParsedStory { chunks, links }
                    })
                    .collect();
                cache.news_width = 0;
            }

            if cache.news_width != avail_w {
                cache.news_wrapped = cache
                    .news_parsed
                    .iter()
                    .map(|parsed| {
                        let wrapped = wrap_story_spans(&parsed.chunks, avail_w);
                        crate::ui::modals::daily_feed::CachedWrappedItem {
                            links: parsed.links.clone(),
                            wrapped_lines: wrapped,
                            year_str: String::new(),
                            elapsed_str: String::new(),
                        }
                    })
                    .collect();
                cache.news_width = avail_w;
            }
            drop(cache);
            modal_state.cache.borrow()
        } else {
            dummy_cache = std::cell::RefCell::new(crate::ui::modals::daily_feed::DailyFeedCache {
                news_wrapped: feed
                    .news
                    .iter()
                    .map(|item| {
                        let raw_story = item.story.as_deref().unwrap_or("");
                        let (chunks, links) = parse_story_html(raw_story);
                        let wrapped = wrap_story_spans(&chunks, avail_w);
                        crate::ui::modals::daily_feed::CachedWrappedItem {
                            links,
                            wrapped_lines: wrapped,
                            year_str: String::new(),
                            elapsed_str: String::new(),
                        }
                    })
                    .collect(),
                ..Default::default()
            });
            dummy_cache.borrow()
        };

        for cached_item in &cache_ref.news_wrapped {
            line_offsets.push(lines.len());
            let is_selected = row_counter == selected_idx;
            let active_link_idx = if is_selected {
                link_idx.min(cached_item.links.len().saturating_sub(1))
            } else {
                0
            };

            for (line_idx, line_words) in cached_item.wrapped_lines.iter().enumerate() {
                let (prefix, prefix_style) = if line_idx == 0 {
                    if is_selected {
                        (" ▶ ", Style::default().fg(theme::BLUE).bold())
                    } else {
                        ("   ", Style::default().fg(theme::GREY))
                    }
                } else {
                    ("   ", Style::default().fg(theme::GREY))
                };

                let mut spans = vec![Span::styled(prefix, prefix_style)];
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
            row_counter += 1;
        }

        if !feed.ongoing.is_empty() || !feed.recent_deaths.is_empty() {
            let div_w = avail_w.saturating_sub(2);
            lines.push(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled("─".repeat(div_w), Style::default().fg(theme::DARK_GREY)),
            ]));
            lines.push(Line::from(""));
        }

        if !feed.ongoing.is_empty() {
            line_offsets.push(lines.len());
            let is_selected = row_counter == selected_idx;
            let (prefix, prefix_style) = if is_selected {
                (" ▶ ", Style::default().fg(theme::BLUE).bold())
            } else {
                ("   ", Style::default().fg(theme::GREY))
            };
            let mut spans = vec![
                Span::styled(prefix, prefix_style),
                Span::styled("Ongoing: ", Style::default().fg(theme::FG).bold()),
            ];

            let ongoing_links = get_ongoing_links(&feed.ongoing);
            let active_link_idx = if is_selected {
                link_idx.min(ongoing_links.len().saturating_sub(1))
            } else {
                0
            };

            let mut link_counter = 0;
            for (og_idx, og) in feed.ongoing.iter().enumerate() {
                if og_idx > 0 {
                    spans.push(Span::styled(" · ", Style::default().fg(theme::GREY)));
                }

                let cur_l_idx = link_counter;
                link_counter += 1;
                let main_style = if is_selected && cur_l_idx == active_link_idx {
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
                };
                spans.push(Span::styled(og.display.clone(), main_style));

                for (_sub_target, sub_display) in &og.sub_events {
                    let sub_l_idx = link_counter;
                    link_counter += 1;
                    spans.push(Span::styled(" (", Style::default().fg(theme::GREY)));
                    let sub_style = if is_selected && sub_l_idx == active_link_idx {
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
                    };
                    spans.push(Span::styled(sub_display.clone(), sub_style));
                    spans.push(Span::styled(")", Style::default().fg(theme::GREY)));
                }
            }
            lines.push(Line::from(spans));
            lines.push(Line::from(""));
            row_counter += 1;
        }

        if !feed.recent_deaths.is_empty() {
            line_offsets.push(lines.len());
            let is_selected = row_counter == selected_idx;
            let (prefix, prefix_style) = if is_selected {
                (" ▶ ", Style::default().fg(theme::BLUE).bold())
            } else {
                ("   ", Style::default().fg(theme::GREY))
            };
            let mut spans = vec![
                Span::styled(prefix, prefix_style),
                Span::styled("Recent deaths: ", Style::default().fg(theme::FG).bold()),
            ];

            let active_link_idx = if is_selected {
                link_idx.min(feed.recent_deaths.len().saturating_sub(1))
            } else {
                0
            };

            for (d_idx, death) in feed.recent_deaths.iter().enumerate() {
                if d_idx > 0 {
                    spans.push(Span::styled(" · ", Style::default().fg(theme::GREY)));
                }
                let death_style = if is_selected && d_idx == active_link_idx {
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
                };
                spans.push(Span::styled(death.name.clone(), death_style));
            }
            lines.push(Line::from(spans));
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
