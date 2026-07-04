# Story 19.5 : Tagging analytique depuis la banque / réconciliation

Status: ready-for-dev

<!-- Rollout du pattern Epic 19 (dimension project_id sur journal_entry_lines +
     helper projects::validate_taggable_in_tx) sur le DERNIER flux de saisie non
     encore tagué : la réconciliation bancaire (Epic 8). Deux features :
     (A) affecter un projet quand une écriture est créée depuis une transaction
         bancaire (manual match + split + règle) ;
     (B) « projet par défaut » sur une règle d'affectation → appliqué à l'accept
         de la règle. -->

## Story

As a comptable/indépendant PME utilisant Kesh,
I want affecter un **projet analytique** lors de la réconciliation d'une transaction bancaire (rapprochement manuel, ventilation, ou règle automatique) et pouvoir déclarer un **projet par défaut sur une règle**,
so that les mouvements bancaires (loyers d'un immeuble en rénovation, encaissements d'un investissement) alimentent les rapports analytiques par projet au même titre que les factures et les écritures manuelles, sans re-saisie.

## Contexte & source

- **Epic 19** — design `_bmad-output/planning-artifacts/epic-19-analytique-projet-design.md` §5 story 19-5 : « Affectation d'un projet lors de la création d'écriture depuis une transaction bancaire ; option "projet par défaut" sur une règle d'affectation. » Dépend de 19-1 (socle, mergée) et idéalement après 19-2 (helper `validate_taggable_in_tx` + sémantique par-ligne, mergée #203).
- **Déjà en place** :
  - colonne `journal_entry_lines.project_id` (migration 19-1) et `NewJournalEntry.project_id` / `NewJournalEntryLine.project_id` (19-2/19-3) → **aucune migration sur les écritures**.
  - `journal_entries::create_in_tx` bind `line.project_id.or(new.project_id)` (`journal_entries.rs:210`) : la ligne prime, le document-level est le fallback ; validation per-ligne étape 0 (`:106-107`).
  - helper partagé `projects::validate_taggable_in_tx(tx, company_id, &[ids])` (`repositories/projects.rs:87-121`, sentinel companies **une prise** → `IN (...) FOR UPDATE`, Pattern 5) — réutilisé tel quel.
  - Les deux constructeurs d'écriture réconciliation **portent déjà `project_id: None` avec un TODO explicite « Story 19-5 »** : `manual.rs:105/111/117`, `split.rs:121/135/144`.
- **Ce que fait cette story** :
  - **Feature A — tag à l'accept** : plumber un `project_id` depuis les DTOs accept jusqu'aux constructeurs, avec la bonne granularité par flux (document-level pour manual/règle, par-ligne pour split).
  - **Feature B — projet par défaut sur règle** : nouvelle colonne `reconciliation_rules.default_project_id`, exposée en CRUD, résolue à l'accept d'une proposition `type=rule`.

## Décisions de granularité (fondations, lire avant les AC)

