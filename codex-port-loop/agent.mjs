import { spawn } from "node:child_process";
import { promises as fs } from "node:fs";
import path from "node:path";
import process from "node:process";
import readline from "node:readline";
import { fileURLToPath } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const agentDir = __dirname;
const packageRoot = path.resolve(agentDir, "..");
const configFile = path.join(agentDir, "config.json");
const runtimeDir = path.join(agentDir, ".runtime");
const stateFile = path.join(runtimeDir, "state.json");
const lastMessageFile = path.join(runtimeDir, "last-message.txt");
const codexBin = process.platform === "win32"
  ? path.join(packageRoot, "node_modules", ".bin", "codex.cmd")
  : path.join(packageRoot, "node_modules", ".bin", "codex");

const retryDelayMs = 10000;
const autoCompactTokenLimit = 120000;
const startupDelayMs = 20000;
const validReasoningLevels = new Set(["none", "minimal", "low", "medium", "high", "xhigh"]);
const eventDivider = "-".repeat(100);
const defaultConfig = {
  projectPath: packageRoot,
  model: "gpt-5.4",
  reasoningLevel: "xhigh",
  loopPrompt: "Keep going porting from C++ to rust.",
  delayBetweenLoopsMs: 1000,
};

let stopRequested = false;

process.on("SIGINT", () => {
  stopRequested = true;
  process.stderr.write("\nStop requested. Finishing the current turn before exit.\n");
});

process.on("SIGTERM", () => {
  stopRequested = true;
  process.stderr.write("\nTermination requested. Finishing the current turn before exit.\n");
});

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});

async function main() {
  await ensureRuntime();

  await ensureConfigFileExists();
  const dangerousApproved = await askDangerousApproval();
  if (!dangerousApproved) {
    console.log("Dangerous mode was not approved. Exiting.");
    return;
  }

  await editConfigInNano();
  const config = await loadConfig();
  let state = await loadState();

  if (state.projectPath && state.projectPath !== config.projectPath) {
    console.log(`Project path changed from ${state.projectPath} to ${config.projectPath}. Starting a new Codex session.`);
    state = {};
  }

  console.log(`Config file: ${configFile}`);
  console.log(`Project path: ${config.projectPath}`);
  console.log(`Prompt: ${config.loopPrompt}`);
  console.log(`Model: ${config.model}`);
  console.log(`Reasoning: ${config.reasoningLevel}`);
  console.log(`Delay between loops: ${config.delayBetweenLoopsMs} ms`);
  console.log("Sandbox: danger-full-access");
  console.log("Approval policy: never");
  console.log(`Auto compaction threshold: ${autoCompactTokenLimit} tokens`);
  console.log(`Loop starts in ${Math.floor(startupDelayMs / 1000)} seconds unless you press Enter to skip...`);

  await countdownDelay(startupDelayMs, "Loop start");

  if (state.sessionId) {
    console.log(`Resuming saved session: ${state.sessionId}`);
  }

  while (!stopRequested) {
    const turnNumber = (state.turnCount ?? 0) + 1;
    const resumeSessionId = state.sessionId ?? null;

    console.log("");
    console.log(`=== Turn ${turnNumber}${resumeSessionId ? ` | ${resumeSessionId}` : " | new session"} ===`);

    try {
      const result = await runTurn(config, resumeSessionId);
      state = {
        createdAt: state.createdAt ?? new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        projectPath: config.projectPath,
        sessionId: result.sessionId,
        turnCount: turnNumber,
        lastUsage: result.usage,
      };
      await saveState(state);

      console.log(`Saved session: ${result.sessionId}`);
      if (result.usage) {
        console.log(
          `Usage: input=${result.usage.input_tokens ?? "?"}, cached=${result.usage.cached_input_tokens ?? "?"}, output=${result.usage.output_tokens ?? "?"}`
        );
      }
    } catch (error) {
      const message = errorMessage(error);
      console.error(message);

      if (resumeSessionId && looksLikeMissingSession(message)) {
        console.error("Saved session could not be resumed. Clearing local state and starting a new session.");
        state = {};
        await saveState(state);
      } else if (!stopRequested) {
        console.error(`Turn failed. Retrying in ${retryDelayMs} ms.`);
        await countdownDelay(retryDelayMs, "Retry");
      }

      continue;
    }

    if (!stopRequested && config.delayBetweenLoopsMs > 0) {
      await countdownDelay(config.delayBetweenLoopsMs, "Next loop");
    }
  }

  console.log("Loop stopped.");
}

