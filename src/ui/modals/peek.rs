use crate::app::App;
use crate::theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

pub fn compute_link_peek_area(peek: &crate::app::types::LinkPeekState, size: Rect) -> Rect {
    let max_w = size.width.saturating_sub(4).clamp(34, 68);
    let title_chars = peek.title.chars().count();
    let desc_chars = peek
        .description
        .as_ref()
        .map(|d| d.chars().count())
        .unwrap_or(0);

    let (width, height) = if peek.is_loading {
        let w = ((title_chars + 6) as u16).max(34).min(max_w);
        (w, 4u16)
    } else {
        let has_desc = peek
            .description
            .as_ref()
            .is_some_and(|d| !d.trim().is_empty());
        let desc_lines = if has_desc { 1usize } else { 0usize };

        let candidate_w = if let Some(extract) = &peek.extract {
            let total_chars = extract.chars().count();
            if total_chars < 70 {
                ((total_chars as u16 + 4)
                    .max(title_chars as u16 + 6)
                    .max(desc_chars as u16 + 4))
                .max(34)
                .min(max_w)
            } else {
                max_w
            }
        } else {
            ((title_chars as u16 + 6).max(desc_chars as u16 + 4))
                .max(34)
                .min(max_w)
        };

        let inner_w = candidate_w.saturating_sub(2) as usize;
        let extract_lines = if let Some(extract) = &peek.extract {
            let wrapped = wrap_text(extract, inner_w);
            wrapped.len().clamp(1, 7)
        } else if !has_desc {
            1usize
        } else {
            0usize
        };

        let inner_h = (desc_lines + extract_lines).clamp(1, 8);
        let h = ((inner_h as u16) + 2).min(size.height.saturating_sub(2));
        (candidate_w, h)
    };

    let x = peek
        .anchor_x
        .saturating_sub(2)
        .min(size.width.saturating_sub(width + 2))
        .max(2);

    let y = if peek.anchor_y + 1 + height <= size.height.saturating_sub(1) {
        peek.anchor_y + 1
    } else {
        peek.anchor_y.saturating_sub(height).max(1)
    };

    Rect::new(x, y, width, height)
}

pub fn render_link_peek(f: &mut Frame, app: &App, size: Rect) {
    let Some(peek) = &app.modals.link_peek else {
        return;
    };

    let area = compute_link_peek_area(peek, size);
    f.render_widget(Clear, area);

    let max_title_chars = (area.width as usize).saturating_sub(6);
    let display_title = if peek.title.chars().count() > max_title_chars {
        let truncated: String = peek.title.chars().take(max_title_chars).collect();
        format!("{}…", truncated)
    } else {
        peek.title.clone()
    };

    let border_type = theme::border_type(app.user_data.config.ui.rounded_borders);
    let block = Block::bordered()
        .border_type(border_type)
        .border_style(Style::default().fg(theme::PINK))
        .style(Style::default().bg(theme::BG))
        .title(Span::styled(
            format!(" {} ", display_title),
            Style::default().fg(theme::PINK).bold(),
        ));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.width < 4 || inner.height < 1 {
        return;
    }

    let mut lines = Vec::new();

    if peek.is_loading {
        let pad_top = (inner.height.saturating_sub(1)) / 2;
        for _ in 0..pad_top {
            lines.push(Line::from(""));
        }
        let spinner = crate::ui::current_spinner_frame();
        lines.push(
            Line::from(vec![
                Span::styled(
                    format!("{} ", spinner),
                    Style::default().fg(theme::LIME).bold(),
                ),
                Span::styled(
                    "loading summary...",
                    Style::default().fg(theme::GREY).italic(),
                ),
            ])
            .alignment(Alignment::Center),
        );
    } else {
        if let Some(desc) = &peek.description {
            if !desc.trim().is_empty() {
                let max_desc = inner.width as usize;
                let desc_text = if desc.chars().count() > max_desc {
                    let d: String = desc.chars().take(max_desc.saturating_sub(1)).collect();
                    format!("{}…", d)
                } else {
                    desc.clone()
                };
                lines.push(Line::from(Span::styled(
                    desc_text,
                    Style::default().fg(theme::BEIGE).italic(),
                )));
            }
        }

        let max_extract_rows = (inner.height as usize).saturating_sub(lines.len());
        if max_extract_rows > 0 {
            if let Some(extract) = &peek.extract {
                let wrapped = wrap_text(extract, inner.width as usize);
                for (idx, w_line) in wrapped.into_iter().take(max_extract_rows).enumerate() {
                    let text = if idx == max_extract_rows - 1
                        && extract.len() > w_line.len() * max_extract_rows
                    {
                        if w_line.ends_with('.') || w_line.ends_with('…') {
                            w_line
                        } else {
                            format!("{}…", w_line.trim_end())
                        }
                    } else {
                        w_line
                    };
                    lines.push(Line::from(Span::styled(
                        text,
                        Style::default().fg(theme::FG),
                    )));
                }
            } else if lines.is_empty() {
                lines.push(Line::from(Span::styled(
                    "no summary available",
                    Style::default().fg(theme::GREY).italic(),
                )));
            }
        }
    }

    while lines.len() < inner.height as usize {
        lines.push(Line::from(""));
    }

    let p = Paragraph::new(lines).style(Style::default().bg(theme::BG));
    f.render_widget(p, inner);
}

fn wrap_text(text: &str, max_width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_len = 0;

    for word in text.split_whitespace() {
        let word_len = unicode_width::UnicodeWidthStr::width(word);
        if current_line.is_empty() {
            current_line.push_str(word);
            current_len = word_len;
        } else if current_len + 1 + word_len <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
            current_len += 1 + word_len;
        } else {
            lines.push(current_line);
            current_line = word.to_string();
            current_len = word_len;
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
