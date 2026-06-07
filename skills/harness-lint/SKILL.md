---
name: harness-lint
description: Use when installing, configuring, migrating, authoring, debugging, or fixing `harness-lint` rules in a code repository. Covers command setup, shared/local rule packs, config upgrades, converting recurring feedback into local rules, writing rule Markdown/GritQL, inspecting a specific rule after lint failures, and fixing code without weakening rules.
metadata:
  short-description: Install, migrate, and author harness-lint rules
---

# harness-lint

Use this skill whenever a repository is being connected to `harness-lint`, a harness-lint config needs migration, a user asks for a new project rule, or a `harness-lint` check fails.

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

Commit `harness.toml`, `harness.lock` when present, and `rules/`. Do not commit `.harness/`; it is generated cache/output.

## Config Migration

When upgrading an existing repository or reviewing stale harness-lint config, read [references/migration.md](references/migration.md) before editing `harness.toml`, `harness.lock`, local rule directories, or exception/ignore settings.

## Pack Workflow

When installing, recommending, updating, restoring, or troubleshooting shared rule packs, read [references/packs.md](references/packs.md). For repository setup, detect the languages/frameworks, search and inspect available packs, summarize relevant candidates, and ask before installing recommended packs unless the user already named exact packs to install. Use CLI pack commands instead of editing `harness.lock` by hand.

## Reference Map

- [references/migration.md](references/migration.md): config key migrations, local rule directory/package conventions, pack updates, and migration verification.
- [references/packs.md](references/packs.md): pack discovery, install specs, local pack semantics, update/restore/remove commands, and common pack failure messages.

## Core Loop

When user feedback or review comments describe a recurring issue, capture the preference as a local rule.

1. Read existing project guidance such as `AGENTS.md`, `CLAUDE.md`, `.cursor/rules`, README files, and review docs.
2. Detect project languages and frameworks from files such as `pyproject.toml`, `package.json`, `go.mod`, `Cargo.toml`, and source extensions.
3. Run `harness-lint rule list` to inspect existing lint rules and decide whether to update one. `rule list` is Markdown-only; do not pass `--json` to that command.
4. For a new recurring feedback item, consider `harness-lint rule suggest "<feedback>"` before creating a local rule so an existing shared pack can be reused when appropriate.
5. Before creating a new rule, decide whether the feedback can be expressed as a reliable GritQL pattern. If it cannot, do not create a harness-lint rule; keep the constraint in agent instructions, review notes, or project documentation instead.
6. If a new rule is needed, create the local rule file with the CLI:

```sh
harness-lint rule create "<constraint>" --language <language> --grit <gritql>
```

7. Edit the created file under the configured local rule directory, usually `rules/`.
8. Run `harness-lint doctor`.
9. Verify the Bad examples, then run the new rule by itself and confirm it reports the expected repository file(s). Do not pass paths to `check` to simulate rule scope; if the rule should only apply to certain files, encode that directly in GritQL with `$filename`. Adjust the GritQL if the rule is too broad, too narrow, or produces no diagnostic:

```sh
harness-lint rule verify <rule-id>
harness-lint check --all --rule <rule-id>
```

10. Run the lint:

```sh
harness-lint check --changed
```

Do not delete, disable, or weaken rules just to make a task pass unless the user explicitly asks.

When doing an end-to-end repository setup, ask before finishing whether the user wants `harness-lint check --changed` wired into an existing git hook. If they agree, inspect the current hook setup first and reuse it; do not introduce a hook manager just for harness-lint.

## Rule Files

Local rules are Markdown files with YAML frontmatter, prose, exactly one executable GritQL block, and Bad/Good examples.

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
- Rule ids, filenames, and local pack folder names should be readable, stable, and tied to real project ownership. Do not invent future/product buckets unless the repository already uses that vocabulary. Any language is allowed, but avoid punctuation and decorative symbols. Use `-` instead of spaces. For English ids, prefer lowercase kebab-case such as `local.no-print-debug`. For Chinese ids, prefer short plain phrases such as `local.禁止使用UI` or `local.禁止-使用-UI`.
- Keep `id` and the filename aligned when practical, so `id: local.no-print-debug` lives in `no-print-debug.md`.
- Use `level: warn` for advisory checks.
- Use `level: error` only when the title, explanation, Bad/Good examples, and GritQL are clear enough to fail builds.
- Add `skill: <skill-name>` only when a lint hit should trigger a specific Codex skill.
- Use exactly one executable fenced `grit` block per rule file. `harness-lint doctor` rejects missing, empty, TODO/comment-only, and multiple GritQL blocks.
- If GritQL cannot express the constraint reliably, do not create a harness-lint rule.
- If a rule should only apply to certain files, express that directly in GritQL with `$filename` conditions, such as `$filename <: r".*src/.*\.ts"` and `!$filename <: r".*\.test\.ts"`.

