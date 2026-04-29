# Optimistic Locking — `is_no_op_change` Pattern

**Origine** : [Story 7-3](../_bmad-output/implementation-artifacts/7-3-kf-004-version-bump-optimization.md) — clôture KF-004 (issue [#4](https://github.com/guycorbaz/kesh/issues/4)).

## Problème

Le pattern `version = version + 1` inconditionnel dans tous les `update()` user-form bumpe `version` même quand aucun champ métier ne diffère. Conséquence UX : deux utilisateurs qui rouvrent le même formulaire et cliquent « Enregistrer » sans rien modifier déclenchent un `409 OPTIMISTIC_LOCK_CONFLICT` trompeur (KF-004).

## Pattern : compare-then-skip côté Rust

Chaque repository qui implémente un `update()` user-form déclare un helper privé `is_no_op_change(...)` co-localisé juste au-dessus de la fonction. Le helper compare manuellement les champs métier de l'entité persistée (`before`) au payload de modification (`changes`) et retourne `true` si aucun ne diffère.

Le `update()` court-circuite la mutation en `tx.rollback()` + `Ok(before)` quand `is_no_op_change` retourne `true`, **après** la version-check applicative et tous les guards métier (status, fiscal_year, accounts actifs).

```rust
// Étape existante : SELECT before + version check.
let before = match before_opt {
    None => { tx.rollback().await.map_err(map_db_error)?; return Err(DbError::NotFound); }
    Some(e) if !e.active => { /* IllegalStateTransition */ }
    Some(e) if e.version != version => {
        tx.rollback().await.map_err(map_db_error)?;
        return Err(DbError::OptimisticLockConflict);
    }
    Some(e) => e,
};

// KF-004 : court-circuit no-op AVANT toute mutation.
if is_no_op_change(&before, &changes) {
    tx.rollback().await.map_err(map_db_error)?;
    return Ok(before);
}

// Mutation existante : UPDATE + audit log + commit.
```

## Repositories couverts

Story 7-3 a appliqué le pattern aux 9 fonctions `update()` user-form du crate `kesh-db` :

| # | Repository | Fonction | Champs comparés |
|---|---|---|---|
| 1 | `contacts.rs` | `update` | `contact_type, name, is_client, is_supplier, address, email, phone, ide_number, default_payment_terms` |
| 2 | `products.rs` | `update` | `name, description, unit_price, vat_rate` |
| 3 | `invoices.rs` | `update` | `contact_id, date, due_date, payment_terms` + lignes (replace-all en ordre) |
| 4 | `accounts.rs` | `update` | `name, account_type` |
| 5 | `bank_accounts.rs` | `upsert_primary` (branche `Some`) | `bank_name, iban, qr_iban` |
| 6 | `companies.rs` | `update` | `name, address, ide_number, org_type, accounting_language, instance_language` |
| 7 | `company_invoice_settings.rs` | `update` | `invoice_number_format, default_receivable_account_id, default_revenue_account_id, default_sales_journal, journal_entry_description_template` |
| 8 | `journal_entries.rs` | `update` | `entry_date, journal, description` + lignes (en ordre `line_order`) |
| 9 | `users.rs` | `update_role_and_active` | `role, active` |

## Hors scope (pattern non applicable)

- **Transitions d'état** : `archive()`, `invoices::validate_invoice`, `invoices::mark_as_paid`, `invoices::delete` — l'état avant ≠ l'état après par construction.
- **Mutations cryptographiques** : `users::update_password` (le hash bcrypt change même si le clair est identique).
- **Compteurs monotones** : `invoice_number_sequences::reserve`, `onboarding::update_step`.
- **Pas de colonne `version`** : `fiscal_years::update_name` (no-op déjà toléré par construction).

## Notes d'implémentation

- **Pas de trait générique `NoOpDetectable<T>`** : 9 implémentations one-shot suffisent, l'abstraction prématurée coûterait plus en lisibilité.
- **Visibilité `fn` (privée)** : co-localisée dans le module repository, pas exposée.
- **Comparaison manuelle field-by-field** : pas de `#[derive(PartialEq)]` sur les structs entités/Update — préserve la flexibilité (par ex. `User` qui interdit la dérivation à cause du `password_hash`).
- **Audit log** : aucune entrée écrite sur no-op (cohérent avec « pas de changement métier = pas de trace »).
- **`updated_at`** : non touché sur no-op (pas d'UPDATE → MariaDB ne déclenche pas `ON UPDATE CURRENT_TIMESTAMP(3)`).
- **HTTP** : transparent — réponse `200 OK` avec l'état actuel, indistinguable du cas mutation effective.

## Concurrence — race acceptée v0.1

Sous REPEATABLE READ + plain SELECT, si une tx parallèle commit entre `BEGIN` et le no-op check, le client peut retourner un snapshot stale au lieu d'un 409. Cette race est documentée dans la spec Story 7-3 §race-condition et acceptée v0.1 :

- **Variant A** (6 cibles `contacts/products/invoices/accounts/company_invoice_settings/companies` + variant C `users`) : exposées à la race.
- **`bank_accounts::upsert_primary`** : protégé par `SELECT FOR UPDATE` étape 1 (pas de race).
- **`journal_entries::update`** : protégé par `SELECT FOR UPDATE` étape 1 (pas de race).

**Mitigation Epic 8 prerequisite** : passer `invoices::update` en `SELECT FOR UPDATE` (entité comptable la plus exposée — sessions de saisie facture longues). Tracé via une issue GitHub follow-up dédiée. Les 5 autres entités variant A restent en pattern optimiste.

## Régression

Tous les `update_*` happy-path, optimistic-conflict, IllegalStateTransition et IDOR cross-tenant existants continuent de passer sans modification — la détection no-op vient APRÈS les guards.
