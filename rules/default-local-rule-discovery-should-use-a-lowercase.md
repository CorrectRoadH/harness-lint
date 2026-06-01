---
id: local.default-local-rule-discovery-should-use-a-lowercase
title: "Default Local Rule Discovery Should Use A Lowercase"
language: rust
level: warn
skill:
tags: [local, ai-feedback]
---

# Default Local Rule Discovery Should Use A Lowercase

Default local rule discovery should use a lowercase rules directory, and harness.toml should allow configuring the rule directory location.

```grit
language rust
`"Rules"`
```

## Bad

```rust
pub const USER_RULE_DIR: &str = "Rules";
```

## Good

```rust
pub const USER_RULE_DIR: &str = "rules";
```
