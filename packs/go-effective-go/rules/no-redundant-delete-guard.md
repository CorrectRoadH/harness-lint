---
id: go-effective-go.no-redundant-delete-guard
title: Delete map entries directly
language: go
level: warn
tags: [go, effective-go, maps]
---

# Delete map entries directly

`delete` is safe when the key is absent, so a presence guard around the same key only adds noise.

```grit
language go
`if _, $ok := $map[$key]; $ok { delete($map, $key) }`
```

## Bad

```go
func Remove(cache map[string]Item, key string) {
    if _, ok := cache[key]; ok {
        delete(cache, key)
    }
}
```

## Good

```go
func Remove(cache map[string]Item, key string) {
    delete(cache, key)
}
```
