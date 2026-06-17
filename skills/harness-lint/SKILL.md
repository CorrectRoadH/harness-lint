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

If a `harness-lint` command prints a deprecation or legacy-construct warning that includes a migration URL (for example `[[suppressions]]` or `[[scan_ignored]]`, both of which link to `MIGRATE.md`), fetch that URL, find the matching section, and apply the migration to `harness.toml` and any affected rule files. Then re-run `harness-lint doctor` to confirm the warning is gone. Do this proactively on the first run against an existing repo — these warnings mean config that is silently doing nothing.

## Surfacing New Features (only when they fit)

`harness-lint whatsnew` and [WHATS-NEW.md](../../WHATS-NEW.md) list features added in recent versions, each with explicit **adopt when / do not adopt** criteria. When setting up a repo, reviewing its config, or after a `harness-lint` upgrade, consult them and mention a feature **only when the repo actually matches its "adopt when" criteria** — never as a blanket reminder. A passive "you could use X" nag is noise; the value is your judgment of fit.

Concretely, for the file sets / `runs_on` feature, recommend it only when you can point to a real trigger:

- **Two or more local rules repeat the same directory region** in their `$filename` (for example several rules all scoped to `frontend/e2e/.*`). Propose one `[file_sets.*]` plus `runs_on` and drop the duplicated `$filename`. Removing a region-`$filename` that was paired with an unscoped fallback branch (a common `rule verify` workaround) also fixes the latent over-match where the fallback fired repo-wide.
- **A local rule must inspect committed generated code** that other rules should skip → a `default_rules = false` file set the rule opts into.
- **An installed pack rule needs a concept the project has not wired** (a `harness.unknown-run-target` error) → add `[file_sets.*]` with the matching `provides`.

Do **not** suggest it for single-file scopes or "region minus a few files" (include-only `runs_on` cannot express exclusions) — those belong in `$filename`. This distinction is the whole point: surface the feature where it pays off, leave the correct `$filename` usages alone.

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
- For *syntactic* file narrowing within a region the rule already scans, express it directly in GritQL with `$filename` conditions, such as `$filename <: r".*src/.*\.ts"` and `!$filename <: r".*\.test\.ts"`.
- To scope a rule to a *region* — especially to reach code most rules skip, such as committed generated code — use `runs_on` plus a `[file_sets.*]` entry, not `$filename`. See "Rule Scope and File Sets" below.

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

## Rule Scope and File Sets

Three scope knobs do three different jobs — never substitute one for another:

- **`$filename` (GritQL)** — syntactic narrowing *within* a region the rule already scans (by filename pattern).
- **`runs_on` + `[file_sets.*]`** — region scope, and the *only* way to make a rule scan a default-closed region such as committed generated code.
- **`[[exceptions]]`** — report-stage only; hides an already-scanned rule's diagnostics on some paths. Never changes what is scanned.

`[ignore]` means "no rule ever scans this" and is unbreakable. Do **not** put generated code a dedicated rule must inspect into `[ignore]`; name it as a default-closed file set instead.

A rule with no `runs_on` scans the `default` region: every visible file not claimed by a `default_rules = false` file set. To make a rule reach a region ordinary rules skip:

```toml
# harness.toml — project owns the paths (layout) and the provides mapping
[file_sets.generated]
paths = ["backend/gen/**/*.pb.go"]
default_rules = false       # removed from default; ordinary rules skip it
provides = ["generated"]    # portable concept a shared pack rule can target
```

```markdown
---
id: local.proto-no-id-getter
language: go
runs_on: ["generated"]      # only this region; never ordinary source
---
```

- `runs_on` lists file-set names and/or concepts a set `provides`; the literal `default` is the implicit region. For both, write `runs_on: ["default", "generated"]`.
- The file-set name is project-owned and renamable. Pack rules reference the portable concept, so renaming the set is safe as long as its `provides` is unchanged.
- An empty `runs_on: []`, a `[file_sets.*]` with no `paths`, a file-set path that overlaps `[ignore]`, and a `runs_on` target nothing provides are all reported by `doctor` (`runs_on`/file-set structural mistakes at `error`).

### Installing a pack that needs a concept

A shared pack rule ships `runs_on: ["<concept>"]` but must never hardcode your paths. When installing such a pack:

1. Read the pack's `INSTALL.md` (or manifest notes) for the concepts its rules expect (e.g. `generated`).
2. Add a `[file_sets.*]` to the project `harness.toml` whose `paths` point at the matching code in this repo and whose `provides` lists that concept.
3. Run `harness-lint doctor`; a `harness.unknown-run-target` error means a rule expects a concept no file set provides yet — add or fix the `provides`.

A `harness-lint update` that pulls new pack rules can surface new `unknown-run-target` errors; resolve them by wiring the newly required concept, not by editing the installed pack.

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
4. If the rule should not apply to a path or case, prefer a narrow rule/pattern fix. For rules you own, encode syntactic file scope directly in GritQL with `$filename`. For external or already-shared rules with a confirmed path exception, add a `[[exceptions]]` entry in `harness.toml` instead of ignoring the whole directory. If the rule is firing in a whole region it should not cover (or should only cover a special region), adjust its `runs_on` / the project `[file_sets.*]` rather than weakening the pattern.
5. Rerun `harness-lint check --changed` before finishing.

Rule exception example:

```toml
[[exceptions]]
rule = "go-effective-go.no-blank-placeholder-assignment"
paths = ["apps/backend/internal/bootstrap/public_track_*_router.go"]
reason = "Generated router adapters intentionally discard unused generated parameters."
```

Use `ignore.paths` only when **no** rule should ever scan those files, such as build output. For generated code that a dedicated rule still needs to inspect, use a default-closed `[file_sets.*]` plus `runs_on`, not `[ignore]`. Rule exceptions hide only results whose rule and path both match; other rules still report in the same files.

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
