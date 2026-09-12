use crate::api::{NetworkCommand, SearchResultItem};
use crate::app::pane::{Pane, PaneContent};
use crate::app::App;

impl App {
    pub fn enter_search_mode(&mut self) {
        self.input_mode = crate::app::InputMode::Search;
        self.modals.search_modal.opens_new_tab = true;
        self.modals.search_modal.input.clear();
        self.modals.search_modal.cursor_pos = 0;
    }

    pub fn edit_search_mode(&mut self) {
        self.input_mode = crate::app::InputMode::Search;
        self.modals.search_modal.opens_new_tab = false;
        let existing_query = match &self.active_pane().content {
            PaneContent::SearchResults { query, .. } => Some(query.clone()),
            _ => None,
        };
        if let Some(query) = existing_query {
            self.modals.search_modal.input = query;
        } else {
            self.modals.search_modal.input.clear();
        }
        self.modals.search_modal.cursor_pos = self.modals.search_modal.input.chars().count();
    }

    pub fn exit_search_mode(&mut self) {
        self.input_mode = crate::app::InputMode::Normal;
        self.modals.search_modal.input.clear();
        self.modals.search_modal.cursor_pos = 0;
    }

    pub fn fetch_random_article(&mut self) {
        if self.active_pane().is_loading {
            return;
        }

        let is_empty = matches!(self.active_pane().content, PaneContent::Empty);
        let pane_id = if is_empty {
            self.active_pane().id
        } else {
            let next_id = self.workspace.next_pane_id;
            self.workspace.next_pane_id += 1;
            let tab_name = "loading...".to_string();
            self.workspace
                .tabs
                .push(crate::app::tab::Tab::new(tab_name, next_id));
            self.workspace.active_tab_idx = self.workspace.tabs.len() - 1;
            next_id
        };

        if let Some(pane) = self.find_pane_mut(pane_id) {
            pane.is_loading = true;
            pane.scroll_offset = 0;
        }

        self.send_fetch_random_article(pane_id);
    }

    pub fn type_search_char(&mut self, c: char) {
        if matches!(
            self.input_mode,
            crate::app::InputMode::Search
                | crate::app::InputMode::RenameList
                | crate::app::InputMode::CreateNewList
                | crate::app::InputMode::SleepTimerPrompt
        ) {
            if let Some((byte_idx, _)) = self
                .modals
                .search_modal
                .input
                .char_indices()
                .nth(self.modals.search_modal.cursor_pos)
            {
                self.modals.search_modal.input.insert(byte_idx, c);
            } else {
                self.modals.search_modal.input.push(c);
            }
            self.modals.search_modal.cursor_pos += 1;
        }
    }

    pub fn backspace_search_char(&mut self) {
        if matches!(
            self.input_mode,
            crate::app::InputMode::Search
                | crate::app::InputMode::RenameList
                | crate::app::InputMode::CreateNewList
                | crate::app::InputMode::SleepTimerPrompt
        ) && self.modals.search_modal.cursor_pos > 0
        {
            let target_char = self.modals.search_modal.cursor_pos - 1;
            if let Some((byte_idx, _)) = self
                .modals
                .search_modal
                .input
                .char_indices()
                .nth(target_char)
            {
                self.modals.search_modal.input.remove(byte_idx);
            } else {
                self.modals.search_modal.input.pop();
            }
            self.modals.search_modal.cursor_pos -= 1;
        }
    }

    pub fn delete_word_left(&mut self) {
        if matches!(
            self.input_mode,
            crate::app::InputMode::Search
                | crate::app::InputMode::RenameList
                | crate::app::InputMode::CreateNewList
                | crate::app::InputMode::SleepTimerPrompt
        ) && self.modals.search_modal.cursor_pos > 0
        {
            let char_indices: Vec<(usize, char)> =
                self.modals.search_modal.input.char_indices().collect();
            let end_char = self.modals.search_modal.cursor_pos.min(char_indices.len());
            let mut start_char = end_char;

            while start_char > 0 && char_indices[start_char - 1].1.is_whitespace() {
                start_char -= 1;
            }
            while start_char > 0 && !char_indices[start_char - 1].1.is_whitespace() {
                start_char -= 1;
            }

            let start_byte = char_indices.get(start_char).map(|(b, _)| *b).unwrap_or(0);
            let end_byte = char_indices
                .get(end_char)
                .map(|(b, _)| *b)
                .unwrap_or(self.modals.search_modal.input.len());

            self.modals.search_modal.input.drain(start_byte..end_byte);
            self.modals.search_modal.cursor_pos = start_char;
        }
    }

