use ratatui::style::Style;
use ratatui::text::Line;

use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Heading {
    pub title: String,
    pub level: u8,
    pub line_idx: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Link {
    pub title: String,
    pub text: String,
    pub span_indices: Vec<(usize, usize)>,
}

impl Link {
    pub fn is_external(&self) -> bool {
        self.title.starts_with("http://")
            || self.title.starts_with("https://")
            || self.title.starts_with("//")
    }

    pub fn is_citation(&self) -> bool {
        self.title.starts_with("#cite_note")
            || self.title.starts_with("#cite_ref")
            || self.title.starts_with("cite_note")
            || self.title.starts_with("cite_ref")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioTrack {
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpokenAudio {
    pub title: String,
    pub duration: Option<String>,
    pub tracks: Vec<AudioTrack>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ImageBlock {
    pub url: String,
    pub alt: Option<String>,
    pub caption: Option<String>,
    pub line_idx: usize,
    pub height_lines: usize,
    pub width_cols: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ParsedDocument {
    pub lines: Vec<Line<'static>>,
    pub plain_text_lower: Vec<String>,
    pub links: Vec<Link>,
    pub headings: Vec<Heading>,
    pub reference_targets: HashMap<String, usize>,
    pub spoken_audio: Option<SpokenAudio>,
    pub categories: Vec<String>,
    pub images: Vec<ImageBlock>,
}

impl ParsedDocument {
    #[inline]
    pub fn validate_invariants(&self) {
        #[cfg(debug_assertions)]
        {
            let mut last_first = (0, 0);
            for link in &self.links {
                if let Some(&(first_line, first_span)) = link.span_indices.first() {
                    debug_assert!(
                        (first_line, first_span) >= last_first,
                        "ParsedDocument link invariant violated: links must be monotonically sorted by (line, span) (found ({}, {}) < ({}, {}))",
                        first_line,
                        first_span,
                        last_first.0,
                        last_first.1
                    );
                    last_first = (first_line, first_span);
                }
                for &(line_idx, span_idx) in &link.span_indices {
                    if let Some(line) = self.lines.get(line_idx) {
                        debug_assert!(
                            span_idx < line.spans.len(),
                            "ParsedDocument link invariant violated: span index {} out of bounds for line {} (len {})",
                            span_idx,
                            line_idx,
                            line.spans.len()
                        );
                    }
                }
            }

            for heading in &self.headings {
                if !self.lines.is_empty() {
                    debug_assert!(
                        heading.line_idx < self.lines.len(),
                        "ParsedDocument heading invariant violated: heading line {} out of bounds (lines len {})",
                        heading.line_idx,
                        self.lines.len()
                    );
                }
            }
        }
    }

    pub fn ensure_plain_text_lower(&mut self) {
        if self.plain_text_lower.is_empty() && !self.lines.is_empty() {
            self.plain_text_lower = self
                .lines
                .iter()
                .map(|line| {
                    let cap = line.spans.iter().map(|s| s.content.len()).sum();
                    let mut full_text = String::with_capacity(cap);
                    for s in &line.spans {
                        full_text.push_str(s.content.as_ref());
                    }
                    full_text.to_lowercase()
                })
                .collect();
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct StyledToken {
    pub text: String,
    pub style: Style,
    pub link_target: Option<String>,
}

pub(crate) struct ParserContext<'a> {
    pub parser: &'a tl::Parser<'a>,
    pub max_width: usize,
    pub show_footnotes: bool,
    pub show_external_links: bool,
    pub heading_marker: bool,
    pub code_line_numbers: bool,
    pub show_icons: bool,
    pub show_images: bool,
    pub max_image_height: usize,
    pub skipping_external_section: bool,
    pub skipping_references_section: bool,
}
