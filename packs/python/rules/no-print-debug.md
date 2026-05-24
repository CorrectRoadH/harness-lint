---
id: python.no-print-debug
title: Avoid committed print debugging
language: python
level: warn
status: warn
tags: [python, logging]
---

# Avoid committed print debugging

Use repository logging or structured diagnostics instead of committed `print` calls.

```grit
language python
`print($value)`
```

## Bad

```python
print(user_id)
```

## Good

```python
logger.info("loaded user %s", user_id)
```
