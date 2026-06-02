---
id: python-pep8.no-boolean-equality
title: Avoid boolean equality comparisons
language: python
level: warn
tags: [python, pep8, readability]
---

# Avoid boolean equality comparisons

Use truthiness, `not`, or an explicit predicate instead of comparing values to `True` or `False`.

```grit
language python
or {
  `$value == True`,
  `$value == False`,
  `$value != True`,
  `$value != False`
}
```

## Bad

```python
if is_active == True:
    enable()
```

## Good

```python
if is_active:
    enable()
```
