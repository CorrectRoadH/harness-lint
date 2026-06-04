---
id: typescript.no-var
title: Avoid var declarations
language: typescript
level: warn
tags: [typescript, javascript]
---

# Avoid var declarations

Use `const` by default and `let` when reassignment is required.

```grit
language js
`var $name = $value` where {
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
var total = 0
```

## Good

```ts
let total = 0
```
