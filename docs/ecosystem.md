# Rule Ecosystem

harness-lint's registry flow is deliberately small:

```sh
harness-lint search "python typed services"
harness-lint inspect python
harness-lint install python
harness-lint outdated
harness-lint update
harness-lint remove python
```

## DX Principles

- Discovery and installation are separate. Users search and inspect before the CLI mutates the repo.
- Marketplace ids are shorthand for explicit origins. `install python` resolves through the catalog, then writes the resolved origin to `harness.toml` and `harness.lock`.
- Origins are explicit. Specs use prefixes such as `local:` and `github:` and installs are written to `harness.lock`.
- Subdirectories are first-class. A single catalog repository can expose `packs/python`, `packs/go`, and `packs/typescript`.
- GitHub and git installs track both the requested ref and the resolved tree hash for the pack subdirectory. This lets `outdated` distinguish "this pack changed" from unrelated repository churn.
- Publishing should be dry-run first. The planned backend API should validate manifest metadata, listed files, source URL, license, and GritQL-only rule bodies before accepting a release.
- Installed project rules stay user-owned. Custom feedback rules live flat under `rules/*.md` by default.

## Pack Authoring

Each pack contains:

```text
harness-pack.toml
rules/*.md
README.md
```

Rules are Markdown files with YAML frontmatter and a single executable `grit` block. Draft rules can ship in a pack, but only `warn` and `enforced` rules execute.

## Pack Series

The catalog is intentionally split into small installable tracks:

| Track | Base pack | Advanced packs |
| --- | --- | --- |
| Python | `python` | `python-pep8`, `python-typing`, `python-async` |
| Go | `go` | `go-effective-go`, `go-concurrency` |
| TypeScript | `typescript` | `typescript-react` |

Users can install only the packs they want:

```sh
harness-lint list --available
harness-lint install python
harness-lint install python-pep8
harness-lint install go-effective-go
```

## Registry Catalog

The CLI reads a JSON catalog from `[registry].url` and falls back to the embedded `site/catalog.json` catalog if the network is unavailable. The value may be a hosted URL, a direct `catalog.json` URL, a `file://` URL, or a local filesystem path. The same JSON file powers the static marketplace page.

- query: feedback text, inferred languages, inferred libraries
- result: pack id, rule id, title, score, reason, install spec
- inspect: pack metadata, supported languages, rules, source, version, install command

This keeps the first marketplace implementation file-based and GitHub-hostable while leaving the backend replaceable later.

## Version Tracking

`harness.lock` records, per installed pack:

- `requested_ref`: the branch or tag from the install spec, such as `main` or `v1.2.0`
- `version`: the resolved git commit
- `pack_path`: the pack subdirectory inside the source repository
- `checksum`: the git tree hash for that pack subdirectory, or a deterministic content hash for local packs

`outdated` checks the latest source and reports only packs whose pack-level checksum changed. `update` refreshes installed git packs under `.harness/packs/` and rewrites the lock. `restore` rebuilds local pack state from the committed `harness.toml`/`harness.lock` metadata on a fresh checkout. `remove <id>` removes the config entry, lock entry, and cached copy.
