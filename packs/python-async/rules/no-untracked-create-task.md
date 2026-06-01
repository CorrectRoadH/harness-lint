---
id: python-async.no-untracked-create-task
title: Avoid untracked create_task calls
language: python
level: warn
tags: [python, async, lifecycle]
---

# Avoid untracked create_task calls

Keep task handles visible so cancellation, errors, and shutdown behavior are explicit.

```grit
language python
`asyncio.create_task($call)`
```

## Bad

```python
asyncio.create_task(send_email(user))
```

## Good

```python
task = asyncio.create_task(send_email(user))
pending_tasks.add(task)
```
