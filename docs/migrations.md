# Migrations

Project migration strategy:

1. Run `harness-lint init`.
2. Ask an LLM agent to read existing agent docs and run `harness-lint rule suggest` for durable constraints.
3. Install existing rule packs when registry candidates match.
4. Keep uncertain rules as `draft`.
5. Promote rules to `warn` after examples are added.
6. Promote critical rules to `enforced` and optionally set level to `error`.

Rule pack migration strategy:

- Keep rule ids stable.
- Use SemVer tags for pack releases.
- Prefer additive rules in minor versions.
- Document breaking rule behavior in major versions.
- Keep `harness-pack.toml` compatible across releases.

