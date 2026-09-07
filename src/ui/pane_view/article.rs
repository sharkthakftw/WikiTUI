use super::scrollbar::render_scroll_indicator;
use crate::app::App;
use crate::theme;
use crate::ui::modals::render_toc_modal;
use ratatui::{
    layout::Rect,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

#[allow(clippy::too_many_arguments)]
pub fn render_article_pane(
    f: &mut Frame,
    app: &mut App,
    tab_idx: usize,
    pane_idx: usize,
    rect: Rect,
    block: Block,
    border_color: ratatui::style::Color,
    is_active: bool,
) {
    let pane = &app.tabs[tab_idx].panes[pane_idx];
    let crate::app::PaneContent::ArticleText { parsed_doc, .. } = &pane.content else {
        return;
    };

    let view_start = pane.scroll_offset.min(parsed_doc.lines.len());
    let view_len =
        (pane.viewport_height + 2).min(parsed_doc.lines.len().saturating_sub(view_start));
    let view_end = view_start + view_len;

    let resolved_proto = crate::graphics::resolve_protocol(app.config.reader.image_protocol);
    if resolved_proto.is_halfblocks() && app.config.reader.show_images {
        let pane = &mut app.tabs[tab_idx].panes[pane_idx];
        if let crate::app::PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            let mut to_request = Vec::new();
            for img in &parsed_doc.images {
                if img.line_idx + img.height_lines > view_start && img.line_idx < view_end {
                    let cols = img.width_cols;
                    let rows = img.height_lines;
                    if !pane.contains_halfblock(&img.url, cols, rows)
                        && !pane.is_pending_decode(&img.url, cols, rows)
                    {
                        pane.pending_image_decodes.insert((img.url.clone(), cols, rows));
                        if let Some(path) = pane
                            .loaded_images
                            .get(&img.url)
                            .cloned()
                            .or_else(|| crate::graphics::cache::get_cached_image_path(&img.url))
                        {
                            to_request.push((img.url.clone(), path, cols, rows));
                        }
                    }
                }
            }
            for (url, path, cols, rows) in to_request {
                app.send_decode_halfblock_image(url, path, cols, rows);
            }
        }
    }

    let pane = &app.tabs[tab_idx].panes[pane_idx];
    let crate::app::PaneContent::ArticleText { parsed_doc, .. } = &pane.content else {
        return;
    };

    let (has_underline, first_link_idx) = if app.config.reader.underline_links {
        let first_idx = parsed_doc.links.partition_point(|link| {
            link.span_indices
                .last()
                .map(|&(l, _)| l < view_start)
                .unwrap_or(true)
        });
        let has = parsed_doc.links[first_idx..]
            .iter()
            .take_while(|l| {
                l.span_indices
                    .first()
                    .is_some_and(|&(first_line, _)| first_line < view_end)
            })
            .any(|l| !l.is_citation());
        (has, first_idx)
    } else {
        (false, 0)
    };

    let selected_link = pane
        .selected_link_idx
        .and_then(|idx| parsed_doc.links.get(idx));

    let has_search_matches =
        if !pane.search.matches.is_empty() && !pane.search.query.trim().is_empty() {
            let first_match_idx = pane
                .search
                .matches
                .partition_point(|m| m.line_idx < view_start);
            first_match_idx < pane.search.matches.len()
                && pane.search.matches[first_match_idx].line_idx < view_end
        } else {
            false
        };

    let mut rendered_lines: Vec<Line<'_>> = Vec::with_capacity(view_len);

    let mut link_ptr = first_link_idx;
    let query_len = pane.search.query.to_lowercase().len();
    let mut match_ptr = if has_search_matches {
        pane.search
            .matches
            .partition_point(|m| m.line_idx < view_start)
    } else {
        0
    };
    let selected_match = pane
        .search
        .selected_match_idx
        .and_then(|idx| pane.search.matches.get(idx));

    for (local_idx, orig_line) in parsed_doc.lines[view_start..view_end].iter().enumerate() {
        let line_idx = view_start + local_idx;

        let mut image_override = None;
        if app.config.reader.show_images {
            for img in &parsed_doc.images {
                if line_idx >= img.line_idx && line_idx < img.line_idx + img.height_lines {
                    let has_img = pane.loaded_images.contains_key(&img.url)
                        || crate::graphics::cache::get_cached_image_path(&img.url).is_some();
                    if resolved_proto.is_halfblocks() {
                        let rel_row = line_idx - img.line_idx;
                        let cols = img.width_cols;
                        let rows = img.height_lines;
                        if let Some(hb_lines) = pane.get_halfblock(&img.url, cols, rows) {
                            if let Some(hb_line) = hb_lines.get(rel_row) {
                                image_override = Some(hb_line.clone());
                            }
                        }
                    } else if resolved_proto.is_kitty() && has_img {
                        image_override = Some(Line::from(""));
                    }
                    break;
                }
            }
        }

        if let Some(line) = image_override {
            rendered_lines.push(line);
            continue;
        }

        let needs_underline = has_underline && link_ptr < parsed_doc.links.len();
        let needs_selected_link = selected_link
            .is_some_and(|l| l.span_indices.iter().any(|(l_idx, _)| *l_idx == line_idx));
        let needs_search = has_search_matches
            && match_ptr < pane.search.matches.len()
            && pane.search.matches[match_ptr].line_idx == line_idx;
        let needs_selection = pane
            .selection
            .text_selection
            .as_ref()
            .is_some_and(|s| s.contains_line(line_idx));

        if !needs_underline && !needs_selected_link && !needs_search && !needs_selection {
            rendered_lines.push(orig_line.clone());
            continue;
        }

        let mut spans: Vec<Span<'_>> = orig_line
            .spans
            .iter()
            .map(|s| Span::styled(s.content.as_ref(), s.style))
            .collect();

        if has_underline {
            let mut curr_ptr = link_ptr;
            while curr_ptr < parsed_doc.links.len() {
                let link = &parsed_doc.links[curr_ptr];
                let Some(&(first_line, _)) = link.span_indices.first() else {
                    curr_ptr += 1;
                    continue;
                };
                let last_line = link
                    .span_indices
                    .last()
                    .map(|&(l, _)| l)
                    .unwrap_or(first_line);
                if first_line > line_idx {
                    break;
                }
                if last_line < line_idx {
                    curr_ptr += 1;
                    link_ptr = curr_ptr;
                    continue;
                }
                if !link.is_citation() {
                    for &(l_idx, span_idx) in &link.span_indices {
                        if l_idx == line_idx {
                            if let Some(span) = spans.get_mut(span_idx) {
                                span.style = span.style.add_modifier(Modifier::UNDERLINED);
                            }
                        }
                    }
                }
                if last_line == line_idx {
                    curr_ptr += 1;
                    link_ptr = curr_ptr;
                } else {
                    curr_ptr += 1;
                }
            }
        }

        if let Some(link) = selected_link {
            for &(l_idx, span_idx) in &link.span_indices {
                if l_idx == line_idx {
                    if let Some(span) = spans.get_mut(span_idx) {
                        span.style = Style::default()
                            .fg(theme::VIOLET)
                            .bold()
                            .add_modifier(Modifier::UNDERLINED);
                    }
                }
            }
        }

        if has_search_matches {
            let mut line_matches = Vec::new();
            while match_ptr < pane.search.matches.len()
                && pane.search.matches[match_ptr].line_idx == line_idx
            {
                let m = &pane.search.matches[match_ptr];
                let is_active = selected_match
                    .is_some_and(|sm| sm.line_idx == m.line_idx && sm.char_offset == m.char_offset);
                line_matches.push((m.char_offset, m.char_offset + query_len, is_active));
                match_ptr += 1;
            }

            if !line_matches.is_empty() {
                spans = build_search_highlighted_spans(&spans, &line_matches);
            }
        }

        if let Some(selection) = &pane.selection.text_selection {
            if selection.contains_line(line_idx) {
                let (start, end) = selection.normalized();
                let line_len: usize = spans.iter().map(|s| s.content.chars().count()).sum();
                let from = if line_idx == start.0 {
                    start.1.min(line_len)
                } else {
                    0
                };
                let to = if line_idx == end.0 {
                    end.1.min(line_len)
                } else {
                    line_len
                };
                if from < to {
                    spans = build_selection_highlighted_spans(&spans, from, to);
                }
            }
        }

        let mut line = Line::from(spans);
        line.alignment = orig_line.alignment;
        rendered_lines.push(line);
    }

    let should_dim =
        (app.config.ui.dim_inactive_panes && !is_active && app.tabs[tab_idx].panes.len() > 1)
            || app.input_mode == crate::app::InputMode::ImageModal;
    if should_dim {
        for line in &mut rendered_lines {
            for span in &mut line.spans {
                span.style = span.style.add_modifier(Modifier::DIM);
            }
        }
    }

    let inner_rect = block.inner(rect);
    let paragraph = Paragraph::new(rendered_lines).block(block);
    f.render_widget(paragraph, rect);

    if app.config.reader.show_images {
        for img in &parsed_doc.images {
            let img_top = img.line_idx;
            let img_bot = img.line_idx + img.height_lines;
            if img_bot > view_start && img_top < view_end {
                let img_path = pane
                    .loaded_images
                    .get(&img.url)
                    .cloned()
                    .or_else(|| crate::graphics::cache::get_cached_image_path(&img.url));

                if let Some(path) = img_path {
                    if resolved_proto.is_kitty() {
                        if let Some(bounds) = calculate_visible_image_bounds(
                            img.line_idx,
                            img.height_lines,
                            img.width_cols,
                            view_start,
                            inner_rect,
                        ) {
                            app.graphics
                                .pending_image_renders
                                .push(crate::app::ImageRenderTask {
                                    path,
                                    screen_x: bounds.screen_x,
                                    screen_y: bounds.screen_y,
                                    cols: bounds.visible_cols,
                                    rows: bounds.visible_rows,
                                    crop_top_lines: bounds.top_clipped,
                                    crop_bot_lines: bounds.bot_clipped,
                                });
                        }
                    }
                } else {
                    app.send_fetch_image(img.url.clone());
                }
            }
        }
    }

    render_scroll_indicator(
        f,
        rect,
        parsed_doc.lines.len(),
        pane.viewport_height,
        pane.scroll_offset,
        border_color,
        is_active,
        app.zen_mode,
        app.config.ui.scroll_indicator,
    );

    if is_active && pane.show_toc && !parsed_doc.headings.is_empty() {
        render_toc_modal(
            f,
            pane,
            parsed_doc,
            rect,
            app.config.reader.toc_section_numbers,
            app.config.ui.rounded_borders,
            app.config.ui.icons,
        );
    }
}

