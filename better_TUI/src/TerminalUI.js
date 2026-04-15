import { Container } from "./widgets.js";

const SGR_MOUSE_PREFIX = "\u001b[<";
const SGR_MOUSE_PATTERN = /^\u001b\[<(\d+);(\d+);(\d+)([Mm])/;
const KEY_SEQUENCES = [
  { sequence: "\u001b[A", key: "up" },
  { sequence: "\u001b[B", key: "down" },
  { sequence: "\u001b[C", key: "right" },
  { sequence: "\u001b[D", key: "left" },
  { sequence: "\u001b[H", key: "home" },
  { sequence: "\u001b[F", key: "end" },
  { sequence: "\u001b[3~", key: "delete" },
  { sequence: "\u001b[Z", key: "tab", shift: true }
];

class Canvas {
  constructor(width, height) {
    this.width = width;
    this.height = height;
    this.rows = Array.from({ length: height }, () => Array(width).fill(" "));
  }

  setCell(x, y, value) {
    if (x < 0 || y < 0 || x >= this.width || y >= this.height) {
      return;
    }

    this.rows[y][x] = value;
  }

  writeText(x, y, text, maxWidth = this.width - x) {
    if (y < 0 || y >= this.height || maxWidth <= 0) {
      return;
    }

    const chars = Array.from(String(text)).slice(0, maxWidth);

    for (let index = 0; index < chars.length; index += 1) {
      this.setCell(x + index, y, chars[index]);
    }
  }

  writeCenteredText(x, y, width, text) {
    const chars = Array.from(String(text)).slice(0, width);
    const start = x + Math.max(Math.floor((width - chars.length) / 2), 0);
    this.writeText(start, y, chars.join(""), width);
  }

  fillRect(bounds, fillChar = " ") {
    for (let row = bounds.y; row < bounds.y + bounds.height; row += 1) {
      for (let column = bounds.x; column < bounds.x + bounds.width; column += 1) {
        this.setCell(column, row, fillChar);
      }
    }
  }

  drawBorder(bounds, { title = "", focused = false } = {}) {
    if (bounds.width <= 0 || bounds.height <= 0) {
      return;
    }

    if (bounds.width === 1 || bounds.height === 1) {
      this.fillRect(bounds, focused ? "#" : "+");
      return;
    }

    const horizontal = focused ? "=" : "-";
    const vertical = focused ? "#" : "|";

    this.setCell(bounds.x, bounds.y, "+");
    this.setCell(bounds.x + bounds.width - 1, bounds.y, "+");
    this.setCell(bounds.x, bounds.y + bounds.height - 1, "+");
    this.setCell(bounds.x + bounds.width - 1, bounds.y + bounds.height - 1, "+");

    for (let column = bounds.x + 1; column < bounds.x + bounds.width - 1; column += 1) {
      this.setCell(column, bounds.y, horizontal);
      this.setCell(column, bounds.y + bounds.height - 1, horizontal);
    }

    for (let row = bounds.y + 1; row < bounds.y + bounds.height - 1; row += 1) {
      this.setCell(bounds.x, row, vertical);
      this.setCell(bounds.x + bounds.width - 1, row, vertical);
    }

    if (title && bounds.width > 4) {
      this.writeText(bounds.x + 2, bounds.y, ` ${title} `, bounds.width - 4);
    }
  }

  toLines() {
    return this.rows.map((row) => row.join(""));
  }
}

export class TerminalUI {
  constructor({ input = process.stdin, output = process.stdout } = {}) {
    this.input = input;
    this.output = output;
    this.lines = [];
    this.mouseCallbacks = new Set();
    this.keyCallbacks = new Set();
    this.inputBuffer = "";
    this.isClosed = false;
    this.isListening = false;
    this.rawModeEnabled = false;
    this.mouseTrackingEnabled = false;
    this.focusedWidget = null;
    this.root = new Container({
      x: "0",
      y: "0",
      width: "100%",
      height: "100%"
    });
    this.root.attachUI(this);

    this.handleData = this.handleData.bind(this);
    this.handleResize = this.handleResize.bind(this);

    this.resizeLines();
    this.output.on("resize", this.handleResize);
  }

  add(widget) {
    const child = this.root.add(widget);
    this.syncInputState();
    return child;
  }

  remove(widget) {
    const child = this.root.remove(widget);

    if (child && this.focusedWidget === child) {
      this.setFocus(null);
    }

    this.syncInputState();
    return child;
  }

  resizeLines() {
    const visibleRows = Math.max((this.output.rows ?? 1) - 1, 1);

    while (this.lines.length < visibleRows) {
      this.lines.push("");
    }

    if (this.lines.length > visibleRows) {
      this.lines.length = visibleRows;
    }

    return visibleRows;
  }

  refresh(nextLines) {
    if (this.isClosed) {
      return;
    }

    const visibleRows = this.resizeLines();
    const width = Math.max(this.output.columns ?? 80, 1);
    let renderedLines = [];

    if (arguments.length === 0 && this.hasWidgets()) {
      renderedLines = this.renderWidgetTree(width, visibleRows);
    } else {
      const sourceLines = arguments.length === 0 ? this.lines : nextLines;

      for (let index = 0; index < visibleRows; index += 1) {
        this.lines[index] = String(sourceLines[index] ?? "");
      }

      renderedLines = this.lines.map((line) => this.fitLine(line, width));
    }

    for (let index = 0; index < visibleRows; index += 1) {
      this.lines[index] = renderedLines[index] ?? "";
    }

    this.writeFrame(renderedLines, visibleRows, width);
  }

  onMouse(callback) {
    if (typeof callback !== "function") {
      throw new TypeError("Mouse callback must be a function.");
    }

    this.mouseCallbacks.add(callback);
    this.syncInputState();

    return () => {
      this.mouseCallbacks.delete(callback);
      this.syncInputState();
    };
  }

  onKey(callback) {
    if (typeof callback !== "function") {
      throw new TypeError("Key callback must be a function.");
    }

    this.keyCallbacks.add(callback);
    this.syncInputState();

    return () => {
      this.keyCallbacks.delete(callback);
      this.syncInputState();
    };
  }

  close() {
    if (this.isClosed) {
      return;
    }

    this.stopInputTracking();
    this.output.off("resize", this.handleResize);
    this.output.write("\u001b[?25h\u001b[0m");
    this.isClosed = true;
  }

  hasWidgets() {
    return this.root.children.length > 0;
  }

  handleResize() {
    this.resizeLines();
    this.refresh();
  }

  setFocus(widget) {
    if (this.focusedWidget === widget) {
      return;
    }

    if (this.focusedWidget) {
      this.focusedWidget.blur();
    }

    this.focusedWidget = widget && widget.isFocusable() ? widget : null;

    if (this.focusedWidget) {
      this.focusedWidget.focus();
    }
  }

  focusRelative(offset) {
    const focusable = this.root.collectFocusable([]);

    if (focusable.length === 0) {
      this.setFocus(null);
      return;
    }

    if (!this.focusedWidget) {
      this.setFocus(offset < 0 ? focusable.at(-1) : focusable[0]);
      return;
    }

    const currentIndex = focusable.indexOf(this.focusedWidget);

    if (currentIndex === -1) {
      this.setFocus(focusable[0]);
      return;
    }

    const nextIndex = (currentIndex + offset + focusable.length) % focusable.length;
    this.setFocus(focusable[nextIndex]);
  }

  syncInputState() {
    if (!this.input.isTTY || !this.output.isTTY) {
      return;
    }

    const needsInput = this.keyCallbacks.size > 0 || this.mouseCallbacks.size > 0 || this.hasWidgets();

    if (!needsInput) {
      this.stopInputTracking();
      return;
    }

    this.ensureInputTracking();

    if (this.mouseCallbacks.size > 0 || this.hasWidgets()) {
      this.enableMouseTracking();
    } else {
      this.disableMouseTracking();
    }
  }

  ensureInputTracking() {
    if (this.isListening) {
      return;
    }

    this.input.setEncoding("utf8");
    this.input.resume();

    if (typeof this.input.setRawMode === "function") {
      this.input.setRawMode(true);
      this.rawModeEnabled = true;
    }

    this.input.on("data", this.handleData);
    this.isListening = true;
  }

  enableMouseTracking() {
    if (this.mouseTrackingEnabled) {
      return;
    }

    this.output.write("\u001b[?1000h\u001b[?1002h\u001b[?1006h");
    this.mouseTrackingEnabled = true;
  }

  disableMouseTracking() {
    if (!this.mouseTrackingEnabled) {
      return;
    }

    this.output.write("\u001b[?1000l\u001b[?1002l\u001b[?1006l");
    this.mouseTrackingEnabled = false;
  }

  stopInputTracking() {
    this.disableMouseTracking();

    if (!this.isListening) {
      return;
    }

    this.input.off("data", this.handleData);

    if (this.rawModeEnabled && typeof this.input.setRawMode === "function") {
      this.input.setRawMode(false);
    }

    this.rawModeEnabled = false;
    this.isListening = false;
    this.inputBuffer = "";
  }

  handleData(chunk) {
    this.inputBuffer += chunk;

    while (this.inputBuffer.length > 0) {
      const result = this.parseNextInput(this.inputBuffer);

      if (!result) {
        return;
      }

      this.inputBuffer = this.inputBuffer.slice(result.length);

      if (!result.event) {
        continue;
      }

      if (result.event.type === "mouse") {
        this.dispatchMouseEvent(result.event);
        continue;
      }

      if (result.event.type === "key") {
        this.dispatchKeyEvent(result.event);
      }
    }
  }

  parseNextInput(buffer) {
    if (buffer.startsWith(SGR_MOUSE_PREFIX)) {
      const match = buffer.match(SGR_MOUSE_PATTERN);

      if (!match) {
        if (/^\u001b\[<[\d;]*$/.test(buffer)) {
          return null;
        }

        return { length: 1, event: null };
      }

      return {
        length: match[0].length,
        event: this.parseMouseEvent(match)
      };
    }

    if (buffer[0] === "\u001b") {
      for (const candidate of KEY_SEQUENCES) {
        if (buffer.startsWith(candidate.sequence)) {
          return {
            length: candidate.sequence.length,
            event: {
              type: "key",
              key: candidate.key,
              text: null,
              ctrl: false,
              alt: false,
              shift: Boolean(candidate.shift),
              raw: candidate.sequence
            }
          };
        }

        if (candidate.sequence.startsWith(buffer)) {
          return null;
        }
      }

      if (buffer.length === 1) {
        return null;
      }

      return {
        length: 1,
        event: {
          type: "key",
          key: "escape",
          text: null,
          ctrl: false,
          alt: false,
          shift: false,
          raw: "\u001b"
        }
      };
    }

    return {
      length: 1,
      event: this.parseCharacterKey(buffer[0])
    };
  }

  parseMouseEvent(match) {
    const rawCode = Number(match[1]);
    const column = Number(match[2]);
    const row = Number(match[3]);
    const state = match[4];

    if (state !== "M") {
      return null;
    }

    const isDrag = (rawCode & 32) !== 0;

    if ((rawCode & 64) !== 0) {
      return null;
    }

    const buttonCode = rawCode & 3;
    const buttonMap = {
      0: "left",
      1: "middle",
      2: "right"
    };

    return {
      type: "mouse",
      action: isDrag ? "drag" : "click",
      button: buttonMap[buttonCode] ?? "unknown",
      x: column - 1,
      y: row - 1,
      column,
      row,
      shift: (rawCode & 4) !== 0,
      alt: (rawCode & 8) !== 0,
      ctrl: (rawCode & 16) !== 0,
      rawCode
    };
  }

  parseCharacterKey(character) {
    const code = character.charCodeAt(0);

    if (character === "\r" || character === "\n") {
      return {
        type: "key",
        key: "enter",
        text: null,
        ctrl: false,
        alt: false,
        shift: false,
        raw: character
      };
    }

    if (character === "\t") {
      return {
        type: "key",
        key: "tab",
        text: null,
        ctrl: false,
        alt: false,
        shift: false,
        raw: character
      };
    }

    if (character === "\u007f" || character === "\b") {
      return {
        type: "key",
        key: "backspace",
        text: null,
        ctrl: false,
        alt: false,
        shift: false,
        raw: character
      };
    }

    if (code >= 1 && code <= 26) {
      return {
        type: "key",
        key: String.fromCharCode(code + 96),
        text: null,
        ctrl: true,
        alt: false,
        shift: false,
        raw: character
      };
    }

    if (character === " ") {
      return {
        type: "key",
        key: "space",
        text: " ",
        ctrl: false,
        alt: false,
        shift: false,
        raw: character
      };
    }

    return {
      type: "key",
      key: character,
      text: character,
      ctrl: false,
      alt: false,
      shift: false,
      raw: character
    };
  }

  dispatchMouseEvent(event) {
    if (!event) {
      return;
    }

    const publicEvent = {
      type: event.action,
      button: event.button,
      x: event.x,
      y: event.y,
      column: event.column,
      row: event.row,
      shift: event.shift,
      alt: event.alt,
      ctrl: event.ctrl,
      rawCode: event.rawCode
    };

    let shouldRouteToWidgets = true;

    for (const callback of this.mouseCallbacks) {
      if (callback(publicEvent) === false) {
        shouldRouteToWidgets = false;
      }
    }

    if (!shouldRouteToWidgets || !this.hasWidgets()) {
      return;
    }

    const target = this.root.findWidgetAt(event.x, event.y);

    if (event.action === "click") {
      this.setFocus(target && target.isFocusable() ? target : null);
    }

    if (target) {
      target.handleMouse({
        type: event.action,
        button: event.button,
        x: event.x,
        y: event.y,
        localX: event.x - target.bounds.x,
        localY: event.y - target.bounds.y,
        shift: event.shift,
        alt: event.alt,
        ctrl: event.ctrl
      });
    }

    this.refresh();
  }

  dispatchKeyEvent(event) {
    if (!event) {
      return;
    }

    let shouldRouteToWidgets = true;

    for (const callback of this.keyCallbacks) {
      if (callback(event) === false) {
        shouldRouteToWidgets = false;
      }
    }

    if (!shouldRouteToWidgets || !this.hasWidgets()) {
      return;
    }

    if (event.key === "tab") {
      this.focusRelative(event.shift ? -1 : 1);
      this.refresh();
      return;
    }

    if (this.focusedWidget) {
      this.focusedWidget.handleKey(event);
      this.refresh();
    }
  }

  renderWidgetTree(width, height) {
    const canvas = new Canvas(width, height);
    this.root.layout({ x: 0, y: 0, width, height });
    this.root.render(canvas);
    return canvas.toLines();
  }

  writeFrame(lines, visibleRows, width) {
    const footer = " ".repeat(width);
    const frame = [
      "\u001b[?25l",
      "\u001b[H",
      lines.join("\n"),
      `\u001b[${visibleRows + 1};1H${footer}`,
      "\u001b[0m"
    ].join("");

    this.output.write(frame);
  }

  fitLine(line, width) {
    const chars = Array.from(String(line)).slice(0, width);

    if (chars.length >= width) {
      return chars.join("");
    }

    return chars.join("") + " ".repeat(width - chars.length);
  }
}
