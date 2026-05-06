use regex::Regex;

use pulse_core::{AttributionConfidence, AttributionOutcome, ParsedTurn, Settings};

use crate::AttributionProvider;

#[derive(Clone, Copy)]
pub struct CwdProvider;

impl AttributionProvider for CwdProvider {
    fn name(&self) -> &'static str { "cwd" }
    fn priority(&self) -> i32 { 50 }

    fn try_attribute(&self, turn: &ParsedTurn, _settings: &Settings) -> Option<AttributionOutcome> {
        let cwd = turn.cwd.as_deref()?;
        let rx = Regex::new(r"(?i)([A-Z][A-Z0-9]+-\d+)").ok()?;
        let captured = rx.captures(cwd)?.get(1)?.as_str().to_uppercase();
        Some(AttributionOutcome {
            task_id: Some(captured),
            confidence: AttributionConfidence::Medium,
            score: 0.75,
        })
    }
}
