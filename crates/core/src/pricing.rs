// USD per 1M tokens. Move to settings JSON before exposing custom models.

use crate::turn::ParsedTurn;

#[derive(Clone, Copy, Debug)]
pub struct ModelPrice {
    pub input: f64,
    pub output: f64,
    pub cache_read: f64,
    pub cache_write: f64,
}

pub fn price_for(model: Option<&str>) -> ModelPrice {
    let m = model.unwrap_or("claude-sonnet-4-6");
    if m.contains("opus") {
        ModelPrice { input: 15.0, output: 75.0, cache_read: 1.5, cache_write: 18.75 }
    } else if m.contains("haiku") {
        ModelPrice { input: 0.8, output: 4.0, cache_read: 0.08, cache_write: 1.0 }
    } else {
        ModelPrice { input: 3.0, output: 15.0, cache_read: 0.3, cache_write: 3.75 }
    }
}

pub fn cost_of(turn: &ParsedTurn) -> f64 {
    let p = price_for(turn.model.as_deref());
    let m = 1_000_000.0;
    (turn.input_tokens as f64) * p.input / m
        + (turn.output_tokens as f64) * p.output / m
        + (turn.cache_creation_tokens as f64) * p.cache_write / m
        + (turn.cache_read_tokens as f64) * p.cache_read / m
}
