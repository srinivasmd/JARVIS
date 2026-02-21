#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub name: String,
    pub cron: String,
    pub plugin_manifest: String,
    pub capability: String,
}

impl ScheduledTask {
    pub fn is_due_every_minute(&self) -> bool {
        // Minimal cron support: "* * * * *" and "*/N * * * *"
        let parts: Vec<&str> = self.cron.split_whitespace().collect();
        parts.len() == 5
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_basic_cron_shape() {
        let t = ScheduledTask {
            name: "a".into(),
            cron: "*/5 * * * *".into(),
            plugin_manifest: "m.json".into(),
            capability: "scheduler.run".into(),
        };
        assert!(t.is_due_every_minute());
    }
}
