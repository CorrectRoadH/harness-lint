---
id: python-typing.no-untyped-def
title: Avoid untyped public functions
language: python
level: warn
tags: [python, typing, api]
---

# Avoid untyped public functions

Public functions should expose parameter and return types so callers and agents know the contract.

```grit
language python
or {
  function_definition(name=$name, parameters=$params, body=$body) where {
    !$params <: contains `:`
  },
  function_definition(name=$name, parameters=$params, return_type=$return_type, body=$body) where {
    !$return_type <: r".+"
  }
}
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
