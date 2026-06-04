---
id: typescript-react.no-children-prop
title: Avoid passing children as a prop
language: typescript
level: warn
tags: [typescript, react, rendering]
---

# Avoid passing children as a prop

Nest children between JSX tags so component ownership and render structure stay visible.

```grit
language js
`<$component children={$value} />` where {
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
<Panel children={<Row />} />
```

## Good

```tsx
<Panel>
  <Row />
</Panel>
```
