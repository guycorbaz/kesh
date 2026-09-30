# Story 25.4-c3-b : L'arrondi au centime, et l'écart passé en écriture

Status: ready-for-dev

**Issue : [#476]** — ⛔ la PR porte `closes #476`, titre ET corps (§ *Issue Tracking Rule*).

**Dernière des trois stories de la 25-4-c3.** Arbitrage de Guy (2026-09-30) : l'écart d'arrondi au
centime se **passe en écriture** sur le **compte de différences d'arrondi** désigné dans les paramètres
(`company_invoice_settings.default_rounding_account_id`, **c3-a1**, mergée #487), que les plans livrés
proposent et désignent d'office (`6940`, **c3-a2**, PR #489). **Empilée sur la c3-a2** : branche
`story/25-4-c3-b-arrondi-au-centime`.

Les findings de la 25-4-c (validation P1 F1–F3, P3 F4) et de sa validation sont la source des faits,
recontrôlés ci-dessous.

## Story

En tant que comptable,
je veux qu'un paiement du montant arrondi au centime solde exactement une facture dont le total a plus de
deux décimales,
afin que la créance se ferme à zéro, sans reste d'un demi-centime ni solde créditeur, et que l'écart soit
visible au grand livre.

## Le défaut, vérifié dans le code

`line_total` est à 4 décimales (`invoices.rs`), la TVA seule arrondie à 2 : le **reste dû peut porter 4
décimales** (10.0050). La QR, les rappels (`reminder_amount_due`, `invoice_pdf_service.rs:112`) et le
dialogue de règlement (`SettleInvoiceDialog.svelte:75`, `toFixed(2)`) réclament **10.01**. Les comparaisons,
elles, lisent le **brut** :

| # | Site | Aujourd'hui | Effet sur un reste brut de 10.0050 |
|---|---|---|---|
| 1 | Filtre des candidats (`reconciliation.rs` dépôt, `HAVING amount_due BETWEEN`) | brut ± 0.05 | candidat — **juste**, à garder (écart ≤ 0.005 < 0.05) |
| 2 | Score — triplet des propositions (`reconciliation.rs:578`) et re-score (`:1304`) | brut | 10.01 ≠ 10.005 : score de montant **0** |
| 3 | Montant affiché (`invoiceAmount`, `:602`) et mention TTC | brut | « 10.005 » |
| 4 | Garde de trop-perçu de l'acceptation (`:1444`) | `tx.amount > due_before` brut | 10.01 **refusé** en trop-perçu |
| 5 | Solde de l'acceptation (`:1559`) | `due_after <= 0` brut | — (4 refuse d'abord) |
| 6 | Garde de trop-perçu du règlement manuel (`invoice_settlements_write.rs:167`) | brut | 10.01 **refusé** |
| 7 | Solde du règlement manuel (`:246`) | brut | un paiement de 10.00 laisse 0.0050 : « partiellement réglée » pour un demi-centime |
| 8 | Contrôle de saisie du dialogue (`SettleInvoiceDialog.svelte:111`) | `n > Number(amountDue)` | le **10.01 qu'il pré-remplit lui-même** est refusé |

Et **payer 10.01 une facture de 10.0050 en arrondissant seulement la comparaison** laisserait la créance
**créditrice de 0.0050** au grand livre (lignes en `DECIMAL(19,4)`) — d'où l'arbitrage : l'écart s'écrit.

L'arrondi existe déjà : `Money::round_to_centimes()` (`crates/kesh-core/src/types/money.rs:66`,
`MidpointAwayFromZero`) et `reminder_amount_due` (`invoice_pdf_service.rs:112`, qui arrondit et refuse un
reste nul).

## Acceptance Criteria

**AC 1 — Une seule définition du « reste dû au centime ».** Un helper, à côté d'`amount_due`
(`invoice_settlements.rs`), rend le reste dû arrondi au centime en s'appuyant sur
`Money::round_to_centimes()` — sans recopier la stratégie. `reminder_amount_due` l'appelle (son refus du
reste nul inchangé). ⛔ Le reste dû **calculé** (`amount_due`, `INVOICE_AMOUNT_DUE_DERIVED_SQL`) n'est pas
modifié : seules les comparaisons et l'affichage arrondissent.

