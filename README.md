# harness lint

[![CI](https://img.shields.io/github/actions/workflow/status/CorrectRoadH/harness-lint/ci.yml?branch=main&label=CI&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/CorrectRoadH/harness-lint?label=release&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/releases)
[![rust](https://img.shields.io/badge/rust-2024-blue?style=flat-square)](Cargo.toml)
[![homebrew](https://img.shields.io/badge/homebrew-CorrectRoadH%2Ftap-fbb040?style=flat-square)](https://github.com/CorrectRoadH/homebrew-tap)

[English](README.md) | [简体中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

![harness lint hero](assets/harness-lint-readme.png)

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

# Skip these paths for every rule.
[ignore]
paths = ["dist/**", "coverage/**"]

# Hide one rule only for matching paths; other rules still check those files.
[[exceptions]]
rule = "typescript.no-console-log"
paths = ["src/generated/**"]
reason = "Generated SDK code is checked in and emits debug output during local mocks."
```

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
