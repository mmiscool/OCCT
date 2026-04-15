import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { Button, Container, Label, TerminalUI, TextInput } from "../src/index.js";

class FakeInput extends EventEmitter {
  constructor() {
    super();
    this.isTTY = true;
    this.encoding = "utf8";
    this.rawMode = false;
    this.resumed = false;
  }

  setEncoding(value) {
    this.encoding = value;
  }

  setRawMode(value) {
    this.rawMode = value;
  }

  resume() {
    this.resumed = true;
  }
}

class FakeOutput extends EventEmitter {
  constructor(rows, columns) {
    super();
    this.isTTY = true;
    this.rows = rows;
    this.columns = columns;
    this.writes = [];
  }

  write(chunk) {
    this.writes.push(chunk);
    return true;
  }
}

const rawInput = new FakeInput();
const rawOutput = new FakeOutput(6, 20);
const rawUi = new TerminalUI({ input: rawInput, output: rawOutput });

assert.equal(rawUi.lines.length, 5);

rawUi.refresh(["hello", "world"]);
assert.equal(rawUi.lines[0], "hello               ");
assert.equal(rawUi.lines[1], "world               ");
assert.match(rawOutput.writes.at(-1), /\u001b\[H/);

let receivedEvent = null;
const detachMouse = rawUi.onMouse((event) => {
  receivedEvent = event;
});

assert.equal(rawInput.rawMode, true);

rawInput.emit("data", "\u001b[<0;4;2M");

assert.deepEqual(receivedEvent, {
  type: "click",
  button: "left",
  x: 3,
  y: 1,
  column: 4,
  row: 2,
  shift: false,
  alt: false,
  ctrl: false,
  rawCode: 0
});

rawInput.emit("data", "\u001b[<32;6;3M");

assert.deepEqual(receivedEvent, {
  type: "drag",
  button: "left",
  x: 5,
  y: 2,
  column: 6,
  row: 3,
  shift: false,
  alt: false,
  ctrl: false,
  rawCode: 32
});

rawOutput.rows = 8;
rawOutput.emit("resize");
assert.equal(rawUi.lines.length, 7);

detachMouse();
assert.equal(rawInput.rawMode, false);

rawUi.close();

const input = new FakeInput();
const output = new FakeOutput(18, 50);
const ui = new TerminalUI({ input, output });
let presses = 0;

const root = new Container({
  x: "2",
  y: "1",
  width: "46",
  height: "15",
  border: true,
  title: "Root"
});

const status = new Label({
  x: "2",
  y: "11",
  width: "40",
  height: "1",
  text: "Idle"
});

const panel = new Container({
  x: "10%",
  y: "2",
  width: "70%",
  height: "8",
  border: true,
  title: "Form"
});

const field = new TextInput({
  x: "10%",
  y: "0",
  width: "80%",
  height: "3",
  placeholder: "Name",
  onChange: ({ value }) => {
    status.text = `Typing: ${value}`;
  }
});

const button = new Button({
  x: "10%",
  y: "3",
  width: "12",
  height: "3",
  label: "Save",
  onPress: () => {
    presses += 1;
    status.text = `Saved: ${field.value}`;
  }
});

panel.add(field);
panel.add(button);
root.add(status);
root.add(panel);
ui.add(root);
ui.refresh();

assert.deepEqual(root.bounds, {
  x: 2,
  y: 1,
  width: 46,
  height: 15
});

assert.deepEqual(panel.bounds, {
  x: 7,
  y: 4,
  width: 30,
  height: 8
});

assert.deepEqual(field.bounds, {
  x: 10,
  y: 5,
  width: 22,
  height: 3
});

input.emit("data", `\u001b[<0;${field.bounds.x + 2};${field.bounds.y + 2}M`);
assert.equal(ui.focusedWidget, field);

input.emit("data", "abc");
assert.equal(field.value, "abc");
assert.equal(status.text, "Typing: abc");

input.emit("data", "\u007f");
assert.equal(field.value, "ab");
assert.equal(status.text, "Typing: ab");

input.emit("data", "\t");
assert.equal(ui.focusedWidget, button);

input.emit("data", "\r");
assert.equal(presses, 1);
assert.equal(status.text, "Saved: ab");

assert.match(output.writes.at(-1), /Save/);

ui.close();

console.log("Smoke test passed.");
