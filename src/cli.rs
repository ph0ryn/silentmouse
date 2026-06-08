use clap::{Args, Parser, Subcommand};

use crate::types::DEFAULT_CLICK_DURATION_MS;

#[derive(Debug, Parser)]
#[command(name = "silentmouse")]
#[command(about = "Post a background-capable left click to a macOS window")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Left-click a target window at a window-local coordinate.
    Click(ClickArgs),
}

#[derive(Debug, Args)]
pub struct ClickArgs {
    /// Target CGWindowID.
    #[arg(short = 'w', long = "window-id")]
    pub window_id: u32,

    /// Window-local X coordinate from the target window's top-left.
    #[arg(short = 'x')]
    pub x: f64,

    /// Window-local Y coordinate from the target window's top-left.
    #[arg(short = 'y')]
    pub y: f64,

    /// Mouse-down hold duration in milliseconds.
    #[arg(short = 'd', long = "duration", default_value_t = DEFAULT_CLICK_DURATION_MS)]
    pub duration_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_click_args() {
        let cli = Cli::parse_from([
            "silentmouse",
            "click",
            "--window-id",
            "42",
            "-x",
            "1920",
            "-y",
            "1080.5",
        ]);

        let Commands::Click(args) = cli.command;
        assert_eq!(args.window_id, 42);
        assert_eq!(args.x, 1920.0);
        assert_eq!(args.y, 1080.5);
        assert_eq!(args.duration_ms, DEFAULT_CLICK_DURATION_MS);
    }

    #[test]
    fn parses_short_window_alias() {
        let cli = Cli::parse_from([
            "silentmouse",
            "click",
            "-w",
            "7",
            "-x",
            "1",
            "-y",
            "2",
            "-d",
            "250",
        ]);

        let Commands::Click(args) = cli.command;
        assert_eq!(args.window_id, 7);
        assert_eq!(args.x, 1.0);
        assert_eq!(args.y, 2.0);
        assert_eq!(args.duration_ms, 250);
    }
}
