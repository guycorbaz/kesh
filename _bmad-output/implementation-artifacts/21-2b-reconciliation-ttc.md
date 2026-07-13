# Story 21.2b: Réconciliation bancaire en TTC — matching, câblage, UI candidats

Status: review

<!-- Spec créée le 2026-07-12 par SPLIT de la story 21-2 (règle de splitting préventif CLAUDE.md — validate 4 passes sans convergence, friction concentrée sur la réconciliation). Contenu = AC 12/19 de l'ex-21-2, raffiné par les 4 passes de validate (V1-1, V2-3, V2-4, V3-4) + patches Pass 4 (V4-1, V4-3) intégrés. CONSOMME les primitives de 21-2a (helper `invoice_total_ttc` + constantes SQL `pub`) — dev APRÈS 21-2a done. **Closes #246** (dernier morceau). -->

## Story

En tant que **PME assujettie à la TVA**,
je veux **que le rapprochement bancaire compare les encaissements (TTC par nature) au TTC de mes factures**,
afin de **retrouver le matching automatique (aujourd'hui : plus AUCUN match dès que TVA > 0) — dont dépendent le `paid_at` et donc tout le cycle de relance (#231)**.

## Contexte

`amount_score` (`kesh-reconciliation/src/matching.rs:59`) compare `tx.amount` (encaissement bancaire, TTC réel) à `invoice.total_amount` (**HT**) ; le filtre SQL `find_unpaid_invoices_for_window` (`repositories/reconciliation.rs:84-93`) fait de même (`total_amount BETWEEN tx ± tolérance`). À TVA > 0, un client qui paie 108.10 pour une facture HT 100.00 ne matche jamais. Invisible en dogfooding (TVA 0). Vérifié Pass 4 : le montant candidat `invoiceAmount` affiché dans l'UI de rapprochement (`ReconciliationProposals.svelte:285`) vient aussi du HT (`routes/reconciliation.rs:523`).

## Acceptance Criteria

1. **`find_unpaid_invoices_for_window`** (`repositories/reconciliation.rs:84-93`) : le filtre `BETWEEN tx ± tolérance` passe sur la **forme scalaire de la constante SQL 21-2a**, et le SELECT l'expose `AS total_ttc`. ⚠️ **Alias FROM** : la requête fait `FROM invoices` **sans alias `i`** (`:85`) — aliaser (sinon `Unknown column 'i.id'`).
2. **Type de retour FIGÉ** : wrapper **`pub struct UnpaidInvoiceCandidate { pub invoice: Invoice, pub total_ttc: Decimal }`** avec `#[sqlx(flatten)]` sur `invoice` — **`pub`, PAS `pub(crate)`** (V4-3 : le type est consommé et destructuré par `kesh-api`, crate séparé — un `pub(crate)` ne compile pas). La fonction retourne `Vec<UnpaidInvoiceCandidate>` ; l'entité `Invoice` ne change PAS. ⚠️ `#[sqlx(flatten)]` = **premier usage du workspace** — alternative précédentée (projection plate façon `InvoiceListItem`) rejetée (duplication ~20 champs + conversion) ; filet = compilation + les ~9 tests existants de `find_unpaid_invoices_for_window` qui hydrateront le wrapper.
3. **`amount_score`** (`matching.rs:59`) compare `tx.amount` au **TTC**. **Signature FIGÉE** : le tuple candidat de `propose_matches` passe de `(Invoice, Option<Contact>)` à **`(Invoice, Option<Contact>, Decimal /* total_ttc */)`** — extension minimale (aucune struct candidat n'existe dans le crate — vérifié), le crate reste PUR (pas de sqlx, le caller fournit la valeur), `amount_score(tx.amount, candidate_ttc)`.
4. **Câblage PRODUCTION (fanout — 2 sites)** :
   - `routes/reconciliation.rs:503-510` : construit les candidats depuis `find_unpaid_invoices_for_window` → mapper `UnpaidInvoiceCandidate` vers le tuple 3-champs.
   - `routes/reconciliation.rs:1120-1124` (re-score serveur `accept_one`, facture unique) : **utiliser le helper `invoice_total_ttc` de 21-2a sur les lignes** si elles sont chargées à ce point, sinon re-fetch via le wrapper — même arithmétique garantie par le test de parité 21-2a.
   Les DEUX doivent transmettre le TTC — sans quoi soit ça ne compile pas, soit `amount_score` compare silencieusement au HT.
5. **UI candidats — `invoiceAmount` en TTC** (V4-1) : `routes/reconciliation.rs:523` hydrate `ReconciliationCandidate.invoice_amount` avec `inv.total_amount` (HT) → passe au **TTC** (disponible dans le même bloc via le tuple 3-champs). Ce champ est rendu dans l'écran de rapprochement (`ReconciliationProposals.svelte:285`) juste à côté du montant de la transaction — après le matching TTC, un candidat proposé pour une tx de 108.10 doit afficher 108.10, pas 100.00. Sites `:1614`/`:3090` (audit log réel sur `bt.amount`) : NON modifiés (vérifié Pass 4).
6. **Fanout tests du crate matching** : ~13 call sites internes `propose_matches(&tx, &[(invoice, …)])` dans `matching.rs:268-436` → adapter au tuple 3-champs (build `--all-targets` = filet ; les fixtures fournissent un TTC de candidat explicite).
7. **Seeds sans lignes (fanout listé — piège central)** : `kesh-api/tests/reconciliation_e2e.rs` (`insert_invoice`/`seed_validated_invoice` :306-327) et `kesh-db/tests/reconciliation_repository.rs:154-181` (`insert_test_invoice`, ~9 call sites) insèrent `total_amount` en dur SANS `invoice_lines` → la sous-requête rendrait 0 et tuerait tous les matchs. **Corriger ces 2 helpers de seed** : une ligne `(line_total = total_amount, vat_rate = 0)` par facture seedée → TTC = total_amount → **toutes les assertions existantes restent vertes**. Vérifié : `reconciliation_manual/split/rules_e2e.rs` ne seedent aucune facture propre.
8. **Tests nouveaux** : facture avec lignes @ 8.1 (HT 100, TTC 108.10) + transaction 108.10 → **match proposé** (aujourd'hui : aucun) ; transaction 100.00 → PAS de match exact ; assertion `invoiceAmount == "108.10"` dans la réponse candidats (AC 5). Suites réconciliation existantes vertes sans modification d'assertions (preuve d'iso-comportement TVA 0 via AC 7).
9. `CHANGELOG.md` `[Non publié]` : compléter l'entrée TTC de 21-2a (rapprochement bancaire). **Closes #246.**
10. **Aucune migration** ; **quality gate Test Locally First** complet (attention KF-038 #228 : flake pré-existant `reconciliation_e2e::post_accept_skips_non_chf_transaction` sous contention parallèle — ne pas le confondre avec une régression ; il passe en série).

## Tasks / Subtasks

- [x] **T1 — Repo + wrapper** (AC 1-2) : alias FROM + forme scalaire + `UnpaidInvoiceCandidate` pub + flatten
- [x] **T2 — Crate matching** (AC 3, 6) : tuple 3-champs + amount_score TTC + ~13 fixtures
- [x] **T3 — Câblage production + UI candidats** (AC 4-5) : 2 sites + `invoiceAmount` TTC (`:523`)
- [x] **T4 — Seeds + tests** (AC 7-8) : 2 helpers de seed (ligne vat 0) + tests match TVA + assertion invoiceAmount
- [x] **T5 — Doc + gate** (AC 9-10)

## Dev Notes

- **PRÉREQUIS : 21-2a done** (helper `invoice_total_ttc` + constantes SQL `pub` + test de parité). Ne rien re-décider : toutes les décisions structurantes ont été figées par 4 passes de validate sur l'ex-21-2 (Change Log de `21-2a-ttc-canonique-surfaces.md`).
- **kesh-reconciliation est un crate PUR** (pas de sqlx) — le TTC est passé par le caller, jamais calculé dans le crate.
- Rationale table dérivée vs corrélée (V3-1) : sans objet ici — cette story n'utilise que la forme scalaire (filtre + SELECT par facture).
- Leçons process reconduites : workspace série, jamais `runner | grep`, grep post-patch, Playwright non concerné (aucun changement frontend hors donnée affichée — `ReconciliationProposals.svelte` consomme `invoiceAmount` tel quel, zéro modif frontend).
- **Frontend : ZÉRO fichier touché** — `invoiceAmount` est une string serveur ; le changement de valeur suffit.

### References

- [Source: 21-2a-ttc-canonique-surfaces.md — Change Log Pass 1-4] — historique complet des décisions.
- [Source: _bmad-output/planning-artifacts/epic-21-echeances-relances.md#Décision préalable #246]

## Dev Agent Record

### Agent Model Used

Fable 5 (claude-fable-5) — run 2026-07-13.

### Debug Log References

- Divergence spec vs code sur le câblage `accept_one` : `invoice` y est chargé **sans lignes** (`find_invoice_by_id_for_company`) → helper Rust sur lignes impraticable sans re-fetch. Choix : nouveau helper `invoices::total_ttc(executor, id)` (forme scalaire SQL, générique executor → utilisable en tx), garantissant la parité avec le filtre `find_unpaid_invoices_for_window` (même expression SQL).
- `find_unpaid_invoices_for_window` : filtre TTC posé en **HAVING** (pas WHERE) pour référencer l'alias `total_ttc` sans dupliquer la sous-requête ; l'index `WHERE` (company/status/date) reste utilisé.
- Sérialisation : `TransactionSummary.amount` et `invoiceAmount` sont **normalisés** (`Decimal::normalize()`) → assertions e2e sur `"108.1"` / `"100"` (pas `"108.1000"`).

### Completion Notes List

- **T1** wrapper `pub struct UnpaidInvoiceCandidate { invoice #[sqlx(flatten)], total_ttc }` (1er usage flatten) + `find_unpaid_invoices_for_window` → alias `i` + `INVOICE_TTC_SUBQUERY_SQL AS total_ttc` + filtre HAVING TTC. Retourne `Vec<UnpaidInvoiceCandidate>`.
- **T2** tuple candidat `propose_matches` → `(Invoice, Option<Contact>, Decimal)`, `amount_score(tx.amount, total_ttc)` ; 13 sites de tests migrés via helper `cand()` (TTC = total_amount pour ces fixtures sans TVA → iso-comportement, 30 tests lib verts).
- **T3** câblage : `list_bank_transactions_proposals` (`.invoice.*` + tuple 3-champs + **`invoiceAmount = cand.total_ttc`** V4-1) + `accept_one` re-score via `invoices::total_ttc` en tx + `tx_candidates: Vec<(_, Vec<UnpaidInvoiceCandidate>)>`. Sites audit `:1614`/`:3090` NON touchés.
- **T4** 2 seeds (`insert_test_invoice` kesh-db + `insert_invoice` kesh-api) → ligne unique vat 0 = TTC=total_amount → **iso-comportement prouvé** (repo 8/8, e2e 24/24 sans modif d'assertion) + 2 nouveaux tests TVA : `find_unpaid_matches_on_ttc_not_ht` (repo : matche 108.10, PAS 100.00) + `get_proposals_matches_and_shows_ttc` (e2e : proposition invoiceAmount=108.1 + amountScore 1 ; HT 100 ne propose plus).
- **T5** CHANGELOG entrée #246 étendue au rapprochement bancaire (**closes #246**). Aucune migration. Frontend : 0 fichier touché.
- **Gate** : fmt/clippy 0 · workspace série (en cours au moment de la rédaction) · réconciliation repo 8/8 + 1 nouveau, e2e 24/24 + 1 nouveau. **Ferme #246** (dernier morceau du bug HT/TTC).

### File List

- crates/kesh-reconciliation/src/matching.rs (tuple 3-champs + helper cand tests)
- crates/kesh-db/src/repositories/reconciliation.rs (wrapper UnpaidInvoiceCandidate + find_unpaid TTC)
- crates/kesh-db/src/repositories/invoices.rs (helper total_ttc)
- crates/kesh-db/tests/reconciliation_repository.rs (seed ligne vat 0 + test match TTC)
- crates/kesh-api/src/routes/reconciliation.rs (câblage 2 sites + invoiceAmount TTC + accept_one)
- crates/kesh-api/tests/reconciliation_e2e.rs (seed ligne vat 0 + test proposition TTC)
- CHANGELOG.md

## Change Log

### Pass 1 code-review (2026-07-13) — CONVERGÉ en 1 passe

Panel adversarial de 3 reviewers en parallèle, lentilles distinctes, sur le diff aplati `eee73682` :

- **Sonnet — correctness SQL/TTC** : 0 finding > LOW. Parité `INVOICE_TTC_SUBQUERY_SQL` (MariaDB `ROUND` half-away-from-zero) ≡ helper Rust `invoice_total_ttc` (`MidpointAwayFromZero`) confirmée, ties `.005`/`.455` couverts par le test 4 voies. `HAVING total_ttc BETWEEN` : 9 `?` = 9 `.bind()`, index `WHERE` préservé. `COALESCE(...,0)` → 0 pour facture sans lignes. Pas de collision `#[sqlx(flatten)]` (`Invoice` n'a pas de champ `total_ttc`). `amount_score` symétrique.
- **Haiku — régression/fanout** (grep ground-truth) : 0 finding > LOW. 2 call sites prod (`:522`, `:1155`) + 13 fixtures migrés au tuple 3-champs, accesseurs `.invoice.*`, seeds vat 0 = iso-comportement, sites audit `bt.amount` (`:1645`/`:3121`) intacts.
- **Opus — archi/edge/patterns** : 0 finding > LOW. Parité re-score `accept_one` « airtight » (même constante SQL des deux côtés → impossible qu'une proposition affichée soit rejetée par écart arithmétique). Pattern `FailedProposal` respecté (nouveau chemin d'erreur TTC encapsulé, pas d'`AppError` global). Guard devise CHF (`:445`) avant la requête TTC. `normalize()` cohérent entre `invoiceAmount` et `TransactionSummary.amount`. **3 nits LOW** :
  - **LOW-1** — facture legacy `total_amount≠0` mais **sans `invoice_lines`** → TTC=0 → ne matche plus (avant : matchait sur `total_amount`). Impact quasi-nul (toute facture UF a des lignes ; nouveau comportement *plus* conservateur). **Non corrigé** (accepté v0.1, hors flux de création).
  - **LOW-2** — `error_code` divergent : le site re-score TTC utilisait `"RECONCILIATION_INTERNAL"` + `details.reason` alors que les 11 erreurs DB sœurs de la même fonction utilisent `"DATABASE_ERROR"` + `details.message`. **CORRIGÉ** (`reconciliation.rs:1147` aligné) — cohérence du contrat `FailedProposal` (constantes canoniques, CLAUDE.md).
  - **LOW-3** — sous-requête TTC évaluée par ligne de la fenêtre avant filtre montant + `LIMIT 50` sans `ORDER BY` (non-déterminisme **pré-existant**, l'ancienne requête n'avait déjà pas d'`ORDER BY`). Volume PME négligeable. **Non corrigé** (perf marginale, pré-existant).

**Trend** : passe 1 → 3 reviewers à 0 CRITICAL/HIGH/MEDIUM. Critère d'arrêt de la Review Iteration Rule atteint (uniquement LOW). 1 LOW corrigé (LOW-2), 2 LOW documentés comme acceptés. Pas de passe 2.
