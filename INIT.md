## 角色与语言

你正在帮用户把当前仓库接入 `harness-lint`。先阅读仓库里的 `AGENTS.md`、`CLAUDE.md`、`.cursor/rules`、`README.md` 和相关项目文档，判断用户主要使用什么语言；后续提问和总结都尽量使用用户习惯的语言。

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

读取 https://raw.githubusercontent.com/CorrectRoadH/harness-lint/refs/heads/main/AGENTS-SNIPPET.md 里的 `<!--HARNESS LINT START-->` 到 `<!--HARNESS LINT END-->` 标识块安装到用户本仓库的 `AGENTS.md` 或 `CLAUDE.md`。如果两个文件有一个是软链接，只安装一次；如果两个都没有，就创建 `AGENTS.md`。

## 3. 帮用户配置初始规则

初始化规则的目标是把用户仓库里已有的 agent 约束沉淀成本地规则。

### 3.1 从用户已有约束初始化本地规则

先检测仓库语言和框架。优先看这些信号：

- Python: `pyproject.toml`、`requirements.txt`、`*.py`
- TypeScript/JavaScript: `package.json`、`tsconfig.json`、`*.ts`、`*.tsx`、`*.js`
- Go: `go.mod`、`*.go`
- Rust: `Cargo.toml`、`*.rs`

从用户已有的 `AGENTS.md`、`CLAUDE.md`、`.cursor/rules`、README 和 review 文档中提取稳定、可重复检查的约定，并直接创建或更新本地规则。规则名、标题、描述、Bad / Good 示例和最终给用户的总结，都应该使用用户习惯的语言；如果用户的仓库文档主要是中文，就用中文写规则内容。创建前先运行 `harness-lint rule list` 查看已有 lint，避免重复规则。

不要把一次性任务、模糊偏好、流程愿望、跨文件意图判断或无法用 GritQL 稳定检查的约束写成 harness-lint rule。创建规则前必须先确认它能被一个可靠的 GritQL pattern 捕捉；如果不能，就不要创建规则，把它保留在 agent 指令、review checklist 或项目文档中。对每条需要新增的可规则化约定，统一用 CLI 创建本地规则文件：

```sh
harness-lint rule create "<constraint>" --language <language> --grit <gritql>
```

然后手动编辑生成的 `rules/*.md`：

- 用用户习惯的语言改好 `id`、`title`、正文说明、Bad / Good 示例。
- 规则 `id` 和文件名要可读、稳定。允许中文和其他语言，但不要使用路径符号或装饰性符号；空格用 `-` 替代；英文最好用全小写 kebab-case，例如 `local.no-print-debug`；中文可以用短句，例如 `local.禁止使用UI` 或 `local.禁止-使用-UI`。
- 尽量让 `id` 和文件名对齐，例如 `id: local.no-print-debug` 对应 `no-print-debug.md`。
- 每个规则文件必须有且只有一个可执行的 `grit` fenced code block；`harness-lint doctor` 会拒绝缺失、空的、TODO/comment-only 或多个 GritQL block。
- 如果规则只应该作用于部分文件，直接在 GritQL 中用 `$filename` 条件表达，例如 `$filename <: r".*src/.*\.ts"` 和 `!$filename <: r".*\.test\.ts"`；不要额外发明 frontmatter scope。
- 还拿不准时保持 `level: warn`；只有团队明确希望失败退出时才改成 `level: error`。
- 如果规则适合触发特定 Codex skill，添加精确的 `skill: <skill-name>`；不确定时留空。常见示例：`tdd` 用于测试先行修复，`triage-issue` 用于根因定位和 issue 计划，`codex-security:fix-finding` 用于明确安全漏洞修复，`build-web-apps:frontend-testing-debugging` 用于前端渲染/交互/回归问题，`build-web-apps:react-best-practices` 用于 React / Next.js 性能或最佳实践问题。

写 GritQL 时：

