# 方案 5：file sets —— 先给文件分区，再让规则声明能跑在哪些区

相关文档：[GOALS](./GOALS.md) · [PLAN-1](./PLAN-1.md) · [PLAN-2](./PLAN-2.md) · [PLAN-3](./PLAN-3.md) · [PLAN-4](./PLAN-4.md) · [DECISION](./DECISION.md)

Status: **Adopted**（见 [DECISION](./DECISION.md)；命名表/别名残留风险在 DECISION 用内建 `generated` 自识别 + `provides` 别名解决）

这版把问题从“某条规则如何穿透 `[ignore]`”改成：

> 项目里有哪些文件区域？普通规则默认跑哪里？哪些规则能跑特殊区域？

这更接近用户心智。用户通常不是先想到“规则 bypass ignore”，而是先知道：

- `src/**` 是普通源码；
- `backend/gen/**` 是生成代码；
- `dist/**` 是构建产物，永远别扫；
- `migrations/**` 可能需要专门规则。

## 核心模型

先把文件分成几类：

```toml
[ignore]
paths = ["dist/**", ".harness/**"]

[file_sets.generated]
paths = ["backend/gen/**"]
default_rules = false
```

含义：

- `[ignore]`：永远不扫。任何规则都不能进入。
- `[file_sets.generated]`：这是一个可被规则选择的文件区域。
- `default_rules = false`：普通规则默认不扫这个区域。

规则声明自己能跑在哪些 file set：

```yaml
runs_on: ["generated"]
```

没有声明的规则等价于：

```yaml
runs_on: ["default"]
```

其中：

```text
default = visible files - all file_sets with default_rules = false
```

也就是说，项目只需要定义一次 `generated` 在哪里。大多数规则什么都不用写，它们默认跑 `default`，自然看不到 generated。只有专门规则才声明 `runs_on: ["generated"]`。

## 为什么不用 `[ignore]` 表示 generated

`[ignore]` 应该只表示“任何规则都不该扫”。

但 generated 的真实语义通常是：

> 大多数规则不该扫，少数 generated 专用规则要扫。

这不是 ignore，而是一个 default-closed file set。

因此：

```toml
# 不推荐
[ignore]
paths = ["backend/gen/**"]
```

应该改成：

```toml
[file_sets.generated]
paths = ["backend/gen/**"]
default_rules = false
```

如果同一路径同时命中 `[ignore]` 和某个 `[file_sets.*]`，应该报配置错误，因为语义冲突：

- `[ignore]` 说“没人能扫”；
- `file_sets` 说“订阅它的规则能扫”。

## 规则语义

规则 frontmatter：

```markdown
---
id: go-protobuf.require-id-getter
title: Proto Message Must Generate GetId
language: go
runs_on: ["generated"]
---
```

完整扫描集：

```text
rule files =
  current run set
  - structural exclusions
  - .gitignore
  - [ignore].paths
  ∩ union(file sets named by runs_on)
  ∩ language match
  ∩ GritQL $filename predicate
```

无 `runs_on`：

```text
runs_on = ["default"]
```

需要同时跑普通源码和生成代码：

```yaml
runs_on: ["default", "generated"]
```

## file set 语义

```toml
[file_sets.generated]
paths = ["backend/gen/**"]
default_rules = false
```

字段：

- `paths`：repo-relative glob 列表，非空。
- `default_rules`：默认规则是否扫描该区域。
  - `false`：从 `default` 中移除，只有显式 `runs_on` 的规则能扫。
  - `true`：仍属于 default，同时也提供一个可命名区域。

`default_rules = false` 是 generated 的典型配置。

`default_rules = true` 用于只是想给某片普通源码起名字，方便少数规则额外声明范围；它不改变普通规则行为。

## 和 PLAN-4 的关系

PLAN-4 的 `domains` 抽象本质上是这个模型的更理论版本：

- `domains.generated` ≈ `file_sets.generated`;
- `applies_to` ≈ `runs_on`;
- default 域 ≈ 默认 file set。

本方案更偏用户语言：

- 项目配置的是“文件区域”，不是“概念域”。
- 规则说“我跑在哪些区域”，不是“我订阅哪些概念”。
- `[ignore]` 从“普通规则别扫”收窄为“任何规则别扫”。

如果要落地，我更倾向使用 PLAN-5 的命名和文档叙述，吸收 PLAN-4 的 owner 拆分思想。

## 四个典型场景

### A. local proto 规则查 generated code

```toml
[file_sets.generated]
paths = ["backend/gen/**/*.pb.go"]
default_rules = false
```

```yaml
runs_on: ["generated"]
```

项目定义 generated 在哪；规则声明自己跑 generated。普通规则不受影响。

### B. installed proto pack

pack 规则可以自带：

```yaml
runs_on: ["generated"]
```

项目只需要定义：

```toml
[file_sets.generated]
paths = ["backend/gen/**/*.pb.go"]
default_rules = false
```

