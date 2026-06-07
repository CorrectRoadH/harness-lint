# Documentation Instructions

## About this project

- This is the Mintlify documentation site for `harness-lint`.
- The docs live in the `docs/` subdirectory of the main
  `CorrectRoadH/harness-lint` repository.
- Pages are MDX files with YAML frontmatter.
- Configuration lives in `docs/docs.json`.
- Mintlify GitOps should use `docs` as the project root/subdirectory.

## Terminology

- Use `harness-lint` for the CLI name.
- Use `rule` for a Markdown file containing one executable GritQL block.
- Use `rule pack` for a versioned collection of rules.
- Use `Lint Driven Development` on first mention, then `LDD`.

## Style preferences

- Use active voice and second person ("you")
- Keep sentences concise: one idea per sentence
- Use sentence case for headings
- Bold for UI elements: Click **Settings**
- Code formatting for file names, commands, paths, and code references
- Do not document rule behavior that contradicts the root `AGENTS.md` rule
  semantics.

## Content boundaries

- Document public CLI behavior, configuration, rule authoring, and rule packs.
- Do not document unreleased internals unless the page clearly marks them as
  development notes.
