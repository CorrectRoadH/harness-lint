---
id: typescript-react.no-string-ref
title: Avoid string refs
language: typescript
level: warn
tags: [typescript, react, refs]
---

# Avoid string refs

Use callback refs or `useRef` so ref ownership is explicit and compatible with modern React.

```grit
language js
`<$component ref="$name" />`
```

## Bad

```tsx
<input ref="searchInput" />
```

## Good

```tsx
const searchInputRef = useRef<HTMLInputElement>(null)
return <input ref={searchInputRef} />
```
