use super::types::{Link, ParsedDocument, StyledToken};
use ratatui::text::{Line, Span};

pub(crate) fn wrap_and_append_block(
    tokens: &[StyledToken],
    doc: &mut ParsedDocument,
    max_width: usize,
) {
    let mut current_line_spans: Vec<Span<'static>> = Vec::new();
    let mut current_line_len = 0;

    let mut active_link_target: Option<String> = None;
    let mut active_link_text = String::new();
    let mut active_link_spans: Vec<(usize, usize)> = Vec::new();
    let mut last_span_link_target: Option<String> = None;

    for token in tokens {
        if token.link_target != active_link_target {
            if let Some(target) = active_link_target.take() {
                if !active_link_spans.is_empty() {
                    doc.links.push(Link {
                        title: target,
                        text: active_link_text.trim().to_string(),
                        span_indices: std::mem::take(&mut active_link_spans),
                    });
                }
                active_link_text.clear();
            }
            active_link_target = token.link_target.clone();
        }

        let words = token.text.split_inclusive(|c: char| c.is_whitespace());

        for word in words {
            if word.is_empty() {
                continue;
            }

            let word_len = unicode_width::UnicodeWidthStr::width(word);

            if current_line_len + word_len > max_width && current_line_len > 0 {
                doc.lines
                    .push(Line::from(std::mem::take(&mut current_line_spans)));
                current_line_len = 0;
                last_span_link_target = None;
            }

            let current_line_idx = doc.lines.len();
            if token.link_target.is_some() {
                active_link_text.push_str(word);
            }

            let can_merge = !current_line_spans.is_empty()
                && last_span_link_target.as_deref() == token.link_target.as_deref()
                && current_line_spans.last().map(|s| s.style) == Some(token.style);

            if can_merge {
                let last_span = current_line_spans.last_mut().unwrap();
                match &mut last_span.content {
                    std::borrow::Cow::Owned(s) => s.push_str(word),
                    std::borrow::Cow::Borrowed(_) => {
                        last_span.content.to_mut().push_str(word);
                    }
                }
            } else {
                if token.link_target.is_some() {
                    active_link_spans.push((current_line_idx, current_line_spans.len()));
                }
                current_line_spans.push(Span::styled(word.to_string(), token.style));
                last_span_link_target = token.link_target.clone();
            }

            current_line_len += word_len;
        }
    }

    if let Some(target) = active_link_target
        && !active_link_spans.is_empty() {
            doc.links.push(Link {
                title: target,
                text: active_link_text.trim().to_string(),
                span_indices: active_link_spans,
            });
        }

    if !current_line_spans.is_empty() {
        doc.lines.push(Line::from(current_line_spans));
        doc.lines.push(Line::from(""));
    }
}
