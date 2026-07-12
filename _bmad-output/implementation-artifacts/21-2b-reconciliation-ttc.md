# Story 21.2b: Réconciliation bancaire en TTC — matching, câblage, UI candidats

Status: ready-for-dev

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

- [ ] **T1 — Repo + wrapper** (AC 1-2) : alias FROM + forme scalaire + `UnpaidInvoiceCandidate` pub + flatten
- [ ] **T2 — Crate matching** (AC 3, 6) : tuple 3-champs + amount_score TTC + ~13 fixtures
- [ ] **T3 — Câblage production + UI candidats** (AC 4-5) : 2 sites + `invoiceAmount` TTC (`:523`)
- [ ] **T4 — Seeds + tests** (AC 7-8) : 2 helpers de seed (ligne vat 0) + tests match TVA + assertion invoiceAmount
- [ ] **T5 — Doc + gate** (AC 9-10)

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

### Debug Log References

### Completion Notes List

### File List

## Change Log
