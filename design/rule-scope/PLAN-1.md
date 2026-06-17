# 方案 1：项目配置打洞

相关文档：[GOALS](./GOALS.md) · [PLAN-2](./PLAN-2.md) · [DECISION](./DECISION.md)

Status: **Superseded**（被 [DECISION](./DECISION.md) 否决；级联前提见 [PLAN-4](./PLAN-4.md)。加法授权场景由 PLAN-5 的 `runs_on: ["default", ...]` 覆盖）

这个方案把“哪条规则可以额外扫描 ignored 路径”写在 `harness.toml`。它把问题当成项目 policy：

```toml
[ignore]
paths = ["backend/gen/**"]

[[scan_ignored]]
rule = "local.proto-no-id-getter"
paths = ["backend/gen/**"]
reason = "This rule checks generated protobuf Go; other rules still ignore backend/gen."
```

## 语义

对指定规则，在默认扫描集之外额外加入这些路径：

```text
rule files = normal files minus [ignore] + [[scan_ignored]].paths for this rule
```

结构排除和 `.gitignore` 仍然不可突破。

## 优点

- 项目 policy 集中在 `harness.toml`，谁能突破 `[ignore]` 一眼可见。
- 适合“共享 pack 规则 + 项目想让它额外扫某些 ignored 路径”的场景。
- 能表达真正的加法需求：正常文件也扫，某些 ignored 文件也扫。

## 问题

- 它把一条规则的扫描范围拆成两处：`language` / `$filename` 在规则里，额外路径在 `harness.toml`。
- 它和 `[ignore]` 有优先级关系，必须解释“为什么这个字段能盖过 ignore”。
- 对当前 gen-only 场景来说，它表达的是“默认范围 + gen”，但真实需求是“只扫 gen”。
- 规则文件本身看不出自己依赖生成代码目录。
- 配置名容易偏机制，比如 `scan_ignored`、`unignore`，读起来不如路径 scope 自然。

## 适合什么时候

如果未来出现这种需求，再考虑它：

> 一个共享规则默认扫普通源码，但某个项目希望它额外扫描被 `[ignore]` 屏蔽的一小段路径。

当前 proto/gen 场景不是这个形状，所以不建议优先采用。

## 取舍了哪些用户旅程

参见 [GOALS](./GOALS.md#用户旅程) J1–J7。

| 旅程 | 体验 | 原因 |
|---|---|---|
| J1 onboarding | 顺 | 普通规则零配置；但要让规则看 gen，需为它加一条 `[[scan_ignored]]`，多一步 |
| J2 local gen-only | **绕** | 规则范围被拆到两个文件：规则在 `rules/`，路径授权在 `harness.toml`。规则文件自己看不出依赖 gen，且表达的是“默认 + gen”而非“只 gen” |
| J3 pack 冲 gen | **顺（主场）** | 项目按 rule id 在自己的 `harness.toml` 里集中授权，pack 不碰路径，项目保留否决权 |
| J4 monorepo 散落 | **绕** | 每条规则 × 每个 gen 目录都要进 `[[scan_ignored]]`；N 条规则重复列同一批散落路径 |
| J5 改写 pack 范围 | 顺（只加不减） | 能集中给 pack 规则加路径；但只能 include，减范围要靠 `[[exceptions]]` |
| J6 目录搬家 | 绕 | 要改 `[ignore]` 和每一条列了它的 `[[scan_ignored]]` |
| J7 主权 | **安全** | 只有项目写的授权能穿透，pack 无法自授权 |

**买到的旅程**：J3、J7——“别人的 pack 想多扫一段，项目集中授权且可否决”。
**牺牲的旅程**：J2、J4——动机场景被拆成两文件，monorepo 下随规则数膨胀。它优化的是 pack 加法授权，不是 local gen-only 的简洁。
