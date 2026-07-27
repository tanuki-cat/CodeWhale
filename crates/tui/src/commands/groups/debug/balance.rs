//! Balance: query the active provider's account balance or credit status.
//!
//! DeepSeek balance is fetched automatically at startup and after each turn
//! completes.  This command reads the cached result and renders it directly.

use crate::tui::app::App;

use super::CommandResult;

/// Show the cached provider account balance.
pub fn balance(app: &mut App) -> CommandResult {
    let info = match app.balance_cell.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => None,
    };
    let Some(info) = info else {
        return CommandResult::message(
            "Balance not yet available — it is fetched automatically after the first turn completes."
                .to_string(),
        );
    };
    let Some(total) = info.total_balance_f64() else {
        return CommandResult::message("Balance response received but could not parse the amount."
            .to_string());
    };
    let symbol = match info.currency.as_str() {
        "CNY" | "cny" => "¥",
        _ => "$",
    };
    let label = if total >= 1000.0 {
        format!("{symbol}{total:.0}")
    } else if total >= 10.0 {
        format!("{symbol}{total:.1}")
    } else {
        format!("{symbol}{total:.2}")
    };
    CommandResult::message(format!(
        "{} account balance: {label}",
        app.api_provider.display_name()
    ))
}
