# harness-lint TODO

This is the single required path, in dependency order.

## Product Boundary

- [x] Only execute GritQL rules.
- [x] Provide lint checks, not code fixes.
- [x] Keep the CLI small: init, check, pack install/update/list, rule suggest/new/list/explain.
- [x] Treat `language` as target metadata, not an execution engine.

## Project Setup

- [x] Create `harness.toml`.
- [x] Create `harness/rules/local/`.
- [x] Create `.harness/` for generated Grit files, pack cache, and diagnostics cache.
- [x] Print short agent installation instructions.
- [x] Document that existing `CLAUDE.md`, `AGENTS.md`, `.cursor/rules`, and README constraints are converted by an LLM during installation, not by `init`.

## Rule Ecosystem

- [x] Define `harness-pack.toml`.
- [x] Install local rule packs.
- [x] Install GitHub rule packs.
- [x] Update installed packs.
- [x] List configured packs.
- [x] Record installed packs in `harness.lock`.

## Rules

- [x] Parse Markdown rule files with YAML frontmatter.
- [x] Extract the first `grit` code block as the only executable body.
- [x] Support `draft`, `warn`, and `enforced` statuses.
- [x] Keep draft rules out of `grit check`.
- [x] Generate local draft rules from feedback.
- [x] Search the planned registry before creating local drafts.
- [x] Ask the user to install matching rule packs instead of silently creating duplicates.

## Lint

- [x] Compile active rules into `.harness/generated/grit/`.
- [x] Run `grit check`.
- [x] Support full, changed, staged, and explicit path checks.
- [x] Filter files by rule language and project ignore patterns.
- [x] Output human diagnostics.
- [x] Output JSON diagnostics with `--json`.
- [x] Cache diagnostics internally without exposing cache as a product feature.

## Agent Workflow

- [x] Provide `harness-lint rule suggest "<feedback>"`.
- [x] Provide `harness-lint rule suggest --local "<feedback>"`.
- [x] Keep uncertain feedback as a reviewable GritQL draft with TODO.
- [x] Instruct agents to run `harness-lint check --changed` before finishing.
