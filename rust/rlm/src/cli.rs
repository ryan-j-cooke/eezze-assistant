use std::env;
use std::io::{self, BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

mod eezze_config;

use crate::eezze_config::{ensure_config_exists, save_config};

/// eezze CLI - helper around Ollama and the recursive LLM server.
#[derive(Parser, Debug)]
#[command(name = "eezze", version, about = "eezze CLI", propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Dependency management (Ollama + models)
    Deps {
        #[command(subcommand)]
        cmd: DepsCommand,
    },

    /// Manage configured model names
    Mdl {
        #[command(subcommand)]
        cmd: MdlCommand,
    },

    /// List available Ollama models (wrapper around `ollama list`)
    Models {
        #[command(subcommand)]
        cmd: ModelsCommand,
    },

    /// Run `ollama serve`
    Serve,
}

#[derive(Subcommand, Debug)]
enum DepsCommand {
    /// Install Ollama for the current OS
    Install,

    /// Download all models defined in the eezze config
    ModelsDownload,
}

#[derive(Subcommand, Debug)]
enum MdlCommand {
    /// Set a model in the eezze config
    ///
    /// `slot` can be one of: recursive, fast, reviewer, embed
    Set {
        slot: String,
        /// New model name (should correspond to `ollama list` output)
        model: String,
    },
}

#[derive(Subcommand, Debug)]
enum ModelsCommand {
    /// List all models from Ollama (`ollama list`)
    List,
}

enum UiEvent {
    TopLine(String),
    BottomLine(String),
    ChildrenDone,
}

struct AppState {
    top_lines: Vec<String>,
    bottom_lines: Vec<String>,
    children_done: bool,
}

impl AppState {
    fn new() -> Self {
        Self {
            top_lines: Vec::new(),
            bottom_lines: Vec::new(),
            children_done: false,
        }
    }

    fn push_top(&mut self, line: String) {
        self.top_lines.push(line);
    }

    fn push_bottom(&mut self, line: String) {
        self.bottom_lines.push(line);
    }

    fn set_children_done(&mut self) {
        self.children_done = true;
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Deps { cmd } => match cmd {
            DepsCommand::Install => deps_install(),
            DepsCommand::ModelsDownload => deps_models_download(),
        },
        Commands::Mdl { cmd } => match cmd {
            MdlCommand::Set { slot, model } => mdl_set(&slot, &model),
        },
        Commands::Models { cmd } => match cmd {
            ModelsCommand::List => models_list(),
        },
        Commands::Serve => serve_ollama(),
    }
}

fn deps_install() -> Result<()> {
    // Ensure config file exists so subsequent commands have a place to write values.
    let _ = ensure_config_exists();

    #[cfg(target_os = "linux")]
    {
        println!(
            "Installing Ollama on Linux using install.sh script (curl -fsSL https://ollama.com/install.sh | sh)..."
        );
        let status = Command::new("sh")
            .arg("-c")
            .arg("curl -fsSL https://ollama.com/install.sh | sh")
            .status()
            .context("failed to execute install script")?;
        if !status.success() {
            eprintln!("Ollama install script exited with status: {}", status);
        }
    }

    #[cfg(target_os = "macos")]
    {
        println!("To install Ollama on macOS, download and install the latest .dmg:");
        println!("  https://ollama.com/download/Ollama.dmg");
        // Best-effort: try to open in default browser.
        let _ = Command::new("open")
            .arg("https://ollama.com/download/Ollama.dmg")
            .status();
    }

    #[cfg(target_os = "windows")]
    {
        println!("To install Ollama on Windows, download and run the installer:");
        println!("  https://ollama.com/download/OllamaSetup.exe");
        // Best-effort: open URL in default browser.
        let _ = Command::new("cmd")
            .args(["/C", "start", "https://ollama.com/download/OllamaSetup.exe"])
            .status();
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        println!("Unsupported OS for automatic Ollama installation. Please visit https://ollama.com/download.");
    }

    Ok(())
}

fn deps_models_download() -> Result<()> {
    let cfg = ensure_config_exists().context("failed to ensure eezze config exists")?;

    let mut models = std::collections::HashSet::new();
    models.insert(cfg.expert_recursive_local);
    models.insert(cfg.expert_fast_model);
    models.insert(cfg.expert_reviewer_model);
    models.insert(cfg.expert_embedding_default);

    println!("Downloading models via `ollama pull` for: {:?}", models);

    for model in models {
        println!("ollama pull {}", model);
        let status = Command::new("ollama")
            .arg("pull")
            .arg(&model)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .with_context(|| format!("failed to run `ollama pull {}`", model))?;

        if !status.success() {
            eprintln!("`ollama pull {}` exited with status {}", model, status);
        }
    }

    Ok(())
}

