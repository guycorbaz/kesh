# Story 1.7 : Gestion des utilisateurs (CRUD)

Status: done

## Story

As a **administrateur**,
I want **créer et gérer les comptes utilisateurs**,
so that **chaque personne ait son propre accès avec le bon niveau de droits**.

### Décisions de conception

- **Rôle unique par utilisateur** : le PRD mentionne "un ou plusieurs rôles" (FR10) mais l'architecture a choisi un rôle hiérarchique unique (Consultation < Comptable < Admin). Chaque niveau hérite des permissions inférieures. Cette story implémente le modèle single-role.
- **Reset password : admin fournit le nouveau mot de passe** : l'epic mentionne "mot de passe temporaire" — on implémente un endpoint où l'admin fournit le nouveau mot de passe dans le body (plus simple, pas besoin de canal sécurisé pour transmettre un mot de passe généré). Le mot de passe est "temporaire" par convention (l'admin demande à l'utilisateur de le changer).
- **Messages d'erreur** : codes d'erreur en anglais SCREAMING_SNAKE_CASE dans le JSON, messages techniques en français dans le champ `message`. L'i18n complète (FR/DE/IT/EN) sera adressée en Epic 2 (Story 2.1).
- **Verrouillage optimiste sur disable/reset-password** : ces endpoints n'acceptent pas de `version` dans le body (pas de DTO dédié). Le serveur lit la version courante via `find_by_id` puis appelle `update_role_and_active` / `update_password` qui incrémentent `version` en DB. Ceci est acceptable pour MVP (opérations admin rares, 2-5 utilisateurs). Le `PUT /users/:id` (update générique) requiert bien `version` dans le body pour le verrouillage optimiste complet.
- **Validation de mot de passe : pas de trim** : la longueur est vérifiée sur la valeur brute (pas de `trim()`). Seule la vérification "pas vide / pas tout-whitespace" utilise `trim()`. Ce qui est validé est ce qui est hashé.
- **Désactivation préserve l'historique** : la désactivation (`active = false`) est un soft-disable. Aucune donnée utilisateur n'est supprimée — l'historique des actions reste intact (exigence epic "son historique d'actions reste intact").
- **Tests API Rust vs Playwright** : les tests T7 sont des tests d'intégration API en Rust (reqwest + spawn_app). Les tests E2E Playwright viendront avec les stories frontend (1.10, 1.11). C'est le pattern établi depuis Stories 1.5/1.6 pour les stories backend-only.

## Acceptance Criteria (AC)

1. **Création d'utilisateur** — Given rôle Admin, When POST /api/v1/users avec `{ username, password, role }`, Then l'utilisateur est créé avec le rôle spécifié et retourné (201 + `UserResponse`). Le `role` doit être un des trois valeurs valides (`Admin`, `Comptable`, `Consultation`) — sinon 422 (serde rejette automatiquement les enum invalides). Le `username` doit avoir entre 1 et 64 caractères (après trim) — sinon 400.
2. **Modification d'utilisateur** — Given rôle Admin, When PUT /api/v1/users/:id avec `{ role, active, version }`, Then le rôle et/ou le statut actif sont mis à jour (200 + `UserResponse` avec version incrémentée). 409 si conflit de version.
3. **Désactivation de compte** — Given rôle Admin, When PUT /api/v1/users/:id/disable, Then le compte est désactivé (`active = false`), ses refresh_tokens invalidés (reason `"admin_disable"`), et retour 200 + `UserResponse` avec version incrémentée.
4. **Interdiction de self-disable** — Given rôle Admin, When PUT /api/v1/users/:id/disable avec id = current_user.user_id, Then 400 avec code `CANNOT_DISABLE_SELF`.
5. **Protection du dernier admin** — Given un seul admin actif restant, When PUT /api/v1/users/:id/disable sur cet admin, Then 400 avec code `CANNOT_DISABLE_LAST_ADMIN`.
6. **Changement de mot de passe** — **Déjà implémenté en Story 1.6** (PUT /api/v1/auth/password). Seul travail : refactorer pour utiliser `validate_password()` configurable (AC10).
7. **Réinitialisation de mot de passe par admin** — Given rôle Admin, When PUT /api/v1/users/:id/reset-password avec `{ newPassword }`, Then le mot de passe est mis à jour, toutes les sessions de l'utilisateur cible invalidées (reason `"password_change"`), retour 200 + `UserResponse` avec version incrémentée.
8. **Politique de mot de passe configurable** — Given `KESH_PASSWORD_MIN_LENGTH` défini (défaut 12, borne [8, 128]), When création, changement ou réinitialisation de mot de passe, Then la politique est appliquée (400 `VALIDATION_ERROR` si non respectée). Si la valeur est hors bornes, warn + fallback au défaut 12 (cohérent avec le pattern de tous les autres paramètres config).
9. **Liste des utilisateurs** — Given rôle Admin, When GET /api/v1/users avec pagination (`?limit=50&offset=0`), Then retour 200 + `{ items: [UserResponse...], total, offset, limit }`. Jamais de `password_hash` dans la réponse.
10. **Détail utilisateur** — Given rôle Admin, When GET /api/v1/users/:id, Then retour 200 + `UserResponse` sans `password_hash` (404 `NOT_FOUND` si inexistant).
11. **Contrôle d'accès** — Given rôle Comptable ou Consultation, When requête sur /api/v1/users/*, Then 403 `FORBIDDEN`.
12. **Login impossible après désactivation** — Given un utilisateur désactivé, When POST /api/v1/auth/login, Then 401 `INVALID_CREDENTIALS` (vérification déjà en place depuis Story 1.5 via `active` check — tester la non-régression).

