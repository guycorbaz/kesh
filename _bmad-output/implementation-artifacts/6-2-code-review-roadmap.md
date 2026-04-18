---
title: Story 6.2 Code Review — Roadmap & Next Steps
date: 2026-04-18
status: SCENARIO_A_CHOSEN
---

# Story 6.2 Code Review — Implementation Roadmap

**User Choice:** Scenario A — Fix CRITICAL + HIGH before merge  
**Expected Effort:** ~4-5 hours patches + ~180 min pass 2 review  
**Timeline:** ~1-2 days total (patches today, pass 2 tomorrow if batching reviews)

---

## PHASE 1️⃣ : Appliquer les patches (Maintenant)

### Step 1A: Apply CRITICAL patches (~20 minutes)

**Patch C1** — Add SIGNAL to migration guard  
- File: `crates/kesh-db/migrations/20260419000002_users_company_id.sql`
- Lines: After line 27, add:
  ```sql
  IF @should_fail = 1 THEN
    SIGNAL SQLSTATE '45000' SET MESSAGE_TEXT = @error_msg;
  END IF;
  ```

**Patch C2** — Add multi-company guard  
- File: `crates/kesh-db/migrations/20260419000002_users_company_id.sql`
- Lines: After backfill (line 8), add before line 11:
  ```sql
  SET @distinct_companies = (SELECT COUNT(DISTINCT company_id) FROM users WHERE company_id IS NOT NULL);
  IF @distinct_companies > 1 THEN
    SET @should_fail = 1;
    SET @error_msg = 'Backfill detected users in multiple companies. Restore from backup or manually clean.';
  END IF;
  ```

**Patch C3** — Change ON DELETE to CASCADE  
- File: `crates/kesh-db/migrations/20260419000002_users_company_id.sql`
- Line 37: Change `ON DELETE RESTRICT` → `ON DELETE CASCADE`

**Patch C4** — Fix seed_changeme_user_only  
- File: `crates/kesh-db/src/test_fixtures.rs`
- Lines 250-258: Wrap with company creation + bind company_id
- (See detailed fix in 6-2-code-review-pass-1.md, M5 section)

**After C1-C4:**
```bash
git add crates/kesh-db/migrations/20260419000002_users_company_id.sql \
       crates/kesh-db/src/test_fixtures.rs
git commit -m "fix(story-6-2): Apply CRITICAL patches C1-C4 — migration guards, FK cascade, fixture company_id binding"
```

---

### Step 1B: Apply HIGH patches (~3-4 hours, focus on H1)

**Patch H1** — Implement HTTP E2E IDOR tests (150-200 lines, ~3 hours)  
- File: `crates/kesh-api/tests/idor_multi_tenant_e2e.rs`
- Implement full HTTP tests for 6 entities:
  1. contacts (GET/PUT/DELETE) → 404 cross-company
  2. products (GET/PUT/DELETE) → 404 cross-company
  3. invoices (GET/PUT) → 404 cross-company
  4. accounts (GET/PUT/DELETE) → 404 cross-company
  5. users (GET/PUT/DELETE) → 404 cross-company
  6. companies/current (GET) → returns only current company
- Use tower test client pattern (see detailed spec in 6-2-code-review-pass-1.md, H1 section)

**Patch H2** — Bootstrap warn log (~1 minute)  
- File: `crates/kesh-api/src/auth/bootstrap.rs`
- Line 34: Change log level from `info` → `warn`

**Patch H3** (optional) — Refresh audit log (~5 minutes)  
- File: `crates/kesh-api/src/routes/auth.rs`
- Add audit log if company_id changes at refresh

**Patch H4** — Add LOCK TABLES (~2 minutes)  
- File: `crates/kesh-db/migrations/20260419000002_users_company_id.sql`
- Add `LOCK TABLES users WRITE;` before ADD COLUMN
- Add `UNLOCK TABLES;` after MODIFY

**Patch H5** — Bootstrap zero-company test (~15 minutes)  
- File: `crates/kesh-api/src/auth/bootstrap.rs` (test section)
- Add test case for bootstrap with no companies

**After H1-H5:**
```bash
git add crates/kesh-api/tests/idor_multi_tenant_e2e.rs \
       crates/kesh-api/src/auth/bootstrap.rs \
       crates/kesh-api/src/routes/auth.rs \
       crates/kesh-db/migrations/20260419000002_users_company_id.sql
git commit -m "fix(story-6-2): Apply HIGH patches H1-H5 — HTTP IDOR tests, bootstrap logging, migration lock, test coverage"
```

---

### Step 1C: Run tests & CI

