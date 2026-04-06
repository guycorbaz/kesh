# Story 1.8 : RBAC & verrouillage optimiste

Status: done

## Story

As a **administrateur**,
I want **que les accès soient contrôlés par rôle et que la concurrence soit gérée**,
so that **les données soient protégées contre les accès non autorisés et les conflits**.

### Décisions de conception

- **Middleware RBAC par fonctions nommées** : une fonction async par niveau de rôle (`require_admin_role`, `require_comptable_role`) utilisée via `axum::middleware::from_fn`. Approche choisie car elle compile proprement avec Axum 0.8 sans types complexes. La factory générique `require_role(min_role)` est possible mais introduit un type de retour `Pin<Box<...>>` inutilement complexe pour 3 niveaux de rôles — les fonctions nommées sont plus simples et tout aussi extensibles.
- **Hiérarchie ordinale** : `Role` implémente `Ord` avec `Consultation(0) < Comptable(1) < Admin(2)`. La comparaison `>=` suffit pour l'héritage de permissions.
- **Inline guards supprimés — pourquoi** : le middleware centralisé empêche qu'un développeur ajoute un nouveau handler et oublie le guard inline. Défense en profondeur au niveau routage plutôt qu'au niveau handler. Les 6 appels `require_admin()` dans `routes/users.rs` sont remplacés par le layer RBAC sur le sous-routeur `/users/*`.
- **Verrouillage optimiste déjà fonctionnel** : le pattern `version` + 409 est implémenté depuis Stories 1.4/1.7. Cette story vérifie que les tests couvrent les AC#5/AC#6, elle ne réimplémente pas.
- **Endpoints comptables absents** : les AC2 concernent des endpoints qui n'existent pas encore (Epic 3+). Le middleware est prêt. Les tests utilisent un endpoint de test `_test/comptable` gardé par `#[cfg(test)]` dans build_router pour ne pas polluer la production.
- **Gardes métier restent dans les handlers** : self-disable, last-admin, password validation nécessitent un contexte métier. Les handlers gardent `Extension<CurrentUser>` pour ces vérifications.
- **`rbac.rs` séparé de `auth.rs`** : l'architecture place "JWT extraction, RBAC" dans `auth.rs`, mais pour la séparation des responsabilités et la lisibilité, le RBAC est dans un fichier dédié `middleware/rbac.rs`. Déviation documentée, cohérente avec `rate_limit.rs` (aussi séparé).

## Acceptance Criteria (AC)

