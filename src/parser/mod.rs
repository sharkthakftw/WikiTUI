pub mod banners;
pub mod blocks;
pub mod categories;
pub mod codeblocks;
pub mod elements;
pub mod images;
pub(crate) mod node;
pub(crate) mod sections;
pub mod spoken;
pub mod tables;
pub mod types;
pub mod utils;

pub use banners::{ArticleBanner, BannerType};
pub use types::{AudioTrack, Heading, ImageBlock, Link, ParsedDocument, SpokenAudio};
pub use utils::url_decode;

use blocks::wrap_and_append_block;
use node::process_node;
use ratatui::style::Style;
use types::StyledToken;

#[allow(clippy::too_many_arguments)]
pub fn parse_wikipedia_html(
    html: &str,
    max_width: usize,
    show_footnotes: bool,
    show_external_links: bool,
    heading_marker: bool,
    code_line_numbers: bool,
    show_icons: bool,
    show_images: bool,
    max_image_height: usize,
) -> ParsedDocument {
    let mut doc = ParsedDocument::default();
    let effective_width = max_width.max(10);

    let Ok(dom) = tl::parse(html, tl::ParserOptions::default()) else {
        return doc;
    };
    let parser = dom.parser();
    let mut ctx = types::ParserContext {
        parser,
        max_width: effective_width,
        show_footnotes,
        show_external_links,
        heading_marker,
        code_line_numbers,
        show_icons,
        show_images,
        max_image_height,
        skipping_external_section: false,
        skipping_references_section: false,
    };
    let mut current_block_tokens: Vec<StyledToken> = Vec::new();

    for handle in dom.children() {
        if let Some(node) = handle.get(parser) {
            process_node(
                node,
                &mut ctx,
                Style::default().fg(crate::theme::FG),
                None,
                &mut current_block_tokens,
                &mut doc,
                None,
                false,
                false,
                0,
            );
        }
    }

    if !current_block_tokens.is_empty() {
        wrap_and_append_block(&current_block_tokens, &mut doc, effective_width);
    }

    doc.links
        .sort_by_key(|l| l.span_indices.first().copied().unwrap_or((0, 0)));
    doc.validate_invariants();

    doc
}
