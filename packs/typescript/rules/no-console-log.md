---
id: typescript.no-console-log
title: Avoid committed console.log
language: typescript
level: warn
status: warn
tags: [typescript, javascript, logging]
---

# Avoid committed console.log

Use repository logging, telemetry, or explicit UI state instead of committed `console.log` calls.

```grit
language js
`console.log($value)`
```

## Bad

```ts
console.log(user)
```

## Good

```ts
logger.info({ userId: user.id }, "loaded user")
```
