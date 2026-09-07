use crate::app::App;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Clear, Paragraph},
    Frame,
};

pub fn render_link_peek(f: &mut Frame, app: &App, size: Rect) {
    let Some(peek) = &app.link_peek else {
        return;
    };

    let width = 52u16.min(size.width.saturating_sub(4)).max(30);
    let height = 6u16.min(size.height.saturating_sub(2)).max(4);

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

    let area = Rect::new(x, y, width, height);
    f.render_widget(Clear, area);

    let max_title_chars = (width as usize).saturating_sub(6);
    let display_title = if peek.title.chars().count() > max_title_chars {
        let truncated: String = peek.title.chars().take(max_title_chars).collect();
        format!("{}…", truncated)
    } else {
        peek.title.clone()
    };

    let border_type = theme::border_type(app.config.ui.rounded_borders);
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
        let spinner = crate::ui::current_spinner_frame();
        lines.push(Line::from(vec![
            Span::styled(format!("{} ", spinner), Style::default().fg(theme::LIME).bold()),
            Span::styled("loading summary...", Style::default().fg(theme::GREY).italic()),
        ]));
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
                    let text = if idx == max_extract_rows - 1 && extract.len() > w_line.len() * max_extract_rows {
                        if w_line.ends_with('.') || w_line.ends_with('…') {
                            w_line
                        } else {
                            format!("{}…", w_line.trim_end())
                        }
                    } else {
                        w_line
                    };
                    lines.push(Line::from(Span::styled(text, Style::default().fg(theme::FG))));
                }
            } else if lines.is_empty() {
                lines.push(Line::from(Span::styled(
                    "no summary available",
                    Style::default().fg(theme::GREY).italic(),
                )));
            }
        }
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
