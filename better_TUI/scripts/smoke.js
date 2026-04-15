import assert from "node:assert/strict";
import { EventEmitter } from "node:events";
import { TerminalUI } from "../src/TerminalUI.js";

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

const input = new FakeInput();
const output = new FakeOutput(6, 20);
const ui = new TerminalUI({ input, output });

assert.equal(ui.lines.length, 5);

ui.refresh(["hello", "world"]);
assert.equal(ui.lines[0], "hello");
assert.equal(ui.lines[1], "world");
assert.match(output.writes.at(-1), /\u001b\[H/);

let receivedEvent = null;
const detachMouse = ui.onMouse((event) => {
  receivedEvent = event;
});

assert.equal(input.rawMode, true);

input.emit("data", "\u001b[<0;4;2M");

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

input.emit("data", "\u001b[<32;6;3M");

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

output.rows = 8;
output.emit("resize");
assert.equal(ui.lines.length, 7);

detachMouse();
assert.equal(input.rawMode, false);

ui.close();

console.log("Smoke test passed.");
