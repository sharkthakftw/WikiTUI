use super::utils::{
    compute_two_column_modal_areas, create_modal_block, create_selectable_line,
    render_selectable_list_column,
};
use crate::app::{App, PaneContent};
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    Frame,
};

pub fn compute_categories_modal_areas(size: Rect) -> (Rect, Rect, Rect) {
    compute_two_column_modal_areas(80, 75, 35, size)
}

pub fn render_categories_modal(f: &mut Frame, app: &App, size: Rect) {
    let pane = app.active_pane();
    let (title, categories) = match &pane.content {
        PaneContent::ArticleText {
            title, parsed_doc, ..
        } => (title.as_str(), &parsed_doc.categories),
        _ => return,
    };

    let total = categories.len();
    let modal_title = format!("categories · {} ({})", title.to_lowercase(), total);
    let icon = if app.user_data.config.ui.icons {
        "󰓹"
    } else {
        ""
    };

    let (container_area, left_area, right_area) = compute_categories_modal_areas(size);
    f.render_widget(ratatui::widgets::Clear, container_area);
    let block = create_modal_block(
        icon,
        &modal_title,
        theme::TEAL,
        app.user_data.config.ui.rounded_borders,
    );
    f.render_widget(block, container_area);

    let left_border_color = if !app.modals.categories_modal.focus_right {
        theme::TEAL
    } else {
        theme::GREY
    };
    let left_block = create_modal_block(
        "",
        "categories",
        left_border_color,
        app.user_data.config.ui.rounded_borders,
    );

    let selected_cat_idx = app
        .modals
        .categories_modal
        .cursor_idx
        .min(total.saturating_sub(1));

    let mut cat_lines = Vec::new();
    if categories.is_empty() {
        cat_lines.push(Line::from(vec![Span::styled(
            "  no categories found for this article.",
            Style::default().fg(theme::GREY).italic(),
        )]));
    } else {
        for (idx, cat) in categories.iter().enumerate() {
            let is_selected = idx == selected_cat_idx;
            cat_lines.push(create_selectable_line(
                cat,
                is_selected,
                !app.modals.categories_modal.focus_right,
                theme::TEAL,
                None,
            ));
        }
    }

    render_selectable_list_column(f, left_area, left_block, cat_lines, selected_cat_idx, total);

    let selected_category = categories
        .get(selected_cat_idx)
        .map(|s| s.as_str())
        .unwrap_or("");
    let right_title = if selected_category.is_empty() {
        "articles".to_string()
    } else {
        format!("articles in {}", selected_category.to_lowercase())
    };
    let right_border_color = if app.modals.categories_modal.focus_right {
        theme::TEAL
    } else {
        theme::GREY
    };
    let right_block = create_modal_block(
        "",
        &right_title,
        right_border_color,
        app.user_data.config.ui.rounded_borders,
    );

    let mut article_lines = Vec::new();
    let mut selected_art_idx = 0;
    let mut total_members = 0;

    if categories.is_empty() {
        article_lines.push(Line::from(vec![Span::styled(
            "  no category selected.",
            Style::default().fg(theme::GREY).italic(),
        )]));
    } else if app
        .modals
        .categories_modal
        .fetching_categories
        .contains(selected_category)
    {
        article_lines.push(Line::from(vec![Span::styled(
            "  fetching articles in category...",
            Style::default().fg(theme::YELLOW).italic(),
        )]));
    } else if let Some(members) = app
        .modals
        .categories_modal
        .cached_members
        .get(selected_category)
    {
        if members.is_empty() {
            article_lines.push(Line::from(vec![Span::styled(
                "  no articles found in this category.",
                Style::default().fg(theme::GREY).italic(),
            )]));
        } else {
            total_members = members.len();
            selected_art_idx = app
                .modals
                .categories_modal
                .article_cursor_idx
                .min(total_members.saturating_sub(1));

            for (idx, member) in members.iter().enumerate() {
                let is_selected = idx == selected_art_idx;
                article_lines.push(create_selectable_line(
                    member,
                    is_selected,
                    app.modals.categories_modal.focus_right,
                    theme::TEAL,
                    None,
                ));
            }
        }
    } else {
        article_lines.push(Line::from(vec![Span::styled(
            "  loading articles...",
            Style::default().fg(theme::GREY).italic(),
        )]));
    }

    render_selectable_list_column(
        f,
        right_area,
        right_block,
        article_lines,
        selected_art_idx,
        total_members,
    );
}

pub fn get_category_item_at(app: &App, is_right: bool, area: Rect, target_y: u16) -> Option<usize> {
    if target_y <= area.y || target_y >= area.y + area.height.saturating_sub(1) {
        return None;
    }
    let row_offset = (target_y - (area.y + 1)) as usize;
    let visible_rows = (area.height.saturating_sub(2)) as usize;
    if is_right {
        let pane = app.active_pane();
        let categories = match &pane.content {
            PaneContent::ArticleText { parsed_doc, .. } => &parsed_doc.categories,
            _ => return None,
        };
        let selected_cat_idx = app
            .modals
            .categories_modal
            .cursor_idx
            .min(categories.len().saturating_sub(1));
        let selected_category = categories.get(selected_cat_idx)?;
        let members = app
            .modals
            .categories_modal
            .cached_members
            .get(selected_category)?;
        let total = members.len();
        let scroll = crate::ui::modals::utils::compute_centered_scroll(
            app.modals.categories_modal.article_cursor_idx,
            visible_rows,
            total,
        );
        let idx = scroll + row_offset;
        if idx < total {
            Some(idx)
        } else {
            None
        }
    } else {
        let pane = app.active_pane();
        let total = match &pane.content {
            PaneContent::ArticleText { parsed_doc, .. } => parsed_doc.categories.len(),
            _ => 0,
        };
        let scroll = crate::ui::modals::utils::compute_centered_scroll(
            app.modals.categories_modal.cursor_idx,
            visible_rows,
            total,
        );
        let idx = scroll + row_offset;
        if idx < total {
            Some(idx)
        } else {
            None
        }
    }
}
