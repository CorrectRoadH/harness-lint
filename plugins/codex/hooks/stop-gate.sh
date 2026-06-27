#!/bin/sh
# Stop hook (Lint Driven Development gate): when the agent tries to finish with
# harness-lint diagnostics on changed files, bounce it back to fix them instead
# of letting it yield a dirty tree.
#
# It blocks via {"decision":"block","reason":...} on stdout (exit 0), which both
# Claude Code and Codex honour as a Stop-hook continuation (the reason is fed
# back to the model). Plain stdout is NOT model context at Stop, so the JSON
# form is required.
#
# Bounded by a per-session attempt counter (keyed by session_id from the hook
# input) so the agent can NEVER be trapped: after CAP forced passes it is
# allowed to stop, and any remaining diagnostics resurface at the next prompt
# via the UserPromptSubmit hook. An infra error from the check itself also never
# blocks.

CAP=3

input=$(cat 2>/dev/null)

command -v harness-lint >/dev/null 2>&1 || exit 0

# Per-session attempt counter keyed by session_id from the hook input JSON.
sid=$(printf '%s' "$input" | sed -n 's/.*"session_id"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p')
[ -n "$sid" ] || sid="default"
sid=$(printf '%s' "$sid" | tr -c 'A-Za-z0-9._-' '_')
state="${TMPDIR:-/tmp}/harness-lint-stop-${sid}"

out=$(harness-lint check --changed 2>&1)
status=$?

# Could not run the check -> never trap the agent on an infra error; allow stop.
if [ "$status" -ne 0 ]; then
  rm -f "$state"
  exit 0
fi

case "$out" in
  "" | "No diagnostics."*)
    rm -f "$state"   # clean -> reset and allow stop
    exit 0
    ;;
esac

# Diagnostics present. Bounded retries so the gate cannot loop forever.
count=0
[ -f "$state" ] && count=$(cat "$state" 2>/dev/null)
case "$count" in '' | *[!0-9]*) count=0 ;; esac

if [ "$count" -ge "$CAP" ]; then
  rm -f "$state"   # give up gating; allow stop, leftovers resurface next prompt
  exit 0
fi

count=$((count + 1))
printf '%s' "$count" > "$state"

reason_body="harness-lint reports issues on your changed files. Per Lint Driven Development, do not finish yet: run \`harness-lint check --changed\` and fix the code until it passes. Do not weaken or delete a rule to silence it; if a report is a false positive, narrow the rule's GritQL or add an [[exceptions]] entry for the specific path with a reason.

$out"

# JSON-escape the reason for the {"reason": "..."} string.
escaped=$(printf '%s' "$reason_body" | awk '
  {
    s = $0
    gsub(/\\/, "\\\\", s)
    gsub(/"/, "\\\"", s)
    gsub(/\t/, "\\t", s)
    gsub(/\r/, "\\r", s)
    if (NR > 1) out = out "\\n"
    out = out s
  }
  END { printf "%s", out }
')

printf '{"decision":"block","reason":"%s"}\n' "$escaped"
exit 0
