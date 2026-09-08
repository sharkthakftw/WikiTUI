use super::utils::render_modal_container_at;
use crate::app::App;
use crate::theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_qr_modal_area(size: Rect) -> Rect {
    let width = 48.min(size.width.saturating_sub(2));
    let height = 26.min(size.height.saturating_sub(2));
    let x = (size.width.saturating_sub(width)) / 2;
    let y = (size.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

pub fn render_qr_modal(f: &mut Frame, app: &App, size: Rect) {
    let Some(qr_state) = &app.modals.qr_modal else {
        return;
    };

    let icon = if app.user_data.config.ui.icons {
        ""
    } else {
        ""
    };
    let area = compute_qr_modal_area(size);
    let inner = render_modal_container_at(
        f,
        area,
        icon,
        "qr code",
        theme::PINK,
        app.user_data.config.ui.rounded_borders,
    );

    let h = inner.height as usize;
    if h == 0 {
        return;
    }

    let n = qr_state.matrix.len();
    let quiet_zone = 2;
    let total_w = n + quiet_zone * 2;
    let total_h = n + quiet_zone * 2;
    let qr_rows = total_h.div_ceil(2);

    let is_dark = |x: usize, y: usize| -> bool {
        if x < quiet_zone || x >= quiet_zone + n || y < quiet_zone || y >= quiet_zone + n {
            false
        } else {
            qr_state.matrix[y - quiet_zone][x - quiet_zone]
        }
    };

    let mut qr_lines = Vec::with_capacity(qr_rows);
    for y in (0..total_h).step_by(2) {
        let mut row_str = String::with_capacity(total_w);
        for x in 0..total_w {
            let top = is_dark(x, y);
            let bot = if y + 1 < total_h {
                is_dark(x, y + 1)
            } else {
                false
            };
            let ch = match (top, bot) {
                (true, true) => '█',
                (true, false) => '▀',
                (false, true) => '▄',
                (false, false) => ' ',
            };
            row_str.push(ch);
        }
        qr_lines.push(
            Line::from(vec![Span::styled(
                row_str,
                Style::default().fg(theme::PINK).bg(theme::BG),
            )])
            .alignment(Alignment::Center),
        );
    }

    let title_line = Line::from(vec![Span::styled(
        format!(" {} ", qr_state.title.to_lowercase()),
        Style::default().fg(theme::PINK).bold(),
    )])
    .alignment(Alignment::Center);

    let max_url_len = (inner.width as usize).saturating_sub(3);
    let url_display = if qr_state.full_url.chars().count() > inner.width as usize {
        let truncated: String = qr_state.full_url.chars().take(max_url_len).collect();
        format!("{}…", truncated)
    } else {
        qr_state.full_url.clone()
    };
    let url_line = Line::from(vec![Span::styled(
        url_display,
        Style::default().fg(theme::GREY),
    )])
    .alignment(Alignment::Center);

    let footer_line = Line::from(vec![
        Span::styled("esc/q", Style::default().fg(theme::YELLOW).bold()),
        Span::styled(" close   ", Style::default().fg(theme::GREY)),
        Span::styled("y", Style::default().fg(theme::YELLOW).bold()),
        Span::styled(" copy link", Style::default().fg(theme::GREY)),
    ])
    .alignment(Alignment::Center);

    let mut lines = Vec::with_capacity(h);

    if h < qr_rows + 3 {
        lines.push(title_line);
        lines.extend(qr_lines);
        lines.push(url_line);
        lines.push(footer_line);
    } else {
        let modal_center = h.saturating_sub(1) / 2;
        let ideal_qr_start = modal_center.saturating_sub((qr_rows.saturating_sub(1)) / 2);
        let qr_start = ideal_qr_start.max(1).min(h.saturating_sub(qr_rows + 2));
        let qr_end = qr_start + qr_rows;

        let top_blanks = qr_start.saturating_sub(2);
        for _ in 0..top_blanks {
            lines.push(Line::from(""));
        }
        lines.push(title_line);
        if qr_start >= 2 {
            lines.push(Line::from(""));
        }

        lines.extend(qr_lines);

        let lines_below = h.saturating_sub(qr_end);
        let slack_after = lines_below.saturating_sub(2);
        let gap1 = if slack_after >= 1 { 1 } else { 0 };
        let gap2 = if slack_after >= 2 { 1 } else { 0 };
        let bottom_pad = slack_after.saturating_sub(2);

        for _ in 0..gap1 {
            lines.push(Line::from(""));
        }
        lines.push(url_line);
        for _ in 0..gap2 {
            lines.push(Line::from(""));
        }
        lines.push(footer_line);
        for _ in 0..bottom_pad {
            lines.push(Line::from(""));
        }
    }

    let p = Paragraph::new(lines).style(Style::default().bg(theme::BG));
    f.render_widget(p, inner);
}
