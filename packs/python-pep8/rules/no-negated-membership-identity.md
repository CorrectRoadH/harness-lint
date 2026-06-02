---
id: python-pep8.no-negated-membership-identity
title: Use not in and is not operators
language: python
level: warn
tags: [python, pep8, readability]
---

# Use not in and is not operators

Prefer `x not in y` and `x is not y` over negating `in` or `is` expressions.

```grit
language python
or {
  `not $value in $items`,
  `not $value is $other`
}
```

## Bad

```python
if not user in active_users:
    return
```

## Good

```python
if user not in active_users:
    return
```
