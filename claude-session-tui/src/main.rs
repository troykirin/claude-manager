use std::io;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::CrosstermBackend, Terminal};

use claude_session_tui::ui::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging and any global state
    claude_session_tui::init()?;

    // Determine sessions directory (arg1 or default to ./demo_sessions)
    let sessions_dir = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("demo_sessions"));

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app and load sessions
    let mut app = App::new()?;
    if let Err(err) = app.load_sessions(sessions_dir).await {
        app.set_error(format!("Failed to load sessions: {}", err));
    }

    // Main loop
    loop {
        terminal.draw(|f| app.render(f))?;

        // Non-blocking event poll with small timeout
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                handle_key_event(&mut app, key)?;
            }
        }

        // Update application state
        app.update();

        if app.should_quit() {
            break;
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    // It's safe to unwrap here as we're cleaning up
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    Ok(())
}

fn handle_key_event(app: &mut App, key: KeyEvent) -> anyhow::Result<()> {
    app.handle_key_event(key)
}
