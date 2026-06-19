---
id: go-generated.no-panic-in-generated
title: Avoid panic in generated Go code
language: go
level: warn
runs_on: ["generated"]
tags: [go, generated]
---

# Avoid panic in generated Go code

A `panic` in committed generated code is a crash you cannot fix at the call site:
the fix belongs in the generator template, and the file will be overwritten on the
next regeneration. Ordinary source may panic deliberately, which is why this rule
runs only on the `generated` region — a project opts a file set into it with
`provides = ["generated"]`. See this pack's `INSTALL.md`.

```grit
language go
`panic($message)`
```

## Bad

```go
func GetId() string {
	panic("field Id is not set")
}
```

## Good

```go
func GetId() (string, error) {
	return "", errFieldUnset
}
```
