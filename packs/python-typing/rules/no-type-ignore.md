---
id: python-typing.no-type-ignore
title: Avoid broad type ignore comments
language: python
level: warn
tags: [python, typing]
---

# Avoid broad type ignore comments

Prefer a narrower type fix, adapter, protocol, or targeted ignore with a reason.

```grit
language python
`# type: ignore`
```

## Bad

```python
payload = load_raw()  # type: ignore
```

## Good

```python
payload: UserPayload = parse_payload(load_raw())
```
