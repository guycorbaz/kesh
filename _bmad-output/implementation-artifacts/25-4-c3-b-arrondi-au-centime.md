# Story 25.4-c3-b : L'arrondi au centime, et l'écart passé en écriture

Status: review

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
- ⛔ **La classification tient compte des DEUX bornes, dans cet ordre** (brut `b`, arrondi `r`, payé `p`) :
  1. `p == r` et `r != b` → solde avec écart (ci-dessus) — y compris quand `r < b` (10.00 sur 10.004) ;
  2. `p > b` → **trop-perçu refusé** — y compris un montant **strictement entre** `b` et `r` (10.008 sur
     10.0050) : sans cette borne, la garde arrondie de l'AC 2 le laisserait passer en « partiel », `due_after`
     deviendrait négatif, `paid_at` se poserait **sans écriture d'écart** et la créance resterait créditrice —
     le défaut que la story ferme (validation P1, F1) ;
  3. sinon (`p <= b`) → règlement ordinaire au montant payé, deux lignes (partiel, ou solde exact si `p == b`).
  La garde de trop-perçu des sites 4 et 6 **est** cette règle : `p > b && p != r` → refus. L'AC 2 (« compare
  au reste arrondi ») se lit ainsi.
- **Le règlement manuel refuse un montant à plus de deux décimales** (400, `scale_within(&amount.normalize(), 2)` dans
  `SettleInvoiceRequest::into_parts`, `invoices.rs:1141`, patron de `dunning_levels.rs:82`) : un paiement
  se fait au centime. Aujourd'hui aucun contrôle d'échelle n'existe (`invoices.rs:1118-1125`). La double
  borne reste la garde de fond (la couche dépôt ne présume pas du handler). Le rapprochement n'est pas
  exposé : `bank_transactions.amount` est `DECIMAL(18,2)`. ⚠️ **`normalize()` d'abord** : `scale_within` lit
  `Decimal::scale()` (`limits.rs:31`), et « 10.000 » a une échelle de 3 pour une valeur au centime — sans
  normalisation, une saisie correcte serait refusée (validation P2). `normalize()` retire **tous** les
  zéros de fin (« 10.000 » → « 10 », « 10.10 » → « 10.1 ») : c'est un **contrôle**, pas une mise à
  l'échelle. `into_parts` rend le montant **tel que reçu** — la comparaison `Decimal` est insensible à
  l'échelle, et `invoice_settlements.amount` le stocke en `DECIMAL(19,4)` (validation P3, F1).
- Reste brut déjà au centime : deux lignes, comme aujourd'hui.

