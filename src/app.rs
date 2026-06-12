use std::path::PathBuf;

use crate::cli::{Command, parse_cli};

pub(crate) struct App {
    pub(crate) config_dir: PathBuf,
    pub(crate) data_dir: PathBuf,
    pub(crate) check_github: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            config_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("byi"),
            data_dir: dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".byi"),
            check_github: true,
        }
    }
}

impl App {
    pub(crate) fn run(&self, args: impl IntoIterator<Item = String>) -> Result<String, String> {
        let cli = match parse_cli(args) {
            Ok(cli) => cli,
            Err(message) if is_clap_display_message(&message) => {
                return Ok(message.trim_end().to_string());
            }
            Err(message) => return Err(message),
        };

        match cli.command {
            None | Some(Command::Hello) => Ok(byi_core::hello_message()),
            Some(Command::Sync { command }) => self.run_sync(command),
            Some(Command::Skill { command }) => self.run_skill(command),
            Some(Command::Tui) => byi_tui::run(self.config_dir.clone(), self.data_dir.clone())
                .map(|_| "TUI 已退出".to_string()),
        }
    }

}

fn is_clap_display_message(message: &str) -> bool {
    !message.starts_with("error:")
        && (message.starts_with("Usage:")
            || message.starts_with("byi ")
            || message.contains("\nUsage:"))
}

