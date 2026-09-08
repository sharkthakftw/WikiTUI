use super::utils::{centered_rect, create_checkbox_line, render_modal_frame_at};
use crate::app::App;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_onboarding_modal_area(size: Rect) -> Rect {
    centered_rect(60, 80, size)
}

pub fn render_category_onboarding_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.user_data.config.ui.icons { "󰠱" } else { "" };
    let area = compute_onboarding_modal_area(size);
    let block = render_modal_frame_at(
        f,
        area,
        icon,
        "welcome to wikid feed",
        theme::VIOLET,
        app.user_data.config.ui.rounded_borders,
    );

    let mut lines = vec![
        Line::from(Span::styled(
            " pick some categories to get started (optional)",
            Style::default().fg(theme::FG).italic(),
        )),
        Line::from(""),
    ];

    for (idx, (display_name, _, _)) in crate::feed::profile::POPULAR_CATEGORIES.iter().enumerate() {
        let is_focused = idx == app.modals.onboarding.cursor_idx;
        let is_checked = app.modals.onboarding.selected.get(idx).copied().unwrap_or(false);

        lines.push(create_checkbox_line(
            display_name,
            is_focused,
            is_checked,
            None,
            theme::VIOLET,
        ));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![Span::styled(
        " j/k: navigate | space: toggle | enter: start feed",
        Style::default().fg(theme::GREY).italic(),
    )]));

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, area);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnboardingHit {
    Category(usize),
    Submit,
}

pub fn get_onboarding_row_at(area: Rect, target_y: u16) -> Option<OnboardingHit> {
    if target_y <= area.y || target_y >= area.y + area.height.saturating_sub(1) {
        return None;
    }
    let inner_y = area.y + 1;
    let cats_count = crate::feed::profile::POPULAR_CATEGORIES.len();
    let start_cat_row = inner_y + 2;
    if target_y >= start_cat_row && target_y < start_cat_row + (cats_count as u16) {
        Some(OnboardingHit::Category((target_y - start_cat_row) as usize))
    } else if target_y == start_cat_row + (cats_count as u16) + 1 {
        Some(OnboardingHit::Submit)
    } else {
        None
    }
}
