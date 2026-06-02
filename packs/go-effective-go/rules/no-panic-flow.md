---
id: go-effective-go.no-panic-flow
title: Avoid panic in library flow
language: go
level: warn
tags: [go, effective-go, errors]
---

# Avoid panic in library flow

Return errors for recoverable failures instead of exposing panics to callers.

```grit
language go
`panic($value)`
```

## Bad

```go
func Parse(input string) *Config {
    if input == "" {
        panic("empty config")
    }
    return parse(input)
}
```

## Good

```go
func Parse(input string) (*Config, error) {
    if input == "" {
        return nil, ErrEmptyConfig
    }
    return parse(input), nil
}
```
