---
id: go-effective-go.no-single-result-type-assertion
title: Use comma ok type assertions
language: go
level: warn
tags: [go, effective-go, interfaces]
---

# Use comma ok type assertions

Use the two-result form before returning asserted interface values so failed assertions can be handled without a panic.

```grit
language go
`return $value.($type)`
```

## Bad

```go
func StringValue(value any) string {
    return value.(string)
}
```

## Good

```go
func StringValue(value any) (string, bool) {
    text, ok := value.(string)
    return text, ok
}
```
