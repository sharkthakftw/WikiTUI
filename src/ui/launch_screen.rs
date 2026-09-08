use crate::app::App;
use crate::theme;
use ratatui::{
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph},
    Frame,
};

pub const LOGO_WIDTH: usize = 53;
pub const LOGO: &[&str] = &[
    r#"                                             "#,
    r#"             .--.     .     .--.\  ___ `'.   "#,
    r#"      _     _|__|   .'|     |__| ' |--.\  \  "#,
    r#"/\    \\   //.--. .'  |     .--. | |    \  ' "#,
    r#"`\\  //\\ // |  |<    |     |  | | |     |  '"#,
    r#"  \`//  \'/  |  | |   | ____|  | | |     |  |"#,
    r#"   \|   |/   |  | |   | \ .'|  | | |     ' .'"#,
    r#"    '        |  | |   |/  . |  | | |___.' /' "#,
    r#"             |__| |    /\  \|__|/_______.'/  "#,
    r#"                  |   |  \  \   \_______|/   "#,
    r#"                  '    \  \  \               "#,
    r#"                 '------'  '---'             "#,
];

pub const QUOTES: &[&str] = &[
    "wikipedia reader for the terminal",
    "it's wicked",
    "just one more before bed",
    "genuine autism",
    "press r",
];

fn center_spans<'a>(spans: Vec<Span<'a>>, inner_width: usize) -> Line<'a> {
    let width: usize = spans
        .iter()
        .map(|s| unicode_width::UnicodeWidthStr::width(s.content.as_ref()))
        .sum();
    let pad = (inner_width.saturating_sub(width)) / 2;
    let mut line = vec![Span::raw(" ".repeat(pad))];
    line.extend(spans);
    Line::from(line)
}

fn center_styled<'a>(text: &'a str, style: Style, inner_width: usize) -> Line<'a> {
    center_spans(vec![Span::styled(text, style)], inner_width)
}

