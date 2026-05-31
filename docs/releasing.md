# Releasing

Release checklist:

1. Update the version in `Cargo.toml`.
2. Run `cargo fmt --check`.
3. Run `cargo test --all`.
4. Tag the release:

```sh
git tag v0.1.0
git push origin v0.1.0
```

The release workflow builds Linux and macOS binaries and attaches them to the GitHub Release.

Distribution paths:

- `brew install CorrectRoadH/tap/harness-lint` for normal installs.
- GitHub Releases for standalone binaries.
- `cargo install --path .` or `cargo build` for local development.

## Homebrew Tap

Keep the public tap in a separate `CorrectRoadH/homebrew-tap` repository. Homebrew exposes that as `CorrectRoadH/tap`, so users can run:

```sh
brew tap CorrectRoadH/tap
brew install harness-lint
```

The tap should contain:

```text
Formula/harness-lint.rb
```

The release workflow updates the tap automatically after it uploads release assets:

1. Set `version` to the tag without the leading `v`.
2. Point each `url` at the matching GitHub Release asset in `CorrectRoadH/harness-lint`.
3. Replace each `sha256` with the asset checksum:

```sh
gh release download v0.1.0 \
  --repo CorrectRoadH/harness-lint \
  --pattern 'harness-lint-*'

shasum -a 256 harness-lint-*
```

4. Test the formula locally:

```sh
brew install ./Formula/harness-lint.rb
brew test ./Formula/harness-lint.rb
```

For a prerelease or private tap test, install directly from a local formula:

```sh
brew install ./Formula/harness-lint.rb
```

The one-line install command should stay:

```sh
brew install CorrectRoadH/tap/harness-lint
```

Rule packs should use SemVer tags and a stable `harness-pack.toml` manifest.
