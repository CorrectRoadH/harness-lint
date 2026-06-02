---
id: go-effective-go.no-else-after-return
title: Omit else after return
language: go
level: warn
tags: [go, effective-go, control-flow]
---

# Omit else after return

Let the successful path continue down the page after guard clauses that return.

```grit
language go
`if $condition { return $value } else { $body }`
```

## Bad

```go
func Name(user *User) string {
    if user == nil {
        return "anonymous"
    } else {
        return user.Name
    }
}
```

## Good

```go
func Name(user *User) string {
    if user == nil {
        return "anonymous"
    }
    return user.Name
}
```
