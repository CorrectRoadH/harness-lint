# Project Notes

## Release

Releases are driven by Git tags. Do not manually publish binaries, manually edit the Homebrew formula, or run a separate release script for normal releases.

Normal flow:

1. Commit the release-ready changes on `main`.
2. Create an annotated tag such as `v0.3.0`.
3. Push `main` if needed, then push the tag:

```sh
git push origin main v0.3.0
```

The `.github/workflows/release.yml` workflow syncs the Cargo version from the tag during CI, builds release binaries, creates the GitHub release, and updates `CorrectRoadH/homebrew-tap`. A `Cargo.toml` / `Cargo.lock` version bump in the source tree is useful for readability, but the release artifact version is derived from the tag.

Before tagging, run the local checks that match the change risk, normally:

```sh
cargo fmt --check
cargo test
cargo build --release
cargo run -- doctor
cargo run -- check --changed
```

## Rule Semantics

Every harness-lint rule must contain exactly one executable GritQL block. Missing, empty, TODO-only, comment-only, or multiple `grit` blocks are errors.

Do not create a harness-lint rule for feedback that cannot be expressed as a reliable GritQL pattern. Keep those constraints in agent instructions, review notes, or project documentation instead.

`harness-lint check` does not accept positional paths. Use `--changed`, `--staged`, or `--all` to choose the run set, and use `--rule <rule-id>` only to select rules. Do not pass paths to simulate rule scope.

For TypeScript/JavaScript rules, use `language js` inside the GritQL block even when the rule frontmatter says `language: typescript`. If a TypeScript parser variant matters, `language js(typescript)` is valid. Other rule languages should use Grit CLI language names such as `python`, `json`, `java`, `hcl`, `css`, `markdown`, `yaml`, `rust`, `ruby`, `php`, `go`, and `sql`.

Markdown rules are a special case — Grit's markdown support is alpha-tier. Always write `language markdown(block)` explicitly: a bare `language markdown` selects the *inline* grammar, so block-structure patterns fail to compile (`pattern definition not found`) or never match. Match AST nodes such as `atx_heading()`, `paragraph()`, or `fenced_code_block()` with `where` regex conditions; multi-token literal snippets like `` `# TODO` `` do not match. GritQL cannot see YAML frontmatter fields, code inside fenced blocks, or inline structure within a block (e.g. a link inside a heading). Markdown feedback that depends on any of those belongs in agent instructions or docs, not in a rule.

If a rule should only apply to certain files and you own the rule, encode the scope inside the GritQL with `$filename`, for example:

```grit
`console.log($value)` where {
  $filename <: r".*src/.*\.ts",
  !$filename <: r".*\.test\.ts"
}
```

If an external or already-shared rule is valid in general but has a confirmed path-specific exception, use `[[exceptions]]` in `harness.toml` instead of adding a broad `ignore.paths` entry:

```toml
[[exceptions]]
rule = "go-effective-go.no-blank-placeholder-assignment"
paths = ["apps/api/internal/router/*_router.gen.go"]
reason = "Generated router adapters intentionally discard unused generated parameters."
```

Use `ignore.paths` only when **no** rule should ever scan those files, such as build output. Do **not** put generated code that a dedicated rule needs to inspect in `[ignore]` — `[ignore]` is unbreakable and no rule can opt back in.

### Choosing a scope mechanism

Three knobs, three jobs — do not substitute one for another:

- **`$filename` (in GritQL)** — syntactic narrowing *within* a region a rule already scans. Use it when you own the rule and the distinction is by filename pattern (e.g. exclude `*.test.ts`).
- **`runs_on` + `[file_sets.*]`** — region scope, including reaching code most rules skip. This is the only way to make a rule scan a default-closed region such as committed generated code.
- **`[[exceptions]]`** — report-stage only; hides an already-scanned rule's diagnostics on some paths. It never changes what is scanned.

To make a rule scan generated code that ordinary rules should skip, name the region as a default-closed file set and have the rule opt in:

```toml
# harness.toml — project owns the layout (paths) and the concept mapping (provides)
[file_sets.generated]
paths = ["backend/gen/**/*.pb.go"]
default_rules = false          # removed from the default region; ordinary rules skip it
provides = ["generated"]       # portable concept a shared pack rule can target
```

```markdown
---
id: local.proto-no-id-getter
language: go
runs_on: ["generated"]         # only this region; never ordinary source
---
```

`runs_on` lists file-set names and/or concepts those sets `provides`; the literal `default` is the implicit region of everything no default-closed set claims. Omit `runs_on` to scan `default`. For a rule that needs both, write `runs_on: ["default", "generated"]`. The file-set name is project-owned and renamable — pack rules reference the portable concept, so renaming the set is safe as long as its `provides` is unchanged. Packs must not hardcode project paths; a pack rule ships `runs_on: ["<concept>"]` and the installing project supplies the matching `[file_sets.*] provides = ["<concept>"]` (a pack may document the concepts it needs in its own `INSTALL.md`).

When you give a rule a `runs_on` region, drop any now-redundant region check from its GritQL `$filename` (keep `$filename` only for genuine sub-narrowing or `!`-exclusions, which `runs_on` cannot express). If a rule's `runs_on` region and its positive `$filename` scope do not overlap, the rule silently scans nothing; `harness-lint doctor` reports that as `harness.runs-on-filename-disjoint` (`warn`).

harness-lint validates that the non-glob paths referenced by `[[exceptions]]`, `[ignore]`, and `[file_sets.*]` still exist. When a referenced file or directory is renamed or deleted, the stale entry stops matching silently — suppressed diagnostics quietly return, an ignore no longer covers anything, or a file set silently scans nothing. These are reported (anchored at `harness.toml`) as `harness.stale-exception-path`, `harness.stale-ignore-path`, and `harness.stale-file-set-path` (`warn` by default). Structural mistakes are reported at `error`: `harness.empty-file-set` (a file set with no paths), `harness.file-set-ignore-overlap` (a file-set path also covered by `[ignore]`, so it can never be reached), `harness.unknown-run-target` (a rule `runs_on` a file set or concept nothing provides — typically a pack update expecting a concept the project never wired up), and `harness.runs-on-filename-disjoint` (`warn` — a rule's `runs_on` region and its `$filename` scope do not overlap, so it matches nothing). Escalate or relax any id through `[overrides]`. Patterns whose first component is already a glob (such as `**/*_test.go`) are not checked because there is no literal prefix to anchor against.

After creating or editing a rule, validate it by running the single rule over the configured file set:

```sh
harness-lint rule verify <rule-id>
harness-lint check --all --rule <rule-id>
```

Then adjust the GritQL if it reports no expected files, reports unrelated files, or has too broad/narrow `$filename` scope.
