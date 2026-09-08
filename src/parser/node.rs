use super::banners;
use super::blocks::wrap_and_append_block;
use super::categories;
use super::codeblocks;
use super::elements;
use super::sections::{heading_info, is_references_heading, is_references_id};
use super::spoken;
use super::tables;
use super::types::{ParsedDocument, ParserContext, StyledToken};
use super::utils::{
    decode_html_entities, extract_title_from_href, to_subscript_str, to_superscript_str,
};
use crate::theme;
use ratatui::style::{Modifier, Style};

fn get_decoded_attr(tag: &tl::HTMLTag, key: &str) -> Option<String> {
    tag.attributes().get(key).flatten().map(|b| {
        let s = b.as_utf8_str();
        if s.contains('&') {
            decode_html_entities(&s).into_owned()
        } else {
            s.into_owned()
        }
    })
}

const MAX_DOM_DEPTH: usize = 128;

fn is_ignored_tag(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "style" | "script" | "noscript" | "head" | "template" | "link" | "meta" | "annotation"
    )
}

#[allow(clippy::too_many_arguments)]
fn process_raw_text(
    bytes: &tl::Bytes,
    ctx: &ParserContext,
    parent_style: Style,
    parent_link: Option<String>,
    current_tokens: &mut Vec<StyledToken>,
    is_sup: bool,
    is_sub: bool,
) {
    if ctx.skipping_external_section || ctx.skipping_references_section {
        return;
    }
    let raw_text = bytes.as_utf8_str();
    let decoded_text = decode_html_entities(&raw_text);
    let transformed = if is_sup {
        to_superscript_str(&decoded_text)
    } else if is_sub {
        to_subscript_str(&decoded_text)
    } else {
        decoded_text.into_owned()
    };
    let cleaned_text = transformed.replace(['\n', '\r', '\t'], " ");
    if !cleaned_text.trim().is_empty() {
        current_tokens.push(StyledToken {
            text: cleaned_text,
            style: parent_style,
            link_target: parent_link,
        });
    }
}

fn handle_section_and_skip_tag<'a>(
    tag: &'a tl::HTMLTag<'a>,
    class_attr: Option<&str>,
    id_attr: Option<&str>,
    ctx: &mut ParserContext<'a>,
    doc: &mut ParsedDocument,
) -> bool {
    if spoken::is_spoken_wikipedia_tag(tag, ctx.parser) {
        if let Some(spoken_audio) = spoken::extract_spoken_audio(tag, ctx.parser) {
            if doc.spoken_audio.is_none() {
                doc.spoken_audio = Some(spoken_audio);
            }
        }
        return true;
    }

    if categories::is_category_links_tag(tag, ctx.parser) {
        categories::extract_categories_from_tag(tag, ctx.parser, &mut doc.categories);
        return true;
    }

    if let Some((level, title, id_opt)) = heading_info(tag, ctx.parser) {
        let lower_title = title.to_lowercase();
        let lower_id = id_opt.as_deref().unwrap_or("").to_lowercase();

        let is_ext = !ctx.show_external_links
            && (lower_title.starts_with("external link")
                || lower_title.starts_with("external_link")
                || lower_id == "external_links"
                || lower_id == "external-links"
                || lower_id == "externallinks");

        let is_refs =
            !ctx.show_footnotes && (is_references_heading(&title) || is_references_id(&lower_id));

        if is_ext {
            ctx.skipping_external_section = true;
            ctx.skipping_references_section = false;
            return true;
        }
        if is_refs {
            ctx.skipping_references_section = true;
            ctx.skipping_external_section = false;
            return true;
        }

        if level <= 2 {
            ctx.skipping_external_section = false;
            ctx.skipping_references_section = false;
        } else if ctx.skipping_external_section || ctx.skipping_references_section {
            return true;
        }
    } else if ctx.skipping_external_section || ctx.skipping_references_section {
        return true;
    }

    if !ctx.show_external_links {
        if let Some(id_str) = id_attr {
            if id_str.eq_ignore_ascii_case("external_links")
                || id_str.eq_ignore_ascii_case("external-links")
                || id_str.eq_ignore_ascii_case("externallinks")
            {
                ctx.skipping_external_section = true;
                return true;
            }
        }
    }

    if !ctx.show_footnotes {
        if let Some(id_str) = id_attr {
            if is_references_id(id_str) {
                if !id_str.starts_with("cite_note") && !id_str.starts_with("cite_ref") {
                    ctx.skipping_references_section = true;
                }
                return true;
            }
        }
    }

    if let Some(class_str) = class_attr {
        if class_str.split_whitespace().any(|cls| {
            matches!(
                cls,
                "sidebar"
                    | "navbox"
                    | "mw-empty-elt"
                    | "noprint"
                    | "hatnote"
                    | "mw-jump-link"
                    | "catlinks"
                    | "vector-menu"
                    | "asbox"
                    | "stub"
                    | "cite-accessibility-label"
                    | "visually-hidden"
                    | "sr-only"
            ) || (!ctx.show_footnotes
                && matches!(
                    cls,
                    "reference"
                        | "reflist"
                        | "references"
                        | "mw-references-wrap"
                        | "references-wrap"
                ))
                || (!ctx.show_images
                    && matches!(
                        cls,
                        "thumbcaption" | "gallerytext" | "caption" | "thumbcaption-inline"
                    ))
        }) {
            return true;
        }
    }

    if !ctx.show_images && tag.name().as_utf8_str() == "figcaption" {
        return true;
    }

    if !ctx.show_footnotes {
        if let Some(id_str) = id_attr {
            if id_str.starts_with("cite_note")
                || id_str.starts_with("cite_ref")
                || id_str == "mw-references-wrap"
            {
                return true;
            }
        }
    }

    false
}

