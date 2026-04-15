import { Button, Container, Label, TerminalUI, TextInput } from "../src/index.js";

const ui = new TerminalUI();

const app = new Container({
  x: "3%",
  y: "5%",
  width: "94%",
  height: "90%",
  border: true,
  title: "better-tui widget demo"
});

const header = new Label({
  x: "2",
  y: "1",
  width: "90%",
  height: "3",
  text: "Tab changes focus. Click fields and buttons.\nType into the input box. Press q or Ctrl+C to quit."
});

const form = new Container({
  x: "3%",
  y: "22%",
  width: "40%",
  height: "58%",
  border: true,
  title: "Form"
});

const previewPanel = new Container({
  x: "46%",
  y: "22%",
  width: "51%",
  height: "58%",
  border: true,
  title: "Preview"
});

const status = new Label({
  x: "2",
  y: "85%",
  width: "90%",
  height: "2",
  text: "Ready."
});

const previewText = new Label({
  x: "2",
  y: "1",
  width: "90%",
  height: "6",
  text: "Current value:\n<empty>"
});

const inputLabel = new Label({
  x: "10%",
  y: "1",
  width: "80%",
  height: "1",
  text: "Message"
});

const helpText = new Label({
  x: "10%",
  y: "10",
  width: "80%",
  height: "3",
  text: "Buttons react to mouse clicks and Enter.\nThe input field accepts typing, arrows,\nand backspace."
});

const messageInput = new TextInput({
  x: "10%",
  y: "2",
  width: "80%",
  height: "3",
  placeholder: "Type a message",
  onChange: ({ value }) => {
    previewText.text = `Current value:\n${value || "<empty>"}`;
    status.text = `Typing: ${value || "<empty>"}`;
  },
  onSubmit: ({ value }) => {
    status.text = `Submitted with Enter: ${value || "<empty>"}`;
  }
});

const applyButton = new Button({
  x: "10%",
  y: "6",
  width: "14",
  height: "3",
  label: "Apply",
  onPress: () => {
    previewText.text = `Applied value:\n${messageInput.value || "<empty>"}`;
    status.text = `Applied value: ${messageInput.value || "<empty>"}`;
  }
});

const clearButton = new Button({
  x: "52%",
  y: "6",
  width: "14",
  height: "3",
  label: "Clear",
  onPress: () => {
    messageInput.setValue("");
    messageInput.cursorIndex = 0;
    previewText.text = "Current value:\n<empty>";
    status.text = "Input cleared.";
  }
});

previewPanel.add(previewText);
form.add(inputLabel);
form.add(messageInput);
form.add(applyButton);
form.add(clearButton);
form.add(helpText);
app.add(header);
app.add(form);
app.add(previewPanel);
app.add(status);
ui.add(app);

function exit() {
  ui.close();
  process.exit(0);
}

ui.onMouse((event) => {
  if (event.type === "drag") {
    status.text = `Dragging with ${event.button} at x=${event.x}, y=${event.y}`;
  }
});

ui.onKey((event) => {
  if ((event.key === "q" && !event.ctrl) || (event.key === "c" && event.ctrl)) {
    exit();
    return false;
  }

  return true;
});

process.on("SIGINT", exit);
process.on("exit", () => {
  ui.close();
});

ui.refresh();
