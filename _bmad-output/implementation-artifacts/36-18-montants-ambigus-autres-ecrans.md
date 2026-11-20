# Story 36-18 : les autres montants « 1.250 » — la règle de la 36-3 sur les sept écrans qui écrivent un montant (#1050, #1159, #1172 à #1177)

Status: split

*Fiche rédigée le 2026-11-15 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-18`, branche
`story/36-18-montants-ambigus-autres-ecrans`, partie de `epic/36-0.18.0` (`fed34a9c7`), par le workflow
`bmad-create-story`, prompts `kesh-gate-logs/prompts/e36-fiche.md` et `e36-p2-commun.md` (qui prime). Release **0.18.0**,
phase 2, rang 18 du fichier d'epic (décision **D6** de Guy du 2026-11-15). **Aucune migration**, aucun changement serveur
ni d'API. Code lu à la tête du worktree (`fed34a9c7`). Voie de développement : **voie 2**. Registre :
`e36-choix-autonomes-36-18.md` (C-36-18-1 à C-36-18-9). Sorties brutes des inventaires :
`/home/gcorbaz/devel/kesh-gate-logs/e36-18/fiche-inventaires.log`. Validation non lancée.*

⚠️ **Sept modules de code : au-delà de la règle de découpage.** La fiche est écrite entière ; chaque critère porte son lot
(**A** ou **B**) ; le découpage est **proposé** à l'orchestrateur (§ *Dev Notes*, *Splitting*), non écrit (C-36-18-9).

## Story

En tant que titulaire qui tient ses livres réels dans Kesh, je veux que tout montant que je tape — solde de départ, prix
et montant attendu d'une facture fournisseur, règlement d'une facture, prix d'un article, frais de rappel, montant minimal
d'une facture, et les bornes du filtre des écritures — se lise comme celui d'une écriture : « 1.250 » refusé comme ambigu
avec un message qui dit comment l'écrire, « 1'250 » accepté pour mille deux cent cinquante, et rien de ce que j'écrivais
juste jusqu'ici (« 1.25 », « 1250 », « 1'250.50 », un montant rouvert) refusé à tort, afin qu'aucun montant ne s'enregistre
au millième de ce que je voulais sans que je le voie.

## Ce qui se lit au code (`fed34a9c7`)

### La règle, déjà écrite par la 36-3 et la 36-3b — réemployée, non réécrite

- **Le prédicat** — `frontend/src/lib/shared/utils/amount-ambiguity.ts:38` `export function isAmbiguousThousands(parts: AmountParts): boolean {`,
  `:39-43` : `parts.fraction.length === 3 && !parts.grouped && /^0*[1-9][0-9]{0,2}$/.test(parts.integerDigits)`.
- **Le lecteur** — `frontend/src/lib/features/journal-entries/balance.ts:110` `export function readJournalAmount(raw: string): JournalAmountReading {`,
  grammaire `:90` `const JOURNAL_AMOUNT_RE = /^([1-9]\d{0,2}(?:['’]\d{3})+|\d{1,15})(?:[.,](\d{1,4}))?$/;`, sans rognage
  (`:111` `if (raw === '') return { kind: 'empty' };`), `:119` `if (isAmbiguousThousands({ integerDigits, fraction, grouped })) return { kind: 'ambiguous' };`,
  `:120` `const payload = fraction === '' ? integerDigits : \`${integerDigits}.${fraction}\`;`.
- **Ses aides existantes** — `balance.ts:128` `lineAmountProblem(debit, credit)` (le message d'une ligne débit/crédit) ;
  `:143` `journalAmountPayload(raw)` (`'0'` pour le vide, `:145`) ; `:157` `journalAmountReader` (`Big(0)` pour le vide,
  `null` hors règle, `:159-160`) ; `:153` `defaultAmountReader` = `isValidAmount(raw) ? parseAmount(raw) : null` (« la
  lecture d'avant la story, que gardent les soldes de départ (#1050) », `:152`).
- **Les aides de la 36-3b, génériques sous un nom de prix** — `frontend/src/lib/features/invoices/unit-price.ts:25-28`
  `unitPricePayload` (`r.kind === 'clear' ? r.payload : null`), `:34-37` `unitPriceProblem`, `:46-50`
  `unitPriceFieldValue` : `const lisible = readableAmount(raw, 2);` … `return new Big(raw).toFixed(4);`. `:17`
  `import { readableAmount } from '$lib/features/supplier-invoices/supplier-invoice-form';` — d'où le cycle qu'un import
  inverse créerait (`:10-12`).
- **La forme rouverte** — `frontend/src/lib/features/supplier-invoices/supplier-invoice-form.ts:214-224` `readableAmount`
  (« 1.2340 » → `1.234`, relevé de la 36-3b, fiche `36-3b-…md:73-76`).
- **Les messages de la 36-3** — `crates/kesh-i18n/locales/fr-CH/messages.ftl:313-314` (même rang dans les quatre
  locales) : `journal-entry-form-amount-ambiguous` conseille « 1.2340 pour un franc vingt-trois et quatre dixièmes de
  centime », `journal-entry-form-amount-unreadable` dit « quatre décimales au plus » — **faux pour un champ au centime**
  (règlement, frais, montant minimal). Les clés `common-*` sont globales :
  `frontend/scripts/lint-i18n-ownership.js:16` `const GLOBAL_NAMESPACES = ['error', 'tooltip', 'common', 'mode', 'shortcut', 'demo'];`.

### Les huit sites, un par issue

**1. Soldes de départ (#1050)** — `frontend/src/routes/(app)/settings/opening-balances/+page.svelte` (759 lignes) :
- `:135` `const balance = $derived(computeBalance(rows));` — lecteur par défaut (`defaultAmountReader`) ;
- `:147-152` et `:163-169` (lignes envoyées) : `if (!isValidAmount(r.debit) || !isValidAmount(r.credit)) return true;`
  `return parseAmount(r.debit).gt(0) || parseAmount(r.credit).gt(0);` ; `:170-172` `complementInvalid` par `isValidAmount` ;
- `:199-200` (générer) et `:245-246` (compléter) : `debit: r.debit === '' ? '0' : r.debit.replace(',', '.'),` — « 1.250 »
  part `1.250`, que le serveur lit 1,25 (`crates/kesh-api/src/routes/opening_balances.rs:363` `let debit = Decimal::from_str(&line.debit)`) ;
- `:490`, `:505`, `:621`, `:634` `aria-invalid={!isValidAmount(row.debit)}` — aucun message sous le champ ;
- `frontend/src/lib/features/opening-balances/opening-balances-totals.ts:39-41`
  `function amount(raw: string): Big { return isValidAmount(raw) ? parseAmount(raw) : new Big(0); }` (totaux actif/passif,
  avertissement du report, contrepartie du complément).
- Aucun montant venu du serveur ne s'y rouvre (la grille part vide ; l'écran se verrouille après génération).
- Pas de rognage aujourd'hui (`isValidAmount(" 12")` est faux) : il n'en aura pas (C-36-18-4).

**2. Prix unitaire d'une facture fournisseur (#1159)** — `supplier-invoice-form.ts` :
- `:146-155` `linesInvalid` : `!isValidAmount(l.unitPrice.trim(), { allowEmpty: false }),` (`:153`) ;
- `:72-90` `sumTtc`, `:117-125` `lineNotInCentimes` : `parseAmount(l.unitPrice.trim())` (`:78`, `:120`) — « 1.250 » × 1 =
  1,25, au centime : **le contrôle du centime de #687 ne l'arrête pas** (« 1.234 » seul l'est) ;
- `:163-173` `linesForServer` : `unitPrice: decimalForServer(l.unitPrice),` (`:169`) ;
- `:232-252` `formLineFromResponse` : `unitPrice: readable(line.unitPrice, 2),` (`:248`) — « 1.2340 » se rouvrirait `1.234`,
  **refusé à tort** une fois la règle posée ;
- la quantité (`:63`, `:152`, `:168`) lit légitimement « 1.250 » comme 1,25 unité : **hors de la règle** (C-e360-6).
- `SupplierInvoiceForm.svelte` (890 lignes) : le champ `:798-804` (`data-testid="supplier-invoice-line-unit-price"`,
  `bind:value={line.unitPrice}`), sans message ; le refus à l'envoi `:372-377` (`supplier-invoices-err-lines`,
  « Chaque ligne requiert une description, un montant et un compte de charge. ») puis `:382-388` (centime).
- Serveur : `crates/kesh-api/src/routes/supplier_invoices.rs:539` `if !scale_within(&line.unit_price.normalize(), MAX_DECIMAL_SCALE) {`
  — « 1.250 » passe, lu 1,25.

**3. Montant de paiement attendu d'une facture fournisseur (#1172)** — `supplier-invoice-form.ts:275-284` `paymentFields` :
`expected === '' ? null : isValidAmount(expected) ? decimalForServer(expected) : expected,` (`:283`) ; champ
`SupplierInvoiceForm.svelte:749-754` (`data-testid="supplier-invoice-expected-amount"`) ; rouvert `:184-185`
`init.expectedPaymentAmount == null ? '' : readableAmount(init.expectedPaymentAmount, 2),` (colonne `DECIMAL(19,4)`,
`crates/kesh-db/migrations/20260628000001_supplier_invoices.sql:40`) — même risque de refus à tort que le prix ; pré-rempli
par un scan QR `:323` `fExpectedAmount = p.expectedAmount;` (montant QR, deux décimales au plus : jamais ambigu). Envoyé
seulement sans QR (`:399-404`, `const req = qrLocked`).

**4. Règlement d'une facture client (#1174)** — `frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte` (327 lignes) :
- `:166-169` `const n = Number(amount);` `if (!amount || !Number.isFinite(n) || n <= 0) {` → « Le montant doit être
  supérieur à zéro » — « 1.250 » vaut 1,25 ; « 1'250 » et « 1,25 » rendent `NaN`, d'où ce message, faux ;
- `:67-76` `centimesDigits` sur la forme canonique de big.js : « 1.250 » → `1.25`, deux décimales, **passe** ;
- `:184` `new Big(amount.trim()).gt(dueToCentime(amountDue))` — un montant mille fois trop bas passe pour un règlement
  partiel ; `:205` `amount: amount.trim(),` part tel quel ;
- pré-rempli `:92` `amount = amountDue !== null ? dueToCentime(amountDue).toFixed(2) : '';` (deux décimales : jamais ambigu) ;
- champ `:294` `<Input id="settle-amount" type="text" inputmode="decimal" bind:value={amount} />`, refus dans le bandeau
  `:304-307` (`{#if clientError}`) ;
- serveur `crates/kesh-api/src/routes/invoices.rs:1139-1147` : `scale_within(&self.amount.normalize(), 2)` — « 10.000 est un
  montant au centime », dit son commentaire ; « 1.250 » passe, lu 1,25.
- **Tests existants contraires à la règle** — `SettleInvoiceDialog.test.ts:70-75` (« 10.008 » refusé pour son échelle),
  `:77-81` (« 1e-3 »), `:83-89` (« 10.000 » accepté et envoyé `'10.000'`) : sous D6, « 10.008 » et « 10.000 » sont ambigus,
  « 1e-3 » illisible (C-36-18-5).

**5. Prix d'un article (#1173)** — `frontend/src/lib/features/products/product-helpers.ts` :
- `:53` `const PRICE_RE = /^(0|[1-9][0-9]{0,14})(\.[0-9]{1,4})?$/;`, `:71-73` `normalizePriceInput` (`raw.trim().replace(',', '.')`),
  `:82-87` `classifyPriceInput` (`empty` / `negative` / `invalid` / `ok`) — « 1.250 » `ok`, « 1,250 » normalisé `1.250` `ok` ;
- `:61-65` `isValidPrice` : **aucun appelant de production** (`git grep -n "isValidPrice" -- frontend/src` : le test et deux
  doc-comments, `:59`, `:80`) ;
- page `frontend/src/routes/(app)/products/+page.svelte` : `:327-334` messages (`product-error-price-required`,
  `-negative`, `-invalid`), `:368` `unitPrice: normalizePriceInput(formPrice),`, champ `:653-662` ;
- rouvert `:306-308` `const fixed = new Big(p.unitPrice).toFixed(4);` `formPrice = fixed.replace(/(\.\d{2})0+$/, '$1');` —
  la regex n'ôte que des zéros qui suivent la deuxième décimale : la sortie a **deux ou quatre** décimales, jamais trois
  (« 1.2340 » reste `1.2340`, « 1.2500 » → `1.25`, « 0.1230 » reste `0.1230`) — **jamais ambiguë**, à prouver par test (AC6) ;
- serveur `crates/kesh-api/src/routes/products.rs:210` `if !scale_within(&unit_price, MAX_DECIMAL_SCALE) {` — « 1.250 »
  passe, lu 1,25, et ce prix est repris par chaque ligne de facture qui emploie l'article.

**6. Frais de rappel (#1175)** — `frontend/src/routes/(app)/settings/dunning/+page.svelte` (306 lignes) : `:104`
`if (fFee.trim() === '' || Number.isNaN(Number(fFee))) {` ; `:111`, `:116` `feeAmount: fFee` envoyé **brut** ; champ `:284` ;
rouvert `:88` `fFee = level.feeAmount;` (colonne `DECIMAL(7,2)`, `crates/kesh-db/migrations/20260714000001_dunning_config.sql:32` :
deux décimales, jamais ambigu). ⚠️ **L'issue dit plus que le code** : le serveur refuse « 1.250 »
(`crates/kesh-api/src/routes/dunning_levels.rs:82-86` `if !scale_within(fee, 2) {` … « Les frais de rappel ne peuvent avoir
plus de 2 décimales. », sans `normalize()` ; `limits.rs:31-32` `value.scale() <= max_scale`). Le défaut réel : ce message
invite à écrire `1.25` pour mille deux cent cinquante, et « 1'250 » est refusé (`Number("1'250")` est `NaN`) ;
`Number(" 1.25 ")` admet les blancs, et le brut part avec eux (C-36-18-6). **Aucun test de composant** pour cette page.

**7. Montant minimal d'une facture (#1176)** — `frontend/src/routes/(app)/settings/invoicing/+page.svelte` : `:276`
`minimumInvoiceAmount: minimumAmount.trim() === '' ? null : minimumAmount.trim(),` — envoyé tel quel ; serveur
`crates/kesh-api/src/routes/company_invoice_settings.rs:489-496` `scale_within(&min.normalize(), 2)` : « 1.250 » normalisé
`1.25`, **accepté** ; champ `:682-688`, sans message ; rouvert `:213` `minimumAmount = s.minimumInvoiceAmount ?? '';` et
`:307` après conflit (colonne `DECIMAL(19,4)`, `20261001000002_invoice_minimum_amount.sql:14` : quatre décimales, jamais
ambigu — la forme exacte se relève à l'étape 0, AC8).

**8. Filtres de montant de la liste des écritures (#1177, P4)** — `frontend/src/routes/(app)/journal-entries/+page.svelte` :
`:80-81` `amountMin: amountMin || undefined,` envoyé brut (et dans l'URL, `:179-185` par `serializeQuery`,
`frontend/src/lib/features/journal-entries/query-helpers.ts:29-34`) ; initialisé depuis l'URL `:146-147` ; champs `:398-418`.
Le serveur lit « 1.250 » 1,25 et refuse « 1'250 » ou « 1,25 » en 400 (`crates/kesh-api/src/services/ecritures.rs:96-101`
`Decimal::from_str(s)`), que la page tait (`:111-113` `entries = []; total = 0;`). Rien n'est écrit : seul le filtre est faux.

### Inventaire par le symptôme

Le symptôme : « un montant tapé est lu avec le point (ou la virgule) pour séparateur décimal, sans contrôle d'ambiguïté ».
Aucun détecteur par exécution praticable (il faudrait un E2E par écran : c'est ce qu'exige AC11 *après* le correctif).
Détecteur par lecture, parti du **champ** (motifs 1 à 3), puis du **lecteur** (motifs 4 à 6), sorties complètes au journal
`kesh-gate-logs/e36-18/fiche-inventaires.log` :

1. `git grep -ln 'inputmode="decimal"' -- frontend/src` → 14 fichiers : `components/invoices/InvoiceForm.svelte` (+ son
   test), `bank-import/ManualStatementForm.svelte`, `invoices/SettleInvoiceDialog.svelte`,
   `journal-entries/JournalEntryForm.svelte`, `journal-entries/VatPurchaseAssistant.svelte`,
   `reconciliation/TransactionSplitModal.svelte`, `supplier-invoices/SupplierInvoiceForm.svelte`, et les routes
   `bank-accounts`, `journal-entries`, `products`, `settings/dunning`, `settings/invoicing`, `settings/opening-balances`.
2. `git grep -n 'type="number"' -- 'frontend/src/**/*.svelte'` → `AutoBackupSection.svelte:361`, `BankProfileForm.svelte:226`,
   `:287-319` (indices de colonnes), `RuleFormModal.svelte:223`, `audit-log/+page.svelte:297`, `settings/dunning/+page.svelte:203`,
   `:277` (jours), `settings/vat-rates/+page.svelte:287`, `:331` (taux de TVA).
3. `git grep -nE "bind:value=\{[^}]*(…Amount|Price|Fee|Balance|Debit|Credit|Minimum|Rate…)" -- 'frontend/src/**/*.svelte'`
   → les mêmes champs, plus `RuleFormModal.svelte:190` (`matchValue`, texte). ⚠️ Ce motif a **manqué** `bind:value={fFee}`
   (frais de rappel) : les alternatives à `\b` ne sont pas reconnues par `git grep -E`. Le motif 1 le couvre.
4. `git grep -n "readJournalAmount\|journalAmountPayload\|isAmbiguousThousands\|unitPricePayload\|journalAmountReader" -- frontend/src ':!*.test.ts'`
   → les sites **déjà corrigés** : `InvoiceForm.svelte`, `bank-accounts/format.ts:320`, `JournalEntryForm.svelte`,
   `vat-purchase.ts:119`, `TransactionSplitModal.svelte:149`, `unit-price.ts`.
5. `git grep -n "isValidAmount(\|parseAmount(\|decimalForServer(\|defaultAmountReader\|computeBalance(\|classifyLine(" -- frontend/src ':!*.test.ts'`
   → `manual-statement.ts:118`, `:254`, `ManualStatementForm.svelte:317`, `vat-purchase.ts:40-94` (sur une charge déjà lue),
   `opening-balances-totals.ts:40`, `supplier-invoice-form.ts:55-283`, `settings/opening-balances/+page.svelte:135-634`.
6. `git grep -n "replace(','\|replace(/,/" -- frontend/src ':!*.test.ts'` → `balance.ts:71`, `product-helpers.ts:72`,
   `settings/opening-balances/+page.svelte:199-200`, `:245-246`.
7. Les corps de requête qui portent un montant (`awk` sur les `interface …Request|Body` de `frontend/src/**/*.ts`, au
   journal) : rien hors des sites ci-dessous — `PaySupplierInvoiceRequest` (`supplier-invoices.types.ts:195-200`) et
   `WriteOffInvoiceRequest` (`invoices.types.ts:320-324`) ne portent **aucun** montant ; `expectedAmountDue`,
   `expectedFees` des rappels (`reminders.types.ts`) sont calculés, non tapés.

**Sonde positive** : le motif 1 rend `SettleInvoiceDialog.svelte` et `settings/dunning/+page.svelte`, sites connus du
défaut ; le motif 4 rend `JournalEntryForm.svelte`, site corrigé. **Angles morts écrits** : un champ de montant sans
`inputmode="decimal"` ni `type="number"` ni nom de montant ; un montant lu ailleurs qu'un champ (CSV, CAMT, QR : formats
machine, hors objet).

| # | Site (ligne lue) | Même défaut ? | Décision |
|---|---|---|---|
| 1 | Soldes de départ, `settings/opening-balances/+page.svelte:135`, `:199-200`, `:245-246` ; `opening-balances-totals.ts:40` | oui | **périmètre, lot A** (#1050) |
| 2 | Prix unitaire fournisseur, `supplier-invoice-form.ts:153`, `:169`, `:248` | oui | **périmètre, lot A** (#1159) |
| 3 | Montant attendu fournisseur, `supplier-invoice-form.ts:283` ; `SupplierInvoiceForm.svelte:185` | oui | **périmètre, lot A** (#1172) |
| 4 | Règlement d'une facture, `SettleInvoiceDialog.svelte:166-205` | oui | **périmètre, lot A** (#1174) |
| 5 | Prix d'un article, `product-helpers.ts:53`, `:72` ; `products/+page.svelte:368` | oui | **périmètre, lot B** (#1173) |
| 6 | Frais de rappel, `settings/dunning/+page.svelte:104`, `:111`, `:116` | en partie (le serveur refuse « 1.250 ») | **périmètre, lot B** (#1175, C-36-18-6) |
| 7 | Montant minimal, `settings/invoicing/+page.svelte:276` | oui | **périmètre, lot B** (#1176) |
| 8 | Filtres de la liste, `journal-entries/+page.svelte:80-81` | oui, sans écriture | **périmètre, lot B, en dernier** (#1177) |
| 9 | Quantités (fournisseur `supplier-invoice-form.ts:63`, `:152`, `:168` ; client `InvoiceForm.svelte`) | non : « 1.250 » unité est une lecture voulue | hors de la règle (C-e360-6) |
| 10 | Taux de TVA, `settings/vat-rates/+page.svelte:287`, `:331` (`type="number"`, 0 à 100) | non : un pourcentage, lu par le navigateur, sans milliers | hors objet |
| 11 | Relevé manuel, `manual-statement.ts:118`, `ManualStatementForm.svelte:317` | non : deux décimales au plus, « 1.250 » refusé | — |
| 12 | Écriture manuelle, assistant TVA achat, éclatement, solde d'ouverture bancaire, prix d'une facture client | non : corrigés (36-3, 36-2, 35-6, 36-3b) | — |
| 13 | `AutoBackupSection.svelte:361`, `BankProfileForm.svelte`, `RuleFormModal.svelte:190`, `:223`, `audit-log/+page.svelte:297`, `dunning/+page.svelte:203`, `:277` | non : entiers ou texte | — |

Aucun site hors des huit issues : **aucune issue neuve** pour l'inventaire.

## Décisions

- **D1 — Une lecture, celle de la 36-3 ; deux aides rendues génériques** (C-36-18-1). Dans `balance.ts`, à côté de
  `readJournalAmount` : `amountEntryPayload(raw: string): string | null` (le `payload` d'une lecture `clear`, `null`
  sinon, vide compris) et `amountEntryProblem(raw: string): 'ambiguous' | 'unreadable' | null` (`null` pour clair ou
  vide) — les corps de `unitPricePayload` et `unitPriceProblem`, **déplacés** ; `unit-price.ts` les ré-exporte sous ses
  noms (`export { amountEntryPayload as unitPricePayload, amountEntryProblem as unitPriceProblem } from …`) :
  `InvoiceForm.svelte` et `unit-price.test.ts` ne changent pas. Ni `readJournalAmount`, ni `isAmbiguousThousands`, ni
  `JOURNAL_AMOUNT_RE` ne changent. Aucune grammaire propre à un écran.
  *Écartées* : (a) importer les noms de prix depuis sept écrans ; (b) un module sous `shared/utils/` qui importerait
  `features/` ; (c) une grammaire par écran (C-36-18-1).
- **D2 — Un montant du serveur ne se rouvre jamais sous une forme ambiguë** (C-36-18-2). `supplier-invoice-form.ts`,
  à côté de `readableAmount` : `readableUnambiguousAmount(raw: string, minDecimals: number): string` =
  `readableAmount(raw, minDecimals)`, sauf si `readJournalAmount` de ce résultat est `ambiguous` : alors
  `new Big(raw).toFixed(4)` ; valeur illisible rendue telle quelle. `unitPriceFieldValue` lui délègue
  (`readableUnambiguousAmount(raw, 2)`), même sortie (`unit-price.test.ts` vert sans modification). Appelée à
  `formLineFromResponse` (prix, `:248`) et `SupplierInvoiceForm.svelte:185` (montant attendu). Les autres sites n'en ont
  pas besoin, ce que les tests de AC2 à AC9 établissent : grille vide (soldes), `toFixed(2)` (règlement), deux ou quatre
  décimales (article), `DECIMAL(7,2)` (frais), quatre décimales brutes (minimum), valeur de l'URL telle quelle (filtres).
  *Écartées* : recopier la logique (DRY) ; déplacer `readableAmount` (six importateurs).
- **D3 — Deux messages communs, et le nombre de décimales en paramètre** (C-36-18-3). Sous chaque champ fautif (bordure
  `border-destructive`, `aria-invalid`) : `common-amount-ambiguous` ou `common-amount-unreadable` (`{ $max }` : **4** pour
  les soldes, le prix et le montant attendu fournisseur, le prix d'article et les filtres ; **2** pour le règlement, les
  frais et le montant minimal). Chaque écran garde ses autres refus (vide, négatif, échelle, reste dû, plafond serveur).
  *Écartées* : réemployer `journal-entry-form-amount-*` (texte faux au centime, entrées `KNOWN_VIOLATIONS`) ; une paire
  par écran.
- **D4 — Chaque écran rogne là où il rogne aujourd'hui** (C-36-18-4) : la saisie rognée (`raw.trim()`) est lue au
  règlement, au prix et au montant attendu fournisseur, au prix d'article, aux frais, au minimum et aux filtres ; les
  soldes de départ lisent la saisie brute, comme aujourd'hui.
- **D5 — Soldes de départ (#1050, lot A)** : `computeBalance(rows, journalAmountReader)` ; les filtres `:147-152`,
  `:163-169`, `complementInvalid` et les quatre `aria-invalid` lisent par `journalAmountReader` (`null` = hors règle) ; la
  charge `:199-200`, `:245-246` devient `journalAmountPayload(r.debit) ?? …` (jamais `null` : `canGenerate` exige
  `isBalanced`, faux dès qu'un montant est hors règle ; `canSubmitComplement` exige `!complementInvalid`) ;
  `opening-balances-totals.ts` `amount(raw)` = `journalAmountReader(raw) ?? new Big(0)`. Sous chaque ligne fautive, une
  rangée `<tr><td colspan=…>` avec le message de `lineAmountProblem(row.debit, row.credit)` (D3, `$max` 4),
  `data-testid="opening-balances-amount-ambiguous"` / `-unreadable`, dans les deux grilles. `defaultAmountReader` perd
  son dernier appelant de production : il reste comme défaut de `computeBalance`/`classifyLine` (tests de la 36-3) et son
  doc-comment cesse de nommer les soldes de départ.
- **D6 — Facture fournisseur, prix unitaire (#1159, lot A)** : `linesInvalid` juge le prix par
  `amountEntryPayload(l.unitPrice.trim()) === null` (le vide reste refusé) ; `sumTtc` et `lineNotInCentimes` lisent le prix
  par la même charge (`null` → ligne ignorée, `null` rendu) ; `linesForServer` envoie la charge ; la quantité garde
  `isValidAmount` / `decimalForServer`. Sous la ligne : le message de `amountEntryProblem` (D3, `$max` 4),
  `data-testid="supplier-invoice-line-unit-price-ambiguous"` / `-unreadable`. À l'envoi, **avant** `linesInvalid`, la
  première ligne au prix ambigu ou illisible arrête l'envoi : `supplier-invoices-err-line-unit-price-ambiguous` ou
  `-unreadable` (`{ $line }`, rang 1-based). Rouvert : D2.
- **D7 — Facture fournisseur, montant attendu (#1172, lot A)** : sans QR (`!qrLocked`), une saisie ambiguë ou illisible
  arrête l'envoi (message commun, `$max` 4, dans `formError` et sous le champ,
  `data-testid="supplier-invoice-expected-amount-ambiguous"` / `-unreadable`) ; `paymentFields` envoie la charge d'une
  saisie claire (« 1'250 » → `1250`). Aujourd'hui une saisie illisible part et le serveur la refuse (`:280-281`) : elle
  est désormais refusée avant. Rouvert : D2. Le montant d'un scan QR (deux décimales) n'est pas touché.
- **D8 — Règlement d'une facture client (#1174, lot A)** : `clientError`, après les dates, lit `amount.trim()` : vide →
  `invoice-error-amount-positive` (inchangé) ; `ambiguous` / `unreadable` → message commun (`$max` 2) ; `clear` : valeur
  ≤ 0 → `invoice-error-amount-positive` ; `centimesDigits(payload) > 2` → `invoice-error-amount-scale` ; au-delà du reste
  dû → `invoice-error-amount-over-due` (comparé à la valeur lue). `onConfirm` reçoit `amount: payload`. « 1,25 » et
  « 1'250 » deviennent acceptés ; « 10.000 » et « 10.008 » sont **ambigus**, « 1e-3 » illisible : les trois tests
  existants (`SettleInvoiceDialog.test.ts:70-89`) sont réécrits pour le constater, l'échelle se prouvant sur « 10.0081 »
  (C-36-18-5).
- **D9 — Prix d'un article (#1173, lot B)** : `classifyPriceInput` lit la saisie rognée : vide → `empty` ; signe → `negative`
  (avant la lecture, message inchangé) ; puis `readJournalAmount` : `ambiguous` → `ambiguous` (neuf, message commun
  `$max` 4) ; `unreadable` → `invalid` (`product-error-price-invalid`, inchangé) ; `clear` → `ok`. `normalizePriceInput`
  rend la charge (« 1'250 » → `1250`, « 10,50 » → `10.50`). `isValidPrice` et `PRICE_RE`, sans appelant de production,
  sont retirés avec leurs tests (C-36-18-8) ; « 007.50 » et « 01 » deviennent acceptés. Rouvert : inchangé (`:306-308`),
  jamais ambigu (AC6).
- **D10 — Frais de rappel (#1175, lot B)** : `submitForm` lit `fFee.trim()` : vide → `dunning-form-error-fee` (inchangé) ;
  `ambiguous` / `unreadable` → message commun (`$max` 2) dans `formError` ; `clear` → `feeAmount: payload`. Bordure et
  `aria-invalid` sur le champ tant qu'il est fautif. Les bornes 0..10'000 et l'échelle restent au serveur.
- **D11 — Montant minimal (#1176, lot B)** : vide → `null` (inchangé) ; `ambiguous` / `unreadable` → message sous le champ
  (`data-testid="settings-minimum-amount-ambiguous"` / `-unreadable`, `$max` 2) et l'enregistrement s'arrête par
  `notifyError` du même texte ; `clear` → `minimumInvoiceAmount: payload`. Le reste (positif, au centime, plafond) au serveur.
- **D12 — Filtres de la liste (#1177, lot B, en dernier)** (C-36-18-7) : `buildQuery` envoie la charge d'une borne claire
  (requête et URL, « 1'250 » → `amountMin=1250`) et omet une borne ambiguë ou illisible ; sous le champ, le message commun
  (`$max` 4) suivi de `journal-entries-filter-amount-not-applied`, `data-testid="journal-entries-filter-amount-min-problem"`
  / `-max-problem`. Une URL portant `amountMin=1.250` (signet d'avant la story) affiche le message et n'applique pas la borne.
- **D13 — Textes** (quatre locales, jamais en fin de fichier) :
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
  - à la suite de `journal-entries-filter-amount-max` (fr-CH `:408`, autres `:414`) : `journal-entries-filter-amount-not-applied`
    — fr-CH « Ce filtre n'est pas appliqué tant que le montant n'est pas corrigé. » ; de-CH « Dieser Filter wird erst
    angewendet, wenn der Betrag korrigiert ist. » ; it-CH « Questo filtro non è applicato finché l'importo non è
    corretto. » ; en-CH « This filter is not applied until the amount is corrected. »
  - Les fallbacks de `i18nMsg` recopient le texte fr-CH à l'identique.
- **D14 — Ce qui ne change pas** : le serveur et l'API (C-e350-19) ; `readJournalAmount`, `isAmbiguousThousands`,
  `isValidAmount`, `parseAmount`, `readableAmount`, `journalAmountPayload`, `journalAmountReader` ; l'écriture manuelle,
  l'assistant TVA, l'éclatement, la facture client (36-3, 36-3b, 36-2) et leurs clés ; les quantités ; le relevé manuel ;
  le solde d'ouverture bancaire (35-6).

## Critères d'acceptation

*Lot A : AC1 à AC5 ; lot B : AC6 à AC9 ; AC10 à AC15 valent pour chaque lot, sur son périmètre.*

1. **AC1 — Les aides partagées** (`balance.test.ts`, `supplier-invoice-form.test.ts`) : `amountEntryPayload` — « 1'250 »
   → `1250`, « 1’250.50 » → `1250.50`, « 1,25 » → `1.25`, « 1250 » → `1250`, « 0.500 » → `0.500`, « 1234.567 » →
   `1234.567`, « 1.2340 » → `1.2340` ; `null` pour « 1.250 », « 12,500 », « 001.250 », « 12'50 », « -5 », « 1e3 », « .5 »,
   « » ; `amountEntryProblem` — `ambiguous`, `unreadable` et `null` sur les mêmes cas ; `readableUnambiguousAmount(…, 2)` —
   « 1.2340 » → `1.2340`, « 12.3450 » → `12.3450`, « 1.2500 » → `1.25`, « 1250.0000 » → `1250.00`, « 33.3333 » →
   `33.3333`, et pour chaque sortie `readJournalAmount(sortie).kind !== 'ambiguous'`. `unit-price.test.ts` et `balance.test.ts`
   d'avant la story verts **sans modification**.
2. **AC2 — Soldes de départ** (`opening-balances-page.test.ts`, `opening-balances-totals.test.ts`) : « 1.250 » au débit d'un
   compte → message ambigu sous la ligne, bouton *Générer* désactivé, aucun appel ; « 1'250 » au débit et « 1'250 » au
   crédit du report → *Générer* envoie `debit: '1250'` ; même chose en mode *Compléter* (`completeOpeningBalances`
   reçoit `1250`) ; « 12'50 » → message illisible ; les totaux actif/passif lisent `1250` pour « 1'250 » et `0` pour « 1.250 ».
   « 1.25 », « 1250 », « 1'250.50 » et « 0.500 » : aucun message, envoyés `1.25`, `1250`, `1250.50`, `0.500`.
3. **AC3 — Prix unitaire fournisseur** (`supplier-invoice-form.test.ts`, `SupplierInvoiceForm.test.ts`) : « 1.250 » → message
   sous la ligne, total vivant sans la ligne, *Enregistrer* → « Ligne 1 : prix unitaire ambigu », `onSubmit` non appelé ;
   « 12'34 » → « … illisible » ; « 1'250 » (quantité 2) → envoyé `unitPrice: '1250'`, `quantity: '2'` ; la quantité
   « 1.250 » reste acceptée (1,25 unité). Une ligne rouverte au prix `1.2340` se montre `1.2340`, sans message, et repart
   `1.2340`. Le test `supplier-invoice-form.test.ts:48` (`['1.5', '3.333', '4.9995']`) passe à `'3.3330'`, ce que le Dev
   Agent Record dit.
4. **AC4 — Montant attendu** : « 1.250 » sans QR → message sous le champ et dans `formError`, `onSubmit` non appelé ;
   « 1'250 » → `expectedPaymentAmount: '1250'` ; vide → `null` ; rouvert `1.2340` → champ `1.2340`, sans message ; un
   scan QR à `1250.00` → aucun message.
5. **AC5 — Règlement** (`SettleInvoiceDialog.test.ts`) : « 1.250 » → message ambigu, *Confirmer* sans effet ; « 1'250 »
   (reste dû 2000) → `onConfirm` reçoit `amount: '1250'` ; « 1,25 » → `'1.25'` ; « 12'34 » → message illisible (« 2
   décimales au plus ») ; « 10.0081 » → `invoice-error-amount-scale` ; « 0 » → `invoice-error-amount-positive` ; le montant
   pré-rempli (`10.01` pour un reste de `10.0050`) passe sans message ; les trois tests contraires réécrits (D8).
6. **AC6 — Prix d'article** (`product-helpers.test.ts`, `products-page.test.ts`) : `classifyPriceInput` — « 1.250 »,
   « 1,250 » → `ambiguous` ; « 1'250 », « 10,50 », « 0 », « 007.50 » → `ok` ; « -1 » → `negative` ; « abc », « 1.23456 »,
   « .5 » → `invalid` ; page : « 1.250 » → message ambigu, aucun appel ; « 1'250 » → `unitPrice: '1250'` ; rouvert aux prix
   serveur `1.2340`, `1.2500`, `1500.0000`, `0.1230` → champs `1.2340`, `1.25`, `1500.00`, `0.1230`, **sans message**.
7. **AC7 — Frais de rappel** (test neuf `frontend/src/routes/(app)/settings/dunning/dunning-page.test.ts`) : « 1.250 » →
   message ambigu, `createDunningLevel` non appelé ; « 1'250 » → `feeAmount: '1250'` ; «  12.50  » → `'12.50'` ;
   « 12'34 » → « … 2 décimales au plus » ; un niveau rouvert à `10.00` s'enregistre sans message. **Étape 0** : constater
   par un `curl` (base jetable) ou un test serveur que `PUT /dunning-levels/{id}` refuse `"1.250"` en 400 aujourd'hui,
   journal `kesh-gate-logs/e36-18/etape0-dunning.log`.
8. **AC8 — Montant minimal** (`settings-invoicing-page.test.ts`) : « 1.250 » → message sous le champ, `updateInvoiceSettings`
   non appelé ; « 1'250 » → `minimumInvoiceAmount: '1250'` ; vide → `null` (test `:176-184` vert sans modification) ;
   rouvert aux formes serveur `5.00` et `1.2500` → aucun message ; **étape 0** : la forme réelle relevée par
   `GET /api/v1/company/invoice-settings` (journal `etape0-minimum.log`).
9. **AC9 — Filtres** (`journal-entries-list-page.test.ts`) : « 1.250 » en *Montant min* → message et phrase « pas appliqué »,
   `fetchJournalEntries` appelé **sans** `amountMin`, URL sans `amountMin` ; « 1'250 » → `amountMin: '1250'` et
   `?amountMin=1250` ; une URL `?amountMin=1.250` → message au chargement, borne non envoyée ; « 100 » → `amountMin: '100'`.
10. **AC10 — Rien de juste n'est refusé à tort** : pour chacun des huit sites, un test par forme — « 1.25 », « 1250 »,
    « 1'250.50 », « 1’250 » (apostrophe typographique), « 1,25 », « 0.50 » — sans message et envoyée normalisée ; les
    formes rouvertes de D2 sans message. La seule forme acceptée hier et refusée demain est celle de la règle : un point ou
    une virgule suivis de trois chiffres après une partie entière non groupée de un à trois chiffres, plus ce que la
    grammaire du journal ne lit pas (`1e3`, `.5`, `5.`, signe, blanc intérieur), dit à la note de version.
11. **AC11 — E2E** (fichier neuf `frontend/tests/e2e/montants-ambigus.spec.ts`, un `test` par écran, fixtures synthétiques,
    contact, compte et facture créés par l'API) : pour chaque écran, saisie « 1.250 » → message visible et **zéro**
    requête d'écriture (écouteur `page.on('request')` armé avant le clic) ; puis « 1'250 » → le corps de la requête
    (`page.waitForRequest`) porte `1250`, la réponse est 2xx, et une relecture par `authedApiContext` rend la valeur à
    1250 (`1250.0000`, `1250.00` selon la colonne). Filtres : la requête `GET /api/v1/journal-entries` porte
    `amountMin=1250`, et aucune borne pour « 1.250 ».
12. **AC12 — Textes** : cinq clés × quatre locales aux emplacements de D13 ; `npm run lint-i18n-ownership` vert sans entrée
    neuve (`common-*` globale, `supplier-invoices-*` dans son dossier, routes hors du parcours) ; `sitesTotal` de
    `frontend/src/lib/shared/i18n-keys.test.ts` (`:1020`, 3050 à `fed34a9c7`) **recompté à la source** par la garde,
    valeur mise à ce qu'elle rend, sans addition (C-e350-18).
13. **AC13 — Manuels** (`.tex` ; PDF à l'intégration, contrôlé aplati) : une phrase par écran, qui renvoie à la règle de
    *Saisie d'une écriture en partie double* (`user-manual.tex:559`) avec l'exemple `1.250`, la forme `1'250` et la limite
    de décimales du champ — `user-manual.tex` § *Reprise de comptabilité* (après l'item *Saisissez les soldes*, `:1004`),
    item *Prix unitaire HT* (`:1126`), § *Les montants* de la facture fournisseur (`:1780-1784`), § *Enregistrer* du
    règlement (`:1443-1445`), § *Le montant minimum* (`:1197`), § *Recherche d'écritures*, item *Montant* (`:881`, la borne
    non appliquée) ; `admin-manual.tex` item *Frais* (`:1338`) et § *Montant minimum* (`:3856`). Contrôle :
    `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'refusé comme ambigu'`
    supérieur de cinq au moins à sa valeur à `fed34a9c7`, relevée à l'étape 0 ; même forme pour `admin-manual.pdf`.
14. **AC14 — Rien d'autre ne change** : `git diff fed34a9c7 --stat` ne nomme ni `crates/` hors `kesh-i18n/locales/`, ni
    `amount-ambiguity.ts`, ni `JournalEntryForm.svelte`, `VatPurchaseAssistant.svelte`, `TransactionSplitModal.svelte`,
    `InvoiceForm.svelte`, `bank-accounts/` ; `amount-ambiguity.test.ts`, `unit-price.test.ts`, `InvoiceForm.test.ts`,
    `format.test.ts` verts sans modification ; `readJournalAmount` inchangé (`git diff fed34a9c7 -- frontend/src/lib/features/journal-entries/balance.ts`
    n'ajoute que les deux aides et leur doc-comment, et le doc-comment de `defaultAmountReader`).
15. **AC15 — Captures** (thème clair, base jetable `kesh_e2e_e3618`, `kesh-gate-logs/e36-18/captures/`) : pour chaque écran,
    le champ à « 1.250 » (message), à « 12'34 » (illisible), et la valeur « 1'250 » enregistrée ; un brouillon fournisseur
    rouvert au prix `1.2340` ; un article rouvert à `1.2340` ; le filtre ambigu et sa phrase.

## Tests à écrire, et la preuve du rouge

Écrits d'abord, sur le code non corrigé, journal `/home/gcorbaz/devel/kesh-gate-logs/e36-18/rouges.log`. Les aides neuves
(D1, D2) se posent d'abord avec la lecture d'aujourd'hui (`amountEntryPayload` → le brut si `isValidAmount`, sinon `null` ;
`readableUnambiguousAmount` → `readableAmount`), pour que les rouges portent sur le comportement et non sur un export
absent.

- **Rouges attendus** : AC1 (« 1.250 » non `null` ; « 1'250 » `null` ; « 1.2340 » → `1.234`) ; AC2 (*Générer* envoie
  `1.250`) ; AC3 (`onSubmit` reçoit `unitPrice: '1.250'` ; rouvert `1.234`) ; AC4 (`expectedPaymentAmount: '1.250'`) ; AC5
  (`onConfirm` reçoit `'1.250'` ; « 1'250 » refusé « supérieur à zéro ») ; AC6 (`classifyPriceInput('1.250') === 'ok'`) ;
  AC7 (`feeAmount: '1.250'` part ; « 1'250 » refusé) ; AC8 (`minimumInvoiceAmount: '1.250'`) ; AC9 (`amountMin: '1.250'`) ;
  AC11 (chaque écran : la requête part avec `1.250`).
- **Verts au départ, et dits** : les formes juste d'AC10 sur les sites qui les lisaient déjà (« 1.25 », « 1250 » partout ;
  « 1,25 » aux soldes et au fournisseur) ; le prix d'article rouvert (AC6, jamais ambigu par construction) ; les montants
  rouverts aux frais, au minimum et au règlement. Ces verts ne prouvent rien d'eux-mêmes : ils gardent la non-régression,
  et les mutations M13, M17 et M18 vérifient qu'ils rougiraient.
- **E2E rouge** : sources de la story remises au commit de base par `git restore --source=fed34a9c7 --worktree -- <fichiers>`
  **après** commit du code, frontend construit, spec rouge écran par écran, sources restaurées (`git restore --worktree` +
  `touch`), journal `e2e-rouges.log`.

**Mutations** (code commité d'abord ; scripts versionnés dans `/home/gcorbaz/devel/kesh-gate-logs/e36-18/mut/` ;
restauration `git restore --worktree -- <fichier>` puis `touch` ; journaux `mut/M*.log`, résumé `mut/resume.log` ;
Vitest seul, les E2E ne se rejouent pas sous mutation) :

| # | Lot | Mutation | Tuée par |
|---|---|---|---|
| M1 | A | `amountEntryPayload` rend `payload` aussi pour `ambiguous` | AC1, AC2, AC3, AC5 |
| M2 | A | `readableUnambiguousAmount` sans le repli `toFixed(4)` | AC1, AC3, AC4 (rouvert) |
| M3 | A | soldes : `computeBalance(rows)` sans `journalAmountReader` | AC2 (*Générer* actif) |
| M4 | A | soldes : la charge reprend `r.debit.replace(',', '.')` | AC2 (`1250`) |
| M5 | A | soldes : la rangée de message retirée | AC2 |
| M6 | A | totaux : `amount()` reprend `isValidAmount` / `parseAmount` | AC2 (totaux) |
| M7 | A | fournisseur : `linesInvalid` reprend `isValidAmount` pour le prix | AC3 |
| M8 | A | fournisseur : `linesForServer` reprend `decimalForServer(l.unitPrice)` | AC3 (`1250`) |
| M9 | A | fournisseur : la garde d'envoi par ligne retirée | AC3 (« Ligne 1 ») |
| M10 | A | fournisseur : `formLineFromResponse` reprend `readable(…, 2)` | AC3 (rouvert) |
| M11 | A | montant attendu : la garde d'envoi retirée | AC4 |
| M12 | A | montant attendu : rouvert par `readableAmount` | AC4 |
| M13 | A | règlement : `clientError` reprend `Number(amount)` | AC5 |
| M14 | A | règlement : `onConfirm` reçoit `amount.trim()` | AC5 (`1250`) |
| M15 | A | règlement : `$max` 4 au lieu de 2 | AC5 (« 2 décimales ») |
| M16 | B | article : `classifyPriceInput` sans le cas `ambiguous` | AC6 |
| M17 | B | article : `normalizePriceInput` reprend `replace(',', '.')` | AC6 (`1250`), AC10 |
| M18 | B | article : `openEdit` passe par `readableAmount(…, 2)` | AC6 (rouvert `1.234` → message) |
| M19 | B | frais : `feeAmount: fFee` brut | AC7 |
| M20 | B | frais : la lecture sans `trim()` | AC7 (« 12.50 » rogné) |
| M21 | B | minimum : `minimumAmount.trim()` envoyé brut | AC8 |
| M22 | B | filtres : `buildQuery` envoie la borne brute | AC9 |
| M23 | B | filtres : la phrase « pas appliqué » retirée | AC9 |

Une mutation équivalente se dit au Dev Agent Record.

## Tâches

- [ ] **Étape 0** : relire à la tête du worktree (après fusion de la 36-15, qui touche `SupplierInvoiceForm.svelte`) les
      lignes citées ; rejouer les sept motifs d'inventaire, sortie à `kesh-gate-logs/e36-18/etape0-inventaires.log` ;
      AC7 (`curl` des frais) et AC8 (forme du minimum) ; valeur de départ du contrôle PDF (AC13) ; relire les LOW du
      dernier rapport de validation.
- [ ] Lot A — tests d'abord (AC1 à AC5), rouges consignés ; `balance.ts` et `unit-price.ts` (D1) ;
      `supplier-invoice-form.ts` (D2, D6, D7) ; `opening-balances-totals.ts` et la page (D5) ; `SupplierInvoiceForm.svelte`
      (D6, D7) ; `SettleInvoiceDialog.svelte` (D8).
- [ ] Lot B — tests d'abord (AC6 à AC9) ; `product-helpers.ts` et la page (D9) ; `settings/dunning` (D10) ;
      `settings/invoicing` (D11) ; `journal-entries/+page.svelte` (D12, en dernier).
- [ ] Quatre `messages.ftl` (D13) ; `i18n-keys.test.ts` recompté (AC12).
- [ ] E2E (AC11), rouge d'abord ; manuels (AC13), `.tex` seuls ; captures (AC15).
- [ ] Mutations M1 à M23 du lot.
- [ ] Gates (voie 2, `e35-dev-commun.md` § *Quels gates, quand*) : en cours, Vitest ciblé (fichiers de test des AC du lot,
      `shared/i18n-keys.test.ts`) par `.gate-front.lock`, `npm run check`, `lint-i18n-ownership` ; **passage en `review`** :
      bases `kesh_e3618` remises à neuf sur la voie 2, `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh kesh_ci_e3618`
      (voie 1, son verrou), frontend complet (`check`, `lint-i18n-ownership`, `test:unit`, `build`), pas d'E2E ;
      **remédiations** : `test-fast.sh --ci` et frontend complet ; **clôture** : `gate-ci` et E2E complet sur
      `kesh_e2e_e3618` recréée (voie 2), au dernier commit de code, échecs nommés contre `docs/testing.md` § « Les échecs
      attendus ». Une commande lourde à la fois. Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e36-18/`, commit en
      première ligne de chacun.
- [ ] Fin du développement : `Status: review`, clé `review` au `sprint-status.yaml`, prompt de la revue P1
      (`36-18-montants-ambigus-autres-ecrans-review-prompt-p1.md`) qui nomme les manuels, le PDF (à l'intégration) et les
      captures.

## Note de version

*Rubrique* **Corrigé** (C-e360-11 ; l'orchestrateur l'assemble dans `## [0.18.0] — Non publié`) :

- Les soldes de départ, le prix unitaire et le montant attendu d'une facture fournisseur, le règlement d'une facture, le
  prix d'un article, les frais de rappel, le montant minimal d'une facture et les bornes du filtre des écritures lisent
  désormais un montant comme une écriture : un montant comme « 1.250 » — un point ou une virgule suivis de trois chiffres,
  sans séparateur de milliers avant — n'est plus lu 1,25 franc en silence ; il est refusé comme ambigu, rien ne
  s'enregistre (le filtre ne s'applique pas), et le message dit comment l'écrire (`1'250` ou `1.25`). L'apostrophe des
  milliers et la virgule décimale sont acceptées partout (`1'250`, `1,25`) ; la saisie à exposant (`1e3`) ou sans chiffre
  avant ou après le point (`.5`, `5.`) n'est plus lue. Un règlement de « 10.000 » est refusé comme ambigu : écrivez
  `10.00` ou `10'000`. Un prix ou un montant attendu enregistré au millième se rouvre sur quatre décimales (`1.2340`) et
  n'est pas refusé. (refs #1050, refs #1159, refs #1172, refs #1173, refs #1174, refs #1175, refs #1176, refs #1177)

*(Si la story se découpe en deux lots, la phrase se partage : lot A refs #1050, #1159, #1172, #1174 ; lot B refs #1173,
#1175, #1176, #1177.)*

## Dev Notes

### Hors périmètre

- Les quantités (fournisseur et client) : « 1.250 » y est une lecture voulue (C-e360-6).
- Le serveur et l'API (C-e350-19) : un client d'API qui envoie « 1.250 » veut 1,25 ; « 1'250 » y reste refusé.
- La convergence des messages de la 36-3 et de la 36-3b (`journal-entry-form-amount-*`, `invoice-line-unit-price-*-error`,
  `reconciliation-split-error-amount-*`) vers les clés communes : **issue à ouvrir** (`technical-debt`, une seule).
- Un montant déjà enregistré trop bas avant la 0.18.0 (« 1.250 » lu 1,25) : rien ne permet de le reconnaître ; aucune
  reprise des données.
- Les taux de TVA (`settings/vat-rates`, `type="number"`) : un pourcentage, hors objet.

### Splitting — sept modules de code : découpage proposé

Modules de code au sens du `CLAUDE.md` : (1) `features/journal-entries` (`balance.ts` ; la page de liste,
`routes/(app)/journal-entries`, au même module métier) ; (2) `features/invoices` (`unit-price.ts`,
`SettleInvoiceDialog.svelte`) ; (3) `features/supplier-invoices` ; (4) soldes de départ (`features/opening-balances` et
`routes/(app)/settings/opening-balances`) ; (5) articles (`features/products` et `routes/(app)/products`) ; (6) réglages
(`routes/(app)/settings/dunning`, `settings/invoicing`) ; (7) `crates/kesh-i18n/locales`. S'y ajoutent les manuels et les
tests transverses (`montants-ambigus.spec.ts`, `i18n-keys.test.ts`). **Sept : la règle s'applique.**

Découpage proposé (à l'orchestrateur de l'écrire, C-36-18-9) :
- **Lot A — 36-18a, ce qui écrit dans les livres** (#1050, #1159, #1172, #1174) : D1, D2, D5 à D8 ; modules (1) `balance.ts`,
  (2), (3), (4), (7) — **cinq** ; AC1 à AC5.
- **Lot B — 36-18b, réglages, catalogue et filtres** (#1173, #1175, #1176, #1177 en dernier) : D9 à D12 ; modules (1) page
  de liste, (5), (6), (7) — **quatre** ; AC6 à AC9 ; part du lot A fusionné (aides de D1, clés communes de D13).

### Fichiers communs avec les autres stories de la phase 2

- `SupplierInvoiceForm.svelte` : la fiche 36-15 (vocabulaire « compte analytique ») le nomme
  (`git show story/36-15-compte-analytique:_bmad-output/implementation-artifacts/36-15-compte-analytique.md`) ; 36-15
  passe avant (ordre de D1/D6) : la 36-18 part de 36-15 fusionnée.
- `messages.ftl` ×4, `user-manual.tex`, `admin-manual.tex` : 36-9, 36-10, 36-15, 36-17, 36-19 y écrivent (branches lues
  par `git diff --name-only origin/epic/36-0.18.0...story/<clé>`) — clés à côté de leurs blocs (D13), paragraphes distincts ;
  PDF après l'union.
- `frontend/src/lib/shared/i18n-keys.test.ts` (`sitesTotal`) : 36-17 le touche ; recompté par la branche puis après
  chaque fusion, sans additionner.
- `settings/invoicing/+page.svelte`, `journal-entries/+page.svelte` : la fiche 36-17 les nomme ; sa branche ne les
  modifie pas à `fed34a9c7` (même commande) ; à recontrôler à l'étape 0.
- `CHANGELOG.md` : **non écrit** par la branche (C-e360-11) ; la *Note de version* ci-dessus.

### Ce qui doit rester vrai

- Un montant juste ne devient jamais refusé (AC10) ; un montant rouvert ne l'est jamais (D2, AC3, AC4, AC6, AC7, AC8).
- Le serveur reste juge ; rien n'est écrit d'office ; big.js partout, jamais `parseFloat`. Fixtures synthétiques seulement.
- Les soldes de départ ne rognent toujours pas (D4).

### E2E

Le défaut se voit dans le navigateur et à la frontière HTTP (chaque montant part tel que tapé) : l'E2E AC11 est exigé, un
test par écran.

### Références

- Issues #1050, #1159, #1172 à #1177 ; Stories 36-3 (#1051), 36-3b (#1171), 35-6 (#1026), 36-2 ; C-e360-6, C-e360-11,
  C-e360-17, C-e350-18, C-e350-19 ; #878 (forme lisible d'un montant rechargé) ; #687 (centime, fournisseur).
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel* ; § *Inventorier les sites NON
  RÉSOLUS* ; § *Règle de splitting préventif*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — Fiche créée (Opus 5.5, `bmad-create-story`) à `fed34a9c7` : huit sites lus au code, inventaire par le
  symptôme (sept motifs, sonde positive, aucune issue neuve), décisions D1 à D14, AC1 à AC15 étiquetés lot A / lot B,
  mutations M1 à M23 ; registre `e36-choix-autonomes-36-18.md` (C-36-18-1 à C-36-18-9). Écart à l'issue #1175 relevé
  (le serveur refuse déjà « 1.250 » en frais de rappel, C-36-18-6). Sept modules de code : découpage en deux lots proposé,
  non écrit. Aucune migration. Validation non lancée.
- 2026-11-15 — **Découpée** sur approbation de l'orchestrateur (C-36-18-10) : `36-18a-montants-ambigus-livres.md`
  (#1050, #1159, #1172, #1174 ; cinq modules ; décisions communes), `ready-for-dev`, et
  `36-18b-montants-ambigus-reglages-filtres.md` (#1173, #1175, #1176, #1177 ; quatre modules ; part de la 36-18a fusionnée),
  `backlog`. Cette fiche passe à `split` et reste comme trace de l'inventaire complet. La convergence des messages est
  ouverte en #1202 ; #1175 est commentée avec le constat de C-36-18-6.
