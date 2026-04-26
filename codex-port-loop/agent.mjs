import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
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

const retryDelaySeconds = 10;
const autoCompactTokenLimit = 120000;
const startupDelaySeconds = 20;
const validReasoningLevels = new Set(["none", "minimal", "low", "medium", "high", "xhigh"]);
const eventDivider = "-".repeat(100);
const subagentPermissionSuffix =
  " You may use subagents, delegation, and parallel agent work when useful. Prefer bounded, non-overlapping subtasks.";
const defaultLoopPrompt =
  "Drive the Rust port forward in decisive implementation steps. Read `portingMilestones.md` and `nextStep.md` at the start of every turn, then immediately work on the highest-priority incomplete milestone. Each turn must attempt a meaningful Rust-owned replacement of an OCCT-backed path, not merely analysis, observability, bookkeeping, or helper reshuffling. Prefer replacing an entire exercised fallback branch or capability family over making the smallest local edit. It is acceptable and expected to touch multiple Rust modules, C ABI glue, tests, and docs in one turn when that is what the port requires. When you find the active fallback, implement the Rust-owned path and remove or strictly narrow the fallback in the same turn; do not stop after adding probes unless the same turn also lands tested Rust behavior. Use compiler errors and failing tests as guidance to finish the larger porting cut, not as a reason to retreat to a tiny safe change. If a prerequisite refactor is needed, do it only as part of the same turn that ports behavior or deletes a fallback. Add or strengthen regression coverage around the user-visible behavior being moved to Rust. Update both control files every turn with completed evidence, the active milestone, the next bounded cut, and exact verification commands.";
const defaultConfig = {
  projectPath: packageRoot,
  model: "gpt-5.5",
  reasoningLevel: "xhigh",
  loopPrompt: `${defaultLoopPrompt}${subagentPermissionSuffix}`,
  delayBetweenLoopsSeconds: 1,
  maxSessionTurns: 30,
};

let stopRequested = false;
let shutdownRequested = false;
let activeChild = null;
let activeChildLabel = null;
let activeDelayController = null;
let forceExitTimer = null;

process.on("SIGINT", () => handleStopSignal("SIGINT"));

process.on("SIGTERM", () => handleStopSignal("SIGTERM"));

main().catch((error) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exitCode = 1;
});

async function main() {
  await ensureRuntime();

  await ensureConfigFileExists();

  
  await editConfigInNano();
  if (stopRequested) {
    console.log("Loop stopped.");
    return;
  }
  const config = await loadConfig();
  let state = await loadState();

  state = alignStateWithConfig(state, config);

  console.log(`Config file: ${configFile}`);
  console.log(`Project path: ${config.projectPath}`);
  console.log(`Prompt: ${config.loopPrompt}`);
  console.log(`Model: ${config.model}`);
  console.log(`Reasoning: ${config.reasoningLevel}`);
  console.log(`Delay between loops: ${config.delayBetweenLoopsSeconds} seconds`);
  console.log(`Max turns per session: ${config.maxSessionTurns}`);
  console.log("Sandbox: danger-full-access");
  console.log("Approval policy: never");
  console.log(`Auto compaction threshold: ${autoCompactTokenLimit} tokens`);
  console.log(`Loop starts in ${startupDelaySeconds} seconds unless you press Enter to skip...`);

  await countdownDelay(startupDelaySeconds, "Loop start");

  if (stopRequested) {
    console.log("Loop stopped.");
    return;
  }

  if (state.sessionId) {
    console.log(
      `Resuming saved session: ${state.sessionId} (${state.sessionTurnCount ?? 0}/${config.maxSessionTurns} turns in current session)`
    );
  }

  while (!stopRequested) {
    if (state.sessionId && (state.sessionTurnCount ?? 0) >= config.maxSessionTurns) {
      console.log(
        `Current session reached ${config.maxSessionTurns} turn(s). Starting a new Codex session for the next loop.`
      );
      state = resetSessionState(state, config);
      await saveState(state);
    }

    const turnNumber = (state.turnCount ?? 0) + 1;
    const resumeSessionId = state.sessionId ?? null;
    const sessionTurnNumber = resumeSessionId ? (state.sessionTurnCount ?? 0) + 1 : 1;

    console.log("");
    console.log(
      `=== Turn ${turnNumber} | session turn ${sessionTurnNumber}${resumeSessionId ? ` | ${resumeSessionId}` : " | new session"} ===`
    );

    try {
      const result = await runTurn(config, resumeSessionId);
      const nextSessionTurnCount = resumeSessionId ? (state.sessionTurnCount ?? 0) + 1 : 1;
      state = {
        createdAt: state.createdAt ?? new Date().toISOString(),
        updatedAt: new Date().toISOString(),
        projectPath: config.projectPath,
        configSignature: buildConfigSignature(config),
        sessionId: result.sessionId,
        turnCount: turnNumber,
        sessionTurnCount: nextSessionTurnCount,
        lastUsage: result.usage,
      };
      await saveState(state);

      console.log(`Saved session: ${result.sessionId}`);
      if (result.usage) {
        console.log(
          `Usage: input=${result.usage.input_tokens ?? "?"}, cached=${result.usage.cached_input_tokens ?? "?"}, output=${result.usage.output_tokens ?? "?"}`
        );
      }

      if (nextSessionTurnCount >= config.maxSessionTurns) {
        console.log("Session turn cap reached. The next loop will start a fresh Codex session.");
      }

      commitAndPush(config, state);
    } catch (error) {
      const message = errorMessage(error);
      if (!stopRequested) {
        console.error(message);
      }

      if (resumeSessionId && looksLikeMissingSession(message)) {
        console.error("Saved session could not be resumed. Clearing local state and starting a new session.");
        state = resetSessionState(state, config);
        await saveState(state);
      } else if (!stopRequested) {
        console.error(`Turn failed. Retrying in ${retryDelaySeconds} seconds.`);
        await countdownDelay(retryDelaySeconds, "Retry");
      }

      continue;
    }

    if (!stopRequested && config.delayBetweenLoopsSeconds > 0) {
      await countdownDelay(config.delayBetweenLoopsSeconds, "Next loop");
      await commitAndPush(config, state);
    }
  }

  console.log("Loop stopped.");
}

