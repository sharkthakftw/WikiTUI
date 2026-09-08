use crate::app::App;
use crate::theme;
use crate::ui::modals::utils::{centered_rect, create_checkbox_line, render_modal_frame_at};
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_save_to_list_modal_area(size: Rect) -> Rect {
    centered_rect(55, 60, size)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveToListHit {
    Toggle(usize),
    CreateNew,
}

pub fn get_save_to_list_item_at(app: &App, area: Rect, target_y: u16) -> Option<SaveToListHit> {
    if target_y <= area.y || target_y >= area.y + area.height.saturating_sub(1) {
        return None;
    }
    let inner_y = area.y + 1;
    let custom_lists_count = app
        .user_data
        .saved_lists
        .lists
        .iter()
        .filter(|l| l.id != "liked")
        .count();
    let start_row = inner_y + 4;
    if target_y >= start_row && target_y < start_row + (custom_lists_count as u16) {
        Some(SaveToListHit::Toggle((target_y - start_row) as usize))
    } else if target_y == start_row + (custom_lists_count as u16) + 1 {
        Some(SaveToListHit::CreateNew)
    } else {
        None
    }
}

pub fn render_save_to_list_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.user_data.config.ui.icons { "★" } else { "" };
    let area = compute_save_to_list_modal_area(size);
    let block = render_modal_frame_at(
        f,
        area,
        icon,
        "save to list",
        theme::VIOLET,
        app.user_data.config.ui.rounded_borders,
    );

    let mut lines = vec![
        Line::from(vec![
            Span::styled(" article: ", Style::default().fg(theme::GREY)),
            Span::styled(
                &app.modals.lists_modal.target_title,
                Style::default().fg(theme::YELLOW).bold(),
            ),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            " select custom lists to save this article into:",
            Style::default().fg(theme::FG).italic(),
        )),
        Line::from(""),
    ];

    let custom_lists: Vec<_> = app
        .user_data
        .saved_lists
        .lists
        .iter()
        .filter(|l| l.id != "liked")
        .collect();
    let list_count = custom_lists.len();
    for (idx, list) in custom_lists.iter().enumerate() {
        let is_focused = idx == app.modals.lists_modal.save_cursor_idx;
        let is_in_list = app
            .user_data
            .saved_lists
            .is_article_in_list(&list.id, &app.modals.lists_modal.target_title);
        let suffix = format!(" ({} articles)", list.articles.len());

        lines.push(create_checkbox_line(
            &list.name,
            is_focused,
            is_in_list,
            Some(&suffix),
            theme::VIOLET,
        ));
    }

    let is_create_focused = app.modals.lists_modal.save_cursor_idx == list_count;
    let create_cursor = if is_create_focused { " ▶ " } else { "   " };
    let create_style = if is_create_focused {
        Style::default().fg(theme::YELLOW).bold()
    } else {
        Style::default().fg(theme::BLUE)
    };
    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled(create_cursor, Style::default().fg(theme::VIOLET).bold()),
        Span::styled("[+] create new list...", create_style),
    ]));

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}
