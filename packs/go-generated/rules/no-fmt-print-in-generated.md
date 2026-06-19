---
id: go-generated.no-fmt-print-in-generated
title: Avoid fmt print debugging in generated Go code
language: go
level: warn
runs_on: ["generated"]
tags: [go, generated]
---

# Avoid fmt print debugging in generated Go code

Generated code should never carry `fmt.Print` debugging: a stray print means the
generator template leaked debug output, and hand-deleting it is pointless because
regeneration restores it. The ordinary `go` pack skips generated files (the
default region excludes them); this rule reaches into the `generated` region so
the template gets fixed instead. See this pack's `INSTALL.md`.

```grit
language go
or {
  `fmt.Print($value)`,
  `fmt.Println($value)`,
  `fmt.Printf($format, $value)`
}
```

## Bad

```go
func decode(raw []byte) {
	fmt.Println("decoding", raw)
}
```

## Good

```go
func decode(raw []byte) error {
	return unmarshal(raw)
}
```
