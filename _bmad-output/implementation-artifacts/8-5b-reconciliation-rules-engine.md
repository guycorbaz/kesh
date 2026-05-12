# Story 8-5b: Moteur de règles d'affectation

Status: ready-for-dev

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

#### §rule-application — flow détaillé GET /proposals

(Pass 1 P-C1 : inline le flow car la spec archivée n'est pas accessible au dev agent.)

**Position dans la response** : pour chaque tx pending, après le calcul des candidates invoice (score 8-4), si **aucune candidate invoice n'a `score >= 0.5`** (ni 0.5 lui-même), appliquer les rules. Si une rule matche, ajouter une candidate avec `type: 'rule'` à la liste `candidates[]` de la tx — **en plus** des candidates invoice de score < 0.5 (coexistence). Le frontend choisit l'affichage prioritaire (la rule en premier visuellement, mais l'invoice reste visible).

**Sign filter (Pass 1 P-H7 ECH-05)** : contrairement au matching invoice 8-4 qui skip les tx débit (`amount <= Decimal::ZERO`), **les rules s'appliquent aux 2 sens** (débit ET crédit). Le use-case canonique (Swisscom, salaires, cotisations sociales) est débit. Le sign-aware côté JE est géré par `build_journal_entry_for_counterparty` (8-5a-base) qui choisit débit/crédit selon `sign(tx.amount)`. **Ne pas hériter du filter sign 8-4 ligne `reconciliation.rs:343`** pour les rules.

**Résolution `counterpartyAccountName` dans la candidate (Pass 2 Q5 AA-F1)** : la candidate type=rule expose `counterpartyAccountId` + `counterpartyAccountName` pour affichage UI sans 2ème round-trip. Le handler `get_proposals`, **après le rule-match** (cf. ci-dessous), fait 1 SELECT additionnel : `SELECT id, number, name FROM accounts WHERE id = ? AND company_id = ?`. Optimisation : pré-charger les accounts mappés `HashMap<i64, AccountInfo>` en même temps que `active_account_ids` (1 SELECT enrichi au lieu de 1 par rule-match). Pattern :

```rust
let accounts_info: HashMap<i64, (String, String)> = sqlx::query_as::<_, (i64, String, String)>(
    "SELECT id, number, name FROM accounts WHERE company_id = ? AND active = TRUE",
)
.bind(company_id)
.fetch_all(&state.pool)
.await?
.into_iter()
.map(|(id, num, name)| (id, (num, name)))
.collect();

// active_account_ids = accounts_info.keys() (no extra query)
```

Le helper `first_matching_rule` retourne `Option<&Rule>`, le handler résout `accounts_info.get(&rule.counterparty_account_id)` pour construire `counterpartyAccountName = format!("{} {}", num, name)`.

**Stratégie chargement `active_account_ids` (Pass 1 P-H4 ECH-02 + Pass 2 Q5 fusionné)** : voir bloc ci-dessus — `accounts_info` HashMap résout les 2 besoins (filter actif + nom display) en 1 query batch.

```rust
let active_account_ids: HashSet<i64> = sqlx::query_scalar::<_, i64>(
    "SELECT id FROM accounts WHERE company_id = ? AND active = TRUE",
)
.bind(company_id)
.fetch_all(&state.pool)
.await?
.into_iter()
.collect();
```

(SELECT inline cohérent pattern `accept_one_split` 8-5a-bis car `accounts::find_by_id_in_company` n'est pas Executor-generic. Ne pas faire 1 query par rule — antipattern O(N) découvert dans la spec d'origine. Pas de nouveau helper kesh-db nécessaire.)

**Order strict (Pass 1 P-M-LOW ECH-13 + Pass 2 Q1 BH-F2)** : `find_active_for_company` ORDER BY **`priority ASC, id ASC`** — tiebreaker à 2 niveaux. (Pass 2 patch : la mention `created_at ASC` 3ème niveau était une régression Pass 1 vs T2.2 — supprimé. `id ASC` suffit comme tiebreaker total puisque id est unique et AUTO_INCREMENT — ordre de création préservé implicitement.) À tester via T3.4 cas `first_matching_rule_respects_id_tiebreaker_on_equal_priority` (test #8 ajouté Pass 2 Q10).

#### §accept-with-rule-flow — steps 1-13 détaillés

(Pass 1 P-C1 : inline car le dev agent doit pouvoir implémenter sans accès à la spec archivée. Pattern dérivé de `post_split` 8-5a-bis + `post_manual` 8-5a-base.)

**Body** :
```json
{
  "bankAccountId": 17,
  "proposals": [
    { "type": "rule", "bankTransactionId": 43, "ruleId": 7, "counterpartyAccountId": 6510 }
  ]
}
```

**Validation pré-flight surface (Pass 1 P-C1 + ECH-08)** dans `post_accept` step 0, pour chaque variant Rule :

```rust
AcceptProposalInput::Rule { bank_transaction_id, rule_id, counterparty_account_id } => {
    if *bank_transaction_id <= 0 { return Err(AppError::Validation("bankTransactionId > 0".into())); }
    if *rule_id <= 0 { return Err(AppError::Validation("ruleId > 0".into())); }
    if *counterparty_account_id <= 0 { return Err(AppError::Validation("counterpartyAccountId > 0".into())); }
}
```

**Flow `accept_one_rule` (inside `with_account_lock`)** — pattern parallèle à `accept_one_split` 8-5a-bis :

1. **Re-fetch tx INSIDE lock (TOCTOU)** : `find_strictly_pending_by_id_for_account(tx_inner, company_id, bank_account_id, bank_transaction_id)` → 404 `RECONCILIATION_TRANSACTION_NOT_PENDING` si None.
2. **Re-fetch rule INSIDE lock** : `find_by_id_for_company(tx_inner, company_id, rule_id)` → 404 `RECONCILIATION_RULE_NOT_FOUND` si None OU si `active=false` (rule désactivée concurrente entre GET et POST).
3. **Bank account lookup INSIDE lock** : SELECT inline `bank_accounts.journal_account_id` (pattern `accept_one_split` Pass 1) :
   ```rust
   let journal_account_id: Option<i64> = sqlx::query_scalar(
       "SELECT journal_account_id FROM bank_accounts WHERE id = ? AND company_id = ?"
   ).bind(bank_account_id).bind(company_id).fetch_one(&mut **tx_inner).await?;
   ```
   → **412 `BANK_ACCOUNT_NOT_CONFIGURED`** si NULL (Pass 1 P-C1 ECH-12 — pattern hérité `accept_one_split`).
4. **Vérifier counterparty_account_id mismatch (AC #120)** : `rule.counterparty_account_id != body_counterparty_account_id` → 400 `RECONCILIATION_RULE_MISMATCH` (per-proposal `FailedProposal`).
5. **Vérifier counterparty_account actif** : SELECT inline `accounts.active WHERE id=? AND company_id=?` → si `active=false` ou None → 404 `ACCOUNT_NOT_FOUND` (per-proposal `FailedProposal`).
6. **Pré-validation `tx.amount != 0`** (pattern manual step 4bis 8-5a-base) → 400 `Validation { reason: "zero_amount_transaction" }`.
7. **Re-validation match côté serveur (AC #119)** : appeler `rule_matches(rule, tx)` → si `false` → `failed: [{ errorCode: 'RECONCILIATION_RULE_NO_LONGER_MATCHES' }]` per-proposal. Cas race : la rule a été modifiée (match_type/match_value) entre GET /proposals et POST /accept.
8. **Resolve entry_date** : `entry_date = tx.value_date.unwrap_or(tx.booking_date)` (cohérent manual flow 8-5a-base). Pas de body field `valueDate` v0.1 pour `type='rule'` (l'utilisateur fait confiance à la date tx).
9. **Resolve fiscal_year** : `fiscal_years::find_open_covering_date(tx_inner, company_id, entry_date)` → 409 `RECONCILIATION_FISCAL_YEAR_CLOSED { entry_date }` si None.
10. **Resolve description JE (Pass 1 P-H2 ECH-01)** : construction handler-side :
    ```rust
    let description = format!("Règle '{}' — {}", rule.label, tx.counterparty_name.as_deref().unwrap_or("(sans contrepartie)"));
    ```
    Max 200 chars (cohérent `MAX_MANUAL_DESCRIPTION_LEN` 8-5a-base). Truncate via `.chars().take(200).collect()` si label + counterparty > 200.
11. **Construire NewJournalEntry** : appeler `manual::build_journal_entry_for_counterparty(tx, journal_account_id, rule.counterparty_account_id, description, entry_date)` (helper 8-5a-base stable). Vérifier `tx.amount != 0` est garanti par step 6.
12. **`journal_entries::create_in_tx(tx_inner, fiscal_year.id, user_id, new_je)`** → journal_entry créé (audit `journal_entry.created` émis automatiquement). Capture `journal_entry_id`.
13. **UPDATE bank_transactions optimistic lock** (P3-H1 8-4 pattern) :
    ```sql
    UPDATE bank_transactions
    SET matched_entry_id = ?, status = 'reconciled',
        auto_match_rejected_at = NULL, updated_at = NOW(3),
        version = version + 1
    WHERE id = ? AND company_id = ? AND status = 'pending'
      AND version = ?
    ```
    Si `rows_affected != 1` → `FailedProposal { errorCode: 'RECONCILIATION_ALREADY_RECONCILED', reason: 'race_during_update' }`.
14. **UPDATE rules.applied_count atomique** (Pass 1 P-M-MEDIUM ECH-10 + BH-F19/F20) :
    ```sql
    UPDATE reconciliation_rules
    SET applied_count = applied_count + 1, last_applied_at = NOW(3), version = version + 1
    WHERE id = ? AND company_id = ?
    ```
    **PAS de clause `AND version = ?`** — l'`applied_count` est un compteur statistique, pas un invariant business. Pas d'optimistic lock ici, les accepts concurrents sur la même rule (depuis comptes bancaires différents) ne doivent pas se bloquer. L'advisory lock `with_account_lock(company_id, bank_account_id)` sérialise déjà les accepts sur le même bank_account.
15. **Audit `reconciliation.accepted`** snake_case top-level + sub-objects, shape :
    ```json
    {
      "bank_transaction_id": 43,
      "type": "rule",
      "rule_id": 7,
      "match_type": "counterparty_contains",
      "counterparty_account_id": 6510,
      "amount": "150.00",
      "journal_entry_id": 999,
      "value_date": "2026-05-15",
      "was_previously_rejected": false
    }
    ```
    (Pass 1 P-H3 AA-F8 + BH-F24 : `type` ajouté pour distinguer invoice/split/rule dans le même action ; shape consistent avec `reconciliation.split_applied` 8-5a-bis.)
16. **Audit `reconciliation_rule.applied`** :
    ```json
    {
      "rule_id": 7,
      "bank_transaction_id": 43,
      "match_type": "counterparty_contains",
      "match_value": "Swisscom",
      "applied_count_after": 24,
      "journal_entry_id": 999
    }
    ```
    (Pass 1 P-H3 + BH-F23 : `applied_count_after` snapshot post-incrément du step 14.)

**Response** : `AcceptResponse { accepted: Vec<AcceptedProposal>, failed: Vec<FailedProposal> }`. Le variant `AcceptedProposal` pour `type='rule'` est défini en §api-response-shapes (Pass 1 P-H6).

#### §api-response-shapes — extension enum tagged pour rule

(Pass 1 P-H6 ECH-09 + ECH-15 + BH-F44 + BH-F45 + AA-F6 — refactor `ReconciliationCandidate` et `AcceptedProposal` en enum tagged Serde.)

**Le pattern existant** : `AcceptedProposal` actuel `{ bank_transaction_id, invoice_id, journal_entry_id, score }`. Pour `type='rule'`, `invoice_id` n'a pas de sens — l'utilisation d'un sentinel 0 (comme `AcceptedProposal.invoice_id = 0` pour Split dans 8-5a-bis) est une **dette tracée v0.2** (cf. limitations héritées 8-5a-bis BH-H1). Pour 8-5b, on **garde le pattern sentinel 0 inchangé** : `accepted[i].invoice_id = 0` pour les accepts type=rule. Cohérence avec 8-5a-bis, refactor `Option<i64>` v0.2 transversal.

**`ReconciliationCandidate` extension** : la struct actuelle a `invoice_id`, `invoice_number`, `invoice_amount`, `invoice_date`, `score`. Pour candidate type=rule, ces champs sont `None` / `0` / `""` / `score=1.0` (placeholder). **Refactor recommandé v0.2** : enum tagged response.

**Pour v0.1 8-5b — refactor explicite minimal** (Pass 1 P-H6) :

```rust
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReconciliationCandidate {
    /// Discriminator type — Pass 1 P-H6 : explicite plutôt que sentinel.
    #[serde(rename = "type")]
    pub candidate_type: CandidateType,
    // Champs invoice (None pour candidate type=rule).
    pub invoice_id: Option<i64>,
    pub invoice_number: Option<String>,
    pub invoice_amount: Option<String>,
    pub invoice_date: Option<chrono::NaiveDate>,
    pub score: Option<MatchScore>,
    // Champs rule (None pour candidate type=invoice).
    pub rule_id: Option<i64>,
    pub rule_label: Option<String>,
    pub rule_match_type: Option<String>,
    pub counterparty_account_id: Option<i64>,
    pub counterparty_account_name: Option<String>,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum CandidateType {
    Invoice,
    Rule,
}
```

**Migration des appelants** : tous les sites qui créent un `ReconciliationCandidate` (handler `get_proposals` 8-4) doivent maintenant ajouter `candidate_type: CandidateType::Invoice` + wrap invoice fields dans `Some(...)` + tous les rule fields à `None`. ~3 sites à patcher dans `reconciliation.rs:get_proposals`. Vérifier ground-truth avant dev-story.

**Frontend `reconciliation.types.ts`** : extension symétrique :
```ts
export interface ReconciliationCandidate {
  type: 'invoice' | 'rule';
  invoiceId: number | null;
  invoiceNumber: string | null;
  invoiceAmount: string | null;
  invoiceDate: string | null;
  score: MatchScore | null;
  ruleId: number | null;
  ruleLabel: string | null;
  ruleMatchType: string | null;
  counterpartyAccountId: number | null;
  counterpartyAccountName: string | null;
}
```

#### §audit-log-shapes — 5 actions distinctes (4 rule + 1 extension)

(Pass 1 P-H3 + AA-F2 + AA-F8 + BH-F22-F24 : shapes complets et inline.)

**1. `reconciliation_rule.created`** — émis sur POST /rules.
```json
{
  "rule_id": 7,
  "label": "Swisscom AG → 6510",
  "match_type": "counterparty_contains",
  "match_value": "Swisscom",
  "counterparty_account_id": 6510,
  "priority": 100,
  "active": true
}
```

**2. `reconciliation_rule.updated`** — émis sur PATCH /rules/{id} (cas nominal ET cas réactivation `active=true`).
```json
{
  "rule_id": 7,
  "before": { "label": "Swisscom → 6510", "match_value": "Swisscom", "priority": 100, "active": true },
  "after": { "label": "Swisscom AG → 6510", "match_value": "Swisscom AG", "priority": 50, "active": true }
}
```
Shape : objets `before`/`after` **complets** (snapshot des 4 champs mutables : `label`, `match_value`, `priority`, `active`). Pas un diff partiel — facilite l'audit replay. (Pass 1 BH-F22 + AA-F9 + ECH-11.)

**3. `reconciliation_rule.deleted`** — émis sur DELETE /rules/{id} **seulement si le UPDATE a effet** (i.e. `soft_delete_by_id_for_company` retourne `true`). Pas d'audit redondant sur DELETE idempotent (AC #111). (Pass 1 BH-F07.)
```json
{
  "rule_id": 7,
  "soft_delete": true,
  "before": { "active": true },
  "after": { "active": false }
}
```

**4. `reconciliation_rule.applied`** — émis sur POST /accept type='rule' (cf. step 16 §accept-with-rule-flow).

**5. Extension `reconciliation.accepted`** pour `type='rule'` (cf. step 15 §accept-with-rule-flow).

#### §i18n-keys — liste nominale (Pass 1 P-M BH-F39)

**Préfixe `reconciliation-rules-*` × 12 clés** :
- `reconciliation-rules-page-title` = "Règles d'affectation"
- `reconciliation-rules-button-new` = "+ Nouvelle règle"
- `reconciliation-rules-cols-label` = "Libellé"
- `reconciliation-rules-cols-match-type` = "Type"
- `reconciliation-rules-cols-match-value` = "Valeur"
- `reconciliation-rules-cols-account` = "Compte"
- `reconciliation-rules-cols-priority` = "Priorité"
- `reconciliation-rules-cols-applied-count` = "Appliquée"
- `reconciliation-rules-cols-actions` = "Actions"
- `reconciliation-rules-action-deactivate` = "Désactiver"
- `reconciliation-rules-action-reactivate` = "Réactiver"
- `reconciliation-rules-action-delete` = "Supprimer"

**Préfixe `reconciliation-rule-applied-*` × 3 clés** (UI cards / badge candidate type=rule) :
- `reconciliation-rule-applied-badge` = "Règle"
- `reconciliation-rule-applied-tooltip` = "Cette transaction matche la règle « { $label } »"
- `reconciliation-rule-applied-success-toast` = "Règle appliquée avec succès."

**Préfixe `reconciliation-rules-error-*` × 4 clés** (gestion erreurs API) :
- `reconciliation-rules-error-duplicate-active` = "Une règle active existe déjà avec ce match."
- `reconciliation-rules-error-archived-account` = "Le compte de contrepartie est archivé."
- `reconciliation-rules-error-no-longer-matches` = "La règle ne matche plus la transaction (modification concurrente)."
- `reconciliation-rules-error-mismatch` = "Le compte sélectionné ne correspond pas à la règle."

**Total : 19 clés** (12 page + 3 applied + 4 errors). Cible spec d'origine ~15 — Pass 1 augmente à 19 pour couvrir les erreurs API.

#### §pass-2-clarifications (Pass 2 Haiku Q6-Q10 — clarifications additionnelles)

**Q6 BH-F13 + AA-F4 — Pre-dev check `DbError::UniqueConstraintViolation`** : le handler `update_in_tx` discrimine SQL 1062 selon le nom de la contrainte violée via `DbError::UniqueConstraintViolation(message: String)`. **Pre-check obligatoire avant dev-story** :
```sh
grep -n "UniqueConstraintViolation" crates/kesh-db/src/errors.rs
```
Si le variant existe avec un message capturé : OK, parser le message pour `uq_reconciliation_rules_match_active`. Si pas exist : créer le variant en T2 (extension kesh-db `DbError` enum) — ajouter dans le mapping `From<sqlx::Error>` la branche `SqlState::IntegrityConstraintViolation { code: "23000", ... }` (SQLSTATE 23000 = UNIQUE).

**Q7 AA-F-#110/#111 — DELETE HTTP code aligné sur 204 No Content** : standard REST. AC #110 et AC #111 conjoints :
- **AC #110** (DELETE soft-delete réussi) → **204 No Content** (pas de body, pas de 200 OK).
- **AC #111** (DELETE idempotent déjà inactive) → **204 No Content** également (idempotent UX, le client ne différencie pas).
- Le handler retourne 204 dans les deux cas. La distinction `audit_log émis ou pas` est interne et invisible du client.

**Q8 AA-F-#114 — Invoice override rule seuil** : la spec d'origine §rule-application disait « tx sans candidate score ≥ 0.5 ». Pass 2 clarification :
- **Si la tx a ≥ 1 candidate invoice avec `score >= 0.5`** → ne PAS appliquer les rules (invoice-priority).
- **Si la tx n'a aucune candidate invoice** OU **tous les candidates invoice ont `score < 0.5`** → appliquer les rules, ajouter la candidate type=rule à `candidates[]` (coexistence avec invoice candidates de score < 0.5 si présents).

Modifier AC #114 : « Given tx avec invoice match score **≥ 0.5** ET rule match aussi, Then la candidate est `type: 'invoice'` (rule ignored). Given tx avec invoice match score **< 0.5** ET rule match, Then les 2 candidates coexistent (invoice + rule) dans la response. »

**Q9 AA-F2 — Construction `description` step 11 handler-side** : le **handler `accept_one_rule` construit** la `description` AVANT d'appeler `manual::build_journal_entry_for_counterparty` :
```rust
let counterparty_display = bt.counterparty_name.as_deref().unwrap_or("(sans contrepartie)");
let raw_description = format!("Règle '{}' — {}", rule.label, counterparty_display);
// Truncate UTF-8 safe à 200 chars (cap MAX_MANUAL_DESCRIPTION_LEN 8-5a-base).
let description: String = raw_description.chars().take(200).collect();
```
Le helper `build_journal_entry_for_counterparty` reçoit `description: String` final. Pas de logique de format inside helper (helper reste pure).

**Q10 AA-F-#115b — Test unit #8 tiebreaker ajouté T3.4** : la liste T3.4 contient maintenant 8 tests (au lieu de 7) :
1-7. (inchangés)
8. **NOUVEAU** `first_matching_rule_respects_id_tiebreaker_on_equal_priority` (AC #115b).

#### §error-precedence-order — codes 8-5b (Pass 1 P-C1 — ajout 412 + 422)

| # | Erreur | HTTP | Code |
|---|---|---|---|
| 9 | `bankAccountId` cross-tenant | 404 | `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` (hérité 8-4/8-5a-base) |
| 10 | `bank_account.journal_account_id` NULL | **412** | **`BANK_ACCOUNT_NOT_CONFIGURED`** (Pass 1 P-C1 — hérité 8-5a-base, applicable au flow accept-with-rule) |
| 11 | `bank_transaction` non pending | 404 | `RECONCILIATION_TRANSACTION_NOT_PENDING` (hérité 8-5a-base) |
| 12 | Rule not found / deactivated | 404 | `RECONCILIATION_RULE_NOT_FOUND` |
| 13 | Rule no longer matches (race) | 409 (failed[]) | `RECONCILIATION_RULE_NO_LONGER_MATCHES` |
| 14 | Rule UNIQUE constraint violation | 409 | `RECONCILIATION_RULE_DUPLICATE` |
| 15 | Rule mismatch (proposal.counterpartyAccountId ≠ rule.counterparty_account_id) | 400 | `RECONCILIATION_RULE_MISMATCH` |
| 16 | Counterparty account archivé (au moment du POST /rules ou POST /accept type=rule) | 404 | `ACCOUNT_NOT_FOUND` (hérité 8-5a-base) |
| 17 | `tx.amount == 0` au flow accept-with-rule | 400 | `VALIDATION_ERROR { reason: "zero_amount_transaction" }` (hérité 8-5a-base) |
| 18 | `entry_date` hors fiscal year ouvert | 409 | `RECONCILIATION_FISCAL_YEAR_CLOSED { entry_date }` (hérité 8-5a-base) |

**Ordre de précédence dans `accept_one_rule`** : 1 (tx pending) → 2 (rule found+active) → 3 (bank_account configured) → 4 (counterparty mismatch) → 5 (counterparty active) → 6 (amount != 0) → 7 (rule re-matches) → 8 (fiscal year) → 9 (UPDATE atomic).

#### §rules-types-rust — définition des types (Pass 1 P-M BH-F11 + BH-F12)

```rust
/// Body input pour POST /rules — champs requis.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewReconciliationRule {
    pub label: String,
    pub match_type: ReconciliationMatchType,
    pub match_value: String,
    pub counterparty_account_id: i64,
    /// Optionnel — défaut 100 si absent (cohérent default DDL).
    #[serde(default = "default_priority")]
    pub priority: i32,
}

fn default_priority() -> i32 { 100 }

/// Body input pour PATCH /rules/{id} — champs optionnels (patch partial).
/// Pass 1 BH-F08 + BH-F13 : `expected_version` séparé des champs métier.
/// `match_type` non patchable v0.1 (changer le type briserait la cohérence
/// historique des audit `reconciliation_rule.applied` qui réfèrent à ce type).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateReconciliationRule {
    /// Optimistic lock — version attendue (obligatoire).
    pub expected_version: i32,
    pub label: Option<String>,
    /// match_type NON patchable v0.1 (cf. §risques 8-5b BH-F13).
    /// pub match_type: Option<...> — out-of-scope.
    pub match_value: Option<String>,
    pub counterparty_account_id: Option<i64>,
    pub priority: Option<i32>,
    pub active: Option<bool>,
}
```

**Validation applicative dans `post_create` (Pass 1 P-M ECH-07) avant DB INSERT** :
- `label.trim().is_empty()` → 400 `VALIDATION_ERROR { reason: "label_empty" }` (avant CHECK DB).
- `label.chars().count() > 120` → 400 `VALIDATION_ERROR { reason: "label_too_long" }` (avant troncature SQL).
- `match_value.trim().is_empty()` → 400 (avant CHECK DB).
- `match_value.chars().count() > 255` → 400.
- `priority` hors [1, 1000] → 400 (avant CHECK DB).
- `match_type == IbanExact` → **normaliser** `match_value` à `.trim().to_uppercase().retain(|c| !c.is_whitespace())` AVANT INSERT (Pass 1 P-H5 ECH-03 : IBAN canonique en DB pour matching reproductible).

**Validation applicative dans `post_update` (Pass 2 Q4 ECH2-1) — appliquer aussi à PATCH** : si `body.match_value.is_some()` ET la rule actuelle a `match_type == IbanExact` (ou si `body.match_type` est tenté — interdit v0.1 cf. UpdateReconciliationRule §rules-types-rust), normaliser le nouveau `match_value` identique. Sinon le PATCH stocke un IBAN non-canonique et `rule_matches` IbanExact échouera (même bug que Pass 1 P-H4 sur création, mais à PATCH). Le helper de normalisation IBAN doit être factorisé (e.g. `fn normalize_iban_canonical(s: &str) -> String` dans `kesh_core::iban` ou inline `kesh-api/routes/reconciliation_rules.rs`).

#### §rules-schema — clarifications Pass 1 (MariaDB + reactivation conflict)

**MariaDB version minimale** (Pass 1 P-H3 BH-F05) : 8-5b exige **MariaDB ≥ 10.6** pour le support d'index UNIQUE sur colonne VIRTUAL (cohérent avec Docker Compose qui pin `mariadb:11.x` selon `docker-compose.yml`). Si le projet doit supporter MariaDB ≤ 10.5, fallback Option B (trigger SQL) ou Option C (check applicatif racy). **Décision v0.1** : pin sur 10.6+, Option A retenue.

**Conflit reactivation UNIQUE** (Pass 1 P-H3 ECH-04 + BH-F27) : si rule R1 soft-deleted + rule R2 active créée entre-temps avec mêmes `(company_id, match_type, match_value)`, alors `PATCH R1 { active: true }` viole `uq_reconciliation_rules_match_active` (active_uniq de R1 passe de NULL à `match_value`, qui collide avec R2). Le handler `update_in_tx` doit :

1. Détecter SQL 1062 sur la contrainte `uq_reconciliation_rules_match_active` (parser `DbError::UniqueConstraintViolation(message)` qui contient le nom de la contrainte violée).
2. Retourner `DbError::DuplicateRule { match_type, match_value }`.
3. Handler mappe → `AppError::ReconciliationRuleDuplicate { match_type, match_value }` → 409 `RECONCILIATION_RULE_DUPLICATE`.

**Distinct du conflit optimistic lock version** (qui se manifeste par `rows_affected = 0` sur le UPDATE — pas SQL 1062). Le handler doit discriminer les 2 cas par leur signature SQL respective.

**Nouvel AC #109b** (Pass 1 P-H3 ECH-04 + AA-F3) :
> **(FR47 — PATCH réactivation conflit Q3)** Given rule R1 `(counterparty_contains, 'Swisscom')` soft-deleted ET rule R2 `(counterparty_contains, 'Swisscom')` créée active entre-temps, When PATCH R1 `{ active: true, expectedVersion: <current> }`, Then `409 RECONCILIATION_RULE_DUPLICATE` body `details = { matchType: 'counterparty_contains', matchValue: 'Swisscom' }`. *Test E2E HTTP : `rule_patch_reactivation_fails_when_concurrent_active_rule_exists`.*

**Nouvel AC #115b** (Pass 1 P-L ECH-13) :
> **(FR47 — tiebreaker `id ASC` à même priorité)** Given 2 rules actives matchant la même tx avec `priority=100` chacune, et rule créée 1ère a `id < rule créée 2ème`, When GET, Then la rule `id` plus petit gagne. *Test unit : `first_matching_rule_respects_id_tiebreaker_on_equal_priority`.*

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

110. **(FR47 — DELETE /rules soft delete Q3)** Given DELETE `/rules/{id}`, Then **`204 No Content`** + rule.active=false en DB (**pas DELETE physique**) + audit `reconciliation_rule.deleted` avec `details.soft_delete=true`. La rule disparaît de `GET /rules?active=true` mais reste accessible via `GET /rules?active=false`. Les anciennes entrées audit `reconciliation_rule.applied` avec ce `rule_id` restent valides. *Test E2E HTTP : `rule_delete_soft_deletes_and_preserves_audit_history`.*

111. **(FR47 — DELETE /rules already inactive idempotent)** Given rule `active=false`, When DELETE, Then **`204 No Content`** no-op (idempotent — le `UPDATE active=false` est sans effet, **pas d'audit redondant**, le handler vérifie le retour `bool` de `soft_delete_by_id_for_company` avant émission de l'audit). Pass 2 Q7 : aligné sur 204 (pas 200) cohérent AC #110. *Test E2E HTTP : `rule_delete_idempotent_when_already_inactive`.*

112. **(FR47 — RBAC mutations Comptable+)** Given user `Consultation`, When POST/PATCH/DELETE `/rules`, Then `403`. GET `/rules` → 200 (read-only accessible). *Test E2E HTTP : `rule_mutations_require_comptable_role`.*

### Application des règles dans GET /proposals (FR47 partie 2)

113. **(FR47 — rule applied to tx without invoice candidate)** Given tx pending sans candidate invoice (counterparty `Swisscom AG`) et rule `counterparty_contains:Swisscom → 6510`, When `GET /proposals?bankAccountId=17`, Then la candidate de la tx contient `{ type: 'rule', ruleId: <id>, counterpartyAccountId: 6510, counterpartyAccountName: '6510 Frais télécom' }`. *Test E2E HTTP : `get_proposals_applies_rule_when_no_invoice_candidate`.*

114. **(FR47 — invoice candidate overrides rule au seuil 0.5)** Pass 2 Q8 — clarification seuil. **Given** tx avec **au moins 1 candidate invoice score ≥ 0.5** ET rule match aussi, When GET, Then `candidates[]` ne contient **que** les candidates invoice — la rule est silencieusement skip (invoice-priority dispatch). **Given** tx avec **uniquement des candidates invoice score < 0.5** (top-1 invoice score = 0.3 par ex.) ET rule match, When GET, Then `candidates[]` contient **les invoice candidates < 0.5 ET la candidate rule** (coexistence — le frontend choisit l'affichage prioritaire visuellement). **Given** tx avec **aucune candidate invoice** ET rule match, When GET, Then `candidates[]` contient uniquement la candidate `type='rule'`. *Tests E2E HTTP : `get_proposals_invoice_candidate_overrides_rule_at_threshold_0_5` (score=0.5 → invoice wins) + `get_proposals_invoice_low_score_coexists_with_rule` (score=0.3 → both candidates).*

115. **(FR47 — rule priority order)** Given 2 rules actives matchant la même tx (`Swisscom` priority=200, `Swisscom AG` priority=100), When GET, Then la rule priority=100 gagne. *Test E2E HTTP : `get_proposals_applies_highest_priority_rule`.*

116. **(FR47 — rule skip si counterparty_account archivé)** Given rule active pointant sur compte `active=false`, When GET, Then la rule est ignorée silencieusement (log debug), pas de candidate type=rule. *Test E2E HTTP : `get_proposals_skips_rule_with_archived_account`.*

117. **(FR47 — rule skip si rule désactivée)** Given rule soft-deleted (`active=false`), When GET, Then la rule n'est pas évaluée pour les tx pending. *Test E2E HTTP : `get_proposals_skips_inactive_rule`.*

### Acceptation avec type='rule' (FR47 partie 3)

118. **(FR47 — POST accept type=rule happy)** Given une candidate type=rule sur tx 42 ET `bank_account.journal_account_id` configuré ET tx pending non-reconciled, When POST accept `{ type: 'rule', bankTransactionId: 42, ruleId: 7, counterpartyAccountId: 6510 }`, Then `200 OK` accepted ET journal_entry à 2 lignes créée (via `manual::build_journal_entry_for_counterparty` 8-5a-base) ET `bank_transactions.status='reconciled'` ET `bank_transactions.auto_match_rejected_at=NULL` ET `reconciliation_rules.applied_count` incrémenté +1 ET `last_applied_at` mis à jour ET audit `reconciliation.accepted` (avec `details.type='rule'`, `details.rule_id`, `details.match_type`, **pas** d'`invoice_id` dans details — `invoice_id=0` sentinel dans la response shape v0.1 dette tracée Pass 1 P-H6) + audit `reconciliation_rule.applied` (avec `details.applied_count_after`). *Test E2E HTTP : `accept_with_rule_creates_journal_entry_and_increments_count`.*

118bis. **(FR47 — POST accept type=rule 412 bank_account non configuré)** Pass 1 P-C1 ECH-12 + AA-F1. Given une candidate type=rule sur tx 42 ET `bank_account.journal_account_id IS NULL`, When POST accept `{ type: 'rule', bankTransactionId: 42, ruleId: 7, counterpartyAccountId: 6510 }`, Then **`412 BANK_ACCOUNT_NOT_CONFIGURED`** body `details.bankAccountId = 17` + lien vers `/bank-accounts` (cohérent flow manual 8-5a-base AC #79). Pas de mutation DB (`bank_transactions.status` reste `pending`, `applied_count` reste inchangé). *Test E2E HTTP : `accept_with_rule_rejects_unconfigured_bank_account_with_412`.*

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
          IbanExact => {
              // Pass 1 P-H5 ECH-03 + BH-F29 : IBAN canonique en DB (uppercase,
              // no whitespace) garanti par validation côté handler `post_create`
              // (cf. §rules-types-rust). Pour défense-in-depth, on normalise
              // aussi le côté tx (qui devrait déjà être canonique post-parser
              // CAMT.053). Aucun cas mixte attendu en pratique.
              let normalize_iban = |s: &str| -> String {
                  s.chars().filter(|c| !c.is_whitespace()).collect::<String>().to_uppercase()
              };
              match tx.counterparty_iban.as_deref() {
                  Some(tx_iban) => normalize_iban(tx_iban) == normalize_iban(&rule.match_value),
                  None => false,
              }
          },
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

  /// Normalisation pour `CounterpartyContains` / `CounterpartyExact` /
  /// `ReferenceContains` UNIQUEMENT (matching insensible casse + trim).
  /// Pass 2 Q2 BH-F7 : NE PAS utiliser pour `IbanExact` qui exige
  /// uppercase canonique + strip whitespace (cf. `normalize_iban` inline
  /// dans `rule_matches` branch `IbanExact`).
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

  **Pass 2 Q3 BH-F13 — mapping 1:1 ReconciliationError → AppError requis dans T4.4** :

  | `ReconciliationError` variant | `AppError` variant | HTTP | Code |
  |---|---|---|---|
  | `RuleNotFound { rule_id }` | `ReconciliationRuleNotFound { rule_id }` | 404 | `RECONCILIATION_RULE_NOT_FOUND` |
  | `RuleNoLongerMatches { rule_id }` | mappé en `FailedProposal` per-proposal (pas AppError global, cohérent pattern `accept_one_*` 8-4/8-5a-bis) | 200 + failed[] | `RECONCILIATION_RULE_NO_LONGER_MATCHES` |
  | `RuleMismatch { rule_id, expected_account, actual_account }` | mappé en `FailedProposal` per-proposal | 200 + failed[] | `RECONCILIATION_RULE_MISMATCH` |
  | `RuleDuplicate { match_type, match_value }` | `ReconciliationRuleDuplicate { match_type, match_value }` | 409 | `RECONCILIATION_RULE_DUPLICATE` |

  Helper `impl From<ReconciliationError> for AppError` à étendre dans `kesh-api::errors` pour les variants `RuleNotFound` + `RuleDuplicate` (mapping AppError global). Les variants `RuleNoLongerMatches` + `RuleMismatch` restent **per-proposal** dans `accept_one_rule` → `FailedProposal` (pas convertis en `AppError` global).

- [ ] T3.4 — Tests unit `kesh-reconciliation::rules` (≥ **8**, Pass 2 Q10 +1) :
  1. `rule_matches_counterparty_contains` (AC #113).
  2. `rule_matches_counterparty_exact_normalize_case` (AC #113).
  3. `rule_matches_iban_exact` (AC #113).
  4. `rule_matches_reference_fallback_chain` (AC #113).
  5. `first_matching_rule_respects_priority_order` (AC #115).
  6. `first_matching_rule_skips_inactive_account` (AC #116).
  7. `first_matching_rule_skips_inactive_rule` (AC #117).
  8. **NOUVEAU Pass 2 Q10** `first_matching_rule_respects_id_tiebreaker_on_equal_priority` (AC #115b).

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

- [ ] T4.5 — Tests E2E HTTP `crates/kesh-api/tests/reconciliation_rules_e2e.rs` *(nouveau)* (≥ 17 tests, Pass 1 P-M BH-F01/F02 + AC #109b/#115b/#118bis ajoutés) :
  1. `rule_create_returns_201_with_audit_log` (AC #101).
  2. `rule_create_rejects_duplicate_match_when_active` (AC #102).
  3. `rule_create_succeeds_when_existing_rule_is_inactive` (AC #103, Q3).
  4. `rule_create_rejects_archived_account` (AC #104).
  5. `rule_list_paginated` (AC #105).
  6. `rule_list_filters_active` (AC #106).
  7. `rule_list_scopes_by_company` (AC #107).
  8. `rule_update_uses_optimistic_lock` (AC #108).
  9. `rule_patch_reactivates_inactive_rule` (AC #109).
  10. `rule_patch_reactivation_fails_when_concurrent_active_rule_exists` (**AC #109b nouveau Pass 1**).
  11. `rule_delete_soft_deletes_and_preserves_audit_history` (AC #110).
  12. `rule_delete_idempotent_when_already_inactive` (AC #111).
  13. `rule_mutations_require_comptable_role` (AC #112).
  14. `get_proposals_applies_rule_when_no_invoice_candidate` (AC #113).
  15. `get_proposals_invoice_candidate_overrides_rule` (AC #114).
  16. `get_proposals_applies_highest_priority_rule` (AC #115).
  17. `get_proposals_skips_rule_with_archived_account` (AC #116).
  18. `get_proposals_skips_inactive_rule` (AC #117).
  19. `accept_with_rule_creates_journal_entry_and_increments_count` (AC #118).
  20. `accept_with_rule_rejects_unconfigured_bank_account_with_412` (**AC #118bis nouveau Pass 1**).
  21. `accept_with_rule_rejects_when_no_longer_matches` (AC #119).
  22. `accept_with_rule_validates_counterparty_account_consistency` (AC #120).
  23. `accept_with_rule_handles_concurrent_rule_update` (AC #121).
  24. `accept_with_rule_emits_triple_audit_log` (AC #122).

  **Total : 24 tests E2E HTTP** (12 CRUD + 5 application + 7 accept-with-rule). Compte initial spec d'origine ~14, augmenté par Pass 1 à 24 pour couvrir AC #109b/#118bis + alignement 1 test = 1 AC.

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
- **`fiscal_years::find_open_covering_date`** : helper Story 3-7 (vérifié/créé 8-5a-base T4 + 8-5a-bis T4), indispensable pour résoudre `fiscal_year_id` à partir d'une `entry_date` dans accept-with-rule. **Nom canonique** Pass 1 P-H6 BH-F42 : ne pas confondre avec `find_open_for_date_for_company` (nom obsolète apparaissant dans la spec d'origine).
- **Helper `manual::build_journal_entry_for_counterparty`** : **livré 8-5a-base, signature stable** (`tx, bank_account_journal_id, counterparty_account_id, description: String, entry_date: NaiveDate`), réutilisé tel quel par 8-5b dans le flow `accept-with-rule` step 11.
- **Helper `kesh_db::repositories::reconciliation::find_strictly_pending_by_id_for_account`** : **livré 8-5a-base**, filtre status='pending' explicite. Pass 1 P-H6 BH-F43 : nom canonique — ne pas confondre avec `bank_transactions::find_pending_by_id_for_account` (helper distinct du repo `bank_transactions`, antipattern naming F8'' Pass 3 hérité 8-4, ne PAS utiliser pour le flow 8-5b qui exige le filter strict).
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

5. **Path-dépendance sur 8-5a-bis (post-merge)** : `manual::build_journal_entry_for_counterparty` et `find_strictly_pending_by_id_for_account` sont stables (livrés 8-5a-base, vérifiés intacts post-merge 8-5a-bis 2026-05-12 PR #83). Vérifier signature avant T4 si un patch CR-XXX a été appliqué entre-temps. Si breaking change → CR explicite. (Pass 1 P-H6 — clean up reference obsolete `find_pending_by_id_for_account`.)

6. **Suppression définitive de la suggestion ML (Q5)** : ne pas implémenter `suggest_rule`, ne pas créer endpoint `/rules/suggest`. Si le dev agent voit du code lié dans la spec d'origine, l'ignorer (caduque post-Q5). Tracer en `Completion Notes` que cet aspect est explicitement out-of-scope.

7. **Refactor `ReconciliationCandidate` enum tagged (Pass 1 P-H6)** : 3 sites à patcher dans `reconciliation.rs:get_proposals` qui construisent actuellement des `ReconciliationCandidate` 8-4. Tous les call-sites doivent ajouter `candidate_type: CandidateType::Invoice` + wrap fields invoice dans `Some(...)` + tous rule fields à `None`. Vérifier ground-truth `grep -c "ReconciliationCandidate {" crates/kesh-api/src/routes/reconciliation.rs` avant dev-story.

8. **Variant `Rule` à `AcceptProposalInput` (Pass 1 P-H6 ECH-08)** : l'enum tagged 8-5a-bis a `Invoice` + `Split`. 8-5b ajoute `Rule { bank_transaction_id, rule_id, counterparty_account_id }`. **3 sites à modifier** dans `reconciliation.rs` : (a) déclaration enum + `#[serde(rename = "rule", rename_all = "camelCase")]` ; (b) `impl AcceptProposalInput { fn bank_transaction_id(&self) -> i64 { ... } }` (pattern match exhaustif — compilateur catch) ; (c) `accept_batch` dispatch `match proposal { Invoice => accept_one_invoice, Split => accept_one_split, Rule => accept_one_rule }`. Pass 1 P-H6 — liste explicite.

9. **MariaDB ≥ 10.6 requis (Pass 1 P-H3)** : la VIRTUAL column `active_uniq` indexable UNIQUE exige MariaDB ≥ 10.6 (cohérent Docker Compose `mariadb:11.x`). Si dev local sur version antérieure → migration échoue. Vérifier `SELECT VERSION()` avant T1.

10. **Frontend page rules : filtre UI inactives (Pass 1 P-M BH-F36 + BH-F37)** : le load function charge `active=true` par défaut, mais la page doit aussi afficher les rules désactivées avec bouton « Réactiver ». Ajouter un toggle UI « Afficher les règles désactivées » (state local `showInactive`) qui re-fetch avec `active=` paramètre absent (= toutes). Pas d'AC dédié v0.1 — comportement UI flexible.

11. **Test access audit depuis Playwright (Pass 1 P-M BH-F38)** : T7.1 « vérifier audit » dans le test E2E end-to-end. Comme il n'y a pas de page d'audit UI v0.1, le test Playwright doit faire un appel API direct `GET /api/v1/audit-log` (si endpoint existe — sinon, à vérifier avant dev-story) ou skipper la vérification d'audit côté Playwright (suffisant car T4.5 #19 + #24 couvrent l'audit E2E HTTP).

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

| Date | Entrée | Auteur |
|------|--------|--------|
| **2026-05-12** | **Pass 2 validate Haiku 4.5** — ~50 findings bruts (BH 20 + ECH 15 + AA 15). Verdict Acceptance Auditor : **CONDITIONAL GO** (0 CRITICAL, 0 HIGH bloquant, ~7-10 MEDIUM réserves). **8 patches appliqués** (Q1-Q10) : [Q1 BH-F2] §rule-application order_by tiebreaker aligné 2-level `priority ASC, id ASC` (suppression régression Pass 1 `created_at` 3ème niveau). [Q2 BH-F7] T3.1 docstring `normalize()` clarifié : ne s'applique PAS à `IbanExact` qui utilise `normalize_iban` inline. [Q3 BH-F13] T3.3 table mapping `ReconciliationError` → `AppError` 1:1 ajoutée (4 variants), helper `From<ReconciliationError>` étendu, `RuleNoLongerMatches`/`RuleMismatch` per-proposal vs `RuleNotFound`/`RuleDuplicate` AppError global. [Q4 ECH2-1] §rules-types-rust IBAN normalization étendue à PATCH (pas seulement CREATE) via helper factorisé `normalize_iban_canonical`. [Q5 AA-F1] §rule-application stratégie unifiée : 1 SELECT batch `accounts_info: HashMap<i64, (number, name)>` qui résout `active_account_ids` ET `counterparty_account_name` candidate type=rule (au lieu de 2 queries). [Q6 BH-F13+AA-F4] §pass-2-clarifications pre-dev check obligatoire `grep DbError::UniqueConstraintViolation` avant dev-story + fallback si variant absent (créer en T2 extension kesh-db). [Q7 AA-F-#110/#111] HTTP code DELETE aligné sur **204 No Content** dans les deux cas (cas réussi + cas idempotent), suppression ambiguïté 200/204. [Q8 AA-F-#114] AC #114 enrichi : seuil **score ≥ 0.5** explicite pour invoice override rule + cas coexistence si invoice score < 0.5 (deux candidates retournées). [Q9 AA-F2] §pass-2-clarifications step 11 description handler-side : handler construit `format!("Règle '{}' — {}", rule.label, counterparty)` AVANT helper, truncate UTF-8-safe 200 chars. [Q10 AA-F-#115b] T3.4 test unit #8 `first_matching_rule_respects_id_tiebreaker_on_equal_priority` ajouté nominalement. **Faux positifs dismiss** : BH-F4 sentinel `invoice_id=0` (defer explicite Pass 1 confirmé), BH-F11 step ordering counterparty (logique OK), BH-F17 index already correct, ECH2-2 MariaDB <10.6 fallback (Docker pinned 11.x), ECH2-7 audit JSON size (limite 1GB jamais atteinte). Trend : Pass 1 = ~17 findings > LOW → Pass 2 = ~7 findings > LOW → Pass 2 post-patches = **0 HIGH résiduel**. **Critère arrêt CLAUDE.md NON encore atteint** (MEDIUM/LOW restants à valider par Pass 3 orthogonal). Prochaine étape : Pass 3 Opus 4.7 (cycle Sonnet → Haiku → Opus) pour confirmer convergence. | Claude (Haiku 4.5 validate) |
| **2026-05-12** | **Pass 1 validate Sonnet 4.6** — 72 findings bruts (Blind Hunter 46 + Edge Case Hunter 16 + Acceptance Auditor 10). Triage : 1 CRITICAL + 7 HIGH + ~10 MEDIUM > LOW post-dédup, verdict Acceptance Auditor `CONDITIONAL GO`. **~20 patches appliqués** : [P-C1] §accept-with-rule-flow inline avec 16 steps détaillés (suppression renvoi à spec archivée `8-5-reconciliation-manuelle-regles-affectation.md`) + ajout step 3 check 412 BANK_ACCOUNT_NOT_CONFIGURED (pattern `accept_one_split` 8-5a-bis hérité) + AC #118bis nouveau. [P-H1 BH-F16+AA-F1+ECH-12] résolution `description` + `entry_date` handler-side (format `"Règle '{label}' — {counterparty}"` max 200 chars, `entry_date = tx.value_date.unwrap_or(tx.booking_date)`). [P-H2 ECH-02] §rule-application stratégie `active_account_ids: HashSet<i64>` 1 query batch SELECT inline pré-boucle tx (cohérent SELECT inline `accept_one_split` 8-5a-bis). [P-H3 ECH-04+BH-F27+AA-F3] §rules-schema clarification MariaDB ≥ 10.6 obligatoire (UNIQUE sur VIRTUAL column) + conflit reactivation UNIQUE `uq_reconciliation_rules_match_active` mappé `DbError::DuplicateRule` → `RECONCILIATION_RULE_DUPLICATE` (distinct du conflit optimistic lock version) + nouvel AC #109b. [P-H4 ECH-03+BH-F29+BH-F30] §rules-types-rust normalisation IBAN canonique (`uppercase + strip whitespace`) imposée à la création + défense-in-depth dans `rule_matches` T3.1. [P-H5 BH-F42+BH-F43] helper names canoniques : `find_strictly_pending_by_id_for_account` (PAS `find_pending_by_id_for_account`), `find_open_covering_date` (PAS `find_open_for_date_for_company`) — supprimer divergences §Dev Notes. [P-H6 ECH-09+ECH-15+BH-F44+BH-F45+AA-F6] §api-response-shapes nouvelle section : `ReconciliationCandidate` refactor en struct avec discriminator `candidate_type: CandidateType { Invoice, Rule }` + tous fields invoice/rule en `Option<>` (refactor mineur 3 sites `get_proposals`). `AcceptedProposal` garde sentinel `invoice_id=0` v0.1 (dette transverse tracée v0.2 avec 8-5a-bis BH-H1). Variant `AcceptProposalInput::Rule` à ajouter à l'enum tagged Serde 8-5a-bis (3 sites code listés §risques 8). [P-H7 ECH-05] §rule-application sign filter clarification : les rules s'appliquent aux 2 sens (débit ET crédit), **NE PAS hériter** du sign filter 8-4 invoice `reconciliation.rs:343`. [P-M ECH-07+BH-F33] validation applicative dans `post_create` AVANT INSERT (label/match_value empty + length + priority range) — éviter messages d'erreur SQL bruts. [P-M BH-F11+BH-F12+BH-F13] §rules-types-rust types Rust `NewReconciliationRule` + `UpdateReconciliationRule` (avec `expected_version` séparé des champs métier + `match_type` non patchable v0.1). [P-M AA-F2+AA-F8+BH-F22-F24] §audit-log-shapes 5 actions distinctes shapes complets (4 rule.* + extension `reconciliation.accepted` avec `details.type='rule'`). [P-M BH-F39] §i18n-keys 19 clés nominales listées (cible spec d'origine ~15 augmentée à 19 pour couvrir erreurs API). [P-M ECH-10+BH-F19+BH-F20] `increment_applied_count_in_tx` UPDATE atomique sans optimistic lock sur version (compteur statistique, pas invariant business). [P-M BH-F36+BH-F37] §risques nouveau point 10 toggle UI inactives. [P-M BH-F38] §risques nouveau point 11 Playwright audit fallback. [P-L ECH-13+BH-F44] nouvel AC #115b tiebreaker `id ASC` à même priorité. [P-L BH-F01+BH-F02] T4.5 24 tests E2E HTTP énumérés nommément (au lieu de 14 vagues). Defers : BH-H1 sentinel `invoice_id=0` Split/Rule (refactor `Option<i64>` v0.2 transverse), BH-F09/F10 list/detail response shapes minor, BH-F25/F26 GET /rules/{id} détail couverage (à ajouter si Pass 2 le détecte), ECH-14 DELETE concurrent pendant accept-with-rule (low-prob accepté), BH-F32/F34 défaults FK + priority range (cohérent v0.1). Trend : Pass 1 = ~17+ findings > LOW. **Critère arrêt CLAUDE.md NON atteint** — Pass 2 obligatoire (Haiku 4.5 cycle CLAUDE.md, briser biais Sonnet auteur Pass 1). | Claude (Sonnet 4.6 validate) |

- **2026-05-07** — Spec créée par split mécanique de 8-5 unifiée (décision Guy 2026-05-07 Q1=B). Découpage scope FR47 rules engine (CRUD + application GET /proposals + extension POST /accept type='rule'). FR46 suggestion ML supprimée (Q5 — reportée v0.2). Soft-delete via colonne synthétique active_uniq (Q3 workaround MariaDB). 4 actions audit distinctes (Q4b). 24 ACs (#101-#124). Tasks T1-T8. Path-dépendance sur 8-5a (helper `manual::build_journal_entry_for_counterparty` réutilisé). Status `8-5b-reconciliation-rules-engine: backlog` jusqu'à 8-5a `done`/merged. Cycle prévu après merge 8-5a : transition `backlog → ready-for-dev` puis `bmad-create-story validate 8-5b` Pass 1 Sonnet (cycle CLAUDE.md, auteur=Opus split, briser biais d'auteur).
