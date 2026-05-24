---
id: go.no-context-todo
title: Avoid context.TODO in application flow
language: go
level: warn
status: warn
tags: [go, context]
---

# Avoid context.TODO in application flow

Accept or derive an explicit context so cancellation and deadlines stay visible.

```grit
language go
`context.TODO()`
```

## Bad

```go
ctx := context.TODO()
```

## Good

```go
ctx, cancel := context.WithTimeout(parentCtx, timeout)
defer cancel()
```
