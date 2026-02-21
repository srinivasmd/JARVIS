use crate::lc_core::Core;

#[derive(Clone)]
pub struct TelegramAdapter {
    token: String,
    chat_id: String,
}

impl TelegramAdapter {
    pub fn new(token: String, chat_id: String) -> Self {
        Self { token, chat_id }
    }

    pub fn poll_once(&self, core: &Core) -> Result<usize, String> {
        // Minimal adapter stub: in production this should call Telegram Bot API.
        let _ = (&self.token, &self.chat_id);
        let _ = core.handle_message("telegram-test")?;
        Ok(1)
    }
}
