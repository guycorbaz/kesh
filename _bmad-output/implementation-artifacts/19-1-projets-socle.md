# Story 19.1 : Socle — entité Projet & dimension analytique

Status: done

<!-- Story-zéro de l'Epic 19 (compta analytique par projet). Pose le PATTERN
     (table projects + dimension project_id sur journal_entry_lines + propagation) ;
     19-2..19-5 rollent ce pattern sur chaque flux de saisie, 19-6 = rapports. -->

## Story

As a comptable/indépendant PME utilisant Kesh,
I want créer et gérer des **projets** (avec sous-projets) et disposer d'une **dimension analytique** sur les écritures comptables,
so that je pourrai ensuite rattacher mes dépenses et revenus à un projet (rénovation, investissement) et les analyser isolément.

## Contexte & source

- **Epic 19** — design `_bmad-output/planning-artifacts/epic-19-analytique-projet-design.md`. Décisions Guy D1-D4 validées 2026-07-01 : hybride (document + ligne), 2 niveaux (projet → sous-projets), tous comptes (charges + produits + bilan), exercice + cumulé.
- **Objectifs métier** : (1) rénovations déductibles = capter *toutes* les dépenses d'un projet ; (2) investissements = analyser le rendement (coût total investi, incl. sous-projets).
- Cette story ne fait **que le socle** : entité + dimension + CRUD + admin. Le tagging des flux (19-2..5) et les rapports (19-6) viennent après.

## Acceptance Criteria

**Modèle de données (kesh-db)**

