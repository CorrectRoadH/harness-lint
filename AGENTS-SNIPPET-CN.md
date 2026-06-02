<!--HARNESS LINT START-->
在该项目开发应该是 LDD（Lint Driven Development）的。当用户反馈或代码审查指出一类代码应该怎么做、或者不应该怎么做时，不要只修复当前这一处。请创建或更新一条能够捕捉该问题的 `harness-lint` 规则，运行 lint 让它报告这个问题，然后再修改代码直到 lint 通过。

创建本地规则时，请使用以下流程：

1. 运行 `harness-lint rule list` 查看已有 lint，判断是否应该更新现有规则。
2. 创建新规则前，先判断这条反馈是否能稳定表达为 GritQL pattern。如果不能，就不要创建 harness-lint rule；把它保留在 agent 指令、review 记录或项目文档中。
3. 如果需要新规则，运行 `harness-lint rule create "<feedback>" --language <language> --grit <gritql>` 创建本地规则文件。
4. 编辑生成的规则文件，补充规则说明和 Bad / Good 示例。
5. 运行 `harness-lint doctor` 确认配置、规则和 Grit 环境正常。
6. 运行 `harness-lint check --all --rule <rule-id>`，确认新规则命中了预期文件。不要通过给 `check` 传路径来模拟规则范围；如果规则只应该作用于部分文件，必须在 GritQL 中用 `$filename` 表达。
7. 运行 `harness-lint check --changed` 执行 lint，确认规则能被加载并按预期工作。

写本地规则时遵守这些最佳实践：

- 规则只表达一条稳定、可重复检查的团队约束。
- 规则 `id` 和文件名要可读、稳定。允许中文和其他语言，但不要使用路径符号或装饰性符号；空格用 `-` 替代；英文最好用全小写 kebab-case，例如 `local.no-print-debug`；中文可以用短句，例如 `local.禁止使用UI` 或 `local.禁止-使用-UI`。
- 尽量让 `id` 和文件名对齐，例如 `id: local.no-print-debug` 对应 `no-print-debug.md`。
- 每个规则文件必须有且只有一个可执行的 `grit` fenced code block。GritQL 先写最小、最确定的坏代码形状；用 `$value`、`$name`、`$body` 这类 metavariable 表示会变化的部分；如果 GritQL 看不准，就不要创建 harness-lint rule。
- 如果规则只应该作用于部分文件，直接在 GritQL 中用 `$filename` 条件表达，例如 `$filename <: r".*src/.*\.ts"` 和 `!$filename <: r".*\.test\.ts"`。
- Bad 示例写最小的违规代码；Good 示例写本项目推荐的替代写法；示例语言要和 `language` 一致。
- 只有 GritQL、说明、Bad / Good 示例都足够清楚时，才使用 `level: error`；否则保持 `level: warn`。

如果你需要写 rule 或者不了解 harness-lint，请先加载 harness-lint skill；如果没有该 skill，可以通过 `npx skills add CorrectRoadH/harness-lint` 安装。

如果 lint 出错，先运行 `harness-lint rule explain <rule-id>` 阅读具体规则。规则正确时修代码；规则误报时收窄 GritQL、补充说明或调整 Bad / Good 示例，但不要为了通过而直接删除或削弱规则。
<!--HARNESS LINT END-->
