---
id: typescript-react.no-clone-element
title: Review cloneElement usage
language: typescript
level: warn
tags: [typescript, react, composition]
---

# Review cloneElement usage

Prefer explicit props, render props, or context over cloning elements and mutating their props invisibly.

```grit
language js
or {
  `React.cloneElement($element, $props)`,
  `cloneElement($element, $props)`
} where {
  or {
    $filename <: r".*\.tsx",
    $filename <: r".*\.jsx",
    $filename <: r".*\.ts",
    $filename <: r".*\.js",
    $filename <: r".*\.mjs",
    $filename <: r".*\.cjs"
  }
}
```

## Bad

```tsx
return React.cloneElement(child, { selected: true })
```

## Good

```tsx
return <Item selected>{child}</Item>
```
