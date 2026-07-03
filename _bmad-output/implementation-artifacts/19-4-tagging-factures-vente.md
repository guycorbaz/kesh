# Story 19.4 : Tagging analytique des factures de vente

Status: review

<!-- Rollout du pattern 19-3 (tag document-level + propagation) sur le flux
     factures CLIENT. Divergence structurelle vs 19-3 : la facture de vente est
     un BROUILLON (l'écriture est postée plus tard à validate_invoice), là où la
     facture fournisseur postait à la création. Le tag est donc stocké sur la
     facture (validé), puis relu/re-validé/propagé au posting. -->

## Story

As a comptable/indépendant PME utilisant Kesh,
I want affecter un **projet analytique** à une facture de vente (à la création ou en édition de brouillon),
so that les revenus du projet alimentent le rapport de rendement (revenus / coût investi) au même titre que les dépenses.

## Contexte & source

- **Epic 19** design §3.3 : `ALTER TABLE invoices ADD COLUMN project_id` + propagation aux lignes d'écriture à la validation. Story 19-4 du découpage, dépend de 19-1 (mergée).
- **Branche stackée sur 19-2** (PR #203) : le helper `projects::validate_taggable_in_tx` (sentinel + FOR UPDATE, Pattern 5) y est introduit — réutilisé tel quel ici.
- **Mécanisme central déjà en place** : `journal_entries::create_in_tx` bind `line.project_id.or(new.project_id)` — il suffit de renseigner `NewJournalEntry.project_id` (aujourd'hui `None` aux sites invoices/credit_notes).
- **Divergence 19-3 assumée** : `mark_as_paid` ne poste AUCUNE écriture en v0.1 (encaissement = réconciliation Epic 6/8) → rien à stamper au paiement, contrairement à `pay_in_tx` supplier.

## Acceptance Criteria

**Migration (kesh-db)**

1. Migration `ALTER TABLE invoices ADD COLUMN project_id BIGINT NULL` + FK `projects(id)` ON DELETE RESTRICT + index — calque exact de `20260702000002_supplier_invoices_project.sql` (19-3). **Non-breaking** → pas de bump `kesh_version_min_required`. Ligne ajoutée à `docs/migrations-idempotence-audit.md` (+ stats 41→42).
2. Compteur `crates/kesh-db/tests/migrations_upgrade_path.rs` bumpé : `assert_eq!(total, 41)` → 42 (`:68-72`) ET fenêtre `total - 18` → `total - 19` (`:96`, + commentaires `:61-95`). `admin_full_export_e2e` data_count reste 32 (colonne, pas de table — vérifier au run).

**Entité + repo (kesh-db)**

3. `Invoice` (`entities/invoice.rs:17-36`) + `NewInvoice` (`:67-74`) + `InvoiceUpdate` (`:83-89`) portent `pub project_id: Option<i64>` (miroir `SupplierInvoice:36-37`). `InvoiceLine`/`NewInvoiceLine` inchangés (tag document-level).
4. SELECTs étendus : `FIND_INVOICE_SCOPED_SQL` (`repositories/invoices.rs:41-43`), SELECT inline de `delete()` (`:816-820`), **SELECT de `credit_notes.rs:222-225`** (colonnes explicites — sans quoi le FromRow `Invoice` casse).
5. `invoices::create` (`:343-416`) : validation projet AVANT l'INSERT via `super::projects::validate_taggable_in_tx(&mut tx, new.company_id, &[pid])` (si `Some`), INSERT étendu (+colonne +bind). Erreurs : inconnu/cross-company → `NotFound` (404), archivé → `IllegalStateTransition` (409).
6. `invoices::update` (brouillon, `:740-756`) : `project_id` modifiable ; validation **uniquement si la valeur change** vers un `Some` différent de la valeur stockée (grandfathering du tag inchangé — éditer l'échéance d'un brouillon dont le projet a été archivé entre-temps ne doit pas 409, leçon 19-2). UPDATE SQL étendu.
7. `validate_invoice` (`:1022-1237`) : **re-validation au posting** — le projet peut avoir été archivé entre le brouillon et la validation ; toute nouvelle entrée au grand livre doit être sur projet actif (sémantique 19-3 « ne pas taguer un projet clos » : chez supplier, création = posting ; ici le posting est différé, la re-validation restaure l'équivalence). Puis `NewJournalEntry.project_id: invoice_before.project_id` (site `:1147`, remplace `None`). Le helper `generate_invoice_journal_lines` (`:933-998`) reste intact (lignes `None`, propagation via `or()`).
8. **Avoir hérite le projet** : `credit_notes::create_credit_note` — `NewJournalEntry.project_id: invoice.project_id` (site `:343`, remplace `None`). La contre-passation reprend le projet de la facture d'origine → **net par projet = 0** (miroir cancel supplier 19-3). PAS de re-validation archivé sur l'avoir (annuler une facture d'un projet archivé doit rester possible — miroir `pay/cancel-after-archive` 19-3).
9. `mark_as_paid` : AUCUN changement (pas d'écriture postée v0.1) — divergence vs supplier documentée.

**API (kesh-api)**

10. `CreateInvoiceRequest` (`routes/invoices.rs:72-82`) + `UpdateInvoiceRequest` (`:84-95`) : `#[serde(default)] project_id` (camelCase `projectId`). `InvoiceResponse` (`:150-172` + `from_parts:187-211`) expose `projectId`. Handler `create_invoice` passe `req.project_id` dans `NewInvoice`. `validate_invoice_handler` inchangé (le projet est déjà persisté).

**Frontend**

11. `InvoiceForm.svelte` (composant partagé create+edit, monté par `/invoices/new`) : sélecteur projet **document-level** arbre 2 niveaux (copie `supplier-invoices/+page.svelte:345-362`, `data-testid="invoice-project"`), state `projectId` initialisé depuis `initialInvoice?.projectId ?? null`, chargement `listProjects()` (actifs) via `$effect` (pattern VAT rates `:46-64`), rendu seulement si `projects.length > 0` OU tag existant (leçon 19-2 BH-L2 : tag historique visible avec option ad-hoc « Projet archivé » si absent de la liste), `projectId` dans les payloads create/update (`onSubmit:318-326`).
12. Détail `/invoices/[id]/+page.svelte` : affichage « Projet analytique » (`projectLabel` via `listProjects(true)` — archivés inclus, miroir détail supplier `:37-42`), `data-testid="invoice-project"`, rien affiché si non tagué.
13. Types TS `invoices.types.ts` : `CreateInvoiceRequest.projectId?` (`:142-148`), `InvoiceResponse.projectId` (`:26-44`) (`UpdateInvoiceRequest` hérite).

**Tests / qualité**

14. Tests repo dans `crates/kesh-db/tests/invoices_validate_vat.rs` (helpers existants `make_contact:23`, `create_and_validate:46`, + `make_project` copié de `supplier_invoices_repository.rs:514-529`) : (a) facture avec projet → validation → **toutes** les lignes de l'écriture de vente portent le projet ; (b) create avec projet archivé rejeté 409 ; (c) create projet inconnu/cross-company rejeté 404 ; (d) update change le projet → validé, update sans changement avec projet archivé → passe (grandfathering) ; (e) validate_invoice avec projet archivé post-création → rejeté ; (f) avoir : net par projet = 0 après contre-passation (dans `credit_notes_repository.rs`, calque `cancel_nets_project_to_zero:618`).
15. Test E2E (`frontend/tests/e2e/invoices.spec.ts`) : création facture avec projet (sélecteur) + assertion détail. Sélecteurs Playwright : attention collision `getByRole('option')` (leçon 19-2 — scoper au conteneur).
16. CHANGELOG [Non publié] : entrée factures de vente (miroir 19-3). Quality gate **Test Locally First** complet, exit-codes vérifiés, workspace serial (kesh-db touché).

## Points de décision (DC)

- **DC1 — Projet modifiable en édition de brouillon** [FIGÉ oui] : formulaire partagé create+edit, `InvoiceUpdate.project_id`. Validation seulement si la valeur change (grandfathering du tag inchangé).
- **DC2 — Re-validation au posting** [FIGÉ oui] : `validate_invoice` re-valide le projet (archivé entre brouillon et validation → 409, l'utilisateur détague ou désarchive). Rétablit l'équivalence avec 19-3 où création = posting.
- **DC3 — Avoir sans re-validation** [FIGÉ] : la contre-passation hérite le projet sans check archivé (annulation toujours possible, net-zéro garanti) — miroir pay/cancel-after-archive 19-3.
- **DC4 — `mark_as_paid` sans stamping** [FIGÉ] : aucune écriture postée v0.1 (encaissement = réconciliation). Le règlement réconcilié pourra être tagué via 19-5.

## Tasks / Subtasks

- [x] **T1 — Migration + compteurs** (AC: 1-2) : fichier SQL calqué 19-3, idempotence-audit, `migrations_upgrade_path` 41→42 + fenêtre.
- [x] **T2 — Entité + repo** (AC: 3-9) : structs, SELECTs (invoices ×2 + credit_notes), create/update validation + grandfathering, validate_invoice re-validation + stamping, credit_note héritage ; tests repo (14a-f).
- [x] **T3 — API** (AC: 10) : DTOs + handler create.
- [x] **T4 — Frontend** (AC: 11-13) : types TS, InvoiceForm sélecteur + payloads, page détail, i18n fallback FR.
- [x] **T5 — Tests E2E + doc** (AC: 15-16) : spec invoices + CHANGELOG.
- [x] **T6 — Gate** (AC: 16) : Test Locally First complet backend serial + frontend + E2E, exit-codes vérifiés.

## Dev Notes

### Ground-truth (cartographie Explore 2026-07-03)

- Dernière migration : `20260702000002_supplier_invoices_project.sql` (à calquer). Compteur : `migrations_upgrade_path.rs:68-72` (`total, 41`) + `:96` (`total - 18`).
- Entités : `entities/invoice.rs` — `Invoice:17-36`, `NewInvoice:67-74`, `InvoiceUpdate:83-89`.
- Repo : `repositories/invoices.rs` — `FIND_INVOICE_SCOPED_SQL:41-43`, `create:343-416` (tx déjà ouverte `:357`, INSERT `:359-371`), `update:740-756`, `validate_invoice:1022-1237` (site `project_id: None` **`:1147`**), `generate_invoice_journal_lines:933-998` (ne pas toucher), `delete` SELECT inline `:816-820`, `mark_as_paid:1261-1387` (aucune écriture).
- Credit notes : `repositories/credit_notes.rs` — SELECT facture `:222-231` (colonnes explicites À ÉTENDRE), site `project_id: None` **`:343`**, helper lignes `:139-197` (ne pas toucher).
- Routes : `routes/invoices.rs` — `CreateInvoiceRequest:72-82`, `UpdateInvoiceRequest:84-95`, `InvoiceResponse:150-172`, `from_parts:187-211`, `create_invoice:482-513` (NewInvoice `:495-505`). Miroir supplier : `routes/supplier_invoices.rs:163-174` (DTO), `:271` (mapping).
- Frontend : `lib/components/invoices/InvoiceForm.svelte` (formulaire partagé, `$effect` VAT rates `:46-64`, `onSubmit:318-326`), `routes/(app)/invoices/[id]/+page.svelte` (`onMount:68-81`), `lib/features/invoices/invoices.types.ts` (`:26-44`, `:142-148`). Sélecteur modèle : `supplier-invoices/+page.svelte:345-362`. Détail supplier modèle : `projectLabel:37-42`, chargement `:57-67`.
- Tests : `tests/invoices_validate_vat.rs` (cible repo, `#[sqlx::test(migrator)]`), `tests/supplier_invoices_repository.rs:514+` (make_project + 4 tests 19-3 à transposer), `tests/credit_notes_repository.rs` (net-zéro), `frontend/tests/e2e/invoices.spec.ts` (`createContactViaApi:39`).

### Pièges connus

1. **SELECT credit_notes.rs:222-225 oublié** → FromRow `Invoice` panique à l'exécution dès qu'un avoir est créé (colonne manquante). C'est LE piège de cette story.
2. **`delete()` SELECT inline** (`invoices.rs:816-820`) — deuxième SELECT à colonnes explicites.
3. Re-validation `validate_invoice` : utiliser le helper DANS la tx existante, AVANT `journal_entries::create_in_tx` (ordre sentinel→projects déjà cohérent — le lock fiscal_years est pris par create_in_tx après).
4. `#[sqlx::test(migrator)]` : DB isolée par test — pas besoin du pattern mk_project idempotent de 19-2 (qui vivait dans le mod tests live-DB).
5. E2E : scoper les sélecteurs option (collision select projet ↔ autocomplete/pickers, leçon 19-2).

### Conventions projet

Multi-tenant scopé company (IDOR), DTOs camelCase, AppError 4xx jamais 500, i18n fallback FR inline, commit par étape BMAD, branche `story/19-4-tagging-factures-vente` (stackée sur story/19-2 — PR base à retarger sur main après merge #203).

### References

- [Source: _bmad-output/planning-artifacts/epic-19-analytique-projet-design.md §3.3, §5]
- [Source: _bmad-output/implementation-artifacts/19-2-tagging-ecritures-manuelles.md — helper + grandfathering + leçons E2E]
- [Source: commit deb918d (19-3) — pattern document-level supplier]

## Dev Agent Record

### Agent Model Used

Claude Fable 5 (claude-fable-5) — dev-story orchestré inline.

### Debug Log References

- Tests ciblés : invoices_validate_vat 13/13 (5 nouveaux 19-4 a-e), credit_notes_repository 6/6 (+1 héritage net-zéro), migrations_upgrade_path 8/8 (compteur 42, fenêtre total-19).
- Frontend : svelte-check 0 erreur, lint-i18n PASS, unit verts, build OK. Clippy 0 warning.
- Gate complet : workspace serial 41 suites — 1 fausse alerte (34 fails = migration 42 non appliquée à la DB dev partagée, hors code ; `cargo sqlx migrate run` puis kesh-db lib 214/214). **E2E 11/11 RÉELS** (invoices + credit-notes + supplier-invoices), dont le test 19-4.
- **Réhabilitation spec invoices.spec.ts** (8 réparations de tests legacy cassés depuis ~mai, pré-existants — vérifiés sur main) : 3× `toHaveURL('/invoices')` (la création redirige vers le détail depuis 5.x P5), taux TVA `7.70` abrogé (Epic 11 vérifie en DB), prix catalogue 4 décimales, regex bouton PDF (aria-label a changé le nom accessible), flux « ouvrir depuis la liste » (on atterrit déjà sur le détail), assertions `toContainText` sur des `<input>` → `toHaveValue`, prérequis compte bancaire principal (helper `ensurePrimaryBankAccountViaApi` — le seed n'en a plus depuis v014-1), regex toast (clé FTL traduit le code, pas le message backend).

### Completion Notes List

- **T1** : migration `20260703000001_invoices_project.sql` (calque 19-3), idempotence-audit +1 ligne + stats 42, compteur upgrade_path 41→42 et fenêtre `total-18`→`total-19` (frontière historique 23 préservée).
- **T2** : `Invoice`/`NewInvoice`/`InvoiceUpdate.project_id` ; **5 SELECTs à colonnes explicites étendus** (FIND_INVOICE_SCOPED_SQL, delete, mark_as_paid-list, credit_notes lock, reconciliation INVOICE_COLUMNS — piège n°1 de la spec ×5 sites réels) ; create → validation helper 19-2 ; update → **pré-lecture non verrouillée + validation seulement si changement** (grandfathering, ordre companies→projects→invoice_row anti-ABBA) ; `is_no_op_change` compare project_id ; validate_invoice → **re-validation par SELECT simple SANS verrou** (on détient la ligne facture : prendre le sentinel créerait l'inversion ABBA avec create/update ; race archivage résiduelle = dette LOW-1 19-3) + stamping ; credit_note → héritage `invoice.project_id` sans re-check (DC3).
- **T3** : DTOs create/update/response + mapping handlers ; ~40 littéraux `NewInvoice`/`InvoiceUpdate` défautés `None` par script ; export CSV souveraineté invoices + colonne `project_id`.
- **T4** : sélecteur document-level arbre 2 niveaux dans `InvoiceForm` ($effect pattern VAT rates, échec toléré, option ad-hoc « Projet archivé », visible si tag existant — leçon 19-2 BH-L2), payloads create/update + reload conflit, page détail `projectLabel` (`listProjects(true)`, fallback `#id`).
- **T5** : E2E « crée une facture avec projet analytique » (POST intercepté ground-truth + libellé détail) ; CHANGELOG [Non publié].

### File List

- crates/kesh-db/migrations/20260703000001_invoices_project.sql (nouvelle)
- docs/migrations-idempotence-audit.md (+1 ligne, stats 42)
- crates/kesh-db/tests/migrations_upgrade_path.rs (42, total-19)
- crates/kesh-db/src/entities/invoice.rs (3 structs)
- crates/kesh-db/src/repositories/invoices.rs (SELECTs, create/update/validate_invoice, no-op, tests inline défautés)
- crates/kesh-db/src/repositories/credit_notes.rs (SELECT + héritage projet)
- crates/kesh-db/src/repositories/reconciliation.rs (INVOICE_COLUMNS)
- crates/kesh-api/src/routes/invoices.rs (DTOs + handlers)
- crates/kesh-api/src/exports/csv_tables.rs (CSV invoices + project_id)
- crates/kesh-reconciliation/src/matching.rs + tests divers (littéraux défautés None)
- crates/kesh-db/tests/invoices_validate_vat.rs (+5 tests 19-4 + helpers)
- crates/kesh-db/tests/credit_notes_repository.rs (+1 test héritage net-zéro)
- frontend/src/lib/features/invoices/invoices.types.ts (projectId)
- frontend/src/lib/components/invoices/InvoiceForm.svelte (sélecteur + payloads + reload)
- frontend/src/routes/(app)/invoices/[id]/+page.svelte (affichage projet)
- frontend/tests/e2e/invoices.spec.ts (+1 test)
- CHANGELOG.md ([Non publié] factures de vente)

## Change Log

- **Dev (2026-07-03)** : T1→T5 complets. Décisions au-delà de la spec, documentées : (a) re-validation au posting par SELECT simple sans verrou (éviter l'inversion ABBA sentinel↔ligne facture — la spec proposait le helper, incompatible avec le lock déjà détenu) ; (b) update = pré-lecture non verrouillée avant le FOR UPDATE facture (même pattern que 19-2 update) ; (c) 5 SELECTs explicites réels au lieu des 2 listés par la spec (reconciliation.rs INVOICE_COLUMNS + mark_as_paid list découverts au build/sweep).
