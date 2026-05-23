# Releasing

Release checklist:

1. Update versions in `Cargo.toml`, `npm/package.json`, and `Formula/harness-lint.rb`.
2. Run `cargo fmt --check`.
3. Run `cargo test --all`.
4. Tag the release:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The release workflow builds Linux and macOS binaries and attaches them to the GitHub Release.

Distribution paths:

- `cargo install --path .` for local development.
- GitHub Releases for standalone binaries.
- `npm/` wrapper for JavaScript ecosystems.
- `Formula/` for Homebrew distribution.

Rule packs should use SemVer tags and a stable `harness-pack.toml` manifest.

