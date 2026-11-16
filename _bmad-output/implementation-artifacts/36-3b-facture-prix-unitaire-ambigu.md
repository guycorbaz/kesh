# Story 36-3b : la facture client ne lit plus un prix unitaire « 1.250 » comme 1,25 franc (#1171)

Status: done

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-3b`, branche
`story/36-3b-facture-prix-unitaire-ambigu`, partie de la tête de la 36-3 terminée (`b1cd53d28`), par le workflow
`bmad-create-story`, prompt `kesh-gate-logs/prompts/e36-fiche.md`. Release **0.17.8**, phase 1. **Aucune migration**,
aucun changement serveur ni d'API. Code lu à la tête du worktree (`b1cd53d28`). Voie de développement : **voie 2**.
Registres : `e36-choix-autonomes-e360.md` de la branche du kickoff (`chore/epic-36-kickoff`, `153bd2bb0`, C-e360-17, qui
fait foi) ; `e36-choix-autonomes-36-3b.md` (C-36-3b-1 à C-36-3b-7). Validation non lancée.*

## Story

En tant que titulaire qui émet une facture client — la première avant la fin de novembre —, je veux qu'un prix unitaire
que Kesh ne peut pas lire sans ambiguïté (« 1.250 » : mille deux cent cinquante francs à l'allemande, ou un franc
vingt-cinq ?) soit refusé avec un message qui dit comment l'écrire, et que « 1'250 » soit accepté pour mille deux cent
cinquante, afin qu'aucune facture ne parte au millième du prix voulu sans que je le voie.

## Ce qui se lit au code (`b1cd53d28`)

**Le défaut** — `frontend/src/lib/components/invoices/InvoiceForm.svelte` (984 lignes), `validateClient` :
- `:509-519` :
  ```ts
  try {
  	const qty = new Big(l.quantity);
  	const price = new Big(l.unitPrice);
  	if (qty.lte(0)) return `Ligne ${i + 1} : quantité > 0`;
  	if (price.lt(0)) return `Ligne ${i + 1} : prix unitaire ≥ 0`;
  	...
  } catch {
  	return `Ligne ${i + 1} : valeurs numériques invalides`;
  }
  ```
  `new Big('1.250')` vaut 1,25 ; `new Big("1'250")`, `new Big('1,25')` et `new Big('')` lèvent ; `new Big('1e3')` vaut 1000,
  `new Big('.5')` 0,5, `new Big('-5')` −5 (relevé à l'exécution, `node -e` sur le big.js du dépôt principal, lecture seule).
- `:594` la charge : `lines: lines.map(stripUiKey),` — `stripUiKey` (`:111-114`) ne retire que `_uiKey` et
  `_vatPlaceholder` : **le prix part tel que tapé**, « 1.250 » compris, et le serveur le lit 1,25.
- Le champ : `:842` `<Input type="text" inputmode="decimal" bind:value={line.unitPrice} />` ; aucun message sous le
  champ, aucune bordure.

**Les chemins du prix dans le formulaire** (`git grep -n "unitPrice\|stripUiKey\|computeInvoiceTotal\|computeLineTotal\|totalAmount" -- frontend/src/lib/components/invoices/InvoiceForm.svelte`) :

```
frontend/src/lib/components/invoices/InvoiceForm.svelte:24:		computeInvoiceTotal,
frontend/src/lib/components/invoices/InvoiceForm.svelte:25:		computeLineTotal,
frontend/src/lib/components/invoices/InvoiceForm.svelte:111:	function stripUiKey(l: LineState): CreateInvoiceLineRequest {
frontend/src/lib/components/invoices/InvoiceForm.svelte:122:				unitPrice: readableAmount(l.unitPrice, 2),
frontend/src/lib/components/invoices/InvoiceForm.svelte:134:				unitPrice: '0.00',
frontend/src/lib/components/invoices/InvoiceForm.svelte:437:			unitPrice: '0.00',
frontend/src/lib/components/invoices/InvoiceForm.svelte:453:			unitPrice: readableAmount(p.unitPrice, 2),
frontend/src/lib/components/invoices/InvoiceForm.svelte:474:		return computeLineTotal(l.quantity, l.unitPrice);
frontend/src/lib/components/invoices/InvoiceForm.svelte:477:	let totalAmount = $derived(computeInvoiceTotal(lines));
frontend/src/lib/components/invoices/InvoiceForm.svelte:511:				const price = new Big(l.unitPrice);
frontend/src/lib/components/invoices/InvoiceForm.svelte:594:				lines: lines.map(stripUiKey),
frontend/src/lib/components/invoices/InvoiceForm.svelte:650:				unitPrice: readableAmount(l.unitPrice, 2),
frontend/src/lib/components/invoices/InvoiceForm.svelte:656:				// les efface en base. `stripUiKey` étant un spread, rien ne signale
frontend/src/lib/components/invoices/InvoiceForm.svelte:842:							<Input type="text" inputmode="decimal" bind:value={line.unitPrice} />
frontend/src/lib/components/invoices/InvoiceForm.svelte:912:						{formatInvoiceTotal(totalAmount)}
```

| Chemin | Ligne | Ce qu'il fait du prix | Décision |
|---|---|---|---|
| Saisie | `:842` | `bind:value` brut | message et bordure sous le champ (D3) |
| Ligne neuve, ligne libre | `:134`, `:437` | `'0.00'` | inchangé (`clear`, partie nulle) |
| Total de ligne vivant | `:473-475` `computeLineTotal(l.quantity, l.unitPrice)`, affiché `:891` | `new Big(qty).times(unitPrice)` (`invoice-helpers.ts:29-35`) : « 1.250 » × 1 = 1.25 | lu par la charge du prix, `0` si refusé (D4) |
| Total de la facture | `:477`, affiché `:912` | `computeInvoiceTotal(lines)` (`invoice-helpers.ts:40-52`) | idem (D4) |
| Validation avant envoi | `:506-519` | `new Big(l.unitPrice)` | lecture du journal (D1, D3) |
| Construction du corps | `:588-595` | brut | charge normalisée, garde après l'attente des réglages (D5) |
| Brouillon rouvert | `:119-123` `unitPrice: readableAmount(l.unitPrice, 2),` | forme lisible de la valeur serveur | jamais ambiguë (D2) |
| « Recharger » après conflit | `:645-650` | idem | idem (D2) |
| Ligne depuis un article | `:449-453` `unitPrice: readableAmount(p.unitPrice, 2),` | idem, valeur de la fiche article | idem (D2) ; le prix de l'article lui-même : **#1173** |

**La forme d'un prix rechargé** — `supplier-invoices/supplier-invoice-form.ts:214-224` `readableAmount` :
`const s = b.toFixed();` … `return decimals < minDecimals ? b.toFixed(minDecimals) : s;`. Relevé à l'exécution (big.js
du dépôt) : « 1.2500 » → `1.25`, « 1250.0000 » → `1250.00`, « 100.0000 » → `100.00`, « 33.3333 » → `33.3333`,
**« 1.2340 » → `1.234`**, **« 12.3450 » → `12.345`**. Un prix « 1.250 » mal saisi avant la story, enregistré 1,25, se
rouvre donc `1.25` (lisible, non refusé : rien ne permet de savoir qu'il était faux) ; un prix légitime au millième
(1,234 franc) se rouvrirait `1.234` et **serait refusé à tort** sans D2. Le commentaire `InvoiceForm.svelte:43-45`
(« `DECIMAL(19,4)`, `1.0000`, `100.0000` ») confirme la forme du serveur.

**La validation serveur du prix** (inchangée) — `crates/kesh-api/src/routes/invoices.rs` :
- `:506` `if req.unit_price < Decimal::ZERO {` (positif ou nul) ;
- `:512` `if !scale_within(&req.unit_price, MAX_DECIMAL_SCALE) {` — `limits.rs:28` `pub const MAX_DECIMAL_SCALE: u32 = 4;`,
  `limits.rs:31-32` `value.scale() <= max_scale` : quatre décimales au plus, **sans** normaliser (« 1.25000 » refusé ;
  écart voulu avec la facture fournisseur, commentaire `supplier_invoices.rs:521-524`) ;
- `:515` `if req.unit_price > *MAX_UNIT_PRICE {` — `limits.rs:13-14`, 10⁹ ;
- `:524` `let line_total = req.quantity * req.unit_price;` borné à `MAX_LINE_TOTAL` (10¹²), **sans arrondi au centime**.
**La règle du centime #687** (« Une facture fournisseur peut porter un TTC à plus de deux décimales… ») ne vise que la
facture **fournisseur** (`crates/kesh-core/src/accounting/purchase.rs:1` « Le montant d'une ligne de facture fournisseur —
Story v017-3b (#687). ») ; la ligne de facture client garde son total à quatre décimales
(`invoice-helpers.ts:25-31`, `.round(4, ROUND_HALF_UP).toFixed(4)`). **Le prix suit donc la grammaire du journal sans
variante** : un à quinze chiffres entiers (ou groupés par apostrophe), une à quatre décimales, aucun signe
(C-36-3b-1). Le plafond de 10⁹ reste au serveur, comme aujourd'hui (le client ne le contrôle pas).

**Ce que la 36-3 fournit, réemployé sans modification** :
- `frontend/src/lib/shared/utils/amount-ambiguity.ts:38` `export function isAmbiguousThousands(parts: AmountParts): boolean {`
  — appelé indirectement, par le lecteur ;
- `frontend/src/lib/features/journal-entries/balance.ts:110` `export function readJournalAmount(raw: string): JournalAmountReading {`
  (`empty` / `clear` avec `value` et `payload` / `ambiguous` / `unreadable`), grammaire `:90`
  `const JOURNAL_AMOUNT_RE = /^([1-9]\d{0,2}(?:['’]\d{3})+|\d{1,15})(?:[.,](\d{1,4}))?$/;` ;
- **non réemployé** : `journalAmountPayload` (`:143-147`), qui rend `'0'` pour le vide (`if (r.kind === 'empty') return '0';`)
  — un prix effacé partirait à zéro (C-36-3b-2).

**Les messages de la 36-3** — `crates/kesh-i18n/locales/fr-CH/messages.ftl:313-314` (même rang dans les quatre
locales) : `journal-entry-form-amount-ambiguous = Montant ambigu : un point ou une virgule suivis de trois chiffres peuvent
séparer les milliers. Écrivez 1'234 pour mille deux cent trente-quatre, ou 1.2340 pour un franc vingt-trois et quatre
dixièmes de centime.` ; `journal-entry-form-amount-unreadable = Montant illisible : chiffres, apostrophe des milliers
(1'234), point ou virgule décimale, quatre décimales au plus.` Les deux disent vrai d'un prix unitaire. Leur affichage au
journal (`JournalEntryForm.svelte:504-527`) : une rangée sous la ligne, `class="text-xs text-destructive pb-2"`,
`data-testid="journal-entry-line-amount-ambiguous"` / `-unreadable` ; bordure `class={amountFlagged(line.debit) ? 'border-destructive' : …}` (`:440`).

**Le lint de propriété des clés** ne parcourt que `src/lib/features` (`frontend/scripts/lint-i18n-ownership.js:17`
`const FEATURES_PATH = path.join(__dirname, '../src/lib/features');`, `:255-256` `walkDir(FEATURES_PATH, …)`) :
`components/invoices/InvoiceForm.svelte` n'y est pas soumis, aucune entrée `KNOWN_VIOLATIONS` n'est à ajouter. La garde
`frontend/src/lib/shared/i18n-keys.test.ts` parcourt tout `src` (`:1301` `parcourir('src');`) : `sitesTotal`
(`:992` `sitesTotal: 3040,`) se recompte.

**Le refus visible aujourd'hui** passe par `validateClient` puis le bandeau `errorMsg` (`:698-701`) ; le bouton d'envoi
(`:933-935`) n'est désactivé que par l'envoi en cours, le conflit, les réglages et les comptes de produit invalides. Les
messages de ligne de `validateClient` sont en français en dur, sauf `invoice-line-vat-rate-disabled-error`
(`fr-CH/messages.ftl:2659` `Ligne { $line } : le taux de TVA { $rate }% a été désactivé. Choisissez un taux actif.`).

### Inventaire par le symptôme

Le symptôme : « un prix ou un montant tapé est lu avec le point pour séparateur décimal, sans contrôle d'ambiguïté ».
L'inventaire complet du dépôt est celui de la fiche 36-3 (§ *Inventaire par le symptôme*, sites 1 à 14) ; rejoué à
`b1cd53d28`, il rend les mêmes fichiers (sortie ci-dessous). Détecteur par lecture, parti du champ (aucun détecteur par
exécution praticable : un E2E par écran) ; **sonde positive** : le premier motif rend `InvoiceForm.svelte`, site connu
du défaut, et `JournalEntryForm.svelte`, site corrigé par la 36-3 (que le troisième motif rend comme lecteur).

`git grep -ln 'inputmode="decimal"' -- frontend/src` :
```
frontend/src/lib/components/invoices/InvoiceForm.svelte
frontend/src/lib/components/invoices/InvoiceForm.test.ts
frontend/src/lib/features/bank-import/ManualStatementForm.svelte
frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte
frontend/src/lib/features/journal-entries/JournalEntryForm.svelte
frontend/src/lib/features/journal-entries/VatPurchaseAssistant.svelte
frontend/src/lib/features/supplier-invoices/SupplierInvoiceForm.svelte
frontend/src/routes/(app)/bank-accounts/+page.svelte
frontend/src/routes/(app)/journal-entries/+page.svelte
frontend/src/routes/(app)/products/+page.svelte
frontend/src/routes/(app)/settings/dunning/+page.svelte
frontend/src/routes/(app)/settings/invoicing/+page.svelte
frontend/src/routes/(app)/settings/opening-balances/+page.svelte
```
`git grep -n "new Big(" -- frontend/src/lib/components/invoices/InvoiceForm.svelte` :
```
frontend/src/lib/components/invoices/InvoiceForm.svelte:510:				const qty = new Big(l.quantity);
frontend/src/lib/components/invoices/InvoiceForm.svelte:511:				const price = new Big(l.unitPrice);
```
`git grep -n "readJournalAmount\|journalAmountPayload\|isAmbiguousThousands" -- frontend/src ':!*.test.ts'` (les lecteurs
corrigés) : `bank-accounts/format.ts:320`, `journal-entries/JournalEntryForm.svelte:185`, `:223-224`,
`journal-entries/balance.ts`, `journal-entries/vat-purchase.ts:119` — **aucun** site de la facture client.

| Site | Ligne lue | Décision |
|---|---|---|
| Prix unitaire de la facture client | `InvoiceForm.svelte:511`, `:594`, `:842` | **périmètre** |
| Quantité de la facture client | `InvoiceForm.svelte:510` `const qty = new Big(l.quantity);` | **hors de la règle** : « 1.250 » y vaut 1,25 unité, lecture voulue ; inchangée (virgule toujours refusée) |
| Prix de l'article (catalogue) | `features/products/product-helpers.ts:53` `PRICE_RE`, `:72` | hors périmètre, **#1173** |
| Prix unitaire fournisseur | `supplier-invoice-form.ts:152-153` | hors périmètre, **#1159** |
| Montant de paiement attendu (fournisseur) | `supplier-invoice-form.ts:283` | hors périmètre, **#1172** |
| Règlement d'une facture | `invoices/SettleInvoiceDialog.svelte:68-76` | hors périmètre, **#1174** |
| Frais de rappel | `settings/dunning/+page.svelte:104` | hors périmètre, **#1175** |
| Montant minimal de facture | `settings/invoicing/+page.svelte:276` | hors périmètre, **#1176** |
| Filtres de la liste des écritures | `routes/(app)/journal-entries/+page.svelte:401-415` | hors périmètre, **#1177** |
| Soldes de départ | `settings/opening-balances/+page.svelte:150-151`, `:199-200` | hors périmètre, **#1050** |
| Écriture manuelle, assistant TVA achat, solde d'ouverture | — | corrigés (36-3, 35-6) |

**Ce qui arrive à un article au prix ambigu** : l'écran article accepte « 1.250 » et l'enregistre 1,25 (#1173). La ligne
préremplie reçoit `readableAmount('1.2500', 2)` = `1.25`, **clair** : elle n'est pas refusée, et l'erreur de l'article
passe sur la facture sans signal — ce que seule #1173 corrige. Un article au prix légitime « 1.2340 » se préremplit
`1.2340` grâce à D2, et non `1.234` refusé à tort.

## Décisions

- **D1 — Le prix unitaire se lit par `readJournalAmount`, tel quel** (C-e360-17, C-36-3b-1). Aucune grammaire propre ;
  ni `balance.ts` ni `amount-ambiguity.ts` ne changent (C-36-3b-6). Conséquences, à dire au manuel : la virgule
  décimale (« 1,25 ») et l'apostrophe des milliers (« 1'250 », « 1’250.50 ») sont admises et partent normalisées
  (`1.25`, `1250`, `1250.50`) ; « 1.250 », « 12,500 », « 001.250 » sont **ambigus** ; « -5 », « 1e3 », « .5 », « 5. »,
  « 1.25000 », « 12'50 » sont **illisibles** (aujourd'hui « -5 » rendait « prix unitaire ≥ 0 », « 1e3 » partait au
  serveur). Le vide garde son message d'aujourd'hui (D3). La quantité ne change pas.
  *Écartées* : (a) une variante à signe pour garder « prix unitaire ≥ 0 » — une seconde grammaire pour un cas que le
  serveur refuse ; (b) modifier `readJournalAmount` pour un paramètre — interdit par C-e360-17.
- **D2 — Un prix venu du serveur ne se rouvre jamais sous une forme ambiguë** (C-36-3b-3). Fonction pure neuve dans
  `frontend/src/lib/features/invoices/invoice-helpers.ts` *(étape 0, V1-5 : dans un module neuf
  `frontend/src/lib/features/invoices/unit-price.ts`, avec `unitPricePayload` et `unitPriceProblem`, testé par
  `unit-price.test.ts` — `invoice-helpers.ts` importer `readableAmount` créerait le cycle avec
  `supplier-invoice-form.ts:13`)* :
  `unitPriceFieldValue(raw: string): string` = `readableAmount(raw, 2)`, sauf si `readJournalAmount` de ce résultat est
  `ambiguous` : alors `new Big(raw).toFixed(4)`. Exemples : « 1.2340 » → `1.2340` ; « 12.3450 » → `12.3450` ;
  « 1.2500 » → `1.25` ; « 1250.0000 » → `1250.00` ; « 100.2000 » → `100.20` ; « 33.3333 » → `33.3333` ;
  « 0.0100 » → `0.01` ; « 0.1250 » → `0.125` (partie nulle, clair) ; *(étape 0, V1-4)* « 0.0010 » → `0.001`,
  « 1000.0000 » → `1000.00`, « 1.0000 » → `1.00`, « 999.1230 » → `999.1230` (borne haute de la partie entière
  ambiguë). Une valeur illisible pour big.js est rendue telle
  quelle (contrat de `readableAmount`). Appelée aux **trois** sites : `initLines` (`:122`), `onProductSelect` (`:453`),
  `reloadFromServer` (`:650`). La quantité garde `readableAmount(…, 0)`.
  *Écartées* : (a) modifier `readableAmount` — partagé par la facture fournisseur (`SupplierInvoiceForm.svelte:185`,
  `supplier-invoice-form.ts:236`) et trois écrans de détail ; (b) quatre décimales toujours — défait #878.
- **D3 — Le refus : sous le champ, et à l'envoi en nommant la ligne** (C-36-3b-2, C-36-3b-4).
  - Sous le champ : une rangée sous la ligne (comme `JournalEntryForm.svelte:504-527`), *(étape 0, V1-1)*
    `<tr><td colspan={LINE_TABLE_COLUMNS}>`, constante nommée à 7 (les sept colonnes du tableau, `:830-900`),
    testée, `class="text-xs text-destructive pb-2"`,
    `data-testid="invoice-line-unit-price-ambiguous"` avec `i18nMsg('journal-entry-form-amount-ambiguous', <fallback
    identique à JournalEntryForm.svelte:511>)`, ou `data-testid="invoice-line-unit-price-unreadable"` avec
    `journal-entry-form-amount-unreadable`. Bordure `border-destructive` sur le champ du prix dans ces deux cas. Rien
    pour un prix vide (aujourd'hui non plus).
  - `validateClient`, après la description et **avant** le bloc `try` : `ambiguous` →
    `i18nMsg('invoice-line-unit-price-ambiguous-error', …, { line: i + 1 })` ; `unreadable` →
    `i18nMsg('invoice-line-unit-price-unreadable-error', …, { line: i + 1 })` ; `empty` →
    `Ligne ${i + 1} : valeurs numériques invalides` (inchangé). Dans le `try`, `price` vaut `new Big(payload)`. La
    branche `price.lt(0)` reste (défense ; inatteignable par la grammaire).
  - Le bouton reste actif : clic et *Entrée* passent par `onSubmit` → `validateClient` (`:552`).
  - Fonction pure neuve dans `invoice-helpers.ts` : `unitPricePayload(raw: string): string | null` — `payload` d'une
    lecture `clear`, `null` sinon (vide compris).
  *Écartées* : (a) `journalAmountPayload` — `'0'` pour un prix vide ; (b) préfixer « Ligne N : » en dur devant les clés
  de la 36-3 — message à moitié français ; (c) désactiver le bouton.
- **D4 — Les totaux lisent la même charge** (C-36-3b-5) : `lineTotal(l)` =
  `computeLineTotal(l.quantity, unitPricePayload(l.unitPrice) ?? '0')` ; `totalAmount` =
  `computeInvoiceTotal(lines.map((l) => ({ quantity: l.quantity, unitPrice: unitPricePayload(l.unitPrice) ?? '0' })))`.
  `computeLineTotal` et `computeInvoiceTotal` ne changent pas. « 1'250 » × 2 → `2’500.00` ; « 1.250 » → `0.00`.
  *Écartées* : garder 1,25 (le défaut) ; un tiret (nouvel état de la colonne).
- **D5 — La charge porte le prix normalisé, relu après l'attente des réglages** (C-36-3b-7). `:594` devient la
  construction de chaque ligne `{ ...stripUiKey(l), unitPrice: unitPricePayload(l.unitPrice) }` ; si l'une rend `null`
  (prix retapé pendant `await getInvoiceSettings()`, `:562`), l'envoi s'arrête : `errorMsg = validateClient() ?? …`,
  `submitting = false`, aucune requête. « 100.00 » part `100.00`, « 33.3333 » `33.3333` (les allers-retours de #878,
  `InvoiceForm.test.ts:694`, `:709`, `:720`, inchangés). *(Étape 0, V1-6 : la charge se construit dans une boucle
  à sortie anticipée — `const payloadLines: CreateInvoiceLineRequest[] = []` ; `for (const l of lines)` ;
  `if (unitPrice === null) { … return; }` ; `payloadLines.push({ ...stripUiKey(l), unitPrice })` —, sans quoi
  `unitPrice: string | null` ne se type pas en `CreateInvoiceLineRequest`. La quantité et la description retapées
  pendant l'attente ne sont pas revalidées : hors périmètre, voir Dev Notes.)*
- **D6 — Textes** (quatre locales, **à côté** d'`invoice-line-vat-rate-disabled-error` — fr-CH `:2659`, de-CH `:2531`,
  it-CH `:2529`, en-CH `:2536` —, jamais en fin de fichier ; la 36-1 ajoute `common-account-no-match` dix lignes plus
  haut, sans chevauchement) :
  - `invoice-line-unit-price-ambiguous-error` — fr-CH : « Ligne { $line } : prix unitaire ambigu — écrivez 1'250 pour
    mille deux cent cinquante francs, ou 1.2500 pour un franc vingt-cinq. » ; de-CH : « Zeile { $line }: mehrdeutiger
    Einzelpreis — schreiben Sie 1'250 für eintausendzweihundertfünfzig Franken oder 1.2500 für einen Franken und
    25 Rappen. » ; it-CH : « Riga { $line }: prezzo unitario ambiguo — scrivete 1'250 per milleduecentocinquanta
    franchi, o 1.2500 per un franco e 25 centesimi. » ; en-CH : « Line { $line }: ambiguous unit price — write 1'250 for
    one thousand two hundred fifty francs, or 1.2500 for one franc and 25 centimes. »
  - `invoice-line-unit-price-unreadable-error` — fr-CH : « Ligne { $line } : prix unitaire illisible — chiffres,
    apostrophe des milliers (1'250), point ou virgule décimale, quatre décimales au plus. » ; de-CH : « Zeile { $line }:
    unlesbarer Einzelpreis — Ziffern, Apostroph als Tausendertrennzeichen (1'250), Punkt oder Komma als
    Dezimaltrennzeichen, höchstens vier Nachkommastellen. » ; it-CH : « Riga { $line }: prezzo unitario illeggibile —
    cifre, apostrofo delle migliaia (1'250), punto o virgola decimale, al massimo quattro decimali. » ; en-CH :
    « Line { $line }: unreadable unit price — digits, apostrophe for thousands (1'250), point or comma for decimals, at
    most four decimals. »
  - Les deux clés de la 36-3, réemployées sous le champ, ne changent pas.
- **D7 — Ce qui ne change pas** : le serveur et l'API (un client d'API envoie toujours un décimal à point ; « 1'250 »
  y reste refusé par le serveur) ; `balance.ts`, `amount-ambiguity.ts`, `readableAmount`, `computeLineTotal`,
  `computeInvoiceTotal` ; la quantité ; la facture fournisseur, l'article, le règlement et les autres sites de
  l'inventaire ; l'écran de détail d'une facture.

## Critères d'acceptation

1. **AC1 — Les fonctions pures** (`features/invoices/invoice-helpers.test.ts` ; *étape 0, V1-5 :*
   `features/invoices/unit-price.test.ts`, qui couvre aussi `unitPriceProblem`) : `unitPricePayload` — « 1'250 » →
   `1250`, « 1’250.50 » → `1250.50`, « 1,25 » → `1.25`, « 100.00 » → `100.00`, « 33.3333 » → `33.3333`, « 0.00 » →
   `0.00`, « 0.125 » → `0.125` ; `null` pour « 1.250 », « 12,500 », « 001.250 », « 12'50 », « -5 », « 1e3 », « .5 »,
   « 1.25000 », « » ; `unitPriceFieldValue` — la table de D2 entière, et pour chaque sortie
   `readJournalAmount(sortie).kind !== 'ambiguous'`.
2. **AC2 — Saisie et refus** (`components/invoices/InvoiceForm.test.ts`, brouillon ouvert, `updateInvoiceMock`) : le
   prix retapé « 1.250 » → `invoice-line-unit-price-ambiguous` visible, champ bordé `border-destructive`, total de ligne
   `0.00` ; *Enregistrer* → le bandeau dit « Ligne 1 : prix unitaire ambigu », `updateInvoiceMock` **non appelé** ;
   même chose par *Entrée* dans le champ (soumission du `<form>`). « 12'34 » → `invoice-line-unit-price-unreadable` et
   « Ligne 1 : prix unitaire illisible ». Prix vidé → « Ligne 1 : valeurs numériques invalides » (inchangé).
3. **AC3 — Accepté et normalisé** : « 1'250 » (quantité 2) → aucun message, total de ligne `2’500.00` et total de la
   facture qui le suit ; *Enregistrer* → `updateInvoiceMock` reçoit `unitPrice: '1250'` ; « 1,25 » part `'1.25'`.
   Les allers-retours de #878 (`InvoiceForm.test.ts:694`, `:709`, `:720`) restent verts sans modification.
4. **AC4 — Un prix enregistré n'est pas refusé à tort** : brouillon aux prix serveur `1.2340`, `1.2500`, `12.3450` →
   champs `1.2340`, `1.25`, `12.3450`, aucun message ; *Enregistrer* sans toucher → charges `'1.2340'`, `'1.25'`,
   `'12.3450'`. Même constat après *Recharger* sur conflit (`reloadFromServer`) et pour une ligne ajoutée depuis un
   article au prix `1.2340` (`onProductSelect`) — **un test par site**. Le test existant `:619-635` (`0.0100` → `0.01`)
   reste vert.
5. **AC5 — La course** : `getInvoiceSettingsMock` qui, appelé à l'envoi, remplace le prix par « 1.250 » avant de
   résoudre → aucune requête, bandeau « Ligne 1 : prix unitaire ambigu », bouton de nouveau actif.
6. **AC6 — E2E** (`frontend/tests/e2e/invoices.spec.ts`, test neuf dans « Factures — création brouillon », montage de
   `:469-499`) : nouvelle facture, contact créé par l'API, une ligne « Prix ambigu E2E », quantité `1`, prix `1.250` ;
   écouteur `page.on('request')` armé avant le clic, qui compte les `POST /api/v1/invoices` ; *Créer la facture* →
   `invoice-line-unit-price-ambiguous` visible, bandeau « prix unitaire ambigu », **zéro** `POST`, URL toujours
   `/invoices/new`. Puis prix `1'250` → *Créer la facture* : le corps (`page.waitForRequest`) porte
   `"unitPrice":"1250"`, la réponse est 201, l'URL devient `/invoices/<id>`, et `GET /api/v1/invoices/<id>` (par
   `authedApiContext`) rend `lines[0].unitPrice === "1250.0000"`. Fixtures synthétiques.
7. **AC7 — Textes** : deux clés neuves × quatre locales à l'emplacement de D6 ; `npm run lint-i18n-ownership` vert sans
   entrée neuve (`components/` hors de son parcours) ; `sitesTotal` de `i18n-keys.test.ts` (`:992`, 3040 à `b1cd53d28`)
   **recompté à la source** par la garde elle-même, valeur mise à ce qu'elle rend, sans addition (attendu 3044 : deux
   sites sous le champ, deux dans `validateClient` ; C-e350-18) ; les fallbacks des deux clés de la 36-3 recopiés à
   l'identique de `JournalEntryForm.svelte:511`, `:524`.
8. **AC8 — Manuel** (`docs/manual/fr/user-manual.tex`, § *Création d'une facture*, item **Lignes de facture**, `:1158`) :
   une phrase dit la forme admise du prix, le refus de l'ambiguïté avec l'exemple `1.250`, la forme d'un prix au
   millième rouvert (`1.2340`) et que la quantité n'est pas concernée. Texte proposé, ajouté à la fin de l'item :
   « Le \textbf{prix unitaire} se lit comme le montant d'une écriture (cf. \emph{Saisie d'une écriture en partie
   double})~: point ou virgule décimale, quatre décimales au plus, apostrophe des milliers (\texttt{1'250}).
   *(Étape 0, V1-7 : la virgule finale devient « ~; la saisie à exposant (\texttt{1e3}), sans chiffre avant le point
   (\texttt{.5}) ou après lui (\texttt{5.}), ou précédée d'un signe, n'est pas lue. »)*
   \textbf{\texttt{1.250} est refusé comme ambigu}~: écrivez \texttt{1'250} pour mille deux cent cinquante francs, ou
   \texttt{1.2500} pour un franc vingt-cinq~; la facture ne s'enregistre pas tant qu'une ligne porte un prix ambigu ou
   illisible, et le message nomme la ligne. Un prix enregistré au millième se rouvre sur quatre décimales
   (\texttt{1.2340}), forme qui n'est pas refusée. La quantité n'est pas concernée. » *(Revue P1, A2 / E1 : la phrase
   sur la réouverture devient « Un prix enregistré qui se lirait ambigu une fois écrit au plus court (\texttt{1.234}) se
   rouvre sur quatre décimales (\texttt{1.2340}), forme qui n'est pas refusée~; un autre prix se rouvre sous sa forme
   la plus courte, deux décimales au moins (\texttt{0.125}, \texttt{1000.123}, \texttt{100.00}). » — « 0.1230 » se
   rouvre `0.123`.)* Le PDF se régénère **à
   l'intégration** (fichier d'epic, *Fichiers communs*) ; contrôle aplati :
   `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'est refusé comme ambigu'`
   ≥ 1 (0 dans le `.tex` à `b1cd53d28` : `git grep -c "est refusé comme ambigu" -- docs/manual/fr/user-manual.tex` vide).
9. **AC9 — Rien d'autre ne change** : `git diff b1cd53d28 --stat` ne nomme ni `crates/` hors `kesh-i18n/locales/`, ni
   `journal-entries/`, ni `shared/utils/amount-ambiguity.ts`, ni `supplier-invoices/`, ni `products/` ;
   `balance.test.ts`, `amount-ambiguity.test.ts`, `supplier-invoice-form.test.ts` verts sans modification.
10. **AC10 — Captures** (thème clair, base jetable `kesh_e2e_e363b`, `kesh-gate-logs/e36-3b/captures/`) : la ligne à
    « 1.250 » (message sous le champ et bandeau après *Créer*) ; à « 12'34 » ; à « 1'250 » × 2 (total `2’500.00`) ; un
    brouillon rouvert au prix `1.2340`.

## Tests à écrire, et la preuve du rouge

Écrits d'abord, sur le code non corrigé, journal `/home/gcorbaz/devel/kesh-gate-logs/e36-3b/rouges.log`. Les deux
fonctions neuves sont d'abord posées avec la lecture d'aujourd'hui (`unitPricePayload` → le brut, `null` seulement si
`new Big` lève ; `unitPriceFieldValue` → `readableAmount(raw, 2)`), pour que les rouges portent sur le comportement et non
sur un export absent.

- **Rouges attendus** : AC1 (« 1.250 » non `null`, « 1'250 » `null`, « 1.2340 » → `1.234`) ; AC2 (aucun message,
  `updateInvoiceMock` appelé avec `1.250`) ; AC3 (« 1'250 » refusé « valeurs numériques invalides ») ; AC4 (champ
  `1.234` et `12.345`, aux trois sites — rouge sur l'assertion de forme, car le code d'avant ne refuse pas) ; AC5
  (requête partie) ; AC6 (le `POST` part avec `"unitPrice":"1.250"`).
- **Verts au départ, et dits** : prix vidé (« valeurs numériques invalides ») ; les allers-retours de #878 ; AC4 pour
  `1.2500` → `1.25` (seule valeur de la table d'AC4 déjà juste). Le rouge d'AC4 sur `1.234` établit la forme du champ ;
  le refus à tort qu'il prévient ne se voit qu'**après** le correctif d'AC2, d'où une seconde vérification au vert : la
  mutation M3 à M5 (retour à `readableAmount`) doit faire rougir AC4 par le **message ambigu**, non seulement par la
  forme.
- E2E rouge : sources de la story remises au commit de base par `git restore --source=b1cd53d28 --worktree -- <fichiers>`
  **après** commit du code, frontend construit, test rouge, sources restaurées (`git restore --worktree` + `touch`),
  journal `e2e-rouges.log`.

**Mutations** (code commité d'abord ; scripts versionnés dans `/home/gcorbaz/devel/kesh-gate-logs/e36-3b/mut/` ;
restauration `git restore --worktree -- <fichier>` puis `touch` ; journaux `mut/M*.log`, résumé `mut/resume.log`) :

| # | Mutation | Tuée par |
|---|---|---|
| M1 | `validateClient` ne consulte plus la lecture (retour à `new Big(l.unitPrice)`) | AC2, AC3 |
| M2 | la charge envoie le prix brut (`lines.map(stripUiKey)`) | AC3 (`'1250'`), AC6 |
| M3 | `initLines` appelle `readableAmount(…, 2)` au lieu d'`unitPriceFieldValue` | AC4 (brouillon) |
| M4 | idem dans `reloadFromServer` | AC4 (Recharger) |
| M5 | idem dans `onProductSelect` | AC4 (article) |
| M6 | `unitPriceFieldValue` sans le repli `toFixed(4)` | AC1, AC4 |
| M7 | `unitPricePayload` rend `'0'` pour le vide | AC1 ; AC5 (prix vidé pendant l'attente) *(étape 0, V1-2 : l'AC2 ne la tue pas, le vide étant refusé par la lecture)* |
| M8 | les rangées de message sous le champ retirées | AC2 |
| M9 | `lineTotal` lit le prix brut | AC2 (total `0.00`), AC3 (`2’500.00`) |
| M10 | la garde `null` de la charge retirée (D5) | AC5 |
| M11 | `ambiguous` et `unreadable` échangés dans `validateClient` | AC2 |
| M12 *(étape 0, V1-3)* | bordure `border-destructive` retirée | AC2 |
| M13 *(étape 0, V1-3)* | le vide traité comme illisible dans `validateClient` | AC2 (prix vidé) |

Une mutation équivalente se dit au Dev Agent Record ; les E2E ne se rejouent pas sous mutation (Vitest seul, dit).

## Tâches

- [x] **Étape 0** : relire à la tête du worktree les lignes citées ; rejouer les commandes d'inventaire et recopier leur
      sortie brute à `kesh-gate-logs/e36-3b/etape0-inventaires.log` ; vérifier que la 36-3 n'a pas bougé depuis
      `b1cd53d28` (`git diff b1cd53d28 -- frontend/src/lib/features/journal-entries/balance.ts frontend/src/lib/shared/utils/amount-ambiguity.ts`
      vide) ; relire les LOW du dernier rapport de validation.
- [x] Tests d'abord (AC1 à AC5), rouges consignés ; puis `invoice-helpers.ts` (D2, D3) ; `InvoiceForm.svelte` (D2 à D5) ;
      quatre `messages.ftl` (D6) ; `i18n-keys.test.ts` recompté (AC7).
- [x] E2E (AC6), rouge d'abord.
- [x] Manuel (AC8), `.tex` seul ; captures (AC10).
- [x] Mutations M1-M11 (et M9b, M12 à M14).
- [x] Gates — passage en `review`, remédiation P1 et clôture faits (Dev Agent Record) — (voie 2, `e35-dev-commun.md` § *Quels gates, quand*) : en cours, Vitest ciblé
      (`components/invoices/`, `features/invoices/invoice-helpers.test.ts`, `shared/i18n-keys.test.ts`) par
      `.gate-front.lock`, `npm run check` et `lint-i18n-ownership` ; **passage en `review`** : bases `kesh_e363b` remises
      à neuf sur la voie 2, `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh kesh_ci_e363b` (voie 1, son verrou),
      frontend complet (`check`, `lint-i18n-ownership`, `test:unit`, `build`), pas d'E2E ; **remédiations** :
      `test-fast.sh --ci` et frontend complet ; **clôture** : `gate-ci` et E2E complet sur `kesh_e2e_e363b` recréée
      (voie 2), au dernier commit de code, échecs nommés contre `docs/testing.md` § « Les échecs attendus ». Journaux
      dans `/home/gcorbaz/devel/kesh-gate-logs/e36-3b/`, commit en première ligne de chacun.
- [x] Fin du développement : `Status: review`, clé `review` au `sprint-status.yaml`, prompt de la revue P1
      (`36-3b-facture-prix-unitaire-ambigu-review-prompt-p1.md`) qui nomme le manuel, le PDF (à l'intégration) et les
      captures.

## Note de version

*Rubrique* **Corrigé** (C-e360-11 ; l'orchestrateur l'assemble dans `## [0.17.8] — Non publié`) :

- Facture client : un prix unitaire comme « 1.250 » — un point ou une virgule suivis de trois chiffres, sans séparateur
  de milliers avant — n'est plus lu 1,25 franc en silence : il est refusé comme ambigu, la facture ne s'enregistre pas,
  et le message dit comment l'écrire (`1'250` ou `1.2500`). L'apostrophe des milliers et la virgule décimale sont
  désormais acceptées dans le prix (`1'250`, `1,25`) ; la saisie à exposant (`1e3`) ou sans chiffre avant ou après
  le point (`.5`, `5.`) n'est plus lue. Un prix enregistré qui se lirait ambigu une fois écrit au plus court
  (`1.234`) se rouvre sur quatre décimales (`1.2340`) et n'est pas refusé. (refs #1171)

## Dev Notes

### Hors périmètre

- Les autres sites du symptôme, chacun avec son issue : #1173 (prix d'un article — un article mal saisi passe sur la
  facture sans signal), #1159 (prix fournisseur), #1172 (paiement attendu), #1174 (règlement), #1175 (frais de rappel),
  #1176 (montant minimal), #1177 (filtres du journal), #1050 (soldes de départ). Aucune issue neuve.
- La quantité (hors de la règle) ; le plafond du prix (10⁹), jugé au serveur seul, comme aujourd'hui.
- *(Étape 0, V1-6)* Une quantité ou une description retapée pendant l'attente de `getInvoiceSettings()` part sans
  contrôle client (le serveur la juge) : la garde de D5 ne relit que le prix. Issue demandée au rapport.
- Un prix déjà enregistré trop bas avant la 0.17.8 (« 1.250 » lu 1,25) : rien ne permet de le reconnaître ; aucune
  reprise des données.
- Le serveur et l'API (C-e350-19).

### Splitting — quatre modules

(1) `frontend/src/lib/components/invoices` ; (2) `frontend/src/lib/features/invoices` (`invoice-helpers.ts` et son
test) ; (3) `crates/kesh-i18n/locales` ; (4) `docs/manual/fr`. S'y ajoutent les tests transverses
`frontend/tests/e2e/invoices.spec.ts` et `frontend/src/lib/shared/i18n-keys.test.ts`. `journal-entries/` et
`shared/utils/` sont importés, non modifiés. **Quatre : la règle ne s'applique pas.**

### Fichiers communs avec les autres stories de la phase 1

- `InvoiceForm.svelte` : la 36-1 ne le modifie pas (`git diff $(git merge-base HEAD story/36-1-entree-compte-produit) story/36-1-entree-compte-produit -- frontend/src/lib/components/invoices/InvoiceForm.svelte`
  vide) ; **`InvoiceForm.test.ts`** : la 36-1 y ajoute 71 lignes — fichier commun à signaler à l'intégration (le fichier
  d'epic ne le liste pas) ; les tests de la 36-3b vont dans un `describe` à eux, en fin de fichier.
- `messages.ftl` ×4 : D6, à côté d'`invoice-line-vat-rate-disabled-error` ; la 36-1 ajoute `common-account-no-match`
  au-dessus de `invoice-line-col-revenue-account` (fr-CH `:2649`).
- `i18n-keys.test.ts` (`sitesTotal`) : recompté par la branche, puis après chaque fusion (36-1 en ajoute), sans
  additionner.
- `docs/manual/fr/user-manual.tex` : § *Création d'une facture* (`:1158`), distinct du § *Saisie d'une écriture* (36-3,
  `:559`) et des paragraphes de 36-4 et 36-5 ; PDF après l'union.
- `CHANGELOG.md` : **non écrit** par la branche (C-e360-11) ; la *Note de version* ci-dessus.
- `frontend/tests/e2e/invoices.spec.ts` : aucune autre story de la phase 1 ne l'annonce.

### Ce qui doit rester vrai

- Le serveur reste juge ; rien n'est écrit d'office ; big.js partout, jamais `parseFloat`. Aucun écran autre que le
  formulaire de facture client ne change de lecture. Fixtures synthétiques seulement.
- Un brouillon existant s'ouvre et se renvoie sans perte de précision (#878) et sans refus à tort (AC4).

### E2E

Le défaut se voit dans le navigateur et à la frontière HTTP (le prix part tel que tapé, `:594`) : l'E2E AC6 est exigé.

### Références

- Issue #1171 ; Story 36-3 (#1051, `readJournalAmount`, `isAmbiguousThousands`) ; Story 35-6 (#1026) ; C-e360-17,
  C-e360-11, C-e350-18, C-e350-19 ; #878 (forme lisible du prix rechargé) ; #687 (règle du centime, fournisseur seul).
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel* ; § *Inventorier les sites NON
  RÉSOLUS*.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`), agent de développement de l'Epic 36, en autonomie, prompt
`kesh-gate-logs/prompts/e36-dev.md`, voie 2 (port 3367, bases `kesh_e363b`, `kesh_e2e_e363b` sur `kesh-mariadb-dev2`).

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/e36-3b/`, commit en première ligne de chacun :
`etape0-inventaires.log` (`b837ee8de`) ; `rouges.log` (`b837ee8de` + tests, code non corrigé) ; `cible-1.log`,
`cible-2.log`, `check-1.log`, `check-2.log`, `i18n-crate-1.log` (avant le commit du code) ; `e2e-rouges-r1.log` et
`e2e-rouges.log` (`9a68c086d`, composant remis à `b1cd53d28`) ; `e2e-cible-vert.log` (`9a68c086d`, E2E et manuel non
commités, identiques à `de75ba63d`) ; `captures.log` (`de75ba63d`) ; `mut/resume.log`, `mut/M*.log` (`de75ba63d`,
script `mut/mut-36-3b.py`) ; `test-fast-ci-review.log`, `gate-ci-review.log`, `front-*-review.log` (passage en `review`).

### Completion Notes List

- **Fait** : D1 à D7 ; module `features/invoices/unit-price.ts` (V1-5, C-36-3b-8) ; `InvoiceForm.svelte` — lecture du
  prix dans `validateClient` (ambigu, illisible, vide, dans cet ordre, avant la quantité : C-36-3b-9), rangée du refus
  sous la ligne (`colspan={LINE_TABLE_COLUMNS}` = 7, C-36-3b-10) et bordure, totaux par la charge, charge construite
  dans une boucle à sortie anticipée après `await getInvoiceSettings()` (V1-6) ; deux clés × quatre locales à côté
  d'`invoice-line-vat-rate-disabled-error` ; `sitesTotal` 3040 → 3044, relevé par la garde (`cible-1.log` : « expected
  3044 to be 3040 ») avant d'écrire ; manuel, item **Lignes de facture** (`.tex` seul, PDF à l'intégration).
- **Rouges** (`rouges.log`, 34 échecs sur 93 tests des deux fichiers) : tous les cas annoncés par la fiche. Verts au
  départ, et dits : le prix vidé (AC2), les allers-retours de #878, `1.2500` → `1.25`, `0.1250`, `0.0100` → `0.01`,
  `0.0010` → `0.001`, `1000.0000`, `1.0000`, `100.2000`, `33.3333`, `1250.0000`, la valeur illisible rendue telle
  quelle, et les cas `null` que `new Big` refusait déjà (« 12,500 », « 12'50 », « » ; « 1'250 » est rouge). Une première
  passe rouge (34 + 1) comptait à tort la ligne « prix vidé » : l'aide de test lisait `className`, que la classe de base
  d'`Input` (`aria-invalid:border-destructive`) faisait contenir ; corrigée en `classList.contains` (aide `borde()`)
  avant tout correctif. Le test « article au millième » a été repris après le correctif : son brouillon sans ligne
  ouvrait une ligne neuve sans description, refusée avant le prix (`cible-1.log`) ; il porte désormais une ligne
  complète. Son rouge (`rouges.log`, forme `1.234`) reste celui de la version d'avant ; la mutation M5 le tue dans sa
  forme finale.
- **E2E** : rouge sur le composant de `b1cd53d28` (`e2e-rouges.log` : « création partie : … "unitPrice":"1.250" … ») ;
  vert, `invoices.spec.ts` entier, 21 passés (`e2e-cible-vert.log`). La suite complète n'a **pas** tourné : elle tourne
  à la clôture (`e35-dev-commun.md`).
- **Mutations** (`mut/resume.log`) : M1 à M14 et M9b (totaux de la facture) jouées en Vitest, **15 tuées sur 15**,
  aucune équivalente. Les E2E ne se rejouent pas sous mutation.
- **Captures** (AC10, `captures/`) : `01-prix-1.250-ambigu-sous-le-champ.png`, `02-prix-1.250-ambigu-bandeau-apres-creer.png`,
  `03-prix-12-34-illisible.png`, `04-prix-1-250-fois-2-total-2500.png`, `05-brouillon-rouvert-prix-1.2340.png`
  (brouillon créé par l'API au prix `1.2340`, rouvert en édition : « 1.2340 », `captures.log`). Constat : le bandeau
  d'erreur reste affiché après correction du prix jusqu'au prochain envoi (capture 04) — comportement d'avant la story
  pour tous les refus de `validateClient` ; issue demandée. *(Annoté à la remédiation P1, A3 : la capture 03 le montre
  aussi — champ « 12'34 » et message « Montant illisible » sous lui, bandeau « Ligne 1 : prix unitaire ambigu » resté
  de la tentative d'avant ; issue #1191. Les captures 01, 02 et 05 n'en portent pas.)*
- **Gates du passage en `review`**, au dernier commit de code `de75ba63d` (bases `kesh_e363b` et `kesh_e2e_e363b`
  remises à neuf sur la voie 2, 91 migrations, seed) :
  - `scripts/test-fast.sh --ci` (voie 2) : **5445 exécutés, 5445 passés, 13 ignorés**, 1 instable —
    `kesh-db::mcp_sends_repository lister_rend_au_plus_deux_cents_lignes`, rouge au premier essai
    (`mcp_sends_repository.rs:975` `assertion failed: liste.windows(2).all(|w| w[0].expires_at <= w[1].expires_at)`),
    vert au second ; sans rapport avec la story (aucun fichier `crates/` hors catalogues) ; issue demandée
    (`test-fast-ci-review.log`) ;
  - `scripts/gate-ci.sh kesh_ci_e363b` (voie 1) : **5452 passés, 0 échoué, 17 ignorés** (`gate-ci-review.log`) ;
  - frontend complet : `npm run check` 0 erreur, 29 avertissements (aucun sur les fichiers de la story) ;
    `lint-i18n-ownership` PASS ; `test:unit` **3577 tests, 236 fichiers**, verts ; `build` vert
    (`front-{check,lint-i18n-ownership,test-unit,build}-review.log`) ;
  - **E2E complet et `gate-ci` de clôture : non lancés**, à la clôture.
  - `git diff --stat b1cd53d28 de75ba63d` (AC9) : 14 fichiers, aucun sous `crates/` hors `kesh-i18n/locales/`, ni
    `journal-entries/`, ni `amount-ambiguity.ts`, ni `supplier-invoices/`, ni `products/`.
- **Écarts à la consigne Edit** : aucun sur le code, les tests ni la fiche. Les mutations passent par le script
  versionné `mut/mut-36-3b.py` (permis) ; un `cp` a gardé le premier journal E2E rouge (`e2e-rouges-r1.log`).
- **Hors périmètre, issue demandée** : la quantité et la description retapées pendant l'attente des réglages (V1-6) ;
  le bandeau qui reste après correction. *(Ouvertes depuis : #1190, #1191 ; le test instable : #1189, KF-062.)*
- **Remédiation de la revue P1** (B 0/0/0/4, E 0/0/0/4, A 0/0/0/4 ; `review-p1-{B,E,A}.md`), commit de code
  `6cb932bd6`, `b06fe15f2`, `e7aaa3efa` (dernier commit de code) — détail au Change Log :
  - **Sept tests neufs** au `describe` 36-3b de `InvoiceForm.test.ts` (Vitest 3577 → 3584) : deux « Ligne 2 », trois
    d'ordre, deux de l'attente des réglages.
  - **Mutations** au commit `e7aaa3efa` (`mut/r1-final/resume.log`, script `mut/mut-36-3b.py`, `MUT_ROUND=r1-final`) :
    **18 tuées sur 18** — R1-M1, R1-M2, M3 à M9, M9b, R1-M10, R1-M11, M12, R1-M13, M14, et les neuves R1-M15
    (`{ line: 1 }`), R1-M16 (prix après la quantité), R1-M17 (charge lue avant l'attente). M1, M2, M10, M11, M13 sont
    reprises en R1-* sur les ancres de `readLineUnitPrice`. Les passages intermédiaires sont gardés sous `mut/r1/`
    (`resume-1.log`, `resume-2.log`).
  - **A1** : les trois tests d'AC4 portent leurs assertions en `expect.soft` ; le décompte des messages passe par
    `querySelectorAll` (sous mutation, plusieurs lignes portent le message et `queryByTestId` levait, `mut/r1/` premier
    passage) ; `prixEnvoyes()` rend `[]` sans charge (une assertion rougit au lieu d'une exception). Sous M3, M4 et M5
    (`mut/r1-final/M3.log`, `M4.log`, `M5.log`), le journal nomme les cinq assertions rougies : « forme rouverte »,
    « message sous le champ », « bandeau », « charge partie », « charge envoyée ».
  - **Gates de remédiation** au commit `e7aaa3efa` (bases de la voie 2 remises à neuf, 91 migrations) :
    `scripts/test-fast.sh --ci` **5445 passés, 13 ignorés**, aucun instable (`test-fast-ci-r1.log`) ; `npm run check`
    0 erreur, 29 avertissements ; `lint-i18n-ownership` PASS ; `test:unit` **3584 tests, 236 fichiers** ; `build` vert
    (`front-*-r1.log`) ; E2E ciblé `invoices.spec.ts` **21 passés** (`e2e-cible-r1.log`). **Ni `gate-ci` ni E2E
    complet** : à la clôture.
- **Passe ciblée P2** (Opus, `review-p2-ciblee.md`, 0/0/0/2), tests seuls, commit `1a060fedf` (**dernier commit de
  code**) :
  - **P2-1** : `queryByTestId` inutilisé retiré des trois tests d'AC4.
  - **P2-2** : deux mutations neuves, jouées sur `1a060fedf` (`mut/r2/resume.log`, `MUT_ROUND=r2`) : **R2-M18** (prix lu
    avant la description) tuée par « description vide et prix « 1.250 » : la description est refusée avant le prix » ;
    **R2-M19** (refus de la quantité nulle retiré) tuée par « quantité 0 et prix clair : la quantité reste refusée ».
    Aucune survivante.
- **Gates de clôture**, au commit `1a060fedf` :
  - Vitest du fichier : 55 passés (`r2-cible-1.log`) ; frontend complet : `check` 0 erreur, `lint-i18n-ownership`
    PASS, `test:unit` **3584 tests, 236 fichiers**, `build` vert (`front-*-cloture.log`) ; `test-fast --ci` non exigé
    pour un commit de tests frontend seul (consigne de l'orchestrateur) ;
  - `scripts/gate-ci.sh kesh_ci_e363b` (voie 1) : **5452 passés, 0 échoué, 17 ignorés** (`gate-ci-cloture.log`) ;
  - **E2E complet** sur `kesh_e2e_e363b` recréée à neuf (voie 2, port 3367, `ss -ltnp` 0 avant et après) :
    **411 passés, 7 échoués, 18 ignorés** (`e2e-cloture.log`). Les sept échecs sont exactement ceux de la KF-029
    (#97) de `docs/testing.md` § « Les échecs attendus » : `mode-expert.spec.ts:26`, `:41`,
    `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`. Aucun échec hors liste.

### File List

- `frontend/src/lib/features/invoices/unit-price.ts` (neuf)
- `frontend/src/lib/features/invoices/unit-price.test.ts` (neuf)
- `frontend/src/lib/components/invoices/InvoiceForm.svelte`
- `frontend/src/lib/components/invoices/InvoiceForm.test.ts` (un `describe` en fin de fichier)
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/tests/e2e/invoices.spec.ts`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `docs/manual/fr/user-manual.tex`
- `_bmad-output/implementation-artifacts/36-3b-facture-prix-unitaire-ambigu.md`,
  `36-3b-facture-prix-unitaire-ambigu-review-prompt-p1.md` (neuf), `e36-choix-autonomes-36-3b.md`,
  `sprint-status.yaml`

## Change Log

- 2026-11-14 — Fiche rédigée (Epic 36, phase 1), sur `b1cd53d28` (tête de la 36-3 terminée). Choix C-36-3b-1 à
  C-36-3b-7 au registre `e36-choix-autonomes-36-3b.md`. Clé ajoutée au `sprint-status.yaml` en `ready-for-dev`.
  Validation non lancée.
- 2026-11-14 — **Validation terminée en une passe** : P1 (Sonnet 5.5, complète) 0 CRITICAL / 0 HIGH / 0 MEDIUM / 7 LOW
  (`kesh-gate-logs/e36-3b/validation-p1.md`) ; trend P1 : 7 LOW → clôture (aucun finding au-dessus de LOW). Les sept LOW
  traités à l'étape 0 du développement, ci-dessous.
- 2026-11-14 — **Étape 0** (Opus 5.5), tête `b837ee8de` : lignes citées relues, inventaires rejoués
  (`etape0-inventaires.log` : mêmes sorties que la fiche ; `git diff b1cd53d28 -- balance.ts amount-ambiguity.ts` vide).
  LOW de la validation, chacun avec sa ligne lue :
  - **V1-1** (colspan) — lu `JournalEntryForm.svelte:497` `<td colspan={tableColCount} class="text-xs text-destructive pb-2">` ;
    au code : `colspan={LINE_TABLE_COLUMNS}` (= 7), testé (`colSpan` et nombre de `thead th`), mutation M14. Fiche D3 annotée.
  - **V1-2** (M7) — lu `InvoiceForm.svelte` d'avant, `:509` `try {` : le vide est refusé par la lecture, non par la
    charge ; colonne « Tuée par » corrigée, variante « prix vidé pendant l'attente » ajoutée à l'AC5 (tue M7, M10, M13).
  - **V1-3** (mutations absentes) — M12 (bordure) et M13 (vide traité comme illisible) ajoutées au tableau et jouées.
  - **V1-4** (table de D2) — « 0.0010 », « 1000.0000 », « 1.0000 », « 999.1230 » ajoutés à D2 et à `unit-price.test.ts`.
  - **V1-5** (cycle) — lu `supplier-invoice-form.ts:13` `import { formatExactAmount } from '$lib/features/invoices/invoice-helpers';` ;
    au code : module neuf `features/invoices/unit-price.ts` (C-36-3b-8). D2 et AC1 annotés.
  - **V1-6** (typage, course) — lu `invoices.types.ts:22` (`unitPrice: string`) ; au code : boucle à sortie anticipée
    (`payloadLines`), `npm run check` 0 erreur (`check-2.log`). La quantité et la description retapées pendant l'attente
    restent sans contrôle client : Dev Notes, issue demandée au rapport.
  - **V1-7** (grammaire plus stricte) — lu `user-manual.tex:2505` (« la notation à exposant (`1e3`), que l'écran ne lit
    pas ») ; demi-phrase ajoutée au manuel et à la note de version.
- 2026-11-14 — **Développement** (Opus 5.5) : tests d'abord (`rouges.log`), correctif `9a68c086d`, E2E rouge puis vert et
  manuel `de75ba63d`, captures, mutations (15/15 tuées). Choix C-36-3b-8 à C-36-3b-11. Gates du passage en `review` au
  Dev Agent Record. `Status: review`.
- 2026-11-14 — **Revue de code P1** (trois lentilles B, E, A ; `kesh-gate-logs/e36-3b/review-p1-{B,E,A}.md`) :
  0 CRITICAL / 0 HIGH / 0 MEDIUM / 12 LOW (4 + 4 + 4). Décision de l'orchestrateur (règle « Tarir la source ») : les LOW
  se corrigent dans la story, une passe ciblée P2 relit ce seul correctif. Sort de chaque LOW, prémisse vérifiée au code :
  - **B1 / E2** (aucun témoin au-delà de « Ligne 1 ») — confirmé : `grep -nF "Ligne 2"` vide dans les tests ; deux
    tests à deux lignes (ambigu, illisible) attendent « Ligne 2 : … » et l'absence de « Ligne 1 : prix unitaire » ;
    mutation R1-M15 `{ line: 1 }`.
  - **E3** (ordre de `validateClient`) — confirmé : `grep -nF "quantité > 0"` vide ; trois tests (quantité 0 et prix
    ambigu → prix ; description vide et prix ambigu → description ; quantité 0 et prix clair → quantité) ; mutation
    R1-M16 (prix contrôlé après la quantité).
  - **E4** (attente des réglages : illisible, rétabli) — confirmé : deux appels `retaperPendantLAttente` seulement ;
    « 12'34 » pendant l'attente → aucune requête, bandeau illisible ; « 1'250 » → charge `1250`, non `100.00` ;
    mutations R1-M10 (garde retirée) et R1-M17 (charge lue avant l'attente).
  - **A1** (M3 à M5 ne prouvaient que la forme) — confirmé aux journaux `mut/M3.log:19`, `M4.log:19`, `M5.log:19` ;
    assertions d'AC4 en `expect.soft`, M3 à M5 rejouées (`mut/r1/`).
  - **A2 / E1** (manuel : « au millième » → quatre décimales) — confirmé : `unit-price.test.ts` `['0.1250', '0.125']` ;
    `user-manual.tex:1158` et la note de version récrits (seule une forme qui serait ambiguë passe aux quatre
    décimales). `.tex` seul, PDF à l'intégration.
  - **B2** (repli inatteignable `'Prix unitaire invalide'`, bandeau qui pouvait nommer une autre erreur) — confirmé ;
    au code : `readLineUnitPrice(i, raw)` du composant rend `{ ok, payload }` ou `{ ok: false, message }`, partagé par
    `validateClient` et la boucle de la charge : le bandeau nomme la ligne dont le prix est en cause, sans repli
    (C-36-3b-7 annoté).
  - **B3** (borne serveur `MAX_UNIT_PRICE` non reflétée au client) — noté, sans changement : comportement d'avant la
    story ; le serveur refuse proprement (`invoices.rs:515`), le message s'affiche au bandeau.
  - **B4** (« Montant ambigu » sous le champ du prix) — noté, sans changement : réemploi voulu des textes de la 36-3
    (D3, C-36-3b-4) ; le bandeau dit « prix unitaire ».
  - **A3** (capture 03 non déclarée) — déclarée au Dev Agent Record (#1191).
  - **A4** (papiers) — C-36-3b-2, -3 et -7 annotés au registre ; case « Gates » libellée « clôture à venir ».
- 2026-11-15 — **Passe ciblée P2** (Opus, une lentille sur `git diff 197ba366e e7aaa3efa`,
  `kesh-gate-logs/e36-3b/review-p2-ciblee.md`) : 0 / 0 / 0 / 2 LOW, tous deux dans les tests — P2-1 (`queryByTestId`
  inutilisé, retiré) et P2-2 (deux témoins d'ordre non éprouvés : mutations R2-M18 et R2-M19, tuées). Commit `1a060fedf`,
  tests seuls, aucune ligne de production.
- 2026-11-15 — **Clôture.** Trend de la revue de code : P1 (trois lentilles) B 0/0/0/4, E 0/0/0/4, A 0/0/0/4 → remédiés
  `6cb932bd6`..`e7aaa3efa` (production : `readLineUnitPrice`, manuel) → P2 ciblée (Opus) 0/0/0/2, remédiés `1a060fedf`
  (tests seuls). Gates de clôture au dernier commit de code `1a060fedf` : `gate-ci` 5452 passés, 0 échoué, 17 ignorés ;
  E2E complet 411 passés, 7 échoués (les sept de la KF-029, #97), 18 ignorés ; frontend complet vert (3584 tests) — au
  Dev Agent Record. Le PDF du manuel se régénère à l'intégration. `Status: done`.
