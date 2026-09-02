mod app;
mod ui;

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};

use app::{App, Screen};

fn main() -> anyhow::Result<()> {
    let _log_guard = init_logging();

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.refresh();
    let result = run(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

/// The TUI owns the terminal, so logs go to a file, never stdout/stderr.
fn init_logging() -> tracing_appender::non_blocking::WorkerGuard {
    let log_dir = std::env::temp_dir().join("universal-net-installer");
    let _ = std::fs::create_dir_all(&log_dir);
    let file_appender = tracing_appender::rolling::never(&log_dir, "uni-tui.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let _ = uni_core::logging::init(non_blocking);
    guard
}

fn run<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> anyhow::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if event::poll(Duration::from_millis(200))?
            && let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            handle_key(app, key.code);
        }
    }
    Ok(())
}

fn handle_key(app: &mut App, code: KeyCode) {
    match app.screen {
        Screen::Dashboard => match code {
            KeyCode::Char('q') | KeyCode::Esc => app.should_quit = true,
            KeyCode::Char('r') => app.refresh(),
            KeyCode::Char('w') => app.open_wifi_screen(),
            _ => {}
        },
        Screen::WifiList => match code {
            KeyCode::Esc => app.cancel_wifi_flow(),
            KeyCode::Up | KeyCode::Char('k') => app.wifi_move_selection(-1),
            KeyCode::Down | KeyCode::Char('j') => app.wifi_move_selection(1),
            KeyCode::Char('r') => app.open_wifi_screen(),
            KeyCode::Enter => app.wifi_confirm_selection(),
            _ => {}
        },
        Screen::WifiPassword => match code {
            KeyCode::Esc => app.cancel_wifi_flow(),
            KeyCode::Enter => app.wifi_submit_password(),
            KeyCode::Backspace => app.password_pop_char(),
            KeyCode::Char(c) => app.password_push_char(c),
            _ => {}
        },
    }
}
