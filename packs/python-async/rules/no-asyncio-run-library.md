---
id: python-async.no-asyncio-run-library
title: Avoid asyncio.run inside library flow
language: python
level: warn
tags: [python, async, api]
---

# Avoid asyncio.run inside library flow

Library code should accept an event loop boundary from the caller instead of creating one mid-flow.

```grit
language python
`asyncio.run($call)`
```

## Bad

```python
def refresh():
    return asyncio.run(fetch_remote())
```

## Good

```python
async def refresh():
    return await fetch_remote()
```
