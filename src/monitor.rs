// src/monitor.rs

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use serde::Deserialize;
use crate::config::Config;

#[derive(Debug, Clone, Deserialize)]
pub struct HookState {
    pub cmd: Option<String>,
    pub cwd: Option<String>,
    pub ts: Option<u64>,
    pub running: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct TerminalState {
    pub command: String,
    pub display_cwd: String,
    pub raw_cwd: String,
    pub git_branch: Option<String>,
    pub git_repo: Option<String>,
    pub small_image_key: String,
    pub small_image_text: String,
    pub is_idle: bool,
    pub start_time: u64,
}

/// Reads the last executed shell command from IPC JSON buffer or fallback file.
pub fn get_terminal_state(config: &Config) -> TerminalState {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut raw_cmd = String::new();
    let mut raw_cwd = String::new();
    let mut cmd_ts = now;
    let mut is_running = false;

    // 1. Try reading state from IPC state JSON buffer (/tmp/ghostty_rpc_state.json)
    let json_path = PathBuf::from("/tmp/ghostty_rpc_state.json");
    if json_path.exists() {
        if let Ok(content) = fs::read_to_string(&json_path) {
            if let Ok(state) = serde_json::from_str::<HookState>(&content) {
                if let Some(c) = state.cmd {
                    raw_cmd = c;
                }
                if let Some(w) = state.cwd {
                    raw_cwd = w;
                }
                if let Some(t) = state.ts {
                    cmd_ts = t;
                }
                if let Some(r) = state.running {
                    is_running = r;
                }
            }
        }
    }

    // 2. Fallback to /tmp/ghostty_last_cmd if command was not found
    if raw_cmd.trim().is_empty() {
        if let Ok(content) = fs::read_to_string("/tmp/ghostty_last_cmd") {
            raw_cmd = content.trim().to_string();
        }
    }

    // 3. Fallback to current working directory if missing
    if raw_cwd.trim().is_empty() {
        if let Ok(cwd) = std::env::current_dir() {
            raw_cwd = cwd.to_string_lossy().to_string();
        } else {
            raw_cwd = "Ghostty".to_string();
        }
    }

    if raw_cmd.trim().is_empty() {
        raw_cmd = "zsh".to_string();
    }

    // Check idle condition
    let idle_threshold = config.general.idle_threshold_secs;
    let elapsed = now.saturating_sub(cmd_ts);
    let is_idle = !is_running && elapsed > idle_threshold;

    // Apply command blacklist & exclude filtering
    let safe_cmd = filter_blacklisted_command(&raw_cmd, &config.general.blacklist_commands, &config.general.exclude);

    // Get Git branch and repository info
    let (git_branch, git_repo) = get_git_info(&raw_cwd, config.general.show_git_branch);

    // Format current working directory according to path_display_mode and show_directory
    let display_cwd = format_cwd(&raw_cwd, &config.general.path_display_mode, config.general.show_directory);

    // Map command name to Discord small image asset key
    let (small_image_key, small_image_text) = map_command_to_icon(&safe_cmd);

    TerminalState {
        command: safe_cmd,
        display_cwd,
        raw_cwd,
        git_branch,
        git_repo,
        small_image_key,
        small_image_text,
        is_idle,
        start_time: cmd_ts,
    }
}

/// Checks if command contains blacklisted or excluded keywords and censors it if necessary.
fn filter_blacklisted_command(cmd: &str, blacklist: &[String], exclude: &[String]) -> String {
    let lower = cmd.to_lowercase();
    for item in blacklist.iter().chain(exclude.iter()) {
        if !item.is_empty() && (lower.starts_with(&item.to_lowercase()) || lower.contains(&format!(" {} ", item.to_lowercase()))) {
            return "[protected command]".to_string();
        }
    }
    cmd.to_string()
}

/// Retrieves Git branch name and repository folder name if path is inside a Git repo.
fn get_git_info(cwd: &str, enabled: bool) -> (Option<String>, Option<String>) {
    if !enabled || cwd.is_empty() || cwd == "unknown" {
        return (None, None);
    }

    // Run git rev-parse --abbrev-ref HEAD
    let branch_output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(cwd)
        .output();

    let branch = match branch_output {
        Ok(out) if out.status.success() => {
            let b = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if b.is_empty() || b == "HEAD" { None } else { Some(b) }
        }
        _ => None,
    };

    // Run git rev-parse --show-toplevel to get repo directory name
    let repo_output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output();

    let repo = match repo_output {
        Ok(out) if out.status.success() => {
            let top_path = String::from_utf8_lossy(&out.stdout).trim().to_string();
            Path::new(&top_path)
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
        }
        _ => None,
    };

    (branch, repo)
}

/// Formats current working directory based on configuration preferences.
fn format_cwd(cwd: &str, mode: &str, show_directory: bool) -> String {
    if !show_directory || mode == "hidden" {
        return "Terminal Workspace".to_string();
    }

    match mode {
        "full" => cwd.to_string(),
        _ => {
            // "folder_only" default
            let path = Path::new(cwd);
            if let Some(file_name) = path.file_name() {
                file_name.to_string_lossy().to_string()
            } else {
                cwd.to_string()
            }
        }
    }
}

/// Maps command name to Discord small image asset key and label.
fn map_command_to_icon(cmd: &str) -> (String, String) {
    let first_word = cmd.split_whitespace().next().unwrap_or("terminal").to_lowercase();

    match first_word.as_str() {
        "nvim" | "vim" | "vi" => ("neovim".to_string(), "Neovim".to_string()),
        "cargo" | "rustc" => ("rust".to_string(), "Rust".to_string()),
        "git" | "gh" => ("git".to_string(), "Git".to_string()),
        "python" | "python3" | "pip" | "uv" => ("python".to_string(), "Python".to_string()),
        "node" | "npm" | "npx" | "yarn" | "pnpm" | "bun" => ("nodejs".to_string(), "Node.js".to_string()),
        "docker" | "docker-compose" => ("docker".to_string(), "Docker".to_string()),
        "go" | "gofmt" => ("golang".to_string(), "Go".to_string()),
        "htop" | "btop" | "top" => ("system".to_string(), "System Monitor".to_string()),
        "ssh" => ("ssh".to_string(), "SSH Session".to_string()),
        "zsh" | "bash" | "fish" | "nu" => ("terminal".to_string(), "Ghostty Shell".to_string()),
        _ => ("terminal".to_string(), "Ghostty Terminal".to_string()),
    }
}