# Story 6.2 : Refactor multi-tenant scoping (`CurrentUser.company_id`)

Status: ready-for-dev

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

**Usages `get_company()` à refactorer** (37 occurrences totales via `grep -rn "get_company" crates/kesh-api/src/routes/`) :

| Fichier | Helper `fn get_company` déclaré ? | Nombre d'appels |
|---|---|---|
| `routes/products.rs` | ✅ L122 | 5 |
| `routes/invoices.rs` | ✅ L260 | 10 |
| `routes/invoice_pdf.rs` | ✅ L343 | 1 |
| `routes/journal_entries.rs` | ✅ L167 (avec commentaire « duplication volontaire ») | 4 |
| `routes/accounts.rs` | ✅ L85 | 2 |
| `routes/company_invoice_settings.rs` | ✅ L70 | 1 |
| `routes/contacts.rs` | ✅ (à vérifier) | ? |
| `routes/bank_accounts.rs` | ✅ (à vérifier) | ? |
| `routes/onboarding.rs` | différent (flow création) | cas particulier |

**Tests existants sur l'isolation multi-tenant** : **aucun**. Aucun test ne vérifie qu'un user d'une company A ne peut pas lire une ressource d'une company B. À ajouter intégralement dans cette story.

## Critères d'acceptation (Given/When/Then, reprise epics.md#Story-6.2 + additions)

1. **Schéma** — **Given** le schéma actuel sans `users.company_id`, **When** migration appliquée, **Then** la table `users` a une colonne `company_id BIGINT NOT NULL FOREIGN KEY REFERENCES companies(id) ON DELETE RESTRICT` + backfill des users existants pointe vers la company avec le plus petit `id` (ou la seule company si mono-tenant).

2. **JWT claims** — **Given** un login réussi, **When** JWT émis, **Then** les claims incluent `company_id: i64` (lu au moment du login depuis `users.company_id`). Les anciens JWT sans `company_id` sont rejetés par `require_auth` (erreur 401 `invalid company_id claim`) — acceptable car leur TTL est 15 min.

3. **`CurrentUser` struct** — **Given** le middleware `require_auth`, **When** JWT décodé, **Then** `CurrentUser { user_id, role, company_id }` est injecté dans les extensions. La staleness est documentée dans le code (un user déplacé vers une autre company au cours de sa session garde l'ancien `company_id` jusqu'à expiration du JWT — même pattern que `role`).

4. **Helper unifié `get_company_for`** — **Given** les 8 helpers `get_company()` dupliqués, **When** refactor, **Then** ils sont remplacés par un **unique** helper partagé `crate::helpers::get_company_for(current_user: &CurrentUser, pool: &MySqlPool) -> Result<Company, AppError>` qui fait `SELECT * FROM companies WHERE id = ?` avec `current_user.company_id`. Les 8 `fn get_company` locaux sont supprimés (DRY).

5. **Toutes les routes API scopent par company_id** — **Given** les routes `contacts`, `products`, `invoices`, `invoice_pdf`, `bank_accounts`, `company_invoice_settings`, `accounts`, `journal_entries`, **When** un handler lit ou écrit une ressource, **Then** la requête SQL filtre sur `company_id = :current_user.company_id`. Aucune route ne fait `SELECT ... LIMIT 1` sans WHERE ni `SELECT ... WHERE company_id IN (SELECT id FROM companies)`.

6. **Tests IDOR cross-company** — **Given** 2 companies A et B avec chacune leurs propres ressources, **When** un user de company A tente d'accéder via ID direct à une ressource de company B (GET `/api/v1/contacts/{id_de_B}`, PUT `/api/v1/invoices/{id_de_B}`, DELETE `/api/v1/bank_accounts/{id_de_B}`, GET `/api/v1/products/{id_de_B}`), **Then** la réponse est `404 NotFound` (**pas** 200 avec données fuitées, **pas** 403 qui révèle l'existence). Au minimum **1 test par entité sensible** : contacts, products, invoices, bank_accounts. Chaque test utilise le helper `seed_accounting_company` deux fois pour créer les deux companies.

7. **Onboarding flow préservé** — **Given** l'onboarding actuel (Story 2-2/2-3) crée une company puis attache un user admin, **When** le flow est joué sur DB fraîche, **Then** le user créé est correctement lié à la company créée (`users.company_id = companies.id`). Le flow continue de fonctionner sans régression (tests d'intégration `onboarding_e2e.rs` verts).

