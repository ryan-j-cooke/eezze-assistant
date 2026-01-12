use std::collections::{HashMap, HashSet};
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::net::TcpListener;

use crate::server::http::create_server;

const EXPERT_RECURSIVE_LOCAL: &str = "qwen2.5:3b";
const EXPERT_FAST_MODEL: &str = "qwen2.5:1.5b";
const EXPERT_REVIEWER_MODEL: &str = "qwen2.5:0.5b";
const EXPERT_EMBEDDING_DEFAULT: &str = "nomic-embed-text";

#[derive(Debug, Serialize, Deserialize)]
struct RuntimeConfig {
    server: ServerConfig,
    models: ModelConfigMap,
    orchestration: OrchestrationConfig,
    index: IndexConfig,
}

#[derive(Debug, Serialize, Deserialize)]
struct ServerConfig {
    host: Option<String>,
    port: u16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum Provider {
    Ollama,
    Openai,
    Local,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModelConfig {
    name: String,
    provider: Provider,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default, rename = "maxTokens")]
    max_tokens: Option<u32>,
}

type ModelConfigMap = HashMap<String, ModelConfig>;

#[derive(Debug, Serialize, Deserialize)]
struct OrchestrationConfig {
    max_attempts: u32,
    confidence_threshold: f64,
    allow_escalation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct IndexConfig {
    dimensions: u32,
    similarity_threshold: f64,
}

fn default_config() -> RuntimeConfig {
    RuntimeConfig {
        server: ServerConfig {
            host: Some("0.0.0.0".to_string()),
            port: 4000,
        },
        models: {
            let mut map = HashMap::new();
            map.insert(
                "fast".to_string(),
                ModelConfig {
                    name: EXPERT_FAST_MODEL.to_string(),
                    provider: Provider::Ollama,
                    temperature: None,
                    max_tokens: None,
                },
            );
            map.insert(
                "reviewer".to_string(),
                ModelConfig {
                    name: EXPERT_REVIEWER_MODEL.to_string(),
                    provider: Provider::Ollama,
                    temperature: None,
                    max_tokens: None,
                },
            );
            map
        },
        orchestration: OrchestrationConfig {
            max_attempts: 3,
            confidence_threshold: 0.75,
            allow_escalation: true,
        },
        index: IndexConfig {
            dimensions: 768,
            similarity_threshold: 0.8,
        },
    }
}

fn load_config() -> Result<RuntimeConfig, Box<dyn std::error::Error>> {
    let path = Path::new("config.json");

    if path.exists() {
        println!("startup.loadConfig: loading from {}", path.display());
        let raw = fs::read_to_string(path)?;
        let cfg: RuntimeConfig = serde_json::from_str(&raw)?;
        Ok(cfg)
    } else {
        println!("startup.loadConfig: using fallback defaults");
        Ok(default_config())
    }
}

async fn check_dependencies(
    config: &RuntimeConfig,
    client: &Client,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut required_models: HashSet<String> = HashSet::new();

    for model in config.models.values() {
        if model.provider == Provider::Ollama {
            required_models.insert(model.name.clone());
        }
    }

    match crate::eezze_config::load_config() {
        Ok(ee_cfg) => {
            required_models.insert(ee_cfg.expert_recursive_local);
            required_models.insert(ee_cfg.expert_fast_model);
            required_models.insert(ee_cfg.expert_reviewer_model);
            required_models.insert(ee_cfg.expert_embedding_default);
        }
        Err(err) => {
            eprintln!(
                "startup.eezzeConfig.load_failed: {} (falling back to built-in defaults)",
                err
            );
            required_models.insert(EXPERT_RECURSIVE_LOCAL.to_string());
            required_models.insert(EXPERT_FAST_MODEL.to_string());
            required_models.insert(EXPERT_REVIEWER_MODEL.to_string());
            required_models.insert(EXPERT_EMBEDDING_DEFAULT.to_string());
        }
    }

    let required_list: Vec<String> = required_models.iter().cloned().collect();

    println!(
        "startup.checkDependencies.start: requiredModels={:?}",
        required_list
    );

    let mut missing: Vec<String> = Vec::new();

    for model_name in &required_list {
        println!("startup.modelCheck.checking: model={}", model_name);

        match client
            .post("http://localhost:11434/api/show")
            .json(&json!({ "name": model_name }))
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    println!("startup.modelCheck.ok: model={}", model_name);
                } else {
                    eprintln!(
                        "startup.modelCheck.httpError: model={} status={}",
                        model_name,
                        response.status()
                    );
                    missing.push(model_name.clone());
                }
            }
            Err(error) => {
                eprintln!(
                    "startup.modelCheck.failed: model={} error={}",
                    model_name, error
                );
                missing.push(model_name.clone());
            }
        }
    }

    if !missing.is_empty() {
        eprintln!(
            "startup.modelsMissing: missingModels={:?}",
            missing
        );
        return Err(format!(
            "Required Ollama models missing: {}",
            missing.join(", ")
        )
        .into());
    }

    println!(
        "startup.checkDependencies.success: requiredModels={:?}",
        required_list
    );

    Ok(())
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigint = signal(SignalKind::interrupt()).expect("failed to install SIGINT handler");
        let mut sigterm = signal(SignalKind::terminate()).expect("failed to install SIGTERM handler");

        tokio::select! {
            _ = sigint.recv() => {
                println!("🛑 Received SIGINT, shutting down...");
            }
            _ = sigterm.recv() => {
                println!("🛑 Received SIGTERM, shutting down...");
            }
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install CTRL+C handler");
        println!("🛑 Received CTRL+C, shutting down...");
    }
}

pub async fn run_rlm_server() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(err) = crate::eezze_config::ensure_config_exists() {
        eprintln!("startup.eezzeConfig.error: {}", err);
    }

    let config = load_config()?;

    let client = Client::new();
    check_dependencies(&config, &client).await?;

    let host = config
        .server
        .host
        .clone()
        .unwrap_or_else(|| "0.0.0.0".to_string());
    let addr: SocketAddr = format!("{}:{}", host, config.server.port).parse()?;

    let app = create_server();

    println!("🚀 Recursive LLM server running at http://{}", addr);

    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}
