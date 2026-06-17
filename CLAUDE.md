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
paths = ["apps/backend/internal/bootstrap/public_track_*_router.go"]
reason = "Generated router adapters intentionally discard unused generated parameters."
```

Use `ignore.paths` only when no rules should scan those files at all, such as generated output.

harness-lint validates that the non-glob paths referenced by `[[exceptions]]` and `[ignore]` still exist. When a referenced file or directory is renamed or deleted, the stale entry stops matching silently — suppressed diagnostics quietly return, or an ignore no longer covers anything. These cases are reported (anchored at `harness.toml`) as `harness.stale-exception-path` and `harness.stale-ignore-path`. They are `warn` by default; escalate a given id to `error` through `[overrides]`. Patterns whose first component is already a glob (such as `**/*_test.go`) are not checked because there is no literal prefix to anchor against.

After creating or editing a rule, validate it by running the single rule over the configured file set:

```sh
harness-lint rule verify <rule-id>
harness-lint check --all --rule <rule-id>
```

Then adjust the GritQL if it reports no expected files, reports unrelated files, or has too broad/narrow `$filename` scope.
