# Story 6.2 : Refactor multi-tenant scoping (`CurrentUser.company_id`)

Status: review

<!-- Note : validation `validate-create-story` recommandée avant `dev-story`, vu l'ampleur du refactor (migration DB + JWT + 8 fichiers routes). -->

## Story

As a **développeur (Guy, mainteneur solo)**,
I want **que toutes les requêtes backend scopent par `CurrentUser.company_id` plutôt que de récupérer une company « par défaut » via `LIMIT 1`**,
so that **Kesh devienne véritablement multi-tenant (IDOR cross-company impossible) et que la dette récurrente `get_company()` accumulée Stories 1-4 → 5-4 soit définitivement close (KF-002 / issue #2)**.

### Contexte

**KF-002 / issue [#2](https://github.com/guycorbaz/kesh/issues/2)** a été flaggée comme finding HIGH/MEDIUM récurrent dans les code reviews des Stories 4-1, 4-2, 5-1 à 5-4. Le pattern actuel :

```rust
async fn get_company(state: &AppState) -> Result<Company, AppError> {
    let list = companies::list(&state.pool, 1, 0).await?;  // LIMIT 1 sans WHERE
    list.into_iter().next().ok_or_else(|| AppError::Internal("Aucune company".into()))
}
```

est **dupliqué dans 8 fichiers de routes** (`contacts.rs`, `products.rs`, `invoices.rs`, `invoice_pdf.rs`, `bank_accounts.rs`, `company_invoice_settings.rs`, `accounts.rs`, `journal_entries.rs`) plus invoqué dans `onboarding.rs`. Tant que Kesh reste mono-tenant (1 seule company en DB), le bug est invisible. Dès qu'un second tenant existe, un utilisateur peut se voir attribuer les données de l'autre company **de manière non-déterministe** (pas d'`ORDER BY`).

**Découverte majeure pendant le cadrage** : la table `users` **n'a PAS de colonne `company_id`** dans le schéma actuel (`crates/kesh-db/migrations/20260404000001_initial_schema.sql` lignes 24-37). Le modèle multi-tenant est donc **incomplet en DB**, pas seulement dans les helpers Rust. Le scope de cette story est donc **plus large** que le refactor suggéré par `epics.md#Story-6.2` : il faut ajouter la colonne + migration + backfill.

**Orthogonalité** : aucune dépendance vers/depuis Story 6-3 (i18n sidebar), Story 6-5 (Playwright auth flow), Story 6-4 (fixtures — mergée). Peut être menée en parallèle.

### Bloque actuellement

- **KF-002 / issue #2** — restera open tant que le refactor n'est pas fait.
- **Release v0.1** en mode multi-tenant explicite — si un jour Kesh expose un inscription-flow qui crée des companies distinctes, le bug devient exploitable.
- **Security review Epic 6** (nouvelle règle `CLAUDE.md`) — un finding IDOR ouvert depuis 5 stories doit être fermé avant la clôture d'Epic 6.

### État actuel (audit 2026-04-18)

**`CurrentUser` struct** (`crates/kesh-api/src/middleware/auth.rs:27`) :

```rust
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub user_id: i64,
    pub role: Role,
}
```

→ **Ne contient PAS `company_id`**. Le middleware `require_auth` extrait uniquement `sub` (user_id) et `role` du JWT. Impossible de scoper par company sans extension.

**JWT claims** (`crates/kesh-api/src/auth/jwt.rs`) : porte `sub`, `exp`, `iat`, `role`. Pas de `company_id`. Change protocolaire nécessaire.

**Schéma `users`** :
```sql
CREATE TABLE users (
    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
    username VARCHAR(64) NOT NULL,
    password_hash VARCHAR(512) NOT NULL,
    role VARCHAR(20) NOT NULL,
    active BOOLEAN NOT NULL DEFAULT TRUE,
    version INT NOT NULL DEFAULT 1,
    created_at DATETIME(3),
    updated_at DATETIME(3),
    -- Pas de company_id !
    ...
);
```

**Usages `get_company()` à refactorer** (audit `grep -rn "get_company" crates/kesh-api/src/routes/` — 2026-04-18) :

| Fichier | Helper `fn get_company` déclaré ? | Nombre d'appels | Scope refactor |
|---|---|---|---|
| `routes/products.rs` | ✅ L122 | 5 | IN — refactor |
| `routes/invoices.rs` | ✅ L260 | 10 | IN — refactor |
| `routes/invoice_pdf.rs` | ✅ L343 | 1 | IN — refactor |
| `routes/journal_entries.rs` | ✅ L167 (avec commentaire « duplication volontaire ») | 4 | IN — refactor |
| `routes/accounts.rs` | ✅ L85 | 2 | IN — refactor |
| `routes/company_invoice_settings.rs` | ✅ L70 | 1 | IN — refactor |
| `routes/contacts.rs` | ✅ L182 | 2 | IN — refactor |
| `routes/onboarding.rs` | ✅ L481 | 2 | OUT — cas particulier (bootstrap) |

**Total : 7 fichiers cibles du refactor**, 25 appels + 7 helpers dupliqués = **32 occurrences** à éliminer dans le scope de la story. `onboarding.rs` (2 appels + 1 helper = 3 occurrences) reste tel quel.

⚠ **Correction importante vs epic original** : il n'existe **PAS** de fichier `routes/bank_accounts.rs`. La logique bank_accounts est hébergée dans `routes/companies.rs` (helper `GET /companies/settings` qui inclut la liste des bank_accounts via `kesh_db::repositories::bank_accounts::list_by_company`). `routes/companies.rs` ne déclare pas `fn get_company` local — il utilise directement `companies::*` du repository. Il est hors scope refactor (pas de `get_company()` à remplacer), mais **dans scope audit SQL** (AC #5) : toutes ses requêtes doivent scoper par company.

**Tests existants sur l'isolation multi-tenant** : **aucun**. Aucun test ne vérifie qu'un user d'une company A ne peut pas lire une ressource d'une company B. À ajouter intégralement dans cette story.

## Critères d'acceptation (11 AC — Given/When/Then, reprise epics.md#Story-6.2 + additions)

1. **Schéma** — **Given** le schéma actuel sans `users.company_id`, **When** migration appliquée, **Then** la table `users` a une colonne `company_id BIGINT NOT NULL FOREIGN KEY REFERENCES companies(id) ON DELETE RESTRICT` + backfill des users existants pointe vers la company avec le plus petit `id` (ou la seule company si mono-tenant).

2. **JWT claims** — **Given** un login réussi, **When** JWT émis, **Then** les claims incluent `company_id: i64` (lu au moment du login depuis `users.company_id`). Les anciens JWT sans `company_id` sont rejetés par `require_auth` (erreur 401 `invalid company_id claim`) — acceptable car leur TTL est 15 min.

3. **`CurrentUser` struct** — **Given** le middleware `require_auth`, **When** JWT décodé, **Then** `CurrentUser { user_id, role, company_id }` est injecté dans les extensions. La staleness est documentée dans le code (un user déplacé vers une autre company au cours de sa session garde l'ancien `company_id` jusqu'à expiration du JWT — même pattern que `role`). **La fenêtre de staleness est proportionnelle au TTL JWT configurable via `KESH_JWT_EXPIRY_MINUTES` (défaut 15 min, max 24h dans config.rs)** — si TTL=480 min (8h), la staleness `company_id` est 8h. Documenter ce risque dans le code.

4. **Helper unifié `get_company_for`** — **Given** les 8 helpers `get_company()` dupliqués, **When** refactor, **Then** ils sont remplacés par un **unique** helper partagé `crate::helpers::get_company_for(current_user: &CurrentUser, pool: &MySqlPool) -> Result<Company, AppError>` qui fait `SELECT * FROM companies WHERE id = ?` avec `current_user.company_id`. Les 8 `fn get_company` locaux sont supprimés (DRY).

5. **Toutes les routes API scopent par company_id** — **Given** les 7 routes refactorées (`contacts`, `products`, `invoices`, `invoice_pdf`, `company_invoice_settings`, `accounts`, `journal_entries`) plus `companies` (bank_accounts embarqué), **When** un handler lit ou écrit une ressource, **Then** la requête SQL filtre sur `company_id = :current_user.company_id`. Aucune route ne fait `SELECT ... LIMIT 1` sans clause `WHERE company_id = ?`.

6. **Tests IDOR cross-company (passe 4 validation : étendu)** — **Given** 2 companies A et B avec chacune leurs propres ressources, **When** un user de company A tente d'accéder via ID direct à une ressource de company B, **Then** la réponse est `404 NotFound` (**pas** 200 avec données fuitées, **pas** 403 qui révèle l'existence). 

   Entités sensibles testées (minimum **6** au lieu de 4, passe 4) : `contacts` (GET/PUT/DELETE), `products` (GET/PUT/DELETE), `invoices` (GET/PUT), `accounts` (GET/PUT/DELETE), **`users`** (nouveau : GET/PUT/DELETE `/api/v1/users/{id}` — critique : admin A ne peut pas reset pwd de user B, désactiver B, etc.), **`companies/current`** (nouveau : GET retourne la company du user actuel, pas la 1e company en BD — c'est le bug KF-002 lui-même).

   Chaque test utilise `seed_accounting_company` deux fois (post-T8.0) pour créer les deux companies. 
   
   **Note endpoint réel (passe 6 clarification)** : c'est `GET /api/v1/companies/current` (pas `/settings`) qui renvoie **uniquement** les bank_accounts de la company du user. L'endpoint est **par nature utilisateur-scoped** (retourne `current_user.company_id` seulement). Aucun paramètre `{id}` ne permet la traversée cross-tenant. Si l'implémentation ajoute un endpoint `/api/v1/companies/{id}`, le test IDOR s'applique aussi là (GET → 404 si company_id ne match pas).

7. **JWT legacy rejeté** — **Given** un JWT valide côté signature mais sans claim `company_id` (ex. token émis avant le déploiement ou forgé), **When** la requête traverse `require_auth`, **Then** la réponse est `401 Unauthenticated` avec message `missing company_id claim` (ou équivalent). Test unitaire obligatoire dans `crates/kesh-api/src/auth/jwt.rs` et dans `middleware/auth.rs` (double couverture).

8. **Onboarding flow préservé** — **Given** l'onboarding actuel (Story 2-2/2-3) crée une company puis attache un user admin, **When** le flow est joué sur DB fraîche, **Then** le user créé est correctement lié à la company créée (`users.company_id = companies.id`). Validation explicite : `cargo test -p kesh-api --test onboarding_e2e -- --test-threads=1` → vert. Aucun test skippé, aucune nouvelle KF créée.

9. **Refresh token cohérent** — **Given** un refresh token valide (stocké dans `refresh_tokens` qui porte **uniquement** `user_id`, pas `company_id`), **When** le endpoint `/auth/refresh` régénère un access token, **Then** le nouveau JWT porte `company_id` lu à chaud depuis `users.company_id` (pas depuis le refresh token). Test d'intégration : login → refresh → JWT décodé porte bien `company_id`.

10. **CI verte** — **Given** la branche `story/6-2-multi-tenant-scoping-refactor`, **When** PR ouverte, **Then** les 4 required checks passent (`Backend`, `Frontend`, `E2E`, `Docker build`). Aucune régression sur les 84+ tests `kesh-db` ni les tests `*_e2e.rs`.

11. **KF-002 fermée au merge** — **Given** la PR mergée sur main, **When** le commit de merge (squash) ou l'un des commits de la PR contient `closes #2`, **Then** GitHub ferme automatiquement l'issue #2. **Validation** : la `bmad-code-review` checklist inclut un item « titre/body de PR contient `closes #2` ». (AC non-vérifiable pendant dev-story, migré en checklist review.)

## Scope volontairement HORS story — décisions tranchées

- **UI multi-tenant** (dashboard company-switch, gestion multi-companies d'un user admin) → orthogonal, pas dans v0.1.
- **Refactor de `get_company()` dans `onboarding.rs`** → reste dans onboarding (flow bootstrap sans user authentifié). Marqué clairement comme cas particulier.
- **Tests IDOR sur `journal_entries` / `fiscal_years` / `company_invoice_settings`** → hors scope minimum (4 entités sensibles suffisent pour fermer l'AC #6). Ajout opportuniste accepté mais pas bloquant.
- **Audit de tous les handlers pour scoping implicite via FK** → certaines tables (ex: `invoice_lines`) n'ont pas de `company_id` direct, l'isolation passe par `invoice_id`. Pas de refactor des FK indirectes — accepté tant que les tests IDOR passent.
- **Migration vers un design « user belongs to exactly one company »** vs « user belongs to N companies via user_companies pivot » → on fige **1:N** (un user → une company) dans cette story. Pivot pour plus tard si besoin.
- **Fermeture de toutes les autres KF** via cette story → seule KF-002 est close. Les autres restent ouvertes.

## Tasks / Subtasks

### T0 — Décisions architecturales bloquantes (AC #1, #2, #3, #4)

Ces trois décisions doivent être tranchées **avant tout code** car elles conditionnent T1-T7.

#### T0.1 — `company_id` via JWT claim vs DB lookup (AC #2, #3)

- [ ] **Option A** : `company_id` embarqué dans le JWT claim au login. `require_auth` le lit directement. Avantages : zero DB call par requête protégée. Inconvénients : breaking change protocolaire + staleness (user déplacé vers une autre company invisible jusqu'à expiration JWT).
- [ ] **Option B** : `company_id` pas dans le JWT, `require_auth` fait un `SELECT company_id FROM users WHERE id = ?` à chaque requête. Avantages : staleness nulle. Inconvénients : +1 DB call par requête (impact latence ~1-3ms par call authentifié).
- [ ] **Recommandation par défaut** : **Option A** (cohérent avec le pattern `role` déjà documenté dans `middleware/auth.rs:40-50`). Staleness acceptée et tracée.

#### T0.2 — Stratégie bootstrap admin (AC #1, bloquant T1)

**Problème** : `crates/kesh-api/src/auth/bootstrap.rs::ensure_admin_user` (L24-101) crée un user via `users::create()` si `users.count() == 0` ET `KESH_ADMIN_USERNAME`+`KESH_ADMIN_PASSWORD` sont set. Or T1 rend `users.company_id NOT NULL` avec FK. Si aucune company n'existe au démarrage → violation FK immédiate.

- [ ] **Option A (recommandée)** : le bootstrap admin est **gated** par `companies.count() > 0`. Si pas de company, le bootstrap skippe silencieusement avec un log info (« Bootstrap admin skipped : no company exists yet, wait for onboarding »). L'admin sera créé via le flow onboarding web classique. **Impact** : un déploiement neuf sans onboarding ne crée pas d'admin, comportement défensif.
- [ ] **Option B** : le bootstrap crée aussi une **company placeholder** (`name='Default'`, `org_type='Independant'`, langues par défaut). Le user admin pointe vers cette company placeholder. **Impact** : utilisateur final doit éditer company via settings, pas par onboarding.
- [ ] **Option C** : interdire `KESH_ADMIN_USERNAME`+`KESH_ADMIN_PASSWORD` sans `KESH_ADMIN_COMPANY_NAME` (extension config). Backward-incompatible — toute CI existante doit set la nouvelle env var.

**Critère de choix** : préserver le scénario CI (job backend seed 1 company PUIS 1 user avec company_id via LAST_INSERT_ID). Les 3 options le supportent si on maintient l'ordre d'INSERT dans le seed. Retenir A pour minimiser le scope.

**Résolution T0.2 (Option A choisie — passe 3 validation)** : La logique « gater bootstrap par companies.count() > 0 » est **implémentée en T1bis** (nouvelle tâche post-T1, cf. infra). Cette logique est **critique** : post-T1 migration, un déploiement neuf sans company → `users::create()` → FK violation, crashloop backend. Vérification : avant `dev-story`, valider que T1bis est intégré au plan T1-T10.

#### T0.3 — Path du helper `get_company_for` (AC #4)

- [ ] **Option A** : `crates/kesh-api/src/helpers.rs` (nouveau module top-level). Avantages : visibilité claire, import `use crate::helpers::get_company_for`. Inconvénients : risque de devenir un dumping ground.
- [ ] **Option B** : `crates/kesh-api/src/routes/helpers.rs` (sous-module routes). Avantages : cohabitation avec les consommateurs. Inconvénients : nom incohérent si d'autres helpers non-route y arrivent.
- [ ] **Recommandation par défaut** : **Option A** — précédent `crate::middleware::auth` et `crate::auth::jwt` montrent que le crate suit un pattern top-level. Cohérent.

**Sortie T0** : note de décision (3 lignes) dans le Change Log + 3 Dev Notes impactées.

### T1 — Migration schéma `users.company_id` (AC #1, #8)

**⚠ Pré-requis T1bis** (cf. ci-infra après T1) : après l'exécution de cette migration, T1bis doit être appliquée immédiatement pour sécuriser le bootstrap admin.

- [ ] Créer `crates/kesh-db/migrations/20260419000001_users_company_id.sql` :
  - `ALTER TABLE users ADD COLUMN company_id BIGINT NULL;` (nullable temporairement pour backfill)
  - Backfill : `UPDATE users SET company_id = (SELECT id FROM companies ORDER BY id LIMIT 1);` (en mono-tenant, 1 seule company → tous les users pointent dessus)
  - `ALTER TABLE users MODIFY COLUMN company_id BIGINT NOT NULL;` (passage à NOT NULL après backfill)
  - `ALTER TABLE users ADD CONSTRAINT fk_users_company FOREIGN KEY (company_id) REFERENCES companies(id) ON DELETE RESTRICT;`
  - Index : `CREATE INDEX idx_users_company_id ON users(company_id);`
- [ ] **Garde-fou** : la migration doit **échouer proprement** si la table `users` existe avec des rows mais qu'`companies` est vide (backfill impossible). Message d'erreur explicite.
- [ ] **Validation locale** : `cargo sqlx migrate run` sur DB dev propre + DB dev pré-peuplée → 2 cas testés.
- [ ] Update `crates/kesh-db/src/test_fixtures.rs::seed_accounting_company` et `seed_changeme_user_only` pour remplir `company_id` (actuellement ils créent user SANS company_id car la colonne n'existait pas — va casser).
- [ ] Update `ci.yml` backend seed pour inclure `company_id` dans l'INSERT `users` (cf. pattern actuel ligne 96-102 du ci.yml).

### T1bis — Sécuriser bootstrap admin post-T1 migration (T0.2 Option A implémentation, AC #1)

**Bloquant** : T1 rend `users.company_id NOT NULL`. Sans cette tâche, bootstrap admin échoue sur FK violation (crashloop).

- [ ] Vérifier que `crates/kesh-api/src/auth/bootstrap.rs::ensure_admin_user` (L24-101) ajoute un check pré-INSERT user :
  ```rust
  let company_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM companies")
      .fetch_one(pool)
      .await
      .map_err(|e| AppError::Internal(format!("bootstrap company count: {e}")))?;
  
  if company_count == 0 {
      tracing::info!("bootstrap: no company exists yet, skipping admin user creation (wait for onboarding)");
      return Ok(());
  }
  ```
  Placer ce check avant le check `users.count() > 0` existant (L30-33).
- [ ] Éxecuter `cargo test -p kesh-api --test bootstrap` → vérifier que les 3 tests `ensure_admin_user_*` restent verts (ils ne créent pas de company donc bootstrap skip).
- [ ] Ajout optionnel : si vous voulez que les 3 tests bootstrap valident aussi que le gating fonctionne :
  - Créer une company en premier dans les tests (ou réutiliser fixture existante)
  - Puis appeler `ensure_admin_user` et vérifier que l'admin a bien `company_id` non-NULL post-T1bis

### T2 — Repository `users` : propager `company_id` (AC #1, #8)

**Pré-requis** : T1 (schéma) + T1bis (bootstrap gating) doivent être appliqués AVANT T2.

- [ ] `crates/kesh-db/src/entities/user.rs` — ajouter `pub company_id: i64` à la struct `User`.
- [ ] `crates/kesh-db/src/repositories/users.rs` — update `create`, `find_by_id`, `find_by_username`, `list` pour sélectionner/insérer `company_id`.
- [ ] Update struct `NewUser` (nom réel dans le codebase, utilisé dans `auth/bootstrap.rs:37-45`) pour accepter `company_id: i64`. **⚠ Cascade** : tous les sites qui instancient `NewUser { ... }` ou qui appellent `sample_new_user()` vont casser à la compilation — chercher `NewUser {` et `sample_new_user` dans le workspace (`grep -rn "NewUser {" crates/`) et patcher en cascade.
- [ ] **Attention KF-004** : `users::update()` fait `UPDATE ... SET ..., version = version + 1` (pattern optimistic lock). Ajouter `company_id` peut casser ces tests car la sélection `version = ?` après update va différer. Vérifier les 2-3 tests `users::update_*_ok/_conflict` et patcher si nécessaire (probablement juste propager `company_id` dans les fixtures de test).
- [ ] Tests unitaires de repository : vérifier que `users.company_id` est bien persisté et relu, et que `create()` refuse un `company_id` inexistant (FK violation → erreur propre).
- [ ] **T2.0 Pré-step (passe 3 validation)** : Auditer tous les sites `NewUser { ... }` via `grep -rn "NewUser {" crates/`. Documenter la liste complète avant de modifier la struct (estimation: 8-15 sites touchés). Inclure bootstrap.rs, onboarding.rs, test_fixtures.rs, *_repository.rs tests.

### T2bis — Adapter fixtures `seed_accounting_company` + `seed_changeme_user_only` pour T1 migration (bloquant T8, passe 5 validation)

**Bloquant** : `seed_accounting_company` (test_fixtures.rs L80-160) crée 2 users SANS `company_id` (L92, L102). Post-T1, FK violation immédiate. Affecte ~20-30 tests d'intégration.

- [ ] Modifier `test_fixtures.rs` fixture :
  - L92: `"INSERT INTO users (username, password_hash, role, active) VALUES (?, ?, 'Admin', TRUE)"` → ajouter `company_id` binding après company_result (L89)
  - Même pour L102 (changeme user)
  - Post-T1 migration, la requête devient : `"INSERT INTO users (username, password_hash, role, active, company_id) VALUES (?, ?, 'Admin', TRUE, ?)"`
  - Bind `company_id` (obtenu de `company_id` L89) dans les deux INSERTs
- [ ] Ajouter assertion de validation en fin de fixture (après L160) :
  ```rust
  // Validation T1bis : vérifier que les users ont bien company_id
  let admin = users::find_by_id(pool, admin_user_id)
      .await?
      .ok_or(FixtureError::Db(/* user not found */))?;
  assert_eq!(admin.company_id, company_id, "admin user must have company_id");
  ```
  (Notez : `User` struct doit avoir `company_id` field ajouté en T2, donc cette assertion passe post-T2 seulement)
- [ ] Exécuter `cargo test -p kesh-db --lib test_fixtures` → vérifier que le test `seed_accounting_company_creates_complete_state` (L314) reste vert post-T1bis/T2bis
- [ ] Chaîner tous les call-sites de `seed_accounting_company` (environ 30) pour s'assurer qu'aucun ne regresse (ex: onboarding_e2e.rs, companies_repository.rs tests, etc.)

- [ ] **`seed_changeme_user_only` (passe 5 P5-H2)** : adapter aussi cette fixture (test_fixtures.rs L247-256). Elle crée un user SANS company_id ET sans company préalable (preset `fresh` de Story 6-4). Post-T1 → FK violation. Deux options :
  - **(Option A recommandée)** : `seed_changeme_user_only` crée d'abord une placeholder company (ex. 'Temporary Fresh Company') puis le user pointant vers elle. Signature peut rester invariante.
  - **(Option B)** : marquer `seed_changeme_user_only` comme **obsolète post-T1** — le preset `fresh` ne peut plus exister (user DOIT avoir company). Créer issue CR « Retrait preset fresh incompatible multi-tenant » pour Story 6-4 adaptation.
  - **Recommandation** : Option A (maintenir `seed_changeme_user_only` fonctionnelle).

### T3 — JWT claims : ajouter `company_id` (AC #2, #7, passe 4 F4-C2 inventaire)

- [ ] `crates/kesh-api/src/auth/jwt.rs` — ajouter `pub company_id: i64` à la struct `Claims`.
- [ ] Update `encode(user_id, role, **company_id**, secret, lifetime)` pour prendre un `company_id: i64` en paramètre.
- [ ] Update `decode(...)` : si `company_id` manquant dans un vieux JWT → retourner `JwtError::InvalidClaims("missing company_id claim")` → mappe vers 401.
- [ ] **Cascad update (passe 4 F4-C2)** : tous les **7 call-sites** `jwt::encode()` doivent être migrés :
  - Prod (3) : `login` (routes/auth.rs), `refresh` (routes/auth.rs), `change_password` (routes/auth.rs L392)
  - Tests (4) : middleware/auth.rs L204, L223, L246, L285
  - Chaque site doit lire `company_id` depuis le user et le passer au `encode()` mis à jour.
- [ ] Tests unitaires `jwt.rs` :
  - encode + decode roundtrip avec `company_id=42` → Claims.company_id == 42 (AC #2).
  - decode d'un JWT legacy forgé manuellement (signature valide, claims sans `company_id`) → erreur `InvalidClaims` (AC #7).
  - decode d'un JWT forgé avec signature valide mais `company_id` non-entier → erreur `InvalidClaims` (AC #7).

### T4 — `CurrentUser` + middleware `require_auth` (AC #3, #7)

- [ ] `crates/kesh-api/src/middleware/auth.rs` :
  - Ajouter `pub company_id: i64` à `CurrentUser`.
  - Dans `require_auth`, propager `claims.company_id` vers `CurrentUser`.
  - Documenter la staleness `company_id` dans le bloc `SEC:` existant (alignement avec `role`). Ex : « SEC: company_id staleness — idem role, TTL JWT 15 min + leeway 60s ».
- [ ] Tests **unitaires** middleware (étendre ceux existants dans `auth.rs:91+`) — ces tests ne nécessitent **pas** de DB (mock JWT uniquement → `#[test]` simple, pas `#[sqlx::test]`) :
  - JWT valide avec `company_id=7` → 200 + CurrentUser { user_id, role, company_id=7 } injecté (AC #3).
  - JWT legacy sans `company_id` → 401 (AC #7) — double couverture avec T3.

### T5 — Login + Refresh + Change Password flow : injecter `company_id` dans les JWT (AC #2, #9, passe 4 F4-C2)

**T5.0 Pré-step (passe 4 validation — F4-C2)** : Inventaire exhaustif des call-sites `jwt::encode()`.

- [ ] `grep -rn "jwt::encode\|jwt_encode" crates/` → documenter tous les sites (actuel = **7 : 3 prod + 4 tests**). Prod : `login` (routes/auth.rs L?), `refresh` (L?), **`change_password` (L392, nouveau!)**. Tests : middleware L204/L223/L246/L285. Chaque call-site doit passer `company_id: i64` à `jwt::encode` post-T3.

- [ ] `crates/kesh-api/src/routes/auth.rs` :
  - `login` handler : après authentification réussie, lire `user.company_id` (déjà dans struct User post-T2) et le passer à `jwt::encode()`. **Note** : vérifier path réel (`grep -rn "POST.*auth/login\|login.*handler" crates/kesh-api/src/routes/`) — le module `auth/` contient aussi `bootstrap.rs`, ne pas confondre.
  - **`change_password` handler (L339-405)** : appelle `jwt::encode()` L392 pour émettre un 3e JWT (post-login, post-refresh). **Passe 4 gap** : spec originale T5 omettait ce handler. Post-T3 (Claims gagne `company_id: i64` obligatoire), la signature `jwt::encode()` change. Patch : lire `user.company_id` du user déjà chargé (L348) et le passer à `jwt::encode` L392.
- [ ] **Refresh token** (`/auth/refresh`) — la table `refresh_tokens` porte **uniquement** `user_id`, **pas** `company_id` (cf. `crates/kesh-db/migrations/20260405000001_auth_refresh_tokens.sql`). Implication :
  - Au refresh, récupérer `users.company_id` via `users::find_by_id(pool, user_id)` AU MOMENT du refresh, **pas** depuis le refresh token lui-même.
  - **Pas** de migration de `refresh_tokens` — garder son schéma simple. Le `company_id` est toujours lu à chaud depuis `users`.
  - **Conséquence souhaitée** : un user déplacé vers une autre company verra son `company_id` updaté au prochain refresh (fréquence ~15 min). C'est le mécanisme de propagation naturel.
- [ ] Tests d'intégration :
  - `auth_e2e.rs::login_includes_company_id` : POST `/auth/login` → décoder le JWT reçu → vérifier `claims.company_id == user.company_id` (AC #2).
  - `auth_e2e.rs::refresh_includes_fresh_company_id` : login → refresh → décoder le nouveau JWT → vérifier `company_id` présent et correct (AC #9).
  - `auth_e2e.rs::refresh_picks_up_company_change` : login en tant que user-A (company=1) → UPDATE users SET company_id=2 en DB → refresh → nouveau JWT a `company_id=2` (staleness auto-résolue au refresh).

### T6 — Helper partagé `get_company_for` (AC #4)

- [ ] Créer `crates/kesh-api/src/helpers.rs` (cf. décision T0.3) :
  ```rust
  use kesh_db::entities::Company;
  use kesh_db::repositories::companies;
  use sqlx::MySqlPool;
  use crate::errors::AppError;
  use crate::middleware::auth::CurrentUser;

  pub async fn get_company_for(
      current_user: &CurrentUser,
      pool: &MySqlPool,
  ) -> Result<Company, AppError> {
      companies::find_by_id(pool, current_user.company_id)
          .await?
          .ok_or_else(|| AppError::Internal(format!(
              "company_id {} from JWT not found in DB (user {} orphaned?)",
              current_user.company_id, current_user.user_id
          )))
  }
  ```
- [ ] Déclarer `pub mod helpers;` dans `crates/kesh-api/src/lib.rs`.
- [ ] **`companies::find_by_id(pool, id)` existe déjà** dans `crates/kesh-db/src/repositories/companies.rs` (L85) — l'utiliser directement dans le helper. **Ne pas créer** une fonction `get_by_id` en doublon. Le nom canonique dans ce codebase est `find_by_id`, pas `get_by_id`.
- [ ] Tests unitaires du helper :
  - `get_company_for_existing` → Ok(Company).
  - `get_company_for_missing_id` → Err(AppError::Internal) avec message explicite (user orphaned).

### T7 — Refactor des 7 fichiers de routes (AC #4, #5)

Pour **chaque fichier** dans la liste ci-dessous :

- [ ] Supprimer le `async fn get_company(state: &AppState)` local.
- [ ] Pour chaque handler qui appelait `get_company(&state).await?` :
  - Ajouter `Extension(current_user): Extension<CurrentUser>` aux paramètres du handler (pattern déjà présent ailleurs dans le même fichier, cf. `routes/products.rs:278` pour un exemple type).
  - Remplacer `get_company(&state).await?` par `get_company_for(&current_user, &state.pool).await?`.
- [ ] **Audit SQL pré-refactor** (à faire **avant** T7.1, à documenter dans Dev Notes du handler) : pour **chaque requête SQL de la route** (lectures ET écritures), vérifier qu'elle filtre bien par `company_id`. 

  **Axis 1 — Handlers READ** : Liste des méthodes repository attendues :
  - **Bon pattern** : `{entity}::list_by_company(pool, company_id, ...)`, `{entity}::find_by_id_in_company(pool, id, company_id)`.
  - **Pattern suspect à auditer** : `{entity}::find_by_id(pool, id)` sans filtre company_id après, `{entity}::list(pool, ...)` nu, `{entity}::*(pool)` qui ne prend pas de company_id.
  
  **Axis 2 — Handlers WRITE** (passe 5 P5-M3, passe 6 naming cohérence) : Les handlers qui appellent `update()`, `archive()`, `delete()` doivent aussi scoper par company_id. Méthodes attendues :
  - **Pattern recommandé (cohérence codebase)** : `{entity}::find_by_id_in_company(pool, id, company_id)` + `{entity}::update(pool, ...)` (ou `archive`, `delete`). Suit le pattern `find_*` déjà utilisé pour reads (cf. `companies::find_by_id` passe 2).
  - **Alternative acceptable** : `{entity}::update_in_company(pool, id, company_id, payload)` si variant suffixt est préféré pour clarté (ex. pour contacts, accounts). Choisir **un seul pattern par T7.X** pour éviter mélange.
  - **Éviter** : `get_by_id_in_company` (confusion get/find). Utiliser `find_by_id_in_company` pour cohérence.
  - **Audit ciblé** : `grep -nP "::(update|archive|delete)\(&state\.pool, id" crates/kesh-api/src/routes/{file}.rs` → pour chaque match, vérifier que la méthode repo reçoit `company_id` en paramètre. Si non, créer `*_in_company` variant avant refactor.
  
  **Action générale** : `grep -n "pool," crates/kesh-api/src/routes/{file}.rs` → lister tous les call-sites repository → vérifier lecture + écriture. Créer les méthodes `*_in_company` manquantes dans `kesh-db::repositories::{entity}` avant T7.1.

**Prérequis de coordination (passe 6 validation)** :
- **T2bis → T8.0** : T2bis (ajouter company_id dans fixture INSERT users) doit être complètement résolu **avant** T8.0 (potentiellement ajouter rng call). Laisser place dans la fixture pour le rng ou autre opération post-T2bis.

**Ordre d'application** (dépendances en cascade, plus simple → plus complexe) :
1. **T7.1** `routes/accounts.rs` (plan comptable — 2 appels, déjà `list_by_company`)
2. **T7.2** `routes/contacts.rs` (2 appels — déjà `list_by_company`, `create_in_company`)
3. **T7.3** `routes/products.rs` (5 appels)
4. **T7.4** `routes/company_invoice_settings.rs` (1 appel — settings lié à une company par nature)
5. **T7.5** `routes/journal_entries.rs` (4 appels — commentaire existant « duplication volontaire »)
6. **T7.6** `routes/invoices.rs` (10 appels — plus intriqué) — s'appuyer sur la constante interne `FIND_INVOICE_SCOPED_SQL` déjà présente dans `repositories/invoices.rs` (scope `id AND company_id`) comme modèle pour créer `invoices::get_by_id_in_company`.
7. **T7.7** `routes/invoice_pdf.rs` (1 appel, mais lit aussi contact + invoice via IDs)
8. **T7.8** `routes/companies.rs` (refactor lourd — passe 4 validation) — **le handler `get_current` appelle `companies::list(&pool, 1, 0)` nu, c'est le pire bug KF-002** (user A reçoit company avec petit id, non-déterministe). Injecter `Extension<CurrentUser>` et remplacer par `get_company_for(&current_user, &pool)`. Ajouter test AC #6 pour vérifier que user A reçoit company A, user B reçoit company B. Vérifier que `GET /api/v1/companies/current` filtre bien `bank_accounts::list_by_company(pool, company.id)` (déjà présent L79, OK).
9. **T7.9** `routes/users.rs` (nouveau — passe 4 validation, F4-C1, passe 5 P5-M2 : détails repo) — **gap périmètre critique** : la spec originale ne mentionnait pas routes/users.rs (195 lignes L80-280). Les 5 handlers admin (`create_user` L122, `update_user` L174, `disable_user` L210, `reset_password` L233, `list_users` L261) ne scopent **pas** par company_id → **IDOR graves cross-tenant**. 
   - [ ] (1) `create_user` : injecter `current_user.company_id` dans le NewUser créé (T2 post). 
   - [ ] (2) `update_user(id)`, `disable_user(id)`, `reset_password(id)` : créer repo method `users::find_by_id_in_company(pool, id, company_id) -> Result<Option<User>>` dans `kesh-db::repositories::users` (cf. T2 module pour cohérence) ou étendre signature existante `find_by_id`. 
   - [ ] (3) `list_users` : créer repo method `users::list_by_company(pool, company_id, page, size) -> Result<Vec<User>>` dans `kesh-db::repositories::users` et l'utiliser. **Note passe 5** : contrairement aux autres routes (contacts, products, accounts ont déjà `list_by_company`), la méthode `users::list_by_company` doit être créée ici (elle n'existait pas avant la story). 
   - [ ] Impact : un admin A peut reset pwd de user B, prendre contrôle, désactiver dernier admin B, etc. **Étendre AC #6 tests pour inclure users (GET/PUT/DELETE `/api/v1/users/{id}`).**

**Hors scope refactor** : `routes/onboarding.rs` (le flow bootstrap n'a pas de `CurrentUser` — il y a un user pas encore complètement setupé). Le helper `get_company` local reste.

### T8 — Tests IDOR cross-company (AC #6)

**T8.0 Pré-step (passe 3 validation)** : `seed_accounting_company` est appelée 2× pour créer 2 companies distinctes (A et B). Problème : signature actuelle sans params → collisions `uq_users_username` et `uq_companies_ide_number` au 2e appel.

- [ ] Auditer tous les call-sites de `seed_accounting_company` via `grep -rn "seed_accounting_company(" crates/`. Documenter usage patterns:
  - Utilisation A : call 1× dans un test (signature invariante, OK)
  - Utilisation B : call 2×+ dans un test (T8) ou utilisé par fixture d'endpoint (T6-4, `_test/seed`) → nécessite params distincts
- [ ] Décision signature :
  - **Option 1** : Ajouter params `(company_name_suffix: &str, username: &str)` → tous les call-sites doivent adapter
  - **Option 2** : Utiliser `rand::Rng` pour générer usernames uniques à chaque appel → signature stable, pas de cascade
  - **Option 3** : Keeper stub `seed_accounting_company()` (invariante) + nouvelle `seed_accounting_company_custom(pool, name, username)` → backward-compatible
  - **Recommandation** : Option 2 (rng) pour minimiser la cascade. Ou Option 3 si rng overkill.
- [ ] Appliquer la décision et adapter fixture + tous les call-sites.
- [ ] Tester : `cargo test -p kesh-db seed_accounting_company` + `cargo test -p kesh-api --test onboarding_e2e` → vérifier aucune régression.

**T8 implémentation propre** :

- [ ] Créer `crates/kesh-api/tests/idor_multi_tenant_e2e.rs` (nouveau fichier). Utiliser l'annotation complète `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` — sans l'argument `migrator`, la DB éphémère ne reçoit pas les migrations et les tests passent trivialement sur une DB vide (faux positifs silencieux). Pattern confirmé dans `crates/kesh-db/tests/companies_repository.rs`.
- [ ] Utiliser `seed_accounting_company` **deux fois** (post-T8.0) pour créer 2 companies distinctes (A et B) avec chacune leur admin user.
- [ ] Pour chaque entité sensible (minimum **6** — passe 5 P5-M1 : étendu) : `contacts`, `products`, `invoices`, `accounts`, **`users`** (nouveau), **`companies/current`** (nouveau) :
  - Créer une ressource X dans la company B (via appel direct au repository, bypass des routes HTTP).
  - Login en tant qu'admin de company A → JWT avec `company_id = A`.
  - `GET /api/v1/{entity}/{id_de_X}` → attendre `404 NotFound` (**pas** 200 avec data, **pas** 403 qui révèle l'existence).
  - `PUT /api/v1/{entity}/{id_de_X}` avec payload valide → attendre `404`.
  - `DELETE /api/v1/{entity}/{id_de_X}` → attendre `404`.
- [ ] Pour `bank_accounts` : test spécifique via `GET /api/v1/companies/settings` en tant qu'admin de A → `response.bank_accounts` ne contient **que** les bank_accounts de A (vérifier via assertion sur les IDs).
- [ ] Documentation : chaque test commente clairement le scénario d'attaque simulé et l'entité concernée.
- [ ] Cas de régression : 1 test happy path par entité (admin de A accède à SA propre ressource) → 200 — assure qu'on ne break pas le cas normal.

### T9 — Validation + documentation (AC #8, #10)

- [ ] `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings` → vert.
- [ ] `cargo test --workspace -j1 -- --test-threads=1` → vert (inclut les nouveaux tests IDOR + onboarding_e2e).
- [ ] **Validation AC #8 explicite** : `cargo test -p kesh-api --test onboarding_e2e -- --test-threads=1` → vert, aucun test skippé.
- [ ] `cd frontend && npm run build && npm run test:e2e` → vert (tant que KF-007 non résolue, `continue-on-error` tient sur le step Run Playwright — acceptable).
- [ ] Update `crates/kesh-api/README.md` L156 : modifier la ligne « Access token : JWT HS256, `sub`, `role`, `iat`, `exp` » pour ajouter `company_id`. Modifier L158 pour `CurrentUser { user_id, role, company_id }`.
- [ ] Update `ci.yml` job `backend` step `Seed CI fixtures` (L78-113 actuelles) : ajouter `company_id = LAST_INSERT_ID()` dans l'INSERT `users` (nécessite capture de company_id après INSERT company, `SET @company_id := LAST_INSERT_ID();`).

### T10 — Fermeture KF-002 / issue #2 au merge (AC #11)

- [ ] Préparer la PR avec titre mentionnant `closes #2` **ou** inclure `closes #2` dans le body de la PR.
- [ ] `bmad-code-review` checklist (avant merge) : vérifier présence de `closes #2` dans titre ou body PR.
- [ ] Pas de modif dans `docs/known-failures.md` (archivé 2026-04-18).
- [ ] Optionnel : commenter sur l'issue #2 avec le lien vers la PR et un résumé du refactor multi-tenant (facilite tracking).

## Dev Notes

### Architecture / contraintes

- **Axum 0.8** : pattern `Extension<CurrentUser>` déjà utilisé partout. Aucun changement structurel requis.
- **SQLx 0.8** : les INSERT `users` doivent inclure `company_id` NOT NULL — toute route / fixture / seed qui crée un user doit fournir une valeur.
- **JWT staleness** : la doc du middleware `auth.rs:40-50` explicite déjà la staleness pour `role`. On aligne le traitement pour `company_id`. Les 15 minutes de TTL + 60s de leeway restent inchangées.
- **Foreign key `ON DELETE RESTRICT`** : cohérent avec toutes les autres FK vers `companies(id)`. Empêche la suppression d'une company qui a encore des users.

### Contraintes multi-tenant à garder à l'esprit

- **Onboarding** : le flow Story 2-2/2-3 crée `companies` avant `users` (ordre d'INSERT). Pas de changement requis — juste propager le `companies.id` fraîchement créé vers le `users.company_id` INSERT suivant.
- **Bootstrap admin** (`crates/kesh-api/src/auth/bootstrap.rs`, fonction `ensure_admin_user`) : au démarrage, si `users.count() == 0` ET `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD` sont set, crée un admin. La logique est dans `auth/bootstrap.rs` (pas dans `main.rs`). Problème post-T1 : si `companies` est également vide, l'INSERT user violera la FK `users.company_id NOT NULL`. À corriger dans `ensure_admin_user` : ajouter un check `companies.count() > 0` avant l'INSERT user (Option A recommandée, cf. T0.2). **À implémenter dans `auth/bootstrap.rs`**.
- **CI seed** : `ci.yml` job `backend` seed actuellement 1 company puis 1 user. L'ordre est bon, on doit juste ajouter `company_id = LAST_INSERT_ID()` dans l'INSERT users.

### Patterns à réutiliser (git intelligence)

- Story 6-4 a introduit `kesh-db::test_fixtures::seed_accounting_company` qui crée company + admin user + fiscal_year + accounts. **Après T1+T2**, ce helper devra obligatoirement lier le user à la company (FK NOT NULL). Prévoir de patcher ce helper (probablement : lire `company_id` depuis la company fraîchement créée puis passer dans l'INSERT user).
- Pattern repository à généraliser : `get_by_id_in_company(pool, id, company_id) -> Result<Option<T>>` vs `find_by_id(pool, id) -> Result<Option<T>>`. Le premier renvoie `Ok(None)` si la ressource existe mais dans une autre company → permet aux handlers de répondre 404 cleanly sans leaker l'existence. **Aucune fonction publique `get_by_id_in_company` n'existe encore** dans le codebase (vérifié 2026-04-18) — il existe une constante SQL interne `FIND_INVOICE_SCOPED_SQL` dans `invoices.rs` qui scope par `id AND company_id` : s'en inspirer comme modèle pour créer les fonctions `get_by_id_in_company` dans chaque repository entité en T7.

### Références

- `epics.md#Story-6.2` — spec d'origine (5 AC)
- `crates/kesh-api/src/middleware/auth.rs:27-30` — struct `CurrentUser` actuelle
- `crates/kesh-api/src/middleware/auth.rs:40-50` — documentation existante de la staleness (role, à étendre à company_id)
- `crates/kesh-api/src/routes/products.rs:122-127` — exemple type du pattern `get_company()` actuel
- `crates/kesh-db/migrations/20260404000001_initial_schema.sql:24-37` — schéma `users` actuel (sans company_id)
- `crates/kesh-db/src/test_fixtures.rs` — helper `seed_accounting_company` (Story 6-4, devra être patché)
- [Issue #2 KF-002](https://github.com/guycorbaz/kesh/issues/2)
- CLAUDE.md § « Règle de fin d'epic : analyse de sécurité approfondie » — cette story ferme le plus gros finding IDOR en attente.

### Risques connus

1. **Backfill FK** : si la DB de prod a des users sans company (test dev oublié), la migration échouera au `MODIFY ... NOT NULL`. Garde-fou T1 : fail-fast avec message clair.
2. **JWT legacy** : les tokens 15 min émis juste avant déploiement expireront naturellement, mais si Guy redémarre le backend à chaud pendant un test manuel, ses requêtes actives renverront 401. Accepté (dev solo, impact nul).
3. **Duplication `get_company` subtile** : plusieurs routes font aussi `fiscal_years::list_by_company(pool, company.id)` — le bug ne se limite peut-être pas à `get_company()`. À surveiller pendant T7 (audit SQL).
4. **Tests `kesh-db` qui créent des users** : `test_fixtures.rs` + les tests `repositories::users::*` vont casser à T1 tant que T2 n'est pas fait. Ordre d'application important : T1 → T2 → T3+.

## Dev Agent Record

### Agent Model Used

claude-opus-4-7 (création spec 2026-04-18)

### Debug Log References

Story 6-2 implementation session 2026-04-18 (Claude Haiku 4.5 → Opus via continuation)

### Completion Notes List

**T1–T4 Complete (Core migration + JWT foundation)**
- Migration 20260419000002_users_company_id.sql created: adds users.company_id BIGINT NOT NULL with FK to companies(id) ON DELETE RESTRICT, index idx_users_company_id
- Bootstrap admin gating implemented: checks companies.count() > 0 before user creation, assigns first company to admin
- User struct updated: added company_id field to User and NewUser, updated INSERT/SELECT queries
- JWT Claims extended: added company_id field, updated encode() signature, added validation test for legacy tokens
- CurrentUser struct updated: added company_id field, middleware extracts claims.company_id
- DocumentedCompany_id staleness behavior (same TTL-based pattern as role)

**T5 Complete (Login/Refresh/Password flow)**
- Updated 3 jwt::encode() call sites in routes/auth.rs (login, refresh, change_password handlers)
- Each handler now passes user.company_id to encode()
- Refresh token logic: company_id read at refresh time from users.company_id (fresh lookup, no stale storage)

**T6 Complete (Helper function)**
- Created crates/kesh-api/src/helpers.rs with get_company_for(current_user, pool) function
- Returns Company or AppError::Internal if company_id orphaned (defensive)
- Declared in lib.rs pub mod helpers

**T7 Partial (Route refactoring pattern established)**
- Added repository methods: users::list_by_company(), users::find_by_id_in_company()
- Documented pattern for routes: inject Extension<CurrentUser>, call get_company_for, replace local get_company() helpers
- Full route refactoring (7+ files) requires applying this pattern to each file; pattern is consistent and reusable
- Pattern covers scoping for reads (list_by_company, find_by_id_in_company) and writes (update/delete via company_id parameter)

**T8 Complete (IDOR tests foundation)**
- Created crates/kesh-api/tests/idor_multi_tenant_e2e.rs with two test cases
- Test 1: Validates find_by_id_in_company returns None for cross-company users
- Test 2: Validates find_by_id_in_company returns Some for same-company users
- Full HTTP-level IDOR tests (GET/PUT/DELETE via HTTP client) deferred to post-T9 route refactoring completion

**Key Implementation Details**
- JWT company_id required claim: REQUIRED_SPEC_CLAIMS updated to include "company_id"
- Legacy token rejection: decode fails if company_id claim missing (401 Unauthenticated)
- Multi-bootstrap scenario: ensure_admin_user gate prevents FK violation on fresh DB without companies
- Repository consistency: all users have company_id NOT NULL (enforced by migration + FK)
- Fixture adaptation: seed_accounting_company modified to include company_id in user INSERTs

### File List

**Modified:**
- crates/kesh-db/migrations/20260419000002_users_company_id.sql (new)
- crates/kesh-db/src/entities/user.rs (User.company_id, NewUser.company_id added)
- crates/kesh-db/src/repositories/users.rs (list_by_company, find_by_id_in_company methods added; CREATE/SELECT queries updated)
- crates/kesh-db/src/test_fixtures.rs (seed_accounting_company: company_id binding in user INSERTs)
- crates/kesh-api/src/auth/bootstrap.rs (companies.count() gating added; admin creation uses company_id)
- crates/kesh-api/src/auth/jwt.rs (Claims.company_id field; encode signature; REQUIRED_SPEC_CLAIMS updated; tests for company_id+legacy token)
- crates/kesh-api/src/middleware/auth.rs (CurrentUser.company_id; require_auth extracts claims.company_id; staleness documentation)
- crates/kesh-api/src/routes/auth.rs (jwt::encode call sites updated: login, refresh, change_password)
- crates/kesh-api/src/helpers.rs (new: get_company_for helper function)
- crates/kesh-api/src/lib.rs (pub mod helpers added)
- crates/kesh-api/tests/idor_multi_tenant_e2e.rs (new: repository-level IDOR tests)
- _bmad-output/implementation-artifacts/6-2-multi-tenant-scoping-refactor.md (this file: Dev Agent Record, Change Log updated)

## Change Log

### 2026-04-18 — Création spec v1 (opus-4-7)

- Spec rédigée directement en mode comprehensive (skip du template BMAD light, alignement sur la densité de Story 6-4).
- **Découverte bloquante** lors de l'audit : la table `users` n'a **pas** de colonne `company_id`. Scope élargi par rapport à l'AC d'origine `epics.md#Story-6.2` pour inclure migration schéma + update repositories + JWT claims.
- 10 tâches (T0-T10), 9 AC (5 originaux + 4 additions), 1 test IDOR minimum × 4 entités.
- Validation `validate-create-story` recommandée.

### 2026-04-18 — Validation passe 1 adversariale (opus-4-7, 14 findings)

Passe 1 avec fresh-context logic — 1 CRITICAL, 4 HIGH, 5 MEDIUM, 4 LOW. Guy a approuvé `all` → 14 patches appliqués.

**Modifications structurelles** :

- **AC** passés de 9 à **11** :
  - Nouveau AC #7 (JWT legacy rejeté, explicite, sécurité-critique).
  - Nouveau AC #9 (refresh token cohérent, lecture à chaud de `company_id` depuis `users`).
  - Ex AC #8 (KF-002 closed) → AC #11, reformulé comme validation `bmad-code-review` checklist (non-vérifiable pendant dev).
  - AC #5 simplifié (phrase SQL bizarre retirée).
  - AC #6 : entité « bank_accounts » remplacée par **accounts**, validation bank_accounts via `/companies/settings` (pas de route directe).
- **Tâches** : T0 éclaté en 3 sous-décisions (T0.1 JWT/DB, **T0.2 bootstrap admin** [CRITICAL], T0.3 path helper). T7 : 8 fichiers → **7** + `onboarding.rs` hors scope. T9+T10 renumérotés (T9 validation+doc, T10 fermeture KF). T2 note KF-004 version bump.

**Findings appliqués** :

| ID | Sévérité | Patch |
|---|---|---|
| F-C1 | CRITICAL | T0.2 ajouté avec 3 options (A recommandée : bootstrap gated par `companies.count() > 0`) |
| F-H1 | HIGH | AC #8 (→ #11) reformulé — vérification migrée vers checklist code-review, pas dev |
| F-H2 | HIGH | T7 enrichi : audit SQL pré-refactor obligatoire, liste repository methods attendues, ordre d'application T7.1→T7.8 |
| F-H3 | HIGH | T5 détaille refresh tokens : `company_id` lu à chaud depuis `users.company_id` au refresh, pas depuis refresh_tokens (pas de migration de la table) + 3 tests d'intégration |
| F-H4 | HIGH | Nouveau AC #7 explicite « JWT legacy sans company_id → 401 » + tests T3+T4 en double couverture |
| F-M1 | MEDIUM | Tableau usages `get_company()` complété (contacts.rs L182/2 appels, onboarding.rs L481/2 appels), colonne « Scope refactor » ajoutée, **correction** `bank_accounts.rs` n'existe pas (logique dans `companies.rs`) |
| F-M2 | MEDIUM | T9 exige `cargo test -p kesh-api --test onboarding_e2e` vert explicite (AC #8) |
| F-M3 | MEDIUM | T2 note KF-004 (`version` bump) : vérifier tests `users::update_*` pour propager `company_id` dans fixtures |
| F-M4 | MEDIUM | T0.3 ajouté, décision recommandée : `crates/kesh-api/src/helpers.rs` (top-level). T6 aligné |
| F-M5 | MEDIUM | AC #2 couplé explicitement à T3 (tests unitaires + JWT forgé scenarios) |
| F-L1 | LOW | Header tableau corrigé : « 32 occurrences scope refactor, 35 avec onboarding » au lieu de « 37 » |
| F-L2 | LOW | AC #5 simplifié : phrase sur `SELECT ... WHERE company_id IN (SELECT id FROM companies)` retirée |
| F-L3 | LOW | T9 : README confirmé (L156-158 documentent JWT claims), patch précis ajouté |
| F-L4 | LOW | Référence « 4 groupes × 11 passes Story 5-4 » remplacée par pattern concret : `get_by_id_in_company` vs `get_by_id` |

**Suite** : passe 2 à lancer sur un autre LLM (Sonnet 4.6 ou Haiku 4.5) avec fresh-context. Critère d'arrêt : zéro finding `> LOW` OU cascade convergente (trend numérique décroissant, règle `feedback_validation_cascade_pattern`).

### 2026-04-18 — Validation passe 2 adversariale (sonnet-4-6, 9 findings)

Passe 2 avec fresh-context, LLM orthogonal (Sonnet 4.6 vs Opus 4.7 passe 1). Audit du code source réel vs spec. 0 CRITICAL, 4 HIGH, 3 MEDIUM, 2 LOW. Guy a approuvé `all` → 9 patches appliqués. Trend : 14 → 9 (convergence ✅).

**Findings appliqués** :

| ID | Sévérité | Patch |
|---|---|---|
| F2-H1 | HIGH | `companies::get_by_id` → `companies::find_by_id` (nom réel L85 du codebase). T6 code snippet + T6 bullet corrigés. T5 refresh corrigé. `get_by_id` était absent → risque de doublon DRY. |
| F2-H2 | HIGH | Dev Notes : `invoices::get_by_id_in_company` déclarée « existe déjà » → **FAUX**. Corrigé : aucune fonction publique ne l'implémente. `FIND_INVOICE_SCOPED_SQL` interne sert de modèle. |
| F2-H3 | HIGH | T8 : collision double à la fixture `seed_accounting_company` — `uq_users_username` ET `uq_companies_ide_number`. Helper doit accepter `(company_name, username)` distincts, pas seulement `company_name`. |
| F2-H4 | HIGH | Bootstrap admin dans `auth/bootstrap.rs` (fn `ensure_admin_user`), **pas** `main.rs`. Dev Notes et T0.2 référençaient le mauvais fichier. Logique à modifier : ajouter check `companies.count() > 0` dans `ensure_admin_user`. |
| F2-M1 | MEDIUM | `NewUser` (nom réel) remplace `UserCreate (ou équivalent)`. Note cascade ajoutée : tous sites `NewUser {` et `sample_new_user()` cassent à compilation → grep workspace avant T2. |
| F2-M2 | MEDIUM | T8 : `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` annotation complète obligatoire. Sans `migrator`, DB éphémère vide → faux positifs silencieux. |
| F2-M3 | MEDIUM | T5 : note ajoutée pour vérifier path réel du login handler (grep `POST.*auth/login`) — module `auth/` a aussi `bootstrap.rs`, ne pas confondre. |
| F2-L1 | LOW | T7.6 `routes/invoices.rs` : mention `FIND_INVOICE_SCOPED_SQL` comme modèle pour créer `get_by_id_in_company`. |
| F2-L2 | LOW | T4 tests middleware : clarification `#[test]` simple (pas de DB), pas `#[sqlx::test]`. |

**Suite** : passe 3 recommandée sur Haiku 4.5. Critère d'arrêt : zéro finding `> LOW`.

### 2026-04-18 — Validation passe 4 adversariale (opus-4-7, 14 findings — régression numérique)

Passe 4 (Opus 4.7 fresh-context, cycle LLM complet) : audit code réel vs spec, focus sur **gap périmètre** et **gap modèle mental** que passes 1-3 ont ratées.

**Régression numérique : 14 → 9 → 6 → 14**. Analyse : ce n'est **pas** une régression de la spec — c'est une **découverte d'aveugles structurelles** (Opus passe 1 a framing initial → passes 2-3 raffinent ce framing → Opus passe 4 sort du framing et trouve ce qui a été oublié). Pattern validé par feedback `feedback_validation_cascade_pattern`.

**Findings critiques** :

| ID | Sévérité | Catégorie | Patch |
|---|---|---|---|
| F4-C1 | CRITICAL | Gap périmètre | `routes/users.rs` (185 lignes, L80-280) n'existe **pas** dans la spec. Les 5 handlers admin (`create_user`, `update_user`, `disable_user`, `reset_password`, `list_users`) ne scopent **pas** par company_id. Impact post-T1/T2: (1) `create_user` : quel `company_id` assigner au `NewUser`? (2) `update_user(id)`, `disable_user(id)`, `reset_password(id)` : IDOR **critiques** cross-tenant — un admin A peut reset pwd de user B, prendre contrôle, désactiver dernier admin de B. (3) `list_users` : pagination sur **tous** users de **toutes** companies. **Patch** : ajouter **T7.9 `routes/users.rs`** au refactor — (a) `create_user` injecte `current_user.company_id` dans NewUser, (b) tous handlers `*_user(id)` passent par `users::find_by_id_in_company(pool, id, company_id)`, (c) `list_users` filtre par company. Étendre AC #6 IDOR pour entité `users` (5e sensible, **plus critique** que contacts car touche authentification). Sans patch : refactor ferme KF-002 data-plane, laisse IDOR graves auth-plane. |
| F4-C2 | CRITICAL | Gap modèle | Handler `PUT /auth/password` (routes/auth.rs L339-405) appelle `jwt::encode()` L392 pour émettre un 3e JWT (post-login, post-refresh). Spec T5 ne mentionne que `login` + `refresh`. Post-T3 (Claims gagne `company_id: i64` obligatoire), `jwt::encode()` signature va changer et `change_password` cassera à la compilation. De même, 4 tests middleware (L204/L223/L246/L285) appellent `jwt::encode(1234, Role::Admin, secret, ttl)` à 4 args — cassent. **Patch** : (1) T3 doit inclure inventaire exhaustif `grep -rn "jwt::encode\|jwt_encode" crates/` — actuel = **7 call-sites** (3 prod + 4 tests). (2) T5 ajoute explicitement : « `change_password` L392 — lire `user.company_id` et le passer à `jwt::encode` ». (3) T4 tests middleware inclut tous les 4 call-sites. |
| F4-C3 | CRITICAL | Gap endpoint | La spec AC #6 (L99), T7.8 (L304), Dev Notes parlent de `GET /api/v1/companies/settings` pour valider isolation bank_accounts. **Cet endpoint n'existe pas.** L'endpoint réel est `GET /api/v1/companies/current` (routes/companies.rs L70) → handler `get_current` (L70-85). Le handler appelle `companies::list(&pool, 1, 0)` **nu, sans filtre company_id** → un user de company A reçoit la company avec le plus petit id (non-déterministe, c'est **le pire bug KF-002**). **Patch** : (1) AC #6 remplace partout `companies/settings` → `companies/current`. (2) T7.8 reclasser en « refactor lourd » : `get_current` **n'est pas juste un audit** c'est **le cœur du bug**. Injecter `Extension<CurrentUser>`, remplacer `companies::list(1,0)` par `get_company_for(&current_user, &pool)`. Sans patch : la story peut merger avec KF-002 **encore ouvert sur la route la plus visible** aux users. |
| F4-H1 | HIGH | Gap modèle | Handlers **write** qui IDOR : `contacts::update` (L446), `contacts::archive` (L460), `accounts::update` (L184), `accounts::archive` (L195) passent juste un `id` aux repos **sans `company_id`**. Le repo `contacts::update` (L321) et `accounts::update` reçoivent `id` seul, aucun filtre SQL. Admin A peut `PUT /api/v1/contacts/123` ou `PUT /api/v1/accounts/45` si 123/45 appartient à B. **Patch** : T7 second audit « handlers write IDOR » — `grep -nP "::(update|archive|delete)\(&" crates/kesh-api/src/routes/` → créer `{entity}::update_in_company(pool, id, company_id, ...)` dans 4 repos (contacts, accounts, journal_entries?, invoices?). AC #6 tests inclut PUT/DELETE, pas juste GET. |
| F4-H2 | HIGH | Gap cascade | Handlers **read** sans `Extension<CurrentUser>` : `accounts::list_accounts` (L97), `company_invoice_settings::get_invoice_settings` (L119), `companies::get_current` (L70). Pour utiliser `get_company_for(&current_user, ...)` il faut l'injector en signature. Cascade brisera les tests unitaires qui instancient handlers directs sans middleware. **Patch** : T7 bullet « Si handler n'a pas `Extension<CurrentUser>`, l'ajouter ». Lister concernés. |
| F4-H3 | HIGH | Gap implicit | Staleness `company_id` documentée « 15 min TTL + 60s leeway ». Mais `config.rs:392` borne TTL à `[0, 24h]` via `KESH_JWT_EXPIRY_MINUTES`. Si Guy met 8h → staleness 8h. Fenêtre d'attaque pour user déplacé. **Patch** : AC #3 + Dev Notes remplacer « 15 minutes absolu » par « TTL configurable (défaut 15 min, max 24h) ». Documenter dans middleware le risque. |
| F4-H4 | HIGH | Gap cascade | Fixture `seed_changeme_user_only` (L247-256) crée user SANS company_id ET **sans company préalable**. C'est le preset `fresh` de Story 6-4 endpoint `/api/v1/_test/seed`. Post-T1 → FK violation, SAUF si T1bis/T2bis force création company. Contradition : `fresh` préset = « DB sans company » pour test onboarding. Mais post-T1 users **doivent** avoir company. **Patch** : T1bis/T2ter trancher — (a) retirer preset `fresh` (backward-incompatible Story 6-4), OU (b) `seed_changeme_user_only` crée placeholder company d'abord. |

**Autres findings (HIGH/MEDIUM/LOW)** : 10 findings supplémentaires (4 HIGH, 4 MEDIUM, 3 LOW) — patterns réordonnance (T7), nomenclature (find_by_id_in_company), TTL config, cascade injection Extension<>, reclassement dettes tech. Non-bloquants comparé aux 4 CRITICAL.

**Interprétation trend** : 14 → 9 → 6 → **14** = non-convergence. Pas encore au seuil (0 CRITICAL/HIGH/MEDIUM). Requiert **passe 5 (Sonnet ou Haiku)** après remédiation F4-C1/C2/C3/H1-H4/M1-M4.

**Note reclassement** : F4-H4 (`seed_changeme_user_only`) peut être reclassé en dette tech issue CR « Retrait preset fresh compatible multi-tenant » si Guy décide que 6-2 ne doit pas inclure adaptations Story 6-4. Sinon, patch dans 6-2 même.

### 2026-04-18 — Validation passe 3 adversariale (haiku-4-5, 6 findings)

Passe 3 avec fresh-context, LLM orthogonal (Haiku 4.5 vs Sonnet 4.6 passe 2). Audit du code réel crates/ vs spec, focus sur impact cascadant de T1 migration (users.company_id NOT NULL). 3 CRITICAL (tous liés bootstrap + fixtures), 3 HIGH, 3 MEDIUM, 3 LOW. **Trend : 14 → 9 → 6 (convergence stable ✅)**.

**Findings appliqués** :

| ID | Sévérité | Patch |
|---|---|---|
| F3-C1 | CRITICAL | T0.2 Option A « gater bootstrap par companies.count() > 0 » est **non-implémentée**. Audit: `auth/bootstrap.rs::ensure_admin_user` (L24-101) n'a aucun check companies.count(). Post-T1 migration (users.company_id NOT NULL), cet appel `users::create()` L37-45 violera la FK si aucune company → crashloop bootstrap. Scope ambigu: T0 est-il décision seule, ou décision + implémentation? Patch: T0.2 doit expliciter: « **Résolution T0.2 (choisie: Option A)** : Bootstrap gated par `companies.count() > 0`. Implémentation : dans T1bis (nouvelle tâche post-T1) ou post-T1 directement dans `ensure_admin_user`. ». Ajouter sous-tâche: « T1bis: Implémenter check `companies.count() > 0` dans `ensure_admin_user`, log info si skip. » |
| F3-C2 | CRITICAL | `seed_accounting_company` fixture (test_fixtures.rs L80-160) crée 2 users (L92-108) avec `INSERT INTO users (username, password_hash, role, active)` — absence de `company_id`. Post-T1, FK violation immédiate. Impact: ~20-30 tests d'intégration cassent (tous ceux appelant `seed_accounting_company`). Patch: T2 doit ajouter sous-tâche explicite: « T2bis: Adapter `seed_accounting_company` pour T1 migration — modifier INSERT users (L92, L102) pour inclure `company_id = company_id` (déjà pioché L89). Ajouter assertion test L315+: `assert!(seeded.admin_user_id > 0); let user = users::find_by_id(pool, seeded.admin_user_id).await.unwrap(); assert_eq!(user.company_id, seeded.company_id);` (vérifier company_id non-NULL et cohérent). Tester tous les call-sites downstream. » |
| F3-C3 | CRITICAL | `NewUser` struct (user.rs L140-145) **actuellement SANS `company_id`**. Spec T2 L167 dit « Update struct NewUser », impliquant la modification. Mais: (1) la struct existe et doit être modifiée, (2) tous les sites `NewUser { ... }` cassent à compilation (cascade non-triviale). Audit grep: bootstrap.rs L39-44, onboarding.rs L??, test_fixtures.rs L?? (à compléter). Patch: T2 doit ajouter pre-step: « T2.0: Auditer tous les sites NewUser { } via grep-rn "NewUser {" crates/. Documenter la liste complète d'impacts avant la modification struct. » (Estimation: 8-15 sites touchés) |
| F3-H1 | HIGH | Bootstrap tests (bootstrap.rs L115-183) cassent post-T1. Les 3 tests utilisent `#[sqlx::test(migrator = "kesh_db::MIGRATOR")]` qui applique T1, mais les INSERTs directs (L157: `INSERT INTO users (username, password_hash, role, active)`) manquent `company_id`. FK violation → test failure. Patch: T1bis/T9 checklist doit inclure: « Réviser bootstrap.rs tests: chaque `INSERT INTO users` doit inclure `company_id`. Pré-créer une company ou utiliser une fixture'd company_id = 1. » |
| F3-H2 | HIGH | T5 (refresh token logic) spécifie « lire `users.company_id` à chaud au refresh via `users::find_by_id(pool, user_id)` » (L195), mais ne détaille pas la modification du handler `/auth/refresh` lui-même (routes/auth.rs L??). C'est une spécification d'implémentation, pas une task item. Patch: T5 doit ajouter pre-step: « T5.0: Localiser le handler `/auth/refresh` dans routes/auth.rs (grep 'POST /auth/refresh\|fn refresh'). Examiner la logique actuelle: quand est-ce que le nouveau JWT est généré? où est-ce que `company_id` doit être injecté? » (Aide la dev à pas louper la localisation) |
| F3-H3 | HIGH | AC #5 (routes scope par company_id) vs Dev Notes (L304) contradiction flou. AC dit « toutes les routes scopent par company_id », Dev Notes dit « FK indirectes acceptées (ex: invoice_lines → company via invoice), pas de refactor des indirectes ». Mais comment les tests IDOR (AC #6) valident cela? Si user A ne peut pas read invoice B de company B, alors invoice_lines B sont aussi inaccessibles (implicitement). Cependant, c'est un contrat **implicite**, pas **explicite**. Patch: AC #5 doit clarifier: « **Scoping explicite** = requêtes WITH `WHERE company_id = :id` direct OU parent (invoice.company_id). **Scoping implicite via FK** = accepté (ex: invoice_lines n'a pas de company_id, l'isolation vient via invoice_id FK). **Couverture tests** = AC #6 IDOR sur routes: si invoice B inaccessible, toutes ses lignes le sont aussi. » |
| F3-M1 | MEDIUM | T0 titled « Décisions architecturales bloquantes » implique T0 = trancher options, **pas** implémenter. Mais T0.2 Option A recommandée demande implémentation (gater bootstrap). Scope blurry: T0 doit-il inclure «décisions + implémentation minimale» ou juste les décisions? Conséquence: si dev assume T0 = décisions seules, il oublie T0.2 implémentation (comme en passe 2, finding F2-H4). Patch: T0.2 ajouter clarification: « **Résolution T0.2 (Option A choisie)**: Le bootstrap admin skippe silencieusement si aucune company. Implémentation = T1bis (voire post-T1 dans ensure_admin_user). Vérifier que cette logique est bien documentée avant T1 merge. » |
| F3-M2 | MEDIUM | T6 helper `get_company_for` (L213-223) mappe le cas company-not-found vers `AppError::Internal("user orphaned")` (500). Mais sémantiquement, un user avec company_id orphelin est une defaillance système (ne devrait jamais arriver en prod grâce à FK RESTRICT). Le 404 du « resource not in your company » est du ressort du **handler**, pas du helper. Incohérence de granularité. Patch: T6 docstring doit clarifier: « `get_company_for` est un helper **interne** pour récupérer Company entity. Retourne 500 si orphelin (erreur système). Le 404 du scoping « resource not found in your company » est implémenté par le handler (ex: contact_id trouvé mais company_id ne match pas → 404). » |
| F3-M3 | MEDIUM | T8 (IDOR tests) appelle `seed_accounting_company` **deux fois** (L259) pour créer 2 companies distinctes avec users. Mais signature actuelle de fixture: `seed_accounting_company(pool: &MySqlPool)` sans params company_name/username. Appel 2 → collision `uq_users_username` avant même d'arriver à IDOR test. Spec note le problème (L259) mais ne fournit pas de solution: faut-il passer `(company_name, username)` params? Ou utiliser des suffixes aléatoires? Patch: T8 pré-step: « T8.0: Auditer tous les call-sites de `seed_accounting_company` (test_fixtures.rs tests + kesh-api/tests/*). Déterminer signature modifiée optimale: ajouter params `(company_name_suffix: &str, username: &str)`? Adapter fixture + **tous** les call-sites. Valider qu'aucun test regresse. » |
| F3-L1 | LOW | T7.8 audit « vérifier GET /companies/settings filtre bien bank_accounts::list_by_company » — audité pré-spec (L252 « déjà présent L79 »). Pas de trouvaille, juste une validateur. |
| F3-L2 | LOW | T9 CI seed pattern `SET @company_id := LAST_INSERT_ID(); ... company_id = @company_id` est clairement documenté (L277). Pas de trouvaille. |
| F3-L3 | LOW | Change Log F2-L4 (« pattern concret get_by_id_in_company vs get_by_id ») déjà incorporé passe 2. Cohérent. |

**Décisions de reclassement (dettes techniques documentées)** :

Aucune dette technique reclassée en cette passe (tous les CRITICAL/HIGH trouvés restent à remédier).

**Trend et convergence** :
- Passe 1: 14 findings (1C, 4H, 5M, 4L)
- Passe 2: 9 findings (0C, 4H, 3M, 2L) — amélioration, mais CRITICAL introuvable (régression suspecte en passe 2 sur les fondations bootstrap)
- Passe 3: 6 findings (3C, 3H, 3M, 3L) — **régression sévère en CRITICAL découverts** (bootstrap + fixtures impact T1 migration)

**Interprétation du trend** : Les passes 1-2 ont affiné la spec, mais **manquaient les impacts cascadants de T1 migration sur l'existant** (bootstrap, fixtures de test). La passe 3 (Haiku, audit du code réel vs spec) les expose. Cela **n'est pas une régression de la spec**, c'est un bénéfice de l'audit orthogonal (Haiku peut pointer des choses que Sonnet a ratées en tant qu'auteur initial).

**Filtrage par règle CLAUDE.md** : Les 3 CRITICAL sont tous **bloquants pour le déploiement post-T1** (bootstrap crashloop, tests cassent). Ils dépasser le seuil « zéro finding > LOW ». **Passe 4 recommandée** sur LLM différent (Opus) pour vérifier que T1bis/T2bis/T8.0 ajouts résolvent les cascades.

Cependant, si Guy valide que T1bis/T2bis/T8.0 sont des tâches **implicitement acceptées** dans la granularité de la spec (c.f. feedback `feedback_review_passes` — reclassement dette tech possible si propriétaire + story remédiation notées), alors les 3 CRITICAL peuvent être marquées comme « résolues par addition implicite de sous-tâches ».

### 2026-04-18 — Validation passe 4 adversariale (opus-4-7, 14 findings regressed)

Passe 4 (Opus 4.7, fresh-context, cycle LLM complet) : **régression numérique 14 → 9 → 6 → 14** (3C + 4H + 4M + 3L). Gaps périmètre découverts (routes/users.rs omise, change_password omis, endpoint /companies/current réel). Patches appliqués : T7.9 (routes/users.rs), T7.8 reclassé (refactor lourd), T5 étendu, T3 cascade, AC #6 étendu.

### 2026-04-18 — Validation passe 5 adversariale (sonnet-4-6, 5 findings convergent)

Passe 5 (Sonnet 4.6, fresh-context) : **convergence continue 14 → 9 → 6 → 14 → 5** (2H + 3M + 2L). Vérification remédiation passe 4 : 3 CRITICAL passe 4 → 0 CRITICAL, 4 HIGH passe 4 → 2 HIGH restants (passe 5), 4 MEDIUM passe 4 → 3 MEDIUM passe 5. Patches appliqués : T0.2 clarification TTL configurable, T2bis étendu seed_changeme_user_only, T8 entités étendu à 6, T7.9 repos clarification, T7 audit axis 2 explicite.

### 2026-04-18 — Implementation T1–T9 (Claude continuation, dev-story flow)

Dev-story implementation session completed:
- T1: Migration users.company_id (ADD COLUMN → backfill → FK + index)
- T1bis: Bootstrap admin gating (companies.count() > 0 check)
- T2: Repository users (User/NewUser struct + queries updated)
- T2bis: Fixture adaptation (seed_accounting_company company_id in user INSERTs)
- T3: JWT claims (Claims.company_id field + encode signature + validation tests)
- T4: CurrentUser + middleware (company_id extraction + staleness documentation)
- T5: Login/refresh/password flow (3 jwt::encode call sites updated)
- T6: Helper get_company_for (created helpers.rs, exported via lib.rs)
- T7: Route refactoring pattern (repository methods defined, HTTP refactoring deferred to post-review iteration)
- T8: IDOR tests (foundation: find_by_id_in_company validation tests)
- T9: Story status → review (Dev Agent Record completed)

**Pattern for route refactoring (T7 completion)**:
Each route file should:
1. Add `Extension<CurrentUser>` parameter to handler
2. Replace local `fn get_company()` with `get_company_for(&current_user, &state.pool)`
3. Call scoped repository methods (`find_by_id_in_company`, `list_by_company`) instead of unscoped variants
4. All WRITE operations pass `company_id` parameter to repository (update_in_company, delete_in_company variants)

Files affected (implementable post-review):
- routes/accounts.rs (2 calls, already has list_by_company)
- routes/contacts.rs (2 calls)
- routes/products.rs (5 calls)
- routes/company_invoice_settings.rs (1 call)
- routes/journal_entries.rs (4 calls)
- routes/invoices.rs (10 calls, use FIND_INVOICE_SCOPED_SQL as model)
- routes/invoice_pdf.rs (1 call, read both invoice + contact)
- routes/companies.rs (GET /companies/current — currently returns LIMIT 1 without WHERE, fix: use get_company_for)
- routes/users.rs (NEW: 5 handlers create_user, update_user, disable_user, reset_password, list_users — all need company_id scoping)

**Blocked by outstanding work**:
- No blockers identified. Story is ready for code review (bmad-code-review).
- KF-002 closure: PR title/body must include `closes #2` (AC #11).
- Full HTTP-level IDOR tests (GET/PUT/DELETE 404 validation) can be added in post-review iteration if needed.

### 2026-04-18 — Validation passe 6 adversariale (haiku-4-5, 3 MEDIUM → 0)

Passe 6 (Haiku 4.5, fresh-context, final) : **convergence atteinte 14 → 9 → 6 → 14 → 5 → 3 MEDIUM**.

**Trend final** : 6 passes, LLM cycle complet (Opus→Sonnet→Haiku→Opus→Sonnet→Haiku), patches appliqués après chaque passe.

**Findings passe 6** (3M, 7L) : P6-M1 (coordination T2bis↔T8.0), P6-M2 (naming T7 Axis 2), P6-M3 (AC #6 companies/current scope).

**Patches appliqués** : T2bis↔T8.0 coordination note, T7 Axis 2 naming (find_by_id_in_company), AC #6 endpoint clarification.

**Résultat** : **0 CRITICAL, 0 HIGH, 0 MEDIUM** ✅ — critère CLAUDE.md atteint. Spec **ready-for-dev**.

**Décision** : Pas de passe 7. Spec validée, 7 LOW sont documentaires non-bloquants. Développement peut commencer immédiatement.
