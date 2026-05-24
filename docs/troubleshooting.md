# Troubleshooting

## Missing Config

Run:

```sh
harness-lint init
```

## Grit Is Not Installed

All executable harness-lint rules require the Grit CLI.

## Existing `.grit`

harness-lint does not overwrite `.grit`. Generated Grit files are written to:

```text
.harness/generated/.grit/
```

## Registry Candidate Is Wrong

Use:

```sh
harness-lint rule suggest --local "<constraint>"
```

Then edit the draft in `harness/rules/`.

## Rule Pack Install Fails

Pack installs clone into a temporary directory first. Failed installs are rolled back before touching the active cached pack.
