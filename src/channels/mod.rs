#[derive(Debug, Clone)]
pub struct IncomingMessage {
    pub channel: String,
    pub user: String,
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct OutgoingMessage {
    pub channel: String,
    pub text: String,
}

pub trait ChannelAdapter {
    fn name(&self) -> &str;
    fn normalize(&self, user: &str, text: &str) -> IncomingMessage;
}

#[derive(Debug, Default)]
pub struct CliAdapter;

impl ChannelAdapter for CliAdapter {
    fn name(&self) -> &str {
        "cli"
    }

    fn normalize(&self, user: &str, text: &str) -> IncomingMessage {
        IncomingMessage {
            channel: self.name().to_string(),
            user: user.to_string(),
            text: text.to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct WebhookAdapter;

impl ChannelAdapter for WebhookAdapter {
    fn name(&self) -> &str {
        "webhook"
    }

    fn normalize(&self, user: &str, text: &str) -> IncomingMessage {
        IncomingMessage {
            channel: self.name().to_string(),
            user: user.to_string(),
            text: text.to_string(),
        }
    }
}