1. **Consultation bloqué sur toutes les ressources protégées** — Given rôle Consultation, When requête (GET, POST, PUT, DELETE) sur /api/v1/users/*, Then 403 `FORBIDDEN`.
2. **Comptable autorisé sur données comptables** — Given rôle Comptable, When requête sur une route protégée Comptable+ (vérifié via endpoint de test `_test/comptable`), Then autorisé (200).
3. **Comptable bloqué sur gestion utilisateurs** — Given rôle Comptable, When requête sur /api/v1/users/*, Then 403 `FORBIDDEN`.
4. **Admin accès complet** — Given rôle Admin, When toute opération (users, comptable, etc.), Then autorisé (hérite Comptable + gestion users).
5. **Verrouillage optimiste — succès** — Given une entité avec version=3, When PUT avec version=3, Then mise à jour réussie, version passe à 4.
6. **Verrouillage optimiste — conflit** — Given une entité avec version=4, When PUT avec version=3 (stale), Then 409 `OPTIMISTIC_LOCK_CONFLICT` avec message explicite.
7. **RBAC appliqué structurellement** — Le middleware RBAC est appliqué sur toutes les routes /api/v1/* (sauf /api/v1/auth/login, /api/v1/auth/logout, /api/v1/auth/refresh). PUT /api/v1/auth/password est protégé par `require_auth` seul (tout rôle authentifié) — pas de RBAC supplémentaire car tout utilisateur doit pouvoir changer son propre mot de passe.

## Tasks / Subtasks

### T1 — Hiérarchie de rôles : `Ord` sur `Role` (AC: #1-#4)
- [x] T1.1 Implémenté `PartialOrd`, `Ord`, `level()` sur Role
- [x] T1.2 7 tests unitaires (hierarchy + levels)

### T2 — Middleware RBAC (AC: #1-#4, #7)
- [x] T2.1 Créé `middleware/rbac.rs` : `require_admin_role`, `require_comptable_role`, `check_role`
- [x] T2.2 Exporté `pub mod rbac;` dans `middleware/mod.rs`
- [x] T2.3 Tests via E2E (middleware testé intégralement dans rbac_e2e.rs)

### T3 — Refactoring du routeur (AC: #1-#4, #7)
- [x] T3.1 Réorganisé build_router() : admin_routes (RBAC Admin) + authenticated_routes + require_auth outer layer
- [x] T3.2 Supprimé require_admin() + 6 appels. Supprimé Extension<CurrentUser> de create_user, list_users, get_user, reset_password. Gardé dans update_user et disable_user.
- [x] T3.3 Vérifié : construction (require_role inner → merge → require_auth outer) = exécution (require_auth first → require_role second → handler)
- [x] T3.4 Route test _test/comptable dans spawn_app() de rbac_e2e.rs avec double layer

### T4 — Tests E2E RBAC (AC: #1-#4, #7)
- [x] T4.1 Créé rbac_e2e.rs avec helpers (spawn_app, login_as, create_and_login_as)
- [x] T4.2 AC#1 : 4 tests Consultation → 403 (GET/POST/PUT users, disable)
- [x] T4.3 AC#2 : 3 tests (Comptable 200, Admin 200, Consultation 403 sur _test/comptable)
- [x] T4.4 AC#3 : 3 tests Comptable → 403 (POST users, GET users, reset-password)
- [x] T4.5 AC#4 : 1 test Admin accès complet (create 201, list 200, _test/comptable 200)
- [x] T4.6 AC#7 : 1 test change_password accessible par Consultation + Comptable (200)

### T5 — Vérification verrouillage optimiste (AC: #5, #6)
- [x] T5.1 AC#5 couvert par `update_user_change_role` (users_e2e.rs) — version incrémentée assertée
- [x] T5.2 AC#6 : test `optimistic_lock_conflict_returns_correct_error_code` ajouté dans rbac_e2e.rs — vérifie 409 + body `OPTIMISTIC_LOCK_CONFLICT`
- [x] T5.3 Pas de lacune supplémentaire

### T6 — Non-régression (AC: #1-#7)
- [x] T6.1 68 tests unitaires passent (61 kesh-api + 7 kesh-db), compilation workspace OK
- [x] T6.2 Gardes métier préservés : self-disable et last-admin checks dans update_user/disable_user avec Extension<CurrentUser> toujours extrait

## Dev Notes

### Architecture du middleware RBAC

Fonctions nommées par niveau de rôle dans `middleware/rbac.rs` :

```rust
use crate::errors::AppError;
use crate::middleware::auth::CurrentUser;
use axum::{extract::Request, middleware::Next, response::Response};
use kesh_db::entities::Role;

/// Middleware : requiert au minimum le rôle Admin.
pub async fn require_admin_role(
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    check_role(&req, Role::Admin)?;
    Ok(next.run(req).await)
}

/// Middleware : requiert au minimum le rôle Comptable (Admin hérite).
pub async fn require_comptable_role(
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    check_role(&req, Role::Comptable)?;
    Ok(next.run(req).await)
}

fn check_role(req: &Request, min_role: Role) -> Result<(), AppError> {
    let current_user = req.extensions()
        .get::<CurrentUser>()
        .ok_or_else(|| AppError::Unauthenticated("missing CurrentUser in extensions".into()))?;
    if current_user.role < min_role {
        return Err(AppError::Forbidden);
    }
    Ok(())
}
```

**Pourquoi `req.extensions().get()` plutôt que `Extension<T>` extracteur** : dans un middleware Axum 0.8 (`async fn` passée à `from_fn`), les extracteurs comme `Extension<T>` ne sont pas disponibles directement car la signature est `(Request, Next) -> Response`. On accède aux extensions via `req.extensions().get::<T>()`. C'est le même pattern que `require_auth` utilise pour injecter `CurrentUser`.

### Réorganisation du routeur

```rust
// Après (1.8) : sous-routeurs par rôle
let admin_routes = Router::new()
    .route("/api/v1/users", get(...).post(...))
    .route("/api/v1/users/:id", get(...).put(...))
    .route("/api/v1/users/:id/disable", put(...))
    .route("/api/v1/users/:id/reset-password", put(...))
    .route_layer(from_fn(rbac::require_admin_role));

let authenticated_routes = Router::new()
    .route("/api/v1/auth/password", put(...));

let protected = Router::new()
    .merge(admin_routes)
    .merge(authenticated_routes)
    .route_layer(from_fn_with_state(state, require_auth));
```

### Handlers après refactoring

Les handlers dans `routes/users.rs` gardent `Extension<CurrentUser>` pour les gardes métier mais perdent l'appel `require_admin()` :

```rust
// AVANT (1.7) — create_user (n'utilise current_user que pour require_admin)
pub async fn create_user(..., Extension(current_user): Extension<CurrentUser>, ...) {
    require_admin(&current_user)?;
    // ... logique métier
}

// APRÈS (1.8) — create_user (Extension<CurrentUser> SUPPRIMÉ car inutilisé)
pub async fn create_user(State(state): State<AppState>, Json(req): Json<CreateUserRequest>) {
    // require_admin() supprimé — le middleware RBAC s'en charge
    // Pas besoin de current_user ici
}

// APRÈS (1.8) — disable_user (Extension<CurrentUser> GARDÉ pour self-disable check)
pub async fn disable_user(..., Extension(current_user): Extension<CurrentUser>, Path(id): Path<i64>) {
    // require_admin() supprimé — le middleware RBAC s'en charge
    if id == current_user.user_id { return Err(AppError::CannotDisableSelf); }
    // ... logique métier utilisant current_user
}
```

Handlers qui utilisent encore `current_user` après le refactoring :
- `update_user` : self-disable check, last-admin check
- `disable_user` : self-disable check (`id == current_user.user_id`)
- `reset_password` : aucun usage de current_user (require_admin suffisait)
- `create_user`, `list_users`, `get_user` : aucun usage de current_user

Pour les handlers qui n'utilisent plus `current_user`, supprimer le paramètre `Extension<CurrentUser>` de leur signature.

### Pièges connus

1. **Ordre des route_layer** : RBAC doit être appliqué sur le sous-routeur AVANT le merge dans le routeur protégé par require_auth. Le flux est : request → require_auth (injecte CurrentUser) → require_role (lit CurrentUser) → handler.
2. **route_layer panic** : un routeur vide avec route_layer panique. Ajouter les routes AVANT le layer.
3. **Axum 0.8 middleware** : les fonctions `async fn` passées à `from_fn` doivent être `Clone + Send`. Les fonctions nommées le sont automatiquement.
4. **Tests existants** : après le refactoring, users_e2e.rs et auth_e2e.rs doivent continuer à passer. Le middleware RBAC remplace le guard inline de façon transparente.
5. **Extension<CurrentUser> dans les handlers** : les handlers continuent de l'extraire via `axum::Extension(current_user)` — ce pattern fonctionne car `CurrentUser` est `Clone` et est dans les extensions de la requête.

### Fichiers à créer / modifier

**Nouveaux fichiers :**
- `crates/kesh-api/src/middleware/rbac.rs` — middleware RBAC (require_admin_role, require_comptable_role, check_role)
- `crates/kesh-api/tests/rbac_e2e.rs` — tests E2E RBAC

**Fichiers à modifier :**
- `crates/kesh-db/src/entities/user.rs` — ajouter `Ord` + `PartialOrd` + `level()` sur Role
- `crates/kesh-api/src/middleware/mod.rs` — exporter `rbac`
- `crates/kesh-api/src/lib.rs` — réorganiser build_router() avec sous-routeurs par rôle + route test `#[cfg(test)]`
- `crates/kesh-api/src/routes/users.rs` — supprimer require_admin() + 6 appels, supprimer `Extension<CurrentUser>` des handlers qui ne l'utilisent plus

**Pas de nouvelle migration, pas de nouvelles dépendances.**

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-1, Story 1.8] — AC et user story
- [Source: _bmad-output/planning-artifacts/architecture.md#ARCH-16,ARCH-31] — RBAC hiérarchique, verrouillage optimiste
- [Source: _bmad-output/implementation-artifacts/1-7-gestion-des-utilisateurs-crud.md] — require_admin pattern, test infrastructure, gardes métier
- [Source: crates/kesh-api/src/middleware/auth.rs] — require_auth middleware (signature: `async fn(State<AppState>, Request, Next) -> Result<Response, AppError>`), CurrentUser struct (`{ user_id: i64, role: Role }`)
- [Source: crates/kesh-api/src/routes/users.rs] — require_admin() inline (6 appels aux lignes 127, 168, 220, 260, 293, 319)
- [Source: crates/kesh-api/src/lib.rs] — build_router() actuel avec sous-routeur protégé unique
- [Source: crates/kesh-api/src/errors.rs] — AppError::Forbidden (403), AppError::Unauthenticated (401)
- [Source: crates/kesh-db/src/entities/user.rs] — Role enum (derives actuels: Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize — PAS Ord/PartialOrd)
- [Source: crates/kesh-db/src/repositories/users.rs] — update_role_and_active (optimistic lock via `WHERE version = ?`)

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

### Completion Notes List

- T1 : Role hierarchy `Ord`/`PartialOrd` avec `level()` (7 tests unitaires)
- T2 : Middleware RBAC `require_admin_role` + `require_comptable_role` + `check_role` helper
- T3 : build_router refactorisé avec sous-routeurs par rôle, require_admin() supprimé (6 appels), Extension<CurrentUser> supprimé de 4 handlers
- T4 : 13 tests E2E RBAC couvrant les 7 AC
- T5 : Verrouillage optimiste vérifié + test renforcé (body 409 `OPTIMISTIC_LOCK_CONFLICT`)
- T6 : 68 tests unitaires passent, gardes métier préservés

### Change Log

- 2026-04-06 : Story 1.8 implémentée — middleware RBAC, refactoring routeur, 13 tests E2E, 7 tests unitaires Role hierarchy
- 2026-04-06 : Code review passe #1 (Sonnet) — 3 patches :
  - F2 : test unauthenticated → 401 ajouté (vérifie que require_auth outer layer rejette avant RBAC)
  - F3 : test Ord/PartialEq consistency ajouté dans Role tests
  - F5 : parenthèses explicites dans removes_active_admin expression

### File List

**Nouveaux fichiers :**
- crates/kesh-api/src/middleware/rbac.rs
- crates/kesh-api/tests/rbac_e2e.rs

**Fichiers modifiés :**
- crates/kesh-db/src/entities/user.rs (Ord/PartialOrd/level sur Role)
- crates/kesh-api/src/middleware/mod.rs (export rbac)
- crates/kesh-api/src/lib.rs (build_router refactored)
- crates/kesh-api/src/routes/users.rs (require_admin supprimé, Extension<CurrentUser> nettoyé)
