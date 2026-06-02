# harness lint

[![CI](https://img.shields.io/github/actions/workflow/status/CorrectRoadH/harness-lint/ci.yml?branch=main&label=CI&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/CorrectRoadH/harness-lint?label=release&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/releases)
[![rust](https://img.shields.io/badge/rust-2024-blue?style=flat-square)](Cargo.toml)
[![homebrew](https://img.shields.io/badge/homebrew-CorrectRoadH%2Ftap-fbb040?style=flat-square)](https://github.com/CorrectRoadH/homebrew-tap)

[English](README.md) | [简体中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

harness-lint は Harness Engineering のための次世代 lint ツールです。vibe coding では、何度修正しても、`AGENTS.md` に指示を書いても、AI がその指示を無視してしまうことがあります。このツールは Lint Driven Development によってその問題を解決します。ユーザーが AI Agent に「やってはいけないこと」を伝えたら、まずそれを固定された lint ルールに変換し、高速で厳格なチェックによって同じ失敗を防ぎます。

従来の lint ツールと比べて、harness lint のルールは人間が読みやすく、理解しやすい形で書けます。また、AI coding のワークフローとベストプラクティスに合わせて設計されています。

## インストール

```sh
brew install getgrit/tap/grit
brew install CorrectRoadH/tap/harness-lint
```

## Agent 用にリポジトリを初期化

```text
READ https://raw.githubusercontent.com/CorrectRoadH/harness-lint/refs/heads/main/INIT.md and install harness lint for this code repo
```

## よく使うコマンド

```sh
harness-lint check --changed
harness-lint check --all
harness-lint rule list
harness-lint search "python typing"
harness-lint list --available
harness-lint install python
harness-lint install python-pep8
harness-lint outdated
harness-lint update
harness-lint remove python
```

## ローカルルール

プロジェクト固有のカスタムルールは、デフォルトでは `rules/*.md` に置きます。別の場所に置きたい場合は、`harness.toml` で設定できます。

```toml
[rules]
local = ["custom-rules"]
```

`harness-lint rule create` は、設定された最初のローカルルールディレクトリに新しいルールを書き込みます。ローカルルールは作成時点で実行可能な GritQL を含める必要があります。

```sh
harness-lint rule create "Avoid print debugging" --language python --grit '`print($value)`'
```

フィードバックを信頼できる GritQL pattern として表現できない場合は、harness-lint rule を作成しないでください。その制約は agent 指示、review checklist、またはプロジェクト文書に残してください。

ルールを作成したら、広い範囲のチェックに頼る前に、そのルールだけを実行して期待するファイルが報告されることを確認してください。`check` に path を渡して rule scope を擬似的に作らないでください。特定のファイルだけに適用する必要がある場合は、GritQL の `$filename` で表現します。

```sh
harness-lint rule verify local.no-print
harness-lint check --all --rule local.no-print
```

ルールファイルの例:

````markdown
---
id: local.no-print
title: Avoid print debugging
language: python
level: warn
skill: tdd
tags: [local, python]
---

# Avoid print debugging

Use logging instead of committed print calls.

```grit
language python
`print($value)`
```

## Bad

```python
print(user)
```

## Good

```python
logger.info("user=%s", user)
```
````

ルールを特定のファイルに限定したい場合は、GritQL の `$filename` 条件を直接書きます。

```grit
`console.log($value)` where {
  $filename <: r".*src/.*\.ts",
  !$filename <: r".*\.test\.ts"
}
```
