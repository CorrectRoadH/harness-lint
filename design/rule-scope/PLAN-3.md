# 方案 3：把 scope 和 ignore-bypass 按规则来源分开授权

相关文档：[GOALS](./GOALS.md) · [PLAN-1](./PLAN-1.md) · [PLAN-2](./PLAN-2.md) · [DECISION](./DECISION.md)

Status: **Superseded**（正确诊断了 PLAN-2 的主权漏洞，但解法是“补丁修补丁”的授权阶梯；[DECISION](./DECISION.md) 采用 PLAN-5 从根上移除级联）

这个方案不是第三种独立机制。它接受 PLAN-2 的 `scan_only`,但修掉它的一个授权漏洞,并把 PLAN-1 收编成同一套模型里的“外部规则授权口”。

## 要解决的问题

PLAN-2 的 `scan_only` 一个字段干了两件语义不同的事:

1. **scope** —— 这条规则只扫这些路径(良性,规则作者本来就该决定)。
2. **ignore-bypass** —— 这条规则能看到被项目 `[ignore]` 屏蔽的文件(特权)。

PLAN-2 把这两件事绑在一起,等于:**只要任何规则声明了 `scan_only`,它就自动获得绕过项目 `[ignore]` 的权限,而项目方无法否决。**

对 local 规则无所谓——作者就是项目方。但对共享 / pack 规则,规则是别人写的:

> 一条 pack 规则写 `scan_only: ["**/*.go"]`,就能在所有安装它的项目里看到本该被 `[ignore]` 屏蔽的代码。项目的 `harness.toml` 里看不出这件事,也拦不住。

这违反 GOALS 约束 #1(生成代码默认不可见):默认实际上变成了“除非任意规则声明 `scan_only`,否则不可见”。DECISION 对此只有一句约定式防御(“共享 pack 规则不应硬编码项目私有路径”),不是强制。

## 核心原则

> `scan_only` 永远是 scope。**绕过 `[ignore]` 是另一回事,必须由项目授权。**

把扫描阶段拆成两层:

- **可见集(visible set)** = `current run set − [ignore] − 结构排除 − .gitignore`。这是项目主权决定的“这个仓库允许被扫的文件”。
- **规则 scope** = 规则用 `scan_only` / `language` / `$filename` 在可见集*之内*再收窄。

普通情况下,任何规则——不管 local 还是 pack——的 `scan_only` 都被 clamp 在可见集内,**够不到 ignored 路径**。

## 谁能突破可见集

突破 `[ignore]`(注意:`.gitignore` 和结构排除仍然永不可破)只授予两种情况:

### (a) 项目本地定义的规则:自由突破

local 规则(项目自己 rules 目录里的规则,如 `local.proto-no-id-getter`)的作者就是项目方。它的 `scan_only` 直接享受 PLAN-2 的语义:可以扫 ignored 路径,无需额外配置。

```markdown
---
id: local.proto-no-id-getter
language: go
scan_only: ["backend/gen/**/*.pb.go"]   # 本地规则,直接穿透 [ignore]
---
```

动机场景(local gen-only proto 规则)因此**和 PLAN-2 一样干净,只写一个字段**。

### (b) 外部 / pack 规则:需要项目显式 grant

pack 规则的 `scan_only` 默认被 clamp 在可见集内。如果项目确实想让某条 pack 规则扫 ignored 路径,必须在自己的 `harness.toml` 里显式开口——这就是 PLAN-1 的形状,被收编进来:

```toml
[ignore]
paths = ["backend/gen/**"]

[[scan_ignored]]
rule = "go-protobuf.require-id-getter"
paths = ["backend/gen/**"]
reason = "This shared rule may scan generated protobuf here; other rules still ignore backend/gen."
```

语义:对该 pack 规则,把这些 `[ignore]` 路径重新放进它的可见集;它的 `scan_only` / `$filename` 仍然继续收窄。其它规则看不到。

## 完整优先级阶梯

```text
1. 结构排除 + .gitignore                      ← 最高,任何字段不可破
2. current run set (--changed/--staged/--all)
3. 可见集 = run set − [ignore]
       + 若规则是 local 且有 scan_only:    把 scan_only 命中的 ignored 路径放回
       + 若规则在 [[scan_ignored]] 中:      把该条目的 ignored 路径放回
4. ∩ scan_only(若有)
5. ∩ language ∩ $filename
```

单调、可讲、每一步都说得清“谁授权的”。

## 健康检查

