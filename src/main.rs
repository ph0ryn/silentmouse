mod cli;
mod error;
mod macos;
mod types;

use clap::Parser;
use cli::{Cli, Commands};
use error::SilentMouseError;
use types::ScreenPoint;

fn main() {
    if let Err(error) = run() {
        eprintln!("silentmouse: {error}");
        std::process::exit(error.exit_code());
    }
}

fn run() -> Result<(), SilentMouseError> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Click(args) => {
            let point = ScreenPoint::new(args.x, args.y)?;
            let result = macos::click_window(args.window_id, point)?;
            println!(
                "clicked window={} pid={} x={} y={} active={} background_flag={} window_location_setter=true",
                result.window_id,
                result.pid,
                format_coord(point.x),
                format_coord(point.y),
                result.target_was_active,
                result.used_background_flag
            );
            Ok(())
        }
    }
}

fn format_coord(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