## Tasks / Subtasks

### T1 — Configuration : politique de mot de passe (AC: #8)
- [x] T1.1 Ajouter `password_min_length: u32` dans `Config` (env `KESH_PASSWORD_MIN_LENGTH`, défaut 12, borne [8, 128]). Rejet au démarrage si hors bornes.
- [x] T1.2 Ajouter le champ dans `Config::from_fields_for_test()` — **mettre à jour TOUS les sites d'appel** : `test_config()` et `test_config_rate_limit()` dans `auth_e2e.rs`, ET `test_state()` dans `middleware/auth.rs` (bloc `#[cfg(test)]`) + 2 appels bootstrap E2E
- [x] T1.3 Extraire la validation de mot de passe dans `auth/password.rs` : `pub fn validate_password(password: &str, min_length: u32) -> Result<(), AppError>`
- [x] T1.4 Refactorer `change_password` (story 1.6) pour utiliser `validate_password()` avec `config.password_min_length` au lieu du 12 hardcodé
- [x] T1.5 Tests unitaires : config parsing (4 tests), validation password (6 tests)

### T2 — DTOs et types réponse (AC: #1, #2, #9, #10)
- [x] T2.1 Créer `CreateUserRequest` avec serde camelCase + Debug masquant password
- [x] T2.2 Créer `UpdateUserRequest` avec serde camelCase
- [x] T2.3 Créer `ResetPasswordRequest` avec serde camelCase + Debug masquant
- [x] T2.4 Créer `UserResponse` (sans password_hash, avec version)
- [x] T2.5 Créer `UserListResponse` paginé
- [x] T2.6 Implémenter `From<User> for UserResponse`
- [x] T2.7 Doc comments sur les structs publics

### T3 — Repository : extensions kesh-db (AC: #9)
- [x] T3.1 Ajouter `count()` et `count_active_by_role()` dans `repositories/users.rs`
- [x] T3.2 Vérifié : list, find_by_id, create, update_role_and_active, update_password couvrent les besoins
- [x] T3.3 Tests d'intégration : couverture via tests E2E (count appelé dans list_users)

### T4 — Routes CRUD utilisateurs (AC: #1-#5, #7, #9-#11)
- [x] T4.1 Créer `crates/kesh-api/src/routes/users.rs` avec 6 handlers
- [x] T4.2 Ajouter `pub mod users;` dans `routes/mod.rs`
- [x] T4.3 Enregistrer les routes dans `build_router()` sous le groupe protégé
- [x] T4.4 Guard `require_admin()` dans `routes/users.rs`
- [x] T4.5 Doc comments sur les handlers publics

