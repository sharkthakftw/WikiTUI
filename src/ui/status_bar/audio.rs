use crate::app::App;
use crate::theme;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;

pub fn format_audio_time(secs: u64, include_hours: bool) -> String {
    if include_hours {
        let h = secs / 3600;
        let m = (secs % 3600) / 60;
        let s = secs % 60;
        format!("{}:{:02}:{:02}", h, m, s)
    } else {
        let m = secs / 60;
        let s = secs % 60;
        format!("{}:{:02}", m, s)
    }
}

pub fn build_audio_progress_bar(app: &App, available_width: usize) -> Vec<Span<'static>> {
    let is_buffering = app.audio_player.is_buffering;
    let is_playing = app.audio_player.is_playing();
    let is_icons = app.user_data.config.ui.icons;

    let icon = if is_buffering {
        if is_icons {
            "󰑐 "
        } else {
            ""
        }
    } else if is_icons {
        if is_playing {
            "󰐊 "
        } else {
            "󰏤 "
        }
    } else if is_playing {
        "playing "
    } else {
        "paused "
    };

    let fill_bg = if is_buffering {
        theme::ORANGE
    } else if is_playing {
        theme::YELLOW
    } else {
        theme::BEIGE
    };

    let elapsed = app.audio_player.elapsed_secs;
    let total_opt = app.audio_player.total_duration_secs;
    let include_hours = total_opt.map_or(elapsed >= 3600, |t| t >= 3600);

    let elapsed_str = format_audio_time(elapsed, include_hours);
    let total_str = total_opt.map_or_else(
        || "--:--".to_string(),
        |t| format_audio_time(t, include_hours),
    );

    let label = format!("{} / {}", elapsed_str, total_str);
    let min_bar_width = label.chars().count() + 8;

    let hint = if is_buffering {
        "[buffering... · A stop]"
    } else if is_playing {
        "[< / > seek · a pause · A stop]"
    } else {
        "[< / > seek · a resume · A stop]"
    };

    let raw_title = app.audio_player.current_title.as_deref().unwrap_or("");
    let title_budget = if available_width > 80 {
        24
    } else if available_width > 60 {
        16
    } else if available_width > 45 {
        10
    } else {
        0
    };

    let display_title = if title_budget > 0 && !raw_title.is_empty() {
        if raw_title.chars().count() > title_budget {
            let truncated: String = raw_title
                .chars()
                .take(title_budget.saturating_sub(1))
                .collect();
            Some(format!("{}…", truncated))
        } else {
            Some(raw_title.to_string())
        }
    } else {
        None
    };

    let title_len = display_title.as_ref().map_or(0, |t| t.chars().count() + 2);

    let sleep_badge = if let Some(rem) = app.audio_player.remaining_sleep_timer_secs() {
        let time_str = if rem >= 3600 {
            format!("{}h{}m", rem / 3600, (rem % 3600).div_ceil(60))
        } else if rem >= 60 {
            format!("{}m", rem.div_ceil(60))
        } else {
            format!("{}s", rem)
        };
        if is_icons {
            Some(format!(" [󰔛 {}]", time_str))
        } else {
            Some(format!(" [zZ {}]", time_str))
        }
    } else {
        None
    };

    let raw_sleep_len = sleep_badge.as_ref().map_or(0, |s| s.chars().count());
    let show_sleep = sleep_badge.is_some()
        && available_width >= icon.chars().count() + min_bar_width + raw_sleep_len;
    let sleep_len = if show_sleep { raw_sleep_len } else { 0 };

    let show_hint = available_width
        > icon.chars().count() + title_len + min_bar_width + sleep_len + hint.len() + 4;
    let hint_len = if show_hint { hint.len() + 2 } else { 0 };

    let bar_width = available_width
        .saturating_sub(icon.chars().count() + title_len + sleep_len + hint_len + 2)
        .clamp(min_bar_width, 36);

    let pad_total = bar_width.saturating_sub(label.chars().count());
    let pad_left = pad_total / 2;
    let pad_right = pad_total.saturating_sub(pad_left);
    let bar_text = format!("{}{}{}", " ".repeat(pad_left), label, " ".repeat(pad_right));

    let ratio = if let Some(total) = total_opt {
        if total > 0 {
            (elapsed as f32 / total as f32).clamp(0.0, 1.0)
        } else {
            0.0
        }
    } else {
        ((elapsed as f32 % 10.0) / 10.0).clamp(0.0, 1.0)
    };

    let filled_count = ((bar_width as f32) * ratio).round() as usize;
    let filled_count = filled_count.min(bar_width);

    let (filled_str, unfilled_str) = bar_text.split_at(filled_count);

    let mut spans = Vec::new();

    spans.push(Span::styled(
        icon,
        Style::default().fg(fill_bg).add_modifier(Modifier::BOLD),
    ));

    if let Some(t) = display_title {
        spans.push(Span::styled(
            format!("{}  ", t),
            Style::default().fg(theme::FG).add_modifier(Modifier::BOLD),
        ));
    }

    if !filled_str.is_empty() {
        spans.push(Span::styled(
            filled_str.to_string(),
            Style::default()
                .fg(theme::BG)
                .bg(fill_bg)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if !unfilled_str.is_empty() {
        spans.push(Span::styled(
            unfilled_str.to_string(),
            Style::default()
                .fg(theme::FG)
                .bg(theme::LIGHT_BG)
                .add_modifier(Modifier::BOLD),
        ));
    }

    if show_sleep
        && let Some(sb) = sleep_badge {
            spans.push(Span::styled(
                sb,
                Style::default()
                    .fg(theme::VIOLET)
                    .add_modifier(Modifier::BOLD),
            ));
        }

    if show_hint {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(hint, Style::default().fg(theme::GREY)));
    }

    spans
}
