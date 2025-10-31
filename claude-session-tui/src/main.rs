use crossterm::event::{self, Event};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::{Duration, SystemTime};
use tokio::time;

use claude_session_tui::ui::App;

#[derive(Debug, Clone)]
enum Msg {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Tick,
}

/// Parse time filter string like "7d", "1w", "24h"
fn parse_time_filter(filter: &str) -> Option<Duration> {
    let filter = filter.to_lowercase();

    if let Some(num_str) = filter.strip_suffix('d') {
        num_str
            .parse::<u64>()
            .ok()
            .map(|n| Duration::from_secs(n * 86400))
    } else if let Some(num_str) = filter.strip_suffix('w') {
        num_str
            .parse::<u64>()
            .ok()
            .map(|n| Duration::from_secs(n * 604800))
    } else if let Some(num_str) = filter.strip_suffix('h') {
        num_str
            .parse::<u64>()
            .ok()
            .map(|n| Duration::from_secs(n * 3600))
    } else if let Some(num_str) = filter.strip_suffix("m") {
        num_str
            .parse::<u64>()
            .ok()
            .map(|n| Duration::from_secs(n * 60))
    } else {
        None
    }
}

/// Filter directory to only include recently modified files
async fn filter_recent_files(
    dir: &PathBuf,
    since_duration: Option<Duration>,
) -> anyhow::Result<Vec<PathBuf>> {
    use std::fs;
    use walkdir::WalkDir;

    let cutoff = if let Some(duration) = since_duration {
        SystemTime::now() - duration
    } else {
        // No filter, return all files
        return Ok(WalkDir::new(dir)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("jsonl"))
                    .unwrap_or(false)
            })
            .map(|entry| entry.path().to_path_buf())
            .collect());
    };

    let files: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.eq_ignore_ascii_case("jsonl"))
                .unwrap_or(false)
        })
        .filter(|entry| {
            if let Ok(metadata) = fs::metadata(entry.path()) {
                if let Ok(modified) = metadata.modified() {
                    modified > cutoff
                } else {
                    true // Include if we can't determine time
                }
            } else {
                true // Include if we can't check
            }
        })
        .map(|entry| entry.path().to_path_buf())
        .collect();

    Ok(files)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging only if RUST_LOG is explicitly set
    // By default, suppress all logging to keep TUI clean
    use tracing_subscriber::filter::EnvFilter;

    let env_filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    } else {
        // No logging by default
        EnvFilter::new("error=off,warn=off,info=off,debug=off,trace=off")
    };

    let _ = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .try_init();
    claude_session_tui::init()?;

    // Parse CLI arguments for data directory
    // Default to ~/.claude/projects
    let default_dir = dirs::home_dir()
        .map(|h| h.join(".claude").join("projects"))
        .unwrap_or_else(|| PathBuf::from("demo_projects"));

    let mut data_dir = default_dir.clone();
    let mut time_filter: Option<Duration> = None;
    {
        let mut args = std::env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    println!("Claude Session TUI - Browse and search conversations\n");
                    println!("Usage: claude-session-tui [OPTIONS]\n");
                    println!("Options:");
                    println!("  -d, --dir <path>     Directory with .jsonl files (default: ~/.claude/projects)");
                    println!("  --since <time>       Only load sessions from the past <time> (e.g., 7d, 1w, 24h)");
                    println!("  -h, --help           Show this help message\n");
                    println!("Examples:");
                    println!("  claude-session-tui                        # Load all sessions from ~/.claude/projects");
                    println!("  claude-session-tui --since 7d             # Load only past 7 days (fast!)");
                    println!(
                        "  claude-session-tui --since 1w --dir ~/custom  # Custom dir, past week\n"
                    );
                    println!("Time format: <number><unit>");
                    println!("  d  = days (e.g., 7d = past 7 days)");
                    println!("  w  = weeks (e.g., 2w = past 2 weeks)");
                    println!("  h  = hours (e.g., 24h = past 24 hours)");
                    println!("  m  = minutes (e.g., 30m = past 30 minutes)");
                    return Ok(());
                }
                "-d" | "--dir" => {
                    if let Some(val) = args.next() {
                        // Expand ~ in the path
                        let expanded = if val.starts_with('~') {
                            dirs::home_dir()
                                .map(|h| {
                                    let rest = &val[1..];
                                    if rest.is_empty() {
                                        h
                                    } else {
                                        h.join(&rest[1..])
                                    }
                                })
                                .unwrap_or_else(|| PathBuf::from(&val))
                        } else {
                            PathBuf::from(&val)
                        };
                        data_dir = expanded;
                    } else {
                        eprintln!("Missing value for --dir");
                    }
                }
                _ if arg.starts_with("--dir=") => {
                    let val = &arg["--dir=".len()..];
                    // Expand ~ in the path
                    let expanded = if val.starts_with('~') {
                        dirs::home_dir()
                            .map(|h| {
                                let rest = &val[1..];
                                if rest.is_empty() {
                                    h
                                } else {
                                    h.join(&rest[1..])
                                }
                            })
                            .unwrap_or_else(|| PathBuf::from(val))
                    } else {
                        PathBuf::from(val)
                    };
                    data_dir = expanded;
                }
                "--since" => {
                    if let Some(val) = args.next() {
                        if let Some(duration) = parse_time_filter(&val) {
                            time_filter = Some(duration);
                        } else {
                            eprintln!(
                                "Invalid time format: {}. Use format like 7d, 1w, 24h, 30m",
                                val
                            );
                        }
                    } else {
                        eprintln!("Missing value for --since");
                    }
                }
                _ if arg.starts_with("--since=") => {
                    let val = &arg["--since=".len()..];
                    if let Some(duration) = parse_time_filter(val) {
                        time_filter = Some(duration);
                    } else {
                        eprintln!(
                            "Invalid time format: {}. Use format like 7d, 1w, 24h, 30m",
                            val
                        );
                    }
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

    // Load sessions asynchronously with optional time filtering
    if let Some(duration) = time_filter {
        // Apply time filter for faster loading
        match filter_recent_files(&data_dir, Some(duration)).await {
            Ok(files) => {
                let file_count = files.len();
                if file_count == 0 {
                    app.set_error("No sessions found in the specified time range".to_string());
                } else if let Err(err) = app.load_sessions_from_files(files).await {
                    app.set_error(format!("Failed to load sessions: {}", err));
                }
            }
            Err(err) => {
                app.set_error(format!("Failed to filter sessions: {}", err));
            }
        }
    } else {
        // No filter, load all files
        if let Err(err) = app.load_sessions(data_dir).await {
            app.set_error(format!("Failed to load sessions: {}", err));
        }
    }

    // Message bus
    let (tx, rx) = mpsc::channel::<Msg>();

    // Keyboard and Mouse task
    let tx_keys = tx.clone();
    tokio::spawn(async move {
        loop {
            // Poll for responsiveness without busy waiting
            if event::poll(Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(ev) = event::read() {
                    match ev {
                        Event::Key(k) => {
                            let _ = tx_keys.send(Msg::Key(k));
                        }
                        Event::Mouse(m) => {
                            let _ = tx_keys.send(Msg::Mouse(m));
                        }
                        _ => {}
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
                Msg::Mouse(m) => {
                    app.handle_mouse_event(m)?;
                }
                Msg::Tick => {
                    app.update();
                }
            }
        }
        guard.terminal.draw(|f| app.render(f))?;
        tokio::time::sleep(Duration::from_millis(16)).await; // ~60 FPS
    }

    // Drop the terminal guard to restore normal terminal before printing
    drop(guard);

    // Print exit message
    println!("\n{}\n", app.get_exit_message());

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
