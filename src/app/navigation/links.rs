use crate::app::pane::PaneContent;
use crate::app::{is_article_link, App};
use crate::layout::SplitDirection;

impl App {
    pub fn clamp_link_selection_to_viewport(&mut self, term_height: u16) {
        let pane = self.active_pane_mut();
        let PaneContent::ArticleText { parsed_doc, .. } = &pane.content else {
            return;
        };
        if parsed_doc.links.is_empty() {
            pane.selected_link_idx = None;
            return;
        }

        let viewport_h = pane.effective_viewport_height(term_height);

        let view_start = pane.scroll_offset;
        let view_end = pane.scroll_offset + viewport_h;

        let is_visible = pane.selected_link_idx.is_some_and(|idx| {
            if let Some(link) = parsed_doc.links.get(idx) {
                link.span_indices
                    .iter()
                    .any(|(l, _)| *l >= view_start && *l < view_end)
            } else {
                false
            }
        });

        if !is_visible {
            let mut candidate_idx = parsed_doc.links.partition_point(|link| {
                link.span_indices
                    .first()
                    .map(|&(l, _)| l < view_start)
                    .unwrap_or(true)
            });

            if candidate_idx > 0 {
                if let Some(prev_link) = parsed_doc.links.get(candidate_idx - 1) {
                    if prev_link
                        .span_indices
                        .last()
                        .is_some_and(|&(l, _)| l >= view_start)
                    {
                        candidate_idx -= 1;
                    }
                }
            }

            #[cfg(debug_assertions)]
            if candidate_idx < parsed_doc.links.len() {
                if let Some(&(first_line, _)) = parsed_doc.links[candidate_idx].span_indices.first() {
                    debug_assert!(
                        first_line >= view_start
                            || parsed_doc.links[candidate_idx]
                                .span_indices
                                .last()
                                .is_some_and(|&(l, _)| l >= view_start)
                    );
                }
            }

            if candidate_idx < parsed_doc.links.len() {
                pane.selected_link_idx = Some(candidate_idx);
            } else {
                pane.selected_link_idx = Some(parsed_doc.links.len() - 1);
            }
        }
    }

    pub fn active_selected_target(&self) -> Option<String> {
        let recent = self.get_continue_reading_articles();
        self.active_pane().selected_target(&recent)
    }

