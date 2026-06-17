# harness-lint agent plugins

These plugins inject harness-lint guidance and **live diagnostics** into your
coding agent through lifecycle hooks, instead of (or in addition to) a static
block in `AGENTS.md`.

## Why hooks instead of AGENTS.md

A block in `AGENTS.md` is read once, far from the moment the agent acts, and is
easily buried under the rest of the context. A hook re-injects the guidance at
the top of context every session and every prompt, and — more importantly — runs
`harness-lint check --changed` and feeds the **actual current violations** to the
agent right before it writes the next line of code. The agent reacts to concrete
diagnostics, not a general reminder.

Both plugins register the same two hooks:

- **`SessionStart`** — injects the Lint Driven Development (LDD) working
  guidance and any diagnostics already present on changed files.
- **`UserPromptSubmit`** — runs `harness-lint check --changed`; when there are
  diagnostics it injects them so the agent fixes them per LDD. It stays silent
  when the tree is clean to avoid context noise.

Both degrade gracefully: if the `harness-lint` binary is not on `PATH`, the
hooks exit without error (SessionStart prints a one-line install nudge).

## Capture command

The hooks cover the "catch and fix" half of LDD. The other half — turning a
correction into a durable rule — is a manual command you run when a session
actually produced reusable feedback:

- **Claude Code:** `/harness-lint-capture`
- **Codex:** `/harness-lint-capture`

It tells the agent to review the session's feedback, keep only what can be a
reliable GritQL pattern, and run the rule-authoring workflow (`rule create` →
fill examples → `doctor` → `rule verify` → `check`). It is deliberately manual:
most turns have nothing rule-worthy, so forcing a capture on every stop would
just burn context.

> The hooks require a POSIX `sh` and the `harness-lint` binary. Windows is not
> covered yet.

## Claude Code

Install from this repo's plugin marketplace:

```text
/plugin marketplace add CorrectRoadH/harness-lint
/plugin install harness-lint@harness-lint
```

The plugin lives in [`claude-code/`](./claude-code) and references its scripts via
`${CLAUDE_PLUGIN_ROOT}`, so it works wherever Claude Code installs it.

## Codex

Codex uses the same hooks schema but loads project hooks from `.codex/`. From your
repo root:

```sh
mkdir -p .codex/hooks
cp path/to/harness-lint/plugins/codex/hooks.json .codex/hooks.json
cp path/to/harness-lint/plugins/codex/hooks/*.sh .codex/hooks/
chmod +x .codex/hooks/*.sh
```

Project-local hooks only load once the `.codex/` layer is trusted. To enable the
hooks for every repo instead, place the same `hooks.json` and `hooks/` scripts
under `~/.codex/` (adjust the script paths in `hooks.json` to absolute paths).

For the `/harness-lint-capture` command, copy the prompt into Codex's prompts
directory:

```sh
mkdir -p ~/.codex/prompts
cp path/to/harness-lint/plugins/codex/prompts/harness-lint-capture.md ~/.codex/prompts/
```

(The Claude Code plugin ships this command automatically — no extra step.)
