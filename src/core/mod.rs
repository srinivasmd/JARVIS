use crate::{
    audit::{AuditEvent, AuditLog},
    channels::{IncomingMessage, OutgoingMessage},
    memory::MemoryStore,
    plugins::PluginRegistry,
    policy::{Permission, Policy},
    providers::{ProviderRequest, ProviderRouter},
    sandbox::{ToolRequest, ToolSandbox},
    scheduler::Scheduler,
};

pub struct Agent {
    policy: Policy,
    memory: MemoryStore,
    plugins: PluginRegistry,
    router: ProviderRouter,
    audit: AuditLog,
    scheduler: Scheduler,
    sandbox: Box<dyn ToolSandbox>,
}

#[derive(Debug)]
pub enum AgentError {
    Provider(crate::providers::ProviderError),
    Sandbox(crate::sandbox::SandboxError),
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentError::Provider(err) => write!(f, "provider error: {err}"),
            AgentError::Sandbox(err) => write!(f, "sandbox error: {err:?}"),
        }
    }
}

impl std::error::Error for AgentError {}

impl From<crate::providers::ProviderError> for AgentError {
    fn from(value: crate::providers::ProviderError) -> Self {
        AgentError::Provider(value)
    }
}

impl From<crate::sandbox::SandboxError> for AgentError {
    fn from(value: crate::sandbox::SandboxError) -> Self {
        AgentError::Sandbox(value)
    }
}

impl Agent {
    pub fn new(policy: Policy, router: ProviderRouter, sandbox: Box<dyn ToolSandbox>) -> Self {
        Self {
            policy,
            memory: MemoryStore::default(),
            plugins: PluginRegistry::new(false),
            router,
            audit: AuditLog::default(),
            scheduler: Scheduler::default(),
            sandbox,
        }
    }

    pub fn register_plugins(&mut self, registry: PluginRegistry) {
        self.plugins = registry;
    }

    pub fn add_schedule(&mut self, name: &str, interval_ticks: u64, payload: &str) {
        self.scheduler
            .add_task(name.to_string(), interval_ticks, payload.to_string());
    }

    pub fn heartbeat(&mut self) -> Vec<String> {
        self.scheduler
            .heartbeat()
            .into_iter()
            .map(|task| task.payload)
            .collect()
    }

    pub fn handle_message(
        &mut self,
        message: IncomingMessage,
    ) -> Result<OutgoingMessage, AgentError> {
        let response = self.run_prompt(message.text)?;
        Ok(OutgoingMessage {
            channel: message.channel,
            text: response,
        })
    }

    pub fn run_tool(&mut self, name: &str, args: &[String]) -> Result<String, AgentError> {
        let result = self.sandbox.execute(
            &self.policy,
            &ToolRequest {
                name: name.to_string(),
                args: args.to_vec(),
            },
        )?;

        self.audit.push(AuditEvent {
            actor: "agent".to_string(),
            action: format!("tool_exec:{name}"),
            allowed: true,
            detail: "tool execution approved by policy".to_string(),
        });

        Ok(result)
    }

    pub fn run_prompt(&mut self, input: impl Into<String>) -> Result<String, AgentError> {
        let input = input.into();
        let mut context = Vec::new();

        if self.policy.allows(&Permission::MemoryRead) {
            if let Some(last) = self.memory.get("last_prompt") {
                context.push(last.to_string());
            }
        }

        let response = self.router.generate(&ProviderRequest {
            user_input: input.clone(),
            context,
        })?;

        if self.policy.allows(&Permission::MemoryWrite) {
            self.memory.put("last_prompt", input.clone());
        } else {
            self.audit.push(AuditEvent {
                actor: "agent".to_string(),
                action: "memory_write".to_string(),
                allowed: false,
                detail: "Skipped storing prompt because memory_write permission is disabled"
                    .to_string(),
            });
        }

        self.audit.push(AuditEvent {
            actor: "agent".to_string(),
            action: "provider_generate".to_string(),
            allowed: true,
            detail: format!(
                "Generated response via {} with {} plugins loaded",
                response.model,
                self.plugins.all().len()
            ),
        });

        Ok(response.output)
    }

    pub fn audit_events(&self) -> &[AuditEvent] {
        self.audit.events()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        plugins::{ManifestPermission, PluginManifest, PluginRegistry, PluginSource},
        providers::ConfiguredEchoProvider,
        sandbox::DenyByDefaultSandbox,
    };

    #[test]
    fn agent_generates_response_and_audits() {
        let policy = Policy::allow_list([Permission::MemoryRead, Permission::MemoryWrite]);
        let router = ProviderRouter::new(vec![Box::new(ConfiguredEchoProvider::new(
            "echo-local",
            "mock-1",
        ))]);
        let mut agent = Agent::new(policy, router, Box::new(DenyByDefaultSandbox));

        let mut registry = PluginRegistry::new(false);
        registry
            .register(PluginManifest {
                name: "notes".to_string(),
                version: "0.1.0".to_string(),
                capabilities: vec!["store_notes".to_string()],
                permissions: vec![ManifestPermission::MemoryRead],
                source: PluginSource::VettedRegistry,
            })
            .expect("plugin should register");
        agent.register_plugins(registry);

        let out = agent.run_prompt("hello").expect("agent should respond");
        assert_eq!(out, "hello");
        assert_eq!(agent.audit_events().len(), 1);
    }

    #[test]
    fn heartbeat_runs_scheduled_task() {
        let policy = Policy::allow_list([Permission::MemoryRead]);
        let router = ProviderRouter::new(vec![Box::new(ConfiguredEchoProvider::new(
            "echo-local",
            "mock-1",
        ))]);
        let mut agent = Agent::new(policy, router, Box::new(DenyByDefaultSandbox));
        agent.add_schedule("tick", 1, "cron:sync");

        let due = agent.heartbeat();
        assert_eq!(due, vec!["cron:sync".to_string()]);
    }
}
