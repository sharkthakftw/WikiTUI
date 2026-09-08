use crate::app::App;
use crate::palette::filter_commands;
use crate::theme;
use crate::ui::modals::utils::{
    compute_centered_scroll, create_search_input_lines, render_modal_container_at,
};
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_palette_modal_area(size: Rect) -> Rect {
    let target_width = (size.width * 60 / 100).clamp(40, 70);
    let target_height = (size.height * 50 / 100).clamp(12, 20);
    let x = (size.width.saturating_sub(target_width)) / 2;
    let y = (size.height.saturating_sub(target_height)) / 3;
    Rect::new(x, y, target_width, target_height)
}

pub fn render_palette_modal(f: &mut Frame, app: &App, size: Rect) {
    let area = compute_palette_modal_area(size);
    let icon = if app.user_data.config.ui.icons {
        ">"
    } else {
        ""
    };
    let inner = render_modal_container_at(
        f,
        area,
        icon,
        "command palette",
        theme::YELLOW,
        app.user_data.config.ui.rounded_borders,
    );

    if inner.height < 3 || inner.width < 10 {
        return;
    }

    let inner_width = inner.width as usize;
    let mut lines = Vec::new();

    let (input_line, divider_line) = create_search_input_lines(
        ">",
        &app.modals.command_palette.query,
        theme::YELLOW,
        inner_width,
    );
    lines.push(input_line);
    lines.push(divider_line);

    let filtered = filter_commands(&app.modals.command_palette.query);
    let visible_rows = (inner.height as usize).saturating_sub(2);
    let scroll_offset = compute_centered_scroll(
        app.modals.command_palette.selected_idx,
        visible_rows,
        filtered.len(),
    );

    if filtered.is_empty() {
        lines.push(Line::from(Span::styled(
            "  no matching commands",
            Style::default().fg(theme::GREY).italic(),
        )));
    } else {
        for (idx, (cmd, match_indices)) in filtered
            .iter()
            .enumerate()
            .skip(scroll_offset)
            .take(visible_rows)
        {
            let is_selected = idx == app.modals.command_palette.selected_idx;
            let cursor_str = if is_selected { " ▶ " } else { "   " };
            let cursor_style = if is_selected {
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::DARK_GREY)
            };

            let base_style = if is_selected {
                Style::default().fg(theme::FG).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::FG)
            };

            let highlight_style = Style::default()
                .fg(theme::YELLOW)
                .add_modifier(Modifier::BOLD);

            let mut item_spans = vec![Span::styled(cursor_str, cursor_style)];

            for (char_idx, ch) in cmd.label.chars().enumerate() {
                if match_indices.contains(&char_idx) {
                    item_spans.push(Span::styled(ch.to_string(), highlight_style));
                } else {
                    item_spans.push(Span::styled(ch.to_string(), base_style));
                }
            }

            if let Some(shortcut) = cmd.shortcut {
                let current_len = 3 + cmd.label.chars().count();
                let shortcut_len = shortcut.chars().count();
                if inner_width > current_len + shortcut_len + 2 {
                    let pad = inner_width - current_len - shortcut_len - 1;
                    item_spans.push(Span::raw(" ".repeat(pad)));
                    item_spans.push(Span::styled(
                        shortcut.to_string(),
                        Style::default().fg(theme::GREY),
                    ));
                }
            }

            lines.push(Line::from(item_spans));
        }
    }

    let p = Paragraph::new(lines);
    f.render_widget(p, inner);
}
