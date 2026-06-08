mod cli;
mod error;
mod macos;
mod types;

use clap::Parser;
use cli::{Cli, Commands, DragArgs, MouseCommands, MouseEventArgs};
use error::SilentMouseError;
use std::time::Duration;
use types::WindowPoint;

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
            let point = WindowPoint::new(args.x, args.y)?;
            let duration = Duration::from_millis(args.duration_ms);
            let result = macos::click_window(args.window_id, point, duration)?;
            println!(
                "clicked window={} pid={} local_x={} local_y={} duration_ms={} active={} background_flag={} window_location_setter=true",
                result.window_id,
                result.pid,
                format_coord(point.x),
                format_coord(point.y),
                args.duration_ms,
                result.target_was_active,
                result.used_background_flag
            );
            Ok(())
        }
        Commands::Drag(args) => drag(args),
        Commands::Mouse(args) => {
            let (kind, event_args) = match args.command {
                MouseCommands::Move(args) => (macos::MouseEventKind::Move, args),
                MouseCommands::Down(args) => (macos::MouseEventKind::Down, args),
                MouseCommands::Drag(args) => (macos::MouseEventKind::Drag, args),
                MouseCommands::Up(args) => (macos::MouseEventKind::Up, args),
            };
            post_mouse(kind, event_args)
        }
    }
}

fn drag(args: DragArgs) -> Result<(), SilentMouseError> {
    let from = WindowPoint::new(args.from_x, args.from_y)?;
    let to = WindowPoint::new(args.to_x, args.to_y)?;
    let duration = Duration::from_millis(args.duration_ms);
    let result = macos::drag_window(args.window_id, from, to, duration)?;
    println!(
        "dragged window={} pid={} from_x={} from_y={} to_x={} to_y={} duration_ms={} steps={} active={} background_flag={} window_location_setter=true",
        result.window_id,
        result.pid,
        format_coord(result.from.x),
        format_coord(result.from.y),
        format_coord(result.to.x),
        format_coord(result.to.y),
        result.duration.as_millis(),
        result.drag_steps,
        result.target_was_active,
        result.used_background_flag
    );
    Ok(())
}

fn post_mouse(kind: macos::MouseEventKind, args: MouseEventArgs) -> Result<(), SilentMouseError> {
    let point = WindowPoint::new(args.x, args.y)?;
    let result = macos::post_mouse_event(args.window_id, point, kind)?;
    println!(
        "posted mouse {} window={} pid={} local_x={} local_y={} active={} background_flag={} window_location_setter=true",
        result.event_name,
        result.window_id,
        result.pid,
        format_coord(point.x),
        format_coord(point.y),
        result.target_was_active,
        result.used_background_flag
    );
    Ok(())
}

fn format_coord(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}
