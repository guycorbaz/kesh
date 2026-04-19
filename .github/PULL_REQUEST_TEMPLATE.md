<!--
Thanks for contributing to Kesh!

Title format (Conventional Commits):
  <type>(<scope>): <short summary>
  e.g.   feat(story-5.4): échéancier factures
         fix(kesh-qrbill): align QRR mod-10 table with SIX 2.2

Types: feat | fix | refactor | docs | test | ci | chore | perf | style
-->

## Summary

<!-- One or two sentences. What does this PR do and why? -->

## Related issue / story

<!-- Closes #NNN, or "Story 5-4 — review pass 2" -->

## Changes

<!-- Bulleted list of the main changes. Group by area if useful. -->

-

## Test plan

<!-- How to verify this PR works. Include commands. -->

- [ ] `cargo fmt --check` clean
- [ ] `cargo clippy -- -D warnings` clean
- [ ] `cargo test` passes (or note known-failures referenced)
- [ ] `cd frontend && npm run check` clean (svelte-check)
- [ ] `cd frontend && npm run test:unit -- --run` passes
- [ ] Playwright e2e if UI touched: `cd frontend && npx playwright test`
- [ ] Manually tested in browser if UI changed
- [ ] i18n: 4 locales updated (fr-CH, de-CH, it-CH, en-CH) if user-facing strings added

## Spec / AC compliance

<!-- If this implements or modifies a story:
     - Which AC does this satisfy?
     - Any documented deviations from spec? Reference Change Log entry.
-->

## Known failures referenced

<!-- If this PR depends on or relates to entries in docs/known-failures.md, list them.
     If it RESOLVES a KF, mark it: "Resolves KF-NNN". -->

## Breaking changes

<!-- API contract changes, migration impact, config changes. -->

- [ ] None
- [ ] DB migration included (irreversible additive changes documented)
- [ ] API contract changed (frontend/clients updated)
- [ ] Config / env vars changed (docs updated)

## Checklist

- [ ] Title follows Conventional Commits
- [ ] Branch is up to date with `main`
- [ ] No secrets / credentials in diff
- [ ] No unrelated changes (unrelated fixes go in separate PRs)
- [ ] Documentation updated (if user-facing or API change)
