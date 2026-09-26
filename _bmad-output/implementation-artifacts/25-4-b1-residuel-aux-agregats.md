# Story 25.4-b1 : Le résiduel aux agrégats — balance âgée et échéancier

Status: ready-for-dev

**Issue : [#416]** — cette story en livre la partie **agrégats** ; la partie **rappels** est la
25-4-b2. ⛔ **La PR de b1 porte `refs #416`, pas `closes`** : l'issue ne se ferme qu'avec b2.

**Mère : `25-4-propager-le-residuel.md`** (`split`). **Sœur livrée : 25-4-a** (PR #472, #455 et #456) —
elle a rendu la formule juste (avoir compté TTC) et posé le test de parité des formes jointes ;
cette story en est le **premier appelant**. ⚠️ Branche **empilée** sur celle de 25-4-a : à rebaser
sur `main` après le merge de #472.

## Story

En tant que comptable,
je veux que la balance âgée et l'échéancier montrent ce que chaque client **doit encore**, et non
ce que la facture valait à l'émission,
afin de relancer les bons clients pour les bons montants, et que la balance âgée concorde avec le
compte débiteurs du grand livre.

## Le défaut, vérifié dans le code

Depuis la 24-2, une facture se règle en plusieurs fois ; le reste dû se **calcule** (24-2, D3). Les
écrans **unitaires** le savent (fiche facture : badge « partiellement payée », « Reste dû »). Les
écrans **agrégés**, non — ils totalisent le **TTC** :

| Site | Ce qu'il somme |
|---|---|
| Balance âgée — `crates/kesh-report/src/aged_receivables.rs:111-131` (`generate`) | `COALESCE(lt.ttc, 0)` dans les cinq tranches et le total |
| Totaux de l'échéancier — `crates/kesh-db/src/repositories/invoices.rs:882-900` (`due_dates_summary`) | `unpaid_total`, `overdue_total` en `lt.ttc` ; le commentaire `:890-892` les dit « montants dus » — faux depuis la 24-2 |
| Lignes de l'échéancier — `list_by_company_paginated` (`:815-821`) | `total_ttc` seul ; **aucun** reste dû par ligne |
| Export CSV de l'échéancier — `list_for_export` (`:2456-2461`), `routes/invoices.rs:1463-1465` | colonne « Total » = TTC, commentée « montant dû » ; statut sans « partiel » (`:1443-1449`) |
| Page échéancier — `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` | colonne Total = `totalTtc` (`:453`), `statusOf` sans `partial` (`:306-314`), dialogue `amountDue={null}` (`:493-507`) |

Une facture de 1 000.— réglée à 900.— pèse **1 000.—** partout, au lieu de 100.—. ⚠️ La balance âgée
est un **instrument de décision** (qui relancer, pour combien) et doit **concorder avec le compte
débiteurs** du grand livre, juste depuis la 24-2 : les deux divergent aujourd'hui exactement du
montant des règlements partiels, et **aucun test ne compare** les deux (vérifié : aucun test de
`kesh-report` ne lit un solde de compte).

⚠️ **Les formes jointes du résiduel n'ont aucun appelant** (`INVOICE_SETTLED_DERIVED_JOIN_SQL`,
`INVOICE_CREDITED_DERIVED_JOIN_SQL`) ; 25-4-a en a prouvé la parité avec les formes scalaires. La
24-2 (D3) a prescrit la **jointure dérivée** pour les listes et agrégats : la forme corrélée y
serait réévaluée par ligne — un N+1 déguisé.

## Acceptance Criteria

### Volet 1 — la grandeur, une seule fois

**AC 1** — Une constante (ou une paire) canonique porte le **reste dû sous forme jointe** : les trois
tables dérivées (`lt` TTC, `cnt` avoir, `st` réglé) et l'expression
`COALESCE(lt.ttc, 0) − COALESCE(cnt.credited, 0) − COALESCE(st.settled, 0)`. Elle vit à côté
d'`amount_due` (`invoice_settlements.rs`), avec un doc-comment qui la dit **miroir** de la forme
scalaire. ⛔ Aucun site de cette story ne réécrit l'expression à la main.

**AC 2** — Le **test de parité** de 25-4-a (`tests/invoice_amount_due_parity.rs`) est étendu : pour
chaque facture du jeu, l'expression jointe de l'AC 1 = `amount_due` scalaire.

### Volet 2 — la balance âgée

**AC 3** — Les cinq tranches et le total de `aged_receivables::generate` somment le **reste dû**
(AC 1), non le TTC. Les tranches restent assises sur `i.due_date` : ⛔ **un règlement partiel ne
rajeunit pas la créance restante**.

**AC 4** — Le périmètre des factures ne change pas : `status = 'validated' AND paid_at IS NULL`
(une facture soldée a `paid_at`, 24-2 D4). Le `HAVING total <> 0` reste, et porte désormais sur le
reste dû. ⚠️ Un reste dû **négatif** (trop-perçu hérité) n'est **pas écrêté** : il apparaît, en
négatif — le doc-comment d'`amount_due` l'exige.

**AC 5** — ⛔ **Invariant de concordance** (test) : sur une société dont le compte débiteurs n'est mû
que par des factures, des règlements et des avoirs — factures sans règlement, partiellement réglée,
soldée, créditée —, le **total de la balance âgée = solde du compte 1100 au grand livre**
(`general_ledger` ou somme `debit − credit`). C'est l'assertion que #416 réclame et qui n'existe
pour aucun écran.

**AC 6** — Doc-comment de `generate` (`:95-99` : « Montants = TTC dérivé ») et libellés : la balance
âgée montre le **reste dû** ; l'écran (`AgedReceivablesView.svelte`) et son CSV gardent leur
structure — seules les valeurs changent. Si un libellé visible dit « TTC » ou « total facturé »,
il devient « reste dû » (4 locales).

### Volet 3 — l'échéancier

**AC 7** — `due_dates_summary` : `unpaid_total` et `overdue_total` somment le **reste dû** ; le
commentaire `:890-892` dit vrai.

**AC 8** — `InvoiceListItem` porte `amount_settled` et `amount_due` (`Decimal`), projetés par **les
deux SELECT** qui le désérialisent — `list_by_company_paginated` **et** `list_for_export` (⚠️ le
doc-comment `:263-269` le rappelle : un oubli échoue au **runtime**). Calculés par la **forme
jointe** (AC 1) ; `total_ttc` reste. ⛔ Pas de N+1 : le nombre de requêtes d'une page ne dépend pas
du nombre de lignes (invariant 7 de la 24-2).

**AC 9** — `InvoiceListItemResponse` (`routes/invoices.rs:333-364`) expose `amountSettled` et
`amountDue` (camelCase, chaîne décimale). ⚠️ Même discipline que la fiche
(`routes/invoices.rs:242-248`) : **toujours calculés** dans une liste, jamais `None` par défaut. Le
type frontend (`invoices.types.ts`) suit.

**AC 10** — Page échéancier :
- une colonne **« Reste dû »** (`amountDue`) à côté du Total TTC, qui reste ;
- le statut de ligne rend **`partial`** quand `paidAt` est nul et `amountSettled > 0` — même règle
  que la fiche (`invoices/[id]/+page.svelte:424-438`), idéalement **la même fonction** partagée ;
- le résumé montre les totaux de l'AC 7 (les libellés disent « reste dû » s'ils disaient autre
  chose) ;
