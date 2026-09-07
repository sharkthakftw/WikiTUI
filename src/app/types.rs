#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageRenderTask {
    pub path: std::path::PathBuf,
    pub screen_x: u16,
    pub screen_y: u16,
    pub cols: u16,
    pub rows: u16,
    pub crop_top_lines: u16,
    pub crop_bot_lines: u16,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConfirmAction {
    DeleteList { list_id: String, title: String },
    DeleteArticle { list_id: String, title: String },
    ResetFeed,
    Quit,
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Search,
    LocalSearch,
    Help,
    CategoryOnboarding,
    SaveToList,
    CreateNewList,
    SavedListsViewer,
    RenameList,
    Confirm,
    Settings,
    Categories,
    DailyFeedModal,
    CommandPalette,
    QrModal,
    ImageModal,
    LinkPeek,
}

#[derive(Clone, Debug)]
pub struct LinkPeekState {
    pub title: String,
    pub raw_target: String,
    pub description: Option<String>,
    pub extract: Option<String>,
    pub is_loading: bool,
    pub anchor_x: u16,
    pub anchor_y: u16,
}

#[derive(Clone, Debug)]
pub struct ImageModalState {
    pub url: String,
    pub alt: Option<String>,
    pub caption: Option<String>,
    pub path: Option<std::path::PathBuf>,
}

#[derive(Clone, Debug)]
pub struct QrModalState {
    pub title: String,
    pub full_url: String,
    pub short_url: Option<String>,
    pub matrix: Vec<Vec<bool>>,
}

pub fn is_article_link(title: &str) -> bool {
    const MEDIA_EXTENSIONS: &[&str] = &[".jpg", ".png", ".svg", ".gif", ".jpeg", ".webp"];
    let lower = title.to_lowercase();
    !lower.starts_with("http://")
        && !lower.starts_with("https://")
        && !MEDIA_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

#[derive(Clone, Debug)]
pub struct ClosedTabState {
    pub title: String,
    pub scroll_offset: usize,
    pub history_back: Vec<String>,
    pub history_forward: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub struct SearchModalState {
    pub input: String,
    pub cursor_pos: usize,
    pub opens_new_tab: bool,
}

#[derive(Clone, Debug, Default)]
pub struct SettingsModalState {
    pub cursor_idx: usize,
}

#[derive(Clone, Debug, Default)]
pub struct CategoriesModalState {
    pub cursor_idx: usize,
    pub article_cursor_idx: usize,
    pub focus_right: bool,
    pub cached_members: std::collections::HashMap<String, Vec<String>>,
    pub fetching_categories: std::collections::HashSet<String>,
}

#[derive(Clone, Debug)]
pub struct OnboardingModalState {
    pub cursor_idx: usize,
    pub selected: Vec<bool>,
}

impl Default for OnboardingModalState {
    fn default() -> Self {
        Self {
            cursor_idx: 0,
            selected: vec![
                false, false, false, false, true, false, false, true, true, false, false, true,
            ],
        }
    }
}

#[derive(Clone, Debug)]
pub struct ListsModalState {
    pub target_title: String,
    pub target_snippet: Option<String>,
    pub save_cursor_idx: usize,
    pub create_return_mode: InputMode,
    pub viewer_list_idx: usize,
    pub viewer_article_idx: usize,
    pub viewer_focus_right: bool,
    pub rename_list_id: String,
}

impl Default for ListsModalState {
    fn default() -> Self {
        Self {
            target_title: String::new(),
            target_snippet: None,
            save_cursor_idx: 0,
            create_return_mode: InputMode::SaveToList,
            viewer_list_idx: 0,
            viewer_article_idx: 0,
            viewer_focus_right: false,
            rename_list_id: String::new(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct CommandPaletteState {
    pub query: String,
    pub selected_idx: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GraphicsState {
    pub pending_image_renders: Vec<ImageRenderTask>,
    pub last_kitty_render_tasks: Vec<ImageRenderTask>,
    pub has_active_kitty_images: bool,
}

#[derive(Clone, Debug, Default)]
pub struct ClosedTabsHistory {
    pub stack: Vec<ClosedTabState>,
}

impl ClosedTabsHistory {
    pub fn push(&mut self, state: ClosedTabState) {
        self.stack.push(state);
        if self.stack.len() > 30 {
            self.stack.remove(0);
        }
    }

    pub fn pop(&mut self) -> Option<ClosedTabState> {
        self.stack.pop()
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }
}

pub struct ConfigManager {
    pub current: crate::config::Config,
    pub last_mtime: Option<std::time::SystemTime>,
    pub last_check: std::time::Instant,
}

impl ConfigManager {
    pub fn new(config: crate::config::Config) -> Self {
        Self {
            last_mtime: crate::config::Config::get_modified_time(),
            last_check: std::time::Instant::now(),
            current: config,
        }
    }

    pub fn check_sync(&mut self) {
        if self.last_check.elapsed() >= std::time::Duration::from_millis(500) {
            self.last_check = std::time::Instant::now();
            self.current.reload_if_changed(&mut self.last_mtime);
        }
    }

    pub fn update_mtime(&mut self) {
        self.last_mtime = crate::config::Config::get_modified_time();
    }
}

impl std::ops::Deref for ConfigManager {
    type Target = crate::config::Config;
    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl std::ops::DerefMut for ConfigManager {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.current
    }
}

pub struct NetworkDispatcher {
    pub next_request_id: u64,
    pub cmd_tx: std::sync::mpsc::Sender<crate::api::NetworkCommand>,
}

impl NetworkDispatcher {
    pub fn new(cmd_tx: std::sync::mpsc::Sender<crate::api::NetworkCommand>) -> Self {
        Self {
            next_request_id: 1,
            cmd_tx,
        }
    }

    pub fn next_request_id(&mut self) -> u64 {
        let req_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1).max(1);
        req_id
    }

    pub fn send(&self, cmd: crate::api::NetworkCommand) {
        let _ = self.cmd_tx.send(cmd);
    }
}

#[derive(Clone, Debug, Default)]
pub struct StatusMessageState {
    pub message: Option<(String, std::time::Instant)>,
}

impl StatusMessageState {
    pub fn set(&mut self, msg: impl Into<String>) {
        self.message = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn get(&self) -> Option<&str> {
        if let Some((msg, time)) = &self.message {
            if time.elapsed().as_secs_f32() < 3.0 {
                return Some(msg.as_str());
            }
        }
        None
    }
}
