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

安装配套的 Codex skill：

```sh
npx skills add CorrectRoadH/harness-lint
```

## 为 Agent 初始化当前仓库

```text
READ https://raw.githubusercontent.com/CorrectRoadH/harness-lint/refs/heads/main/INIT.md and install harness lint for this code repo
```

## Agent 插件（Claude Code 与 Codex）

写进 `AGENTS.md` 的静态指令只会被读一次，且离 agent 真正动手很远。[`plugins/`](plugins/) 里的插件改用生命周期 hook：每次会话重新注入 Lint Driven Development 指引，并在每轮 prompt 前运行 `harness-lint check --changed`，把**当前实际违规**喂给 agent，让它在写下一行代码前就修复。

Claude Code：

```text
/plugin marketplace add CorrectRoadH/harness-lint
/plugin install harness-lint@harness-lint
```

Codex：

```text
codex plugin marketplace add CorrectRoadH/harness-lint
codex plugin install harness-lint
```

两边都附带 `/harness-lint-capture` 命令：审视一次会话里的反馈，把可复用的纠正沉淀成规则（LDD 的另一半）。详见 [`plugins/README.md`](plugins/README.md)。

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

## 配置示例

`harness.toml` 用来控制检查哪些文件、本地规则放在哪里、安装哪些规则包，以及哪些规则结果需要特殊处理。

```toml
# 可选的项目名，会出现在生成的配置里。
[project]
name = "my-service"

# 默认检查行为。
[lint]
# warn 只报告问题；error 会让检查失败。
default_level = "warn"
# `harness-lint check --changed` 用它作为 Git 对比基准。
changed_base = "origin/main"
# 在多次运行之间复用文件级检查结果。
cache = true

# 本项目自己维护的规则文件目录。
[rules]
local = ["rules"]

# 已安装的共享规则包。
[packs]
typescript = "github:CorrectRoadH/harness-lint@main#packs/typescript"

# 不改规则文件，单独调整某条规则的级别。
[overrides]
"typescript.no-console-log" = "error"

# 关闭指定规则。
[disabled]
rules = ["typescript.no-explicit-any"]

# 这些路径会被所有规则跳过，没有任何规则会扫描它们。
[ignore]
paths = ["dist/**", "coverage/**"]

# 一个命名的文件分区：大多数规则应跳过它，只有少数规则需要它。
# default_rules = false 把它从 default 区移除，于是普通规则不会扫描它；
# provides 列出可移植的概念名，让共享 pack 规则不必硬编码你的目录结构就能命中它。
[file_sets.generated]
paths = ["backend/gen/**/*.pb.go", "packages/proto/gen/**"]
default_rules = false
provides = ["generated"]

# 只隐藏某条规则在指定路径上的结果；其他规则仍会检查这些文件。
[[exceptions]]
rule = "typescript.no-console-log"
paths = ["src/generated/**"]
reason = "Generated SDK code is checked in and emits debug output during local mocks."
```

规则通过 frontmatter 里的 `runs_on` 选择进入某个分区。没有 `runs_on` 时，规则扫描 **default** 区（即所有可见、且没有被任何 `default_rules = false` 的集合占据的文件）：

```markdown
---
id: local.proto-no-id-getter
title: Proto messages must generate GetId
language: go
runs_on: ["generated"]   # 只扫描 generated 分区；永远不碰普通源码
---
```

### 配置如何组合

harness-lint 按顺序回答三个相互独立的问题。正是因为把它们分开，上面这些开关才能可预测地叠加。

1. **规则是否启用？** pack 的默认禁用列表和 `[disabled]` 会彻底关闭一条规则；`[overrides]` 只改它的级别。被关闭的规则会跳过后续步骤。
2. **规则扫描哪些文件？** 从整个仓库出发，再按优先级依次应用：
   - 结构性排除——`.git`、`node_modules`、`target`、`.harness`、你的规则目录，以及被 `.gitignore` 忽略的文件，永远不可扫描，任何配置都无法覆盖这一点；
   - `[ignore].paths`——从每条规则中移除，没有任何方式能重新加回来；
   - **file sets**——剩余文件会被分区。`default_rules = false` 的集合会从 `default` 区移除；规则只有在 `runs_on` 里点名该集合（或它 `provides` 的某个概念）时才能触达它。没有 `runs_on` 的规则扫描 `default` 区；
   - 之后再用规则的语言以及 GritQL 的 `$filename` 谓词收窄剩余文件。
3. **结果是否上报？** `[[exceptions]]` 会在匹配路径上隐藏某条已扫描规则的诊断。

`runs_on` 是 exclusive scope（独占范围），不是后门：一条规则能触达一个默认关闭的 file set，仅仅因为它主动申请了，而且永远只有该规则能进。集合的*位置*（`paths`）归项目所有、写在 `harness.toml` 里；规则的*目标*则是一个可移植的概念名（`generated`），所以共享 pack 规则可以直接发布 `runs_on: ["generated"]`，无需知道你的生成代码放在哪里——你只用一个 `provides` 把两者连起来。可以随意给 file set 改名；只要它的 `provides` 仍然列着那个概念，每条 pack 规则都照常工作。既要普通源码又要某个分区？两个都列上：`runs_on: ["default", "generated"]`。

harness-lint 还会检查自身配置的完整性：`[[exceptions]]` / `[ignore]` / `[file_sets.*]` 中已不存在的路径、与 `[ignore]` 重叠或没有任何路径的 file set、`[disabled]` / `[overrides]` 中点名未知规则的条目，以及任何 `runs_on` 点名了无人提供的 file set 或概念的规则——这些都会被上报（默认 warn，file-set／run-target 这类结构性错误为 error；可通过 `[overrides]` 按 id 调整）。


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