- le dialogue de règlement reçoit **`amountDue={inv.amountDue}`** : pré-rempli, et sa garde client
  de trop-perçu (`SettleInvoiceDialog.svelte:108-112`) redevient active depuis l'échéancier. Le
  commentaire `:493-499` qui explique pourquoi il ne l'était pas est retiré.

**AC 11** — Export CSV de l'échéancier : une colonne **« Reste dû »** après « Total » (clé d'en-tête
×4, `CSV_HEADER_KEYS` / `CSV_HEADER_FALLBACKS`) ; le statut rend « partiellement payée »
(`payment-status-partial`, clé existante) ; le commentaire `:1463-1464` « TTC (montant dû) » est
corrigé.

### Volet 4 — tests, textes

**AC 12** — Tests, au minimum :

| Test | Prouve |
|---|---|
| parité étendue (AC 2) | forme jointe du reste dû = `amount_due` |
| `aged_uses_amount_due_not_ttc` | facture 8,1 % de 1 081.— réglée 900.— : pèse 181.— dans sa tranche |
| `aged_partial_payment_keeps_original_bucket` | la tranche suit `due_date`, pas la date du règlement |
| `aged_total_matches_receivable_ledger` | AC 5 |
| `due_dates_summary_totals_are_amount_due` | AC 7 |
| `list_items_carry_amount_due` (repo **et** e2e HTTP) | AC 8-9, les deux SELECT ; la valeur traverse la frontière |
| export CSV : colonne reste dû, statut partiel | AC 11 |
| Vitest échéancier | colonne, statut `partial`, dialogue pré-rempli |
| **E2E** `invoices_echeancier.spec.ts` | le cas de la 24-3 (`:147-158`) cesse de **saisir** le montant : il vérifie qu'il est **pré-rempli** au reste dû, sur une facture partiellement réglée |

