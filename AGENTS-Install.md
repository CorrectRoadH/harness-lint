<!--HARNESS LINT START-->
When user feedback or a code review points to a recurring code-quality issue, do not fix only the current instance first. Create or update a `harness-lint` rule that captures the issue, run the lint so it reports the problem, and then change the code until the lint passes.

For code-related fixes, use this workflow:

1. Convert the feedback into a rule with `harness-lint rule suggest "<feedback>"`, or update an existing rule under `Rules/`.
2. If the rule should trigger a specific Codex skill, add `skill: <skill-name>` to the rule frontmatter.
3. Run `harness-lint check --changed` and confirm the lint identifies the issue.
4. Fix the code.
5. Run `harness-lint check --changed` again before finishing.

Example rule frontmatter:

```yaml
---
id: local.prefer-pydantic-validation
title: Prefer Pydantic Validation
language: python
level: warn
status: draft
skill: tdd
tags: [local, ai-feedback, python]
---
```
<!--HARNESS LINT END-->