fn handle_banner_tag<'a>(
    tag: &'a tl::HTMLTag<'a>,
    class_attr: Option<&str>,
    current_tokens: &mut Vec<StyledToken>,
    doc: &mut ParsedDocument,
    ctx: &ParserContext<'a>,
) -> bool {
    if let Some(class_str) = class_attr {
        if let Some(banner_type) = banners::classify_ambox_class(class_str) {
            banners::render_ambox_banner(
                tag,
                ctx.parser,
                current_tokens,
                doc,
                ctx.max_width,
                banner_type,
            );
            return true;
        }
    }
    false
}

fn handle_media_tag<'a>(
    tag: &'a tl::HTMLTag<'a>,
    tag_name: &str,
    class_attr: Option<&str>,
    current_tokens: &mut Vec<StyledToken>,
    doc: &mut ParsedDocument,
    ctx: &mut ParserContext<'a>,
) -> bool {
    if let Some(class_str) = class_attr {
        if class_str.split_whitespace().any(|cls| cls == "infobox") {
            if ctx.show_images {
                if !current_tokens.is_empty() {
                    wrap_and_append_block(current_tokens, doc, ctx.max_width);
                    current_tokens.clear();
                }
                crate::parser::images::render_image_node(tag, ctx.parser, doc, ctx);
            }
            return true;
        }
    }

    if tag_name == "figure"
        || tag_name == "figure-inline"
        || class_attr
            .map(|c| c.contains("thumb") || c.contains("gallerybox"))
            .unwrap_or(false)
    {
        if ctx.show_images {
            if !current_tokens.is_empty() {
                wrap_and_append_block(current_tokens, doc, ctx.max_width);
                current_tokens.clear();
            }
            crate::parser::images::render_image_node(tag, ctx.parser, doc, ctx);
        }
        return true;
    }

    if !ctx.show_images
        && (tag_name == "figcaption"
            || class_attr
                .map(|c| c.contains("thumbcaption") || c.contains("gallerytext"))
                .unwrap_or(false))
    {
        return true;
    }

    if tag_name == "img" {
        if ctx.show_images && !current_tokens.is_empty() {
            wrap_and_append_block(current_tokens, doc, ctx.max_width);
            current_tokens.clear();
        }
        if ctx.show_images {
            crate::parser::images::render_image_node(tag, ctx.parser, doc, ctx);
        }
        return true;
    }

    false
}

