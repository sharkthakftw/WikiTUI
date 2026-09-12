use super::types::CellLinkInfo;
use crate::parser::types::StyledToken;
use ratatui::text::Span;
use unicode_width::UnicodeWidthStr;

pub(crate) fn wrap_cell_tokens(
    tokens: &[StyledToken],
    max_width: usize,
) -> (Vec<Vec<Span<'static>>>, Vec<CellLinkInfo>) {
    let max_width = max_width.max(1);
    let mut lines: Vec<Vec<Span<'static>>> = Vec::new();
    let mut current_line: Vec<Span<'static>> = Vec::new();
    let mut current_line_len = 0;

    let mut links: Vec<CellLinkInfo> = Vec::new();
    let mut active_link_target: Option<String> = None;
    let mut active_link_text = String::new();
    let mut active_link_spans: Vec<(usize, usize)> = Vec::new();

    for token in tokens {
        if token.text == "\n" {
            if let Some(target) = active_link_target.take()
                && !active_link_spans.is_empty() {
                    links.push((
                        target,
                        std::mem::take(&mut active_link_text).trim().to_string(),
                        std::mem::take(&mut active_link_spans),
                    ));
                }
            lines.push(std::mem::take(&mut current_line));
            current_line_len = 0;
            continue;
        }

        if token.link_target != active_link_target {
            if let Some(target) = active_link_target.take()
                && !active_link_spans.is_empty() {
                    links.push((
                        target,
                        std::mem::take(&mut active_link_text).trim().to_string(),
                        std::mem::take(&mut active_link_spans),
                    ));
                }
            active_link_target = token.link_target.clone();
        }

        let words = token.text.split_inclusive(|c: char| c.is_whitespace());
        for word in words {
            if word.is_empty() {
                continue;
            }

            let mut remaining_word = word;
            while !remaining_word.is_empty() {
                let word_len = UnicodeWidthStr::width(remaining_word);

                if current_line_len + word_len <= max_width {
                    let current_line_idx = lines.len();
                    if token.link_target.is_some() {
                        active_link_spans.push((current_line_idx, current_line.len()));
                        active_link_text.push_str(remaining_word);
                    }
                    current_line.push(Span::styled(remaining_word.to_string(), token.style));
                    current_line_len += word_len;
                    break;
                }

                if current_line_len > 0 {
                    lines.push(std::mem::take(&mut current_line));
                    current_line_len = 0;
                    continue;
                }

                let mut take_chars = 0;
                let mut take_w = 0;
                for ch in remaining_word.chars() {
                    let ch_w = unicode_width::UnicodeWidthChar::width(ch).unwrap_or(0);
                    if take_w + ch_w > max_width && take_chars > 0 {
                        break;
                    }
                    take_w += ch_w;
                    take_chars += ch.len_utf8();
                }

                let chunk = &remaining_word[..take_chars];
                remaining_word = &remaining_word[take_chars..];

                let current_line_idx = lines.len();
                if token.link_target.is_some() {
                    active_link_spans.push((current_line_idx, current_line.len()));
                    active_link_text.push_str(chunk);
                }
                current_line.push(Span::styled(chunk.to_string(), token.style));
                current_line_len += take_w;

                if !remaining_word.is_empty() {
                    lines.push(std::mem::take(&mut current_line));
                    current_line_len = 0;
                }
            }
        }
    }

    if let Some(target) = active_link_target
        && !active_link_spans.is_empty() {
            links.push((
                target,
                active_link_text.trim().to_string(),
                active_link_spans,
            ));
        }

    if !current_line.is_empty() || lines.is_empty() {
        lines.push(current_line);
    }

    (lines, links)
}
