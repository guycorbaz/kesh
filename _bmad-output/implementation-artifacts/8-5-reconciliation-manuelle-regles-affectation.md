# Story 8-5: Réconciliation manuelle & règles d'affectation

Status: archived-split

> **⚠️ Spec archivée 2026-05-07 — superseded par split 8-5a + 8-5b (décision Guy)**
>
> Cette spec unifiée a été scindée en deux sous-stories **avant `bmad-create-story validate` Pass 1**, suite à la **décision Guy 2026-05-07** (Q1 = Option B). Justifications cumulées :
> - **Règle CLAUDE.md « splitter si > 5 modules »** : 8-5 unifiée touchait 5-6 modules au seuil critique (`kesh-reconciliation`, `kesh-db`, `kesh-api`, `frontend`, `kesh-i18n`, potentiel `kesh-core`).
> - **Profondeur d'incertitude** : 3 features fonctionnelles distinctes (FR45 manual + FR48 split, FR46 suggestion, FR47 rules engine) — frontière fonctionnelle propre identifiée dans la section « Risque de splitting » ci-dessous (§frontière-split-proposée).
> - **Précédent rétro Epic 7 / Story 7-1** (7 passes review faute de splitting préventif) — précédent applicable.
> - **Volume estimé** ~3 000 lignes — au-dessus du mental-model adversarial fiable single-pass.
>
> **Nouvelles specs actives :**
> - **[`8-5a-reconciliation-manuelle-split.md`](8-5a-reconciliation-manuelle-split.md)** — `ready-for-dev` 2026-05-07 — FR45 (manual match) + FR48 (transaction split). Crée des journal_entries directement à la volée (sans facture pré-existante). Lève les limitations héritées 8-4 L19 / L20 / L23 (partielle). ACs #75-#100 (~26 ACs sur manual+split). Tasks T1-T6.
> - **[`8-5b-reconciliation-rules-engine.md`](8-5b-reconciliation-rules-engine.md)** — `backlog` (transition `ready-for-dev` après 8-5a `done`/merged) — FR47 (rules engine : règles persistées + application en GET /proposals + POST /accept). **Path-dépendance 8-5a** : réutilise le helper `manual::build_journal_entry_for_counterparty` extrait par 8-5a. ACs #101-#124 (~24 ACs). Tasks T1-T7. **FR46 suggestion ML reportée v0.2** (décision Guy Q5 2026-05-07 — pas d'algorithme de suggestion automatique livré v0.1, l'utilisateur crée ses rules manuellement via le CRUD).
>
> **Décisions Guy verrouillées 2026-05-07 (Q1-Q5) impactant 8-5a/b :**
> - **Q1 = Option B** : split confirmé.
> - **Q2 = Breaking change OK** sur `POST /accept` discriminator `type: 'invoice' | 'manual' | 'rule' | 'split'` — pas de backward-compat avec format 8-4 (Kesh pas en production). Pris en charge par **8-5a** (extension `POST /accept` + migration des 21 tests E2E HTTP 8-4 existants pour ajouter `type: 'invoice'` explicite).
> - **Q3 = Soft-delete pour rules** (impact 8-5b uniquement) : `DELETE /rules/{id}` → `UPDATE active=false, updated_at=NOW(3), version=version+1`. UNIQUE partiel `(company_id, match_type, match_value) WHERE active=true` (workaround MariaDB via colonne synthétique ou trigger SQL — design dans 8-5b §rules-schema).
> - **Q4a = Actions distinctes pour match events** : `reconciliation.manual_matched` + `reconciliation.split_applied` (8-5a) ; `reconciliation_rule.applied` (8-5b). Pas de modifiers Vec.
> - **Q4b = Actions CRUD distinctes** (8-5b uniquement) : `reconciliation_rule.created` / `.updated` / `.deleted` (3 actions distinctes pour les mutations de configuration).
> - **Q5 = FR46 suggestion ML reportée v0.2** : 8-5b ne livre PAS d'algorithme de suggestion. Le user crée ses rules manuellement via le CRUD `POST /reconciliation/rules`. Suggestion ML potentiellement traitée via Story 8-5c v0.2 ou Epic 11+.
>
> **Cette spec reste vivante en tant que référence des décisions de conception détaillées** (§manual-match-flow, §rules-schema, §rule-application, §accept-with-rule-flow, §split-flow, §rule-suggestion algorithme — caduque post-Q5 mais préservé pour mémoire, §audit-log-actions, §error-precedence-order). 8-5a et 8-5b citent ces sections par renvoi. **Ne pas modifier cette spec** — toute évolution se fait sur 8-5a ou 8-5b.
>
> **Validations conservées :** aucune passe `bmad-create-story validate` n'a été exécutée sur 8-5 unifiée (split décidé pré-validate). Les 5 questions Q1-Q5 listées ci-dessus capturent les décisions de design Guy qui auraient été émises en passe(s) de validate. 8-5a / 8-5b démarrent leur propre cycle validate post-création.

---

<!-- Note historique : ce fichier était initialement la spec active créée par `bmad-create-story 8-5` (Opus 4.7) post-merge PR #77 (Story 8-4 done). Il a été archivé le 2026-05-07 par split mécanique post-décisions Q1-Q5 Guy. Les sections ci-dessous (Story / Contexte / Scope / Décisions / AC / Tasks / Dev Notes) restent intactes à titre de référence. -->

## Story

As a **utilisateur Kesh (PME / indépendant suisse, comptable interne ou fiduciaire)**,
I want **réconcilier manuellement les transactions bancaires sans candidate auto-matchée en sélectionnant un compte de contrepartie ou en éclatant la transaction agrégée, ET créer/gérer des règles d'affectation automatique pour que les prochains imports soient traités sans saisie**,
so that **mon backlog de transactions `pending` se résorbe (frais bancaires, salaires, cotisations sociales) et que ma fiduciaire (Lisa) atteigne 80% d'auto-affectation au fil des mois (cf. UX scenario PRD §164)**.

### Contexte

**Story 8-5 = cinquième et dernière story de l'Epic 8 « Import Bancaire & Réconciliation »**, après **8-1a/8-1b** (parser CAMT.053 + persistance), **8-2** (CSV multi-encodage profils banque), **8-3** (détection doublons + rejet partiel + KF #70), **8-4** (matching automatique avec score). Elle clôt la **partie B de l'Epic 8** (réconciliation : auto + manuel + règles).

