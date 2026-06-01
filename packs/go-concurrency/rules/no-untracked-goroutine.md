---
id: go-concurrency.no-untracked-goroutine
title: Review untracked goroutines
language: go
level: warn
tags: [go, concurrency, lifecycle]
---

# Review untracked goroutines

Tie goroutines to a context, errgroup, worker pool, or explicit lifecycle owner.

```grit
language go
`go $call`
```

## Bad

```go
go sendEmail(user)
```

## Good

```go
group.Go(func() error {
    return sendEmail(ctx, user)
})
```
