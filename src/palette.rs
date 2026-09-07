use crate::app::App;
use crate::layout::SplitDirection;

#[derive(Clone, Copy, Debug)]
pub struct CommandDef {
    pub id: &'static str,
    pub label: &'static str,
    pub shortcut: Option<&'static str>,
    pub execute: fn(&mut App),
}

pub const COMMANDS: &[CommandDef] = &[
    CommandDef {
        id: "toggle_zen_mode",
        label: "toggle zen mode",
        shortcut: Some("z"),
        execute: |app| app.toggle_zen_mode(),
    },
    CommandDef {
        id: "search_wikipedia",
        label: "search wikipedia",
        shortcut: Some("ctrl-s"),
        execute: |app| app.enter_search_mode(),
    },
    CommandDef {
        id: "random_article",
        label: "random article",
        shortcut: Some("r"),
        execute: |app| app.fetch_random_article(),
    },
    CommandDef {
        id: "table_of_contents",
        label: "table of contents",
        shortcut: Some("o"),
        execute: |app| app.toggle_toc(),
    },
    CommandDef {
        id: "toggle_feed_mode",
        label: "toggle feed mode",
        shortcut: Some("F"),
        execute: |app| app.toggle_feed_mode(),
    },
    CommandDef {
        id: "toggle_images",
        label: "toggle inline images",
        shortcut: Some("I"),
        execute: |app| app.toggle_images(),
    },
    CommandDef {
        id: "play_pause_audio",
        label: "play / pause audio",
        shortcut: Some("a"),
        execute: |app| app.toggle_spoken_audio(),
    },
    CommandDef {
        id: "stop_spoken_audio",
        label: "stop spoken audio",
        shortcut: Some("A"),
        execute: |app| app.stop_spoken_audio(),
    },
    CommandDef {
        id: "sleep_timer",
        label: "sleep timer",
        shortcut: None,
        execute: |app| app.open_sleep_timer_prompt(),
    },
    CommandDef {
        id: "save_to_list",
        label: "save to list",
        shortcut: Some("m"),
        execute: |app| app.open_save_to_list_modal(),
    },
    CommandDef {
        id: "saved_lists_viewer",
        label: "saved lists viewer",
        shortcut: Some("M"),
        execute: |app| app.open_saved_lists_viewer(),
    },
    CommandDef {
        id: "split_pane_vertically",
        label: "split pane vertically",
        shortcut: Some("ctrl-w v"),
        execute: |app| app.split_active_pane(SplitDirection::Vertical),
    },
    CommandDef {
        id: "split_pane_horizontally",
        label: "split pane horizontally",
        shortcut: Some("ctrl-w s"),
        execute: |app| app.split_active_pane(SplitDirection::Horizontal),
    },
    CommandDef {
        id: "close_active_pane",
        label: "close active pane",
        shortcut: Some("x"),
        execute: |app| app.close_active_pane(),
    },
    CommandDef {
        id: "new_tab",
        label: "new tab",
        shortcut: Some("alt-t"),
        execute: |app| app.new_tab(),
    },
    CommandDef {
        id: "alternate_tab",
        label: "visit previous tab",
        shortcut: Some("%"),
        execute: |app| app.toggle_alternate_tab(),
    },
    CommandDef {
        id: "settings",
        label: "settings",
        shortcut: Some(","),
        execute: |app| {
            app.input_mode = crate::app::InputMode::Settings;
        },
    },
    CommandDef {
        id: "copy_article_link",
        label: "copy article link",
        shortcut: Some("Y"),
        execute: |app| app.copy_article_link(),
    },
    CommandDef {
        id: "qr_code",
        label: "get qr code",
        shortcut: Some(""),
        execute: |app| app.open_qr_modal(),
    },
    CommandDef {
        id: "restore_session",
        label: "restore session",
        shortcut: Some("S"),
        execute: |app| {
            if let Some(session) = crate::session::SessionState::load() {
                session.restore_to_app(app);
            }
        },
    },
    CommandDef {
        id: "reopen_closed_tab",
        label: "reopen closed tab",
        shortcut: Some("u"),
        execute: |app| app.reopen_last_closed(),
    },
    CommandDef {
        id: "help",
        label: "help",
        shortcut: Some("?"),
        execute: |app| app.toggle_help_popup(),
    },
    CommandDef {
        id: "check_for_updates",
        label: "check for updates",
        shortcut: Some("U"),
        execute: |app| app.check_for_updates(),
    },
    CommandDef {
        id: "quit_wikid",
        label: "quit wikid",
        shortcut: Some("q"),
        execute: |app| app.quit(),
    },
];

pub fn filter_commands(query: &str) -> Vec<(&'static CommandDef, Vec<usize>)> {
    let trimmed = query.trim();
    let stripped = trimmed.strip_prefix(':').unwrap_or(trimmed);
    let q = stripped.trim().to_lowercase();
    if q.is_empty() {
        return COMMANDS.iter().map(|cmd| (cmd, Vec::new())).collect();
    }

    let mut matches = Vec::new();
    for cmd in COMMANDS {
        let label = cmd.label;
        if let Some(pos) = label.find(&q) {
            let match_indices = (pos..pos + q.len()).collect();
            matches.push((cmd, match_indices, 0));
        } else {
            let mut match_indices = Vec::new();
            let mut q_chars = q.chars().peekable();
            for (idx, ch) in label.char_indices() {
                if let Some(&qc) = q_chars.peek() {
                    if ch == qc {
                        match_indices.push(idx);
                        q_chars.next();
                    }
                }
            }
            if q_chars.peek().is_none() {
                matches.push((cmd, match_indices, 1));
            }
        }
    }

    matches.sort_by_key(|(_, _, score)| *score);
    matches
        .into_iter()
        .map(|(cmd, indices, _)| (cmd, indices))
        .collect()
}
