---
id: python.no-eval-exec
title: Avoid eval and exec
language: python
level: warn
status: warn
tags: [python, security]
---

# Avoid eval and exec

Avoid dynamic code execution in normal application code. Prefer explicit parsers, registries, or strategy objects.

```grit
language python
or {
  `eval($code)`,
  `exec($code)`
}
```

## Bad

```python
result = eval(expression)
```

## Good

```python
result = expression_parser.parse(expression)
```