pub fn clamp_to_char_boundary(s: &str, mut idx: usize) -> usize {
    if idx >= s.len() {
        return s.len();
    }
    while idx > 0 && !s.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

fn build_search_highlighted_spans<'a>(
    spans: &[Span<'a>],
    line_matches: &[(usize, usize, bool)],
) -> Vec<Span<'a>> {
    if line_matches.is_empty() {
        return spans.to_vec();
    }

    let mut new_spans = Vec::with_capacity(spans.len() + line_matches.len() * 2);
    let mut global_offset = 0;

    for span in spans {
        let text = span.content.as_ref();
        let span_len = text.len();
        let span_start = global_offset;
        let span_end = span_start + span_len;

        let mut text_cursor = 0;

        for &(m_start, m_end, is_active) in line_matches {
            if m_end <= span_start || m_start >= span_end {
                continue;
            }

            let raw_rel_start = m_start.saturating_sub(span_start).max(text_cursor);
            let raw_rel_end = (m_end.saturating_sub(span_start)).min(span_len);

            let rel_match_start = clamp_to_char_boundary(text, raw_rel_start);
            let rel_match_end = clamp_to_char_boundary(text, raw_rel_end);
            text_cursor = clamp_to_char_boundary(text, text_cursor);

            if rel_match_start > text_cursor && rel_match_start <= span_len {
                let unmatch_span = match &span.content {
                    std::borrow::Cow::Borrowed(s) => {
                        Span::styled(&s[text_cursor..rel_match_start], span.style)
                    }
                    std::borrow::Cow::Owned(s) => {
                        Span::styled(s[text_cursor..rel_match_start].to_string(), span.style)
                    }
                };
                new_spans.push(unmatch_span);
                text_cursor = rel_match_start;
            }

            if rel_match_end > text_cursor && rel_match_end <= span_len {
                let bg_color = if is_active {
                    theme::YELLOW
                } else {
                    theme::BEIGE
                };
                let match_style = Style::default().bg(bg_color).fg(theme::BG).bold();
                let match_span = match &span.content {
                    std::borrow::Cow::Borrowed(s) => {
                        Span::styled(&s[text_cursor..rel_match_end], match_style)
                    }
                    std::borrow::Cow::Owned(s) => {
                        Span::styled(s[text_cursor..rel_match_end].to_string(), match_style)
                    }
                };
                new_spans.push(match_span);
                text_cursor = rel_match_end;
            }
        }

        if text_cursor < span_len {
            let trailing_span = match &span.content {
                std::borrow::Cow::Borrowed(s) => Span::styled(&s[text_cursor..], span.style),
                std::borrow::Cow::Owned(s) => {
                    Span::styled(s[text_cursor..].to_string(), span.style)
                }
            };
            new_spans.push(trailing_span);
        }

        global_offset = span_end;
    }

    new_spans
}

