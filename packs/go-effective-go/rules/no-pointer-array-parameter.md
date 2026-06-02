---
id: go-effective-go.no-pointer-array-parameter
title: Prefer slices over pointer array parameters
language: go
level: warn
tags: [go, effective-go, arrays, slices]
---

# Prefer slices over pointer array parameters

Use slices for sequence parameters unless the fixed array size is part of the contract.

```grit
language go
`func $name($arg *[$size]$value) { $body }`
```

## Bad

```go
func Fill(buf *[1024]byte) {
    for i := range buf {
        buf[i] = 0
    }
}
```

## Good

```go
func Fill(buf []byte) {
    for i := range buf {
        buf[i] = 0
    }
}
```
