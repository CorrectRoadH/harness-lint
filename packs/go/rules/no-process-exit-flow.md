---
id: go.no-process-exit-flow
title: Avoid process exits in normal service flow
language: go
level: warn
status: warn
tags: [go, lifecycle, errors]
---

# Avoid process exits in normal service flow

Return errors to the application boundary instead of calling `log.Fatal` or `os.Exit` from reusable service code.

```grit
language go
or {
  `log.Fatal($value)`,
  `os.Exit($code)`
}
```

## Bad

```go
log.Fatal(err)
```

## Good

```go
return err
```
