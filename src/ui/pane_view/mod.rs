pub mod article;
pub mod scrollbar;
pub mod search;

pub use article::{get_image_at_coord, get_link_at_coord};
pub use search::{
    compute_search_result_lines_count, count_wrapped_lines, get_search_result_at_line, wrap_text,
};

use crate::app::{App, PaneContent};
use crate::theme;
use ratatui::{
    layout::{Alignment, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
    Frame,
};

pub fn render_single_active_pane(f: &mut Frame, app: &mut App, area: Rect) {
    let active_tab_idx = app.workspace.active_tab_idx;
    let active_pane_idx = app.workspace.tabs[active_tab_idx].active_pane_idx;
    render_pane_at(f, app, active_tab_idx, active_pane_idx, area, true);
}

pub fn render_panes(f: &mut Frame, app: &mut App, main_area: Rect) {
    let active_tab_idx = app.workspace.active_tab_idx;
    let rects = app.workspace.tabs[active_tab_idx]
        .layout_root
        .compute_rects(main_area);
    let active_pane_idx = app.workspace.tabs[active_tab_idx].active_pane_idx;

    for (pane_idx, rect) in rects {
        let is_active = pane_idx == active_pane_idx;
        render_pane_at(f, app, active_tab_idx, pane_idx, rect, is_active);
    }
}

fn render_pane_at(
    f: &mut Frame,
    app: &mut App,
    tab_idx: usize,
    pane_idx: usize,
    rect: Rect,
    is_active: bool,
) {
    let content_width = if app.workspace.zen_mode {
        rect.width.saturating_sub(2) as usize
    } else {
        rect.width.saturating_sub(4) as usize
    };

    let render_opts = crate::app::pane::ArticleRenderOptions {
        width: content_width,
        show_footnotes: app.user_data.config.reader.show_footnotes,
        show_external_links: app.user_data.config.reader.show_external_links,
        heading_marker: app.user_data.config.reader.heading_marker,
        code_line_numbers: app.user_data.config.reader.code_line_numbers,
        show_icons: app.user_data.config.ui.icons,
        show_images: app.user_data.config.reader.show_images,
        max_image_height: app.user_data.config.reader.max_image_height,
    };
    let is_image_modal = app.input_mode == crate::app::InputMode::ImageModal;
    let has_multiple_panes = app.workspace.tabs[tab_idx].panes.len() > 1;
    let should_dim =
        (app.user_data.config.ui.dim_inactive_panes && !is_active && has_multiple_panes) || is_image_modal;

    let pane = &mut app.workspace.tabs[tab_idx].panes[pane_idx];
    pane.viewport_width = content_width;
    pane.ensure_parsed_width(render_opts);
    pane.viewport_height = if app.workspace.zen_mode {
        rect.height as usize
    } else {
        rect.height.saturating_sub(2) as usize
    };

    let border_color = if is_image_modal {
        theme::DARK_GREY
    } else {
        match &pane.content {
            PaneContent::SearchResults { .. } => {
                if is_active {
                    theme::YELLOW
                } else {
                    theme::DARK_GREY
                }
            }
            _ => {
                if is_active {
                    theme::PINK
                } else {
                    theme::DARK_GREY
                }
            }
        }
    };

    let title = match &pane.content {
        PaneContent::Empty => String::new(),
        PaneContent::SearchResults { query, .. } => {
            format!(" search: {} ", query.to_lowercase())
        }
        PaneContent::ArticleText { title, .. } => {
            format!(" {} ", title.to_lowercase())
        }
        PaneContent::Error(_) => " error ".to_string(),
    };

    let border_type = app.user_data.config.ui.border_type();

    let title_style = if should_dim {
        Style::default().fg(theme::DARK_GREY).dim()
    } else {
        Style::default().fg(border_color)
    };

    let border_style = if should_dim {
        Style::default().fg(theme::DARK_GREY).dim()
    } else {
        Style::default().fg(border_color)
    };

    let block = if app.workspace.zen_mode {
        Block::default().padding(Padding::horizontal(1))
    } else {
        Block::bordered()
            .border_type(border_type)
            .border_style(border_style)
            .title(Line::from(Span::styled(title, title_style)))
            .padding(Padding::horizontal(1))
    };

    if pane.is_loading {
        let vertical_offset = (rect.height.saturating_sub(2) / 2) as usize;
        let mut lines = Vec::new();
        for _ in 0..vertical_offset {
            lines.push(Line::from(""));
        }

        let spinner = crate::ui::current_spinner_frame();

        let mut loading_spans = vec![
            Span::styled(
                format!("{} ", spinner),
                Style::default().fg(theme::LIME).bold(),
            ),
            Span::styled(
                "loading wikipedia data...",
                Style::default().fg(theme::BEIGE).bold(),
            ),
        ];
        if should_dim {
            for span in &mut loading_spans {
                span.style = span.style.add_modifier(Modifier::DIM);
            }
        }
        lines.push(Line::from(loading_spans));
        let loading_p = Paragraph::new(lines)
            .block(block)
            .alignment(Alignment::Center);
        f.render_widget(loading_p, rect);
        return;
    }

    let is_search = matches!(pane.content, PaneContent::SearchResults { .. });
    let is_empty = matches!(pane.content, PaneContent::Empty);
    let is_error = matches!(pane.content, PaneContent::Error(_));

    if is_empty {
        crate::ui::launch_screen::render_launch_screen(f, app, rect, block);
    } else if is_search {
        let pane = &app.workspace.tabs[tab_idx].panes[pane_idx];
        if let PaneContent::SearchResults { items, .. } = &pane.content {
            search::render_search_pane(
                f,
                rect,
                block,
                pane,
                items,
                border_color,
                if is_image_modal { false } else { is_active },
                app.workspace.zen_mode,
                app.user_data.config.ui.scroll_indicator,
                app.user_data.config.ui.icons,
                should_dim,
            );
        }
    } else if is_error {
        let pane = &app.workspace.tabs[tab_idx].panes[pane_idx];
        if let PaneContent::Error(err_msg) = &pane.content {
            let vertical_offset = (rect.height.saturating_sub(2) / 2) as usize;
            let mut lines = Vec::new();
            for _ in 0..vertical_offset {
                lines.push(Line::from(""));
            }
            let err_style = if should_dim {
                Style::default()
                    .fg(theme::RED)
                    .bold()
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default().fg(theme::RED).bold()
            };
            lines.push(Line::from(Span::styled(
                format!("error: {}", err_msg),
                err_style,
            )));
            let err_p = Paragraph::new(lines)
                .block(block)
                .alignment(Alignment::Center);
            f.render_widget(err_p, rect);
        }
    } else {
        article::render_article_pane(
            f,
            app,
            tab_idx,
            pane_idx,
            rect,
            block,
            border_color,
            if is_image_modal { false } else { is_active },
        );
    }
}