- 针对具体语言时先写 `language <name>`。Grit 支持的语言名以 Grit CLI 为准，例如 `js`、`python`、`json`、`java`、`hcl`、`css`、`markdown`、`yaml`、`rust`、`ruby`、`php`、`go`、`sql` 等。TypeScript/JavaScript 规则的 Grit 语言写 `language js`；需要 TypeScript parser 变体时可以写 `language js(typescript)`。
- 先匹配最小、最确定的坏代码形状，宁可窄一点，也不要为了覆盖更多场景造成误报。
- 用 `$value`、`$name`、`$body` 这类 metavariable 表示会变化的部分。
- 先直接匹配禁止形状；只有出现真实误报后，再补例外条件。
- `where` 条件之间用逗号分隔，不要用分号。
- 如果规则依赖跨文件语义、项目配置、所有权或意图，GritQL 看不准，就不要创建 harness-lint rule，也不要硬造一个不可靠的 pattern。

GritQL 示例：

````markdown
```grit
language js
`console.log($value)` where {
  $filename <: r".*src/.*\.ts",
  !$filename <: r".*\.test\.ts"
}
```

```grit
language python
`print($value)`
```

```grit
language go
`context.TODO()`
```
````

写 Bad / Good 示例时：

- Bad 示例应该是最小的、应该被规则命中的代码。
- Good 示例应该展示本项目推荐写法，不只是把坏代码删掉。
- 示例语言要和 `language` 一致。
- 示例只放这条规则关心的边界，不要塞大段无关脚手架。
- `level: error` 的规则必须有清晰的 GritQL 和 Bad / Good 示例。

完成后用用户的语言告诉用户“我帮你写了哪些规则”，并给一个 one-shot 摘要，不要只说“已创建规则”。

创建或修改每条规则后，必须先验证 Bad 示例能被规则抓到，再单独运行这条规则确认实际仓库命中范围。不要通过给 `check` 传路径来模拟规则范围；如果规则只应该作用于部分文件，必须在 GritQL 中用 `$filename` 表达：

```sh
harness-lint rule verify <rule-id>
harness-lint check --all --rule <rule-id>
```

检查输出里应该出现预期命中的文件，且不应该出现不相关文件。若规则没有诊断、范围太大或范围太小，先调整 GritQL，再继续全局检查。

One-shot 示例：

```text
我从 AGENTS.md 和 README 里提取了 3 条可以自动检查的团队约束，并写成本地规则：

1. `local.typescript-no-console-debug`：提交代码里不要留下 `console.log` 调试输出。
2. `local.react-no-index-key`：React 列表渲染不要用数组下标做 key。
3. `local.api-errors-need-context`：API 错误日志需要带上 request id 或业务上下文。

```

最后运行：

```sh
harness-lint doctor
harness-lint rule verify <rule-id>
harness-lint check --all --rule <rule-id>
harness-lint check --changed
```

## 配置 Hook
在最终总结前，必须主动询问用户是否需要继续帮助配置 git hook，比如 commit hook。不要把这一步写成“需要你确认后再接入”“等你确认”或其他被动等待表述；如果还没有配置 hook，就用一个明确问题收尾，例如：“需要我继续帮你把 `harness-lint check --changed` 接到 git hook 里吗？”

如果用户同意，先检查当前仓库是否已有 git hook 配置。已有配置就复用并追加 harness-lint 检查；没有配置时，用最适合该项目的最佳实践安装与配置 git hook，不要为了这一步额外引入新的 hook 管理工具。

## 最后一步
向用户总结：

- 已安装或确认的工具。
- 初始化了哪些文件。
- 写入或更新了哪个 agent 指令文件。
- 发现了哪些语言/框架。
- 从用户已有约束中创建了哪些本地规则；哪些无法用 GritQL 描述，所以没有写到 lint 下面
- 已主动询问是否需要继续帮助配置 git hook；如果用户同意并已配置，也说明采用了哪种 hook 方案。
