function clamp(value, min, max) {
  return Math.max(min, Math.min(value, max));
}

function normalizeMetric(value, fallback) {
  return String(value ?? fallback).trim();
}

export function resolveMetric(metric, parentSize, fallback = 0) {
  const raw = normalizeMetric(metric, fallback);

  if (raw.endsWith("%")) {
    const percent = Number.parseFloat(raw.slice(0, -1));
    return Number.isFinite(percent) ? Math.floor((parentSize * percent) / 100) : fallback;
  }

  const exact = Number.parseInt(raw, 10);
  return Number.isFinite(exact) ? exact : fallback;
}

export class Widget {
  constructor({
    x = "0",
    y = "0",
    width = "100%",
    height = "100%",
    visible = true,
    disabled = false,
    name = ""
  } = {}) {
    this.x = normalizeMetric(x, "0");
    this.y = normalizeMetric(y, "0");
    this.width = normalizeMetric(width, "100%");
    this.height = normalizeMetric(height, "100%");
    this.visible = visible;
    this.disabled = disabled;
    this.name = name;
    this.parent = null;
    this.ui = null;
    this.focusable = false;
    this.focused = false;
    this.bounds = { x: 0, y: 0, width: 0, height: 0 };
  }

  setLayout({ x, y, width, height } = {}) {
    if (x !== undefined) {
      this.x = normalizeMetric(x, this.x);
    }

    if (y !== undefined) {
      this.y = normalizeMetric(y, this.y);
    }

    if (width !== undefined) {
      this.width = normalizeMetric(width, this.width);
    }

    if (height !== undefined) {
      this.height = normalizeMetric(height, this.height);
    }

    return this;
  }

  attachUI(ui) {
    this.ui = ui;
    return this;
  }

  detachUI() {
    this.ui = null;
    return this;
  }

  layout(parentBounds) {
    this.bounds = this.resolveBounds(parentBounds);
    return this.bounds;
  }

  resolveBounds(parentBounds) {
    if (!this.visible || parentBounds.width <= 0 || parentBounds.height <= 0) {
      return {
        x: parentBounds.x,
        y: parentBounds.y,
        width: 0,
        height: 0
      };
    }

    const offsetX = Math.max(resolveMetric(this.x, parentBounds.width, 0), 0);
    const offsetY = Math.max(resolveMetric(this.y, parentBounds.height, 0), 0);
    const x = parentBounds.x + Math.min(offsetX, Math.max(parentBounds.width - 1, 0));
    const y = parentBounds.y + Math.min(offsetY, Math.max(parentBounds.height - 1, 0));
    const remainingWidth = Math.max(parentBounds.x + parentBounds.width - x, 0);
    const remainingHeight = Math.max(parentBounds.y + parentBounds.height - y, 0);
    const width = clamp(resolveMetric(this.width, parentBounds.width, remainingWidth), 0, remainingWidth);
    const height = clamp(resolveMetric(this.height, parentBounds.height, remainingHeight), 0, remainingHeight);

    return { x, y, width, height };
  }

  containsPoint(x, y) {
    return (
      this.visible &&
      x >= this.bounds.x &&
      y >= this.bounds.y &&
      x < this.bounds.x + this.bounds.width &&
      y < this.bounds.y + this.bounds.height
    );
  }

  isFocusable() {
    return !this.disabled && this.focusable && this.visible && this.bounds.width > 0 && this.bounds.height > 0;
  }

  focus() {
    this.focused = true;
  }

  blur() {
    this.focused = false;
  }

  findWidgetAt(x, y) {
    return this.containsPoint(x, y) ? this : null;
  }

  collectFocusable(list) {
    if (this.isFocusable()) {
      list.push(this);
    }

    return list;
  }

  render(_canvas) {}

  handleMouse(_event) {
    return false;
  }

  handleKey(_event) {
    return false;
  }
}

export class Container extends Widget {
  constructor({
    children = [],
    border = false,
    title = "",
    fillChar = " ",
    ...layout
  } = {}) {
    super(layout);
    this.children = [];
    this.border = border;
    this.title = title;
    this.fillChar = fillChar;

    for (const child of children) {
      this.add(child);
    }
  }

  attachUI(ui) {
    super.attachUI(ui);

    for (const child of this.children) {
      child.attachUI(ui);
    }

    return this;
  }

  detachUI() {
    for (const child of this.children) {
      child.detachUI();
    }

    return super.detachUI();
  }

