// Reads ~/.claude/projects/<encoded-cwd>/<session>.jsonl. Only `type=="assistant"`
// records carry billable usage; everything else returns None.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::Deserialize;

use pulse_core::ParsedTurn;

use crate::{IngestProvider, ParseError};

pub const PROVIDER_NAME: &str = "claude-code";

#[derive(Clone, Copy)]
pub struct ClaudeCodeProvider;

impl IngestProvider for ClaudeCodeProvider {
    fn name(&self) -> &'static str { PROVIDER_NAME }

    fn watch_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();
        if let Some(home) = dirs::home_dir() {
            roots.push(home.join(".claude").join("projects"));
        }
        roots
    }

    fn parse_line(&self, line: &str) -> Result<Option<ParsedTurn>, ParseError> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        let raw: RawRecord = serde_json::from_str(trimmed)?;
        if raw.r#type.as_deref() != Some("assistant") {
            return Ok(None);
        }
        let Some(message) = raw.message else { return Ok(None) };
        let Some(usage) = message.usage else { return Ok(None) };
        let Some(message_id) = message.id else { return Ok(None) };
        Ok(Some(ParsedTurn {
            message_id,
            session_id: raw.session_id.unwrap_or_else(|| "unknown".into()),
            request_id: raw.request_id,
            ts: raw.timestamp.unwrap_or_else(Utc::now),
            provider: PROVIDER_NAME,
            model: message.model,
            branch: raw.git_branch,
            cwd: raw.cwd,
            input_tokens: usage.input_tokens,
            output_tokens: usage.output_tokens,
            cache_creation_tokens: usage.cache_creation_input_tokens,
            cache_read_tokens: usage.cache_read_input_tokens,
        }))
    }
}

#[derive(Debug, Deserialize)]
struct RawRecord {
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    timestamp: Option<DateTime<Utc>>,
    #[serde(default, rename = "sessionId")]
    session_id: Option<String>,
    #[serde(default, rename = "requestId")]
    request_id: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default, rename = "gitBranch")]
    git_branch: Option<String>,
    #[serde(default)]
    message: Option<RawMessage>,
}

#[derive(Debug, Deserialize)]
struct RawMessage {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    usage: Option<RawUsage>,
}

#[derive(Debug, Deserialize)]
struct RawUsage {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
    #[serde(default)]
    cache_creation_input_tokens: u64,
    #[serde(default)]
    cache_read_input_tokens: u64,
}
