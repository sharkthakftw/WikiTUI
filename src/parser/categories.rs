use super::utils::decode_html_entities;

pub fn is_category_links_tag(tag: &tl::HTMLTag, _parser: &tl::Parser) -> bool {
    let id_match = tag
        .attributes()
        .get("id")
        .flatten()
        .map(|b| b.as_utf8_str().to_lowercase())
        .is_some_and(|id| id == "catlinks" || id == "mw-normal-catlinks");

    if id_match {
        return true;
    }

    

    tag
        .attributes()
        .get("class")
        .flatten()
        .map(|b| b.as_utf8_str().to_lowercase())
        .is_some_and(|cls| {
            cls.split_whitespace()
                .any(|c| c == "catlinks" || c == "mw-normal-catlinks")
        })
}

pub fn extract_categories_from_tag(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    categories: &mut Vec<String>,
) {
    collect_categories_recursive(tag, parser, categories);
}

fn collect_categories_recursive(
    tag: &tl::HTMLTag,
    parser: &tl::Parser,
    categories: &mut Vec<String>,
) {
    // Avoid hidden categories (maintenance tags)
    if let Some(id) = tag.attributes().get("id").flatten()
        && id.as_utf8_str().to_lowercase() == "mw-hidden-catlinks" {
            return;
        }
    if let Some(cls) = tag.attributes().get("class").flatten()
        && cls.as_utf8_str().contains("mw-hidden-catlinks") {
            return;
        }

    let name = tag.name().as_utf8_str();
    if name == "a"
        && let Some(href_attr) = tag.attributes().get("href").flatten() {
            let href = href_attr.as_utf8_str();
            if href.contains("/wiki/Category:") || href.starts_with("Category:") {
                let binding = tag.inner_text(parser);
                let text = decode_html_entities(&binding);
                let trimmed = text.trim();
                if !trimmed.is_empty()
                    && trimmed != "Categories"
                    && trimmed != "Category"
                    && !categories.iter().any(|c| c.eq_ignore_ascii_case(trimmed))
                {
                    categories.push(trimmed.to_string());
                }
            }
        }

    for child_handle in tag.children().top().iter() {
        if let Some(tl::Node::Tag(child_tag)) = child_handle.get(parser) {
            collect_categories_recursive(child_tag, parser, categories);
        }
    }
}
