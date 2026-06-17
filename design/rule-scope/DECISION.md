# 决策：采用 file sets（PLAN-5），按 owner 拆分扫描范围

相关文档：[GOALS](./GOALS.md) · [PLAN-1](./PLAN-1.md) · [PLAN-2](./PLAN-2.md) · [PLAN-3](./PLAN-3.md) · [PLAN-4](./PLAN-4.md) · [PLAN-5](./PLAN-5.md)

Status: **Proposed（取代早先的 “采用 PLAN-2 / scan_only” 决策）**

## 决策

采用 [PLAN-5](./PLAN-5.md) 的 **file sets + `runs_on`** 模型，吸收 [PLAN-4](./PLAN-4.md) 的 owner 拆分洞察：

- 项目在 `harness.toml` 用 `[file_sets.*]` 命名“多数规则不该看、个别规则需要”的文件区域（**布局知识，项目拥有，用路径表达**）。
- 规则用 `runs_on` 声明自己跑在哪些区域（**身份知识，规则/pack 作者拥有，用可移植的区域名表达，绝不含项目路径**）。
- 无 `runs_on` 的规则跑 `default`；`[ignore]` 收窄为“任何规则都不扫”。

[PLAN-1](./PLAN-1.md)、[PLAN-2](./PLAN-2.md)、[PLAN-3](./PLAN-3.md) 标记为 **Superseded**：三者共享“把 `[ignore]` 当全局减法，再逐规则打洞”的前提，会级联（见 PLAN-4 病根分析）。

```toml
# harness.toml —— 区域定义一次
[file_sets.generated]
paths = ["apps/backend/gen/**/*.pb.go", "services/*/gen/**", "packages/proto/gen/**"]
default_rules = false
provides = ["generated"]     # 见下文“解决命名表风险”
```

```markdown
---
id: local.proto-no-id-getter
language: go
runs_on: ["generated"]
---
```

## 为什么从 PLAN-2 改判

