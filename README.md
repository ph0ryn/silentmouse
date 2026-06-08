# silentmouse

`silentmouse` posts a macOS left click to a target `CGWindowID` without requiring
the target app to be focused. Coordinates are relative to the target window's
top-left corner.

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
event sequencing. `click` and `drag` are convenience commands built from raw
mouse events. `click --duration` / `-d` controls the milliseconds between mouse
down and mouse up, and defaults to `35`. `drag --duration` / `-d` controls the
total drag time.

## Permissions

macOS requires Accessibility permission for the process that posts the event.
On first use, `silentmouse` requests the Accessibility prompt. Grant access to
the terminal, binary, or app wrapper that launched it, then retry the command.
For repeatable testing, run it from a stable path.

## Smoke Test

The workspace includes `tools/smoke/MouseProbe.app`. Launch it, find its
`CGWindowID`, then click near the center of that window:

```sh
cargo run --manifest-path silentmouse/Cargo.toml -- click -w <window-id> -x 250 -y 250
```
