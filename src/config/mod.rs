use std::env;

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub api_base: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TelegramConfig {
    pub enabled: bool,
    pub bot_token: Option<String>,
    pub chat_id: Option<String>,
    pub webhook_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub web_bind: String,
    pub default_mode: String,
    pub registry_path: String,
    pub registry_secret: String,
    pub benchmark_iterations: usize,
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub llm: LlmConfig,
    pub telegram: TelegramConfig,
    pub runtime: RuntimeConfig,
}

impl AppConfig {
    pub fn from_env() -> Self {
        let llm = LlmConfig {
            provider: env_var("JARVIS_LLM_PROVIDER", "echo-local"),
            model: env_var("JARVIS_LLM_MODEL", "mock-1"),
            api_base: env::var("JARVIS_LLM_API_BASE").ok(),
            api_key: env::var("JARVIS_LLM_API_KEY").ok(),
        };

        let telegram = TelegramConfig {
            enabled: env_bool("JARVIS_TELEGRAM_ENABLED", false),
            bot_token: env::var("JARVIS_TELEGRAM_BOT_TOKEN").ok(),
            chat_id: env::var("JARVIS_TELEGRAM_CHAT_ID").ok(),
            webhook_url: env::var("JARVIS_TELEGRAM_WEBHOOK_URL").ok(),
        };

        let runtime = RuntimeConfig {
            web_bind: env_var("JARVIS_WEB_BIND", "127.0.0.1:7878"),
            default_mode: env_var("JARVIS_DEFAULT_MODE", "chat"),
            registry_path: env_var("JARVIS_REGISTRY_PATH", "registry"),
            registry_secret: env_var("JARVIS_REGISTRY_SECRET", "local-dev-secret"),
            benchmark_iterations: env::var("JARVIS_BENCH_ITERATIONS")
                .ok()
                .and_then(|v| v.parse::<usize>().ok())
                .filter(|v| *v > 0)
                .unwrap_or(100),
        };

        Self {
            llm,
            telegram,
            runtime,
        }
    }

    pub fn validate_telegram(&self) -> Result<(), String> {
        if !self.telegram.enabled {
            return Ok(());
        }

        if self.telegram.bot_token.is_none() {
            return Err("telegram enabled but JARVIS_TELEGRAM_BOT_TOKEN is missing".to_string());
        }

        if self.telegram.chat_id.is_none() {
            return Err("telegram enabled but JARVIS_TELEGRAM_CHAT_ID is missing".to_string());
        }

        Ok(())
    }

    pub fn redacted_summary(&self) -> String {
        format!(
            "provider={} model={} web_bind={} telegram_enabled={} registry_path={} bench_iterations={}",
            self.llm.provider,
            self.llm.model,
            self.runtime.web_bind,
            self.telegram.enabled,
            self.runtime.registry_path,
            self.runtime.benchmark_iterations
        )
    }
}

fn env_var(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_bool(key: &str, default: bool) -> bool {
    env::var(key)
        .ok()
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "on"))
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_load_without_env() {
        let config = AppConfig::from_env();
        assert!(!config.llm.provider.is_empty());
        assert!(!config.runtime.web_bind.is_empty());
    }
}
