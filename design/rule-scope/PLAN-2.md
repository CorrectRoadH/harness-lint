# 方案 2：规则声明 exclusive path scope

相关文档：[GOALS](./GOALS.md) · [PLAN-1](./PLAN-1.md) · [DECISION](./DECISION.md)

Status: **Superseded**（曾被早先决策采用；现因 J3/J4 退化与 J7 主权漏洞被 [DECISION](./DECISION.md) 改判。未来或作为 file sets 之上的语法糖补回）

这个方案把“这条规则只扫描哪些路径”写进规则 frontmatter。它把问题当成规则 scope：

```markdown
---
id: local.proto-no-id-getter
title: Proto Message Must Generate GetId
language: go
scan_only: ["backend/gen/**/*.pb.go"]
---
```

## 字段语义

`scan_only` 表示：

> 这条规则只扫描这些 repo-relative glob 命中的文件。

完整扫描集：

```text
rule files =
  current run set
  ∩ scan_only globs
  ∩ language match
  ∩ GritQL $filename predicate
```

`current run set` 指本次命令原本选择的文件集合：`--changed` 只看 changed files，`--staged` 只看 staged files，`--all` 才看全仓。`scan_only` 不会把一次 changed check 变成全仓扫描。

如果规则没有 `scan_only`，行为保持现状：

```text
rule files =
  current run set
  - [ignore].paths
  ∩ language match
  ∩ GritQL $filename predicate
```

结构排除和 `.gitignore` 总是在最前面生效，`scan_only` 也不能覆盖它们。

## 与 `[ignore]` 的关系

带 `scan_only` 的规则不再使用默认扫描集，因此 `[ignore]` 不参与这条规则的扫描范围计算。

这意味着：如果 `scan_only` 命中了 `[ignore]` 里的路径，这条规则可以扫描它；其它没有 `scan_only` 的规则仍然看不到那里。

这个绕过是有边界的：

- 它不是“额外加路径”，而是“只扫这些路径”。
- 它只影响声明该字段的单条规则。
- 它不能突破结构排除或 `.gitignore`。
- 它仍会被 `language` 和 `$filename` 继续收窄。

## 为什么叫 `scan_only`

`applies_to` 更顺，但容易被理解成“也适用于这些路径”。这里的真实语义是 exclusive scope，也就是“只扫这些路径”。

因此推荐 `scan_only`。它不优雅，但明确。

备选：

| 名字 | 问题 |
|---|---|
| `applies_to` | 太像普通描述，容易漏掉“only”语义 |
| `targets` | 不说明是扫描范围还是诊断目标 |
| `paths` / `files` | 太泛，和其它配置混淆 |
| `unignore` / `scan_ignored` | 暴露实现机制，且容易变成项目 policy |

## 格式

- 值是非空 glob 列表。
- glob 相对 repo 根。
- 语法与 `[ignore].paths` 一致。
- 只支持 include，不支持 exclude。
- 空列表是错误。
- 如果需要进一步排除，优先写进 GritQL `$filename`。
- 如果只是要隐藏某些诊断，使用 `[[exceptions]]`，但那不改变扫描范围。

## 健康检查

`scan_only` 是规则 frontmatter 的一部分，所以校验也应锚到规则文件，而不是 `harness.toml`。

应该检查：

- glob 语法是否合法；
- 列表是否为空；
- 非 leading-glob 的路径前缀是否存在；
- 是否指向结构排除路径。

这些检查属于 rule health / doctor，不属于 report-stage `[[exceptions]]`。

## 适用范围

这个能力主要给 local / project-specific 规则使用。例如：

- generated protobuf 检查；
- 某目录专用架构规则；
- migration 文件专用规则；
- 测试 fixture 专用规则。

共享 pack 规则不应硬编码项目私有路径。如果未来需要“项目让某个共享规则额外扫描 ignored 路径”，那是方案 1 的问题，不应塞进 `scan_only`。

## 优点

- 规则文件自说明：读规则时就知道它是不是 gen-only。
- 生成代码默认仍然不可见，只有声明的规则能看见。
- 对当前需求表达准确：proto/gen 规则本来就是只扫 gen，不是全仓规则。
- 不需要新增 include/exclude 小语言。
- 和 `language`、`$filename` 一样属于规则扫描范围，而不是报告抑制。

## 代价

- 不能表达“默认范围 + ignored 额外路径”的加法需求。
- `scan_only` 同时承担了 scope 和 bounded ignore bypass，需要文档说清楚。
- 如果共享规则也想用项目私有路径，需要另一个项目级配置，而不是滥用这个字段。

## 取舍了哪些用户旅程

参见 [GOALS](./GOALS.md#用户旅程) J1–J7。

| 旅程 | 体验 | 原因 |
|---|---|---|
| J1 onboarding | 顺 | 普通规则零配置；gen 规则只加一个字段 |
| J2 local gen-only | **最佳** | 一个字段、一个文件、规则自说明。动机场景的最优解 |
| J3 pack 冲 gen | **绕到不可用** | pack 作者要把本仓库 gen 路径写进 `scan_only`——他根本不知道。pack 想用只能改 `.harness/packs/` 下的快照，update 即丢 |
| J4 monorepo 散落 | **绕** | 每条规则各自重复列三处 gen 的深路径；gen 搬家要改每一条带 `scan_only` 的规则 |
| J5 改写 pack 范围 | **绕** | 改 pack 规则范围 = 改快照文件，脆弱且会被 update 覆盖 |
| J6 目录搬家 | **绕** | gen 搬家 → 改每一条 `scan_only` |
| J7 主权 | **漏洞** | installed pack 规则声明 `scan_only` 即可静默穿透项目 `[ignore]`，项目无否决权（PLAN-3 要修的正是这个） |

**买到的旅程**：J1、J2——单条 local gen 规则的体验全场最佳，落地最快。
**牺牲的旅程**：J3、J4、J7——把“扫哪里”的路径知识锁死在规则里，于是跨来源（pack）和散落多路径（monorepo）全都退化，还在 J7 上开了主权漏洞。它优化的是 N=1 的 local 场景，恰好不是 GOALS 强调的两个现实约束。
