# better-tui

Minimal terminal UI helper with one class and no external dependencies.

## What it does

- Exposes a `lines` array where each item is one line on screen.
- Automatically resizes that array to `terminal height - 1`.
- Renders the buffer with `refresh()`.
- Lets you subscribe to mouse click events with `onMouse(callback)`.

## Usage

```js
import { TerminalUI } from "./src/TerminalUI.js";

const ui = new TerminalUI();

ui.lines[0] = "Hello";
ui.lines[1] = "Click in the terminal.";
ui.refresh();

const detachMouse = ui.onMouse((event) => {
  ui.lines[2] = `Clicked ${event.button} at ${event.x}, ${event.y}`;
  ui.refresh();
});

process.on("exit", () => {
  detachMouse();
  ui.close();
});
```

## Scripts

- `pnpm run demo` starts an interactive demo.
- `pnpm run smoke` runs a non-interactive smoke test.