### T5 — Erreurs et codes (AC: #4, #5, #7, #11)
- [x] T5.1 Ajouté `Forbidden` → 403 `FORBIDDEN`
- [x] T5.2 Ajouté `CannotDisableSelf` → 400, `CannotDisableLastAdmin` → 400
- [x] T5.3 Vérifié : UniqueConstraintViolation → 409 RESOURCE_CONFLICT
- [x] T5.4 Vérifié : NotFound → 404, OptimisticLockConflict → 409

### T6 — Logique métier handlers (AC: #1-#5, #7)
- [x] T6.1 `create_user` implémenté (trim username, validate password, hash_async, 201)
- [x] T6.2 `update_user` implémenté (find_by_id, update_role_and_active, 409 version)
- [x] T6.3 `disable_user` implémenté (self-disable check, last-admin guard, revoke_all)
- [x] T6.4 `reset_password` implémenté (validate, hash_async, revoke_all, re-fetch)
- [x] T6.5 `list_users` implémenté (pagination clampée, count+list)
- [x] T6.6 `get_user` implémenté (find_by_id, 404)

### T7 — Tests E2E (AC: #1-#12)
- [x] T7.1 Créé `crates/kesh-api/tests/users_e2e.rs` avec helpers (spawn_app, login_admin, create_user_api, login_as)
- [x] T7.2 Tests création : 7 tests (success 201, invalid role 422, duplicate 409, short pwd 400, empty/whitespace username 400, non-admin 403, no passwordHash in response)
- [x] T7.3 Tests modification : 3 tests (change role 200, reactivate 200, version conflict 409)
- [x] T7.4 Tests liste : 2 tests (paginated 200, non-admin 403)
- [x] T7.5 Tests détail : 2 tests (success 200 + no passwordHash, not found 404)
- [x] T7.6 Tests désactivation : 4 tests (success 200, self-disable 400, last admin protection, login impossible after disable 401)
- [x] T7.7 Tests reset password : 3 tests (success 200 + version check + login with new pwd, short pwd 400, non-admin 403)
- [x] T7.8 Test politique configurable : 1 test (min_length=20, rejet sur create + reset + change_password)

### T8 — Non-régression change_password et tests existants (AC: #6, #8)
- [x] T8.1 Refactoring vérifié : change_password utilise validate_password(), 61 tests unitaires passent
- [x] T8.2 5 sites d'appel from_fields_for_test() mis à jour (test_config, test_config_rate_limit, test_state, 2 bootstrap E2E)

## Dev Notes

### Architecture des endpoints

Tous les endpoints `/api/v1/users/*` sont protégés par `require_auth` (middleware existant) et vérifient `Role::Admin` via un guard partagé `require_admin()`. Le middleware RBAC formel (route_layer) sera implémenté en Story 1.8 — pour 1.7, le guard inline dans chaque handler suffit. Story 1.8 pourra le remplacer par un middleware centralisé.

**Pattern handler Axum 0.8 :**
```rust
fn require_admin(current_user: &CurrentUser) -> Result<(), AppError> {
    if current_user.role != Role::Admin {
        return Err(AppError::Forbidden);
    }
    Ok(())
}

async fn create_user(
    State(state): State<AppState>,
    Extension(current_user): Extension<CurrentUser>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), AppError> {
    require_admin(&current_user)?;
    // ...
}
```

### Réponses JSON — Conventions

Format erreur conforme à l'architecture : `{ "error": { "code": "CODE", "message": "..." } }` (champ `details` optionnel). Codes de statut :

| Endpoint | Succès | Erreurs |
|----------|--------|---------|
| POST /users | 201 + UserResponse | 400, 403, 409, 422 |
| PUT /users/:id | 200 + UserResponse | 400, 403, 404, 409 |
| PUT /users/:id/disable | 200 + UserResponse | 400, 403, 404 |
| PUT /users/:id/reset-password | 200 + UserResponse | 400, 403, 404 |
| GET /users | 200 + UserListResponse | 403 |
| GET /users/:id | 200 + UserResponse | 403, 404 |

### Authentification vs Autorisation (couches d'erreur)

Les endpoints `/api/v1/users/*` passent par deux couches :
1. **`require_auth` middleware** (existant) → 401 `UNAUTHENTICATED` si token absent/invalide/expiré
2. **`require_admin()` guard** (nouveau, dans handler) → 403 `FORBIDDEN` si rôle ≠ Admin

