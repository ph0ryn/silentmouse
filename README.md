# silentmouse

`silentmouse` is a macOS-only CLI that posts mouse events to a target
`CGWindowID` without requiring the target app to be focused.

It is a thin wrapper around the macOS event delivery path used by AppKit and
CoreGraphics. Coordinates are window-local: `(0, 0)` is the top-left corner of
the target window.

## Install

```sh
cargo install --git https://github.com/ph0ryn/silentmouse.git
```

## Usage

```sh
silentmouse mouse move -w 1 -x 250 -y 250
silentmouse mouse down -w 1 -x 250 -y 250
silentmouse mouse drag -w 1 -x 300 -y 300
silentmouse mouse up -w 1 -x 300 -y 300

silentmouse drag -w 1 --from-x 250 --from-y 250 --to-x 300 --to-y 300 -d 500

silentmouse click --window-id 1 -x 250 -y 250
silentmouse click -w 1 -x 250 -y 250
silentmouse click -w 1 -x 250 -y 250 --duration 200
```

The `mouse` command is the raw event API. It does not store state; callers own
event sequencing, drag state, interpolation, retries, and target selection.

`click` and `drag` are convenience commands built from raw mouse events.
`click --duration` / `-d` controls the milliseconds between mouse down and mouse
up, and defaults to `35`. `drag --duration` / `-d` controls the total drag time.

## Behavior

`silentmouse` posts mouse events to the process that owns the target window. It
does not move the global cursor, and it is designed to work when the target app
is not focused.

Coordinates are window-local. The target app may still observe normal mouse
event side effects, including modifier flags used for background delivery.

## Permissions

macOS requires Accessibility permission for the process that posts the event.
On first use, `silentmouse` requests the Accessibility prompt. Grant access to
the terminal, binary, or app wrapper that launched it, then retry the command.
For repeatable testing, run it from a stable path.

## Build

```sh
cargo build --release
```

The binary is written to: `target/release/silentmouse`

## Development

For code changes, run:

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

Use a release build when checking the final binary:

```sh
cargo build --release
```

Live smoke tests require a macOS GUI session, Accessibility permission, and a
known target window id. Use a target app that visibly reacts to mouse input:

```sh
cargo run -- click -w <window-id> -x 250 -y 250
cargo run -- drag -w <window-id> --from-x 250 --from-y 250 --to-x 320 --to-y 320 -d 500
```

Keep TCC-dependent smoke checks separate from unit tests because they depend on
the local machine's Accessibility state.