fn build_selection_highlighted_spans<'a>(
    spans: &[Span<'a>],
    sel_start: usize,
    sel_end: usize,
) -> Vec<Span<'a>> {
    let mut new_spans = Vec::new();
    let mut global_offset = 0;

    for span in spans {
        let span_len = span.content.chars().count();
        let span_start = global_offset;
        let span_end = span_start + span_len;

        if sel_end <= span_start || sel_start >= span_end {
            new_spans.push(span.clone());
        } else {
            let rel_start = sel_start.saturating_sub(span_start).min(span_len);
            let rel_end = sel_end.saturating_sub(span_start).min(span_len);

            let mut byte_start = span.content.len();
            let mut byte_end = span.content.len();
            for (char_idx, (b_idx, _)) in span.content.char_indices().enumerate() {
                if char_idx == rel_start {
                    byte_start = b_idx;
                }
                if char_idx == rel_end {
                    byte_end = b_idx;
                    break;
                }
            }

            if rel_start > 0 {
                let prefix_span = match &span.content {
                    std::borrow::Cow::Borrowed(s) => Span::styled(&s[..byte_start], span.style),
                    std::borrow::Cow::Owned(s) => {
                        Span::styled(s[..byte_start].to_string(), span.style)
                    }
                };
                new_spans.push(prefix_span);
            }

            if rel_end > rel_start {
                let sel_style = Style::default().bg(theme::PINK).fg(theme::BG).bold();
                let sel_span = match &span.content {
                    std::borrow::Cow::Borrowed(s) => {
                        Span::styled(&s[byte_start..byte_end], sel_style)
                    }
                    std::borrow::Cow::Owned(s) => {
                        Span::styled(s[byte_start..byte_end].to_string(), sel_style)
                    }
                };
                new_spans.push(sel_span);
            }

            if rel_end < span_len {
                let suffix_span = match &span.content {
                    std::borrow::Cow::Borrowed(s) => Span::styled(&s[byte_end..], span.style),
                    std::borrow::Cow::Owned(s) => {
                        Span::styled(s[byte_end..].to_string(), span.style)
                    }
                };
                new_spans.push(suffix_span);
            }
        }

        global_offset = span_end;
    }

    new_spans
}

