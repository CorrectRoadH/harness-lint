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

## 0.6.0 — Robustness release & markdown authoring guidance

**What it is.** A correctness-focused release; no config migration needed.

- **`harness-lint cache clear`** — new command that wipes `.harness/cache`. `check`
  also garbage-collects file-cache entries older than 30 days automatically, and a
  corrupt cache entry now self-heals as a cache miss instead of failing every run.
- **Crash-safe locks.** `.harness` locks are OS advisory file locks now: a killed
  or Ctrl-C'd run can no longer leave a stale lock that blocks later checks.
- **Typos fail loudly.** `check --rule <unknown-id>` / `--tag <unknown-tag>` is an
  error instead of a silent "No diagnostics" pass, and `doctor` warns
  (`rule-languages`) when a rule's `language:` value is not a name harness-lint
  recognizes (an unknown language scans every file).
- **Markdown rule guidance.** Grit's markdown support is alpha; the docs and the
  agent skill now say exactly what works: write `language markdown(block)`
  explicitly, match AST nodes (`atx_heading()`, `fenced_code_block()`), and do not
  build rules that need frontmatter fields, fence contents, or inline-in-block
  structure. See the GritQL patterns reference, section "Markdown-Specific Notes".
- **Correctness fixes.** CRLF rule files parse; non-ASCII and whitespace filenames
  survive `check --changed`; files deleted from the worktree no longer abort
  `--changed`/`--staged`; `.mdx` files are scanned by `language: markdown` rules;
  grit output that fails to parse warns instead of silently reporting clean;
  `rule create` refuses to overwrite an existing rule file; bare `vendor/pack`
  specs that exist locally are no longer mistaken for GitHub shorthands.

**Adopt it when:** always — update and rerun `harness-lint doctor` once. The only
behavior you might notice is the new error on misspelled `--rule`/`--tag` values,
which previously passed silently.

---

## 0.5.0 — Agent plugins (Claude Code & Codex)

**What it is.** Plugins in [`plugins/`](plugins/) that deliver harness-lint
guidance through agent lifecycle hooks instead of a static `AGENTS.md` block:

- **`SessionStart`** injects the Lint Driven Development working guidance plus any
  diagnostics already present on changed files.
- **`UserPromptSubmit`** runs `harness-lint check --changed` and injects the
  current violations before the agent writes more code (silent when clean).
- A manual **`/harness-lint-capture`** command reviews a session's feedback and
  turns the reusable, GritQL-expressible corrections into rules.

Both Claude Code and Codex use the same `hooks.json` schema and both install from
a plugin marketplace; they differ only in the marketplace command. The hooks
degrade gracefully when the `harness-lint` binary is absent.

**Adopt it when:**

- **Agents keep ignoring the harness-lint guidance you wrote into `AGENTS.md`.**
  A hook re-injects it every session and surfaces live violations at the moment
  the agent is about to act, which a one-time static block cannot.
- **You want violations surfaced before code is written,** not only when the
  human remembers to run `check`.

**Do *not* adopt it for:**

- **A repo where `AGENTS.md` guidance is already being followed.** The hooks add
  a small amount of context to every prompt; if the static block works for you,
  the plugin is redundant.
- **Automatically inventing rules.** `/harness-lint-capture` is deliberately
  manual — most turns produce nothing rule-worthy, so it is not wired to `Stop`.

**Install.** Claude Code: `/plugin marketplace add CorrectRoadH/harness-lint`
then `/plugin install harness-lint@harness-lint`. Codex: `codex plugin marketplace
add CorrectRoadH/harness-lint` then `codex plugin add harness-lint@harness-lint`. See
[`plugins/README.md`](plugins/README.md).

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
