Close the Lint Driven Development loop for this session.

1. Review the conversation so far and list the **corrections and feedback** the
   user gave about how code in this repo should or should not be written —
   especially anything phrased as "don't do X", "do it this way instead", "I
   already told you", or a review comment that points at a class of mistake.
   Ignore one-off task instructions that are not reusable constraints.

2. For each item, decide whether it can be expressed as a **reliable GritQL
   pattern** over a concrete bad-code shape. If it cannot (it needs judgment,
   broad context, or fuzzy matching), do **not** make a rule — note that it
   belongs in docs/review notes instead, and move on.

3. Before creating anything, run `harness-lint rule list` and check whether an
   existing rule already covers it (update that rule rather than duplicating).

4. If a new rule is warranted, load the harness-lint skill if you have not
   already, then follow the authoring workflow:
   - `harness-lint rule create "<feedback>" --language <language> --grit <gritql>`
   - edit the rule file: description + smallest Bad example + Good replacement
   - `harness-lint doctor`
   - `harness-lint rule verify <rule-id>` (prove the Bad example triggers)
   - `harness-lint check --all --rule <rule-id>` (confirm expected files only)
   - `harness-lint check --changed`

   Use `language js` for TypeScript/JavaScript. Scope by filename inside GritQL
   with `$filename` when the rule applies only to certain paths. Keep `level:
   warn` unless the pattern, description, and examples are all unambiguous.

5. If nothing in this session qualifies as a reusable, GritQL-expressible
   constraint, say so in one line and stop — do not invent a rule to have one.

$ARGUMENTS
