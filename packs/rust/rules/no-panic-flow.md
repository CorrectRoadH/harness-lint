---
id: rust.no-panic-flow
title: Avoid panic in normal application flow
language: rust
level: warn
tags: [rust, errors]
---

# Avoid panic in normal application flow

Return errors or handle invalid states explicitly so callers can decide how failures should surface.

```grit
language rust
or {
  `panic!()`,
  `panic!($message)`
}
```

## Bad

```rust
fn load_config(path: &Path) -> Config {
    panic!("missing config: {}", path.display())
}
```

## Good

```rust
fn load_config(path: &Path) -> anyhow::Result<Config> {
    anyhow::bail!("missing config: {}", path.display())
}
```
