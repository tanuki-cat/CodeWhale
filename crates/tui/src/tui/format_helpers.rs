//! Small string builders that compose status-bar / footer chips and
//! one-off informational messages.
//!
//! Each helper is a pure function over a small slice of `App` or
//! response data. Grouped here so the composer/footer renderer doesn't
//! need to scroll past their bodies, and so the labels can be unit
//! tested in isolation.

use crate::models::Usage;
use crate::tui::app::App;

/// Build the multi-line "Cache warmup complete: …" status message
/// shown after a prefix-cache warmup turn finishes. Handles all four
/// combinations of `prompt_cache_hit_tokens` / `prompt_cache_miss_tokens`
/// being present or absent so we never report "0% cache hit" for an
/// API call that didn't surface telemetry at all.
pub(super) fn cache_warmup_result(usage: &Usage) -> String {
    let cache = match (
        usage.prompt_cache_hit_tokens,
        usage.prompt_cache_miss_tokens,
    ) {
        (Some(hit), Some(miss)) => format!("Cache warmup complete: hit {hit} | miss {miss}"),
        (Some(hit), None) => format!("Cache warmup complete: hit {hit} | miss unavailable"),
        (None, Some(miss)) => format!("Cache warmup complete: hit unavailable | miss {miss}"),
        (None, None) => "Cache warmup complete: cache telemetry unavailable".to_string(),
    };
    format!(
        "{cache}\nNote: the first warmup is usually a miss. Later requests that reuse the same stable prefix may hit the provider cache; a hit is not guaranteed."
    )
}

/// Format prefix stability info for the opt-in TUI footer chip.
pub(super) fn prefix_stability_chip(app: &App) -> Option<(String, ratatui::style::Color)> {
    let pct = app.prefix_stability_pct?;
    let changes = app.prefix_change_count;

    let color = if changes == 0 {
        // Perfect stability: green
        ratatui::style::Color::Green
    } else if pct >= 95 {
        // Excellent: green
        ratatui::style::Color::Green
    } else if pct >= 80 {
        // Good: yellow
        ratatui::style::Color::Yellow
    } else {
        // Poor: red — cache is churning
        ratatui::style::Color::Red
    };

    let label = if changes == 0 {
        format!("cache prefix {pct}%")
    } else {
        format!(
            "cache prefix {pct}% ({changes} change{})",
            if changes == 1 { "" } else { "s" }
        )
    };

    Some((label, color))
}

/// Render the response body for `/models` / `models list` — the current
/// model is starred and other available models follow underneath.
pub(super) fn available_models_message(current_model: &str, models: &[String]) -> String {
    let mut lines = vec![format!("Available models ({})", models.len())];
    for model in models {
        if model == current_model {
            lines.push(format!("* {model} (current)"));
        } else {
            lines.push(format!("  {model}"));
        }
    }
    lines.join("\n")
}

/// Render the response body for `/balance` — the active provider's remaining
/// balance / credits, with an optional usage breakdown on a second line.
pub(super) fn balance_message(balance: &crate::pricing::ProviderBalance) -> String {
    let main = format!(
        "{} balance: {}{:.2}",
        balance.provider_label, balance.currency_symbol, balance.amount
    );
    match &balance.detail {
        Some(detail) => format!("{main}\n{detail}"),
        None => main,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_message_renders_amount_and_detail() {
        let balance = crate::pricing::ProviderBalance {
            provider_label: "OpenRouter".to_string(),
            amount: 6.75,
            currency_symbol: "$",
            detail: Some("used $3.25 of $10.00".to_string()),
            footer_info: None,
        };
        let msg = balance_message(&balance);
        assert!(msg.contains("OpenRouter balance: $6.75"), "got: {msg}");
        assert!(msg.contains("used $3.25 of $10.00"), "got: {msg}");
    }

    #[test]
    fn balance_message_without_detail_is_single_line() {
        let balance = crate::pricing::ProviderBalance {
            provider_label: "DeepSeek".to_string(),
            amount: 57.73,
            currency_symbol: "¥",
            detail: None,
            footer_info: None,
        };
        let msg = balance_message(&balance);
        assert_eq!(msg, "DeepSeek balance: ¥57.73");
    }

    #[test]
    fn available_models_message_marks_current_model() {
        let models = vec![
            "deepseek-v4-pro".to_string(),
            "deepseek-v4-flash".to_string(),
        ];
        let msg = available_models_message("deepseek-v4-pro", &models);
        assert!(msg.contains("* deepseek-v4-pro (current)"), "got: {msg}");
        assert!(msg.contains("  deepseek-v4-flash"), "got: {msg}");
        assert!(msg.starts_with("Available models (2)"), "got: {msg}");
    }

    #[test]
    fn cache_warmup_result_handles_missing_telemetry() {
        let usage = Usage {
            prompt_cache_hit_tokens: None,
            prompt_cache_miss_tokens: None,
            ..Default::default()
        };
        let msg = cache_warmup_result(&usage);
        assert!(msg.contains("cache telemetry unavailable"), "got: {msg}");
    }
}
