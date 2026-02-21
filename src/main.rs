#[path = "api.rs"]
mod lc_api;
#[path = "core.rs"]
mod lc_core;
#[path = "provider.rs"]
mod lc_provider;
#[path = "sandbox.rs"]
mod lc_sandbox;
#[path = "scheduler.rs"]
mod lc_scheduler;
mod adapters {
    pub mod telegram;
}

use lc_provider::{FallbackProvider, LlmRequest, MockProvider, OpenAiProvider};
use std::{env, fs, sync::Arc};

#[derive(Debug, Clone)]
struct Config {
    api_bind: String,
    default_command: String,
    default_chat_prompt: String,
    db_path: String,
    encryption_key: String,
    llm_primary_provider: String,
    llm_primary_api_key: Option<String>,
    llm_primary_model: String,
    llm_primary_api_base: String,
    llm_fallback_provider: String,
    llm_fallback_api_key: Option<String>,
    llm_fallback_model: String,
    llm_fallback_api_base: String,
    telegram_bot_token: Option<String>,
    telegram_chat_id: Option<String>,
    scheduler_tasks_file: String,
}

fn load_config(path: &str) -> Result<Config, String> {
    let src = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(Config {
        api_bind: read_toml_value(&src, "api_bind").unwrap_or_else(|| "127.0.0.1:8080".into()),
        default_command: read_toml_value(&src, "default_command").unwrap_or_else(|| "chat".into()),
        default_chat_prompt: read_toml_value(&src, "default_chat_prompt")
            .unwrap_or_else(|| "hello".into()),
        db_path: read_toml_value(&src, "db_path").unwrap_or_else(|| "lightclaw.db".into()),
        encryption_key: read_toml_value(&src, "encryption_key").unwrap_or_else(|| "dev-key".into()),
        llm_primary_provider: read_toml_value(&src, "llm_primary_provider")
            .unwrap_or_else(|| "openai".into()),
        llm_primary_api_key: read_toml_value(&src, "llm_primary_api_key").filter(|v| !v.is_empty()),
        llm_primary_model: read_toml_value(&src, "llm_primary_model")
            .unwrap_or_else(|| "gpt-4o-mini".into()),
        llm_primary_api_base: read_toml_value(&src, "llm_primary_api_base")
            .unwrap_or_else(|| "http://api.openai.com/v1".into()),
        llm_fallback_provider: read_toml_value(&src, "llm_fallback_provider")
            .unwrap_or_else(|| "mock".into()),
        llm_fallback_api_key: read_toml_value(&src, "llm_fallback_api_key")
            .filter(|v| !v.is_empty()),
        llm_fallback_model: read_toml_value(&src, "llm_fallback_model")
            .unwrap_or_else(|| "gpt-4o-mini".into()),
        llm_fallback_api_base: read_toml_value(&src, "llm_fallback_api_base")
            .unwrap_or_else(|| "http://api.openai.com/v1".into()),
        telegram_bot_token: read_toml_value(&src, "telegram_bot_token").filter(|v| !v.is_empty()),
        telegram_chat_id: read_toml_value(&src, "telegram_chat_id").filter(|v| !v.is_empty()),
        scheduler_tasks_file: read_toml_value(&src, "scheduler_tasks_file")
            .unwrap_or_else(|| "config/scheduler.toml".into()),
    })
}

fn read_toml_value(src: &str, key: &str) -> Option<String> {
    src.lines()
        .find(|line| line.trim_start().starts_with(&format!("{key} =")))
        .and_then(|line| line.split_once('='))
        .map(|(_, value)| value.trim().trim_matches('"').to_string())
}

fn build_core(cfg: &Config) -> Result<Arc<lc_core::Core>, String> {
    let primary = build_provider(
        &cfg.llm_primary_provider,
        &cfg.llm_primary_api_key,
        &cfg.llm_primary_model,
        &cfg.llm_primary_api_base,
    );
    let fallback = build_provider(
        &cfg.llm_fallback_provider,
        &cfg.llm_fallback_api_key,
        &cfg.llm_fallback_model,
        &cfg.llm_fallback_api_base,
    );
    let provider: Arc<dyn lc_provider::LlmProvider> =
        Arc::new(FallbackProvider { primary, fallback });
    Ok(Arc::new(lc_core::Core::new(
        provider,
        &cfg.db_path,
        &cfg.encryption_key,
    )?))
}

fn build_provider(
    provider: &str,
    api_key: &Option<String>,
    model: &str,
    api_base: &str,
) -> Arc<dyn lc_provider::LlmProvider> {
    match provider {
        "openai" if api_key.is_some() => Arc::new(OpenAiProvider {
            api_key: api_key.clone().unwrap_or_default(),
            model: model.to_string(),
            api_base: api_base.to_string(),
        }),
        _ => Arc::new(MockProvider),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let raw_command = args.get(1).map(|s| s.as_str()).unwrap_or("chat");
    let config_path = "lightclaw.toml";

    if raw_command == "init" {
        if !std::path::Path::new(config_path).exists() {
            fs::write(config_path, include_str!("../lightclaw.toml")).expect("write config");
            println!("created {}", config_path);
        }
        return;
    }

    let cfg = load_config(config_path).expect("config load");
    let command = args
        .get(1)
        .map(|s| s.as_str())
        .unwrap_or(cfg.default_command.as_str());
    let core = build_core(&cfg).expect("core");

    match command {
        "start" => {
            lc_api::run_api(&cfg.api_bind, core).expect("api run");
        }
        "chat" => {
            let prompt = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| cfg.default_chat_prompt.clone());
            println!("{}", core.handle_message(&prompt).expect("chat"));
        }
        "telegram-poll-once" => {
            let adapter = adapters::telegram::TelegramAdapter::new(
                cfg.telegram_bot_token.expect("telegram_bot_token missing"),
                cfg.telegram_chat_id.expect("telegram_chat_id missing"),
            );
            println!(
                "handled {} updates",
                adapter.poll_once(&core).expect("telegram")
            );
        }
        "run-scheduler-once" => {
            let tasks = lc_core::Core::load_tasks(&cfg.scheduler_tasks_file).expect("tasks");
            println!(
                "executed {} task(s)",
                core.run_scheduler_once(&tasks).expect("scheduler")
            );
        }
        "test-llm" => {
            let prompt = args
                .get(2)
                .cloned()
                .unwrap_or_else(|| "Respond with the single word: ok".to_string());
            let provider = build_provider(
                &cfg.llm_primary_provider,
                &cfg.llm_primary_api_key,
                &cfg.llm_primary_model,
                &cfg.llm_primary_api_base,
            );
            match provider.generate(LlmRequest { prompt }) {
                Ok(response) => {
                    println!("model={}", response.model);
                    println!("{}", response.text);
                }
                Err(error) => {
                    eprintln!("primary llm request failed: {}", error);
                    std::process::exit(2);
                }
            }
        }
        _ => eprintln!("unknown command"),
    }
}
