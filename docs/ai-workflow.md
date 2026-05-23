# AI Workflow

harness-lint is designed for a specific agent loop: recurring user feedback should become a rule, not just a one-off code edit.

Recommended instruction:

```markdown
When the user expresses a recurring coding preference, create or update a
harness-lint rule instead of only changing the current code.
Run `harness check --changed` before finishing.
```

Example:

```sh
harness-lint rule suggest "Python validation should prefer pydantic models"
```

This creates a draft rule in `harness/rules/local/`. The draft can be edited, tested, and promoted:

```sh
harness-lint rule test local.python-validation-should-prefer-pydantic-models
harness-lint rule set-status local.python-validation-should-prefer-pydantic-models warn
```

Agents should not delete rules, downgrade levels, or disable checks to make a task pass unless the user explicitly asks for that.

## Installation Conversion

When installing harness-lint in a project that already has agent instructions, use an LLM-driven conversion step:

```text
install harness: read CLAUDE.md, AGENTS.md, .cursor/rules, README.md, and relevant docs.
Run `harness-lint init`.
Convert existing durable coding constraints into local harness-lint draft rules under `harness/rules/local/`.
Use `harness-lint rule suggest "<constraint>"` for each preference that can become a rule.
The command should search the harness registry using the detected project languages and libraries.
If a suitable existing rule exists, ask the user whether to install its rule pack.
Only create a local draft when no suitable existing rule exists or the user wants a local project rule.
Do not rely on `harness-lint init` to infer rules automatically.
Do not convert vague advice into enforced rules; keep uncertain rules as draft.
Run `harness-lint rule list` and summarize the generated drafts.
```

This is intentionally not automatic `init` behavior. The conversion needs judgment: one paragraph in `CLAUDE.md` may become several rules, a soft guideline may stay as documentation, and a durable preference should include Bad and Good examples before promotion.

The preferred conversion order is:

1. Search the registry with `harness-lint rule suggest "<constraint>"`.
2. Ask the user before installing a matching external rule pack.
3. Use `harness-lint rule suggest --local "<constraint>"` only when no existing rule is suitable.
