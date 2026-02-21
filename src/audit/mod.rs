#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditEvent {
    pub actor: String,
    pub action: String,
    pub allowed: bool,
    pub detail: String,
}

#[derive(Debug, Default)]
pub struct AuditLog {
    events: Vec<AuditEvent>,
}

impl AuditLog {
    pub fn push(&mut self, event: AuditEvent) {
        self.events.push(event);
    }

    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }
}