Un utilisateur non authentifié reçoit 401 (jamais 403). Un utilisateur authentifié non-Admin reçoit 403.

### Sécurité — Règles absolues

1. **Jamais sérialiser `User` directement** — toujours mapper vers `UserResponse` via `From<User>` (sans password_hash)
2. **Debug masqué** pour tous les DTOs contenant des mots de passe — impl manuelle avec `"***"`
3. **`hash_password_async()`** (spawn_blocking) pour le hashing Argon2id — ne jamais bloquer le runtime Tokio. Les versions sync `hash_password()` et `verify_password()` existent pour `LazyLock<DUMMY_HASH>` et `dummy_verify()` — **ne pas les supprimer**
4. **Anti-énumération** : les erreurs de création retournent des messages génériques côté client
5. **Dernière admin-garde** : compter les admins actifs AVANT de désactiver. Note : race condition théorique si deux admins désactivent simultanément — acceptable pour MVP (2-5 utilisateurs), documenter comme dette technique
6. **Self-disable interdit** : comparer `current_user.user_id` avec l'ID cible
7. **`version` dans UserResponse** : indispensable pour le verrouillage optimiste côté frontend (Story 1.8)

### Anti-patterns à éviter (leçons stories 1.5/1.6)

- **NE PAS** dériver `Serialize`/`Deserialize` sur `User` entity — utiliser `UserResponse` DTO
- **NE PAS** dériver `Debug` automatiquement sur DTOs contenant password/token — impl manuelle masquant les secrets
- **NE PAS** dériver `#[sqlx::Type]` sur l'enum `Role` — implémentation manuelle `Type`/`Encode`/`Decode` déjà en place
- **NE PAS** logger les passwords, JWT, refresh_tokens, ni les request/response bodies complets
- **NE PAS** utiliser `.unwrap()` ou `.expect()` en code de production
- **NE PAS** retourner de message d'erreur différenciant "username inexistant" vs "username existant" (attaque par énumération)
- **NE PAS** oublier le catch-all exhaustif dans le match `IntoResponse` pour `AppError` — pas de `_ =>`

### Invalidation des sessions

Pattern identique à Story 1.6 :
```rust
refresh_token_repo::revoke_all_for_user(&state.pool, target_user_id, "admin_disable").await?;
```

**Valeurs exactes du CHECK constraint DB** (toute typo échouera en DB) :
`'logout'` | `'rotation'` | `'password_change'` | `'admin_disable'` | `'theft_detected'` | `NULL`

Les tokens pré-1.6 avec `revoked_reason = NULL` sont traités comme logout (pas de mass-revoke).

### Politique de mot de passe — Refactoring

Placer dans `crates/kesh-api/src/auth/password.rs` (co-localisé avec hash/verify) :

```rust
/// Valide un mot de passe selon la politique configurable.
/// Vérifie : non-vide, pas uniquement whitespace, longueur >= min_length sur la valeur BRUTE.
/// Ce qui est validé est ce qui sera hashé — pas de trim sur la longueur.
pub fn validate_password(password: &str, min_length: u32) -> Result<(), AppError> {
    if password.is_empty() || password.chars().all(char::is_whitespace) {
        return Err(AppError::Validation("Le mot de passe ne peut pas être vide".into()));
    }
    if password.chars().count() < min_length as usize {
        return Err(AppError::Validation(
            format!("Le mot de passe doit contenir au moins {} caractères", min_length)
        ));
    }
    Ok(())
}
```

Note : valider la longueur sur la valeur brute (pas trimmed) pour que la validation corresponde exactement à ce qui est hashé. Utiliser `.chars().count()` (Unicode-aware) comme dans le `change_password` existant de Story 1.6.

### Tests — Infrastructure existante

Réutiliser le pattern de `tests/auth_e2e.rs` :
- `spawn_app()` / `spawn_app_with_config()` pour serveur éphémère
- `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` pour DB fraîche
- `TestApp` avec `reqwest::Client`
- Helper `login()` pour obtenir un access_token Admin
- `max_connections = 2` dans le pool de test