  add(child) {
    if (!(child instanceof Widget)) {
      throw new TypeError("Containers can only add widget instances.");
    }

    if (child.parent) {
      child.parent.remove(child);
    }

    child.parent = this;
    this.children.push(child);

    if (this.ui) {
      child.attachUI(this.ui);
    }

    return child;
  }

  remove(child) {
    const index = this.children.indexOf(child);

    if (index === -1) {
      return null;
    }

    this.children.splice(index, 1);
    child.parent = null;
    child.detachUI();

    return child;
  }

  getContentBounds() {
    if (!this.border) {
      return { ...this.bounds };
    }

    if (this.bounds.width < 3 || this.bounds.height < 3) {
      return {
        x: this.bounds.x,
        y: this.bounds.y,
        width: 0,
        height: 0
      };
    }

    return {
      x: this.bounds.x + 1,
      y: this.bounds.y + 1,
      width: this.bounds.width - 2,
      height: this.bounds.height - 2
    };
  }

  layout(parentBounds) {
    super.layout(parentBounds);
    const contentBounds = this.getContentBounds();

    for (const child of this.children) {
      child.layout(contentBounds);
    }

    return this.bounds;
  }

  render(canvas) {
    if (!this.visible || this.bounds.width === 0 || this.bounds.height === 0) {
      return;
    }

    if (this.fillChar !== " ") {
      canvas.fillRect(this.bounds, this.fillChar);
    }

    if (this.border) {
      canvas.drawBorder(this.bounds, { title: this.title, focused: this.focused });
    }

    for (const child of this.children) {
      child.render(canvas);
    }
  }

  findWidgetAt(x, y) {
    if (!this.containsPoint(x, y)) {
      return null;
    }

    for (let index = this.children.length - 1; index >= 0; index -= 1) {
      const child = this.children[index];
      const hit = child.findWidgetAt(x, y);

      if (hit) {
        return hit;
      }
    }

    return this;
  }

  collectFocusable(list) {
    super.collectFocusable(list);

    for (const child of this.children) {
      child.collectFocusable(list);
    }

    return list;
  }
}

export class Label extends Widget {
  constructor({ text = "", ...layout } = {}) {
    super(layout);
    this.text = text;
  }

  render(canvas) {
    if (!this.visible || this.bounds.width === 0 || this.bounds.height === 0) {
      return;
    }

    const lines = String(this.text).split("\n");

    for (let index = 0; index < Math.min(lines.length, this.bounds.height); index += 1) {
      canvas.writeText(this.bounds.x, this.bounds.y + index, lines[index], this.bounds.width);
    }
  }
}

export class Button extends Widget {
  constructor({ label = "Button", onPress = null, ...layout } = {}) {
    super({
      width: "12",
      height: "3",
      ...layout
    });
    this.label = label;
    this.onPress = onPress;
    this.focusable = true;
  }

  render(canvas) {
    if (!this.visible || this.bounds.width === 0 || this.bounds.height === 0) {
      return;
    }

    if (this.bounds.width >= 4 && this.bounds.height >= 3) {
      canvas.drawBorder(this.bounds, { focused: this.focused });

      const text = this.focused ? `> ${this.label} <` : `[ ${this.label} ]`;
      const row = this.bounds.y + Math.floor(this.bounds.height / 2);
      canvas.writeCenteredText(this.bounds.x + 1, row, this.bounds.width - 2, text);
      return;
    }

    canvas.writeCenteredText(this.bounds.x, this.bounds.y, this.bounds.width, this.label);
  }

  press(event) {
    if (typeof this.onPress === "function") {
      this.onPress({
        widget: this,
        ui: this.ui,
        event
      });
    }

    return true;
  }

  handleMouse(event) {
    if (event.type === "click" && event.button === "left") {
      return this.press(event);
    }

    return false;
  }

  handleKey(event) {
    if (event.key === "enter" || event.key === "space") {
      return this.press(event);
    }

    return false;
  }
}

export class TextInput extends Widget {
  constructor({
    value = "",
    placeholder = "",
    maxLength = Infinity,
    onChange = null,
    onSubmit = null,
    ...layout
  } = {}) {
    super({
      width: "20",
      height: "3",
      ...layout
    });
    this.value = String(value);
    this.placeholder = placeholder;
    this.maxLength = maxLength;
    this.onChange = onChange;
    this.onSubmit = onSubmit;
    this.cursorIndex = this.value.length;
    this.scrollOffset = 0;
    this.focusable = true;
  }

  getInnerBounds() {
    if (this.bounds.width < 3 || this.bounds.height < 3) {
      return { x: this.bounds.x, y: this.bounds.y, width: this.bounds.width, height: this.bounds.height };
    }

    return {
      x: this.bounds.x + 1,
      y: this.bounds.y + 1,
      width: this.bounds.width - 2,
      height: this.bounds.height - 2
    };
  }

