// src/config.rs

use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub general: GeneralConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct GeneralConfig {
    #[serde(default = "default_refresh_interval_ms")]
    pub refresh_interval_ms: u64,

    #[serde(default = "default_show_directory")]
    pub show_directory: bool,


    #[serde(default = "default_path_display_mode")]
    pub path_display_mode: String,

    #[serde(default = "default_show_git_branch")]
    pub show_git_branch: bool,

    #[serde(default = "default_idle_threshold")]
    pub idle_threshold_secs: u64,

    #[serde(default)]
    pub exclude: Vec<String>,

    #[serde(default)]
    pub blacklist_commands: Vec<String>,

    #[serde(default = "default_client_id")]
    pub client_id: String,

    #[serde(default = "default_large_image")]
    pub large_image: String,

    #[serde(default = "default_small_image")]
    pub small_image: String,
}

fn default_client_id() -> String { "1429846275737518222".to_string() }


fn default_refresh_interval_ms() -> u64 { 500 }
fn default_show_directory() -> bool { true }
fn default_path_display_mode() -> String { "folder_only".to_string() }
fn default_show_git_branch() -> bool { true }
fn default_idle_threshold() -> u64 { 300 }
fn default_large_image() -> String { "ghostty".to_string() }
fn default_small_image() -> String { "terminal".to_string() }

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig {
                refresh_interval_ms: default_refresh_interval_ms(),

                show_directory: default_show_directory(),
                path_display_mode: default_path_display_mode(),
                show_git_branch: default_show_git_branch(),
                idle_threshold_secs: default_idle_threshold(),
                exclude: Vec::new(),
                blacklist_commands: vec![
                    "sudo".to_string(),
                    "ssh".to_string(),
                    "op".to_string(),
                    "bw".to_string(),
                    "pass".to_string(),
                ],
                client_id: default_client_id(),
                large_image: default_large_image(),
                small_image: default_small_image(),

            },
        }
    }
}


impl Config {
    /// Loads the configuration from the specified TOML file, falling back to default if missing.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = match get_config_path() {
            Ok(p) => p,
            Err(_) => return Ok(Config::default()),
        };
        if !config_path.exists() {
            return Ok(Config::default());
        }
        let config_content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_content).unwrap_or_else(|_| Config::default());
        Ok(config)
    }
}

/// Loads the configuration from a specific path.
pub fn load_config(path: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let config_content = fs::read_to_string(path)?;
    let config: Config = toml::from_str(&config_content)?;
    Ok(config)
}



/// Gets the path to the configuration file.
fn get_config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let project_dirs = ProjectDirs::from("com", "your_name", "ghostty-rpc")
        .ok_or("Unable to find project directories")?;
    let config_dir = project_dirs.config_dir();
    let config_file = config_dir.join("config.toml");

    // Create the config directory if it doesn't exist
    if !config_dir.exists() {
        std::fs::create_dir_all(config_dir)?;
    }

    Ok(config_file)
}