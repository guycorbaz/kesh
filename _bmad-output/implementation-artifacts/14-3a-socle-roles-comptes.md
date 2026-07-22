# Story 14.3a : Socle des rôles de comptes — attribut explicite, chart-agnostique

## Status

ready-for-dev

## Story

**As a** utilisateur de Kesh (indépendant, PME, association) ou fiduciaire qui adapte son plan comptable,
**I want** que chaque compte porte un **rôle explicite** (créances clients, TVA due, capital, résultat reporté…) que je peux consulter et modifier, ainsi qu'un indicateur **postable / non-postable**,
**So that** Kesh cesse de deviner la fonction d'un compte à partir de son **numéro** — mon plan comptable n'est pas légalement figé, je dois pouvoir le renuméroter ou le restructurer sans casser la comptabilité.

## Contexte

### Le principe (Guy, 2026-07-21)

> **Chaque compte a un RÔLE explicite. Le numéro ne sert JAMAIS à déduire le rôle.**

Le plan comptable suisse (norme Käfer / PME) est un **usage**, pas une obligation légale. Un utilisateur peut renuméroter ses comptes, en ajouter, en supprimer. Tout code qui écrit `WHERE number = '1100'` est un piège silencieux.

### État des lieux (cartographie 2026-07-22, 2 agents Explore + vérification directe)