pack 作者不硬编码项目路径，项目也不逐条授权规则。

如果项目没有定义 `generated`，安装该 pack 后应报清楚的错误或 warning：

> rule `go-protobuf.require-id-getter` runs_on `generated`, but no `[file_sets.generated]` is configured.

### C. installed 通用规则

无 `runs_on`，默认 `runs_on: ["default"]`。

它不会扫描 `default_rules = false` 的 generated 区域。

### D. 某规则需要同时扫普通源码和特殊区域

```yaml
runs_on: ["default", "generated"]
```

这覆盖了 PLAN-1 的 additive 需求，但不需要 `[[scan_ignored]]`。

## 健康检查

应该检查：

- `[file_sets.*].paths` 为空：error。
- file set glob 非法：error。
- file set 的非 leading-glob 前缀不存在：warn。
- file set 路径命中结构排除、`.gitignore` 或 `[ignore]`：error 或 warn，建议 error。
- 规则 `runs_on` 引用不存在的 file set：error。
- file set 定义了但没有规则引用，且 `default_rules = false`：info/warn。
- `[ignore]` 和 `[file_sets.*]` 路径重叠：error。

## 取舍

### 优点

- 更符合用户心智：先描述目录/文件区域，再描述规则能跑哪里。
- 大多数规则不用改。无 `runs_on` 就只跑 `default`。
- generated 不再滥用 `[ignore]`。它是“默认不扫，但专门规则可扫”的 file set。
- installed pack 更可移植。pack 写 `runs_on: ["generated"]`，项目写 generated 路径。
- 支持 additive 场景：`runs_on: ["default", "generated"]`。
- 不需要 local/pack 特权阶梯。规则来源不影响扫描语义。

### 代价

- 这是新模型，不是一个小字段。文档、迁移和配置解释成本高于 PLAN-2。
- 用户需要理解：定义 `default_rules = false` 的 file set 会把这些文件从 default 中拿走。
- 需要一套推荐 file set 名称，尤其是 `generated`，否则 pack 之间可能各叫各的。
- 如果当前目标只是 local proto 规则扫 gen，PLAN-2 的 `scan_only` 更快落地。

## 我建议的方向

如果 harness-lint 只想短期修 local gen 规则，选 PLAN-2。

如果 harness-lint 要认真支持共享 rule packs，选 PLAN-5 作为长期模型：

> `[ignore]` = 永远不扫；`file_sets` = 文件区域；无 `runs_on` 的规则跑 default；特殊规则声明自己跑特殊区域。

我会把 PLAN-4 的核心洞察保留下来，但用 PLAN-5 的名字和叙述方式落地。它少一点架构术语，多一点“这个目录是什么、哪些规则能来”的直觉。

## 取舍了哪些用户旅程

参见 [GOALS](./GOALS.md#用户旅程) J1–J7。旅程账与 PLAN-4 几乎一致（同一套白名单分区 + owner 拆分），差别只在叙述语言（“文件区域 / `runs_on`”而非“概念域 / `applies_to`”）和迁移成本。

| 旅程 | 体验 | 原因 |
|---|---|---|
| J1 onboarding | 顺（但有迁移成本） | 普通规则零配置；但现存项目要把 gen 从 `[ignore]` 搬进 `[file_sets.X] default_rules=false`，语义从“谁都别扫”变“订阅者可扫”，是一次破坏性重分类 |
| J2 local gen-only | **中（比 PLAN-2 贵）** | 建 file set + 规则 `runs_on`（两文件）；`default_rules=false` 是个静默减法动作 |
| J3 pack 冲 gen | **最佳** | pack 写 `runs_on: ["generated"]`，项目写 `[file_sets.generated]` 路径，按区域名自动绑定，不逐条授权 |
| J4 monorepo 散落 | **最佳** | 一个 `[file_sets.generated]` 列三处散落路径，所有 `runs_on:["generated"]` 规则一并命中 |
| J5 改写 pack 范围 | 顺 | 加路径 = 改 file set；额外区域 = `runs_on:["default","generated"]`。不碰 pack |
| J6 目录搬家 | **最佳** | 改一处 file set；stale 检查锚到 `harness.toml` |
| J7 主权 | **安全** | file set 须显式订阅；`[ignore]` = 任何规则都不扫；二者路径重叠报错 |

**买到的旅程**：与 PLAN-4 相同——J3、J4、J6、J7，用“文件区域”这个更贴近用户心智的词承载同样的可移植性与主权保证。
**牺牲的旅程**：J2 仍比 PLAN-2 贵一个文件；并比 PLAN-4 多一笔 **J1 迁移账**——`[ignore]` 与 `file_sets` 不能共存，现存 gen 配置必须逐条搬迁。
**残留风险**：同 PLAN-4——区域命名表不可强制、无别名；`default_rules` 这个布尔名同样藏着“静默从 default 减扫”的副作用，和它批评 PLAN-2 的 `scan_only` 是同类毛病。
