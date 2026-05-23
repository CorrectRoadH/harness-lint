# harness-lint Architecture

harness-lint is a thin GritQL ecosystem CLI. It owns project configuration, rule pack installation, local draft rules, file selection, and reporting. It does not own code repair or another lint engine.

## Boundaries

- Executable rules are always GritQL.
- `language` is target metadata for path filtering and Grit parser hints.
- The CLI does not expose fix, cache-management, rule mutation, or CI-specific output commands.
- User feedback becomes a draft rule or an installed pack recommendation.

## Data Flow

```text
harness.toml
  -> load local rules and installed packs
  -> parse Markdown rule files
  -> compile active GritQL rules into .harness/generated/.grit/
  -> run grit check
  -> print human or JSON diagnostics
```

## Agent Flow

```text
feedback
  -> infer project language/library context
  -> search registry
  -> suggest pack install when a match exists
  -> otherwise write harness/rules/local/*.md draft
```

Installation conversion is intentionally LLM-driven. The install prompt asks the agent to read `CLAUDE.md`, `AGENTS.md`, `.cursor/rules`, README, and relevant docs, then turn durable constraints into reviewable GritQL draft rules.

## Project Layout

```text
harness.toml
harness.lock
harness/rules/local/*.md
.harness/packs/
.harness/generated/.grit/
.harness/cache/
```

`harness/` and `harness.toml` are user-owned and committed. `.harness/` is generated.

## CLI Surface

```text
harness-lint init
harness-lint check [--changed|--staged] [paths...]
harness-lint pack add <id> <spec>
harness-lint pack update
harness-lint pack list
harness-lint rule suggest [--local] <feedback>
harness-lint rule new <id> <title> [--language <language>]
harness-lint rule list
harness-lint rule explain <rule-id>
```

## Rule File

````markdown
---
id: python.no-print
title: Avoid print debugging
language: python
level: warn
status: draft
tags: [python]
---

# Avoid print debugging

Use logging instead of committed print calls.

```grit
language python
`print($value)`
```
````
