## 角色与语言

你正在帮用户把当前仓库接入 `harness-lint`。先阅读仓库里的 `AGENTS.md`、`CLAUDE.md`、`.cursor/rules`、`README.md` 和相关项目文档，判断用户主要使用什么语言；后续提问和总结都尽量使用用户习惯的语言。

提问时从用户体验出发，不要只问“要加什么规则”。可以问：

- 你最希望 agent 以后自动避免哪类重复问题？
- 代码 review 里最常被指出的风格、架构或安全问题是什么？
- 有没有团队约定是新人或 AI 经常漏掉的？

如果已有文档已经能回答这些问题，先据此执行，只在确实需要取舍时再问。

## 0. 安装 agent skill

先安装 `harness-lint` skill，让 agent 后续知道规则怎么写、命令怎么调、lint 失败时怎么查看并修复具体 rule：

```sh
npx skills add CorrectRoadH/harness-lint -a codex -g -y
```

如果当前环境不是 Codex，改用对应 agent 名称；想安装给所有支持的 agent，可用：

```sh
npx skills add CorrectRoadH/harness-lint -a '*' -g -y
```

安装后继续执行下面步骤。若 skill 暂时无法安装，也不要中断接入流程；继续按本文档和仓库文档操作。

## 1. 检查安装

先确认工具可用：

```sh
command -v grit
grit --version
command -v harness-lint
harness-lint doctor
```

如果 `harness-lint` 不存在，帮用户安装

```sh
brew install getgrit/tap/grit
brew install CorrectRoadH/tap/harness-lint
```

安装后重新运行 `harness-lint doctor`。如果用户的环境不是 macOS/Homebrew，根据项目环境选择等价安装方式，并明确告诉用户你做了什么。

## 2. 初始化本仓库配置

在仓库根目录运行：

```sh
harness-lint init
```

确认生成或更新了：

```text
harness.toml
rules/
.harness/
```

`harness.toml` 和 `rules/` 应提交到仓库；`.harness/` 应忽略。

然后把下面的标识块加入用户已有的 `AGENTS.md` 或 `CLAUDE.md`。优先更新已经存在的 agent 指令文件；如果两个都没有，创建 `AGENTS.md`。如果文件里已经有 `<!--HARNESS LINT START-->` 和 `<!--HARNESS LINT END-->`，只替换标识块内部内容，不要重复插入。

把 `AGENTS-SNIPPET-CN.md` 里的 `<!--HARNESS LINT START-->` 到 `<!--HARNESS LINT END-->` 标识块安装到用户本仓库的 `AGENTS.md` 或 `CLAUDE.md`。如果两个文件有一个是软链接，只安装一次；如果两个都没有，就创建 `AGENTS.md`。

## 3. 帮用户配置初始规则

初始化规则的目标是把用户仓库里已有的 agent 约束沉淀成本地规则。

### 3.1 从用户已有约束初始化本地规则

先检测仓库语言和框架。优先看这些信号：

- Python: `pyproject.toml`、`requirements.txt`、`*.py`
- TypeScript/JavaScript: `package.json`、`tsconfig.json`、`*.ts`、`*.tsx`、`*.js`
- Go: `go.mod`、`*.go`
- Rust: `Cargo.toml`、`*.rs`

从用户已有的 `AGENTS.md`、`CLAUDE.md`、`.cursor/rules`、README 和 review 文档中提取稳定、可重复检查的约定，并直接创建或更新本地规则。规则名、标题、描述、Bad / Good 示例和最终给用户的总结，都应该使用用户习惯的语言；如果用户的仓库文档主要是中文，就用中文写规则内容。创建前先运行 `harness-lint rule list` 查看已有 lint，避免重复规则。

不要把一次性任务、模糊偏好或无法检查的愿望强行变成 enforced 规则；不确定的先保持 `draft`。对每条需要新增的可规则化约定，统一用 CLI 创建本地 draft 骨架和规则文件名：

```sh
harness-lint rule draft "<constraint>"
```

然后手动编辑生成的 `rules/*.md`：

- 补充 `language`。
- 用用户习惯的语言改好 `id`、`title`、正文说明、Bad / Good 示例。
- 能写 GritQL 时写 GritQL。
- 还拿不准时保持 `status: draft`。
- 如果规则适合触发特定 Codex skill，添加精确的 `skill: <skill-name>`；不确定时留空。常见示例：`tdd` 用于测试先行修复，`triage-issue` 用于根因定位和 issue 计划，`codex-security:fix-finding` 用于明确安全漏洞修复，`build-web-apps:frontend-testing-debugging` 用于前端渲染/交互/回归问题，`build-web-apps:react-best-practices` 用于 React / Next.js 性能或最佳实践问题。

完成后用用户的语言告诉用户“我帮你写了哪些规则”，并给一个 one-shot 摘要，不要只说“已创建规则”。

One-shot 示例：

```text
我从 AGENTS.md 和 README 里提取了 3 条可以自动检查的团队约束，并写成本地 draft 规则：

1. `local.typescript-no-console-debug`：提交代码里不要留下 `console.log` 调试输出。
2. `local.react-no-index-key`：React 列表渲染不要用数组下标做 key。
3. `local.api-errors-need-context`：API 错误日志需要带上 request id 或业务上下文。

这些规则现在都还是 `draft`，因为我已经写了说明和 Bad / Good 示例，但其中第 3 条还需要再确认 GritQL 能否稳定覆盖你们的日志封装。
```

最后运行：

```sh
harness-lint doctor
harness-lint check --changed
```

向用户总结：

- 已安装或确认的工具。
- 初始化了哪些文件。
- 写入或更新了哪个 agent 指令文件。
- 发现了哪些语言/框架。
- 从用户已有约束中创建了哪些本地规则，哪些仍是 draft。
