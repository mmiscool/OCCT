import { TerminalUI } from "../src/TerminalUI.js";

const ui = new TerminalUI();
let clickCount = 0;
let lastClick = "No clicks yet.";

function render() {
  const lines = new Array(ui.lines.length).fill("");
  const boxStart = 7;

  lines[0] = "TerminalUI demo";
  lines[1] = "Click anywhere in the terminal window.";
  lines[2] = "Press q or Ctrl+C to quit.";
  lines[4] = `Terminal size: ${process.stdout.columns} columns x ${process.stdout.rows} rows`;
  lines[5] = `Visible buffer lines: ${ui.lines.length}`;
  lines[6] = `Clicks: ${clickCount} | Last click: ${lastClick}`;

  for (let row = boxStart; row < lines.length; row += 1) {
    const marker = row === boxStart ? "< click area >" : "";
    lines[row] = `${String(row).padStart(2, "0")} ${marker}`;
  }

  ui.refresh(lines);
}

function exit() {
  process.stdout.off("resize", render);
  ui.close();
  process.exit(0);
}

ui.onMouse((event) => {
  clickCount += 1;
  lastClick = `${event.button} button at x=${event.x}, y=${event.y}`;
  render();
});

process.stdout.on("resize", render);
process.stdin.on("data", (chunk) => {
  if (chunk === "q" || chunk === "Q" || chunk === "\u0003") {
    exit();
  }
});

process.on("SIGINT", exit);
process.on("exit", () => {
  ui.close();
});

render();
