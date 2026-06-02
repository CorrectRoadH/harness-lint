---
id: go-effective-go.no-ignored-error-blank
title: Do not discard call errors with blank identifiers
language: go
level: warn
tags: [go, effective-go, errors, blank-identifier]
---

# Do not discard call errors with blank identifiers

When a call returns a value and an error, do not use `_` to ignore the error result; check it and preserve the reason for failure.

```grit
language go
`$value, _ := $call($args)`
```

## Bad

```go
func IsDir(path string) bool {
    info, _ := os.Stat(path)
    return info.IsDir()
}
```

## Good

```go
func IsDir(path string) (bool, error) {
    info, err := os.Stat(path)
    if err != nil {
        return false, err
    }
    return info.IsDir(), nil
}
```
