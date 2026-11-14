# Story 36-3 : l'écriture manuelle ne lit plus « 1.234 » comme 1,234 franc (#1051)

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 par l'agent de rédaction des fiches de l'Epic 36 (Opus 5.5), worktree `kesh-36-3`, branche
`story/36-3-ecriture-montant-ambigu`, par le workflow `bmad-create-story`, prompt `kesh-gate-logs/prompts/e36-fiche.md`.
Release **0.17.8**, phase 1. **Aucune migration**, aucun changement serveur ni d'API. Code lu à la tête du worktree
(`4de0bcc7e`). Voie de développement : **voie 2**. Registres : `e36-choix-autonomes-e360.md` (étape 0 de 36-3, C-e360-6
amendé par Guy le 2026-11-14, C-e360-11) ; `e36-choix-autonomes-36-3.md` (C-36-3-1 à C-36-3-6). Validation non lancée.*

## Story

En tant que titulaire qui passe une écriture manuelle — ou qui la prépare par l'assistant TVA achat —, je veux qu'un
montant que Kesh ne peut pas lire sans ambiguïté (« 1.234 » : mille deux cent trente-quatre à l'allemande, ou un franc
vingt-trois ?) soit refusé avec un message qui dit comment l'écrire, et que la forme suisse « 1'234,50 » soit acceptée,
afin qu'aucune écriture équilibrée ne parte au millième du montant voulu sans que je le voie.

## Ce qui se lit au code (`4de0bcc7e`)

**La grammaire du journal** — `frontend/src/lib/features/journal-entries/balance.ts` :
- `:50-53` `amountRegExp` : `` return new RegExp(`^${sign}\\d{1,${o.maxIntegerDigits}}([.,]\\d{1,${o.maxDecimals}})?$`); ``
  — un point **ou** une virgule est toujours un séparateur **décimal** ; aucune apostrophe admise.
- `:61-65` `isValidAmount(raw, options?)` ; `:68-71` `parseAmount` : `return new Big(raw.replace(',', '.'));` —
  « 1.234 » est valide et vaut 1,234.
- `:83-104` `computeBalance(lines)` et `:143-162` `classifyLine(line)` lisent chaque montant par `isValidAmount` puis
  `parseAmount` (`:87`, `:90`, `:94`, `:153`, `:158`).

**L'écran de l'écriture manuelle** — `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte` :
- `:20` `import { computeBalance, classifyLine, formatSwissAmount, isValidAmount } from './balance';`
- `:168-169` `const balance = $derived(computeBalance(lines));` / `const lineStatuses = $derived(lines.map(classifyLine));`
- `:176-184` `canSubmit` : pas de ligne `partial`, deux lignes `valid` au moins, `balance.isBalanced`.
- `:208-209` la charge : `debit: (line.debit === '' ? '0' : line.debit.replace(',', '.')),` (idem `credit`).
- `:283-292` *Ctrl+S* : `if (canSubmit) handleSubmit();` — le raccourci passe par la même garde.
- `:413-428` les deux champs (`type="text" inputmode="decimal"`), bordure rouge si `!isValidAmount(...)`.
- `:479-485` sous la ligne, pour **tout** montant invalide :
  `{i18nMsg('journal-entry-form-max-decimals', 'Maximum 4 décimales')}` — même pour « abc ».

**L'assistant TVA achat** (même module) — `VatPurchaseAssistant.svelte` :
- `:11` `import { isValidAmount, parseAmount } from './balance';`
- `:72-76` `htValid` : `isValidAmount(htAmount) && htAmount.trim() !== '' && parseAmount(htAmount).round(2, Big.roundHalfUp).gt(0)`.
- `:101-110` `insert()` passe `htAmount` **brut** à `buildPurchaseVatLines` et à `lineVatAmount` ;
  `vat-purchase.ts:38`, `:45` le lisent par `parseAmount`. « 1.234 » donne une ligne de charge `1.23` : l'ambiguïté
  disparaît à l'insertion, le formulaire ne la voit plus. Aucun message sous le champ HT (bordure seule, `:166-176`).

**La règle de la 35-6** — `frontend/src/lib/features/bank-accounts/format.ts` :
- `:244` `const GROUP_SEPARATOR = "['’\\s]";` (apostrophes droite et typographique, blancs) ; `:280-332`
  `openingAmountKind`, dont le prédicat d'ambiguïté, `:321-328` :
  ```ts
  if (
  	hasDecimal &&
  	typedFraction.length === 3 &&
  	!grouped &&
  	/^0*[1-9][0-9]{0,2}$/.test(integerDigits)
  ) {
  	return { kind: 'ambiguous' };
  }
  ```
- `routes/(app)/bank-accounts/+page.svelte:261-264` affiche `bank-accounts-opening-error-amount-ambiguous`
  (fr-CH `messages.ftl:1780` ; de-CH, it-CH, en-CH `:1702`), qui conseille « Écrivez 1'234 … ou 1.2340 ».
- `format.test.ts:166-169`, `:210`, `:247` et la table `opening-amount-cases.json` tiennent ce prédicat ; la même table
  est relue par le test de parité Rust (`crates/kesh-api/src/routes/opening_balances.rs`).

