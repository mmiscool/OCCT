# better-tui

Minimal terminal UI library with widget classes and no external dependencies.

## What it does

- Exposes a low-level `lines` array where each item is one rendered line.
- Automatically sizes the screen buffer to `terminal height - 1`.
- Supports nested widgets with exact values like `"12"` and relative values like `"50%"`.
- Includes `Container`, `Label`, `Button`, and `TextInput` classes.
- Routes mouse `click` and `drag` events plus keyboard input to focused widgets.

## Usage

```js
import { Button, Container, Label, TerminalUI, TextInput } from "better-tui";

const ui = new TerminalUI();
const app = new Container({
  x: "2",
  y: "1",
  width: "90%",
  height: "80%",
  border: true,
  title: "Example"
});

const status = new Label({
  x: "2",
  y: "8",
  width: "80%",
  height: "2",
  text: "Ready"
});

const input = new TextInput({
  x: "10%",
  y: "2",
  width: "80%",
  height: "3",
  placeholder: "Type here"
});

const button = new Button({
  x: "10%",
  y: "6",
  width: "12",
  height: "3",
  label: "Save",
  onPress: () => {
    status.text = `Saved: ${input.value}`;
  }
});

app.add(input);
app.add(button);
app.add(status);
ui.add(app);
ui.refresh();

ui.onKey((event) => {
  if ((event.key === "q" && !event.ctrl) || (event.key === "c" && event.ctrl)) {
    ui.close();
    process.exit(0);
  }
});
```

## Scripts

- `pnpm run demo` starts an interactive demo.
- `pnpm run smoke` runs a non-interactive smoke test.
