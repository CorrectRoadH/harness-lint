# Rule Packs

A rule pack is the distribution unit for reusable harness-lint rules.

```text
harness-rules-python/
├── harness-pack.toml
├── rules/
│   ├── prefer-pydantic.md
│   └── no-bare-except.md
├── tests/
└── README.md
```

`harness-pack.toml`:

```toml
[pack]
id = "python"
name = "Python Best Practices"
version = "1.2.0"
description = "GritQL rules for Python projects."
license = "MIT"

[compat]
harness = ">=0.1.0"
grit = ">=0.1.0"
languages = ["python"]

[rules.prefer-pydantic]
path = "rules/prefer-pydantic.md"
default_level = "warn"
tags = ["python", "validation", "ai-style"]
```

Rules can also be auto-discovered from `rules/*.md` when `[rules]` is omitted.

External packs are configured in `harness.toml`:

```toml
[packs]
python = "github:CorrectRoadH/harness-lint@v1.2.0#packs/python"
go = "CorrectRoadH/harness-lint/packs/go@main"
local_python = "local:../harness-rules-python"
```

Use `harness-lint search <query>` and `harness-lint inspect <id>` before installing. `harness-lint install <id>` installs from the registry catalog; `harness-lint install <id> <spec>` installs an explicit source.

Use `harness-lint list --available` or `harness-lint search` with no query to list the catalog. The built-in catalog is split into small tracks so base rules and advanced rules can be downloaded and upgraded separately:

- `python`, `python-pep8`, `python-typing`, `python-async`
- `go`, `go-effective-go`, `go-concurrency`
- `typescript`, `typescript-react`

Git and local specs may point at a pack subdirectory with `#path/to/pack`; GitHub shorthand may also include the path before the ref, like `CorrectRoadH/harness-lint/packs/python@main`. GitHub tree URLs such as `https://github.com/CorrectRoadH/harness-lint/tree/main/packs/python` are accepted too. If no subdirectory is given, harness-lint also checks `packs/<id>` in the source repository.

Installed pack versions, requested refs, pack paths, and pack-level checksums are recorded in `harness.lock`. Use `harness-lint outdated` to check for upstream changes, `harness-lint update` or `harness-lint restore` to refresh local copies, and `harness-lint remove <id>` to uninstall a pack.