早先的决策按“一个 local 规则扫 gen”这单条旅程选了 PLAN-2，那时还没写明用户画像。[GOALS](./GOALS.md#用户旅程) 把用户落实成两个**并存的现实约束**后，账变了：

1. **用户同时装线上 pack 又自己写规则（J3）。** pack 作者不知道本仓库 gen 在哪，`scan_only` 这种“路径写进规则”的字段对 pack 根本不可用——他没法硬编码你的布局。
2. **用户是 monorepo，同一个 `generated` 概念散落多处（J4）。** 全仓只有一个 `harness.toml`、无 per-package 配置，把散落路径塞进每条规则会随规则数 × 目录数膨胀。

把七条旅程拉成统一标尺横向比（各 PLAN 末尾的表），结论清楚：

| 旅程 | PLAN-2 | PLAN-3 | **PLAN-5（采用）** |
|---|---|---|---|
| J1 onboarding | 顺 | 顺 | 顺（有迁移成本） |
| J2 local gen-only | **最佳** | **最佳** | 中（多一个文件） |
| J3 pack 冲 gen | 不可用 | 顺（有仪式） | **最佳** |
| J4 monorepo 散落 | 绕 | 绕 | **最佳** |
| J5 改写 pack 范围 | 绕 | 顺 | 顺 |
| J6 目录搬家 | 绕 | 绕 | **最佳** |
| J7 主权 | **漏洞** | 修好 | 安全 |

PLAN-2 只在 J2 单条上最优，却在用户明确强调的 J3 / J4 上退化，并在 J7 开主权漏洞。PLAN-5 在 J3 / J4 / J6 / J7 上一致最佳——**正是用户画像所在的四条**。

## 接受的代价

PLAN-5 不是免费的，明确接受两笔：

1. **J2 比 PLAN-2 贵一个文件。** 单条 local gen 规则要建 file set + 写 `runs_on`，而非一个字段。我们判定这笔代价值得：换来的是 J3/J4 的可移植与定义一次。
2. **“`default_rules = false` 静默从 default 减扫”这个魔法。** 文档必须重点讲，doctor 要在定义新的 default-closed 区域时提示它会从哪些规则手里拿走文件。

不接受的代价：让用户为 J2 的省事付出 J7 的主权漏洞（PLAN-2）或一条隐形的 local/pack 来源依赖（PLAN-3）。

## 解决残留风险：命名表与别名

PLAN-4/5 的阿喀琉斯之踵是：`runs_on` 引用的区域名是 pack 与项目之间**不可强制**的约定。pack 写 `runs_on: ["generated"]`，项目却把区域叫 `[file_sets.gen]` 就对不上。三层防御：

1. **不做二进制魔法，靠 pack `INSTALL.md` + 安装期 AI 接线。** 不在二进制里内建 `DO NOT EDIT` 头自识别（那是语言相关、易错、且把“静默减扫”藏进二进制）。改为：pack 在自己的 `INSTALL.md`（或 manifest 的说明段）里声明“本 pack 的规则 `runs_on` 哪些概念（如 `generated`），请确保项目有一个 `[file_sets.*] provides=[...]` 指向你的生成代码”。安装该 pack 时，经 harness-lint skill 的 AI 读 `INSTALL.md`，把对应的 `[file_sets.*]` 写进项目 `harness.toml`。一切**显式、写在 `harness.toml`、人可审查**，没有藏在二进制里的减扫。`generated` 只是一个**推荐概念名（约定）**，不是内建检测。

2. **file set 用 `provides` 声明它满足哪些概念。** 别名方向是“项目把本地区域映射到 pack 期望的概念名”，而不是改名：

   ```toml
   [file_sets.codegen]            # 项目爱叫什么叫什么
   paths = ["apps/**/gen/**"]
   default_rules = false
   provides = ["generated", "proto"]   # 它满足这两个 pack 概念
   ```

   pack 规则 `runs_on: ["generated"]` 因此命中 `codegen` 区域。pack 保持可移植，项目保留命名自由，映射这件事**owner 是项目**（它既懂布局也懂自己装了哪些 pack）。

3. **未被 provide 的概念 = 响亮报错。** 规则 `runs_on` 一个既不是已定义 file set 名、也没被任何 `provides` 满足的概念 → error，锚到规则文件：

   > rule `go-protobuf.require-id-getter` runs_on `generated`, but no file set provides it. Add `[file_sets.<name>] provides = ["generated"]`.

   抓拼写错误，也抓“装了 pack 但没接好概念”。沿用现有 stale-path 健康检查的“响亮失败”风格。

这样把不可强制的约定变成：pack 用 `INSTALL.md` 声明所需概念、安装期 AI 照着接线、`provides` 显式映射、缺失则响亮报错——全程显式可审查，不靠二进制里的隐式魔法。

## 规范语义

```text
rule files =
  current run set (--changed / --staged / --all)
  − 结构排除 + .gitignore           ← 最高，任何字段不可破
  − [ignore].paths                  ← 任何规则都不扫
  ∩ ⋃( runs_on 解析出的 file sets )  ← 并集，无 specificity；无 runs_on ⇒ {default}
  ∩ language match
  ∩ GritQL $filename predicate

default = visible files − ⋃( default_rules=false 的 file sets )
```

- `runs_on` 是并集，满足交换律/结合律 ⇒ 无顺序依赖、无“谁盖过谁”，不需要 PLAN-3 的 local/pack 授权阶梯。
- 规则来源（local vs pack）**不影响扫描语义**：主权靠“pack 看不到未订阅区域”结构性保住，不靠按来源授权。
- `[ignore]` 与 `[file_sets.*]` 路径重叠 ⇒ 报错（语义冲突：前者说没人扫，后者说订阅者可扫）。

## 健康检查

- `[file_sets.*].paths` 为空、glob 非法、非 leading-glob 前缀不存在（stale）、命中结构排除/`.gitignore`/`[ignore]` → 复用 `harness.stale-ignore-path` 同款逻辑，锚到 `harness.toml`。
- 规则 `runs_on` 引用不存在且无 `provides` 满足的区域/概念 → error，锚到规则文件。
- 规则 `runs_on` 为空列表 → error。
- file set 定义了但无规则订阅且 `default_rules=false` → info/warn（可能死配置）。
- 定义新的 `default_rules=false` 区域时，doctor 提示它将从哪些 default 规则手里移除文件（让“静默减扫”可见）。

## 落地分期

1. **先落 file sets + `runs_on` + `provides` + `[ignore]` 收窄语义**，覆盖 J1–J4 主路径。
2. **再落 pack `INSTALL.md` 约定 + skill 安装期接线**，消化命名表风险。
3. **健康检查与 doctor 提示**随上述一并补齐。

> 纯 local、单规则、不装 pack 的项目，PLAN-2 的 `scan_only` 确实更省一个文件。但这不是 GOALS 写明的用户画像；为不存在的简单场景牺牲 J3/J4/J7 不划算。若未来确认有大量这类纯 local 用户，再考虑把 `scan_only` 作为 file sets 之上的语法糖补回，而非反过来。

## 需要落地的文档要求

- README 配置组合说明采用 [GOALS](./GOALS.md) 的三阶段模型，扫描阶段以 file sets + `runs_on` 为中心。
- 规则作者指南：需要限定区域时用 `runs_on`；需要细粒度语法判断时用 `$filename`；pack 规则只用可移植区域名/概念，绝不写项目路径。
- `[ignore]` 文档明确它是“任何规则都不扫”，与 `default_rules=false` 的 file set 区分。
- `[[exceptions]]` 文档明确：它只隐藏诊断，不改扫描范围。
- 提供推荐概念词表（至少 `generated`）；pack 作者指南说明用 `INSTALL.md` 声明所需概念，skill 文档说明安装期如何把概念接到项目 `[file_sets.*]`。
