---
id: python-pep8.no-none-equality
title: Use identity comparisons for None
language: python
level: warn
tags: [python, pep8, readability]
---

# Use identity comparisons for None

Prefer identity checks for `None` so code follows Python conventions and avoids overloaded equality surprises.

```grit
language python
or {
  `$value == None`,
  `$value != None`
}
```

## Bad

```python
if user != None:
    return
```

## Good

```python
if user is not None:
    return
```
