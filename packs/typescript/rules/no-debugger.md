---
id: typescript.no-debugger
title: Avoid committed debugger statements
language: typescript
level: warn
tags: [typescript, javascript, debugging]
---

# Avoid committed debugger statements

Remove `debugger` statements before committing application code.

```grit
language js
`debugger`
```

## Bad

```ts
debugger
```

## Good

```ts
logger.debug("state changed")
```
