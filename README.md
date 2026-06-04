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

## Scoped Suppressions

Use `ignore.paths` only for files that should not be checked by any rule, such as generated output. When one rule is noisy for a known path but other rules should still run there, add a scoped suppression:

```toml
[[suppressions]]
rule = "go-effective-go.no-blank-placeholder-assignment"
paths = ["apps/backend/internal/bootstrap/public_track_*_router.go"]
reason = "Generated router adapters intentionally discard unused generated parameters."
```

Scoped suppressions are applied after checks run. They hide only diagnostics whose `rule` and `path` both match, so the same files remain visible to every other rule. `reason` is optional but recommended for future reviewers and AI agents.
