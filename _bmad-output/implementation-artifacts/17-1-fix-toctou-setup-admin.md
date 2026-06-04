# Story 17.1: Fix race condition TOCTOU sur la création du 1er admin (`POST /setup/admin`)

Status: ready-for-dev

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

## Story

As a **opérateur d'une instance Kesh exposée au réseau pendant la fenêtre d'onboarding**,
I want **que la création du tout premier administrateur soit atomique (impossible que deux requêtes concurrentes créent deux admins)**,
so that **une race condition ne puisse pas compromettre le bootstrap de sécurité de l'installation (deux comptes admin non concertés avec des identifiants distincts)**.

## Contexte & provenance

- **Issue GitHub** : [#133](https://github.com/guycorbaz/kesh/issues/133) `[v0.2] TOCTOU race on POST /setup/admin allows 2 concurrent admins with distinct usernames` — labellisée `v0.2-milestone`.
- **Origine** : dette technique L1 (catégorie B, limitation v0.1 documentée) de la Story v011-5 (onboarding self-service). Spec validate Pass 1 BH1-5 / ECH1-4.
- **Epic** : 17 « Infra & Souveraineté » — **story-zéro sécurité** (la plus petite, sécurise la base avant les grosses stories 17-2/17-3/17-4). Cf. `_bmad-output/planning-artifacts/epic-17.md` décision **D10**.
- **Scope** : strictement le fix d'atomicité + conversion du test race documentaire en assertion stricte. Aucune nouvelle feature, aucune migration, aucun changement de contrat HTTP.

## Le bug (TOCTOU)

Le handler `create_admin` (`crates/kesh-api/src/routes/setup.rs:63-221`) exécute la séquence suivante **sans atomicité** :

1. `SELECT COUNT(*) FROM users` sur `&state.pool` (ligne 102) → si `user_count > 0` → `410 SETUP_ALREADY_COMPLETE`.
2. `users::create(...)` → `INSERT INTO users` (ligne ~145).

Entre l'étape 1 et l'étape 2, **rien ne sérialise deux requêtes concurrentes**. Deux requêtes simultanées avec des **usernames distincts** (`alice`, `bob`) peuvent toutes deux lire `user_count == 0`, puis toutes deux réussir leur INSERT → **2 admins créés**.

> Note : la contrainte `UNIQUE users.username` ne protège QUE le cas où les deux requêtes utilisent le **même** username (le 2e INSERT échoue en `UniqueConstraintViolation` → 410). Elle ne protège PAS le cas usernames distincts, qui est précisément le bug.

Atténuations v0.1 existantes (à **conserver**, défense en profondeur) : rate-limit IP (5/15 min), gate auto-disable `410` une fois `user_count > 0`, recommandation manuel admin de binder en loopback/LAN privé avant 1er boot.

## Acceptance Criteria

1. **AC1 — Atomicité du check+insert** : la séquence « vérifier `user_count == 0` » puis « INSERT du 1er admin » s'exécute à l'intérieur d'**une seule transaction** ouverte sur `state.pool`, précédée de l'acquisition d'un **verrou exclusif sérialisant** (`SELECT ... FOR UPDATE`) sur une row sentinelle globale. Deux appels concurrents à `POST /api/v1/setup/admin` ne peuvent JAMAIS aboutir à plus d'un utilisateur en base, quels que soient leurs usernames.

2. **AC2 — Sentinelle = `_kesh_version` (row globale `id = 1`)** : le verrou est pris via `SELECT id FROM _kesh_version WHERE id = 1 FOR UPDATE` en **première instruction** de la transaction, AVANT le `SELECT COUNT(*)` et avant tout autre SELECT non-locking. Choix justifié : `_kesh_version` est un singleton **install-wide** (garanti par migration `20260522000001_kesh_version.sql` + CHECK `id = 1`), sémantiquement aligné avec un gate « premier admin de l'installation » (global, pas per-tenant). Le verrou est relâché automatiquement au `commit`/`rollback` de la transaction (pas de `GET_LOCK`/`RELEASE_LOCK` manuel à gérer). **Si la row `_kesh_version id=1` est absente** (`fetch_optional → None`, ex. migration 10-2 non appliquée), le helper retourne `DbError::Invariant` → `AppError::Internal` → **HTTP 500** (bug structurel d'installation) — il NE retourne PAS `Ok(())` silencieusement (sinon aucun verrou acquis → race rouverte).

