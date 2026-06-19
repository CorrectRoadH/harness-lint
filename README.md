<div align="center">

<img src="assets/harness-lint-readme.png" alt="harness lint — turn repeated AI mistakes into fast, strict lint rules" width="860" />

[![CI](https://img.shields.io/github/actions/workflow/status/CorrectRoadH/harness-lint/ci.yml?branch=main&label=CI&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/CorrectRoadH/harness-lint?label=release&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/releases)
[![rust](https://img.shields.io/badge/rust-2024-blue?style=flat-square)](Cargo.toml)
[![homebrew](https://img.shields.io/badge/homebrew-CorrectRoadH%2Ftap-fbb040?style=flat-square)](https://github.com/CorrectRoadH/homebrew-tap)

[English](README.md) · [简体中文](README.zh.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

</div>

> **Lint Driven Development for coding agents** — when the AI ignores your instructions, turn the correction into a fast, strict lint rule so it can't make the same mistake again.

harness-lint is a next-generation lint tool for Harness Engineering. In vibe coding, AI often ignores your instructions, even after repeated corrections or after you write them into `AGENTS.md`. This tool solves that problem with Lint Driven Development. When a user tells an AI agent what not to do, the requirement is first converted into a fixed lint rule. Fast, strict checks then prevent the AI from making the same mistake again.

Compared with traditional lint tools, harness lint rules are highly human-readable and easy to understand. They are also designed for AI coding workflows and best practices.

## Install

```sh
brew install getgrit/tap/grit
brew install CorrectRoadH/tap/harness-lint
```

To install the companion Codex skill:

```sh
npx skills add CorrectRoadH/harness-lint
```

## Init Harness Lint For Your Repo, For Agents

```text
READ https://raw.githubusercontent.com/CorrectRoadH/harness-lint/refs/heads/main/INIT.md and install harness lint for this code repo
```

## Agent Plugins (Claude Code & Codex)

A static block in `AGENTS.md` is read once, far from the moment the agent acts.
The plugins in [`plugins/`](plugins/) instead use lifecycle hooks to re-inject the
Lint Driven Development guidance every session, and — more usefully — run
`harness-lint check --changed` on each prompt and feed the **actual current
violations** to the agent right before it writes more code.

Claude Code:

```text
/plugin marketplace add CorrectRoadH/harness-lint
/plugin install harness-lint@harness-lint
```

Codex:

```text
codex plugin marketplace add CorrectRoadH/harness-lint
codex plugin install harness-lint
```

Both also ship a `/harness-lint-capture` command that reviews a session's
feedback and turns reusable corrections into rules (the other half of LDD). See
[`plugins/README.md`](plugins/README.md) for details.

## Common Commands

```sh
harness-lint check --changed
harness-lint check --all
harness-lint rule list
harness-lint search "python typing"
harness-lint list --available
harness-lint install python
harness-lint install python-pep8
harness-lint outdated
harness-lint update
harness-lint remove python
```

## Configuration Demo

`harness.toml` controls which files are checked, where local rules live, which rule packs are installed, and which rule results should be treated differently.

```toml
# Optional project name shown in generated config.
[project]
name = "my-service"

# Default lint behavior.
[lint]
# warn reports a problem; error makes the check fail.
default_level = "warn"
# Used by `harness-lint check --changed`.
changed_base = "origin/main"
# Reuse file-level results between runs.
cache = true

# Local project-owned rule files.
[rules]
local = ["rules"]

# Shared rule packs to install and restore.
[packs]
typescript = "github:CorrectRoadH/harness-lint@main#packs/typescript"

# Change the level of one rule without editing the rule file.
[overrides]
"typescript.no-console-log" = "error"

# Turn off specific rules.
[disabled]
rules = ["typescript.no-explicit-any"]

# Skip these paths for every rule. Nothing scans them.
[ignore]
paths = ["dist/**", "coverage/**"]

# A named file region most rules should skip, but a few rules need.
# default_rules = false removes it from the default region, so ordinary
# rules don't scan it; provides lists portable concept names a shared pack
# rule can target without hardcoding your layout.
[file_sets.generated]
paths = ["backend/gen/**/*.pb.go", "packages/proto/gen/**"]
default_rules = false
provides = ["generated"]

# Hide one rule only for matching paths; other rules still check those files.
[[exceptions]]
rule = "typescript.no-console-log"
paths = ["src/generated/**"]
reason = "Generated SDK code is checked in and emits debug output during local mocks."
```

A rule opts into a region with `runs_on` in its frontmatter. With no `runs_on`, a rule scans the **default** region (everything visible that no `default_rules = false` set claims):

```markdown
---
id: local.proto-no-id-getter
title: Proto messages must generate GetId
language: go
runs_on: ["generated"]   # only the generated region; never ordinary source
---
```

### How configuration composes

harness-lint answers three independent questions, in order. Keeping them separate is what lets the knobs above stack predictably.

1. **Is the rule on?** A pack's default-disabled list and `[disabled]` turn a rule off entirely; `[overrides]` only changes its severity. An off rule skips the rest.
2. **Which files does the rule scan?** Start from the repo, then apply, in precedence order:
   - structural exclusions — `.git`, `node_modules`, `target`, `.harness`, your rule directories, and `.gitignore`d files are never scannable and nothing overrides this;
   - `[ignore].paths` — removed from every rule; nothing can opt back in;
   - **file sets** — the remaining files are partitioned. A `default_rules = false` set is removed from the `default` region; a rule reaches it only by naming the set (or a concept it `provides`) in `runs_on`. A rule with no `runs_on` scans `default`;
   - the rule's language and any GritQL `$filename` predicate then narrow what remains.
3. **Are the results reported?** `[[exceptions]]` hides a scanned rule's diagnostics on matching paths.

`runs_on` is exclusive scope, not a back door: a rule reaches a default-closed file set only because it asked, and only ever that rule. The set's *location* (`paths`) is project-owned in `harness.toml`; the rule's *target* is a portable name (`generated`), so a shared pack rule can ship `runs_on: ["generated"]` without knowing where your generated code lives — you connect the two with one `provides`. Rename the file set freely; as long as its `provides` still lists the concept, every pack rule keeps working. Need both ordinary source and a region? List both: `runs_on: ["default", "generated"]`.

harness-lint also checks its own config integrity: `[[exceptions]]` / `[ignore]` / `[file_sets.*]` paths that no longer exist, file sets that overlap `[ignore]` or have no paths, `[disabled]` / `[overrides]` entries that name an unknown rule, and any rule whose `runs_on` names a file set or concept nothing provides — all reported (warn by default, file-set/run-target structural errors at error; adjust per id with `[overrides]`).

When harness-lint detects a deprecated or never-implemented construct (such as `[[suppressions]]` or `[[scan_ignored]]`), it prints a warning linking to [MIGRATE.md](MIGRATE.md), which gives a mechanical migration for each. An AI agent set up with the harness-lint skill follows that link and applies the migration for you.

For newly added (non-breaking) features and when to adopt them, run `harness-lint whatsnew` or see [WHATS-NEW.md](WHATS-NEW.md). Each entry states when a feature pays off and when to keep your existing setup, so an agent suggests it only where it fits rather than nagging.

## Local Rules

Custom project rules live in `rules/*.md` by default. To put them somewhere else, configure `harness.toml`:

```toml
[rules]
local = ["custom-rules"]
```

`rule create` writes new rules to the first configured local rule directory. A local rule must include executable GritQL at creation time:

```sh
harness-lint rule create "Avoid print debugging" --language python --grit '`print($value)`'
```

If feedback cannot be expressed as a reliable GritQL pattern, do not create a harness-lint rule for it. Keep that guidance in agent instructions, review checklists, or project docs instead.

After creating a rule, run it by itself and confirm it reports the expected file(s) before relying on broader checks. Do not pass paths to `check` to simulate rule scope; if the rule should only apply to certain files, encode that in GritQL with `$filename`.

```sh
harness-lint rule verify local.no-print
harness-lint check --all --rule local.no-print
```

Rule file example:

````markdown
---
id: local.no-print
title: Avoid print debugging
language: python
level: warn
skill: tdd
tags: [local, python]
---

# Avoid print debugging

Use logging instead of committed print calls.

```grit
language python
`print($value)`
```

## Bad

```python
print(user)
```

## Good

```python
logger.info("user=%s", user)
```
````

To limit a rule to specific files, write the file scope directly in GritQL with `$filename`:

```grit
language js
`console.log($value)` where {
  $filename <: r".*src/.*\.ts",
  !$filename <: r".*\.test\.ts"
}
```