Writing GritQL:

- Start with `language <name>` when the rule targets a specific language. Use Grit CLI language names such as `js`, `python`, `json`, `java`, `hcl`, `css`, `markdown`, `yaml`, `rust`, `ruby`, `php`, `go`, and `sql`. For TypeScript/JavaScript rules, use `language js` in the GritQL block even when rule frontmatter says `language: typescript`; use `language js(typescript)` when the TypeScript parser variant is needed.
- Prefer the smallest syntax shape that proves the rule. A narrow pattern with fewer false positives is better than a broad one that guesses intent.
- Use metavariables such as `$value`, `$name`, or `$body` for parts that may vary.
- Match the forbidden shape directly first. Add exceptions only after a real false positive appears.
- Separate `where` conditions with commas, not semicolons.
- If a rule spans files, project configuration, semantic ownership, or intent that GritQL cannot see, do not create a harness-lint rule until there is a reliable executable pattern.

Example patterns:

````markdown
```grit
language js
`console.log($value)` where {
  $filename <: r".*src/.*\.ts",
  !$filename <: r".*\.test\.ts"
}
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
- Bad/Good examples and executable GritQL are required quality gates for every rule. Use `level: error` only when the team wants the diagnostic to fail checks.

## Debugging Lint Failures

When `harness-lint` reports a bug, first identify the rule id from the diagnostic output. Then inspect the specific rule:

```sh
harness-lint rule explain <rule-id>
```

Use the `source:` path from `rule explain` to open the rule Markdown and read the rationale, GritQL, and Bad/Good examples.

For targeted validation:

```sh
harness-lint rule verify <rule-id>
harness-lint check --all --rule <rule-id>
harness-lint check --changed --rule <rule-id>
harness-lint --json check --changed --rule <rule-id>
```

Fixing flow:

1. If the rule correctly describes the project convention, fix the code.
2. If the rule is ambiguous, improve its title, prose, Bad/Good examples, or GritQL while keeping it at `level: warn`. If no reliable GritQL can express it, remove it from harness-lint rules and keep the guidance in project documentation instead.
3. If the rule is a false positive, adjust the GritQL pattern and rerun the targeted check.
4. If the rule should not apply to a path or case, prefer a narrow rule/pattern fix. For rules you own, encode file scope directly in GritQL with `$filename`. For external or already-shared rules with a confirmed path exception, add a `[[exceptions]]` entry in `harness.toml` instead of ignoring the whole directory.
5. Rerun `harness-lint check --changed` before finishing.

Rule exception example:

```toml
[[exceptions]]
rule = "go-effective-go.no-blank-placeholder-assignment"
paths = ["apps/backend/internal/bootstrap/public_track_*_router.go"]
reason = "Generated router adapters intentionally discard unused generated parameters."
```

Use `ignore.paths` only when no rules should scan those files at all, such as generated output. Rule exceptions hide only results whose rule and path both match; other rules still report in the same files.

## Useful Commands

```sh
harness-lint doctor
harness-lint check --changed
harness-lint check --staged
harness-lint check --all
harness-lint check --changed --rule <rule-id>
harness-lint list --available
harness-lint search <query>
harness-lint inspect <pack-id>
harness-lint install <pack-id>
harness-lint outdated
harness-lint update
harness-lint restore
harness-lint remove <pack-id>
harness-lint rule verify <rule-id>
harness-lint rule list
harness-lint rule suggest "<feedback>"
harness-lint rule explain <rule-id>
harness-lint rule create "<constraint>" --language <language> --grit <gritql>
```
