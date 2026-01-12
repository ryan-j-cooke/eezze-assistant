use std::process::{Command, Stdio};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};

mod eezze_config;

use crate::eezze_config::{ensure_config_exists, load_config, save_config, EezzeConfig};

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
    println!("Starting `ollama serve` (press Ctrl+C to stop)...");
    let status = Command::new("ollama")
        .arg("serve")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("failed to run `ollama serve`")?;

    if !status.success() {
        eprintln!("`ollama serve` exited with status {}", status);
    }

    Ok(())
}