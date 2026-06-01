---
id: go.no-panic-flow
title: Avoid panic in normal service flow
language: go
level: warn
tags: [go, errors]
---

# Avoid panic in normal service flow

Return or handle errors in application and service code. Keep `panic` for truly unrecoverable initialization failures.

```grit
language go
`panic($value)`
```

## Bad

```go
panic(err)
```

## Good

```go
return err
```