- **14-1 a retiré** le hardcode `EQUITY_RESULT_ACCOUNT_NUMBERS = ["2979","2800"]` de `balance_sheet.rs` en rendant les fonds propres **entièrement virtuels**. Ce trou est fermé.
- **Le seul hardcode fonctionnel restant** en Rust de production est `company_invoice_settings.rs` : 6 lookups `WHERE number = '1100' / '3000' / '2000'` (`:275`, `:284`, `:296` + miroir `:396`, `:405`, `:416`) + un fail-fast d'onboarding `if receivable.is_none() || revenue.is_none()` (`:308-311`, `:428-430`). **Sa suppression est la Story 14-3b**, pas celle-ci.
- **Aucune notion de compte non-postable n'existe.** Les comptes titres (`1`, `10`, `28`) sont postables comme les autres ; la seule validation à la saisie est `active = TRUE` (`journal_entries.rs:152-156` create, `:702-706` update, 2 copies).
- **Aucune réactivation de compte archivé** (issue **#269**) : `accounts::update` rejette explicitement un compte archivé (`:186-192` `IllegalStateTransition`), aucune fonction `reactivate`, aucune route, aucun bouton. Le précédent existe pour les projets (`unarchive_project`, `POST /projects/{id}/unarchive`).

### Décision de découpage (Guy, 2026-07-22)

La story 14-3 initiale touchait 6 crates + le frontend → **règle de splitting préventif** (CLAUDE.md, seuil 5 modules) déclenchée. Split validé :

- **14-3a (cette story) — SOCLE.** Pose le modèle et l'UI d'administration : colonnes `role` + `postable`, enum `AccountRole`, entity/repo/API, 3 plans comptables annotés, page Plan comptable enrichie, **réactivation #269**, i18n. **Aucun consommateur du rôle.** Le comportement métier de l'application est **strictement inchangé**.
- **14-3b — CONSOMMATEURS** (story suivante, rollout mécanique) : garde de postabilité à la saisie d'écriture + filtres frontend (7 sites), présentation des fonds propres **par rôle** au bilan (backend + CSV + PDF + frontend), remplacement des 6 lookups par numéro de `company_invoice_settings` par des lookups par rôle.

Ce découpage suit le pattern « story-zero pose le pattern → rollout mécanique » de CLAUDE.md.

### Décision — vocabulaire des rôles (Guy, 2026-07-22 : périmètre « étendu »)

**10 rôles**, stockés en `VARCHAR(32)` PascalCase (même convention que `account_type`), `NULL` = aucun rôle :

| Rôle | Sens | Compte des plans livrés | Singleton ? |
|---|---|---|---|
| `Receivable` | Créances clients (débiteurs) | 1100 | ✅ |
| `DefaultRevenue` | Produit par défaut de facturation | 3000 | ✅ |
| `Payable` | Dettes fournisseurs (créanciers) | 2000 | ✅ |
| `VatRecoverable` | Impôt préalable (TVA récupérable) | 1171 | ✅ |
| `VatPayable` | TVA due | 2200 | ✅ |
| `VatSettlement` | Décompte TVA | 2206 | ✅ |
| `EquityCapital` | Capital (social / exploitant / association) | 2800 | ❌ multi |
| `EquityReserve` | Réserves, fonds affectés/libres, prélèvements et apports privés | 2900, 2850, 2860 | ❌ multi |
| `RetainedEarnings` | Bénéfice / perte reporté | 2970 | ✅ |
| `CurrentYearResult` | Résultat de l'exercice | 2979 | ✅ |

**Singleton** = au plus **un** compte par société peut porter ce rôle (il alimente un champ de configuration unique ou une ligne unique du bilan). `EquityCapital` / `EquityReserve` sont multi-valués : l'indépendant a 2800 capital + 2850 prélèvements + 2860 apports, l'association a 2850 fonds affectés + 2860 fonds libres.

Les **3 plans livrés** (`pme.json`, `association.json`, `independant.json`) utilisent **les mêmes numéros** pour ces 10 rôles — vérifié entrée par entrée. Le backfill est donc uniforme.

### Décision — unicité STRUCTURELLE des rôles singleton (vérifiée empiriquement sur MariaDB)

Un contrôle applicatif « SELECT puis INSERT » laisse une fenêtre de course. Fix **structurel** : une colonne générée `STORED` + un index UNIQUE.

```sql
singleton_role VARCHAR(32) AS (CASE WHEN role IN (<les 8 singletons>) THEN role ELSE NULL END) STORED
CONSTRAINT uq_accounts_company_singleton_role UNIQUE (company_id, singleton_role)
```

`NULL` n'entre pas dans une contrainte UNIQUE MySQL/MariaDB → les comptes sans rôle et les rôles multi-valués sont libres.

**Vérifié sur le conteneur `kesh-mariadb` (2026-07-22)** : plusieurs `NULL` OK, plusieurs `EquityReserve` sur la même société OK, même rôle singleton sur deux sociétés différentes OK, doublon singleton sur la même société → **`ERROR 1062 Duplicate entry '1-Receivable'`**. `ALTER TABLE ADD COLUMN … STORED + ADD CONSTRAINT UNIQUE` puis `UPDATE` de backfill fonctionne sur une table existante peuplée.

**Le backup/restore tolère nativement la colonne générée** — `backup.rs:88-96` (`non_generated_columns`) filtre déjà `EXTRA NOT LIKE '%GENERATED%'` et le doc-comment de `TableExport.column_names:73` dit explicitement « hors colonnes générées ». Rien à faire de ce côté (mais un test le prouve, cf. AC-G).

### Décision — backfill par numéro, UNE SEULE FOIS, en migration (Guy, 2026-07-22)

Les installations existantes (dont le NAS de Guy en v0.7.0) ont déjà un plan comptable **créé par Kesh à partir des plans livrés**. La migration affecte donc les rôles par numéro, `WHERE role IS NULL` (idempotent).

**Le numéro sert une fois, dans une migration de données — jamais dans le code applicatif.** Le principe reste tenu : après la migration, aucune ligne de Rust ne déduit un rôle d'un numéro. L'utilisateur peut tout modifier depuis la page Plan comptable.

### Décision — postabilité : deux backfills, tous deux chart-agnostiques

`postable BOOLEAN NOT NULL DEFAULT TRUE`. Backfill à `FALSE` pour :

1. **Les comptes qui ont des enfants** (comptes titres / de regroupement) — `WHERE EXISTS (SELECT 1 FROM accounts c WHERE c.parent_id = a.id)`. **Purement structurel, aucun numéro.**
2. **Le compte de rôle `CurrentYearResult`** — en modèle temps réel virtuel (14-1), l'application **calcule** le résultat de l'exercice à chaque rendu ; y poster une écriture serait un double-comptage garanti. C'est le « durcissement de 14-1 » annoncé dans `balance_sheet.rs:36-38`.

**`RetainedEarnings` reste postable** — décision explicite : un utilisateur qui **migre** depuis un autre logiciel doit pouvoir poser son report à nouveau d'ouverture. Le rendre non-postable casserait le persona cible de la Story 14-4. L'utilisateur peut le passer non-postable lui-même après migration.

Le backfill #1 s'exécute **avant** #2 dans la migration ; l'ordre n'a pas d'importance fonctionnelle (les deux mettent à `FALSE`) mais il est figé pour la reproductibilité.

### Hors scope (garde-fous — tout ceci est 14-3b ou plus tard)

- ❌ **Appliquer** `postable` à la saisie d'écriture (`journal_entries.rs:152-156` / `:702-706`) → **14-3b**. Ici la colonne est posée, lisible et modifiable, mais **rien ne la lit** côté métier.
- ❌ **Filtrer** les sélecteurs de compte du frontend (7 sites dupliqués) → **14-3b**.
- ❌ **Présentation des fonds propres par rôle** au bilan (`balance_sheet.rs`, CSV, PDF, `BalanceSheetView`) → **14-3b**.
- ❌ **Remplacer** les 6 lookups `number = '1100'/'3000'/'2000'` de `company_invoice_settings.rs` → **14-3b**.
- ❌ **Rôles de trésorerie** (caisse/banque) : écartés du vocabulaire (option non retenue).
- ❌ **Bilan d'ouverture / soldes de départ** → **Story 14-4**.
- ❌ **Réouverture d'exercice** → **Story 14-2**.
- ❌ **Migration du plan comptable d'un org_type à un autre**, import de plan personnalisé : hors Epic 14.

## Acceptance Criteria

### A. Schéma — colonnes `role` et `postable`

- **Given** la table `accounts` existante, **When** la migration s'applique, **Then** elle gagne :
  - `role VARCHAR(32) NULL` avec `CONSTRAINT chk_accounts_role CHECK (role IS NULL OR BINARY role IN (BINARY '<les 10 rôles>'))` — liste fermée, `BINARY` pour neutraliser la collation, **calquée sur `chk_accounts_type`** (`20260411000001_accounts.sql:20`) ;
  - `postable BOOLEAN NOT NULL DEFAULT TRUE` ;
  - `singleton_role VARCHAR(32) AS (CASE WHEN role IN (<les 8 singletons>) THEN role ELSE NULL END) STORED` ;
  - `CONSTRAINT uq_accounts_company_singleton_role UNIQUE (company_id, singleton_role)`.
- **And** la migration est **non-breaking** au sens de la politique P1/P3 (`ADD COLUMN` nullable + `ADD COLUMN NOT NULL DEFAULT` + nouvelle contrainte) → **PAS de bump `kesh_version_min_required`**. Justification à écrire dans l'en-tête SQL : un binaire antérieur sélectionne des colonnes **explicites** (`accounts.rs:17` `COLUMNS`, `:19` `FIND_BY_ID_SQL`) et ne verra jamais `role`/`postable`/`singleton_role` ; ses `INSERT` laissent `role` à `NULL` (aucun conflit sur l'UNIQUE) et `postable` à son défaut.
- **And** `docs/migrations-idempotence-audit.md` reçoit sa ligne (verdict attendu `tracked-by-sqlx`) **et** la section « Statistiques » est incrémentée (`Total : 54 → 55`, `tracked-by-sqlx : 43 → 44`). L'oubli est un finding MEDIUM en code review (politique P5).
- **And** `ALGORITHM=INSTANT` n'est **pas** applicable ici (une colonne générée `STORED` impose une reconstruction de table) — le documenter dans l'en-tête SQL plutôt que de le tenter. Volume concerné : ~84 comptes par société.

