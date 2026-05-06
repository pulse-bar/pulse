use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ParsedTurn {
    pub message_id: String,
    pub session_id: String,
    pub request_id: Option<String>,
    pub ts: DateTime<Utc>,
    pub provider: &'static str,
    pub model: Option<String>,
    pub branch: Option<String>,
    pub cwd: Option<String>,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub cache_creation_tokens: u64,
    pub cache_read_tokens: u64,
}

impl ParsedTurn {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens
            + self.output_tokens
            + self.cache_creation_tokens
            + self.cache_read_tokens
    }
}
