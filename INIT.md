# harness-lint 仓库初始化提示词

把这份文件复制给 AI coding agent，用来在一个仓库第一次接入 `harness-lint`。请 agent 按下面流程执行，不要只给计划。

## 角色与语言

你正在帮用户把当前仓库接入 `harness-lint`。先阅读仓库里的 `AGENTS.md`、`CLAUDE.md`、`.cursor/rules`、`README.md` 和相关项目文档，判断用户主要使用什么语言；后续提问和总结都尽量使用用户习惯的语言。

提问时从用户体验出发，不要只问“要加什么规则”。可以问：

- 你最希望 agent 以后自动避免哪类重复问题？
- 代码 review 里最常被指出的风格、架构或安全问题是什么？
- 有没有团队约定是新人或 AI 经常漏掉的？

如果已有文档已经能回答这些问题，先据此执行，只在确实需要取舍时再问。

## 1. 检查安装

先确认工具可用：

```sh
command -v harness-lint
harness-lint doctor
command -v grit
grit --version
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

帮下面 AGENTS 指引帮助用户安装到用户本仓库的 AGENTS.md 与 CLAUDE.md (如果有一个是软链接就只用安装一次)。如果没有就不用。把下面的内容插到最后就行。

READ https://raw.githubusercontent.com/CorrectRoadH/harness-lint/refs/heads/main/AGENTS-Install.md

## 3. 帮用户配置初始规则

先检测仓库语言和框架。优先看这些信号：

- Python: `pyproject.toml`、`requirements.txt`、`*.py`
- TypeScript/JavaScript: `package.json`、`tsconfig.json`、`*.ts`、`*.tsx`、`*.js`
- Go: `go.mod`、`*.go`
- Rust: `Cargo.toml`、`*.rs`
- Markdown/Obsidian: 大量 `*.md`、`.obsidian/`

再从用户已有的 `AGENTS.md`、`CLAUDE.md`、`.cursor/rules`、README 和 review 文档中提取稳定、可重复检查的约定。不要把一次性任务、模糊偏好或无法检查的愿望强行变成 enforced 规则；不确定的先保持 `draft`。

对每条可规则化的约定，先运行：

```sh
harness-lint rule suggest "<constraint>"
```

如果返回已有规则包候选，先用用户的语言解释候选，再询问是否安装。不要擅自安装外部规则包。

如果没有合适候选，创建本地草稿：

```sh
harness-lint rule suggest --local "<constraint>"
```

然后编辑 `rules/*.md`：

- 补充 `language`。
- 补充 Bad / Good 示例。
- 能写 GritQL 时写 GritQL。
- 还拿不准时保持 `status: draft`。
- 如果规则适合触发特定 Codex skill，添加 `skill: <skill-name>`。

最后运行：

```sh
harness-lint rule list
harness-lint check --changed
```

向用户总结：

- 已安装或确认的工具。
- 初始化了哪些文件。
- 写入或更新了哪个 agent 指令文件。
- 发现了哪些语言/框架。
- 创建了哪些规则，哪些仍是 draft，下一步需要用户确认什么。