3. **AC3 — Re-check sous verrou** : le `SELECT COUNT(*) FROM users` est ré-exécuté **à l'intérieur de la transaction verrouillée** (pas avant). Si `user_count > 0` sous verrou → `rollback` + `410 SETUP_ALREADY_COMPLETE` + `state.users_exist.store(true, Release)` + `record_failed_attempt(ip)` (comportement identique à l'actuel, mais désormais race-safe).

4. **AC4 — INSERT dans la même transaction** : l'INSERT du 1er admin se fait via un variant transaction-aware (`users::create_in_tx(&mut tx, ...)`), PAS via `users::create(pool, ...)` qui ouvre sa propre transaction interne et casserait l'atomicité. Le `tx.commit()` valide check+insert ensemble.

5. **AC5 — Comportement HTTP inchangé en nominal** : un appel unique sur DB vide retourne toujours `200 OK` + `LoginResponse` + cookies HttpOnly (`build_auth_cookies`). Un 2e appel retourne toujours `410`. Validation username/password, rate-limit, JWT, refresh token, reset rate-limit : comportements et codes inchangés. Aucune régression sur les tests `setup_admin_e2e.rs` existants (AC #9/#10/#11/#13/#14/#22).

6. **AC6 — Test race converti en assertion stricte** : le test `toctou_race_two_distinct_usernames_documents_l1` (`crates/kesh-api/tests/setup_admin_e2e.rs:348-418`) perd son attribut `#[ignore]` et asserte désormais le comportement corrigé : exactement **1** utilisateur en base après deux requêtes concurrentes `alice`/`bob`, et l'ensemble des deux status HTTP est `{200, 410}` (un succès, un auto-disable), dans un ordre quelconque. Le test est renommé pour refléter qu'il garantit désormais l'invariant (ex. `toctou_race_two_distinct_usernames_creates_exactly_one_admin`).

7. **AC7 — Aucune migration, aucun bump version** : la story ne touche AUCUN fichier `crates/kesh-db/migrations/*.sql`, donc PAS d'entrée `docs/migrations-idempotence-audit.md` ni de bump `kesh_version_min_required` (cf. CLAUDE.md §Migration breaking policy — non applicable ici).

8. **AC8 — Quality gate vert** : `cargo fmt --all -- --check`, `cargo build --workspace --all-targets`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (mode serial `-j1 --test-threads=1` car la modif touche `kesh-db` + tests d'intégration DB). Frontend non touché (pas de `npm` requis).

## Tasks / Subtasks

- [ ] **T1 — Ajouter `users::create_in_tx` transaction-aware** (AC: #4)
  - [ ] Dans `crates/kesh-db/src/repositories/users.rs`, extraire le corps INSERT de `create` (lignes ~23-end) dans un nouveau `pub async fn create_in_tx(tx: &mut sqlx::Transaction<'_, sqlx::MySql>, new: NewUser) -> Result<User, DbError>` qui exécute l'INSERT sur `&mut **tx` SANS ouvrir/committer de transaction (l'appelant gère le cycle de vie).
  - [ ] Refactorer `create(pool, new)` pour DÉLÉGUER : `let mut tx = pool.begin()?; let user = create_in_tx(&mut tx, new).await?; tx.commit()?; Ok(user)` — préserve l'API publique existante (DRY, cf. CLAUDE.md). Tous les call sites actuels de `users::create` restent inchangés.
  - [ ] Vérifier que la gestion `last_insert_id() == 0 → DbError::Invariant` et le `map_db_error` (incl. `UniqueConstraintViolation` code 1062) sont préservés dans `create_in_tx`.

- [ ] **T2 — Helper de verrou sentinelle setup** (AC: #1, #2)
  - [ ] **Emplacement = `crates/kesh-db/src/repositories/users.rs`** (colocalisation avec `create_in_tx`, module déjà déclaré dans `mod.rs` → PAS de nouveau fichier ni de `pub mod` à ajouter). Ne PAS créer un nouveau module `system.rs`/`kesh_version.rs` (sinon il faut ajouter `pub mod …;` dans `crates/kesh-db/src/repositories/mod.rs`, oubli fréquent → erreur de compilation). `onboarding.rs` est une alternative acceptable (existe déjà) mais `users.rs` est préféré.
  - [ ] Signature : `pub async fn acquire_setup_sentinel_lock(tx: &mut Transaction<'_, MySql>) -> Result<(), DbError>`. **`pub` OBLIGATOIRE** (appelé depuis la crate `kesh-api` — `pub(crate)` casserait la compilation).
  - [ ] Le helper exécute `SELECT id FROM _kesh_version WHERE id = 1 FOR UPDATE` via `fetch_optional(&mut **tx)`.
  - [ ] **⚠️ DIVERGENCE CRITIQUE vs `acquire_company_sentinel_lock`** : ce dernier retourne `Ok(())` même si `fetch_optional` rend `None` (acceptable pour `companies` car le `company_id` est pré-validé en amont par l'appelant). Pour la sentinelle GLOBALE `_kesh_version id=1`, il n'y a AUCUNE pré-validation : si la row est absente, le `FOR UPDATE` ne verrouille rien et la race TOCTOU **persiste silencieusement**. Donc le helper DOIT traiter `None` comme une erreur :
    ```rust
    let row: Option<(i64,)> = sqlx::query_as("SELECT id FROM _kesh_version WHERE id = 1 FOR UPDATE")
        .fetch_optional(&mut **tx).await.map_err(map_db_error)?;
    if row.is_none() {
        return Err(DbError::Invariant(
            "row _kesh_version id=1 absente — migration 20260522000001 non appliquée ou DELETE manuel".into()
        ));
    }
    Ok(())
    ```
    Côté handler, `DbError::Invariant` → `AppError::Internal` → **500** (bug structurel d'installation, ni 410 ni 400).
  - [ ] Documenter (`///`) : (a) verrou install-wide relâché au commit/rollback ; (b) **DOIT être la PREMIÈRE instruction de la tx, avant tout SELECT non-locking** (sous REPEATABLE READ InnoDB, le snapshot MVCC est figé au 1er *consistent read* non-locking ; placer le `FOR UPDATE` en tête garantit que le `SELECT COUNT(*)` suivant — 1er non-locking read — fige son snapshot APRÈS le commit de la tx concurrente, donc voit les données committées) ; (c) `None` = bug structurel → `Invariant`.

- [ ] **T3 — Réécrire la section critique de `create_admin`** (AC: #1, #3, #4, #5)
  - [ ] Dans `crates/kesh-api/src/routes/setup.rs`, remplacer la séquence non-transactionnelle (lignes ~99-167) par : `let mut tx = state.pool.begin().await?;` → **acquérir le verrou sentinelle EN PREMIÈRE INSTRUCTION de la tx** (T2) → `SELECT COUNT(*) FROM users` sur `&mut *tx` → si `> 0` : `tx.rollback()` + `users_exist=true` + `record_failed_attempt` + `Err(SetupAlreadyComplete)` → sinon `users::create_in_tx(&mut tx, ...)` → `tx.commit()`.
  - [ ] **Ordre non négociable (F3 correctness)** : le `FOR UPDATE` (verrou sentinelle) doit précéder TOUT SELECT non-locking dans la tx. Le fetch du company stub (read) doit donc se faire soit **hors tx avant `begin()`** (recommandé — simple), soit dans la tx mais **strictement après** `acquire_setup_sentinel_lock`. Ne JAMAIS lire la company (ni aucun non-locking SELECT) avant le verrou : cela figerait prématurément le snapshot MVCC de la 2e requête → `COUNT(*)` stale → race rouverte. Le hash Argon2id se fait aussi hors tx (cf. sous-tâche suivante).
  - [ ] Conserver APRÈS le commit (hors section critique) : `state.users_exist.store(true, Release)`, création JWT + refresh token, `reset` rate-limit IP, `build_auth_cookies`, retour `200 + LoginResponse`. **Rationale** : la fenêtre TOCTOU concerne uniquement count+insert user ; le refresh token reste post-commit comme aujourd'hui (un échec y produit un 500 mais l'admin existe — comportement actuel préservé).
  - [ ] Garder la gestion `UniqueConstraintViolation` (mappée depuis `create_in_tx`) → `users_exist=true` + `410` (défense en profondeur même-username, désormais redondante avec le verrou mais inoffensive).
  - [ ] **Gestion d'erreur de la tx** : si `create_in_tx` (ou le `COUNT`, ou le verrou) remonte une erreur **autre** que `UniqueConstraintViolation`, laisser le `tx` (non-committé) être droppé → rollback automatique sqlx, puis retourner l'erreur (`Err(AppError::Database(e))` / `AppError::Internal` selon le cas). Pas de `tx.rollback().await` explicite requis (le drop suffit) ; un `rollback()` explicite est acceptable si jugé plus lisible — trancher et noter le choix dans le Change Log du dev-story. Sur le chemin 410 « user_count > 0 sous verrou », le rollback (explicite ou par drop) doit précéder le retour.
  - [ ] Hash Argon2id : le réaliser **hors** de la section verrouillée si possible (CPU coûteux) pour minimiser la durée de tenue du verrou — soit avant `pool.begin()`, soit le déplacer. ⚠️ Mais le hash dépend du password validé : valider username/password AVANT d'ouvrir la tx (déjà le cas lignes 78-97), puis hasher avant `begin()`. Documenter ce choix (minimise la fenêtre de contention du verrou InnoDB).

- [ ] **T4 — Convertir le test race en garantie stricte** (AC: #6)
  - [ ] Dans `crates/kesh-api/tests/setup_admin_e2e.rs`, retirer `#[ignore = "..."]` du test race (lignes 348-418), le renommer (ex. `toctou_race_two_distinct_usernames_creates_exactly_one_admin`).
  - [ ] Remplacer l'observation `eprintln!` + asserts laxistes (`>= 1`, `<= 2`) par : `assert_eq!(final_count, 1, "le verrou sentinelle garantit exactement 1 admin")` + assertion que `{res_a.status(), res_b.status()}` == `{200, 410}` (set, ordre indifférent — ex. trier les deux codes et comparer à `[200, 410]`).
  - [ ] Vérifier que la row sentinelle `_kesh_version (id=1)` existe bien dans la DB de test (créée par `kesh_db::MIGRATOR` via la migration `20260522000001`) — sinon le `FOR UPDATE` ne verrouille rien. (Elle existe : la migration fait `INSERT ... VALUES (1, '0.1.0', '0.1.0')`.)
  - [ ] S'assurer que les deux requêtes partagent réellement le même process backend (même `AppState`/pool) pour que la contention DB se produise — le test spawn déjà 2 `tokio::spawn` sur le même serveur de test.

- [ ] **T5 — CHANGELOG (requis — fix de sécurité)** (AC: #7)
  - [ ] Ajouter une section `## [Non publié]` en tête de `CHANGELOG.md` (au-dessus de `[0.1.8]`) avec une entrée `### Sécurité` : « Création du 1er administrateur (`POST /setup/admin`) désormais atomique — fermeture d'une race condition TOCTOU qui pouvait, sous requêtes concurrentes, créer deux comptes admin (#133). » Sera datée/versionnée au prep de release v0.2.0. Comportement utilisateur inchangé. (Un fix de sécurité se trace au CHANGELOG — cf. CLAUDE.md §Synchroniser TOUTES les docs.)

- [ ] **T6 — Quality gate** (AC: #8)
  - [ ] `cargo fmt --all -- --check` + `cargo build --workspace --all-targets` + `cargo clippy --workspace --all-targets -- -D warnings`.
  - [ ] `cargo test --workspace -j1 -- --test-threads=1` (serial, car touche `kesh-db` + tests intégration DB). Pré-requis : MariaDB démarré + droits `CREATE/DROP` sur `*.*` (cf. README §Tests, sqlx crée des bases éphémères `_sqlx_test_*`).
  - [ ] Confirmer 0 régression sur les 8 tests `setup_admin_e2e.rs` + le test race désormais actif.

## Dev Notes

### Fichiers à toucher (et leur état actuel)

| Fichier | Action | État actuel / ce qui change |
|---|---|---|
| `crates/kesh-api/src/routes/setup.rs` | **UPDATE** | Handler `create_admin` (63-221). Section critique non-transactionnelle 99-167 → wrappée dans tx + verrou sentinelle. Le reste (rate-limit, validation, JWT, cookies) préservé. |
| `crates/kesh-db/src/repositories/users.rs` | **UPDATE** | `create(pool, new)` (23+) ouvre sa propre tx interne → ajouter `create_in_tx(&mut tx, new)` + faire déléguer `create`. |
| `crates/kesh-db/src/repositories/users.rs` (helper lock) | **UPDATE** | Helper `pub async fn acquire_setup_sentinel_lock(tx)` colocalisé avec `create_in_tx` (module déjà déclaré → PAS de `pub mod` à ajouter). Miroir de `acquire_company_sentinel_lock` (`bank_accounts.rs:588-598`) sur `_kesh_version id=1`, **mais `None` → `DbError::Invariant`** (divergence critique, cf. T2). |
| `crates/kesh-api/tests/setup_admin_e2e.rs` | **UPDATE** | Test race 348-418 : retirer `#[ignore]`, asserts stricts. |
| `CHANGELOG.md` | **UPDATE** (optionnel) | Section `## [Non publié]` § Sécurité. |

### Pattern de référence à RÉUTILISER (anti-réinvention)

`acquire_company_sentinel_lock` (`crates/kesh-db/src/repositories/bank_accounts.rs:588-598`) — le pattern de verrou sentinelle `SELECT ... FOR UPDATE` existe DÉJÀ dans le projet et est éprouvé (mitigation L5 race primary, FINDING-9 Pass 3 Opus Story v014-1) :

```rust
pub async fn acquire_company_sentinel_lock(
    tx: &mut Transaction<'_, MySql>,
    company_id: i64,
) -> Result<(), DbError> {
    let _: Option<(i64,)> = sqlx::query_as("SELECT id FROM companies WHERE id = ? FOR UPDATE")
        .bind(company_id)
        .fetch_optional(&mut **tx)
        .await
        .map_err(map_db_error)?;
    Ok(())
}
```

Le helper setup en est le calque exact sur `_kesh_version WHERE id = 1` (pas de paramètre `company_id` — sentinelle globale fixe). **Ne PAS inventer** un mécanisme de lock applicatif ad hoc, ni un `Mutex` Rust en mémoire (inefficace multi-process, et le déploiement vise potentiellement plusieurs instances).

### Pourquoi `SELECT ... FOR UPDATE` et PAS `GET_LOCK`

- `SELECT ... FOR UPDATE` sur la row sentinelle = verrou **transaction-scoped**, relâché automatiquement au commit/rollback. Cohérent avec le pattern existant `acquire_company_sentinel_lock`. Pas de fuite de verrou si le handler panique (rollback implicite).
- `GET_LOCK('name', timeout)` = verrou **connection-scoped** advisory, nécessite `RELEASE_LOCK` explicite et une discipline de libération sur tous les chemins d'erreur. Plus fragile. **Rejeté** (l'issue #133 le mentionne comme alternative, mais le pattern FOR UPDATE projet est préférable).

### Pourquoi `_kesh_version` et PAS `companies`

- `_kesh_version` a **toujours exactement une row** (`id = 1`, garantie migration `20260522000001` + CHECK `chk_kesh_version_single_row`). Sentinelle globale fiable, disponible même avant toute création de company.
- La row `companies` du stub peut ne pas exister ou son `id` n'est pas un invariant fixe au moment du setup (le stub est créé par le bootstrap cas 1, mais le gate setup est un concern **global install**, pas per-tenant). Verrouiller `_kesh_version` exprime mieux la sémantique « un seul setup d'installation à la fois ».
- Aucune contention runtime avec la mise à jour `last_boot_at` de `_kesh_version` (Story 10-2 downgrade protection) : celle-ci a lieu une fois au boot, pas pendant le traitement des requêtes.

### Séquence cible du handler (pseudo-ordre)

```
1. rate-limit IP check                      (inchangé, hors tx)
2. valider username (trim) + password (>=12) (inchangé, hors tx)
3. hash Argon2id du password                 (DÉPLACÉ hors tx — CPU coûteux, minimise tenue verrou)
4. let mut tx = state.pool.begin()
5.   acquire_setup_sentinel_lock(&mut tx)    // SELECT _kesh_version id=1 FOR UPDATE
6.   user_count = SELECT COUNT(*) FROM users (&mut *tx)
7.   if user_count > 0 { tx.rollback(); users_exist=true; record_failed_attempt; return 410 }
8.   company_id = fetch stub company         (read)
9.   user = users::create_in_tx(&mut tx, NewUser{...hash...})  // handle UniqueConstraint -> 410
10.  tx.commit()
11. users_exist.store(true, Release)         (post-commit, inchangé)
12. JWT + refresh token + reset rate-limit + build_auth_cookies + 200  (post-commit, inchangé)
```

### Invariants à NE PAS casser (régressions)

- **Cookies HttpOnly** : `build_auth_cookies` (réutilisée `/login` + v011-5) — ne pas toucher.
- **`AppState::users_exist`** (`Arc<AtomicBool>`, `lib.rs:37-44`) : sémantique `Release` au store / `Acquire` au load (middleware `auth.rs:102-104` gate 423). Conserver les 3 points de `store(true, Release)` (gate 410, post-INSERT, UniqueConstraint).
- **Codes HTTP** : `SetupRequired` = 423, `SetupAlreadyComplete` = 410 (`errors.rs:648-661`). Inchangés.
- **Bootstrap matrice 6 cas** (`auth/bootstrap.rs:51-307`) : NE PAS toucher. Le bootstrap au boot et le endpoint setup au runtime sont des chemins distincts ; le fix ne concerne que le runtime endpoint. (Le cas 1 du bootstrap crée le stub company + initialise `users_exist`.)
- **Rate-limiter partagé** (`state.rate_limiter`, quota commun avec `/login`) : conserver `record_failed_attempt` sur le chemin 410 et `reset` sur le chemin succès.
- **Note perf (non bloquante)** : la version actuelle fait un pre-check `SELECT COUNT(*)` hors-tx sur `&state.pool` (lignes 99-115) permettant un early-exit 410 sans ouvrir de transaction. Le fix supprime ce pre-check : chaque POST sur DB déjà initialisée paie désormais une `begin()` + verrou InnoDB. Acceptable — le rate-limiter (5/15 min) plafonne le débit et la route n'est sollicitée que pendant la fenêtre d'onboarding. Comportement observable (410) identique.

### Dialecte & compat MariaDB

- `SELECT ... FOR UPDATE` = InnoDB standard, compatible MariaDB ≥ 10.2 (le projet cible 10.11 parité prod NAS, compat ≥ 10.6 — cf. Story 10-1 D3). Aucun souci de dialecte.
- Lock wait timeout InnoDB par défaut (50 s) : non atteignable en pratique (section critique = 1 SELECT count + 1 INSERT, < quelques ms). Pas de tuning nécessaire.

### Déterminisme du test race (T4)

Avec le verrou, les deux requêtes se sérialisent sur `_kesh_version id=1 FOR UPDATE` :
- La 1re à obtenir le verrou : `user_count == 0` → INSERT → commit → `200`.
- La 2e (bloquée jusqu'au commit de la 1re) : voit `user_count == 1` → rollback → `410`.

Résultat **déterministe et assertable** : `final_count == 1`, status set = `{200, 410}`. L'ordre (qui de alice/bob gagne) reste non déterministe — n'asserter QUE le set des codes et le count, pas qui gagne.

**Pourquoi le `COUNT(*)` de la 2e tx voit bien la donnée committée (REPEATABLE READ)** : sous l'isolation par défaut InnoDB (REPEATABLE READ), le snapshot MVCC d'une transaction est figé à son **premier consistent read non-locking**, pas au `BEGIN`. Comme le `BEGIN` de tx2 est immédiatement suivi du `SELECT … FOR UPDATE` (locking read, qui ne fige PAS le snapshot consistent et bloque tx2 jusqu'au commit de tx1), le `SELECT COUNT(*)` qui suit est le **1er non-locking read** de tx2 → il fige son snapshot APRÈS le déblocage (donc après le commit de tx1) → voit `user_count == 1`. C'est précisément pourquoi le `FOR UPDATE` DOIT être la 1re instruction (aucun non-locking SELECT avant lui).

### Hors-scope (ne PAS faire dans 17-1)

- Ne pas refactorer le bootstrap `ensure_admin_user`. Ne pas toucher au middleware 423. Ne pas modifier le frontend `/setup`. Ne pas ajouter de migration. Ne pas généraliser le verrou à d'autres endpoints (17-2/17-4 décideront de leur propre besoin).

### Project Structure Notes

- Backend Rust workspace multi-crates : `kesh-api` (handler + tests E2E), `kesh-db` (repos users + helper lock). Cohérent avec la séparation existante (logique persistance dans `kesh-db`, HTTP dans `kesh-api`).
- Le helper de verrou appartient à `kesh-db` (couche persistance), comme `acquire_company_sentinel_lock`. Le handler `kesh-api` l'appelle.
- Aucune variance de structure détectée — le fix s'inscrit exactement dans les patterns établis.

### References

- [Source: crates/kesh-api/src/routes/setup.rs#create_admin (63-221)] — handler cible, section critique 99-167.
- [Source: crates/kesh-db/src/repositories/users.rs#create (23+)] — ouvre sa propre tx → besoin `create_in_tx`.
- [Source: crates/kesh-db/src/repositories/bank_accounts.rs#acquire_company_sentinel_lock (588-598)] — pattern verrou sentinelle à calquer.
- [Source: crates/kesh-db/migrations/20260522000001_kesh_version.sql (31-42)] — table singleton `_kesh_version id=1`, sentinelle.
- [Source: crates/kesh-api/tests/setup_admin_e2e.rs (348-418)] — test race `#[ignore]` à convertir.
- [Source: crates/kesh-api/src/lib.rs (28-44)] — `AppState::users_exist` sémantique Acquire/Release.
- [Source: crates/kesh-api/src/errors.rs (93-108, 648-661)] — variants `SetupRequired` (423) / `SetupAlreadyComplete` (410).
- [Source: crates/kesh-api/src/auth/bootstrap.rs (51-307)] — bootstrap 6 cas (hors-scope, ne pas casser).
- [Source: _bmad-output/planning-artifacts/epic-17.md#Story 17-1 + décision D10] — story-zéro sécurité, ordre Epic 17.
- [Source: GitHub #133] — issue d'origine, dette L1 Story v011-5.
- [Source: CLAUDE.md §Migration breaking policy] — pas de migration ici → pas de bump/audit.

## Change Log — spec validate

**Cycle `bmad-create-story validate 17-1` CONVERGÉ en 2 passes (2026-06-04)** — critère d'arrêt CLAUDE.md atteint (0 finding > LOW), budget 2/8.

| Passe | Modèle | Findings | > LOW | Patches |
|---|---|---|---|---|
| 1 | Sonnet 4.6 | 1C + 1H + 1M + 4L | 3 | 7 |
| 2 | Haiku 4.5 | 1L | 0 | 1 |

- **Trend > LOW** : Passe 1 = 3 → Passe 2 = 0 (**convergé**).
- **Patches notables Passe 1** :
  - **F1 (CRITICAL)** : le helper sentinelle ne doit PAS calquer aveuglément `acquire_company_sentinel_lock` (qui retourne `Ok(())` sur `None`) — pour la sentinelle globale `_kesh_version id=1`, `None` → `DbError::Invariant` → 500, sinon la race reste ouverte silencieusement si la migration 10-2 manque.
  - **F2 (HIGH)** : colocaliser le helper dans `users.rs` (module déjà déclaré) pour éviter l'oubli `pub mod` dans `mod.rs`.
  - **F3 (MEDIUM)** : `FOR UPDATE` doit être la 1re instruction de la tx (snapshot MVCC REPEATABLE READ) — aucun SELECT non-locking avant le verrou.
- **Passe 2 Haiku** : ground-truth propre (aucune hallucination), a confirmé `DbError::Invariant → 500` (`crates/kesh-api/src/errors.rs`), 1 seul LOW (clarté rollback du tx) patché.
- **Décisions de reclassement** : aucune.

## Dev Agent Record

### Agent Model Used

(à compléter par dev-story — recommandation : Opus pour l'attention au flux transactionnel cross-crate, mais story petite → Sonnet acceptable)

### Debug Log References

### Completion Notes List

### File List
