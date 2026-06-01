---
id: python-pep8.no-bare-except
title: Avoid bare except clauses
language: python
level: warn
tags: [python, pep8, errors]
---

# Avoid bare except clauses

Catch a specific exception type so control flow and failure handling stay explicit.

```grit
language python
`except:`
```

## Bad

```python
try:
    load_user()
except:
    return None
```

## Good

```python
try:
    load_user()
except UserNotFound:
    return None
```
