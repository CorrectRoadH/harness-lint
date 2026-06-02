---
id: python-pep8.no-type-equality
title: Use isinstance for type checks
language: python
level: warn
tags: [python, pep8, readability]
---

# Use isinstance for type checks

Use `isinstance` instead of comparing `type(...)` results so subclass behavior is handled correctly.

```grit
language python
or {
  `type($value) == $kind`,
  `type($value) != $kind`
}
```

## Bad

```python
if type(user) == AdminUser:
    promote(user)
```

## Good

```python
if isinstance(user, AdminUser):
    promote(user)
```
