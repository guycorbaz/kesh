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

1. **Algorithme de matching** (FR44 partie 1) — `kesh_reconciliation::matching::propose_matches(tx: &BankTransaction, candidates: &[(Invoice, Option<Contact>)]) -> Vec<MatchProposal>` (signature canonique alignée avec T2.2 — H6 Pass 1 patch). Pure (sans I/O), unit-testable. Score de confiance ∈ [0.0..=1.0] dérivé de 3 critères pondérés (montant exact, référence, contact). Le caller (kesh-api) charge les `Contact` correspondants en parallèle des `Invoice` candidates pour permettre le scoring contact (cf. §matching-algo). Cf. §matching-algo pour la signature détaillée + §candidate-window pour le candidate set.
2. **Repository candidates** — `kesh_db::repositories::reconciliation::find_unpaid_invoices_for_window(pool, company_id, period_from, period_to, amount_min, amount_max) -> Vec<Invoice>` filtré multi-tenant + fenêtre temporelle ± 30 jours autour de la transaction + filtre montant ± 0.05 CHF (cf. §candidate-window). L'amount window évite de remonter 100% des factures impayées sur grandes companies.
3. **Mutex par compte bancaire** — advisory lock MariaDB `GET_LOCK('reconcile:company_id:bank_account_id', timeout)` + `RELEASE_LOCK` autour du flow `propose_matches → accept_batch → INSERT journal_entry/UPDATE bank_transactions.status` pour empêcher les imports concurrents de proposer/finaliser les mêmes propositions sur le même compte. Cf. §mutex-account.
4. **Routes API** :
   - `GET /api/v1/reconciliation/proposals?bankAccountId={id}` — retourne les propositions pour les `bank_transactions.status='pending'` du compte.
   - `POST /api/v1/reconciliation/accept` — body : `{ bankAccountId: number, proposals: [{ bankTransactionId, invoiceId }] }` — `bankAccountId` requis pour acquérir le mutex AVANT de charger les transactions (H2 Pass 1 patch — sinon le lock dépendrait d'une donnée chargée hors-lock). **Toutes** les propositions du batch DOIVENT pointer sur des transactions du même `bankAccountId` ; mismatch détecté retourne `400 Validation`. Met à jour `bank_transactions.status='reconciled'` + `matched_entry_id` et **lie** chaque transaction à l'écriture existante de la facture (`invoice.journal_entry_id`). **Pas de création d'écriture v0.1** — l'écriture « Banque / Client » sera créée par 8-5 ou v0.2 (cf. **L20**).
   - `POST /api/v1/reconciliation/reject` — body : `{ bankAccountId: number, bankTransactionIds: [...] }` (cohérent avec accept — M2 Pass 1 patch ajoute le mutex sur reject aussi). Pour refuser explicitement des propositions automatiques (la transaction reste `pending` mais marquée comme « manually reviewed » via colonne `auto_match_rejected_at`, à exploiter en 8-5).
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
| **Référence** | 0.40 | Cf. pseudo-code formel ci-dessous (M3 Pass 1 patch — boundary 1.0 vs 0.5 vs 0.0 explicitée). |
| **Contact** | 0.10 | `1.0` si match substring **bidirectionnel** sur les noms normalisés ; sinon `0.0`. (H4 Pass 1 patch.) |

**Pseudo-code `reference_score`** :

```rust
fn reference_score(tx_ref: Option<&str>, tx_eid: Option<&str>, tx_tid: Option<&str>, invoice_number: Option<&str>) -> f64 {
    let tx_norm = normalize(coalesce(tx_ref, tx_eid, tx_tid).unwrap_or("")); // trim + lowercase
    let inv_norm = normalize(invoice_number.unwrap_or(""));
    if tx_norm.is_empty() || inv_norm.is_empty() {
        return 0.0; // Référence absente d'un côté ou l'autre — pas de match crédible.
    }
    if tx_norm.contains(&inv_norm) || inv_norm.contains(&tx_norm) {
        // Containment dans une direction OU l'autre : ex. tx="REF INV-2026-001 PAID" contient
        // invoice="inv-2026-001", OU tx="INV-2026-001" est contenue dans invoice="INV-2026-001-A".
        return 1.0;
    }
    // Préfixe partagé ≥ 4 chars (utile pour numéros tronqués) — bidirectionnel.
    // A2-1 (Pass 2 review) — `.chars()` au lieu de `.bytes()` pour
    // compter des graphèmes UTF-8 logiques. Les noms suisses contiennent
    // souvent des chars non-ASCII (Müller, École, René, Sàrl) ; un seuil
    // exprimé en chars est plus prévisible que en bytes pour le user.
    let common_prefix = tx_norm.chars().zip(inv_norm.chars()).take_while(|(a, b)| a == b).count();
    if common_prefix >= 4 {
        return 0.5;
    }
    0.0
}
```

**Pseudo-code `contact_score`** :

```rust
fn contact_score(tx_counterparty: Option<&str>, contact_name: Option<&str>) -> f64 {
    let tx_norm = normalize(tx_counterparty.unwrap_or(""));
    let contact_norm = normalize(contact_name.unwrap_or(""));
    if tx_norm.is_empty() || contact_norm.is_empty() {
        return 0.0;
    }
    // H4 Pass 1 patch — bidirectionnel : CAMT.053 contient souvent un nom plus long
    // (ville/forme juridique) que `contact.name` (forme courte saisie utilisateur),
    // mais l'inverse est aussi possible (CSV bruité, contact.name avec suffixe). On
    // accepte les deux directions, plus tolérant pour le user.
    if tx_norm.contains(&contact_norm) || contact_norm.contains(&tx_norm) {
        1.0
    } else {
        0.0
    }
}
```

**Score final** : `score = 0.50 * amount_score + 0.40 * reference_score + 0.10 * contact_score`. Décision conservatrice du poids amount (50%) : la combinaison « bon montant + bonne référence » garantit ≥ 0.90, ce qui matérialise le cas nominal QR-Bill paiement (référence 27 digits = invoice_number, montant exact). Le contact reste minoritaire car les noms contreparties dans CAMT.053 sont parfois bruités (ex. « ALICE MARTIN-DUBOIS GMBH » vs `contact.name = "Alice Martin-Dubois Sàrl"` — l'utilisateur peut tolérer 0.10 de pertes).

**Sérialisation et arrondi (M6 Pass 1 patch)** : le score est sérialisé JSON en `f64` brut côté API, mais le frontend affiche `Math.round(score.total * 100)` (entier 0..100) pour le badge couleur. Boundary IEEE 754 : `0.5 + 0.4 = 0.8999999999999999` selon la machine — `Math.round(0.8999999... * 100) = 90` (rounding half-to-even arrondit à l'entier le plus proche). La règle backend miroir Rust : `(score.total * 100.0).round() as u8` pour les badge thresholds. Documenter dans T6.3 frontend.

**Justification R5 (epic-8.md)** — Score de confiance, seuil par défaut, configurabilité, exposé UI :
- **Seuil par défaut v0.1** : aucun (toutes les propositions avec `score > 0.0` sont retournées). Le frontend trie par `score DESC` et affiche les top-N (N=3 par défaut, configurable v0.2). L'utilisateur **sélectionne explicitement** ce qu'il accepte ; pas d'auto-acceptation v0.1.
- **Configurabilité** : différée v0.2 (paramètre `auto_accept_threshold` par tenant ou par règle 8-5).
- **Exposé UI** : oui, score affiché en pourcent (`78%`) avec badge couleur seuil empirique (≥ 90% vert, 70-89% jaune, < 70% rouge) — décision UX inspirée du flow PRD ligne 112 (scénario Marc indépendant : import PostFinance avec propositions automatiques de paiement). L1 Pass 1 patch (auparavant §134 Sophie était une mauvaise référence — §134 traite des doublons d'import, pas de la réconciliation).

**Pure helper** : `propose_matches` ne fait pas d'I/O. Le caller (`kesh-api`) charge `candidates` via `find_unpaid_invoices_for_window` et passe les vecs au helper. Cette pureté permet le test unitaire intensif (12+ cas) sans setup DB.

#### §candidate-window (fenêtre de candidats)

Pour éviter d'appliquer le matching contre **toutes** les factures impayées de la company (qui peut en avoir des centaines), `find_unpaid_invoices_for_window` filtre par :

1. **`company_id`** (KF-002 Pattern 1, scoping multi-tenant systématique).
2. **`status = 'validated' AND paid_at IS NULL AND journal_entry_id IS NOT NULL`** (factures validées **avec écriture comptable déjà émise** et non payées — l'état des factures éligibles à la réconciliation v0.1). **M1 Pass 1 patch** : le filtre `journal_entry_id IS NOT NULL` est ajouté au repo pour éviter de remonter des candidates qui seraient rejetées au step 6 de §accept-flow (UX hostile : remonter une candidate haute score puis 409 à l'accept). Le cas `journal_entry_id IS NULL` sur facture validée est pathologique mais possible si une migration s'est mal passée — sera adressé par 8-5 manual.
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

#### §accept-flow (flow d'acceptation d'un batch de propositions)

`POST /api/v1/reconciliation/accept` body `{ bankAccountId: number, proposals: [{ bankTransactionId, invoiceId }] }` :

**Préambule (avant la boucle)** :

0. Validation body : `proposals.len() > 0`, `bankAccountId > 0`, tous les `bankTransactionId` distincts (pas de doublons dans le batch). Sinon `400 Validation`.
1. **Acquire UN seul lock pour tout le batch (H5 Pass 1 patch)** : `tx_outer = pool.begin().await?` ; `with_account_lock(&mut tx_outer, company_id, bank_account_id, 5)`. Le lock est tenu pour **toute la durée du batch**, pas par-proposal — sinon une requête concurrente peut s'intercaler entre proposals N et N+1.

**Boucle proposal-par-proposal (à l'intérieur du lock partagé)** :

Pour chaque proposition, on utilise un **savepoint MariaDB** pour garantir partial success sans rollback du lot. Le nom de savepoint `sp_{bank_transaction_id}` est garanti unique par la validation step 0 (« tous les `bankTransactionId` distincts ») — C2-5 Pass 2 patch confirme que la collision est impossible dans un même batch. Cas inter-batch sans intérêt car chaque batch ouvre sa propre `tx_outer`.

```rust
for proposal in &fields.proposals {
    let savepoint = format!("sp_{}", proposal.bank_transaction_id);
    sqlx::query(&format!("SAVEPOINT {savepoint}")).execute(&mut *tx_outer).await?;
    match accept_one(&mut *tx_outer, proposal, &fields, current_user).await {
        Ok(accepted_entry) => {
            sqlx::query(&format!("RELEASE SAVEPOINT {savepoint}")).execute(&mut *tx_outer).await?;
            response.accepted.push(accepted_entry);
        }
        Err(e) => {
            sqlx::query(&format!("ROLLBACK TO SAVEPOINT {savepoint}")).execute(&mut *tx_outer).await?;
            response.failed.push(FailedProposal { proposal: *proposal, error_code: e.code() });
        }
    }
}
```

**Helper `accept_one`** — flow par proposition (C2-6 Pass 2 patch — la numérotation 2-10 ci-dessous est volontairement continue avec les steps 0-1 du préambule, traités hors-helper avant la boucle ; pas de step 1 dans le helper) :

2. Charger `BankTransaction` (scoped `company_id`) — `404 BANK_TRANSACTION_NOT_FOUND` si introuvable / cross-tenant.
3. **Vérifier `bank_transaction.bank_account_id == fields.bank_account_id`** — sinon `400 Validation` (proposition incohérente avec le lock acquis sur ce compte ; H2 Pass 1 patch).
4. Vérifier `bank_transaction.status == 'pending'` — sinon `409 RECONCILIATION_ALREADY_RECONCILED`.
5. Charger `Invoice` (scoped `company_id`) — `404 INVOICE_NOT_FOUND` si introuvable / cross-tenant.
6. Vérifier `invoice.status == 'validated' AND invoice.paid_at IS NULL AND invoice.journal_entry_id IS NOT NULL` — sinon `409 RECONCILIATION_INVOICE_NOT_ELIGIBLE` avec `details.reason ∈ {"invoice_not_validated", "invoice_already_paid", "invoice_journal_entry_not_set"}`. Le filtre `journal_entry_id IS NOT NULL` est aussi appliqué en amont au repo (M1 Pass 1 patch) pour éviter de remonter des candidats inéligibles dans GET proposals.
7. **Re-calculer le score localement (M7 Pass 1 patch)** : appeler `propose_matches(&bank_transaction, &[(invoice.clone(), Some(contact))])` — le score doit être ré-évalué côté serveur (pas pris du body) pour audit trail fiable. Si le score résultant est `0.0` (cas pathologique : la candidate window a remonté l'invoice mais aucun critère ne match), accepter quand même (l'utilisateur a explicitement validé) mais log warning.
8. **Liaison écriture comptable (H1 Pass 1 patch — redesign v0.1 « link only ») — UPDATE inline dans la même tx** :
   - `UPDATE bank_transactions SET matched_entry_id = invoice.journal_entry_id, status = 'reconciled', updated_at = NOW(3), version = version + 1 WHERE id = ? AND company_id = ?`
   - `UPDATE invoices SET paid_at = ?, version = version + 1, updated_at = NOW(3) WHERE id = ? AND company_id = ? AND version = ?` avec `paid_at = bank_transaction.value_date.unwrap_or(bank_transaction.booking_date)` (timestamp source défini : value_date prioritaire si présent — date effective de l'écriture en banque, sinon booking_date) **et** `version = ?` matche la version chargée au step 5 (optimistic lock pour éviter race vs un autre flow qui modifierait la facture entre step 5 et step 8 — pattern hérité Story 5-4).
   - **Pas d'appel à `mark_as_paid` du repo invoices** : sa signature `pool: &MySqlPool` est incompatible avec la `&mut Transaction` ouverte par `with_account_lock` (nested tx interdites MariaDB), et son code-comment confirme « Ne crée AUCUNE écriture comptable en v0.1 ». L'UPDATE inline reproduit le contrat sans changement de comportement.
   - **Pas de création d'écriture comptable « Banque / Client »** v0.1 — déférée 8-5 (création manuelle par compte de contrepartie) ou v0.2 (auto via règles d'affectation FR47). Cf. **L20**.
9. Audit log `reconciliation.accepted` avec `details = { bank_transaction_id, invoice_id, score: { total, amountScore, referenceScore, contactScore }, batch_size, journal_entry_id }` — schéma aligné avec §audit-log-actions (M4 Pass 1 patch). Une entrée par proposal acceptée.
10. Retourner `AcceptedProposal { bank_transaction_id, invoice_id, journal_entry_id, score }`.

**Post-boucle** : `RELEASE_LOCK` (M9 Pass 1 patch — explicitement avant le commit pour libérer la connexion plus tôt) + `tx_outer.commit()`. Si `commit` échoue, **toutes** les acceptations sont rollback (transaction unique) — c'est le comportement DB nominal. Le response body est calculé avant commit ; en cas d'échec commit on retourne `500 Internal` et le frontend doit retry l'ensemble.

**Atomicité savepoint** : si la 5e proposal d'un batch de 10 échoue (ex. `409 RECONCILIATION_ALREADY_RECONCILED`), seul son savepoint est rollback ; les 4 premières et les 5 suivantes peuvent succeed. Le response body retourne `{ accepted: [...], failed: [{ proposal, error_code, details? }] }`. **Partial success > all-or-nothing** car un batch de 50 où une seule proposition a un état caduc casserait tout le travail utilisateur.

#### §reject-flow

`POST /api/v1/reconciliation/reject` body `{ bankAccountId: number, bankTransactionIds: [42, 43, 99] }` — marque ces transactions comme « manuellement revues mais sans match auto ».

**Lock partagé (M2 Pass 1 patch — symétrie avec accept)** : un POST reject concurrent à un POST accept sur la même tx pourrait poser `auto_match_rejected_at` sur une tx qui vient d'être `reconciled`. Le mutex `with_account_lock(company_id, bankAccountId, 5s)` est acquis en début de handler pour serializer les flows accept/reject sur le même compte.

**Flow** :

1. Validation body : `bankTransactionIds.len() > 0`, `bankAccountId > 0`, IDs distincts. Sinon `400 Validation`.
2. Acquire `with_account_lock(&mut tx, company_id, bankAccountId, 5)`.
3. Pour chaque ID, dans la même tx : vérifier `bank_transaction.bank_account_id == fields.bank_account_id` (sinon `400 Validation`), vérifier `status == 'pending'` (sinon `failed: [{ error_code: 'RECONCILIATION_ALREADY_RECONCILED' }]`), puis `UPDATE bank_transactions SET auto_match_rejected_at = NOW(3), updated_at = NOW(3), version = version + 1 WHERE id = ? AND company_id = ? AND status = 'pending'`. La condition `status = 'pending'` dans le WHERE est redondante avec le check applicatif mais protège contre une race théorique.
4. Audit log `reconciliation.rejected` avec `details = { bank_transaction_ids: [...success ids], count }` (1 entrée par batch ; les IDs dans `failed` ne sont pas inclus dans l'audit).
5. `RELEASE_LOCK` explicit + `tx.commit()`.

**Response body shape (C2-1 Pass 2 patch — symétrique POST accept)** :

```json
{
  "rejected": [{ "bankTransactionId": 42, "rejectedAt": "2026-05-06T18:30:00Z" }, { "bankTransactionId": 43, "rejectedAt": "2026-05-06T18:30:00Z" }],
  "failed": [{ "bankTransactionId": 99, "errorCode": "RECONCILIATION_ALREADY_RECONCILED" }]
}
```

Le frontend utilise `rejected[*].bankTransactionId` pour disparition de la liste UI ; `failed[*]` pour toast d'erreur partielle.

**Effet sur les autres helpers** : `find_pending_transactions_for_account` (cf. §candidate-window pour la fonction qui liste les transactions à proposer côté GET) **filtre** `auto_match_rejected_at IS NULL` (L6 Pass 1 patch — la fonction qui exclut est `find_pending_transactions_for_account`, **pas** `find_unpaid_invoices_for_window` qui interroge `invoices`). Une transaction rejetée n'apparaît plus dans `GET /proposals`.

**Page 8-5 manual** (à venir) listera ces transactions en priorité (« attendent un appariement manuel »).

**Note v0.1** : on **ne supprime pas** le `auto_match_rejected_at` si l'utilisateur change d'avis ; il devra passer par 8-5 manual pour faire le matching (cf. **L23**).

#### §audit-log-actions (8-4)

Symétrique au pattern 8-3 (canonical action + modifiers triés). Deux nouvelles actions :

| Action | Contexte | `details_json` |
|---|---|---|
| `reconciliation.accepted` | Un lot accepté | `{ bank_transaction_id, invoice_id, score: { total, amountScore, referenceScore, contactScore }, batch_size, journal_entry_id }` (une entrée par transaction acceptée — M4 Pass 1 patch : `score` est un objet, `journal_entry_id` toujours présent ; A2-9 Pass 2 patch : `batch_size = total proposals submitted in the request body` indépendamment des succès/échecs — permet à un audit a posteriori de calculer le taux d'acceptation `accepted_count / batch_size` du batch) |
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

**Modules touchés par 8-4** : **5** (L9 Pass 1 patch — `kesh-core` retiré : aucune extension `BankTransactionDraft` n'est requise pour le matching, qui se fait sur `BankTransaction` chargé depuis `kesh-db` directement) — (`kesh-reconciliation` *(nouveau crate vraiment activé)*, `kesh-db`, `kesh-api`, `frontend`, `kesh-i18n`). **Au seuil > 5** est non-atteint, splitting préventif non requis — la story garde sa scope unique.

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

44. **(GET proposals — happy)** Given un compte avec 3 transactions `pending` dont 2 ont des factures candidates, When `GET /api/v1/reconciliation/proposals?bankAccountId=17`, Then `200 OK` avec body conforme au schéma canonique (H3 Pass 1 patch — aligné sur T6.1 TypeScript) :
    ```json
    {
      "proposals": [
        {
          "bankTransactionId": 42,
          "transaction": {
            "bookingDate": "2026-05-15",
            "amount": "1234.56",
            "currency": "CHF",
            "counterpartyName": "ACME GMBH"
          },
          "candidates": [
            {
              "invoiceId": 101,
              "invoiceNumber": "INV-2026-001",
              "invoiceAmount": "1234.56",
              "invoiceDate": "2026-04-20",
              "score": {
                "total": 1.0,
                "amountScore": 1.0,
                "referenceScore": 1.0,
                "contactScore": 1.0
              }
            }
          ]
        }
      ]
    }
    ```
    Le 3e (sans candidate) apparaît avec `candidates: []`. **Pas de booléens flags `amountMatch`/`referenceMatch`/`contactMatch`** — le frontend dérive ces flags localement depuis `score.amountScore == 1.0` etc. si besoin d'affichage. *Test E2E HTTP : `get_proposals_returns_candidates_with_scores`.*

45. **(GET proposals — multi-tenant)** Given une transaction `pending` du `company_B` sur le même IBAN qu'un compte `company_A`, When user `company_A` GET proposals, Then la transaction `company_B` n'apparaît pas. *Test E2E HTTP : `get_proposals_scopes_by_company`.*

46. **(GET proposals — RBAC)** Le sub-router `comptable_routes` (hérité 8-1b) protège la route. **Test consolidé dans AC #60** (L7 Pass 1 patch — auparavant AC #46 disait « pas re-testé » alors qu'AC #60 + T5.6#11 le teste effectivement ; redondance résolue en pointant ici vers AC #60).

47. **(GET proposals — exclude auto_match_rejected)** Given une tx avec `auto_match_rejected_at != NULL`, When GET proposals, Then la tx n'apparaît pas (réservée à 8-5 manual). *Test E2E HTTP : `get_proposals_excludes_rejected_transactions`.*

### Routes API — accept

48. **(POST accept — happy)** Given une proposition `{ bankTransactionId: 42, invoiceId: 101 }` valide envoyée dans body `{ bankAccountId: 17, proposals: [...] }` (H2 Pass 1 patch — bankAccountId requis), When POST accept, Then `200 OK` body `{ accepted: [{ bankTransactionId: 42, invoiceId: 101, journalEntryId: 999, score: { total: 1.0, ... } }], failed: [] }` + `bank_transactions.status='reconciled'` + `bank_transactions.matched_entry_id=999` + `invoices.paid_at = bank_transaction.value_date.unwrap_or(bank_transaction.booking_date)` (timestamp source défini, C2 Pass 1 patch) + audit log `reconciliation.accepted` avec `details.score.total = 1.0` + `details.journal_entry_id = 999`. *Test E2E HTTP : `post_accept_reconciles_transaction_and_invoice`.*

49. **(POST accept — partial success)** Given un batch de 3 proposals dont 1 a un état caduc (tx déjà `reconciled`), When POST accept, Then `200 OK` body `{ accepted: [2 entries], failed: [1 entry with error_code=RECONCILIATION_ALREADY_RECONCILED] }`. Les 2 succès sont commit, le 3e n'est pas rollback. *Test E2E HTTP : `post_accept_handles_partial_failure`.*

50. **(POST accept — invoice not eligible)** Given proposition pointant sur `invoice.status='draft'` (pas validée), When POST accept, Then `failed: [{ error_code: 'RECONCILIATION_INVOICE_NOT_ELIGIBLE', details: { reason: 'invoice_not_validated' } }]`. Symétrique pour `paid_at != NULL`. *Test E2E HTTP : `post_accept_rejects_unvalidated_or_paid_invoice`.*

51. **(POST accept — multi-tenant safety)** Given user `company_A` POST accept avec `invoiceId` appartenant à `company_B`, Then `failed: [{ error_code: '404', details: { reason: 'invoice_not_found' } }]` (pas 403 — pas de leak d'existence cross-tenant). *Test E2E HTTP : `post_accept_does_not_leak_cross_tenant_invoice`.*

52. **(POST accept — mutex contention)** Given un lock actif sur `(company_id, bank_account_id)`, When un second POST accept arrive, Then `409 RECONCILIATION_ACCOUNT_LOCKED` après timeout. *Test E2E HTTP avec 2 requêtes concurrentes : `post_accept_returns_409_on_account_lock_contention`.*

### Routes API — reject

53. **(POST reject — happy)** Given POST reject `{ bankAccountId: 17, bankTransactionIds: [42, 43] }` (H2 Pass 1 patch — bankAccountId requis), When les 2 transactions sont en `pending`, Then `200 OK` + body `{ rejected: [{ bankTransactionId: 42, ... }, { bankTransactionId: 43, ... }], failed: [] }` (C2-1 Pass 2 patch shape) + `auto_match_rejected_at` set sur les 2 + audit log `reconciliation.rejected` 1 entrée pour le batch avec `details = { bank_transaction_ids: [42, 43], count: 2 }` (C2-3 Pass 2 patch shape explicite). *Test E2E HTTP : `post_reject_marks_transactions_as_manually_reviewed`.*

54. **(POST reject — already reconciled)** Given une tx déjà `reconciled`, When POST reject sur cette tx, Then `failed: [{ error_code: 'RECONCILIATION_ALREADY_RECONCILED' }]` (pas d'effet sur `auto_match_rejected_at`). *Test E2E HTTP : `post_reject_skips_reconciled_transactions`.*

### UI frontend `features/reconciliation/`

55. **(UI — page liste propositions)** Given `/reconciliation` avec un `bankAccountId` sélectionné, Then une table affiche : ligne par tx pending, colonnes (date, montant, contrepartie, candidate top-1 invoice avec score badge couleur, checkbox de sélection). *Test Vitest : `ReconciliationProposals.test.ts: renders proposals with score badges`.*

56. **(UI — bouton « Valider sélection »)** Given des checkboxes cochées, When click « Valider », Then POST accept avec le batch + toast succès + refresh liste (les acceptées disparaissent). *Test Playwright : `reconciliation accept batch end-to-end`.*

57. **(UI — bouton « Rejeter sélection »)** Given des checkboxes cochées, When click « Rejeter », Then POST reject + toast succès + transactions disparaissent (filtrées hors-liste). *Test Playwright : `reconciliation reject manual review flow`.*

58. **(UI — pas de candidate)** Given une tx sans candidate (score 0 partout), Then la ligne affiche « Aucune proposition automatique » + lien désactivé/futur vers 8-5 manual. *Test Vitest : `ReconciliationProposals.test.ts: shows neutral state for tx without candidates`.*

### Sécurité & multi-tenant

59. **(KF-002 Pattern 1)** Given user `company_A`, When les helpers `find_unpaid_invoices_for_window` / `find_pending_transactions_for_account` (C2-2 Pass 2 patch — nom canonique aligné avec T3.1bis) sont appelés, Then ils filtrent **systématiquement** par `company_id = current_user.company_id`. *Tests : sqlx tests AC #38 + #45 + couvert par AC #51 E2E.*

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
  --
  -- M8 (Pass 1 review) — ALGORITHM=INSTANT évite la copie complète de
  -- la table sur MariaDB 10.3+ (instant ADD COLUMN nullable). Sans
  -- cette directive, ALTER TABLE bloque les writes pendant minutes/heures
  -- sur grandes tables production. LOCK=NONE garantit la concurrent
  -- DML pendant la migration.
  ALTER TABLE bank_transactions
    ADD COLUMN auto_match_rejected_at DATETIME(3) NULL AFTER matched_entry_id,
    ALGORITHM=INSTANT, LOCK=NONE;

  -- 2. Index pour find_unpaid_invoices_for_window — couvre les 4 colonnes
  --    filtrées par la query du repo (status, paid_at IS NULL implicit,
  --    journal_entry_id IS NOT NULL implicit, date). M1 Pass 1 patch :
  --    le filtre journal_entry_id IS NOT NULL est ajouté au repo, mais
  --    l'index reste sur (company_id, status, paid_at, date) pour
  --    couvrir le cas nominal sans bloating l'index avec journal_entry_id.
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
             AND journal_entry_id IS NOT NULL \
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
- [ ] T3.1bis — Implémenter `find_pending_transactions_for_account` (L10 Pass 1 patch — subtask explicite pour la 2e fonction du repo) :
  ```rust
  /// Charge les transactions bancaires `pending` (et NON
  /// `auto_match_rejected_at != NULL`) pour un compte donné, scopées
  /// multi-tenant. Tri par `booking_date DESC` puis `id DESC` pour
  /// présenter les transactions récentes d'abord.
  pub async fn find_pending_transactions_for_account<'e, E>(
      executor: E,
      company_id: i64,
      bank_account_id: i64,
      limit: i64,
  ) -> Result<Vec<BankTransaction>, DbError>
  where
      E: sqlx::Executor<'e, Database = MySql>,
  {
      sqlx::query_as::<_, BankTransaction>(
          "SELECT ... FROM bank_transactions \
           WHERE company_id = ? \
             AND bank_account_id = ? \
             AND status = 'pending' \
             AND auto_match_rejected_at IS NULL \
           ORDER BY booking_date DESC, id DESC \
           LIMIT ?"
      )
      .bind(company_id).bind(bank_account_id).bind(limit)
      .fetch_all(executor).await.map_err(map_db_error)
  }
  ```
  **Pagination v0.1** : `limit: i64` paramètre obligatoire (default 100 côté handler GET proposals — cf. **L24**), évite réponses unbounded.
- [ ] T3.1ter — Bonus helper batch-load Contacts (M5 Pass 1 patch) :
  ```rust
  pub async fn find_contacts_by_ids<'e, E>(executor: E, company_id: i64, ids: &[i64]) -> Result<HashMap<i64, Contact>, DbError>
  where E: sqlx::Executor<'e, Database = MySql>
  ```
  Pour batch-load des contacts distincts d'un set de candidate invoices.
- [ ] T3.2 — Tests d'intégration `#[sqlx::test]` (≥ 5) — créer `crates/kesh-db/tests/reconciliation_repository.rs` ou inline :
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
  - Sub-router `comptable_routes` (cf. `crates/kesh-api/src/lib.rs:90` pattern 8-1b — L4 Pass 1 patch : ligne 299 était une route `bank_profiles`, pas la définition du sub-router).
- [ ] T5.2 — Handler `GET /proposals` :
  1. Parse `bankAccountId` de la query string (validation `i64 > 0`).
  2. `find_pending_transactions_for_account(...)`.
  3. Pour chaque tx : `find_unpaid_invoices_for_window` (1 query par tx) + **batch-load des Contacts** distinct des candidates (M5 Pass 1 patch — éviter N×M queries) via `find_contacts_by_ids(tx, &distinct_contact_ids)` (1 query par batch de candidates) + `propose_matches(tx, &candidates_with_contacts)`. Pour 100 transactions × 20 candidates × 15 contacts distincts, le coût total est `100 × (1 invoice query + 1 batch contact query) = 200 queries`, pas `100 × 20 = 2000` queries.
  4. Retour shape : `{ proposals: [{ bankTransactionId, transaction: {...summary}, candidates: [{ invoiceId, score: {...}, invoice: {...summary} }] }] }`.
- [ ] T5.3 — Handler `POST /accept` (cf. §accept-flow pour le détail des steps 0-10) :
  1. Parse body `{ bankAccountId, proposals: [{ bankTransactionId, invoiceId }] }` (H2 Pass 1 patch).
  2. Validation step 0 (proposals non-vides, bankAccountId > 0, IDs distincts).
  3. Ouvrir UNE tx outer, acquérir `with_account_lock` UNE fois pour tout le batch (H5 Pass 1 patch — single lock pour toute la durée).
  4. Boucle proposal-par-proposal avec savepoints (cf. §accept-flow) — partial success via `SAVEPOINT/RELEASE/ROLLBACK TO SAVEPOINT`.
  5. UPDATE inline `bank_transactions` + `invoices.paid_at` (H1 Pass 1 patch — pas d'appel `mark_as_paid`, voir §accept-flow step 8).
  6. Audit log par proposal acceptée + `RELEASE_LOCK` explicite (M9 Pass 1 patch) + `tx.commit()`.
  7. Retourner `{ accepted, failed }`.
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
- [ ] T9.2 — README.md `## Feuille de route` : Epic 8 reste 🚧 En cours (**4/5 après merge 8-4** — L8 Pass 1 patch corrigé : 8-4 est la 4e story et 8-5 reste backlog donc 4/5, pas 5/5). Si décision retro Epic 8 considère que 4/5 stories suffisent comme première vague (partie A import + partie B matching auto), alors « ✅ Done » ; sinon 8-5 (réconciliation manuelle + règles) à enchaîner.
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

- **Pas de dépendance circulaire** : `kesh-reconciliation → kesh-core, kesh-db` (cf. `architecture.md:270` — L3 Pass 1 patch : ligne 269 décrit l'inverse `kesh-api → kesh-reconciliation`). Vérifier `cargo metadata` après ajout des deps.
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
| L24 | Pagination GET proposals via `limit` query param hardcodé 100 (L11 Pass 1 patch) | Pas de cursor-based pagination, pas d'`offset`. Pour un compte avec > 100 transactions pending, le user voit les 100 plus récentes (tri `booking_date DESC, id DESC`). Acceptable v0.1 — un compte avec > 100 pending est anormal en pratique (l'utilisateur a probablement laissé son backlog s'accumuler, à traiter via 8-5 manual). Pagination complète reportée v0.2 si retours utilisateurs. |
| L25 | RBAC test v0.1 couvre seulement le rôle `Consultation` (403) — pas l'unauth (401) (L12 Pass 1 patch) | Le 401 unauth est testé dans le suite générique de routes Story 1-5 (auth middleware). Re-tester pour `/reconciliation/*` est marginal mais pourrait être ajouté en Pass 2 review si Auditor flag. |
| L26 | `find_unpaid_invoices_for_window` peut remonter une candidate avec `score=0.0` post-filter du helper (M3 boundary), aboutissant à `candidates: []` côté frontend ; impossible de distinguer « 0 candidate dans repo window » vs « N candidates mais tous score 0 » (L13 Pass 1 patch) | Diagnostic gap : le user voit le même état neutre dans les 2 cas. Acceptable v0.1 — le cas « tous score 0 » est rare (filtres repo amount/date sont stricts donc une candidate qui passe le filter mais score 0 implique mismatch sur ref ET contact, peu probable). Si KF émerge en prod, ajouter un champ `candidatesFiltered: number` dans la réponse pour signaler que des candidates existaient mais ont été filtrées par score. |
| L27 | Epic-8 AC « écriture comptable créée OU liée » partiellement adressé (L14 Pass 1 patch) | 8-4 ne livre que la partie « liée » (`matched_entry_id ← invoice.journal_entry_id`). La partie « créée » (création d'écriture Banque/Client) est explicitement déférée 8-5 (création manuelle FR45) ou v0.2 (auto via règles d'affectation FR47). Pas un drift d'AC mais un découpage scope explicite de l'Epic 8 sur 2 stories successives. |
| L28 | `contact_score` bidirectionnel — false positives possibles entre formes juridiques courtes (A2-2 Pass 2) | Ex. `tx.counterparty_name = "ACME SA"` ⊂ `contact.name = "ACME GMBH SA"` → `contact_score = 1.0` alors que ce sont peut-être deux entités distinctes. Trade-off conscient H4 Pass 1 : on préfère false positives 0.10 plutôt que false negatives sur les noms longs CAMT (« ACME GMBH BERLIN » vs `contact.name = "ACME"` court). Mitigation : le contact ne pèse que 10% du score final, donc seul + sans amount/reference match → score < 70% → badge rouge → user vigilant. |
| L29 | Optimistic lock `version=?` UPDATE invoices au step 8 = defense-in-depth, pas race-prevention dans la même tx (A2-4 Pass 2) | Le `with_account_lock` + le SELECT/UPDATE dans la même `tx_outer` rend impossible une race entre step 5 (load) et step 8 (UPDATE). Le check `version=?` est conservé pour homogénéité avec Story 5-4 invoices repo (qui gère un cas non-tx) et pour défense-en-profondeur si un futur refactor sortait l'UPDATE de la tx. À ne PAS retirer comme « code mort » lors d'un cleanup. |
| L30 | Score re-calculé serveur (M7 Pass 1) peut différer du score affiché GET proposals si données ont changé entre proposal et accept (A2-5 Pass 2) | Drift typique : invoice.contact_id changé, invoice.invoice_number édité, etc. Le score audit reflète l'état au moment du commit, pas au moment du GET. Acceptable v0.1 : audit fiable > UX cohérente. Si KF émerge sur dérive systématique, ajouter un champ `score_at_proposal_time` côté frontend pour comparison. |
| L31 | `propose_matches` accepte `Option<Contact>` mais §accept-flow charge le contact côté handler — si contact archivé/supprimé après proposal, `contact_score = 0.0` au re-calc (A2-6 Pass 2) | Le caller (kesh-api) charge contact à chaque appel ; un soft-delete entre GET proposals et POST accept change le score. Acceptable v0.1 : rare en pratique (un contact référencé par invoice non payée n'est typiquement pas supprimé). |
| L32 | Boundary Decimal precision : tolérance amount filter ± 0.05 stocke en DB des values arrondies différemment (A2-8 Pass 2) | Si `tx.amount = 100.005` (rounded import) vs `invoice.total_amount = 99.96` (rounded entry), la fenêtre repo BETWEEN [99.955, 100.055] inclut 99.96 (mismatch 0.045). Score amount = 0.0 (non-exact), donc score final ≤ 0.50 — mismatch visible UI. Acceptable v0.1. |
| L33 | GET proposals response n'expose pas `journal_entry_id` mais audit log POST accept l'inclut (A2-10 Pass 2) | Le frontend ne sait pas, en GET, vers quelle écriture la proposition pointera (cette information serait redondante avec invoice → journal_entry_id qui est implicit). Si l'utilisateur veut naviguer vers l'écriture après accept, il passe par `GET /invoices/{id}` (route Story 5-2). UX defer v0.2 (ajouter `invoice.journalEntryId` au candidate shape). |
| L34 | Pas de `rejection_reason` field v0.1 sur `auto_match_rejected_at` (A2-11 Pass 2) | Quand 8-5 manual prendra le relais, le user n'aura pas le contexte « pourquoi le user précédent a rejeté ». Defer 8-5 : ajouter un champ `rejection_reason ENUM('wrong_invoice', 'pending_payment', 'wrong_account', 'other') NULL` ou un audit log enrichi. v0.1 : le user contrôle son propre flow donc rare en pratique. |

### Références

- Spec d'origine 8-3 (story précédente) : [`8-3-detection-doublons-rejet-partiel.md`](8-3-detection-doublons-rejet-partiel.md)
- Spec 8-1b (foundation `bank_transactions.status` + `matched_entry_id`) : [`8-1b-camt053-persistence-ui.md`](8-1b-camt053-persistence-ui.md)
- Epic 8 plan : [`epic-8.md`](../planning-artifacts/epic-8.md) — §Risques R5 (score de confiance)
- PRD : [`prd.md`](../planning-artifacts/prd.md) — FR44 (§438), §134 scenario Sophie réconciliation
- Architecture : [`architecture.md`](../planning-artifacts/architecture.md) — section « Dépendances inter-crates » (lignes 266-274, en particulier ligne 270 pour `kesh-reconciliation → kesh-core, kesh-db`), section « Mapping Exigences → Structure » (lignes 628-643, pour la carte FR → modules), lignes 491-498 pour la structure crate. L2 Pass 1 patch : les références §11.5 et §17 du spec original n'existaient pas dans architecture.md (pas de numérotation de sections).
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
| 2026-05-06 | **Pass 1 spec validate (Sonnet 4.6, 3 sub-agents parallèles)** — Cycle CLAUDE.md auteur=Opus → Pass 1=Sonnet pour briser biais d'auteur. Sub-agents fenêtres fraîches : Coherence Auditor (14 findings C1-C14, internal consistency + AC↔task matrix), Source Fidelity Auditor (8 findings S1-S8 + 16 claims vérifiées clean contre PRD/epic-8/architecture/code base), Adversarial Reviewer (16 findings A1-A16 cyniques via skill `bmad-review-adversarial-general`). Verdict implicit : **CONDITIONAL GO** (6 HIGH bloquants). Triage 38 bruts → **0 CRITICAL + 6 HIGH + 9 MEDIUM + 14 LOW + 2 REJECT** = 29 actionables. **29 patches appliqués (Option `A` — toutes catégories)** : **HIGH** — H1 (S4+S5+S8+A3+C2 cluster) refonte §accept-flow step 6 v0.1 « link only » : pas d'appel `mark_as_paid` (incompatible nested tx + ne crée pas d'écriture comptable), UPDATE inline `bank_transactions.matched_entry_id ← invoice.journal_entry_id, status='reconciled', version++` + `invoices.paid_at = bank_transaction.value_date.unwrap_or(booking_date), version++` (timestamp source défini, optimistic lock), création écriture « Banque/Client » explicitement déférée 8-5/v0.2 (cf. L20 + nouveau L27). H2 (A6+C10) ajout `bankAccountId` au body POST accept ET POST reject (lock acquis AVANT chargement transactions, sinon impossible). H3 (C11) shape réponse GET proposals alignée sur schema canonique (objet `score: { total, amountScore, referenceScore, contactScore }` + `transaction: {...}` + `invoice: {invoiceNumber, invoiceAmount, invoiceDate}` — pas de booléens flags `amountMatch/...`). H4 (A2) `contact_score` bidirectionnel (substring match dans les 2 directions, plus tolérant pour le user, résout incohérence formule vs AC #34 test). H5 (A1) UN seul `with_account_lock` pour TOUT le batch (pas par-proposal — sinon gap entre proposals N et N+1) + savepoints MariaDB pour partial success sans rollback du lot. H6 (C6) signature `propose_matches(tx, &[(Invoice, Option<Contact>)])` alignée §Scope item 1 vs T2.2. **MEDIUM** — M1 (A4) `find_unpaid_invoices_for_window` ajoute filtre `journal_entry_id IS NOT NULL` pour éviter de remonter des candidates rejetées au step 6. M2 (A8) POST reject acquiert aussi le mutex (symétrie avec accept, évite race vs concurrent accept). M3 (A9+C9) pseudo-code `reference_score` formel (containment bidirectionnel = 1.0, common_prefix ≥ 4 chars = 0.5, sinon 0.0) + pseudo-code `contact_score` formel. M4 (C4) audit log shape aligné §accept-flow step 9 vs §audit-log-actions table (`score: { total, amountScore, ... }` objet, `journal_entry_id` toujours présent). M5 (A5) batch-load Contacts via `find_contacts_by_ids` au lieu de N×M queries (200 queries au lieu de 2000 pour 100 tx × 20 candidates × 15 contacts). M6 (A10) arrondi explicite IEEE 754 `(score.total * 100.0).round() as u8` côté backend ET frontend pour éviter non-déterminisme à la frontière 90%. M7 (A13) score re-calculé côté serveur dans accept_one (pas pris du body) pour audit trail fiable. M8 (A11) migration `ALTER TABLE bank_transactions ... ALGORITHM=INSTANT, LOCK=NONE` (MariaDB 10.3+ instant ADD COLUMN nullable, évite outage prod sur grandes tables). M9 (A7) `RELEASE_LOCK` explicite avant `tx.commit()` (pas seulement `.ok()` après le résultat — explicite pour libérer la connexion plus tôt). **LOW** — L1 (S1) PRD §134 Sophie → §112 Marc (correction citation). L2 (S2) architecture.md §11.5/§17 fictifs → références par numéros de lignes (266-274, 628-643). L3 (S3) architecture.md:269 → 270 (kesh-reconciliation deps). L4 (S7) lib.rs:299 → 90 (comptable_routes définition, pas une route bank_profiles). L5 (C3) §reject-flow step 1 strikethrough SQL résiduel supprimé via refonte §reject-flow complète. L6 (C8) §reject-flow step 3 nom de fonction corrigé (`find_pending_transactions_for_account` qui filtre, pas `find_unpaid_invoices_for_window`). L7 (C5) AC #46 reformulé pour pointer explicitement vers AC #60 (résout redondance/contradiction). L8 (C12) T9.2 « 5/5 » → « 4/5 » (8-5 reste backlog). L9 (C14) `kesh-core` retiré du splitting risk count (pas d'extension `BankTransactionDraft` requise) → 5 modules au lieu de 6, sous-seuil. L10 (C7) T3.1bis subtask explicite pour `find_pending_transactions_for_account` + T3.1ter `find_contacts_by_ids` batch helper. L11 (A12) limitation L24 pagination GET proposals via `limit` query param hardcodé 100. L12 (A14) limitation L25 RBAC test couvre seulement Consultation (401 unauth couvert ailleurs). L13 (A15) limitation L26 score=0 diagnostic gap acceptable v0.1. L14 (S6) limitation L27 Epic-8 AC « créée OU liée » partiellement adressé (8-4 = liée, 8-5 = créée). **REJECT** — C13 `AsyncFnOnce` nightly : Source Fidelity confirme rust-toolchain.toml 1.85 stable. A16 status enum 'reconciled' check : confirmé via Source Fidelity (`BankTransactionStatus { Pending, Reconciled }` exporté kesh-db). **Trend findings > LOW : 15 → 0 post-patches Pass 1**. **Critère d'arrêt CLAUDE.md** : 0 finding > LOW post-patches. Spec post-Pass-1 : ~870 lignes (vs ~600 avant), 34 ACs inchangés en numérotation (#30-#63), 9 tasks T1-T9 (T3.1bis + T3.1ter ajoutés en sous-items), 5 modules (L9 retire kesh-core), 11 limitations connues (L17-L27). Sections enrichies : §matching-algo (pseudo-code formel reference_score + contact_score + arrondi IEEE 754), §accept-flow (refonte H1+H2+H5 avec single lock + savepoints + UPDATE inline), §reject-flow (refonte avec mutex partagé + body bankAccountId), §candidate-window (filtre journal_entry_id IS NOT NULL), §audit-log-actions (shape score objet), §error-precedence-order (8 niveaux maintenus). **Pass 2 Haiku 4.5 recommandée** (cycle Sonnet → Haiku, fenêtre fraîche, focus sur les sections les plus modifiées : §accept-flow, §reject-flow, §matching-algo) pour confirmation orthogonale avant `bmad-dev-story 8-4`. | Claude (Opus 4.7 1M coordinator + Sonnet 4.6 sub-agents, bmad-create-story validate Pass 1) |
| 2026-05-06 | **Pass 2 spec validate (Haiku 4.5, 3 sub-agents parallèles)** — Cycle CLAUDE.md Sonnet(P1) → Haiku(P2). Sub-agents fenêtres fraîches : Coherence Auditor (6 findings C2-1..C2-6, 2 MEDIUM + 4 LOW), Source Fidelity Auditor (**0 findings, verdict GO** — tous les 29 patches Pass 1 vérifiés clean contre PRD/epic-8/architecture/code base), Adversarial Reviewer (12 findings A2-1..A2-12). Triage 18 bruts → **0 CRITICAL + 0 HIGH + 4 MEDIUM + 12 LOW + 2 REJECT** = 16 actionables. Trend Pass 1 (15>LOW) → Pass 2 (4 MEDIUM) = **-73%**. **16 patches appliqués (Option `A` — toutes catégories)** : **MEDIUM** — C2-4 (régression Pass 1 M1) ajout `AND journal_entry_id IS NOT NULL` au pseudo-SQL T3.1 (la prose §candidate-window le documentait mais le bloc SQL ne l'avait pas, dev qui copie le pseudo-code aurait raté le filtre). C2-1 spec'd la response shape POST reject `{ rejected: [{bankTransactionId, rejectedAt}], failed: [{bankTransactionId, errorCode}] }` symétrique POST accept. A2-1 pseudo-code reference_score `bytes()` → `chars()` pour boundary UTF-8 Unicode-aware (noms suisses Müller/École/René/Sàrl). A2-9 clarifié `batch_size = total proposals submitted in body` (pas accepted_count, pas failed_count) — permet calcul taux acceptance a posteriori. **LOW** — C2-2 typo AC #59 `find_pending_for_account` → `find_pending_transactions_for_account` (canonique aligné T3.1bis). C2-3 AC #53 audit details schema explicite `{ bank_transaction_ids: [42, 43], count: 2 }`. C2-5 commentaire savepoint collision impossible par validation step 0 (« tous bankTransactionId distincts »). C2-6 commentaire numbering §accept-flow steps 2-10 volontairement continu avec préambule 0-1 (pas de step 1 dans le helper accept_one). A2-2 limitation L28 contact_score bidirectionnel false positives (ACME SA ⊂ ACME GMBH SA) — trade-off H4 conscient, mitigation = contact pèse 10% donc score isolé < 70% → badge rouge. A2-4 limitation L29 optimistic lock UPDATE invoices = defense-in-depth (pas race-prevention dans même tx) — homogénéité Story 5-4 + futur refactor sortie tx. A2-5 limitation L30 score re-calculé serveur ≠ score affiché GET si données changent — audit fiable > UX cohérente. A2-6 limitation L31 contact archivé/supprimé entre GET et POST → score=0 au re-calc, acceptable rare. A2-8 limitation L32 Decimal precision boundary mismatch dans fenêtre repo, score binaire visible UI. A2-10 limitation L33 GET proposals sans `journal_entry_id` exposé (defer v0.2). A2-11 limitation L34 pas de `rejection_reason` field v0.1 (defer 8-5 enrichment). **REJECT** — A2-7 DoS via mutex naming (rate-limiting hors scope 8-4, pattern projet existant). A2-12 30-day window hardcoded (déjà documenté L17/L24/L26 v0.2). A2-3 savepoint collision (couvert par C2-5). **Trend findings > LOW : 4 → 0 post-patches Pass 2**. **Critère d'arrêt CLAUDE.md atteint** (0 finding > LOW). Spec post-Pass-2 : ~895 lignes (vs ~870 Pass 1, +2.9%), 34 ACs inchangés (#30-#63), 9 tasks T1-T9, 5 modules, **18 limitations connues (L17-L34)**. Sections enrichies : §matching-algo (`.chars()` UTF-8 + commentaire), §reject-flow (response shape JSON canonique), §accept-flow (commentaire savepoint + numbering), §candidate-window pseudo-SQL (filtre journal_entry_id ajouté au bloc SQL, pas seulement à la prose), §audit-log-actions (sémantique batch_size). **Pass 3 Opus 4.7 recommandée** (cycle Sonnet → Haiku → Opus, fenêtre fraîche, focus sur les sections les plus modifiées Pass 1+2 cumulé : §accept-flow refonte H1+H5+savepoints, §matching-algo pseudo-code) pour clore le cycle modèle CLAUDE.md avant `bmad-dev-story 8-4`. **Acceptance Auditor Source Fidelity Haiku verdict GO** confirmé pour la 1ère fois post-patches. | Claude (Opus 4.7 1M coordinator + Haiku 4.5 sub-agents, bmad-create-story validate Pass 2) |
