# GritQL Rules

Every executable harness-lint rule is a GritQL rule.

Rule skeleton:

````markdown
---
id: python.no-print
title: Avoid print debugging
language: python
level: warn
status: warn
tags: [python, debug]
fixable: false
---

# Avoid print debugging

Use logging instead of committed `print` calls.

```grit
language python
`print($value)`
```

## Bad

```python
print(user)
```

## Good

```python
logger.info("user=%s", user)
```
````

Guidelines:

- Keep each rule focused on one preference.
- Add both Bad and Good examples before promotion to `enforced`.
- Keep uncertain generated rules as `draft`.
- Prefer installing maintained rule packs before creating project-local rules.
- If a preference cannot be expressed in GritQL yet, keep the rule as `draft`
  with a TODO GritQL body rather than adding another execution path.
