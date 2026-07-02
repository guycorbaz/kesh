# Story 19.1 : Socle — entité Projet & dimension analytique

Status: in-progress

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

1. Migration `CREATE TABLE projects` : `id`, `company_id` (FK companies, scoping multi-tenant), `parent_id` (FK projects NULL = racine), `code` (VARCHAR, UNIQUE `(company_id, code)`), `name`, `description` (NULL), `status` (`active`/`archived`, défaut `active`), `start_date`/`end_date` (NULL), `created_at`/`updated_at`. Index `(company_id, parent_id)`. InnoDB utf8mb4. **Non-breaking** (CREATE TABLE) → **pas** de bump `kesh_version_min_required`. Ligne ajoutée à `docs/migrations-idempotence-audit.md`.
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
- **DC3 — Archivage vs suppression** [FIGÉ] : pas de DELETE dur (préserve l'historique analytique). `status='archived'` masque des sélecteurs. Suppression d'un projet racine ayant des sous-projets interdite (archiver d'abord).
- **DC4 — `code` obligatoire + unique/company** [proposition] : identifiant court lisible (ex. `RENOV-CHALET`) en plus du nom. Unicité `(company_id, code)`. *À confirmer : `code` obligatoire ou optionnel ?* Défaut : obligatoire (facilite le référencement dans les rapports).

## Tasks / Subtasks (à affiner au dev)

- [ ] **T1 — Migrations** (AC: 1,2) : `CREATE TABLE projects` + `ALTER journal_entry_lines ADD project_id` (2 fichiers ou 1) ; 2 lignes idempotence-audit ; **pas** de bump min_required.
- [ ] **T2 — Entité + repo** (AC: 3,4) : `entities/project.rs` (+ mod/lib) ; `repositories/projects.rs` (CRUD + garde 2 niveaux + scoping company + unicité code) ; `DbError` variants si besoin ; tests repo.
- [ ] **T3 — API** (AC: 5) : `routes/projects.rs` (handlers + DTOs) ; montage dans `comptable_routes` (lib.rs) ; mapping erreurs 4xx ; tests intégration.
- [ ] **T4 — Frontend admin** (AC: 6) : feature `projects` (api/types) + page `/settings/projects` (arbre + form + archivage) + entrée menu Administration + i18n ; unit test helper.
- [ ] **T5 — Export/backup** (AC: 7) : ajouter `projects` au sérialiseur/manifeste + `TABLES_TO_TRUNCATE` + bump compteur `admin_full_export_e2e` ; round-trip vert.
- [ ] **T6 — Gate** (AC: 8) : Test Locally First complet (fmt/clippy/build/test workspace serial + front check/lint/unit/build + E2E), exit-codes vérifiés.

## Dev Notes

### Ground-truth (à compléter par cartographie Explore)
- `journal_entry_lines` : `crates/kesh-db/migrations/20260412000001_journal_entries.sql` (id/entry_id/account_id/line_order/debit/credit).
- Pattern CRUD de référence : `vat_rates` (Epic 11) — entité + repo + route + frontend settings.
- Backup : `admin_full_export` — TABLES_TO_TRUNCATE + manifeste + `admin_full_export_e2e.rs` data_count.
- Migration breaking policy : CREATE TABLE + ADD COLUMN nullable = **non-breaking** (CLAUDE.md P1/P3) → pas de bump.

### Conventions projet
- Multi-tenant : toute requête scopée `company_id` (IDOR). DTOs camelCase (serde rename_all). `AppError` 4xx typée jamais 500. i18n route-file fallback FR inline. Test Locally First exit-code (pas de pipe grep, `feedback_cargo_test_pipe_masks_exit`). Ajout table → TABLES_TO_TRUNCATE + manifeste + compteur (`project_12_1_avoirs_pr` leçon).
