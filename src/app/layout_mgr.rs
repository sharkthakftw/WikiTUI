use crate::app::pane::Pane;
use crate::app::tab::Tab;
use crate::app::App;
use crate::layout::SplitDirection;

impl App {
    pub(crate) fn find_pane(&self, target_id: usize) -> Option<&Pane> {
        for tab in &self.workspace.tabs {
            for pane in &tab.panes {
                if pane.id == target_id {
                    return Some(pane);
                }
            }
        }
        None
    }

    pub(crate) fn find_pane_mut(&mut self, target_id: usize) -> Option<&mut Pane> {
        for tab in &mut self.workspace.tabs {
            for pane in &mut tab.panes {
                if pane.id == target_id {
                    return Some(pane);
                }
            }
        }
        None
    }

    pub fn new_tab(&mut self) {
        let name = "new tab".to_string();
        self.workspace
            .tabs
            .push(Tab::new(name, self.workspace.next_pane_id));
        self.workspace.next_pane_id += 1;
        self.workspace.prev_tab_idx = Some(self.workspace.active_tab_idx);
        self.workspace.active_tab_idx = self.workspace.tabs.len() - 1;
    }

    pub fn next_tab(&mut self) {
        if self.workspace.tabs.len() > 1 {
            let next = (self.workspace.active_tab_idx + 1) % self.workspace.tabs.len();
            self.switch_to_tab(next);
        }
    }

    pub fn prev_tab(&mut self) {
        if self.workspace.tabs.len() > 1 {
            let prev = if self.workspace.active_tab_idx == 0 {
                self.workspace.tabs.len() - 1
            } else {
                self.workspace.active_tab_idx - 1
            };
            self.switch_to_tab(prev);
        }
    }

    pub fn switch_to_tab(&mut self, idx: usize) {
        if idx < self.workspace.tabs.len() && idx != self.workspace.active_tab_idx {
            self.workspace.prev_tab_idx = Some(self.workspace.active_tab_idx);
            self.workspace.active_tab_idx = idx;
        }
    }

    pub fn toggle_alternate_tab(&mut self) {
        if let Some(prev_idx) = self.workspace.prev_tab_idx
            && prev_idx < self.workspace.tabs.len() && prev_idx != self.workspace.active_tab_idx {
                self.switch_to_tab(prev_idx);
            }
    }

    fn record_closed_panes(&mut self, panes: impl IntoIterator<Item = Pane>) {
        for pane in panes {
            if let Some(title) = pane.title() {
                self.workspace
                    .closed_tabs_stack
                    .push(crate::app::ClosedTabState {
                        title,
                        scroll_offset: pane.scroll_offset,
                        history_back: pane.history_back,
                        history_forward: pane.history_forward,
                    });
            }
        }
    }

