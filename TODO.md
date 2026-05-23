# harness-lint TODO

这个清单按依赖顺序排列。每一项都默认需要完成，不分 MVP / optional；后面的能力依赖前面的协议、数据结构或运行链路。

## 1. 定义产品边界

- [x] 明确项目定位：harness-lint 是 GritQL 规则生态与规则创作工具，不重新实现 lint 后端。
- [x] 明确第一类用户场景：把 AI coding feedback 沉淀成可执行、可更新、可 review 的规则。
- [x] 明确非目标：不默认使用 LLM 执行 lint，不把自然语言规范当作阻塞性检查。
- [x] 明确核心输出格式：终端诊断、JSON、AI-readable Markdown。
- [x] 明确规则生命周期：draft -> warn -> enforced。

## 2. 定义仓库与项目配置协议

- [x] 设计 `harness.toml` 项目配置格式。
- [x] 设计 `harness.lock` 锁定文件格式。
- [x] 设计本地规则目录：`harness/rules/local/`。
- [x] 设计外部规则缓存目录：`.harness/packs/`。
- [x] 设计生成给 Grit 使用的中间目录：`.harness/generated/grit/`。
- [x] 设计忽略规则与路径过滤协议。
- [x] 设计 severity override 协议。
- [x] 设计 rule enable / disable 协议。

## 3. 定义规则包协议

- [x] 设计 `harness-pack.toml` manifest 格式。
- [x] 定义规则包元数据：id、name、version、description、license、source。
- [x] 定义兼容性元数据：harness 版本、Grit 版本、支持语言。
- [x] 定义规则索引结构：rule id、path、default level、tags、language、fixable。
- [x] 定义规则包目录结构：`rules/`、`tests/`、`README.md`。
- [x] 定义包内规则命名空间，避免不同包 rule id 冲突。
- [x] 定义规则包更新策略与版本 pinning 规则。

## 4. 定义规则文件协议

- [x] 采用 Markdown + YAML frontmatter 作为用户可读规则格式。
- [x] 定义 frontmatter 字段：id、title、engine、language、level、status、tags、fixable。
- [x] 定义正文结构：说明、GritQL 代码块、Bad examples、Good examples。
- [x] 定义 draft 规则允许缺少完整 GritQL，但不能进入 enforced。
- [x] 定义规则测试样例的解析规则。
- [x] 定义 rule explain 所需的文档字段。
- [x] 定义本地用户规则与外部规则的覆盖优先级。

## 5. 定义内部数据模型

- [x] 定义 `PackSpec`：用户配置中的包引用。
- [x] 定义 `ResolvedPack`：下载或解析后的规则包。
- [x] 定义 `RulePack`：manifest + rules 的内存模型。
- [x] 定义 `RuleDefinition`：单条规则的结构化表示。
- [x] 定义 `CompiledRules`：面向 Grit CLI 的生成结果。
- [x] 定义 `Diagnostic`：统一诊断格式。
- [x] 定义 `FixResult`：自动修复结果格式。
- [x] 定义 `ProjectContext`：rule authoring 使用的项目上下文。
- [x] 定义 `RuleDraft`：从用户反馈生成的规则草稿。

## 6. 定义核心接口

- [x] 定义 `RuleSource`：负责安装、解析、更新规则包。
- [x] 定义 `RuleCompiler`：负责把 harness 规则编译为 Grit 可执行配置。
- [x] 定义 `RuleEngine`：负责执行检查和修复。
- [x] 定义 `RuleAuthoring`：负责从用户反馈创建或更新规则草稿。
- [x] 定义 `Reporter`：负责输出终端、JSON、Markdown 等格式。
- [x] 定义错误类型与错误码。
- [x] 定义接口之间的数据流边界。

## 7. 实现 CLI 框架

- [x] 初始化 Rust CLI 项目。
- [x] 选择 CLI 参数解析库。
- [x] 实现全局选项：`--config`、`--cwd`、`--json`、`--verbose`。
- [x] 实现命令路由。
- [x] 实现项目根目录发现。
- [x] 实现统一错误展示。
- [x] 实现日志与调试输出。

