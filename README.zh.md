# harness lint

[![CI](https://img.shields.io/github/actions/workflow/status/CorrectRoadH/harness-lint/ci.yml?branch=main&label=CI&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/actions/workflows/ci.yml)
[![release](https://img.shields.io/github/v/release/CorrectRoadH/harness-lint?label=release&style=flat-square)](https://github.com/CorrectRoadH/harness-lint/releases)
[![rust](https://img.shields.io/badge/rust-2024-blue?style=flat-square)](Cargo.toml)
[![homebrew](https://img.shields.io/badge/homebrew-CorrectRoadH%2Ftap-fbb040?style=flat-square)](https://github.com/CorrectRoadH/homebrew-tap)

[English](README.md) | [简体中文](README.zh.md) | [日本語](README.ja.md) | [한국어](README.ko.md)

harness-lint 是一个用于 Harness Engineering 的新时代 Lint 工具。在 vibe coding 中，AI 经常不按你的要求来做，就算你反复纠正，或者把要求写进 `AGENTS.md`，它也可能不遵守。该工具用 Lint Driven Development 解决这个问题：当用户告诉 AI Agent 不要做什么时，先把要求转换成固定的 lint 规则，再用快速、严格的检查防止 AI 重复犯错。

对比传统的 lint，harness lint 的规则是高度人类可读的、可理解的。并且适配 AI Coding 工作流与最佳实践。

## 安装

```sh
brew install getgrit/tap/grit
brew install CorrectRoadH/tap/harness-lint
```

## 为 Agent 初始化当前仓库

```text
READ https://raw.githubusercontent.com/CorrectRoadH/harness-lint/refs/heads/main/INIT.md and install harness lint for this code repo
```


## 常用命令

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


## 本地规则

自定义项目规则默认放在 `rules/*.md`。如果想放在别的位置，可以在 `harness.toml` 中配置：

```toml
[rules]
local = ["custom-rules"]
```

`harness-lint rule create` 会把新规则写入第一个配置的本地规则目录。本地规则必须在创建时提供可执行 GritQL：

```sh
harness-lint rule create "禁止 print 调试" --language python --grit '`print($value)`'
```

如果一条反馈无法稳定表达成 GritQL pattern，就不要创建 harness-lint rule。把这类约束保留在 agent 指令、review checklist 或项目文档中。

创建规则后，先单独运行这条规则并确认它命中了预期文件，再依赖更大范围的检查。不要通过给 `check` 传路径来模拟规则范围；如果规则只应该作用于部分文件，必须在 GritQL 中用 `$filename` 表达。

```sh
harness-lint rule verify local.no-print
harness-lint check --all --rule local.no-print
```

规则文件示例：

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

如果一条规则只应该作用于部分文件，直接在 GritQL 里用 `$filename` 写文件范围：

```grit
language js
`console.log($value)` where {
  $filename <: r".*src/.*\.ts",
  !$filename <: r".*\.test\.ts"
}
```