**Attention T1.2** : l'ajout de `password_min_length` dans `from_fields_for_test()` cassera la compilation de **3 sites d'appel** :
1. `test_config()` dans `crates/kesh-api/tests/auth_e2e.rs`
2. `test_config_rate_limit()` dans `crates/kesh-api/tests/auth_e2e.rs`
3. `test_state()` dans `crates/kesh-api/src/middleware/auth.rs` (bloc `#[cfg(test)]`)

### Fichiers à créer / modifier

**Nouveaux fichiers :**
- `crates/kesh-api/src/routes/users.rs` — handlers CRUD + guard require_admin
- `crates/kesh-api/tests/users_e2e.rs` — tests E2E

**Fichiers à modifier :**
- `crates/kesh-api/src/config.rs` — ajouter `password_min_length`
- `crates/kesh-api/src/errors.rs` — ajouter `Forbidden`, `CannotDisableSelf`, `CannotDisableLastAdmin`
- `crates/kesh-api/src/lib.rs` — enregistrer routes users dans `build_router()`
- `crates/kesh-api/src/routes/mod.rs` — exporter module `users`
- `crates/kesh-api/src/routes/auth.rs` — refactorer `change_password` pour utiliser `validate_password()` configurable
- `crates/kesh-api/src/auth/password.rs` — ajouter `validate_password()`
- `crates/kesh-db/src/repositories/users.rs` — ajouter `count()`
- `crates/kesh-api/tests/auth_e2e.rs` — mettre à jour `test_config()` et `test_config_rate_limit()`
- `crates/kesh-api/src/middleware/auth.rs` — mettre à jour `test_state()` dans le bloc `#[cfg(test)]`

**Pas de nouvelle migration** — le schéma `users` et `refresh_tokens` existants couvrent tous les besoins.

### Project Structure Notes

- Routes CRUD dans `routes/users.rs` (parallèle à `routes/auth.rs` existant)
- Tests E2E dans `tests/users_e2e.rs` (parallèle à `tests/auth_e2e.rs`)
- Validation password partagée dans `auth/password.rs` (co-localisée avec hash/verify)
- Guard `require_admin()` dans `routes/users.rs` (sera déplacé en middleware en Story 1.8)
- Aucun conflit de structure détecté avec l'architecture définie

### Pièges connus (Stories 1.5/1.6)

1. **ConnectInfo** : `spawn_app()` doit utiliser `.into_make_service_with_connect_info::<SocketAddr>()` — déjà en place
2. **chrono::TimeDelta** vs `std::time::Duration` : utiliser `TimeDelta` pour l'arithmétique avec `NaiveDateTime`
3. **Pool timeout en tests** : garder `max_connections = 2` pour éviter `PoolTimedOut` en parallèle
4. **enum Role SQLx** : implémentation manuelle `Type`/`Encode`/`Decode` (pas de derive) — existant dans `entities/user.rs`
5. **BINARY CHECK** : les valeurs Role dans la DB sont case-sensitive (`'Admin'` pas `'admin'`)
6. **`route_layer` panic** : ajouter les routes AVANT d'appliquer les layers
7. **Argon2 ~50ms** : toujours `spawn_blocking` pour hash/verify — ne jamais bloquer le runtime
8. **env_lock()** : sérialiser les tests qui modifient des variables d'environnement
9. **serde Role enum** : si `role` dans le JSON est invalide, serde retourne 422 Unprocessable Entity automatiquement — ne pas réinventer la validation

### Dépendances — Aucune nouvelle

Toutes les crates nécessaires sont déjà dans Cargo.toml :
- `argon2 = "0.5"`, `jsonwebtoken = "9"`, `uuid = "1"`, `chrono = "0.4"`
- `axum = "0.8"`, `tower = "0.5"`, `tower-http = "0.6"`
- `sqlx = "0.8"`, `serde = "1"`, `thiserror = "2"`

### Documentation

Conformément à la règle architecturale #3 : ajouter `///` doc comments Rust sur toutes les structs, enums, fonctions et champs publics créés dans cette story.

### References

