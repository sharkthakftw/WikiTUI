use crate::api::SearchResultItem;
use crate::parser::{parse_wikipedia_html, ParsedDocument};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ArticleRenderOptions {
    pub width: usize,
    pub show_footnotes: bool,
    pub show_external_links: bool,
    pub heading_marker: bool,
    pub code_line_numbers: bool,
    pub show_icons: bool,
    pub show_images: bool,
    pub max_image_height: usize,
}

#[derive(Clone, Debug)]
pub enum PaneContent {
    Empty,
    SearchResults {
        query: String,
        items: Vec<SearchResultItem>,
    },
    ArticleText {
        title: String,
        raw_html: String,
        parsed_doc: Box<ParsedDocument>,
        last_render_options: ArticleRenderOptions,
    },
    Error(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LocalMatch {
    pub line_idx: usize,
    pub span_idx: usize,
    pub char_offset: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextSelection {
    pub start: (usize, usize),
    pub end: (usize, usize),
}

impl TextSelection {
    pub fn normalized(&self) -> ((usize, usize), (usize, usize)) {
        if self.start <= self.end {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    pub fn contains_line(&self, line_idx: usize) -> bool {
        let (start, end) = self.normalized();
        line_idx >= start.0 && line_idx <= end.0
    }
}

#[derive(Clone, Debug, Default)]
pub struct PaneSearchState {
    pub query: String,
    pub matches: Vec<LocalMatch>,
    pub selected_match_idx: Option<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct PaneSelectionState {
    pub text_selection: Option<TextSelection>,
    pub is_mouse_selecting: bool,
    pub selection_anchor: Option<(usize, usize)>,
}

#[derive(Clone, Debug)]
pub struct Pane {
    pub id: usize,
    pub content: PaneContent,
    pub selected_idx: usize,
    pub scroll_offset: usize,
    pub viewport_width: usize,
    pub viewport_height: usize,
    pub selected_link_idx: Option<usize>,
    pub search: PaneSearchState,
    pub selection: PaneSelectionState,
    pub is_loading: bool,
    pub loading_title: Option<String>,
    pub show_toc: bool,
    pub selected_toc_idx: Option<usize>,
    pub toc_focused: bool,
    pub loaded_images: std::collections::HashMap<String, std::path::PathBuf>,
    pub halfblock_cache:
        std::collections::HashMap<String, Vec<(usize, usize, Vec<ratatui::text::Line<'static>>)>>,
    pub pending_image_decodes: std::collections::HashSet<(String, usize, usize)>,

    pub history_back: Vec<String>,
    pub history_forward: Vec<String>,
    pub intra_jump_back: Vec<usize>,
    pub intra_jump_forward: Vec<usize>,
    pub current_request_id: u64,
    pub opened_at: Option<std::time::Instant>,
    pub has_marked_read: bool,
}

impl Pane {
    pub fn get_halfblock(&self, url: &str, cols: usize, rows: usize) -> Option<&[ratatui::text::Line<'static>]> {
        self.halfblock_cache
            .get(url)?
            .iter()
            .find(|(c, r, _)| *c == cols && *r == rows)
            .map(|(_, _, lines)| lines.as_slice())
    }

    pub fn contains_halfblock(&self, url: &str, cols: usize, rows: usize) -> bool {
        self.halfblock_cache
            .get(url)
            .is_some_and(|entries| entries.iter().any(|(c, r, _)| *c == cols && *r == rows))
    }

    pub fn insert_halfblock(
        &mut self,
        url: String,
        cols: usize,
        rows: usize,
        lines: Vec<ratatui::text::Line<'static>>,
    ) {
        let entries = self.halfblock_cache.entry(url).or_default();
        if let Some(existing) = entries.iter_mut().find(|(c, r, _)| *c == cols && *r == rows) {
            existing.2 = lines;
        } else {
            entries.push((cols, rows, lines));
        }
    }

    pub fn is_pending_decode(&self, url: &str, cols: usize, rows: usize) -> bool {
        self.pending_image_decodes
            .iter()
            .any(|(u, c, r)| *c == cols && *r == rows && u == url)
    }

    pub fn new(id: usize) -> Self {
        Self {
            id,
            content: PaneContent::Empty,
            selected_idx: 0,
            scroll_offset: 0,
            viewport_width: 0,
            viewport_height: 0,
            selected_link_idx: None,
            search: PaneSearchState::default(),
            selection: PaneSelectionState::default(),
            is_loading: false,
            loading_title: None,
            show_toc: false,
            selected_toc_idx: None,
            toc_focused: false,
            loaded_images: std::collections::HashMap::new(),
            halfblock_cache: std::collections::HashMap::new(),
            pending_image_decodes: std::collections::HashSet::new(),

            history_back: Vec::new(),
            history_forward: Vec::new(),
            intra_jump_back: Vec::new(),
            intra_jump_forward: Vec::new(),
            current_request_id: 0,
            opened_at: None,
            has_marked_read: false,
        }
    }

    pub fn prepare_for_article_fetch(&mut self, title: &str) {
        self.is_loading = true;
        self.loading_title = Some(title.to_string());
        self.selected_link_idx = None;
        self.selection = PaneSelectionState::default();
        self.intra_jump_back.clear();
        self.intra_jump_forward.clear();
        self.scroll_offset = 0;
        self.show_toc = false;
        self.selected_toc_idx = None;
        self.halfblock_cache.clear();
    }

    pub fn selected_target(&self, recent_articles: &[String]) -> Option<String> {
        match &self.content {
            PaneContent::SearchResults { items, .. } => {
                items.get(self.selected_idx).map(|item| item.title.clone())
            }
            PaneContent::Empty => recent_articles.get(self.selected_idx).cloned(),
            PaneContent::ArticleText { parsed_doc, .. } => self
                .selected_link_idx
                .and_then(|idx| parsed_doc.links.get(idx))
                .map(|link| link.title.clone()),
            _ => None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn ensure_parsed_width(&mut self, opts: ArticleRenderOptions) {
        if let PaneContent::ArticleText {
            raw_html,
            parsed_doc,
            last_render_options,
            ..
        } = &mut self.content
        {
            if *last_render_options == opts {
                return;
            }
            **parsed_doc = parse_wikipedia_html(
                raw_html,
                opts.width,
                opts.show_footnotes,
                opts.show_external_links,
                opts.heading_marker,
                opts.code_line_numbers,
                opts.show_icons,
                opts.show_images,
                opts.max_image_height,
            );
            *last_render_options = opts;
            if let Some(idx) = self.selected_link_idx {
                if idx >= parsed_doc.links.len() {
                    self.selected_link_idx = if parsed_doc.links.is_empty() {
                        None
                    } else {
                        Some(parsed_doc.links.len() - 1)
                    };
                }
            }
            self.recompute_local_matches();
        }
    }

    pub fn recompute_local_matches(&mut self) {
        self.search.matches.clear();
        let query = self.search.query.to_lowercase();
        if query.trim().is_empty() {
            self.search.selected_match_idx = None;
            return;
        }

        if let PaneContent::ArticleText { parsed_doc, .. } = &self.content {
            for (line_idx, line) in parsed_doc.lines.iter().enumerate() {
                if let Some(full_lower) = parsed_doc.plain_text_lower.get(line_idx) {
                    for (match_pos, _) in full_lower.match_indices(&query) {
                        let mut current_offset = 0;
                        let mut start_span_idx = 0;
                        for (idx, span) in line.spans.iter().enumerate() {
                            let span_len = span.content.len();
                            if current_offset + span_len > match_pos {
                                start_span_idx = idx;
                                break;
                            }
                            current_offset += span_len;
                        }
                        self.search.matches.push(LocalMatch {
                            line_idx,
                            span_idx: start_span_idx,
                            char_offset: match_pos,
                        });
                    }
                }
            }
            if !self.search.matches.is_empty() {
                if let Some(sel) = self.search.selected_match_idx {
                    self.search.selected_match_idx = Some(sel.min(self.search.matches.len() - 1));
                } else {
                    self.search.selected_match_idx = Some(0);
                }
            } else {
                self.search.selected_match_idx = None;
            }
        }
    }

    pub fn title(&self) -> Option<String> {
        match &self.content {
            PaneContent::ArticleText { title, .. } => Some(title.clone()),
            _ => None,
        }
    }

    pub fn focused_link(&self) -> Option<&crate::parser::types::Link> {
        if let PaneContent::ArticleText { parsed_doc, .. } = &self.content {
            self.selected_link_idx
                .and_then(|idx| parsed_doc.links.get(idx))
        } else {
            None
        }
    }

    pub fn effective_viewport_height(&self, term_height: u16) -> usize {
        if self.viewport_height > 0 {
            self.viewport_height
        } else {
            (term_height as usize).saturating_sub(4).max(1)
        }
    }

    pub fn page_scroll_step(&self, term_height: u16) -> usize {
        (self.effective_viewport_height(term_height) * 3 / 4).max(1)
    }

    pub fn max_scroll(&self, term_height: u16) -> usize {
        let viewport = self.effective_viewport_height(term_height);
        match &self.content {
            PaneContent::ArticleText { parsed_doc, .. } => {
                parsed_doc.lines.len().saturating_sub(viewport)
            }
            _ => 0,
        }
    }
}
