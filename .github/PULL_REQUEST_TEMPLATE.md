<!-- adapted from comparegraph .github/PULL_REQUEST_TEMPLATE.md; the provenance table follows superpowers 7.0.0 (MIT) -->
## Who produced this change (required)
We weigh a change by what produced it and who read it. Agents are welcome; say which one.

| Field | Value |
|-------|-------|
| Model + version | |
| Harness + version | |
| Plugins installed | |
| Human who reviewed this diff | |

## What and why

## Which refusal or lens does this touch
Name the refusal code (`docs/plans/2026-10-23-v0-1-the-engine.md`, "The refusals") or the lens, or write "none".
Which test went red first, and where is it?

## Checklist
- [ ] Tests written first (a failing test existed before the fix)
- [ ] A new refusal has a test that was seen red first
- [ ] Fixture touched? If yes: the answer key in `fixtures/README.md` still matches the site
- [ ] Decision row added or updated in `docs/decisions.md` for any provisional choice, and for any change to the law
      (evidence classes, strata, refusals, human classes)
- [ ] No model call, network call, API key or new dependency entered the engine
- [ ] `cargo test -p personajury` and `cargo clippy -p personajury --tests -- -D warnings` pass locally
