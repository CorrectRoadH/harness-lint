---
id: rust.no-unwrap-expect
title: Review unwrap and expect calls
language: rust
level: warn
tags: [rust, errors]
---

# Review unwrap and expect calls

Prefer `?`, explicit matches, or domain errors over panicking on `Option` and `Result` values in application flow.

```grit
language rust
or {
  `$value.unwrap()`,
  `$value.expect($message)`
}
```

## Bad

```rust
let token = request.headers().get("token").unwrap();
```

## Good

```rust
let token = request
    .headers()
    .get("token")
    .ok_or_else(|| anyhow::anyhow!("missing token"))?;
```
