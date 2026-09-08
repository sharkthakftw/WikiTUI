use super::utils::{centered_rect, render_modal_container_at};
use crate::app::{App, SettingItem};
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

pub fn compute_settings_modal_area(size: Rect) -> Rect {
    centered_rect(55, 80, size)
}

pub fn render_settings_modal(f: &mut Frame, app: &App, size: Rect) {
    let icon = if app.user_data.config.ui.icons { "󰒓" } else { "" };
    let area = compute_settings_modal_area(size);
    let inner = render_modal_container_at(
        f,
        area,
        icon,
        "settings (config.toml)",
        theme::ORANGE,
        app.user_data.config.ui.rounded_borders,
    );

    let mut lines: Vec<Line> = Vec::new();
    let mut current_section = "";

    for (idx, item) in SettingItem::ALL.iter().enumerate() {
        let section = item.section();
        if section != current_section {
            if !current_section.is_empty() {
                lines.push(Line::from(""));
            }
            lines.push(Line::from(vec![
                Span::styled(" [", Style::default().fg(theme::GREY)),
                Span::styled(section, Style::default().fg(theme::ORANGE).bold()),
                Span::styled("]", Style::default().fg(theme::GREY)),
            ]));
            current_section = section;
        }

        let is_focused = idx == app.modals.settings_modal.cursor_idx;
        let prefix = if is_focused { " ▶ " } else { "   " };
        let prefix_style = if is_focused {
            Style::default().fg(theme::YELLOW).bold()
        } else {
            Style::default().fg(theme::GREY)
        };

        let label = item.label();
        let label_style = if is_focused {
            Style::default().fg(theme::YELLOW).bold()
        } else {
            Style::default().fg(theme::FG)
        };

        let value_span = match item {
            SettingItem::LikedReadonly => {
                let val = app.user_data.config.general.liked_readonly;
                bool_span(val)
            }
            SettingItem::AutoRestoreSession => {
                let val = app.user_data.config.general.auto_restore_session;
                bool_span(val)
            }
            SettingItem::ConfirmQuit => {
                let val = app.user_data.config.general.confirm_quit;
                bool_span(val)
            }
            SettingItem::HintMode => {
                let val = app.user_data.config.general.hint_mode;
                let text = match val {
                    crate::config::HintMode::Semantic => "semantic",
                    crate::config::HintMode::Numbered => "numbered",
                    crate::config::HintMode::None => "none",
                };
                Span::styled(
                    format!("◄  {:>8}  ►", text),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
            SettingItem::RoundedBorders => {
                let val = app.user_data.config.ui.rounded_borders;
                bool_span(val)
            }
            SettingItem::Icons => {
                let val = app.user_data.config.ui.icons;
                bool_span(val)
            }
            SettingItem::ScrollIndicator => {
                let val = app.user_data.config.ui.scroll_indicator;
                bool_span(val)
            }
            SettingItem::Stats => {
                let val = app.user_data.config.ui.stats;
                bool_span(val)
            }
            SettingItem::DimInactivePanes => {
                let val = app.user_data.config.ui.dim_inactive_panes;
                bool_span(val)
            }
            SettingItem::HeadingMarker => {
                let val = app.user_data.config.reader.heading_marker;
                bool_span(val)
            }
            SettingItem::ScrollLines => {
                let val = app.user_data.config.reader.scroll_lines;
                Span::styled(
                    format!("◄  {:>2} lines  ►", val),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
            SettingItem::UnderlineLinks => {
                let val = app.user_data.config.reader.underline_links;
                bool_span(val)
            }
            SettingItem::ShowFootnotes => {
                let val = app.user_data.config.reader.show_footnotes;
                bool_span(val)
            }
            SettingItem::ShowExternalLinks => {
                let val = app.user_data.config.reader.show_external_links;
                bool_span(val)
            }
            SettingItem::TocSectionNumbers => {
                let val = app.user_data.config.reader.toc_section_numbers;
                bool_span(val)
            }
            SettingItem::CodeLineNumbers => {
                let val = app.user_data.config.reader.code_line_numbers;
                bool_span(val)
            }
            SettingItem::ShowImages => {
                let val = app.user_data.config.reader.show_images;
                bool_span(val)
            }
            SettingItem::ImageProtocol => {
                let val = app.user_data.config.reader.image_protocol;
                let text = match val {
                    crate::config::ImageProtocol::Auto => "auto",
                    crate::config::ImageProtocol::Kitty => "kitty",
                    crate::config::ImageProtocol::Halfblocks => "halfblocks",
                    crate::config::ImageProtocol::Off => "off",
                };
                Span::styled(
                    format!("◄  {:>10}  ►", text),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
            SettingItem::HalfblockFilter => {
                let val = app.user_data.config.reader.halfblock_filter;
                let text = match val {
                    crate::config::HalfblockFilter::Nearest => "nearest",
                    crate::config::HalfblockFilter::Triangle => "triangle",
                    crate::config::HalfblockFilter::Catmullrom => "catmullrom",
                    crate::config::HalfblockFilter::Gaussian => "gaussian",
                    crate::config::HalfblockFilter::Lanczos3 => "lanczos3",
                };
                Span::styled(
                    format!("◄  {:>10}  ►", text),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
            SettingItem::SearchLimit => {
                let val = app.user_data.config.search.limit;
                Span::styled(
                    format!("◄  {:>2} items  ►", val),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
            SettingItem::NetworkTimeout => {
                let val = app.user_data.config.network.timeout;
                Span::styled(
                    format!("◄  {:>2}s  ►", val),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
            SettingItem::OfflineCache => {
                let val = app.user_data.config.network.offline_cache;
                bool_span(val)
            }
            SettingItem::CacheLifetime => {
                let val = app.user_data.config.network.cache_lifetime;
                Span::styled(
                    format!("◄  {:>3}h  ►", val),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
            SettingItem::MouseSupport => {
                let val = app.user_data.config.input.mouse_support;
                bool_span(val)
            }
            SettingItem::ScrollSpeed => {
                let val = app.user_data.config.input.scroll_speed;
                Span::styled(
                    format!("◄  {:>2} lines  ►", val),
                    Style::default().fg(theme::TEAL).bold(),
                )
            }
        };

        let pad_len = 34_usize.saturating_sub(label.len());
        let dots = ".".repeat(pad_len.max(2));

        lines.push(Line::from(vec![
            Span::styled(prefix, prefix_style),
            Span::styled(label, label_style),
            Span::styled(format!(" {} ", dots), Style::default().fg(theme::DARK_GREY)),
            value_span,
        ]));
    }

    lines.push(Line::from(""));
    let focused_item = SettingItem::ALL.get(app.modals.settings_modal.cursor_idx).copied();
    if let Some(item) = focused_item {
        lines.push(Line::from(vec![
            Span::styled("   ", Style::default()),
            Span::styled(
                item.description(),
                Style::default().fg(theme::BEIGE).italic(),
            ),
        ]));
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            "space/enter: toggle  │  h/l: adjust  │  r: reset",
            Style::default().fg(theme::GREY),
        ),
    ]));

    let scroll = compute_settings_scroll(app.modals.settings_modal.cursor_idx, inner.height as usize);
    let paragraph = Paragraph::new(lines).scroll((scroll as u16, 0));
    f.render_widget(paragraph, inner);
}

fn bool_span(val: bool) -> Span<'static> {
    if val {
        Span::styled("[ ON ] ", Style::default().fg(theme::LIME).bold())
    } else {
        Span::styled("[ OFF ]", Style::default().fg(theme::GREY))
    }
}

pub fn get_item_line_index(item_idx: usize) -> usize {
    let mut cur_line = 0;
    let mut current_section = "";
    for (idx, item) in SettingItem::ALL.iter().enumerate() {
        let section = item.section();
        if section != current_section {
            if !current_section.is_empty() {
                cur_line += 1;
            }
            cur_line += 1;
            current_section = section;
        }
        if idx == item_idx {
            return cur_line;
        }
        cur_line += 1;
    }
    cur_line
}

pub fn compute_settings_total_lines() -> usize {
    let mut cur_line = 0;
    let mut current_section = "";
    for item in SettingItem::ALL.iter() {
        let section = item.section();
        if section != current_section {
            if !current_section.is_empty() {
                cur_line += 1;
            }
            cur_line += 1;
            current_section = section;
        }
        cur_line += 1;
    }
    cur_line + 4
}

pub fn compute_settings_scroll(cursor_idx: usize, inner_height: usize) -> usize {
    let total_lines = compute_settings_total_lines();
    let target_line = get_item_line_index(cursor_idx);
    crate::ui::modals::utils::compute_centered_scroll(target_line, inner_height, total_lines)
}

pub fn get_setting_row_at(
    inner: Rect,
    target_y: u16,
    cursor_idx: usize,
) -> Option<(usize, SettingItem, u16)> {
    if target_y < inner.y || target_y >= inner.y + inner.height {
        return None;
    }
    let scroll = compute_settings_scroll(cursor_idx, inner.height as usize);
    let row_in_inner = (target_y - inner.y) as usize + scroll;
    let mut cur_line = 0;
    let mut current_section = "";

    for (idx, item) in SettingItem::ALL.iter().enumerate() {
        let section = item.section();
        if section != current_section {
            if !current_section.is_empty() {
                cur_line += 1;
            }
            cur_line += 1;
            current_section = section;
        }
        if cur_line == row_in_inner {
            let value_start_x = inner.x + 36;
            return Some((idx, *item, value_start_x));
        }
        cur_line += 1;
    }
    None
}
