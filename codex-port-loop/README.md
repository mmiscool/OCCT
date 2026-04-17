# Codex Port Loop

This runner uses the local `@openai/codex` package, reuses the Codex CLI login already configured on this machine, keeps a saved Codex session until it is rotated, and repeatedly sends the configured loop prompt.

The default prompt is milestone-driven:

`Drive the Rust port forward in milestone-sized steps. Read portingMilestones.md and nextStep.md at the start of every turn.`

## Install

```bash
cd /home/user/projects/OCCT
pnpm install
```

## Run

```bash
cd /home/user/projects/OCCT
pnpm codexAgent
```

## Config

On first run, the script creates `config.json` in this folder if it does not already exist.

Default generated config:

```json
{
  "projectPath": "/home/user/projects/OCCT",
  "model": "gpt-5.4",
  "reasoningLevel": "xhigh",
  "loopPrompt": "Drive the Rust port forward in milestone-sized steps. Read `portingMilestones.md` and `nextStep.md` at the start of every turn. Work on the highest-priority incomplete milestone. Prefer deleting OCCT fallbacks, moving a user-visible capability to Rust, and adding regression coverage over reshuffling a single seam. Do not spend two consecutive turns on a refactor unless that turn removes a fallback or lands new tested Rust-owned behavior. Update both files every turn with completed evidence, the active milestone, the next bounded cut, and exact verification commands. You may use subagents, delegation, and parallel agent work when useful. Prefer bounded, non-overlapping subtasks.",
  "delayBetweenLoopsSeconds": 1,
  "maxSessionTurns": 30
}
```

Fields:

- `projectPath`: absolute or relative path to the project Codex should work in.
- `model`: Codex model name.
- `reasoningLevel`: one of `none`, `minimal`, `low`, `medium`, `high`, or `xhigh`.
- `loopPrompt`: the prompt sent on each loop iteration. If it does not already authorize delegation, the runner appends a subagent-permission suffix automatically.
- `delayBetweenLoopsSeconds`: non-negative integer delay between completed turns.
- `maxSessionTurns`: positive integer cap for how many turns a single saved Codex session may accumulate before the runner starts a fresh one.

## Behavior

- Automatic compaction is hard-coded with `model_auto_compact_token_limit=120000`.
- The active session state is stored in `codex-port-loop/.runtime/state.json`.
- The runner starts a fresh Codex session when the saved loop strategy changes (`projectPath`, `model`, `reasoningLevel`, `loopPrompt`, or `maxSessionTurns`) or when the current session reaches `maxSessionTurns`.
- On every launch, the runner asks once for explicit dangerous-mode approval.
- After approval, it opens `config.json` in `nano` and waits for `nano` to exit.
- The loop starts 20 seconds after you close `nano`, unless you press `Enter` to skip the pause.
- After that confirmation step, the runner uses `--dangerously-bypass-approvals-and-sandbox` with `approval_policy="never"` and `sandbox_mode="danger-full-access"` for the Codex turns in that run.
- Every Codex turn is launched with `features.multi_agent=true`.
- Every streamed Codex event is printed live to the terminal with a horizontal separator, including messages, tool activity, completions, and stderr lines.
- Every pause shows a live countdown in the terminal, and pressing `Enter` skips the remaining wait.
- Pressing `Ctrl+C` stops the current wait immediately, signals any active child process to exit, and force-exits if shutdown does not complete promptly.
- The default prompt expects two control files in the repo root: `portingMilestones.md` for milestone ordering and `nextStep.md` for the current bounded cut.

## Reset the saved session

Delete `codex-port-loop/.runtime/state.json`.
