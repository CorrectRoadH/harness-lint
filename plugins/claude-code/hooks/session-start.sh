#!/bin/sh
# SessionStart hook: inject harness-lint / Lint Driven Development guidance into
# the agent context, plus any diagnostics already present on changed files.
# Stdout (exit 0) is surfaced to the model as additional context.

if ! command -v harness-lint >/dev/null 2>&1; then
  printf '%s\n' "This repo uses harness-lint (Lint Driven Development), but the \`harness-lint\` binary is not on PATH. Install it (e.g. \`brew install CorrectRoadH/tap/harness-lint\`) before relying on lint."
  exit 0
fi

cat <<'EOF'
This repository uses harness-lint with Lint Driven Development (LDD).
When code review or user feedback identifies a class of mistake, do not only fix
the one instance: if it can be expressed as a reliable GritQL pattern, capture it
as a harness-lint rule, run lint until it reports the problem, then fix the code
until lint passes. If it cannot be a reliable pattern, keep it in docs, not a rule.

Key commands: `harness-lint check --changed`, `harness-lint rule list`,
`harness-lint rule explain <id>`, `harness-lint rule verify <id>`.
Load the harness-lint skill before authoring or debugging rules.
When lint fails, fix the code or narrow the rule — never weaken or delete a rule
just to make the check pass.
EOF

# Surface diagnostics already present on changed files at session start.
out=$(harness-lint check --changed 2>&1)
status=$?
if [ "$status" -ne 0 ]; then
  printf '\nharness-lint could not check changed files; this is not a clean result:\n%s\n' "$out"
else
  case "$out" in
    "" | "No diagnostics."*) : ;;
    *) printf '\nCurrent harness-lint diagnostics on changed files:\n%s\n' "$out" ;;
  esac
fi

exit 0
