use crate::app::App;
use crate::theme;
use crate::ui::modals::utils::{
    compute_two_column_modal_areas, create_modal_block, create_selectable_line,
    render_selectable_list_column,
};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    Frame,
};

pub fn compute_saved_lists_viewer_areas(size: Rect) -> (Rect, Rect, Rect) {
    compute_two_column_modal_areas(80, 80, 35, size)
}

pub fn compute_list_viewer_scroll(
    cursor_idx: usize,
    visible_rows: usize,
    total_items: usize,
) -> usize {
    crate::ui::modals::utils::compute_centered_scroll(cursor_idx, visible_rows, total_items)
}

pub fn get_saved_lists_viewer_item_at(
    app: &App,
    is_right: bool,
    area: Rect,
    target_y: u16,
) -> Option<usize> {
    if target_y <= area.y || target_y >= area.y + area.height.saturating_sub(1) {
        return None;
    }
    let row_offset = (target_y - (area.y + 1)) as usize;
    let visible_rows = (area.height.saturating_sub(2)) as usize;
    if is_right {
        let selected_list = app.user_data.saved_lists.lists.get(app.modals.lists_modal.viewer_list_idx)?;
        let total = selected_list.articles.len();
        let scroll =
            compute_list_viewer_scroll(app.modals.lists_modal.viewer_article_idx, visible_rows, total);
        let idx = scroll + row_offset;
        if idx < total {
            Some(idx)
        } else {
            None
        }
    } else {
        let total = app.user_data.saved_lists.lists.len();
        let scroll =
            compute_list_viewer_scroll(app.modals.lists_modal.viewer_list_idx, visible_rows, total);
        let idx = scroll + row_offset;
        if idx < total {
            Some(idx)
        } else {
            None
        }
    }
}

pub fn render_saved_lists_viewer_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.user_data.config.ui.icons { "★" } else { "" };
    let (container_area, left_area, right_area) = compute_saved_lists_viewer_areas(size);
    f.render_widget(ratatui::widgets::Clear, container_area);
    let block = create_modal_block(
        icon,
        "saved lists & articles",
        theme::VIOLET,
        app.user_data.config.ui.rounded_borders,
    );
    f.render_widget(block, container_area);

    let left_border_color = if !app.modals.lists_modal.viewer_focus_right {
        theme::VIOLET
    } else {
        theme::GREY
    };
    let left_block = create_modal_block(
        "",
        "custom lists",
        left_border_color,
        app.user_data.config.ui.rounded_borders,
    );

    let mut list_lines = Vec::new();
    if app.user_data.saved_lists.lists.is_empty() {
        list_lines.push(Line::from(Span::styled(
            " no lists created yet.",
            Style::default().fg(theme::GREY).italic(),
        )));
    } else {
        for (idx, list) in app.user_data.saved_lists.lists.iter().enumerate() {
            let is_selected = idx == app.modals.lists_modal.viewer_list_idx;
            let is_active = !app.modals.lists_modal.viewer_focus_right;
            let suffix = format!(" ({})", list.articles.len());

            list_lines.push(create_selectable_line(
                &list.name,
                is_selected,
                is_active,
                theme::VIOLET,
                Some(&suffix),
            ));
        }
    }

    render_selectable_list_column(
        f,
        left_area,
        left_block,
        list_lines,
        app.modals.lists_modal.viewer_list_idx,
        app.user_data.saved_lists.lists.len(),
    );

    let right_border_color = if app.modals.lists_modal.viewer_focus_right {
        theme::YELLOW
    } else {
        theme::GREY
    };

    let selected_list = app.user_data.saved_lists.lists.get(app.modals.lists_modal.viewer_list_idx);
    let right_title = selected_list
        .map(|l| format!("articles in '{}'", l.name))
        .unwrap_or_else(|| "articles".to_string());

    let right_block = create_modal_block(
        "",
        &right_title,
        right_border_color,
        app.user_data.config.ui.rounded_borders,
    );

    fn article_has_spoken_audio(app: &App, title: &str) -> bool {
        title.starts_with("Spoken:")
        || app.audio_player.current_title.as_deref() == Some(title)
        || app.workspace.tabs.iter().flat_map(|t| &t.panes).any(|p| {
            matches!(&p.content, crate::app::PaneContent::ArticleText { title: t, parsed_doc, .. } if t.eq_ignore_ascii_case(title) && parsed_doc.spoken_audio.is_some())
        })
        || crate::api::article::get_cached_article(title, 24 * 365)
            .is_some_and(|c| c.contains("spoken-wikipedia") || c.contains("Spoken Wikipedia"))
    }

    let mut article_lines = Vec::new();
    let right_total = selected_list.map(|l| l.articles.len()).unwrap_or(0);
    if let Some(list) = selected_list {
        if list.articles.is_empty() {
            article_lines.push(Line::from(Span::styled(
                " no articles saved in this list.",
                Style::default().fg(theme::GREY).italic(),
            )));
        } else {
            for (idx, article) in list.articles.iter().enumerate() {
                let is_selected = idx == app.modals.lists_modal.viewer_article_idx;
                let is_active = app.modals.lists_modal.viewer_focus_right;
                let has_audio = article_has_spoken_audio(app, article);
                let suffix = if has_audio {
                    if app.user_data.config.ui.icons {
                        Some(" 󰎆")
                    } else {
                        Some(" ♪")
                    }
                } else {
                    None
                };

                article_lines.push(create_selectable_line(
                    article,
                    is_selected,
                    is_active,
                    theme::VIOLET,
                    suffix,
                ));
            }
        }
    }

    render_selectable_list_column(
        f,
        right_area,
        right_block,
        article_lines,
        app.modals.lists_modal.viewer_article_idx,
        right_total,
    );
}