    pub fn delete_search_char(&mut self) {
        if matches!(
            self.input_mode,
            crate::app::InputMode::Search
                | crate::app::InputMode::RenameList
                | crate::app::InputMode::CreateNewList
                | crate::app::InputMode::SleepTimerPrompt
        )
            && let Some((byte_idx, _)) = self
                .modals
                .search_modal
                .input
                .char_indices()
                .nth(self.modals.search_modal.cursor_pos)
            {
                self.modals.search_modal.input.remove(byte_idx);
            }
    }

    pub fn move_search_cursor_left(&mut self) {
        if matches!(
            self.input_mode,
            crate::app::InputMode::Search
                | crate::app::InputMode::RenameList
                | crate::app::InputMode::CreateNewList
                | crate::app::InputMode::SleepTimerPrompt
        ) {
            self.modals.search_modal.cursor_pos =
                self.modals.search_modal.cursor_pos.saturating_sub(1);
        }
    }

    pub fn move_search_cursor_right(&mut self) {
        if matches!(
            self.input_mode,
            crate::app::InputMode::Search
                | crate::app::InputMode::RenameList
                | crate::app::InputMode::CreateNewList
                | crate::app::InputMode::SleepTimerPrompt
        ) {
            let char_count = self.modals.search_modal.input.chars().count();
            if self.modals.search_modal.cursor_pos < char_count {
                self.modals.search_modal.cursor_pos += 1;
            }
        }
    }

    pub fn move_search_cursor_home(&mut self) {
        if matches!(
            self.input_mode,
            crate::app::InputMode::Search
                | crate::app::InputMode::RenameList
                | crate::app::InputMode::CreateNewList
                | crate::app::InputMode::SleepTimerPrompt
        ) {
            self.modals.search_modal.cursor_pos = 0;
        }
    }

    pub fn move_search_cursor_end(&mut self) {
        if matches!(
            self.input_mode,
            crate::app::InputMode::Search
                | crate::app::InputMode::RenameList
                | crate::app::InputMode::CreateNewList
                | crate::app::InputMode::SleepTimerPrompt
        ) {
            self.modals.search_modal.cursor_pos = self.modals.search_modal.input.chars().count();
        }
    }

    pub fn submit_search(&mut self) {
        let query = self.modals.search_modal.input.trim().to_string();
        let open_new_tab = self.modals.search_modal.opens_new_tab;
        self.exit_search_mode();

        if !query.is_empty() {
            if open_new_tab {
                let is_empty = matches!(self.active_pane().content, PaneContent::Empty);
                if !is_empty {
                    self.new_tab();
                }
            }
            let request_id = self.next_request_id();
            let active_pane = self.active_pane_mut();
            let pane_id = active_pane.id;
            active_pane.current_request_id = request_id;
            active_pane.is_loading = true;
            active_pane.loading_title = Some(format!("search: {}", query.to_lowercase()));
            active_pane.selected_idx = 0;
            let limit = self.user_data.config.search.limit;
            let timeout = self.user_data.config.network.timeout;
            self.network.send(NetworkCommand::Search {
                request_id,
                pane_id,
                query,
                limit,
                timeout,
            });
        }
    }

    pub(crate) fn search_result_line_range(
        items: &[SearchResultItem],
        selected_idx: usize,
    ) -> (usize, usize) {
        let start = items
            .iter()
            .take(selected_idx)
            .map(|item| 2 + usize::from(!item.snippet.is_empty()))
            .sum();

        let end = if let Some(item) = items.get(selected_idx) {
            start + 2 + usize::from(!item.snippet.is_empty())
        } else {
            start
        };
        (start, end)
    }