fn handle_table_or_code_tag<'a>(
    tag: &'a tl::HTMLTag<'a>,
    tag_name: &str,
    class_attr: Option<&str>,
    current_tokens: &mut Vec<StyledToken>,
    doc: &mut ParsedDocument,
    ctx: &ParserContext<'a>,
) -> bool {
    if tag_name == "table" {
        if !current_tokens.is_empty() {
            wrap_and_append_block(current_tokens, doc, ctx.max_width);
            current_tokens.clear();
        }
        tables::render_table(
            tag,
            ctx.parser,
            doc,
            ctx.max_width,
            ctx.show_footnotes,
            ctx.show_icons,
        );
        return true;
    }

    if tag_name == "pre" {
        if !current_tokens.is_empty() {
            wrap_and_append_block(current_tokens, doc, ctx.max_width);
            current_tokens.clear();
        }
        let lang = codeblocks::extract_language(tag);
        codeblocks::render_code_block(
            tag,
            ctx.parser,
            doc,
            ctx.max_width,
            lang,
            ctx.code_line_numbers,
        );
        return true;
    }

    if let Some(class_str) = class_attr {
        if class_str.contains("mw-highlight") {
            if !current_tokens.is_empty() {
                wrap_and_append_block(current_tokens, doc, ctx.max_width);
                current_tokens.clear();
            }
            let lang = codeblocks::extract_language(tag);
            for child_handle in tag.children().top().iter() {
                if let Some(tl::Node::Tag(pre_tag)) = child_handle.get(ctx.parser) {
                    if pre_tag.name().as_utf8_str() == "pre" {
                        codeblocks::render_code_block(
                            pre_tag,
                            ctx.parser,
                            doc,
                            ctx.max_width,
                            lang,
                            ctx.code_line_numbers,
                        );
                        return true;
                    }
                }
            }
        }
    }

    false
}

#[allow(clippy::too_many_arguments)]
fn handle_text_and_style_tag<'a>(
    tag: &'a tl::HTMLTag<'a>,
    tag_name: &str,
    class_attr: Option<&str>,
    parent_style: Style,
    parent_link: Option<String>,
    current_tokens: &mut Vec<StyledToken>,
    doc: &mut ParsedDocument,
    list_item_idx: Option<usize>,
    ctx: &ParserContext<'a>,
) -> (Style, Option<String>) {
    let mut current_style = parent_style;
    let mut current_link = parent_link;

    match tag_name {
        "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
            current_style = elements::handle_heading_tag(
                tag,
                ctx.parser,
                tag_name,
                doc,
                current_tokens,
                ctx.heading_marker,
                current_style,
            );
        }
        "b" | "strong" => {
            current_style = current_style.add_modifier(Modifier::BOLD);
        }
        "i" | "em" => {
            current_style = current_style.add_modifier(Modifier::ITALIC);
        }
        "code" | "kbd" | "samp" | "tt" => {
            current_style = current_style.fg(theme::ORANGE);
        }
        "abbr" => {
            current_style = current_style.add_modifier(Modifier::UNDERLINED);
        }
        "a" => {
            current_style = current_style.fg(theme::BLUE);
            if let Some(href) = tag
                .attributes()
                .get("href")
                .flatten()
                .map(|b| b.as_utf8_str())
            {
                if let Some(title) = extract_title_from_href(href.as_ref()) {
                    let is_external = title.starts_with("http://") || title.starts_with("https://");
                    if is_external {
                        current_style = current_style.fg(theme::TEAL);
                    }
                    current_link = Some(title);
                }
            } else if let Some(title_attr) = tag
                .attributes()
                .get("title")
                .flatten()
                .map(|b| decode_html_entities(&b.as_utf8_str()).into_owned())
            {
                current_link = Some(title_attr);
            }
        }
        "li" => {
            let prefix = if let Some(idx) = list_item_idx {
                format!("{}. ", idx)
            } else {
                "• ".to_string()
            };
            current_tokens.push(StyledToken {
                text: prefix,
                style: Style::default().fg(theme::GREY),
                link_target: None,
            });
        }
        "img" => {
            if let Some(token) = elements::handle_img_tag(tag, class_attr, &current_link) {
                current_tokens.push(token);
            }
        }
        _ => {}
    }

    (current_style, current_link)
}

