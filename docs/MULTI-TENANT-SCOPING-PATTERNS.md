# Multi-Tenant Scoping Verification Patterns

**Document:** Internal patterns guide for multi-tenant access control  
**Updated:** 2026-04-24 (KF-002 audit)  
**Status:** Production v0.1 Reference

---

## Table of Contents

1. [Overview](#overview)
2. [Pattern 1: Middleware-Based Tenant Extraction](#pattern-1-middleware-based-tenant-extraction)
3. [Pattern 2: JWT-Embedded Tenant ID](#pattern-2-jwt-embedded-tenant-id)
4. [Pattern 3: Repository-Level Filtering](#pattern-3-repository-level-filtering)
5. [Pattern 4: Defensive Validation](#pattern-4-defensive-validation)
6. [Pattern 5: Lock Ordering for Multi-Statement Transactions](#pattern-5-lock-ordering-for-multi-statement-transactions)
7. [Anti-Patterns to Avoid](#anti-patterns-to-avoid)
8. [Testing Multi-Tenant Scoping](#testing-multi-tenant-scoping)
9. [Automation Opportunities](#automation-opportunities)

---

## Overview

**Tenant Context:** `company_id` in Kesh application

**Tenant Lifecycle:**
```
User created → assigned to company → JWT includes company_id
    ↓
API request arrives with JWT
    ↓
Middleware extracts company_id from JWT
    ↓
Handler receives Extension(current_user) with company_id
    ↓
All database queries filtered by company_id
    ↓
Response contains only authorized company's data
```

**Key Principle:** Tenant ID is **immutable and externally verified** for each request.

---

## Pattern 1: Middleware-Based Tenant Extraction

**File:** `crates/kesh-api/src/middleware/auth.rs`

### How It Works

```rust
// Step 1: Define CurrentUser struct with tenant info
#[derive(Clone, Debug)]
pub struct CurrentUser {
    pub user_id: i64,
    pub company_id: i64,  // ← TENANT ID
    pub role: UserRole,
}

// Step 2: Middleware extracts from JWT
pub async fn auth_middleware(
    req: Request,
    next: Next,
) -> Result<Request, AppError> {
    let token = extract_token_from_header(&req)?;
    let claims = verify_and_decode_jwt(token)?;  // Validates signature
    
    let current_user = CurrentUser {
        user_id: claims.sub,
        company_id: claims.company_id,
        role: claims.role,
    };
    
    // Step 3: Add to request extensions
    request.extensions_mut().insert(current_user);
    Ok(request)
}

// Step 4: Handlers extract from extensions
pub async fn handler(
    Extension(current_user): Extension<CurrentUser>,
    // ...
) -> Result {
    // current_user.company_id is now available
}
```

### Invariant

- **JWT is signed with secret:** Prevents forgery
- **Tenant ID cannot be modified:** Only backend can issue JWT
- **Every request validated:** Middleware runs before handler

### When to Use

✅ Use this pattern when:
- Handler needs to know which company owns the request
- Database operations require company scoping
- User permissions are per-company

❌ Don't use when:
- Endpoint is truly public (health check, i18n)
- Endpoint handles user authentication (login endpoint)

---

## Pattern 2: JWT-Embedded Tenant ID

**File:** `crates/kesh-api/src/auth/jwt.rs`

### How It Works

```rust
// At login time
pub fn encode(
    user_id: i64,
    role: UserRole,
    company_id: i64,  // ← Embed company_id in JWT
    secret: &[u8],
    expiry: Duration,
) -> Result<String, AppError> {
    let claims = Claims {
        sub: user_id,
        company_id,  // ← Stored in JWT
        role,
        iat: Utc::now(),
        exp: Utc::now() + expiry,
    };
    
    encode_and_sign(claims, secret)
}

// At request time, middleware decodes
let claims = decode_and_verify(token, secret)?;
let company_id = claims.company_id;  // ← Extracted and used
```

### Invariant

- **Tenant ID immutable during request:** Same company for all database operations
- **Signed JWT prevents tampering:** Cannot change company_id without signature
- **Expires after duration:** Default 15 minutes (see config)

### Staleness Window

**Issue:** If user is reassigned to a different company during an active session, the JWT will still contain the old company_id until token expires or is refreshed.

**Mitigation:** 
- JWT expiry: 15 minutes default
- Refresh token: Database-backed with revocation support
- Defensive check: `get_company_for()` verifies company still exists

**Example timeline:**
```
13:00 — User logs in with company_id=42 (JWT issued)
13:05 — Admin reassigns user to company_id=99
13:07 — User makes API request with JWT from 13:00 (company_id=42)
        ↓ Middleware extracts company_id=42 from JWT
        ↓ `get_company_for()` validates company exists
        ↓ User can still see company 42's data (stale for 8 minutes)
13:15 — JWT expires, user must refresh (database check detects new company)
```

**Risk Level:** LOW — 15 minute window acceptable for typical SaaS

---

## Pattern 3: Repository-Level Filtering

**File:** `crates/kesh-db/src/repositories/*.rs`

### How It Works

```rust
// CORRECT: Always include company_id in WHERE clause
pub async fn find_invoice_by_id(
    pool: &MySqlPool,
    company_id: i64,  // ← Explicit parameter
    invoice_id: i64,
) -> Result<Option<Invoice>, DbError> {
    sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE company_id = ? AND id = ?"
    )
    .bind(company_id)
    .bind(invoice_id)
    .fetch_optional(pool)
    .await
}

// WRONG: Missing company_id filter
pub async fn find_invoice_by_id_broken(
    pool: &MySqlPool,
    invoice_id: i64,
) -> Result<Option<Invoice>, DbError> {
    sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE id = ?"  // ← No company_id!
    )
    .bind(invoice_id)
    .fetch_optional(pool)
    .await
}

// CORRECT USAGE: Handler passes company_id
pub async fn get_invoice_handler(
    Extension(current_user): Extension<CurrentUser>,
    Path(invoice_id): Path<i64>,
) -> Result {
    invoices::find_invoice_by_id(
        &state.pool,
        current_user.company_id,  // ← Always passed
        invoice_id,
    )
    .await?
}
```

### Invariant

- **company_id is explicit parameter:** Cannot be forgotten
- **Repository functions never access global state:** Avoids "current company" gotchas
- **Queries are stateless:** Same function works in any context

### When to Use

✅ Use this pattern:
- All SELECT queries on company data
- All UPDATE/DELETE queries on company data
- JOIN queries involving company-scoped tables

❌ Don't use for:
- Global metadata (VAT rates, system config)
- User-unrelated tables

---

## Pattern 4: Defensive Validation

**File:** `crates/kesh-api/src/helpers.rs`

### How It Works

```rust
pub async fn get_company_for(
    current_user: &CurrentUser,
    pool: &MySqlPool,
) -> Result<Company, AppError> {
    // Defensive check: verify company exists
    // This catch cases where:
    // 1. Company was deleted during user's session
    // 2. User was reassigned between companies
    // 3. JWT is stale or tampered
    
    companies::find_by_id(pool, current_user.company_id)
        .await?
        .ok_or(AppError::Forbidden)  // ← Clear error semantics
}

// Usage in handler
pub async fn list_invoices(
    Extension(current_user): Extension<CurrentUser>,
) -> Result {
    // Defensive: verify company exists before querying invoices
    let company = get_company_for(&current_user, &state.pool).await?;
    
    // Now safe to query invoices with company.id
    invoices::list_by_company(&state.pool, company.id).await?
}
```

### Invariant

- **Every handler validates company exists** — Catches edge cases
- **Fast failure:** If company missing, error returned immediately
- **Clear error:** `Forbidden` indicates permission issue, not data absence

### Benefits

1. **Catches stale JWT:** If company deleted, request fails clearly
2. **Catches reassignment:** If user moved to different company, doesn't get stale data
3. **Explicit not implicit:** Handler code clearly shows the validation

---

## Pattern 5: Lock Ordering for Multi-Statement Transactions

**Files:** `crates/kesh-api/src/routes/onboarding.rs`, any handler taking multiple `SELECT FOR UPDATE` locks

### Why Lock Ordering Matters

When a transaction holds multiple row-level locks (`SELECT ... FOR UPDATE`), two concurrent transactions acquiring the same locks **in reverse order** form a deadlock cycle. InnoDB **detects** such a cycle at wait time, whether it spans one table or several (`innodb_deadlock_detect = ON`, the default — measured `1` on MariaDB 10.11.16), and immediately rolls back a victim transaction, which receives error 1213 (`ER_LOCK_DEADLOCK`) without waiting for `innodb_lock_wait_timeout`. Without a replay, that victim is a 500 for the user. **The defence is the replay of the route** (`kesh_db::retry`, `kesh_api::retry`, Story 15-5e1); a consistent lock order only **reduces the frequency** of such cycles. The canonical lock orders are written in the doc-comments of `invoices::validate_invoice` and `supplier_invoices::create_in_tx`; the module doc of `crates/kesh-db/src/retry.rs` describes what InnoDB does (and does not: a cycle through a `GET_LOCK` named lock is not detected).

### Global Lock Order — a frequency convention, not a guarantee

**Transactions that lock more than one row SHOULD acquire locks in this order:**

```
1. onboarding_state  (singleton row, taken first)
2. companies         (target company row — sentinel `SELECT id FROM companies WHERE id=? FOR UPDATE`)
3. projects          (analytical project rows for the target company — Epic 19, Story 19-3)
4. accounts          (account rows for the target company)
5. company_invoice_settings  (settings row for the target company)
```

Rationale: this matches the natural dependency direction (state machine → tenant → tenant data → tenant settings), and following it **reduces the frequency** of deadlocks. It **cannot exclude** them (Story 15-5e1, choice C54), for three reasons:

- **Shared locks taken at insertion.** Every flow that writes to the journal takes, when it inserts the entry and its lines, **shared** locks through the foreign keys — `fk_journal_entries_company` on the `companies` row, `fk_jel_account` on each account written, `fk_jel_project` on each project tagged — **after** the fiscal year lock. No "accounts before fiscal year" order holds end to end.
- **Flows that take the fiscal year first.** The rule batch proposal (`accept_one_rule`: fiscal year, then the `companies` sentinel and the rule's project through `validate_taggable_in_tx`), the split batch proposal (`accept_one_split`: fiscal year, then step 0 of `create_in_tx` on the lines' projects), the manual match with a project (`post_manual`: fiscal year at step 6, then sentinel and project at step 6bis) and the split match (`post_split`: fiscal year, then step 0 of `create_in_tx`) all take **fiscal year, then sentinel and projects** — the reverse of `journal_entries::create` (step 0, then step 1), of `create_opening_entry` (sentinel, then fiscal year) and of the supplier invoice with a project.
- **Flows that lock their own row first.** The `PUT` and `DELETE` of journal entries lock the entry before anything else (see their rows below).

**The defence is the replay.** Every route that writes to the journal is replayed on deadlock by an envelope — `kesh_db::retry::retry_on_deadlock` (error `DbError`) or `kesh_api::retry::retry_app_on_deadlock` (error `AppError`) —, and the route registry (`crates/kesh-api/tests/audit_route_registry.rs`) keeps the closed list of those routes and checks that each one calls an envelope.

### Where This Applies

| Endpoint | Lock sequence | File |
|----------|---------------|------|
| `POST /onboarding/finalize` | onboarding_state → company → accounts → settings → fiscal_years (auto-create via `create_if_absent_in_tx`) | `routes/onboarding.rs` finalize |
| `POST /onboarding/coordinates`, `/org-type`, `/accounting-language` | company only (single lock, safe) | same |
| `POST /onboarding/reset` | onboarding_state (gate-check only — released before reset_demo) | `routes/onboarding.rs` reset |
| `kesh_seed::seed_demo` | companies (count-validation only — released before destructive ops) | `kesh-seed/src/lib.rs` |
| `POST /supplier-invoices` (create) | see the canonical doc-comment of `supplier_invoices::create_in_tx` (« Ordre des verrous ») — the order is written there only | `repositories/supplier_invoices.rs::create_in_tx` (Story 19-3, 15-5e1) |
| `POST /projects/*` (create/update/archive/unarchive) | companies (sentinel) → projects | `routes/projects.rs` (Story 19-1) |
| `fiscal_years::create / update_name / close / find_*_locked` | fiscal_years only (single table, internal tx). **`close`** (Story 15-12a, #543) : exercices antérieurs lus sans verrou puis verrouillés **un par un par clé primaire** (`start_date` croissant — ordre fixé par le code), puis l'exercice, puis relecture verrouillante des antérieurs ouverts ; **`create`** : pré-contrôles (chevauchement, nom) puis exercices postérieurs clos (`find_later_closed_in_tx`) ; **`update_name`** : l'exercice puis son homonyme ; **`find_open_covering_date`** : parcours, ordre du plan. ⚠️ Cycles **clôture / contre-passation** et **création / contre-passation** (et les quatre annulations qui portent la contre-passation) : celle-ci tient l'exercice d'une origine puis parcourt depuis le premier exercice — résolus par le **rejeu des deux côtés** (Pattern 5 : l'ordre réduit la fréquence, le rejeu est la défense ; `create_fiscal_year` et `close_fiscal_year` sont rejouées depuis la 15-12a). | `kesh-db/src/repositories/fiscal_years.rs` |
| `invoices::validate_invoice` | see the canonical doc-comment of `invoices::validate_invoice` (« Ordre des locks ») — the order is written there only | `kesh-db/src/repositories/invoices.rs` |
| `POST /credit-notes` | see the doc-comment of `credit_notes::create_credit_note` (« Ordre des locks ») — the order is written there only; all the accounts the credit note writes are locked **in share mode, by id, before the fiscal year** (Story 15-6a); replayed on deadlock (Story 15-5e2) | `repositories/credit_notes.rs::create_credit_note` |
| `POST /reconciliation/accept` (`accept_batch`) | **Per proposal**: rounding account (`rounding_account_for_write`), then fiscal year (`find_open_covering_date`) for an invoice proposal (`accept_one_invoice`); fiscal year, then sentinel and projects for a rule or split proposal (`accept_one_rule`, `accept_one_split`). **Between proposals**, in the same transaction (one savepoint per proposal): the fiscal year held by proposal *n* precedes the rounding account, the sentinel or the projects of proposal *n+1* — **fiscal year → rounding account** at the batch level, the reverse of `invoices::validate_invoice`: that is the cycle of #536, closed by replaying both sides | `routes/reconciliation.rs` |
| `PUT /api/v1/journal-entries/{id}` (Story 15-8a, #532) | **journal_entries → [companies → projects] → fiscal_years (the entry's year, then later years by `start_date`) → (accounts, shared, via the FKs of the line `INSERT`)** | `repositories/journal_entries.rs::update` |
| `DELETE /api/v1/journal-entries/{id}` (Story 15-8b, #532) | **journal_entries + fiscal_years (the entry and its year, one joined `FOR UPDATE`) → fiscal_years (later years by `start_date`) → (FK checks of the `DELETE` on the rows that reference the entry)**, no `INSERT` but the audit row | `repositories/journal_entries.rs::delete_in_tx` |

**Notes:**

- **`PUT /journal-entries/{id}`** — The entry's `FOR UPDATE` must be the **first act** of the transaction: under `REPEATABLE READ`, any plain read before it would freeze a read view older than the wait, and the guard would miss a reversal committed meanwhile (`journal_entries::update`, doc-comment « Sérialisation »). Locking the entry before `companies → projects` reverses the order of creation (a frequency convention, cf. Global Lock Order). Three **inherited** cycles remain, each also valid for every later fiscal year the PUT locks: **fiscal year ↔ account** (customer settlement, opening complement), **fiscal year ↔ companies** (creation / reversal / settlement inserting a header), **project ↔ fiscal year** (reversal of an entry of the same year carrying the project). Mitigation: replayed (`retry_on_deadlock`, `"journal_entries::update"`) — the repository opens and closes its own transaction, the deadlock rolled it back, and the `version` check refuses a second pass. ⚠️ A cycle InnoDB does not detect ends in `innodb_lock_wait_timeout` (1205, **not** retried) → 500. Tested: `update_and_a_reversal_of_the_same_year_can_deadlock` (`kesh-db/tests/journal_entries_modification.rs`).
- **`DELETE /journal-entries/{id}`** — Same first-act rule as the `PUT`: the joined `FOR UPDATE` comes before any plain read, so the guard sees a reversal or a later-year close committed while it waited (`journal_entries::delete_in_tx`, doc-comment « Sérialisation »). It takes neither the `companies` sentinel, nor a project, nor an account. **No known cycle.** Mitigation: replayed (`retry_on_deadlock`, `"journal_entries::delete"`) **by uniformity with the `PUT`**, not for a known cycle — the transaction is replayed whole and the deadlock rolled it back. Tested: `delete_waits_for_a_concurrent_reversal_then_refuses`, `delete_waits_for_a_concurrent_close_of_a_later_year_then_refuses` (`kesh-db/tests/journal_entries_modification.rs`).

### Known Risk — KF-002-H-002 (resolved 2026-05-03)

**Issue:** `seed_demo` and `reset` use a **lock-and-release** pattern: they acquire `FOR UPDATE` only for count-validation (seed_demo) or gate-check (reset), then **commit before** the destructive sub-operation runs (`bulk_create_from_chart`, `companies::update`, `reset_demo`). The lock therefore serializes only the precondition check, NOT the side-effect. A concurrent endpoint running between commit and side-effect can leave inconsistent state visible (handled via `DbError::NotFound`/`OptimisticLockConflict` retries today). Additionally, if a future endpoint takes locks in `accounts → company → onboarding_state` order (reverse), it can deadlock against `finalize`.

**Mitigation:** the lock order above is a convention that reduces the frequency of deadlocks; the defence is the replay. Every route that writes to the journal is replayed by an envelope, and the route registry (`crates/kesh-api/tests/audit_route_registry.rs`) holds the closed list of those routes, checks that each one calls an envelope, and forbids a direct call to the `retry_with` primitive in `src/routes/` outside `post_accept`. A new route that writes to the journal is added to the registry, replayed.

**Resolution status:**

- ✅ **Deadlock-retry envelopes** (`crates/kesh-db/src/retry.rs`, `crates/kesh-api/src/retry.rs`) — catch `ER_LOCK_DEADLOCK` (1213, **not** 1205 `lock_wait_timeout`) and retry with exponential backoff (50 → 100 ms between attempts 1↔2 and 2↔3; max 3 attempts → ≈ 150 ms added latency worst case): `retry_on_deadlock` for a route whose write is one repository function, `retry_app_on_deadlock` for a route whose transaction is opened in the handler (`finalize` among them). The route registry lists the replayed routes. [Fix issue #43; Stories 15-5e1, 15-5e2]

**How to use the retry helper for new endpoints:**

```rust
// Route whose transaction is opened in the handler (`Result<_, AppError>`):
kesh_api::retry::retry_app_on_deadlock("module::operation", || {
    let pool = state.pool.clone();
    async move { handler_inner(&pool, /* args */).await }
})
.await

// Route whose write is one repository function (`Result<_, DbError>`):
kesh_db::retry::retry_on_deadlock("module::operation", || {
    let new = new.clone();
    async move { repository::create(&pool, new).await }
})
.await

// Generic form — the first argument names the operation, carried by the
// `warn!` (target `kesh_db::retry`) emitted before each new attempt:
kesh_db::retry::retry_with(
    "module::operation",
    kesh_db::retry::DEFAULT_MAX_DEADLOCK_ATTEMPTS,
    kesh_api::retry::is_app_deadlock,
    || { /* … */ },
).await
```

Required: a deadlock (1213) rolls back the **whole** victim transaction — counters and audit rows included —, so replaying it is safe **as long as every write lives in that transaction**. The rule that makes an attempt replayable lives in the doc-comment of `kesh_db::retry::retry_on_deadlock` and in the module doc-comment of `kesh_api::retry` (Story 15-5e1) — read it there; Pattern 5 does not restate it, so that no third copy has to be kept true. ⚠️ The generic `retry_with` form above is the primitive the envelopes call: in `src/routes/`, only `post_accept` (predicate widened to 1305) may call it directly — the route registry fails otherwise.

### When to Use

✅ **The envelope applies to every route that writes to the journal** — whatever the number of tables it locks; the route registry checks it.

✅ **The lock order remains a frequency convention** for any transaction that:
- Calls `SELECT ... FOR UPDATE` on more than one table
- Calls a helper function that itself locks (transitive locking)
- Calls a repository fn whose internal locks are not documented (audit it before extending)

❌ Single-row locks don't need the order — but document the lock acquisition site so reviewers can spot it later.

### Code Reference

```rust
// CORRECT: lock in documented order (reduces the frequency of deadlocks)
async fn finalize() -> Result<...> {
    let mut tx = pool.begin().await?;
    let state = sqlx::query_as!("SELECT ... FROM onboarding_state ... FOR UPDATE")  // 1st
        .fetch_one(&mut *tx).await?;
    let company = sqlx::query_as!("SELECT ... FROM companies ORDER BY id LIMIT 1 FOR UPDATE")  // 2nd
        .fetch_one(&mut *tx).await?;
    insert_with_defaults_in_tx(&mut tx, company.id).await?;  // 3rd: locks accounts internally
    tx.commit().await?;
}
```

```rust
// WRONG: reverse order makes a deadlock against finalize far more likely
async fn bad_handler() -> Result<...> {
    let mut tx = pool.begin().await?;
    let accounts = sqlx::query!("SELECT ... FROM accounts WHERE ... FOR UPDATE")  // accounts FIRST
        .fetch_all(&mut *tx).await?;
    let state = sqlx::query!("SELECT ... FROM onboarding_state ... FOR UPDATE")  // onboarding_state SECOND
        .fetch_one(&mut *tx).await?;
    // Deadlock cycle: this tx holds accounts, finalize() holds onboarding_state, both wait —
    // InnoDB rolls back one of them (1213); only a replay hides it from the user
}
```

---

## Anti-Patterns to Avoid

### ❌ Anti-Pattern 1: Global Company Context

```rust
// WRONG: Implicit global state
thread_local! {
    static CURRENT_COMPANY: RefCell<Option<i64>> = RefCell::new(None);
}

pub async fn list_invoices() -> Result {
    let company_id = CURRENT_COMPANY.with(|c| c.borrow().clone())?;
    // ↓ Easy to forget to set CURRENT_COMPANY
    // ↓ Concurrency issues in async
    invoices::list_by_company(&pool, company_id).await
}
```

**Why bad:**
- Hidden dependency in code
- Easy to miss initialization
- Not thread-safe with async
- Impossible to test in isolation

**Correct approach:**
```rust
pub async fn list_invoices(
    Extension(current_user): Extension<CurrentUser>,
) -> Result {
    // ✅ Explicit tenant parameter
    invoices::list_by_company(&pool, current_user.company_id).await
}
```

### ❌ Anti-Pattern 2: SQL Concatenation

```rust
// WRONG: String concatenation (SQL injection risk)
let query = format!(
    "SELECT * FROM invoices WHERE company_id = {}",
    company_id  // ← Not parameterized!
);
sqlx::query_as::<_, Invoice>(&query).fetch_one(pool).await
```

**Why bad:**
- SQL injection vulnerability
- No type safety
- Sqlx compile-time checks bypassed

**Correct approach:**
```rust
// ✅ Parameterized query
sqlx::query_as::<_, Invoice>(
    "SELECT * FROM invoices WHERE company_id = ?"
)
.bind(company_id)
.fetch_one(pool)
.await
```

### ❌ Anti-Pattern 3: Trusting User Input

```rust
// WRONG: Using company_id from query parameter
#[derive(Deserialize)]
pub struct ListRequest {
    company_id: i64,  // ← User-provided!
}

pub async fn list_invoices(
    Json(req): Json<ListRequest>,
) -> Result {
    // What if user provides company_id=999 (not their company)?
    invoices::list_by_company(&pool, req.company_id).await
}
```

**Why bad:**
- User can query ANY company
- No authentication check
- IDOR vulnerability

**Correct approach:**
```rust
pub async fn list_invoices(
    Extension(current_user): Extension<CurrentUser>,  // ← From JWT
) -> Result {
    // company_id comes from authenticated JWT, not user input
    invoices::list_by_company(&pool, current_user.company_id).await
}
```

### ❌ Anti-Pattern 4: Inconsistent Error Handling

```rust
// WRONG: Different error types leak information
let invoice = invoices::find_by_id(&pool, company_id, invoice_id).await?;
if invoice.company_id != current_user.company_id {
    return Err(AppError::Forbidden);  // ← Reveals it exists
}
```

**Why bad:**
- Attacker learns whether resource exists
- Different error codes for "not found" vs "forbidden"
- Enables enumeration attacks

**Correct approach:**
```rust
// ✅ Repository handles filtering, handler doesn't need check
let invoice = invoices::find_by_id(
    &pool,
    current_user.company_id,  // ← Filter at DB level
    invoice_id,
).await?
.ok_or(AppError::NotFound)?  // ← Same error for both cases
```

---

## Testing Multi-Tenant Scoping

### Unit Test Pattern

```rust
#[tokio::test]
async fn test_invoice_list_scoped_by_company() {
    let pool = setup_test_db().await;
    
    // Setup: Create two companies with invoices
    let company_1 = create_test_company(&pool, "Company A").await;
    let company_2 = create_test_company(&pool, "Company B").await;
    
    let invoice_1 = create_test_invoice(&pool, company_1.id, "INV-001").await;
    let invoice_2 = create_test_invoice(&pool, company_2.id, "INV-002").await;
    
    // Test: List invoices for company_1 should NOT include company_2's invoices
    let results = invoices::list_by_company(&pool, company_1.id).await.unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, invoice_1.id);
    assert!(!results.iter().any(|i| i.id == invoice_2.id));  // ← Verify isolation
}
```

### Integration Test Pattern

```rust
#[tokio::test]
async fn test_api_endpoint_returns_only_authorized_company_data() {
    // Setup: Create test app with two companies
    let app = create_test_app().await;
    let company_1 = app.create_company("Company A").await;
    let company_2 = app.create_company("Company B").await;
    
    let user_1 = app.create_user("user1", company_1.id).await;
    let user_2 = app.create_user("user2", company_2.id).await;
    
    let invoice_1 = app.create_invoice(company_1.id, "INV-001").await;
    let invoice_2 = app.create_invoice(company_2.id, "INV-002").await;
    
    // Test: user_1 (company_1) cannot see company_2's invoices
    let response = app
        .get_as_user("/api/v1/invoices", user_1)
        .await;
    
    assert_eq!(response.status(), 200);
    let body: ListResponse = serde_json::from_str(&response.body()).unwrap();
    
    // ← Verify tenant isolation
    assert_eq!(body.items.len(), 1);
    assert_eq!(body.items[0].id, invoice_1.id);
}
```

---

## Automation Opportunities

### Opportunity 1: Query Builder with Automatic Scoping

**Current state:** Developers must remember `WHERE company_id = ?`

**Proposed:** Compile-time-enforced scoping

```rust
// Instead of raw SQL:
sqlx::query("SELECT * FROM invoices WHERE company_id = ? AND status = ?")
    .bind(company_id)
    .bind("draft")

// Could use builder pattern:
Query::new("invoices")
    .for_company(company_id)  // ← Compiler-enforced
    .where_eq("status", "draft")
    .fetch_all(&pool)
    .await
```

**Benefit:** Impossible to forget company scoping

### Opportunity 2: Automatic Repository Generation

**Current state:** Each repository manually implements company filtering

**Proposed:** Derive macros that auto-generate scoped queries

```rust
#[derive(Repository)]
#[repository(table = "invoices", company_scoped = true)]
struct InvoiceRepository;

// Generates:
// - find_by_id(pool, company_id, id)
// - list(pool, company_id)
// - create(pool, company_id, new_invoice)
// - update(pool, company_id, id, changes)
// - delete(pool, company_id, id)
```

**Benefit:** Consistent scoping across all repositories

### Opportunity 3: Middleware Assertion

**Current state:** Developers trust that company_id is correct

**Proposed:** Runtime assertions that verify scoping

```rust
#[derive(ScopedQuery)]
#[assert_company_id = true]  // Middleware enforces WHERE company_id = ?
async fn find_invoice(
    Extension(current_user): Extension<CurrentUser>,
    Path(id): Path<i64>,
) -> Result {
    let invoice = invoices::find_by_id(&pool, current_user.company_id, id).await?;
    // ↑ Middleware verifies that company_id was actually used
}
```

**Benefit:** Catches missing scoping at runtime

---

## Conclusion

**Multi-tenant scoping in Kesh follows these core principles:**

1. **Tenant ID from JWT** — Immutable, verified by signature
2. **Explicit parameter passing** — Developers cannot forget
3. **Repository-level filtering** — Database always includes WHERE company_id
4. **Defensive validation** — Every handler double-checks company exists
5. **Clear error semantics** — 403 Forbidden for access denial

**For future stories:**
- Consider automation opportunities (query builder, macros)
- Keep patterns documented in this file
- Audit new endpoints against these patterns
- Add integration tests for multi-tenant scoping

---

**Document Version:** 1.0  
**Last Updated:** 2026-04-24  
**Next Review:** After v0.1 release
