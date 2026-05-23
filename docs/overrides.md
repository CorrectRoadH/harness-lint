# Local Rules And Overrides

Rule precedence:

1. Disabled rules are removed from execution.
2. Local rules override external rules with the same rule id.
3. `overrides` in `harness.toml` override rule severity.
4. Draft rules are never executed.

Example:

```toml
[overrides]
"python.prefer-pydantic" = "error"

[disabled]
rules = ["python.no-print-debug"]
```

Local rules live in:

```text
harness/rules/local/
```

External packs are cached in:

```text
.harness/packs/
```

Only `harness/`, `harness.toml`, and `harness.lock` are intended to be committed.

