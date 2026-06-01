---
id: python-async.no-time-sleep
title: Avoid time.sleep in async-capable code
language: python
level: warn
status: warn
tags: [python, async, blocking]
---

# Avoid time.sleep in async-capable code

Use an async sleep or inject a scheduler so one request cannot block the event loop.

```grit
language python
`time.sleep($duration)`
```

## Bad

```python
time.sleep(1)
```

## Good

```python
await asyncio.sleep(1)
```
