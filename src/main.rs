use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyboardEnhancementFlags,
        PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{
    io,
    sync::mpsc,
    time::{Duration, Instant},
};

use wikid::api::{self, NetworkCommand, NetworkEvent};
use wikid::app::App;
use wikid::keybinds;
use wikid::mouse;
use wikid::ui;

fn restore_terminal() {
    let _ = wikid::graphics::kitty::clear_all_kitty_images(&mut io::stdout());
    let _ = disable_raw_mode();
    let _ = execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        PopKeyboardEnhancementFlags
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        EnterAlternateScreen,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    );

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        restore_terminal();
        original_hook(panic_info);
    }));

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (cmd_tx, cmd_rx) = mpsc::channel::<NetworkCommand>();
    let (ev_tx, ev_rx) = mpsc::channel::<NetworkEvent>();

    let worker = std::thread::spawn(move || {
        api::run_worker(cmd_rx, ev_tx);
    });

    let mut app = App::new(cmd_tx.clone());
    if app.user_data.config.input.mouse_support {
        let _ = execute!(io::stdout(), EnableMouseCapture);
    }
    let run_res = run_app(&mut terminal, &mut app, &ev_rx);

    restore_terminal();
    let _ = terminal.show_cursor();

    drop(app);
    drop(cmd_tx);
    let _ = worker.join();

    run_res
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
    ev_rx: &mpsc::Receiver<NetworkEvent>,
) -> Result<(), Box<dyn std::error::Error>> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();
    let mut mouse_capture_enabled = app.user_data.config.input.mouse_support;

    while app.running {
        app.check_config_sync();
        app.audio_player.poll_status();
        if app.audio_player.sleep_timer_expired {
            app.audio_player.sleep_timer_expired = false;
            app.set_status_message("sleep timer paused playback");
        }

        if app.user_data.config.input.mouse_support != mouse_capture_enabled {
            mouse_capture_enabled = app.user_data.config.input.mouse_support;
            if mouse_capture_enabled {
                let _ = execute!(io::stdout(), EnableMouseCapture);
            } else {
                let _ = execute!(io::stdout(), DisableMouseCapture);
            }
        }

        while let Ok(ev) = ev_rx.try_recv() {
            app.handle_network_event(ev);
        }

        terminal.draw(|f| ui::draw(f, app))?;

        if app.graphics.pending_image_renders != app.graphics.last_kitty_render_tasks {
            let mut stdout = io::stdout();

            use crossterm::ExecutableCommand;
            use std::io::Write;

            let _ = stdout.execute(crossterm::cursor::SavePosition);

            if !app.graphics.pending_image_renders.is_empty() {
                let _ = wikid::graphics::kitty::clear_all_kitty_images(&mut stdout);
                for task in &app.graphics.pending_image_renders {
                    let _ = wikid::graphics::kitty::render_kitty_image_from_path(
                        &mut stdout,
                        wikid::graphics::kitty::KittyImageArgs {
                            path: &task.path,
                            screen_x: task.screen_x,
                            screen_y: task.screen_y,
                            cols: task.cols,
                            rows: task.rows,
                            crop_top_lines: task.crop_top_lines,
                            crop_bot_lines: task.crop_bot_lines,
                        },
                    );
                }
                app.graphics.has_active_kitty_images = true;
            } else if app.graphics.has_active_kitty_images {
                let _ = wikid::graphics::kitty::clear_all_kitty_images(&mut stdout);
                app.graphics.has_active_kitty_images = false;
            }

            let _ = stdout.execute(crossterm::cursor::RestorePosition);
            let _ = stdout.flush();

            app.graphics.last_kitty_render_tasks = app.graphics.pending_image_renders.clone();
        }
        app.graphics.pending_image_renders.clear();

        let has_loading = app.user_data.feed.is_fetching
            || (app.user_data.feed.active && app.user_data.feed.items.is_empty())
            || app.audio_player.is_playing()
            || (app.modals.daily_feed_modal.is_some() && app.daily_feed.is_none())
            || app
                .workspace
                .tabs
                .iter()
                .any(|t| t.panes.iter().any(|p| p.is_loading));
        let active_tick = if has_loading {
            Duration::from_millis(80)
        } else {
            tick_rate
        };

        let timeout = active_tick
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if event::poll(timeout)? {
            let size = terminal.size()?;
            match event::read()? {
                Event::Key(key) if key.kind == event::KeyEventKind::Press => {
                    keybinds::handle_key_event(app, key, size.width, size.height);
                }
                Event::Mouse(mouse_event) if app.user_data.config.input.mouse_support => {
                    mouse::handle_mouse_event(app, mouse_event, size.width, size.height);
                }
                Event::Resize(_, _) => {
                    app.graphics.last_kitty_render_tasks.clear();
                    wikid::graphics::kitty::invalidate_kitty_cache();
                    let _ = terminal.clear();
                }
                _ => {}
            }
        }

        if last_tick.elapsed() >= active_tick {
            last_tick = Instant::now();
        }
    }

    Ok(())
}