**AC 4 — Le compte d'arrondi manquant.** L'écart ne s'écrit que sur le compte désigné, **actif, imputable,
charge ou produit, de la société** (vérifié **au moment d'écrire** : il a pu être archivé depuis, #486).
Absent ou invalide → refus lisible, **sans rien écrire** : règlement manuel en 400 avec un code dédié (par
ex. `ROUNDING_ACCOUNT_NOT_CONFIGURED`) et un message qui renvoie à *Paramètres → Facturation* ;
rapprochement en `FailedProposal` du même code (pattern batch). Un paiement qui ne produit **pas** d'écart
n'exige rien.

**AC 5 — Le règlement manuel et le dialogue.** Sites 6 et 7 : garde et solde au centime (AC 3). Site 8 :
le dialogue compare la saisie au reste **arrondi** (big.js, arrondi loin de zéro — l'équivalent de
`MidpointAwayFromZero` sur un positif) ; le 10.01 pré-rempli est accepté. Il **refuse aussi une saisie à
plus de deux décimales** (valeur normalisée : « 10.000 » passe, « 10.008 » non), avec un message propre
(clé neuve, 4 locales) — sans quoi 10.008 passerait le contrôle client (≤ 10.01) pour tomber sur le 400
d'échelle du serveur. Avec ce contrôle, « saisie > reste arrondi » équivaut côté client à la double borne de
l'AC 3 (démontré en validation P3 : `r` vaut toujours le centime inférieur ou supérieur de `b`).
⛔ **Compter les décimales sur la forme canonique big.js** (`new Big(saisie).toString()`, partie après le
point ; big.js n'a pas de `.dp()`), **jamais par une regex sur la saisie brute** : le champ est du texte
libre, et « 1e-3 » n'a pas de point littéral pour trois décimales réelles (validation P3, F2). Le code d'erreur de l'AC 4 a son
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
- règlement manuel de **10.008** sur reste brut 10.0050 → 400 avec le **code d'erreur d'échelle** asserté
  (et non le seul statut : la garde actuelle le refuse déjà en 400 *trop-perçu*, un test sur le statut
  serait vert avant le patch — validation P3, F4), rien d'écrit ; et, au niveau du dépôt (`settle_invoice`
  appelé directement), **trop-perçu refusé**, `paid_at` non posé. ⚠️ Ce second test est vert avant le patch
  aussi : c'est un **garde de non-régression de la double borne**, discriminé par la mutation « borne
  `p > b` retirée », non par l'état d'avant ;
- facture à reste brut **10.004** : règlement manuel de **10.00** → soldée, écart **débit** 0.004 ;
- règlement manuel de 10.01 sur 10.0050 → soldé (API) ; « 10.000 » accepté (normalisation) ; Vitest du
  dialogue : 10.01 accepté, 10.008 et « 1e-3 » refusés avec le message d'échelle ;
- règlements successifs 5.00 puis 5.01 sur 10.0050 → le premier partiel, le second solde avec écart ;
- compte d'arrondi absent, puis archivé → refus, rien d'écrit (manuel et rapprochement) ;
- annulation d'un règlement à trois lignes → reste dû restauré, créance et compte d'arrondi revenus ;
- `reminder_amount_due` inchangé (tests existants verts) ;
- mutations : helper court-circuité, troisième ligne retirée, borne `p > b` retirée (le test 10.008 rougit).

**AC 8 — Textes.** `admin-manual.tex` (*Compte de différences d'arrondi*) : retirer « n'est encore lu par
aucune écriture » ; dire ce que fait l'écart et quand un règlement est refusé faute de compte.
`user-manual.tex` : là où il décrit le règlement manuel et le rapprochement, une phrase sur le solde au
centime et la troisième ligne. PDF régénérés, contrôlés aplatis (attention aux ligatures). CHANGELOG
`[0.12.1]` *Fixed* (#476).

## Tasks / Subtasks

- [x] **T1 — le helper** (AC 1) : `invoice_settlements`, `reminder_amount_due` rebranché.
- [x] **T2 — le rapprochement** (AC 2, 3, 4) : triplet, re-score, affichage, garde, solde et troisième ligne.
- [x] **T3 — le règlement manuel** (AC 3, 4, 5) : garde, solde, troisième ligne, code d'erreur.
- [x] **T4 — le dialogue et l'i18n** (AC 5).
- [x] **T5 — l'annulation** (AC 6).
- [x] **T6 — tests et mutations** (AC 7).
- [x] **T7 — textes** (AC 8).
- [x] **T8 — gates** : backend complet (base remise à zéro ; ⛔ `kesh-db` touché ⇒ gate complet même en
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

### Sites hors des huit, triés (validation P1, F2)

- **Échéancier** (`invoices/due-dates/+page.svelte:456`) et **en-tête de la fiche facture**
  (`invoices/[id]/+page.svelte:1040`) affichent `amountDue` brut via `formatInvoiceTotal` →
  `formatSwissAmount` → `big.toFixed(2)` (`journal-entries/balance.ts:93`). Le mode par défaut de big.js,
  `roundHalfUp`, arrondit l'équidistant **loin de zéro** : c'est `MidpointAwayFromZero`, sur un positif comme
  sur un négatif. **Hors périmètre, légitimes** : le frontend ne peut pas appeler le helper Rust, et son
  formatage donne déjà le même centime. Rien à changer ; le dialogue (site 8) suit la même règle big.js.

### Limite connue — le reste inférieur au demi-centime ([#490])

Un reste **brut** strictement entre 0 et 0.005 (avoir de 10.00 sur 10.004 ; ou règlement partiel de 10.00
sur 10.004 **déjà enregistré** sous l'ancien code, qu'aucune migration ne rejoue) a un arrondi nul : aucun paiement ne l'égale, 0.01 le dépasse. La facture ne peut être soldée.
**Défaut antérieur** (0.01 est déjà refusé aujourd'hui), **ni créé ni fermé ici** : ne pas l'élargir dans
cette story. Un reste brut négatif ou nul reste inchangé (tout paiement `p > 0` est un trop-perçu).

### Pièges

- ⚠️ **`DbError::ConfigurationRequired` n'est PAS le patron de l'AC 4** (validation P1, F3). C'est le voisin
  le plus proche (compte de banque manquant, `invoice_settlements_write.rs:126`), mais son mapping
  (`kesh-api/src/errors.rs:2913-2922`) jette le champ dans un `warn!` et rend un code générique
  `CONFIGURATION_REQUIRED`. L'AC 4 exige un code **dédié** : une variante propre (DbError → AppError →
  `ROUNDING_ACCOUNT_NOT_CONFIGURED`), et son pendant `FailedProposal` au rapprochement.
- **La contre-passation d'un compte d'arrondi archivé est déjà couverte** (validation P1, F4) — non par la
  garde `active` de `create_in_tx`, mais par le pré-contrôle `archived_accounts_in_tx`
  (`journal_entries.rs:1723`, dans `reverse_in_tx_inner` `:1518`), qui rend un 400
  `ReversalAccountsArchived` **nommant** les comptes à réactiver. Rien à écrire pour l'AC 6 de ce côté ;
  un test le confirme (annulation après archivage du compte d'arrondi → 400 nommant 6940).

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

Claude Opus 5.5 (`claude-opus-5-5`).

### Debug Log References

- Clippy `explicit_auto_deref` sur l'appel de `rounding_account_for_write` au rapprochement (`&mut **tx` → `tx`),
  au premier gate complet.
- Mutation M1 d'abord **jouée à vide** : le remplacement, sans `assert`, ne trouvait plus la ligne après
  `cargo fmt`. Rejouée avec contrôle (regex + `assert n == 1`).

### Completion Notes List

- **Le cœur, une seule fois** (`invoice_settlements.rs`) : `amount_due_to_centime` (sur
  `Money::round_to_centimes`), `classify_payment` (double borne, trois cas dans l'ordre de l'AC 3) et
  `settlement_journal_lines` (deux ou trois lignes, écart au crédit ou au débit). Rapprochement et règlement
  manuel les appellent tous deux ; `reminder_amount_due` appelle le helper.
- **Le compte d'arrondi au moment d'écrire** : `company_invoice_settings::rounding_account_for_write`
  (réglage + compte actif, imputable, charge ou produit, de la société, `FOR UPDATE`), appelé **seulement**
  quand la classification produit un écart. Variante dédiée `DbError::RoundingAccountNotConfigured` →
  400 `ROUNDING_ACCOUNT_NOT_CONFIGURED` (message qui renvoie à *Paramètres → Facturation*) ; au rapprochement,
  `FailedProposal` du même code.
- **Échelle** : `SettleInvoiceRequest::into_parts` refuse `scale_within(&amount.normalize(), 2)` en
  `VALIDATION_ERROR` ; le montant est rendu tel que reçu.
- **Audits** : `settled_amount` est désormais le montant **réglé** ; `paid_amount` et `rounding_difference`
  s'y ajoutent (règlement manuel, `reconciliation.accepted`, `invoice.paid`/`partially_settled`).
- **Dialogue** : `dueToCentime` (big.js `toFixed(2)`, `roundHalfUp`) pour le pré-remplissage et la
  comparaison ; `centimesDigits` compte les décimales sur `new Big(saisie).toFixed()` ; la saisie envoyée est
  nettoyée (`trim`). Clé `invoice-error-amount-scale` (4 locales, exemption #30 comme ses voisines),
  `sitesTotal` 1756 → 1757 recompté (`SettleInvoiceDialog.svelte` 18 → 19 `i18nMsg(`).
- **AC 6** : aucun code — `reverse_in_tx` retourne toutes les lignes, `archived_accounts_in_tx` nomme le compte
  d'arrondi archivé ; doc-comment de la réouverture mis à jour (il disait encore que l'encaissement ne bumpait
  `version` qu'au solde, faux depuis la 25-4-c2).
- **Tests** (périmètre : ce commit contre `e44b2b1a`…`262100d2`, la branche avant dev) : **13** Rust neufs
  (3 unitaires dans `invoice_settlements.rs`, 4 dans `reconciliation_e2e.rs`, 6 dans
  `invoice_echeancier_e2e.rs`) et **6** Vitest (`SettleInvoiceDialog.test.ts`, fichier neuf) — recomptés par
  `git diff | grep -cE '^\+\s*#\[(sqlx::)?test'` et `grep -c "it("`.
- **Mutations**, chacune tuée : M1 helper court-circuité (8 tests rouges) ; M2 troisième ligne retirée (6) ;
  M3 borne `p > b` retirée (1 — le test d'échelle, volet dépôt) ; M4 refus d'échelle serveur retiré (1) ;
  F1 dialogue comparé au brut (1) ; F2 contrôle d'échelle client retiré (2) ; F3 décimales comptées sur la
  saisie brute (2).
- **Gates** : backend complet sur base remise à zéro, `scripts/test-fast.sh` (fmt + clippy + nextest) —
  **2544/2544** ; frontend complet — check 0 erreur, lint i18n PASS, **845/845** Vitest, build ; **E2E
  complet** sur `kesh_e2e` reconstruite, 18:26 UTC (KF-045 hors fenêtre) — **227 passés, 19 ignorés,
  9 échecs**, tous expliqués : les 7 KF-029, et `product-revenue-account:133` + `products:185`, pollution
  d'état entre specs produits (rejoués seuls : un rouge au premier passage, les deux verts au second).

### File List

- `crates/kesh-db/src/errors.rs`
- `crates/kesh-db/src/repositories/invoice_settlements.rs`
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`
- `crates/kesh-db/src/repositories/company_invoice_settings.rs`
- `crates/kesh-db/src/repositories/reconciliation.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/routes/invoices.rs`
- `crates/kesh-api/src/routes/invoice_pdf_service.rs`
- `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/tests/reconciliation_e2e.rs`
- `crates/kesh-api/tests/invoice_echeancier_e2e.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte`
- `frontend/src/lib/features/invoices/SettleInvoiceDialog.test.ts` (neuf)
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/scripts/lint-i18n-ownership.js`
- `docs/manual/fr/admin-manual.tex` + `.pdf`
- `docs/manual/fr/user-manual.tex` + `.pdf`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- **2026-09-30** — Créée : huit sites de comparaison recontrôlés ; au solde, règlement au reste **brut** et
  écart en **troisième ligne** sur le compte désigné (arbitrage de Guy) ; refus lisible si le compte manque ou
  est devenu invalide.
- **2026-09-30** — Validation P1 (Sonnet) : 1 HIGH, 1 MEDIUM, 2 LOW, tous retenus après contrôle dans le
  code. **F1** (HIGH) : la garde arrondie de l'AC 2 × la classification binaire de l'AC 3 laissaient un
  paiement manuel entre brut et arrondi (10.008 sur 10.0050) solder la facture sans écart — classification à
  double borne, refus d'échelle > 2 au règlement manuel, test et mutation dédiés. **F2** (MED) : échéancier et
  en-tête de facture triés hors périmètre (big.js `roundHalfUp` = loin de zéro). **F3** (LOW) :
  `ConfigurationRequired` signalé comme faux patron. **F4** (LOW) : mécanisme d'archivage à la
  contre-passation précisé (`archived_accounts_in_tx`), test ajouté à l'AC 7 par les Pièges.
- **2026-09-30** — Validation P2 (Haiku) : 2 findings rendus (MED, LOW), **tous deux réfutés** — ils
  reprochent aux manuels de ne pas encore porter le texte que l'AC 8 demande d'écrire, ce que le prompt
  excluait. La passe déclarait l'axe 3 non exercé et ne rendait rien sur l'impact du refus d'échelle :
  **repris par l'orchestrateur**, qui y trouve trois défauts réels. (1) MED : `scale_within` lit l'échelle
  et non la valeur (« 10.000 » refusé) → `normalize()` prescrit. (2) MED : le dialogue laissait passer
  10.008 jusqu'au 400 serveur → contrôle client des deux décimales, clé i18n neuve. (3) MED hors périmètre :
  reste brut entre 0 et 0.005 insoldable, défaut antérieur → issue [#490], limite écrite. Tests ajoutés
  (normalisation, deux paiements successifs). Aucune occurrence de règlement à plus de deux décimales dans
  les tests existants (`grep` sur `crates/*/tests`, `frontend/tests`).
- **2026-09-30** — Validation P3 ciblée (Sonnet) sur la remédiation P2 : 3 MED, 1 LOW, tous retenus.
  F1 (LOW) : `into_parts` rend le montant reçu, `normalize()` n'est qu'un contrôle. F2 (MED) : décimales
  comptées sur la forme canonique big.js, pas par regex (« 1e-3 ») ; test ajouté. F3 (MED) : la seconde
  voie d'arrivée de #490 décrivait le cas que la story **ferme** — reformulée (règlement historique non
  rejoué), fiche et issue. F4 (MED) : le test 10.008 asserte le code d'échelle ; le test de dépôt déclaré
  garde de mutation. Équivalence client ⇔ double borne démontrée par la lentille (algèbre + 10⁶ cas).
- **2026-09-30** — Validation P4 ciblée (Haiku) sur `b1853bb4` : **0 finding**, trois axes déclarés et
  prouvés (six affirmations recontrôlées contre le code et big.js). Remarque non retenue : « `p > b && p != r`
  → refus » est la condition exacte du refus, `p == r` relevant du cas 1. **Boucle close** : trend
  1 HIGH/1 MED/2 LOW → 3 MED (orchestrateur ; 2 réfutés à la lentille) → 3 MED/1 LOW → 0 ; lentilles
  Sonnet → Haiku → Sonnet → Haiku ; P3 et P4 ciblées ; aucune remédiation ne touche de code.
- **2026-09-30** — Implémentée (T1–T8) : cœur partagé (`amount_due_to_centime`, `classify_payment`,
  `settlement_journal_lines`), compte d'arrondi revérifié au moment d'écrire, code dédié, refus d'échelle,
  dialogue au centime ; 13 tests Rust et 6 Vitest neufs, 7 mutations tuées ; gates backend 2544/2544,
  frontend 845/845, E2E 227/19/9 expliqués.

[#476]: https://github.com/guycorbaz/kesh/issues/476
[#490]: https://github.com/guycorbaz/kesh/issues/490
