---
id: rust.no-todo-unimplemented
title: Avoid committed todo and unimplemented macros
language: rust
level: warn
tags: [rust, scaffolding]
---

# Avoid committed todo and unimplemented macros

Replace scaffolding macros with real behavior, explicit errors, or a failing test that documents the missing branch.

```grit
language rust
or {
  `todo!()`,
  `todo!($message)`,
  `unimplemented!()`,
  `unimplemented!($message)`
}
```

## Bad

```rust
fn render() -> Html {
    todo!("wire renderer")
}
```

## Good

```rust
fn render() -> anyhow::Result<Html> {
    anyhow::bail!("renderer is not configured")
}
```
