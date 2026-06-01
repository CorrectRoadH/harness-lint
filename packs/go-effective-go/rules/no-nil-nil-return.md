---
id: go-effective-go.no-nil-nil-return
title: Avoid nil nil returns
language: go
level: warn
tags: [go, effective-go, errors]
---

# Avoid nil nil returns

Returning `(nil, nil)` from value-plus-error APIs leaves callers without either data or an explanation.

```grit
language go
`return nil, nil`
```

## Bad

```go
func Find(id string) (*User, error) {
    return nil, nil
}
```

## Good

```go
func Find(id string) (*User, error) {
    return nil, ErrNotFound
}
```
