use crate::parser::utils::decode_html_entities;

pub fn is_references_heading(title: &str) -> bool {
    let lower = title.to_lowercase();
    let trimmed = lower.trim();
    trimmed == "references"
        || trimmed == "notes"
        || trimmed == "notes and references"
        || trimmed == "footnotes"
        || trimmed == "citations"
        || trimmed == "reference"
        || trimmed == "footnote"
        || trimmed == "citation"
        || trimmed.starts_with("references ")
        || trimmed.starts_with("notes ")
        || trimmed.starts_with("footnotes ")
}

pub fn is_references_id(id_str: &str) -> bool {
    let lower = id_str.to_lowercase();
    lower == "references"
        || lower == "notes"
        || lower == "notes_and_references"
        || lower == "footnotes"
        || lower == "citations"
        || lower.starts_with("cite_note")
        || lower.starts_with("cite_ref")
        || lower == "mw-references-wrap"
}

pub fn heading_info<'a>(
    tag: &'a tl::HTMLTag<'a>,
    parser: &'a tl::Parser<'a>,
) -> Option<(u8, String, Option<String>)> {
    let tag_name = tag.name().as_utf8_str();
    let id_attr = tag
        .attributes()
        .get("id")
        .flatten()
        .map(|b| decode_html_entities(&b.as_utf8_str()).into_owned());

    if matches!(tag_name.as_ref(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
        let level = tag_name
            .chars()
            .nth(1)
            .and_then(|c| c.to_digit(10))
            .unwrap_or(1) as u8;
        let title = decode_html_entities(&tag.inner_text(parser))
            .trim()
            .to_string();
        return Some((level, title, id_attr));
    }

    if let Some(cls) = tag
        .attributes()
        .get("class")
        .flatten()
        .map(|b| b.as_utf8_str())
        && cls.contains("mw-heading") {
            for child_handle in tag.children().top().iter() {
                if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
                    let child_name = child_tag.name().as_utf8_str();
                    if matches!(child_name.as_ref(), "h1" | "h2" | "h3" | "h4" | "h5" | "h6") {
                        let level = child_name
                            .chars()
                            .nth(1)
                            .and_then(|c| c.to_digit(10))
                            .unwrap_or(1) as u8;
                        let title = decode_html_entities(&child_tag.inner_text(parser))
                            .trim()
                            .to_string();
                        let child_id = child_tag
                            .attributes()
                            .get("id")
                            .flatten()
                            .map(|b| decode_html_entities(&b.as_utf8_str()).into_owned())
                            .or(id_attr);
                        return Some((level, title, child_id));
                    }
                }
            }
        }

    None
}
