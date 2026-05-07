# Story 8-5b: Moteur de règles d'affectation

Status: backlog

<!-- Issue de scission de Story 8-5 (`8-5-reconciliation-manuelle-regles-affectation.md`) le 2026-05-07,
     RE-SPLIT 8-5a effectif 2026-05-07 post-Pass-3 validate Opus 4.7 :

     Path-dépendance mise à jour 2026-05-07 :
     8-5b reste backlog jusqu'à **8-5a-bis** (split + breaking change /accept) `done`/merged.
     Path-dépendance détaillée :
     - 8-5a-zero (foundation column `bank_account.journal_account_id`) → fournit la base pour
       résoudre serveur-side le ledger account banque dans le flow `accept-with-rule`.
     - 8-5a-base (helper `manual::build_journal_entry_for_counterparty`) → réutilisé tel quel
       par 8-5b pour le flow `accept-with-rule` (création d'une journal_entry à 2 lignes à la
       volée à partir d'une rule).
     - **8-5a-bis (breaking POST /accept type='invoice'/'split' obligatoire)** → 8-5b ajoute
       `type='rule'` à l'enum `AcceptType` livré par 8-5a-bis. Le breaking change Q2 est porté
       par 8-5a-bis (et non par 8-5b) pour regrouper la migration des 15 sites POST /accept
       8-4 dans une seule story.

     Voir 8-5-reconciliation-manuelle-regles-affectation.md (status `archived-split`)
     pour les décisions de conception détaillées (§rules-schema, §rule-application, §accept-with-rule-flow).
     Voir 8-5a-reconciliation-manuelle-split.md (status `archived-split-bis`) pour le contexte
     du re-split 8-5a → 8-5a-zero/8-5a-base/8-5a-bis. -->

## Story

As a **fiduciaire ou utilisateur Kesh récurrent (Lisa, comptable d'une PME avec 8+ clients)**,
I want **créer et gérer des règles d'affectation automatique (counterparty contains, IBAN exact, reference contains) qui sont appliquées aux transactions bancaires `pending` lors de l'import et lors de l'accept, pour que les transactions récurrentes (Swisscom, salaires, cotisations sociales) soient pré-imputées sans saisie manuelle**,
so that **mon taux d'auto-affectation atteigne 80% au fil des mois (cf. UX scenario PRD §164) et que je passe moins de temps sur les transactions répétitives**.

### Contexte

**Story 8-5b = seconde moitié de la story unifiée 8-5**, scindée pré-`bmad-create-story validate` pour respecter la règle de splitting CLAUDE.md (> 5 modules + 3 features distinctes). Voir [`8-5-reconciliation-manuelle-regles-affectation.md`](8-5-reconciliation-manuelle-regles-affectation.md) (status `archived-split`) pour la spec d'origine — toutes les **décisions de conception** (§rules-schema, §rule-application, §accept-with-rule-flow) y sont documentées en détail et restent valides pour 8-5b avec les amendements Q3/Q4a/Q4b/Q5 listés ci-dessous.

**Dépendance bloquante (mise à jour 2026-05-07 post-re-split 8-5a)** : Story **8-5a-bis** (`8-5a-bis-split-breaking-accept`) doit être stable (review-closed avec 0 findings > LOW, mergée sur `main`) avant que 8-5b ne soit `ready-for-dev`. 8-5b consomme :
- `kesh_reconciliation::manual::build_journal_entry_for_counterparty` (helper public 8-5a-base — utilisé par flow `accept-with-rule` step 8 §accept-with-rule-flow)
- `kesh_db::repositories::reconciliation::find_strictly_pending_by_id_for_account` (helper 8-5a-base, partagé par /manual + /split + flow rule)
- `bank_account.journal_account_id` column (foundation 8-5a-zero) — résolu serveur-side pour la ligne banque du flow `accept-with-rule`
- **Breaking change `POST /accept` discriminator** (8-5a-bis) avec enum `AcceptType { Invoice, Split }` — 8-5b ajoute `type='rule'` à cet enum
- `fiscal_years::find_open_covering_date` (vérifié 8-5a-base T4)

**Pourquoi 8-5b après 8-5a-bis (post-re-split 2026-05-07) :** la valeur utilisateur du moteur de règles est conditionnée par l'existence du helper `manual::build_journal_entry_for_counterparty` (pour appliquer une rule = créer une journal_entry à la volée comme un manual-match) — livré par 8-5a-base et stabilisé par 8-5a-bis (qui ajoute `type='split'` au discriminator `AcceptType`). Sans 8-5a-bis, 8-5b devrait soit dupliquer ce code (anti-DRY), soit refaire le breaking POST /accept en parallèle (risque de conflit Cargo / Vec ordre tagged enum). Path dep stable confirmée par le re-split 8-5a → 8-5a-zero/base/bis.

**8-5b livre la valeur utilisateur d'optimisation :**
- **FR47** — moteur de règles persistées (CRUD + application en GET /proposals + POST /accept)
- **Soft-delete des rules** (Q3 décision Guy 2026-05-07) : préserve l'audit historique `reconciliation_rule.applied`.

**8-5b ne livre PAS** :
- FR45 manual match (8-5a-base, post-re-split)
- FR48 transaction split (8-5a-bis, post-re-split)
- Foundation `bank_account.journal_account_id` (8-5a-zero, post-re-split)
- Breaking change `POST /accept` discriminator (8-5a-bis, post-re-split — 8-5b ajoute juste `type='rule'`)
- **FR46 suggestion ML automatique post-manual-match** (reportée v0.2 / Story 8-5c potentielle ou Epic 11+ — décision Guy Q5 2026-05-07)
- Algorithme heuristique déterministe `suggest_rule` (la fonction décrite §rule-suggestion de la spec d'origine est **caduque**, voir Q5)
- Endpoint `POST /reconciliation/rules/suggest` (jamais livré)

**Status sprint :** `8-5b-reconciliation-rules-engine: backlog` au moment de la création (2026-05-07). Transition vers `ready-for-dev` après que **8-5a-bis** (post-re-split 2026-05-07) ait clos son cycle review (0 findings > LOW + merged main) — qui implique aussi que 8-5a-zero et 8-5a-base soient déjà mergées (séquence stricte).

**Pré-requis closed (au moment du démarrage 8-5b) :**
- ✅ Story **8-5a-zero** — column `bank_account.journal_account_id` + UI configuration (foundation).
- ✅ Story **8-5a-base** — helper public `kesh-reconciliation::manual::build_journal_entry_for_counterparty` + helper repo `find_strictly_pending_by_id_for_account` + variants AppError (`BankAccountNotConfigured`, `ReconciliationFiscalYearClosed`, `ReconciliationTransactionNotPending`).
- ✅ Story **8-5a-bis** — helper `kesh-reconciliation::split::*` + breaking change `POST /accept` discriminator `AcceptType { Invoice, Split }` (8-5b ajoute `Rule`) + migration des 15 sites POST /accept dans `reconciliation_e2e.rs`.
- ✅ Story 8-4 — `kesh-reconciliation` crate base + `with_account_lock` + audit log helpers.
- ✅ Story 6-2 — multi-tenant scoping pattern KF-002 Pattern 1.

**Crate cible** : extension de `kesh-reconciliation` avec 1 nouveau module `rules` (engine matching counterparty/IBAN/reference + sélection priorité). Le module `manual` existe déjà (livré 8-5a-base post-re-split) et est réutilisé tel quel par le flow `accept-with-rule`. Le module `split` existe déjà (livré 8-5a-bis post-re-split) mais n'est pas réutilisé par 8-5b (pas de rule type=split).

### Scope verrouillé — ce qui est livré par 8-5b

1. **Repository `reconciliation_rules` (FR47)** — nouvelle table `reconciliation_rules { id, company_id, label, match_type ENUM('counterparty_contains','counterparty_exact','reference_contains','iban_exact'), match_value VARCHAR(255), counterparty_account_id, priority INT, active BOOLEAN, applied_count BIGINT, last_applied_at, version, created_at, updated_at }`. Helpers Executor générique : `find_active_for_company`, `list_by_company_paginated`, `find_by_id_for_company`, `create_in_tx`, `update_in_tx`, `soft_delete_by_id_for_company`, `increment_applied_count_in_tx`. Cf. spec d'origine §rules-schema **avec amendement Q3 (soft-delete)**.

2. **Routes API CRUD règles (FR47)** :
   - `GET /api/v1/reconciliation/rules?active=true&page=1&perPage=50` — liste paginée toutes règles tenant (sub-router `authenticated_routes` — tous rôles authentifiés peuvent lire).
   - `GET /api/v1/reconciliation/rules/{id}` — détail (authenticated).
   - `POST /api/v1/reconciliation/rules` — crée (Comptable+).
   - `PATCH /api/v1/reconciliation/rules/{id}` — update partiel (Comptable+, optimistic lock `version`, peut réactiver une rule via `active=true`).
   - `DELETE /api/v1/reconciliation/rules/{id}` — **soft-delete** via `UPDATE active=false, updated_at=NOW(3), version=version+1` (Q3 décision Guy 2026-05-07 — pas de DELETE physique v0.1, conserve l'audit historique `reconciliation_rule.applied` avec FK `rule_id` toujours valide).

3. **Application des règles à l'import (`GET /proposals` extension)** — extension de `GET /api/v1/reconciliation/proposals` (héritée 8-4 + breaking change 8-5a) : pour chaque tx pending sans candidate score≥0.5, appliquer les `reconciliation_rules` actives de la company **par ordre de priorité strict** (plus petit `priority` = plus prioritaire ; égalité départagée par `id ASC`). Premier match = candidate top-1 avec discriminator `type: 'rule'`. La response inclut `appliedRule: { id, label, matchType }`. **Pas d'auto-acceptation v0.1** : l'utilisateur valide explicitement (cohérent L18 héritée 8-4).

4. **Application des règles à l'acceptation (`POST /accept` extension type='rule')** — extension de `POST /api/v1/reconciliation/accept` (déjà breaking 8-5a avec discriminator obligatoire) : ajout du type `rule` :
   ```json
   { "type": "rule", "bankTransactionId": 43, "ruleId": 7, "counterpartyAccountId": 6510 }
   ```
   Flow : (a) re-vérifier match côté serveur (idempotence anti-race), (b) appeler `manual::build_journal_entry_for_counterparty` (helper 8-5a réutilisé) pour construire la journal_entry à 2 lignes, (c) `journal_entries::create_in_tx`, (d) UPDATE `bank_transactions.status='reconciled'`, (e) UPDATE `reconciliation_rules.applied_count += 1, last_applied_at = NOW(3), version+1` (optimistic lock sur `rules.version`), (f) audit log dual `reconciliation.accepted` (avec `rule_id` dans details_json) + `reconciliation_rule.applied`.

5. **Audit log 4 nouvelles actions (Q4b)** :
   - `reconciliation_rule.created` — émis sur `POST /rules`
   - `reconciliation_rule.updated` — émis sur `PATCH /rules/{id}` (avec `details.before` / `details.after` champs modifiés)
   - `reconciliation_rule.deleted` — émis sur `DELETE /rules/{id}` (avec `details.soft_delete: true`)
   - `reconciliation_rule.applied` — émis sur `POST /accept type='rule'` (avec `details.applied_count_after`)

6. **Frontend extensions** :
   - Page `/reconciliation/rules` : CRUD complet règles (liste + form modal create/edit + bouton « Désactiver »/« Réactiver » via PATCH active=false/true + bouton « Supprimer » via DELETE soft).
   - Composant `RulesList.svelte` (table + actions par ligne).
   - Composant `RuleFormModal.svelte` (form modal create/edit, valide `label` non-vide, `matchValue` non-vide, `counterpartyAccountId` sélectionné).
   - Extension de `ReconciliationProposals.svelte` (héritée 8-4/8-5a) : pour candidate `type: 'rule'`, afficher différemment (badge bleu « Règle » + label règle + nom du compte de contrepartie).
   - Migration de `acceptProposal` 8-5a (`type: 'invoice'`) : ajouter le cas `type: 'rule'` qui envoie `{ type: 'rule', bankTransactionId, ruleId, counterpartyAccountId }`.

7. **i18n** — ~15 nouvelles clés (`reconciliation-rules-*` × 12, `reconciliation-rule-applied-*` × 3) × 4 locales fr/de/it/en-CH. **Pas** les clés `reconciliation-suggestion-*` (caduques Q5).

8. **Tests** — Unit `kesh-reconciliation::rules` (≥ 6 cas matchType + priorité + skip archived account), Integration `kesh-db::reconciliation_rules` (≥ 6 sqlx multi-tenant + soft-delete + UNIQUE partiel), E2E HTTP `kesh-api` (≥ 14 tests : 9 rules CRUD + 5 application accept-with-rule), Vitest (≥ 4), Playwright (≥ 1 actif + 1 a11y).

9. **Sync** sprint-status + audit log 4 nouvelles actions discriminantes (cf. point 5).

**HORS scope 8-5b (→ v0.2 ou jamais) :**

- **Suggestion ML automatique post-manual-match (FR46 originale)** — décision Guy Q5 2026-05-07 : reportée v0.2 (Story 8-5c potentielle ou Epic 11+). 8-5b ne livre pas l'algorithme déterministe `suggest_rule` ni l'endpoint `POST /reconciliation/rules/suggest`.
- **Auto-acceptation des règles à fort score** (e.g. seuil par tenant `auto_accept_threshold = 0.95`) — reporté v0.2 (couplé à L18 héritée 8-4, à traiter conjointement avec configurabilité utilisateur).
- **Règles avec regex** (au-delà des 4 `match_type` v0.1 : `counterparty_contains`, `counterparty_exact`, `reference_contains`, `iban_exact`) — reporté v0.2 si demande utilisateur.
- **Application batch des règles à un dataset historique** (« re-process toutes les tx pending avec les règles actuelles ») — la route `POST /reconciliation/apply-rules` n'est pas livrée v0.1 ; les règles s'appliquent uniquement sur GET /proposals à la volée. Reporté v0.2.
- **Multi-currency rules** (règles par devise) — reporté Story 11. Cohérent avec L38 héritée 8-4.
- **Préview des règles avant CRUD** (montrer combien de tx pending matcheront la nouvelle règle avant de la créer) — reporté v0.2.
- **Export CSV des rules pour backup/migration** — reporté v0.2 (cf. L49 spec d'origine).

### Décisions de conception (rappel — voir spec d'origine pour le détail)

Toutes les décisions §rules-schema, §rule-application, §accept-with-rule-flow, §audit-log-actions de [`8-5-reconciliation-manuelle-regles-affectation.md`](8-5-reconciliation-manuelle-regles-affectation.md) §127-405 s'appliquent telles quelles à 8-5b, **avec les amendements suivants verrouillés par décisions Guy 2026-05-07** :

#### Q3 — Soft-delete pour `reconciliation_rules` (impact §rules-schema + §api-routes)

**Decision** : `DELETE /api/v1/reconciliation/rules/{id}` ne supprime PAS la ligne — il fait `UPDATE reconciliation_rules SET active=false, updated_at=NOW(3), version=version+1 WHERE id=? AND company_id=?`. Justification :

- Préserve l'audit historique `reconciliation_rule.applied` (FK `rule_id` reste valide pour les `applied_count` antérieurs).
- Reactivation possible via `PATCH /rules/{id}` avec `{ active: true }` (sans recréer la rule).
- Aligne avec le pattern de soft-delete pour `accounts.active` (Story 3-1) et `bank_profiles.active` (Story 8-2).

**Schéma** : la spec d'origine §rules-schema définit déjà `active BOOLEAN NOT NULL DEFAULT TRUE`, donc le soft-delete est natif. **Mais** la contrainte UNIQUE `uq_reconciliation_rules_match (company_id, match_type, match_value)` doit être **partielle** sur `WHERE active=true` pour permettre :
- Désactiver une rule `(counterparty_contains, "Swisscom")` puis recréer une rule active avec mêmes match (cas reactivation après archivage).
- Sans le partial UNIQUE, la création échouerait avec `409 RECONCILIATION_RULE_DUPLICATE` même si l'ancienne rule est désactivée.

**MariaDB n'a pas de UNIQUE partiel natif** (contrairement à PostgreSQL `WHERE` clause). Workarounds :

- **Option A (recommandée) — colonne synthétique `active_uniq`** : ajout d'une colonne calculée `active_uniq VARCHAR(255) GENERATED ALWAYS AS (IF(active, match_value, NULL)) VIRTUAL` + UNIQUE `(company_id, match_type, active_uniq)`. NULL n'entre pas dans UNIQUE en MariaDB. Lisible, déclaratif.
- **Option B — trigger SQL** : `BEFORE INSERT/UPDATE` qui vérifie l'absence de rule active avec mêmes `(company_id, match_type, match_value)` et lève `SIGNAL SQLSTATE '45000' MESSAGE_TEXT='RECONCILIATION_RULE_DUPLICATE'`. Plus complexe, moins lisible.
- **Option C — check applicatif strict** : pas de UNIQUE en DB, le repo `create_in_tx` fait un `SELECT ... WHERE active=true` avant INSERT. Risque de race INSERT-INSERT (ramené à du `409` via SQL error 1062 sur la UNIQUE non-partial — mais cette UNIQUE non-partial bloquerait aussi reactivation, donc inutile).

**Choix retenu pour v0.1 8-5b** : **Option A (colonne synthétique `active_uniq`)**. À implémenter en T1 migration. Le commentaire SQL doit mentionner explicitement le contournement Q3 et référer à cette spec.

#### Q4b — Actions audit distinctes pour CRUD rule

`reconciliation_rule.created` / `.updated` / `.deleted` / `.applied` (4 actions distinctes). Pas de pattern modifiers Vec, pas de réutilisation d'une action générique `reconciliation_rule.mutated`. Cohérent avec la décision Guy Q4a (actions distinctes pour match events).

#### Q5 — Pas de suggestion ML / pas d'algorithme `suggest_rule`

Le user crée ses rules **manuellement** via le CRUD `POST /reconciliation/rules`. Pas d'endpoint `POST /reconciliation/rules/suggest`, pas d'objet `ruleSuggestion` dans aucune response. La fonction `suggest_rule` décrite §rule-suggestion de la spec d'origine est **caduque** et n'est pas implémentée en 8-5b.

L'utilisateur observe les transactions pending dans `/reconciliation`, identifie un pattern récurrent (e.g. « Swisscom AG » apparaît 3 fois ce mois), et crée la rule manuellement via `/reconciliation/rules` UI. Pas d'aide automatique v0.1.

**Reporté v0.2** : suggestion ML basée sur l'historique des accepts (« cette tx ressemble à 8 autres déjà acceptées vers le compte 6510 — créer une règle ? ») potentiellement traitée via Story 8-5c v0.2 ou Epic 11+ avec un vrai modèle d'embeddings.

#### §rule-application valide tel quel

Steps 1-3 + position dans response avec discriminator `type: 'rule'` — cohérent avec breaking change 8-5a (discriminator `type` obligatoire dans response candidates).

#### §accept-with-rule-flow valide tel quel

Steps 2-14 cohérents. Le helper `manual::build_journal_entry_for_counterparty` étape 8 est livré par 8-5a. Le double audit step 12+13 (`reconciliation.accepted` + `reconciliation_rule.applied`) reflète la décision Q4b.

#### §error-precedence-order — codes 8-5b uniquement

| # | Erreur | HTTP | Code |
|---|---|---|---|
| 12 | Rule not found / deactivated | 404 | `RECONCILIATION_RULE_NOT_FOUND` |
| 13 | Rule no longer matches (race entre GET /proposals et POST /accept) | 409 (failed[]) | `RECONCILIATION_RULE_NO_LONGER_MATCHES` |
| 14 | Rule UNIQUE constraint violation (match_type+match_value déjà existant active) | 409 | `RECONCILIATION_RULE_DUPLICATE` |
| 15 | Rule mismatch (proposal.counterpartyAccountId ≠ rule.counterparty_account_id) | 400 | `RECONCILIATION_RULE_MISMATCH` |

(Codes #9-#11 du tableau étendu sont 8-5a.)

## Acceptance Criteria

Numérotation héritée de la spec 8-5 d'origine pour traçabilité. ACs #101-#124 (~24 ACs sur rules CRUD + application + accept-with-rule + UI rules page). Les ACs #75-#100 sont du ressort de 8-5a.

### Règles d'affectation CRUD (FR47)

101. **(FR47 — POST /rules happy)** Given user Comptable, body `{ label: 'Swisscom AG → 6510', matchType: 'counterparty_contains', matchValue: 'Swisscom', counterpartyAccountId: 6510, priority: 100 }`, When POST `/api/v1/reconciliation/rules`, Then `201 Created` + body `{ id, ..., active: true, appliedCount: 0, lastAppliedAt: null }` + audit `reconciliation_rule.created`. *Test E2E HTTP : `rule_create_returns_201_with_audit_log`.*

102. **(FR47 — POST /rules duplicate UNIQUE active)** Given une rule **active** existante `(company_id, 'counterparty_contains', 'Swisscom')`, When POST avec mêmes match_type+match_value, Then `409 RECONCILIATION_RULE_DUPLICATE`. *Test E2E HTTP : `rule_create_rejects_duplicate_match_when_active`.*

103. **(FR47 — Q3 reactivation après soft-delete)** Given une rule `(counterparty_contains, 'Swisscom')` soft-deleted (`active=false`), When POST avec mêmes match_type+match_value, Then `201 Created` (le UNIQUE partiel `WHERE active=true` permet la création — la rule désactivée et la nouvelle rule active coexistent). *Test E2E HTTP : `rule_create_succeeds_when_existing_rule_is_inactive`.*

104. **(FR47 — POST /rules archived counterparty_account)** Given `counterpartyAccountId` pointant sur compte archivé, When POST, Then `404 ACCOUNT_NOT_FOUND`. *Test E2E HTTP : `rule_create_rejects_archived_account`.*

105. **(FR47 — GET /rules pagination)** Given 25 rules pour la company, When `GET /rules?page=1&perPage=10`, Then `items.length=10`, `total=25`. *Test E2E HTTP : `rule_list_paginated`.*

106. **(FR47 — GET /rules filter active)** Given 5 active + 3 inactive rules, When `GET /rules?active=true`, Then `items.length=5`. *Test E2E HTTP : `rule_list_filters_active`.*

107. **(FR47 — GET /rules multi-tenant)** Given rule de company_B, When user company_A `GET /rules`, Then la rule company_B n'apparaît pas. *Test E2E HTTP : `rule_list_scopes_by_company`.*

108. **(FR47 — PATCH /rules optimistic lock)** Given rule `version=3`, When PATCH avec `version=2` (stale), Then `409 OPTIMISTIC_LOCK_VIOLATION`. Given PATCH avec `version=3`, Then `200 OK` + `version=4` + audit `reconciliation_rule.updated` avec `details.before` / `details.after` champs modifiés. *Test E2E HTTP : `rule_update_uses_optimistic_lock`.*

109. **(FR47 — PATCH réactivation via active=true)** Given rule soft-deleted (`active=false`), When PATCH `{ active: true, version: <current> }`, Then `200 OK` + rule re-activée. *Test E2E HTTP : `rule_patch_reactivates_inactive_rule`.*

110. **(FR47 — DELETE /rules soft delete Q3)** Given DELETE `/rules/{id}`, Then `204 No Content` + rule.active=false en DB (**pas DELETE physique**) + audit `reconciliation_rule.deleted` avec `details.soft_delete=true`. La rule disparaît de `GET /rules?active=true` mais reste accessible via `GET /rules?active=false`. Les anciennes entrées audit `reconciliation_rule.applied` avec ce `rule_id` restent valides. *Test E2E HTTP : `rule_delete_soft_deletes_and_preserves_audit_history`.*

111. **(FR47 — DELETE /rules already inactive idempotent)** Given rule `active=false`, When DELETE, Then `200 OK` ou `204 No Content` no-op (idempotent — le `UPDATE active=false` est sans effet, pas d'audit redondant). *Test E2E HTTP : `rule_delete_idempotent_when_already_inactive`.*

112. **(FR47 — RBAC mutations Comptable+)** Given user `Consultation`, When POST/PATCH/DELETE `/rules`, Then `403`. GET `/rules` → 200 (read-only accessible). *Test E2E HTTP : `rule_mutations_require_comptable_role`.*

### Application des règles dans GET /proposals (FR47 partie 2)

113. **(FR47 — rule applied to tx without invoice candidate)** Given tx pending sans candidate invoice (counterparty `Swisscom AG`) et rule `counterparty_contains:Swisscom → 6510`, When `GET /proposals?bankAccountId=17`, Then la candidate de la tx contient `{ type: 'rule', ruleId: <id>, counterpartyAccountId: 6510, counterpartyAccountName: '6510 Frais télécom' }`. *Test E2E HTTP : `get_proposals_applies_rule_when_no_invoice_candidate`.*

114. **(FR47 — invoice candidate overrides rule)** Given tx avec invoice match `score=0.95` ET rule match aussi, When GET, Then la candidate est `{ type: 'invoice', ... }` (rule ignored, score-based dispatch). *Test E2E HTTP : `get_proposals_invoice_candidate_overrides_rule`.*

115. **(FR47 — rule priority order)** Given 2 rules actives matchant la même tx (`Swisscom` priority=200, `Swisscom AG` priority=100), When GET, Then la rule priority=100 gagne. *Test E2E HTTP : `get_proposals_applies_highest_priority_rule`.*

116. **(FR47 — rule skip si counterparty_account archivé)** Given rule active pointant sur compte `active=false`, When GET, Then la rule est ignorée silencieusement (log debug), pas de candidate type=rule. *Test E2E HTTP : `get_proposals_skips_rule_with_archived_account`.*

117. **(FR47 — rule skip si rule désactivée)** Given rule soft-deleted (`active=false`), When GET, Then la rule n'est pas évaluée pour les tx pending. *Test E2E HTTP : `get_proposals_skips_inactive_rule`.*

### Acceptation avec type='rule' (FR47 partie 3)

118. **(FR47 — POST accept type=rule happy)** Given une candidate type=rule sur tx 42, When POST accept `{ type: 'rule', bankTransactionId: 42, ruleId: 7, counterpartyAccountId: 6510 }`, Then `200 OK` accepted ET journal_entry à 2 lignes créée (via `manual::build_journal_entry_for_counterparty` 8-5a) ET `bank_transactions.status='reconciled'` ET `reconciliation_rules.applied_count` incrémenté ET `last_applied_at` mis à jour ET audit `reconciliation.accepted` (avec `details.rule_id`, `details.match_type`, **pas** d'`invoice_id`) + audit `reconciliation_rule.applied`. *Test E2E HTTP : `accept_with_rule_creates_journal_entry_and_increments_count`.*

119. **(FR47 — POST accept type=rule re-validation match)** Given une candidate type=rule retournée par GET, modification de la rule entre GET et POST (e.g. match_value changé), When POST accept, Then `failed: [{ errorCode: 'RECONCILIATION_RULE_NO_LONGER_MATCHES' }]`. *Test E2E HTTP : `accept_with_rule_rejects_when_no_longer_matches`.*

120. **(FR47 — POST accept type=rule mismatch counterpartyAccountId)** Given body proposal avec `counterpartyAccountId=9999` mais rule.counterparty_account_id=6510, When POST, Then `400 RECONCILIATION_RULE_MISMATCH`. *Test E2E HTTP : `accept_with_rule_validates_counterparty_account_consistency`.*

121. **(FR47 — POST accept type=rule optimistic lock rules.version)** Given une rule `version=2` au moment du GET /proposals, modification concurrente de la rule (PATCH par admin → `version=3`) entre GET et POST accept, When POST accept (utilise version=2 implicite via re-load), Then **soit** le re-validation match passe (la modif n'a pas changé `match_type`/`match_value`) et l'UPDATE `applied_count` se fait sur `version=3` final, **soit** la modif a changé le match → `RECONCILIATION_RULE_NO_LONGER_MATCHES` (cas AC #119). *Test E2E HTTP : `accept_with_rule_handles_concurrent_rule_update`.*

122. **(FR47 — POST accept type=rule audit dual)** Given POST accept type=rule happy, When commit, Then audit_log contient EXACTEMENT 3 entrées : `reconciliation.accepted` (avec rule_id) + `reconciliation_rule.applied` + `journal_entry.created` (émis par `create_in_tx`). Pas de duplicate. *Test E2E HTTP : `accept_with_rule_emits_triple_audit_log`.*

### UI page /reconciliation/rules

123. **(UI — page /reconciliation/rules CRUD)** Given user Comptable navigue `/reconciliation/rules`, Then table des règles + bouton « + Nouvelle règle » + actions par ligne (Edit, Désactiver/Réactiver, Supprimer). Form modal `RuleFormModal` valide avant submit (label non-vide, match_value non-vide, counterparty_account_id sélectionné). *Test Playwright : `rules_crud_end_to_end` + Vitest : `RuleFormModal: validates required fields`.*

124. **(UI — accessibilité a11y axe rules page)** Given `/reconciliation/rules` rendue avec ≥ 5 règles, When axe-core scan, Then 0 violation. Idem pour `RuleFormModal` ouvert. *Test Playwright : `accessibility — rules page axe scan`, `accessibility — rule form modal axe scan`.*

## Tasks / Subtasks

### T1. Migration DB `reconciliation_rules` avec UNIQUE partiel Q3 (AC #101-#112, #122)

- [ ] T1.1 — Créer `crates/kesh-db/migrations/20260MMDD000001_reconciliation_rules.sql` :
  ```sql
  CREATE TABLE reconciliation_rules (
      id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
      company_id BIGINT UNSIGNED NOT NULL,
      label VARCHAR(120) NOT NULL,
      match_type ENUM('counterparty_contains','counterparty_exact','reference_contains','iban_exact') NOT NULL,
      match_value VARCHAR(255) NOT NULL,
      counterparty_account_id BIGINT UNSIGNED NOT NULL,
      priority INT NOT NULL DEFAULT 100,
      active BOOLEAN NOT NULL DEFAULT TRUE,
      applied_count BIGINT UNSIGNED NOT NULL DEFAULT 0,
      last_applied_at DATETIME(3) NULL,
      version INT NOT NULL DEFAULT 1,
      created_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
      updated_at DATETIME(3) NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
      -- Q3 décision Guy 2026-05-07 : soft-delete via active=false. UNIQUE doit être partiel
      -- sur active=true pour permettre reactivation après archivage. MariaDB n'a pas de UNIQUE
      -- partiel natif → workaround via colonne synthétique active_uniq (NULL si !active, NULL exclus du UNIQUE).
      active_uniq VARCHAR(255) GENERATED ALWAYS AS (IF(active, match_value, NULL)) VIRTUAL,
      CONSTRAINT fk_reconciliation_rules_company FOREIGN KEY (company_id) REFERENCES companies (id),
      CONSTRAINT fk_reconciliation_rules_account FOREIGN KEY (counterparty_account_id) REFERENCES accounts (id),
      CONSTRAINT chk_reconciliation_rules_match_value_non_empty CHECK (CHAR_LENGTH(TRIM(match_value)) > 0),
      CONSTRAINT chk_reconciliation_rules_label_non_empty CHECK (CHAR_LENGTH(TRIM(label)) > 0),
      CONSTRAINT chk_reconciliation_rules_priority_range CHECK (priority BETWEEN 1 AND 1000),
      -- UNIQUE partiel via active_uniq (NULL not unique-counted en MariaDB)
      CONSTRAINT uq_reconciliation_rules_match_active UNIQUE (company_id, match_type, active_uniq),
      INDEX idx_reconciliation_rules_company_active_priority (company_id, active, priority, id)
  ) ENGINE=InnoDB CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
  ```

- [ ] T1.2 — Mettre à jour `crates/kesh-db/src/test_fixtures.rs` `TABLES_TO_TRUNCATE` const → ajouter `"reconciliation_rules"` (lesson 8-1b retro).

- [ ] T1.3 — Vérifier `cargo test -p kesh-db --lib test_fixtures` avec MariaDB up + `KESH_TEST_MODE=true` (lesson 8-3 retro CHECK constraints invisibles sans DB).

- [ ] T1.4 — Vérifier `EXPLAIN SELECT ... FROM reconciliation_rules WHERE company_id=? AND active=TRUE ORDER BY priority` post-migration — confirmer `type=ref` ou `range`, pas `ALL`.

- [ ] T1.5 — Vérifier comportement UNIQUE partiel : INSERT 2 rules `(c, ct_contains, 'Swisscom', active=true)` doit échouer SQL 1062 ; INSERT 2 rules dont 1 active 1 inactive avec mêmes match doit réussir.

### T2. Repository `kesh-db::reconciliation_rules` (AC #101-#112, #122)

- [ ] T2.1 — Créer `crates/kesh-db/src/entities/reconciliation_rule.rs` :
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
  #[serde(rename_all = "camelCase")]
  pub struct ReconciliationRule {
      pub id: i64,
      pub company_id: i64,
      pub label: String,
      pub match_type: ReconciliationMatchType,
      pub match_value: String,
      pub counterparty_account_id: i64,
      pub priority: i32,
      pub active: bool,
      pub applied_count: i64,
      pub last_applied_at: Option<NaiveDateTime>,
      pub version: i32,
      pub created_at: NaiveDateTime,
      pub updated_at: NaiveDateTime,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
  #[serde(rename_all = "snake_case")]
  pub enum ReconciliationMatchType {
      CounterpartyContains,
      CounterpartyExact,
      ReferenceContains,
      IbanExact,
  }
  // + impl Type/Encode/Decode<MySql> comme BankTransactionStatus
  ```

- [ ] T2.2 — Créer `crates/kesh-db/src/repositories/reconciliation_rules.rs` :
  - `find_active_for_company<E>(executor, company_id) -> Result<Vec<ReconciliationRule>, DbError>` (ordre `priority ASC, id ASC`).
  - `list_by_company_paginated(pool, company_id, active_filter: Option<bool>, page, per_page) -> Result<(Vec<...>, u64), DbError>`.
  - `find_by_id_for_company<E>(executor, company_id, id) -> Result<Option<ReconciliationRule>, DbError>`.
  - `create_in_tx(tx, company_id, user_id, NewReconciliationRule) -> Result<ReconciliationRule, DbError>` (mappe SQL 1062 sur `uq_reconciliation_rules_match_active` → `DbError::DuplicateRule`).
  - `update_in_tx(tx, company_id, id, expected_version, UpdateReconciliationRule) -> Result<ReconciliationRule, DbError>` (optimistic lock).
  - `soft_delete_by_id_for_company(tx, company_id, id) -> Result<bool, DbError>` (`UPDATE active=false, updated_at=NOW(3), version=version+1` ; idempotent — si déjà active=false retourne `false`).
  - `increment_applied_count_in_tx(tx, company_id, rule_id) -> Result<(), DbError>` (`UPDATE applied_count = applied_count+1, last_applied_at = NOW(3), version=version+1`).

- [ ] T2.3 — Mettre à jour `crates/kesh-db/src/repositories/mod.rs` (`pub mod reconciliation_rules`) et `entities/mod.rs`.

- [ ] T2.4 — Tests `#[sqlx::test]` (≥ 6) `crates/kesh-db/tests/reconciliation_rules_repository.rs` :
  1. `create_and_find_by_id_scopes_by_company` (AC #101, #107).
  2. `unique_match_type_value_per_company_when_active` (AC #102).
  3. `unique_partial_allows_create_when_existing_inactive` (AC #103, Q3).
  4. `list_filters_active` (AC #105, #106).
  5. `update_uses_optimistic_lock` (AC #108).
  6. `soft_delete_sets_active_false_idempotent` (AC #110, #111).
  7. `increment_applied_count_atomic` (couvre AC #118 partiellement).

### T3. Helper `kesh-reconciliation::rules` (AC #113-#122)

- [ ] T3.1 — Créer `crates/kesh-reconciliation/src/rules.rs` :
  ```rust
  use kesh_db::entities::{BankTransaction, ReconciliationRule, ReconciliationMatchType};
  use std::collections::HashSet;

  /// Teste si une règle match une tx. Pure (zéro I/O).
  pub fn rule_matches(rule: &ReconciliationRule, tx: &BankTransaction) -> bool {
      use ReconciliationMatchType::*;
      match rule.match_type {
          CounterpartyContains => match_contains(tx.counterparty_name.as_deref(), &rule.match_value),
          CounterpartyExact   => match_exact(tx.counterparty_name.as_deref(), &rule.match_value),
          ReferenceContains   => {
              let reference = tx.reference.as_deref()
                  .or(tx.end_to_end_id.as_deref())
                  .or(tx.transaction_id.as_deref());
              match_contains(reference, &rule.match_value)
          },
          IbanExact => tx.counterparty_iban.as_deref() == Some(&rule.match_value),
      }
  }

  /// Applique la liste ordonnée de règles à une tx, retourne la première qui match
  /// ET dont counterparty_account_id existe dans active_account_ids (caller injecte le HashSet
  /// pré-calculé pour O(1) lookup).
  pub fn first_matching_rule<'a>(
      rules: &'a [ReconciliationRule],
      tx: &BankTransaction,
      active_account_ids: &HashSet<i64>,
  ) -> Option<&'a ReconciliationRule> {
      rules.iter()
          .filter(|r| r.active)
          .filter(|r| active_account_ids.contains(&r.counterparty_account_id))
          .find(|r| rule_matches(r, tx))
  }

  fn normalize(s: &str) -> String { s.trim().to_lowercase() }
  fn match_contains(haystack: Option<&str>, needle: &str) -> bool {
      let Some(h) = haystack else { return false };
      normalize(h).contains(&normalize(needle))
  }
  fn match_exact(haystack: Option<&str>, needle: &str) -> bool {
      let Some(h) = haystack else { return false };
      normalize(h) == normalize(needle)
  }
  ```

- [ ] T3.2 — Étendre `crates/kesh-reconciliation/src/lib.rs` :
  ```rust
  pub mod rules;
  pub use rules::{rule_matches, first_matching_rule};
  ```

- [ ] T3.3 — Étendre `crates/kesh-reconciliation/src/errors.rs` (variants ajoutés) :
  ```rust
  pub enum ReconciliationError {
      // ... 8-4 + 8-5a variants conservés
      RuleNoLongerMatches { rule_id: i64 },
      RuleMismatch { rule_id: i64, expected_account: i64, actual_account: i64 },
      RuleDuplicate { match_type: String, match_value: String },
      RuleNotFound { rule_id: i64 },
  }
  ```

- [ ] T3.4 — Tests unit `kesh-reconciliation::rules` (≥ 6) :
  1. `rule_matches_counterparty_contains` (AC #113).
  2. `rule_matches_counterparty_exact_normalize_case` (AC #113).
  3. `rule_matches_iban_exact` (AC #113).
  4. `rule_matches_reference_fallback_chain` (AC #113).
  5. `first_matching_rule_respects_priority_order` (AC #115).
  6. `first_matching_rule_skips_inactive_account` (AC #116).
  7. `first_matching_rule_skips_inactive_rule` (AC #117).

### T4. Routes API (AC #101-#112, #113-#122)

- [ ] T4.1 — Créer `crates/kesh-api/src/routes/reconciliation_rules.rs` (parallèle à `bank_profiles.rs` pattern CRUD) :
  - `pub fn router_mutations() -> Router<AppState>` : POST `/api/v1/reconciliation/rules`, PATCH/DELETE `/api/v1/reconciliation/rules/{id}` (sub-router comptable_routes).
  - `pub fn router_reads() -> Router<AppState>` : GET list+detail (sub-router authenticated_routes).
  - Handlers : `create`, `list`, `detail`, `update` (PATCH partial avec `expected_version` body), `delete` (soft).

- [ ] T4.2 — Étendre `crates/kesh-api/src/routes/reconciliation.rs` (du 8-4/8-5a) :
  - Modifier `get_proposals` pour appliquer les rules en fallback (cf. spec d'origine §rule-application).
  - Étendre `post_accept` pour gérer `type='rule'` (cf. spec d'origine §accept-with-rule-flow). Body schema validé : `{ type: 'rule', bankTransactionId, ruleId, counterpartyAccountId }`.

- [ ] T4.3 — Étendre `crates/kesh-api/src/lib.rs` mounting :
  - `comptable_routes` : ajouter mutations `/rules`.
  - `authenticated_routes` : ajouter GET list+detail rules.

- [ ] T4.4 — Étendre `crates/kesh-api/src/errors.rs` (4 nouvelles variants) :
  - `AppError::ReconciliationRuleNotFound { rule_id }` → `404 RECONCILIATION_RULE_NOT_FOUND`.
  - `AppError::ReconciliationRuleDuplicate { match_type, match_value }` → `409 RECONCILIATION_RULE_DUPLICATE`.
  - `AppError::ReconciliationRuleMismatch { rule_id }` → `400 RECONCILIATION_RULE_MISMATCH`.
  - `AppError::ReconciliationRuleNoLongerMatches { rule_id }` → mappé en `failed[]` shape (per-proposal error, pas HTTP global).

- [ ] T4.5 — Tests E2E HTTP `crates/kesh-api/tests/reconciliation_rules_e2e.rs` *(nouveau)* (≥ 14 tests) :
  1-9. Rules CRUD (AC #101-#112).
  10-12. Rule application GET /proposals (AC #113-#117).
  13-16. Rule accept (AC #118-#122).

### T5. Frontend feature `features/reconciliation/rules` + page (AC #123-#124)

- [ ] T5.1 — Créer `frontend/src/lib/features/reconciliation/rules/` :
  - `rules.api.ts` (CRUD client).
  - `rules.types.ts` (`ReconciliationRule`, `MatchType` enum).
  - `RulesList.svelte` (table + actions).
  - `RuleFormModal.svelte` (form modal create/edit).
  - `RulesList.test.ts`, `RuleFormModal.test.ts` (Vitest).

- [ ] T5.2 — Étendre `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` :
  - Pour candidate `type: 'rule'` : badge bleu « Règle » + label règle + nom du compte de contrepartie.

- [ ] T5.3 — Étendre `frontend/src/lib/features/reconciliation/reconciliation.api.ts` `acceptProposal` :
  - Ajouter le cas `type: 'rule'` qui envoie `{ type: 'rule', bankTransactionId, ruleId, counterpartyAccountId }`.

- [ ] T5.4 — Créer `frontend/src/routes/(app)/reconciliation/rules/+page.svelte` (route protégée) :
  - Layout : titre + bouton « + Nouvelle règle » + `RulesList` + `RuleFormModal` ouvert sur création/édition.
  - Action « Désactiver » → PATCH `{ active: false }`. Action « Réactiver » → PATCH `{ active: true }`. Action « Supprimer » → DELETE soft.

- [ ] T5.5 — Créer `frontend/src/routes/(app)/reconciliation/rules/+page.ts` (load function) :
  - Charger initial `GET /reconciliation/rules?active=true&page=1&perPage=50`.

- [ ] T5.6 — Tests Vitest (≥ 4) :
  1. `RulesList: renders rules with priority sort` (AC #123).
  2. `RulesList: renders inactive rules differently when active=false filter` (AC #110).
  3. `RuleFormModal: validates required fields before submit` (AC #123).
  4. `ReconciliationProposals: renders rule candidate with blue badge` (AC #113 UI).

### T6. i18n (AC implicite UI)

- [ ] T6.1 — Ajouter ~15 nouvelles clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (préfixes stricts `reconciliation-rules-*` × 12, `reconciliation-rule-applied-*` × 3). FR canonical.
- [ ] T6.2 — Traductions DE / IT / EN — pas de copies françaises (lesson 8-2 H13). Vocabulaire bancaire suisse.
- [ ] T6.3 — Vérifier `npm run lint-i18n-ownership` PASS sur 4 locales.

### T7. Tests E2E Playwright + a11y (AC #123-#124)

- [ ] T7.1 — Créer `frontend/tests/e2e/reconciliation-rules.spec.ts` (≥ 1 actif) :
  1. `rules CRUD end-to-end` : navigate `/reconciliation/rules`, créer rule, éditer, désactiver, réactiver, supprimer, vérifier audit.

- [ ] T7.2 — Tests a11y axe (AC #124) : 2 scénarios — page rules avec ≥ 5 rules + RuleFormModal ouvert.

- [ ] T7.3 — Helper `seedReconciliationRulesFixture(page, ...)` — crée des rules de test avec différentes priorités/match_types.

### T8. Sync sprint-status + README (AC implicite Epic 8 progress)

- [ ] T8.1 — `_bmad-output/implementation-artifacts/sprint-status.yaml` : transition `8-5b-reconciliation-rules-engine: backlog → ready-for-dev` (post-merge 8-5a) puis `→ in-progress` puis `→ review` (post `dev-story`).
- [ ] T8.2 — README.md `## Feuille de route` : Epic 8 transition vers ✅ Done après merge 8-5b (clôt l'epic Import bancaire & Réconciliation).
- [ ] T8.3 — README.md `## Fonctionnalités` : ajouter ligne « Règles d'affectation automatique (FR47) » sous la section import bancaire.
- [ ] T8.4 — Pas d'issue GitHub à fermer (8-5b n'a pas de KF/CR pré-tracée). KF-026 #76 reste ouverte (multi-candidates UI v0.2 — non adressée 8-5).

## Dev Notes

### API surface livrée 8-1b/8-2/8-3/8-4/8-5a — patterns à réutiliser

- **Multi-tenant scoping** (KF-002 Pattern 1) : tous les helpers DB filtrent par `(company_id, ...)`. Cross-tenant = 404, jamais 403.
- **Audit log atomique** : helper `audit_log::insert_in_tx(tx, NewAuditLogEntry { ... })`. Une entrée par operation distincte. Pour 8-5b : `reconciliation_rule.{created,updated,deleted,applied}`.
- **Erreurs structurées** : `AppError::Custom { ... }` ou variantes typées dédiées (préféré).
- **i18n key ownership** : préfixe strict, kebab-case, lint-i18n-ownership pass (Story 6-3).
- **Repository pattern + sqlx** : Executor générique `<E: Executor>` (pattern 8-3 / 8-4 / 8-5a).
- **Advisory lock per-account** : `with_account_lock(tx, company_id, bank_account_id, 5)` réutilisé pour `accept-with-rule` (sérialisation cross-flows sur le même compte).
- **`journal_entries::create_in_tx`** : helper Story 5-2, accepte tx ouverte par caller, ne commit pas. Émet audit `journal_entry.created` automatiquement.
- **`fiscal_years::find_open_for_date_for_company`** : helper Story 3-7 (vérifié/créé 8-5a T4), indispensable pour résoudre `fiscal_year_id` à partir d'une `entry_date` dans accept-with-rule.
- **Helper `manual::build_journal_entry_for_counterparty`** : **livré 8-5a, réutilisé tel quel** par 8-5b dans le flow `accept-with-rule` step 8.
- **Helper `bank_transactions::find_pending_by_id_for_account`** : livré 8-5a, réutilisé par flow `accept-with-rule`.
- **Breaking change `POST /accept` discriminator type** : livré 8-5a (type='invoice' obligatoire). 8-5b ajoute type='rule' au discriminator.

### Lessons leçons des stories précédentes

- **8-4 retro** (cycle 4 passes review pour 5 modules / ~2200 lignes) : 8-5b découpée à ~1800 lignes pour viser ≤ 3 passes review.
- **8-3 retro** (CHECK constraints invisibles sans MariaDB up) : T1 ajoute 3 CHECK + 1 UNIQUE partiel (Q3 workaround) + 1 colonne synthétique — vérifier avec MariaDB up local + KESH_TEST_MODE=true. **Le test du UNIQUE partiel** (AC #103) ne passe que MariaDB up.
- **8-2 retro H7** : pas de breaking change ParseCsvOutcome équivalent ici. Le breaking change discriminator `POST /accept` est déjà fait par 8-5a.
- **5-2 leçon** (`create_in_tx` pour atomicité) : la nouvelle route accept-with-rule **doit** utiliser `create_in_tx` plutôt que `create` (qui ouvre sa propre tx, incompatible avec la tx du `with_account_lock`).
- **8-4 patch P3-H1** (optimistic lock UPDATE bank_transactions `AND version = ?`) : appliquer **systématiquement** sur tous les UPDATE bank_transactions de 8-5b (accept-with-rule).
- **Q3 décision Guy 2026-05-07** : MariaDB n'a pas de UNIQUE partiel natif → workaround colonne synthétique `active_uniq` (Option A). Ne pas tenter Option B (trigger SQL) ni Option C (check applicatif racy) sans CR explicite.
- **Q5 décision Guy 2026-05-07** : pas de fonction `suggest_rule`, pas d'endpoint `/rules/suggest`, pas d'objet `ruleSuggestion` dans aucune response. Reporté v0.2.

### Patterns architecturaux à respecter

- **Pas de dépendance circulaire** : `kesh-reconciliation → kesh-core, kesh-db` (cohérent 8-4/8-5a). Le nouveau module `rules` consomme `kesh_db::entities::{BankTransaction, ReconciliationRule, ReconciliationMatchType}`.
- **Pas d'`f64` pour montants** : `Decimal` partout. Le `f64` n'apparaît que dans le score 8-4 hérité, pas dans 8-5b.
- **Tests : éviter le coupling temporel** : utiliser des dates fixes dans les seeds.
- **`auto_match_rejected_at=NULL` au accept-with-rule** : indispensable pour cohérence avec manual-match (cas race « rejet auto → rule applied → accept »).

### Source tree à toucher

**DB** :
- `crates/kesh-db/migrations/20260MMDD000001_reconciliation_rules.sql` *(nouveau, T1)*
- `crates/kesh-db/src/repositories/reconciliation_rules.rs` *(nouveau, T2)*
- `crates/kesh-db/src/repositories/mod.rs` (re-export `pub mod reconciliation_rules`)
- `crates/kesh-db/src/entities/reconciliation_rule.rs` *(nouveau, T2)*
- `crates/kesh-db/src/entities/mod.rs` (re-export)
- `crates/kesh-db/src/test_fixtures.rs` (TABLES_TO_TRUNCATE += `"reconciliation_rules"`)
- `crates/kesh-db/tests/reconciliation_rules_repository.rs` *(nouveau, ≥ 7 tests)*

**Backend `kesh-reconciliation`** :
- `crates/kesh-reconciliation/Cargo.toml` (deps inchangées)
- `crates/kesh-reconciliation/src/lib.rs` (refactor — module `rules` ajouté)
- `crates/kesh-reconciliation/src/rules.rs` *(nouveau, pure ; pas de fonction `suggest_rule` per Q5)*
- `crates/kesh-reconciliation/src/errors.rs` (4 variantes ajoutées)

**Backend `kesh-api`** :
- `crates/kesh-api/src/routes/reconciliation_rules.rs` *(nouveau, CRUD)*
- `crates/kesh-api/src/routes/reconciliation.rs` (extension : `get_proposals` rule application + `post_accept` type='rule')
- `crates/kesh-api/src/routes/mod.rs` (`pub mod reconciliation_rules`)
- `crates/kesh-api/src/lib.rs` (mount routes)
- `crates/kesh-api/src/errors.rs` (4 nouvelles variantes)
- `crates/kesh-api/tests/reconciliation_rules_e2e.rs` *(nouveau, ≥ 14 tests)*

**i18n** :
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (~15 nouvelles clés × 4 locales)

**Frontend** :
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts` (extension `acceptProposal` type='rule')
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` (extension badge bleu rule candidate)
- `frontend/src/lib/features/reconciliation/rules/rules.api.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/rules/rules.types.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/rules/RulesList.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/rules/RulesList.test.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/rules/RuleFormModal.test.ts` *(nouveau)*
- `frontend/src/routes/(app)/reconciliation/rules/+page.svelte` *(nouveau, route)*
- `frontend/src/routes/(app)/reconciliation/rules/+page.ts` *(nouveau, load function)*
- `frontend/tests/e2e/reconciliation-rules.spec.ts` *(nouveau, Playwright)*

### Standards de test

- **Unit `kesh-reconciliation`** : `#[cfg(test)] mod tests` inline `rules.rs`. ≥ 7 unit tests T3.4.
- **Intégration `kesh-db`** : `#[sqlx::test]`. ≥ 7 tests T2.4.
- **E2E HTTP `kesh-api`** : helper `spawn_app(pool)` (pattern 8-1b/8-2/8-3/8-4/8-5a). ≥ 14 tests T4.5.
- **Vitest frontend** : `npm run test:unit -- reconciliation-rules`. ≥ 4 tests T5.6.
- **Playwright** : `frontend/tests/e2e/reconciliation-rules.spec.ts`. ≥ 1 actif + 2 a11y.

### Checklist locale avant push

```sh
# Backend (cf. CLAUDE.md « Test Locally First »)
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1   # MariaDB up requis (T1 migration check + T2 sqlx + T3 unit + T4 E2E)

# Frontend
cd frontend
npm run check
npm run lint-i18n-ownership   # T6.3
npm run test:unit
npm run build

# E2E (MariaDB up + seed CI + browsers installés)
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- reconciliation-rules.spec.ts
```

### Limitations connues v0.1 (sous-ensemble 8-5b — voir spec d'origine pour la liste complète L41-L50)

| # | Limitation | Justification |
|---|---|---|
| L41 | Règles avec patterns simples (4 `match_type` v0.1) — pas de regex | Décision conservatrice. v0.1 : `counterparty_contains`, `counterparty_exact`, `reference_contains`, `iban_exact`. v0.2 : ajouter `match_type='regex'` avec validation côté serveur (pas de DB-side regex). |
| L42 | Pas d'auto-acceptation des candidates type=rule (cohérent L18 héritée 8-4) | Toutes les candidates rule sont retournées dans GET /proposals. L'utilisateur valide explicitement comme pour les invoice candidates. v0.2 : seuil `auto_accept_threshold` configurable par tenant + par rule. |
| L43 | Pas d'application batch des règles à un dataset historique | La route `POST /reconciliation/apply-rules` n'est pas livrée v0.1. Les règles s'appliquent uniquement à GET /proposals à la volée. v0.2 : route batch pour reprocesser un range de dates. |
| L46 | Suggestion de règle déterministe sans ML | **Décision Guy Q5 2026-05-07** : ni heuristique déterministe, ni ML. L'utilisateur crée ses rules manuellement via UI. v0.2+ (Story 8-5c potentielle) : training sur historique pour suggérer des matches. |
| L48 | UI rule application sans preview de matches | Le frontend ne montre pas combien de tx pending matcheraient une nouvelle règle avant sa création. L'utilisateur doit créer puis observer. v0.2 : endpoint `POST /reconciliation/rules/preview`. |
| L49 | Pas d'export CSV des rules pour backup/migration | Si un utilisateur veut migrer ses rules vers une nouvelle instance Kesh, il doit les recréer manuellement via UI. v0.2 : endpoint `GET /reconciliation/rules/export.csv` + `POST /reconciliation/rules/import`. |
| L50 | Rule applied count non décrémenté si reconciliation revertée | Cohérent L45 (pas d'undo v0.1). Si l'utilisateur modifie l'écriture comptable manuellement après accept-with-rule, `reconciliation_rules.applied_count` reste incrémenté (compte « accept tenté », pas « accept abouti »). Acceptable v0.1. |

### Risques et points d'attention pour le dev agent

1. **Concurrence rules accept** : un Comptable accepte une rule candidate, et au même moment, un autre Comptable édite la rule (PATCH change `match_value`). Le optimistic lock sur `rules.version` au step 11 §accept-with-rule-flow protège, mais l'utilisateur observe `failed: [{ errorCode: 'RECONCILIATION_RULE_NO_LONGER_MATCHES' }]` même si l'edit est anodin. Acceptable v0.1.

2. **Rule sur compte archivé** : si un Comptable archive un compte qui est `counterparty_account_id` de plusieurs rules actives, les rules continuent d'apparaître mais skippent silencieusement à l'application. UX peut surprendre. **Recommandation** : ajouter une UI warning « Cette règle utilise un compte archivé » dans `RulesList.svelte` (T5.1). Si non livré v0.1 → ajouter LXX dette UX.

3. **Migration UNIQUE partiel via colonne synthétique (Q3)** : tester avec MariaDB up que (a) deux rules actives avec mêmes match échouent SQL 1062, (b) une rule active + une rule inactive avec mêmes match coexistent, (c) reactivation via PATCH active=true ne fail pas si une autre rule active a aussi été créée entre-temps avec mêmes match (cas pathologique → 409 RECONCILIATION_RULE_DUPLICATE attendu). AC #103 + #109.

4. **Test E2E HTTP volume** : 14 nouveaux tests minimum. Pas de dette test acceptable (lessons 8-4 retro). Les 9 rules CRUD + 5 application accept-with-rule sont **incontournables** (sécurité multi-tenant + RBAC + audit log dual + Q3 reactivation flow).

5. **Path-dépendance sur 8-5a** : si `manual::build_journal_entry_for_counterparty` ou `find_pending_by_id_for_account` ont changé entre 8-5a livré et 8-5b démarré (rare mais possible si un patch CR-XXX est appliqué entre-temps), vérifier la signature avant T4. Si breaking change → CR explicite.

6. **Suppression définitive de la suggestion ML (Q5)** : ne pas implémenter `suggest_rule`, ne pas créer endpoint `/rules/suggest`. Si le dev agent voit du code lié dans la spec d'origine, l'ignorer (caduque post-Q5). Tracer en `Completion Notes` que cet aspect est explicitement out-of-scope.

### Références

- [`8-5-reconciliation-manuelle-regles-affectation.md`](8-5-reconciliation-manuelle-regles-affectation.md) — spec d'origine `archived-split` (référence des décisions de conception détaillées).
- [`8-5a-reconciliation-manuelle-split.md`](8-5a-reconciliation-manuelle-split.md) — sous-story 8-5a, livre les helpers réutilisés par 8-5b.
- [`epic-8.md`](../planning-artifacts/epic-8.md) — Story 8-5 ACs originaux (FR45-FR48), section « Risques » R6 R7.
- [`prd.md`](../planning-artifacts/prd.md) §FR45-FR48 lignes 439-442.
- [`8-4-reconciliation-matching-automatique.md`](8-4-reconciliation-matching-automatique.md) — patterns repo + mutex + audit + savepoint à réutiliser.
- [`architecture.md`](../planning-artifacts/architecture.md) §11.5 (kesh-reconciliation), §17 (FR42-FR53 mapping), L491-L498 (modules `matching/rules/mutex`).
- [`ux-design-specification.md`](../planning-artifacts/ux-design-specification.md) §164 scenario Lisa fiduciaire, §329 modèle « routine fluide » règles d'affectation.
- [Story 5-2 `journal_entries::create_in_tx`](../../crates/kesh-db/src/repositories/journal_entries.rs) — helper transaction-bound.
- [Story 3-7 `fiscal_years::*`](../../crates/kesh-db/src/repositories/fiscal_years.rs) — résolution `fiscal_year_id` from `entry_date`.
- KF-026 #76 — multi-candidates UI (v0.2, non adressée 8-5).

## Dev Agent Record

### Agent Model Used

À renseigner par le dev agent au moment de l'implémentation.

### Debug Log References

(à compléter par dev-story)

### Completion Notes List

(à compléter par dev-story)

### File List

(à compléter par dev-story)

## Change Log

- **2026-05-07** — Spec créée par split mécanique de 8-5 unifiée (décision Guy 2026-05-07 Q1=B). Découpage scope FR47 rules engine (CRUD + application GET /proposals + extension POST /accept type='rule'). FR46 suggestion ML supprimée (Q5 — reportée v0.2). Soft-delete via colonne synthétique active_uniq (Q3 workaround MariaDB). 4 actions audit distinctes (Q4b). 24 ACs (#101-#124). Tasks T1-T8. Path-dépendance sur 8-5a (helper `manual::build_journal_entry_for_counterparty` réutilisé). Status `8-5b-reconciliation-rules-engine: backlog` jusqu'à 8-5a `done`/merged. Cycle prévu après merge 8-5a : transition `backlog → ready-for-dev` puis `bmad-create-story validate 8-5b` Pass 1 Sonnet (cycle CLAUDE.md, auteur=Opus split, briser biais d'auteur).
