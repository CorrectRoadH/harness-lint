---
id: typescript-react.no-dangerous-html
title: Review dangerous HTML rendering
language: typescript
level: warn
tags: [typescript, react, security]
---

# Review dangerous HTML rendering

Avoid direct HTML injection unless the content has a trusted sanitizer and ownership boundary.

```grit
language js
`dangerouslySetInnerHTML={$value}` where {
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
<article dangerouslySetInnerHTML={{ __html: body }} />
```

## Good

```tsx
<Markdown content={body} />
```
