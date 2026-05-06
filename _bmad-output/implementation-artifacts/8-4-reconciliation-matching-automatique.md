# Story 8-4: Réconciliation & matching automatique

Status: ready-for-dev

<!-- Note: Validation est obligatoire (règle CLAUDE.md « Review Iteration Rule »). Lancer `bmad-create-story validate 8-4` Pass 1 Sonnet (cycle Opus auteur → Sonnet, fenêtre fraîche) avant `bmad-dev-story 8-4`. -->

## Story

As a **utilisateur Kesh (PME / indépendant suisse)**,
I want **que les transactions bancaires importées soient automatiquement appariées aux factures et écritures comptables connues, avec un score de confiance, et que je puisse valider en lot les propositions retenues**,
so that **mon travail de réconciliation soit minimal et que les paiements de factures soient repérés sans saisie manuelle**.

### Contexte

**Story 8-4 = quatrième story de l'Epic 8 « Import Bancaire & Réconciliation »**, après **8-1a/8-1b** (parser CAMT.053 + persistance), **8-2** (CSV multi-encodage), **8-3** (détection doublons + rejet partiel + KF #70). Elle ouvre la **partie B de l'Epic 8** (réconciliation), 8-1/8-2/8-3 ayant clos la partie A (import).

8-4 livre **FR44** (matching automatique avec score) intégralement, et démarre la fondation de **8-5** (réconciliation manuelle + règles d'affectation, FR45-FR47).

**Status sprint** : `8-4-reconciliation-matching-automatique: backlog → ready-for-dev` après création de cette spec.

**Pré-requis closed (prep sprint Epic 7→8)** :
- ✅ KF-020 #49 — `SELECT FOR UPDATE` pour update no-op (closed PR #64)
- ✅ KF-002-H-002 #43 — deadlock-retry middleware (closed PR #65)
- ✅ Foundation 8-1b — schéma `bank_transactions { status, matched_entry_id }` posé en avance, enum `BankTransactionStatus { Pending, Reconciled }` exporté

**Crate cible** : `kesh-reconciliation` (placeholder existant `crates/kesh-reconciliation/src/lib.rs` créé en Story 1-1 scaffold). 8-4 le **démarre vraiment** (un seul module aujourd'hui, vide).

### Scope verrouillé — ce qui est livré par 8-4

1. **Algorithme de matching** (FR44 partie 1) — `kesh_reconciliation::matching::propose_matches(tx: &BankTransaction, candidates: &[Invoice]) -> Vec<MatchProposal>`. Pure (sans I/O), unit-testable. Score de confiance ∈ [0.0..=1.0] dérivé de 3 critères pondérés (montant exact, référence, contact). Cf. §matching-algo.
2. **Repository candidates** — `kesh_db::repositories::reconciliation::find_unpaid_invoices_for_window(pool, company_id, period_from, period_to, amount_min, amount_max) -> Vec<Invoice>` filtré multi-tenant + fenêtre temporelle ± 30 jours autour de la transaction + filtre montant ± 0.05 CHF (cf. §candidate-window). L'amount window évite de remonter 100% des factures impayées sur grandes companies.
3. **Mutex par compte bancaire** — advisory lock MariaDB `GET_LOCK('reconcile:company_id:bank_account_id', timeout)` + `RELEASE_LOCK` autour du flow `propose_matches → accept_batch → INSERT journal_entry/UPDATE bank_transactions.status` pour empêcher les imports concurrents de proposer/finaliser les mêmes propositions sur le même compte. Cf. §mutex-account.
4. **Routes API** :
   - `GET /api/v1/reconciliation/proposals?bankAccountId={id}` — retourne les propositions pour les `bank_transactions.status='pending'` du compte.
   - `POST /api/v1/reconciliation/accept` — body : `{ proposals: [{ bankTransactionId, invoiceId, journalEntryId? }] }` — accepte un lot, met à jour `bank_transactions.status='reconciled'` + `matched_entry_id`, **lie** la transaction à l'écriture existante OU **crée** une écriture si la facture n'a pas encore son `journal_entry_id` (cas paiement direct sans facture validée — différé v0.2 dans **L20** documentée).
   - `POST /api/v1/reconciliation/reject` — body : `{ bankTransactionIds: [...] }` — pour refuser explicitement des propositions automatiques (la transaction reste `pending` mais marquée comme « manually reviewed », à exploiter en 8-5).
5. **Frontend feature `features/reconciliation/`** — nouvelle page `/reconciliation` (route protégée RBAC `Comptable`) listant les propositions du compte sélectionné. Chaque ligne : transaction bancaire ↔ proposition top-1 + score. Sélection multi-checkbox + bouton « Valider les sélectionnées ». Cas « aucune proposition » → ligne neutre avec lien vers 8-5 (création manuelle, à venir).
6. **i18n** — ~10 nouvelles clés `reconciliation-*` (4 locales fr/de/it/en-CH).
7. **Tests** — Unit `kesh-reconciliation::matching` (≥ 12 cas couvrant tous les chemins de scoring + edge cases), Integration `kesh-db::reconciliation` (≥ 4 sqlx tests sur `find_unpaid_invoices_for_window` + multi-tenant scoping), E2E HTTP `kesh-api::reconciliation` (≥ 6 tests : list, accept happy, accept partial, reject, mutex contention, multi-tenant), Vitest (≥ 3), Playwright (≥ 2 actifs).
8. **Sync** sprint-status + README + audit log `reconciliation.accepted` / `reconciliation.rejected` discriminants.

**HORS scope 8-4 (reportés Stories 8-5 / v0.2) :**

- **Création manuelle de contrepartie** (FR45) → 8-5.
- **Règles d'affectation automatique** (FR46, FR47) → 8-5 entière.
- **Éclatement transaction agrégée FR48** → 8-5.
- **Matching against journal_entries non-invoice** (paiements salaires, frais bancaires, virements internes) → 8-5 partial / v0.2.
- **Matching multi-période** (la fenêtre est ± 30 jours, suffisant pour le cas nominal client suisse paiement à 30 jours) → si une facture est payée tardivement (> 60 jours), reportée en 8-5 manual.
- **Score configurable par utilisateur** (seuils min auto-accept) → v0.2 (sera une « rule ») ; v0.1 affiche le score sans seuil filtrant côté backend.
- **Création d'écriture comptable depuis bank_transaction sans facture pré-existante** (le user choisit un compte de contrepartie type frais bancaires) → 8-5.
- **Détection paiement partiel** (montant tx ≠ montant facture mais "matche" sémantiquement) → v0.2 (8-5 traitera le `accept` avec écart documenté en `journal_entry`).

### Décisions de conception (clés)

#### §matching-algo (algorithme de score)

Le helper `propose_matches` calcule un score `f64 ∈ [0.0..=1.0]` par paire `(BankTransaction, Invoice)` candidate, dérivé de **3 critères pondérés** :

| Critère | Poids | Calcul |
|---|---|---|
| **Montant exact** | 0.50 | `1.0` si `tx.amount == invoice.total_amount` (Decimal exact, pas de tolérance), sinon `0.0`. |
| **Référence** | 0.40 | `1.0` si normalisation(`tx.reference|end_to_end_id|transaction_id`) contient ou égale normalisation(`invoice.invoice_number`) (case-insensitive, trim, accept substring containment) ; `0.5` si seul un préfixe ≥ 4 chars matche ; sinon `0.0`. |
| **Contact** | 0.10 | `1.0` si `normalize(tx.counterparty_name)` substring-matche `normalize(contact.name)` du `invoice.contact_id` ; sinon `0.0`. |

**Score final** : `score = 0.50 * amount_score + 0.40 * reference_score + 0.10 * contact_score`. Décision conservatrice du poids amount (50%) : la combinaison « bon montant + bonne référence » garantit ≥ 0.90, ce qui matérialise le cas nominal QR-Bill paiement (référence 27 digits = invoice_number, montant exact). Le contact reste minoritaire car les noms contreparties dans CAMT.053 sont parfois bruités (ex. « ALICE MARTIN-DUBOIS GMBH » vs `contact.name = "Alice Martin-Dubois Sàrl"` — l'utilisateur peut tolérer 0.10 de pertes).

**Justification R5 (epic-8.md)** — Score de confiance, seuil par défaut, configurabilité, exposé UI :
- **Seuil par défaut v0.1** : aucun (toutes les propositions avec `score > 0.0` sont retournées). Le frontend trie par `score DESC` et affiche les top-N (N=3 par défaut, configurable v0.2). L'utilisateur **sélectionne explicitement** ce qu'il accepte ; pas d'auto-acceptation v0.1.
- **Configurabilité** : différée v0.2 (paramètre `auto_accept_threshold` par tenant ou par règle 8-5).
- **Exposé UI** : oui, score affiché en pourcent (`78%`) avec badge couleur seuil empirique (≥ 90% vert, 70-89% jaune, < 70% rouge) — décision UX inspirée du flow PRD §134 / Sophie.

**Pure helper** : `propose_matches` ne fait pas d'I/O. Le caller (`kesh-api`) charge `candidates` via `find_unpaid_invoices_for_window` et passe les vecs au helper. Cette pureté permet le test unitaire intensif (12+ cas) sans setup DB.

#### §candidate-window (fenêtre de candidats)

Pour éviter d'appliquer le matching contre **toutes** les factures impayées de la company (qui peut en avoir des centaines), `find_unpaid_invoices_for_window` filtre par :

1. **`company_id`** (KF-002 Pattern 1, scoping multi-tenant systématique).
2. **`status = 'validated' AND paid_at IS NULL`** (factures validées non payées — l'état des factures éligibles à la réconciliation).
3. **`date BETWEEN tx.booking_date - 30 days AND tx.booking_date + 30 days`** : couvre paiement à 30 jours typique suisse + tolérance 30 jours retard (clients lents). Au-delà, l'utilisateur passera par 8-5 manual.
4. **`total_amount BETWEEN tx.amount - 0.05 AND tx.amount + 0.05`** : tolérance de 5 centimes pour absorber les arrondis comptables d'un côté ou de l'autre. **Note** : le helper `propose_matches` n'accepte que les amounts **exactement** égaux ; cette tolérance amount au repo est un **filtre** (réduit le candidate set avant pondération), pas une **acceptation** (le score amount reste binaire 0/1 dans le helper). Justifié pour ne pas exclure une facture à 100.01 quand la transaction est 100.00 — le helper donnera score=0 sur amount mais peut quand même remonter par référence ; l'utilisateur verra le mismatch dans l'UI.

**Index DB requis** : `idx_invoices_company_status_paid_date (company_id, status, paid_at, date)` — à créer dans la migration T1. Couvre les 3 colonnes filtrées (paid_at est NULL pour les candidates, status='validated' filtre constant). Vérifier `EXPLAIN` post-migration pour confirmer.

**Pas de filtre par `bank_account_id`** : une facture n'est pas liée à un compte bancaire spécifique (le compte est porté par la transaction, pas par la facture). Le matching cross-comptes est explicitement permis.

#### §mutex-account (concurrence imports/réconciliations)

**Risque** : deux utilisateurs sur la même company peuvent (a) importer simultanément deux relevés du même compte qui contiennent les mêmes paiements, (b) accepter en parallèle deux propositions distinctes pointant sur la même `invoice_id`. Sans verrou, on aboutit à `bank_transactions.matched_entry_id` colliding ou à des doublons d'écriture journal.

**Mitigation v0.1 — advisory lock applicatif** :

```rust
// Dans kesh-reconciliation::mutex
pub async fn with_account_lock<F, T>(
    tx: &mut sqlx::Transaction<'_, sqlx::MySql>,
    company_id: i64,
    bank_account_id: i64,
    timeout_secs: u32,
    f: F,
) -> Result<T, ReconciliationError>
where
    F: AsyncFnOnce(&mut sqlx::Transaction<'_, sqlx::MySql>) -> Result<T, ReconciliationError>,
{
    let lock_name = format!("reconcile:{company_id}:{bank_account_id}");
    let acquired: i32 = sqlx::query_scalar("SELECT GET_LOCK(?, ?)")
        .bind(&lock_name).bind(timeout_secs).fetch_one(&mut **tx).await?;
    if acquired != 1 {
        return Err(ReconciliationError::AccountLocked { bank_account_id, timeout_secs });
    }
    let result = f(tx).await;
    sqlx::query("SELECT RELEASE_LOCK(?)").bind(&lock_name).execute(&mut **tx).await.ok();
    result
}
```

**Mapping HTTP** : `ReconciliationError::AccountLocked` → `409 RECONCILIATION_ACCOUNT_LOCKED` avec `details = { bank_account_id, retry_after_seconds }`. Le frontend affiche un toast « Un autre import/réconciliation est en cours sur ce compte, réessayez dans quelques secondes ».

**Timeout** : 5 secondes par défaut (configurable via env `KESH_RECONCILIATION_LOCK_TIMEOUT_SECS`, default 5). Justifié : un `accept_batch` typique de 10 propositions tourne en < 1s ; 5s couvre les cas pathologiques sans frustrer l'utilisateur en attente.

**Choix vs alternatives** :
- ❌ `SELECT ... FOR UPDATE` sur `bank_accounts`: pose un verrou row-level qui peut être tenu plus longtemps par d'autres queries ; sémantique moins claire que le named lock.
- ❌ Mutex Tokio en mémoire : pas multi-instance (Kesh peut tourner en 2 réplicas derrière un load-balancer en v0.2).
- ✅ Advisory lock MariaDB `GET_LOCK` : portable multi-instance (le lock est au niveau DB), nommé, libère automatiquement à la fin de session si l'app crashe (timeout DB).

**Note KF-020 #49 (closed Epic 7)** : `SELECT FOR UPDATE` est désormais utilisé pour la race no-op des updates. Ici, `GET_LOCK` est complémentaire pour le scope « éviter imports/reconciliations concurrents », qui n'est pas un update no-op mais une serialization de séquences d'opérations.

#### §accept-flow (flow d'acceptation d'une proposition)

`POST /api/v1/reconciliation/accept` body `{ proposals: [{ bankTransactionId, invoiceId }] }` — pour chaque proposition :

1. Acquire `with_account_lock(tx, company_id, bank_account_id, 5)`.
2. Charger `BankTransaction` (scoped `company_id`) — `404` si introuvable / cross-tenant.
3. Charger `Invoice` (scoped `company_id`) — `404` si introuvable / cross-tenant.
4. Vérifier `bank_transaction.status == 'pending'` — sinon `409 RECONCILIATION_ALREADY_RECONCILED` (l'utilisateur a peut-être un onglet caduc).
5. Vérifier `invoice.status == 'validated' AND invoice.paid_at IS NULL` — sinon `409 RECONCILIATION_INVOICE_NOT_ELIGIBLE`.
6. **Lier l'écriture comptable** :
   - Si `invoice.journal_entry_id IS NOT NULL` (cas normal post-Story 5-2 facture validée) : `UPDATE bank_transactions SET matched_entry_id = invoice.journal_entry_id, status = 'reconciled' WHERE id = ?`. **Pas** de création d'écriture supplémentaire ; la facture validée a déjà son écriture (« Client / Vente »), la transaction bancaire vient confirmer le paiement (« Banque / Client » sera créée au flow 8-5 ou via mark_paid existant Story 5-4). Décision v0.1 : **le `accept` 8-4 marque seulement la liaison `matched_entry_id` + `status='reconciled'`**, **et déclenche `mark_paid` sur la facture** (pose `paid_at = tx.booking_date_or_value_date.preferred()`). L'écriture « Banque / Client » sera créée par `mark_paid` (Story 5-4 wiring existant à vérifier ; si non, créer dans T6).
   - Si `invoice.journal_entry_id IS NULL` (cas non-nominal — facture en brouillon non validée) : reject `409 RECONCILIATION_INVOICE_NOT_ELIGIBLE` (cf. step 5). **Pas de création d'écriture v0.1**.
7. Audit log `reconciliation.accepted` avec `details = { bank_transaction_id, invoice_id, score, batch_size }`.
8. Release lock et commit transaction.

**Atomicité par proposal** : chaque proposal est traitée dans une transaction propre — si la 5e d'un batch de 10 échoue, les 4 premières restent acceptées. Le response body retourne `{ accepted: [...], failed: [{ proposal, error_code }] }` pour permettre au frontend de retry/diagnostiquer. Décision : **partial success > all-or-nothing** car un batch de 50 où une seule proposal a un état caduc casserait tout le travail utilisateur.

#### §reject-flow

`POST /api/v1/reconciliation/reject` body `{ bankTransactionIds: [42, 43, 99] }` — marque ces transactions comme « manuellement revues mais sans match auto » :

1. Pour chaque ID : `UPDATE bank_transactions SET status = 'pending' WHERE id = ? AND company_id = ?`. *(Non, status reste `pending` mais on pose un nouveau champ `auto_match_rejected_at: Option<NaiveDateTime>` pour distinguer « jamais vu » vs « rejeté par user »).*
2. Migration T1 ajoute la colonne `bank_transactions.auto_match_rejected_at DATETIME(3) NULL`.
3. La query `find_unpaid_invoices_for_window` n'est PAS appelée pour les transactions avec `auto_match_rejected_at IS NOT NULL` (elles n'apparaissent plus dans `/proposals`).
4. La page 8-5 manual (à venir) listera ces transactions en priorité (« attendent un appariement manuel »).
5. Audit log `reconciliation.rejected` avec `details = { bank_transaction_ids, count }`.

**Note v0.1** : on **ne supprime pas** le `auto_match_rejected_at` si l'utilisateur change d'avis ; il devra passer par 8-5 manual pour faire le matching. Acceptable v0.1.

#### §audit-log-actions (8-4)

Symétrique au pattern 8-3 (canonical action + modifiers triés). Deux nouvelles actions :

| Action | Contexte | `details_json` |
|---|---|---|
| `reconciliation.accepted` | Un lot accepté | `{ bank_transaction_id, invoice_id, score, batch_size, journal_entry_id }` (une entrée par transaction acceptée) |
| `reconciliation.rejected` | Un lot rejeté | `{ bank_transaction_ids: [...], count }` (une entrée par batch reject) |

**Pas de modifiers `Vec<String>`** ici (pas de variantes confirmables comme en 8-3) — actions plain. La cohérence avec le pattern 8-3 « canonical action » est maintenue par le suffixe `.accepted`/`.rejected` qui est sémantiquement déterministe (pas une explosion combinatoire).

#### §error-precedence-order (8-4)

| # | Erreur | HTTP | Overridable ? |
|---|---|---|---|
| 1 | RBAC (sub-router `comptable_routes`) | 403 | Non |
| 2 | Validation body (proposals vide, invoiceId négatif, etc.) | 400 | Non |
| 3 | Bank transaction not found / cross-tenant | 404 | Non |
| 4 | Invoice not found / cross-tenant | 404 | Non |
| 5 | Bank transaction already reconciled | 409 | Non |
| 6 | Invoice not eligible (not validated OR already paid) | 409 | Non |
| 7 | Account locked (advisory lock timeout) | 409 | **Oui** (retry-after) |
| 8 | DB error / mutex acquisition failed | 500 | Non |

### Risque de splitting (CLAUDE.md check)

**Modules touchés par 8-4** : 6 (`kesh-reconciliation` *(nouveau crate vraiment activé)*, `kesh-db`, `kesh-api`, `frontend`, `kesh-i18n`, `kesh-core` *(extension `BankTransactionDraft` pas attendue mais possible)*). **Au seuil > 5** énoncé par CLAUDE.md.

**Décision : pas de split préventif**, **risque tracé**. Justifications :

1. **Profondeur d'incertitude faible** : tous les patterns sont établis (multi-tenant scoping 6-2/7-1, audit log 1-8, advisory lock pattern 6-3, RBAC sub-router, etc.). 8-4 est une **extension cohérente** qui implémente un crate dédié *placeholder* (`kesh-reconciliation`).
2. **Frontière de split naturelle absente** : l'algorithme `propose_matches` est purement testable mais sans le repo `find_unpaid_invoices_for_window`, on ne peut pas valider end-to-end ; sans la route API, le frontend est inutile. Tous les modules sont co-dépendants en chaîne courte.
3. **Volume estimé** : ~1 800-2 200 lignes net (vs ~3 700 pour 8-3 plus complexe avec 4 panneaux UI). Le matching algo est compact ; l'UI page reconciliation est moyenne (~400 lignes). Tenable en mental model unique.
4. **Précédents** : 8-3 sur 6 modules a tenu sur 3 passes review avec 0 HIGH/CRITICAL — patterns acquis transposables.

**Trigger d'arrêt** : si **Pass 4 spec validate** ne converge pas (≥ 1 finding > LOW), splitter rétroactivement selon la frontière fonctionnelle :

- **8-4a backend matching core** : T1 (migration `auto_match_rejected_at` + index), T2 (`kesh-reconciliation::matching` algo + tests), T3 (`kesh-db::repositories::reconciliation::find_unpaid_invoices_for_window`), T4 (`kesh-reconciliation::mutex` advisory lock helper) + tests sqlx + unit. Livrable testable en isolation : helper compile, tests verts, **pas de routes ni frontend**. Backward-compat absolue.
- **8-4b API + frontend** : T5 (routes API `proposals`/`accept`/`reject` + audit log), T6 (frontend `features/reconciliation/`), T7 (i18n), T8 (Playwright + Vitest), T9 (sync). Dépend de 8-4a comme path-dep.

**Décision pré-implementation** prise par Guy si la trigger se déclenche — pas appliquée par défaut.

## Acceptance Criteria

Numérotation continue 8-3 (qui s'arrêtait à 29). Donc 8-4 commence à #30.

### Algorithme de matching (FR44 partie 1)

30. **(FR44 — score amount exact)** Given une transaction bancaire à 1234.56 CHF et une facture à 1234.56 CHF (référence et contact identiques), When `propose_matches` est appelé, Then `score == 1.00` (0.50 amount + 0.40 reference + 0.10 contact). *Test unitaire `kesh-reconciliation::matching` : `score_full_match_returns_1_0`.*

31. **(FR44 — score amount-only)** Given une transaction à 200 CHF et une facture à 200 CHF mais référence différente et contact différent, When `propose_matches`, Then `score == 0.50` (montant seul). *Test unitaire : `score_amount_only_returns_0_50`.*

32. **(FR44 — score reference partiel)** Given référence transaction "INV-2026-001" et `invoice_number` "INV-2026-001-A" (préfixe matche ≥ 4 chars), When `propose_matches`, Then `reference_score == 0.5` → contribution 0.20. *Test unitaire : `score_reference_prefix_match_returns_0_5`.*

33. **(FR44 — score reference exact via end_to_end_id fallback)** Given `tx.reference = NULL`, `tx.end_to_end_id = "INV-2026-042"`, et `invoice_number = "INV-2026-042"`, When `propose_matches`, Then `reference_score == 1.0` (le helper applique `coalesce(reference, end_to_end_id, transaction_id)` comme 8-3). *Test unitaire : `score_reference_falls_back_to_end_to_end_id`.*

34. **(FR44 — score contact substring)** Given `tx.counterparty_name = "ACME GMBH BERLIN"` et `contact.name = "ACME Sàrl"`, When `propose_matches`, Then `contact_score == 0.0` (pas de substring). Given même tx mais `contact.name = "ACME GMBH"`, Then `contact_score == 1.0`. *Test unitaire : `score_contact_substring_match`.*

35. **(FR44 — pas de match si tous critères 0)** Given une tx à 100 CHF avec ref "ABC" et une invoice à 200 CHF avec ref "XYZ" et contact différent, When `propose_matches`, Then la paire **n'est PAS retournée** (score == 0.0 → filtré). *Test unitaire : `score_zero_filters_out_proposal`.*

36. **(FR44 — tolérance amount au repo, pas au score)** Given une tx à 100.00 CHF et une invoice à 100.03 CHF (dans la fenêtre repo ± 0.05) avec ref/contact identiques, When `propose_matches`, Then la paire est retournée (le repo l'a remontée) mais avec `amount_score == 0.0` → score final = `0.40 + 0.10 = 0.50`. L'utilisateur voit le mismatch dans l'UI et décide. *Test unitaire : `score_amount_mismatch_within_repo_window_returns_partial`.*

### Repository find_unpaid_invoices_for_window

37. **(repo — fenêtre temporelle ± 30 jours)** Given une tx au 2026-05-15 et 4 invoices : (a) 2026-04-10 (35 j avant → hors fenêtre), (b) 2026-04-20 (25 j avant), (c) 2026-05-15 (même jour), (d) 2026-06-20 (36 j après → hors fenêtre), When `find_unpaid_invoices_for_window`, Then seules (b) et (c) sont retournées. *Test sqlx : `find_unpaid_invoices_filters_by_30_day_window`.*

38. **(repo — multi-tenant scoping KF-002 Pattern 1)** Given une invoice 2026-05-15 du `company_A` et une autre identique du `company_B`, When `find_unpaid_invoices_for_window` est appelé pour `company_A`, Then seul `company_A` est retourné. *Test sqlx : `find_unpaid_invoices_scopes_by_company`.*

39. **(repo — filtre status=validated AND paid_at IS NULL)** Given une invoice payée (`paid_at != NULL`) et une autre en draft (`status='draft'`), When `find_unpaid_invoices_for_window`, Then aucune n'est retournée — seules les `status='validated' AND paid_at IS NULL` sont éligibles. *Test sqlx : `find_unpaid_invoices_excludes_paid_and_draft`.*

40. **(repo — fenêtre amount ± 0.05 CHF)** Given tx à 100.00 et 3 invoices : (a) 99.94 (hors fenêtre 0.06), (b) 99.95 (limite incluse), (c) 100.05 (limite incluse), When `find_unpaid_invoices_for_window`, Then (b) et (c) retournées. *Test sqlx : `find_unpaid_invoices_filters_by_amount_window`.*

### Mutex per bank account (advisory lock)

41. **(mutex — acquisition + release nominal)** Given une réconciliation en cours (`with_account_lock` actif sur `(company_id=1, bank_account_id=17)`), When un second appel concurrent `with_account_lock` arrive sur le même `(1, 17)`, Then le second échoue avec `ReconciliationError::AccountLocked` après `timeout_secs` (5 par défaut). *Test sqlx avec 2 connexions parallèles : `mutex_blocks_concurrent_account_lock`.*

42. **(mutex — pas d'interférence cross-account)** Given lock actif sur `(company_id=1, bank_account_id=17)`, When un appel sur `(company_id=1, bank_account_id=18)`, Then succès immédiat. *Test sqlx : `mutex_does_not_block_cross_account`.*

43. **(mutex — release sur erreur)** Given `with_account_lock(tx, ..., |tx| async { Err(...) })` (la fonction caller retourne Err), When le lock est relâché, Then un appel suivant `GET_LOCK` sur la même clé succède. *Test sqlx : `mutex_releases_on_error_path`.*

### Routes API — listing propositions

44. **(GET proposals — happy)** Given un compte avec 3 transactions `pending` dont 2 ont des factures candidates, When `GET /api/v1/reconciliation/proposals?bankAccountId=17`, Then `200 OK` avec body `{ proposals: [{ bankTransactionId, candidates: [{ invoiceId, score, amountMatch, referenceMatch, contactMatch }] }] }`. Le 3e (sans candidate) apparaît avec `candidates: []`. *Test E2E HTTP : `get_proposals_returns_candidates_with_scores`.*

45. **(GET proposals — multi-tenant)** Given une transaction `pending` du `company_B` sur le même IBAN qu'un compte `company_A`, When user `company_A` GET proposals, Then la transaction `company_B` n'apparaît pas. *Test E2E HTTP : `get_proposals_scopes_by_company`.*

46. **(GET proposals — RBAC)** **Hérité 8-1b** — sub-router `comptable_routes`. **Pas re-testé 8-4** mais pattern identique.

47. **(GET proposals — exclude auto_match_rejected)** Given une tx avec `auto_match_rejected_at != NULL`, When GET proposals, Then la tx n'apparaît pas (réservée à 8-5 manual). *Test E2E HTTP : `get_proposals_excludes_rejected_transactions`.*

### Routes API — accept

48. **(POST accept — happy)** Given une proposition `{ bankTransactionId: 42, invoiceId: 101 }` valide, When POST accept, Then `200 OK` body `{ accepted: [{ bankTransactionId: 42, journalEntryId: 999 }], failed: [] }` + `bank_transactions.status='reconciled'` + `bank_transactions.matched_entry_id=999` + `invoices.paid_at != NULL` + audit log `reconciliation.accepted`. *Test E2E HTTP : `post_accept_reconciles_transaction_and_invoice`.*

49. **(POST accept — partial success)** Given un batch de 3 proposals dont 1 a un état caduc (tx déjà `reconciled`), When POST accept, Then `200 OK` body `{ accepted: [2 entries], failed: [1 entry with error_code=RECONCILIATION_ALREADY_RECONCILED] }`. Les 2 succès sont commit, le 3e n'est pas rollback. *Test E2E HTTP : `post_accept_handles_partial_failure`.*

50. **(POST accept — invoice not eligible)** Given proposition pointant sur `invoice.status='draft'` (pas validée), When POST accept, Then `failed: [{ error_code: 'RECONCILIATION_INVOICE_NOT_ELIGIBLE', details: { reason: 'invoice_not_validated' } }]`. Symétrique pour `paid_at != NULL`. *Test E2E HTTP : `post_accept_rejects_unvalidated_or_paid_invoice`.*

51. **(POST accept — multi-tenant safety)** Given user `company_A` POST accept avec `invoiceId` appartenant à `company_B`, Then `failed: [{ error_code: '404', details: { reason: 'invoice_not_found' } }]` (pas 403 — pas de leak d'existence cross-tenant). *Test E2E HTTP : `post_accept_does_not_leak_cross_tenant_invoice`.*

52. **(POST accept — mutex contention)** Given un lock actif sur `(company_id, bank_account_id)`, When un second POST accept arrive, Then `409 RECONCILIATION_ACCOUNT_LOCKED` après timeout. *Test E2E HTTP avec 2 requêtes concurrentes : `post_accept_returns_409_on_account_lock_contention`.*

### Routes API — reject

53. **(POST reject — happy)** Given POST reject `{ bankTransactionIds: [42, 43] }`, When les 2 transactions sont en `pending`, Then `200 OK` + `auto_match_rejected_at` set sur les 2 + audit log `reconciliation.rejected` (1 entrée pour le batch). *Test E2E HTTP : `post_reject_marks_transactions_as_manually_reviewed`.*

54. **(POST reject — already reconciled)** Given une tx déjà `reconciled`, When POST reject sur cette tx, Then `failed: [{ error_code: 'RECONCILIATION_ALREADY_RECONCILED' }]` (pas d'effet sur `auto_match_rejected_at`). *Test E2E HTTP : `post_reject_skips_reconciled_transactions`.*

### UI frontend `features/reconciliation/`

55. **(UI — page liste propositions)** Given `/reconciliation` avec un `bankAccountId` sélectionné, Then une table affiche : ligne par tx pending, colonnes (date, montant, contrepartie, candidate top-1 invoice avec score badge couleur, checkbox de sélection). *Test Vitest : `ReconciliationProposals.test.ts: renders proposals with score badges`.*

56. **(UI — bouton « Valider sélection »)** Given des checkboxes cochées, When click « Valider », Then POST accept avec le batch + toast succès + refresh liste (les acceptées disparaissent). *Test Playwright : `reconciliation accept batch end-to-end`.*

57. **(UI — bouton « Rejeter sélection »)** Given des checkboxes cochées, When click « Rejeter », Then POST reject + toast succès + transactions disparaissent (filtrées hors-liste). *Test Playwright : `reconciliation reject manual review flow`.*

58. **(UI — pas de candidate)** Given une tx sans candidate (score 0 partout), Then la ligne affiche « Aucune proposition automatique » + lien désactivé/futur vers 8-5 manual. *Test Vitest : `ReconciliationProposals.test.ts: shows neutral state for tx without candidates`.*

### Sécurité & multi-tenant

59. **(KF-002 Pattern 1)** Given user `company_A`, When les helpers `find_unpaid_invoices_for_window` / `bank_transactions::find_pending_for_account` sont appelés, Then ils filtrent **systématiquement** par `company_id = current_user.company_id`. *Tests : sqlx tests AC #38 + #45 + couvert par AC #51 E2E.*

60. **(RBAC — sub-router comptable)** Routes `/api/v1/reconciliation/*` sont dans `comptable_routes` (cf. 8-1b). User `Consultation` → `403 Forbidden`. *Test E2E HTTP : `reconciliation_routes_require_comptable_role`.*

### i18n & accessibilité

61. **(i18n — 4 locales)** Given les ~10 nouvelles clés (`reconciliation-page-title`, `reconciliation-labels-validate-selected`, `reconciliation-labels-reject-selected`, `reconciliation-labels-no-proposal`, `reconciliation-labels-score`, `reconciliation-errors-account-locked`, `reconciliation-errors-already-reconciled`, `reconciliation-errors-invoice-not-eligible`, `reconciliation-toast-accept-success`, `reconciliation-toast-reject-success`), When `npm run lint-i18n-ownership`, Then PASS sur les 4 locales fr/de/it/en-CH. *Test : CI Story 6-3.*

62. **(Accessibilité — axe-core)** Given la page `/reconciliation` rendue avec ≥ 5 propositions, When `axe-core` scan, Then zéro violation. *Test Playwright : `accessibility — reconciliation page axe scan zero violations`.*

### Performance NFR

63. **(perf — propose_matches O(N×M) sur 1000×500 < 50ms)** Given 1000 transactions pending et 500 candidates, When le helper `propose_matches` est appelé en boucle (1 tx × 500 invoices = 500 itérations × 1000 tx), Then la durée totale < 50ms (pure CPU, pas d'I/O). Smoke test instrumenté `Instant::now()` non-bloquant CI (warning si > 50ms). *Test unitaire `kesh-reconciliation::matching` : `propose_matches_handles_1000_x_500_under_50ms`.*

## Tasks / Subtasks

### T1. Migration DB (`bank_transactions.auto_match_rejected_at` + index invoice candidates) (AC #37, #40, #47, #53)

- [ ] T1.1 — Créer `crates/kesh-db/migrations/20260507100001_reconciliation_8_4.sql` :
  ```sql
  -- Story 8-4 — réconciliation matching automatique.
  -- 1. auto_match_rejected_at : tx.status reste 'pending' mais marquée
  --    comme « manuellement revue sans match auto » (cf. §reject-flow).
  ALTER TABLE bank_transactions
    ADD COLUMN auto_match_rejected_at DATETIME(3) NULL AFTER matched_entry_id;

  -- 2. Index pour find_unpaid_invoices_for_window — couvre les 3 colonnes
  --    filtrées par la query du repo (status, paid_at IS NULL implicit, date).
  CREATE INDEX IF NOT EXISTS idx_invoices_company_validated_unpaid_date
    ON invoices (company_id, status, paid_at, date);
  ```
- [ ] T1.2 — Vérifier `cargo test -p kesh-db --lib test_fixtures` avec MariaDB up + KESH_TEST_MODE=true (lesson 8-3 retro : CHECK constraints / index identity à valider DB-up).
- [ ] T1.3 — Ne **pas** modifier `TABLES_TO_TRUNCATE` (les tables existent déjà, on ajoute juste une colonne et un index).
- [ ] T1.4 — Vérifier `EXPLAIN SELECT ... FROM invoices WHERE company_id = ? AND status = ? AND paid_at IS NULL AND date BETWEEN ? AND ?` post-migration — confirmer que le nouvel index est utilisé (`type=range` ou `ref` selon les bornes, pas `ALL`).

### T2. Helper `kesh-reconciliation::matching` (AC #30 à #36, #63)

- [ ] T2.1 — Étendre `crates/kesh-reconciliation/src/lib.rs` :
  ```rust
  pub mod matching;
  pub mod mutex;
  pub mod errors;

  pub use matching::{propose_matches, MatchProposal, MatchScore};
  pub use mutex::with_account_lock;
  pub use errors::ReconciliationError;
  ```
- [ ] T2.2 — Créer `crates/kesh-reconciliation/src/matching.rs` :
  ```rust
  use kesh_db::entities::{BankTransaction, Invoice, Contact};
  use rust_decimal::Decimal;

  /// Score de confiance d'une proposition de matching ∈ [0.0..=1.0].
  /// Pondération : 0.50 amount + 0.40 reference + 0.10 contact.
  #[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
  pub struct MatchScore {
      pub total: f64,
      pub amount_score: f64,
      pub reference_score: f64,
      pub contact_score: f64,
  }

  /// Proposition de matching d'une transaction bancaire vers une facture.
  #[derive(Debug, Clone, serde::Serialize)]
  pub struct MatchProposal {
      pub bank_transaction_id: i64,
      pub invoice_id: i64,
      pub score: MatchScore,
  }

  /// Calcule les propositions pour UNE transaction bancaire contre N
  /// candidates. Retourne uniquement les paires `score.total > 0.0`,
  /// triées par `score.total DESC`. Pure (zéro I/O).
  pub fn propose_matches(
      tx: &BankTransaction,
      candidates: &[(Invoice, Option<Contact>)],
  ) -> Vec<MatchProposal> { ... }

  fn amount_score(tx_amount: Decimal, invoice_amount: Decimal) -> f64 { ... }
  fn reference_score(tx_ref: Option<&str>, tx_eid: Option<&str>, tx_tid: Option<&str>, invoice_number: Option<&str>) -> f64 { ... }
  fn contact_score(tx_counterparty: Option<&str>, contact_name: Option<&str>) -> f64 { ... }
  ```
- [ ] T2.3 — Tests unitaires `#[cfg(test)] mod tests` inline `matching.rs` (≥ 12) :
  1. `score_full_match_returns_1_0` (AC #30).
  2. `score_amount_only_returns_0_50` (AC #31).
  3. `score_reference_prefix_match_returns_0_5` (AC #32).
  4. `score_reference_falls_back_to_end_to_end_id` (AC #33).
  5. `score_reference_falls_back_to_transaction_id` — symétrique fallback chain.
  6. `score_contact_substring_match` (AC #34) — variantes positive et négative.
  7. `score_zero_filters_out_proposal` (AC #35).
  8. `score_amount_mismatch_within_repo_window_returns_partial` (AC #36).
  9. `propose_matches_returns_sorted_desc` — vérifie tri par score.
  10. `propose_matches_empty_candidates_returns_empty` — edge case.
  11. `propose_matches_handles_unicode_normalization` — `"René"` vs `"Rene"` → décision : pas de NFC normalisation v0.1 (cohérent avec 8-3 L16). Test asserte le comportement (pas de match).
  12. `propose_matches_handles_1000_x_500_under_50ms` (AC #63 perf smoke).

### T3. Repository `kesh-db::repositories::reconciliation::find_unpaid_invoices_for_window` (AC #37 à #40, #59)

- [ ] T3.1 — Créer `crates/kesh-db/src/repositories/reconciliation.rs` :
  ```rust
  use chrono::NaiveDate;
  use rust_decimal::Decimal;
  use sqlx::{MySql, mysql::MySqlPool};

  use crate::entities::Invoice;
  use crate::errors::{DbError, map_db_error};

  /// Charge les factures candidates pour la réconciliation d'une
  /// transaction bancaire — filtre temporel ± `window_days` autour de
  /// `tx_date` ET filtre montant ± `amount_tolerance` autour de
  /// `tx_amount`. Multi-tenant scoped (KF-002 Pattern 1).
  pub async fn find_unpaid_invoices_for_window<'e, E>(
      executor: E,
      company_id: i64,
      tx_date: NaiveDate,
      tx_amount: Decimal,
      window_days: i64,
      amount_tolerance: Decimal,
  ) -> Result<Vec<Invoice>, DbError>
  where
      E: sqlx::Executor<'e, Database = MySql>,
  {
      sqlx::query_as::<_, Invoice>(
          "SELECT ... FROM invoices \
           WHERE company_id = ? \
             AND status = 'validated' \
             AND paid_at IS NULL \
             AND date BETWEEN DATE_SUB(?, INTERVAL ? DAY) AND DATE_ADD(?, INTERVAL ? DAY) \
             AND total_amount BETWEEN ? - ? AND ? + ?"
      ).bind(company_id).bind(tx_date).bind(window_days).bind(tx_date).bind(window_days)
       .bind(tx_amount).bind(amount_tolerance).bind(tx_amount).bind(amount_tolerance)
       .fetch_all(executor).await.map_err(map_db_error)
  }

  /// Charge les transactions bancaires `pending` (et NON
  /// `auto_match_rejected_at != NULL`) pour un compte donné.
  pub async fn find_pending_transactions_for_account<'e, E>(...) -> Result<Vec<BankTransaction>, DbError>
  where E: sqlx::Executor<'e, Database = MySql> { ... }
  ```
- [ ] T3.2 — Tests d'intégration `#[sqlx::test]` (≥ 4) — créer `crates/kesh-db/tests/reconciliation_repository.rs` ou inline :
  1. `find_unpaid_invoices_filters_by_30_day_window` (AC #37).
  2. `find_unpaid_invoices_scopes_by_company` (AC #38, #59).
  3. `find_unpaid_invoices_excludes_paid_and_draft` (AC #39).
  4. `find_unpaid_invoices_filters_by_amount_window` (AC #40).
  5. `find_pending_transactions_excludes_auto_match_rejected` (AC #47).

### T4. Helper `kesh-reconciliation::mutex::with_account_lock` (AC #41, #42, #43, #52)

- [ ] T4.1 — Créer `crates/kesh-reconciliation/src/mutex.rs` (cf. §mutex-account pour la signature complète).
- [ ] T4.2 — Créer `crates/kesh-reconciliation/src/errors.rs` avec `ReconciliationError::AccountLocked { bank_account_id, timeout_secs }` + `From<sqlx::Error>`.
- [ ] T4.3 — Tests sqlx (≥ 3) inline ou `tests/mutex.rs` :
  1. `mutex_blocks_concurrent_account_lock` (AC #41) — 2 connexions parallèles via `tokio::spawn`.
  2. `mutex_does_not_block_cross_account` (AC #42).
  3. `mutex_releases_on_error_path` (AC #43).
  4. `mutex_release_lock_on_drop` — bonus : si la closure caller panic, le lock se libère via `Drop` ? *Note : `GET_LOCK` se libère à la fin de session si pas explicitement RELEASE — comportement par défaut MariaDB. Tester implicitement via tokio::time::timeout sur acquisition retry.*

### T5. Routes API `kesh-api::routes::reconciliation` (AC #44 à #54, #60)

- [ ] T5.1 — Créer `crates/kesh-api/src/routes/reconciliation.rs` :
  - `pub fn router() -> Router<AppState>` avec 3 routes : `GET /proposals`, `POST /accept`, `POST /reject`.
  - Sub-router `comptable_routes` (cf. lib.rs:299 pattern 8-1b).
- [ ] T5.2 — Handler `GET /proposals` :
  1. Parse `bankAccountId` de la query string (validation `i64 > 0`).
  2. `find_pending_transactions_for_account(...)`.
  3. Pour chaque tx : `find_unpaid_invoices_for_window` + load `Contact` (1 query par tx, optimisable mais OK v0.1) + `propose_matches`.
  4. Retour shape : `{ proposals: [{ bankTransactionId, transaction: {...summary}, candidates: [{ invoiceId, score: {...}, invoice: {...summary} }] }] }`.
- [ ] T5.3 — Handler `POST /accept` :
  1. Parse body `{ proposals: [{ bankTransactionId, invoiceId }] }`.
  2. Pour chaque proposal : ouvrir une tx, `with_account_lock`, valider chargement + état, `UPDATE bank_transactions SET matched_entry_id = invoice.journal_entry_id, status='reconciled'`, `UPDATE invoices SET paid_at = NOW(...)` (déclencher `mark_paid` si helper existe Story 5-4 sinon UPDATE direct), audit log `reconciliation.accepted`, commit.
  3. Collecter succès/échecs et retourner `{ accepted, failed }`.
- [ ] T5.4 — Handler `POST /reject` :
  1. Parse body `{ bankTransactionIds: [...] }`.
  2. Pour chaque ID : valider `status='pending'`, `UPDATE bank_transactions SET auto_match_rejected_at = NOW()`.
  3. Audit log `reconciliation.rejected` (1 entrée pour le batch).
- [ ] T5.5 — Étendre `crates/kesh-api/src/errors.rs` :
  - `AppError::ReconciliationAccountLocked { bank_account_id, timeout_secs }` → `409 RECONCILIATION_ACCOUNT_LOCKED`.
  - `AppError::ReconciliationAlreadyReconciled { bank_transaction_id }` → `409 RECONCILIATION_ALREADY_RECONCILED`.
  - `AppError::ReconciliationInvoiceNotEligible { invoice_id, reason }` → `409 RECONCILIATION_INVOICE_NOT_ELIGIBLE` (`reason` ∈ `{"invoice_not_validated", "invoice_already_paid"}`).
- [ ] T5.6 — Tests E2E HTTP `crates/kesh-api/tests/reconciliation_e2e.rs` (≥ 6) :
  1. `get_proposals_returns_candidates_with_scores` (AC #44).
  2. `get_proposals_scopes_by_company` (AC #45).
  3. `get_proposals_excludes_rejected_transactions` (AC #47).
  4. `post_accept_reconciles_transaction_and_invoice` (AC #48).
  5. `post_accept_handles_partial_failure` (AC #49).
  6. `post_accept_rejects_unvalidated_or_paid_invoice` (AC #50).
  7. `post_accept_does_not_leak_cross_tenant_invoice` (AC #51).
  8. `post_accept_returns_409_on_account_lock_contention` (AC #52) — 2 requêtes concurrentes via `tokio::spawn` après acquisition manuelle d'un `GET_LOCK` côté test.
  9. `post_reject_marks_transactions_as_manually_reviewed` (AC #53).
  10. `post_reject_skips_reconciled_transactions` (AC #54).
  11. `reconciliation_routes_require_comptable_role` (AC #60) — tester avec un user `Consultation`, attendre `403`.

### T6. Frontend feature `features/reconciliation/` (AC #55 à #58, #62)

- [ ] T6.1 — Créer `frontend/src/lib/features/reconciliation/reconciliation.api.ts` :
  ```ts
  export interface ReconciliationProposal {
      bankTransactionId: number;
      transaction: { date: string; amount: string; counterparty: string };
      candidates: ReconciliationCandidate[];
  }
  export interface ReconciliationCandidate {
      invoiceId: number;
      invoiceNumber: string;
      invoiceAmount: string;
      score: { total: number; amountScore: number; referenceScore: number; contactScore: number };
  }

  export async function getReconciliationProposals(bankAccountId: number): Promise<{ proposals: ReconciliationProposal[] }> { ... }
  export async function acceptReconciliation(proposals: { bankTransactionId: number; invoiceId: number }[]): Promise<...> { ... }
  export async function rejectReconciliation(bankTransactionIds: number[]): Promise<...> { ... }
  ```
- [ ] T6.2 — Créer `frontend/src/routes/(app)/reconciliation/+page.svelte` (route protégée) :
  - Sélecteur `bankAccountId` (réutilise les `bankAccounts` chargés au layout).
  - Table propositions : colonnes (date, montant, contrepartie, top-1 candidate, score badge, checkbox).
  - 2 boutons : « Valider sélection » / « Rejeter sélection ».
  - data-testid sur tous les éléments interactifs (lessons KF-008/KF-010).
- [ ] T6.3 — Score badge component (inline ou extrait) : couleurs `>= 90% vert` / `70-89% jaune` / `< 70% rouge`.
- [ ] T6.4 — Tests Vitest `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts` (≥ 3) :
  1. `renders proposals with score badges` (AC #55).
  2. `shows neutral state for tx without candidates` (AC #58).
  3. `accept button posts batch with selected ids`.

### T7. i18n (AC #61)

- [ ] T7.1 — Ajouter ~10 nouvelles clés dans `crates/kesh-i18n/locales/fr-CH/messages.ftl` (préfixe `reconciliation-*` strict). FR canonical, traductions DE/IT/EN à suivre.
- [ ] T7.2 — Traductions DE / IT / EN — pas de copies françaises (lesson 8-2 H13). Vocabulaire bancaire suisse.
- [ ] T7.3 — Vérifier `npm run lint-i18n-ownership` PASS.

### T8. Tests E2E Playwright (AC #56, #57, #62)

- [ ] T8.1 — Créer `frontend/tests/e2e/reconciliation.spec.ts` (≥ 2 actifs) :
  1. `reconciliation accept batch end-to-end` (AC #56) — login, navigate, select 2 propositions, validate, assert toast + disparition liste.
  2. `reconciliation reject manual review flow` (AC #57) — sélection + rejet, assert disparition.
  3. `accessibility — reconciliation page axe scan zero violations` (AC #62).
- [ ] T8.2 — Helper `seedReconciliationFixture(page, ...)` — crée 1 bank_import + 3 bank_transactions + 2 invoices candidates pour reproductibilité Playwright. **Note** : ce helper closure la dette laissée en 8-3 (2 Playwright skipped pour `seedBankProfile` manquant) — l'opportunité existe ici de poser le pattern réutilisable. Décision : prioritaire car débloque aussi les 2 tests Playwright skipped 8-3 si Guy le souhaite (hors scope strict 8-4 mais bonus si trivial).
- [ ] T8.3 — `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- reconciliation.spec.ts` localement avant push.

### T9. Sync sprint-status + README (AC implicite Epic 8 progress)

- [ ] T9.1 — `_bmad-output/implementation-artifacts/sprint-status.yaml` : transition `8-4-reconciliation-matching-automatique: backlog → ready-for-dev` (cette story creation) puis `ready-for-dev → review` (post `dev-story`).
- [ ] T9.2 — README.md `## Feuille de route` : Epic 8 reste 🚧 En cours (5/5 après merge 8-4 — ou 4/5 si 8-5 reportée).
- [ ] T9.3 — README.md `## Fonctionnalités` : ajouter ligne « Réconciliation automatique des factures avec score de confiance » sous la section import bancaire.
- [ ] T9.4 — Pas d'issue GitHub à fermer (8-4 n'a pas de KF/CR pré-tracée — créer une seule pour Story 8-5 si nécessaire post-implem).

## Dev Notes

### API surface 8-1b/8-2/8-3 livrée — patterns à réutiliser

- **Multi-tenant scoping** (KF-002 Pattern 1) : tous les helpers DB filtrent par `(company_id, ...)`. Cross-tenant = 404, jamais 403.
- **Audit log atomique** : helper `audit_log::insert_in_tx(tx, NewAuditLogEntry { ... })`. Une entrée par operation distincte.
- **Erreurs structurées** : `AppError::Custom { status, code, message, details }` ou variantes typées dédiées (préféré).
- **i18n key ownership** : préfixe `reconciliation-*` strict, kebab-case, lint-i18n-ownership pass (Story 6-3).
- **`rust_decimal::Decimal`** : Decimal exact partout, jamais `f64` pour les montants. Le score est `f64` (lui n'est pas un montant comptable).
- **Repository pattern + sqlx** : `pool: &MySqlPool` ou `&mut Transaction<'_, MySql>` via Executor générique (pattern 8-3 `find_in_dedup_window`).
- **Test locally first** (CLAUDE.md) : avant push, lancer la séquence backend + frontend + E2E. **En particulier T1 (migration)** : vérifier `cargo test -p kesh-db --lib test_fixtures` avec MariaDB up + KESH_TEST_MODE=true (lesson 8-1b/8-3 retro).

### Lessons leçons des stories précédentes

- **8-3 retro** (CHECK constraints invisibles sans MariaDB up) : si T1 ajoute des CHECK constraints, vérifier avec MariaDB up local. *Pour 8-4 : pas de nouveau CHECK, juste un index — risque moindre, mais le seed direct SQL dans les tests E2E doit respecter `chk_bank_transactions_status` (déjà connu post-8-3 hotfix).*
- **8-3 retro** (Pass review effective sur 6 modules) : 3 passes Sonnet → Haiku → Opus suffisent quand patterns acquis. Estimation 8-4 : 2-3 passes attendues.
- **8-2 retro H7** (`ParseCsvOutcome` sans breaking) : pour `kesh-reconciliation` qui démarre vraiment, **pas de contrainte backward-compat publique** — le crate était placeholder. On peut concevoir l'API librement. Mais si on prévoit `cargo publish` future, structurer dès maintenant pour zéro dépendance interne (`kesh-core`/`kesh-db` deps acceptées car non-publishable selon décision archi #7 inverse de `kesh-import`).

### Patterns architecturaux à respecter

- **Pas de dépendance circulaire** : `kesh-reconciliation → kesh-core, kesh-db` (cf. architecture.md:269). Vérifier `cargo metadata` après ajout des deps.
- **Tests : éviter le coupling temporel** : les tests sqlx avec `chrono::Utc::now()` sont fragiles si on compare des timestamps précis. Utiliser des dates fixes (`NaiveDate::from_ymd_opt(2026, 5, 15)`) dans les seeds, pas `Utc::now()`.
- **Pas d'`f64` pour montants** : `score: f64` est OK (c'est un ratio, pas un montant). `tx.amount` et `invoice.total_amount` restent `Decimal`.

### Source tree à toucher

**DB** :
- `crates/kesh-db/migrations/20260507100001_reconciliation_8_4.sql` *(nouveau)*
- `crates/kesh-db/src/repositories/reconciliation.rs` *(nouveau)*
- `crates/kesh-db/src/repositories/mod.rs` (re-export `pub mod reconciliation`)
- `crates/kesh-db/tests/reconciliation_repository.rs` *(nouveau)*
- `crates/kesh-db/src/entities/bank_transaction.rs` (extension `pub auto_match_rejected_at: Option<NaiveDateTime>`)

**Backend `kesh-reconciliation`** *(crate placeholder activé)* :
- `crates/kesh-reconciliation/Cargo.toml` (deps `kesh-core`, `kesh-db`, `sqlx`, `chrono`, `rust_decimal`, `serde`, `thiserror`, `tracing`)
- `crates/kesh-reconciliation/src/lib.rs` (refactor — modules `matching`, `mutex`, `errors`)
- `crates/kesh-reconciliation/src/matching.rs` *(nouveau, pure)*
- `crates/kesh-reconciliation/src/mutex.rs` *(nouveau, advisory lock)*
- `crates/kesh-reconciliation/src/errors.rs` *(nouveau)*

**Backend `kesh-api`** :
- `crates/kesh-api/src/routes/reconciliation.rs` *(nouveau)*
- `crates/kesh-api/src/routes/mod.rs` (`pub mod reconciliation`)
- `crates/kesh-api/src/lib.rs` (mount route under `comptable_routes`)
- `crates/kesh-api/src/errors.rs` (3 nouvelles variantes)
- `crates/kesh-api/Cargo.toml` (dep `kesh-reconciliation`)
- `crates/kesh-api/tests/reconciliation_e2e.rs` *(nouveau)*

**i18n** :
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (~10 nouvelles clés × 4 locales)

**Frontend** :
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` *(nouveau, table)*
- `frontend/src/lib/features/reconciliation/ScoreBadge.svelte` *(nouveau, petit composant)*
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts` *(nouveau)*
- `frontend/src/routes/(app)/reconciliation/+page.svelte` *(nouveau, route)*
- `frontend/src/routes/(app)/reconciliation/+page.ts` *(nouveau, load function pour `bankAccounts`)*
- `frontend/tests/e2e/reconciliation.spec.ts` *(nouveau)*

### Standards de test

- **Unit `kesh-reconciliation`** : `#[cfg(test)] mod tests` inline `matching.rs` + `mutex.rs`. ≥ 12 unit + ≥ 3 sqlx mutex.
- **Intégration `kesh-db`** : `#[sqlx::test]`. ≥ 5 tests T3.2.
- **E2E HTTP `kesh-api`** : helper `spawn_app(pool)` (pattern 8-1b/8-2/8-3). ≥ 11 tests T5.6.
- **Vitest frontend** : `npm run test:unit -- reconciliation`. ≥ 3 tests T6.4.
- **Playwright** : `frontend/tests/e2e/reconciliation.spec.ts`. ≥ 2 actifs + 1 a11y.

### Checklist locale avant push

```sh
# Backend
cargo fmt --all -- --check
cargo build --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -j1 -- --test-threads=1   # MariaDB up requis (T1 migration check + T3 sqlx + T4 mutex tests + T5 E2E)

# Frontend
cd frontend
npm run check
npm run lint-i18n-ownership   # AC #61
npm run test:unit
npm run build

# E2E (MariaDB up + seed CI + browsers installés)
PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64 npm run test:e2e -- reconciliation.spec.ts
```

### Limitations connues v0.1

| # | Limitation | Justification |
|---|---|---|
| L17 | Score amount strictement binaire (0 ou 1, pas de gradient sur écart de centimes) | Choix conservateur v0.1 : un montant à 100.05 vs 100.00 doit être visible comme mismatch (l'utilisateur juge), pas un faux positif silencieux. Gradient amount reporté v0.2 si KF émerge sur trop de propositions « presque-match » non-matchées. |
| L18 | Pas de seuil auto-accept v0.1 | Toutes les propositions `score > 0` sont retournées ; l'utilisateur valide explicitement. Les paramètres seuil par tenant sont reportés v0.2 (couplés aux règles d'affectation 8-5 / FR47). |
| L19 | Pas de matching contre `journal_entries` non-invoice | Un paiement de salaire ou frais bancaires sans facture pré-existante n'est pas matché auto v0.1. L'utilisateur passera par 8-5 manual. Reporté 8-5. |
| L20 | Pas de création d'écriture si `invoice.journal_entry_id IS NULL` | Le `accept` exige une facture déjà validée (`status='validated'` + `journal_entry_id != NULL`). Sinon `409 RECONCILIATION_INVOICE_NOT_ELIGIBLE`. La création d'écriture from-scratch (paiement sans facture) est une feature distincte 8-5 / v0.2. |
| L21 | Pas de paiement partiel (transaction.amount ≠ invoice.total_amount avec tolérance > 0.05) | Le repo filtre amount ± 0.05 ; au-delà, la facture n'est pas remontée comme candidate. Acceptable v0.1 : les paiements partiels en suisse sont rares (paiement à 30 jours est typiquement complet). Reporté v0.2 (8-5 manual avec écart documenté). |
| L22 | Mutex `GET_LOCK` libéré à la fin de session DB si l'app crashe avant `RELEASE_LOCK` | Comportement standard MariaDB. La fenêtre de blocage est bornée par la durée d'une connexion idle (typique pool sqlx ~5min). Acceptable v0.1 ; mitigation curative `KILL CONNECTION` admin si lock zombie en prod (rarissime). |
| L23 | `auto_match_rejected_at` non réversible v0.1 | Un user qui rejette par erreur doit aller en 8-5 manual pour traiter la transaction. Pas de bouton « annuler le rejet » v0.1. |

### Références

- Spec d'origine 8-3 (story précédente) : [`8-3-detection-doublons-rejet-partiel.md`](8-3-detection-doublons-rejet-partiel.md)
- Spec 8-1b (foundation `bank_transactions.status` + `matched_entry_id`) : [`8-1b-camt053-persistence-ui.md`](8-1b-camt053-persistence-ui.md)
- Epic 8 plan : [`epic-8.md`](../planning-artifacts/epic-8.md) — §Risques R5 (score de confiance)
- PRD : [`prd.md`](../planning-artifacts/prd.md) — FR44 (§438), §134 scenario Sophie réconciliation
- Architecture : [`architecture.md`](../planning-artifacts/architecture.md) — §11.5 dépendances inter-crates (kesh-reconciliation), §17 carte FR → modules, ligne 491-498 structure crate
- KF-020 #49 closed (PR #64) : `SELECT FOR UPDATE` foundation pour cohabitation avec `GET_LOCK` advisory
- KF-002-H-002 #43 closed (PR #65) : deadlock-retry middleware

## Dev Agent Record

### Agent Model Used

(à remplir par dev-story)

### Debug Log References

(à remplir par dev-story)

### Completion Notes List

(à remplir par dev-story)

### File List

(pré-rempli depuis §Source tree pour éviter d'oublier un fichier en fin de story.)

**DB** :
- `crates/kesh-db/migrations/20260507100001_reconciliation_8_4.sql` *(nouveau)*
- `crates/kesh-db/src/repositories/reconciliation.rs` *(nouveau)*
- `crates/kesh-db/src/repositories/mod.rs`
- `crates/kesh-db/tests/reconciliation_repository.rs` *(nouveau)*
- `crates/kesh-db/src/entities/bank_transaction.rs`

**Backend `kesh-reconciliation`** :
- `crates/kesh-reconciliation/Cargo.toml`
- `crates/kesh-reconciliation/src/lib.rs`
- `crates/kesh-reconciliation/src/matching.rs` *(nouveau)*
- `crates/kesh-reconciliation/src/mutex.rs` *(nouveau)*
- `crates/kesh-reconciliation/src/errors.rs` *(nouveau)*

**Backend `kesh-api`** :
- `crates/kesh-api/src/routes/reconciliation.rs` *(nouveau)*
- `crates/kesh-api/src/routes/mod.rs`
- `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/Cargo.toml`
- `crates/kesh-api/tests/reconciliation_e2e.rs` *(nouveau)*

**i18n** :
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl`

**Frontend** :
- `frontend/src/lib/features/reconciliation/reconciliation.api.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/reconciliation.types.ts` *(nouveau)*
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/ScoreBadge.svelte` *(nouveau)*
- `frontend/src/lib/features/reconciliation/ReconciliationProposals.test.ts` *(nouveau)*
- `frontend/src/routes/(app)/reconciliation/+page.svelte` *(nouveau)*
- `frontend/src/routes/(app)/reconciliation/+page.ts` *(nouveau)*
- `frontend/tests/e2e/reconciliation.spec.ts` *(nouveau)*

**Story file & sprint** :
- `_bmad-output/implementation-artifacts/8-4-reconciliation-matching-automatique.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

### Change Log

| Date | Action | Auteur |
|------|--------|--------|
| 2026-05-06 | Création de la story par `/bmad-create-story 8-4` post-merge PR #75 (Story 8-3 done). Spec construite à partir d'epic-8.md Story 8-4 ACs (FR44 + matching automatique) + foundation 8-1b (`bank_transactions.status` + `matched_entry_id` posés en avance) + patterns 8-2/8-3 (multi-tenant scoping, audit log canonique, RBAC sub-router, repository Executor générique). 34 ACs définis (AC #30 à #63 — numérotation continue 8-3) + 9 tasks T1-T9 + 6 modules touchés (au seuil > 5 — splitting risque documenté). Décisions de conception clés : §matching-algo (score f64 ∈ [0..1] pondéré 0.50 amount + 0.40 reference + 0.10 contact, helper pure sans I/O), §candidate-window (filtrage repo ± 30 jours + ± 0.05 CHF avec index dédié `idx_invoices_company_validated_unpaid_date`), §mutex-account (advisory lock MariaDB `GET_LOCK('reconcile:company_id:bank_account_id', 5s)` portable multi-instance), §accept-flow (transaction par proposal + partial success + audit log par tx), §reject-flow (colonne `auto_match_rejected_at` ajoutée, transactions exclues de `/proposals` après reject), §audit-log-actions (`reconciliation.accepted`/`reconciliation.rejected` actions plain, pas de modifiers Vec — variantes confirmables absentes ici), §error-precedence-order (8 niveaux d'erreurs documentés). Crate `kesh-reconciliation` (placeholder existant Story 1-1) **vraiment activé** : 3 modules `matching` + `mutex` + `errors`. Pas de breaking change `kesh-import` (8-3 invariant publishable préservé, mais `kesh-reconciliation` n'est pas publishable selon décision archi #7 inverse). Splitting risque CLAUDE.md documenté : 6 modules (kesh-reconciliation, kesh-db, kesh-api, frontend, kesh-i18n, kesh-core) au-dessus du seuil > 5 ; pas de split préventif (volume estimé ~1800-2200 lignes, patterns acquis 8-2/8-3, frontière `kesh-reconciliation core (matching+mutex+repo)` vs `API+frontend` pré-identifiée comme split rétroactif possible si Pass 4 spec validate ne converge pas). Limitations v0.1 documentées : L17 score amount binaire, L18 pas de seuil auto-accept, L19 pas de matching journal_entries non-invoice, L20 pas de création écriture sans facture validée, L21 pas de paiement partiel, L22 GET_LOCK libéré à fin session si crash app, L23 reject non-réversible v0.1. Hors scope explicite : FR45 (création manuelle), FR46/FR47 (règles d'affectation), FR48 (éclatement transaction agrégée) — tous reportés Story 8-5. Status `8-4-reconciliation-matching-automatique: backlog → ready-for-dev`. Status sync : 8-3 `review → done` (post-merge PR #75 squash 60daf26). Prochaine étape : `bmad-create-story validate 8-4` Pass 1 Sonnet (cycle CLAUDE.md, auteur=Opus, Pass 1=Sonnet pour briser biais d'auteur). | Claude (Opus 4.7 1M context, bmad-create-story exécution) |
