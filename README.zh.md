# harness-lint

一个很小的 GritQL 规则生态 CLI，用来把反复出现的代码反馈变成可运行的 lint 规则。

harness-lint 主要做三件事：

- 初始化项目里的 GritQL 规则目录
- 安装和更新规则包
- 运行 lint，并帮助 agent 把反馈转成规则草稿

它不自动修代码。可执行规则只使用 GritQL。

## 安装

```sh
brew install CorrectRoadH/tap/harness-lint
```

`check` 需要单独安装 `grit`：

```sh
brew install getgrit/tap/grit
```

检查环境：

```sh
harness-lint doctor
```

## 第一次接入仓库

如果你想让 AI coding agent 帮一个仓库第一次接入 harness-lint，把 [INIT.md](INIT.md) 的内容复制给 agent。

它会引导 agent 完成：

- 检查并安装 `harness-lint` 和 `grit`
- 运行 `harness-lint init`
- 把 harness-lint 标识块写入用户仓库的 `AGENTS.md` 或 `CLAUDE.md`
- 根据项目语言和已有 agent 指令生成初始规则草稿

手动初始化也可以：

```sh
harness-lint init
```

这会创建：

```text
harness.toml
rules/
.harness/
```

提交 `harness.toml` 和 `rules/`，忽略 `.harness/`。

## 常用命令

```sh
harness-lint doctor
harness-lint check --changed
harness-lint check --staged
harness-lint check [paths...]
harness-lint check --all
harness-lint pack search "python typing"
harness-lint pack inspect python
harness-lint pack add <id> <source>
harness-lint pack update
harness-lint pack list
harness-lint rule suggest "<feedback>"
harness-lint rule suggest --local "<feedback>"
harness-lint rule new <id> <title> --language <language>
harness-lint rule list
harness-lint rule explain <rule-id>
```

当其他工具需要结构化输出时，可以对 `check`、`rule list` 和 `doctor` 使用 `--json`。

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

## Agent 工作流

当用户反馈或 code review 指向一个反复出现的代码质量问题时，不要只修当前实例。先创建或更新一条 `harness-lint` 规则，让 lint 能报告问题，再改代码直到 lint 通过。

推荐流程：

```sh
harness-lint rule suggest "<feedback>"
harness-lint check --changed
# 修代码或完善规则
harness-lint check --changed
```

如果 registry 里有合适规则包，先询问用户是否安装；如果没有，再创建本地规则草稿。
