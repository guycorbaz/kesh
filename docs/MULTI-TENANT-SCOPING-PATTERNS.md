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
    pool: &PgPool,
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
    pool: &PgPool,
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
    pool: &PgPool,
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

When a transaction holds multiple row-level locks (`SELECT ... FOR UPDATE`), MariaDB does not detect cross-table deadlocks proactively. Two concurrent transactions acquiring the same locks **in reverse order** will deadlock until `innodb_lock_wait_timeout` (50s default) elapses, returning a 500 to the user.

### Global Lock Order (v0.1)

**All transactions that lock more than one row MUST acquire locks in this order:**

```
1. onboarding_state  (singleton row, taken first)
2. companies         (target company row)
3. accounts          (account rows for the target company)
4. company_invoice_settings  (settings row for the target company)
```

Rationale: this matches the natural dependency direction (state machine → tenant → tenant data → tenant settings). Reverse-order acquisition creates a deadlock cycle.

### Where This Applies

| Endpoint | Lock sequence | File |
|----------|---------------|------|
| `POST /onboarding/finalize` | onboarding_state → company → accounts → settings → fiscal_years (auto-create via `create_if_absent_in_tx`) | `routes/onboarding.rs` finalize |
| `POST /onboarding/coordinates`, `/org-type`, `/accounting-language` | company only (single lock, safe) | same |
| `POST /onboarding/reset` | onboarding_state (gate-check only — released before reset_demo) | `routes/onboarding.rs` reset |
| `kesh_seed::seed_demo` | companies (count-validation only — released before destructive ops) | `kesh-seed/src/lib.rs` |
| `fiscal_years::create / update_name / close / find_*_locked` | fiscal_years only (single table, internal tx) — `FOR UPDATE` locks pour pré-check unicité/overlap et figer le before-snapshot d'audit log. Pas de chaîne cross-table. | `kesh-db/src/repositories/fiscal_years.rs` |
| `invoices::validate_invoice` | invoices → fiscal_years (via `find_open_covering_date`) → invoice_number_sequences → journal_entries | `kesh-db/src/repositories/invoices.rs` |

### Known Risk — Tracked as KF-002-H-002

**Issue:** `seed_demo` and `reset` use a **lock-and-release** pattern: they acquire `FOR UPDATE` only for count-validation (seed_demo) or gate-check (reset), then **commit before** the destructive sub-operation runs (`bulk_create_from_chart`, `companies::update`, `reset_demo`). The lock therefore serializes only the precondition check, NOT the side-effect. A concurrent endpoint running between commit and side-effect can leave inconsistent state visible (handled via `DbError::NotFound`/`OptimisticLockConflict` retries today). Additionally, if a future endpoint takes locks in `accounts → company → onboarding_state` order (reverse), it can deadlock against `finalize`. No deadlock-detection retry is implemented; failures surface as 500 after `innodb_lock_wait_timeout`.

**Mitigation (v0.1):** all current write endpoints follow the documented order. New endpoints **MUST** follow it or be added to a deny list. Under single-tenant single-user the lock-and-release race window is unreachable in practice.

**Resolution plan (v0.2):**
- Add a deadlock-retry middleware that catches `ER_LOCK_DEADLOCK` (1213) and retries with exponential backoff (max 3 attempts)
- Document any endpoint that intentionally diverges from the global order
- Add CI check that grep-detects `FOR UPDATE` patterns and verifies lock order via static analysis

### When to Use

✅ Apply this rule when a transaction:
- Calls `SELECT ... FOR UPDATE` on more than one table
- Calls a helper function that itself locks (transitive locking)
- Calls a repository fn whose internal locks are not documented (audit it before extending)

❌ Single-row locks don't need this discipline — but document the lock acquisition site so reviewers can spot it later.

### Code Reference

```rust
// CORRECT: lock in documented order
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
// WRONG: reverse order will deadlock against finalize
async fn bad_handler() -> Result<...> {
    let mut tx = pool.begin().await?;
    let accounts = sqlx::query!("SELECT ... FROM accounts WHERE ... FOR UPDATE")  // accounts FIRST
        .fetch_all(&mut *tx).await?;
    let state = sqlx::query!("SELECT ... FROM onboarding_state ... FOR UPDATE")  // onboarding_state SECOND
        .fetch_one(&mut *tx).await?;
    // Deadlock cycle: this tx holds accounts, finalize() holds onboarding_state, both wait
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