pub fn render_launch_screen(f: &mut Frame, app: &App, rect: Rect, block: Block) {
    let inner_width = (rect.width as usize).saturating_sub(4);
    let inner_height = (rect.height as usize).saturating_sub(2);

    let left_pad = (inner_width.saturating_sub(LOGO_WIDTH)) / 2;
    let pad_str = " ".repeat(left_pad);

    let recent_articles = app.get_continue_reading_with_timestamps();
    let show_recent = !recent_articles.is_empty() && inner_height >= (LOGO.len() + 10);
    let displayed_count = if show_recent {
        recent_articles.len().min(7)
    } else {
        0
    };

    let show_stats = app.user_data.config.ui.stats;
    let mut lines = Vec::new();
    let total_content_height = LOGO.len()
        + 4
        + if show_stats { 2 } else { 0 }
        + if show_recent { displayed_count + 2 } else { 0 };
    let v_pad = inner_height.saturating_sub(total_content_height) / 2;

    for _ in 0..v_pad {
        lines.push(Line::from(""));
    }

    for &line in LOGO {
        lines.push(Line::from(vec![
            Span::raw(&pad_str),
            Span::styled(line, Style::default().fg(theme::PINK).bold()),
        ]));
    }

    lines.push(Line::from(""));
    let quote = QUOTES[app.launch_quote_idx % QUOTES.len()];
    let quote_len = quote.chars().count();
    if inner_width >= quote_len {
        lines.push(center_styled(
            quote,
            Style::default().fg(theme::GREY).italic(),
            inner_width,
        ));
    } else {
        lines.push(center_styled(
            "wikipedia reader for the terminal",
            Style::default().fg(theme::GREY).italic(),
            inner_width,
        ));
    }

    if show_stats {
        lines.push(Line::from(""));
        let show_icons = app.user_data.config.ui.icons;
        let icon_art = if show_icons { "󰈙 " } else { "" };
        let icon_edits = if show_icons { "󰑐 " } else { "" };
        let icon_users = if show_icons { "󰒓 " } else { "" };
        let icon_pages = if show_icons { "󰠱 " } else { "" };

        let stats_spans = if inner_width >= 75 {
            vec![
                Span::styled(
                    format!(
                        "{}{} articles",
                        icon_art,
                        crate::api::stats::format_metric(app.wiki_stats.articles)
                    ),
                    Style::default().fg(theme::LIME),
                ),
                Span::styled("   ·   ", Style::default().fg(theme::DARK_GREY)),
                Span::styled(
                    format!(
                        "{}{} edits",
                        icon_edits,
                        crate::api::stats::format_metric(app.wiki_stats.edits)
                    ),
                    Style::default().fg(theme::TEAL),
                ),
                Span::styled("   ·   ", Style::default().fg(theme::DARK_GREY)),
                Span::styled(
                    format!(
                        "{}{} active editors",
                        icon_users,
                        crate::api::stats::format_metric(app.wiki_stats.activeusers)
                    ),
                    Style::default().fg(theme::YELLOW),
                ),
                Span::styled("   ·   ", Style::default().fg(theme::DARK_GREY)),
                Span::styled(
                    format!(
                        "{}{} pages",
                        icon_pages,
                        crate::api::stats::format_metric(app.wiki_stats.pages)
                    ),
                    Style::default().fg(theme::VIOLET),
                ),
            ]
        } else {
            vec![
                Span::styled(
                    format!(
                        "{}{} articles",
                        icon_art,
                        crate::api::stats::format_metric(app.wiki_stats.articles)
                    ),
                    Style::default().fg(theme::LIME),
                ),
                Span::styled("   ·   ", Style::default().fg(theme::DARK_GREY)),
                Span::styled(
                    format!(
                        "{}{} active editors",
                        icon_users,
                        crate::api::stats::format_metric(app.wiki_stats.activeusers)
                    ),
                    Style::default().fg(theme::YELLOW),
                ),
            ]
        };

        lines.push(center_spans(stats_spans, inner_width));
    }
    lines.push(Line::from(""));

    let actions: Vec<(&str, &str)> = if inner_width >= 75 {
        vec![
            ("Featured", "f"),
            ("News", "n"),
            ("On This Day", "d"),
            ("Trending", "t"),
            ("Random", "r"),
        ]
    } else if inner_width >= 50 {
        vec![
            ("Featured", "f"),
            ("News", "n"),
            ("History", "d"),
            ("Top", "t"),
            ("Random", "r"),
        ]
    } else {
        vec![("Featured", "f"), ("News", "n"), ("History", "d")]
    };

    let mut action_spans = Vec::new();
    for (idx, (label, key)) in actions.iter().enumerate() {
        if idx > 0 {
            action_spans.push(Span::raw("    "));
        }
        action_spans.push(Span::styled(*label, Style::default().fg(theme::FG)));
        action_spans.push(Span::styled("[", Style::default().fg(theme::DARK_GREY)));
        action_spans.push(Span::styled(
            *key,
            Style::default().fg(theme::YELLOW).bold(),
        ));
        action_spans.push(Span::styled("]", Style::default().fg(theme::DARK_GREY)));
    }

    lines.push(center_spans(action_spans, inner_width));

    if show_recent {
        lines.push(Line::from(""));
        lines.push(center_styled(
            "── continue reading ──",
            Style::default().fg(theme::GREY),
            inner_width,
        ));

        let hint_mode = app.user_data.config.general.hint_mode;
        let hints_map = if hint_mode == crate::config::HintMode::Semantic {
            app.current_continue_reading_hints()
        } else {
            recent_articles
                .iter()
                .take(displayed_count)
                .map(|(t, _)| (t.clone(), None))
                .collect()
        };

        let items: Vec<(
            String,
            String,
            String,
            Option<crate::app::recent::SemanticHint>,
        )> = recent_articles
            .iter()
            .take(displayed_count)
            .enumerate()
            .zip(hints_map)
            .map(|((idx, (_, ts)), (title, hint))| {
                let num_prefix = if hint_mode == crate::config::HintMode::Numbered {
                    format!("{}. ", idx + 1)
                } else {
                    String::new()
                };
                let time_str = ts
                    .map(crate::app::recent::format_relative_time)
                    .unwrap_or_default();
                (num_prefix, title, time_str, hint)
            })
            .collect();

        let max_item_width = items
            .iter()
            .map(|(p, t, time_str, _)| {
                let base_w = p.len() + unicode_width::UnicodeWidthStr::width(t.as_str());
                if !time_str.is_empty() {
                    base_w + 3 + unicode_width::UnicodeWidthStr::width(time_str.as_str())
                } else {
                    base_w
                }
            })
            .max()
            .unwrap_or(0);

        let block_pad = (inner_width.saturating_sub(max_item_width)) / 2;

        for (num_prefix, title, time_str, hint) in items {
            let mut line_spans = vec![Span::raw(" ".repeat(block_pad))];

            if !num_prefix.is_empty() {
                line_spans.push(Span::styled(num_prefix, Style::default().fg(theme::GREY)));
            }

            if let Some(h) = hint {
                for (idx, ch) in title.char_indices() {
                    if idx == h.char_idx {
                        line_spans.push(Span::styled(
                            ch.to_string(),
                            Style::default().fg(theme::PINK).bold(),
                        ));
                    } else {
                        line_spans
                            .push(Span::styled(ch.to_string(), Style::default().fg(theme::FG)));
                    }
                }
            } else {
                line_spans.push(Span::styled(title, Style::default().fg(theme::FG)));
            }

            if !time_str.is_empty() {
                line_spans.push(Span::styled(
                    format!(" · {}", time_str),
                    Style::default().fg(theme::DARK_GREY),
                ));
            }
            lines.push(Line::from(line_spans));
        }
    }

    let p = Paragraph::new(lines).block(block);
    f.render_widget(p, rect);
}