```bash
# Run unit tests
cargo test -p kesh-api --lib
cargo test -p kesh-db --lib

# Run integration tests
cargo test -p kesh-api --test '*_e2e'

# Run E2E tests (if setup allows)
npm run test:e2e

# Check CI simulation locally
cargo fmt --check
cargo clippy -- -D warnings
```

Expected: ✅ All tests green (if not, fix regressions)

---

### Step 1D: Create PR & merge

```bash
# Push to remote
git push origin story/6-2-multi-tenant-scoping-refactor

# Create PR (via GitHub or gh CLI) with body including:
gh pr create \
  --title "fix(story-6-2): Multi-tenant scoping refactor — KF-002 closure" \
  --body "## Summary
- Refactor 8 routes to scope by company_id
- Add users.company_id to schema with FK constraint
- Update JWT + CurrentUser to include company_id
- Implement HTTP IDOR tests (6 entities, 404 responses)
- Fix migration guards + FK cascade + test fixtures

Closes #2

## Test Plan
- [ ] All 84+ kesh-db tests green
- [ ] All kesh-api unit/integration tests green
- [ ] All E2E tests green (IDOR, onboarding, refresh)
- [ ] CI checks pass (Backend, Frontend, E2E, Docker)

🤖 Generated with Claude Code / bmad-code-review"

# Merge when CI green
```

---

## PHASE 2️⃣ : Passe 2 de revue (Après patches)

**Timing:** After patches applied and tests green (same day or next morning)  
**Duration:** ~3 hours (180 min agents + triage)  
**Model:** Sonnet 4.6 (different from pass 1 Haiku to avoid author bias)

### Why Pass 2?

Per CLAUDE.md rule:
> Tant qu'une passe de revue remonte **au moins un finding > LOW**, on **relance une nouvelle passe**

Pass 1 findings: 4 CRITICAL + 5 HIGH + 7 MEDIUM = **16 findings > LOW**

→ **Passe 2 required**

### What We're Verifying

Pass 2 will check:
1. **Do patches address original findings?** (C1-C5 fixed, H1-H5 fixed)
2. **Did patches introduce regressions?** (new bugs in migration, tests)
3. **Are there NEW findings from patches?** (edge cases in new test code, etc.)

### Convergence Target

Trend metric (feedback from previous stories 1.3→5-4):
- Pass 1: 18 findings (4C + 5H + 7M + 2L)
- Pass 2 target: ~8-12 findings (mostly M/L, ~1 C/H if regression found)
- Pass 3 (if needed): ~2-4 findings (L only)

---

## CHECKLIST — Before Pass 2

- [ ] All CRITICAL patches C1-C4 applied ✓
- [ ] All HIGH patches H1-H5 applied ✓
- [ ] `cargo test` green ✓
- [ ] CI checks pass ✓
- [ ] PR created with `closes #2` ✓
- [ ] Local branch synced, ready for fresh review ✓

Then: `bmad-code-review` again with Sonnet model

---

## PHASE 3️⃣ : Post-Pass-2 (If needed)

If pass 2 finds additional blockers (unlikely if patches solid):
- **Pass 3** with Opus (next in cycle: Haiku → Sonnet → Opus)
- Or use different review strategy (edge-case focus, security audit focus, etc.)

If pass 2 finds only LOW findings:
- **Review complete** ✅
- Document findings, merge with confidence

---

## FAQ

**Q: Can I skip pass 2?**  
A: CLAUDE.md rule requires pass 2 because findings > LOW exist. Pass 1 was Haiku; pass 2 must be different LLM (Sonnet) to catch author-bias regressions. Budget allows ~8 passes max, you're at 1/8.

**Q: How long will pass 2 take?**  
A: ~3 hours (180 min parallel agents + triage), similar to pass 1.

**Q: What if pass 2 finds more blockers?**  
A: Then pass 3 (Opus). Expectation is convergence (18→8→2 findings). If stuck, escalate to deeper security audit or architectural review.

**Q: Can I merge before pass 2?**  
A: Not recommended per CLAUDE.md, which formalized this pattern from lessons learned in stories 1.3→5-4. Risk of shipping regressions.

---

## ESTIMATED TIMELINE

- **Today (2026-04-18):** Apply patches C1-C4 (20 min) + H1-H5 (3-4 hours) = ~4.5 hours
- **Tonight:** Run tests locally (30 min)
- **Tomorrow (2026-04-19):** Pass 2 review (180 min) + final cleanup (30 min)
- **End of day 2026-04-19:** Merge to main (if pass 2 green)

---

## CONTACT

Questions or blockers during patching? Document them with inline comments and we'll revisit in pass 2.

Generated: 2026-04-18  
Document: `6-2-code-review-roadmap.md`
