use super::scrollbar::render_scroll_indicator;
use crate::api::SearchResultItem;
use crate::app::pane::Pane;
use crate::theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;

    for word in text.split_whitespace() {
        let word_width = unicode_width::UnicodeWidthStr::width(word);
        if current_line.is_empty() {
            current_line.push_str(word);
            current_width = word_width;
        } else if current_width + 1 + word_width <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
            current_width += 1 + word_width;
        } else {
            lines.push(current_line);
            current_line = word.to_string();
            current_width = word_width;
        }
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines
}

pub fn count_wrapped_lines(text: &str, max_width: usize) -> usize {
    if text.trim().is_empty() {
        0
    } else {
        wrap_text(text, max_width).len()
    }
}

pub fn compute_search_result_lines_count(
    items: &[SearchResultItem],
    selected_idx: usize,
    inner_width: usize,
) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let snippet_lines = if !item.snippet.is_empty() {
                let wrap_w = if i == selected_idx {
                    inner_width.saturating_sub(3).max(10)
                } else {
                    inner_width.saturating_sub(4).max(10)
                };
                count_wrapped_lines(&item.snippet, wrap_w)
            } else {
                0
            };
            1 + snippet_lines + 1
        })
        .collect()
}

pub fn get_search_result_at_line(
    items: &[SearchResultItem],
    selected_idx: usize,
    inner_width: usize,
    target_line: usize,
) -> Option<usize> {
    let mut cur_line = 0;
    let counts = compute_search_result_lines_count(items, selected_idx, inner_width);
    for (i, count) in counts.iter().enumerate() {
        if target_line >= cur_line && target_line < cur_line + count {
            return Some(i);
        }
        cur_line += count;
    }
    None
}

#[allow(clippy::too_many_arguments)]
pub fn render_search_pane(
    f: &mut Frame,
    rect: Rect,
    block: Block,
    pane: &Pane,
    items: &[SearchResultItem],
    border_color: ratatui::style::Color,
    is_active: bool,
    zen_mode: bool,
    scroll_indicator: bool,
    show_icons: bool,
    should_dim: bool,
) {
    if items.is_empty() {
        let vertical_offset = (rect.height.saturating_sub(2) / 2) as usize;
        let mut lines = Vec::new();
        for _ in 0..vertical_offset {
            lines.push(Line::from(""));
        }
        let no_res_style = if should_dim {
            Style::default()
                .fg(theme::RED)
                .bold()
                .add_modifier(Modifier::DIM)
        } else {
            Style::default().fg(theme::RED).bold()
        };
        lines.push(Line::from(Span::styled(
            "no search results found",
            no_res_style,
        )));
        let no_res_p = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(no_res_p, rect);
    } else {
        let inner_width = (rect.width as usize).saturating_sub(4).max(20);
        let item_counts = compute_search_result_lines_count(items, pane.selected_idx, inner_width);
        let total_lines: usize = item_counts.iter().sum();
        let view_start = pane.scroll_offset;
        let view_len = (pane.viewport_height + 2).min(total_lines.saturating_sub(view_start));
        let view_end = view_start + view_len;

        let mut rendered_lines = Vec::with_capacity(view_len);
        let mut cur_line = 0;

        for (i, item) in items.iter().enumerate() {
            let item_height = item_counts[i];
            let item_start = cur_line;
            let item_end = item_start + item_height;
            cur_line = item_end;

            if item_end <= view_start {
                continue;
            }
            if item_start >= view_end {
                break;
            }

            let is_selected = i == pane.selected_idx;
            let title_lower = item.title.to_lowercase();
            let snippet_lower = item.snippet.to_lowercase();
            let audio_str = if item.has_audio {
                if show_icons {
                    " 󰎆"
                } else {
                    " ♪"
                }
            } else {
                ""
            };
            let audio_w = unicode_width::UnicodeWidthStr::width(audio_str);

            let mut item_lines = Vec::with_capacity(item_height);

            if is_selected {
                let badge_str = format!(" {} ", i + 1);
                let badge_w = unicode_width::UnicodeWidthStr::width(badge_str.as_str());
                let title_w = unicode_width::UnicodeWidthStr::width(title_lower.as_str());
                let pad_1 = inner_width.saturating_sub(badge_w + 1 + title_w + audio_w);

                let mut title_spans = vec![
                    Span::styled(
                        badge_str,
                        Style::default().bg(theme::LIME).fg(theme::BG).bold(),
                    ),
                    Span::styled(" ", Style::default().bg(theme::LIGHT_BG)),
                    Span::styled(
                        title_lower,
                        Style::default().bg(theme::LIGHT_BG).fg(theme::LIME).bold(),
                    ),
                ];
                if !audio_str.is_empty() {
                    title_spans.push(Span::styled(
                        audio_str,
                        Style::default().bg(theme::LIGHT_BG).fg(theme::PINK).bold(),
                    ));
                }
                title_spans.push(Span::styled(
                    " ".repeat(pad_1),
                    Style::default().bg(theme::LIGHT_BG),
                ));
                item_lines.push(Line::from(title_spans));

                if !snippet_lower.is_empty() {
                    let wrap_w = inner_width.saturating_sub(3).max(10);
                    for s_line in wrap_text(&snippet_lower, wrap_w) {
                        let s_w = unicode_width::UnicodeWidthStr::width(s_line.as_str());
                        let pad_s = inner_width.saturating_sub(3 + s_w);
                        item_lines.push(Line::from(vec![
                            Span::styled("   ", Style::default().bg(theme::LIGHT_BG)),
                            Span::styled(
                                s_line,
                                Style::default().bg(theme::LIGHT_BG).fg(theme::GREY),
                            ),
                            Span::styled(" ".repeat(pad_s), Style::default().bg(theme::LIGHT_BG)),
                        ]));
                    }
                }
            } else {
                let mut title_spans = vec![
                    Span::styled(
                        format!(" {:>2} ", i + 1),
                        Style::default().fg(theme::DARK_GREY),
                    ),
                    Span::styled(title_lower, Style::default().fg(theme::FG).bold()),
                ];
                if !audio_str.is_empty() {
                    title_spans.push(Span::styled(audio_str, Style::default().fg(theme::PINK)));
                }
                item_lines.push(Line::from(title_spans));

                if !snippet_lower.is_empty() {
                    let wrap_w = inner_width.saturating_sub(4).max(10);
                    for s_line in wrap_text(&snippet_lower, wrap_w) {
                        item_lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled(s_line, Style::default().fg(theme::GREY)),
                        ]));
                    }
                }
            }
            item_lines.push(Line::from(""));

            for (offset, line) in item_lines.into_iter().enumerate() {
                let line_idx = item_start + offset;
                if line_idx >= view_start && line_idx < view_end {
                    rendered_lines.push(line);
                }
            }
        }
        if should_dim {
            for line in &mut rendered_lines {
                for span in &mut line.spans {
                    span.style = span.style.add_modifier(Modifier::DIM);
                }
            }
        }
        let results_p = Paragraph::new(rendered_lines).block(block);
        f.render_widget(results_p, rect);

        render_scroll_indicator(
            f,
            rect,
            total_lines,
            pane.viewport_height,
            pane.scroll_offset,
            border_color,
            is_active,
            zen_mode,
            scroll_indicator,
        );
    }
}