    pub fn activate_selected(&mut self, term_height: u16) {
        let selected_title = self.active_selected_target();

        if let Some(target) = selected_title {
            if let Some(anchor) = target.strip_prefix('#') {
                let pane = self.active_pane_mut();
                if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
                    let target_line_opt = parsed_doc
                        .reference_targets
                        .get(anchor)
                        .or_else(|| parsed_doc.reference_targets.get(&target))
                        .copied()
                        .or_else(|| {
                            parsed_doc.reference_targets.iter().find_map(|(k, &line)| {
                                if k == anchor || k.ends_with(anchor) || anchor.ends_with(k) {
                                    Some(line)
                                } else {
                                    None
                                }
                            })
                        })
                        .or_else(|| {
                            parsed_doc.links.iter().find_map(|l| {
                                if l.title == target || l.title == anchor {
                                    l.span_indices.first().map(|(line, _)| *line)
                                } else {
                                    None
                                }
                            })
                        });

                    if let Some(target_line) = target_line_opt {
                        let current_scroll = pane.scroll_offset;
                        pane.intra_jump_back.push(current_scroll);
                        pane.intra_jump_forward.clear();
                        pane.scroll_offset = target_line;

                        if let Some(target_link_idx) = parsed_doc.links.iter().position(|l| {
                            l.span_indices.iter().any(|(line, _)| *line == target_line)
                        }) {
                            pane.selected_link_idx = Some(target_link_idx);
                        }

                        self.clamp_link_selection_to_viewport(term_height);
                        self.set_status_message(if anchor.starts_with("cite_note") {
                            "jumped to reference (ctrl-o to return)"
                        } else {
                            "jumped to citation (ctrl-o to return)"
                        });
                    }
                }
            } else if target.starts_with("http://")
                || target.starts_with("https://")
                || target.starts_with("//")
            {
                if crate::clipboard::copy_to_clipboard(&target) {
                    self.set_status_message(format!("copied external link: {}", target));
                } else {
                    self.set_status_message("failed to copy (no clipboard backend found)");
                }
            } else if is_article_link(&target) {
                self.open_article(&target);
            }
        }
    }

    pub fn activate_search_result_digit(&mut self, digit: char) {
        let idx = if digit == '0' {
            9
        } else {
            (digit as usize) - ('1' as usize)
        };
        let pane = self.active_pane_mut();
        let target_title = if let PaneContent::SearchResults { items, .. } = &mut pane.content {
            if idx < items.len() {
                pane.selected_idx = idx;
                Some(items[idx].title.clone())
            } else {
                None
            }
        } else {
            None
        };

        if let Some(title) = target_title {
            self.open_article(&title);
        }
    }

    pub fn activate_selected_in_new_tab(&mut self) {
        let selected_title = self.active_selected_target();

        if let Some(title) = selected_title.filter(|t| is_article_link(t)) {
            self.new_tab();
            let pane_id = self.active_pane().id;
            let active_pane = self.active_pane_mut();
            active_pane.prepare_for_article_fetch(&title);
            self.send_fetch_article(pane_id, title);
        }
    }

    pub fn activate_selected_in_background_tab(&mut self) {
        let selected_title = self.active_selected_target();

        if let Some(title) = selected_title.filter(|t| is_article_link(t)) {
            self.open_article_in_background_tab(&title);
        }
    }

    pub fn activate_selected_in_split(&mut self, direction: SplitDirection) {
        let selected_title = self.active_selected_target();

        if let Some(title) = selected_title.filter(|t| is_article_link(t)) {
            self.split_active_pane(direction);
            let pane_id = self.active_pane().id;
            let active_pane = self.active_pane_mut();
            active_pane.prepare_for_article_fetch(&title);
            self.send_fetch_article(pane_id, title);
        }
    }

    pub fn focus_next_link(&mut self) {
        let pane = self.active_pane_mut();
        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            if parsed_doc.links.is_empty() {
                return;
            }
            let next_idx = match pane.selected_link_idx {
                Some(idx) => (idx + 1) % parsed_doc.links.len(),
                None => 0,
            };
            pane.selected_link_idx = Some(next_idx);

            let link_line = parsed_doc.links[next_idx]
                .span_indices
                .first()
                .map_or(0, |(l, _)| *l);
            if link_line < pane.scroll_offset {
                pane.scroll_offset = link_line;
            } else if link_line >= pane.scroll_offset + 10 {
                pane.scroll_offset = link_line.saturating_sub(5);
            }
        }
    }

    pub fn focus_prev_link(&mut self) {
        let pane = self.active_pane_mut();
        if let PaneContent::ArticleText { parsed_doc, .. } = &pane.content {
            if parsed_doc.links.is_empty() {
                return;
            }
            let len = parsed_doc.links.len();
            let prev_idx = match pane.selected_link_idx {
                Some(idx) => {
                    if idx == 0 {
                        len - 1
                    } else {
                        idx - 1
                    }
                }
                None => len - 1,
            };
            pane.selected_link_idx = Some(prev_idx);

            let link_line = parsed_doc.links[prev_idx]
                .span_indices
                .first()
                .map_or(0, |(l, _)| *l);
            if link_line < pane.scroll_offset {
                pane.scroll_offset = link_line;
            } else if link_line >= pane.scroll_offset + 10 {
                pane.scroll_offset = link_line.saturating_sub(5);
            }
        }
    }

    pub fn copy_focused_link(&mut self) {
        let pane = self.active_pane();
        let target_url = match &pane.content {
            PaneContent::ArticleText {
                title, parsed_doc, ..
            } => {
                if let Some(idx) = pane.selected_link_idx {
                    if let Some(link) = parsed_doc.links.get(idx) {
                        if link.title.starts_with("http://")
                            || link.title.starts_with("https://")
                            || link.title.starts_with("//")
                        {
                            link.title.clone()
                        } else {
                            format!(
                                "https://en.wikipedia.org/wiki/{}",
                                link.title.replace(' ', "_")
                            )
                        }
                    } else {
                        format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"))
                    }
                } else {
                    format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"))
                }
            }
            PaneContent::SearchResults { items, .. } => {
                if let Some(item) = items.get(pane.selected_idx) {
                    format!(
                        "https://en.wikipedia.org/wiki/{}",
                        item.title.replace(' ', "_")
                    )
                } else {
                    return;
                }
            }
            _ => return,
        };

        if crate::clipboard::copy_to_clipboard(&target_url) {
            self.set_status_message(format!("copied: {}", target_url));
        } else {
            self.set_status_message("failed to copy (no clipboard backend found)");
        }
    }

    pub fn copy_article_link(&mut self) {
        let pane = self.active_pane();
        let target_url = match &pane.content {
            PaneContent::ArticleText { title, .. } => {
                format!("https://en.wikipedia.org/wiki/{}", title.replace(' ', "_"))
            }
            PaneContent::SearchResults { query, .. } => {
                format!(
                    "https://en.wikipedia.org/wiki/Special:Search?search={}",
                    query.replace(' ', "_")
                )
            }
            _ => return,
        };

        if crate::clipboard::copy_to_clipboard(&target_url) {
            self.set_status_message(format!("copied article: {}", target_url));
            self.mark_active_article_read();
        } else {
            self.set_status_message("failed to copy (no clipboard backend found)");
        }
    }
}
