# Installing `go-generated`

This pack is the reference example of a **concept-scoped** pack: every rule ships
`runs_on: ["generated"]` instead of hardcoding a path. The rules do nothing until
the installing project declares which files *are* generated and exposes them under
the portable `generated` concept.

## Concepts this pack expects

| Concept     | What the project must point it at                              |
| ----------- | -------------------------------------------------------------- |
| `generated` | Committed, generated Go code (e.g. `*.pb.go`, generated mocks) |

## Wiring (required)

Add a default-closed file set to your `harness.toml` whose `provides` lists
`generated`, pointing `paths` at wherever your generated Go actually lives:

```toml
[file_sets.generated]
paths = ["backend/gen/**/*.pb.go", "internal/mocks/**/*.go"]
default_rules = false        # ordinary rules stop scanning these files...
provides = ["generated"]     # ...and this pack's rules reach in via the concept
```

Then install the pack:

```sh
harness-lint install go-generated
harness-lint doctor
harness-lint check --all
```

## Why the wiring is not optional

`runs_on: ["generated"]` is exclusive scope. If you install the pack but never
declare a file set that `provides = ["generated"]`, the rules match **nothing**
and `harness-lint doctor` reports `harness.unknown-run-target` (an error): a rule
runs on a concept no file set provides.

The split is deliberate:

- The pack owns the **concept** (`generated`) and the rule logic. It ships the
  same `runs_on: ["generated"]` to every project without knowing your layout.
- Your `harness.toml` owns the **location** (`paths`). Rename `[file_sets.generated]`
  to anything you like — as long as its `provides` still lists `generated`, the
  pack rules keep working.
- `default_rules = false` is what makes this region *worth* a dedicated pack:
  ordinary rules (your `go` pack, etc.) skip generated code, and only the rules
  that explicitly ask via `runs_on` ever look at it.

If you want a rule to scan both ordinary source and generated code, that rule
lists both regions: `runs_on: ["default", "generated"]`.