#[allow(clippy::too_many_arguments)]
fn process_tag_node<'a>(
    tag: &'a tl::HTMLTag<'a>,
    ctx: &mut ParserContext<'a>,
    parent_style: Style,
    parent_link: Option<String>,
    current_tokens: &mut Vec<StyledToken>,
    doc: &mut ParsedDocument,
    list_item_idx: Option<usize>,
    is_sup: bool,
    is_sub: bool,
    depth: usize,
) {
    let tag_name_cow = tag.name().as_utf8_str();
    let tag_name = tag_name_cow.as_ref();

    if is_ignored_tag(tag_name) {
        return;
    }

    let class_attr = get_decoded_attr(tag, "class");
    let id_attr = get_decoded_attr(tag, "id");

    if handle_section_and_skip_tag(tag, class_attr.as_deref(), id_attr.as_deref(), ctx, doc) {
        return;
    }

    if handle_banner_tag(tag, class_attr.as_deref(), current_tokens, doc, ctx) {
        return;
    }

    if handle_media_tag(
        tag,
        tag_name,
        class_attr.as_deref(),
        current_tokens,
        doc,
        ctx,
    ) {
        return;
    }

    if handle_table_or_code_tag(
        tag,
        tag_name,
        class_attr.as_deref(),
        current_tokens,
        doc,
        ctx,
    ) {
        return;
    }

    let is_block_element = matches!(
        tag_name,
        "p" | "h1"
            | "h2"
            | "h3"
            | "h4"
            | "h5"
            | "h6"
            | "li"
            | "ul"
            | "ol"
            | "div"
            | "section"
            | "blockquote"
    );

    if is_block_element && !current_tokens.is_empty() {
        wrap_and_append_block(current_tokens, doc, ctx.max_width);
        current_tokens.clear();
    }

    if let Some(ref id_str) = id_attr {
        if id_str.starts_with("cite_note") || id_str.starts_with("cite_ref") {
            doc.reference_targets
                .entry(id_str.clone())
                .or_insert_with(|| doc.lines.len());
        }
    }

    let (current_style, current_link) = handle_text_and_style_tag(
        tag,
        tag_name,
        class_attr.as_deref(),
        parent_style,
        parent_link,
        current_tokens,
        doc,
        list_item_idx,
        ctx,
    );

    let is_ordered_list = tag_name == "ol";
    let mut item_counter = 1;
    let current_is_sup = is_sup || tag_name == "sup";
    let current_is_sub = is_sub || tag_name == "sub";

    for child_handle in tag.children().top().iter() {
        if let Some(child_node) = child_handle.get(ctx.parser) {
            let child_list_idx = if is_ordered_list {
                if let tl::Node::Tag(child_tag) = child_node {
                    if child_tag.name().as_utf8_str() == "li" {
                        let idx = item_counter;
                        item_counter += 1;
                        Some(idx)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            process_node(
                child_node,
                ctx,
                current_style,
                current_link.clone(),
                current_tokens,
                doc,
                child_list_idx,
                current_is_sup,
                current_is_sub,
                depth + 1,
            );
        }
    }

    if tag_name == "a" {
        if let Some(ref target) = current_link {
            current_tokens.extend(elements::handle_external_link_pill(target, &current_link));
        }
    }

    if is_block_element && !current_tokens.is_empty() {
        wrap_and_append_block(current_tokens, doc, ctx.max_width);
        current_tokens.clear();
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn process_node<'a>(
    node: &'a tl::Node<'a>,
    ctx: &mut ParserContext<'a>,
    parent_style: Style,
    parent_link: Option<String>,
    current_tokens: &mut Vec<StyledToken>,
    doc: &mut ParsedDocument,
    list_item_idx: Option<usize>,
    is_sup: bool,
    is_sub: bool,
    depth: usize,
) {
    if depth > MAX_DOM_DEPTH {
        return;
    }
    match node {
        tl::Node::Raw(bytes) => {
            process_raw_text(
                bytes,
                ctx,
                parent_style,
                parent_link,
                current_tokens,
                is_sup,
                is_sub,
            );
        }
        tl::Node::Tag(tag) => {
            process_tag_node(
                tag,
                ctx,
                parent_style,
                parent_link,
                current_tokens,
                doc,
                list_item_idx,
                is_sup,
                is_sub,
                depth,
            );
        }
        tl::Node::Comment(_) => {}
    }
}
