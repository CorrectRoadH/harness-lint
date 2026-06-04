---
id: typescript.no-console-log
title: Avoid committed console.log
language: typescript
level: warn
tags: [typescript, javascript, logging]
---

# Avoid committed console.log

Use repository logging, telemetry, or explicit UI state instead of committed `console.log` calls.

```grit
language js
`console.log($value)` where {
  or {
    $filename <: r".*\.ts",
    $filename <: r".*\.tsx",
    $filename <: r".*\.js",
    $filename <: r".*\.jsx",
    $filename <: r".*\.mjs",
    $filename <: r".*\.cjs"
  }
}
```

## Bad

```ts
console.log(user)
```

## Good

```ts
logger.info({ userId: user.id }, "loaded user")
```