**AC 2 — Le rapprochement.** Sites 2 et 3 : le triplet des propositions et le re-score reçoivent le reste
dû **arrondi** ; `invoiceAmount` affiche le reste arrondi, et la mention `invoiceTotalTtc` compare et
affiche au centime. Site 1 inchangé (le doc-comment dit pourquoi la tolérance suffit). Site 4 : la garde
de trop-perçu compare au reste **arrondi**.

**AC 3 — Le solde exact, avec la troisième ligne.** Quand un paiement (rapprochement **ou** règlement
manuel) **égale le reste dû arrondi** et que le reste **brut** en diffère :
- le règlement est enregistré pour le reste **brut** (`invoice_settlements.amount`, `DECIMAL(19,4)`), si
  bien que le reste dû tombe à **zéro exactement** et que `paid_at` se pose ;
- l'écriture porte **trois lignes** : débit contrepartie (banque, caisse…) du montant **payé** ; crédit
  créance du reste **brut** ; et l'**écart** sur le compte de différences d'arrondi — au **crédit** si le
  paiement dépasse le brut (10.01 contre 10.0050), au **débit** sinon (10.00 contre 10.004). Écriture
  équilibrée, aucune ligne à zéro (`chk_jel_debit_credit_exclusive`).
- Un paiement **inférieur** au reste arrondi reste un règlement partiel ordinaire (montant payé, deux
  lignes) ; un paiement **supérieur** reste un trop-perçu refusé.
- Reste brut déjà au centime : deux lignes, comme aujourd'hui.

**AC 4 — Le compte d'arrondi manquant.** L'écart ne s'écrit que sur le compte désigné, **actif, imputable,
charge ou produit, de la société** (vérifié **au moment d'écrire** : il a pu être archivé depuis, #486).
Absent ou invalide → refus lisible, **sans rien écrire** : règlement manuel en 400 avec un code dédié (par
ex. `ROUNDING_ACCOUNT_NOT_CONFIGURED`) et un message qui renvoie à *Paramètres → Facturation* ;
rapprochement en `FailedProposal` du même code (pattern batch). Un paiement qui ne produit **pas** d'écart
n'exige rien.

**AC 5 — Le règlement manuel et le dialogue.** Sites 6 et 7 : garde et solde au centime (AC 3). Site 8 :
le dialogue compare la saisie au reste **arrondi** (big.js, arrondi loin de zéro — l'équivalent de
`MidpointAwayFromZero` sur un positif) ; le 10.01 pré-rempli est accepté. Le code d'erreur de l'AC 4 a son
libellé dans les 4 locales (`reminder-error-label.ts` / libellés d'erreur existants : suivre le patron du
dépôt).

**AC 6 — L'annulation.** Annuler un règlement à trois lignes (règlement manuel annulé, rapprochement
annulé) contre-passe **les trois lignes** et rend le reste dû brut d'avant — à vérifier sur
`journal_entries::reverse_in_tx` (`:1437`) et par test. Le site de réouverture
(`invoice_settlements_write.rs:486`, `due_after > 0`) reste juste sur le brut — le doc-comment le dit.

