use crate::app::{App, InputMode, PaneContent};
use crate::theme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;

use super::audio::build_audio_progress_bar;
use super::history::build_history_trail;

pub fn get_mode_badge(app: &App) -> (&'static str, Color) {
    if app.feed.active {
        (" FEED ", theme::PINK)
    } else {
        match app.input_mode {
            InputMode::Normal => (" NORMAL ", theme::LIME),
            InputMode::Search => (" SEARCH ", theme::BEIGE),
            InputMode::LocalSearch => (" FIND ", theme::YELLOW),
            InputMode::CategoryOnboarding => (" SETUP ", theme::VIOLET),
            InputMode::SaveToList
            | InputMode::SavedListsViewer
            | InputMode::CreateNewList
            | InputMode::RenameList => (" LISTS ", theme::VIOLET),
            InputMode::Settings => (" CONFIG ", theme::ORANGE),
            InputMode::Help => (" HELP ", theme::GREY),
            InputMode::Confirm => (" PROMPT ", theme::RED),
            InputMode::Categories => (" CATEGORIES ", theme::TEAL),
            InputMode::DailyFeedModal => (" DAILY ", theme::TEAL),
            InputMode::CommandPalette => (" COMMAND ", theme::YELLOW),
            InputMode::QrModal => (" QR CODE ", theme::PINK),
            InputMode::ImageModal => (" IMAGE ", theme::PINK),
            InputMode::LinkPeek => (" PEEK ", theme::PINK),
        }
    }
}

pub fn get_center_spans(
    app: &App,
    active_pane: &crate::app::Pane,
    available_width: usize,
) -> Vec<Span<'static>> {
    match app.input_mode {
        InputMode::Search => vec![Span::styled(
            "type query · enter search · esc cancel",
            Style::default()
                .fg(theme::BEIGE)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::LocalSearch => {
            let matches_info = if active_pane.search.matches.is_empty() {
                "no matches".to_string()
            } else {
                format!(
                    "match {}/{}",
                    active_pane.search.selected_match_idx.unwrap_or(0) + 1,
                    active_pane.search.matches.len()
                )
            };
            vec![Span::styled(
                format!(
                    "/: {}_ · {} · n next · N prev · esc exit",
                    active_pane.search.query, matches_info
                ),
                Style::default()
                    .fg(theme::YELLOW)
                    .add_modifier(Modifier::BOLD),
            )]
        }
        InputMode::CategoryOnboarding => vec![Span::styled(
            "j/k navigate · space toggle · enter start feed",
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::Help => vec![Span::styled(
            "esc/q/? close",
            Style::default()
                .fg(theme::PINK)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::SaveToList => vec![Span::styled(
            "j/k navigate · space toggle · c new list · esc done",
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::CreateNewList | InputMode::RenameList => vec![Span::styled(
            "enter confirm · esc cancel",
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::SavedListsViewer => vec![Span::styled(
            "h/l pane · j/k navigate · enter open · r rename · d delete · esc close",
            Style::default()
                .fg(theme::VIOLET)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::Confirm => vec![Span::styled(
            "y/enter confirm · n/esc cancel",
            Style::default().fg(theme::RED).add_modifier(Modifier::BOLD),
        )],
        InputMode::Settings => vec![Span::styled(
            "j/k navigate · space/enter toggle · h/l adjust · r reset · esc close",
            Style::default()
                .fg(theme::ORANGE)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::Categories => vec![Span::styled(
            "j/k navigate · enter open category · y copy URL · esc close",
            Style::default()
                .fg(theme::TEAL)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::DailyFeedModal => {
            if let Some(modal) = &app.daily_feed_modal {
                if modal.kind == crate::ui::modals::DailyFeedKind::OnThisDay {
                    vec![Span::styled(
                        "1-4 category · j/k navigate · tab links · enter read · esc close",
                        Style::default()
                            .fg(theme::BLUE)
                            .add_modifier(Modifier::BOLD),
                    )]
                } else if modal.kind == crate::ui::modals::DailyFeedKind::News {
                    vec![Span::styled(
                        "j/k navigate · tab links · enter read · esc close",
                        Style::default()
                            .fg(theme::BLUE)
                            .add_modifier(Modifier::BOLD),
                    )]
                } else {
                    vec![Span::styled(
                        "j/k navigate · enter read · esc close",
                        Style::default()
                            .fg(theme::BLUE)
                            .add_modifier(Modifier::BOLD),
                    )]
                }
            } else {
                vec![Span::styled(
                    "j/k navigate · enter read · esc close",
                    Style::default()
                        .fg(theme::BLUE)
                        .add_modifier(Modifier::BOLD),
                )]
            }
        }
        InputMode::CommandPalette => vec![Span::styled(
            "type to search · up/down navigate · enter run · esc close",
            Style::default()
                .fg(theme::YELLOW)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::QrModal => vec![Span::styled(
            "esc/q close · y copy link",
            Style::default()
                .fg(theme::PINK)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::ImageModal => vec![Span::styled(
            "esc/q close · y copy url",
            Style::default()
                .fg(theme::PINK)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::LinkPeek => vec![Span::styled(
            "enter open · esc/p close",
            Style::default()
                .fg(theme::PINK)
                .add_modifier(Modifier::BOLD),
        )],
        InputMode::Normal => {
            if app.audio_player.is_active() {
                build_audio_progress_bar(app, available_width)
            } else if app.feed.active {
                vec![Span::styled(
                    "j/k browse · l like · enter read · t tab · r reset · esc exit",
                    Style::default().fg(theme::GREY),
                )]
            } else if active_pane.show_toc || active_pane.toc_focused {
                vec![Span::styled(
                    "j/k navigate contents · enter jump · esc/o close",
                    Style::default()
                        .fg(theme::LIME)
                        .add_modifier(Modifier::BOLD),
                )]
            } else if let Some(link) = active_pane.focused_link().filter(|l| l.is_external()) {
                vec![Span::styled(
                    format!("↗ external {} · enter/y copy URL", link.title),
                    Style::default()
                        .fg(theme::TEAL)
                        .add_modifier(Modifier::BOLD),
                )]
            } else if matches!(active_pane.content, PaneContent::ArticleText { .. })
                && (!active_pane.history_back.is_empty() || !active_pane.history_forward.is_empty())
            {
                build_history_trail(active_pane, available_width)
            } else if matches!(active_pane.content, PaneContent::Empty) {
                vec![Span::styled(
                    "ctrl-s search · F feed · , settings · ? help · q quit",
                    Style::default().fg(theme::GREY),
                )]
            } else {
                let has_spoken =
                    if let PaneContent::ArticleText { parsed_doc, .. } = &active_pane.content {
                        parsed_doc.spoken_audio.is_some()
                    } else {
                        false
                    };
                if has_spoken {
                    vec![Span::styled(
                        "ctrl-s search · a listen · r random · F feed · , settings · ? help",
                        Style::default().fg(theme::GREY),
                    )]
                } else {
                    vec![Span::styled(
                        "ctrl-s search · r random · F feed · , settings · ? help · q quit",
                        Style::default().fg(theme::GREY),
                    )]
                }
            }
        }
    }
}
