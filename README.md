# harness-lint

Tiny CLI for a GritQL rule ecosystem.

[中文 README](README.zh.md)

harness-lint does three things:

- initialize a project for GritQL rules
- install and update rule packs
- run lint checks and help agents turn feedback into draft rules

It does not fix code. GritQL is the only executable rule language.

## Install

```sh
brew install CorrectRoadH/tap/harness-lint
```

Or install from a custom tap checkout:

```sh
brew tap CorrectRoadH/tap
brew install harness-lint
```

`grit` must be installed separately for `check`:

```sh
brew install getgrit/tap/grit
```

## Project Setup

For first-time setup by an AI coding agent, copy [INIT.md](INIT.md) into the agent. It walks the agent through installing `harness-lint`, initializing the repository, updating `AGENTS.md` or `CLAUDE.md`, and drafting initial rules from the repository's existing instructions.

```sh
harness-lint init
```

This creates:

```text
harness.toml
rules/
.harness/
```

Commit `harness.toml` and `rules/`. Ignore `.harness/`.

## Agent Install Prompt

For a full first-time setup flow, give [INIT.md](INIT.md) to an LLM coding agent. For a shorter install prompt, use:

```text
install harness: read CLAUDE.md, AGENTS.md, .cursor/rules, README.md, and relevant docs.
Run `harness-lint init`.
For each durable coding constraint or recurring code review comment, run `harness-lint rule suggest "<constraint>"`.
If registry candidates exist, ask before installing the rule pack.
If no good pack exists, create a local draft rule.
Rules must be GritQL; uncertain rules stay draft.
For code-related fixes, write or update the lint first, run it to identify the problem, fix the code, then run `harness-lint check --changed` again.
If a rule should trigger a specific Codex skill, add `skill: <skill-name>` to the rule frontmatter.
Run `harness-lint rule list` and summarize the result.
```

## CLI

```sh
harness-lint check --changed
harness-lint check --staged
harness-lint check [paths...]
harness-lint check --all
harness-lint doctor
harness-lint pack search "python typing"
harness-lint pack inspect python
harness-lint pack add <id> <source>
harness-lint pack update
harness-lint pack list
harness-lint rule suggest "<feedback>"
harness-lint rule suggest --local "<feedback>"
harness-lint rule new <id> <title> --language <language>
harness-lint rule list
harness-lint rule explain <rule-id>
```

Use `--json` with `check` or `rule list` when another tool needs structured output.

`doctor` checks the project root, configuration, local rule directories, rule packs, Git availability, and the `grit` binary used by `check`.

## Rule Packs

The intended ecosystem flow is:

```sh
harness-lint pack search "pydantic typed service rules"
harness-lint pack inspect python
# Run the install command printed by search or inspect.
harness-lint check --changed
```

Search uses local project signals such as `pyproject.toml`, `go.mod`, `package.json`, `tsconfig.json`, and common library names, then prints install commands. `inspect` shows the pack before it mutates the project. Installed pack origins are recorded in `harness.lock`.

Custom project rules live directly in `rules/*.md` by default. To place them somewhere else, set `[rules].local` in `harness.toml`; authoring commands write new local drafts to the first configured directory.

## Obsidian Vault Checks

GritQL stays the only executable rule language. For vault-wide checks that need a repository index, opt in with `harness.toml`:

```toml
[obsidian]
markdown_links = true
orphan_files = true
flat_attachment_dir = "Attachments"
note_roots = ["Notes"]
```

This adds lint diagnostics for missing Markdown/Wikilink targets, orphan note or attachment files, and nested attachments under the configured attachment directory.

## Rule File

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
