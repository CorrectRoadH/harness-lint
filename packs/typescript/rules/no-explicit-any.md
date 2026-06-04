---
id: typescript.no-explicit-any
title: Avoid explicit any
language: typescript
level: warn
tags: [typescript, typing]
---

# Avoid explicit any

Prefer `unknown`, generics, discriminated unions, or domain types over `any`.

```grit
language js
`any` where {
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
function serialize(value: any) {
  return JSON.stringify(value)
}
```

## Good

```ts
function serialize(value: UserEvent) {
  return JSON.stringify(value)
}
```
