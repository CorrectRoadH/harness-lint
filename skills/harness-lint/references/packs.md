# harness-lint Pack Reference

Use this reference when the task involves finding, installing, updating, restoring, removing, or troubleshooting shared `harness-lint` rule packs.

## Discovery And Recommendation

Before writing local rules for a new repository setup, check whether existing packs cover the same ground:

```sh
harness-lint list --available
harness-lint search <language-or-framework>
harness-lint search <project-constraint-keyword>
harness-lint inspect <pack-id>
```

For one specific feedback item, also try:

```sh
harness-lint rule suggest "<feedback>"
```

Detect language/framework signals from manifests and source files, then summarize relevant packs in the user's language. Use a Markdown table with pack id, what it covers, why it matches or does not match the repo, and whether installation is recommended. Ask before installing recommended packs unless the user explicitly named exact pack ids or specs to install.

If no pack fits, say that clearly and continue with local rules only for constraints that can be expressed as reliable GritQL.

## Install Specs

Catalog install:

```sh
harness-lint install <pack-id>
```

Explicit source installs:

```sh
harness-lint install <id> local:<path>
harness-lint install <id> github:<owner>/<repo>@<ref>#<path>
harness-lint install <id> git:<url-or-owner/repo>@<ref>#<path>
```

The `@<ref>` and `#<path>` parts are optional when the pack root is at the repository root and the default branch is acceptable. GitHub URLs and shorthands are accepted; examples include `github:CorrectRoadH/harness-lint@main#packs/python` and `https://github.com/CorrectRoadH/harness-lint/tree/main/packs/python`.

`harness-lint install` currently supports catalog, local, and git/GitHub sources. Do not suggest `cargo:`, `pip:`, or generic URL pack sources as installable sources unless the CLI adds support.

After install:

```sh
harness-lint doctor
harness-lint check --all
```

Commit `harness.toml` and `harness.lock`. Do not commit `.harness/`.

## Local Pack Semantics

There are two local pack modes:

- Local rule folders under `[rules].local`: direct `.md` files belong to the `Local Rules` group; each direct child folder is shown as a local pack, and the folder name is the pack name in `harness-lint rule list`. These folders do not need `harness-pack.toml`.
- Explicit installed local packs such as `harness-lint install demo local:./packs/demo`: the target directory must be a real pack with `harness-pack.toml`.

Use folder names that match current project domains, languages, or ownership. Avoid arbitrary abbreviations or future-only buckets that the repo does not already use.

## Maintenance

Use pack commands instead of hand-editing lock state:

```sh
harness-lint list
harness-lint outdated
harness-lint update
harness-lint restore
harness-lint remove <pack-id>
```

Use `restore` when `.harness/` is missing or a fresh checkout needs to rebuild the generated pack cache from `harness.lock`. Use `update` when intentionally refreshing installed pack contents and rewriting lock checksums.

After updates, run:

```sh
harness-lint doctor
harness-lint rule list
harness-lint check --all
```

If new diagnostics appear, fix code, tune owned local rules, or add narrow `[[exceptions]]` entries for confirmed rule/path exceptions. Do not hide broad source directories with `ignore.paths` just to silence updated packs.

## Common Failure Messages

- `pack '<id>' was not found in the catalog`: the catalog id is wrong or the pack is not published. Use `harness-lint list --available`, `search`, or an explicit source spec.
- `local pack path does not exist: <path>`: the explicit `local:<path>` points at a missing directory.
- `failed to read <path>/harness-pack.toml`: an explicit local pack path exists but is not a manifest-backed pack. If the goal is project-local rules, put Markdown rules under `rules/` or a child folder instead.
- `unsupported pack source for '<id>'`: the parsed source is not currently installable. Use catalog, `local:`, `github:`, or `git:` sources.
- `pack '<id>' is missing from harness.lock; run 'harness-lint update' first`: config and lock are out of sync.
- `local pack '<id>' differs from harness.lock`: a local installed pack changed. Run `harness-lint update` only if the change is intentional.