### B. Backfill des installations existantes

- **Given** une base issue d'une v0.7.0 avec un plan comptable seedé, **When** la migration s'applique, **Then** les rôles sont affectés selon la table de correspondance du §Contexte, **`WHERE role IS NULL`** (idempotent, ne surcharge jamais un rôle déjà posé).
- **And** `postable = FALSE` pour tout compte ayant au moins un enfant (`parent_id` pointant sur lui), **sans référence à un numéro**.
- **And** `postable = FALSE` pour le compte de rôle `CurrentYearResult`.
- **And** `RetainedEarnings` reste `postable = TRUE`.
- **And** le backfill est **multi-société** : il porte sur toutes les lignes de `accounts`, pas sur une société particulière.
- **And** si une société a déjà **deux** comptes portant le même numéro cible… impossible (`uq_accounts_company_number` le garantit) → le backfill ne peut pas violer l'UNIQUE singleton. **À prouver par un test** (AC-G).

### C. Modèle Rust

- **Given** l'entité `Account`, **When** on la lit, **Then** elle expose `role: Option<AccountRole>` et `postable: bool`.
- **And** `AccountRole` est un enum suivant **exactement** le pattern de `AccountType` (`entities/account.rs:11-66`) : `as_str()`, `FromStr` strict, impls **manuelles** `Type<MySql>` / `Encode` / `Decode` (délégation à `String`) — **pas** de `#[derive(sqlx::Type)]`, cohérence avec l'existant.
- **And** `AccountRole` expose une fonction `is_singleton(&self) -> bool` (source de vérité Rust, **miroir de la liste SQL** de la colonne générée — un commentaire croisé dans les deux fichiers rappelle qu'ils doivent rester synchrones).
- **And** `NewAccount` gagne `role: Option<AccountRole>` et `postable: bool` ; `AccountUpdate` gagne `role: Option<AccountRole>` et `postable: bool`.
- **And** les **deux** listes de colonnes du repository sont mises à jour : `accounts.rs:17` (`COLUMNS`) **et** `accounts.rs:19` (`FIND_BY_ID_SQL`, qui duplique la liste en littéral) — omettre la seconde casse `FromRow` au runtime, pas à la compilation.
- **And** `account_snapshot_json` (`accounts.rs:27-38`) inclut `role` et `postable` (sinon l'audit log ment sur les modifications de rôle).
- **And** `is_no_op_change` (`accounts.rs:164-166`) compare aussi `role` et `postable` — sans quoi un changement de rôle seul est silencieusement ignoré (KF-004 court-circuit).

### D. API comptes

- **Given** `GET /api/v1/accounts`, **When** je liste, **Then** chaque `AccountResponse` porte `role` (`null` si aucun) et `postable` — ajout de champs, **contrat rétro-compatible**.
- **And** `POST /api/v1/accounts` accepte `role` (optionnel, défaut `null`) et `postable` (optionnel, défaut `true`).
- **And** `PUT /api/v1/accounts/{id}` accepte `role` et `postable` (avec `version` pour le verrouillage optimiste).
- **Given** un rôle singleton déjà porté par un autre compte de la société, **When** je l'affecte à un second compte, **Then** l'API répond **409 `RESOURCE_CONFLICT`** avec un message exploitable — la violation `1062` de l'UNIQUE doit être **mappée**, pas remontée en 500. Vérifier le mapping existant de `map_db_error` et l'étendre si `uq_accounts_company_singleton_role` n'est pas déjà couvert par la règle générique « duplicate entry → conflit ».
- **And** RBAC inchangé : lecture pour tout rôle authentifié, écriture `comptable_routes` (`lib.rs:276-284`).

### E. Réactivation d'un compte archivé (issue #269)

- **Given** un compte archivé (`active = FALSE`), **When** un Admin/Comptable le réactive via `PUT /api/v1/accounts/{id}/reactivate` (body `{ version }`), **Then** `active` repasse à `TRUE`, `version` est incrémenté et une entrée d'audit `account.reactivated` est écrite **dans la même transaction** (pattern `archive`, `accounts.rs:270+`).
- **And** la réactivation est **refusée** (`IllegalStateTransition` → 409) si le **compte parent est archivé** — symétrique du garde-fou de `archive()` qui refuse d'archiver un compte ayant des enfants actifs (`accounts.rs:274-283`). Sans ce garde, on obtient un compte actif sous un parent inactif : incohérence d'arborescence.
- **And** réactiver un compte **déjà actif** est un no-op idempotent qui retourne l'entité inchangée **sans** bumper `version` **ni** écrire d'audit (cohérent avec le court-circuit no-op KF-004 de `update`).
- **And** verrouillage optimiste : `version` incorrect → `OPTIMISTIC_LOCK_CONFLICT` (409).
- **And** côté frontend, une ligne archivée de la page Plan comptable affiche un bouton **« Réactiver »** (aujourd'hui `{#if canModify() && account.active}` masque **toute** action sur les lignes archivées, `+page.svelte:287`).

### F. Plans comptables & seed

- **Given** `ChartEntry` (`kesh-core/src/chart_of_accounts/mod.rs:38-46`), **When** un plan JSON est désérialisé, **Then** un champ optionnel `role` est accepté (`#[serde(default)] pub role: Option<AccountRole>`) — **non-breaking** pour un JSON sans rôle.
- **And** les 3 plans (`pme.json`, `association.json`, `independant.json`) portent `"role": "…"` sur les entrées de la table de correspondance, et **uniquement** sur celles-ci.
- **And** `validate_chart` (`mod.rs:88-114`) rejette un plan où **un rôle singleton apparaît deux fois** — la validation existe déjà pour l'unicité des numéros et l'existence des parents ; c'est le même esprit, et ça attrape une faute de frappe dans un JSON avant qu'elle n'atteigne la DB.
- **Given** une nouvelle société créée par l'onboarding, **When** `bulk_create_from_chart` (`accounts.rs:421`) insère le plan, **Then** les rôles du JSON sont persistés **et** `postable` est calculé : `FALSE` pour toute entrée qui est parent d'une autre entrée du plan **ou** portant le rôle `CurrentYearResult`, `TRUE` sinon. Un nouveau seed doit produire **exactement** l'état qu'aurait produit le backfill de la migration sur une base pré-existante (**invariant testé**, AC-G).
- **And** `AccountRole` est **partagé, pas dupliqué** : `kesh-core` possède déjà son propre `AccountType` dupliqué de celui de `kesh-db` (`chart_of_accounts/mod.rs:16-21` vs `entities/account.rs:11-17`) — cette dette existante ne doit **pas** être reproduite. Définir `AccountRole` dans **`kesh-core`** et le réexporter/convertir côté `kesh-db`, ou l'inverse, selon le sens de dépendance réel du workspace (à vérifier dans les `Cargo.toml` : `kesh-db` dépend-il de `kesh-core` ?). Décision à figer **avant** d'écrire le code, pas pendant.

### G. Tests

**Repository (`crates/kesh-db/src/repositories/accounts.rs`, tests inline — 11 tests existants, numéros `T100`/`T200` chart-agnostiques)**

- `create` avec rôle + `postable=false` → relu correctement (round-trip `Encode`/`Decode`).
- `update` change le rôle seul → `version` bumpé + audit `account.updated` dont le `details.before/after` contient l'ancien et le nouveau rôle.
- `update` no-op (mêmes name/type/role/postable) → **pas** de bump de version, **pas** d'audit.
- Rôle singleton en double sur la même société → erreur DB mappée (pas un panic).
- Même rôle singleton sur **deux sociétés** → accepté.
- Deux comptes `EquityReserve` sur la même société → accepté.
- `reactivate` : nominal, parent archivé → refus, déjà actif → no-op sans bump, mauvaise `version` → conflit, audit `account.reactivated` écrit.

**Plans comptables (`kesh-core/src/chart_of_accounts/mod.rs`, 12 tests existants)**

- Les 3 plans se chargent avec leurs rôles ; chaque rôle singleton apparaît **exactement une fois** par plan.
- `validate_chart` rejette un plan avec un singleton dupliqué.

**Migration (`crates/kesh-db/tests/`)**

- `migrations_fresh_install.rs` / `migrations_upgrade_path.rs` : ces suites comptent les lignes de `accounts` (`("accounts", 4)`, `migrations_upgrade_path.rs:~228`) — **vérifier que le compte reste juste** ; cette story n'ajoute aucun compte, seulement des colonnes, donc le compteur ne doit **pas** bouger. Ajouter l'assertion que les colonnes existent et que le backfill a bien tourné sur les comptes seedés par les migrations (`1171`, `2206` insérés par `20260614000001_vat_accounts_config.sql`).
- **Invariant « seed ≡ backfill »** : une base fraîche (seed via `bulk_create_from_chart`) et une base migrée (backfill SQL) produisent le **même** `(role, postable)` pour chaque numéro des 3 plans. Ce test est le filet le plus important de la story.

**Backup / export (`crates/kesh-api/tests/`)**

- `serialize_accounts_csv` (`exports/csv_tables.rs:198-231`) : l'en-tête et les lignes gagnent `role` + `postable` (12 colonnes au lieu de 10). L'en-tête est écrit **à la main** — l'oubli est silencieux.
- La colonne générée `singleton_role` **n'apparaît pas** dans le manifeste de backup ni dans le CSV export → test explicite (le filtre `EXTRA NOT LIKE '%GENERATED%'` de `backup.rs:88-96` est censé s'en charger ; le prouver plutôt que le supposer).
- Les suites `admin_backup_e2e` / `admin_full_import_e2e` restent vertes : un export → import complet round-trip les nouvelles colonnes.

**API E2E (`crates/kesh-api/tests/`)**

- **Il n'existe aujourd'hui aucun test E2E dédié aux comptes** (seul `idor_multi_tenant_e2e.rs` touche `/api/v1/accounts`). Créer `accounts_e2e.rs` : list/create/update avec rôle, conflit 409 sur singleton dupliqué, cycle archive → reactivate, refus de réactivation sous parent archivé, RBAC (un Lecteur ne peut ni créer ni réactiver), IDOR (compte d'une autre société → 404).

**Frontend**

- Vitest sur la page ou sur un composant extrait : rendu de la colonne Rôle, sélecteur de rôle, bouton Réactiver visible **uniquement** sur les lignes archivées.
- Playwright `frontend/tests/e2e/accounts.spec.ts` (116 l. existantes) : étendre avec (a) affectation d'un rôle via le dialog Modifier, (b) cycle archiver → afficher les archivés → réactiver → le compte réapparaît dans la liste par défaut. Conserver la convention `data-testid` existante (`account-row-{number}-…`).

**Gate**

- Backend : `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (ou `DATABASE_URL=…:3307 scripts/test-fast.sh` sur la DB tmpfs, ~3 min).
- Frontend : `npm run check`, `npm run lint-i18n-ownership`, `npm run test:unit`, `npm run build`.
- E2E Playwright (la story touche le frontend) : `npm run test:e2e` avec `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`.

### H. Interface — page Plan comptable

- **Given** la page `/accounts`, **When** je consulte la liste, **Then** une colonne **Rôle** affiche le libellé traduit du rôle (ou un tiret si aucun), et un indicateur signale les comptes **non-postables**.
- **And** les dialogs Créer et Modifier exposent un sélecteur **Rôle** (liste des 10 rôles + « Aucun ») et une case **Postable**.
- **And** une erreur 409 sur rôle singleton dupliqué affiche un message explicite nommant le compte déjà porteur du rôle (pas un « Erreur inattendue »).
- **And** les lignes archivées affichent un bouton **Réactiver** (cf. AC-E).
- **And** la page est **entièrement internationalisée**. Aujourd'hui elle code le français en dur (`ACCOUNT_TYPES`/`TYPE_LABELS` `+page.svelte:18-25`, messages de validation `:117-121` / `:164-167`, messages d'erreur `:142` / `:188` / `:217`) **alors que les clés FTL existent déjà et ne sont utilisées par personne** (`accounts-title`, `accounts-add`, `accounts-edit`, `accounts-archive`, `account-field-*`, `account-type-*`, `account-archived-label` — `fr-CH/messages.ftl:146-159`). Migrer la page vers `i18nMsg` plutôt que d'y ajouter une nouvelle couche de français en dur. La page vit sous `src/routes/`, donc **hors périmètre** de `lint-i18n-ownership` (qui ne parcourt que `src/lib/features/**`, `scripts/lint-i18n-ownership.js`) : les clés singulier `account-*` existantes restent utilisables telles quelles.
- **And** nouvelles clés dans **les 4 locales** (`crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`) : `account-field-role`, `account-role-none`, `account-role-<slug>` × 10 (calqué sur `account-type-<slug>:155-158`), `account-field-postable`, `account-postable-no`, `accounts-reactivate`, `accounts-reactivated`, `accounts-role-conflict`. Attention : `fr-CH` fait 1330 lignes contre 1238 pour les 3 autres — **vérifier l'écart avant d'insérer**, ne pas se fier au numéro de ligne d'une locale pour les autres.
- **And** Svelte 5 runes (`$state`, `$derived`, `$props`), `data-testid` sur chaque élément interactif, `aria-label` sur les boutons icône, gestion `OPTIMISTIC_LOCK_CONFLICT` et `RESOURCE_CONFLICT` explicite — conventions de `RuleFormModal.svelte`.

### I. Non-régression — le comportement métier ne change pas

- **Given** cette story livrée, **When** j'utilise Kesh normalement (saisie d'écritures, factures, rapports, réconciliation), **Then** **rien ne change** : `postable` n'est lu par aucun code métier, les fonds propres du bilan restent rendus comme en 14-1, `company_invoice_settings` cherche toujours ses comptes par numéro.
- **And** les 1900 tests backend et 418 tests frontend de la baseline 14-1 restent verts.

## Tasks / Subtasks

- [ ] **T1** Décider et figer l'emplacement de `AccountRole` (`kesh-core` vs `kesh-db`) en inspectant le sens de dépendance dans les `Cargo.toml` — **avant** toute écriture de code. Ne pas dupliquer l'enum comme l'a été `AccountType` — AC-F.
- [ ] **T2** Migration `2026MMDD000001_accounts_role_postable.sql` : `role` + CHECK, `postable`, `singleton_role` généré + UNIQUE, backfill rôles `WHERE role IS NULL`, backfill `postable` (enfants puis `CurrentYearResult`). En-tête commentée au format des migrations récentes (justification métier, bloc « Idempotence », mention non-breaking / pas de bump) — AC-A/B.
- [ ] **T3** `docs/migrations-idempotence-audit.md` : ligne du tableau + section « Statistiques » (54→55, tracked-by-sqlx 43→44) — AC-A.
- [ ] **T4** `entities/account.rs` : enum `AccountRole` (pattern `AccountType:11-66`) + `is_singleton()` + champs sur `Account` / `NewAccount` / `AccountUpdate` — AC-C.
- [ ] **T5** `repositories/accounts.rs` : `COLUMNS:17` **et** `FIND_BY_ID_SQL:19`, `account_snapshot_json:27-38`, `is_no_op_change:164`, `create:41`, `update:170` (UPDATE SQL + binds), `bulk_create:354`, `bulk_create_from_chart:421` (rôles du chart + calcul de `postable`) — AC-C/F.
- [ ] **T6** `repositories/accounts.rs` : `reactivate(pool, id, version, user_id)` — garde parent archivé, no-op si déjà actif, verrou optimiste, audit `account.reactivated`, calqué sur `archive:270` — AC-E.
- [ ] **T7** `routes/accounts.rs` : DTO (`AccountResponse`, `CreateAccountRequest`, `UpdateAccountRequest`) + `ReactivateAccountRequest` + handler + route `PUT /{id}/reactivate` dans `comptable_routes` (`lib.rs:282-284`) + mapping 1062 → 409 — AC-D/E.
- [ ] **T8** `kesh-core/chart_of_accounts` : `ChartEntry.role` (`#[serde(default)]`) + validation singleton dans `validate_chart:88` + annotation `role` des 3 JSON — AC-F.
- [ ] **T9** `exports/csv_tables.rs:198-231` : 2 colonnes de plus dans l'en-tête **et** les lignes — AC-G.
- [ ] **T10** i18n × 4 locales (nouvelles clés) — AC-H.
- [ ] **T11** Frontend : `accounts.types.ts` (`AccountRole`, champs sur `AccountResponse`/`Create`/`Update`), `accounts.api.ts` (`reactivateAccount`), page `/accounts/+page.svelte` (colonne Rôle, sélecteurs dialogs, case Postable, bouton Réactiver, migration i18n complète) — AC-E/H.
- [ ] **T12** Tests : repo inline, chart, migrations (dont invariant « seed ≡ backfill »), `accounts_e2e.rs` (nouveau), CSV export, Vitest, Playwright `accounts.spec.ts` — AC-G.
- [ ] **T13** Doc : `CHANGELOG.md` (entrée [Non publié]) + `README.md` si la feuille de route bouge. Gate complet backend + frontend + E2E — AC-G.

## Dev Notes

### Pièges, par ordre de coût

1. **`FIND_BY_ID_SQL` duplique la liste de colonnes** (`accounts.rs:19`) au lieu de réutiliser `COLUMNS:17`. Ajouter les champs à `Account` sans mettre à jour **les deux** produit une erreur `FromRow` **au runtime seulement** — la compilation passe. C'est le piège n°1 de cette story.
2. **`is_no_op_change`** (`accounts.rs:164-166`) ne compare aujourd'hui que `name` et `account_type`. Non étendu, un `PUT` qui ne change **que** le rôle est silencieusement ignoré et retourne 200 avec l'ancienne valeur. Bug utilisateur invisible.
3. **La liste des rôles singleton existe à deux endroits** : le `CASE WHEN role IN (…)` de la colonne générée (SQL, figé par la migration) et `AccountRole::is_singleton()` (Rust). Ils **doivent** rester synchrones. Commentaire croisé obligatoire dans les deux fichiers. Une divergence donne soit un 409 inexpliqué, soit une unicité non appliquée.
4. **`chk_accounts_role` est une liste fermée** : ajouter un rôle plus tard = migration. C'est volontaire (cohérent avec `chk_accounts_type:20`) et cohérent avec le `FromStr` strict de l'enum. Ne pas « assouplir » en retirant le CHECK.
5. **`kesh-core::AccountType` et `kesh-db::AccountType` sont déjà dupliqués** (`chart_of_accounts/mod.rs:16-21` vs `entities/account.rs:11-17`, conversion via `as_str()`). Ne pas reproduire ce pattern pour `AccountRole` — T1.
6. **La migration ne peut pas être `ALGORITHM=INSTANT`** à cause de la colonne générée `STORED`. Ne pas copier-coller le pattern de `20260531000001_bank_accounts_archived.sql:15-17` sans réfléchir : il échouerait.
7. **Les 3 plans utilisent les mêmes numéros** pour les 10 rôles (vérifié). Ne pas écrire trois backfills différents.
8. **La page `/accounts` code le français en dur** alors que les clés FTL existent, inutilisées. Ne pas empiler une nouvelle couche de littéraux français — AC-H.

### Contrats backend (ground-truth vérifié par lecture directe, 2026-07-22)

- **Table** `crates/kesh-db/migrations/20260411000001_accounts.sql:6-23` — `number VARCHAR(10)`, `account_type VARCHAR(20)`, `parent_id` auto-réf RESTRICT, `active BOOLEAN DEFAULT TRUE`, `version INT DEFAULT 1`, `uq_accounts_company_number:19`, `chk_accounts_type:20` (modèle du CHECK `BINARY`). **Aucun ALTER ultérieur** : le schéma est celui d'origine. **Aucun index secondaire** hors UNIQUE et FK.
- **Entity** `crates/kesh-db/src/entities/account.rs` — `AccountType:11-17`, `as_str:19-28`, `FromStr:30-41`, `Type<MySql>:42-50`, `Encode:52-59`, `Decode:61-66`, `Account:69-82` (`#[serde(rename_all="camelCase")]` + `sqlx::FromRow`), `NewAccount:87-93`, `AccountUpdate:99-102` (commentaire `:96` « Le numéro n'est PAS modifiable après création »).
- **Repo** `crates/kesh-db/src/repositories/accounts.rs` — `COLUMNS:17`, `FIND_BY_ID_SQL:19`, `account_snapshot_json:27-38`, `create:41`, `find_by_id:98`, `find_by_id_in_company:108`, `count_by_company:127`, `list_by_company:138` (filtre `active = TRUE` si `include_archived=false`, `:153`), `is_no_op_change:164`, `update:170` (rejet compte archivé `:186-192`, no-op `:207-210`, `UPDATE … SET name = ?, account_type = ?, version = version + 1 WHERE id = ? AND version = ? AND active = TRUE` `:213-215`, audit `account.updated`), `archive:270` (garde enfants actifs `:274-283`, audit `account.archived`), `bulk_create:354`, `bulk_create_from_chart:421` (tri topologique par longueur de numéro puis numéro `:434-437`, résolution `parent_number → parent_id`, **pas d'audit** — contexte système), `delete_all_by_company:497`. Tests inline `:521-1100` (11 tests, helpers `test_pool:528`, `cleanup_test_accounts:553` purge `number LIKE 'T%'`).
- **Routes** `crates/kesh-api/src/routes/accounts.rs` — `ListAccountsQuery:23`, `CreateAccountRequest:30`, `UpdateAccountRequest:39`, `ArchiveAccountRequest:47`, `AccountResponse:53-64` + `From<Account>:66-81`, `list_accounts:89`, `create_account:108` (validation `:115-145` : trim, `number` ≤ 10, `name` ≤ 255, parent existant **et actif**), `update_account:164`, `archive_account:195`. Câblage `crates/kesh-api/src/lib.rs:276` (`comptable_routes`), `:278-280`, `:282-284`, `:566-568`, `:572`.
- **Chart** `crates/kesh-core/src/chart_of_accounts/mod.rs` — `AccountType:16-21` (dupliqué), `ChartEntry:38-46` (`#[serde(rename_all="camelCase")]`, `#[serde(rename="type")] account_type`, `parent_number: Option<String>`), `resolve_name:49-57` (fallback `fr` puis numéro), `include_str!:60-62`, `load_chart:71-85` (`"pme"|"association"|"independant"`), `validate_chart:88-114` (unicité numéros + parents existants), tests `:116-317`. Plans : `crates/kesh-core/assets/charts/{pme,association,independant}.json` — 84 / 81 / 84 entrées, une entrée par ligne.
- **Seed / onboarding** — `crates/kesh-api/src/routes/onboarding.rs:196`, `:205`, `:577`, `:590`, `:706`, `:713-729` ; démo `crates/kesh-seed/src/lib.rs:142-155` puis `:189-211`.
- **Saisie d'écriture (à NE PAS toucher ici — 14-3b)** — `crates/kesh-db/src/repositories/journal_entries.rs:146-167` (create) et `:696-718` (update) : `SELECT id FROM accounts WHERE company_id = ? AND active = TRUE AND id IN (…)` puis comparaison de cardinalité → `DbError::InactiveOrInvalidAccounts`. Deux copies à factoriser en 14-3b.
- **Config facturation (à NE PAS toucher ici — 14-3b)** — `crates/kesh-db/src/repositories/company_invoice_settings.rs:275`/`:284`/`:296` (variante pool) et `:396`/`:405`/`:416` (variante tx, marquée `MIRROR`), fail-fast `:308-311` / `:428-430`.
- **Backup** `crates/kesh-db/src/backup.rs` — `TABLES_TO_TRUNCATE:58` (`accounts`, FK self-réf), `TableExport.column_names:73` (« hors colonnes générées »), `non_generated_columns:88-96` (`EXTRA NOT LIKE '%GENERATED%'`), `export_table:120`, `restore_tables_in_tx:394` (INSERT paramétrés depuis les colonnes du manifeste). → **la colonne générée est déjà gérée**, ne rien y modifier.
- **Export CSV global** `crates/kesh-api/src/exports/csv_tables.rs:198-231` — en-tête écrit **à la main** (10 colonnes) + `write_record` ligne à ligne. C'est **là** qu'il faut ajouter les 2 colonnes.
- **i18n** `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — format Fluent, clés kebab-case préfixées par domaine. Comptes : `accounts-title:146` → `account-archived-label:159` (dont `account-type-asset|liability|revenue|expense:155-158`), `nav-accounts:75`, `error-inactive-accounts:205`. `fr-CH` = 1330 lignes, les 3 autres = 1238.

### Contrats frontend (ground-truth vérifié, 2026-07-22)

- `frontend/src/lib/features/accounts/accounts.types.ts` — `AccountType:1`, `AccountResponse:3-15`, `CreateAccountRequest:17-22`, `UpdateAccountRequest:24-28`, `ArchiveAccountRequest:30-32`. Aucun composant dans ce dossier.
- `frontend/src/lib/features/accounts/accounts.api.ts` — `fetchAccounts:9`, `createAccount:14`, `updateAccount:18`, `archiveAccount:27`. **Pas de `reactivateAccount`**.
- `frontend/src/routes/(app)/accounts/+page.svelte` (423 l.) — page unique (liste + 3 dialogs). `ACCOUNT_TYPES:18`, `TYPE_LABELS:20-25` (français en dur), arborescence `treeAccounts:59-84` + indentation `:274`, toggle archivés `:252`, `createValidation:117-121`, mapping `RESOURCE_CONFLICT:142`, `editValidation:164-167`, `OPTIMISTIC_LOCK_CONFLICT:188`/`:217`, `canModify:231-234` (Admin ou Comptable), ligne archivée `:273`/`:284`, bouton archiver conditionné `{#if canModify() && account.active}` `:287`, dialogs `:311-345` (créer), `:369-389` (modifier, numéro `disabled`), `:406-423` (archiver).
- **Précédent réactivation** — `frontend/src/lib/features/projects/projects.api.ts:40` (`unarchiveProject`), `frontend/src/routes/(app)/settings/projects/+page.svelte:135` / `:273` / `:297`, clés `projects-unarchive` / `projects-unarchived`. Autre : `reconciliation-rules-actions-reactivate`.
- **i18n runtime** — `frontend/src/lib/shared/utils/i18n.svelte.ts` (`i18nMsg(key, fallback, args?)`, store runes, `GET /api/v1/i18n/messages`). Import **obligatoirement** depuis ce module. `frontend/scripts/lint-i18n-ownership.js:16` — ne parcourt que `src/lib/features/**`, namespaces globaux `error|tooltip|common|mode|shortcut|demo`.
- **Style de référence pour un formulaire** — `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte` : `interface Props` + `$props()`, `$state`, `$derived`, callbacks en props (`onSuccess`/`onCancel`), `version` dans chaque mutation, `isApiError` de `$lib/shared/utils/api-client`, toasts `svelte-sonner` sur la page / `errorMsg` local dans les modals.
- **E2E** `frontend/tests/e2e/accounts.spec.ts` (116 l.) — seed `with-company`, `loginAndGoToAccounts`, `data-testid` : `account-row-{number}`, `-number`, `-name`, `-type-badge`, `-edit-button`, `-archive-button`, `account-create-button`, `account-show-archived-toggle`, `account-{create,edit,archive}-dialog-{submit,cancel,confirm}`. Couvre titre `:33`, arborescence `:38`, badges `:46`, compteur `:54`, création `:61`, modification `:87`, toggle archivés `:110`. **Pas de test d'archivage effectif ni de réactivation.**

### Conventions de migration (d'après les 3 dernières)

- Nommage `YYYYMMDDHHMMSS_snake_case.sql` ; dernière du repo : `20260715000001_invoice_reminders.sql`.
- En-tête de commentaire long **obligatoire** : story + issue, justification métier de chaque colonne, décision FK/index, bloc explicite « Idempotence (docs/migrations-idempotence-audit.md) : … », mention non-breaking / bump `min_required`.
- Convention majoritaire : **pas de `IF NOT EXISTS`** (verdict `tracked-by-sqlx`, 43 des 54 migrations).
- Justifier explicitement l'absence d'index quand c'est un choix (YAGNI), modèle : `20260531000001_bank_accounts_archived.sql:7-13`.
- **Pas de bump `kesh_version_min_required`** ici (P2/P3) — seul bump du repo à ce jour : `20260714000002_email_templates_reminder.sql` (`'0.7.0'`).

### Leçons de review à appliquer dès le dev

- **Un patch = un test** (`feedback_review_patch_needs_test`) — chaque correction de review vient avec sa régression testée, sinon la remédiation devient la source des findings suivants.
- **Fix structurel > incrémental** — l'unicité des rôles singleton est garantie par le schéma (colonne générée + UNIQUE), pas par un `SELECT` applicatif. Même esprit que la garde `create_in_tx` de 14-1.
- **Grep ground-truth obligatoire** sur tout finding CRITICAL/HIGH affirmant l'absence d'un patch ou la présence d'un anti-pattern (`grep -nF`), en particulier avec un reviewer Haiku.

### Références

- **Story 14-1** `_bmad-output/implementation-artifacts/14-1-cloture-report-des-soldes.md` — modèle temps réel virtuel, retrait du hardcode `EQUITY_RESULT_ACCOUNT_NUMBERS`, renvoi explicite du durcissement à 14-3 (`balance_sheet.rs:30-38`).
- **Story 14-3b** (suivante) — consommateurs des rôles.
- **Story 14-4** — bilan d'ouverture, dépend de cette story.
- **Issue #269** — réactiver un compte archivé.
- Epic 14 « Clôture d'Exercice » (`_bmad-output/planning-artifacts/epics.md:1316-1332`, ex-Epic 13, FR60/FR61/FR62).
- Politiques CLAUDE.md : `## Migration breaking policy` (P1-P5), `## Règle de splitting préventif`, `## Review Iteration Rule`, `## Test Locally First`.
- Cartographie : 2 agents Explore (backend + frontend) + vérification directe des ancres, 2026-07-22. Contrainte d'unicité validée empiriquement sur `kesh-mariadb`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List