## 8. 实现配置读取

- [x] 读取 `harness.toml`。
- [x] 校验配置 schema。
- [x] 读取 `harness.lock`。
- [x] 合并默认配置。
- [x] 合并本地规则、外部规则、用户 override。
- [x] 支持配置不存在时给出 init 提示。

## 9. 实现 `harness init`

- [x] 创建 `harness.toml`。
- [x] 创建 `harness/rules/local/`。
- [x] 创建 `.harness/` 工作目录。
- [x] 生成默认 `.gitignore` 片段或提示。
- [x] 检测已有 `.grit/` 配置并避免破坏用户配置。
- [x] 在安装文档中要求 LLM 读取 `CLAUDE.md`、`AGENTS.md`、`.cursor/rules`。
- [x] 在安装过程中由 LLM 把现有规范转换成 proposed rule 草稿。
- [x] 输出需要加入 AI agent instructions 的短协议。

## 10. 实现规则源解析

- [x] 实现 local source：读取本地规则包或本地规则目录。
- [x] 实现 git source：下载指定 git 仓库。
- [x] 实现版本 pinning。
- [x] 实现 `.harness/packs/` 缓存。
- [x] 实现 `harness.lock` 更新。
- [x] 校验规则包 manifest。
- [x] 校验规则包内 rule id 唯一性。

## 11. 实现规则包命令

- [x] 实现 `harness pack add <spec>`。
- [x] 实现 `harness pack update`。
- [x] 实现 `harness pack list`。
- [x] 实现 `harness pack remove <id>`。
- [x] 实现包安装失败回滚。
- [x] 实现版本冲突与命名冲突提示。

## 12. 实现规则解析

- [x] 解析 Markdown frontmatter。
- [x] 提取 GritQL 代码块。
- [x] 提取说明文本。
- [x] 提取 Bad / Good examples。
- [x] 校验必填字段。
- [x] 校验 draft / warn / enforced 的字段约束。
- [x] 把解析结果转换为 `RuleDefinition`。

## 13. 实现 Grit 编译器

- [x] 将启用的规则生成到 `.harness/generated/grit/patterns/`。
- [x] 生成 `.harness/generated/grit/grit.yaml`。
- [x] 保留 rule id、title、level、tags 等元数据。
- [x] 处理本地规则覆盖外部规则。
- [x] 处理 severity override。
- [x] 处理 disabled rules。
- [x] 处理只包含 draft 的规则，不进入 Grit check。

## 14. 实现 Grit engine adapter

- [x] 检测 `grit` CLI 是否安装。
- [x] 检测 Grit 版本兼容性。
- [x] 调用 `grit check`。
- [x] 支持 `grit check --fix`。
- [x] 支持传入目标 paths。
- [x] 支持 JSON / JSONL 输出解析。
- [x] 把 Grit 输出转换为统一 `Diagnostic`。
- [x] 处理 Grit 执行失败、规则语法错误、缺少语言支持等错误。

## 15. 实现文件选择与增量运行

- [x] 实现全量文件发现。
- [x] 实现基于 git 的 changed files 发现。
- [x] 支持 staged 文件。
- [x] 支持 untracked 文件。
- [x] 支持指定 base：`--base origin/main`。
- [x] 按规则的 language 与 glob 过滤文件。
- [x] 按规则包和 engine 分组执行。
- [x] 避免把 `.harness/`、`.git/`、依赖目录传给 Grit。

## 16. 实现缓存

- [x] 设计 cache key：文件内容 hash + 规则 hash + engine version + config hash。
- [x] 存储每个文件每组规则的诊断结果。
- [x] 支持规则变更后自动失效。
- [x] 支持配置变更后自动失效。
- [x] 支持 `--no-cache`。
- [x] 支持 `--refresh-cache`。
- [x] 支持缓存清理命令。

## 17. 实现 check / fix 命令

