# What's new in harness-lint

Per-version feature highlights, each with **when to adopt** and **when not to** —
so an AI agent (or a person) can decide whether a feature actually fits a repo
instead of adopting it reflexively. `harness-lint whatsnew` prints a short
version of this; this file is the full guide.

This file lives at a stable URL so tools can point to it:
`https://github.com/CorrectRoadH/harness-lint/blob/main/WHATS-NEW.md`

> This is **not** a deprecation notice. Nothing here is required. For config that
> is broken or deprecated (and *does* need changing), see
> [MIGRATE.md](MIGRATE.md).

---

## 0.4.0 — File sets and `runs_on`

**What it is.** A rule can scope itself to a named region of the repo with
`runs_on`, and a project can expose regions with `[file_sets.*]`:

```toml
# harness.toml — the project owns the layout
[file_sets.generated]
paths = ["backend/gen/**/*.pb.go"]
default_rules = false        # ordinary rules skip it
provides = ["generated"]     # a portable concept a shared-pack rule can target
```

```markdown
---
id: local.proto-no-id-getter
language: go
runs_on: ["generated"]       # only this region
---
```

A rule with no `runs_on` scans the `default` region (everything visible that no
`default_rules = false` set claims). `runs_on` lists file-set names, concepts a
set `provides`, and/or the literal `default`.

**Adopt it when:**

- **Two or more rules share the same directory region.** Define the region once
  as a `[file_sets.*]` and have each rule `runs_on` it, instead of repeating the
  same `$filename` regex in every rule. (One definition to update when the
  directory moves.)
- **A rule must scan committed generated code** that ordinary rules should skip.
  Make the region `default_rules = false` and have the one rule opt in. This is
  the only mechanism that reaches default-closed code — `[ignore]` cannot, and
  `[[exceptions]]` only hides results, it does not widen scanning.
- **You ship or install a rule pack whose rules target generated code.** The
  pack writes `runs_on: ["generated"]` (portable); the project supplies
  `[file_sets.*] provides = ["generated"]` pointing at its real paths. Neither
  side hardcodes the other's knowledge.

**Do *not* adopt it for:**

- **A single file or a one-off path.** `$filename <: r"…/service\.go$"` in the
  GritQL is lighter and self-contained — a whole file set is overkill.
- **"A region *minus* a few files."** `runs_on` is include-only (a union of
  regions); it cannot express exclusions. Keep `!$filename <: r"…"` in the
  GritQL for the carve-outs. (You may still use `runs_on` for the include side
  and `$filename` only for the exclusions.)
- **Suppressing a shared-pack rule on some paths.** That is the reporting stage —
  use `[[exceptions]]`. `runs_on` is for rules you author.

**Don't double-scope.** If you move a region into `runs_on`, drop the now-redundant
`$filename` region check from the GritQL. `harness-lint doctor` reports
`harness.runs-on-filename-disjoint` when a rule's `runs_on` region and its
`$filename` scope don't overlap (the rule would silently match nothing).

**Health checks added with this feature:** `harness.stale-file-set-path` (warn),
`harness.empty-file-set` / `harness.file-set-ignore-overlap` /
`harness.unknown-run-target` (error), and `harness.runs-on-filename-disjoint`
(warn). All adjustable per id through `[overrides]`.

See the [configuration reference](docs/reference/harness-toml.mdx) and
[rule format](docs/reference/rule-format.mdx) for the full field docs.