8. **KF-002 / issue #2 close** — **Given** la story mergée, **When** commit de merge, **Then** le message contient `closes #2`. GitHub ferme automatiquement l'issue. Pas de ligne à modifier dans `docs/known-failures.md` (fichier archivé depuis 2026-04-18, cf. PR #23).

9. **CI verte** — **Given** la branche `story/6-2-multi-tenant-scoping-refactor`, **When** PR ouverte, **Then** les 4 required checks passent (`Backend`, `Frontend`, `E2E`, `Docker build`). Aucune régression sur les 84+ tests `kesh-db` ni les tests `*_e2e.rs`.

## Scope volontairement HORS story — décisions tranchées

- **UI multi-tenant** (dashboard company-switch, gestion multi-companies d'un user admin) → orthogonal, pas dans v0.1.
- **Refactor de `get_company()` dans `onboarding.rs`** → reste dans onboarding (flow bootstrap sans user authentifié). Marqué clairement comme cas particulier.
- **Tests IDOR sur `journal_entries` / `fiscal_years` / `company_invoice_settings`** → hors scope minimum (4 entités sensibles suffisent pour fermer l'AC #6). Ajout opportuniste accepté mais pas bloquant.
- **Audit de tous les handlers pour scoping implicite via FK** → certaines tables (ex: `invoice_lines`) n'ont pas de `company_id` direct, l'isolation passe par `invoice_id`. Pas de refactor des FK indirectes — accepté tant que les tests IDOR passent.
- **Migration vers un design « user belongs to exactly one company »** vs « user belongs to N companies via user_companies pivot » → on fige **1:N** (un user → une company) dans cette story. Pivot pour plus tard si besoin.
- **Fermeture de toutes les autres KF** via cette story → seule KF-002 est close. Les autres restent ouvertes.

## Tasks / Subtasks

### T0 — Décision architecturale : `company_id` via JWT claim vs DB lookup (AC #2, #3)

- [ ] **Discussion + décision** (avant tout code) :
  - **Option A** : `company_id` embarqué dans le JWT claim au login. `require_auth` le lit directement. Avantages : zero DB call par requête protégée. Inconvénients : breaking change protocolaire + migration users qui changent de company invisible jusqu'à expiration JWT.
  - **Option B** : `company_id` pas dans le JWT, `require_auth` fait un `SELECT company_id FROM users WHERE id = ?` à chaque requête. Avantages : staleness nulle. Inconvénients : +1 DB call par requête.
- [ ] **Recommandation par défaut** : **Option A** (cohérent avec le pattern `role` déjà documenté dans `middleware/auth.rs:40-50`). Staleness acceptée et tracée.
- [ ] **Sortie** : note de décision dans Dev Notes + impact sur T3/T4.

### T1 — Migration schéma `users.company_id` (AC #1, #7)

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

### T2 — Repository `users` : propager `company_id` (AC #1, #7)

- [ ] `crates/kesh-db/src/entities/user.rs` — ajouter `pub company_id: i64` à la struct `User`.
- [ ] `crates/kesh-db/src/repositories/users.rs` — update `create`, `get_by_id`, `get_by_username`, `list` pour sélectionner/insérer `company_id`.
- [ ] Update signature `UserCreate` (ou équivalent) pour accepter `company_id`.
- [ ] Tests unitaires de repository : vérifier que `users.company_id` est bien persisté et relu.

### T3 — JWT claims : ajouter `company_id` (AC #2)

- [ ] `crates/kesh-api/src/auth/jwt.rs` — ajouter `pub company_id: i64` à la struct `Claims`.
- [ ] Update `encode(...)` pour prendre un `company_id: i64` en paramètre.
- [ ] Update `decode(...)` : si `company_id` manquant dans un vieux JWT → retourner `JwtError::InvalidClaims("missing company_id")` → mappe vers 401.
- [ ] Update tests unitaires jwt : encode/decode avec `company_id` + test decode d'un JWT legacy (sans company_id) → 401 attendu.

### T4 — `CurrentUser` + middleware `require_auth` (AC #3)

- [ ] `crates/kesh-api/src/middleware/auth.rs` :
  - Ajouter `pub company_id: i64` à `CurrentUser`.
  - Dans `require_auth`, parser `company_id` des claims (si absent → 401).
  - Documenter la staleness `company_id` dans le bloc `SEC:` existant (alignement avec `role`).
- [ ] Tests unitaires middleware : JWT valide avec `company_id` → 200 + CurrentUser bien injecté ; JWT sans `company_id` → 401.

### T5 — Login flow : injecter `company_id` dans le JWT (AC #2)

- [ ] `crates/kesh-api/src/routes/auth.rs` (ou équivalent) — `login` handler : après authentification réussie, lire `user.company_id` et le passer à `jwt::encode`.
- [ ] Idem pour le refresh token endpoint (`/auth/refresh`) : le refresh token doit également porter `company_id` pour que le nouveau JWT l'ait aussi.
- [ ] Tests d'intégration `auth_e2e.rs` (ou similaire) : login → GET `/api/v1/users/me` → vérifier que la réponse inclut `company_id` (si exposé) ou que les queries suivantes scopent bien.

### T6 — Helper partagé `get_company_for` (AC #4)

- [ ] Créer `crates/kesh-api/src/helpers.rs` (ou `crates/kesh-api/src/routes/helpers.rs`) :
  ```rust
  pub async fn get_company_for(
      current_user: &CurrentUser,
      pool: &MySqlPool,
  ) -> Result<Company, AppError> {
      companies::get_by_id(pool, current_user.company_id)
          .await?
          .ok_or_else(|| AppError::Internal(format!(
              "company_id {} from JWT not found in DB",
              current_user.company_id
          )))
  }
  ```
- [ ] Ajouter `companies::get_by_id(pool, id)` dans `kesh-db::repositories::companies` si pas déjà existant.
- [ ] Tests unitaires du helper.

### T7 — Refactor des 8 fichiers de routes (AC #4, #5)

Pour **chaque fichier** dans (`contacts.rs`, `products.rs`, `invoices.rs`, `invoice_pdf.rs`, `bank_accounts.rs`, `company_invoice_settings.rs`, `accounts.rs`, `journal_entries.rs`) :

- [ ] Supprimer le `async fn get_company(state: &AppState)` local.
- [ ] Pour chaque handler qui appelait `get_company(&state).await?` :
  - Ajouter `Extension(current_user): Extension<CurrentUser>` aux paramètres du handler (pattern déjà présent ailleurs dans le même fichier).
  - Remplacer `get_company(&state).await?` par `get_company_for(&current_user, &state.pool).await?`.
- [ ] **Audit SQL** : pour chaque requête qui touche une entité de cette route, vérifier qu'elle filtre bien par `company_id`. Beaucoup de requêtes actuelles sont déjà OK (ex: `contacts::list_by_company(pool, company.id, ...)`), mais certaines pourraient encore passer par une autre voie. Corriger toute route qui fait un `list(pool)` nu ou un `get_by_id(pool, id)` sans vérifier l'appartenance à la company.

**Priorité d'ordre** (dépendances en cascade) :
1. `routes/accounts.rs` (plan comptable — plus simple)
2. `routes/contacts.rs`, `routes/products.rs` (CRUD simples)
3. `routes/bank_accounts.rs`, `routes/company_invoice_settings.rs` (CRUD config)
4. `routes/journal_entries.rs` (plus complexe, a déjà un commentaire « duplication volontaire »)
5. `routes/invoices.rs`, `routes/invoice_pdf.rs` (les plus intriqués, 11 appels au total)

### T8 — Tests IDOR cross-company (AC #6)

- [ ] Créer `crates/kesh-api/tests/idor_multi_tenant_e2e.rs`.
- [ ] Utiliser `seed_accounting_company` **deux fois** pour créer 2 companies distinctes (A et B) avec chacune leur admin user.
- [ ] Pour chaque entité sensible (minimum 4 : `contacts`, `products`, `invoices`, `bank_accounts`) :
  - Créer une ressource X dans la company B (via appel direct au repository, bypass des routes).
  - Login en tant qu'admin de company A → JWT avec `company_id = A`.
  - GET `/api/v1/{entity}/{id_de_X}` → attendre `404 NotFound` (pas 200, pas 403).
  - PUT / DELETE équivalents → idem 404.
- [ ] Documentation : chaque test commente clairement le scénario d'attaque simulé.

### T9 — Fermeture KF-002 / issue #2 (AC #8)

- [ ] Vérifier que le commit de merge contiendra `closes #2` dans sa description (ou son premier commit si squash).
- [ ] Pas de modif dans `docs/known-failures.md` (archivé 2026-04-18, cf. nouvelle règle CLAUDE.md `Issue Tracking Rule`).
- [ ] Optionnel : commenter sur l'issue #2 avec le lien vers la PR et un résumé du refactor.

### T10 — Validation + documentation (AC #9)

- [ ] `cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings` → vert.
- [ ] `cargo test --workspace -j1 -- --test-threads=1` → vert (inclut les nouveaux tests IDOR).
- [ ] `cd frontend && npm run build && npm run test:e2e` → vert (après Story 6-5 éventuellement si Playwright encore bloqué, sinon `continue-on-error` tient).
- [ ] Mise à jour `crates/kesh-api/README.md` si le flow auth est documenté (ajouter `company_id` dans les claims JWT).
- [ ] Si un changelog projet existe : note de refactor multi-tenant.

## Dev Notes

### Architecture / contraintes

- **Axum 0.8** : pattern `Extension<CurrentUser>` déjà utilisé partout. Aucun changement structurel requis.
- **SQLx 0.8** : les INSERT `users` doivent inclure `company_id` NOT NULL — toute route / fixture / seed qui crée un user doit fournir une valeur.
- **JWT staleness** : la doc du middleware `auth.rs:40-50` explicite déjà la staleness pour `role`. On aligne le traitement pour `company_id`. Les 15 minutes de TTL + 60s de leeway restent inchangées.
- **Foreign key `ON DELETE RESTRICT`** : cohérent avec toutes les autres FK vers `companies(id)`. Empêche la suppression d'une company qui a encore des users.

### Contraintes multi-tenant à garder à l'esprit

- **Onboarding** : le flow Story 2-2/2-3 crée `companies` avant `users` (ordre d'INSERT). Pas de changement requis — juste propager le `companies.id` fraîchement créé vers le `users.company_id` INSERT suivant.
- **Bootstrap admin** (`crates/kesh-api/src/main.rs`) : au démarrage, si `users` est vide ET `KESH_ADMIN_USERNAME`/`KESH_ADMIN_PASSWORD` sont set, crée un admin. Problème : il n'y a peut-être pas de company non plus → on doit créer une company par défaut aussi, OU attendre qu'un user arrive par onboarding. **À trancher en T5**.
- **CI seed** : `ci.yml` job `backend` seed actuellement 1 company puis 1 user. L'ordre est bon, on doit juste ajouter `company_id = LAST_INSERT_ID()` dans l'INSERT users.

### Patterns à réutiliser (git intelligence)

- Story 6-4 a introduit `kesh-db::test_fixtures::seed_accounting_company` qui crée company + admin user + fiscal_year + accounts. **Après T1+T2**, ce helper devra obligatoirement lier le user à la company (FK NOT NULL). Prévoir de patcher ce helper.
- `bmad-code-review` 4 groupes × 11 passes sur Story 5-4 a produit le plus haut volume de findings `get_company` — réutiliser les patterns de remédiation identifiés (ex: `get_invoice_by_id_in_company` vs `get_invoice_by_id`).

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

(à remplir lors de `dev-story`)

### Completion Notes List

(à remplir lors de `dev-story`)

### File List

(à remplir lors de `dev-story`)

## Change Log

### 2026-04-18 — Création spec (opus-4-7)

- Spec rédigée directement en mode comprehensive (skip du template BMAD light, alignement sur la densité de Story 6-4).
- **Découverte bloquante** lors de l'audit : la table `users` n'a **pas** de colonne `company_id`. Scope élargi par rapport à l'AC d'origine `epics.md#Story-6.2` pour inclure migration schéma + update repositories + JWT claims.
- 10 tâches (T0-T10) avec ordre explicite (T1→T2→T3 requis par cascade FK NOT NULL).
- 9 AC (5 originaux epics.md + 4 additions : schéma, JWT, CurrentUser, onboarding préservé, CI verte).
- 1 test IDOR minimum par entité sensible × 4 entités (contacts, products, invoices, bank_accounts).
- Validation `validate-create-story` recommandée (complexité + impact transverse justifient les 5 passes adversariales).