- [x] 实现 `harness check`。
- [x] 实现 `harness check --changed`。
- [x] 实现 `harness check --staged`。
- [x] 实现 `harness fix`。
- [x] 实现 `harness fix --changed`。
- [x] 实现根据 severity 决定退出码。
- [x] 实现只检查指定 rule 或 tag。
- [x] 实现只检查指定 paths。

## 18. 实现 reporter

- [x] 实现人类友好的终端输出。
- [x] 实现 JSON 输出。
- [x] 实现 JSONL 输出。
- [x] 实现 AI-readable Markdown 输出。
- [x] 实现 GitHub Actions annotation 输出。
- [x] 预留 SARIF 输出接口。
- [x] 输出 rule explain 链接或本地路径。

## 19. 实现 rule 查询命令

- [x] 实现 `harness rule list`。
- [x] 实现 `harness rule explain <rule-id>`。
- [x] 实现 `harness rule enable <rule-id>`。
- [x] 实现 `harness rule disable <rule-id>`。
- [x] 实现 `harness rule set-level <rule-id> <level>`。
- [x] 实现按 tag、pack、language 过滤规则。

## 20. 实现用户反馈到规则草稿

- [x] `harness rule suggest` 基于项目语言/library 搜索计划中的规则后端。
- [x] 找到已有规则时提示安装对应 rule pack，而不是直接生成本地规则。
- [x] 实现 `harness rule new` 交互式创建。
- [x] 实现 `harness rule suggest "<feedback>"`。
- [x] 从 feedback 中生成 rule id、title、description、examples。
- [x] 生成 Markdown 规则草稿到 `harness/rules/local/`。
- [x] 标记新规则为 `status = "draft"`。
- [x] 允许用户补充 Bad / Good examples。
- [ ] 如果可以确定 GritQL，则生成候选 GritQL。当前先通过 registry 搜索已有规则，后续接 LLM/后端生成。
- [x] 如果不能确定 GritQL，则生成 TODO 占位并保持 draft。

## 21. 实现规则草稿验证

- [x] 实现 `harness rule test <rule-id>`。
- [x] 使用 Bad examples 验证规则能命中。
- [x] 使用 Good examples 验证规则不误报。
- [ ] 对可修复规则验证修复输出。当前先验证 text/regex examples，Grit autofix 需接真实 Grit test runner。
- [x] 规则测试通过后允许从 draft 提升到 warn。
- [x] enforced 规则必须有至少一个 Bad 和一个 Good example。

## 22. 实现 AI agent 集成协议

- [x] 生成可插入 `CLAUDE.md` / `AGENTS.md` 的短协议。
- [x] 约定 AI 在用户反馈编码偏好时运行 `harness rule suggest`。
- [x] 约定 AI 在结束前运行 `harness check --changed`。
- [x] 约定 AI 优先修复 lint，而不是删除或降级规则。
- [x] 为 AI-readable Markdown reporter 提供稳定格式。

## 23. 实现文档

- [x] 写 README。
- [x] 写快速开始。
- [x] 写规则文件格式文档。
- [x] 写规则包格式文档。
- [x] 写 AI agent 集成文档。
- [x] 写安装时由 LLM 读取现有 agent 文档并转换规则的说明。
- [x] 写 GritQL 规则编写指南。
- [x] 写本地规则与外部规则覆盖说明。
- [x] 写故障排查文档。

## 24. 实现测试

- [x] 为配置解析写单元测试。
- [x] 为规则包 manifest 解析写单元测试。
- [x] 为规则 Markdown 解析写单元测试。
- [x] 为 Grit 编译器写快照测试。
- [x] 为 git changed files 发现写集成测试。
- [x] 为 Grit adapter 写集成测试。
- [x] 为 rule suggest 写测试。
- [x] 为 CLI 命令写端到端测试。

## 25. 实现发布与分发

- [x] 配置 Rust release build。
- [x] 配置 GitHub Releases。
- [x] 设计 npm wrapper。
- [x] 设计 Homebrew formula。
- [x] 设计 cargo install 路径。
- [x] 设计规则包发布规范。
- [x] 设计规则包版本更新流程。
- [x] 设计迁移策略。
