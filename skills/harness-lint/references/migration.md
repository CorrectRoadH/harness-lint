# harness-lint Migration Reference

Use this reference when a repository already has `harness.toml`, `harness.lock`, local rules, or installed packs and the user asks to migrate, upgrade, modernize, or check best practices.

## First Checks

Run from the target repository root:

```sh
harness-lint --version
harness-lint doctor
harness-lint outdated
harness-lint rule list
```

`rule list` always prints Markdown grouped by pack. Do not pass `--json` to `rule list`; JSON output is not supported for that command.

## Config Key Migrations

- Remove stale `[obsidian]` blocks. Current harness-lint no longer owns Obsidian checks, so those keys are dead config.
- Rename legacy `[[suppressions]]` entries to `[[exceptions]]`. The loader still accepts `[[suppressions]]` for compatibility, but it warns so existing projects know they should upgrade the config.
- Keep path-specific rule allowances in `[[exceptions]]`, not broad `ignore.paths`, when only one rule should be hidden for specific files.
- Use `ignore.paths` only when no rule should scan those files at all, such as generated output, build artifacts, vendored fixtures, or external SDK snapshots.

Example:

```toml
[[exceptions]]
rule = "go-effective-go.no-blank-placeholder-assignment"
paths = ["apps/backend/internal/bootstrap/public_track_*_router.go"]
reason = "Generated router adapters intentionally discard unused generated parameters."
```

## Local Rules And Pack Grouping

- Default local rules live under lowercase `rules/`. Rename older `Rules/` directories and update project instructions that still point to `Rules/`.
- `[rules].local` may list one or more local rule roots. `harness-lint rule create` writes to the first configured local rule root.
- A Markdown rule directly inside a configured local rule root belongs to the default local group.
- A folder inside a configured local rule root is treated as a local pack. The folder name is the pack name in `harness-lint rule list`.
- Keep local rule filenames stable and readable. Use the file/folder name to communicate the package or rule ownership rather than hiding that in prose.

## Rule List Output

Use `harness-lint rule list` for human review. The output is a Markdown table grouped by pack and includes:

- pack name as its own Markdown heading
- level
- id
- description

This makes the command suitable for README snippets, review notes, and agent summaries without a separate formatting step.

## Pack Updates

Use installed pack maintenance commands instead of editing `harness.lock` by hand:

```sh
harness-lint outdated
harness-lint update
harness-lint doctor
harness-lint check --all
```

If new diagnostics appear after a pack update, calibrate them explicitly:

- Fix the reported code when the rule matches project policy.
- Add or update a local rule if the project needs a narrower convention.
- Disable a shared rule only when the project has decided not to adopt it yet.
- Add `[[exceptions]]` only for confirmed rule/path exceptions.

Do not hide broad production directories with `ignore.paths` just to quiet a newly updated pack.

## Generated State

- Commit `harness.toml`, `harness.lock`, and local rule files.
- Do not commit `.harness/`; it is generated cache and restored from config/lock state.

## Migration Verification

After a migration, run:

```sh
harness-lint doctor
harness-lint rule list
harness-lint check --changed
```

For pack or config migrations with wider impact, also run:

```sh
harness-lint check --all
```
