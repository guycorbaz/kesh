# Story 36-18a : les montants « 1.250 » qui écrivent dans les livres — soldes de départ, facture fournisseur, règlement (#1050, #1159, #1172, #1174)

Status: ready-for-dev

*Fiche issue du découpage de la 36-18 (`36-18-montants-ambigus-autres-ecrans.md`, `Status: split`), approuvé par
l'orchestrateur le 2026-11-15 (sept modules de code, règle de Guy « stories courtes » ; C-36-18-10). Rédigée par l'agent de
rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-18`, code lu à `fed34a9c7` (tête de `epic/36-0.18.0`).
Release **0.18.0**, phase 2, décision **D6** de Guy du 2026-11-15. **Aucune migration**, aucun changement serveur ni
d'API. Voie **2**, bases `kesh_e3618a`, `kesh_e2e_e3618a`, `kesh_ci_e3618a` (voie 1). **Part de la 36-15 fusionnée**
(`SupplierInvoiceForm.svelte`, que la fiche 36-15 touche). Registre : `e36-choix-autonomes-36-18.md` (C-36-18-1 à
C-36-18-10) — **les décisions communes aux deux lots sont ici** ; la 36-18b y renvoie. Inventaires bruts :
`/home/gcorbaz/devel/kesh-gate-logs/e36-18/fiche-inventaires.log`. Validation non lancée.*

## Story

En tant que titulaire qui tient ses livres réels dans Kesh, je veux qu'un solde de départ, un prix unitaire ou un montant
attendu de facture fournisseur, et le montant d'un règlement de facture se lisent comme le montant d'une écriture —
« 1.250 » refusé comme ambigu avec un message qui dit comment l'écrire, « 1'250 » accepté —, sans que rien de ce que
j'écrivais juste jusqu'ici (« 1.25 », « 1250 », « 1'250.50 », un montant rouvert) devienne refusé, afin qu'aucune écriture
ne passe au millième du montant voulu sans que je le voie.

## Ce qui se lit au code (`fed34a9c7`)

### La règle, déjà écrite par la 36-3 et la 36-3b — réemployée, non réécrite

- **Le prédicat** — `frontend/src/lib/shared/utils/amount-ambiguity.ts:38` `export function isAmbiguousThousands(parts: AmountParts): boolean {`,
  `:39-43` : `parts.fraction.length === 3 && !parts.grouped && /^0*[1-9][0-9]{0,2}$/.test(parts.integerDigits)`.
- **Le lecteur** — `frontend/src/lib/features/journal-entries/balance.ts:110` `export function readJournalAmount(raw: string): JournalAmountReading {`,
  grammaire `:90` `const JOURNAL_AMOUNT_RE = /^([1-9]\d{0,2}(?:['’]\d{3})+|\d{1,15})(?:[.,](\d{1,4}))?$/;`, sans rognage
  (`:111`), `:119` `if (isAmbiguousThousands({ integerDigits, fraction, grouped })) return { kind: 'ambiguous' };`.
- **Ses aides** — `balance.ts:128` `lineAmountProblem(debit, credit)` ; `:143` `journalAmountPayload(raw)` (`'0'` pour le
  vide, `:145`) ; `:157` `journalAmountReader` (`Big(0)` pour le vide, `null` hors règle, `:159-160`) ; `:153`
  `defaultAmountReader` = `isValidAmount(raw) ? parseAmount(raw) : null`, « que gardent les soldes de départ (#1050) » (`:152`).
- **Les aides de la 36-3b, génériques sous un nom de prix** — `frontend/src/lib/features/invoices/unit-price.ts:25-28`
  `unitPricePayload`, `:34-37` `unitPriceProblem`, `:46-50` `unitPriceFieldValue` (`readableAmount(raw, 2)`, repli
  `new Big(raw).toFixed(4)` si ambigu) ; `:17` importe `readableAmount` de `supplier-invoice-form.ts` — un import inverse
  ferait un cycle (`:10-12`).
- **La forme rouverte** — `frontend/src/lib/features/supplier-invoices/supplier-invoice-form.ts:214-224` `readableAmount`
  (« 1.2340 » → `1.234`, relevé de la fiche 36-3b, `:73-76`).
- **Les messages de la 36-3** — `crates/kesh-i18n/locales/fr-CH/messages.ftl:313-314` conseillent `1.2340` et « quatre
  décimales au plus » : faux pour un règlement au centime. Les clés `common-*` sont globales
  (`frontend/scripts/lint-i18n-ownership.js:16` `const GLOBAL_NAMESPACES = ['error', 'tooltip', 'common', 'mode', 'shortcut', 'demo'];`).

### Les quatre sites

**1. Soldes de départ (#1050)** — `frontend/src/routes/(app)/settings/opening-balances/+page.svelte` (759 lignes) :
- `:135` `const balance = $derived(computeBalance(rows));` — lecteur par défaut ;
- `:147-152`, `:163-169` : `if (!isValidAmount(r.debit) || !isValidAmount(r.credit)) return true;`
  `return parseAmount(r.debit).gt(0) || parseAmount(r.credit).gt(0);` ; `:170-172` `complementInvalid` par `isValidAmount` ;
- `:199-200` (générer) et `:245-246` (compléter) : `debit: r.debit === '' ? '0' : r.debit.replace(',', '.'),` — « 1.250 »
  part `1.250`, lu 1,25 par le serveur (`crates/kesh-api/src/routes/opening_balances.rs:363` `Decimal::from_str(&line.debit)`) ;
- `:490`, `:505`, `:621`, `:634` `aria-invalid={!isValidAmount(row.debit)}`, aucun message ;
- `frontend/src/lib/features/opening-balances/opening-balances-totals.ts:39-41`
  `function amount(raw: string): Big { return isValidAmount(raw) ? parseAmount(raw) : new Big(0); }`.
- Aucun montant du serveur ne s'y rouvre (grille vide ; écran verrouillé après génération) ; aucun rognage aujourd'hui.

**2. Prix unitaire d'une facture fournisseur (#1159)** — `supplier-invoice-form.ts` :
- `:146-155` `linesInvalid` : `!isValidAmount(l.unitPrice.trim(), { allowEmpty: false }),` (`:153`) ;
- `:72-90` `sumTtc`, `:117-125` `lineNotInCentimes` : `parseAmount(l.unitPrice.trim())` (`:78`, `:120`) — « 1.250 » est au
  centime : le contrôle de #687 ne l'arrête pas ;
- `:169` `unitPrice: decimalForServer(l.unitPrice),` ; `:248` `unitPrice: readable(line.unitPrice, 2),` (rouvert : « 1.2340 »
  → `1.234`, refusé à tort une fois la règle posée) ;
- la quantité (`:63`, `:152`, `:168`) lit « 1.250 » comme 1,25 unité : **hors de la règle** (C-e360-6) ;
- `SupplierInvoiceForm.svelte` (890 lignes) : champ `:798-804` (`data-testid="supplier-invoice-line-unit-price"`), refus à
  l'envoi `:372-377` (`supplier-invoices-err-lines`) puis `:382-388` (centime) ;
- serveur `crates/kesh-api/src/routes/supplier_invoices.rs:539` `scale_within(&line.unit_price.normalize(), MAX_DECIMAL_SCALE)` :
  « 1.250 » passe, lu 1,25.

**3. Montant attendu d'une facture fournisseur (#1172)** — `supplier-invoice-form.ts:283`
`expected === '' ? null : isValidAmount(expected) ? decimalForServer(expected) : expected,` ; champ
`SupplierInvoiceForm.svelte:749-754` ; rouvert `:184-185` `readableAmount(init.expectedPaymentAmount, 2)` (colonne
`DECIMAL(19,4)`, `crates/kesh-db/migrations/20260628000001_supplier_invoices.sql:40`) ; scan QR `:323`
`fExpectedAmount = p.expectedAmount;` (deux décimales au plus : jamais ambigu) ; envoyé sans QR seulement (`:399-404`,
`const req = qrLocked`).

**4. Règlement d'une facture client (#1174)** — `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte` (327 lignes) :
- `:166-169` `const n = Number(amount);` → « Le montant doit être supérieur à zéro » — « 1.250 » vaut 1,25 ; « 1'250 » et
  « 1,25 » rendent `NaN`, d'où ce message, faux ;
- `:67-76` `centimesDigits` (forme canonique) : « 1.250 » → `1.25`, passe ; `:184` comparaison au reste dû — un montant
  mille fois trop bas passe pour un règlement partiel ; `:205` `amount: amount.trim(),` ;
- pré-rempli `:92` `dueToCentime(amountDue).toFixed(2)` (jamais ambigu) ; champ `:294` ; bandeau `:304-307` ;
- serveur `crates/kesh-api/src/routes/invoices.rs:1139-1147` `scale_within(&self.amount.normalize(), 2)` ;
- **tests existants contraires** — `SettleInvoiceDialog.test.ts:70-75` (« 10.008 », échelle), `:77-81` (« 1e-3 »), `:83-89`
  (« 10.000 » accepté et envoyé) (C-36-18-5).

### Inventaire par le symptôme

Fait pour toute la 36-18 ; sept motifs, sorties brutes au journal `kesh-gate-logs/e36-18/fiche-inventaires.log` (champ :
`inputmode="decimal"`, `type="number"`, `bind:value` d'un nom de montant ; lecteur : `readJournalAmount` et ses aides,
`isValidAmount|parseAmount|decimalForServer|computeBalance`, `replace(','`, corps de requête portant un montant). **Sonde
positive** : le premier motif rend `SettleInvoiceDialog.svelte`, site connu ; le quatrième rend `JournalEntryForm.svelte`,
site corrigé. ⚠️ Le motif `bind:value` a manqué `fFee` (`\b` non reconnu par `git grep -E`), que le premier couvre. **Angles
morts** : un champ sans `inputmode`, `type="number"` ni nom de montant ; un montant lu hors d'un champ (CSV, CAMT, QR).

| # | Site | Décision |
|---|---|---|
| 1 | Soldes de départ (`opening-balances/+page.svelte:135-634`, `opening-balances-totals.ts:40`) | **36-18a** (#1050) |
| 2 | Prix unitaire fournisseur (`supplier-invoice-form.ts:153`, `:169`, `:248`) | **36-18a** (#1159) |
| 3 | Montant attendu fournisseur (`supplier-invoice-form.ts:283`, `SupplierInvoiceForm.svelte:185`) | **36-18a** (#1172) |
| 4 | Règlement (`SettleInvoiceDialog.svelte:166-205`) | **36-18a** (#1174) |
| 5 | Prix d'article, frais de rappel, montant minimal, filtres de la liste | **36-18b** (#1173, #1175, #1176, #1177) |
| 6 | Quantités ; taux de TVA (`settings/vat-rates/+page.svelte:287`, `:331`) ; relevé manuel (deux décimales) ; écrans des 36-3, 36-3b, 36-2, 35-6 ; entiers et texte | hors de la règle ou déjà corrigés (détail : fiche 36-18, § *Inventaire*) |

Aucun site hors des huit issues : aucune issue neuve.

## Décisions

*D1 à D4 et D9 valent pour les deux lots ; la 36-18b y renvoie.*

- **D1 — Une lecture, celle de la 36-3 ; deux aides rendues génériques** (C-36-18-1). Dans `balance.ts`, à côté de
  `readJournalAmount` : `amountEntryPayload(raw: string): string | null` (`payload` d'une lecture `clear`, `null` sinon,
  vide compris) et `amountEntryProblem(raw: string): 'ambiguous' | 'unreadable' | null` — les corps de `unitPricePayload`
  et `unitPriceProblem`, **déplacés** ; `unit-price.ts` les ré-exporte sous ses noms
  (`export { amountEntryPayload as unitPricePayload, amountEntryProblem as unitPriceProblem } from …`). Ni
  `readJournalAmount`, ni `isAmbiguousThousands`, ni `JOURNAL_AMOUNT_RE` ne changent ; aucune grammaire propre à un écran.
  *Écartées* : importer les noms de prix depuis tous les écrans ; un module sous `shared/utils/` qui importerait
  `features/` ; une grammaire par écran.
- **D2 — Un montant du serveur ne se rouvre jamais ambigu** (C-36-18-2). `supplier-invoice-form.ts`, à côté de
  `readableAmount` : `readableUnambiguousAmount(raw: string, minDecimals: number): string` = `readableAmount(raw,
  minDecimals)`, sauf si la lecture de ce résultat est `ambiguous` : alors `new Big(raw).toFixed(4)` ; valeur illisible
  rendue telle quelle. `unitPriceFieldValue` lui délègue (`readableUnambiguousAmount(raw, 2)`), même sortie. Appelée à
  `formLineFromResponse` (`:248`) et `SupplierInvoiceForm.svelte:185`. *Écartées* : recopier la logique ; déplacer
  `readableAmount` (six importateurs).
- **D3 — Deux messages communs, décimales en paramètre** (C-36-18-3). Sous chaque champ fautif (bordure
  `border-destructive`, `aria-invalid="true"`, et `aria-describedby` vers l'`id` du message — *validation P1, V1-6* ; le
  règlement compris, D8) : `common-amount-ambiguous` ou `common-amount-unreadable` (`{ $max }` : **4** pour
  les soldes, le prix et le montant attendu fournisseur ; **2** pour le règlement). Les autres refus de chaque écran restent.
  *Écartées* : réemployer `journal-entry-form-amount-*` (texte faux au centime, entrées `KNOWN_VIOLATIONS`) ; une paire par écran.
- **D4 — Chaque écran rogne là où il rogne aujourd'hui** (C-36-18-4) : saisie rognée au règlement, au prix et au montant
  attendu fournisseur ; les soldes de départ lisent la saisie brute, comme aujourd'hui.
- **D5 — Soldes de départ (#1050)** : `computeBalance(rows, journalAmountReader)` ; les filtres `:147-152`, `:163-169`,
  `complementInvalid` et les quatre `aria-invalid` lisent par `journalAmountReader` ; la charge `:199-200`, `:245-246`
  devient `journalAmountPayload(…)` (jamais `null` : `canGenerate` exige `isBalanced`, `canSubmitComplement`
  `!complementInvalid`) ; `opening-balances-totals.ts` `amount(raw)` = `journalAmountReader(raw) ?? new Big(0)`. Sous chaque
  ligne fautive, dans les deux grilles, une rangée `<tr><td colspan=…>` avec le message de `lineAmountProblem(row.debit,
  row.credit)` (`$max` 4), `data-testid="opening-balances-amount-ambiguous"` / `-unreadable`, avec un `id` par ligne que
  les deux champs de la ligne nomment dans `aria-describedby` (*V1-6*). *Validation P1, V1-9 :* après D5,
  `defaultAmountReader` n'a plus aucun appelant de production (`git grep -n "computeBalance(\|classifyLine(" -- frontend/src ':!*.test.ts'`
  à `fed34a9c7` : `JournalEntryForm.svelte:179-180`, qui passent `journalAmountReader`, et `opening-balances/+page.svelte:135`,
  que D5 convertit) ; il est **retiré**, et le paramètre `read` de `computeBalance` et `classifyLine` devient obligatoire.
  Les appels de `balance.test.ts` qui comptaient sur le défaut (`:105`, `:116`, `:126`, `:132`, `:140`, `:149`, `:159-187`)
  passent `journalAmountReader` explicitement (mêmes attentes : aucun de ces montants n'est ambigu) ; les assertions qui ne
  documentaient que l'ancien défaut (`:282-288`, `:318`) sont retirées, ce que le Dev Agent Record dit.
- **D6 — Prix unitaire fournisseur (#1159)** : `linesInvalid` juge le prix par `amountEntryPayload(l.unitPrice.trim()) ===
  null` ; `sumTtc` et `lineNotInCentimes` lisent la même charge (`null` → ligne ignorée / `null`) ; `linesForServer` envoie
  la charge ; la quantité garde `isValidAmount` / `decimalForServer`. Sous la ligne : `amountEntryProblem` (`$max` 4),
  `data-testid="supplier-invoice-line-unit-price-ambiguous"` / `-unreadable`. À l'envoi, **avant** `linesInvalid`, la
  première ligne fautive arrête l'envoi : `supplier-invoices-err-line-unit-price-ambiguous` / `-unreadable` (`{ $line }`,
  1-based). Rouvert : D2.
- **D7 — Montant attendu (#1172)** : sans QR, une saisie ambiguë ou illisible arrête l'envoi (message commun `$max` 4 dans
  `formError` et sous le champ, `data-testid="supplier-invoice-expected-amount-ambiguous"` / `-unreadable`) ;
  `paymentFields` envoie la charge d'une saisie claire. Rouvert : D2. Le montant d'un scan QR n'est pas touché.
  *Validation P1, V1-4 :* **contrat de `paymentFields`** — vide → `null` ; claire → la charge ; ambiguë ou illisible →
  `null` (jamais la saisie brute : une fonction exportée ne doit pas pouvoir envoyer « 1.250 » ; la garde du formulaire
  a arrêté l'envoi avant). **Ordre des gardes de l'envoi** : date de facture, échéance antérieure (`:371`), **montant
  attendu** (sans QR), **prix des lignes** (D6), `linesInvalid` (`:372`), centime (`:382`).
- **D8 — Règlement (#1174)** : `clientError`, après les dates, lit `amount.trim()` : vide → `invoice-error-amount-positive` ;
  `ambiguous` / `unreadable` → message commun (`$max` 2) ; `clear` : ≤ 0 → `invoice-error-amount-positive` ;
  `centimesDigits(payload) > 2` → `invoice-error-amount-scale` ; au-delà du reste dû → `invoice-error-amount-over-due`.
  `onConfirm` reçoit `amount: payload`. Les trois tests contraires (`:70-89`) sont réécrits : « 10.000 » et « 10.008 »
  ambigus, « 1e-3 » illisible, l'échelle prouvée sur « 10.0081 » (C-36-18-5). *Validation P1, V1-1 :* les décimales se
  comptent toujours sur la forme **canonique** de big.js (`centimesDigits`, `:67-76`, #476), appliquée à la charge —
  « 10.0100 » et « 1250.0000 » restent acceptés et partent tels quels ; les deux tests réécrits ne gardant plus cet
  invariant, deux tests neufs le gardent (AC5) et M16 le vise. *V1-6 :* le champ `settle-amount` (`:294`) reçoit la
  bordure, `aria-invalid` et `aria-describedby` vers le bandeau (`id` posé à `:304-307`) quand l'erreur porte sur le montant.
- **D9 — Textes** (quatre locales, jamais en fin de fichier ; fallbacks = texte fr-CH à l'identique) :
  - à la suite de `common-account-no-match` (fr-CH `:2657`, de-CH `:2529`, it-CH `:2527`, en-CH `:2534`) :
    - `common-amount-ambiguous` — fr-CH « Montant ambigu : un point ou une virgule suivis de trois chiffres peuvent
      séparer les milliers. Écrivez 1'250 pour mille deux cent cinquante, ou 1.25 pour un franc vingt-cinq. » ; de-CH
      « Mehrdeutiger Betrag: Ein Punkt oder ein Komma gefolgt von drei Ziffern kann Tausender trennen. Schreiben Sie 1'250
      für eintausendzweihundertfünfzig oder 1.25 für einen Franken und 25 Rappen. » ; it-CH « Importo ambiguo: un punto o
      una virgola seguiti da tre cifre possono separare le migliaia. Scrivete 1'250 per milleduecentocinquanta, o 1.25 per
      un franco e 25 centesimi. » ; en-CH « Ambiguous amount: a point or a comma followed by three digits may separate
      thousands. Write 1'250 for one thousand two hundred fifty, or 1.25 for one franc and 25 centimes. »
    - `common-amount-unreadable` — fr-CH « Montant illisible : chiffres, apostrophe des milliers (1'250), point ou virgule
      décimale, { $max } décimales au plus. » ; de-CH « Unlesbarer Betrag: Ziffern, Apostroph als Tausendertrennzeichen
      (1'250), Punkt oder Komma als Dezimaltrennzeichen, höchstens { $max } Nachkommastellen. » ; it-CH « Importo
      illeggibile: cifre, apostrofo delle migliaia (1'250), punto o virgola decimale, al massimo { $max } decimali. » ;
      en-CH « Unreadable amount: digits, apostrophe for thousands (1'250), point or comma for decimals, at most { $max }
      decimals. »
  - à la suite de `supplier-invoices-err-line-not-in-centimes` (fr-CH `:2977`, de-CH `:2927`, it-CH `:2928`, en-CH `:2933`) :
    `supplier-invoices-err-line-unit-price-ambiguous` — fr-CH « Ligne { $line } : prix unitaire ambigu — écrivez 1'250 pour
    mille deux cent cinquante francs, ou 1.25 pour un franc vingt-cinq. » ; `supplier-invoices-err-line-unit-price-unreadable`
    — fr-CH « Ligne { $line } : prix unitaire illisible — chiffres, apostrophe des milliers (1'250), point ou virgule
    décimale, quatre décimales au plus. » ; de-CH, it-CH, en-CH : les textes de `invoice-line-unit-price-*-error` de la
    36-3b (fiche `36-3b-…md:235-247`), « 1.2500 » remplacé par « 1.25 ».
- **D10 — Ce qui ne change pas** : le serveur et l'API (C-e350-19) ; `readJournalAmount`, `isAmbiguousThousands`,
  `isValidAmount`, `parseAmount`, `readableAmount`, `journalAmountPayload`, `journalAmountReader` ; les écrans et clés des
  36-3, 36-3b, 36-2, 35-6 ; les quantités ; les écrans de la 36-18b.

## Critères d'acceptation

1. **AC1 — Les aides partagées** (`balance.test.ts`, `supplier-invoice-form.test.ts`) : `amountEntryPayload` — « 1'250 » →
   `1250`, « 1’250.50 » → `1250.50`, « 1,25 » → `1.25`, « 1250 » → `1250`, « 0.500 » → `0.500`, « 1234.567 » → `1234.567`,
   « 1.2340 » → `1.2340` ; `null` pour « 1.250 », « 12,500 », « 001.250 », « 12'50 », « -5 », « 1e3 », « .5 », « » ;
   `amountEntryProblem` — `ambiguous`, `unreadable`, `null` sur les mêmes cas ; `readableUnambiguousAmount(…, 2)` —
   « 1.2340 » → `1.2340`, « 12.3450 » → `12.3450`, « 1.2500 » → `1.25`, « 1250.0000 » → `1250.00`, « 33.3333 » →
   `33.3333`, et chaque sortie non ambiguë. `unit-price.test.ts` vert **sans modification** ; `balance.test.ts` modifié
   pour le seul retrait de `defaultAmountReader` (D5, V1-9), ses autres attentes inchangées.
2. **AC2 — Soldes de départ** (`opening-balances-page.test.ts`, `opening-balances-totals.test.ts`) : *fixture équilibrée
   sous l'ancienne lecture (V1-3)* — « 1.250 » au débit d'un compte d'actif et « 1.25 » au crédit du report : avant la
   story, *Générer* est actif et envoie `debit: '1.250'` (le rouge) ; après, message ambigu sous la ligne, *Générer*
   désactivé, aucun appel. Mode *Compléter* : « 1.250 » au débit d'un actif (contrepartie calculée) — avant,
   `completeOpeningBalances` reçoit `1.250` ; après, aucun appel. « 1'250 » au débit et au crédit du report → *Générer*
   envoie `debit: '1250'` ; « 12'50 » → message illisible ; totaux : `1250` pour « 1'250 », `0` pour « 1.250 » ; le
   message a un `id` que les deux champs de la ligne nomment dans `aria-describedby`.
3. **AC3 — Prix unitaire fournisseur** (`supplier-invoice-form.test.ts`, `SupplierInvoiceForm.test.ts`) : « 1.250 » →
   message sous la ligne, total vivant sans la ligne, *Enregistrer* → « Ligne 1 : prix unitaire ambigu », `onSubmit` non
   appelé ; « 12'34 » → « … illisible » ; « 1'250 » (quantité 2) → `unitPrice: '1250'`, `quantity: '2'` ; la quantité
   « 1.250 » reste acceptée ; une ligne rouverte au prix `1.2340` se montre `1.2340`, sans message, et repart `1.2340`. Le
   cas `supplier-invoice-form.test.ts:48` (`['1.5', '3.333', '4.9995']`) passe à `'3.3330'`, ce que le Dev Agent Record dit.
4. **AC4 — Montant attendu** : « 1.250 » sans QR → message sous le champ et dans `formError`, `onSubmit` non appelé ;
   « 1'250 » → `expectedPaymentAmount: '1250'` ; vide → `null` ; rouvert `1.2340` → `1.2340` sans message ; scan QR
   `1250.00` → aucun message. Test unitaire de `paymentFields` (V1-4) : « 1.250 » et « 12'34 » →
   `expectedPaymentAmount: null`, « 1'250 » → `'1250'`. Ordre des gardes : échéance antérieure **et** montant attendu
   ambigu → le message de l'échéance ; montant attendu ambigu **et** ligne au prix ambigu → celui du montant attendu.
5. **AC5 — Règlement** (`SettleInvoiceDialog.test.ts`) : « 1.250 » → message ambigu, *Confirmer* sans effet ; « 1'250 »
   (reste dû 2000) → `amount: '1250'` ; « 1,25 » → `'1.25'` ; « 12'34 » → illisible, « 2 décimales au plus » ;
   « 10.0081 » → `invoice-error-amount-scale` ; « 0 » → `invoice-error-amount-positive` ; le pré-rempli (`10.01` pour
   `10.0050`) sans message ; les trois tests contraires réécrits (D8). *V1-1, règle #476 gardée* : « 10.0100 » (reste dû
   `10.0100`) → aucun message, `amount: '10.0100'` ; « 1250.0000 » (reste dû 2000) → aucun message, `amount: '1250.0000'`.
   *V1-6* : sur « 1.250 », le champ porte `aria-invalid="true"` et `aria-describedby` vers le bandeau.
6. **AC6 — Rien de juste n'est refusé** : pour chacun des quatre sites, un test par forme — « 1.25 », « 1250 », « 1'250.50 »,
   « 1’250 », « 1,25 », « 0.50 » — sans message, envoyée normalisée ; *V1-5* : une saisie bordée de blancs («  1250  »)
   au règlement, au prix et au montant attendu fournisseur → sans message, envoyée `1250` (les soldes de départ, qui ne
   rognent pas, la refusent comme aujourd'hui, D4) ; les formes rouvertes de D2 sans message. Seules deviennent refusées la
   forme de la règle et ce que la grammaire du journal ne lit pas — `1e3`, `.5`, `5.`, signe, blanc intérieur, **plus de
   quatre décimales, même nulles** (« 10.01000 », accepté au règlement avant la story : un test constate le refus
   illisible) —, dit à la note de version.
7. **AC7 — E2E** (fichier neuf `frontend/tests/e2e/montants-ambigus.spec.ts`, un `test` par écran : soldes de départ, facture
   fournisseur prix et montant attendu, règlement ; fixtures synthétiques créées par l'API ; société vierge par
   `seedTestState('with-company')` pour les soldes, comme `opening-balances.spec.ts:74-80`) : « 1.250 » → message visible ;
   *V1-7* : l'écouteur `page.on('request')`, armé avant le clic, compte les seules requêtes d'écriture de l'écran, par
   **méthode et route** — `POST /api/v1/opening-balances` et `POST /api/v1/opening-balances/complete`
   (`opening-balances.api.ts:27`, `:38`), `POST /api/v1/supplier-invoices` et `PUT /api/v1/supplier-invoices/{id}`
   (`supplier-invoices.api.ts:57`, `:69`), `POST /api/v1/invoices/{id}/settlements` (`invoices.api.ts:150`) — et en
   attend **zéro** ; le compteur n'est qu'un indice, la preuve est la relecture par `authedApiContext` de l'état d'avant
   (aucune écriture d'ouverture, aucune facture fournisseur neuve, reste dû inchangé). Puis « 1'250 » → le corps
   (`page.waitForRequest`, même filtre) porte `1250`, réponse 2xx, relecture comparée **en valeur**
   (`new Big(lu).eq(1250)`, le serveur rendant `1250.0000`), jamais en chaîne.
8. **AC8 — Textes** : quatre clés × quatre locales (D9) ; `npm run lint-i18n-ownership` vert sans entrée neuve ;
   `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` (`:1020`, 3050 à `fed34a9c7`) **recompté à la source**, sans
   addition (C-e350-18).
9. **AC9 — Manuel** (`docs/manual/fr/user-manual.tex` ; PDF à l'intégration) : une phrase par écran, qui renvoie à la règle
   de *Saisie d'une écriture en partie double* (`:559`), porte la chaîne exacte `refusé comme ambigu`, l'exemple `1.250`,
   la forme `1'250` et la limite de décimales du champ — § *Reprise de comptabilité*, après l'item *Saisissez les soldes*
   (`:1004`) ; § *Les montants* de la facture fournisseur (`:1780-1784`), dont la phrase « Kesh les lit tels quels et les
   enregistre au point » (`:1781`), devenue fausse pour le prix et le montant attendu, est **corrigée** (*V1-8*) ;
   § *Enregistrer* du règlement (`:1443-1445`), avec l'exemple « \texttt{10.000} est refusé comme ambigu : écrivez
   \texttt{10.00} ou \texttt{10'000} » (*V1-8*). *V1-2* — **contrôle de clôture de la story, sur le `.tex`** :
   `/usr/bin/grep -n 'refusé comme ambigu' docs/manual/fr/user-manual.tex` rend une ligne dans chacune des trois sections
   citées (en plus de `:1158`, la 36-3b), et `/usr/bin/grep -n 'tels quels' docs/manual/fr/user-manual.tex` ne rend plus
   la ligne du § *Les montants*. **Contrôle d'intégration seulement** (PDF régénéré après l'union des branches) :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o 'refusé comme ambigu' | wc -l`,
   comparé à sa valeur à l'étape 0 (**1** à `fed34a9c7`, relevé par la validation P1) — `grep -c` sur un flux aplati ne rend
   que 0 ou 1.
10. **AC10 — Rien d'autre ne change** : `git diff <base> --stat` ne nomme ni `crates/` hors `kesh-i18n/locales/`, ni
    `amount-ambiguity.ts`, `JournalEntryForm.svelte`, `VatPurchaseAssistant.svelte`, `TransactionSplitModal.svelte`,
    `InvoiceForm.svelte`, `bank-accounts/`, ni les fichiers de la 36-18b ; `amount-ambiguity.test.ts`, `unit-price.test.ts`,
    `InvoiceForm.test.ts`, `format.test.ts` verts sans modification ; `balance.ts` n'ajoute que les deux aides et des
    doc-comments, et ne retire que `defaultAmountReader` (D5) ; `readJournalAmount` inchangé.
11. **AC11 — Captures** (thème clair, base jetable `kesh_e2e_e3618a`, `kesh-gate-logs/e36-18a/captures/`) : chaque écran à
    « 1.250 », à « 12'34 », et la valeur « 1'250 » enregistrée ; un brouillon fournisseur rouvert au prix `1.2340`.

## Tests à écrire, et la preuve du rouge

Écrits d'abord, sur le code non corrigé, journal `/home/gcorbaz/devel/kesh-gate-logs/e36-18a/rouges.log`. Les aides neuves
se posent d'abord avec la lecture d'aujourd'hui (`amountEntryPayload` → le brut si `isValidAmount`, `null` sinon ;
`readableUnambiguousAmount` → `readableAmount`), pour que les rouges portent sur le comportement.

- **Rouges attendus** : AC1 (« 1.250 » non `null` ; « 1'250 » `null` ; « 1.2340 » → `1.234`) ; AC2 (*Générer* envoie
  `1.250`) ; AC3 (`unitPrice: '1.250'` ; rouvert `1.234`) ; AC4 (`'1.250'`) ; AC5 (`'1.250'` ; « 1'250 » refusé « supérieur
  à zéro ») ; AC7 (chaque requête part avec `1.250`).
- **Verts au départ, et dits** : les formes justes d'AC6 déjà lues (« 1.25 », « 1250 » partout ; « 1,25 » aux soldes et au
  fournisseur) ; le pré-rempli du règlement. Les mutations M13 et M14 vérifient qu'ils rougiraient.
- **E2E rouge** : sources remises à la base par `git restore --source=<base> --worktree -- <fichiers>` **après** commit du
  code, frontend construit, spec rouge écran par écran, sources restaurées (`git restore --worktree` + `touch`), journal
  `e2e-rouges.log`.

**Mutations** (code commité d'abord ; scripts dans `/home/gcorbaz/devel/kesh-gate-logs/e36-18a/mut/` ; restauration
`git restore --worktree -- <fichier>` puis `touch` ; journaux `mut/M*.log`, résumé `mut/resume.log` ; Vitest seul) :

| # | Mutation | Tuée par |
|---|---|---|
| M1 | `amountEntryPayload` rend `payload` aussi pour `ambiguous` | AC1, AC2, AC3, AC5 |
| M2 | `readableUnambiguousAmount` sans le repli `toFixed(4)` | AC1, AC3, AC4 |
| M3 | soldes : `computeBalance(rows)` sans `journalAmountReader` | AC2 |
| M4 | soldes : la charge reprend `r.debit.replace(',', '.')` | AC2 |
| M5 | soldes : la rangée de message retirée | AC2 |
| M6 | totaux : `amount()` reprend `isValidAmount` / `parseAmount` | AC2 |
| M7 | fournisseur : `linesInvalid` reprend `isValidAmount` pour le prix | AC3 |
| M8 | fournisseur : `linesForServer` reprend `decimalForServer(l.unitPrice)` | AC3 |
| M9 | fournisseur : la garde d'envoi par ligne retirée | AC3 |
| M10 | fournisseur : `formLineFromResponse` reprend `readable(…, 2)` | AC3 |
| M11 | montant attendu : la garde d'envoi retirée | AC4 |
| M12 | montant attendu : rouvert par `readableAmount` | AC4 |
| M13 | règlement : `clientError` reprend `Number(amount)` | AC5, AC6 |
| M14 | règlement : `onConfirm` reçoit `amount.trim()` | AC5, AC6 |
| M15 | règlement : `$max` 4 au lieu de 2 | AC5 |
| M16 | règlement : `centimesDigits` compte sur la chaîne de la charge, non sur la forme canonique (V1-1) | AC5 (« 10.0100 », « 1250.0000 ») |
| M17 | règlement : la lecture sans `trim()` (V1-5) | AC6 (« 1250 » bordé de blancs) |
| M18 | `paymentFields` rend la saisie brute pour une saisie non claire (V1-4) | AC4 (test de `paymentFields`) |

Une mutation équivalente se dit au Dev Agent Record.

## Tâches

- [ ] **Étape 0** : partir de `epic/36-0.18.0` **avec la 36-15 fusionnée** ; relire les lignes citées à la tête du
      worktree ; rejouer les motifs d'inventaire (`kesh-gate-logs/e36-18a/etape0-inventaires.log`) ; valeur de départ du
      contrôle PDF (AC9) ; relire les LOW du dernier rapport de validation.
- [ ] Tests d'abord (AC1 à AC6), rouges consignés ; `balance.ts` et `unit-price.ts` (D1) ; `supplier-invoice-form.ts`
      (D2, D6, D7) ; `opening-balances-totals.ts` et la page (D5) ; `SupplierInvoiceForm.svelte` (D6, D7) ;
      `SettleInvoiceDialog.svelte` (D8) ; quatre `messages.ftl` (D9) ; `i18n-keys.test.ts` recompté (AC8).
- [ ] E2E (AC7), rouge d'abord ; manuel (AC9), `.tex` seul, son contrôle `.tex` à la clôture ; captures (AC11) ;
      mutations M1 à M18.
- [ ] Gates (voie 2, `e35-dev-commun.md` § *Quels gates, quand*) : en cours, Vitest ciblé par `.gate-front.lock`,
      `npm run check`, `lint-i18n-ownership` ; **passage en `review`** : bases `kesh_e3618a` remises à neuf (voie 2),
      `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh kesh_ci_e3618a` (voie 1, son verrou), frontend complet, pas d'E2E ;
      **remédiations** : `test-fast.sh --ci` et frontend complet ; **clôture** : `gate-ci` et E2E complet sur
      `kesh_e2e_e3618a` recréée, au dernier commit de code, échecs nommés contre `docs/testing.md` § « Les échecs attendus ».
      Une commande lourde à la fois ; journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e36-18a/`, commit en tête de chacun.
- [ ] Fin du développement : `Status: review`, clé `review` au `sprint-status.yaml`, prompt de revue P1
      (`36-18a-montants-ambigus-livres-review-prompt-p1.md`) qui nomme le manuel, le PDF (à l'intégration) et les captures.

## Note de version

*Rubrique* **Corrigé** (C-e360-11 ; l'orchestrateur l'assemble dans `## [0.18.0] — Non publié`) :

- Les soldes de départ, le prix unitaire et le montant attendu d'une facture fournisseur, et le règlement d'une facture
  lisent désormais un montant comme une écriture : un montant comme « 1.250 » — un point ou une virgule suivis de trois
  chiffres, sans séparateur de milliers avant — n'est plus lu 1,25 franc en silence ; il est refusé comme ambigu, rien ne
  s'enregistre, et le message dit comment l'écrire (`1'250` ou `1.25`). L'apostrophe des milliers et la virgule décimale
  sont acceptées (`1'250`, `1,25`) ; la saisie à exposant (`1e3`), sans chiffre avant ou après le point (`.5`, `5.`) ou
  à plus de quatre décimales, même nulles (`10.01000`), n'est plus lue. Un règlement de « 10.000 » est refusé comme ambigu :
  écrivez `10.00` ou `10'000`. Un prix ou un montant attendu enregistré au millième se rouvre sur quatre décimales
  (`1.2340`) et n'est pas refusé. **Avant la 0.18.0**, un montant tapé avec trois chiffres après le point ou la virgule
  (« 1.250 ») a été lu sans avertissement comme un franc vingt-cinq : relisez vos soldes de départ, les prix et montants
  attendus de vos factures fournisseur et vos règlements saisis ainsi. (refs #1050, refs #1159, refs #1172, refs #1174)

## Dev Notes

### Hors périmètre

- Prix d'article, frais de rappel, montant minimal, filtres : **36-18b** (#1173, #1175, #1176, #1177).
- Les quantités (C-e360-6) ; le serveur et l'API (C-e350-19).
- La convergence des messages des 36-3, 36-3b et 36-2 vers les clés communes : **#1202**.
- Un montant déjà enregistré trop bas avant la 0.18.0 : rien ne permet de le reconnaître ; aucune reprise.

### Splitting — cinq modules

(1) `features/journal-entries` (`balance.ts`) ; (2) `features/invoices` (`unit-price.ts`, `SettleInvoiceDialog.svelte`) ;
(3) `features/supplier-invoices` ; (4) soldes de départ (`features/opening-balances`, `routes/(app)/settings/opening-balances`) ;
(5) `crates/kesh-i18n/locales`. S'y ajoutent le manuel et les tests transverses (`montants-ambigus.spec.ts`,
`i18n-keys.test.ts`). **Cinq : la règle ne s'applique pas — de justesse** (*V1-9*) : le compte range
`features/opening-balances` et `routes/(app)/settings/opening-balances` en un seul module métier, et ne compte ni
`shared/` (un test), ni `docs/manual`, ni `tests/e2e` ; compté par dossier de premier niveau, il en fait sept.

### Fichiers communs avec les autres stories de la phase 2

- `SupplierInvoiceForm.svelte` : nommé par la fiche 36-15 (vocabulaire « compte analytique ») ; **la 36-18a part de la
  36-15 fusionnée**.
- `messages.ftl` ×4, `user-manual.tex` : 36-9, 36-10, 36-15, 36-17, 36-19 y écrivent (`git diff --name-only
  origin/epic/36-0.18.0...story/<clé>`) — clés à côté de leurs blocs (D9), paragraphes distincts ; PDF après l'union.
- `i18n-keys.test.ts` (`sitesTotal`) : 36-17 le touche ; recompté par la branche puis après chaque fusion.
- `frontend/tests/e2e/montants-ambigus.spec.ts` : créé ici, complété par la 36-18b.
- `CHANGELOG.md` : non écrit par la branche (C-e360-11).

### Ce qui doit rester vrai

- Un montant juste ne devient jamais refusé (AC6) ; un montant rouvert jamais (D2).
- Le serveur reste juge ; big.js partout, jamais `parseFloat` ; fixtures synthétiques seulement.

### E2E

Le défaut se voit dans le navigateur et à la frontière HTTP (chaque montant part tel que tapé) : AC7 exigé.

### Références

- Issues #1050, #1159, #1172, #1174, #1202 ; fiche d'origine `36-18-montants-ambigus-autres-ecrans.md` (`split`) ;
  Stories 36-3 (#1051), 36-3b (#1171), 35-6 (#1026), 36-2 ; C-e360-6, C-e360-11, C-e350-18, C-e350-19 ; #878, #687.
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel* ; § *Règle de splitting préventif*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Fiche créée (Opus 5.5) par découpage de la 36-18 (C-36-18-10), à `fed34a9c7` : quatre sites, décisions D1
  à D10 (D1 à D4 et D9 communes aux deux lots), AC1 à AC11, mutations M1 à M15 ; cinq modules ; aucune migration.
  Validation non lancée.
- 2026-11-15 — **Validation P1** (`kesh-gate-logs/e36-18/validation-36-18a-p1.md`) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 8 LOW ;
  les dix remédiés, aucun réfuté. Prémisses relues au code : `SettleInvoiceDialog.svelte:69`
  `const canon = new Big(raw.trim()).toFixed();` (V1-1) ; `/usr/bin/grep -c "refusé comme ambigu" docs/manual/fr/user-manual.tex`
  → `1` (V1-2) ; `invoices.api.ts:150` `` apiClient.post(`/api/v1/invoices/${id}/settlements`, req) ``,
  `opening-balances.api.ts:27`, `:38`, `supplier-invoices.api.ts:57`, `:69` (V1-7) ; `git grep -n "defaultAmountReader\|computeBalance(\|classifyLine("`
  hors `balance.ts` : `JournalEntryForm.svelte:179-180`, `opening-balances/+page.svelte:135` et `balance.test.ts` seuls
  (V1-9). Remèdes : V1-1 AC5 garde #476, M16 ; V1-2 contrôle `.tex` à la clôture, PDF à l'intégration par `grep -o | wc -l` ;
  V1-3 fixture d'AC2 équilibrée sous l'ancienne lecture ; V1-4 contrat de `paymentFields` (`null`) et ordre des gardes,
  M18 ; V1-5 blancs et cinq décimales, M17 ; V1-6 `aria-invalid` et `aria-describedby`, règlement compris ; V1-7 requêtes
  comptées par méthode et route, relecture en valeur ; V1-8 `:1781` corrigée, « 10.000 » au manuel ; V1-9
  `defaultAmountReader` retiré (mort en production), décompte nuancé ; V1-10 note de version : relire les montants à
  trois décimales saisis avant la 0.18.0. Propagé à la 36-18b. Passe 2 à lancer.
