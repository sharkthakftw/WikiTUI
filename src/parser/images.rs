use crate::parser::types::{ImageBlock, ParsedDocument, ParserContext};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use tl::{HTMLTag, Parser};

pub(crate) fn render_image_node(
    tag: &HTMLTag,
    parser: &Parser,
    doc: &mut ParsedDocument,
    ctx: &ParserContext,
) {
    if !ctx.show_images {
        return;
    }

    let (src, alt, width_px, height_px) = extract_image_attributes(tag, parser);
    let Some(url) = normalize_image_url(&src) else {
        return;
    };

    let caption = extract_caption(tag, parser).or(alt.clone());

    let max_cols = ctx.max_width.saturating_sub(4).max(10);
    let max_rows = ctx.max_image_height.max(5);

    let (cols, rows) = calculate_terminal_dimensions(width_px, height_px, max_cols, max_rows);

    doc.lines.push(Line::from(""));
    let line_idx = doc.lines.len();

    let image_block = ImageBlock {
        url,
        alt,
        caption: caption.clone(),
        line_idx,
        height_lines: rows,
        width_cols: cols,
    };

    let inner_width = cols.saturating_sub(2);
    for r in 0..rows {
        let content = if r == 0 {
            format!("┌{}┐", "─".repeat(inner_width))
        } else if r == rows - 1 {
            format!("└{}┘", "─".repeat(inner_width))
        } else if r == rows / 2 {
            let label = " 🖼 [image] ";
            if inner_width >= label.chars().count() {
                let pad = inner_width - label.chars().count();
                let left_pad = pad / 2;
                let right_pad = pad - left_pad;
                format!(
                    "│{}{}{}│",
                    " ".repeat(left_pad),
                    label,
                    " ".repeat(right_pad)
                )
            } else {
                format!("│{}│", " ".repeat(inner_width))
            }
        } else {
            format!("│{}│", " ".repeat(inner_width))
        };
        let mut line = Line::from(vec![Span::styled(
            content,
            Style::default().fg(crate::theme::GREY),
        )]);
        line.alignment = Some(ratatui::layout::Alignment::Center);
        doc.lines.push(line);
    }

    if let Some(cap) = caption
        && !cap.trim().is_empty() {
            doc.lines.push(Line::from(""));
            let cap_line = cap.trim().to_string();
            let mut line = Line::from(vec![Span::styled(
                cap_line,
                Style::default()
                    .fg(crate::theme::GREY)
                    .add_modifier(Modifier::ITALIC),
            )]);
            line.alignment = Some(ratatui::layout::Alignment::Center);
            doc.lines.push(line);
        }
    doc.lines.push(Line::from(""));

    doc.images.push(image_block);
}

fn normalize_image_url(src: &str) -> Option<String> {
    let clean = src.trim();
    if clean.is_empty() {
        return None;
    }

    if clean.contains("/static/images/")
        || clean.contains("red_pencile.svg")
        || clean.contains("Padlock")
        || clean.contains("Symbol_")
        || clean.contains("Ambox_")
        || clean.contains("Question_book")
        || clean.to_lowercase().contains("logo")
        || clean.ends_with(".svg")
    {
        return None;
    }

    if clean.starts_with("//") {
        Some(format!("https:{}", clean))
    } else if clean.starts_with("http://") || clean.starts_with("https://") {
        Some(clean.to_string())
    } else if clean.starts_with('/') {
        Some(format!("https://en.wikipedia.org{}", clean))
    } else {
        None
    }
}

fn find_first_img<'a>(tag: &'a HTMLTag<'a>, parser: &'a Parser<'a>) -> Option<&'a HTMLTag<'a>> {
    if tag.name().as_utf8_str() == "img" {
        return Some(tag);
    }
    for child_handle in tag.children().top().iter() {
        if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser)
            && let Some(found) = find_first_img(child_tag, parser) {
                return Some(found);
            }
    }
    None
}