**AC 7 — Tests.** Chacun aurait échoué avant le patch :
- facture à reste brut **10.0050** : virement de **10.01** candidat, score de montant 1, **accepté**, facture
  soldée (`paid_at`, audit `invoice.paid`), règlement de 10.0050, écriture à trois lignes (écart **crédit**
  0.0050 sur le compte d'arrondi), **créance soldée à zéro** au grand livre ; virement de 10.02 refusé en
  trop-perçu ;
- facture à reste brut **10.004** : règlement manuel de **10.00** → soldée, écart **débit** 0.004 ;
- règlement manuel de 10.01 sur 10.0050 → soldé (API) ; Vitest du dialogue : 10.01 accepté ;
- compte d'arrondi absent, puis archivé → refus, rien d'écrit (manuel et rapprochement) ;
- annulation d'un règlement à trois lignes → reste dû restauré, créance et compte d'arrondi revenus ;
- `reminder_amount_due` inchangé (tests existants verts) ;
- mutations : helper court-circuité, troisième ligne retirée.

**AC 8 — Textes.** `admin-manual.tex` (*Compte de différences d'arrondi*) : retirer « n'est encore lu par
aucune écriture » ; dire ce que fait l'écart et quand un règlement est refusé faute de compte.
`user-manual.tex` : là où il décrit le règlement manuel et le rapprochement, une phrase sur le solde au
centime et la troisième ligne. PDF régénérés, contrôlés aplatis (attention aux ligatures). CHANGELOG
`[0.12.1]` *Fixed* (#476).

## Tasks / Subtasks

- [ ] **T1 — le helper** (AC 1) : `invoice_settlements`, `reminder_amount_due` rebranché.
- [ ] **T2 — le rapprochement** (AC 2, 3, 4) : triplet, re-score, affichage, garde, solde et troisième ligne.
- [ ] **T3 — le règlement manuel** (AC 3, 4, 5) : garde, solde, troisième ligne, code d'erreur.
- [ ] **T4 — le dialogue et l'i18n** (AC 5).
- [ ] **T5 — l'annulation** (AC 6).
- [ ] **T6 — tests et mutations** (AC 7).
- [ ] **T7 — textes** (AC 8).
- [ ] **T8 — gates** : backend complet (base remise à zéro ; ⛔ `kesh-db` touché ⇒ gate complet même en
  cours de boucle), frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Arrondir le reste dû calculé** ou le TTC stocké : seules les comparaisons et l'affichage arrondissent.
- ⛔ **Recopier la stratégie d'arrondi** (`round_dp_with_strategy(2, MidpointAwayFromZero)` à la main) : le
  helper, sur `Money::round_to_centimes()`.
- ⛔ **Enregistrer le règlement au montant payé quand il solde** : le reste brut resterait à ±0.0050 et la
  créance ne se fermerait pas.
- ⛔ **Arrondir le filtre SQL** (`ROUND` dans le `HAVING`) : la tolérance couvre l'écart, et ce serait une
  seconde définition.
- ⛔ **Écrire l'écart sur un compte déduit** (produit par défaut, charges diverses) quand le réglage manque :
  refuser (AC 4).
- ⚠️ **Le verrou optimiste de la 25-4-c2** (`UPDATE invoices … version = ?`) et l'ordre des gardes (score
  avant trop-perçu) : ne rien y changer.

### Où regarder

| Fichier | Pourquoi |
|---|---|
| `crates/kesh-db/src/repositories/invoice_settlements.rs` | `amount_due`, le helper à poser |
| `crates/kesh-core/src/types/money.rs:66` | `round_to_centimes` |
| `crates/kesh-api/src/routes/invoice_pdf_service.rs:112` | `reminder_amount_due` |
| `crates/kesh-api/src/routes/reconciliation.rs:570-610, 1300-1580` | propositions, re-score, garde, écriture, solde |
| `crates/kesh-db/src/repositories/invoice_settlements_write.rs:160-260, 470-500` | règlement manuel, annulation |
| `crates/kesh-db/src/repositories/journal_entries.rs:1437` | `reverse_in_tx` |
| `crates/kesh-db/src/repositories/company_invoice_settings.rs` | le réglage (c3-a1) |
| `crates/kesh-api/src/routes/invoices.rs:1183` | `settle_invoice_handler`, mapping d'erreur |
| `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:70-115` | dialogue |

### Gardes-fous du dépôt

- Aucune migration. Modules : `kesh-db`, `kesh-api`, `frontend`, `kesh-i18n` (+ `docs`) — sous le seuil.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-30** — Créée : huit sites de comparaison recontrôlés ; au solde, règlement au reste **brut** et
  écart en **troisième ligne** sur le compte désigné (arbitrage de Guy) ; refus lisible si le compte manque ou
  est devenu invalide.

[#476]: https://github.com/guycorbaz/kesh/issues/476
