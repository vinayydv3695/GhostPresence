# Ghostty RPC - Discord Rich Presence Integration

Ghostty RPC is a fast, Rust-based application that integrates with Discord's Rich Presence feature, providing real-time updates of your Ghostty terminal activity, Git branch & repository, active process, and idle status directly on Discord.

![Ghostty Logo](assets/ghostty.png)

## Features

- **Real-time Terminal Activity**: Displays current running process (`nvim`, `cargo`, `git`, `python`, `node`, `docker`, etc.).
- **Git Branch & Repository Awareness**: Auto-detects Git repositories and active branch names (`git: main 🌿`).
- **Dynamic Tool Icons**: Maps command names to Discord small image asset keys.
- **Smart Idle Detection**: Automatically switches status to "Idle in terminal" after custom inactivity thresholds.
- **Privacy & Censor Filters**: Redacts sensitive commands (`sudo`, `ssh`, `op`, `bw`) and supports path display modes (`folder_only`, `full`, `hidden`).
- **Shell Integration Hooks**: Shell hook generator (`ghostty-rpc --install-hooks`) for Zsh, Bash, Fish, and Nushell.
- **Official Logo Assets**: Included high-resolution SVG and 512x512 PNG Ghostty logo assets in `assets/`.
- **Systemd Integration**: User-level service for automatic daemon background execution.

---

## Official Logo Assets

The official Ghostty logo assets are included in `assets/`:
- **Vector Logo**: [assets/ghostty.svg](assets/ghostty.svg)
- **High-Res PNG**: [assets/ghostty.png](assets/ghostty.png)

> **Discord App Setup**: Upload `ghostty.png` as an asset named `ghostty` under your Discord Application (ID: `1429846275737518222`) in the Discord Developer Portal so it renders as the large image in Rich Presence.

---

## Quick Start & Shell Hook Setup

1. Build & install `ghostty-rpc`:
   ```bash
   cargo build --release
   sudo cp target/release/ghostty-rpc /usr/local/bin/
   ```

2. Print shell integration hooks for your shell:
   ```bash
   ghostty-rpc --install-hooks
   ```

3. Add the output hook snippet to your `~/.zshrc`, `~/.bashrc`, or `~/.config/fish/config.fish`.

4. Check detected status:
   ```bash
   ghostty-rpc --status
   ```

---

## CLI Usage

```bash
ghostty-rpc [OPTIONS]
```

### Options

- `-c, --config <FILE>`: Specify path to a custom configuration TOML file.
- `-d, --debug`: Enable verbose debug logging.
- `-o, --once`: Run presence update once and exit.
- `-i, --interval <SECS>`: Set refresh update interval in seconds.
- `--status`: Print current detected Ghostty terminal status and exit.
- `--install-hooks`: Print shell integration hook scripts.

---

## Configuration

Configuration file is located at `~/.config/ghostty-rpc/config.toml`. See [assets/config.toml.example](assets/config.toml.example).

```toml
[general]
refresh_interval = 5
show_directory = true
path_display_mode = "folder_only"   # "folder_only", "full", or "hidden"
show_git_branch = true
idle_threshold_secs = 300
blacklist_commands = ["sudo", "ssh", "op", "bw", "pass"]
large_image = "ghostty"
small_image = "terminal"
```

---

## Service Management

Manage the Ghostty RPC service via systemd:

```bash
# Enable & start user service
systemctl --user enable ghostty-rpc.service
systemctl --user start ghostty-rpc.service

# Check service status
systemctl --user status ghostty-rpc.service
```

---

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.
