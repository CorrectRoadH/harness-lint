# harness-lint

harness-lint is a GritQL rule ecosystem and rule-authoring CLI. It focuses on making rules easy to install, update, explain, test, and create from user feedback.

The project is intentionally thin around lint execution. GritQL is the only executable rule language; harness-lint manages rule packs, project configuration, local rules, incremental file selection, reports, and AI-friendly workflows.

## Install

From this repository:

```sh
cargo build
target/debug/harness-lint --help
```

All executable rules are GritQL rules and require the `grit` CLI.

## Initialize A Project

```sh
harness-lint init
```

This creates:

```text
harness.toml
harness/rules/local/
.harness/
```

`harness/` is user-owned and should be committed. `.harness/` is a generated work directory and should stay ignored.

## Install Harness Into An Agent

Use this instruction when asking an LLM coding agent to install harness-lint in an existing project:

```text
install harness: read CLAUDE.md, AGENTS.md, .cursor/rules, README.md, and relevant docs.
Run `harness-lint init`.
Convert existing durable coding constraints into local harness-lint draft rules under `harness/rules/local/`.
Use `harness-lint rule suggest "<constraint>"` for each preference that can become a rule.
The command should first search the harness rule registry using the project language/library context.
If a relevant existing rule is found, ask whether to install that rule pack.
Only create a local draft when no suitable existing rule is available or the user prefers a local rule.
Do not rely on `harness-lint init` to infer rules automatically.
Do not convert vague advice into enforced rules; keep uncertain rules as draft.
Run `harness-lint rule list` and summarize the generated drafts.
```

The conversion from existing agent docs to rules is intentionally an LLM task. The CLI creates the project structure; the agent reads the docs, understands the constraints, splits them into rule-sized units, and creates reviewable drafts.

## Check And Fix

```sh
harness-lint check
harness-lint check --changed
harness-lint check --staged
harness-lint fix --changed
```

Report formats:

```sh
harness-lint --json check
harness-lint --jsonl check
harness-lint --markdown check
harness-lint --github check
harness-lint --sarif check
```

## Rule Packs

Add a local pack:

```sh
harness-lint pack add python local:../harness-rules-python
```

Add a GitHub pack:

```sh
harness-lint pack add python github:harness-lint/rules-python@v1.2.0
```

Update installed packs:

```sh
harness-lint pack update
```

List or remove packs:

```sh
harness-lint pack list
harness-lint pack remove python
```

## Local Rules

Create a rule from explicit fields:

```sh
harness-lint rule new no-todo "No TODO" --language markdown
```

Create a rule from feedback:

```sh
harness-lint rule suggest "Python validation should prefer pydantic models"
```

`rule suggest` first infers project languages and libraries, searches the configured harness rule registry, and prints candidate packs to install. Use `--local` to skip registry search and create a local draft immediately.

List and explain rules:

```sh
harness-lint rule list
harness-lint rule explain local.no-todo
```

Test, enable, disable, and tune rules:

```sh
harness-lint rule test local.no-todo
harness-lint rule disable local.no-todo
harness-lint rule enable local.no-todo
harness-lint rule set-level local.no-todo error
harness-lint rule set-status local.no-todo warn
```

## Rule File Format

Rules are Markdown files with YAML frontmatter:

````markdown
---
id: local.no-todo
title: No TODO
language: markdown
level: warn
status: warn
tags: [local]
fixable: false
---

# No TODO

Do not leave TODO markers in committed notes.

```grit
language markdown
`TODO`
```

## Bad

```markdown
TODO: finish this later
```

## Good

```markdown
Finish this before committing.
```
````

Rule bodies are always GritQL. `language` is target metadata used for path
selection and as a parser hint inside the GritQL snippet; it does not choose a
different runner.

Rule statuses:

- `draft`: saved and reviewable, not executed.
- `warn`: executed and reported as a warning unless overridden.
- `enforced`: must include Bad and Good examples, and can fail the run when level is `error`.

## AI Agent Protocol

Add this to `CLAUDE.md`, `AGENTS.md`, or another agent instruction file:

```markdown
When the user expresses a recurring coding preference, create or update a
harness-lint rule instead of only changing the current code.
Run `harness-lint check --changed` before finishing.
```

The intended loop is simple:

1. The user gives feedback.
2. The agent runs `harness-lint rule suggest` to search existing rule packs first.
3. The agent fixes the current code.
4. The agent runs `harness-lint check --changed`.

More docs:

- [Rule packs](docs/rule-packs.md)
- [AI workflow](docs/ai-workflow.md)
- [GritQL rules](docs/gritql-rules.md)
- [Overrides](docs/overrides.md)
- [Troubleshooting](docs/troubleshooting.md)
- [Migrations](docs/migrations.md)
