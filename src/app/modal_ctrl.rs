use super::{App, InputMode, PaneContent};

impl App {
    pub fn open_save_to_list_modal(&mut self) {
        let pane = self.active_pane();
        let (title, snippet) = match &pane.content {
            PaneContent::ArticleText { title, .. } => (title.clone(), None),
            PaneContent::SearchResults { items, .. } => {
                if let Some(item) = items.get(pane.selected_idx) {
                    (item.title.clone(), Some(item.snippet.clone()))
                } else {
                    return;
                }
            }
            _ => return,
        };

        if title.trim().is_empty() {
            return;
        }

        self.user_data.saved_lists = crate::saved_lists::SavedListsStore::load();
        self.modals.lists_modal.target_title = title;
        self.modals.lists_modal.target_snippet = snippet;
        self.modals.lists_modal.save_cursor_idx = 0;
        self.input_mode = InputMode::SaveToList;
    }

    pub fn open_saved_lists_viewer(&mut self) {
        self.user_data.saved_lists = crate::saved_lists::SavedListsStore::load();
        self.modals.lists_modal.viewer_list_idx = 0;
        self.modals.lists_modal.viewer_article_idx = 0;
        self.modals.lists_modal.viewer_focus_right = false;
        self.input_mode = InputMode::SavedListsViewer;
    }

    pub fn submit_create_new_list(&mut self) {
        let name = self.modals.search_modal.input.trim().to_string();
        if !name.is_empty() {
            let list_id = self.user_data.saved_lists.create_list(&name);
            if !self.modals.lists_modal.target_title.is_empty() {
                let target_title = self.modals.lists_modal.target_title.clone();
                let added = self
                    .user_data
                    .saved_lists
                    .toggle_article_in_list(&list_id, &target_title);
                self.mark_active_article_read();
                self.record_article_saved(&target_title, added);
            }
            self.set_status_message(format!("created list '{}'", name));
        }
        self.modals.search_modal.input.clear();
        self.modals.search_modal.cursor_pos = 0;
        self.input_mode = self.modals.lists_modal.create_return_mode.clone();
    }

    pub fn toggle_help_popup(&mut self) {
        if self.input_mode == InputMode::Help {
            self.input_mode = InputMode::Normal;
        } else {
            self.input_mode = InputMode::Help;
        }
    }

    pub fn toggle_categories_modal(&mut self) {
        if self.input_mode == InputMode::Categories {
            self.input_mode = InputMode::Normal;
            return;
        }

        if matches!(self.active_pane().content, PaneContent::ArticleText { .. }) {
            self.modals.categories_modal.cursor_idx = 0;
            self.modals.categories_modal.article_cursor_idx = 0;
            self.modals.categories_modal.focus_right = false;
            self.input_mode = InputMode::Categories;

            let first_cat =
                if let PaneContent::ArticleText { parsed_doc, .. } = &self.active_pane().content {
                    parsed_doc.categories.first().cloned()
                } else {
                    None
                };
            if let Some(cat) = first_cat {
                self.fetch_category_members_if_needed(&cat);
            }
        }
    }
}