1. Migration `CREATE TABLE projects` : `id`, `company_id` (FK companies, scoping multi-tenant), `parent_id` (FK projects NULL = racine), `code` (VARCHAR(32), UNIQUE `(company_id, code)`), `name` (VARCHAR(150)), `description` (TEXT NULL), **`archived BOOLEAN NOT NULL DEFAULT FALSE`** (choix retenu au dev : calque le flag `archived` de `bank_accounts` plutôt qu'un `status` ENUM — plus simple, cohérent avec le codebase ; les stories 19-2..5 référencent `archived`/`project.archived`), `start_date`/`end_date` (DATE NULL), `version`, `created_at`/`updated_at`. Index `(company_id, parent_id)` + `(company_id, archived)`. InnoDB utf8mb4. **Non-breaking** (CREATE TABLE) → **pas** de bump `kesh_version_min_required`. Ligne ajoutée à `docs/migrations-idempotence-audit.md`.
2. Migration `ALTER TABLE journal_entry_lines ADD COLUMN project_id BIGINT NULL` + FK vers `projects(id)` + index. **Non-breaking** (ADD COLUMN nullable) → pas de bump. Ligne idempotence-audit.
3. **Contrainte 2 niveaux (D2)** appliquée en repo : un projet dont `parent_id` est renseigné doit avoir un parent **racine** (`parent.parent_id IS NULL`) ; un projet ayant des sous-projets ne peut pas recevoir de `parent_id`. Violations → `DbError` métier clair (pas de panic).

**Entité + repo (kesh-db)**

4. Entité `Project` (+ `NewProject`, `UpdateProject`) FromRow, re-exportée. Repo `projects` : `create`, `list` (par company, tri racine puis sous-projets, filtrable `include_archived`), `get` (scopé company), `update`, `archive`/`unarchive` (pas de DELETE dur si des lignes d'écriture y sont rattachées — archivage). Toutes les requêtes **scopées `company_id`** (IDOR-safe). Unicité `code` par company → `DbError` dédié.

**API (kesh-api)**

5. Routes `/api/v1/projects` (Comptable+, sous `comptable_routes`/`require_comptable_role`) : `GET` (liste, `?includeArchived`), `GET /{id}`, `POST` (create), `PUT /{id}` (update), `POST /{id}/archive`, `POST /{id}/unarchive`. DTOs camelCase. Erreurs métier 4xx (jamais 500) : code dupliqué → 409/422, parent invalide (2 niveaux) → 400, not found/cross-company → 404.

**Frontend (admin)**

6. Page **Administration → Projets** (`/settings/projects` ou `/projects`) : liste en **arbre 2 niveaux** (projet → sous-projets), création/édition (code, nom, description, projet parent optionnel, dates), archivage/désarchivage, toggle « afficher archivés ». i18nMsg avec fallback FR. Entrée de menu dans le groupe **Administration** de `+layout.svelte`.

**Intégration export/backup (CRITIQUE)**

7. La nouvelle table `projects` est ajoutée au **round-trip `.keshbackup`** : sérialiseur/manifeste d'export + `TABLES_TO_TRUNCATE` (restore import) + compteur `data_count` de `admin_full_export_e2e.rs` bumpé. Le round-trip export→import reste vert (aucune table oubliée).

**Tests / qualité**

8. Tests repo (create/list arbre/get scopé/update/archive + unicité code + garde 2 niveaux + IDOR cross-company), intégration API (CRUD + 4xx erreurs + auth 401 + Comptable gate), frontend unit (helper d'arbre éventuel), E2E optionnel (créer projet + sous-projet + archiver). Round-trip backup vert. Quality gate **Test Locally First** exit-code vérifié (pas de `cargo test | grep`). Compteurs migration/export mis à jour cohéremment.

## Points de décision (DC)

- **DC1 — Nom de l'entité** [défaut « Projet », à confirmer Guy] : l'UI/les rapports diront « Projet ». Alternatives évoquées : Affaire, Chantier. *Choix par défaut retenu pour ne pas bloquer le socle ; renommable (libellés i18n) sans impact schéma.*
- **DC2 — Hiérarchie 2 niveaux** [FIGÉ D2] : `parent_id` auto-référent, contrainte 1 seul niveau appliquée en repo (pas de contrainte SQL récursive — MariaDB ne le permet pas simplement).
- **DC3 — Archivage vs suppression** [FIGÉ] : pas de DELETE dur (préserve l'historique analytique). `archived = TRUE` masque des sélecteurs. **Gardes anti-orphelin (code-review Pass 1)** : (a) on refuse d'archiver une racine ayant des sous-projets **actifs** (archiver les enfants d'abord) ; (b) on refuse de désarchiver un sous-projet dont la racine est archivée ; (c) on refuse un parent archivé à la création/édition. Ces trois règles interdisent l'état « sous-projet actif sous racine archivée ».
- **DC4 — `code` obligatoire + unique/company** [proposition] : identifiant court lisible (ex. `RENOV-CHALET`) en plus du nom. Unicité `(company_id, code)`. *À confirmer : `code` obligatoire ou optionnel ?* Défaut : obligatoire (facilite le référencement dans les rapports).

## Tasks / Subtasks (à affiner au dev)

- [x] **T1 — Migrations** (AC: 1,2) : `CREATE TABLE projects` + `ALTER journal_entry_lines ADD project_id` (2 fichiers ou 1) ; 2 lignes idempotence-audit ; **pas** de bump min_required.
- [x] **T2 — Entité + repo** (AC: 3,4) : `entities/project.rs` (+ mod/lib) ; `repositories/projects.rs` (CRUD + garde 2 niveaux + scoping company + unicité code) ; `DbError` variants si besoin ; tests repo.
- [x] **T3 — API** (AC: 5) : `routes/projects.rs` (handlers + DTOs) ; montage dans `comptable_routes` (lib.rs) ; mapping erreurs 4xx ; tests intégration.
- [x] **T4 — Frontend admin** (AC: 6) : feature `projects` (api/types) + page `/settings/projects` (arbre + form + archivage) + entrée menu Administration + i18n ; unit test helper.
- [x] **T5 — Export/backup** (AC: 7) : ajouter `projects` au sérialiseur/manifeste + `TABLES_TO_TRUNCATE` + bump compteur `admin_full_export_e2e` ; round-trip vert.
- [x] **T6 — Gate** (AC: 8) : Test Locally First complet (fmt/clippy/build/test workspace serial + front check/lint/unit/build + E2E), exit-codes vérifiés.

## Dev Notes

### Ground-truth (à compléter par cartographie Explore)
- `journal_entry_lines` : `crates/kesh-db/migrations/20260412000001_journal_entries.sql` (id/entry_id/account_id/line_order/debit/credit).
- Pattern CRUD de référence : `vat_rates` (Epic 11) — entité + repo + route + frontend settings.
- Backup : `admin_full_export` — TABLES_TO_TRUNCATE + manifeste + `admin_full_export_e2e.rs` data_count.
- Migration breaking policy : CREATE TABLE + ADD COLUMN nullable = **non-breaking** (CLAUDE.md P1/P3) → pas de bump.

### Conventions projet
- Multi-tenant : toute requête scopée `company_id` (IDOR). DTOs camelCase (serde rename_all). `AppError` 4xx typée jamais 500. i18n route-file fallback FR inline. Test Locally First exit-code (pas de pipe grep, `feedback_cargo_test_pipe_masks_exit`). Ajout table → TABLES_TO_TRUNCATE + manifeste + compteur (`project_12_1_avoirs_pr` leçon).

## Change Log — code-review

**Dev** : T1 migration (projects 2 niveaux + project_id sur journal_entry_lines, non-breaking) + T2 entité/repo (CRUD company-scoped, verrou optimiste, has_children) + T3 routes (garde hiérarchie 2 niveaux, sentinel lock + audit) + T4 frontend (page arbre /settings/projects) + T5 backup (TABLES_TO_TRUNCATE + compteur 30→31) + T6 gate. Cartographie pattern par agent Explore (vat_rates/bank_accounts).

**Code-review — CONVERGÉ 3 passes (Sonnet → Haiku → Opus)** :
- **Pass 1 (Sonnet, 2 couches)** : 4 MEDIUM + LOW. (a) **500 latent** : longueur code/name non validée → MariaDB 1406 non mappée → 500 ; fix `validate_fields` bornes 32/150 + maxlength. (b) **anti-orphelin** : archiver une racine avec sous-projets actifs / parent archivé au create → enfants orphelins ; fix `has_active_children` + gardes create/archive/unarchive + frontend (exclusion racines archivées, empty-state sur `projects.length`). (c) spec `archived BOOLEAN` (calque bank_accounts). (d) tests AC8 : +403 Comptable, +IDOR cross-company, +longueur, +gardes archive.
- **Pass 2 (Haiku, 2 couches)** : 1 MEDIUM-UX (toggleArchive `load()` sur erreur → rafraîchit version après 409 concurrent) + 1 LOW (test désarchivage sous racine archivée). Correctness 0>LOW ground-truthé.
- **Pass 3 (Opus, architectural)** : **0 > LOW**. 6 axes tracés OK : FK `project_id` RESTRICT inerte (pas de hard-delete) ; backup FK_CHECKS=0 transactionnel (ordre `projects` non-critique) ; **sentinel-lock sérialise le TOCTOU hiérarchie** (pas de 3e niveau ni cycle sous concurrence) ; verrou+garde cohérents (même tx) ; scoping complet ; forward-compat epic (19-3/4/6 = ADD COLUMN additifs). LOW : +garde `start_date ≤ end_date` (ajouté, utile 19-6).

**Trend > LOW : 4 (Sonnet) → 1 (Haiku) → 0 (Opus).** Gate final : fmt + clippy 0w + **12 tests intégration** + migration/backup schéma-sync + export round-trip 7/7 (30→31) + front check 0/lint/340 unit/build + **E2E projets 1/1 réel**. Story-zéro Epic 19 : pose le pattern dimension+propagation pour 19-2..6.
