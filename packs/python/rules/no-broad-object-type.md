---
id: python.no-broad-object-type
title: Avoid broad object types
language: python
level: warn
tags: [python, typing]
---

# Avoid broad object types

Prefer concrete models, explicit unions, protocols, or typed boundary objects. Use `object` only for true opaque interop boundaries.

```grit
language python
`object`
```

## Bad

```python
def serialize(value: object) -> dict:
    return {"value": value}
```

## Good

```python
def serialize(value: UserEvent) -> SerializedEvent:
    return SerializedEvent(value=value.name)
```
