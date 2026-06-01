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
- Rule ids and filenames should be readable and stable. Any language is allowed, but avoid punctuation and decorative symbols. Use `-` instead of spaces. For English ids, prefer lowercase kebab-case such as `local.no-print-debug`. For Chinese ids, prefer short plain phrases such as `local.禁止使用UI` or `local.禁止-使用-UI`.
- Keep `id` and the filename aligned when practical, so `id: local.no-print-debug` lives in `no-print-debug.md`.
- Use `level: warn` for advisory checks.
- Use `level: error` only when the title, explanation, Bad/Good examples, and GritQL are clear enough to fail builds.
- Add `skill: <skill-name>` only when a lint hit should trigger a specific Codex skill.
- Use at most one fenced `grit` block per rule file. `harness-lint doctor` rejects multiple GritQL blocks because only one executable pattern belongs to one rule.
- If GritQL cannot express the constraint yet, leave the GritQL block out and keep a TODO in prose instead of inventing another execution path.

Writing GritQL:

- Start with `language <name>` when the rule targets a specific language, for example `language typescript` or `language python`.
- Prefer the smallest syntax shape that proves the rule. A narrow pattern with fewer false positives is better than a broad one that guesses intent.
- Use metavariables such as `$value`, `$name`, or `$body` for parts that may vary.
- Match the forbidden shape directly first. Add exceptions only after a real false positive appears.
- If a rule spans files, project configuration, semantic ownership, or intent that GritQL cannot see, keep it prose-only at `level: warn` until there is a reliable executable pattern.

Example patterns:

````markdown
```grit
language typescript
`console.log($value)`
```

```grit
language python
`print($value)`
```

```grit
language go
`context.TODO()`
```
````

Bad/Good examples:

- Bad examples must be minimal code that should trigger the rule.
- Good examples must show the preferred local style, not only “delete the bad code”.
- Keep examples in the same language as `language`.
- Include exactly the edge case the rule is about; avoid large unrelated scaffolding.
- When a rule is `level: error`, Bad/Good examples and executable GritQL are required quality gates.

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
2. If the rule is ambiguous, improve its title, prose, Bad/Good examples, or GritQL while keeping it at `level: warn`.
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
