use crate::app::pane::PaneContent;
use crate::app::App;

impl App {
    pub fn scroll_down_lines(&mut self, lines: usize, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let step = lines.max(1);
            let pane = self.active_pane_mut();
            if matches!(&pane.content, PaneContent::ArticleText { .. }) {
                let max_scroll = pane.max_scroll(term_height);
                pane.scroll_offset = (pane.scroll_offset + step).min(max_scroll);
            }
            self.clamp_link_selection_to_viewport(term_height);
            self.maybe_mark_article_read();
        } else {
            let is_empty = matches!(self.active_pane().content, PaneContent::Empty);
            if is_empty {
                let recent = self.get_continue_reading_articles();
                if !recent.is_empty() {
                    let pane = self.active_pane_mut();
                    pane.selected_idx = (pane.selected_idx + lines).min(recent.len() - 1);
                }
            } else {
                let pane = self.active_pane_mut();
                match &pane.content {
                    PaneContent::SearchResults { items, .. } if !items.is_empty() => {
                        pane.selected_idx = (pane.selected_idx + lines).min(items.len() - 1);
                        Self::keep_search_selection_visible(pane, term_height);
                    }
                    _ => {}
                }
            }
        }
    }

    pub fn scroll_up_lines(&mut self, lines: usize, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let step = lines.max(1);
            let pane = self.active_pane_mut();
            pane.scroll_offset = pane.scroll_offset.saturating_sub(step);
            self.clamp_link_selection_to_viewport(term_height);
        } else {
            let pane = self.active_pane_mut();
            pane.selected_idx = pane.selected_idx.saturating_sub(lines);
            if matches!(pane.content, PaneContent::SearchResults { .. }) {
                Self::keep_search_selection_visible(pane, term_height);
            }
        }
    }

    pub fn select_next_item(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        let step = if is_article {
            self.user_data.config.reader.scroll_lines.max(1)
        } else {
            1
        };
        self.scroll_down_lines(step, term_height);
    }

    pub fn select_prev_item(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        let step = if is_article {
            self.user_data.config.reader.scroll_lines.max(1)
        } else {
            1
        };
        self.scroll_up_lines(step, term_height);
    }

    pub fn scroll_page_down(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        let pane = self.active_pane_mut();
        let step = pane.page_scroll_step(term_height);

        if is_article {
            let max_scroll = pane.max_scroll(term_height);
            pane.scroll_offset = (pane.scroll_offset + step).min(max_scroll);
            self.clamp_link_selection_to_viewport(term_height);
            self.maybe_mark_article_read();
        } else {
            match &pane.content {
                PaneContent::SearchResults { items, .. } if !items.is_empty() => {
                    pane.selected_idx = (pane.selected_idx + step).min(items.len() - 1);
                    Self::keep_search_selection_visible(pane, term_height);
                }
                _ => {}
            }
        }
    }

    pub fn scroll_page_up(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        let pane = self.active_pane_mut();
        let step = pane.page_scroll_step(term_height);

        if is_article {
            pane.scroll_offset = pane.scroll_offset.saturating_sub(step);
            self.clamp_link_selection_to_viewport(term_height);
        } else if let PaneContent::SearchResults { .. } = &pane.content {
            pane.selected_idx = pane.selected_idx.saturating_sub(step);
            Self::keep_search_selection_visible(pane, term_height);
        }
    }

    pub fn jump_to_top(&mut self) {
        let pane = self.active_pane_mut();
        pane.scroll_offset = 0;
        pane.selected_idx = 0;
        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            pane.selected_link_idx = if !parsed_doc.links.is_empty() {
                Some(0)
            } else {
                None
            };
        }
    }

    pub fn jump_to_bottom(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let pane = self.active_pane_mut();
            pane.scroll_offset = pane.max_scroll(term_height);
            self.clamp_link_selection_to_viewport(term_height);
            self.maybe_mark_article_read();
        } else {
            let pane = self.active_pane_mut();
            match &pane.content {
                PaneContent::SearchResults { items, .. } if !items.is_empty() => {
                    pane.selected_idx = items.len() - 1;
                    Self::keep_search_selection_visible(pane, term_height);
                }
                _ => {}
            }
        }
    }

    pub fn jump_next_heading(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let pane = self.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                let next_h = parsed_doc
                    .headings
                    .iter()
                    .find(|h| h.line_idx > pane.scroll_offset);
                if let Some(next_h) = next_h {
                    pane.scroll_offset = next_h.line_idx;
                }
            }
            self.clamp_link_selection_to_viewport(term_height);
            self.maybe_mark_article_read();
        }
    }

    pub fn jump_prev_heading(&mut self, term_height: u16) {
        let is_article = matches!(self.active_pane().content, PaneContent::ArticleText { .. });
        if is_article {
            let pane = self.active_pane_mut();
            if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                let prev_h = parsed_doc
                    .headings
                    .iter()
                    .rfind(|h| h.line_idx < pane.scroll_offset);
                if let Some(prev_h) = prev_h {
                    pane.scroll_offset = prev_h.line_idx;
                }
            }
            self.clamp_link_selection_to_viewport(term_height);
        }
    }
}
