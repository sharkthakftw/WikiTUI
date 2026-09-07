use crate::app::App;
use crate::theme;
use crate::ui::modals::utils::render_modal_frame_at;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_search_modal_area(size: Rect) -> Rect {
    let width = (size.width * 30 / 100).clamp(30, 60).min(size.width);
    let x = (size.width.saturating_sub(width)) / 2;
    let y = (size.height * 40 / 100).min(size.height.saturating_sub(3));
    Rect::new(x, y, width, 3.min(size.height))
}

pub fn render_search_modal(f: &mut Frame, app: &App, size: Rect) {
    let area = compute_search_modal_area(size);
    let (icon, title, border_color) = if app.input_mode == crate::app::InputMode::RenameList {
        let ic = if app.config.ui.icons { "★" } else { "" };
        (ic, "rename list", theme::VIOLET)
    } else if app.input_mode == crate::app::InputMode::CreateNewList {
        let ic = if app.config.ui.icons { "★" } else { "" };
        (ic, "create new list", theme::VIOLET)
    } else if app.input_mode == crate::app::InputMode::SleepTimerPrompt {
        let ic = if app.config.ui.icons { "󰔛" } else { "" };
        (ic, "sleep timer (minutes, or 'off')", theme::VIOLET)
    } else {
        let ic = if app.config.ui.icons { "󰍉" } else { "" };
        (ic, "search wikipedia", theme::BEIGE)
    };
    let search_block = render_modal_frame_at(
        f,
        area,
        icon,
        title,
        border_color,
        app.config.ui.rounded_borders,
    );

    let visible_width = (area.width as usize).saturating_sub(6);
    let chars: Vec<char> = app.search_modal.input.chars().collect();
    let total_len = chars.len();
    let cursor_pos = app.search_modal.cursor_pos.min(total_len);

    let mut scroll_offset = 0;
    if cursor_pos >= visible_width && visible_width > 0 {
        scroll_offset = cursor_pos + 1 - visible_width;
    }

    let end_idx = (scroll_offset + visible_width).min(total_len);
    let visible_chars = &chars[scroll_offset..end_idx];
    let rel_cursor_pos = cursor_pos.saturating_sub(scroll_offset);

    let mut spans = Vec::new();
    spans.push(Span::styled(
        " > ",
        Style::default().fg(border_color).bold(),
    ));

    for (i, &ch) in visible_chars.iter().enumerate() {
        if i == rel_cursor_pos {
            spans.push(Span::styled(
                ch.to_string(),
                Style::default().bg(border_color).fg(theme::BG).bold(),
            ));
        } else {
            spans.push(Span::styled(
                ch.to_string(),
                Style::default().fg(theme::FG).bold(),
            ));
        }
    }

    if rel_cursor_pos >= visible_chars.len() {
        spans.push(Span::styled("_", Style::default().fg(border_color).bold()));
    }

    let input_text = Line::from(spans);
    let search_paragraph = Paragraph::new(input_text).block(search_block);
    f.render_widget(search_paragraph, area);
}