- [Source: _bmad-output/planning-artifacts/epics.md#Epic-1, Story 1.7] — AC et user story
- [Source: _bmad-output/planning-artifacts/architecture.md#ARCH-14-ARCH-26] — Auth, RBAC, API patterns
- [Source: _bmad-output/planning-artifacts/prd.md#FR9-FR17] — Exigences fonctionnelles utilisateurs
- [Source: _bmad-output/implementation-artifacts/1-6-refresh-token-gestion-de-session.md] — Learnings, patterns, pièges
- [Source: _bmad-output/implementation-artifacts/1-5-authentification-login-logout-jwt.md] — Auth foundation, timing mitigation
- [Source: crates/kesh-db/src/entities/user.rs] — User entity, Role enum, NewUser, UserUpdate
- [Source: crates/kesh-db/src/repositories/users.rs] — Repository CRUD existant (create, find_by_id, find_by_username, list, update_role_and_active, update_password)
- [Source: crates/kesh-api/src/routes/auth.rs] — change_password handler à refactorer (lignes 344-352, validation hardcodée 12 chars)
- [Source: crates/kesh-api/src/errors.rs] — AppError enum à étendre (variants existants : InvalidCredentials, Unauthenticated, Validation, Internal, Database, RateLimited, InvalidRefreshToken)
- [Source: crates/kesh-api/src/config.rs] — Config à étendre (from_fields_for_test prend 10 params actuellement)
- [Source: crates/kesh-db/src/repositories/refresh_tokens.rs] — revoke_all_for_user(pool, user_id, reason) confirmé existant

## Dev Agent Record

### Agent Model Used

Claude Opus 4.6 (1M context)

### Debug Log References

### Completion Notes List

- T1 : Config `password_min_length` ajouté (env parsing, from_fields_for_test, 5 call sites, validate_password function, 10 tests unitaires)
- T2-T4 : DTOs (CreateUserRequest, UpdateUserRequest, ResetPasswordRequest, UserResponse, UserListResponse), handlers (6 endpoints), guard require_admin, routes enregistrées dans build_router
- T5 : 3 nouveaux variants AppError (Forbidden, CannotDisableSelf, CannotDisableLastAdmin) avec IntoResponse exhaustif
- T6 : Logique métier complète (create, update, disable, reset_password, list, get) avec toutes les gardes métier
- T7 : 22 tests E2E couvrant tous les AC (création, modification, liste, détail, désactivation, reset password, politique configurable)
- T8 : Non-régression vérifiée (61 tests unitaires passent, compilation workspace OK)

### Change Log

- 2026-04-06 : Story 1.7 implémentée — CRUD utilisateurs, politique de mot de passe configurable, 3 nouveaux variants AppError, 6 endpoints REST, 22 tests E2E, 10 tests unitaires
- 2026-04-06 : Code review passe #1 (Sonnet) — 6 patches appliqués :
  - F1+F3 (HIGH) : `update_user` ajoute gardes self-disable, last-admin, revocation sessions sur transition active→false
  - F4 (HIGH) : test `disable_last_admin_returns_400` réécrit pour tester le vrai scénario 400
  - F5 (bad_spec) : AC#8 amendé — fallback au lieu d'échec au démarrage
  - F6 (MEDIUM) : 3 tests 403 ajoutés (update, disable, get non-admin)
  - F7 (MEDIUM) : test `reset_password_revokes_sessions` ajouté
  - F8 (LOW) : `.clone()` inutiles supprimés dans create_user et reset_password
- 2026-04-06 : Code review passe #2 (Haiku) — 3 patches appliqués :
  - P1 (HIGH) : `update_user` guard étendu pour bloquer demotion de rôle (Admin→non-Admin) du dernier admin
  - P2 (HIGH) : 3 tests ajoutés — self-disable, last-admin deactivation, last-admin demotion via PUT /:id
  - P3 (MEDIUM) : test session revocation via update_user deactivation ajouté

### File List

**Nouveaux fichiers :**
- crates/kesh-api/src/routes/users.rs
- crates/kesh-api/tests/users_e2e.rs

**Fichiers modifiés :**
- crates/kesh-api/src/config.rs
- crates/kesh-api/src/errors.rs
- crates/kesh-api/src/lib.rs
- crates/kesh-api/src/routes/mod.rs
- crates/kesh-api/src/routes/auth.rs
- crates/kesh-api/src/auth/password.rs
- crates/kesh-db/src/repositories/users.rs
- crates/kesh-api/tests/auth_e2e.rs
- crates/kesh-api/src/middleware/auth.rs
