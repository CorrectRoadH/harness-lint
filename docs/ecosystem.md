# Rule Ecosystem

harness-lint's registry flow is deliberately small:

```sh
harness-lint pack search "python typed services"
harness-lint pack inspect python
harness-lint pack add python github:CorrectRoadH/harness-lint@main#packs/python
harness-lint pack update
```

## DX Principles Learned From ClawHub

- Discovery and installation are separate. Users search and inspect before the CLI mutates the repo.
- Origins are explicit. Specs use prefixes such as `local:` and `github:` and installs are written to `harness.lock`.
- Subdirectories are first-class. A single catalog repository can expose `packs/python`, `packs/go`, and `packs/typescript`.
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

## Planned Registry API

The CLI currently has a local catalog stub with the same shape the hosted backend should return:

- query: feedback text, inferred languages, inferred libraries
- result: pack id, rule id, title, score, reason, install spec
- inspect: pack metadata, supported languages, rules, source, version, install command

This lets agent install flows work now while leaving the network backend replaceable later.
