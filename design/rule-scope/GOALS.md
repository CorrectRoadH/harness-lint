# 规则扫描范围设计：目标和边界

相关文档：[PLAN-1](./PLAN-1.md) · [PLAN-2](./PLAN-2.md) · [DECISION](./DECISION.md)

## 问题

有些规则需要检查生成代码，例如通过 `.pb.go` / `.pb.ts` 判断 proto 里是否生成了 `GetId()`。

但生成代码目录通常会放进 `[ignore]`，因为大多数业务规则不应该扫描生成代码。现在的问题是：

> 如何让一条专门规则扫描某个被 `[ignore]` 屏蔽的路径，同时不让其它规则也扫描那里？

这个设计只解决“某条规则扫描哪些文件”。它不改变规则是否启用，也不改变诊断是否被报告。

## 用户与用户旅程

### 我们的用户与现实约束

harness-lint 跑在一个仓库根上：配置发现从 cwd 向上走到 `.git` 或 `harness.toml`，**全仓只有一个 `harness.toml`**（`find_project_root`），目前没有 per-package 嵌套配置。因此本设计的任何字段都是**仓库全局**的。

典型用户是 monorepo：一个 `harness.toml` 管 `apps/backend`、`apps/web`、`packages/*`，每个子树各有自己的源码和生成代码（`apps/backend/gen/**`、`apps/web/src/gen/**`、`packages/proto/gen/**`）。同一个“生成代码”概念在仓库里**散落多处**。

规则**同时来自两个来源**，一个仓库里两者并存：

- **installed pack 规则**：项目在 `[packs]` 里装的共享规则（`github:` / registry，快照到 `.harness/packs/<id>/`）。**别人写的**，必须可移植，不能硬编码本仓库布局。
- **local 规则**：项目自己在 `[rules].local` 目录里写的规则。**项目自己写的**，知道布局。

“一条规则扫哪些文件”这件事，在这两类规则上**归属不同的人**——这是评估每个方案的核心。

### 用户旅程

下面每条旅程标注**谁动手、动几处、会不会静默坏**。方案优劣 = 它让这些旅程变顺还是变绕。

- **J1 onboarding（零配置预期）。** 新仓库装 `go-protobuf` 和 `no-console-log` 两个 pack，再写一条 local 规则。期望：扫普通源码的规则**零配置**就位；生成代码位置**只声明一次**。
- **J2 local gen-only 规则（动机场景）。** 团队写 `local.proto-no-id-getter`，要扫 `apps/backend/gen/**/*.pb.go`，而该目录在 `[ignore]` 里。衡量：改几个文件、规则本身能不能自说明“我冲 gen 来”。
- **J3 installed pack 冲着 gen 来（可移植性）。** 项目装的 pack 里有条 proto 规则要看 gen。**pack 作者不知道本仓库 gen 在哪**。要把“pack 想看 gen”接到“gen 在 `apps/backend/gen`”。衡量：pack 要不要硬编码路径、项目逐条授权还是一次定义、项目能不能拒绝。
- **J4 monorepo 散落的同概念目录。** `apps/backend/gen`、`services/foo/gen`、`packages/proto/gen` 三处都是生成代码，共一个 root `harness.toml`。衡量：同概念多路径，配置是定义一次还是随规则数 / 目录数膨胀。
- **J5 项目改写 installed 规则的范围。** 某条通用 pack 规则在本仓库扫多了或扫少了。项目想加 / 减一段路径，但**不 fork pack**（pack 是 `.harness/packs/` 下的快照，update 会覆盖）。衡量：能不能不碰 pack 就调范围、改动落在哪。
- **J6 目录搬家。** gen 从 `apps/backend/gen` 搬到 `apps/backend/internal/gen`。衡量：改几处、有没有东西静默失效（stale 路径健康检查要抓到）。
- **J7 主权（危险旅程）。** 一条粗心或恶意的 installed pack 规则想扫项目明确 `[ignore]` 的文件（secrets、vendored 代码）。衡量：installed 规则能不能在项目不知情下穿透 `[ignore]`。

### 把旅程当验收轴

判一个方案好不好，不是看字段优不优雅，而是看它在 J1–J7 上各让谁付出什么代价。没有方案能让七条全顺；每个方案都在某几条上买顺、在另几条上买绕。各 PLAN 末尾的“取舍了哪些用户旅程”就是逐条交代这笔账。

特别注意 **J3（两种来源并存）和 J4（monorepo 散落同概念）正是本节开头强调的两个现实约束**——它们最能区分“路径写在规则里”（PLAN-1/2/3）和“概念定义一次、规则订阅”（PLAN-4/5）两条路线。

## 非目标

- 不给 grit 增加 protobuf parser。
- 不允许规则扫描结构性排除路径，例如 `.git`、`.harness`、`node_modules`、`target`、规则目录，以及 `.gitignore` 排除的文件。
- 不引入任意脚本扩展，配置仍保持声明式。
- 不用 `[[exceptions]]` 解决扫描范围问题。`[[exceptions]]` 只隐藏已经产生的诊断，不减少扫描。

## 三阶段模型

harness-lint 的配置应按三个阶段理解：

| 阶段 | 负责什么 | 配置位置 |
|---|---|---|
| 启用阶段 | 哪些规则启用、禁用、改 severity | `harness.toml` 的 `[disabled]`、`[overrides]` |
| 扫描阶段 | 某条规则能看到哪些文件 | 规则的 `language`、新路径 scope、GritQL `$filename`，以及项目 `[ignore]` |
| 报告阶段 | 已产生的诊断是否显示 | `harness.toml` 的 `[[exceptions]]` |

本设计只改扫描阶段。

## 设计约束

1. **生成代码默认不可见。** `[ignore]` 里的生成代码仍然不被普通规则扫描。
2. **按规则 opt-in。** 只有显式声明的规则可以看到被忽略的目标路径。
3. **扫描和报告分离。** 不能用 `[[exceptions]]` 给错误的扫描范围补洞。
4. **结构排除不可突破。** 新能力不能覆盖 `.gitignore` 或内部目录排除。
5. **语义要像规则 scope。** 规则读者应能从规则文件本身看出它是不是 gen-only、test-only 或某目录 only。
6. **名字不能暗藏相反含义。** 字段名应让人知道它是限制扫描范围，而不是额外加几个路径。
7. **健康检查能发现死配置。** 如果路径 scope 指向不存在的具体前缀，应像 stale ignore / stale exception 一样被报告。

## 需要决定

1. 配置写在哪里：`harness.toml`，还是规则 frontmatter。
2. 字段叫什么：例如 `scan_only`、`applies_to`、`targets`。
3. 字段语义是什么：替换默认扫描集，还是在默认扫描集上加路径。
4. 是否自动绕过 `[ignore]`：如果路径 scope 指向 ignored 文件，它是否能扫描。
5. 与 `language`、`$filename`、`[[exceptions]]` 如何组合。
