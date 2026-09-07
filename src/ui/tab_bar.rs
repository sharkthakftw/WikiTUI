use crate::app::{App, PaneContent};
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Default)]
pub struct TabBarCache {
    pub key: u64,
    pub rendered_line: Line<'static>,
    pub tab_titles: Vec<String>,
    pub visible_range: (usize, usize),
}

fn compute_tab_bar_key(app: &App, area_width: u16) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    app.active_tab_idx.hash(&mut hasher);
    app.tabs.len().hash(&mut hasher);
    area_width.hash(&mut hasher);
    app.config.ui.icons.hash(&mut hasher);

    for tab in &app.tabs {
        tab.name.hash(&mut hasher);
        tab.active_pane_idx.hash(&mut hasher);
        tab.panes.len().hash(&mut hasher);

        let loading_pane = tab.panes.iter().find(|p| p.is_loading);
        if let Some(pane) = loading_pane {
            true.hash(&mut hasher);
            pane.loading_title.as_deref().hash(&mut hasher);
            crate::ui::current_spinner_frame().hash(&mut hasher);
        } else if let Some(pane) = tab.panes.get(tab.active_pane_idx) {
            false.hash(&mut hasher);
            match &pane.content {
                PaneContent::ArticleText {
                    title, parsed_doc, ..
                } => {
                    0u8.hash(&mut hasher);
                    title.hash(&mut hasher);
                    parsed_doc.spoken_audio.is_some().hash(&mut hasher);
                    app.saved_lists
                        .is_article_saved_anywhere(title)
                        .hash(&mut hasher);
                }
                PaneContent::SearchResults { query, .. } => {
                    1u8.hash(&mut hasher);
                    query.hash(&mut hasher);
                }
                PaneContent::Error(e) => {
                    2u8.hash(&mut hasher);
                    e.hash(&mut hasher);
                }
                PaneContent::Empty => {
                    3u8.hash(&mut hasher);
                }
            }
        }
    }

    hasher.finish()
}

fn build_tab_bar_line(
    tab_titles: &[String],
    active_idx: usize,
    area_width: u16,
) -> (Line<'static>, (usize, usize)) {
    let total_tabs = tab_titles.len();
    let active_idx = active_idx.min(total_tabs.saturating_sub(1));
    let (start_idx, end_idx) = compute_visible_range(tab_titles, active_idx, area_width);

    let mut tab_spans = Vec::new();
    tab_spans.push(Span::raw(" "));

    if start_idx > 0 {
        tab_spans.push(Span::styled(
            "< ",
            Style::default().fg(theme::YELLOW).bold(),
        ));
    }

    for (i, title) in tab_titles
        .iter()
        .enumerate()
        .take(end_idx + 1)
        .skip(start_idx)
    {
        let is_active = i == active_idx;
        if is_active {
            let active_style = Style::default().fg(theme::LIME).bg(theme::LIGHT_BG).bold();
            tab_spans.push(Span::styled(format!(" {} ", title), active_style));
        } else {
            let inactive_style = Style::default().fg(theme::GREY);
            tab_spans.push(Span::styled(format!(" {} ", title), inactive_style));
        }
        tab_spans.push(Span::raw("  "));
    }

    if end_idx + 1 < total_tabs {
        tab_spans.push(Span::styled(
            "> ",
            Style::default().fg(theme::YELLOW).bold(),
        ));
    }

    (Line::from(tab_spans), (start_idx, end_idx))
}

