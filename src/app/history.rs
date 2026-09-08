use super::App;

impl App {
    pub fn open_article(&mut self, title: &str) {
        let current_title = self.active_pane().title();
        let pane_id = self.active_pane().id;
        let active_pane = self.active_pane_mut();
        if let Some(old_title) = current_title {
            if old_title != title {
                active_pane.history_back.push(old_title);
                active_pane.history_forward.clear();
            }
        }
        active_pane.prepare_for_article_fetch(title);
        self.send_fetch_article(pane_id, title.to_string());
    }

    pub fn open_article_in_background_tab(&mut self, title: &str) {
        let cur_tab = self.workspace.active_tab_idx;
        self.new_tab();
        let pane_id = self.active_pane().id;
        self.active_pane_mut().prepare_for_article_fetch(title);
        self.send_fetch_article(pane_id, title.to_string());
        self.workspace.active_tab_idx = cur_tab;
        self.set_status_message(format!("opened '{}' in background tab", title));
    }

    pub fn intra_jump_back(&mut self, term_height: u16) {
        let pane = self.active_pane_mut();
        if let Some(prev_scroll) = pane.intra_jump_back.pop() {
            let cur_scroll = pane.scroll_offset;
            pane.intra_jump_forward.push(cur_scroll);
            pane.scroll_offset = prev_scroll;
            self.clamp_link_selection_to_viewport(term_height);
        }
    }

    pub fn intra_jump_forward(&mut self, term_height: u16) {
        let pane = self.active_pane_mut();
        if let Some(next_scroll) = pane.intra_jump_forward.pop() {
            let cur_scroll = pane.scroll_offset;
            pane.intra_jump_back.push(cur_scroll);
            pane.scroll_offset = next_scroll;
            self.clamp_link_selection_to_viewport(term_height);
        }
    }

    pub fn history_back(&mut self) {
        let current_title = self.active_pane().title();
        let active_pane = self.active_pane_mut();
        if let Some(target_title) = active_pane.history_back.pop() {
            if let Some(cur) = current_title {
                active_pane.history_forward.push(cur);
            }
            let pane_id = active_pane.id;
            active_pane.prepare_for_article_fetch(&target_title);
            self.send_fetch_article(pane_id, target_title);
        }
    }

    pub fn history_forward(&mut self) {
        let current_title = self.active_pane().title();
        let active_pane = self.active_pane_mut();
        if let Some(target_title) = active_pane.history_forward.pop() {
            if let Some(cur) = current_title {
                active_pane.history_back.push(cur);
            }
            let pane_id = active_pane.id;
            active_pane.prepare_for_article_fetch(&target_title);
            self.send_fetch_article(pane_id, target_title);
        }
    }
}
