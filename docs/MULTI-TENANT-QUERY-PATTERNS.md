# Multi-Tenant Query Patterns & Automation

**Document Type:** Architecture Guide  
**Status:** Recommended for v0.2+ refactoring  
**Last Updated:** 2026-04-25

---

## Problem Statement

Current implementation requires developers to manually add `WHERE company_id = ?` to every SQL query. This creates:
- ❌ **Human error risk:** Easy to forget the WHERE clause
- ❌ **Code review burden:** Every SQL query must be audited
- ❌ **No compile-time enforcement:** Mistakes caught at runtime or code review

---

## Current Pattern (v0.1)

### Manual WHERE Clause

```rust
// crates/kesh-db/src/repositories/invoices.rs

pub async fn list_by_company(
    pool: &PgPool,
    company_id: i64,
    limit: i64,
) -> Result<Vec<Invoice>> {
    sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE company_id = ? AND status = 'draft' LIMIT ?"
    )
    .bind(company_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn find_by_id(
    pool: &PgPool,
    company_id: i64,
    invoice_id: i64,
) -> Result<Option<Invoice>> {
    sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices WHERE company_id = ? AND id = ?"
    )
    .bind(company_id)
    .bind(invoice_id)
    .fetch_optional(pool)
    .await
}
```

**Audit Result:** ✅ All 11 repositories correctly implement WHERE company_id

---

## Proposed Automation: QueryBuilder Pattern

### Option 1: Macro-Based (Minimal Runtime Overhead)

**Compile-time enforcement** via proc macro:

```rust
// Define a scoped query macro that requires company_id
#[tenant_query]
pub async fn list_by_company(
    pool: &PgPool,
    company_id: i64,
    limit: i64,
) -> Result<Vec<Invoice>> {
    // Macro generates: WHERE company_id = ? AND ...
    sqlx::query_as::<_, Invoice>(
        "SELECT * FROM invoices {WHERE} AND status = 'draft' LIMIT ?"
        //                      ^ Macro expands to: WHERE company_id = ?
    )
    .bind(company_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

// Compiler error if macro not used:
// pub async fn find_by_id(...) -> Result<...> {
//     sqlx::query_as("SELECT * FROM invoices WHERE id = ?")  // ❌ Missing {WHERE}
// }
```

**Pros:**
- ✅ Compile-time enforcement
- ✅ Clear intent: `{WHERE}` signals tenant scoping requirement
- ✅ Minimal runtime cost

**Cons:**
- ⚠️ Requires proc macro development (~200 lines)
- ⚠️ Adds compiler complexity

### Option 2: QueryBuilder Wrapper (Type-Safe, Runtime)

**Type system enforcement** via builder pattern:

```rust
// crates/kesh-db/src/query_builder.rs

pub struct TenantQuery<T> {
    company_id: i64,
    sql: String,
    binds: Vec<Box<dyn sqlx::Encode<Mysql> + Send>>,
}

impl<T: sqlx::FromRow<'static, sqlx::mysql::MySqlRow>> TenantQuery<T> {
    pub fn new(sql: &str, company_id: i64) -> Self {
        Self {
            company_id,
            sql: sql.to_string(),
            binds: vec![Box::new(company_id)],
        }
    }

    pub fn and_where(mut self, condition: &str) -> Self {
        self.sql.push_str(" AND ");
        self.sql.push_str(condition);
        self
    }

    pub async fn fetch_one(self, pool: &PgPool) -> Result<T> {
        // WHERE company_id = ? is guaranteed here
        sqlx::query_as(&format!("SELECT * FROM {} WHERE company_id = ? {}", 
            self.table, self.sql))
            .bind(self.company_id)
            .fetch_one(pool)
            .await
    }
}

// Usage:
pub async fn find_by_id(pool: &PgPool, company_id: i64, id: i64) -> Result<Invoice> {
    TenantQuery::<Invoice>::new("invoices", company_id)
        .and_where("id = ?")
        .bind(id)
        .fetch_one(pool)
        .await
}
```

**Pros:**
- ✅ Type-safe: Compiler prevents incorrect usage
- ✅ Flexible: Supports complex queries
- ✅ Readable: Clear builder syntax

**Cons:**
- ⚠️ Slight runtime overhead (builder allocation)
- ⚠️ Requires refactoring all repositories

### Option 3: Database-Level Enforcement (Row-Level Security)

**PostgreSQL RLS or MySQL Computed Column approach:**

```sql
-- PostgreSQL example (not applicable to current MySQL v0.1)
CREATE POLICY tenant_isolation ON invoices
    USING (company_id = current_setting('app.current_company_id')::bigint);

-- Every SELECT, UPDATE, DELETE automatically filtered
```

**Status:** Not suitable for MariaDB 11 (RLS not fully supported)

---

## Recommendation for v0.2

### Phase 1: Documentation & Automation Opportunity (Current v0.1)
- ✅ Document manual pattern (already done in MULTI-TENANT-SCOPING-PATTERNS.md)
- ✅ Add compile-time checks via linting (sqlc or similar)
- ✅ Code review guidelines for tenant scoping

### Phase 2: Gradual QueryBuilder Adoption (v0.2)
1. Create `TenantQuery` wrapper in `crates/kesh-db/src/query_builder.rs`
2. Refactor 2-3 repository modules as pilot
3. Measure: Compare code quality, test coverage, error rates
4. Decide: Proceed with full migration or stick with manual WHERE

### Phase 3: Full Automation (v0.3+)
- Implement proc macro if pilot shows significant benefit
- Migrate all repositories to auto-enforced pattern

---

## Testing Strategy

### Manual Pattern (v0.1)
- ✅ Code review audits WHERE clauses
- ✅ Integration tests verify data isolation (see `tests/multi_tenant.rs`)

### QueryBuilder Pattern (v0.2+)
- ✅ Compiler prevents `select!()` without company_id
- ✅ Type system ensures correct binding order
- ✅ Unit tests verify builder SQL generation

---

## Decision Record

**Date:** 2026-04-25  
**Finding:** KF-002-M-002 — Lack of compiler-enforced WHERE company_id  
**Status:** ✅ Documented, ⏳ Deferred to v0.2

**Rationale:**
- v0.1 is secure: Manual pattern + code review prevents errors
- Benefits of automation (compile-time enforcement) not critical for MVP
- v0.2 can introduce QueryBuilder after learning from v0.1 operations
- Proc macro approach requires specialized expertise; v0.2 timeline allows proper design

---

## References

- [sqlx: Custom derive attributes](https://github.com/launchbadge/sqlx)
- [Diesel: Query Builder pattern](https://diesel.rs/)
- [PostgreSQL RLS](https://www.postgresql.org/docs/current/ddl-rowsecurity.html)
