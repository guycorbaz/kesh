# Story 25.4-c : Le résiduel au rapprochement bancaire

Status: ready-for-dev

**Issue : [#420]** — ⛔ la PR porte `closes #420`, titre ET corps (§ *Issue Tracking Rule*). Voisine :
**[#476]** (arrondi de la QR contre garde de trop-perçu), dont l'inclusion est une question à Guy (Q1).

**Mère : `25-4-propager-le-residuel.md`** (`split`) — source des faits. **Sœurs** : 25-4-a (mergée, #472 :
le reste dû juste), 25-4-b1 (mergée, #475 : formes jointes aux agrégats), 25-4-b2 (PR #479 : les
rappels). Indépendante de b2 : branche `story/25-4-c-residuel-au-rapprochement` tirée de `main`
(`b10109be`).

## Story

En tant que comptable,
je veux que le rapprochement bancaire reconnaisse le virement du **solde** d'une facture déjà
partiellement réglée,
afin de ne pas avoir à rapprocher à la main un paiement qui correspond exactement à ce que le client
devait encore.

## Le défaut, vérifié dans le code — pire que ce que dit l'issue

#420 annonce un **score nul**. C'est en amont que ça casse : la facture **n'est même pas candidate**.

| Site | Grandeur | Effet sur 1 000.— réglée de 400.—, virement de 600.— |
|---|---|---|
| Filtre des candidats — `find_unpaid_invoices_for_window`, `crates/kesh-db/src/repositories/reconciliation.rs:93-130` | `HAVING total_ttc BETWEEN ? - ? AND ? + ?` (`:115`), TTC par **sous-requête corrélée** (`:107`, `INVOICE_TTC_SUBQUERY_SQL`) | 1 000 ∉ 600 ± 0.05 : **exclue**, jamais scorée |
| Score — `propose_matches`, `crates/kesh-reconciliation/src/matching.rs:68` | `amount_score(tx.amount, *total_ttc)`, binaire (`:113`) | 0 si elle passait le filtre |
| Re-score à l'acceptation — `accept_one_invoice`, `crates/kesh-api/src/routes/reconciliation.rs:1196` | `invoices::total_ttc(&mut **tx, invoice.id)` | score 0 → `RECONCILIATION_SCORE_TOO_LOW` (`:1226-1235`), **avant** la garde de trop-perçu |
| Montant affiché dans la proposition — `reconciliation.rs:594` | `cand.total_ttc` (commentaire `:591-593` « = TTC ») | l'écran montre 1 000, pas 600 (`ReconciliationProposals.svelte:285`) |

Déjà justes : la garde de trop-perçu (`:1329-1344`, `amount_due`), le solde atteint (`:1444-1451`),
l'écriture (montant de la transaction, `:1387-1422`).

⚠️ Le défaut est né de la 24-2 (règlements multiples) — *la correction déplace le défaut vers ce
qu'elle vient d'écrire*. Aucun test ne pose un candidat sur une facture partiellement réglée ;
`accept_settles_an_invoice_in_two_payments` (`crates/kesh-api/tests/reconciliation_e2e.rs:2792`) passe
par l'API directe, grâce au score de **référence**, et n'asserte ni candidat ni score.

### Ce que l'inventaire a établi, et qui fixe les choix

1. **Le moteur de score est pur** : le montant arrive dans le triplet `(Invoice, Option<Contact>,
   Decimal)` (`matching.rs:63`). Passer au reste dû, c'est changer **ce que l'appelant y met** —
   `matching.rs` ne demande que nommage et doc (`:8-11`, `:58-60`). `propose_matches` a exactement deux
   appelants : `reconciliation.rs:575` (propositions) et `:1208` (re-score).
2. **Le filtre doit passer à la forme JOINTE** (`amount_due_derived_joins()` +
   `INVOICE_AMOUNT_DUE_DERIVED_SQL`, `invoice_settlements.rs:93-106`), celle que la 25-4-b1 a mise en
   service pour les listes : la forme corrélée y est interdite (`invoice_settlements.rs:32-34`, « N+1
   déguisé »). Le **re-score** porte sur UNE facture : forme scalaire (`amount_due`), dans la
   transaction.
3. **Les factures fournisseur ne sont pas concernées** : absentes du rapprochement ; elles se règlent
   en une fois (`supplier_invoices.rs:554-593`). Les règles, éclatements et rapprochements manuels ne
   portent aucune facture (`invoice_id: 0`, `reconciliation.rs:1965`, `:2374` ; `post_manual`
   `:2709-2710`).
4. **L'avoir** : une facture créditée est `cancelled` (`credit_notes.rs:586`, dépôt) et sort du filtre
   `status = 'validated'` ; le terme « avoir » du reste dû y vaut 0 en pratique — la forme jointe le
   porte de toute façon.
5. **L'arrondi (#476)** : `line_total` est à 4 décimales (`invoices.rs:425-427`), la TVA seule
   arrondie à 2 (`:165-176`) — le **reste dû peut avoir 4 décimales** (10.0050). La transaction bancaire
   est à 2 (`bank_imports.sql:62`). Conséquences :
   - le **score** ne peut jamais valoir 1 : 10.01 ≠ 10.005 après `normalize` (`matching.rs:113`) ;
   - les **gardes de trop-perçu** comparent la valeur brute : `reconciliation.rs:1336`,
     `invoice_settlements_write.rs:167` — le paiement exact de la QR (10.01) est refusé ;
   - les **tests de solde** aussi : `reconciliation.rs:1451` et `invoice_settlements_write.rs:233`
     (`due_after <= 0`) — un paiement de 10.00 laisse 0.0050, la facture reste « partiellement réglée »
     pour un demi-centime, sans `paid_at`.
   Le filtre, lui, tolère l'écart grâce aux ± 0.05.
6. **`LIMIT 50` sans `ORDER BY`** (`reconciliation.rs:116`, dépôt) : au-delà de 50 candidats, lesquels
   sont gardés n'est pas déterminé. Préexistant, hors périmètre — à ne pas aggraver.

## Acceptance Criteria

### Volet 1 — le reste dû dans le rapprochement

**AC 1** — `find_unpaid_invoices_for_window` filtre sur le **reste dû** par la forme **jointe** : les
tables dérivées d'`amount_due_derived_joins()` et `INVOICE_AMOUNT_DUE_DERIVED_SQL`, jamais réécrites à
la main, ni la forme corrélée. Le candidat porte le reste dû (le champ est renommé — `amount_due` — et
tous ses lecteurs suivent) ; son doc-comment le dit.

**AC 2** — `propose_matches` reçoit le **reste dû** aux deux appels : propositions (`:575`) et re-score
à l'acceptation (`:1196-1208`, forme scalaire `invoice_settlements::amount_due` **dans la transaction**,
à la place d'`invoices::total_ttc`). `matching.rs` : noms et doc-comments disent « montant à régler »,
plus « TTC ».

**AC 3** — Le montant affiché dans la proposition (`invoice_amount`, `:594`) est le **reste dû** ; le
commentaire `:591-593` suit *(présentation : Q3)*.

**AC 4** — Une facture **sans règlement** se comporte exactement comme avant : reste dû = TTC ;
candidats, scores et montant affiché inchangés. Les tests existants le tiennent
(`reconciliation_repository.rs:484-531`, `:807` ; `reconciliation_e2e.rs:819-822`), noms et messages
mis à jour là où ils disent « TTC ».

### Volet 2 — l'arrondi au centime *(si Q1 l'inclut)*

**AC 5** — Une seule grandeur, **le reste dû arrondi au centime** (`MidpointAwayFromZero`, la stratégie
de la QR — `generator.rs:38-39`), sert de montant à régler : filtre des candidats, score, garde de
trop-perçu du rapprochement (`:1336`) **et** du règlement manuel (`invoice_settlements_write.rs:167`),
tests de solde (`:1451` et `:233`). Un helper unique, à côté d'`amount_due`, la porte ; aucun site ne
réarrondit à sa façon. ⛔ Le reste dû **stocké/calculé** n'est pas modifié : seule la comparaison
arrondit.

**AC 6** — Sur une facture dont le reste dû brut est 10.0050 : le virement de **10.01** est candidat,
score 1 sur le montant, s'accepte, et **solde** la facture (`paid_at` posé, audit `invoice.paid`) ; un
virement de **10.02** reste un trop-perçu refusé ; un règlement manuel de 10.01 solde aussi.

### Volet 3 — tests, textes

**AC 7** — Tests qui auraient échoué avant le patch, chacun sur une facture **partiellement réglée** à
TVA **non nulle** :
- dépôt : la facture 1 000 réglée 400 est **candidate** pour 600, et ne l'est plus pour 1 000 ;
- propositions (e2e API) : `amountScore == 1.0` et `invoiceAmount` = le reste ;
- acceptation (e2e API) : le virement du solde s'accepte et solde la facture — sans passer par le
  score de référence (numéro de facture absent de la transaction) ;
- *(si Q1)* le cas 10.0050 de l'AC 6, au rapprochement et au règlement manuel ;
- Playwright : une facture réglée en partie apparaît dans les propositions avec son reste.

**AC 8** — Manuel : `user-manual.tex` § rapprochement dit que la proposition porte sur **ce qui reste
à payer** et qu'un solde de facture partiellement réglée est reconnu. ⚠️ **Le paragraphe du score
(`:1362-1371`) est faux sur le code, indépendamment de cette story** : il annonce un score gradué
(« écart < 1 CHF = score moyen »), un critère « Date », une « référence QR Bill », un seuil de 80 % et
un auto-accept à 95 % — rien de tel n'existe *(portée : Q2)*. PDF régénéré, contrôlé aplati.
CHANGELOG `[0.12.1]` *Fixed*.

## Tasks / Subtasks

- [ ] **T1 — candidats** (AC 1, 4) : forme jointe, champ renommé, lecteurs.
- [ ] **T2 — score et re-score** (AC 2, 3) : les deux appelants, affichage, `matching.rs`.
- [ ] **T3 — arrondi** (AC 5, 6) *(selon Q1)* : helper, cinq sites.
- [ ] **T4 — tests et mutations** (AC 7).
- [ ] **T5 — textes** (AC 8).
- [ ] **T6 — gates** : backend complet (base remise à zéro), frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **La forme corrélée dans le filtre des candidats** : c'est une liste (`LIMIT 50`).
- ⛔ **Réécrire la formule du reste dû** : `INVOICE_AMOUNT_DUE_DERIVED_SQL` et `amount_due` existent.
- ⛔ **Rendre le score gradué** : le binaire est une décision v0.1 assumée (`matching.rs:108-117`) ; cette
  story change la grandeur comparée, pas la forme du score.
- ⛔ **Toucher l'écriture** : elle porte le montant de la transaction, c'est juste.
- ⚠️ **L'ordre des gardes à l'acceptation** : le score (`:1226`) précède le trop-perçu (`:1336`). Un
  virement supérieur au reste doit continuer de sortir en `RECONCILIATION_OVERPAYMENT` s'il passe le
  score, en `SCORE_TOO_LOW` sinon — ne pas l'inverser sans le dire.
- ⚠️ `get_proposals_invoice_candidate_overrides_rule` (`reconciliation_rules_e2e.rs:962`) dépend du seuil
  0,5 (`INVOICE_OVERRIDE_THRESHOLD`, `:556`) : un score de montant qui change change la préséance.

### Où regarder

| Fichier | Pourquoi |
|---|---|
| `crates/kesh-db/src/repositories/reconciliation.rs:55-130` | candidats, fenêtre, tolérance |
| `crates/kesh-reconciliation/src/matching.rs:1-145` | score, triplet |
| `crates/kesh-api/src/routes/reconciliation.rs:480-640, 1056-1600` | propositions, acceptation, gardes |
| `crates/kesh-db/src/repositories/invoice_settlements.rs:27-220` | formes jointe et scalaire |
| `crates/kesh-db/src/repositories/invoice_settlements_write.rs:160-260` | règlement manuel (Q1) |
| `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:285`, `reconciliation.types.ts:30` | affichage |
| `docs/manual/fr/user-manual.tex:1355-1420` | rapprochement |

### Gardes-fous du dépôt

- Aucune migration. Repositories `kesh-db` touchés ⇒ **gate complet même en cours de boucle de revue**.
- Modules : `kesh-db`, `kesh-reconciliation`, `kesh-api`, `frontend` — **quatre** (cinq avec
  `kesh-i18n` si un libellé change).

## Questions pour Guy

**Q1 — inclure #476 (l'arrondi) ?** Le reste dû peut porter des demi-centimes ; comparé brut, il
empêche le score de valoir 1, fait refuser le paiement exact de la QR comme trop-perçu, et laisse une
facture « partiellement réglée » pour 0.005. C'est la même comparaison que cette story touche, aux
mêmes sites, plus le règlement manuel. **Recommandé : l'inclure** (volet 2) — sans lui, la 25-4-c
reconnaît le solde d'une facture à 2 décimales mais pas celui d'une facture dont le TTC en a 4.

**Q2 — le manuel du rapprochement.** Le paragraphe du score est faux sur le code (score gradué, date,
référence QR, seuils 80/95 % — inexistants), et deux autres passages aussi : l'acceptation par lot
dite « atomique » alors qu'elle est en succès partiel, et le rapprochement manuel / l'éclatement dits
« par facture » alors qu'ils n'en portent aucune (`:1392`, `:1396-1420`). **Recommandé** : corriger
dans cette story le paragraphe du score (il décrit la grandeur que la story change) ; ouvrir une issue
pour les deux autres.

**Q3 — le montant affiché.** Proposé : le **reste dû** seul (« 600.00 ») ; sur une facture réglée en
partie, ajouter « reste dû sur 1 000.00 » pour que le comptable reconnaisse la facture ?

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-28** — Créée. Inventaire par exploration, citations clés vérifiées ; le défaut est
  l'**exclusion** par le filtre SQL, pas seulement un score nul ; un cinquième site d'arrondi trouvé
  (`invoice_settlements_write.rs:233`) ; trois questions à Guy.

[#416]: https://github.com/guycorbaz/kesh/issues/416
[#420]: https://github.com/guycorbaz/kesh/issues/420
[#476]: https://github.com/guycorbaz/kesh/issues/476
