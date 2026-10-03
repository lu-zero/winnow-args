# For coding agents

[`CONTRIBUTING.md`](./CONTRIBUTING.md) is the project's conventions: tasks,
layout, performance posture, dependencies, style, commits. Read it first. This
file adds what an agent in particular gets wrong.

## Slop warning

This codebase was largely AI-generated. Be skeptical of existing code,
comments and docs: a pattern being there does not make it correct.

## Working here

- `just check` passes before a commit. `.claude/hooks/` checks your tool calls
  (rustdoc, comment and commit-message length); there is no git hook.
- Sibling checkouts (`../usage`, `../winnow`, `../bpaf`, `../clap`) are for
  reading: never copy from them, and never build against them.
- `.agents/todo/` is local scratch for work in progress: gitignored, pruned at
  will, and cited by nothing that is committed. A fact that matters goes in a
  comment as a sentence, or in `docs/CHECKLIST.md` once it is status.
- A commit you assisted carries `Assisted-by: AGENT:MODEL`
  (`Assisted-by: Claude:claude-opus-5-5`); list specialized analysis tools
  after the model, not git, cargo or editors. Never add `Co-Authored-By` or a
  `Signed-off-by`: the sign-off is the human's.

## Unslop rules

Enforced. Each is something a past pass got wrong and had to clean up.

**Comments**
- Default to no comment. Add one only when the *why* is non-obvious: a hidden
  constraint, a workaround for a specific bug, an invariant a reader would
  break without warning.
- One sentence beats a paragraph. A comment that needs several paragraphs to
  justify a few lines means the code needs simplifying.
- Never restate what the names already say.
- Never reference the task, a commit, a PR or issue number, or a session:
  that belongs in the commit message.
- No commented-out code and no `// removed: …` markers; git is the history.
- Hard limits: no comment line over **150 characters**, no paragraph over
  **5 lines** (3 is better). Hitting either means cut, not wrap. Longer
  justification goes in the commit message or `docs/DESIGN.md`.

**Dead weight**
- No speculative abstraction for a single call site: no knob, trait or
  feature flag without a second caller that needs it today.
- No error handling, fallback or validation for a case the caller's own
  guarantees rule out.
- `#[allow(dead_code)]` does not keep something "just in case": delete it.

**Never downgrade a domain type to make code fast**
- `String`/`&str` is for text that is just text. A path stays
  `PathBuf`/`OsString`; a command-line value stays `&BStr` until its type
  converts it.
- When a typed value is hot in a profile, use the cheap accessor at the
  comparison site (`as_bytes()`), not a weaker field type.

**Tests exercise project logic, not the standard library**
- No test whose assertions would hold for any correct implementation of the
  primitive underneath.
- One test at the real decision point (a branch, an edge case, a regression)
  beats a test added so that "added a function, added a test" is true.

**Scope: stay inside what you were asked for**
- When work surfaces a different, deeper subsystem, stop and report before
  editing it. Agreement that an idea has merit is not authorization.
- A signature change that cascades through many call sites is a point to
  confirm the shape before the mechanical part.
- A change that was reverted once needs fresh, explicit confirmation to try
  again.
