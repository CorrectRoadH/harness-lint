# harness-lint

Tiny CLI for a GritQL rule ecosystem.

harness-lint does three things:

- initialize a project for GritQL rules
- install and update rule packs
- run lint checks and help agents turn feedback into draft rules

It does not fix code. GritQL is the only executable rule language.

## Install

```sh
cargo build
target/debug/harness-lint --help
```

`grit` must be installed separately for `check`.

## Project Setup

```sh
harness-lint init
```

This creates:

```text
harness.toml
harness/rules/
.harness/
```

Commit `harness.toml` and `harness/rules/`. Ignore `.harness/`.

## Agent Install Prompt

Give this to an LLM coding agent:

```text
install harness: read CLAUDE.md, AGENTS.md, .cursor/rules, README.md, and relevant docs.
Run `harness-lint init`.
For each durable coding constraint, run `harness-lint rule suggest "<constraint>"`.
If registry candidates exist, ask before installing the rule pack.
If no good pack exists, create a local draft rule.
Rules must be GritQL; uncertain rules stay draft.
Run `harness-lint rule list` and summarize the result.
```

## CLI

```sh
harness-lint check --changed
harness-lint check --staged
harness-lint check [paths...]
harness-lint check --all
harness-lint pack search "python typing"
harness-lint pack inspect python
harness-lint pack add <id> <local:PATH|github:OWNER/REPO@TAG>
harness-lint pack update
harness-lint pack list
harness-lint rule suggest "<feedback>"
harness-lint rule suggest --local "<feedback>"
harness-lint rule new <id> <title> --language <language>
harness-lint rule list
harness-lint rule explain <rule-id>
```

Use `--json` with `check` or `rule list` when another tool needs structured output.

## Rule Packs

The intended ecosystem flow is:

```sh
harness-lint pack search "pydantic typed service rules"
harness-lint pack inspect python
harness-lint pack add python github:CorrectRoadH/harness-lint@main#packs/python
harness-lint check --changed
```

Search uses local project signals such as `pyproject.toml`, `go.mod`, `package.json`, `tsconfig.json`, and common library names, then prints install commands. `inspect` shows the pack before it mutates the project. Installed pack origins are recorded in `harness.lock`.

This repository includes seed packs under `packs/python`, `packs/go`, and `packs/typescript`. For local development:

```sh
harness-lint pack add python local:/Users/ctrdh/Code/harness-lint/packs/python
```

Custom project rules live directly in `harness/rules/*.md`.

## Rule File

````markdown
---
id: local.no-print
title: Avoid print debugging
language: python
level: warn
status: draft
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
