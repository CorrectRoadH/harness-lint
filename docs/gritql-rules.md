# GritQL Rules

Use `engine: grit` when a rule depends on syntax or AST structure.

Rule skeleton:

````markdown
---
id: python.no-print
title: Avoid print debugging
engine: grit
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
- Use text or regex rules for simple document/template checks.

