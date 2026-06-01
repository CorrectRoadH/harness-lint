---
name: harness-lint
description: Use when installing, configuring, authoring, debugging, or fixing `harness-lint` rules in a code repository. Covers command setup, converting recurring feedback into local rules, writing rule Markdown/GritQL, inspecting a specific rule after lint failures, and fixing code without weakening rules.
metadata:
  short-description: Install and author harness-lint rules
---

# harness-lint

Use this skill whenever a repository is being connected to `harness-lint`, a user asks for a new project rule, or a `harness-lint` check fails.

## Setup

From the target repository root:

```sh
command -v harness-lint
harness-lint doctor
command -v grit
grit --version
```

If missing on macOS/Homebrew:

```sh
brew install getgrit/tap/grit
brew install CorrectRoadH/tap/harness-lint
```

Initialize the repo:

```sh
harness-lint init
```

Commit `harness.toml` and `rules/`. Do not commit `.harness/`; it is generated cache/output.

## Core Loop

When user feedback or review comments describe a recurring issue, capture the preference as a local rule.

1. Read existing project guidance such as `AGENTS.md`, `CLAUDE.md`, `.cursor/rules`, README files, and review docs.
2. Detect project languages and frameworks from files such as `pyproject.toml`, `package.json`, `go.mod`, `Cargo.toml`, and source extensions.
3. Run `harness-lint rule list` to inspect existing lint rules and decide whether to update one.
4. If a new rule is needed, create the local rule skeleton and rule filename with the CLI:

```sh
harness-lint rule create "<constraint>"
```

5. Edit the created file under the configured local rule directory, usually `rules/`.
6. Run `harness-lint doctor`.
7. Run the lint:

```sh
harness-lint check --changed
```

Do not delete, disable, or weaken rules just to make a task pass unless the user explicitly asks.

## Rule Files

Local rules are Markdown files with YAML frontmatter, prose, optional GritQL, and Bad/Good examples.

````markdown
---
id: local.no-print-debug
title: Avoid Print Debugging
language: python
level: warn
status: draft
skill: tdd
tags: [local, python, debug]
---

# Avoid Print Debugging

Use structured logging instead of committed `print` calls.

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

Authoring rules:

- Keep one rule focused on one durable preference.
- Use `status: draft` while the rule is uncertain or lacks a reliable GritQL pattern.
- Use `status: warn` for active advisory checks, and `status: enforced` only after the title, explanation, Bad/Good examples, and GritQL are clear enough to fail builds.
- Add `skill: <skill-name>` only when a lint hit should trigger a specific Codex skill.
- If GritQL cannot express the constraint yet, keep a draft with a TODO instead of inventing another execution path.

## Debugging Lint Failures

When `harness-lint` reports a bug, first identify the rule id from the diagnostic output. Then inspect the specific rule:

```sh
harness-lint rule explain <rule-id>
```

Use the `source:` path from `rule explain` to open the rule Markdown and read the rationale, GritQL, and Bad/Good examples.

For targeted validation:

```sh
harness-lint check --changed --rule <rule-id>
harness-lint check --all --rule <rule-id>
harness-lint --json check --changed --rule <rule-id>
```

Fixing flow:

1. If the rule correctly describes the project convention, fix the code.
2. If the rule is ambiguous, improve its title, prose, Bad/Good examples, or GritQL while keeping it as `draft`.
3. If the rule is a false positive, adjust the GritQL pattern and rerun the targeted check.
4. If the rule should not apply to a path or case, prefer a narrow rule/pattern fix. Use `harness.toml` overrides, disabled rules, or ignore paths only when the project intentionally wants that policy.
5. Rerun `harness-lint check --changed` before finishing.

## Useful Commands

```sh
harness-lint doctor
harness-lint check --changed
harness-lint check --staged
harness-lint check --all
harness-lint check --changed --rule <rule-id>
harness-lint rule list
harness-lint rule explain <rule-id>
harness-lint rule create "<constraint>"
```