**Le serveur** n'est pas modifié (C-e350-19, second commentaire de #1051) ; il rend les montants à quatre décimales
(`crates/kesh-api/src/routes/journal_entries.rs:222-223` `debit: l.debit.to_string()`, colonne `DECIMAL(19,4)` ;
`crates/kesh-api/tests/opening_balances_e2e.rs:1681` `assert_eq!(body["lines"][1]["debit"], "400.0000");`). Une écriture
rouverte en édition (`form-helpers.ts:27-30` `amountToFieldValue`, valeur gardée telle quelle) porte donc « 1.2340 »,
jamais « 1.234 » : la règle ne la refuse pas (AC4 le constate).

**Le message de la ligne** dépend aujourd'hui d'une seule clé, détenue par le formulaire au titre de la dette #30
(`frontend/scripts/lint-i18n-ownership.js:79`
`'src/lib/features/journal-entries/JournalEntryForm.svelte:journal-entry-form-max-decimals',`), et l'E2E existant l'attend
(`frontend/tests/e2e/journal-entries.spec.ts:256-257` `await expect(page.getByText(/Maximum 4 décimales/).first()).toBeVisible();`).

### Inventaire des appelants partagés (`isValidAmount`, `parseAmount`, `computeBalance`, `classifyLine`)

Commande, à la tête du worktree :
`git grep -n "isValidAmount\|parseAmount\|amountRegExp\|computeBalance\|classifyLine\|AmountFormatOptions" -- frontend/src ':!*.test.ts'`
— sortie (hors `balance.ts` lui-même) :

```
frontend/src/lib/features/bank-import/ManualStatementForm.svelte:317:	const badBalance = (raw: string) => raw.trim() !== '' && !isValidAmount(raw.trim(), MANUAL_BALANCE_FORMAT);
frontend/src/lib/features/bank-import/manual-statement.ts:118:	if (t === '' || !isValidAmount(t, format)) return null;
frontend/src/lib/features/bank-import/manual-statement.ts:119:	return parseAmount(t);
frontend/src/lib/features/bank-import/manual-statement.ts:254:	if (t !== '' && isValidAmount(t, format)) return parseAmount(t).toFixed(2);
frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:168:	const balance = $derived(computeBalance(lines));
frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:169:	const lineStatuses = $derived(lines.map(classifyLine));
frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:417:	... !isValidAmount(line.debit) ...
frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:427:	... !isValidAmount(line.credit) ...
frontend/src/lib/features/journal-entries/JournalEntryForm.svelte:479:	{#if !isValidAmount(line.debit) || !isValidAmount(line.credit)}
frontend/src/lib/features/journal-entries/VatPurchaseAssistant.svelte:73:	isValidAmount(htAmount) &&
frontend/src/lib/features/journal-entries/VatPurchaseAssistant.svelte:75:	parseAmount(htAmount).round(2, Big.roundHalfUp).gt(0)
frontend/src/lib/features/journal-entries/vat-purchase.ts:38/39/45/84/92:	parseAmount(...)
frontend/src/lib/features/opening-balances/opening-balances-totals.ts:40:	return isValidAmount(raw) ? parseAmount(raw) : new Big(0);
frontend/src/lib/features/supplier-invoices/supplier-invoice-form.ts:58/63/78/120:	parseAmount(...)
frontend/src/lib/features/supplier-invoices/supplier-invoice-form.ts:152:	!isValidAmount(l.quantity.trim()) ||
frontend/src/lib/features/supplier-invoices/supplier-invoice-form.ts:153:	!isValidAmount(l.unitPrice.trim(), { allowEmpty: false }),
frontend/src/lib/features/supplier-invoices/supplier-invoice-form.ts:283:	expected === '' ? null : isValidAmount(expected) ? decimalForServer(expected) : expected,
frontend/src/routes/(app)/settings/opening-balances/+page.svelte:135:	const balance = $derived(computeBalance(rows));
frontend/src/routes/(app)/settings/opening-balances/+page.svelte:150-151, 166-167, 171, 490, 505, 621, 634:	isValidAmount / parseAmount
```

