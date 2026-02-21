use crate::lc_provider::{LlmProvider, LlmRequest};
use crate::lc_sandbox::WasmSandbox;
use crate::lc_scheduler::ScheduledTask;
use std::collections::BTreeMap;
use std::fs;
use std::sync::{Arc, Mutex};

pub struct Core {
    pub provider: Arc<dyn LlmProvider>,
    pub sandbox: WasmSandbox,
    memory: Mutex<BTreeMap<String, String>>, // demo encrypted store wrapper
    encryption_key: u8,
}

impl Core {
    pub fn new(
        provider: Arc<dyn LlmProvider>,
        _db_path: &str,
        encryption_key: &str,
    ) -> Result<Self, String> {
        let key = encryption_key.bytes().fold(0u8, |acc, b| acc ^ b);
        Ok(Self {
            provider,
            sandbox: WasmSandbox,
            memory: Mutex::new(BTreeMap::new()),
            encryption_key: key,
        })
    }

    pub fn handle_message(&self, input: &str) -> Result<String, String> {
        let answer = self.provider.generate(LlmRequest {
            prompt: input.to_string(),
        })?;
        self.put_memory("last_user_message", input)?;
        self.put_memory("last_assistant_message", &answer.text)?;
        Ok(answer.text)
    }

    pub fn put_memory(&self, key: &str, value: &str) -> Result<(), String> {
        let encrypted: String = value
            .bytes()
            .map(|b| (b ^ self.encryption_key) as char)
            .collect();
        self.memory
            .lock()
            .map_err(|_| "memory lock")?
            .insert(key.into(), encrypted);
        Ok(())
    }

    pub fn get_memory(&self, key: &str) -> Result<Option<String>, String> {
        let map = self.memory.lock().map_err(|_| "memory lock")?;
        Ok(map.get(key).map(|v| {
            v.bytes()
                .map(|b| (b ^ self.encryption_key) as char)
                .collect()
        }))
    }

    pub fn run_scheduler_once(&self, tasks: &[ScheduledTask]) -> Result<usize, String> {
        let mut fired = 0usize;
        for task in tasks {
            if task.is_due_every_minute() {
                let manifest =
                    crate::lc_sandbox::WasmSandbox::load_manifest(&task.plugin_manifest)?;
                let _ = self.sandbox.execute(&manifest, &task.capability)?;
                fired += 1;
            }
        }
        Ok(fired)
    }

    pub fn load_tasks(path: &str) -> Result<Vec<ScheduledTask>, String> {
        let src = fs::read_to_string(path).map_err(|e| e.to_string())?;
        let mut tasks = Vec::new();
        for chunk in src.split("[[tasks]]").skip(1) {
            let name = read_toml_value(chunk, "name").unwrap_or_else(|| "task".into());
            let cron = read_toml_value(chunk, "cron").unwrap_or_else(|| "* * * * *".into());
            let plugin_manifest = read_toml_value(chunk, "plugin_manifest").unwrap_or_default();
            let capability = read_toml_value(chunk, "capability").unwrap_or_default();
            tasks.push(ScheduledTask {
                name,
                cron,
                plugin_manifest,
                capability,
            });
        }
        Ok(tasks)
    }
}

fn read_toml_value(section: &str, key: &str) -> Option<String> {
    section
        .lines()
        .find(|line| line.trim_start().starts_with(&format!("{key} =")))
        .and_then(|line| line.split_once('='))
        .map(|(_, value)| value.trim().trim_matches('"').to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lc_provider::MockProvider;

    #[test]
    fn memory_round_trip() {
        let core = Core::new(Arc::new(MockProvider), ":memory:", "dev-secret").expect("core");
        core.put_memory("a", "b").expect("put");
        assert_eq!(core.get_memory("a").expect("get"), Some("b".into()));
        let out = core.handle_message("hello").expect("handle");
        assert!(out.contains("echo"));
    }
}
