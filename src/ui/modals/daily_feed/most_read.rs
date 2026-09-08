use super::types::FeedEntry;
use crate::app::App;
use crate::theme;
use crate::ui::modals::utils::create_selectable_line;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub fn render_most_read_modal(
    f: &mut Frame,
    app: &App,
    entries: &[FeedEntry],
    modal_area: Rect,
    modal_block: Block,
    selected_idx: usize,
) {
    let total = entries.len();
    let inner_height = modal_area.height.saturating_sub(2) as usize;
    let scroll = app
        .modals
        .daily_feed_modal
        .as_ref()
        .map(|m| m.scroll)
        .unwrap_or(0)
        .min(total.saturating_sub(inner_height));

    let mut lines = Vec::new();
    if entries.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  no entries found.",
            Style::default().fg(theme::GREY).italic(),
        )]));
    } else {
        let avail_w = (modal_area.width as usize).saturating_sub(8);
        for (idx, entry) in entries.iter().enumerate() {
            let is_selected = idx == selected_idx;
            let title = crate::ui::truncate_to_width(&entry.title, avail_w);

            lines.push(create_selectable_line(
                &title,
                is_selected,
                true,
                theme::BLUE,
                entry.suffix.as_deref(),
            ));
        }
    }

    let p = Paragraph::new(lines)
        .block(modal_block)
        .scroll((scroll as u16, 0));

    f.render_widget(p, modal_area);
}
