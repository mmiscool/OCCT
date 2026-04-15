# Codex Port Loop

This runner uses the local `@openai/codex` package, reuses the Codex CLI login already configured on this machine, keeps one persistent session, and repeatedly sends:

`Keep going porting from C++ to rust.`

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
  "delayBetweenLoopsMs": 1000
}
```

Fields:

- `projectPath`: absolute or relative path to the project Codex should work in.
- `model`: Codex model name.
- `reasoningLevel`: one of `none`, `minimal`, `low`, `medium`, `high`, or `xhigh`.
- `delayBetweenLoopsMs`: non-negative integer delay between completed turns.

## Behavior

- The prompt is fixed in code and is not configurable.
- Automatic compaction is hard-coded with `model_auto_compact_token_limit=120000`.
- The active session state is stored in `codex-port-loop/.runtime/state.json`.
- On every launch, the runner asks once for explicit dangerous-mode approval.
- After approval, it opens `config.json` in `nano` and waits for `nano` to exit.
- The loop starts 20 seconds after you close `nano`.
- After that confirmation step, the runner uses `--dangerously-bypass-approvals-and-sandbox` with `approval_policy="never"` and `sandbox_mode="danger-full-access"` for the Codex turns in that run.
- Every streamed Codex event is printed live to the terminal with a horizontal separator, including messages, tool activity, completions, and stderr lines.

## Reset the saved session

Delete `codex-port-loop/.runtime/state.json`.
