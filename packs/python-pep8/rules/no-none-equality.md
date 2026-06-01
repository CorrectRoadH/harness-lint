---
id: python-pep8.no-none-equality
title: Use is None comparisons
language: python
level: warn
status: warn
tags: [python, pep8, readability]
---

# Use is None comparisons

Prefer identity checks for `None` so code follows Python conventions and avoids overloaded equality surprises.

```grit
language python
`$value == None`
```

## Bad

```python
if user == None:
    return
```

## Good

```python
if user is None:
    return
```