(Trois lignes abrégées par `...` pour la largeur ; la sortie brute est à recopier au Change Log à l'étape 0.) Tests
appelants : `git grep -ln "isValidAmount\|parseAmount\|computeBalance\|classifyLine" -- 'frontend/src/**/*.test.ts'` →
`bank-import/manual-statement.test.ts`, `journal-entries/balance.test.ts`, `supplier-invoices/supplier-invoice-form.test.ts`.

| Appelant | Décision |
|---|---|
| `JournalEntryForm.svelte` | **dans le périmètre** : lecture du journal (D2), par un lecteur passé à `computeBalance` et `classifyLine` (D3) |
| `VatPurchaseAssistant.svelte`, `vat-purchase.ts` | **dans le périmètre** : HT lu par la lecture du journal, charge normalisée passée aux fonctions pures (D5) ; `vat-purchase.ts` reste inchangé (il reçoit une chaîne déjà normalisée) |
| `bank-import/manual-statement.ts`, `ManualStatementForm.svelte` (relevé manuel) | **inchangé** : `isValidAmount` avec options n'est pas touché ; deux décimales au plus, « 1.234 » y est déjà refusé (`manual-statement.ts:46-49` `MANUAL_LINE_AMOUNT_FORMAT`, `maxDecimals: 2`) |
| `opening-balances/opening-balances-totals.ts`, `settings/opening-balances/+page.svelte` (soldes de départ) | **inchangé**, hors périmètre : **#1050** ; `computeBalance` sans lecteur garde exactement sa lecture d'aujourd'hui (D3) |
| `supplier-invoices/supplier-invoice-form.ts:152-153` (prix unitaire) | **inchangé**, hors périmètre : **#1159** |
| `supplier-invoices/supplier-invoice-form.ts:63`, `:152` (quantité) | **inchangé**, hors de la règle : « 1.234 » y vaut 1,234 unité |
| `supplier-invoices/supplier-invoice-form.ts:283` (montant de paiement attendu) | **inchangé**, hors périmètre : issue à ouvrir (*Inventaire par le symptôme*, ligne 4) |

Aucun appelant ne voit sa lecture changer : `isValidAmount`, `parseAmount` et `amountRegExp` ne sont **pas** modifiés ;
`computeBalance` et `classifyLine` gagnent un paramètre **facultatif** dont le défaut est la lecture d'aujourd'hui.

### Inventaire par le symptôme — tout champ de montant saisi au clavier

Le symptôme est « un montant tapé est lu avec le point ou la virgule pour séparateur décimal, sans contrôle
d'ambiguïté ». Détecteur par lecture (aucun détecteur par exécution praticable : il faudrait un E2E par écran), parti du
champ et non d'une liste de fonctions :

- `git grep -ln 'inputmode="decimal"' -- frontend/src` → `InvoiceForm.svelte` (+ son test), `bank-import/ManualStatementForm.svelte`,
  `invoices/SettleInvoiceDialog.svelte`, `journal-entries/JournalEntryForm.svelte`, `journal-entries/VatPurchaseAssistant.svelte`,
  `supplier-invoices/SupplierInvoiceForm.svelte`, `routes/(app)/bank-accounts/+page.svelte`, `routes/(app)/journal-entries/+page.svelte`,
  `routes/(app)/products/+page.svelte`, `routes/(app)/settings/dunning/+page.svelte`, `routes/(app)/settings/invoicing/+page.svelte`,
  `routes/(app)/settings/opening-balances/+page.svelte`.
- `git grep -n "replace(',', '.')" -- 'frontend/src/**/*.svelte' 'frontend/src/**/*.ts' ':!*.test.ts'` →
  `JournalEntryForm.svelte:208`, `:209` ; `balance.ts:70` ; `products/product-helpers.ts:72` ;
  `settings/opening-balances/+page.svelte:199`, `:200`, `:245`, `:246`.
- `git grep -n 'type="number"' -- 'frontend/src/**/*.svelte'` (angle mort du premier motif : le navigateur lit lui-même) →
  `AutoBackupSection.svelte:361`, `BankProfileForm.svelte:226`, `:319`, `TransactionSplitModal.svelte:313`,
  `RuleFormModal.svelte:223`, `audit-log/+page.svelte:297`.

**Sonde positive** : le premier motif rend bien `JournalEntryForm.svelte` et `VatPurchaseAssistant.svelte`, les deux
sites connus du défaut. **Angles morts écrits** : un champ de montant sans `inputmode="decimal"` ni `type="number"` ;
un montant lu ailleurs que dans un champ (import CSV, CAMT : hors objet, formats machine).

| # | Site (ligne lue) | Même défaut ? | Décision |
|---|---|---|---|
| 1 | `JournalEntryForm.svelte:208-209`, `:417`, `:427` | oui | **périmètre** |
| 2 | `VatPurchaseAssistant.svelte:72-76`, `:105` | oui | **périmètre** |
| 3 | `settings/opening-balances/+page.svelte:150-151`, `:199-200` | oui | hors périmètre, **#1050** |
| 4 | `supplier-invoice-form.ts:152-153` (prix unitaire) ; `:283` (montant de paiement attendu, `isValidAmount(expected) ? decimalForServer(expected)`) | oui | prix : **#1159** ; montant attendu : **issue à ouvrir** |
| 5 | `components/invoices/InvoiceForm.svelte:511` `const price = new Big(l.unitPrice);` (facture **client**, prix unitaire) | oui (« 1.250 » vaut 1,25 ; « 1,250 » illisible pour `Big`) | hors périmètre, **issue à ouvrir** — ⚠️ écran de la première facture de Guy |
| 6 | `features/products/product-helpers.ts:53` `const PRICE_RE = /^(0\|[1-9][0-9]{0,14})(\.[0-9]{1,4})?$/;`, `:72` (prix d'un article) | oui | hors périmètre, **issue à ouvrir** |
| 7 | `features/invoices/SettleInvoiceDialog.svelte:68-76` `centimesDigits` (`new Big(raw.trim()).toFixed()` : « 1.250 » a deux décimales significatives et passe) | oui | hors périmètre, **issue à ouvrir** |
| 8 | `settings/dunning/+page.svelte:104` `Number.isNaN(Number(fFee))` (frais de rappel) | oui | hors périmètre, **issue à ouvrir** |
| 9 | `settings/invoicing/+page.svelte:276` `minimumAmount.trim()` envoyé tel quel (montant minimal de facture) | oui (lu par le serveur) | hors périmètre, **issue à ouvrir** |
| 10 | `routes/(app)/journal-entries/+page.svelte:401-415` (filtres *montant min/max* de la liste) | oui, sans écriture | hors périmètre, **issue à ouvrir** (P4 : un filtre, rien n'est écrit) |
| 11 | `reconciliation/TransactionSplitModal.svelte:313` (`type="number" step="0.01"`) | lecture par le navigateur | hors périmètre : **36-2** l'examine (H2 de C-e360-5) |
| 12 | `bank-import/ManualStatementForm.svelte` | non : deux décimales au plus, « 1.234 » et « 1.250 » refusés | — |
| 13 | `routes/(app)/bank-accounts/+page.svelte` | non : corrigé par la 35-6 | touché seulement par l'extraction (D1) |
| 14 | `AutoBackupSection.svelte:361`, `BankProfileForm.svelte:226`, `:319`, `RuleFormModal.svelte:223`, `audit-log/+page.svelte:297` | non : entiers (intervalle, colonnes, priorité, pagination) | — |

Les lignes 4 (montant attendu) à 10 demandent **une issue par site** à l'orchestrateur (rapport). Leur lecture est
celle de l'étape de rédaction ; l'issue la confirme.

## Décisions

- **D1 — Le prédicat d'ambiguïté de la 35-6 s'extrait vers un utilitaire partagé** (C-e360-6). Fichier neuf
  `frontend/src/lib/shared/utils/amount-ambiguity.ts` :
  `isAmbiguousThousands(parts: { integerDigits: string; fraction: string; grouped: boolean }): boolean`, qui rend
  `fraction.length === 3 && !grouped && /^0*[1-9][0-9]{0,2}$/.test(integerDigits)` — `fraction` est la partie
  décimale **telle que saisie** (vide sans séparateur, ce qui rend `hasDecimal` implicite). `openingAmountKind`
  (`format.ts:321-328`) l'appelle à la place de son test en ligne, sans autre changement : la table
  `opening-amount-cases.json`, `format.test.ts` et le test de parité Rust restent verts **inchangés**. Doc-comment qui
  nomme les deux appelants et renvoie à la table.
  *Écartées* : (a) réemployer `openingAmountKind` entier — sa grammaire admet signes, blancs et souligné que le journal
  refuse (C-e360-6) ; (b) placer le prédicat dans `balance.ts` et l'importer depuis `bank-accounts` — le prompt de Guy
  et C-e360-6 demandent un utilitaire partagé, et `shared/utils/` est le lieu des utilitaires transverses (`iban.ts`,
  `local-date.ts`).
- **D2 — La lecture du journal admet l'apostrophe des milliers et refuse l'ambiguïté** (C-e360-6 amendé). Fonction neuve
  dans `balance.ts` : `readJournalAmount(raw: string): JournalAmountReading`, avec
  `kind: 'empty' | 'clear' | 'ambiguous' | 'unreadable'` et, pour `clear`, `value: Big` et `payload: string`.
  Grammaire, sans rognage (le journal n'en fait pas aujourd'hui, inchangé) :
  `^(?:\d{1,3}(?:['’]\d{3})+|\d{1,15})(?:[.,](\d{1,4}))?$`, partie entière de quinze chiffres au plus une fois les
  apostrophes retirées. Les apostrophes admises sont la droite (U+0027) et la typographique (U+2019, celle que Kesh
  affiche, `balance.ts:137`) ; les groupes sont **stricts**, de trois chiffres (C-36-3-2). `grouped` = la saisie porte
  une apostrophe. Ambiguë si `isAmbiguousThousands(...)` (D1) ; sinon claire, `payload` = chiffres entiers sans
  apostrophe, puis `.` et la fraction telle que saisie si elle existe (« 1'234,50 » → `1234.50` ; zéros de tête
  gardés, comme la 35-6). Exemples : « 1.234 », « 1,234 », « 12.500 », « 001.234 » → `ambiguous` ; « 1'234 »,
  « 1'234,50 », « 1’234.567 », « 1234.567 », « 0.500 », « 1.2340 », « 1.23 » → `clear` ; « 12'34 », « 1''234 »,
  « '1234 », « 1'234' », « 1 234 », « 1.23456 », « 1.234,5 », « abc », « 100, » → `unreadable` ; « » → `empty`.
  *Écartées* : (a) la grammaire de la 35-6 (blancs, signe, souligné, groupes libres) — plus large que la décision de
  Guy, et un groupe libre (« 12'34 ») n'a pas de lecture sûre ; (b) modifier `isValidAmount` ou `amountRegExp` — touche
  le relevé manuel, les soldes de départ et les factures fournisseur (C-e360-6, option (a) écartée) ; (c) les blancs
  comme séparateurs — non demandés, et l'espace d'un copier-coller se confondrait avec une saisie tronquée.
- **D3 — `computeBalance` et `classifyLine` prennent un lecteur facultatif ; le défaut ne change pas.** Type
  `AmountReader = (raw: string) => Big | null` (`null` : hors format ; `''` → `Big(0)`). Lecteur par défaut, exporté
  `defaultAmountReader` : `raw => (isValidAmount(raw) ? parseAmount(raw) : null)` — la lecture exacte d'aujourd'hui,
  que gardent les soldes de départ (`settings/opening-balances/+page.svelte:135`). Lecteur du journal, exporté
  `journalAmountReader` : `readJournalAmount(raw)` → `Big(0)` si `empty`, `value` si `clear`, `null` sinon.
  `JournalEntryForm` passe `journalAmountReader` aux deux. `hasInvalidAmount` et l'état `partial` suivent le lecteur.
  *Écartées* : (a) une seconde paire `computeJournalBalance` / `classifyJournalLine` — duplique la logique d'équilibre
  (DRY) ; (b) normaliser le champ à la frappe (réécrire « 1'234,50 » en « 1234.50 ») — l'utilisateur ne verrait plus ce
  qu'il a tapé, et l'ambiguïté se perdrait comme dans l'assistant aujourd'hui.
- **D4 — L'écran de l'écriture : deux messages, la charge normalisée** (C-36-3-3). Sous une ligne : si l'un des montants
  est `ambiguous`, le message **ambigu** ; sinon, si l'un est `unreadable`, le message **illisible** ; la bordure rouge
  de chaque champ suit `kind` ∉ {`empty`, `clear`}. La clé `journal-entry-form-max-decimals` (« Maximum 4 décimales »,
  fausse pour « abc » et pour « 12'34 », forme que D2 rend possible) est **retirée** des quatre locales et de
  `lint-i18n-ownership.js:79`, remplacée par `journal-entry-form-amount-unreadable`. La charge `:208-209` devient
  `payload` de la lecture (`'0'` pour `empty`) — `canSubmit` garantit qu'aucune ligne non vide n'est `ambiguous` ni
  `unreadable`, *Ctrl+S* compris (`:291`). Textes (quatre locales, à côté du bloc `journal-entry-form-*`, `:312-313`) :
  - `journal-entry-form-amount-ambiguous` — fr-CH : « Montant ambigu : un point ou une virgule suivis de trois chiffres
    peuvent séparer les milliers. Écrivez 1'234 pour mille deux cent trente-quatre, ou 1.2340 pour un franc vingt-trois
    et quatre dixièmes de centime. » ; de-CH, it-CH, en-CH : le texte de `bank-accounts-opening-error-amount-ambiguous`
    (`:1702`) recopié — juste ici, puisque le journal admet désormais « 1'234 » et quatre décimales.
  - `journal-entry-form-amount-unreadable` — fr-CH : « Montant illisible : chiffres, apostrophe des milliers (1'234),
    point ou virgule décimale, quatre décimales au plus. » ; de-CH : « Unlesbarer Betrag: Ziffern, Apostroph als
    Tausendertrennzeichen (1'234), Punkt oder Komma als Dezimaltrennzeichen, höchstens vier Nachkommastellen. » ;
    it-CH : « Importo illeggibile: cifre, apostrofo delle migliaia (1'234), punto o virgola decimale, al massimo
    quattro decimali. » ; en-CH : « Unreadable amount: digits, apostrophe for thousands (1'234), point or comma for
    decimals, at most four decimals. »
  *Écartées* : (a) réemployer la clé `bank-accounts-opening-error-amount-ambiguous` — détenue par `bank-accounts`
  (lint de propriété) ; (b) garder « Maximum 4 décimales » pour tout refus — faux pour les formes groupées mal formées
  que D2 introduit ; (c) changer seulement le texte de `journal-entry-form-max-decimals` — le nom de la clé mentirait.
- **D5 — L'assistant TVA achat lit son HT par la même lecture** (C-36-3-4). Fonction pure neuve dans `vat-purchase.ts` :
  `readHtAmount(raw: string): { kind: JournalAmountReading['kind']; payload: string | null; usable: boolean }` —
  `usable` = `clear` et `value.round(2, Big.roundHalfUp).gt(0)` (la règle de `:72-76`, inchangée). L'assistant :
  `htValid` = `usable` ; `insert()` passe `payload` (et non `htAmount` brut) à `buildPurchaseVatLines` et à
  `lineVatAmount` ; sous le champ HT, le message `journal-entry-form-amount-ambiguous` quand `kind === 'ambiguous'`
  (une entrée de plus au lint, `VatPurchaseAssistant.svelte:journal-entry-form-amount-ambiguous`, comme les clés
  `vat-purchase-*` du même composant au titre de #30). Les autres refus gardent la bordure seule (inchangé).
  *Écartées* : (a) une clé `vat-purchase-ht-ambiguous` — même texte en double dans quatre locales ; (b) une suite de
  composant pour l'assistant — il n'en a pas (`vat-purchase.test.ts` ne teste que les fonctions pures, 35-7 AC4) :
  la fonction pure et l'E2E couvrent le comportement.
- **D6 — Ce qui ne change pas** : le serveur (C-e350-19) ; `isValidAmount`, `parseAmount`, `amountRegExp` ; la quantité
  ; les soldes de départ (#1050) ; les factures fournisseur (#1159) ; le relevé manuel ; `formatSwissAmount` ;
  `openingAmountKind` hors son appel du prédicat. Une écriture rouverte en édition (montants à quatre décimales venus du
  serveur) n'est jamais refusée par la règle (AC4).

## Critères d'acceptation

1. **AC1 — Le prédicat partagé** (`shared/utils/amount-ambiguity.test.ts`, neuf) : ambigu pour
   `{ '1', '234', false }`, `{ '12', '500', false }`, `{ '001', '234', false }` ; non ambigu pour
   `{ '1234', '567', false }` (quatre chiffres), `{ '0', '500', false }` (partie nulle), `{ '1', '234', true }` (groupé),
   `{ '1', '23', false }`, `{ '1', '2340', false }`, `{ '1', '', false }`, `{ '000', '123', false }`.
2. **AC2 — La 35-6 intacte** : `format.ts:321-328` appelle `isAmbiguousThousands` ; `format.test.ts` et
   `opening-amount-cases.json` verts **sans modification** (`git diff <base> -- frontend/src/lib/features/bank-accounts/format.test.ts frontend/src/lib/features/bank-accounts/opening-amount-cases.json` vide) ;
   `bank-accounts` : seul `format.ts` change.
3. **AC3 — La lecture du journal** (`balance.test.ts`) : la table de D2 entière, `kind` et `payload` (« 1'234,50 » →
   `1234.50`, « 1’234.567 » → `1234.567`, « 0.500 » → `0.500`, « 1.2340 » → `1.2340`) ; seize chiffres entiers
   (« 1'234'567'890'123'456 ») → `unreadable`. Non-régression du défaut : `isValidAmount('1.234')` reste `true`,
   `isValidAmount("1'234")` reste `false` ; `computeBalance` **sans lecteur** sur « 1.234 » / « 1.234 » reste équilibré
   (`isBalanced` vrai, totaux 1.234) ; **avec** `journalAmountReader`, `hasInvalidAmount` vrai et `isBalanced` faux ;
   `classifyLine` avec `journalAmountReader` : « 1.234 » → `partial`, « 1'234,50 » → `valid`.
4. **AC4 — L'écran de l'écriture** (`JournalEntryForm.create.test.ts`, `JournalEntryForm.edit.test.ts`) : deux lignes à
   « 1.234 » (débit, crédit) → le message ambigu est affiché, *Valider* désactivé, *Ctrl+S* n'appelle pas
   `createJournalEntry` ; « 12'34 » → le message illisible ; « 1'234,50 » des deux côtés → « ✓ Équilibré »,
   *Valider* appelle `createJournalEntry` avec `debit: '1234.50'` et `credit: '1234.50'` sur les lignes concernées.
   En édition, une écriture à `debit: '1.2340'` (forme du serveur) s'affiche sans message et se renvoie avec
   `'1.2340'`.
5. **AC5 — L'assistant** (`vat-purchase.test.ts`, `readHtAmount`) : « 1.234 » → `ambiguous`, `usable` faux ;
   « 1'234.50 » → `payload` `1234.50`, `usable` vrai ; « 0.004 » → `clear`, `usable` faux (règle du centime
   inchangée) ; `buildPurchaseVatLines` sur le `payload` de « 1'234.50 » à 8.10 → charge `1234.50`, TVA `99.99`
   (1234.50 × 8.10 ÷ 100 = 99.9945, arrondi au centime), contrepartie `1334.49`.
6. **AC6 — Les autres écrans inchangés** : `git diff <base> --stat` ne nomme ni `bank-import/`, ni `opening-balances/`,
   ni `settings/opening-balances/`, ni `supplier-invoices/`, ni `crates/` hors `kesh-i18n/locales/` ; leurs suites
   Vitest (`manual-statement.test.ts`, `supplier-invoice-form.test.ts`, `format.test.ts`) vertes sans modification.
7. **AC7 — E2E de l'écriture** (`frontend/tests/e2e/journal-entries.spec.ts`, test neuf dans « Page écritures —
   saisie ») : « 1.234 » au débit et au crédit → le message ambigu visible, *Valider* désactivé ; *Ctrl+S* ; **aucune**
   requête `POST /api/v1/journal-entries` émise (écouteur `page.on('request')` armé avant la saisie, compté à zéro).
   Puis les deux champs remplis à « 1'234,50 » → « ✓ Équilibré » → *Valider* : le corps de la requête
   (`page.waitForRequest`) porte `"debit":"1234.50"` et `"credit":"1234.50"`, la réponse est 201, et la liste montre
   l'écriture (libellé unique) au montant `1’234.50`. Le test existant `:238-261` attend désormais `/Montant illisible/`
   au lieu de `/Maximum 4 décimales/` (« 10.99999 » reste refusé).
8. **AC8 — E2E de l'assistant** (`frontend/tests/e2e/vat-purchase-assistant.spec.ts`, montage de `:37-64`) : HT
   « 1.234 » → message ambigu visible, *Insérer les lignes* désactivé ; HT « 1'234.50 » → *Insérer* → la première ligne
   porte `1234.50` au débit et l'écriture est équilibrée.
9. **AC9 — Textes** : deux clés neuves × quatre locales, à côté du bloc `journal-entry-form-*` (jamais en fin de
   fichier) ; `journal-entry-form-max-decimals` retirée des quatre locales et du lint ; `lint-i18n-ownership.js` :
   entrées `JournalEntryForm.svelte:journal-entry-form-amount-ambiguous`, `…:journal-entry-form-amount-unreadable`,
   `VatPurchaseAssistant.svelte:journal-entry-form-amount-ambiguous` ; `npm run lint-i18n-ownership` vert ;
   `sitesTotal` de `i18n-keys.test.ts` (3036 à `4de0bcc7e`, `:985`) **recompté à la source** par le test lui-même, la
   valeur attendue mise à ce que la source rend, sans addition (C-e350-18).
10. **AC10 — Manuel** (`docs/manual/fr/user-manual.tex`, § *Saisie d'une écriture en partie double*, `:542-562`) :
    l'item **Montant** (`:559`, `\item \textbf{Montant} : positif uniquement (la colonne débit/crédit indique le sens).`) dit la forme admise et le refus de l'ambiguïté, et une phrase dit que l'assistant TVA
    achat lit son montant HT de la même façon (le manuel le cite à `:3708` « cf. \emph{Saisie d'une écriture} » sans le
    décrire). Texte proposé : « \textbf{Montant} : positif uniquement (la colonne débit/crédit indique le sens)~;
    point ou virgule décimale, quatre décimales au plus, et l'apostrophe des milliers (\texttt{1'234,50}).
    \textbf{Un point ou une virgule suivis d'exactement trois chiffres, après une partie entière de un à trois chiffres
    sans apostrophe, sont refusés comme ambigus} (\texttt{1.234}~: mille deux cent trente-quatre, ou un franc
    vingt-trois~?)~: écrivez \texttt{1'234} ou \texttt{1.2340}, comme pour le solde d'ouverture d'un compte
    (\S\ref{sec:solde-ouverture-compte-bancaire}). L'assistant TVA achat applique la même règle à son montant HT. »
    Le PDF se régénère **à l'intégration**, après l'union (fichier d'epic, *Fichiers communs*) ; contrôle aplati :
    `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -c 'apostrophe des milliers'`
    ≥ 1 (aucune occurrence à `4de0bcc7e` dans le `.tex` : `git grep -c "apostrophe des milliers" -- docs/manual/fr/user-manual.tex` vide).
11. **AC11 — Captures** (thème clair, base jetable, `kesh-gate-logs/e36-3/captures/`) : l'écriture à « 1.234 » (message
    ambigu) ; à « 12'34 » (message illisible) ; à « 1'234,50 » équilibrée ; l'assistant à « 1.234 ».

## Tests à écrire, et la preuve du rouge

Écrits d'abord, sur le code non corrigé, journal `/home/gcorbaz/devel/kesh-gate-logs/e36-3/rouges.log`. Pour que les
rouges portent sur le comportement et non sur un export absent, les fonctions neuves sont d'abord posées avec la
lecture d'aujourd'hui (`isAmbiguousThousands` → `false` ; `readJournalAmount` → `isValidAmount`/`parseAmount`, `payload`
= `raw.replace(',', '.')` ; `readHtAmount` → la règle `:72-76`), puis remplacées.

- **Rouges attendus** : AC1 (cas ambigus) ; AC3 (« 1.234 » `clear` au lieu d'`ambiguous`, « 1'234,50 » `unreadable`,
  `journalAmountReader` équilibré sur « 1.234 ») ; AC4 (pas de message ambigu, *Valider* actif, `createJournalEntry`
  appelé avec `1.234` ; « 1'234,50 » refusé) ; AC5 ; AC7 (le `POST` part avec `"debit":"1.234"`) ; AC8.
- **Verts au départ, et dits** : AC2 (la 35-6, avant et après l'extraction) ; les non-régressions d'AC3 (défaut
  inchangé) ; AC4 en édition (`1.2340`) ; AC6.

**Mutations** (code commité d'abord ; scripts versionnés dans `/home/gcorbaz/devel/kesh-gate-logs/e36-3/mut/` ;
restauration `git restore --worktree -- <fichier>` puis `touch` ; journaux `mut/*.log`, résumé `mut/resume.log`) :

| # | Mutation | Tuée par |
|---|---|---|
| M1 | `readJournalAmount` ne consulte pas le prédicat (tout `clear`) | AC3, AC4, AC7 |
| M2 | `isAmbiguousThousands` rend toujours `false` | AC1, AC2 (`format.test.ts`), AC3 |
| M3 | le prédicat oublie `!grouped` | AC1, AC3 (« 1’234.567 ») |
| M4 | le prédicat admet une partie nulle (`/^0*[0-9]{1,3}$/`) | AC1, AC3 (« 0.500 ») |
| M5 | `payload` garde les apostrophes | AC3, AC4, AC7 (le serveur refuserait) |
| M6 | `JournalEntryForm` appelle `computeBalance` / `classifyLine` sans lecteur | AC4, AC7 |
| M7 | `defaultAmountReader` remplacé par la lecture du journal | AC3 (non-régression du défaut) |
| M8 | l'assistant passe `htAmount` brut à `buildPurchaseVatLines` | AC8 (« 1'234.50 ») |
| M9 | `readHtAmount` ignore `ambiguous` | AC5, AC8 |
| M10 | une ligne ambiguë affiche le message illisible | AC4 |
| M11 | groupes libres (`['’]\d+` au lieu de `['’]\d{3}`) | AC3, AC4 (« 12'34 ») |

Une mutation équivalente se dit au Dev Agent Record.

## Tâches

- [ ] **Étape 0** : relire à la tête du worktree les lignes citées ci-dessus ; relancer les trois commandes d'inventaire
      et recopier leur sortie brute au Change Log ; confirmer que le serveur rend quatre décimales (`journal_entries.rs:222-223`,
      un test qui le montre cité) ; relire les LOW du dernier rapport de validation.
- [ ] Tests d'abord (AC1, AC3, AC4, AC5), rouges consignés ; puis `shared/utils/amount-ambiguity.ts` (D1) et l'appel dans
      `format.ts` ; `balance.ts` (D2, D3) ; `JournalEntryForm.svelte` (D4) ; `vat-purchase.ts` et
      `VatPurchaseAssistant.svelte` (D5) ; quatre `messages.ftl` et le lint (AC9) ; `i18n-keys.test.ts` recompté.
- [ ] E2E (AC7, AC8), rouges d'abord ; le test `:238-261` ajusté.
- [ ] Manuel (AC10), `.tex` seul ; captures (AC11).
- [ ] Mutations M1-M11.
- [ ] Gates (voie 2, `e35-dev-commun.md` § *Quels gates, quand*) : en cours, Vitest ciblé
      (`journal-entries/`, `bank-accounts/format.test.ts`, `shared/utils/amount-ambiguity.test.ts`) par
      `.gate-front.lock`, `npm run check` et `lint-i18n-ownership` ; **passage en `review`** : bases `kesh_e363`
      remises à neuf sur la voie 2, `scripts/test-fast.sh --ci` (le test de parité Rust relit `opening-amount-cases.json`),
      `scripts/gate-ci.sh kesh_ci_e363` (voie 1, son verrou), frontend complet (`check`, `lint-i18n-ownership`,
      `test:unit`, `build`), pas d'E2E ; **remédiations** : `test-fast.sh --ci` et frontend complet ; **clôture** :
      `gate-ci` et E2E complet sur `kesh_e2e_e363` recréée (voie 2), au dernier commit de code, échecs nommés contre
      `docs/testing.md` § « Les échecs attendus ». Journaux dans `/home/gcorbaz/devel/kesh-gate-logs/e36-3/`.
- [ ] Fin du développement : `Status: review`, clé `review` au `sprint-status.yaml`, prompt de la revue P1
      (`36-3-ecriture-montant-ambigu-review-prompt-p1.md`) qui nomme le manuel et les captures.

## Note de version

*Rubrique* **Corrigé** (C-e360-11 ; l'orchestrateur l'assemble dans `## [0.17.8] — Non publié`) :

- Écriture manuelle et assistant TVA achat : un montant comme « 1.234 » ou « 12,500 » — un point ou une virgule suivis de
  trois chiffres, sans séparateur de milliers avant — n'est plus lu au millième en silence : il est refusé comme ambigu,
  avec un message qui dit comment l'écrire (`1'234` ou `1.2340`). L'apostrophe des milliers est désormais acceptée
  (`1'234,50`), comme pour le solde d'ouverture d'un compte. Un montant mal formé est signalé « illisible », et non plus
  « Maximum 4 décimales ». (refs #1051)

## Dev Notes

### Hors périmètre

- Soldes de départ des réglages : **#1050** ; prix unitaire des factures fournisseur : **#1159** ; la quantité (hors de la
  règle). Sites 4 (montant attendu), 5 à 10 de l'inventaire par le symptôme : une issue par site, à ouvrir par
  l'orchestrateur. Le site 5 (prix unitaire de la **facture client**, `InvoiceForm.svelte:511`) est sur le chemin de la
  première facture de Guy : signalé au rapport pour qu'il en fixe le rang.
- Le serveur (`Decimal::from_str`) : C-e350-19.
- Le message illisible pour l'assistant (bordure seule aujourd'hui) : inchangé.

### Splitting — cinq modules, à la limite

Modules touchés au sens du `CLAUDE.md` : (1) `frontend/src/lib/features/journal-entries` ; (2)
`frontend/src/lib/shared/utils` ; (3) `frontend/src/lib/features/bank-accounts` (une ligne, l'appel du prédicat) ;
(4) `crates/kesh-i18n/locales` ; (5) `docs/manual/fr`. S'y ajoutent des fichiers d'outillage et de test transverses, qui
suivent les modules : `frontend/scripts/lint-i18n-ownership.js`, `frontend/src/lib/shared/i18n-keys.test.ts`,
`frontend/tests/e2e/{journal-entries,vat-purchase-assistant}.spec.ts`. **Cinq : la règle ne s'applique pas**, sans marge.

### Fichiers communs avec les autres stories de la phase 1 (tableau du fichier d'epic)

- `JournalEntryForm.svelte`, `balance.ts` : 36-3 seule. `JournalEntryForm.svelte` consomme `AccountAutocomplete`, que
  36-1 modifie (lignée A : 36-3 part de 36-1 fusionnée, ou en parallèle de 36-2 si 36-1 l'est — C-e360-4).
- `messages.ftl` ×4 : à côté du bloc `journal-entry-*` (`:312-313`), jamais en fin de fichier.
- `i18n-keys.test.ts` (`sitesTotal`) : recompté à la source par la branche, puis après chaque fusion, sans additionner.
- `docs/manual/fr/user-manual.tex` : § *Saisie d'une écriture* (`:542-562`), distinct des paragraphes de 36-4 et 36-5 ;
  PDF après l'union.
- `CHANGELOG.md` : **non écrit** par la branche (C-e360-11) ; la *Note de version* ci-dessus.
- Fichier non listé au tableau de l'epic : `frontend/scripts/lint-i18n-ownership.js` (et `shared/utils/`, neuf) —
  signalés au rapport ; aucune autre story de la phase 1 ne les annonce.

### Ce qui doit rester vrai

- Le serveur reste juge ; rien n'est écrit d'office ; aucun écran autre que l'écriture manuelle et l'assistant ne change
  de lecture. Big.js partout, jamais `parseFloat`. Fixtures synthétiques seulement.

### Références

- Issue #1051 ; Story 35-6 (#1026, `openingAmountKind`) ; C-e360-6 (amendé le 2026-11-14), C-e350-19 ; #1050, #1159.
- `CLAUDE.md` : § *Recette visuelle* ; § *Le prompt d'une passe doit NOMMER le manuel* ; § *Inventorier les sites NON
  RÉSOLUS*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-14 — Fiche rédigée (Epic 36, phase 1), sur `4de0bcc7e`. Choix C-36-3-1 à C-36-3-6 au registre
  `e36-choix-autonomes-36-3.md`. Validation non lancée.
