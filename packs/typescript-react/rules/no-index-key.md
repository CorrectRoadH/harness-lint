---
id: typescript-react.no-index-key
title: Avoid array index keys
language: typescript
level: warn
tags: [typescript, react, rendering]
---

# Avoid array index keys

Use stable domain ids for React keys so reordering does not confuse component state.

```grit
language js
`key={index}`
```

## Bad

```tsx
items.map((item, index) => <Row key={index} item={item} />)
```

## Good

```tsx
items.map((item) => <Row key={item.id} item={item} />)
```
