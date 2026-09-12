use crate::app::{App, InputMode, PaneContent, TextSelection};
use ratatui::layout::Rect;

pub fn get_char_coord_in_article_pane(
    pane: &crate::app::Pane,
    rect: Rect,
    col: u16,
    row: u16,
) -> Option<(usize, usize)> {
    let PaneContent::ArticleText { parsed_doc, .. } = &pane.content else {
        return None;
    };

    if col < rect.x || col >= rect.x + rect.width || row < rect.y || row >= rect.y + rect.height {
        return None;
    }

    let inner_y = rect.y + 1;
    let inner_x = rect.x + 2;

    let line_offset = if row < inner_y {
        0
    } else {
        (row - inner_y) as usize
    };

    let line_idx = pane.scroll_offset + line_offset;
    let line_idx = line_idx.min(parsed_doc.lines.len().saturating_sub(1));

    let char_col = if col < inner_x {
        0
    } else {
        (col - inner_x) as usize
    };

    Some((line_idx, char_col))
}

pub fn handle_selection_down(
    app: &mut App,
    col: u16,
    row: u16,
    term_width: u16,
    term_height: u16,
) -> bool {
    if app.input_mode != InputMode::Normal {
        return false;
    }

    if row == 0 || row >= term_height.saturating_sub(1) {
        return false;
    }

    let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
    let tab = app.active_tab_mut();
    let rects = tab.layout_root.compute_rects(main_rect);

    for (pane_idx, rect) in rects {
        if col >= rect.x && col < rect.x + rect.width && row >= rect.y && row < rect.y + rect.height
        {
            tab.active_pane_idx = pane_idx;
            let pane = &mut tab.panes[pane_idx];
            if matches!(pane.content, PaneContent::ArticleText { .. })
                && let Some(coord) = get_char_coord_in_article_pane(pane, rect, col, row) {
                    pane.selection.text_selection = None;
                    pane.selection.selection_anchor = Some(coord);
                    pane.selection.is_mouse_selecting = true;
                    return true;
                }
        }
    }
    false
}

pub fn handle_selection_drag(
    app: &mut App,
    col: u16,
    row: u16,
    term_width: u16,
    term_height: u16,
) -> bool {
    if app.input_mode != InputMode::Normal {
        return false;
    }

    let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
    let tab = app.active_tab_mut();
    let rects = tab.layout_root.compute_rects(main_rect);

    if let Some(&(_, rect)) = rects.iter().find(|(idx, _)| *idx == tab.active_pane_idx) {
        let pane = &mut tab.panes[tab.active_pane_idx];
        if pane.selection.is_mouse_selecting
            && let Some(anchor) = pane.selection.selection_anchor
                && let Some(coord) = get_char_coord_in_article_pane(pane, rect, col, row) {
                    pane.selection.text_selection = Some(TextSelection {
                        start: anchor,
                        end: coord,
                    });
                    return true;
                }
    }
    false
}

pub fn handle_selection_up(app: &mut App) {
    let tab = app.active_tab_mut();
    let pane = &mut tab.panes[tab.active_pane_idx];
    if pane.selection.is_mouse_selecting {
        pane.selection.is_mouse_selecting = false;
        if let Some(selection) = pane.selection.text_selection {
            let (start, end) = selection.normalized();
            if start != end
                && let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                    let text = extract_selected_text(parsed_doc, &selection);
                    if !text.trim().is_empty() {
                        let count = text.chars().count();
                        if crate::clipboard::copy_to_clipboard(&text) {
                            app.set_status_message(format!(
                                "copied {} characters to clipboard",
                                count
                            ));
                        }
                    }
                }
        }
    }
}

pub fn extract_selected_text(
    doc: &crate::parser::ParsedDocument,
    selection: &TextSelection,
) -> String {
    let ((start_line, start_col), (end_line, end_col)) = selection.normalized();
    let mut lines_out = Vec::new();

    let citation_spans: std::collections::HashSet<(usize, usize)> = doc
        .links
        .iter()
        .filter(|l| l.is_citation())
        .flat_map(|l| l.span_indices.iter().copied())
        .collect();

    for line_idx in start_line..=end_line.min(doc.lines.len().saturating_sub(1)) {
        if let Some(line) = doc.lines.get(line_idx) {
            let mut line_buf = String::new();
            let mut global_char_pos = 0;

            let line_from = if line_idx == start_line { start_col } else { 0 };
            let line_to = if line_idx == end_line {
                end_col
            } else {
                usize::MAX
            };

            for (span_idx, span) in line.spans.iter().enumerate() {
                let span_len = span.content.chars().count();
                let span_start = global_char_pos;
                let span_end = span_start + span_len;
                global_char_pos = span_end;

                if citation_spans.contains(&(line_idx, span_idx)) {
                    continue;
                }

                if span_end <= line_from || span_start >= line_to {
                    continue;
                }

                let rel_from = line_from.saturating_sub(span_start).min(span_len);
                let rel_to = (line_to.saturating_sub(span_start)).min(span_len);

                if rel_from < rel_to {
                    let chars: String = span
                        .content
                        .chars()
                        .skip(rel_from)
                        .take(rel_to - rel_from)
                        .collect();
                    line_buf.push_str(&chars);
                }
            }

            lines_out.push(line_buf);
        }
    }

    let joined = lines_out.join("\n");
    strip_citations(&joined)
}

fn strip_citations(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '[' {
            let mut bracket_content = String::new();
            let mut found_close = false;

            while let Some(&next_c) = chars.peek() {
                if next_c == ']' {
                    chars.next();
                    found_close = true;
                    break;
                } else if next_c == '\n' {
                    break;
                } else {
                    bracket_content.push(chars.next().unwrap());
                    if bracket_content.len() > 30 {
                        break;
                    }
                }
            }

            if found_close {
                let trimmed = bracket_content.trim();
                let is_citation = trimmed.chars().all(|ch| ch.is_ascii_digit())
                    || trimmed.starts_with("note ")
                    || trimmed.starts_with("nb ")
                    || trimmed == "citation needed"
                    || (trimmed.len() <= 3 && trimmed.chars().all(|ch| ch.is_ascii_alphabetic()));

                if !is_citation {
                    out.push('[');
                    out.push_str(&bracket_content);
                    out.push(']');
                }
            } else {
                out.push('[');
                out.push_str(&bracket_content);
            }
        } else {
            out.push(c);
        }
    }

    out
}
