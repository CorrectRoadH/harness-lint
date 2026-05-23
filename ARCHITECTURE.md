# harness-lint Architecture

harness-lint 是一个围绕 GritQL 的规则生态与规则创作工具。它不重新实现 lint 后端，而是解决 GritQL 规则如何被安装、更新、组合、解释、测试，以及用户反馈如何沉淀为本地规则的问题。

## 目标

项目的核心目标是让用户把个人或团队的编码偏好变成确定性的 lint 规则，尤其是面向 AI coding 的长期反馈闭环。

典型场景：

- 用户不希望 AI 只在当前回答里修一次代码，而是把偏好沉淀为长期规则。
- 用户希望从 `CLAUDE.md`、`AGENTS.md`、`.cursor/rules` 等文档中抽取已有规范。
- 用户希望安装别人维护的 GritQL 规则包，并能安全更新。
- 用户希望本地规则可读、可 review、可测试、可提交进 git。
- 用户希望 lint 很快，能基于 git 增量运行。

## 非目标

- 不重新实现 GritQL。
- 不重新实现通用 AST parser。
- 不默认使用 LLM 执行 lint。
- 不把不可验证的自然语言规范直接作为阻塞性检查。
- 不把外部规则包和用户本地规则混在一起。

## 总体结构

```text
harness-lint
├── CLI
├── Config Loader
├── Rule Source
├── Rule Parser
├── Rule Compiler
├── Grit Adapter
├── Incremental Runner
├── Cache
├── Reporter
├── Rule Registry
└── Rule Authoring
```

数据流：

```text
harness.toml
  -> Config Loader
  -> Rule Source
  -> Rule Parser
  -> Rule Compiler
  -> Grit Adapter
  -> Diagnostics
  -> Reporter
```

用户反馈到规则的数据流：

```text
feedback text
  -> Project language/library detection
  -> Rule Registry search
  -> existing rule pack candidate
  -> install prompt
  -> RuleDraft fallback
  -> harness/rules/local/*.md
  -> Rule Parser
  -> Rule Test
  -> warn / enforced rule
```

## 文件布局

项目使用以下目录协议：

```text
project-root/
├── harness.toml
├── harness.lock
├── harness/
│   └── rules/
│       └── local/
│           └── *.md
└── .harness/
    ├── packs/
    ├── generated/
    │   └── grit/
    │       ├── grit.yaml
    │       └── patterns/
    └── cache/
```

`harness/` 是用户资产，应该提交进 git。

`.harness/` 是工具工作目录，默认不提交。它保存下载的规则包、生成给 Grit 的配置和缓存。

## 配置文件

`harness.toml` 描述项目启用哪些规则包、本地规则在哪里、如何覆盖规则级别，以及增量检查策略。

示例：

```toml
[project]
name = "example"

[lint]
default_level = "warn"
changed_base = "origin/main"
cache = true

[rules]
local = ["harness/rules/local"]

[packs]
python = "github:harness-lint/rules-python@1.2.0"
ai = "github:harness-lint/rules-ai-style@0.3.0"
obsidian = "github:harness-lint/rules-obsidian@0.1.0"

[overrides]
"python.prefer-pydantic" = "error"
"ai.no-unrequested-refactor" = "warn"

[disabled]
rules = ["python.no-print-debug"]

[ignore]
paths = [
  "dist/**",
  "node_modules/**",
  ".venv/**"
]
```

`harness.lock` 锁定外部规则包的精确版本、commit、checksum 和解析后的依赖信息。

## 规则包协议

规则包是 harness-lint 生态的基本分发单位。规则包可以来自 git、本地目录，后续可以扩展 npm、cargo、pip、OCI registry 等来源。

推荐目录结构：

```text
harness-rules-python/
├── harness-pack.toml
├── rules/
│   ├── prefer-pydantic.md
│   ├── no-bare-except.md
│   └── no-print-debug.md
├── tests/
└── README.md
```

`harness-pack.toml` 示例：

```toml
[pack]
id = "python"
name = "Python Best Practices"
version = "1.2.0"
description = "GritQL rules for Python projects."
license = "MIT"

[compat]
harness = ">=0.1.0"
grit = ">=0.1.0"
languages = ["python"]

[rules.prefer-pydantic]
path = "rules/prefer-pydantic.md"
default_level = "warn"
tags = ["python", "validation", "ai-style"]
```

规则包必须有唯一 `pack.id`。包内规则 id 建议带包命名空间，例如 `python.prefer-pydantic`。

## 规则文件协议

规则文件采用 Markdown + YAML frontmatter。这样规则既是可执行单元，也是人类可读文档。

示例：

````markdown
---
id: python.prefer-pydantic
title: Prefer Pydantic for structured validation
language: python
level: warn
status: draft
tags: [python, validation, ai-style]
fixable: false
---

# Prefer Pydantic for structured validation

When validating structured Python input, use Pydantic models instead of
manual dictionary validation.

```grit
language python
// GritQL pattern goes here.
```

## Bad

```python
def parse_user(data):
    if not isinstance(data["name"], str):
        raise ValueError("name")
```

## Good

```python
from pydantic import BaseModel

class User(BaseModel):
    name: str
```
````

规则状态：

- `draft`：可以缺少完整 GritQL 或测试样例，不参与阻塞检查。
- `warn`：可以参与检查，但默认不阻塞。
- `enforced`：必须有可执行规则和测试样例，可以阻塞 CI 或 agent 流程。

## 核心数据模型