async function commitAndPush(config, state) {
  const commitMessage = `Codex loop checkpoint - turn ${state.turnCount}`;

  try {
    await runGitCommand(config.projectPath, ["add", "."]);
    await runGitCommand(config.projectPath, ["commit", "-m", commitMessage], [0, 1]);
    await runGitCommand(config.projectPath, ["push"]);
  } catch (error) {
    console.error(`Failed to create git checkpoint: ${errorMessage(error)}`);
  }
}

async function runGitCommand(cwd, args, allowedExitCodes = [0]) {
  const result = spawnSync("git", args, {
    cwd,
    stdio: "inherit",
  });

  if (result.error) {
    throw result.error;
  }

  if (result.signal) {
    throw new Error(`git ${args[0]} terminated by signal ${result.signal}`);
  }

  if (allowedExitCodes.includes(result.status)) {
    return;
  }

  throw new Error(`git ${args[0]} exited with code ${result.status}`);
}

async function runTurn(config, sessionId) {
  const args = sessionId ? buildResumeArgs(config, sessionId) : buildNewTurnArgs(config);
  const child = spawn(codexBin, args, {
    cwd: config.projectPath,
    stdio: ["ignore", "pipe", "pipe"],
  });
  const childExit = waitForChildExit(child, "the active Codex turn");

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

  const { exitCode, exitSignal } = await childExit;

  await stdoutClosed;

  if (exitCode !== 0 || exitSignal || !sawTurnCompleted || !resolvedSessionId) {
    const exitDetail = exitSignal ? ` (signal ${exitSignal})` : exitCode !== 0 ? ` (exit ${exitCode})` : "";
    throw new Error(
      `Codex turn failed${exitDetail}: ${stderrBuffer.trim() || "missing completion event"}`
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
    "-c",
    "features.multi_agent=true",
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


async function editConfigInNano() {
  console.log(`Opening ${configFile} in nano. Save and exit nano to continue.`);

  const child = spawn("nano", [configFile], {
    cwd: agentDir,
    stdio: "inherit",
  });
  const { exitCode, exitSignal } = await waitForChildExit(child, "the config editor");

  if (stopRequested) {
    return;
  }

  if (exitCode !== 0 || exitSignal) {
    throw new Error(`nano exited${exitSignal ? ` with signal ${exitSignal}` : ` with code ${exitCode}`}.`);
  }
}

function normalizeConfig(config) {
  const merged = { ...defaultConfig, ...config };
  const projectPath = path.resolve(agentDir, String(merged.projectPath ?? "").trim());
  const model = String(merged.model ?? "").trim();
  const reasoningLevel = String(merged.reasoningLevel ?? "").trim();
  const loopPrompt = String(merged.loopPrompt ?? "").trim();
  const delayBetweenLoopsSeconds = parseDelayBetweenLoopsSeconds(merged);
  const maxSessionTurns = parseMaxSessionTurns(merged);

  if (!model) {
    throw new Error(`Invalid config in ${configFile}: "model" must be a non-empty string.`);
  }

  if (!reasoningLevel || !validReasoningLevels.has(reasoningLevel)) {
    throw new Error(
      `Invalid config in ${configFile}: "reasoningLevel" must be one of ${Array.from(validReasoningLevels).join(", ")}.`
    );
  }

  if (!loopPrompt) {
    throw new Error(`Invalid config in ${configFile}: "loopPrompt" must be a non-empty string.`);
  }

  const normalizedLoopPrompt = loopPrompt.includes(subagentPermissionSuffix.trim())
    ? loopPrompt
    : `${loopPrompt}${subagentPermissionSuffix}`;

  return {
    projectPath,
    model,
    reasoningLevel,
    loopPrompt: normalizedLoopPrompt,
    delayBetweenLoopsSeconds,
    maxSessionTurns,
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

function buildConfigSignature(config) {
  return createHash("sha256")
    .update(
      JSON.stringify({
        projectPath: config.projectPath,
        model: config.model,
        reasoningLevel: config.reasoningLevel,
        loopPrompt: config.loopPrompt,
        maxSessionTurns: config.maxSessionTurns,
      })
    )
    .digest("hex");
}

function resetSessionState(state, config) {
  return {
    createdAt: state.createdAt ?? new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    projectPath: config.projectPath,
    configSignature: buildConfigSignature(config),
    turnCount: state.turnCount ?? 0,
    sessionTurnCount: 0,
    lastUsage: state.lastUsage ?? null,
  };
}

function alignStateWithConfig(state, config) {
  const nextState = state ?? {};
  const configSignature = buildConfigSignature(config);
  const hadPersistedState = Object.keys(nextState).length > 0;

  if (!hadPersistedState) {
    return {
      projectPath: config.projectPath,
      configSignature,
      turnCount: 0,
      sessionTurnCount: 0,
    };
  }

  if (nextState.projectPath && nextState.projectPath !== config.projectPath) {
    console.log(
      `Project path changed from ${nextState.projectPath} to ${config.projectPath}. Starting a new Codex session.`
    );
    return resetSessionState(nextState, config);
  }

  if (nextState.configSignature !== configSignature) {
    console.log(
      nextState.configSignature
        ? "Loop config changed. Starting a new Codex session."
        : "Saved session predates the current loop strategy. Starting a new Codex session."
    );
    return resetSessionState(nextState, config);
  }

  if (nextState.sessionId && (nextState.sessionTurnCount ?? 0) >= config.maxSessionTurns) {
    console.log(
      `Saved session already reached ${config.maxSessionTurns} turn(s). Starting a new Codex session.`
    );
    return resetSessionState(nextState, config);
  }

  return {
    ...nextState,
    projectPath: config.projectPath,
    configSignature,
    turnCount: nextState.turnCount ?? 0,
    sessionTurnCount: nextState.sessionTurnCount ?? 0,
  };
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

async function countdownDelay(seconds, label) {
  const totalSeconds = Math.ceil(seconds);
  if (totalSeconds <= 0 || stopRequested) {
    return;
  }

  printSection("countdown", `${label} begins in ${totalSeconds}s. Press Enter to skip.`);
  const delayController = registerActiveDelay();
  let interrupted = false;

  try {
    if (!process.stdin.isTTY) {
      for (let secondsRemaining = totalSeconds; secondsRemaining > 0; secondsRemaining -= 1) {
        if (stopRequested) {
          interrupted = true;
          break;
        }
        process.stdout.write(`\r${label} in ${secondsRemaining}s.                    `);
        try {
          await delay(1000, delayController.signal);
        } catch (error) {
          if (isAbortError(error)) {
            interrupted = true;
            break;
          }
          throw error;
        }
      }

      if (interrupted || stopRequested) {
        process.stdout.write(`\r${label} stopped.                     \n`);
        return;
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
        if (stopRequested) {
          interrupted = true;
          break;
        }
        process.stdout.write(`\r${label} in ${secondsRemaining}s. Press Enter to skip.   `);
        try {
          await delay(1000, delayController.signal);
        } catch (error) {
          if (isAbortError(error)) {
            interrupted = true;
            break;
          }
          throw error;
        }
        if (skipped) {
          break;
        }
      }
    } finally {
      process.stdin.removeListener("data", onData);
      process.stdin.pause();
    }

    if (skipped) {
      process.stdout.write(`\r${label} skipped.                     \n`);
      return;
    }
  } finally {
    clearActiveDelay(delayController);
  }

  if (interrupted || stopRequested) {
    process.stdout.write(`\r${label} stopped.                     \n`);
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

function handleStopSignal(signal) {
  if (shutdownRequested) {
    forceExit(signal);
    return;
  }

  shutdownRequested = true;
  stopRequested = true;
  const target = activeChildLabel ?? (activeDelayController ? "the current wait" : "the loop");

  process.stderr.write(
    `\n${signal === "SIGTERM" ? "Termination requested." : "Stop requested."} Stopping ${target}. Press Ctrl+C again to force exit.\n`
  );

  abortActiveDelay();

  if (activeChild && !activeChild.killed) {
    try {
      activeChild.kill(signal === "SIGTERM" ? "SIGTERM" : "SIGINT");
    } catch {
      // Best effort only.
    }
  }

  if (!activeChild) {
    return;
  }

  forceExitTimer = setTimeout(() => {
    forceExit(signal);
  }, 1000);
  forceExitTimer.unref?.();
}

function forceExit(signal) {
  stopRequested = true;
  abortActiveDelay();

  if (forceExitTimer) {
    clearTimeout(forceExitTimer);
    forceExitTimer = null;
  }

  if (activeChild && !activeChild.killed) {
    try {
      activeChild.kill("SIGKILL");
    } catch {
      // Best effort only.
    }
  }

  process.exit(signal === "SIGTERM" ? 143 : 130);
}

function waitForChildExit(child, label) {
  activeChild = child;
  activeChildLabel = label;

  return new Promise((resolve, reject) => {
    const clear = () => {
      if (activeChild === child) {
        activeChild = null;
        activeChildLabel = null;
      }
      if (forceExitTimer) {
        clearTimeout(forceExitTimer);
        forceExitTimer = null;
      }
    };

    child.once("error", (error) => {
      clear();
      reject(error);
    });

    child.once("close", (exitCode, exitSignal) => {
      clear();
      resolve({ exitCode, exitSignal });
    });
  });
}

function registerActiveDelay() {
  const controller = new AbortController();
  activeDelayController = controller;
  return controller;
}

function clearActiveDelay(controller) {
  if (activeDelayController === controller) {
    activeDelayController = null;
  }
}

function abortActiveDelay() {
  if (activeDelayController) {
    activeDelayController.abort();
    activeDelayController = null;
  }
}

function isAbortError(error) {
  return error instanceof Error && error.name === "AbortError";
}

function delay(ms, signal) {
  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      if (signal) {
        signal.removeEventListener("abort", onAbort);
      }
      resolve();
    }, ms);

    const onAbort = () => {
      clearTimeout(timer);
      reject(Object.assign(new Error("Delay aborted"), { name: "AbortError" }));
    };

    if (!signal) {
      return;
    }

    if (signal.aborted) {
      onAbort();
      return;
    }

    signal.addEventListener("abort", onAbort, { once: true });
  });
}

function parseDelayBetweenLoopsSeconds(config) {
  const secondsValue = String(config.delayBetweenLoopsSeconds ?? "").trim();
  if (/^\d+$/.test(secondsValue)) {
    return Number(secondsValue);
  }
  throw new Error(`Invalid config in ${configFile}: "delayBetweenLoopsSeconds" must be set to a non-negative integer.`);
}

function parseMaxSessionTurns(config) {
  const turnsValue = String(config.maxSessionTurns ?? "").trim();
  if (/^[1-9]\d*$/.test(turnsValue)) {
    return Number(turnsValue);
  }
  throw new Error(`Invalid config in ${configFile}: "maxSessionTurns" must be set to a positive integer.`);
}