| Flux | Endpoint / builder | Écriture | Granularité tag 19-5 | Validation projet |
|------|--------------------|----------|----------------------|-------------------|
| **Invoice match** | `accept type=invoice` (`accept_one_invoice`) | **aucune** (réutilise `invoice.journal_entry_id`) | **hérité** du tag facture 19-4 — **rien à faire** | déjà faite à `validate_invoice` (19-4) |
| **Manual match** | `/manual` (`post_manual` → `build_journal_entry_for_counterparty`) | 2 lignes (banque + contrepartie), **mono-usage** | **document-level** (`NewJournalEntry.project_id`, propagé aux 2 lignes via `.or()`) | **explicite** avant `create_in_tx` (DC2 : `new.project_id` n'est PAS validé par le repo) |
| **Split** | `/split` **et** `accept type=split` (`accept_one_split` → `build_split_journal_entry`) | N+1 lignes, **multi-usage** | **par ligne de split** (chaque `SplitProposalLine`/`SplitLineInput.project_id`) ; la ligne banque reste non taguée | **automatique** par `create_in_tx` (validation per-ligne étape 0) |
| **Rule accept** | `accept type=rule` (`accept_one_rule` → `build_journal_entry_for_counterparty`) | 2 lignes, mono-usage | **document-level** = **projet par défaut de la règle** (Feature B), résolu serveur | **explicite** avant `create_in_tx` (re-validation à l'accept, cf. DC3) |

## Acceptance Criteria

### Feature B — migration & entité règle (kesh-db)

1. Migration `20260704000001_reconciliation_rules_default_project.sql` : `ALTER TABLE reconciliation_rules ADD COLUMN default_project_id BIGINT NULL` + FK `projects(id)` **ON DELETE RESTRICT** + `INDEX (default_project_id)` — calque de `20260703000001_invoices_project.sql` (19-4). **Non-breaking** (ADD COLUMN nullable) → **pas** de bump `kesh_version_min_required`. Ligne ajoutée à `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx`, stats 42→43).
2. Compteur `crates/kesh-db/tests/migrations_upgrade_path.rs` bumpé : `assert_eq!(total, 42)` → 43 (`:71`) ET fenêtre `total - 19` → `total - 20` (`:98`, ajuster commentaires `:84-95` si besoin). `admin_full_export_e2e` data_count : colonne (pas de table), **inchangé** — vérifier au run.
3. Entité `ReconciliationRule` (`entities/reconciliation_rule.rs:107-123`) porte `pub default_project_id: Option<i64>`. `NewReconciliationRule` (`:130-137`) et `UpdateReconciliationRule` (`:147-154`) portent `default_project_id` — pour le PATCH, `Option<Option<i64>>` **N'EST PAS** retenu (v0.1 : `Option<i64>` avec `Some(None)`-sémantique via un champ dédié serait sur-ingénierie) ; suivre le pattern des autres champs `UpdateReconciliationRule` (voir DC4 pour la sémantique « effacer le projet par défaut »).

### Feature B — repo & routes règle (kesh-db + kesh-api)

4. `reconciliation_rules::COLUMNS` (`repositories/reconciliation_rules.rs:27`) inclut `default_project_id` → tous les SELECT (`list_all_by_company:51`, `find_active_for_company:73`, `list_by_company_paginated:98`, `find_by_id_for_company:144`) le retournent (FromRow `ReconciliationRule`). **Piège** : si les SELECT listent les colonnes en dur au lieu de `COLUMNS`, les étendre — sinon FromRow panique au décodage.
5. `create_in_tx` (`:177`, INSERT `:184`) : **valide** `default_project_id` (si `Some`) via `super::projects::validate_taggable_in_tx(tx, company_id, &[pid])` avant l'INSERT, puis persiste la colonne. `update_in_tx` (`:245`) : idem validation si le champ change vers un `Some` (grandfathering du tag inchangé, leçon 19-2/19-4), UPDATE étendu, `version` bumpée.
6. DTOs `routes/reconciliation_rules.rs` : `CreateRuleRequest` (`:89-98`) + `UpdateRuleRequest` (`:104-115`) portent `#[serde(default)] default_project_id: Option<i64>` (camelCase `defaultProjectId`). `ReconciliationRuleResponse` (`:53-68`) expose `defaultProjectId`. Handlers `post_create` / `patch` passent le champ dans `NewReconciliationRule` / `UpdateReconciliationRule`. Erreurs : projet inconnu/cross-company → 404, archivé → 409 (mapping `DbError` existant, jamais 500).

### Feature A — tag à l'accept (kesh-reconciliation + kesh-api)

7. **Builder manual** `build_journal_entry_for_counterparty` (`kesh-reconciliation/src/manual.rs:67-121`) : nouveau paramètre `project_id: Option<i64>` (dernier param). Le `NewJournalEntry.project_id` (`:105`) prend cette valeur (remplace `None`) ; les 2 lignes gardent `project_id: None` (propagation via `.or()`). Doc `///` mise à jour (retirer le TODO 19-5).
8. **Builder split** `build_split_journal_entry` (`kesh-reconciliation/src/split.rs:90-148`) : le paramètre `splits: &[SplitDetail]` porte désormais un `project_id: Option<i64>` par split (étendre `SplitDetail`). Chaque ligne de contrepartie (`:131-136`) prend `project_id: split.project_id` ; la ligne banque (`:117-122`) et `NewJournalEntry.project_id` (`:145`) restent `None`. Doc `///` mise à jour.
9. **DTO accept split** : `SplitProposalLine` (`routes/reconciliation.rs:262-268`) et `SplitLineInput` (`:2647-2653`, standalone `/split`) portent `#[serde(default)] project_id: Option<i64>` (camelCase `projectId`). Le mapping vers `SplitDetail` (dans `accept_one_split:1240` **et** `post_split:2692`) reporte `project_id`. La validation per-ligne est **automatique** (`create_in_tx` étape 0).
10. **DTO manual** : `ManualMatchBody` (`routes/reconciliation.rs:2256-2264`, standalone `/manual`) porte `#[serde(default)] project_id: Option<i64>`. `post_manual` (`:2295`) : **valide** le projet (`projects::validate_taggable_in_tx`, si `Some`) dans la tx **avant** l'appel builder, puis passe `project_id` au builder. Erreur projet → mappée 4xx (jamais 500).
11. **Rule accept** `accept_one_rule` (`reconciliation.rs:1609`, appel builder `:1805`) : résout le projet = `rule.default_project_id` (Feature B). Si `Some`, le **valide** via `projects::validate_taggable_in_tx` dans la tx du savepoint (re-validation à l'accept — le projet a pu être archivé depuis la création de la règle, DC3), puis le passe au builder. Projet archivé → **`FailedProposal`** per-proposition (HTTP 200 batch, pattern `accept_batch` CLAUDE.md), **jamais** une `AppError` globale. `error_code` canonique `"PROJECT_ARCHIVED"` (ou constante équivalente déjà présente), `details: { "projectId": <id> }`.
12. **Invoice accept inchangé** : `accept_one_invoice` (`:927`) ne crée pas d'écriture → aucun champ projet sur la variante `Invoice` de `AcceptProposalInput` (`:205`). Le tag vient de la facture (19-4). Documenté, pas de code.

### Frontend

13. **Form règle** `rules/RuleFormModal.svelte` : sélecteur « Projet par défaut » arbre 2 niveaux (copie `supplier-invoices/+page.svelte:345-362` : racines + sous-projets `↳`, option « — Aucun » = `null`), `data-testid="rule-form-default-project"`, state `defaultProjectId` initialisé `rule?.defaultProjectId ?? null`, chargement `listProjects()` (actifs) via `$effect`, rendu seulement si `projects.length > 0` OU tag existant (option ad-hoc « Projet archivé » — leçon 19-2 BH-L2). Payloads create/patch portent `defaultProjectId`. Types `rules/rules.types.ts` : `RuleResponse.defaultProjectId`, `CreateRuleRequest`/`UpdateRuleRequest.defaultProjectId`.
14. **Manual match** `ManualMatchModal.svelte` : sélecteur projet document-level (même composant que 13), `data-testid="manual-match-project"`, state `projectId` (init `null` au reset `:62-71`), inclus dans le payload `/manual` (`:100`). Type `reconciliation.types.ts` `ManualMatchBody`/équivalent + `reconciliation.api.ts` : `projectId`.
15. **Split modal** `TransactionSplitModal.svelte` : sélecteur projet **par ligne de split** (colonne « Projet » dans le tableau des splits), `data-testid="split-line-project-{i}"`, `projectId` par ligne dans le payload (`/split` et batch accept). Types `reconciliation.types.ts` split line + api.
16. **Affichage** : le projet des écritures de réconciliation est visible via le détail de l'écriture (`journal-entries/[id]` — déjà fait en 19-2, colonne Projet par ligne). Aucune vue réconciliation supplémentaire requise v1 (les proposals ne détaillent pas les lignes d'écriture). Optionnel : badge « projet par défaut » sur la ligne de règle dans `RulesList.svelte` (LOW, non bloquant).

### Non-régression & intégration

17. **Écritures** : aucune migration sur `journal_entry_lines` (colonne 19-1). Le round-trip backup reste vert sans modif (sérialiseur export dynamique par colonnes ; `reconciliation_rules` déjà couverte — la nouvelle colonne est incluse automatiquement, vérifier `admin_backup_e2e` équivalence per-table).
18. Flux Epic 8 intacts : accept invoice/rule/split **sans** projet (tous `project_id: None`/absent) produisent exactement les mêmes écritures qu'avant (tests réconciliation existants verts). La signature builder change → **tous les call sites** (`accept_one_rule`, `accept_one_split`, `post_manual`, `post_split`, tests de `manual.rs`/`split.rs`) passent `None`/le nouveau champ.
19. **Export CSV souveraineté** : si `reconciliation_rules` figure dans `csv_tables.rs` (export souveraineté), ajouter `default_project_id` à la liste de colonnes (leçon 12-1/19-4 : une colonne taguable doit voyager dans l'export). Vérifier au sweep ; si la table n'y est pas, pas d'action.

### Tests / qualité

20. **Tests repo règle** (`crates/kesh-db/tests/` — nouveau `reconciliation_rules_repository.rs` ou extension existante ; helper `make_project` copié de `supplier_invoices_repository.rs:514-529`) : create règle avec `default_project_id` relu correctement ; create avec projet archivé → 409 (`IllegalStateTransition`) ; projet inconnu/cross-company → 404 (`NotFound`) ; update change le projet → validé ; update sans changement avec projet archivé → passe (grandfathering).
21. **Tests réconciliation** (`crates/kesh-api/tests/` réconciliation e2e/intégration existants) : (a) accept `type=rule` d'une règle avec `default_project_id` → **les 2 lignes** de l'écriture portent le projet (ground-truth via `journal_entry_lines`) ; (b) accept `type=split` avec `project_id` par ligne → chaque ligne de contrepartie porte son projet, ligne banque non taguée ; (c) `/manual` avec projet → 2 lignes taguées ; (d) accept `type=rule` dont le `default_project_id` a été archivé après création → `FailedProposal` `PROJECT_ARCHIVED` (HTTP 200, `accepted` vide, `failed[0]` renseigné), les autres proposals du batch non impactées ; (e) accept sans projet (règle sans défaut, split/manual sans projectId) → écritures identiques au comportement Epic 8 (non-régression). Tests unitaires builders `manual.rs`/`split.rs` : `project_id` propagé correctement.
22. **Frontend** : unit tests `rules.api`/`reconciliation.api` (sérialisation `defaultProjectId`/`projectId`), `RuleFormModal.test.ts`/`ManualMatchModal.test.ts`/`TransactionSplitModal.test.ts` étendus. E2E Playwright : créer une règle avec projet par défaut → réconcilier une transaction via cette règle → vérifier le tag (ground-truth API) ; scoper les sélecteurs `option` au conteneur (collision `getByRole('option')`, leçon 19-2/19-4).
23. Quality gate **Test Locally First** complet, exit-codes vérifiés (pas de pipe grep — `feedback_cargo_test_pipe_masks_exit`), **workspace serial** (kesh-db touché) : `cargo test --workspace -j1 -- --test-threads=1`. CHANGELOG [Non publié] : entrée réconciliation bancaire.

## Points de décision (DC)

- **DC1 — Granularité par flux** [FIGÉ] : manual + règle = document-level (mono-usage, `NewJournalEntry.project_id`) ; split = par-ligne (multi-usage, `SplitDetail.project_id`). Invoice = hérité (rien). Rationale : une ventilation sépare une transaction en plusieurs finalités → potentiellement plusieurs projets ; un rapprochement simple porte une seule finalité. Aligne D1 du design (document-level pour les documents, par-ligne pour les écritures multi-lignes). *(À confirmer Guy si un doute — voir §Questions.)*
- **DC2 — Validation document-level explicite** [FIGÉ] : `create_in_tx` **ne valide pas** `new.project_id` (per-ligne uniquement, DC2 de 19-2 — préserve pay/cancel-after-archive). Donc manual + règle valident le projet **eux-mêmes** via `validate_taggable_in_tx` avant `create_in_tx` (pattern 19-3/19-4). Split n'a rien à valider en plus (per-ligne = automatique).
- **DC3 — Re-validation du projet par défaut à l'accept** [FIGÉ] : le `default_project_id` d'une règle est validé à la **création/édition** de la règle ET **re-validé à l'accept** (le projet a pu être archivé entre-temps ; toute nouvelle entrée au grand livre doit être sur projet actif — sémantique 19-4 DC2 posting). Archivé à l'accept → `FailedProposal` per-proposition (l'utilisateur détague la règle, désarchive le projet, ou réconcilie manuellement). **Rejeté** : appliquer le tag archivé silencieusement (viole « ne pas taguer un projet clos ») ; ignorer le tag et réconcilier sans projet (silencieux, l'utilisateur croit son mouvement tagué).
- **DC4 — Effacer le projet par défaut d'une règle** [FIGÉ] : le PATCH règle utilise `Option<i64>` ; envoyer `defaultProjectId: null` efface le défaut, l'omettre (via `#[serde(default)]`) le laisse inchangé — mais comme `UpdateReconciliationRule` porte déjà des `Option<T>` avec sémantique « None = inchangé », clarifier : suivre le pattern des champs existants (`label`, `match_value`…). Si le pattern existant est « None = inchangé », alors effacer nécessite un sentinel ; **v0.1 : accepter que le projet par défaut ne soit pas effaçable via PATCH partiel** (on peut le réaffecter, l'archivage du projet le neutralise fonctionnellement) — documenter comme limitation LOW si le pattern repo l'impose. **Le dev tranche à la lecture du pattern `update_in_tx` existant** et documente le choix retenu dans le Change Log.
- **DC5 — Standalone vs batch** [FIGÉ] : les endpoints standalone `/manual` et `/split` **et** le batch `/accept` (`type=split`) partagent les builders → tous reçoivent le champ projet. `/accept type=rule` résout le projet de la règle (pas de champ client). Cohérence : aucun call site builder ne reste à `None` codé en dur (sauf la ligne banque et le document-level du split, intentionnels).

## Tasks / Subtasks

- [ ] **T1 — Migration + entité règle** (AC: 1-3) : SQL calqué 19-4, idempotence-audit, `migrations_upgrade_path` 42→43 + fenêtre ; `ReconciliationRule`/`New`/`Update` + `default_project_id`.
- [ ] **T2 — Repo règle** (AC: 4-5) : `COLUMNS` + SELECTs, `create_in_tx`/`update_in_tx` validation (helper 19-2) + grandfathering + persist ; tests repo (AC20).
- [ ] **T3 — Routes règle** (AC: 6) : DTOs create/patch/response `defaultProjectId`, mapping handlers, mapping erreurs 4xx.
- [ ] **T4 — Builders réconciliation** (AC: 7-8) : `build_journal_entry_for_counterparty` +param `project_id` ; `SplitDetail.project_id` + `build_split_journal_entry` ; MAJ tous les call sites + tests unitaires builders (AC18).
- [ ] **T5 — DTOs accept + handlers** (AC: 9-12) : `SplitProposalLine`/`SplitLineInput.project_id` + mapping `SplitDetail` (accept_one_split + post_split) ; `ManualMatchBody.project_id` + validation post_manual ; `accept_one_rule` résolution + re-validation + `FailedProposal PROJECT_ARCHIVED` ; invoice inchangé.
- [ ] **T6 — Frontend** (AC: 13-16) : sélecteur projet RuleFormModal (défaut) + ManualMatchModal (doc) + TransactionSplitModal (par ligne) ; types + api ; i18n fallback FR.
- [ ] **T7 — Non-régression + tests réconciliation** (AC: 17-22) : suites Epic 8 vertes, backup round-trip, export CSV si applicable ; tests réconciliation (a-e) ; frontend unit + E2E.
- [ ] **T8 — Gate** (AC: 23) : Test Locally First complet backend serial + frontend (check/lint/unit/build) + E2E, exit-codes vérifiés ; CHANGELOG.

## Dev Notes

### Ground-truth (cartographie Explore 2026-07-04)

**Écritures (déjà taguables — ne pas re-migrer)**
- `NewJournalEntry.project_id` `crates/kesh-db/src/entities/journal_entry.rs:177` ; `NewJournalEntryLine.project_id` `:190` ; persisté `JournalEntryLine.project_id` `:152`.
- `journal_entries::create_in_tx` `crates/kesh-db/src/repositories/journal_entries.rs:90` — validation per-ligne étape 0 `:106-107`, bind propagation `line.project_id.or(new.project_id)` `:210`. **Ne valide PAS `new.project_id`** (DC2).
- Helper partagé `projects::validate_taggable_in_tx` `crates/kesh-db/src/repositories/projects.rs:87-121` (sentinel `bank_accounts::acquire_company_sentinel_lock` `bank_accounts.rs`, `IN (...) FOR UPDATE`, dédup interne, no-op si vide). Erreurs `NotFound` / `IllegalStateTransition("le projet analytique est archivé")`.

**Builders réconciliation (portent déjà `project_id: None` + TODO 19-5)**
- `kesh-reconciliation/src/manual.rs:67-121` — `build_journal_entry_for_counterparty`, `NewJournalEntry.project_id: None` `:105`, lignes `:107-118`. Utilisé par `accept_one_rule` ET `post_manual`.
- `kesh-reconciliation/src/split.rs:90-148` — `build_split_journal_entry`, `SplitDetail` (struct à étendre), `NewJournalEntry.project_id: None` `:145`, ligne banque `:117-122`, lignes splits `:131-136`. Utilisé par `accept_one_split` ET `post_split`.

**Accept flow (kesh-api)**
- `crates/kesh-api/src/routes/reconciliation.rs` — batch `post_accept:540` → `accept_batch:803` (savepoints per-proposal) → `accept_one:855` (dispatch) → `accept_one_invoice:927` (pas de JE, `invoice.journal_entry_id:1061`) / `accept_one_split:1240` (builder `:1481`, create `:1488`) / `accept_one_rule:1609` (règle rechargée, `rule_matches:1760`, builder `:1805`, create `:1812`).
- DTOs : `AcceptBody:185-188`, `AcceptProposalInput:202-236` (enum tag=type : `Invoice:205`, `Split:210`, `Rule:230`), `SplitProposalLine:262-268`. Standalone : `ManualMatchBody:2256-2264` (`post_manual:2295`), `SplitBody:2655-2662` + `SplitLineInput:2647-2653` (`post_split:2692`). Réponses : `AcceptResponse{accepted,failed}:136-139`, `FailedProposal:152`.
- Pattern batch `FailedProposal` (CLAUDE.md §Pattern batch) : erreur per-proposition (projet archivé) → `failed[]` + HTTP 200, JAMAIS AppError globale. Champ identifiant = `bank_transaction_id`. `error_code` constante, `details` JSON.

**Règles (kesh-db + kesh-api)**
- Entité `crates/kesh-db/src/entities/reconciliation_rule.rs` — `ReconciliationRule:107-123`, `NewReconciliationRule:130-137`, `UpdateReconciliationRule:147-154`, enum `ReconciliationMatchType:30-47`.
- Table migration `crates/kesh-db/migrations/20260513000001_reconciliation_rules.sql:36-71` (générée `active_uniq`, FK `counterparty_account_id→accounts`). **Pas** de `project_id` aujourd'hui.
- Repo `crates/kesh-db/src/repositories/reconciliation_rules.rs` — `COLUMNS:27`, `list_all_by_company:51`, `find_active_for_company:73`, `list_by_company_paginated:98`, `find_by_id_for_company:144`, `create_in_tx:177` (INSERT `:184`), `update_in_tx:245`, `soft_delete_by_id_for_company:355`, `increment_applied_count_in_tx:407`.
- Routes `crates/kesh-api/src/routes/reconciliation_rules.rs` — `CreateRuleRequest:89-98`, `UpdateRuleRequest:104-115`, `ReconciliationRuleResponse:53-68`. Enregistrement `lib.rs:385-390,587-592`.
- Engine `kesh-reconciliation/src/rules.rs` — `rule_matches:46`, `first_matching_rule:85` (consommé `reconciliation.rs:483` pour les candidates).

**Migration counters**
- Dernière migration : `20260703000001_invoices_project.sql` (calque). `migrations_upgrade_path.rs:71` (`total, 42`) + `:98` (`total - 19`).

**Frontend**
- Réconciliation : `frontend/src/lib/features/reconciliation/` — `ReconciliationProposals.svelte` (batch accept `onAccept:122`, candidates rule/invoice/split), `ManualMatchModal.svelte` (counterparty selector `:149`, payload `:100`, reset `:62-71`), `TransactionSplitModal.svelte`, `reconciliation.api.ts`, `reconciliation.types.ts`.
- Règles : `frontend/src/lib/features/reconciliation/rules/` — `RuleFormModal.svelte` (counterparty `:24`, submit `:64-74`), `RulesList.svelte`, `rules.api.ts`, `rules.types.ts`. Pages `routes/(app)/reconciliation/+page.svelte` + `rules/+page.svelte`.
- Sélecteur projet modèle : `supplier-invoices/+page.svelte:345-362`. Feature projects : `frontend/src/lib/features/projects/` (`listProjects`).

### Conventions projet

- Multi-tenant scopé `company_id` (IDOR) ; DTOs camelCase (`serde rename_all`) ; `AppError` 4xx typée, **jamais 500** sur erreur métier ; erreur per-proposition batch → `FailedProposal` (jamais AppError globale) ; i18n fallback FR inline ; Pattern 5 verrouillage (sentinel companies avant projects) — le helper 19-2 le gère. Pas de `unreachable!()` dans les match (`tracing::error!` + `AppError::Internal` si variant manquant). Commit par étape BMAD, branche `story/19-5-tagging-banque-reconciliation` (créée).
- **Story = rollout mécanique** du pattern 19-1/19-2/19-4 (design §5) → revue file-by-file, pas de nouveau composant générique tant que le sélecteur projet reste dupliqué (déjà 4 usages : justifierait une extraction — mais hors scope 19-5, à noter en dette LOW si le reviewer insiste).

### Pièges connus (anti-régression)

1. **Signature builder changée sans MAJ des call sites** → build cassé. `build_journal_entry_for_counterparty` : `accept_one_rule` + `post_manual` + tests `manual.rs`. `build_split_journal_entry` / `SplitDetail` : `accept_one_split` + `post_split` + tests `split.rs`. Défauter `None` partout sauf les 2 flux tagués.
2. **Valider `new.project_id` document-level dans `create_in_tx`** → NON : le repo ne le valide pas (DC2). Manual/règle valident **avant** l'appel. Split n'a pas ce souci (per-ligne).
3. **Projet archivé du défaut de règle → AppError globale** → casse le batch entier. DC3 : `FailedProposal` per-proposition uniquement.
4. **SELECT règle à colonnes en dur** (si `COLUMNS` non utilisé) → FromRow `ReconciliationRule` panique dès qu'une règle est relue. Étendre tous les SELECT.
5. **Ordre de verrouillage** : `validate_taggable_in_tx` prend le sentinel companies. À l'accept, l'ordre global companies→projects→fiscal_years doit tenir : valider le projet **avant** `create_in_tx` (qui prend le lock fiscal_years). Cohérent avec 19-2/19-3.
6. **E2E** : scoper les `getByRole('option')` au conteneur du select projet (collision avec les `<option>` natifs et les autocompletes — leçon 19-2/19-4). `KESH_COOKIE_SECURE=false` obligatoire en local (cf. `docs/testing.md`).

### References

- [Source: _bmad-output/planning-artifacts/epic-19-analytique-projet-design.md §5 story 19-5, §2 D1]
- [Source: _bmad-output/implementation-artifacts/19-2-tagging-ecritures-manuelles.md — helper `validate_taggable_in_tx`, grandfathering, DC2 validation per-ligne, leçons E2E]
- [Source: _bmad-output/implementation-artifacts/19-4-tagging-factures-vente.md — pattern document-level + re-validation au posting + DC4 « règlement réconcilié taguable via 19-5 »]
- [Source: CLAUDE.md §Pattern batch — FailedProposal per-proposal ; §Migration breaking policy]
- [Source: docs/MULTI-TENANT-SCOPING-PATTERNS.md Pattern 5 — sentinel companies → projects]

## Questions pour Guy (non bloquantes — défauts pris)

1. **Split multi-projets (DC1)** : ok pour un projet **par ligne de ventilation** (une transaction éclatée peut concerner plusieurs projets) plutôt qu'un seul projet pour toute la ventilation ? *(Défaut retenu : par-ligne. Si tu préfères un seul projet par split, on simplifie en document-level.)*
2. **Projet par défaut archivé à l'accept (DC3)** : ok pour un **échec per-proposition** (`FailedProposal`, la réconciliation de cette transaction échoue jusqu'à correction) plutôt qu'appliquer/ignorer silencieusement ? *(Défaut retenu : FailedProposal, cohérent « ne pas taguer un projet clos ».)*
