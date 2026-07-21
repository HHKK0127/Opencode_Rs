//! Model pricing for cost estimation.

use crate::config::ModelFamily;

/// Per-million-token prices in USD.
#[derive(Debug, Clone, Copy)]
pub struct ModelPricing {
    /// Input token price (USD / 1M tokens).
    pub input_per_million: f64,
    /// Output token price (USD / 1M tokens).
    pub output_per_million: f64,
    /// Cached read price (USD / 1M tokens). `0.0` if unsupported.
    pub cache_read_per_million: f64,
    /// Cache write price (USD / 1M tokens). `0.0` if unsupported.
    pub cache_write_per_million: f64,
}

/// Resolve pricing for a model identifier. Returns `None` if unknown.
pub fn pricing_for_model(model: &str) -> Option<ModelPricing> {
    let m = model.to_ascii_lowercase();
    let family = ModelFamily::from_model(&m);
    Some(match family {
        ModelFamily::Opus4 => ModelPricing {
            input_per_million: 15.0,
            output_per_million: 75.0,
            cache_read_per_million: 1.5,
            cache_write_per_million: 18.75,
        },
        ModelFamily::Sonnet4 => ModelPricing {
            input_per_million: 3.0,
            output_per_million: 15.0,
            cache_read_per_million: 0.30,
            cache_write_per_million: 3.75,
        },
        ModelFamily::Haiku45 => ModelPricing {
            input_per_million: 1.0,
            output_per_million: 5.0,
            cache_read_per_million: 0.10,
            cache_write_per_million: 1.25,
        },
        ModelFamily::Gpt4 => ModelPricing {
            input_per_million: 5.0,
            output_per_million: 15.0,
            cache_read_per_million: 0.0,
            cache_write_per_million: 0.0,
        },
        ModelFamily::Reasoning => ModelPricing {
            input_per_million: 10.0,
            output_per_million: 40.0,
            cache_read_per_million: 0.0,
            cache_write_per_million: 0.0,
        },
        ModelFamily::Other => return None,
    })
}

/// Cost estimate for a [`Usage`](crate::types::Usage) block.
#[derive(Debug, Clone, Copy, Default)]
pub struct UsageCostEstimate {
    /// Input cost in USD.
    pub input_usd: f64,
    /// Output cost in USD.
    pub output_usd: f64,
    /// Cache-read cost in USD.
    pub cache_read_usd: f64,
    /// Cache-write cost in USD.
    pub cache_write_usd: f64,
}

impl UsageCostEstimate {
    /// Total cost in USD.
    pub fn total_usd(&self) -> f64 {
        self.input_usd + self.output_usd + self.cache_read_usd + self.cache_write_usd
    }
}

/// Format a USD amount to 4 decimal places.
pub fn format_usd(amount: f64) -> String {
    format!("${amount:.4}")
}

/// Compute a cost estimate for a usage block on a given model.
pub fn estimate_cost(model: &str, usage: &crate::types::Usage) -> UsageCostEstimate {
    let Some(pricing) = pricing_for_model(model) else {
        return UsageCostEstimate::default();
    };
    let f = |tokens: u32, per_million: f64| (tokens as f64 / 1_000_000.0) * per_million;
    UsageCostEstimate {
        input_usd: f(usage.input_tokens, pricing.input_per_million),
        output_usd: f(usage.output_tokens, pricing.output_per_million),
        cache_read_usd: f(
            usage.cache_read_input_tokens,
            pricing.cache_read_per_million,
        ),
        cache_write_usd: f(
            usage.cache_creation_input_tokens,
            pricing.cache_write_per_million,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Usage;

    #[test]
    fn opus_pricing() {
        let u = Usage {
            input_tokens: 1_000_000,
            output_tokens: 1_000_000,
            ..Default::default()
        };
        let cost = estimate_cost("claude-opus-4-6", &u);
        assert!((cost.input_usd - 15.0).abs() < 1e-6);
        assert!((cost.output_usd - 75.0).abs() < 1e-6);
    }
}
