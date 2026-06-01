---
id: go-effective-go.no-init-flow
title: Review init functions
language: go
level: warn
status: warn
tags: [go, effective-go, package-design]
---

# Review init functions

Prefer explicit constructors and dependency setup over hidden package initialization.

```grit
language go
`func init() { $body }`
```

## Bad

```go
func init() {
    connect()
}
```

## Good

```go
func NewClient(cfg Config) (*Client, error) {
    return connect(cfg)
}
```
