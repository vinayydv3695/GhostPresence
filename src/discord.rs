// src/discord.rs

use discord_presence::Client;
use crate::config::Config;
use crate::monitor::TerminalState;

/// Struct to manage the Discord Rich Presence connection.
pub struct DiscordRpc {
    client: Client,
    large_image: String,
    small_image: String,
    last_error_logged: bool,
}

impl DiscordRpc {
    /// Creates a new instance of DiscordRpc.
    pub fn new(config: &Config) -> Self {
        let mut client = Client::new(1429846275737518222);
        let _ = client.start();

        Self {
            client,
            large_image: config.general.large_image.clone(),
            small_image: config.general.small_image.clone(),
            last_error_logged: false,
        }
    }

    /// Updates the Discord Rich Presence with the enhanced terminal state.
    pub fn update_state(&mut self, state: &TerminalState) {
        let details = if state.is_idle {
            "Idle in terminal".to_string()
        } else {
            format!("Running {}", state.command)
        };

        let presence_state = match (&state.git_branch, &state.git_repo) {
            (Some(branch), Some(repo)) => format!("{} (git: {})", repo, branch),
            (Some(branch), None) => format!("in {} (git: {})", state.display_cwd, branch),
            (None, _) => format!("in {}", state.display_cwd),
        };

        let large_image = if self.large_image.is_empty() { "ghostty".to_string() } else { self.large_image.clone() };
        let small_image = if state.small_image_key.is_empty() { self.small_image.clone() } else { state.small_image_key.clone() };

        let result = self.client.set_activity(|activity| {
            activity
                .details(&details)
                .state(&presence_state)
                .assets(|assets| {
                    assets
                        .large_image(&large_image)
                        .large_text("Ghostty Terminal")
                        .small_image(&small_image)
                        .small_text(&state.small_image_text)
                })
                .timestamps(|timestamps| timestamps.start(state.start_time))
        });

        if let Err(e) = result {
            if !self.last_error_logged {
                log::info!("Waiting for Discord desktop app to start (connection error: {})...", e);
                self.last_error_logged = true;
            }
        } else {
            if self.last_error_logged {
                log::info!("Connected to Discord successfully!");
                self.last_error_logged = false;
            }
        }
    }
}