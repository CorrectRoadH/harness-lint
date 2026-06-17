# Migrating harness-lint configuration

When `harness-lint` detects a deprecated or never-implemented construct in
`harness.toml` or a rule file, it prints a warning that links here. Each section
below is a self-contained migration an AI agent (or a person) can apply
mechanically. After applying one, run `harness-lint doctor` to confirm the
warning is gone.

This file lives at a stable URL so tools can point to it:
`https://github.com/CorrectRoadH/harness-lint/blob/main/MIGRATE.md`

---

## `[[suppressions]]` → `[[exceptions]]`

**Warning:** `` `[[suppressions]]` is deprecated; rename it to `[[exceptions]]` ``

`[[suppressions]]` was renamed to `[[exceptions]]`. harness-lint still reads the
old name but warns. The shape is identical — only the table name changed.

```toml
# Before
[[suppressions]]
rule = "go-effective-go.no-blank-placeholder-assignment"
paths = ["apps/api/internal/router/*_router.gen.go"]

# After
[[exceptions]]
rule = "go-effective-go.no-blank-placeholder-assignment"
paths = ["apps/api/internal/router/*_router.gen.go"]
```

Rename every `[[suppressions]]` block to `[[exceptions]]`. No other change.

---

## `[[scan_ignored]]` → `[file_sets]` + rule `runs_on`

**Warning:** `` `[[scan_ignored]]` is not a supported key and is silently ignored ``

`[[scan_ignored]]` was an early proposal that was never implemented. If your
`harness.toml` contains it (it may have been copied from an old README), it does
nothing — the rule it names is not actually scanning the listed paths. The
supported mechanism is a **default-closed file set** that the rule opts into
with `runs_on`.

```toml
# Before — silently dead; the rule does NOT scan backend/gen
[[scan_ignored]]
rule = "local.proto-no-id-getter"
paths = ["backend/gen/**"]
reason = "This rule inspects generated protobuf Go."
```

```toml
# After — name the region, mark it default-closed, expose a portable concept
[file_sets.generated]
paths = ["backend/gen/**"]
default_rules = false        # ordinary rules skip it
provides = ["generated"]     # a portable concept the rule (or a pack) can target
```

Then add `runs_on` to the rule's frontmatter so it opts into that region:

```markdown
---
id: local.proto-no-id-getter
language: go
runs_on: ["generated"]       # only the generated region; never ordinary source
---
```

If the rule should scan ordinary source **and** the region, use
`runs_on: ["default", "generated"]`. Delete the `[[scan_ignored]]` block.

---

## Generated code in `[ignore]` that a dedicated rule needs

**Not auto-detected — apply only when a rule must inspect ignored generated code.**

`[ignore]` means "no rule ever scans this," and nothing can opt back in. If you
put generated code in `[ignore]` but also need one rule to inspect it (for
example a proto rule that checks the generated `.pb.go`), that rule can never
reach it. Move the path out of `[ignore]` and into a default-closed file set:

```toml
# Before — the proto rule can never see backend/gen
[ignore]
paths = ["backend/gen/**", "dist/**"]
```

```toml
# After — dist stays ignored; backend/gen becomes a reachable, default-closed region
[ignore]
paths = ["dist/**"]

[file_sets.generated]
paths = ["backend/gen/**"]
default_rules = false
provides = ["generated"]
```

Leave paths that **no** rule needs (build output like `dist/**`) in `[ignore]`.
harness-lint reports `harness.file-set-ignore-overlap` (error) if the same path
is in both `[ignore]` and a file set.

---

## Installing a pack whose rules `runs_on` a concept

**Warning:** `harness.unknown-run-target` — a rule `runs_on` a concept nothing provides.

A shared pack rule ships `runs_on: ["<concept>"]` but never hardcodes your
paths. After installing such a pack (or after `harness-lint update` pulls a new
rule), read the pack's `INSTALL.md` for the concepts it expects, then add a
`[file_sets.*]` that points at the matching code in your repo and `provides`
that concept:

```toml
[file_sets.generated]
paths = ["backend/gen/**/*.pb.go"]
default_rules = false
provides = ["generated"]     # satisfies the pack rule's runs_on: ["generated"]
```

Run `harness-lint doctor`; the `harness.unknown-run-target` error clears once a
file set provides the concept.
