<!--HARNESS LINT START-->
在该项目开发应该是LDD(LINT DRIVE DEVELOPE)的，当用户反馈或代码审查出一个代码应该怎么做或者不应该怎么做的规范的时候，不要只先修复当前这一处。请创建或更新一条能够捕捉该问题的 `harness-lint` 规则，运行 lint 让它报告这个问题，然后再修改代码直到 lint 通过。

创建本地规则时，请使用以下流程：

1. 运行 `harness-lint rule list` 查看已有 lint，判断是否应该更新现有规则。
2. 如果需要新规则，运行 `harness-lint rule create "<feedback>"` 创建本地规则骨架和规则文件名。
3. 编辑生成的规则文件，补充 `language`、规则说明、GritQL、Bad / Good 示例。
4. 运行 `harness-lint doctor` 确认配置、规则和 Grit 环境正常。
5. 运行 `harness-lint check --changed` 执行 lint，确认规则能被加载并按预期工作。

如果你需要写 rule 或者不了解 harness-lint，请先加载 harness-lint skill；如果没有该 skill，可以通过 `npx skills add CorrectRoadH/harness-lint` 安装。
<!--HARNESS LINT END-->
