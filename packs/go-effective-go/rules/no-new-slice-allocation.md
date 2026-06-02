---
id: go-effective-go.no-new-slice-allocation
title: Use make for slices
language: go
level: warn
tags: [go, effective-go, allocation]
---

# Use make for slices

Use `make` for slices so the slice descriptor points at initialized storage.

```grit
language go
`new([]$value)`
```

## Bad

```go
func Buffer() *[]byte {
    return new([]byte)
}
```

## Good

```go
func Buffer() []byte {
    return make([]byte, 0)
}
```
