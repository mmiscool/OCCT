const SGR_MOUSE_PREFIX = "\u001b[<";
const SGR_MOUSE_PATTERN = /^\u001b\[<(\d+);(\d+);(\d+)([Mm])/;

export class TerminalUI {
  constructor({ input = process.stdin, output = process.stdout } = {}) {
    this.input = input;
    this.output = output;
    this.lines = [];
    this.mouseCallbacks = new Set();
    this.inputBuffer = "";
    this.isClosed = false;
    this.isListening = false;
    this.rawModeEnabled = false;

    this.handleData = this.handleData.bind(this);
    this.handleResize = this.handleResize.bind(this);

    this.resizeLines();
    this.output.on("resize", this.handleResize);
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

  refresh(nextLines = this.lines) {
    if (this.isClosed) {
      return;
    }

    const visibleRows = this.resizeLines();

    if (nextLines !== this.lines) {
      for (let index = 0; index < visibleRows; index += 1) {
        this.lines[index] = String(nextLines[index] ?? "");
      }
    } else {
      for (let index = 0; index < visibleRows; index += 1) {
        this.lines[index] = String(this.lines[index] ?? "");
      }
    }

    const width = Math.max(this.output.columns ?? 80, 1);
    const renderedLines = this.lines.map((line) => this.fitLine(line, width));
    const footer = " ".repeat(width);
    const frame = [
      "\u001b[?25l",
      "\u001b[H",
      renderedLines.join("\n"),
      `\u001b[${visibleRows + 1};1H${footer}`,
      "\u001b[0m"
    ].join("");

    this.output.write(frame);
  }

  onMouse(callback) {
    if (typeof callback !== "function") {
      throw new TypeError("Mouse callback must be a function.");
    }

    this.ensureMouseTracking();
    this.mouseCallbacks.add(callback);

    return () => {
      this.mouseCallbacks.delete(callback);

      if (this.mouseCallbacks.size === 0) {
        this.stopMouseTracking();
      }
    };
  }

  close() {
    if (this.isClosed) {
      return;
    }

    this.stopMouseTracking();
    this.output.off("resize", this.handleResize);
    this.output.write("\u001b[?25h\u001b[0m");
    this.isClosed = true;
  }

  handleResize() {
    this.resizeLines();
    this.refresh();
  }

  ensureMouseTracking() {
    if (!this.input.isTTY || !this.output.isTTY) {
      throw new Error("Mouse tracking requires TTY input and output.");
    }

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
    this.output.write("\u001b[?1000h\u001b[?1006h");
    this.isListening = true;
  }

  stopMouseTracking() {
    if (!this.isListening) {
      return;
    }

    this.output.write("\u001b[?1000l\u001b[?1006l");
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
      const startIndex = this.inputBuffer.indexOf(SGR_MOUSE_PREFIX);

      if (startIndex === -1) {
        this.inputBuffer = "";
        return;
      }

      if (startIndex > 0) {
        this.inputBuffer = this.inputBuffer.slice(startIndex);
      }

      const match = this.inputBuffer.match(SGR_MOUSE_PATTERN);

      if (!match) {
        if (/^\u001b\[<[\d;]*$/.test(this.inputBuffer)) {
          return;
        }

        this.inputBuffer = this.inputBuffer.slice(1);
        continue;
      }

      this.inputBuffer = this.inputBuffer.slice(match[0].length);

      const mouseEvent = this.parseMouseEvent(match);

      if (mouseEvent) {
        for (const callback of this.mouseCallbacks) {
          callback(mouseEvent);
        }
      }
    }
  }

  parseMouseEvent(match) {
    const rawCode = Number(match[1]);
    const column = Number(match[2]);
    const row = Number(match[3]);
    const state = match[4];

    if (state !== "M") {
      return null;
    }

    if ((rawCode & 32) !== 0 || (rawCode & 64) !== 0) {
      return null;
    }

    const buttonCode = rawCode & 3;
    const buttonMap = {
      0: "left",
      1: "middle",
      2: "right"
    };

    return {
      type: "click",
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

  fitLine(line, width) {
    const text = Array.from(line).slice(0, width).join("");

    if (text.length >= width) {
      return text;
    }

    return text + " ".repeat(width - text.length);
  }
}