async function runTurn(config, sessionId) {
  const args = sessionId ? buildResumeArgs(config, sessionId) : buildNewTurnArgs(config);
  const child = spawn(codexBin, args, {
    cwd: config.projectPath,
    stdio: ["ignore", "pipe", "pipe"],
  });

  let resolvedSessionId = sessionId ?? null;
  let usage = null;
  let sawTurnCompleted = false;
  let stderrBuffer = "";

  const stdoutLines = readline.createInterface({ input: child.stdout });
  const stdoutClosed = new Promise((resolve) => stdoutLines.once("close", resolve));
  stdoutLines.on("line", (line) => {
    if (!line.trim()) {
      return;
    }

    let event;
    try {
      event = JSON.parse(line);
    } catch {
      printSection("stdout", line);
      return;
    }

    printEvent(event);

    if (event.type === "thread.started" && typeof event.thread_id === "string") {
      resolvedSessionId = event.thread_id;
      return;
    }

    if (event.type === "turn.completed") {
      sawTurnCompleted = true;
      usage = event.usage ?? null;
    }
  });

  const stderrLines = readline.createInterface({ input: child.stderr });
  stderrLines.on("line", (line) => {
    if (!line.trim()) {
      return;
    }
    stderrBuffer += `${line}\n`;
    printSection("stderr", line);
  });

  const exitCode = await new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("close", resolve);
  });

  await stdoutClosed;

  if (exitCode !== 0 || !sawTurnCompleted || !resolvedSessionId) {
    throw new Error(
      `Codex turn failed${exitCode !== 0 ? ` (exit ${exitCode})` : ""}: ${stderrBuffer.trim() || "missing completion event"}`
    );
  }

  return {
    sessionId: resolvedSessionId,
    usage,
  };
}

function buildNewTurnArgs(config) {
  return [
    "exec",
    ...buildSharedArgs(config),
    config.loopPrompt,
  ];
}

function buildResumeArgs(config, sessionId) {
  return [
    "exec",
    "resume",
    ...buildSharedArgs(config),
    sessionId,
    config.loopPrompt,
  ];
}

function buildSharedArgs(config) {
  return [
    "--json",
    "--output-last-message",
    lastMessageFile,
    "--dangerously-bypass-approvals-and-sandbox",
    "-c",
    'approval_policy="never"',
    "-c",
    'sandbox_mode="danger-full-access"',
    "-c",
    `model_auto_compact_token_limit=${autoCompactTokenLimit}`,
    "-c",
    `model_reasoning_effort="${config.reasoningLevel}"`,
    "--model",
    config.model,
  ];
}

async function ensureRuntime() {
  await fs.mkdir(runtimeDir, { recursive: true });

  try {
    await fs.access(codexBin);
  } catch {
    throw new Error(
      `Missing local Codex package at ${codexBin}. Run "npm install" inside ${agentDir} first.`
    );
  }
}

async function loadConfig() {
  try {
    const raw = await fs.readFile(configFile, "utf8");
    return normalizeConfig(JSON.parse(raw));
  } catch (error) {
    if (error?.code === "ENOENT") {
      const config = normalizeConfig(defaultConfig);
      await fs.writeFile(configFile, `${JSON.stringify(config, null, 2)}\n`, "utf8");
      console.log(`Created default config at ${configFile}`);
      return config;
    }
    throw error;
  }
}

async function ensureConfigFileExists() {
  try {
    await fs.access(configFile);
  } catch (error) {
    if (error?.code !== "ENOENT") {
      throw error;
    }

    const config = normalizeConfig(defaultConfig);
    await fs.writeFile(configFile, `${JSON.stringify(config, null, 2)}\n`, "utf8");
    console.log(`Created default config at ${configFile}`);
  }
}

async function askDangerousApproval() {
  const rl = readline.createInterface({
    input: process.stdin,
    output: process.stdout,
  });

  try {
    const answer = await question(
      rl,
      "Approve dangerous Codex execution with full permissions for this run? Type 'yes' to continue: "
    );
    return answer.trim().toLowerCase() === "yes";
  } finally {
    rl.close();
  }
}

async function editConfigInNano() {
  console.log(`Opening ${configFile} in nano. Save and exit nano to continue.`);

  const exitCode = await new Promise((resolve, reject) => {
    const child = spawn("nano", [configFile], {
      cwd: agentDir,
      stdio: "inherit",
    });

    child.once("error", reject);
    child.once("close", resolve);
  });

  if (exitCode !== 0) {
    throw new Error(`nano exited with code ${exitCode}.`);
  }
}

