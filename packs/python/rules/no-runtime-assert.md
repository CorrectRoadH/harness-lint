---
id: python.no-runtime-assert
title: Avoid assert for runtime validation
language: python
level: warn
tags: [python, validation]
---

# Avoid assert for runtime validation

Use explicit exceptions or validation helpers in runtime code. Python assertions can be removed with optimized execution.

```grit
language python
`assert $value`
```

## Bad

```python
assert user.email
```

## Good

```python
if not user.email:
    raise ValueError("user email is required")
```
