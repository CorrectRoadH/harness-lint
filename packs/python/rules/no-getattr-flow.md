---
id: python.no-getattr-flow
title: Avoid getattr in normal application flow
language: python
level: warn
status: warn
tags: [python, typing, dynamic-access]
---

# Avoid getattr in normal application flow

Prefer explicit typed fields, dedicated adapters, or boundary objects. Reserve `getattr` for true dynamic interop such as plugin hooks, third-party SDK objects, or test shims.

```grit
language python
`getattr($object, $name)`
```

## Bad

```python
value = getattr(user, field_name)
```

## Good

```python
value = user.email
```