  notifyChange() {
    if (typeof this.onChange === "function") {
      this.onChange({
        widget: this,
        ui: this.ui,
        value: this.value
      });
    }
  }

  setValue(value) {
    this.value = String(value).slice(0, this.maxLength);
    this.cursorIndex = clamp(this.cursorIndex, 0, this.value.length);
    this.notifyChange();
    return this;
  }

  ensureCursorVisible(width) {
    if (width <= 0) {
      this.scrollOffset = 0;
      return;
    }

    if (this.cursorIndex < this.scrollOffset) {
      this.scrollOffset = this.cursorIndex;
    }

    if (this.cursorIndex > this.scrollOffset + width - 1) {
      this.scrollOffset = this.cursorIndex - width + 1;
    }

    this.scrollOffset = clamp(this.scrollOffset, 0, Math.max(this.value.length - width + 1, 0));
  }

  focus() {
    super.focus();
    this.cursorIndex = clamp(this.cursorIndex, 0, this.value.length);
  }

  render(canvas) {
    if (!this.visible || this.bounds.width === 0 || this.bounds.height === 0) {
      return;
    }

    canvas.drawBorder(this.bounds, { focused: this.focused });

    const inner = this.getInnerBounds();

    if (inner.width <= 0 || inner.height <= 0) {
      return;
    }

    const row = inner.y + Math.floor(inner.height / 2);

    if (this.value.length === 0) {
      if (!this.focused && this.placeholder) {
        canvas.writeText(inner.x, row, this.placeholder, inner.width);
      }

      if (this.focused) {
        canvas.setCell(inner.x, row, "|");
      }

      return;
    }

    this.ensureCursorVisible(inner.width);

    const visibleValue = this.value.slice(this.scrollOffset, this.scrollOffset + inner.width);
    canvas.writeText(inner.x, row, visibleValue, inner.width);

    if (!this.focused) {
      return;
    }

    const cursorColumn = clamp(this.cursorIndex - this.scrollOffset, 0, inner.width - 1);
    canvas.setCell(inner.x + cursorColumn, row, "|");
  }

  insertText(text) {
    if (!text || this.value.length >= this.maxLength) {
      return false;
    }

    const room = this.maxLength - this.value.length;
    const nextText = text.slice(0, room);
    this.value =
      this.value.slice(0, this.cursorIndex) +
      nextText +
      this.value.slice(this.cursorIndex);
    this.cursorIndex += nextText.length;
    this.notifyChange();

    return true;
  }

  handleMouse(event) {
    if (event.type !== "click" || event.button !== "left") {
      return false;
    }

    const inner = this.getInnerBounds();

    if (inner.width <= 0) {
      return false;
    }

    if (this.value.length === 0) {
      this.cursorIndex = 0;
      this.scrollOffset = 0;
      return true;
    }

    const relativeX = clamp(event.x - inner.x, 0, inner.width - 1);
    this.ensureCursorVisible(inner.width);
    this.cursorIndex = clamp(this.scrollOffset + relativeX, 0, this.value.length);
    this.ensureCursorVisible(inner.width);

    return true;
  }

  handleKey(event) {
    switch (event.key) {
      case "left":
        this.cursorIndex = clamp(this.cursorIndex - 1, 0, this.value.length);
        return true;
      case "right":
        this.cursorIndex = clamp(this.cursorIndex + 1, 0, this.value.length);
        return true;
      case "home":
        this.cursorIndex = 0;
        return true;
      case "end":
        this.cursorIndex = this.value.length;
        return true;
      case "backspace":
        if (this.cursorIndex === 0) {
          return false;
        }

        this.value =
          this.value.slice(0, this.cursorIndex - 1) +
          this.value.slice(this.cursorIndex);
        this.cursorIndex -= 1;
        this.notifyChange();
        return true;
      case "delete":
        if (this.cursorIndex >= this.value.length) {
          return false;
        }

        this.value =
          this.value.slice(0, this.cursorIndex) +
          this.value.slice(this.cursorIndex + 1);
        this.notifyChange();
        return true;
      case "enter":
        if (typeof this.onSubmit === "function") {
          this.onSubmit({
            widget: this,
            ui: this.ui,
            value: this.value
          });
        }

        return true;
      default:
        if (event.text && !event.ctrl && !event.alt) {
          return this.insertText(event.text);
        }

        return false;
    }
  }
}
