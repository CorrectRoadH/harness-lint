---
id: go.no-fmt-print-debug
title: Avoid fmt print debugging
language: go
level: warn
tags: [go, logging]
---

# Avoid fmt print debugging

Use the repository logger instead of committed `fmt.Print` debugging in service code.

```grit
language go
or {
  `fmt.Print($value)`,
  `fmt.Println($value)`,
  `fmt.Printf($format, $value)`
}
```

## Bad

```go
fmt.Println(userID)
```

## Good

```go
logger.Info("loaded user", "user_id", userID)
```
