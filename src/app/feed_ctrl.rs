use crate::app::{App, InputMode};

impl App {
    pub fn open_daily_feed_modal(&mut self, kind: crate::ui::modals::DailyFeedKind) {
        if self.daily_feed.is_none() {
            self.send_fetch_daily_feed();
        }
        self.modals.daily_feed_modal = Some(crate::ui::modals::DailyFeedModalState {
            kind,
            cursor_idx: 0,
            link_idx: 0,
            otd_tab: crate::ui::modals::OnThisDayTab::Events,
            scroll: 0,
            cache: std::cell::RefCell::new(crate::ui::modals::DailyFeedCache::default()),
        });
        self.input_mode = InputMode::DailyFeedModal;
    }

    pub fn close_daily_feed_modal(&mut self) {
        self.modals.daily_feed_modal = None;
        self.input_mode = InputMode::Normal;
    }

    pub fn maybe_fetch_feed_batch(&mut self) {
        if !self.user_data.feed.is_fetching && self.user_data.feed.active_idx + 3 >= self.user_data.feed.items.len() {
            self.user_data.feed.is_fetching = true;
            self.send_fetch_feed_batch();
        }
    }

    pub fn toggle_feed_mode(&mut self) {
        let is_active = self.user_data.feed.toggle_active();
        if is_active {
            if !self.user_data.feed.profile.has_onboarded {
                self.input_mode = InputMode::CategoryOnboarding;
            } else if self.user_data.feed.items.is_empty() {
                self.maybe_fetch_feed_batch();
            }
        }
    }

    pub fn submit_category_onboarding(&mut self) {
        let chosen_indices: Vec<usize> = self
            .modals
            .onboarding
            .selected
            .iter()
            .enumerate()
            .filter_map(|(idx, &sel)| if sel { Some(idx) } else { None })
            .collect();

        self.user_data.feed.profile.complete_onboarding(&chosen_indices);
        self.input_mode = InputMode::Normal;
        if self.user_data.feed.items.is_empty() {
            self.maybe_fetch_feed_batch();
        }
    }

    pub fn reset_feed(&mut self) {
        self.user_data.feed.reset();
        self.user_data.saved_lists.clear_list("liked");
        self.modals.onboarding.cursor_idx = 0;
        self.modals.onboarding.selected = vec![
            false, false, false, false, true, false, false, true, true, false, false, true,
        ];
        self.input_mode = InputMode::CategoryOnboarding;
        self.set_status_message("feed reset: select initial categories");
    }

    pub fn toggle_feed_like(&mut self) {
        if let Some((title, _snippet, is_liked)) = self.user_data.feed.toggle_like() {
            self.user_data
                .saved_lists
                .set_article_in_list("liked", "Liked", &title, is_liked);
        }
    }

    pub fn maybe_mark_article_read(&mut self) {
        let (should_mark, title, categories) = {
            let pane = self.active_pane_mut();
            if pane.has_marked_read {
                return;
            }
            let crate::app::pane::PaneContent::ArticleText {
                title, parsed_doc, ..
            } = &pane.content
            else {
                return;
            };
            let dwelled = pane
                .opened_at
                .is_some_and(|t| t.elapsed() >= std::time::Duration::from_secs(8));
            let scrolled = pane.scroll_offset > 10;
            if dwelled || scrolled {
                pane.has_marked_read = true;
                (true, title.clone(), parsed_doc.categories.clone())
            } else {
                (false, String::new(), Vec::new())
            }
        };
        if should_mark {
            self.user_data.feed.profile.seen_articles.insert(title);
            self.user_data.feed.profile.record_engagement(&categories, 15);
            self.user_data.feed.profile.save();
        }
    }

    pub fn mark_active_article_read(&mut self) {
        let (should_mark, title, categories) = {
            let pane = self.active_pane_mut();
            if pane.has_marked_read {
                return;
            }
            let crate::app::pane::PaneContent::ArticleText {
                title, parsed_doc, ..
            } = &pane.content
            else {
                return;
            };
            pane.has_marked_read = true;
            (true, title.clone(), parsed_doc.categories.clone())
        };
        if should_mark {
            self.user_data.feed.profile.seen_articles.insert(title);
            self.user_data.feed.profile.record_engagement(&categories, 15);
            self.user_data.feed.profile.save();
        }
    }

    pub fn record_article_saved(&mut self, target_title: &str, added: bool) {
        let categories = {
            let pane = self.active_pane();
            if let crate::app::pane::PaneContent::ArticleText {
                title, parsed_doc, ..
            } = &pane.content
            {
                if title.eq_ignore_ascii_case(target_title) {
                    parsed_doc.categories.clone()
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        };

        if added {
            self.user_data
                .feed
                .profile
                .seen_articles
                .insert(target_title.to_string());
            if !categories.is_empty() {
                self.user_data.feed.profile.record_engagement(&categories, 50);
            }
        } else if !categories.is_empty() {
            self.user_data.feed.profile.record_engagement(&categories, -50);
        }
        self.user_data.feed.profile.save();
    }
}
