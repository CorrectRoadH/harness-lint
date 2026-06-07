# harness-lint docs

This directory contains the Mintlify documentation site for
[`CorrectRoadH/harness-lint`](https://github.com/CorrectRoadH/harness-lint).

Mintlify should be configured to deploy from this repository with the docs
root set to `docs`.

## Local development

Install the Mintlify CLI:

```sh
npm i -g mint
```

Run the preview from this directory, where `docs.json` lives:

```sh
cd docs
mint dev
```

View your local preview at `http://localhost:3000`.

## Publishing changes

Changes are deployed by Mintlify GitOps from the default branch of
`CorrectRoadH/harness-lint`, using `docs` as the project subdirectory.