```rust
struct PackSpec {
    id: String,
    source: PackSource,
    version_req: Option<String>,
}

struct ResolvedPack {
    spec: PackSpec,
    local_path: PathBuf,
    version: String,
    checksum: String,
}

struct RulePack {
    id: String,
    name: String,
    version: String,
    rules: Vec<RuleDefinition>,
}

struct RuleDefinition {
    id: String,
    title: String,
    language: Option<String>,
    level: Severity,
    status: RuleStatus,
    tags: Vec<String>,
    fixable: bool,
    body: RuleBody,
    examples: Vec<RuleExample>,
}

struct Diagnostic {
    rule_id: String,
    level: Severity,
    message: String,
    path: PathBuf,
    start_line: u32,
    start_column: u32,
    end_line: Option<u32>,
    end_column: Option<u32>,
    fix_available: bool,
}
```

这些模型是内部稳定边界。CLI、Grit adapter、reporter、rule authoring 都围绕它们通信。

## 核心接口

```rust
trait RuleSource {
    fn resolve(&self, spec: PackSpec) -> Result<ResolvedPack>;
    fn update(&self, lock: LockEntry) -> Result<ResolvedPack>;
}

trait RuleCompiler {
    fn compile(&self, packs: Vec<RulePack>, config: ProjectConfig) -> Result<CompiledRules>;
}

trait GritRunner {
    fn check(&self, rules: CompiledRules, files: Vec<PathBuf>) -> Result<Vec<Diagnostic>>;
    fn fix(&self, rules: CompiledRules, files: Vec<PathBuf>) -> Result<FixResult>;
}

trait RuleAuthoring {
    fn suggest_rule(&self, feedback: String, context: ProjectContext) -> Result<RuleDraft>;
}

trait Reporter {
    fn report(&self, diagnostics: Vec<Diagnostic>, options: ReportOptions) -> Result<()>;
}
```

第一版实现 local / git `RuleSource` 和 Grit CLI `GritRunner`。可扩展的是规则包来源，不扩展第二套规则执行路径；规则文件里的可执行部分始终是 GritQL。

## Grit 集成方式

harness-lint 把启用的规则编译成 Grit 可读的 `.grit` 项目结构，然后调用 Grit CLI。

```text
harness rules
  -> .harness/generated/grit/patterns/*.md
  -> .harness/generated/grit/grit.yaml
  -> grit check --grit-dir .harness/generated/grit <paths>
```

Grit adapter 负责：

- 检测 `grit` 是否安装。
- 校验 Grit 版本。
- 执行 `grit check`。
- 执行 `grit check --fix`。
- 解析 JSON / JSONL 输出。
- 转换为统一 `Diagnostic`。

harness-lint 不直接解释 GritQL 语义。

## 增量运行

增量运行以 git 为主：

```text
harness-lint check --changed
  -> git diff --name-only --diff-filter=ACMR <base>...HEAD
  -> staged files
  -> untracked files
  -> path ignore
  -> rule language/glob filter
  -> Grit CLI execution
```

默认 base 来自 `harness.toml` 的 `lint.changed_base`，命令行 `--base` 可以覆盖。

缓存 key：

```text
hash(
  file_content_hash,
  rule_content_hash,
  grit_version,
  harness_config_hash
)
```

规则、配置、Grit 版本或文件内容任一变化，缓存自动失效。

## 用户反馈到规则

这是 harness-lint 区别于普通 lint wrapper 的核心能力。

用户可以运行：

```bash
harness-lint rule suggest "Python 里结构化参数校验统一用 pydantic，不要手写 dict 校验"
```

系统生成：

```text
harness/rules/local/python.prefer-pydantic.md
```

生成规则默认是 `draft`。如果没有足够信息生成可靠 GritQL，文件中保留 TODO，但仍然沉淀了 rule id、标题、说明和例子。

推荐的 AI agent 协议：

```markdown
When the user expresses a recurring coding preference, create or update a
harness-lint rule instead of only changing the current code.
Run `harness-lint check --changed` before finishing.
```

AI 不应该通过删除规则、降级 severity 或忽略 diagnostics 来解决问题，除非用户明确要求。

## CLI 命令

```text
harness init
harness-lint check [paths...]
harness-lint check --changed
harness-lint check --staged
harness fix [paths...]
harness fix --changed

harness pack add <spec>
harness pack update
harness pack list
harness pack remove <id>

harness-lint rule list
harness-lint rule explain <rule-id>
harness-lint rule new
harness-lint rule suggest <feedback>
harness-lint rule test <rule-id>
harness-lint rule enable <rule-id>
harness-lint rule disable <rule-id>
harness-lint rule set-level <rule-id> <level>
```

## Reporter

Reporter 输出应该同时服务人类、CI 和 AI agent。

需要支持：

- human terminal output
- JSON
- JSONL
- AI-readable Markdown
- GitHub Actions annotations
- SARIF interface placeholder

AI-readable Markdown 应保持稳定结构，方便 agent 读取并修复。

## 错误处理

错误分为几类：

- 配置错误：配置文件不存在、字段非法、override 指向不存在规则。
- 规则包错误：下载失败、版本冲突、manifest 非法、rule id 冲突。
- 规则错误：frontmatter 缺失、GritQL 语法错误、draft 规则被 enforced。
- Grit 错误：Grit 未安装、版本不兼容、执行失败。
- 运行错误：git base 不存在、路径不存在、缓存损坏。

错误信息必须包含：

- 错误类型。
- 相关文件路径。
- 修复建议。
- 对 AI agent 友好的简短说明。

## 扩展方向

后续可以扩展更多 source：

- npm package
- cargo crate
- pip package
- OCI artifact
- HTTP tarball

后续不增加 text、regex、external、LLM advisory 等并行执行路径。用户反馈要落成 rule 时，先生成可审核的 GritQL draft；如果暂时不能表达为 GritQL，就保持 draft 和 TODO，而不是引入另一种执行器。
