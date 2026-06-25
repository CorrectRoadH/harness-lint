#!/bin/sh
# UserPromptSubmit hook: run harness-lint on changed files and, when there are
# diagnostics, inject them into the agent context so the agent fixes them per LDD.
# Stays silent when the working tree is clean to avoid context noise.
# Stdout (exit 0) is surfaced to the model as additional developer context.

command -v harness-lint >/dev/null 2>&1 || exit 0

out=$(harness-lint check --changed 2>&1)
status=$?
if [ "$status" -ne 0 ]; then
  cat <<EOF
harness-lint could not check changed files. Treat this as an unresolved lint state,
not a clean result. Run \`harness-lint check --changed\` yourself and fix the
configuration, git base, GritQL environment, or reported diagnostics before
finishing.

$out
EOF
  exit 0
fi

case "$out" in
  "" | "No diagnostics."*) exit 0 ;;
esac

cat <<EOF
harness-lint reports issues on your changed files. Follow Lint Driven Development:
fix the code until \`harness-lint check --changed\` passes. Do not weaken or delete
a rule to silence it; if a report is a false positive, narrow the rule's GritQL or
add an [[exceptions]] entry for the specific path with a reason.

$out
EOF

exit 0