fn extract_image_attributes(
    tag: &HTMLTag,
    parser: &Parser,
) -> (String, Option<String>, Option<usize>, Option<usize>) {
    if let Some(cls) = tag
        .attributes()
        .get("class")
        .flatten()
        .map(|b| b.as_utf8_str())
        && (cls.contains("mwe-math") || cls.contains("math-fallback") || cls.contains("noviewer")) {
            return (String::new(), None, None, None);
        }

    if let Some(img_tag) = find_first_img(tag, parser) {
        return parse_img_tag(img_tag);
    }

    (String::new(), None, None, None)
}

fn parse_img_tag(tag: &HTMLTag) -> (String, Option<String>, Option<usize>, Option<usize>) {
    if let Some(cls) = tag
        .attributes()
        .get("class")
        .flatten()
        .map(|b| b.as_utf8_str())
        && (cls.contains("mwe-math") || cls.contains("math-fallback") || cls.contains("noviewer")) {
            return (String::new(), None, None, None);
        }

    let raw_src = tag
        .attributes()
        .get("srcset")
        .flatten()
        .and_then(|b| {
            let s = b.as_utf8_str();
            s.split(',')
                .filter_map(|cand| cand.split_whitespace().next())
                .next_back()
                .map(|u| u.to_string())
        })
        .or_else(|| {
            tag.attributes()
                .get("src")
                .flatten()
                .map(|b| b.as_utf8_str().to_string())
        })
        .or_else(|| {
            tag.attributes()
                .get("data-src")
                .flatten()
                .map(|b| b.as_utf8_str().to_string())
        })
        .unwrap_or_default();

    let src = crate::parser::utils::decode_html_entities(&raw_src).into_owned();

    let alt = tag
        .attributes()
        .get("alt")
        .flatten()
        .map(|b| crate::parser::utils::decode_html_entities(&b.as_utf8_str()).into_owned())
        .filter(|s| !s.trim().is_empty());

    let width = tag
        .attributes()
        .get("width")
        .flatten()
        .and_then(|b| b.as_utf8_str().parse::<usize>().ok());

    let height = tag
        .attributes()
        .get("height")
        .flatten()
        .and_then(|b| b.as_utf8_str().parse::<usize>().ok());

    if let (Some(w), Some(h)) = (width, height)
        && (w < 40 || h < 40) {
            return (String::new(), None, None, None);
        }

    (src, alt, width, height)
}

fn extract_caption(tag: &HTMLTag, parser: &Parser) -> Option<String> {
    for child_handle in tag.children().top().iter() {
        if let Some(tl::Node::Tag(cap_tag)) = child_handle.get(parser) {
            let name = cap_tag.name().as_utf8_str();
            let cls = cap_tag
                .attributes()
                .get("class")
                .flatten()
                .map(|b| b.as_utf8_str().to_string())
                .unwrap_or_default();
            if name == "figcaption" || cls.contains("thumbcaption") || cls.contains("gallerytext") {
                let text = cap_tag.inner_text(parser).trim().to_string();
                if !text.is_empty() {
                    return Some(crate::parser::utils::decode_html_entities(&text).into_owned());
                }
            }
            if let Some(sub) = extract_caption(cap_tag, parser) {
                return Some(sub);
            }
        }
    }
    None
}

fn calculate_terminal_dimensions(
    w_px: Option<usize>,
    h_px: Option<usize>,
    max_cols: usize,
    max_rows: usize,
) -> (usize, usize) {
    if let (Some(w), Some(h)) = (w_px, h_px)
        && w > 0 && h > 0 {
            let term_aspect = (w as f64) / (h as f64) * 2.0;
            let mut cols = (max_rows as f64 * term_aspect).round() as usize;
            cols = cols.clamp(10, max_cols);
            let rows = ((cols as f64) / term_aspect).round() as usize;
            let rows = rows.clamp(3, max_rows);
            return (cols, rows);
        }
    (max_cols.min(40), max_rows.min(15))
}
