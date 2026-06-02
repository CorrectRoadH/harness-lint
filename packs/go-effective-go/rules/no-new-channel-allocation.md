---
id: go-effective-go.no-new-channel-allocation
title: Use make for channels
language: go
level: warn
tags: [go, effective-go, allocation]
---

# Use make for channels

Use `make` for channels so the channel value is initialized and ready to communicate.

```grit
language go
`new(chan $value)`
```

## Bad

```go
func WorkQueue() *chan Work {
    return new(chan Work)
}
```

## Good

```go
func WorkQueue() chan Work {
    return make(chan Work)
}
```
