<!--HARNESS LINT START-->
Development in this project should follow LDD (Lint Driven Development). When user feedback or code review points out how a class of code should or should not be written, do not only fix the current instance. Create or update a `harness-lint` rule that can catch the issue, run lint so it reports the problem, and then modify the code until lint passes.

When creating a local rule, use this workflow:

1. Run `harness-lint rule list` to inspect existing lint rules and decide whether an existing rule should be updated.
2. Before creating a new rule, decide whether the feedback can be expressed as a reliable GritQL pattern. If it cannot, do not create a harness-lint rule; keep it in agent instructions, review notes, or project documentation instead.
3. If a new rule is needed, run `harness-lint rule create "<feedback>" --language <language> --grit <gritql>` to create the local rule file.
4. Edit the generated rule file and fill in the rule description and Bad / Good examples.
5. Run `harness-lint doctor` to confirm that the configuration, rules, and Grit environment are healthy.
6. Run `harness-lint rule verify <rule-id>` to prove the Bad examples trigger.
7. Run `harness-lint check --all --rule <rule-id>` and confirm the new rule reports the expected file(s). Do not pass paths to `check` to simulate rule scope; if the rule should only apply to certain files, encode that in GritQL with `$filename`.
8. Run `harness-lint check --changed` to execute lint and confirm that the rule can be loaded and works as expected.

Follow these best practices when writing local rules:

- Each rule should express exactly one stable, repeatedly checkable team constraint.
- Rule `id` values and filenames should be readable and stable. Chinese and other languages are allowed, but do not use path symbols or decorative symbols. Replace spaces with `-`. English should preferably use lowercase kebab-case, such as `local.no-print-debug`. Chinese can use short phrases, such as `local.禁止使用UI` or `local.禁止-使用-UI`.
- Keep the `id` and filename aligned whenever possible. For example, `id: local.no-print-debug` should correspond to `no-print-debug.md`.
- Each rule file must contain exactly one executable `grit` fenced code block. Start the GritQL with the smallest and most certain bad-code shape. Use metavariables such as `$value`, `$name`, and `$body` for parts that vary. If the GritQL is not reliable enough, do not create a harness-lint rule.
- For TypeScript/JavaScript rules, use `language js` inside the GritQL block even when the rule frontmatter says `language: typescript`. `language js(typescript)` is valid when the TypeScript parser variant matters. Other rule languages should use Grit CLI language names such as `python`, `json`, `java`, `hcl`, `css`, `markdown`, `yaml`, `rust`, `ruby`, `php`, `go`, and `sql`.
- If a rule should only apply to certain files, express that directly in GritQL with `$filename` conditions, such as `$filename <: r".*src/.*\.ts"` and `!$filename <: r".*\.test\.ts"`.
- Bad examples should show the smallest violating code. Good examples should show the replacement pattern recommended by this project. Example languages must match `language`.
- Use `level: error` only when the GritQL, description, and Bad / Good examples are all clear enough. Otherwise, keep `level: warn`.

If you need to write a rule or are not familiar with harness-lint, load the harness-lint skill first. If that skill is not available, install it with `npx skills add CorrectRoadH/harness-lint`.

If lint fails, first run `harness-lint rule explain <rule-id>` to read the specific rule. When the rule is correct, fix the code. When the rule is a false positive, narrow the GritQL, add clarification, or adjust the Bad / Good examples, but do not delete or weaken the rule just to make lint pass.
<!--HARNESS LINT END-->
