---
id: rust.no-dbg-macro
title: Avoid committed dbg macros
language: rust
level: warn
tags: [rust, debug]
---

# Avoid committed dbg macros

Use structured logging, assertions, or explicit test expectations instead of committed `dbg!` calls.

```grit
language rust
`dbg!($value)`
```

## Bad

```rust
let user = dbg!(load_user(id)?);
```

## Good

```rust
let user = load_user(id)?;
tracing::debug!(user_id = %user.id, "loaded user");
```
