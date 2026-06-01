# harness lint

harness-lint is a next-generation lint tool for Harness Engineering. In vibe coding, AI often ignores your instructions, even after repeated corrections or after you write them into `AGENTS.md`. This tool solves that problem with Lint Driven Development. When a user tells an AI agent what not to do, the requirement is first converted into a fixed lint rule. Fast, strict checks then prevent the AI from making the same mistake again.

Compared with traditional lint tools, harness lint rules are highly human-readable and easy to understand. They are also designed for AI coding workflows and best practices.

[中文 README](README.zh.md)

## Install

```sh
brew install getgrit/tap/grit
brew install CorrectRoadH/tap/harness-lint
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

`rule create` writes new rules to the first configured local rule directory.

Rule file example:

````markdown
---
id: local.no-print
title: Avoid print debugging
language: python
level: warn
status: draft
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
