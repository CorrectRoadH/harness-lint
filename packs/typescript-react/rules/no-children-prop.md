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
`<$component children={$value} />`
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