⛔ **Chaque facture de test porte une TVA non nulle** (leçon de 25-4-a). ⛔ Mutations à exécuter et
consigner : TTC remis dans une tranche de la balance âgée ; `amount_due` retiré de `list_for_export`
seul ; `partial` retiré de `statusOf` ; `amountDue={null}` remis.

**AC 13** — Textes (`docs/manual/fr/user-manual.tex`, PDF **contrôlé aplati**) : § Balance âgée
(`\label{sec:balance-agee}`, « encours débiteur (le total dû, TVA comprise) ») dit **reste dû après
règlements partiels et avoirs** ; § Échéancier (`:927`) décrit la colonne et le pré-remplissage.
CHANGELOG `[0.12.1]` *Fixed*. README : rien, sauf si la ligne v0.12.1 cite la balance âgée.

## Tasks / Subtasks

- [ ] **T1 — la grandeur jointe** (AC 1-2).
- [ ] **T2 — balance âgée** (AC 3-6), dont l'invariant de concordance.
- [ ] **T3 — échéancier, backend** (AC 7-9, AC 11).
- [ ] **T4 — échéancier, frontend** (AC 10), i18n.
- [ ] **T5 — tests et mutations** (AC 12).
- [ ] **T6 — textes** (AC 13).
- [ ] **T7 — gates** : backend complet (base remise à zéro), frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Ne pas stocker le reste dû** (24-2, D3).
- ⛔ **Ne pas utiliser `amount_due` scalaire dans une boucle** sur les lignes d'une liste : N+1.
- ⛔ **Ne pas retirer `total_ttc`** de la liste : l'échéancier affiche les deux, et d'autres écrans le
  lisent.
- ⚠️ **Le tri** « TotalAmount » de l'échéancier trie sur `i.total_amount` (HT) alors que la colonne
  affiche le TTC — défaut **préexistant**, hors périmètre ; ne pas l'aggraver, le signaler.

### Où regarder

| Fichier | Rôle |
|---|---|
| `crates/kesh-db/src/repositories/invoice_settlements.rs` | constantes jointes, `amount_due` |
| `crates/kesh-report/src/aged_receivables.rs:95-175` | requête, doc |
| `crates/kesh-report/tests/aged_receivables.rs` | tests existants (leur facture « payée » est montée par `UPDATE paid_at`) |
| `crates/kesh-db/src/repositories/invoices.rs:246-275, 800-830, 882-930, 2436-2470` | `InvoiceListItem`, liste, résumé, export |
| `crates/kesh-api/src/routes/invoices.rs:333-364, 1364-1470` | DTO de liste, export CSV |
| `frontend/src/routes/(app)/invoices/due-dates/+page.svelte` | page |
| `frontend/src/routes/(app)/invoices/[id]/+page.svelte:424-438` | `paymentStatus()` de la fiche, à partager |
| `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:35-112` | pré-remplissage, garde client |
| `frontend/tests/e2e/invoices_echeancier.spec.ts:100-160, 200-201` | cas de la 24-3 ; « la colonne Total affiche le TTC (montant dû) » |
| `crates/kesh-report/src/general_ledger.rs` | solde d'un compte, pour l'AC 5 |

### Gardes-fous du dépôt

- Aucune migration.
- Repositories touchés : **gate complet même en cours de boucle** (§ Exception `kesh-db`).
- i18n : les nouvelles clés et tout `i18nMsg(` ajouté bougent `sitesTotal` — recompter.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-27** — Créée (Opus 5.5) après découpage de la 25-4-b (arbitrages Q1 ⇒ six modules).
  Sites revérifiés dans le code ; 25-4-a en est le socle.

[#416]: https://github.com/guycorbaz/kesh/issues/416
