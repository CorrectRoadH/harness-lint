---
id: python-pep8.no-lambda-assignment
title: Avoid assigning lambdas
language: python
level: warn
tags: [python, pep8, readability]
---

# Avoid assigning lambdas

Use `def` for named functions so tracebacks, documentation, and call sites show a real function name.

```grit
language python
`$name = lambda $args: $body`
```

## Bad

```python
is_even = lambda value: value % 2 == 0
```

## Good

```python
def is_even(value):
    return value % 2 == 0
```