- `scan_only` 自身校验仍按 PLAN-2:空列表错误、glob 语法、stale 路径前缀、结构排除路径——锚到规则文件。
- 新增 warn(锚到 `harness.toml` 或规则文件):某规则的 `scan_only` 字面前缀落在 `[ignore]` 覆盖范围内时,提示“此规则将绕过 ignore 扫描 X”。把隐式 bypass 变可见。
- `[[scan_ignored]]` 的 stale 路径检查复用现有 `harness.stale-ignore-path` 同款逻辑。
- 若一条 **pack** 规则的 `scan_only` 命中了 ignored 路径但**没有**对应的 `[[scan_ignored]]` grant:不是错误,而是按 (b) clamp 掉那部分,并 warn 提示“该 pack 规则想扫 ignored 路径,但项目未授权,已忽略”。

## 与现有机制的组合(三阶段模型)

1. **启用阶段。** `[disabled]` / `[overrides]` 不变。
2. **扫描阶段。** 先算可见集(含 local 自由突破和 `[[scan_ignored]]` grant),再用 `scan_only` / `language` / `$filename` 收窄。
3. **报告阶段。** `[[exceptions]]` 仍只隐藏诊断,不改扫描范围。

## 优点

- 修掉 PLAN-2 的主权漏洞:**项目的 `[ignore]` 不再能被导入的代码静默穿透。**
- local gen-only 场景的体验**和 PLAN-2 完全一样**,不加 ceremony。
- PLAN-1 不再是“未来也许要”的弃案,而是 (b) 这一格的自然解,且只在真正需要时出现。
- 授权边界 = 规则来源(local vs pack),这个区分 harness-lint 已经存在,可强制、可落地,不是新概念。
- 所有突破都可见:local 在规则文件里,pack 在 `harness.toml` 里,doctor 还会主动 warn。

## 代价

- 引入“可见集 vs 规则 scope”两层心智模型,文档要讲清楚。
- 需要在扫描阶段知道规则是 local 还是 pack(来源信息)。
- `.gitignore` 仍不可破:若 gen 代码是 `.gitignore` 而非 `[ignore]`,本方案和 PLAN-1/2 一样都够不到——这个前提需要在 GOALS 写明,不属于本方案能解决的范围。

## 和 DECISION 的关系

DECISION 当前选 PLAN-2、把 PLAN-1 推到“未来再说”。本方案建议把 DECISION 改成:

> 采用 PLAN-2 的 `scan_only` 语义,**但 ignore-bypass 按规则来源授权**:local 规则自由突破,pack 规则需 `[[scan_ignored]]` 显式 grant。

即在 DECISION 增补一节“授权边界”,指向本文件。这样既保留 PLAN-2 的人体工学,又不在第一版就 ship 主权漏洞。

## 取舍了哪些用户旅程

参见 [GOALS](./GOALS.md#用户旅程) J1–J7。本方案 = J2 取 PLAN-2 的体验、J3 取 PLAN-1 的体验,代价是用户要同时学两套。

| 旅程 | 体验 | 原因 |
|---|---|---|
| J1 onboarding | 顺 | 同 PLAN-2 |
| J2 local gen-only | **最佳** | local 规则的 `scan_only` 直接穿透,和 PLAN-2 完全一样,一个字段 |
| J3 pack 冲 gen | 顺(但有仪式) | pack 的 `scan_only` 被 clamp,项目用 `[[scan_ignored]]` 显式 grant——即 PLAN-1 的形状 |
| J4 monorepo 散落 | **绕** | local 规则仍各自重复散落路径(继承 PLAN-2 的病);pack 侧继承 PLAN-1 的逐条授权 verbose |
| J5 改写 pack 范围 | 顺 | 通过 grant 加路径,不碰 pack |
| J6 目录搬家 | **绕** | local 规则重复路径都要改;pack 侧改 grant |
| J7 主权 | **修好** | pack 不经 grant 够不到 ignored,项目重获否决权 |

**买到的旅程**:J2 + J3 + J7——既保住 local 单字段的爽,又堵上主权漏洞。
**牺牲的旅程**:J4 没改善(路径仍随规则数重复),且新增一条**隐形的来源依赖**——同样写 `scan_only`,local 规则能穿透、pack 规则不能,而这个差别**在规则文件里看不出来**,要靠读者知道规则来源。它用“再加一层授权阶梯”换主权,补丁修补丁,这正是 PLAN-4 要移除的级联。
