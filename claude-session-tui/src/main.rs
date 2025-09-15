use crossterm::event::{self, Event, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;
use tokio::time;

use claude_session_tui::ui::App;

#[derive(Debug, Clone)]
enum Msg {
    Key(crossterm::event::KeyEvent),
    Tick,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging once (ignore if already set)
    let _ = tracing_subscriber::fmt().try_init();
    // Library init logs a startup message, but does not set a subscriber
    claude_session_tui::init()?;

    // Parse CLI arguments for data directory
    let mut data_dir = PathBuf::from("demo_projects");
    {
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!(
                        "Usage: claude-session-tui [--dir <path>]\n\nOptions:\n  -d, --dir <path>   Directory containing .jsonl session files (default: demo_projects)\n  -h, --help         Show this help and exit"
                    );
                    return Ok(());
                }
                "-d" | "--dir" => {
                    if let Some(val) = args.next() {
                        data_dir = PathBuf::from(val);
                    } else {
                        eprintln!("Missing value for --dir");
                    }
                }
                _ if arg.starts_with("--dir=") => {
                    let val = &arg["--dir=".len()..];
                    data_dir = PathBuf::from(val);
                }
                _ => {
                    eprintln!("Unknown argument: {}", arg);
                }
            }
        }
    }

    // Setup terminal guard for proper cleanup
    let mut guard = TerminalGuard::new()?;
    let mut app = App::new().unwrap();

    // Load sessions asynchronously with error handling
    if let Err(err) = app.load_sessions(data_dir).await {
        app.set_error(format!("Failed to load sessions: {}", err));
    }

    // Message bus
    let (tx, rx) = mpsc::channel::<Msg>();

    // Keyboard task
    let tx_keys = tx.clone();
    tokio::spawn(async move {
        loop {
            // Poll for responsiveness without busy waiting
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(ev) = event::read() {
                    if let Event::Key(k) = ev {
                        let _ = tx_keys.send(Msg::Key(k));
                        if k.kind == KeyEventKind::Press {
                            if let crossterm::event::KeyCode::Char('q')
                            | crossterm::event::KeyCode::Esc = k.code
                            {
                                // Soft attempt to also send Quit, but main loop will interpret 'q' anyway
                            }
                        }
                    }
                }
            }
        }
    });

    // Tick task for periodic updates
    let tx_tick = tx.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_millis(250));
        loop {
            interval.tick().await;
            let _ = tx_tick.send(Msg::Tick);
        }
    });

    // UI loop: receive -> update -> draw
    while !app.should_quit() {
        if let Ok(msg) = rx.try_recv() {
            match msg {
                Msg::Key(k) => {
                    app.handle_key_event(k)?;
                }
                Msg::Tick => {
                    app.update();
                }
            }
        }
        guard.terminal.draw(|f| app.render(f))?;
        tokio::time::sleep(Duration::from_millis(16)).await; // ~60 FPS
    }

    Ok(())
}

// Terminal guard for proper setup/cleanup (similar to ratatui starter)
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TerminalGuard {
    fn new() -> anyhow::Result<Self> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;
        Ok(Self { terminal })
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let mut stdout = io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
    }
}
