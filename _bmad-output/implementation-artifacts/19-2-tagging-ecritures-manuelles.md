# Story 19.2 : Tagging analytique des écritures manuelles

Status: review

<!-- Rollout du pattern posé par 19-1 (dimension project_id sur journal_entry_lines)
     et 19-3 (validation projet + propagation). Ici : tag PAR LIGNE dans le
     formulaire d'écriture manuelle (décision D1 : les écritures manuelles
     portent le projet ligne par ligne, contrairement aux documents). -->

## Story

As a comptable/indépendant PME utilisant Kesh,
I want affecter un **projet analytique à chaque ligne** d'une écriture manuelle (saisie ou modification),
so that mes régularisations, amortissements et écritures diverses alimentent les rapports analytiques par projet (dépenses, rendement) au même titre que les factures.

## Contexte & source

- **Epic 19** — design `_bmad-output/planning-artifacts/epic-19-analytique-projet-design.md` (D1 : hybride document + **ligne pour les écritures manuelles**). Story 19-2 du découpage, dépend de 19-1 (done, PR #201).
- **Déjà en place** : table `projects` + repo + routes + feature frontend (19-1) ; colonne `journal_entry_lines.project_id` (migration 19-1, **aucune nouvelle migration nécessaire**) ; `NewJournalEntry.project_id` entry-level stampé sur toutes les lignes (19-3, flux documents) ; pattern de validation projet sous sentinel lock (19-3, `supplier_invoices.rs`).
- **Ce que fait cette story** : porter le tag jusqu'au **niveau ligne** — entité, repo (SELECT/INSERT/validation), DTOs API, formulaire de saisie et affichage détail.

## Acceptance Criteria

**kesh-core (draft comptable)**

1. `JournalEntryLineDraft` (`crates/kesh-core/src/accounting/balance.rs:72-81`) porte `pub project_id: Option<i64>` (dimension analytique opaque, documentée `///`). `validate()` ne la lit ni ne la modifie — les lignes du draft traversent la validation **verbatim** (aucun filtrage/réordonnancement, vérifié ground-truth). Sites de construction à mettre à jour : 2 handlers (`journal_entries.rs:422`, `:531`) + helper de test `balance.rs:208`.

**Entités + repo (kesh-db)**

2. `JournalEntryLine` (`entities/journal_entry.rs:144-151`) et `NewJournalEntryLine` (`:181-185`) portent `pub project_id: Option<i64>`.
3. `LINE_COLUMNS` (`repositories/journal_entries.rs:44`) inclut `project_id` → tous les SELECT de lignes le retournent. **Attention** : `list_all_lines_by_company` (`:917-932`) a ses colonnes **en dur** (`jel.id, jel.entry_id, ...`) — à mettre à jour aussi.
4. `create_in_tx` (boucle INSERT `:182-198`) bind **`line.project_id.or(new.project_id)`** : le tag par-ligne prime, fallback sur le tag entry-level (compat 19-3 — les flux documents construisent leurs lignes avec `project_id: None`, comportement inchangé).
5. `update` (boucle INSERT `:708-727`) insère aussi `project_id` par ligne. `is_no_op_change` (`:532-550`) compare `project_id` (sinon un changement de projet seul serait avalé comme no-op sans bump de version). `entry_snapshot_json` (audit, `:494-509`) sérialise `project_id` par ligne.
6. **Validation projet par-ligne** dans `create_in_tx` ET `update`, dans la même tx : collecter les `project_id` **distincts des lignes** (`line.project_id` uniquement — voir DC2), si non-vide → `acquire_company_sentinel_lock` (**une seule prise**, `bank_accounts.rs:588`) puis `SELECT id, archived FROM projects WHERE id IN (...) AND company_id = ? FOR UPDATE` ; tout id manquant → `DbError::NotFound`, tout `archived` → `DbError::IllegalStateTransition("le projet analytique est archivé")`. Ordre de verrouillage Pattern 5 respecté : sentinel `companies` AVANT `projects` (anti-deadlock ABBA, cf. `docs/MULTI-TENANT-SCOPING-PATTERNS.md` + `supplier_invoices.rs:274-304`).

**API (kesh-api)**

7. `CreateJournalEntryLineRequest` (`routes/journal_entries.rs:62-71`) porte `project_id: Option<i64>` (camelCase `projectId` via serde) — sert aussi au PUT (le DTO update réutilise les mêmes lignes). `JournalEntryLineResponse` (`:94-104` + `From` `:122-132`) expose `projectId`.
8. Handlers `create_journal_entry` (`:389-496`) et `update_journal_entry` (`:500-606`) : le `project_id` de chaque ligne request passe dans le `JournalEntryLineDraft`, traverse `validate()`, et est recopié dans `NewJournalEntryLine` (mapping `:468-476` et `:5xx` équivalent). `NewJournalEntry.project_id` reste `None` sur ces routes (le tag manuel est par-ligne). Erreurs mappées 4xx : projet inconnu/cross-company → 404, archivé → 4xx métier (jamais 500) — vérifier le mapping `DbError::IllegalStateTransition` existant.

**Frontend (formulaire + affichage)**

9. `JournalEntryForm.svelte` : colonne « Projet » par ligne — `<select>` arbre 2 niveaux **copié du pattern 19-3** (`supplier-invoices/+page.svelte:345-362` : racines + sous-projets indentés `↳`, option « — Aucun » = `null`), lié à `lines[i].projectId`, `data-testid` par ligne. Rendu seulement si `projects.length > 0` (aucun projet défini → formulaire identique à avant, zéro friction). `LineDraft` (`form-helpers.ts:9-13`) + hydratation édition (`lineResponseToDraft` `:33-39`, `fromJournalEntryResponse` `:45-50`) + payload `handleSubmit` (`JournalEntryForm.svelte:147-160`) portent `projectId`.
10. Page hôte `journal-entries/+page.svelte` charge `listProjects()` (actifs seuls, comme 19-3) et le passe en prop au formulaire (même pattern que `accounts`).
11. Détail `journal-entries/[id]/+page.svelte` : colonne « Projet » dans le tableau des lignes (`:101-125`), résolue via `Map<number, ProjectResponse>` chargée par `listProjects(true)` (**archivés inclus** — l'historique doit rester lisible), affichage `code — name`, vide si ligne non taguée. i18nMsg fallback FR inline (clés type `journal-entry-form-col-project`, `journal-entry-project-none`).
12. Types TS : `CreateJournalEntryLineRequest` (`journal-entries.types.ts:35-40`) et `JournalEntryLineResponse` (`:11-18`) portent `projectId`.

**Non-régression & intégration**

13. **Aucune migration** (colonne existe depuis 19-1) → rien à faire côté idempotence-audit / breaking policy / manifeste backup (le sérialiseur export est dynamique par colonnes ; `journal_entry_lines` déjà couverte). Le round-trip backup reste vert sans modification.
14. Flux 19-3 intact : création/pay/cancel facture fournisseur avec projet — la validation par-ligne ne s'applique **pas** au `project_id` entry-level, donc `pay_succeeds_when_project_archived_after_tagging` reste vert (DC2).

**Tests / qualité**

15. Tests repo (`journal_entries.rs` mod tests, helper `mk_entry:1052`) : création avec tags par-ligne mixtes (2 projets différents + 1 ligne sans projet) relue correctement ; projet archivé rejeté (`IllegalStateTransition`) ; projet inexistant et cross-company rejetés (`NotFound`) ; update qui change uniquement le projet d'une ligne → PAS no-op (version bumpée, tag persisté) ; fallback entry-level préservé (ligne None + `new.project_id` Some → ligne taguée). Calquer les helpers 19-3 (`supplier_invoices_repository.rs:514-522` `make_project`).
16. Tests route (mod tests inline `routes/journal_entries.rs`) : mapping erreurs projet → 4xx. Frontend : unit `form-helpers` (hydratation `projectId`). E2E Playwright optionnel (saisie écriture avec projet).
17. Quality gate **Test Locally First** complet, exit-codes vérifiés (pas de pipe grep — `feedback_cargo_test_pipe_masks_exit`), workspace serial si kesh-db touché : `cargo test --workspace -j1 -- --test-threads=1`.

## Points de décision (DC)

- **DC1 — Sémantique ligne vs entry-level** [FIGÉ] : à l'INSERT, `line.project_id.or(new.project_id)`. La ligne prime ; le stamping entry-level 19-3 reste le fallback. Aucun flux existant ne fournit les deux.
- **DC2 — Périmètre de validation** [FIGÉ — anti-régression] : la validation repo (AC6) porte **uniquement sur les `project_id` par-ligne explicites**, pas sur `new.project_id`. Rationale : les flux 19-3 (pay/cancel) stampent l'entry-level sur des écritures de règlement/annulation **après** un éventuel archivage du projet — c'est voulu (test `pay_succeeds_when_project_archived_after_tagging`). Valider `new.project_id` dans `create_in_tx` casserait ce comportement. Le flux document garde sa validation à la création du document (inchangée, `supplier_invoices.rs:274-304`).
- **DC3 — `project_id` dans le draft kesh-core** [FIGÉ] : option retenue = champ opaque sur `JournalEntryLineDraft` (traverse `validate()` intact). Rejeté : réassociation par index post-validate (fragile, dépend d'un invariant d'ordre implicite). kesh-core reste sans I/O (ARCH-1) — c'est une donnée, pas un accès.
- **DC4 — Affichage liste** [FIGÉ] : pas de badge projet dans la **liste** des écritures v1 (les lignes n'y sont pas détaillées) ; le détail suffit. Extension possible en 19-6 (drill-down rapports).

## Tasks / Subtasks

- [x] **T1 — kesh-core** (AC: 1) : `JournalEntryLineDraft.project_id` + doc + sites de construction (2 handlers + helper test).
- [x] **T2 — Entités + repo** (AC: 2-6) : entités ; `LINE_COLUMNS` + `list_all_lines_by_company` ; INSERT create/update avec `or()` ; `is_no_op_change` ; `entry_snapshot_json` ; helper de validation par-ligne (extraction possible en fn partagée du pattern `supplier_invoices.rs:274-304` — sentinel unique + `IN (...) FOR UPDATE`) ; tests repo.
- [x] **T3 — API** (AC: 7-8) : DTOs request/response + mapping handlers create/update ; tests mapping erreurs.
- [x] **T4 — Frontend** (AC: 9-12) : types TS + `LineDraft` + hydratation ; sélecteur par ligne dans `JournalEntryForm` ; chargement projects page hôte ; colonne Projet page détail ; i18n fallback FR ; unit test form-helpers.
- [x] **T5 — Non-régression** (AC: 13-14) : suite supplier_invoices verte (surtout `pay_succeeds_when_project_archived_after_tagging`) ; round-trip backup vert sans modif.
- [x] **T6 — Gate** (AC: 17) : Test Locally First complet backend (serial) + frontend (check/lint/unit/build) + E2E si formulaire touché, exit-codes vérifiés.

## Dev Notes

### Ground-truth (cartographie Explore 2026-07-03)

- **Entités** : `crates/kesh-db/src/entities/journal_entry.rs` — `JournalEntryLine:144`, `NewJournalEntry:166` (porte déjà `project_id:175`), `NewJournalEntryLine:181`.
- **Repo** : `crates/kesh-db/src/repositories/journal_entries.rs` — `LINE_COLUMNS:44`, `create_in_tx:90` (INSERT lignes `:182-198`, bind actuel `new.project_id`), `update:553` (INSERT `:708-727` **sans** project_id aujourd'hui), `is_no_op_change:532`, `entry_snapshot_json:494`, SELECTs lignes `:228,279,469,656,758`, `list_all_lines_by_company:917` (colonnes en dur).
- **Validation 19-3 à répliquer** : `crates/kesh-db/src/repositories/supplier_invoices.rs:274-304` (bloc complet avec commentaire Pattern 5). Sentinel : `bank_accounts::acquire_company_sentinel_lock` (`bank_accounts.rs:588`). Différence 19-2 : N projets possibles par écriture → **une** prise de sentinel puis un seul `SELECT ... IN (...) FOR UPDATE` sur les ids distincts.
- **Route** : `crates/kesh-api/src/routes/journal_entries.rs` — DTOs `:62-104`, handlers create `:389` / update `:500`, `project_id: None` actuels `:467` et `:572`. `validate()` (`kesh-core/src/accounting/balance.rs:144-197`) préserve les lignes verbatim.
- **Frontend** : `frontend/src/lib/features/journal-entries/` (`JournalEntryForm.svelte` lignes-table `:286-371`, `form-helpers.ts`, `journal-entries.types.ts`, `journal-entries.api.ts`) ; page hôte `frontend/src/routes/(app)/journal-entries/+page.svelte` ; détail `[id]/+page.svelte:101-125` ; sélecteur 19-3 à copier `supplier-invoices/+page.svelte:345-362` ; feature projects `frontend/src/lib/features/projects/` (`listProjects`).
- **Tests modèles** : `crates/kesh-db/tests/supplier_invoices_repository.rs:514-670` (make_project + rejets archivé/inconnu + pay-after-archive) ; tests repo journal inline `journal_entries.rs:947+` (`mk_entry:1052`).

### Conventions projet

- Multi-tenant : toute requête scopée `company_id` (IDOR). DTOs camelCase (`serde rename_all`). `AppError` 4xx typée, jamais 500 sur erreur métier. i18n fallback FR inline. Pattern 5 verrouillage : sentinel companies avant projects. Pas de `unreachable!()` dans les match. Commit par étape BMAD, branche `story/19-2-tagging-ecritures-manuelles` (créée).
- Story = **rollout mécanique** du pattern 19-1/19-3 (design Epic 19 §5) → revue file-by-file possible, pas de sur-ingénierie : pas de nouveau composant générique tant que 2 usages seulement (form + éventuel 19-5).

### Pièges connus (anti-régression)

1. **`is_no_op_change` sans `project_id`** → changement de projet seul silencieusement perdu (pas d'UPDATE). AC5/AC15 le verrouillent.
2. **Valider `new.project_id` dans le repo** → casse pay/cancel-after-archive 19-3. DC2 + AC14 le verrouillent.
3. **`list_all_lines_by_company` oublié** (colonnes en dur, n'utilise pas `LINE_COLUMNS`) → project_id absent des consommateurs aval (19-6 rapports lira ce chemin).
4. **Reprise multiple du sentinel** (une par projet) → inutile et bruyant ; une prise unique puis `IN (...)`.
5. **Frontend `value={null}` vs `undefined`** : le `<select>` 19-3 utilise `null` ; garder `projectId: number | null` de bout en bout (pas `undefined`) pour la sérialisation JSON explicite.

### References

- [Source: _bmad-output/planning-artifacts/epic-19-analytique-projet-design.md §2 D1, §5]
- [Source: _bmad-output/implementation-artifacts/19-1-projets-socle.md — pattern socle + DC3 archivage]
- [Source: docs/MULTI-TENANT-SCOPING-PATTERNS.md Pattern 5 — ordre sentinel companies → projects]
- [Source: commit deb918d (19-3) — validation projet + stamping + sélecteur frontend]

## Dev Agent Record

### Agent Model Used

Claude Fable 5 (claude-fable-5) — dev-story orchestré inline.

### Debug Log References

- Gate intermédiaire backend : `cargo build --workspace --all-targets` exit 0, clippy 0 warning.
- Tests repo : `cargo test -p kesh-db --lib repositories::journal_entries::tests -- --test-threads=1` → 29/29 verts (7 nouveaux 19-2).
- Tests route : 6/6 verts (3 nouveaux 19-2).
- Frontend : svelte-check 0 erreur, lint-i18n-ownership PASS (3 clés whitelistées, quirk #30), 346 tests unit verts (2 nouveaux), build OK.
- Gate complet : fmt exit 0, clippy 0 warning, `cargo test --workspace -j1 -- --test-threads=1` → 41 suites. 5 échecs initiaux = mes tests 19-2 non-idempotents inter-runs (codes projets résiduels, DB partagée) → `mk_project` rendu idempotent (DELETE avant INSERT), re-run kesh-db+kesh-api serial exit 0, 1236 tests.
- E2E Playwright RÉELS : 21/21 verts (journal-entries + projects + vat-purchase-assistant), 6 skipped pré-existants. Le test 19-2 vérifie le tag par-ligne ground-truth via l'API.

### Completion Notes List

- **T1** : `JournalEntryLineDraft.project_id` opaque (validate() intact, lignes verbatim) ; 83 sites `NewJournalEntryLine`/draft défautés `None` par script (comme 19-3 pour `NewJournalEntry`).
- **T2** : `LINE_COLUMNS` + `list_all_lines_by_company` (colonnes en dur) + INSERT create/update `line.project_id.or(new.project_id)` + `is_no_op_change` compare project_id + snapshot audit `projectId`. **Helper partagé `projects::validate_taggable_in_tx`** (sentinel unique + `IN (...) FOR UPDATE`, Pattern 5) — le bloc inline 19-3 de `supplier_invoices.rs` refactoré dessus (DRY).
- **Ordre de verrouillage** : validation en étape 0 AVANT le lock fiscal_years/entry (create_in_tx ET update) — le flux fournisseur 19-3 prend companies→projects→fiscal_years ; valider après aurait créé une inversion ABBA inter-flux.
- **Grandfathering (update)** : les projets déjà tagués sur l'écriture sont exemptés de la validation (SELECT prior scopé company anti-IDOR) — sinon archiver un projet rendrait toute écriture historique non-éditable. Nouveau projet archivé → refusé. Testé (a)+(b).
- **DC2 respecté** : `new.project_id` (document-level) jamais re-validé dans le repo → `pay_succeeds_when_project_archived_after_tagging` (19-3) reste vert.
- **T4** : sélecteur arbre 2 niveaux par ligne (pattern 19-3, `data-testid journal-entry-line-project-{i}`), colonne conditionnelle `projects.length > 0`, option ad-hoc « Projet archivé » pour le round-trip édition d'un tag historique, colspan dynamiques ; page hôte charge `listProjects()` (échec toléré) ; détail : colonne Projet conditionnelle (`listProjects(true)` pour lire les archivés) + colspan tfoot.
- **CHANGELOG** [Non publié] : entrée écritures manuelles.
- E2E : +1 test tagging nominal (création projet API + select ligne 1 + ground-truth API lines[0].projectId). 3 fixes de spec induits : (a) tous les `getByRole('option')` nus scopés au `listbox` de l'autocomplete (les `<option>` natifs du select projet matchent role=option dès qu'un projet existe) ; (b) test « liste vide » one-shot `isVisible()` → `locator.or()` auto-wait ; (c) test « suppression » matching exact (l'entrée « MODIFIÉ » du test d'édition matchait la regex substring — séquence cassée pré-existante révélée par le run complet).
- Env E2E local : `KESH_COOKIE_SECURE=false` OBLIGATOIRE (cookies Secure de 10-5 non envoyés par le request context Playwright sur http://127.0.0.1 → 401 systématiques) — documenté dans docs/testing.md.

### File List

- crates/kesh-core/src/accounting/balance.rs (JournalEntryLineDraft.project_id + helper test)
- crates/kesh-db/src/entities/journal_entry.rs (JournalEntryLine + NewJournalEntryLine.project_id)
- crates/kesh-db/src/repositories/projects.rs (helper validate_taggable_in_tx)
- crates/kesh-db/src/repositories/journal_entries.rs (LINE_COLUMNS, validation étape 0, INSERT or(), is_no_op_change, snapshot, grandfathering update, +7 tests)
- crates/kesh-db/src/repositories/supplier_invoices.rs (refactor DRY → helper)
- crates/kesh-db/src/repositories/credit_notes.rs / invoices.rs (sites défautés None)
- crates/kesh-reconciliation/src/manual.rs / split.rs (sites défautés None)
- crates/kesh-api/src/routes/journal_entries.rs (DTOs projectId, mapping handlers, +3 tests)
- crates/kesh-api/src/exports/csv_tables.rs (helper test)
- crates/kesh-api/tests/{exports_global,reports,reports_export}_e2e.rs (sites défautés None)
- crates/kesh-db/tests/{kf005_fulltext_index_e2e,report_aggregates}.rs (sites défautés None)
- crates/kesh-report/tests/{vat_report_reconciliation,vat_report_recoverable}.rs (sites défautés None)
- frontend/src/lib/features/journal-entries/journal-entries.types.ts (projectId types)
- frontend/src/lib/features/journal-entries/form-helpers.ts (LineDraft.projectId + hydratation)
- frontend/src/lib/features/journal-entries/form-helpers.test.ts (+2 tests projectId)
- frontend/src/lib/features/journal-entries/vat-purchase.ts / vat-purchase.test.ts (littéraux défautés)
- frontend/src/lib/features/journal-entries/JournalEntryForm.svelte (colonne + sélecteur par ligne)
- frontend/src/routes/(app)/journal-entries/+page.svelte (chargement projects + props)
- frontend/src/routes/(app)/journal-entries/[id]/+page.svelte (colonne Projet détail)
- frontend/tests/e2e/journal-entries.spec.ts (+1 test tagging, sélecteurs listbox-scoped, 2 tests robustifiés)
- docs/testing.md (prérequis KESH_COOKIE_SECURE=false)
- frontend/scripts/lint-i18n-ownership.js (3 clés whitelistées #30)
- CHANGELOG.md ([Non publié] entrée 19-2)

## Change Log

- **Dev (2026-07-03)** : T1→T6 complets (kesh-core draft opaque + repo validation Pattern 5 étape 0 + grandfathering update + API DTOs + frontend sélecteur par ligne/détail). Décisions au-delà de la spec, documentées : grandfathering des tags archivés pré-existants à l'update (sinon écritures historiques non-éditables) + validation AVANT le lock fiscal_years (ordre global companies→projects→fiscal_years, anti-ABBA inter-flux avec 19-3).
