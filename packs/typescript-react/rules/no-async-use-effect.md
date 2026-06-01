---
id: typescript-react.no-async-use-effect
title: Avoid async useEffect callbacks
language: typescript
level: warn
tags: [typescript, react, effects]
---

# Avoid async useEffect callbacks

Keep the effect callback synchronous and handle cancellation inside it.

```grit
language js
`useEffect(async () => { $body })`
```

## Bad

```tsx
useEffect(async () => {
  await loadUser()
}, [])
```

## Good

```tsx
useEffect(() => {
  void loadUser()
}, [])
```
