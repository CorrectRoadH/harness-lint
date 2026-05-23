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
python = "github:harness-lint/rules-python@v1.2.0"
local_python = "local:../harness-rules-python"
```

Installed pack versions are recorded in `harness.lock`.