    pub(crate) fn keep_search_selection_visible(
        pane: &mut crate::app::pane::Pane,
        term_height: u16,
    ) {
        let PaneContent::SearchResults { items, .. } = &pane.content else {
            return;
        };
        if items.is_empty() {
            return;
        }

        let (selected_start, selected_end) =
            Self::search_result_line_range(items, pane.selected_idx);
        let viewport_height = if pane.viewport_height > 0 {
            pane.viewport_height
        } else {
            (term_height as usize).saturating_sub(4).max(1)
        };

        if selected_start < pane.scroll_offset {
            pane.scroll_offset = selected_start;
        } else if selected_end > pane.scroll_offset + viewport_height {
            pane.scroll_offset = selected_end - viewport_height;
        }
    }

    pub fn enter_local_search_mode(&mut self) {
        self.input_mode = crate::app::InputMode::LocalSearch;
        let pane = self.active_pane_mut();
        if let crate::app::PaneContent::ArticleText { parsed_doc, .. } = &mut pane.content {
            parsed_doc.ensure_plain_text_lower();
        }
        pane.search.query.clear();
        pane.search.matches.clear();
        pane.search.selected_match_idx = None;
    }

    pub fn clear_local_search(&mut self) {
        let pane = self.active_pane_mut();
        pane.search.query.clear();
        pane.search.matches.clear();
        pane.search.selected_match_idx = None;
    }

    pub(crate) fn keep_local_match_visible(pane: &mut crate::app::pane::Pane, term_height: u16) {
        let target_line = match (pane.search.selected_match_idx, &pane.search.matches) {
            (Some(idx), matches) if !matches.is_empty() => matches[idx].line_idx,
            _ => return,
        };

        let viewport_height = if pane.viewport_height > 0 {
            pane.viewport_height
        } else {
            (term_height as usize).saturating_sub(4).max(1)
        };

        let margin = 8.min(viewport_height.saturating_sub(1) / 2);

        let top_threshold = pane.scroll_offset.saturating_add(margin);
        let bottom_threshold = (pane.scroll_offset + viewport_height).saturating_sub(margin);

        if target_line < top_threshold {
            pane.scroll_offset = target_line.saturating_sub(margin);
        } else if target_line >= bottom_threshold {
            pane.scroll_offset = (target_line + margin + 1).saturating_sub(viewport_height);
        }
    }

    fn sync_link_focus_to_current_match(pane: &mut Pane) {
        let Some(match_idx) = pane.search.selected_match_idx else {
            return;
        };
        let Some(m) = pane.search.matches.get(match_idx) else {
            return;
        };
        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content
            && let Some(link_idx) = parsed_doc.links.iter().position(|link| {
                link.span_indices
                    .iter()
                    .any(|&(l, s)| l == m.line_idx && s == m.span_idx)
            }) {
                pane.selected_link_idx = Some(link_idx);
            }
    }

    pub fn update_local_search(&mut self, term_height: u16) {
        let pane = self.active_pane_mut();
        pane.recompute_local_matches();
        if !pane.search.matches.is_empty() {
            pane.search.selected_match_idx = Some(0);
            Self::keep_local_match_visible(pane, term_height);
            Self::sync_link_focus_to_current_match(pane);
        }
    }

    pub fn next_local_match(&mut self, term_height: u16) {
        let pane = self.active_pane_mut();
        if pane.search.matches.is_empty() {
            return;
        }
        let next_idx = match pane.search.selected_match_idx {
            Some(idx) => (idx + 1) % pane.search.matches.len(),
            None => 0,
        };
        pane.search.selected_match_idx = Some(next_idx);
        Self::keep_local_match_visible(pane, term_height);
        Self::sync_link_focus_to_current_match(pane);
    }

    pub fn prev_local_match(&mut self, term_height: u16) {
        let pane = self.active_pane_mut();
        if pane.search.matches.is_empty() {
            return;
        }
        let len = pane.search.matches.len();
        let prev_idx = match pane.search.selected_match_idx {
            Some(idx) => {
                if idx == 0 {
                    len - 1
                } else {
                    idx - 1
                }
            }
            None => len - 1,
        };
        pane.search.selected_match_idx = Some(prev_idx);
        Self::keep_local_match_visible(pane, term_height);
        Self::sync_link_focus_to_current_match(pane);
    }
}