pub fn compute_tab_titles(app: &App) -> Vec<String> {
    app.tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let loading_pane = tab.panes.iter().find(|p| p.is_loading);
            let show_icons = app.config.ui.icons;
            let (icon, raw_title, is_saved) = if let Some(pane) = loading_pane {
                let title = pane
                    .loading_title
                    .as_deref()
                    .unwrap_or("loading...")
                    .to_lowercase();
                (crate::ui::current_spinner_frame(), title, false)
            } else if let Some(active_pane) = tab.panes.get(tab.active_pane_idx) {
                match &active_pane.content {
                    PaneContent::ArticleText {
                        title, parsed_doc, ..
                    } => {
                        let saved = app.saved_lists.is_article_saved_anywhere(title);
                        let has_audio = parsed_doc.spoken_audio.is_some();
                        let icon_str = if show_icons {
                            if has_audio {
                                "󰎆"
                            } else {
                                "≡"
                            }
                        } else if has_audio {
                            "♪"
                        } else {
                            ""
                        };
                        (icon_str, title.to_lowercase(), saved)
                    }
                    PaneContent::SearchResults { query, .. } => (
                        if show_icons { "󰍉" } else { "" },
                        format!("search: {}", query.to_lowercase()),
                        false,
                    ),
                    PaneContent::Error(_) => (
                        if show_icons { "󰅚" } else { "" },
                        "error".to_string(),
                        false,
                    ),
                    PaneContent::Empty => (
                        if show_icons { "󰋜" } else { "" },
                        tab.name.to_lowercase(),
                        false,
                    ),
                }
            } else {
                (
                    if show_icons { "󰋜" } else { "" },
                    tab.name.to_lowercase(),
                    false,
                )
            };

            let star = if is_saved {
                if show_icons {
                    " ★"
                } else {
                    " *"
                }
            } else {
                ""
            };

            if icon.is_empty() {
                if app.tabs.len() > 1 {
                    format!("{} {}{}", i + 1, raw_title, star)
                } else {
                    format!("{}{}", raw_title, star)
                }
            } else if app.tabs.len() > 1 {
                format!("{} {} {}{}", icon, i + 1, raw_title, star)
            } else {
                format!("{} {}{}", icon, raw_title, star)
            }
        })
        .collect()
}

pub fn compute_visible_range(
    tab_titles: &[String],
    active_idx: usize,
    area_width: u16,
) -> (usize, usize) {
    let total_tabs = tab_titles.len();
    if total_tabs == 0 {
        return (0, 0);
    }

    let active_idx = active_idx.min(total_tabs - 1);
    let max_available_width = (area_width as usize).saturating_sub(4);

    let mut start_idx = active_idx;
    let mut end_idx = active_idx;
    let mut current_width = tab_titles[active_idx].chars().count() + 4;

    loop {
        let mut expanded = false;

        if end_idx + 1 < total_tabs {
            let next_w = tab_titles[end_idx + 1].chars().count() + 4;
            if current_width + next_w <= max_available_width {
                end_idx += 1;
                current_width += next_w;
                expanded = true;
            }
        }

        if start_idx > 0 {
            let prev_w = tab_titles[start_idx - 1].chars().count() + 4;
            if current_width + prev_w <= max_available_width {
                start_idx -= 1;
                current_width += prev_w;
                expanded = true;
            }
        }

        if !expanded {
            break;
        }
    }

    (start_idx, end_idx)
}

fn ensure_tab_bar_cache(app: &mut App, area_width: u16) -> &TabBarCache {
    let key = compute_tab_bar_key(app, area_width);
    let needs_update = app
        .tab_bar_cache
        .as_ref()
        .map(|c| c.key != key)
        .unwrap_or(true);
    if needs_update {
        let tab_titles = compute_tab_titles(app);
        let active_idx = app.active_tab_idx.min(tab_titles.len().saturating_sub(1));
        let (new_line, visible_range) = build_tab_bar_line(&tab_titles, active_idx, area_width);
        app.tab_bar_cache = Some(TabBarCache {
            key,
            rendered_line: new_line,
            tab_titles,
            visible_range,
        });
    }
    app.tab_bar_cache.as_ref().unwrap()
}

pub fn get_tab_at_col(app: &mut App, area_width: u16, target_col: u16) -> Option<usize> {
    if app.tabs.is_empty() {
        return None;
    }

    let cache = ensure_tab_bar_cache(app, area_width);
    let (start_idx, end_idx) = cache.visible_range;
    let tab_titles = &cache.tab_titles;
    let total_tabs = tab_titles.len();
    let mut col: u16 = 1;
    if start_idx > 0 {
        if target_col >= col && target_col < col + 2 {
            return Some(start_idx - 1);
        }
        col += 2;
    }

    for (i, title) in tab_titles
        .iter()
        .enumerate()
        .take(end_idx + 1)
        .skip(start_idx)
    {
        let tab_width = (title.chars().count() + 2) as u16;
        if target_col >= col && target_col < col + tab_width {
            return Some(i);
        }
        col += tab_width + 2;
    }

    if end_idx + 1 < total_tabs && target_col >= col && target_col < col + 2 {
        return Some(end_idx + 1);
    }

    None
}

pub fn render(f: &mut Frame, app: &mut App, area: Rect) {
    if app.tabs.is_empty() {
        return;
    }

    let cache = ensure_tab_bar_cache(app, area.width);
    let tab_bar_paragraph = Paragraph::new(cache.rendered_line.clone());
    f.render_widget(tab_bar_paragraph, area);
}