pub fn get_link_at_coord(
    parsed_doc: &crate::parser::ParsedDocument,
    scroll_offset: usize,
    pane_rect: Rect,
    col: u16,
    row: u16,
) -> Option<usize> {
    if pane_rect.width < 3 || pane_rect.height < 3 {
        return None;
    }
    let inner_x = pane_rect.x + 1;
    let inner_y = pane_rect.y + 1;
    let inner_w = pane_rect.width.saturating_sub(2);
    let inner_h = pane_rect.height.saturating_sub(2);

    if col < inner_x || col >= inner_x + inner_w || row < inner_y || row >= inner_y + inner_h {
        return None;
    }

    let row_in_pane = (row - inner_y) as usize;
    let line_idx = scroll_offset + row_in_pane;
    let line = parsed_doc.lines.get(line_idx)?;

    let target_x = (col - inner_x) as usize;
    let mut cur_x = 0;
    let mut target_span_idx = None;

    for (span_idx, span) in line.spans.iter().enumerate() {
        let span_w = unicode_width::UnicodeWidthStr::width(span.content.as_ref());
        if target_x >= cur_x && target_x < cur_x + span_w {
            target_span_idx = Some(span_idx);
            break;
        }
        cur_x += span_w;
    }

    let span_idx = target_span_idx?;

    parsed_doc
        .links
        .iter()
        .position(|link| link.span_indices.contains(&(line_idx, span_idx)))
}

