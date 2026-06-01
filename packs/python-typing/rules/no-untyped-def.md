---
id: python-typing.no-untyped-def
title: Avoid untyped public functions
language: python
level: warn
status: warn
tags: [python, typing, api]
---

# Avoid untyped public functions

Public functions should expose parameter and return types so callers and agents know the contract.

```grit
language python
`def $name($args):`
```

## Bad

```python
def load_user(user_id):
    return repo.get(user_id)
```

## Good

```python
def load_user(user_id: str) -> User:
    return repo.get(user_id)
```
