// src/main.rs

use clap::{Command, Arg};
use env_logger::Env;
use log::{error, info};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use signal_hook::consts::signal::*;
use signal_hook::flag;

mod config;
mod discord;
mod monitor;

fn main() {
    // Set up command-line interface
    let matches = Command::new("Ghostty RPC")
        .version("1.1.0")
        .about("Updates Discord Rich Presence with Ghostty terminal activity & Git status")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Path to custom configuration file"),
        )
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .action(clap::ArgAction::SetTrue)
                .help("Enable debug logging level"),
        )
        .arg(
            Arg::new("once")
                .short('o')
                .long("once")
                .action(clap::ArgAction::SetTrue)
                .help("Run presence update once and exit"),
        )
        .arg(
            Arg::new("interval")
                .short('i')
                .long("interval")
                .value_name("SECONDS")
                .help("Update refresh interval in seconds"),
        )
        .arg(
            Arg::new("status")
                .long("status")
                .action(clap::ArgAction::SetTrue)
                .help("Print current detected Ghostty terminal status and exit"),
        )
        .arg(
            Arg::new("install-hooks")
                .long("install-hooks")
                .action(clap::ArgAction::SetTrue)
                .help("Generate and print shell integration hook scripts for zsh, bash, fish"),
        )
        .get_matches();

    // Initialize logging
    let log_level = if matches.get_flag("debug") { "debug" } else { "info" };
    env_logger::init_from_env(Env::default().default_filter_or(log_level));

    // Handle --install-hooks
    if matches.get_flag("install-hooks") {
        print_shell_hooks();
        return;
    }

    // Load configuration
    let config = if let Some(config_path) = matches.get_one::<String>("config") {
        config::load_config(PathBuf::from(config_path)).unwrap_or_else(|err| {
            error!("Failed to load config file: {}", err);
            std::process::exit(1);
        })
    } else {
        config::Config::load().unwrap_or_else(|err| {
            error!("Failed to load config: {}", err);
            config::Config::default()
        })
    };

    // Handle --status
    if matches.get_flag("status") {
        let state = monitor::get_terminal_state(&config);
        println!("=== Ghostty RPC Terminal Status ===");
        println!("Command:       {}", state.command);
        println!("Display CWD:   {}", state.display_cwd);
        println!("Raw CWD:       {}", state.raw_cwd);
        println!("Git Branch:    {}", state.git_branch.as_deref().unwrap_or("none"));
        println!("Git Repo:      {}", state.git_repo.as_deref().unwrap_or("none"));
        println!("Icon Key:      {}", state.small_image_key);
        println!("Is Idle:       {}", state.is_idle);
        println!("Start Time:    {}", state.start_time);
        return;
    }

    // Initialize Discord RPC client
    let mut discord_rpc = discord::DiscordRpc::new(&config);

    // Register signal handlers (SIGTERM, SIGINT, SIGHUP) for clean termination
    let term = Arc::new(AtomicBool::new(false));
    let _ = flag::register(SIGTERM, Arc::clone(&term));
    let _ = flag::register(SIGINT, Arc::clone(&term));
    let _ = flag::register(SIGHUP, Arc::clone(&term));

    // Daemon loop
    let interval = matches
        .get_one::<String>("interval")
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(config.general.refresh_interval);

    info!("Starting Ghostty RPC daemon (interval: {}s)...", interval);

    loop {
        // Read current terminal activity
        let state = monitor::get_terminal_state(&config);

        // Update Discord Rich Presence
        discord_rpc.update_state(&state);

        // Log update summary
        info!(
            "Updated Discord RPC -> Cmd: '{}' | CWD: '{}' | Git: {}",
            state.command,
            state.display_cwd,
            state.git_branch.as_deref().unwrap_or("none")
        );

        if term.load(Ordering::Relaxed) {
            info!("Termination signal received, exiting Ghostty RPC.");
            break;
        }

        if matches.get_flag("once") {
            break;
        }

        for _ in 0..interval {
            if term.load(Ordering::Relaxed) {
                break;
            }
            thread::sleep(Duration::from_secs(1));
        }
    }
}

/// Print shell integration hook scripts for Zsh, Bash, Fish, and Nushell.
fn print_shell_hooks() {
    print!("{}", r#"# Ghostty RPC Shell Integration Hooks
# Add the relevant block to your shell configuration file (~/.zshrc or ~/.bashrc).

# --- ZSH (~/.zshrc) ---
ghostty_rpc_preexec() {
  local cmd="$1"
  local cwd="$PWD"
  local ts=$(date +%s)
  printf '{"cmd":"%s","cwd":"%s","ts":%s,"running":true}' "$cmd" "$cwd" "$ts" > /tmp/ghostty_rpc_state.json 2>/dev/null
}

ghostty_rpc_precmd() {
  local cwd="$PWD"
  local ts=$(date +%s)
  printf '{"cmd":"zsh","cwd":"%s","ts":%s,"running":false}' "$cwd" "$ts" > /tmp/ghostty_rpc_state.json 2>/dev/null
}

autoload -Uz add-zsh-hook
add-zsh-hook preexec ghostty_rpc_preexec
add-zsh-hook precmd ghostty_rpc_precmd
"#);
}