---
id: typescript-react.no-dangerous-html
title: Review dangerous HTML rendering
language: typescript
level: warn
status: warn
tags: [typescript, react, security]
---

# Review dangerous HTML rendering

Avoid direct HTML injection unless the content has a trusted sanitizer and ownership boundary.

```grit
language js
`dangerouslySetInnerHTML={$value}`
```

## Bad

```tsx
<article dangerouslySetInnerHTML={{ __html: body }} />
```

## Good

```tsx
<Markdown content={body} />
```