    fn remove_tab_internal(&mut self, idx: usize) {
        let removed_tab = self.workspace.tabs.remove(idx);
        if let Some(prev) = self.workspace.prev_tab_idx {
            if prev == idx {
                self.workspace.prev_tab_idx = None;
            } else if prev > idx {
                self.workspace.prev_tab_idx = Some(prev - 1);
            }
        }
        self.record_closed_panes(removed_tab.panes);
        if self.workspace.active_tab_idx > idx {
            self.workspace.active_tab_idx -= 1;
        } else if self.workspace.active_tab_idx >= self.workspace.tabs.len() {
            self.workspace.active_tab_idx = self.workspace.tabs.len().saturating_sub(1);
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx >= self.workspace.tabs.len() {
            return;
        }
        if idx == self.workspace.active_tab_idx {
            self.close_current_tab();
        } else if self.workspace.tabs.len() > 1 {
            self.remove_tab_internal(idx);
        }
    }

    pub fn close_current_tab(&mut self) {
        self.maybe_mark_article_read();
        let closed_idx = self.workspace.active_tab_idx;
        if self.workspace.tabs.len() > 1 {
            self.remove_tab_internal(closed_idx);
        } else {
            let new_pane_id = self.workspace.next_pane_id;
            self.workspace.next_pane_id += 1;
            let old_tab = std::mem::replace(
                &mut self.workspace.tabs[0],
                Tab::new("home".to_string(), new_pane_id),
            );
            self.record_closed_panes(old_tab.panes);
            self.workspace.active_tab_idx = 0;
            self.workspace.prev_tab_idx = None;
        }
    }

    pub fn split_active_pane(&mut self, direction: SplitDirection) {
        self.mark_active_article_read();
        let new_pane_id = self.workspace.next_pane_id;
        self.workspace.next_pane_id += 1;

        let tab = self.active_tab_mut();
        let current_pane_idx = tab.active_pane_idx;

        tab.panes.push(Pane::new(new_pane_id));
        let new_pane_idx = tab.panes.len() - 1;

        tab.layout_root
            .split_pane(current_pane_idx, new_pane_idx, direction);
        tab.active_pane_idx = new_pane_idx;
    }

    pub fn close_active_pane(&mut self) {
        self.maybe_mark_article_read();
        if self.active_tab().panes.len() <= 1 {
            self.close_current_tab();
            return;
        }

        let tab = self.active_tab_mut();
        let target_idx = tab.active_pane_idx;

        let closed_state = if let Some(new_root) = tab.layout_root.remove_pane(target_idx) {
            tab.layout_root = new_root;
            tab.layout_root.decrement_indices_above(target_idx);

            let removed_pane = tab.panes.remove(target_idx);
            if tab.active_pane_idx >= tab.panes.len() {
                tab.active_pane_idx = tab.panes.len().saturating_sub(1);
            }
            removed_pane
                .title()
                .map(|title| crate::app::ClosedTabState {
                    title,
                    scroll_offset: removed_pane.scroll_offset,
                    history_back: removed_pane.history_back,
                    history_forward: removed_pane.history_forward,
                })
        } else {
            None
        };

        if let Some(closed) = closed_state {
            self.workspace.closed_tabs_stack.push(closed);
        }
    }

    pub fn reopen_last_closed(&mut self) {
        if let Some(closed) = self.workspace.closed_tabs_stack.pop() {
            let pane_id = self.workspace.next_pane_id;
            self.workspace.next_pane_id += 1;

            let mut pane = Pane::new(pane_id);
            pane.prepare_for_article_fetch(&closed.title);
            pane.scroll_offset = closed.scroll_offset;
            pane.history_back = closed.history_back;
            pane.history_forward = closed.history_forward;

            self.send_fetch_article(pane_id, closed.title.clone());

            let tab = Tab {
                name: closed.title,
                panes: vec![pane],
                active_pane_idx: 0,
                layout_root: crate::layout::LayoutNode::Leaf(0),
            };

            let is_single_empty_home = self.workspace.tabs.len() == 1
                && self.workspace.tabs[0].name == "home"
                && self.workspace.tabs[0].panes.len() == 1
                && self.workspace.tabs[0].panes[0].title().is_none();

            if is_single_empty_home {
                self.workspace.tabs[0] = tab;
                self.workspace.active_tab_idx = 0;
            } else {
                self.workspace.tabs.push(tab);
                self.workspace.active_tab_idx = self.workspace.tabs.len() - 1;
            }
        }
    }

    pub fn navigate_panes(&mut self, dir: char, term_width: u16, term_height: u16) {
        use crate::layout::find_pane_in_direction;
        use ratatui::layout::Rect;

        let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
        let tab = self.active_tab_mut();
        let rects = tab.layout_root.compute_rects(main_rect);

        if let Some(next_idx) = find_pane_in_direction(&rects, tab.active_pane_idx, dir) {
            tab.active_pane_idx = next_idx;
        }
    }

    pub fn move_pane(&mut self, dir: char, term_width: u16, term_height: u16) {
        use crate::layout::find_pane_in_direction;
        use ratatui::layout::Rect;

        let main_rect = Rect::new(0, 1, term_width, term_height.saturating_sub(2));
        let tab = self.active_tab_mut();
        let rects = tab.layout_root.compute_rects(main_rect);

        if let Some(target_idx) = find_pane_in_direction(&rects, tab.active_pane_idx, dir) {
            let active_idx = tab.active_pane_idx;
            tab.layout_root.swap_panes(active_idx, target_idx);
        }
    }

    pub fn resize_active_split(&mut self, delta: i16) {
        let tab = self.active_tab_mut();
        let target_idx = tab.active_pane_idx;
        tab.layout_root.resize_pane(target_idx, delta);
    }
}
