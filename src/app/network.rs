use crate::api::NetworkCommand;
use crate::app::App;

impl App {
    pub fn next_request_id(&mut self) -> u64 {
        self.network.next_request_id()
    }

    pub fn send_fetch_article(&mut self, pane_id: usize, title: String) {
        let request_id = self.next_request_id();
        if let Some(pane) = self.find_pane_mut(pane_id) {
            pane.current_request_id = request_id;
            pane.loading_title = Some(title.clone());
        }
        self.network.send(NetworkCommand::FetchArticle {
            request_id,
            pane_id,
            title,
            timeout: self.user_data.config.network.timeout,
            offline_cache: self.user_data.config.network.offline_cache,
            cache_lifetime: self.user_data.config.network.cache_lifetime,
        });
    }

    pub fn send_fetch_random_article(&mut self, pane_id: usize) {
        let request_id = self.next_request_id();
        if let Some(pane) = self.find_pane_mut(pane_id) {
            pane.current_request_id = request_id;
            pane.loading_title = Some("random article".to_string());
        }
        self.network.send(NetworkCommand::FetchRandomArticle {
            request_id,
            pane_id,
            timeout: self.user_data.config.network.timeout,
            offline_cache: self.user_data.config.network.offline_cache,
            cache_lifetime: self.user_data.config.network.cache_lifetime,
        });
    }

    pub fn send_fetch_feed_batch(&self) {
        self.network.send(NetworkCommand::FetchFeedBatch {
            timeout: self.user_data.config.network.timeout,
        });
    }

    pub fn send_fetch_daily_feed(&self) {
        self.network.send(NetworkCommand::FetchDailyFeed {
            timeout: self.user_data.config.network.timeout,
            offline_cache: self.user_data.config.network.offline_cache,
        });
    }

    pub fn send_fetch_stats(&self) {
        if self.user_data.config.ui.stats {
            self.network.send(NetworkCommand::FetchStats {
                timeout: self.user_data.config.network.timeout,
            });
        }
    }

    pub fn check_for_updates(&mut self) {
        self.set_status_message("checking for updates...".to_string());
        self.network.send(NetworkCommand::CheckForUpdates {
            timeout: self.user_data.config.network.timeout,
        });
    }

    pub fn send_fetch_image(&self, url: String) {
        if self.user_data.config.reader.show_images {
            self.network.send(NetworkCommand::FetchImage {
                url,
                timeout: self.user_data.config.network.timeout,
            });
        }
    }

    pub fn send_decode_halfblock_image(
        &self,
        url: String,
        path: std::path::PathBuf,
        cols: usize,
        rows: usize,
    ) {
        self.network.send(NetworkCommand::DecodeHalfblockImage {
            url,
            path,
            cols,
            rows,
            filter: self.user_data.config.reader.halfblock_filter,
        });
    }

    pub fn send_predecode_kitty_image(&self, path: std::path::PathBuf) {
        self.network
            .send(NetworkCommand::PredecodeKittyImage { path });
    }
}
