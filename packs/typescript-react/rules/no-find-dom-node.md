---
id: typescript-react.no-find-dom-node
title: Avoid findDOMNode
language: typescript
level: warn
tags: [typescript, react, refs]
---

# Avoid findDOMNode

Use explicit refs instead of querying a component instance for its underlying DOM node.

```grit
language js
or {
  `ReactDOM.findDOMNode($value)`,
  `findDOMNode($value)`
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
const node = ReactDOM.findDOMNode(this)
```

## Good

```tsx
const nodeRef = useRef<HTMLDivElement>(null)
```
