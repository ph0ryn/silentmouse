# AGENTS.md

## API boundary

Treat `mouse` as a thin wrapper around the macOS mouse event API.
Treat every top-level command other than `mouse` as a convenience command built
from `mouse` events.

`silentmouse` does not keep state across invocations. Callers own event
sequencing, operation state, interpolation, retries, and target window
selection.

Coordinates are window-local, with `(0, 0)` at the top-left corner of the target
window. `mouse up` must also receive an explicit coordinate from the caller
instead of relying on hidden state.

## macOS API model

This repository is macOS-only. Do not add a cross-platform abstraction; keep the
implementation close to the macOS event delivery model.

Use CoreGraphics window list APIs to resolve target metadata from a window id.

- `CGWindowListCreateDescriptionFromArray`
- `CGWindowListCopyWindowInfo`
- `kCGWindowNumber`
- `kCGWindowOwnerPID`
- `kCGWindowBounds`
- `kCGWindowIsOnscreen`

The event generation path is `NSEvent.mouseEventWithType` to `CGEvent`, then
`CGEvent::post_to_pid` to the target pid. Treat this as target process delivery,
not as a normal global HID post.

Set both screen coordinates and window-local coordinates on each event.

- screen coordinate: equivalent to `CGEventSetLocation`
- window-local coordinate: private API `CGEventSetWindowLocation`

Because the public CLI uses window-local coordinates, derive the screen
coordinate from the target window bounds. `local(x, y)` maps to
`screen(bounds.minX + x, bounds.minY + y)`.

## Event fields

Set the known field map on mouse events. Do not remove these fields unless the
delivery model is intentionally changed.

- field `3`: mouse button number
- field `7`: subtype, value `3`
- field `91`: window under mouse pointer, target `CGWindowID`
- field `92`: window that can handle event, target `CGWindowID`

Button numbers follow AppKit / CoreGraphics mouse button numbering. Events that
need AppKit's `clickCount` should pass through the caller-provided count.

## Private CoreGraphics symbol

Treat `CGEventSetWindowLocation` as unavailable through `RTLD_DEFAULT`. Resolve
it by explicitly opening the CoreGraphics framework.

- framework path:
  `/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics`
- open: `dlopen(..., RTLD_NOW)`
- symbol: `dlsym(..., "CGEventSetWindowLocation")`

Do not treat failure to resolve this symbol as success. A screen-coordinate-only
fallback is not the validated window-local event delivery path.

Keep unsafe FFI and private API calls isolated in the macOS bridge layer.

## Background delivery

Set the Command flag when the target app is inactive. Without it, events may not
reach a background window.

The target app may interpret the event as Command-modified mouse input. If this
behavior changes, rerun live smoke testing against a non-frontmost target
window.

## Accessibility / TCC

The process that posts events needs Accessibility permission. Request the prompt
on first run with `AXIsProcessTrustedWithOptions`.

TCC can depend on the binary path, launcher, code signature, and app wrapper.
Do not treat permission state verified in one development setup as a guarantee
for another distribution shape.

Keep TCC-dependent live smoke checks separate from unit tests.
