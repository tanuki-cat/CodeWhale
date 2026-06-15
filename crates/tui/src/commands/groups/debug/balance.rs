//! Balance: query the active provider's account balance or credit status.
//!
//! Providers whose balance/credit API is wired (see
//! [`crate::pricing::balance_endpoint`]) emit an [`AppAction::FetchBalance`]
//! that the UI event loop fulfills over the network — that path is where the
//! live `Config` (and therefore the API key) is available. Providers without a
//! supported balance API get an explicit "not supported" message instead.

use crate::tui::app::{App, AppAction};

use super::CommandResult;

/// Query provider account balance / credits.
pub fn balance(app: &mut App) -> CommandResult {
    if crate::pricing::balance_endpoint(app.api_provider).is_some() {
        CommandResult::action(AppAction::FetchBalance)
    } else {
        CommandResult::message(format!(
            "Balance check is not supported for {} yet. Check the provider dashboard for account balance details.",
            app.api_provider.display_name()
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ApiProvider, Config};
    use crate::tui::app::{App, TuiOptions};
    use std::path::PathBuf;

    fn test_app(provider: ApiProvider) -> App {
        let options = TuiOptions {
            model: "deepseek-v4-pro".to_string(),
            workspace: PathBuf::from("/tmp/test-workspace"),
            config_path: None,
            config_profile: None,
            allow_shell: false,
            use_alt_screen: true,
            use_mouse_capture: false,
            use_bracketed_paste: true,
            max_subagents: 1,
            skills_dir: PathBuf::from("/tmp/test-skills"),
            memory_path: PathBuf::from("memory.md"),
            notes_path: PathBuf::from("notes.txt"),
            mcp_config_path: PathBuf::from("mcp.json"),
            use_memory: false,
            start_in_agent_mode: false,
            skip_onboarding: true,
            yolo: false,
            resume_session_id: None,
            initial_input: None,
        };
        let mut app = App::new(options, &Config::default());
        app.api_provider = provider;
        app
    }

    #[test]
    fn balance_emits_fetch_action_for_supported_provider() {
        let mut app = test_app(ApiProvider::Deepseek);
        let result = balance(&mut app);
        assert!(matches!(result.action, Some(AppAction::FetchBalance)));
        assert!(result.message.is_none());
    }

    #[test]
    fn balance_reports_not_supported_for_local_provider() {
        let mut app = test_app(ApiProvider::Ollama);
        let result = balance(&mut app);
        assert!(result.action.is_none());
        let msg = result.message.expect("message");
        assert!(msg.contains("not supported"), "got: {msg}");
    }
}