pub fn get_image_at_coord(
    parsed_doc: &crate::parser::ParsedDocument,
    scroll_offset: usize,
    pane_rect: Rect,
    col: u16,
    row: u16,
) -> Option<crate::parser::ImageBlock> {
    if pane_rect.width < 3 || pane_rect.height < 3 {
        return None;
    }
    let inner_x = pane_rect.x + 1;
    let inner_y = pane_rect.y + 1;
    let inner_w = pane_rect.width.saturating_sub(2);
    let inner_h = pane_rect.height.saturating_sub(2);

    if col < inner_x || col >= inner_x + inner_w || row < inner_y || row >= inner_y + inner_h {
        return None;
    }

    let row_in_pane = (row - inner_y) as usize;
    let line_idx = scroll_offset + row_in_pane;

    parsed_doc
        .images
        .iter()
        .find(|img| line_idx >= img.line_idx && line_idx < img.line_idx + img.height_lines)
        .cloned()
}

pub struct VisibleImageBounds {
    pub screen_x: u16,
    pub screen_y: u16,
    pub visible_cols: u16,
    pub visible_rows: u16,
    pub top_clipped: u16,
    pub bot_clipped: u16,
}

pub fn calculate_visible_image_bounds(
    img_top: usize,
    img_height: usize,
    img_width: usize,
    view_start: usize,
    inner_rect: Rect,
) -> Option<VisibleImageBounds> {
    let (screen_y, visible_rows, top_clipped, bot_clipped) = if img_top < view_start {
        let top_clipped = view_start - img_top;
        let rows = img_height.saturating_sub(top_clipped);
        let visible = (rows as u16).min(inner_rect.height);
        let bot_clipped = rows.saturating_sub(visible as usize);
        (
            inner_rect.y,
            visible,
            top_clipped as u16,
            bot_clipped as u16,
        )
    } else {
        let rel_line = img_top - view_start;
        let max_rows = inner_rect.height.saturating_sub(rel_line as u16);
        let visible = (img_height as u16).min(max_rows);
        let bot_clipped = img_height.saturating_sub(visible as usize);
        (
            inner_rect.y + (rel_line as u16),
            visible,
            0,
            bot_clipped as u16,
        )
    };

    let visible_cols = (img_width as u16).min(inner_rect.width);
    let left_pad = inner_rect.width.saturating_sub(img_width as u16) / 2;
    let screen_x = inner_rect.x + left_pad;

    if visible_rows > 0 && visible_cols > 0 {
        Some(VisibleImageBounds {
            screen_x,
            screen_y,
            visible_cols,
            visible_rows,
            top_clipped,
            bot_clipped,
        })
    } else {
        None
    }
}
