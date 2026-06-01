---
id: go-concurrency.no-time-after-loop
title: Review time.After allocations
language: go
level: warn
status: warn
tags: [go, concurrency, timers]
---

# Review time.After allocations

Use reusable timers in hot loops so timer allocations and delayed cleanup do not pile up.

```grit
language go
`time.After($duration)`
```

## Bad

```go
case <-time.After(timeout):
    return ErrTimeout
```

## Good

```go
timer := time.NewTimer(timeout)
defer timer.Stop()
```
