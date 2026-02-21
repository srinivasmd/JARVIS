use std::env;

use jarvis::{
    benchmark::run_startup_benchmark,
    channels::{ChannelAdapter, CliAdapter},
    core::Agent,
    policy::{Permission, Policy},
    providers::{EchoProvider, ProviderRouter},
    registry::{publish_from_file, PluginRegistryService},
    sandbox::DenyByDefaultSandbox,
    web::run_web_ui_once,
};

fn build_agent() -> Agent {
    let policy = Policy::allow_list([
        Permission::MemoryRead,
        Permission::MemoryWrite,
        Permission::ToolExec,
    ]);
    let router = ProviderRouter::new(vec![Box::new(EchoProvider)]);
    Agent::new(policy, router, Box::new(DenyByDefaultSandbox))
}

fn main() {
    let mut args = env::args().skip(1);
    let command = args.next().unwrap_or_else(|| "chat".to_string());

    match command.as_str() {
        "serve-web" => {
            let mut agent = build_agent();
            if let Err(err) = run_web_ui_once("127.0.0.1:7878", &mut agent) {
                eprintln!("web ui failed: {err}");
                std::process::exit(1);
            }
        }
        "publish-plugin" => {
            let path = args
                .next()
                .unwrap_or_else(|| "examples/plugin-manifest.json".to_string());
            let service = PluginRegistryService::new("registry", "local-dev-secret");
            match publish_from_file(&service, path, "notes", "0.1.0") {
                Ok(published) => println!("published {}:{}", published.name, published.version),
                Err(err) => {
                    eprintln!("publish failed: {err}");
                    std::process::exit(1);
                }
            }
        }
        "bench" => {
            let result = run_startup_benchmark(100);
            println!(
                "startup benchmark: iterations={} avg={}us total={}ms",
                result.iterations,
                result.average_micros,
                result.total.as_millis()
            );
        }
        other => {
            let prompt = if other == "chat" {
                args.next()
                    .unwrap_or_else(|| "Hello from Secure LightClaw v2".to_string())
            } else {
                other.to_string()
            };

            let mut agent = build_agent();
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
