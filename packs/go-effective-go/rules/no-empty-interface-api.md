---
id: go-effective-go.no-empty-interface-api
title: Avoid empty interface API boundaries
language: go
level: warn
tags: [go, effective-go, api]
---

# Avoid empty interface API boundaries

Prefer concrete types, type parameters, or narrow interfaces over broad `interface{}` contracts.

```grit
language go
`interface{}`
```

## Bad

```go
func Encode(value interface{}) ([]byte, error) {
    return json.Marshal(value)
}
```

## Good

```go
func EncodeUser(value User) ([]byte, error) {
    return json.Marshal(value)
}
```
