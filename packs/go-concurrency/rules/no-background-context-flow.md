---
id: go-concurrency.no-background-context-flow
title: Avoid context.Background in request flow
language: go
level: warn
tags: [go, concurrency, context]
---

# Avoid context.Background in request flow

Use the caller's context so cancellation, deadlines, and tracing continue through the call chain.

```grit
language go
`context.Background()`
```

## Bad

```go
ctx := context.Background()
```

## Good

```go
ctx := req.Context()
```
