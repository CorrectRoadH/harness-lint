---
id: go-effective-go.no-new-map-allocation
title: Use make for maps
language: go
level: warn
tags: [go, effective-go, allocation]
---

# Use make for maps

Use `make` for maps so the map value is initialized before writes.

```grit
language go
`new(map[$key]$value)`
```

## Bad

```go
func Counts() *map[string]int {
    return new(map[string]int)
}
```

## Good

```go
func Counts() map[string]int {
    return make(map[string]int)
}
```