fn mdl_set(slot: &str, model: &str) -> Result<()> {
    let mut cfg = ensure_config_exists().context("failed to load eezze config")?;

    match slot {
        "recursive" | "expert_recursive_local" | "main" => {
            cfg.expert_recursive_local = model.to_string();
        }
        "fast" | "expert_fast_model" => {
            cfg.expert_fast_model = model.to_string();
        }
        "reviewer" | "expert_reviewer_model" => {
            cfg.expert_reviewer_model = model.to_string();
        }
        "embed" | "embedding" | "expert_embedding_default" => {
            cfg.expert_embedding_default = model.to_string();
        }
        other => {
            eprintln!(
                "Unknown slot '{}'. Use one of: recursive, fast, reviewer, embed",
                other
            );
            return Ok(());
        }
    }

    save_config(&cfg).context("failed to save eezze config")?;
    println!("Updated '{}' model to '{}'", slot, model);
    Ok(())
}

fn models_list() -> Result<()> {
    println!("Listing models via `ollama list`...");
    let status = Command::new("ollama")
        .arg("list")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to run `ollama list`")?;

    if !status.success() {
        eprintln!("`ollama list` exited with status {}", status);
    }

    Ok(())
}

fn serve_ollama() -> Result<()> {
    println!("Starting `ollama serve` and `rlm` server (press Ctrl+C or 'q' to stop UI)...");

    let mut ollama_child = Command::new("ollama")
        .arg("serve")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to start `ollama serve`")?;

    let current_exe = env::current_exe().context("failed to determine current executable path")?;
    let rlm_path = current_exe
        .parent()
        .map(|p| p.join("rlm"))
        .ok_or_else(|| anyhow::anyhow!("could not determine rlm binary path"))?;

    let mut rlm_child = Command::new(rlm_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("failed to start `rlm` server")?;

    let ollama_stdout = ollama_child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture ollama stdout"))?;
    let ollama_stderr = ollama_child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture ollama stderr"))?;

    let rlm_stdout = rlm_child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture rlm stdout"))?;
    let rlm_stderr = rlm_child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture rlm stderr"))?;

    let (tx, rx) = mpsc::channel::<UiEvent>();

    {
        let tx = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(ollama_stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if tx.send(UiEvent::TopLine(line)).is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    {
        let tx = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(ollama_stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if tx.send(UiEvent::TopLine(line)).is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    {
        let tx = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(rlm_stdout);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if tx.send(UiEvent::BottomLine(line)).is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    {
        let tx = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(rlm_stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if tx.send(UiEvent::BottomLine(line)).is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });
    }

    {
        let tx = tx.clone();
        thread::spawn(move || {
            let _ = ollama_child.wait();
            let _ = rlm_child.wait();
            let _ = tx.send(UiEvent::ChildrenDone);
        });
    }

    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("failed to create terminal")?;

    let mut app = AppState::new();
    let mut should_quit = false;
    let mut saw_children_done = false;

    while !should_quit {
        while let Ok(event) = rx.try_recv() {
            match event {
                UiEvent::TopLine(line) => app.push_top(line),
                UiEvent::BottomLine(line) => app.push_bottom(line),
                UiEvent::ChildrenDone => {
                    app.set_children_done();
                    saw_children_done = true;
                }
            }
        }

        terminal
            .draw(|f| {
                let size = f.size();

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
                    .split(size);

                let top_height = chunks[0].height as usize;
                let bottom_height = chunks[1].height as usize;

                let top_start = app
                    .top_lines
                    .len()
                    .saturating_sub(top_height.saturating_sub(2));
                let bottom_start = app
                    .bottom_lines
                    .len()
                    .saturating_sub(bottom_height.saturating_sub(2));

                let top_text = app.top_lines[top_start..].join("\n");
                let bottom_text = app.bottom_lines[bottom_start..].join("\n");

                let top_paragraph = Paragraph::new(top_text)
                    .block(Block::default().borders(Borders::ALL).title("Ollama"));
                let bottom_paragraph = Paragraph::new(bottom_text)
                    .block(Block::default().borders(Borders::ALL).title("rlm"));

                f.render_widget(top_paragraph, chunks[0]);
                f.render_widget(bottom_paragraph, chunks[1]);

                let logo_lines = [
                    "  ___  ___ ",
                    " | __|/ _ \\",
                    " | _|| (_) |",
                    " |___|\\___/",
                    "   EEZZE   ",
                ];
                let logo_width = logo_lines
                    .iter()
                    .map(|s| s.len())
                    .max()
                    .unwrap_or(0) as u16;
                let logo_height = logo_lines.len() as u16;

                if size.width > logo_width && size.height >= logo_height {
                    let area = Rect::new(
                        size.x + size.width - logo_width,
                        size.y,
                        logo_width,
                        logo_height,
                    );
                    let logo_text = logo_lines.join("\n");
                    let logo_paragraph = Paragraph::new(logo_text).alignment(Alignment::Right);
                    f.render_widget(logo_paragraph, area);
                }
            })
            .context("failed to draw UI")?;

        if event::poll(Duration::from_millis(50)).context("failed to poll events")? {
            if let CEvent::Key(key) = event::read().context("failed to read event")? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        should_quit = true;
                    }
                    _ => {}
                }
            }
        }

        if saw_children_done && app.children_done && rx.try_recv().is_err() {
            should_quit = true;
        }
    }

    disable_raw_mode().context("failed to disable raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, LeaveAlternateScreen).context("failed to leave alternate screen")?;

    Ok(())
}