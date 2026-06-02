---
id: go-effective-go.no-blank-placeholder-assignment
title: Remove blank identifier placeholders
language: go
level: warn
tags: [go, effective-go, blank-identifier]
---

# Remove blank identifier placeholders

Blank identifier assignments that only silence unused-variable errors mark unfinished code; use the value or remove the work-in-progress placeholder.

```grit
language go
`_ = $value`
```

## Bad

```go
func Load(path string) error {
    file, err := os.Open(path)
    if err != nil {
        return err
    }
    _ = file
    return nil
}
```

## Good

```go
func Load(path string) error {
    file, err := os.Open(path)
    if err != nil {
        return err
    }
    defer file.Close()
    return parse(file)
}
```
