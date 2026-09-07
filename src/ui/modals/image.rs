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

pub fn compute_image_modal_area(size: Rect) -> Rect {
    let width = (size.width * 90 / 100)
        .max(40)
        .min(size.width.saturating_sub(2));
    let height = (size.height * 90 / 100)
        .max(15)
        .min(size.height.saturating_sub(2));
    let x = (size.width.saturating_sub(width)) / 2;
    let y = (size.height.saturating_sub(height)) / 2;
    Rect {
        x,
        y,
        width,
        height,
    }
}

pub fn render_image_modal(f: &mut Frame, app: &mut App, size: Rect) {
    let Some(modal) = app.image_modal.clone() else {
        return;
    };

    let area = compute_image_modal_area(size);
    let icon = if app.config.ui.icons { "󰋩" } else { "" };

    let filename = modal
        .url
        .rsplit('/')
        .next()
        .unwrap_or("image")
        .split('?')
        .next()
        .unwrap_or("image");

    let title = modal
        .alt
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or(filename);

    let max_title_len = (area.width as usize).saturating_sub(12);
    let display_title = if title.chars().count() > max_title_len {
        let t: String = title.chars().take(max_title_len).collect();
        format!("{}…", t)
    } else {
        title.to_string()
    };

    let inner = render_modal_container_at(
        f,
        area,
        icon,
        &display_title,
        theme::PINK,
        app.config.ui.rounded_borders,
    );

    if inner.width < 4 || inner.height < 4 {
        return;
    }

    let has_caption = modal.caption.as_ref().is_some_and(|c| !c.trim().is_empty());
    let footer_h = if has_caption { 2u16 } else { 0u16 };
    let canvas_h = inner.height.saturating_sub(footer_h);
    let canvas_w = inner.width;

    let img_path = modal
        .path
        .clone()
        .or_else(|| crate::graphics::cache::get_cached_image_path(&modal.url));

    let resolved_proto = crate::graphics::resolve_protocol(app.config.reader.image_protocol);

    if let Some(path) = img_path {
        let dimensions = image::image_dimensions(&path).ok();
        let (orig_w, orig_h) = dimensions.unwrap_or((400, 300));

        let canvas_px_w = canvas_w as f32;
        let canvas_px_h = (canvas_h * 2) as f32;

        let scale = (canvas_px_w / (orig_w as f32)).min(canvas_px_h / (orig_h as f32));

        let display_cols = ((orig_w as f32 * scale).round() as u16)
            .max(2)
            .min(canvas_w);
        let display_rows = ((orig_h as f32 * scale / 2.0).round() as u16)
            .max(2)
            .min(canvas_h);

        let pad_top = (canvas_h.saturating_sub(display_rows)) / 2;

        if resolved_proto.is_kitty() {
            let screen_x = inner.x + (canvas_w.saturating_sub(display_cols)) / 2;
            let screen_y = inner.y + pad_top;
            app.graphics
                .pending_image_renders
                .push(crate::app::ImageRenderTask {
                    path,
                    screen_x,
                    screen_y,
                    cols: display_cols,
                    rows: display_rows,
                    crop_top_lines: 0,
                    crop_bot_lines: 0,
                });
        } else if let Ok(bytes) = std::fs::read(&path) {
            if let Some(hb_lines) = crate::graphics::halfblocks::render_halfblock_image_from_bytes(
                &bytes,
                display_cols as usize,
                display_rows as usize,
                app.config.reader.halfblock_filter,
            ) {
                let mut lines = Vec::with_capacity(canvas_h as usize);
                for _ in 0..pad_top {
                    lines.push(Line::from(""));
                }
                for line in hb_lines {
                    lines.push(line.alignment(Alignment::Center));
                }
                let p = Paragraph::new(lines).style(Style::default().bg(theme::BG));
                let canvas_rect = Rect::new(inner.x, inner.y, inner.width, canvas_h);
                f.render_widget(p, canvas_rect);
            }
        }
    } else {
        app.send_fetch_image(modal.url.clone());
        let loading_lines = vec![
            Line::from(""),
            Line::from(vec![Span::styled(
                "loading image...",
                Style::default().fg(theme::GREY).italic(),
            )])
            .alignment(Alignment::Center),
        ];
        let p = Paragraph::new(loading_lines).style(Style::default().bg(theme::BG));
        let canvas_rect = Rect::new(inner.x, inner.y, inner.width, canvas_h);
        f.render_widget(p, canvas_rect);
    }

    if has_caption && footer_h > 0 {
        if let Some(caption) = &modal.caption {
            let max_c = (inner.width as usize).saturating_sub(4);
            let c_display = if caption.chars().count() > max_c {
                let s: String = caption.chars().take(max_c).collect();
                format!("{}…", s)
            } else {
                caption.clone()
            };
            let footer_lines = vec![
                Line::from(""),
                Line::from(vec![Span::styled(
                    c_display,
                    Style::default().fg(theme::FG).italic(),
                )])
                .alignment(Alignment::Center),
            ];
            let footer_rect = Rect::new(
                inner.x,
                inner.y + canvas_h,
                inner.width,
                footer_h.min(inner.height.saturating_sub(canvas_h)),
            );
            let footer_p = Paragraph::new(footer_lines).style(Style::default().bg(theme::BG));
            f.render_widget(footer_p, footer_rect);
        }
    }
}
