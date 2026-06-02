---
id: go-effective-go.no-recursive-stringer-format
title: Avoid recursive String formatting
language: go
level: warn
tags: [go, effective-go, formatting]
---

# Avoid recursive String formatting

Inside `String()`, convert the receiver before formatting it with a string verb so `fmt` does not call `String()` again.

```grit
language go
`func ($receiver $type) String() string { return fmt.Sprintf($format, $receiver) }`
```

## Bad

```go
type MyString string

func (m MyString) String() string {
    return fmt.Sprintf("MyString=%s", m)
}
```

## Good

```go
type MyString string

func (m MyString) String() string {
    return fmt.Sprintf("MyString=%s", string(m))
}
```
