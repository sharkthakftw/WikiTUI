use crate::app::pane::PaneContent;
use crate::app::App;

impl App {
    pub fn toggle_spoken_audio(&mut self) {
        if self.audio_player.is_active() {
            self.audio_player.toggle_pause();
            return;
        }

        let track_info = match &self.active_pane().content {
            PaneContent::ArticleText {
                parsed_doc, title, ..
            } => {
                if let Some(spoken) = &parsed_doc.spoken_audio {
                    spoken
                        .tracks
                        .first()
                        .map(|t| (title.clone(), t.url.clone(), spoken.duration.clone()))
                } else {
                    None
                }
            }
            _ => None,
        };

        if let Some((play_title, track_url, duration)) = track_info {
            let success = self
                .audio_player
                .play(&play_title, &track_url, duration.as_deref());
            if !success {
                if self.audio_player.backend.is_none() {
                    self.set_status_message(
                        "no audio backend found (install mpv, ffplay, or cvlc)".to_string(),
                    );
                } else {
                    self.set_status_message("failed to start audio playback".to_string());
                }
            }
        } else {
            self.set_status_message("no spoken audio available for this article".to_string());
        }
    }

    pub fn seek_spoken_audio(&mut self, delta_secs: i64) {
        if self.audio_player.is_active() {
            self.audio_player.seek(delta_secs);
        }
    }

    pub fn stop_spoken_audio(&mut self) {
        if self.audio_player.is_active() {
            self.audio_player.stop();
        }
    }

    pub fn set_audio_sleep_timer(&mut self, minutes: u64) {
        self.set_audio_sleep_timer_secs(minutes * 60);
    }

    pub fn set_audio_sleep_timer_secs(&mut self, secs: u64) {
        if secs == 0 {
            self.cancel_audio_sleep_timer();
            return;
        }
        if !self.audio_player.is_active() {
            self.set_status_message("no audio currently playing".to_string());
            return;
        }
        self.audio_player.set_sleep_timer(secs);
        let formatted = if secs >= 3600 && secs.is_multiple_of(3600) {
            format!("{}h", secs / 3600)
        } else if secs >= 60 && secs.is_multiple_of(60) {
            format!("{}m", secs / 60)
        } else if secs >= 60 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}s", secs)
        };
        self.set_status_message(format!("sleep timer set for {}", formatted));
    }

    pub fn cancel_audio_sleep_timer(&mut self) {
        if self.audio_player.sleep_timer_deadline.is_some() {
            self.audio_player.clear_sleep_timer();
            self.set_status_message("sleep timer cancelled".to_string());
        } else {
            self.set_status_message("no active sleep timer".to_string());
        }
    }

    pub fn open_sleep_timer_prompt(&mut self) {
        if !self.audio_player.is_active() {
            self.set_status_message("no audio currently playing".to_string());
            return;
        }
        self.search_modal.input.clear();
        self.search_modal.cursor_pos = 0;
        self.input_mode = crate::app::InputMode::SleepTimerPrompt;
    }

    pub fn submit_sleep_timer_prompt(&mut self) {
        let input = self.search_modal.input.trim().to_lowercase();
        self.search_modal.input.clear();
        self.search_modal.cursor_pos = 0;
        self.input_mode = crate::app::InputMode::Normal;

        if input.is_empty() {
            return;
        }
        if input == "off" || input == "0" {
            self.cancel_audio_sleep_timer();
            return;
        }
        if let Some(secs) = crate::audio::parse_duration_to_secs(&input) {
            self.set_audio_sleep_timer_secs(secs);
        } else if let Ok(mins) = input.parse::<u64>() {
            self.set_audio_sleep_timer_secs(mins * 60);
        } else {
            self.set_status_message(format!("invalid duration: {}", input));
        }
    }
}
