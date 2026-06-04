---
id: typescript-react.no-default-props
title: Avoid defaultProps on function components
language: typescript
level: warn
tags: [typescript, react, props]
---

# Avoid defaultProps on function components

Use default parameter values so TypeScript sees the defaulting behavior at the component boundary.

```grit
language js
`$component.defaultProps = $value` where {
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
Button.defaultProps = {
  disabled: false,
}
```

## Good

```tsx
function Button({ disabled = false }: ButtonProps) {
  return <button disabled={disabled} />
}
```