8-5 livre **FR45** (création manuelle de contrepartie pour transactions inconnues), **FR46** (suggestion de règle après affectation manuelle), **FR47** (gestion CRUD des règles d'affectation), **FR48** (éclatement de transaction agrégée). Elle **lève** les limitations connues v0.1 documentées en 8-4 : **L19** (matching journal_entries non-invoice → création directe d'écriture banque/contrepartie), **L20** (création d'écriture sans facture pré-existante), **L23** (`auto_match_rejected_at` non réversible — réversible via création manuelle).

**Status sprint** : `8-5-reconciliation-manuelle-regles-affectation: backlog → ready-for-dev` après création de cette spec.

**Pré-requis closed** :
- ✅ Story 8-4 — `kesh-reconciliation` crate activé (matching, mutex, errors), routes `GET /proposals` + `POST /accept` + `POST /reject`, audit `reconciliation.accepted`/`reconciliation.rejected`, frontend `features/reconciliation/`, schema `bank_transactions { status, matched_entry_id, auto_match_rejected_at }`.
- ✅ Story 6-2 — multi-tenant scoping pattern KF-002 Pattern 1.
- ✅ Story 5-2 — `journal_entries::create_in_tx` (helper transaction-bound, indispensable pour créer une écriture comptable atomiquement avec le UPDATE bank_transactions du flow accept manuel).
- ✅ Stories 4-1, 3-1 — entités `Contact`, `Account` chargées par les selectors UI.

**Crate cible** : extension de `kesh-reconciliation` (modules existants : `matching`, `mutex`, `errors`) avec 3 nouveaux modules : `manual` (FR45 création écriture manuelle), `rules` (FR46/47 moteur de règles), `split` (FR48 éclatement transaction).

### Scope verrouillé — ce qui est livré par 8-5

1. **Création manuelle de contrepartie (FR45)** — nouvelle route `POST /api/v1/reconciliation/manual-match` (sub-router `comptable_routes`). Body : `{ bankAccountId, bankTransactionId, counterpartyAccountId, description?, valueDate? }`. Crée **atomiquement** : (a) une `journal_entry` 2 lignes (compte bancaire ↔ compte de contrepartie) via `journal_entries::create_in_tx`, (b) UPDATE `bank_transactions.status='reconciled'`, `matched_entry_id=<new_je_id>`, `auto_match_rejected_at=NULL` (réversible — lève **L23**), (c) audit log `reconciliation.manual_matched` + `journal_entry.created`. Cf. §manual-match-flow.

2. **Repository `reconciliation_rules` (FR46/FR47)** — nouvelle table `reconciliation_rules { id, company_id, label, match_type ENUM('counterparty_contains','counterparty_exact','reference_contains','iban_exact'), match_value VARCHAR(255), counterparty_account_id, priority INT, active BOOLEAN, applied_count INT DEFAULT 0, last_applied_at, created_at, updated_at, version }`. Helpers Executor générique : `find_active_for_company`, `find_by_id_for_company`, `create_in_tx`, `update_in_tx`, `delete_by_id_for_company`. Cf. §rules-schema.

3. **Routes API CRUD règles (FR47)** :
   - `GET /api/v1/reconciliation/rules?active=true&page=1&perPage=50` — liste paginée toutes règles tenant.
   - `GET /api/v1/reconciliation/rules/{id}` — détail.
   - `POST /api/v1/reconciliation/rules` — crée (Comptable+).
   - `PATCH /api/v1/reconciliation/rules/{id}` — update partiel (Comptable+, optimistic lock `version`).
   - `DELETE /api/v1/reconciliation/rules/{id}` — soft-delete via `active=false` (pas de DELETE physique v0.1, conserve l'historique audit).

4. **Application des règles à l'import** — extension de `GET /api/v1/reconciliation/proposals` (héritée 8-4) : pour chaque tx pending sans candidate score≥0.5, appliquer les `reconciliation_rules` actives de la company **par ordre de priorité strict** (plus petit `priority` = plus prioritaire ; égalité départagée par `id ASC`). Premier match = candidate top-1. La response inclut `appliedRule: { id, label, matchType }` à côté du candidate pour traçabilité UI. **Pas d'auto-acceptation v0.1** : l'utilisateur valide explicitement (cohérent L18 héritée 8-4).

5. **Suggestion de règle post-manual-match (FR46)** — la response de `POST /manual-match` inclut un objet `ruleSuggestion: { matchType, matchValue, counterpartyAccountId, label }` calculé depuis la transaction (e.g. `{ matchType: 'counterparty_contains', matchValue: 'Swisscom', counterpartyAccountId: 6510, label: 'Swisscom → Frais télécom' }` si tx.counterparty_name contient 'Swisscom'). Le frontend affiche un toast « Créer une règle ? » avec bouton qui ouvre un modal pré-rempli (cf. §rule-suggestion).

6. **Éclatement de transaction agrégée (FR48)** — nouvelle route `POST /api/v1/reconciliation/split` (Comptable+). Body : `{ bankAccountId, bankTransactionId, splits: [{ counterpartyAccountId, amount, description }] }` avec `sum(splits[*].amount) === bankTransaction.amount` (Decimal exact, validation backend). Crée **atomiquement** : (a) UNE seule `journal_entry` à N+1 lignes (1 ligne compte bancaire au montant total + N lignes contreparties), (b) UPDATE `bank_transactions.status='reconciled'`, `matched_entry_id=<new_je_id>`, (c) audit log `reconciliation.split_applied` 1 entrée. **Pas de table `bank_transaction_splits` séparée** : la décomposition est portée par la `journal_entry` à N+1 lignes (SSOT comptable, pas de duplication). Cf. §split-flow.

7. **Frontend extensions** :
   - Page `/reconciliation/rules` : CRUD complet règles (liste + form modal create/edit).
   - Composant `ManualMatchModal.svelte` : sélecteur `Account` (autocomplete plan comptable) + textarea description + suggestion de règle post-confirm.
   - Composant `TransactionSplitModal.svelte` : tableau de splits éditable (ajout/suppression de ligne) + indicateur balance live (sum vs tx.amount).
   - Extension de `ReconciliationProposals.svelte` (héritée 8-4) : 2 boutons supplémentaires par ligne tx sans candidate auto : « Affecter manuellement » (ouvre `ManualMatchModal`) + « Éclater » (ouvre `TransactionSplitModal`).

8. **i18n** — ~25 nouvelles clés (`reconciliation-manual-*`, `reconciliation-rules-*`, `reconciliation-split-*`, `reconciliation-suggestion-*`) × 4 locales fr/de/it/en-CH.

9. **Tests** — Unit `kesh-reconciliation::rules` (≥ 8 cas matchType + priorité), Integration `kesh-db::reconciliation_rules` (≥ 5 sqlx multi-tenant), E2E HTTP `kesh-api` (≥ 14 tests : manual-match happy/cross-tenant, rules CRUD, rules application, split happy/imbalance, suggestion), Vitest (≥ 6), Playwright (≥ 3 actifs + axe).

10. **Sync** sprint-status + README + audit log 6 nouvelles actions discriminantes (`reconciliation.manual_matched`, `reconciliation.split_applied`, `reconciliation_rule.created/updated/deleted/applied`).

**HORS scope 8-5 (reportés v0.2 / Epic ultérieur) :**

- **Auto-acceptation des règles à fort score** (e.g. seuil par tenant `auto_accept_threshold = 0.95`) — reporté v0.2 (couplé à L18 héritée 8-4, à traiter conjointement avec configurabilité utilisateur).
- **Règles avec regex** (au-delà des 4 `match_type` v0.1 : `counterparty_contains`, `counterparty_exact`, `reference_contains`, `iban_exact`) — reporté v0.2 si demande utilisateur. Le regex coût en pédagogie + injection SQL côté DB-side LIKE est non-négligeable.
- **Application batch des règles à un dataset historique** (« re-process toutes les tx pending avec les règles actuelles ») — la route `POST /reconciliation/apply-rules` n'est pas livrée v0.1 ; les règles s'appliquent uniquement sur GET /proposals à la volée. Reporté v0.2.
- **Multi-currency** (split avec comptes en devises différentes, règles par devise) — reporté Story 11. Cohérent avec **L38** héritée 8-4.
- **Paiement partiel avec écart documenté** (L21 héritée 8-4) — reporté v0.2. La création manuelle 8-5 permet de poser une écriture banque/perte de change en cas de mismatch montant, mais sans liaison à une invoice spécifique (l'invoice reste impayée).
- **Annulation de réconciliation** (revertir un `accept` ou un `manual-match` à postériori) — reporté v0.2. v0.1 : pas d'undo, l'utilisateur passe par modification d'écriture comptable manuelle (Story 3-3).
- **Liaison split → invoices multiples** (une tx 1500 CHF qui couvre 3 factures de 500 CHF chacune avec liaison bidirectionnelle `bank_transactions.matched_entry_id` ↔ N invoices) — reporté v0.2. v0.1 : split ne lie qu'à des comptes de contrepartie type **frais/produits**, pas à des **factures clients/fournisseurs**. Pour un cas réel multi-factures, l'utilisateur passera par 8-4 accept successif sur des sous-transactions ou contournera via création manuelle FR45.
- **Préview des règles avant CRUD** (montrer combien de tx pending matcheront la nouvelle règle avant de la créer) — reporté v0.2.

### Décisions de conception (clés)

#### §manual-match-flow (FR45 — création écriture manuelle)

`POST /api/v1/reconciliation/manual-match` body :

```json
{
  "bankAccountId": 17,
  "bankTransactionId": 42,
  "counterpartyAccountId": 6510,
  "description": "Frais télécom mai 2026 — Swisscom",
  "valueDate": "2026-05-15"
}
```

**Flow** :

0. Validation body (`bankAccountId > 0`, `bankTransactionId > 0`, `counterpartyAccountId > 0`, `description` ≤ 500 chars trim, `valueDate` parsable ou absent).
1. Pré-flight `bank_accounts::find_by_id_for_company(pool, company_id, bankAccountId)` → 404 si cross-tenant (KF-002 pattern).
2. Pré-flight `accounts::find_by_id_for_company(pool, company_id, counterpartyAccountId)` → 404 si cross-tenant ou archivé.
3. Acquérir `with_account_lock(&mut tx_outer, company_id, bankAccountId, 5)` — réutilise le helper 8-4 §mutex-account (sérialisation entre flows accept manuel + accept auto + reject + split sur le même compte).
4. Charger `bank_transaction` scoped `(company_id, bankAccountId)` via nouveau helper `bank_transactions::find_pending_by_id_for_account(&mut tx_outer, company_id, bankAccountId, bankTransactionId)`. Si `None` → `404 BANK_TRANSACTION_NOT_FOUND`. Si `status != 'pending'` → `409 RECONCILIATION_ALREADY_RECONCILED`.
5. **Construire `NewJournalEntry` à 2 lignes** :
   - `entry_date = valueDate.unwrap_or(bank_transaction.value_date.unwrap_or(bank_transaction.booking_date))`.
   - `description = body.description.unwrap_or_else(|| format!("Réconciliation manuelle — {}", bank_transaction.reference.as_deref().unwrap_or("(sans référence)")))`.
   - `journal = "Banque"` (5 journaux figés FR22, cohérent flow 8-4).
   - **Lignes** :
     - **Cas tx.amount > 0 (crédit titulaire — encaissement)** : ligne 1 débit `bank_account.linked_account_id` (compte comptable du compte bancaire) au montant `tx.amount`, ligne 2 crédit `counterpartyAccountId` au même montant.
     - **Cas tx.amount < 0 (débit titulaire — paiement)** : ligne 1 débit `counterpartyAccountId` au montant `tx.amount.abs()`, ligne 2 crédit `bank_account.linked_account_id` au même montant.
   - L'invariant débit==crédit est garanti par construction (2 lignes équivalentes) ; le helper `journal_entries::create_in_tx` re-vérifie en step 6 (defense-in-depth).
6. Appeler `journal_entries::create_in_tx(&mut tx_outer, fiscal_year_id, current_user.user_id, new_je)` → retourne `JournalEntryWithLines`. **Note `fiscal_year_id`** : résolu par helper `fiscal_years::find_open_for_date_for_company(&mut tx_outer, company_id, entry_date)` ; si aucun exercice ouvert ne couvre `entry_date` → `409 RECONCILIATION_FISCAL_YEAR_CLOSED` (l'utilisateur doit ouvrir un exercice ou choisir une autre date).
7. `UPDATE bank_transactions SET status='reconciled', matched_entry_id=<new_je.id>, auto_match_rejected_at=NULL, version=version+1, updated_at=NOW(3) WHERE id=? AND company_id=? AND status='pending' AND version=?` (optimistic lock defense-in-depth comme 8-4 step 8 §accept-flow ; `auto_match_rejected_at=NULL` permet le scenario « rejet → manual match » qui réversibilise L23).
8. Audit log `reconciliation.manual_matched` :
   - `entity_type='bank_transaction'`, `entity_id=bankTransactionId`.
   - `details = { bank_transaction_id, counterparty_account_id, journal_entry_id, amount: <tx.amount as String>, description, value_date: <entry_date>, was_previously_rejected: <auto_match_rejected_at IS NOT NULL avant UPDATE> }`.
9. **Calculer la suggestion de règle (FR46)** — cf. §rule-suggestion. Si suggestion non-nulle, l'inclure dans la response.
10. `tx_outer.commit()`. Retourner `200 OK` body :

```json
{
  "bankTransactionId": 42,
  "journalEntryId": 999,
  "ruleSuggestion": {
    "matchType": "counterparty_contains",
    "matchValue": "Swisscom",
    "counterpartyAccountId": 6510,
    "label": "Swisscom AG → 6510 Frais télécom"
  }
}
```

`ruleSuggestion` peut être `null` si la transaction n'a ni `counterparty_name` ni `reference` exploitable (cf. §rule-suggestion).

**Note dual audit absent** : contrairement à 8-4 (`reconciliation.accepted` + `invoice.paid`), 8-5 manual-match n'émet PAS un dual audit avec une autre entité. Une seule entrée `reconciliation.manual_matched`. Le `journal_entries::create_in_tx` émet par lui-même un audit `journal_entry.created` (helper Story 3-2) — ce qui constitue déjà la traçabilité comptable.

#### §rules-schema (FR47 — table reconciliation_rules)

Migration T2 :

```sql
CREATE TABLE reconciliation_rules (
    id BIGINT UNSIGNED NOT NULL AUTO_INCREMENT PRIMARY KEY,
    company_id BIGINT UNSIGNED NOT NULL,
    label VARCHAR(120) NOT NULL,
    -- match_type : 4 variantes v0.1, étendable v0.2 (regex, etc.)
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
    CONSTRAINT fk_reconciliation_rules_company FOREIGN KEY (company_id) REFERENCES companies (id),
    CONSTRAINT fk_reconciliation_rules_account FOREIGN KEY (counterparty_account_id) REFERENCES accounts (id),
    -- match_value non-vide après trim, trim côté applicatif avant insert
    CONSTRAINT chk_reconciliation_rules_match_value_non_empty CHECK (CHAR_LENGTH(TRIM(match_value)) > 0),
    -- label non-vide
    CONSTRAINT chk_reconciliation_rules_label_non_empty CHECK (CHAR_LENGTH(TRIM(label)) > 0),
    -- priority dans [1..1000] pour éviter abus / ordre déterministe
    CONSTRAINT chk_reconciliation_rules_priority_range CHECK (priority BETWEEN 1 AND 1000),
    -- Un même match_type+match_value est unique par tenant (évite doublons)
    CONSTRAINT uq_reconciliation_rules_match UNIQUE (company_id, match_type, match_value),
    INDEX idx_reconciliation_rules_company_active_priority (company_id, active, priority, id)
) ENGINE=InnoDB CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci;
```

**Justification design** :
- `applied_count` + `last_applied_at` permettent à l'UI d'afficher les règles « efficaces » (souvent appliquées) vs « obsolètes » (jamais utilisées depuis 6 mois → suggestion archivage v0.2).
- **`uq_reconciliation_rules_match`** garantit qu'on ne crée pas accidentellement deux règles `counterparty_contains:Swisscom` qui se contrediraient. Si l'utilisateur veut quand même 2 règles avec le même match (priorité différente), il doit changer le `match_value` (e.g. `Swisscom AG` vs `Swisscom`).
- **`active=false` au lieu de DELETE physique** : conserve l'audit `reconciliation_rule.applied` historique, l'UI peut filtrer `active=true` par défaut.
- **`priority` dans `[1..1000]`** : 1 = top priorité, 1000 = catch-all. UI propose par défaut 100 et indique « la règle s'appliquera après celles de priorité < 100 ». Si égalité, départage par `id ASC` (ordre de création).
- **Pas de FK cascade** : si l'utilisateur archive un compte (`accounts.active=false`), les règles pointant dessus restent mais leur application échoue silencieusement (cf. §rule-application step 4).

#### §rule-application (FR47 — application des règles dans GET /proposals)

Extension du handler `GET /api/v1/reconciliation/proposals` hérité 8-4 :

**Étape supplémentaire après le 4-pass §accept-flow §candidate-window** (post-Pass D scoring) :

```
Pour chaque tx dont candidates est vide OU score top-1 < 0.5 :
    1. Charger les rules actives de la company en cache (1 query batch en début de handler).
    2. Pour chaque rule en ordre (priority ASC, id ASC) :
        a. Tester `match_type(rule, tx)` :
           - counterparty_contains  → tx.counterparty_name.normalize().contains(rule.match_value.normalize())
           - counterparty_exact     → tx.counterparty_name.normalize() == rule.match_value.normalize()
           - reference_contains     → coalesce(tx.reference, tx.end_to_end_id, tx.transaction_id).normalize().contains(rule.match_value.normalize())
           - iban_exact             → tx.counterparty_iban == rule.match_value (case-sensitive, IBAN canonical)
        b. Si match ET `accounts::find_by_id_for_company(rule.counterparty_account_id, active=true)` retourne Some → push candidate `RuleAppliedCandidate { rule_id, label, matchType, counterparty_account: { id, name }, score: 0.5 (badge jaune ; pas de score amount/reference structuré) }` et BREAK.
        c. Sinon (compte archivé/supprimé) → continuer la boucle (skip silencieux ; log debug).
    3. Si aucune rule ne match → tx reste dans la response avec `candidates: []` (état neutre 8-5 manual).
```

**Position dans la response** :

```json
{
  "proposals": [
    {
      "bankTransactionId": 42,
      "transaction": {...},
      "candidates": [
        {
          "type": "invoice",
          "invoiceId": 101,
          "score": { "total": 0.95, ... }
        }
      ]
    },
    {
      "bankTransactionId": 43,
      "transaction": {...},
      "candidates": [
        {
          "type": "rule",
          "ruleId": 7,
          "ruleLabel": "Swisscom AG → 6510 Frais télécom",
          "matchType": "counterparty_contains",
          "counterpartyAccountId": 6510,
          "counterpartyAccountName": "6510 Frais de télécommunications"
        }
      ]
    },
    {
      "bankTransactionId": 44,
      "transaction": {...},
      "candidates": []
    }
  ]
}
```

**Discriminator `type` dans candidate** :  `"invoice" | "rule"`. Le frontend affiche le label différemment selon le type (invoice → numéro facture + score badge, rule → label règle + compte de contrepartie).

**Pas de double candidate** : si à la fois une invoice match avec score>0 ET une règle match, le **invoice gagne** (score-based dispatch vs déterministe). Justification : une invoice match implique paiement client, sémantiquement plus prioritaire qu'une règle frais bancaires.

**Performance** : la phase rules ajoute O(N×R) opérations (N tx pending, R règles actives). Pour N=100 et R=50, ~5000 string comparisons in-memory — non bloquant. Si R > 200 émerge en pratique, optimiser v0.2 (e.g. trie pré-calculé en mémoire, ou pré-filter par `match_type` indexed).

**Audit `reconciliation_rule.applied`** : émis SEULEMENT au moment de l'`accept` (pas du `GET /proposals`) — cf. §accept-with-rule-flow.

#### §accept-with-rule-flow (acceptation d'une candidate de type rule)

`POST /api/v1/reconciliation/accept` hérité 8-4 étendu : le body `proposals[*]` accepte désormais 2 formes (discriminator `type`) :

```json
{
  "bankAccountId": 17,
  "proposals": [
    { "type": "invoice", "bankTransactionId": 42, "invoiceId": 101 },
    { "type": "rule", "bankTransactionId": 43, "ruleId": 7, "counterpartyAccountId": 6510 }
  ]
}
```

**Backward-compat 8-4** : si `type` absent, on suppose `"invoice"` (compatible avec les clients API 8-4 qui envoient `{ bankTransactionId, invoiceId }` sans `type`). Documenté dans la spec API + Dev Notes.

**Flow `accept_one_with_rule`** (extension du flow 8-4 §accept-flow steps 2-10, **dans la même boucle proposal-par-proposal et savepoint qu'en 8-4**) :

2. Charger `BankTransaction` (scoped) — 404 si introuvable.
3. Vérifier `bank_account_id` cohérence.
4. Vérifier `status == 'pending'` — sinon `409 RECONCILIATION_ALREADY_RECONCILED`.
5. Charger `Rule` scoped `(company_id)` — 404 si introuvable. Vérifier `rule.active = true`. Vérifier `rule.counterparty_account_id == proposal.counterpartyAccountId` (le frontend passe `counterpartyAccountId` redondamment pour cohérence avec `manual-match` flow ; si mismatch → `400 RECONCILIATION_RULE_MISMATCH`, défense contre client incohérent).
6. **Re-vérifier le match côté serveur** — recalculer `match_type(rule, tx)` (cf. §rule-application step 2a). Si `false` → `failed: [{ errorCode: 'RECONCILIATION_RULE_NO_LONGER_MATCHES' }]` (la tx a été modifiée OU la règle a été modifiée entre GET proposals et POST accept ; rare mais possible).
7. Charger `Account` scoped (`counterparty_account_id`) — 404 si introuvable / archivé.
8. **Construire `NewJournalEntry` à 2 lignes** (réutilise §manual-match-flow step 5 — extraction d'un helper `kesh-reconciliation::manual::build_journal_entry_for_counterparty(tx, counterparty_account_id, description, entry_date)`).
9. `journal_entries::create_in_tx(...)` — comme §manual-match-flow step 6.
10. UPDATE `bank_transactions` (`status='reconciled', matched_entry_id, auto_match_rejected_at=NULL, version+1`).
11. UPDATE `reconciliation_rules SET applied_count = applied_count + 1, last_applied_at = NOW(3), version = version + 1 WHERE id = ? AND company_id = ? AND active = TRUE`. **Optimistic lock** sur `rules.version` (defense-in-depth contre concurrent rule update Story 8-5 par admin).
12. Audit log `reconciliation.accepted` (action existante 8-4) — détails enrichis :
    - `details = { bank_transaction_id, counterparty_account_id (NEW), journal_entry_id, batch_size, rule_id (NEW), match_type (NEW) }`.
    - **Pas** d'`invoice_id` ni de `score` structure — discriminés par présence/absence (`rule_id IS NOT NULL` vs `invoice_id IS NOT NULL`).
13. Audit log `reconciliation_rule.applied` 1 entrée par accept de type rule :
    - `entity_type='reconciliation_rule'`, `entity_id=rule.id`.
    - `details = { bank_transaction_id, journal_entry_id, applied_count_after }`.
14. Retourner `AcceptedProposal { bank_transaction_id, journal_entry_id, type: 'rule', rule_id, counterparty_account_id }`.

**Pour les proposals de `type='invoice'`** : flow 8-4 §accept-flow inchangé.

**Note batch heterogène** : un batch peut mélanger invoice et rule proposals — chacun traité dans son propre savepoint (cohérent partial success 8-4).

#### §split-flow (FR48 — éclatement transaction agrégée)

`POST /api/v1/reconciliation/split` body :

```json
{
  "bankAccountId": 17,
  "bankTransactionId": 42,
  "description": "Salaires mai 2026",
  "valueDate": "2026-05-31",
  "splits": [
    { "counterpartyAccountId": 5000, "amount": "5000.00", "description": "Salaire Alice Martin" },
    { "counterpartyAccountId": 5000, "amount": "4500.00", "description": "Salaire Bob Müller" },
    { "counterpartyAccountId": 5700, "amount": "1200.00", "description": "Charges sociales" }
  ]
}
```

(tx 10 700 CHF débit titulaire éclaté en 3 imputations).

**Validation body** :
- `bankAccountId > 0`, `bankTransactionId > 0`.
- `splits.len() >= 2` (un split à 1 ligne = équivalent au manual-match, refusé pour clarté API).
- `splits.len() <= 50` (anti-DoS, cohérent avec batch limits 8-3).
- Pour chaque split : `counterpartyAccountId > 0`, `amount > 0` (montant absolu, le sign du flow est dérivé du sign de tx.amount), `description` ≤ 200 chars.
- **Invariant balance** : `sum(splits[*].amount) == bank_transaction.amount.abs()` (Decimal exact, **pas de tolérance**) — sinon `400 RECONCILIATION_SPLIT_IMBALANCE` avec `details = { expected: <tx.amount.abs()>, actual: <sum>, difference: <delta> }`. La validation strict force l'utilisateur à corriger en UI plutôt que silently absorber l'écart.

**Flow** (savepoint-free car opération unique sans partial success — soit tout réussit, soit tout rollback) :

0. Validation body comme ci-dessus.
1. Pré-flight `bank_accounts::find_by_id_for_company` — 404 si cross-tenant.
2. Pré-flight TOUS les `splits[*].counterpartyAccountId` via 1 query batch `accounts::find_active_by_ids_for_company(pool, company_id, &distinct_ids)` → 404 si l'un est introuvable/archivé, message `details = { missing_account_ids: [...] }`.
3. Acquérir `with_account_lock(&mut tx_outer, company_id, bankAccountId, 5)`.
4. Charger `bank_transaction` scoped `(company_id, bankAccountId)` — 404 si introuvable, 409 si déjà `reconciled`.
5. Vérifier balance Decimal : `sum_splits == tx.amount.abs()`.
6. **Construire `NewJournalEntry` à N+1 lignes** :
   - `entry_date = valueDate.unwrap_or(tx.value_date.unwrap_or(tx.booking_date))`.
   - `description = body.description.unwrap_or_else(|| format!("Éclatement — {}", tx.reference.as_deref().unwrap_or("transaction agrégée")))`.
   - **Cas tx.amount > 0 (encaissement éclaté)** : ligne 1 débit `bank_account.linked_account_id` au montant `tx.amount`, lignes 2..N+1 crédit `splits[i].counterpartyAccountId` au montant `splits[i].amount` chacune.
   - **Cas tx.amount < 0 (paiement éclaté)** : ligne 1 crédit `bank_account.linked_account_id` au montant `tx.amount.abs()`, lignes 2..N+1 débit `splits[i].counterpartyAccountId` au montant `splits[i].amount` chacune.
   - L'invariant débit==crédit est vérifié par construction (sum splits == abs(tx)) ; double-check par `journal_entries::create_in_tx` step 6.
7. `journal_entries::create_in_tx(...)`.
8. `UPDATE bank_transactions SET status='reconciled', matched_entry_id=<new_je.id>, auto_match_rejected_at=NULL, version+1` (optimistic lock).
9. Audit log `reconciliation.split_applied` 1 entrée :
   - `entity_type='bank_transaction'`, `entity_id=bankTransactionId`.
   - `details = { bank_transaction_id, journal_entry_id, splits_count, splits: [{ counterparty_account_id, amount: <Decimal as String>, description }], total_amount: <tx.amount.abs() as String> }`.
10. `tx_outer.commit()`. Retourner `200 OK` body `{ bankTransactionId, journalEntryId, splitsCount }`.

**Pas de `ruleSuggestion` post-split** : l'éclatement est par nature ad-hoc (chaque éclatement est unique, pas de pattern réutilisable). Si l'utilisateur veut une règle pour des splits récurrents (e.g. salaires mensuels), il devra la créer manuellement (la table `reconciliation_rules` v0.1 ne supporte pas les splits — `counterparty_account_id` est singulier).

#### §rule-suggestion (FR46 — suggestion automatique post-manual-match)

Algorithme déterministe pour suggérer une règle après un `POST /manual-match` :

```rust
fn suggest_rule(tx: &BankTransaction, counterparty_account: &Account) -> Option<RuleSuggestion> {
    // Priorité 1 : iban exact (si counterparty_iban présent et valide)
    if let Some(iban) = &tx.counterparty_iban {
        if !iban.trim().is_empty() {
            return Some(RuleSuggestion {
                match_type: MatchType::IbanExact,
                match_value: iban.clone(),
                counterparty_account_id: counterparty_account.id,
                label: format!("{} → {} {}", iban, counterparty_account.number, counterparty_account.name),
            });
        }
    }
    // Priorité 2 : counterparty_contains sur le nom raccourci (premier mot ≥ 4 chars)
    if let Some(name) = &tx.counterparty_name {
        let first_word = name.split_whitespace().next()?.trim();
        if first_word.len() >= 4 {
            return Some(RuleSuggestion {
                match_type: MatchType::CounterpartyContains,
                match_value: first_word.to_string(),
                counterparty_account_id: counterparty_account.id,
                label: format!("{} → {} {}", first_word, counterparty_account.number, counterparty_account.name),
            });
        }
    }
    // Priorité 3 : reference_contains sur le premier token alphanum ≥ 4 chars
    let ref_source = tx.reference.as_deref().or(tx.end_to_end_id.as_deref()).or(tx.transaction_id.as_deref())?;
    let token = ref_source.split_whitespace().next()?.trim();
    if token.len() >= 4 && token.chars().any(|c| c.is_alphabetic()) {
        return Some(RuleSuggestion {
            match_type: MatchType::ReferenceContains,
            match_value: token.to_string(),
            counterparty_account_id: counterparty_account.id,
            label: format!("Réf '{}' → {} {}", token, counterparty_account.number, counterparty_account.name),
        });
    }
    // Aucune suggestion exploitable
    None
}
```

**Justification** :
- **IBAN exact prioritaire** : un IBAN identique implique avec certitude la même contrepartie (paie récurrente, abonnement). Match déterministe.
- **Contains nom > exact nom** : l'utilisateur recevra `Swisscom (Switzerland) AG`, `Swisscom AG`, `SWISSCOM AG` selon les paiements — `contains "Swisscom"` couvre les variations.
- **Premier mot ≥ 4 chars** : évite les suggestions sur des préfixes type « SA », « AG », « DE » qui matcheraient trop largement.
- **Pas de garantie unique** : la suggestion peut entrer en conflit avec une règle existante (UNIQUE `(company_id, match_type, match_value)`). Le frontend gère via le UNIQUE constraint backend qui retourne `409 CONFLICT` au moment du POST `/rules` — l'utilisateur ajuste alors le `match_value` ou abandonne.

**UI** : la response `manual-match` inclut `ruleSuggestion: RuleSuggestion | null`. Le frontend affiche un toast persistant 10s « Voulez-vous créer une règle ? Swisscom → 6510 Frais télécom » avec bouton « Créer » qui ouvre `RuleFormModal` pré-rempli ; l'utilisateur peut ajuster avant submit.

#### §audit-log-actions (8-5)

Symétrique 8-4 (pas de modifiers Vec — chaque action est sémantiquement déterministe). Six nouvelles actions :

| Action | `entity_type` | `entity_id` | `details_json` |
|---|---|---|---|
| `reconciliation.manual_matched` | `bank_transaction` | tx.id | `{ bank_transaction_id, counterparty_account_id, journal_entry_id, amount, description, value_date, was_previously_rejected }` |
| `reconciliation.split_applied` | `bank_transaction` | tx.id | `{ bank_transaction_id, journal_entry_id, splits_count, splits: [...], total_amount }` |
| `reconciliation_rule.created` | `reconciliation_rule` | rule.id | `{ rule_id, label, match_type, match_value, counterparty_account_id, priority }` |
| `reconciliation_rule.updated` | `reconciliation_rule` | rule.id | `{ rule_id, before: {...}, after: {...} }` (champs modifiés uniquement) |
| `reconciliation_rule.deleted` | `reconciliation_rule` | rule.id | `{ rule_id, soft_delete: true }` (active=false) |
| `reconciliation_rule.applied` | `reconciliation_rule` | rule.id | `{ bank_transaction_id, journal_entry_id, applied_count_after }` (1 entrée par accept de type rule) |

**Pas de réutilisation de `reconciliation.accepted` avec marqueur `paid_via='manual'`** : la décision est prise de garder des actions distinctes pour que les consumers BI/audit query puissent filtrer simplement par action. Ajout d'un marqueur `paid_via` aurait imposé à TOUS les consumers existants 8-4 de filtrer le marqueur ; pas un trade-off acceptable. Précédent : 5-4 a établi `paid_via` sur `invoice.paid` pour discriminer 8-4 vs 5-4 (où les actions sont identiques par nature). Ici, sémantiquement, manual-match n'est pas un accept (pas d'invoice candidate) — actions distinctes justifiées.

#### §error-precedence-order (8-5 ajouts)

Ajouts par rapport au tableau §error-precedence-order de 8-4 :

| # | Erreur | HTTP | Code |
|---|---|---|---|
| 9 | Counterparty account not found / archived / cross-tenant | 404 | `ACCOUNT_NOT_FOUND` |
| 10 | Fiscal year closed (manual-match step 6 / split step 7) | 409 | `RECONCILIATION_FISCAL_YEAR_CLOSED` |
| 11 | Split imbalance (sum splits ≠ tx.amount) | 400 | `RECONCILIATION_SPLIT_IMBALANCE` |
| 12 | Rule not found / deactivated | 404 | `RECONCILIATION_RULE_NOT_FOUND` |
| 13 | Rule no longer matches (race entre GET /proposals et POST /accept) | 409 (failed[]) | `RECONCILIATION_RULE_NO_LONGER_MATCHES` |
| 14 | Rule UNIQUE constraint violation (match_type+match_value déjà existant) | 409 | `RECONCILIATION_RULE_DUPLICATE` |
| 15 | Rule mismatch (proposal.counterpartyAccountId ≠ rule.counterparty_account_id) | 400 | `RECONCILIATION_RULE_MISMATCH` |

### Risque de splitting (CLAUDE.md check)

**Modules touchés par 8-5** (count) :

1. `kesh-reconciliation` — 3 nouveaux modules (`manual` build_journal_entry helper, `rules` engine, `split` validator).
2. `kesh-db` — extension repositories `reconciliation`, nouveau repo `reconciliation_rules`, nouveau helper `bank_transactions::find_pending_by_id_for_account`, migration T2.
3. `kesh-api` — 3 nouvelles routes (`manual-match`, `split`, CRUD `/rules`) + extension `accept` (discriminator type), nouvelles AppError variants.
4. `frontend` — page `/reconciliation/rules`, 2 nouveaux modals (`ManualMatchModal`, `TransactionSplitModal`), composant `RuleFormModal`, extensions `ReconciliationProposals.svelte`.
5. `kesh-i18n` — 25 nouvelles clés × 4 locales.
6. **(potentiel `kesh-core`)** — si on extrait des helpers purs (e.g. `suggest_rule`, validation balance split), 6e module.

**Compte serré : 5 ou 6 modules** selon que `kesh-core` est touché. Au seuil critique de la règle CLAUDE.md « > 5 modules ».

**Profondeur d'incertitude** :

- **3 features fonctionnelles distinctes** (manual FR45, rules FR46/47, split FR48) qui pourraient chacune justifier sa propre story.
- **Patterns acquis** : multi-tenant scoping (KF-002 Pattern 1 maîtrisé), audit log atomique (helper 1-8), advisory lock (8-4), Executor générique repos (8-2/8-3/8-4), savepoint partial success (8-3/8-4), CRUD route pattern (5-1, 4-1, 8-2 bank_profiles). **Aucune nouveauté technique majeure** — c'est de la composition.
- **Volume estimé** : ~2 800-3 500 lignes net (vs ~2 100 pour 8-4 et ~3 700 pour 8-3/8-2). Au-dessus de 8-4 mais en-dessous de 8-2 / 8-3, la story la plus grosse de l'epic.

**Décision préliminaire : split préventif RECOMMANDÉ** — proposer à Guy avant `bmad-create-story validate 8-5` Pass 1.

**Frontière de split proposée** :

- **8-5a — Réconciliation manuelle + Split** (FR45 + FR48) : T1 (helper `bank_transactions::find_pending_by_id_for_account`), T2 (helper `kesh-reconciliation::manual::build_journal_entry_for_counterparty`), T3 (route `POST /manual-match`), T4 (route `POST /split` + validator balance), T5 (frontend `ManualMatchModal` + `TransactionSplitModal` + extensions `ReconciliationProposals`), T6 (i18n keys + Vitest), T7 (Playwright + Audit). Volume estimé ~1 500 lignes. Cycle review 2-3 passes attendu.
- **8-5b — Moteur de règles d'affectation** (FR46 + FR47) : T1 (migration `reconciliation_rules` table), T2 (repo `kesh-db::reconciliation_rules`), T3 (helper `kesh-reconciliation::rules::match_against_rule` + suggester `suggest_rule`), T4 (routes CRUD `/rules` + extension `GET /proposals` rule application + extension `POST /accept` rule type), T5 (frontend `/reconciliation/rules` page + `RuleFormModal` + suggestion toast), T6 (i18n + Vitest), T7 (Playwright + Audit). **Path-dépendance sur 8-5a** (réutilise `manual::build_journal_entry_for_counterparty` pour le flow accept-with-rule). Volume estimé ~1 800 lignes. Cycle 2-3 passes.

**Avantages du split 8-5a/b** :
- Chaque sous-story tient dans un mental model adversarial fiable (1 500 / 1 800 lignes vs 3 200).
- 8-5a livre une valeur utilisateur immédiate (résorption backlog tx pending) avant 8-5b qui est une optimisation d'efficacité (Lisa fiduciaire).
- 8-5a peut être merged et utilisé par les utilisateurs PME individuels (Marc, Sophie) ; 8-5b est davantage Lisa-orienté (fiduciaire 8 clients).
- Cycle review réduit (≤ 3 passes par sous-story vs 5+ probable sur 8-5 unifiée).

**Trigger d'arrêt automatique** : si **Pass 4 spec validate 8-5 unifiée** ne converge pas (≥ 1 finding > LOW), splitter rétroactivement selon la frontière ci-dessus.

**Décision finale** prise par **Guy avant `bmad-create-story validate 8-5` Pass 1** :

- **Option A — Story 8-5 unifiée** : tenter 8-5 en une seule story, accepter que le cycle review puisse atteindre 4-5 passes. Justifié si le volume « patterns acquis » domine la complexité réelle (cf. 8-3 qui a tenu sur 6 modules).
- **Option B — Split 8-5a/b avant validate** : créer 8-5a immédiatement, archiver 8-5 unifiée comme `archived-split` (précédent 8-1 → 8-1a/b). Recommandé pour reduce cycle time (estimation 6-7 passes review total via 2 stories vs 8-10 via 1 story).
- **Option C — Réduction de scope 8-5 unifié** : reporter FR48 split en 8-6 v0.2, garder FR45+FR46+FR47 en 8-5 unifiée (manual + rules engine). Justifié si FR48 est jugé non-critique v0.1 (les utilisateurs Marc/Sophie peuvent contourner via plusieurs manual-match).

**Recommandation auteur** : **Option B** (split 8-5a/b) — la volumétrie + le découpage fonctionnel propre (manual+split déjà cohérents, rules indépendant) maximisent les chances de convergence rapide. **Demander à Guy avant validate.**

## Acceptance Criteria

Numérotation continue 8-4 (qui s'arrêtait à 74). Donc 8-5 commence à #75.

### Création manuelle de contrepartie (FR45)

75. **(FR45 — happy paiement débit)** Given une `bank_transaction` `pending` débit `-150.00 CHF` (frais bancaires) sur `bank_account_id=17` lié au compte comptable 1020, et un compte `6810 Frais bancaires` actif, When `POST /api/v1/reconciliation/manual-match { bankAccountId: 17, bankTransactionId: 42, counterpartyAccountId: 6810, description: "Frais TWINT mai" }`, Then `200 OK` body `{ bankTransactionId: 42, journalEntryId: 999, ruleSuggestion: ... }` ET `journal_entries` table contient 1 nouvelle entry à 2 lignes (1020 crédit 150.00 + 6810 débit 150.00) ET `bank_transactions.status='reconciled'`, `matched_entry_id=999`, `auto_match_rejected_at=NULL`. *Test E2E HTTP : `manual_match_creates_journal_entry_for_debit_transaction`.*

76. **(FR45 — happy encaissement crédit)** Given tx pending crédit `+200.00 CHF`, compte contrepartie `7510 Intérêts bancaires`, When manual-match, Then journal_entry à 2 lignes (1020 débit 200 + 7510 crédit 200). *Test E2E HTTP : `manual_match_creates_journal_entry_for_credit_transaction`.*

77. **(FR45 — multi-tenant safety counterparty)** Given user company_A POST manual-match avec `counterpartyAccountId` appartenant à company_B, Then `404 ACCOUNT_NOT_FOUND` (KF-002 pattern, pas 403). *Test E2E HTTP : `manual_match_does_not_leak_cross_tenant_account`.*

78. **(FR45 — multi-tenant safety bank_account)** Given user company_A POST manual-match avec `bankAccountId` appartenant à company_B, Then `404 BANK_ACCOUNT_NOT_FOUND`. *Test E2E HTTP : `manual_match_returns_404_on_cross_tenant_bank_account`.*

79. **(FR45 — already reconciled idempotency)** Given tx déjà `reconciled` (matched_entry_id != NULL), When POST manual-match, Then `409 RECONCILIATION_ALREADY_RECONCILED`. *Test E2E HTTP : `manual_match_rejects_already_reconciled_transaction`.*

80. **(FR45 — fiscal year closed)** Given `entry_date` qui tombe dans un exercice fiscal `Closed`, When manual-match, Then `409 RECONCILIATION_FISCAL_YEAR_CLOSED`. *Test E2E HTTP : `manual_match_rejects_closed_fiscal_year`.*

81. **(FR45 — réversibilise rejet auto, lève L23)** Given tx `pending` avec `auto_match_rejected_at != NULL` (rejetée 8-4), When POST manual-match, Then `200 OK` ET tx update `status='reconciled'`, `auto_match_rejected_at=NULL`, audit `details.was_previously_rejected=true`. *Test E2E HTTP : `manual_match_reverses_auto_rejection`.*

82. **(FR45 — archived counterparty account)** Given `counterpartyAccountId` pointant sur compte `active=false`, When POST manual-match, Then `404 ACCOUNT_NOT_FOUND` (cohérent comportement 8-4 qui exclut les comptes archivés). *Test E2E HTTP : `manual_match_rejects_archived_account`.*

83. **(FR45 — RBAC Comptable+)** Given user `Consultation`, When POST manual-match, Then `403 Forbidden` (sub-router comptable_routes). *Test E2E HTTP : `manual_match_requires_comptable_role`.*

84. **(FR45 — audit log canonique)** Given POST manual-match happy, When commit, Then audit_log contient 2 entrées : `(action='reconciliation.manual_matched', entity_type='bank_transaction', entity_id=42, details = { bank_transaction_id, counterparty_account_id, journal_entry_id, amount, description, value_date, was_previously_rejected })` ET `(action='journal_entry.created', entity_type='journal_entry', entity_id=999)` (émis par `journal_entries::create_in_tx` lui-même, héritage Story 3-2). *Test E2E HTTP : `manual_match_emits_audit_log_pair`.*

### Suggestion de règle post-manual-match (FR46)

85. **(FR46 — suggestion IBAN exact)** Given tx avec `counterparty_iban='CH9300762011623852957'`, `counterparty_name='Swisscom AG'`, When POST manual-match, Then `ruleSuggestion.matchType='iban_exact'`, `matchValue='CH9300762011623852957'`. *Test E2E HTTP : `manual_match_suggests_iban_rule_when_iban_present`.*

86. **(FR46 — suggestion counterparty_contains fallback)** Given tx avec `counterparty_iban=NULL`, `counterparty_name='Swisscom (Switzerland) AG'`, When POST manual-match, Then `ruleSuggestion.matchType='counterparty_contains'`, `matchValue='Swisscom'` (premier mot ≥ 4 chars). *Test E2E HTTP : `manual_match_suggests_counterparty_contains_when_iban_absent`.*

87. **(FR46 — suggestion reference fallback)** Given tx avec `counterparty_iban=NULL`, `counterparty_name=NULL`, `reference='SALARY-2026-MAY-ALICE'`, When POST manual-match, Then `ruleSuggestion.matchType='reference_contains'`, `matchValue='SALARY-2026-MAY-ALICE'` (premier token alphanum ≥ 4 chars). *Test E2E HTTP : `manual_match_suggests_reference_when_counterparty_absent`.*

88. **(FR46 — pas de suggestion si tx anonyme)** Given tx avec `counterparty_iban=NULL`, `counterparty_name=NULL`, `reference=NULL`, `end_to_end_id=NULL`, `transaction_id='AB1'` (≤ 4 chars), When POST manual-match, Then `ruleSuggestion=null`. *Test E2E HTTP : `manual_match_returns_null_suggestion_when_no_exploitable_field`.*

### Règles d'affectation CRUD (FR47)

89. **(FR47 — POST /rules happy)** Given user Comptable, body `{ label: 'Swisscom AG → 6510', matchType: 'counterparty_contains', matchValue: 'Swisscom', counterpartyAccountId: 6510, priority: 100 }`, When POST `/api/v1/reconciliation/rules`, Then `201 Created` + body `{ id, ..., active: true, appliedCount: 0, lastAppliedAt: null }` + audit `reconciliation_rule.created`. *Test E2E HTTP : `rule_create_returns_201_with_audit_log`.*

90. **(FR47 — POST /rules duplicate UNIQUE)** Given une rule existante `(company_id, 'counterparty_contains', 'Swisscom')`, When POST avec mêmes match_type+match_value, Then `409 RECONCILIATION_RULE_DUPLICATE`. *Test E2E HTTP : `rule_create_rejects_duplicate_match`.*

91. **(FR47 — POST /rules archived counterparty_account)** Given `counterpartyAccountId` pointant sur compte archivé, When POST, Then `404 ACCOUNT_NOT_FOUND`. *Test E2E HTTP : `rule_create_rejects_archived_account`.*

92. **(FR47 — GET /rules pagination)** Given 25 rules pour la company, When `GET /rules?page=1&perPage=10`, Then `items.length=10`, `total=25`. *Test E2E HTTP : `rule_list_paginated`.*

93. **(FR47 — GET /rules filter active)** Given 5 active + 3 inactive rules, When `GET /rules?active=true`, Then `items.length=5`. *Test E2E HTTP : `rule_list_filters_active`.*

94. **(FR47 — GET /rules multi-tenant)** Given rule de company_B, When user company_A `GET /rules`, Then la rule company_B n'apparaît pas. *Test E2E HTTP : `rule_list_scopes_by_company`.*

95. **(FR47 — PATCH /rules optimistic lock)** Given rule `version=3`, When PATCH avec `version=2` (stale), Then `409 OPTIMISTIC_LOCK_VIOLATION`. Given PATCH avec `version=3`, Then `200 OK` + `version=4` + audit `reconciliation_rule.updated` avec `details.before` / `details.after` champs modifiés. *Test E2E HTTP : `rule_update_uses_optimistic_lock`.*

96. **(FR47 — DELETE /rules soft delete)** Given DELETE `/rules/{id}`, Then `204 No Content` + rule.active=false en DB (pas DELETE physique) + audit `reconciliation_rule.deleted` avec `details.soft_delete=true`. La rule disparaît de `GET /rules?active=true`. *Test E2E HTTP : `rule_delete_soft_deletes_and_emits_audit`.*

97. **(FR47 — DELETE /rules already inactive)** Given rule `active=false`, When DELETE, Then `200 OK` no-op (idempotent). *Test E2E HTTP : `rule_delete_idempotent_when_already_inactive`.*

98. **(FR47 — RBAC mutations Comptable+)** Given user `Consultation`, When POST/PATCH/DELETE `/rules`, Then `403`. GET `/rules` → 200 (read-only accessible). *Test E2E HTTP : `rule_mutations_require_comptable_role`.*

### Application des règles dans GET /proposals (FR47 partie 2)

99. **(FR47 — rule applied to tx without invoice candidate)** Given tx pending sans candidate invoice (counterparty `Swisscom AG`) et rule `counterparty_contains:Swisscom → 6510`, When `GET /proposals?bankAccountId=17`, Then la candidate de la tx contient `{ type: 'rule', ruleId: <id>, counterpartyAccountId: 6510 }`. *Test E2E HTTP : `get_proposals_applies_rule_when_no_invoice_candidate`.*

100. **(FR47 — invoice candidate overrides rule)** Given tx avec invoice match `score=0.95` ET rule match aussi, When GET, Then la candidate est `{ type: 'invoice', ... }` (rule ignored, score-based dispatch). *Test E2E HTTP : `get_proposals_invoice_candidate_overrides_rule`.*

101. **(FR47 — rule priority order)** Given 2 rules actives matchant la même tx (`Swisscom` priority=200, `Swisscom AG` priority=100), When GET, Then la rule priority=100 gagne. *Test E2E HTTP : `get_proposals_applies_highest_priority_rule`.*

102. **(FR47 — rule skip si counterparty_account archivé)** Given rule active pointant sur compte `active=false`, When GET, Then la rule est ignorée silencieusement (log debug), pas de candidate type=rule. *Test E2E HTTP : `get_proposals_skips_rule_with_archived_account`.*

103. **(FR47 — POST accept type=rule happy)** Given une candidate type=rule sur tx 42, When POST accept `{ type: 'rule', bankTransactionId: 42, ruleId: 7, counterpartyAccountId: 6510 }`, Then `200 OK` accepted ET journal_entry à 2 lignes créée ET `bank_transactions.status='reconciled'` ET `reconciliation_rules.applied_count` incrémenté ET audit `reconciliation.accepted` (avec rule_id) + audit `reconciliation_rule.applied`. *Test E2E HTTP : `accept_with_rule_creates_journal_entry_and_increments_count`.*

104. **(FR47 — POST accept type=rule re-validation match)** Given une candidate type=rule retournée par GET, modification de la rule entre GET et POST (e.g. match_value changé), When POST accept, Then `failed: [{ errorCode: 'RECONCILIATION_RULE_NO_LONGER_MATCHES' }]`. *Test E2E HTTP : `accept_with_rule_rejects_when_no_longer_matches`.*

105. **(FR47 — POST accept type=rule mismatch counterpartyAccountId)** Given body proposal avec `counterpartyAccountId=9999` mais rule.counterparty_account_id=6510, When POST, Then `400 RECONCILIATION_RULE_MISMATCH`. *Test E2E HTTP : `accept_with_rule_validates_counterparty_account_consistency`.*

106. **(FR47 — backward compat 8-4 type omitted = invoice)** Given body proposal `{ bankTransactionId, invoiceId }` (sans `type`), When POST, Then traité comme `type='invoice'` (flow 8-4 inchangé). *Test E2E HTTP : `accept_legacy_8_4_format_still_works`.*

### Éclatement de transaction agrégée (FR48)

107. **(FR48 — split happy paiement)** Given tx pending débit `-10700.00`, body `splits: [{ accountId: 5000, amount: 5000, description: 'Salaire Alice' }, { accountId: 5000, amount: 4500, description: 'Salaire Bob' }, { accountId: 5700, amount: 1200, description: 'Charges' }]`, When POST `/split`, Then `200 OK` + 1 journal_entry à 4 lignes (1020 crédit 10700 + 5000 débit 5000 + 5000 débit 4500 + 5700 débit 1200) + `bank_transactions.status='reconciled'`. *Test E2E HTTP : `split_creates_journal_entry_with_n_plus_1_lines`.*

108. **(FR48 — split balance violation)** Given tx `-10700.00`, splits sum=10500 (200 missing), When POST split, Then `400 RECONCILIATION_SPLIT_IMBALANCE` body `details = { expected: '10700.00', actual: '10500.00', difference: '-200.00' }`. *Test E2E HTTP : `split_rejects_imbalanced_payload`.*

109. **(FR48 — split min 2 lignes)** Given splits.len=1, When POST, Then `400 Validation` (« splits doit contenir ≥ 2 lignes — utilisez /manual-match pour 1 ligne »). *Test E2E HTTP : `split_rejects_single_line_payload`.*

110. **(FR48 — split max 50 lignes)** Given splits.len=51, When POST, Then `400 Validation` (« splits ≤ 50 lignes »). *Test E2E HTTP : `split_rejects_too_many_lines`.*

111. **(FR48 — split multi-tenant safety)** Given un `splits[i].counterpartyAccountId` appartient à company_B, When POST, Then `404 ACCOUNT_NOT_FOUND` body `details.missing_account_ids = [<id>]`. *Test E2E HTTP : `split_does_not_leak_cross_tenant_account`.*

112. **(FR48 — split déjà réconciliée)** Given tx `reconciled`, When POST, Then `409 RECONCILIATION_ALREADY_RECONCILED`. *Test E2E HTTP : `split_rejects_already_reconciled`.*

113. **(FR48 — split audit log)** Given POST split happy 3 lignes, When commit, Then audit `(action='reconciliation.split_applied', entity_id=tx.id, details = { ..., splits: [...3 entries...], total_amount: '10700.00' })`. *Test E2E HTTP : `split_emits_audit_log`.*

### UI frontend extensions

114. **(UI — bouton Affecter manuellement)** Given une ligne tx pending sans candidate sur `/reconciliation`, Then 2 boutons additionnels « Affecter manuellement » et « Éclater » apparaissent à droite de la ligne. *Test Vitest : `ReconciliationProposals.test.ts: shows manual+split buttons for tx without candidate`.*

115. **(UI — ManualMatchModal)** Given click « Affecter manuellement » sur tx 42, Then modal ouvert avec sélecteur Account (autocomplete plan comptable) + textarea description + datepicker valueDate (pré-rempli = tx.value_date ?? tx.booking_date). *Test Vitest + Playwright : `manual_match_modal_renders_with_prefilled_fields`.*

116. **(UI — TransactionSplitModal)** Given click « Éclater » sur tx 42 (-10700), Then modal ouvert avec tableau splits éditable + sticker balance « 0.00 / 10 700.00 CHF » live (mis à jour à chaque input). Valid quand sum match. *Test Vitest : `split_modal_balance_indicator_updates_live`.*

117. **(UI — page /reconciliation/rules CRUD)** Given user Comptable navigue `/reconciliation/rules`, Then table des règles + bouton « + Nouvelle règle » + actions par ligne (Edit, Désactiver, Supprimer). Form modal `RuleFormModal` valide avant submit (label non-vide, match_value non-vide, counterparty_account_id sélectionné). *Test Playwright : `rules_crud_end_to_end`.*

118. **(UI — toast suggestion post manual-match)** Given POST manual-match retourne `ruleSuggestion`, Then toast 10s avec bouton « Créer la règle » qui ouvre `RuleFormModal` pré-rempli avec les valeurs suggérées. *Test Playwright : `manual_match_suggestion_toast_creates_prefilled_rule`.*

119. **(UI — accessibilité a11y axe)** Given `/reconciliation/rules` rendue avec ≥ 5 règles, When axe-core scan, Then 0 violation. Idem pour `ManualMatchModal` ouvert et `TransactionSplitModal` ouvert. *Tests Playwright : `accessibility — rules page axe scan`, `accessibility — modal manual-match axe scan`, `accessibility — modal split axe scan`.*

### Sécurité & multi-tenant

120. **(KF-002 Pattern 1 — toutes nouvelles tables/repos scoped company_id)** Given user company_A, When tous les helpers `reconciliation_rules::*` sont appelés, Then ils filtrent **systématiquement** par `(company_id, ...)`. *Tests sqlx couverts par AC #94.*

121. **(RBAC — toutes routes mutations sub-router comptable_routes)** Routes `POST/PATCH/DELETE /rules` + `POST /manual-match` + `POST /split` + `POST /accept` (extension) sont sous comptable_routes. GET routes sont sous authenticated_routes. *Test E2E HTTP : `all_8_5_mutations_require_comptable_role` (consolide AC #83 + #98).*

### i18n & accessibilité

122. **(i18n — 25 nouvelles clés × 4 locales)** Given les nouvelles clés `reconciliation-manual-*` (5), `reconciliation-rules-*` (12), `reconciliation-split-*` (5), `reconciliation-suggestion-*` (3), When `npm run lint-i18n-ownership`, Then PASS sur fr/de/it/en-CH. *Test : CI Story 6-3.*

### Performance & limites

123. **(perf — application règles O(N×R) sur 100×50 < 50ms)** Given 100 tx pending et 50 rules actives, When le helper `apply_rules_to_pending_transactions` est appelé in-memory, Then durée totale < 50ms (pure CPU, pas d'I/O). *Test unitaire `kesh-reconciliation::rules` : `apply_rules_handles_100_x_50_under_50ms`.*

124. **(perf — split max 50 lignes < 200ms commit)** Given un POST split avec 50 lignes max, When commit, Then la latence end-to-end < 200ms (1 INSERT journal_entry header + 51 INSERT lines + 1 UPDATE bank_transactions + 1 audit_log INSERT, sous lock). *Smoke test E2E HTTP non-bloquant CI (warning si > 200ms).*

## Tasks / Subtasks

### T1. Migration DB (`reconciliation_rules` + helper `find_pending_by_id_for_account`) (AC #75-#80, #89-#98)

- [ ] T1.1 — Créer `crates/kesh-db/migrations/20260508000001_reconciliation_rules.sql` :
  - `CREATE TABLE reconciliation_rules` avec contraintes (cf. §rules-schema).
  - Index `idx_reconciliation_rules_company_active_priority`.
  - UNIQUE `uq_reconciliation_rules_match`.
- [ ] T1.2 — Mettre à jour `crates/kesh-db/src/test_fixtures.rs` `TABLES_TO_TRUNCATE` const → ajouter `"reconciliation_rules"` (lesson 8-1b retro).
- [ ] T1.3 — Vérifier `cargo test -p kesh-db --lib test_fixtures` avec MariaDB up + `KESH_TEST_MODE=true` (lesson 8-3 retro CHECK constraints invisibles sans DB).
- [ ] T1.4 — Vérifier `EXPLAIN SELECT ... FROM reconciliation_rules WHERE company_id=? AND active=TRUE ORDER BY priority` post-migration — confirmer `type=ref` ou `range`, pas `ALL`.

### T2. Repository `kesh-db::reconciliation_rules` (AC #89-#98, #120)

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
  - `create_in_tx(tx, company_id, user_id, NewReconciliationRule) -> Result<ReconciliationRule, DbError>`.
  - `update_in_tx(tx, company_id, id, expected_version, UpdateReconciliationRule) -> Result<ReconciliationRule, DbError>` (optimistic lock).
  - `soft_delete_by_id_for_company(tx, company_id, id) -> Result<bool, DbError>` (`UPDATE active=false`).
  - `increment_applied_count_in_tx(tx, company_id, rule_id) -> Result<(), DbError>` (`UPDATE applied_count = applied_count+1, last_applied_at = NOW(3), version=version+1`).
- [ ] T2.3 — Mettre à jour `crates/kesh-db/src/repositories/mod.rs` (`pub mod reconciliation_rules`) et `entities/mod.rs`.
- [ ] T2.4 — Tests `#[sqlx::test]` (≥ 5) `crates/kesh-db/tests/reconciliation_rules_repository.rs` :
  1. `create_and_find_by_id_scopes_by_company` (AC #89, #94, #120).
  2. `unique_match_type_value_per_company` (AC #90).
  3. `list_filters_active` (AC #92, #93).
  4. `update_uses_optimistic_lock` (AC #95).
  5. `soft_delete_sets_active_false` (AC #96, #97).
  6. `increment_applied_count_atomic` (couvre AC #103 partiellement).

### T3. Helper `bank_transactions::find_pending_by_id_for_account` (AC #75-#83, #107-#113)

- [ ] T3.1 — Étendre `crates/kesh-db/src/repositories/bank_transactions.rs` :
  ```rust
  /// Charge une transaction `pending` par id, scopée tenant + compte.
  /// Utilisé par /manual-match, /split, et /accept pour pré-flight ownership.
  pub async fn find_pending_by_id_for_account<'e, E>(
      executor: E,
      company_id: i64,
      bank_account_id: i64,
      id: i64,
  ) -> Result<Option<BankTransaction>, DbError>
  where E: sqlx::Executor<'e, Database = MySql>,
  ```
- [ ] T3.2 — Test inline `#[sqlx::test]` (≥ 2) :
  1. `find_pending_by_id_scopes_by_account_and_company`.
  2. `find_pending_by_id_returns_none_for_reconciled_tx`.

### T4. Helpers `kesh-reconciliation::manual` + `rules` + `split` (AC #75-#76, #85-#88, #99-#106, #107, #123)

- [ ] T4.1 — Créer `crates/kesh-reconciliation/src/manual.rs` :
  ```rust
  use kesh_core::accounting::NewJournalEntry;
  use kesh_db::entities::{BankTransaction, Account};
  use rust_decimal::Decimal;
  use chrono::NaiveDate;

  /// Construit une `NewJournalEntry` à 2 lignes (ou N+1 lignes dans le cas split)
  /// pour réconciliation manuelle ou split. Pure (zéro I/O).
  /// Le caller fournit le `journal_account_id` (compte comptable lié au bank_account).
  pub fn build_journal_entry_for_counterparty(
      tx: &BankTransaction,
      bank_account_journal_id: i64,
      counterparty_account_id: i64,
      description: String,
      entry_date: NaiveDate,
  ) -> NewJournalEntry { ... }

  /// Variante N+1 lignes pour split (FR48).
  pub fn build_journal_entry_for_split(
      tx: &BankTransaction,
      bank_account_journal_id: i64,
      splits: &[(i64 /* account_id */, Decimal, String /* description */)],
      description: String,
      entry_date: NaiveDate,
  ) -> NewJournalEntry { ... }
  ```
- [ ] T4.2 — Créer `crates/kesh-reconciliation/src/rules.rs` :
  ```rust
  use kesh_db::entities::{BankTransaction, ReconciliationRule, ReconciliationMatchType, Account};

  /// Teste si une règle match une tx. Pure.
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
  /// ET dont counterparty_account_id existe (caller gère la check active+exists via accounts_by_id).
  pub fn first_matching_rule<'a>(
      rules: &'a [ReconciliationRule],
      tx: &BankTransaction,
      active_account_ids: &std::collections::HashSet<i64>,
  ) -> Option<&'a ReconciliationRule> { ... }

  /// Suggestion de règle déterministe post-manual-match.
  pub fn suggest_rule(tx: &BankTransaction, counterparty_account: &Account) -> Option<RuleSuggestion> {
      // Cf. §rule-suggestion algorithm
  }

  fn normalize(s: &str) -> String { /* trim + lowercase */ }
  fn match_contains(haystack: Option<&str>, needle: &str) -> bool { /* normalize both, .contains */ }
  fn match_exact(haystack: Option<&str>, needle: &str) -> bool { /* normalize both, == */ }

  pub struct RuleSuggestion {
      pub match_type: ReconciliationMatchType,
      pub match_value: String,
      pub counterparty_account_id: i64,
      pub label: String,
  }
  ```
- [ ] T4.3 — Créer `crates/kesh-reconciliation/src/split.rs` (validateur balance pure) :
  ```rust
  /// Vérifie que sum(splits[*].amount) == tx.amount.abs() (Decimal exact).
  pub fn validate_split_balance(tx_amount: Decimal, splits: &[Decimal]) -> Result<(), SplitImbalance> {
      let sum: Decimal = splits.iter().sum();
      let expected = tx_amount.abs();
      if sum != expected {
          return Err(SplitImbalance { expected, actual: sum, difference: sum - expected });
      }
      Ok(())
  }
  pub struct SplitImbalance { pub expected: Decimal, pub actual: Decimal, pub difference: Decimal }
  ```
- [ ] T4.4 — Étendre `crates/kesh-reconciliation/src/lib.rs` :
  ```rust
  pub mod manual;
  pub mod rules;
  pub mod split;

  pub use manual::{build_journal_entry_for_counterparty, build_journal_entry_for_split};
  pub use rules::{rule_matches, first_matching_rule, suggest_rule, RuleSuggestion};
  pub use split::{validate_split_balance, SplitImbalance};
  ```
- [ ] T4.5 — Étendre `crates/kesh-reconciliation/src/errors.rs` (variants ajoutés) :
  ```rust
  pub enum ReconciliationError {
      // ... 8-4 variants conservés
      RuleNoLongerMatches { rule_id: i64 },
      RuleMismatch { rule_id: i64, expected_account: i64, actual_account: i64 },
      RuleDuplicate { match_type: String, match_value: String },
      SplitImbalance { expected: Decimal, actual: Decimal, difference: Decimal },
      FiscalYearClosed { entry_date: NaiveDate },
  }
  ```
- [ ] T4.6 — Tests unit `kesh-reconciliation` (≥ 8) :
  1. `manual_build_je_creates_2_lines_for_credit_tx` (AC #76).
  2. `manual_build_je_creates_2_lines_for_debit_tx` (AC #75).
  3. `split_build_je_creates_n_plus_1_lines` (AC #107).
  4. `split_validate_balance_exact_match_ok` (AC #107).
  5. `split_validate_balance_imbalance_returns_error` (AC #108).
  6. `rule_matches_counterparty_contains` (AC #99).
  7. `rule_matches_iban_exact` (AC #85).
  8. `first_matching_rule_respects_priority_order` (AC #101).
  9. `first_matching_rule_skips_inactive_account` (AC #102).
  10. `suggest_rule_iban_exact_when_iban_present` (AC #85).
  11. `suggest_rule_counterparty_fallback_when_iban_absent` (AC #86).
  12. `suggest_rule_returns_none_when_no_exploitable_field` (AC #88).
  13. `apply_rules_handles_100_x_50_under_50ms` (AC #123).

### T5. Routes API (AC #75-#83, #89-#98, #99-#106, #107-#113, #121)

- [ ] T5.1 — Créer `crates/kesh-api/src/routes/reconciliation_rules.rs` (parallèle à `bank_profiles.rs` pattern CRUD) :
  - `pub fn router_mutations() -> Router<AppState>` : POST `/api/v1/reconciliation/rules`, PATCH/DELETE `/api/v1/reconciliation/rules/{id}` (sub-router comptable_routes).
  - `pub fn router_reads() -> Router<AppState>` : GET list+detail (sub-router authenticated_routes).
  - Handlers : `create`, `list`, `detail`, `update` (PATCH partial avec `If-Match` ou `expected_version` body), `delete` (soft).
- [ ] T5.2 — Étendre `crates/kesh-api/src/routes/reconciliation.rs` (du 8-4) :
  - Handler `post_manual_match` (cf. §manual-match-flow).
  - Handler `post_split` (cf. §split-flow).
  - Modifier `post_accept` pour gérer `type='rule'` (cf. §accept-with-rule-flow). Backward-compat : si `type` absent, traiter `'invoice'`.
  - Modifier `get_proposals` pour appliquer les rules en fallback (cf. §rule-application).
- [ ] T5.3 — Étendre `crates/kesh-api/src/lib.rs` mounting :
  - `comptable_routes` : ajouter `POST /api/v1/reconciliation/manual-match`, `POST /api/v1/reconciliation/split`, mutations rules.
  - `authenticated_routes` : ajouter GET list+detail rules.
- [ ] T5.4 — Étendre `crates/kesh-api/src/errors.rs` (5 nouvelles variants) :
  - `AppError::AccountNotFound { account_id }` (si pas déjà existant) → `404 ACCOUNT_NOT_FOUND`.
  - `AppError::ReconciliationFiscalYearClosed { entry_date }` → `409 RECONCILIATION_FISCAL_YEAR_CLOSED`.
  - `AppError::ReconciliationSplitImbalance { expected, actual, difference }` → `400 RECONCILIATION_SPLIT_IMBALANCE`.
  - `AppError::ReconciliationRuleNotFound { rule_id }` → `404 RECONCILIATION_RULE_NOT_FOUND`.
  - `AppError::ReconciliationRuleDuplicate { match_type, match_value }` → `409 RECONCILIATION_RULE_DUPLICATE`.
  - `AppError::ReconciliationRuleMismatch { rule_id }` → `400 RECONCILIATION_RULE_MISMATCH`.
  - `AppError::ReconciliationRuleNoLongerMatches { rule_id }` → mappé en `failed[]` shape (pas un HTTP error global mais un per-proposal error).
- [ ] T5.5 — Tests E2E HTTP `crates/kesh-api/tests/reconciliation_manual_e2e.rs` (≥ 14) :
  1-10. Manual-match (AC #75-#84).
  11-13. Suggestion (AC #85-#88).
  14-22. Rules CRUD + multi-tenant (AC #89-#98).
  23-28. Rule application + accept (AC #99-#106).
  29-35. Split (AC #107-#113).
  36. RBAC consolidé (AC #121).

### T6. Frontend feature `features/reconciliation` extensions + page `/reconciliation/rules` (AC #114-#119)

- [ ] T6.1 — Étendre `frontend/src/lib/features/reconciliation/reconciliation.api.ts` :
  ```ts
  export async function manualMatchTransaction(
      bankAccountId: number,
      bankTransactionId: number,
      counterpartyAccountId: number,
      description?: string,
      valueDate?: string,
  ): Promise<{ bankTransactionId: number; journalEntryId: number; ruleSuggestion: RuleSuggestion | null }>;

  export async function splitTransaction(
      bankAccountId: number,
      bankTransactionId: number,
      splits: { counterpartyAccountId: number; amount: string; description: string }[],
      description?: string,
      valueDate?: string,
  ): Promise<{ bankTransactionId: number; journalEntryId: number; splitsCount: number }>;
  ```
- [ ] T6.2 — Créer `frontend/src/lib/features/reconciliation/rules/` :
  - `rules.api.ts` (CRUD client).
  - `rules.types.ts` (`ReconciliationRule`, `MatchType` enum).
  - `RulesList.svelte` (table + actions).
  - `RuleFormModal.svelte` (form modal create/edit).
  - `RulesList.test.ts`, `RuleFormModal.test.ts` (Vitest).
- [ ] T6.3 — Créer `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte` :
  - Props : `bankTransaction: ReconciliationProposal['transaction']`, `bankAccountId: number`.
  - Sélecteur `Account` autocomplete (accounts plan comptable de la company, filtré par `class IN (5,6,7)` recommandé pour contreparties usuelles).
  - Textarea description (200 chars max).
  - Datepicker `valueDate`.
  - On submit : `manualMatchTransaction(...)` + dispatch event `success` avec `ruleSuggestion`.
- [ ] T6.4 — Créer `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` :
  - Props : `bankTransaction`, `bankAccountId`.
  - Tableau splits éditable (ajout/suppression de ligne, min 2 max 50).
  - Sticker balance live computed `sum vs |tx.amount|` (vert si exact match, rouge sinon).
  - Bouton submit désactivé tant que balance ≠ exact.
- [ ] T6.5 — Étendre `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` :
  - Pour chaque ligne tx avec `candidates: []` : 2 boutons `Affecter manuellement` + `Éclater`.
  - Pour candidate `type: 'rule'` : afficher différemment (badge bleu « Règle » + label règle).
  - On modal success : refresh la liste (les tx réconciliées disparaissent).
- [ ] T6.6 — Créer `frontend/src/routes/(app)/reconciliation/rules/+page.svelte` (route protégée) :
  - Layout : titre + bouton « + Nouvelle règle » + `RulesList` + `RuleFormModal` ouvert sur création/édition.
- [ ] T6.7 — Suggestion toast post-manual-match : étendre le composant qui consomme la response pour afficher toast 10s + bouton vers `RuleFormModal` pré-rempli.
- [ ] T6.8 — Tests Vitest (≥ 6) :
  1. `ReconciliationProposals: shows manual+split buttons for tx without candidate` (AC #114).
  2. `ReconciliationProposals: renders rule candidate with blue badge` (AC #99 UI).
  3. `ManualMatchModal: prefills value date from tx.value_date` (AC #115).
  4. `TransactionSplitModal: balance indicator updates live` (AC #116).
  5. `RulesList: renders rules with priority sort` (AC #117).
  6. `RuleFormModal: validates required fields before submit` (AC #117).

### T7. i18n (AC #122)

- [ ] T7.1 — Ajouter ~25 nouvelles clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (préfixes stricts `reconciliation-manual-*`, `reconciliation-rules-*`, `reconciliation-split-*`, `reconciliation-suggestion-*`). FR canonical.
- [ ] T7.2 — Traductions DE / IT / EN — pas de copies françaises (lesson 8-2 H13). Vocabulaire bancaire suisse.
- [ ] T7.3 — Vérifier `npm run lint-i18n-ownership` PASS sur 4 locales.

### T8. Tests E2E Playwright (AC #117, #118, #119)

- [ ] T8.1 — Créer `frontend/tests/e2e/reconciliation-manual.spec.ts` (≥ 3 actifs) :
  1. `manual-match end-to-end` : login Comptable, navigate `/reconciliation`, click « Affecter manuellement » sur tx sans candidate, sélectionner compte, valider, vérifier toast succès + tx disparaît + suggestion toast apparaît.
  2. `split end-to-end` : click « Éclater » sur tx -10700, ajouter 3 lignes, vérifier balance indicator passe au vert, valider, vérifier disparition.
  3. `rules CRUD end-to-end` : navigate `/reconciliation/rules`, créer rule, éditer, désactiver, supprimer, vérifier audit.
- [ ] T8.2 — Tests a11y axe (AC #119) : 3 scénarios sur chaque modal/page (rules, manual-match, split).
- [ ] T8.3 — Helper `seedReconciliationFixture(page, ...)` (étendu de 8-4 si livré) — crée bank_transactions avec varietés (sans candidate, avec invoice candidate, avec rule candidate).

### T9. Sync sprint-status + README (AC implicite Epic 8 progress)

- [ ] T9.1 — `_bmad-output/implementation-artifacts/sprint-status.yaml` : transition `8-5: backlog → ready-for-dev` (cette story creation) puis `ready-for-dev → review` (post `dev-story`).
- [ ] T9.2 — README.md `## Feuille de route` : Epic 8 reste 🚧 En cours jusqu'au merge 8-5 puis ✅ Done si retro Epic 8 statue.
- [ ] T9.3 — README.md `## Fonctionnalités` : ajouter ligne « Réconciliation manuelle, règles d'affectation et éclatement de transactions » sous la section import bancaire.
- [ ] T9.4 — Pas d'issue GitHub à fermer (8-5 n'a pas de KF/CR pré-tracée). KF-026 #76 reste ouverte (multi-candidates UI v0.2 — non adressée 8-5).

## Dev Notes

### API surface livrée 8-1b/8-2/8-3/8-4 — patterns à réutiliser

- **Multi-tenant scoping** (KF-002 Pattern 1) : tous les helpers DB filtrent par `(company_id, ...)`. Cross-tenant = 404, jamais 403.
- **Audit log atomique** : helper `audit_log::insert_in_tx(tx, NewAuditLogEntry { ... })`. Une entrée par operation distincte. Pour 8-5 : `reconciliation.manual_matched`, `reconciliation.split_applied`, `reconciliation_rule.{created,updated,deleted,applied}`.
- **Erreurs structurées** : `AppError::Custom { ... }` ou variantes typées dédiées (préféré).
- **i18n key ownership** : préfixe strict, kebab-case, lint-i18n-ownership pass (Story 6-3).
- **`rust_decimal::Decimal`** : Decimal exact partout pour amounts. Validateur `validate_split_balance` utilise `==` strict.
- **Repository pattern + sqlx** : Executor générique `<E: Executor>` (pattern 8-3 / 8-4).
- **Advisory lock per-account** : `with_account_lock(tx, company_id, bank_account_id, 5)` réutilisé pour manual-match + split + accept-with-rule (sérialisation cross-flows sur le même compte).
- **Savepoint partial success** : appliqué uniquement `POST /accept` (cohérent 8-4) ; `POST /manual-match` et `POST /split` sont single-tx all-or-nothing (pas de batch).
- **`journal_entries::create_in_tx`** : helper Story 5-2, accepte tx ouverte par caller, ne commit pas. Émet audit `journal_entry.created` automatiquement.
- **`fiscal_years::find_open_for_date_for_company`** : helper Story 3-7, indispensable pour résoudre `fiscal_year_id` à partir d'une `entry_date` dans manual-match + split.

### Lessons leçons des stories précédentes

- **8-4 retro** (cycle 4 passes review pour 5 modules / ~2200 lignes) : **8-5 est plus large (3 features distinctes, ~3000+ lignes)** — l'argument splitting préventif (Option B 8-5a/b) est fort. Si Guy choisit Option A unifiée, accepter ≥ 4 passes review.
- **8-3 retro** (CHECK constraints invisibles sans MariaDB up) : T1 ajoute 3 CHECK + 1 UNIQUE + 1 FK — vérifier avec MariaDB up local + KESH_TEST_MODE=true.
- **8-2 retro H7** (`ParseCsvOutcome` sans breaking) : pour la modification de `POST /accept` avec discriminator `type`, conserver backward-compat 8-4 (type omitted = invoice). Documenter dans la doc API + Dev Notes.
- **5-2 leçon** (`create_in_tx` pour atomicité) : les 3 nouvelles routes manual/split/accept-with-rule **doivent** utiliser `create_in_tx` plutôt que `create` (qui ouvre sa propre tx, incompatible avec la tx du `with_account_lock`).
- **8-4 patch P3-H1** (optimistic lock UPDATE bank_transactions `AND version = ?`) : appliquer **systématiquement** sur tous les UPDATE bank_transactions de 8-5 (manual, split, accept-with-rule) pour défense-in-depth contre futures mutations concurrentes.
- **8-4 leçon dual audit** : pour manual-match, on n'émet qu'une action `reconciliation.manual_matched` (pas de dual `journal_entry.created` — c'est `create_in_tx` qui le fait par lui-même). Symétriquement pour split.

### Patterns architecturaux à respecter

- **Pas de dépendance circulaire** : `kesh-reconciliation → kesh-core, kesh-db` (cohérent 8-4). Les nouveaux modules `manual`, `rules`, `split` consomment `kesh_db::entities::{BankTransaction, Account, ReconciliationRule}` et `kesh_core::accounting::NewJournalEntry`.
- **Pas d'`f64` pour montants** : `Decimal` partout (`splits[i].amount`, `tx.amount`). Le `f64` n'apparaît que dans le score 8-4 hérité, pas dans 8-5.
- **Tests : éviter le coupling temporel** : utiliser des dates fixes dans les seeds (`NaiveDate::from_ymd_opt(2026, 5, 15)`).
- **`auto_match_rejected_at=NULL` au manual-match** : indispensable pour éviter qu'une tx manual-matched apparaisse comme « rejetée + matched » (état incohérent).

### Source tree à toucher

**DB** :
- `crates/kesh-db/migrations/20260508000001_reconciliation_rules.sql` *(nouveau)*
- `crates/kesh-db/src/repositories/reconciliation_rules.rs` *(nouveau)*
- `crates/kesh-db/src/repositories/mod.rs` (re-export `pub mod reconciliation_rules`)
- `crates/kesh-db/src/repositories/bank_transactions.rs` (extension `find_pending_by_id_for_account`)
- `crates/kesh-db/src/entities/reconciliation_rule.rs` *(nouveau)*
- `crates/kesh-db/src/entities/mod.rs` (re-export)
- `crates/kesh-db/src/test_fixtures.rs` (TABLES_TO_TRUNCATE += `"reconciliation_rules"`)
- `crates/kesh-db/tests/reconciliation_rules_repository.rs` *(nouveau)*

**Backend `kesh-reconciliation`** :
- `crates/kesh-reconciliation/Cargo.toml` (deps inchangées : `kesh-core`, `kesh-db`, `sqlx`, `chrono`, `rust_decimal`, `serde`, `thiserror`, `tracing`)
- `crates/kesh-reconciliation/src/lib.rs` (refactor — modules `manual`, `rules`, `split` ajoutés)
- `crates/kesh-reconciliation/src/manual.rs` *(nouveau, pure)*
- `crates/kesh-reconciliation/src/rules.rs` *(nouveau, pure + suggester)*
- `crates/kesh-reconciliation/src/split.rs` *(nouveau, validator)*
- `crates/kesh-reconciliation/src/errors.rs` (variants ajoutés)

**Backend `kesh-api`** :
- `crates/kesh-api/src/routes/reconciliation_rules.rs` *(nouveau, CRUD)*
- `crates/kesh-api/src/routes/reconciliation.rs` (extension manual-match + split + accept-with-rule + get_proposals avec rule application)
- `crates/kesh-api/src/routes/mod.rs` (`pub mod reconciliation_rules`)
- `crates/kesh-api/src/lib.rs` (mount routes)
- `crates/kesh-api/src/errors.rs` (5+ nouvelles variantes)
- `crates/kesh-api/tests/reconciliation_manual_e2e.rs` *(nouveau, ≥ 14 tests)*

**i18n** :
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (~25 nouvelles clés × 4 locales)

**Frontend** :
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts` (extension manual-match + split)
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` (extensions rule candidate + boutons manual/split)
- `frontend/src/lib/features/reconciliation/ManualMatchModal.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/rules/rules.api.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/rules/rules.types.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/rules/RulesList.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/rules/RuleFormModal.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/rules/RulesList.test.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/rules/RuleFormModal.test.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/ManualMatchModal.test.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/TransactionSplitModal.test.ts` *(nouveau)*
- `frontend/src/routes/(app)/reconciliation/rules/+page.svelte` *(nouveau, route)*
- `frontend/src/routes/(app)/reconciliation/rules/+page.ts` *(nouveau, load function)*
- `frontend/src/routes/(app)/reconciliation/+page.svelte` (extension intégration modals)
- `frontend/tests/e2e/reconciliation-manual.spec.ts` *(nouveau)*

### Standards de test

- **Unit `kesh-reconciliation`** : `#[cfg(test)] mod tests` inline `manual.rs` + `rules.rs` + `split.rs`. ≥ 13 unit tests T4.6.
- **Intégration `kesh-db`** : `#[sqlx::test]`. ≥ 6 tests T2.4 + 2 tests T3.2.
- **E2E HTTP `kesh-api`** : helper `spawn_app(pool)` (pattern 8-1b/8-2/8-3/8-4). ≥ 36 tests T5.5 (sous-totaux : 10 manual + 4 suggestion + 9 rules CRUD + 6 rule application + 7 split).
- **Vitest frontend** : `npm run test:unit -- reconciliation`. ≥ 6 tests T6.8.
- **Playwright** : `frontend/tests/e2e/reconciliation-manual.spec.ts`. ≥ 3 actifs + 3 a11y.

### Checklist locale avant push

```sh
# Backend (cf. CLAUDE.md « Test Locally First »)
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1   # MariaDB up requis (T1 migration check + T2 sqlx + T3 sqlx + T4 unit + T5 E2E)

# Frontend
cd frontend
npm run check
npm run lint-i18n-ownership   # AC #122
npm run test:unit
npm run build

# E2E (MariaDB up + seed CI + browsers installés)
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- reconciliation-manual.spec.ts
```

### Limitations connues v0.1

| # | Limitation | Justification |
|---|---|---|
| L41 | Règles avec patterns simples (4 `match_type` v0.1) — pas de regex | Décision conservatrice : regex coût en pédagogie utilisateur + injection SQL côté DB-side LIKE. v0.1 : `counterparty_contains`, `counterparty_exact`, `reference_contains`, `iban_exact`. v0.2 : ajouter `match_type='regex'` avec validation côté serveur (pas de DB-side regex). |
| L42 | Pas d'auto-acceptation des candidates type=rule (cohérent L18 héritée 8-4) | Toutes les candidates rule sont retournées dans GET /proposals avec score=0.5 (badge jaune). L'utilisateur valide explicitement comme pour les invoice candidates. v0.2 : seuil `auto_accept_threshold` configurable par tenant + par rule (priorité ≥ X = auto-accept). |
| L43 | Pas d'application batch des règles à un dataset historique | La route `POST /reconciliation/apply-rules` n'est pas livrée v0.1. Les règles s'appliquent uniquement à GET /proposals à la volée (effet sur les nouvelles tx pending). Si l'utilisateur crée une rule après import, les anciennes tx pending bénéficient de la rule au prochain GET. v0.2 : route batch pour reprocesser un range de dates. |
| L44 | Split ne lie pas vers des invoices multiples | v0.1 : split crée un journal_entry à N+1 lignes pointant sur des comptes de contrepartie type frais/produits (cohérent avec FR48 reformulé). Lier 1 split vers N invoices clients (paiement multi-factures) est reporté v0.2. Workaround v0.1 : utiliser N `manual-match` séparés sur des sous-transactions virtuelles (anti-pattern, déconseillé). |
| L45 | Annulation de réconciliation non disponible | v0.1 : pas d'undo `accept` / `manual-match` / `split`. L'utilisateur doit modifier l'écriture comptable manuellement (Story 3-3). v0.2 : route `POST /reconciliation/revert/{bankTransactionId}` qui re-set `status='pending'` + supprime la journal_entry liée + audit `reconciliation.reverted`. |
| L46 | Suggestion de règle déterministe sans ML | Algorithme heuristique (IBAN > counterparty_contains > reference_contains). Pas de pondération ML basée sur l'historique des accepts. Acceptable v0.1 — une suggestion « bête » bien explicitée vaut mieux qu'un modèle opaque. v0.2+ : training sur historique pour suggérer des matches plus subtils. |
| L47 | Multi-currency split non supporté | Tous les splits doivent être dans la même currency que la tx (CHF v0.1, mono-CHF cohérent L38 héritée 8-4). v0.2 / Story 11 : supporter splits multi-devise avec taux de change journal_entry. |
| L48 | UI rule application sans preview de matches | Le frontend ne montre pas combien de tx pending matcheraient une nouvelle règle avant sa création. L'utilisateur doit créer puis observer. v0.2 : endpoint `POST /reconciliation/rules/preview` qui retourne le count de matches sans persister. |
| L49 | Pas d'export CSV des rules pour backup/migration | Si un utilisateur veut migrer ses rules vers une nouvelle instance Kesh, il doit les recréer manuellement via UI. v0.2 : endpoint `GET /reconciliation/rules/export.csv` + `POST /reconciliation/rules/import` (batch CSV). |
| L50 | Rule applied count non décrémenté si reconciliation revertée | Cohérent L45 (pas d'undo v0.1). Si l'utilisateur modifie l'écriture comptable manuellement après accept-with-rule, `reconciliation_rules.applied_count` reste incrémenté (compte « accept tenté », pas « accept abouti »). Acceptable v0.1. |

### Levée des limitations héritées 8-4

| # héritée 8-4 | Status 8-5 | Mécanisme |
|---|---|---|
| L19 (matching journal_entries non-invoice) | **LEVÉE** | FR45 manual-match crée une journal_entry directement sans facture pré-existante (compte de contrepartie librement choisi). FR47 rules permet l'auto-application de cette mécanique. |
| L20 (création écriture sans facture) | **LEVÉE** | `journal_entries::create_in_tx` invoqué par manual-match et split. Pas de dépendance à `invoice.journal_entry_id`. |
| L23 (`auto_match_rejected_at` non réversible) | **PARTIELLEMENT LEVÉE** | Manual-match RESET `auto_match_rejected_at=NULL` (cas « rejet → manual reverse »). Bouton « Annuler le rejet » UI v0.1 toujours absent — l'utilisateur doit explicitement créer une écriture. v0.2 : bouton dédié. |
| L21 (paiement partiel) | **NON LEVÉE v0.1** | Reportée v0.2. Workaround 8-5 : manual-match avec compte « Différence de paiement » + invoice toujours dans `paid_at IS NULL`. |
| L18 (seuil auto-accept) | **NON LEVÉE v0.1** | Reportée v0.2 (cf. L42 nouvelle). |

### Risques et points d'attention pour le dev agent

1. **Concurrence rules accept** : un Comptable accepte une rule candidate, et au même moment, un autre Comptable édite la rule (PATCH change `match_value`). Le optimistic lock sur `rules.version` au step 11 §accept-with-rule-flow protège, mais l'utilisateur observe `failed: [{ errorCode: 'RECONCILIATION_RULE_NO_LONGER_MATCHES' }]` même si l'edit est anodin. Acceptable v0.1.

2. **Rule sur compte archivé** : si un Comptable archive un compte qui est `counterparty_account_id` de plusieurs rules actives, les rules continuent d'apparaître mais skippent silencieusement à l'application. UX peut surprendre. **Recommandation** : ajouter une UI warning « Cette règle utilise un compte archivé » dans `RulesList.svelte` (T6.2). Si non livré v0.1 → ajouter LXX dette UX.

3. **Test E2E HTTP volume** : 36 tests minimum est important. Le dev agent peut être tenté de différer une partie (cf. dette test 8-4 §dette-test-e2e-http). **Recommandation** : si scope vraiment trop large dans une seule session dev, accepter de différer 5-7 tests les moins critiques (split N+1 lignes large, rule edge cases) et tracer en `dette-test-e2e-http`. **Mais** : les 10 tests manual-match + 9 rules CRUD + 6 rule application sont **incontournables** (sécurité multi-tenant + RBAC).

4. **Migration CHECK constraints visibles uniquement avec MariaDB up** : T1 ajoute 3 CHECK + 1 UNIQUE — vérifier avec `cargo test -p kesh-db --lib test_fixtures` MariaDB-up local avant push (lesson 8-3 retro).

5. **Backward-compat 8-4 `POST /accept`** : si le dev agent oublie de gérer `type` absent comme `'invoice'`, les tests E2E HTTP 8-4 vont casser (cf. AC #106). Couvrir explicitement par test.

6. **`fiscal_years::find_open_for_date_for_company` peut ne pas exister** : vérifier dans `crates/kesh-db/src/repositories/fiscal_years.rs` Story 3-7. Si le helper n'existe qu'avec une signature différente, créer une version compatible (Executor générique, multi-tenant scoped) avant T5.

### Références

- [`epic-8.md`](../planning-artifacts/epic-8.md) — Story 8-5 ACs originaux (FR45-FR48), section « Risques » R6 R7.
- [`prd.md`](../planning-artifacts/prd.md) §FR45-FR48 lignes 439-442.
- [`8-4-reconciliation-matching-automatique.md`](8-4-reconciliation-matching-automatique.md) — patterns repo + mutex + audit + savepoint à réutiliser.
- [`architecture.md`](../planning-artifacts/architecture.md) §11.5 (kesh-reconciliation), §17 (FR42-FR53 mapping), L491-L498 (modules `matching/rules/mutex`).
- [`ux-design-specification.md`](../planning-artifacts/ux-design-specification.md) §164 scenario Lisa fiduciaire, §329 modèle « routine fluide » règles d'affectation.
- [Story 5-2 `journal_entries::create_in_tx`](../../crates/kesh-db/src/repositories/journal_entries.rs) — helper transaction-bound.
- [Story 3-7 `fiscal_years::*`](../../crates/kesh-db/src/repositories/fiscal_years.rs) — résolution `fiscal_year_id` from `entry_date`.
- KF-026 #76 — multi-candidates UI (v0.2, non adressée 8-5).
- Findings résiduels 8-4 documentés (non-bloquants) : A6-2 POST currency guard gap MEDIUM (à fixer début Story 11), `reject_batch` reload asymétrique LOW, `find_pending_by_id_for_account` naming LOW (renommé en T3.1 pour éviter ambiguïté).

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

- **2026-05-07** — Spec créée par `bmad-create-story 8-5` (Opus 4.7) post-merge PR #77 (Story 8-4 done squash `22fd6d4`). 50 ACs (#75-#124) + 9 tasks T1-T9. Splitting risque documenté avec **recommandation Option B (split 8-5a/b)** pour décision Guy avant validate. Limitations connues v0.1 L41-L50 ajoutées. Levée explicite des limitations 8-4 L19/L20 (partielle L23). Status `8-5: backlog → ready-for-dev`. **Décision Guy attendue** : Option A unifiée vs Option B split vs Option C reduced scope avant `bmad-create-story validate 8-5` Pass 1 Sonnet.
