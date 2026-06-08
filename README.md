# silentmouse

`silentmouse` posts a macOS left click to a target `CGWindowID` without requiring
the target app to be focused.

```sh
silentmouse click --window-id 1 -x 1920 -y 1080
silentmouse click -w 1 -x 1920 -y 1080
```

The v1 CLI is intentionally narrow: one left click, one target window, one
screen-space coordinate.

## Permissions

macOS requires Accessibility permission for the process that posts the event.
On first use, `silentmouse` requests the Accessibility prompt. Grant access to
the terminal, binary, or app wrapper that launched it, then retry the command.
For repeatable testing, run it from a stable path.

## Smoke Test

The workspace includes `tools/smoke/MouseProbe.app`. Launch it, find its
`CGWindowID`, then click near the center of that window:

```sh
cargo run --manifest-path silentmouse/Cargo.toml -- click -w <window-id> -x <x> -y <y>
```
