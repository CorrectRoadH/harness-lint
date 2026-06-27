#!/bin/sh
# SessionStart hook: inject harness-lint / Lint Driven Development guidance into
# the agent context, plus any diagnostics already present on changed files.
# Stdout (exit 0) is surfaced to the model as additional developer context.

if ! command -v harness-lint >/dev/null 2>&1; then
  printf '%s\n' "This repo uses harness-lint (Lint Driven Development), but the \`harness-lint\` binary is not on PATH. Install it (e.g. \`brew install CorrectRoadH/tap/harness-lint\`) before relying on lint."
  exit 0
fi

cat <<'EOF'
This repo uses harness-lint (Lint Driven Development): when review or feedback
reveals a recurring mistake, capture it as a harness-lint rule when it's a
reliable GritQL pattern (otherwise keep it in docs), then fix code until lint
passes. Never weaken or delete a rule to pass a check. Load the harness-lint
skill to author or debug rules.

Commands: `harness-lint check --changed`, `rule list`, `rule explain <id>`, `rule verify <id>`.
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