function normalizeConfig(config) {
  const merged = { ...defaultConfig, ...config };
  const projectPath = path.resolve(agentDir, String(merged.projectPath ?? "").trim());
  const model = String(merged.model ?? "").trim();
  const reasoningLevel = String(merged.reasoningLevel ?? "").trim();
  const loopPrompt = String(merged.loopPrompt ?? "").trim();
  const delayBetweenLoopsMs = Number.parseInt(String(merged.delayBetweenLoopsMs ?? ""), 10);

  if (!model) {
    throw new Error(`Invalid config in ${configFile}: "model" must be a non-empty string.`);
  }

  if (!reasoningLevel || !validReasoningLevels.has(reasoningLevel)) {
    throw new Error(
      `Invalid config in ${configFile}: "reasoningLevel" must be one of ${Array.from(validReasoningLevels).join(", ")}.`
    );
  }

  if (!Number.isFinite(delayBetweenLoopsMs) || delayBetweenLoopsMs < 0) {
    throw new Error(`Invalid config in ${configFile}: "delayBetweenLoopsMs" must be a non-negative integer.`);
  }

  if (!loopPrompt) {
    throw new Error(`Invalid config in ${configFile}: "loopPrompt" must be a non-empty string.`);
  }

  return {
    projectPath,
    model,
    reasoningLevel,
    loopPrompt,
    delayBetweenLoopsMs,
  };
}

async function loadState() {
  try {
    const raw = await fs.readFile(stateFile, "utf8");
    return JSON.parse(raw);
  } catch (error) {
    if (error?.code === "ENOENT") {
      return {};
    }
    throw error;
  }
}

async function saveState(state) {
  await fs.writeFile(stateFile, `${JSON.stringify(state, null, 2)}\n`, "utf8");
}

function looksLikeMissingSession(message) {
  const normalized = message.toLowerCase();
  return (
    normalized.includes("resume") &&
    (normalized.includes("not found") ||
      normalized.includes("no session") ||
      normalized.includes("unknown session") ||
      normalized.includes("unknown thread"))
  );
}

function errorMessage(error) {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

function question(rl, prompt) {
  return new Promise((resolve) => rl.question(prompt, resolve));
}

async function countdownDelay(ms, label) {
  const totalSeconds = Math.ceil(ms / 1000);
  if (totalSeconds <= 0) {
    return;
  }

  printSection("countdown", `${label} begins in ${totalSeconds}s. Press Enter to skip.`);

  if (!process.stdin.isTTY) {
    for (let secondsRemaining = totalSeconds; secondsRemaining > 0; secondsRemaining -= 1) {
      process.stdout.write(`\r${label} in ${secondsRemaining}s.                    `);
      await delay(1000);
    }
    process.stdout.write(`\r${label} now.                         \n`);
    return;
  }

  let skipped = false;
  const onData = (chunk) => {
    const text = chunk.toString();
    if (text === "\n" || text === "\r\n" || text.trim() === "") {
      skipped = true;
    }
  };

  process.stdin.resume();
  process.stdin.on("data", onData);

  try {
    for (let secondsRemaining = totalSeconds; secondsRemaining > 0; secondsRemaining -= 1) {
      process.stdout.write(`\r${label} in ${secondsRemaining}s. Press Enter to skip.   `);
      await delay(1000);
      if (skipped) {
        break;
      }
    }
  } finally {
    process.stdin.removeListener("data", onData);
  }

  if (skipped) {
    process.stdout.write(`\r${label} skipped.                     \n`);
    return;
  }

  process.stdout.write(`\r${label} now.                         \n`);
}

function printEvent(event) {
  const titleParts = [String(event.type ?? "event")];
  if (event.item?.type) {
    titleParts.push(String(event.item.type));
  }

  let body = null;

  if (typeof event.item?.text === "string" && event.item.text.trim()) {
    body = event.item.text;
  } else if (event.usage && typeof event.usage === "object") {
    body = event.usage;
  } else {
    body = event;
  }

  printSection(titleParts.join(" | "), body);
}

function printSection(title, body) {
  console.log("");
  console.log(eventDivider);
  console.log(title);
  if (typeof body === "string") {
    const text = body.trimEnd();
    if (text) {
      console.log(text);
    }
    return;
  }
  if (body != null) {
    console.log(body);
  }
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
