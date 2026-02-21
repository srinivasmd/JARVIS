use std::env;

use jarvis::{
    benchmark::run_startup_benchmark,
    channels::{ChannelAdapter, CliAdapter},
    config::AppConfig,
    core::Agent,
    policy::{Permission, Policy},
    providers::{provider_from_config, ProviderRouter},
    registry::{publish_from_file, PluginRegistryService},
    sandbox::DenyByDefaultSandbox,
    web::run_web_ui_once,
};

fn build_agent(config: &AppConfig) -> Agent {
    let policy = Policy::allow_list([
        Permission::MemoryRead,
        Permission::MemoryWrite,
        Permission::ToolExec,
    ]);
    let router = ProviderRouter::new(vec![provider_from_config(&config.llm)]);
    Agent::new(policy, router, Box::new(DenyByDefaultSandbox))
}

fn main() {
    let mut args = env::args().skip(1);
    let config = AppConfig::from_env();

    if let Err(err) = config.validate_telegram() {
        eprintln!("config error: {err}");
        std::process::exit(1);
    }

    let command = args
        .next()
        .unwrap_or_else(|| config.runtime.default_mode.clone());

    match command.as_str() {
        "serve-web" => {
            let mut agent = build_agent(&config);
            if let Err(err) = run_web_ui_once(&config.runtime.web_bind, &mut agent) {
                eprintln!("web ui failed: {err}");
                std::process::exit(1);
            }
        }
        "publish-plugin" => {
            let path = args
                .next()
                .unwrap_or_else(|| "examples/plugin-manifest.json".to_string());
            let service = PluginRegistryService::new(
                config.runtime.registry_path.clone(),
                config.runtime.registry_secret.clone(),
            );
            match publish_from_file(&service, path, "notes", "0.1.0") {
                Ok(published) => println!("published {}:{}", published.name, published.version),
                Err(err) => {
                    eprintln!("publish failed: {err}");
                    std::process::exit(1);
                }
            }
        }
        "bench" => {
            let result = run_startup_benchmark(config.runtime.benchmark_iterations);
            println!(
                "startup benchmark: iterations={} avg={}us total={}ms",
                result.iterations,
                result.average_micros,
                result.total.as_millis()
            );
        }
        "show-config" => {
            println!("{}", config.redacted_summary());
            println!("llm_api_base_set={}", config.llm.api_base.is_some());
            println!("llm_api_key_set={}", config.llm.api_key.is_some());
            println!(
                "telegram_webhook_set={}",
                config.telegram.webhook_url.is_some()
            );
        }
        other => {
            let prompt = if other == "chat" {
                args.next()
                    .unwrap_or_else(|| "Hello from Secure LightClaw v2".to_string())
            } else {
                other.to_string()
            };

            let mut agent = build_agent(&config);
            let adapter = CliAdapter;
            let incoming = adapter.normalize("local-user", &prompt);

            match agent.handle_message(incoming) {
                Ok(response) => {
                    println!("{}", response.text);
                    let tool_result = agent
                        .run_tool("echo", &["tool".to_string(), "ok".to_string()])
                        .expect("tool should run when permission granted");
                    println!("tool: {tool_result}");
                }
                Err(err) => {
                    eprintln!("agent failed: {err}");
                    std::process::exit(1);
                }
            }
        }
    }
}
