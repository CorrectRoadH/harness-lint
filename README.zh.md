# harness lint

harness-lint 是一个用于 Harness Enginnering 的新时代 Lint 工具。在 vibe coding中，经常 AI 会不按你的要求来做，就算你反复批评，写在AGENTS.md里面也不遵守。该工具解决了这个问题，使用 Lint Drive Development 的方式。当用户告诉 AI Agent 不要做什么的时候，总是把要求先转换成固定的 lint。通过快速、高速、严格的检查防止你的 AI 犯错。

对比传统的 lint，harness lint 的规则是高度人类可读的、可理解的。并且适配 AI Coding 工作流与最佳实践。

## 安装

```sh
brew install getgrit/tap/grit
brew install CorrectRoadH/tap/harness-lint
```

## init harness lint for your repo For Agent
```
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

`rule suggest --local` 和 `rule new` 会把新规则写入第一个配置的本地规则目录。

规则文件示例：

````markdown
---
id: local.no-print
title: Avoid print debugging
language: python
level: warn
status: draft
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
