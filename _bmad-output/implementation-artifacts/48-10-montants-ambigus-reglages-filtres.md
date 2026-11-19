# Story 48-10 : les montants « 1.250 » des réglages, du catalogue et des filtres — prix d'article, frais de rappel, montant minimal, filtres des écritures (#1173, #1175, #1176, #1177)

Status: review

*Fiche **reprise de la 36-18b** (`36-18b-montants-ambigus-reglages-filtres.md`, créée le 2026-11-15 par le découpage de la
36-18, `backlog`, jamais validée, écrite à `fed34a9c7`) : la 36-18b est `superseded` au `sprint-status`, la 48-10 la remplace.
Rédigée le 2026-11-19 (Sonnet 5.5), worktree `kesh-48-10`, branche `story/48-10-montants-ambigus-reglages-filtres`, **code relu à
`af4e0e313`** (tête de `epic/48-corrections-0-20-0`, `main` = v0.19.0 plus la 48-13 et la 48-15 fusionnées). Release **0.20.0**,
Epic 48, voie B (frontend, manuels), sortie à la coupure du jeudi 2026-11-26 : **la 48-10 sort après la 48-14, la 48-2, puis, selon
l'avancement, 49-3, 49-2, 49-1c, 48-1** (plan de la release). **Aucune migration**, aucun changement serveur ni d'API (si le
développement en découvre le besoin, il s'arrête et le dit). Bases `kesh_e4810`, `kesh_e2e_e4810` (voie B), `kesh_ci_e4810`
(`gate-ci`, voie A). **Les décisions communes sont celles de la 36-18a** (`36-18a-montants-ambigus-livres.md`, `done`, D1 à D4 et
D9), **dont le code est dans `main`** : cette fiche les réemploie et ne les répète pas. Inventaires bruts :
`/home/gcorbaz/devel/kesh-gate-logs/e4810/fiche-inventaires.log`. Validation P1 faite et remédiée, P2 remédiée (voir le Change Log) ; validation terminée (P3 ciblée, 0/0/0/8, LOW corrigés dans le texte).*

## Story

En tant que titulaire qui tient ses livres réels dans Kesh, je veux que le prix d'un article, les frais d'un niveau de rappel, le
montant minimal d'une facture et les bornes du filtre des écritures se lisent comme le montant d'une écriture — « 1.250 » refusé
comme ambigu, « 1'250 » accepté —, sans que rien de juste jusqu'ici devienne refusé et sans que ma saisie s'efface, afin qu'un
prix d'article mille fois trop bas ne passe pas sur mes factures et que le filtre ne me montre pas une liste fausse sans le dire.

**Usage réel** : Guy tient ses livres réels dans Kesh depuis le 2026-10-31 et utilise le catalogue d'articles, les rappels (frais)
et la facturation. Un montant lu mille fois trop petit est une erreur comptable silencieuse : un prix d'article « 1.250 » devient
1,25 franc sur toute facture qui reprend l'article. Les AC prouvent, **aux quatre sites**, le refus explicite d'un montant ambigu,
sans perte de saisie, et qu'aucune requête d'écriture ne part.
**Priorités sur GitHub** : #1173 **P1**, #1176 **P1**, #1175 **P3**, #1177 **P4** : **#1173 et #1176 sont relevées en P1 par décision de Guy du 2026-11-19** (un prix ou un minimum lu mille fois trop bas est une erreur comptable silencieuse, critère P1 du `CLAUDE.md`) ; le fichier d'épique dit encore « P3, P4 », à jour au prochain recompte.

## Ce qui se lit au code (`af4e0e313`)

### La règle, déjà livrée par les 36-3, 36-3b et 36-18a — réemployée, non réécrite

Contrairement à la 36-18b, **tout est dans `main`** :

- **Le prédicat** — `frontend/src/lib/shared/utils/amount-ambiguity.ts:38` `export function isAmbiguousThousands(parts: AmountParts): boolean {`.
- **Le lecteur** — `frontend/src/lib/features/journal-entries/balance.ts:110` `export function readJournalAmount(raw: string): JournalAmountReading {`,
  grammaire `:90` `const JOURNAL_AMOUNT_RE = /^([1-9]\d{0,2}(?:['’]\d{3})+|\d{1,15})(?:[.,](\d{1,4}))?$/;`, aucun rognage, aucun signe.
- **Les aides de la 36-18a (D1)** — `balance.ts:173` `export function amountEntryPayload(raw: string): string | null {` (charge d'une lecture claire ;
  `null` pour l'ambigu, l'illisible **et le vide**) ; `balance.ts:183` `export function amountEntryProblem(raw: string): 'ambiguous' | 'unreadable' | null {`.
- **Le message commun (D3, D9)** — `frontend/src/lib/features/journal-entries/amount-entry-message.ts:18`
  `export function amountEntryMessage(problem: 'ambiguous' | 'unreadable', max: 2 | 4): string {` ; clés `common-amount-ambiguous`
  (`crates/kesh-i18n/locales/fr-CH/messages.ftl:2842`) et `common-amount-unreadable` (`:2843`, `{ $max }`). **Neuf depuis la 36-18b** :
  l'aide du message n'existait pas, la 36-18b devait écrire les `data-testid` et l'interpolation elle-même.
- **Un site de l'Epic 43 a déjà le même motif** : `routes/(app)/settings/invoicing/+page.svelte:76-100` (`groupMaxReading`,
  `groupMaxError` → `amountEntryMessage(r.problem, 2)`), champ `:763-786` (bordure `border-destructive`, `aria-invalid`,
  `aria-describedby`, `id` du message, `role="alert"`), et `frontend/src/lib/features/invoices/group-settlement-max.ts:20`
  `readGroupMax` ; l'envoi « seulement si modifié » `+page.svelte:329` (`groupMaxChanged`). **Le modèle de l'écran du montant
  minimal est là, sur la même page.**

### Les quatre sites

**1. Prix d'un article (#1173)** — `frontend/src/lib/features/products/product-helpers.ts` :
- `:53` `const PRICE_RE = /^(0|[1-9][0-9]{0,14})(\.[0-9]{1,4})?$/;`, `:71-73` `normalizePriceInput` (`return raw.trim().replace(',', '.');`),
  `:82-87` `classifyPriceInput` (`empty` / `negative` / `invalid` / `ok`) — « 1.250 » `ok`, « 1,250 » normalisé `1.250` `ok` ; « 1'250 » `invalid` ;
- `:61-65` `isValidPrice` : **aucun appelant de production** (`git grep -n "isValidPrice\|PRICE_RE" -- frontend/src` : le fichier lui-même, ses doc-comments et son test) ;
- page `frontend/src/routes/(app)/products/+page.svelte` : `:328-341` `formValidation` (nom requis, nom trop long — **48-15** —,
  puis `classifyPriceInput(formPrice)` `:332` : `product-error-price-required`, `-negative`, `-invalid`) ; `:373` `unitPrice: normalizePriceInput(formPrice),` ;
  champ `:653-665` (`id="form-price"`, sans `aria-invalid` ni message propre) ; le motif de refus s'affiche **au pied du formulaire**,
  `:726-730` (`{#if formError}` … `{:else if formTouched && formValidation}`), et le bouton est `disabled={formSubmitting || !!formValidation}` `:736` ;
- rouvert `:306-311` `const fixed = new Big(p.unitPrice).toFixed(4);` `formPrice = fixed.replace(/(\.\d{2})0+$/, '$1');` — sortie à deux ou
  quatre décimales, **jamais trois** (« 1.2340 » reste `1.2340`, « 1.2500 » → `1.25`, « 0.1230 » reste `0.1230`) : jamais ambiguë, **à prouver par test** (AC1) ;
- serveur `crates/kesh-api/src/routes/products.rs:210` `if !scale_within(&unit_price, MAX_DECIMAL_SCALE) {` : « 1.250 » passe, lu 1,25, puis repris par chaque ligne de facture qui emploie l'article.
- **Changé depuis la 36-18b** : la 48-15 a retiré le `maxlength` du nom et ajouté la garde de 255 caractères dans `formValidation` (`:330-331`) ; l'ordre des
  gardes (nom, puis prix) **ne change pas**, la 48-10 n'y touche que la branche du prix.

**2. Frais de rappel (#1175)** — `frontend/src/routes/(app)/settings/dunning/+page.svelte` (306 lignes, **inchangée depuis la 36-18b**) :
`:104` `if (fFee.trim() === '' || Number.isNaN(Number(fFee))) {` ; `:111` `await createDunningLevel({ delayDays: delay, feeAmount: fFee });` et
`:116` `feeAmount: fFee,` — envoyé **brut** ; champ `:284` ; rouvert `:88` `fFee = level.feeAmount;` (colonne `DECIMAL(7,2)`,
`crates/kesh-db/migrations/20260714000001_dunning_config.sql:32` : jamais ambigu) ; refus affiché `:291` `formError` (`data-testid="dunning-form-error"`, `role="alert"`).
⚠️ **L'issue dit plus que le code** (commentée par l'orchestrateur) : le serveur refuse déjà « 1.250 »
(`crates/kesh-api/src/routes/dunning_levels.rs:82-86` `if !scale_within(fee, 2) {` … « Les frais de rappel ne peuvent avoir plus de 2 décimales. »,
**sans** `normalize()`). Le défaut réel : ce message pousse à écrire `1.25` pour mille deux cent cinquante ; « 1'250 » est refusé
(`Number("1'250")` est `NaN`) ; `Number("1e3")` et `Number(" 1.25 ")` passent et partent bruts. **Aucun test de composant** pour la page (le test est neuf).

**3. Montant minimal d'une facture (#1176)** — `frontend/src/routes/(app)/settings/invoicing/+page.svelte` (817 lignes ; **la 43-7 y a ajouté le plafond du
règlement groupé**, `:76-100`, `:329`, `:763-786`) : `:323` `minimumInvoiceAmount: minimumAmount.trim() === '' ? null : minimumAmount.trim(),` envoyé tel
quel ; champ `:735-742` (`data-testid="settings-minimum-invoice-amount"`, sans message ni `aria-invalid`), aide `:743-748` ; rouvert `:254` et `:360`
(`minimumAmount = s.minimumInvoiceAmount ?? '';`) ; serveur `crates/kesh-api/src/routes/company_invoice_settings.rs:521`
`|| !crate::routes::limits::scale_within(&min.normalize(), 2))` : « 1.250 » normalisé `1.25`, **accepté**, lu un franc vingt-cinq (colonne
`DECIMAL(19,4)`, `20261001000002_invoice_minimum_amount.sql:14` ; **forme rendue relevée à l'étape 0**, AC3).

**4. Filtres de montant de la liste des écritures (#1177, P4)** — `frontend/src/routes/(app)/journal-entries/+page.svelte` (669 lignes) :
`:96-97` `amountMin: amountMin || undefined,` / `amountMax: amountMax || undefined,` envoyés bruts dans `buildQuery` `:93-106` ; l'URL les reprend
par `$effect` `:201-208` (`serializeQuery`, `frontend/src/lib/features/journal-entries/query-helpers.ts:29-34`, avec `.trim()`) ; initialisés depuis l'URL
`:162-163` (`parseQueryFromUrl`, `query-helpers.ts:81-85`, sans validation) ; gestionnaires `:227-235` (`debouncedLoad`) ; champs `:427-452`
(`id="filter-amount-min"` / `-max`). Le serveur lit « 1.250 » 1,25 et refuse « 1'250 » ou « 1,25 » en 400
(`crates/kesh-api/src/services/ecritures.rs:96-103` `analyser_montant`, `Decimal::from_str`), que la page tait (`:127-130` `entries = []; total = 0;`).
Rien n'est écrit : seul le filtre est faux. **Les filtres sont les seuls à ne pas arrêter une écriture : la saisie garde son texte, c'est l'envoi du filtre qui change.**

### Inventaire par le symptôme (commandes et sorties : `kesh-gate-logs/e4810/fiche-inventaires.log`)

Détecteur par lecture, parti du **symptôme** (un champ où l'on tape un montant) et non d'une liste de noms : **tout** `inputmode="decimal"`, tout
`type="number"`, tout `bind:value` d'un nom de montant (`Amount|Price|Fee|Montant|Balance|Rate|Debit|Credit|Max|Min`), puis, pour chaque champ trouvé, la
lecture de sa saisie ; `Number(`/`parseFloat(` ne servent qu'à compléter (cent lignes de bruit : identifiants, dates, indices). **Sonde positive** : le
premier motif rend `settings/dunning/+page.svelte:284`, site connu. Trente-sept champs `inputmode="decimal"`, classés :

| Site | Lecture | Décision |
|---|---|---|
| `products/+page.svelte:661` | `PRICE_RE` | **48-10** (#1173) |
| `settings/dunning/+page.svelte:284` | `Number()` | **48-10** (#1175) |
| `settings/invoicing/+page.svelte:738` (minimum) | brut | **48-10** (#1176) |
| `journal-entries/+page.svelte:433`, `:446` | brut | **48-10** (#1177) |
| `settings/invoicing/+page.svelte:766` (écart du règlement groupé) | `readGroupMax` → `readJournalAmount` | déjà corrigé (43-7) |
| `opening-balances/+page.svelte` (sept champs), `SupplierInvoiceForm.svelte:815`, `:879`, `SettleInvoiceDialog.svelte:330` | `journalAmountReader`, `amountEntryProblem` | corrigé (36-18a) |
| `InvoiceForm.svelte:984` (prix), `JournalEntryForm.svelte` (montants `:910-940`), `VatPurchaseAssistant.svelte:301`, `:373`, `ConversionAssistant.svelte` (montants), `TransactionSplitModal.svelte:413` (`lireMontant` → `readJournalAmount`) | `readJournalAmount` | corrigé (36-3, 36-3b, 36-2, 35-6) |
| `JournalEntryForm.svelte:1005`, `:1020` (cours), `ConversionAssistant.svelte:359` (cours) | `readRate` (`fx.ts:137-144`, `isValidAmount`) | **hors de la règle** (cours, angle mort assumé) |
| `bank-accounts/+page.svelte:720` (solde d'ouverture) | `openingAmountKind` (`bank-accounts/format.ts:283`, L7 : l'ambigu est refusé) | corrigé (35-6, 36-4) |
| `ManualStatementForm.svelte:388`, `:403`, `:482`, `:492` | `isValidAmount(…, MANUAL_BALANCE_FORMAT)` : **deux décimales seulement**, « 1.250 » est refusé (non lu 1,25) | hors de la règle (déjà refusé), dit à la 36-18a |
| `InvoiceForm.svelte:979` (quantité client, lue par `new Big(l.quantity)`, `:610`), `SupplierInvoiceForm.svelte:871` (quantité), quantités des factures fournisseur (`decimalForServer`) | — | **hors de la règle** (C-e360-6, angle mort assumé) |
| `exchange-rates/RateCell.svelte:153` (cours) | `isValidRateInput` (`exchange-rates-grid.ts:70`) | **hors de la règle** : un cours a légitimement trois décimales (« 0.935 »), le point ne sépare jamais des milliers ; angle mort assumé |
| `settings/vat-rates/+page.svelte:287`, `:331` (`type="number"`) | taux en pour cent | hors de la règle (un taux, deux décimales) |
| `type="number"` restants (indices de colonnes, jours, priorité, sauvegarde) | entiers | hors de la règle |

**Angles morts** : un champ sans `inputmode="decimal"`, sans `type="number"` et dont la variable ne porte aucun nom de montant ; un montant lu hors d'un champ (CSV, CAMT, QR). **Aucun site
hors des quatre issues : aucune issue neuve.** Trois angles morts **assumés, sans issue** (décisions de l'orchestrateur) : les **quantités** (une quantité a légitimement des décimales,
« 1.250 » pour une heure et quart n'est pas un séparateur de milliers : C-e360-6, hors de la règle) ; les **cours de change** (trois décimales légitimes, « 0.935 ») ; la **lecture côté serveur et
l'API externe**, qui lit toujours « 1.250 » 1,25 (C-e350-19, hors périmètre).

## Décisions

*Lecture, aides, forme rouverte, messages communs et rognage : D1 à D4 et D9 de la 36-18a, réemployés sans modification — D3 compris dans sa forme
remédiée (tout champ fautif porte la bordure, `aria-invalid="true"` et `aria-describedby` vers l'`id` de son message). Les deux seuls écarts à la 36-18b
viennent du code neuf : l'aide `amountEntryMessage` (aucun texte de message écrit dans les écrans) et le modèle `groupMax`.*

- **D1 — Prix d'article (#1173)** : `classifyPriceInput` lit la saisie rognée : vide → `empty` ; signe `-` suivi d'un chiffre → `negative` (avant la lecture, message inchangé) ;
  puis `readJournalAmount` : `ambiguous` → `ambiguous` (neuf) ; `unreadable` → `invalid` (`product-error-price-invalid`, inchangé) ; `clear` → `ok`. **Signature fixée** :
  `normalizePriceInput(raw): string | null` rend `amountEntryPayload(raw.trim())` (`null` pour le vide, l'ambigu et l'illisible), et l'appelant (`:373`) ne l'atteint qu'après `formValidation` vide,
  avec une **garde** (`if (price === null) return;`). `isValidPrice` et `PRICE_RE`, sans appelant de production, sont **retirés** avec leurs tests ; « 007.50 » et « 01 » deviennent acceptés
  (zéros de tête gardés dans la charge). **Affichage (D3 de la 36-18a)** : tout refus du prix **autre que le vide** (`ambiguous`, `invalid`, `negative`) se pose **sous le champ** (`id="form-price-error"`,
  `data-testid="product-form-price-error"`, champ en `aria-invalid="true"`, `aria-describedby`, bordure `border-destructive`) — l'ambigu avec `amountEntryMessage('ambiguous', 4)`, l'illisible et le
  négatif avec leurs clés actuelles. **`formValidation` rend ce même texte** (il commande le bouton `disabled` `:736` et la garde `:363-364`) ; le pied du formulaire (`:726-730`) garde
  `formValidation` pour **le nom** (vide, trop long) **et pour le prix vide** (`product-error-price-required`, qui n'a pas de lieu sous le champ : seuls `ambiguous`, `invalid` et `negative` passent
  sous le champ) ; après **Entrée** (`formError` = ce texte, rouge, `:726`), il **ne répète pas** le message du prix déjà affiché sous le champ (`{#if formError && formError !== priceMessage}`).
  **`priceMessage`** = le message affiché sous le champ (`''` pour le prix vide et pour un prix lisible). **À la saisie du prix** (`oninput`, `:663`), `formError` n'est remis à `''` **que s'il égale `priceMessage`** (`if (formError === priceMessage) formError = ''`) : une correction après Entrée ne laisse aucun message du prix périmé, et le refus du serveur (doublon de nom) ou le message des taux de TVA, qui ne viennent pas du prix, restent. Les scénarios « Entrée » (D1, AC12, M19) sont **défensifs** : un bouton de soumission désactivé n'envoie pas le formulaire par Entrée dans le navigateur, le test passe par `fireEvent.submit` / `requestSubmit()`. Rouvert : **inchangé**, jamais ambigu (AC1).
  *Écartées* : garder `PRICE_RE` et y ajouter le prédicat (une seconde grammaire) ; remplacer `product-error-price-invalid` par `common-amount-unreadable` (hors du défaut, quatre locales de plus) ;
  laisser le prix au seul pied du formulaire (hors de la D3 commune).
- **D2 — Frais de rappel (#1175)** : **D3 de la 36-18a réemployée telle quelle** : la lecture est **dérivée de la saisie** (`feeReading = $derived(readJournalAmount(fFee.trim()))`), non
  jugée au seul envoi ; vide → `dunning-form-error-fee` (inchangé, à l'envoi) ; un signe en tête → `unreadable` (message commun) ; `ambiguous` / `unreadable` → `amountEntryMessage(problem, 2)`
  **sous le champ des frais** (`<p id="{uid}-fee-error" role="alert" data-testid="dunning-form-fee-error">`, champ en `aria-invalid="true"`, `aria-describedby="{uid}-fee-error"`, bordure
  `border-destructive`), **hors** du bloc commun `formError` (`+page.svelte:291`, sans `id`, pour le délai, le vide et les refus du serveur, **inchangé**) ; `submitForm` s'arrête avant l'appel
  tant que la lecture n'est pas claire ; `clear` → `feeAmount: payload`, à la création **et** à la modification. Le formulaire **reste ouvert** avec la saisie. Bornes (0 à `FEE_MAX`) et
  échelle au serveur. *Écartée* : le message dans `formError` (le bloc ne désigne pas le champ).
- **D3 — Montant minimal (#1176)** : sur le modèle de `groupMax` de la même page : `minimumReading = $derived(readJournalAmount(minimumAmount.trim()))` ; vide → `null`
  (inchangé) ; `ambiguous` / `unreadable` → message `amountEntryMessage(problem, 2)` sous le champ (`data-testid="settings-minimum-invoice-amount-error"`, `id`,
  `aria-describedby`, bordure, `aria-invalid`), le **bouton Enregistrer désactivé** (`disabled={submitting || … || minimumBlocked}`, avec `minimumBlocked = minimumReading.kind === 'ambiguous' || minimumReading.kind === 'unreadable'` ; `minimumReading` est toujours un objet, on ne le nie pas) tant que la lecture n'est **ni claire ni vide** (un minimum vide, « aucun seuil », reste enregistrable) (comme `!groupMaxReading.ok`, `+page.svelte:804-805`),
  et `save()` s'arrête **avant** l'appel, par `notifyError` du même texte, comme pour le plafond ; `clear` → `payload`.
  L'**échelle** (plus de deux décimales significatives) reste jugée par le serveur, comme avant ; elle n'est pas dupliquée ici (hors du défaut). Contrairement au
  plafond du règlement, le minimum est envoyé à chaque enregistrement (comportement inchangé) : sa forme rouverte ne peut pas être ambiguë (AC3).
  *Écartée* : réemployer `readGroupMax` (borne `100`, texte du plafond).
- **D4 — Filtres (#1177, P4, en dernier)** (C-36-18-7) : `buildQuery` envoie la charge d'une borne claire (requête et URL, « 1'250 » → `amountMin=1250`) et **omet**
  une borne ambiguë ou illisible ; **le champ garde sa saisie** (aucune réécriture de ce que l'utilisateur tape). Sous le champ, `amountEntryMessage(problem, 4)`
  suivi de `journal-entries-filter-amount-not-applied`, bordure, `aria-invalid="true"` et `aria-describedby` vers l'`id` du message, `data-testid="journal-entries-filter-amount-min-problem"` /
  `-max-problem`. Une URL `amountMin=1.250` (signet ancien) affiche le message et n'applique pas la borne ; le `$effect` qui recopie l'état dans l'URL retire alors la borne. Un signet
  `amountMin=1,25` (refusé en 400 jusqu'ici) s'applique désormais. Le filtre vide, ou fait de blancs, n'envoie rien (inchangé). **Un signe n'est pas lu** : « -5 » est `unreadable`
  (`balance.ts:90`, aucun signe), la borne est omise avec le message illisible et la phrase « pas appliqué » — elle n'atteint plus le serveur, qui ne juge plus que `min > max`.
  *Écartées* : ne pas recharger (filtre périmé tu) ; envoyer quand même (le défaut) ; réécrire le champ en `1250`.
- **D5 — Texte** (quatre locales, à la suite de `journal-entries-filter-amount-max` : fr-CH `messages.ftl:409`, de-CH, it-CH, en-CH `:415`) :
  `journal-entries-filter-amount-not-applied` — fr-CH « Ce filtre n'est pas appliqué tant que le montant n'est pas corrigé. » ; de-CH « Dieser Filter wird erst
  angewendet, wenn der Betrag korrigiert ist. » ; it-CH « Questo filtro non è applicato finché l'importo non è corretto. » ; en-CH « This filter is not applied
  until the amount is corrected. » Fallback Svelte = texte fr-CH. **Seule clé neuve** : le message du prix, des frais et du minimum est `common-amount-*`.
- **D6 — Ce qui ne change pas** : le serveur et l'API ; `balance.ts`, `amount-entry-message.ts`, `unit-price.ts`, `group-settlement-max.ts` ; les écrans des 36-18a,
  36-3, 36-3b, 36-2, 35-6, 43-7 ; **toute facture, avoir, écriture ou prix d'article déjà enregistré** — aucune donnée n'est relue ni réécrite, un prix ne se
  recalcule pas, un PDF archivé se rend à l'identique (la story ne touche ni la facture ni son rendu).

## Critères d'acceptation

1. **AC1 — Prix d'article** (`product-helpers.test.ts`, `products-page.test.ts`) : `classifyPriceInput` — « 1.250 », « 1,250 » → `ambiguous` ; « 1'250 », « 10,50 »,
   « 0 », « 007.50 » → `ok` ; « -1 » → `negative` ; « abc », « 1.23456 », « .5 » → `invalid` ; page : « 1.250 » → message ambigu sous le champ, `createProduct` et
   `updateProduct` non appelés, saisie conservée ; « 1'250 » → `unitPrice: '1250'` ; rouvert aux prix serveur `1.2340`, `1.2500`, `1500.0000`, `0.1230` → champs
   `1.2340`, `1.25`, `1500.00`, `0.1230`, **sans message** et enregistrables ; **prix effacé** à l'édition → « Le prix est obligatoire » **visible au pied**, bouton désactivé.
2. **AC2 — Frais de rappel** (test neuf `frontend/src/routes/(app)/settings/dunning/dunning-page.test.ts`) : « 1.250 » → message ambigu **sous le champ**, `createDunningLevel` et
   `updateDunningLevel` non appelés, formulaire ouvert, saisie conservée ; « 1'250 » → `feeAmount: '1250'` **à la création, puis à la modification** (deux tests, M4a et M4b) ;
   «  12.50  » → `'12.50'` ; « 12'34 » → message illisible (« … 2 décimales au plus », M10a) ; « -5 » et « 1e3 » → refus sans appel ; un niveau rouvert à `10.00` s'enregistre sans
   message. **Étape 0** : constater par un `curl` (base jetable) ou un test serveur que `PUT /api/v1/dunning-levels/{id}` refuse `"1.250"` en 400, journal
   `kesh-gate-logs/e4810/etape0-dunning.log`.
3. **AC3 — Montant minimal** (`settings-invoicing-page.test.ts`) : « 1.250 » → message sous le champ, `updateInvoiceSettings` non appelé, **bouton Enregistrer `disabled`** (comme pour le
   plafond, `+page.svelte:804-805`), et **actif** pour un minimum vide ; la soumission par Entrée (`fireEvent.submit`) d'un « 1.250 » n'appelle pas `updateInvoiceSettings` (garde de `save()`) ; « 12'34 » → message illisible (« … 2 décimales au plus », M10b) ; « 1'250 » → `minimumInvoiceAmount: '1250'` ; vide → `null` (test existant
   `:176-185`, vert sans modification) ; rouvert à `5.00` et à la forme réelle rendue par le serveur → aucun message ; **étape 0** : forme réelle relevée par
   `GET /api/v1/company/invoice-settings` (`etape0-minimum.log`). Le plafond du règlement groupé (43-7) garde ses tests verts.
4. **AC4 — Filtres** (`journal-entries-list-page.test.ts`, qui n'a aujourd'hui aucun test de montant ; mock d'`i18nMsg` déjà interpolant, `:20-22`) : « 1.250 » en *Montant min* → message et
   phrase « pas appliqué », `fetchJournalEntries` appelé **sans** `amountMin`, URL sans `amountMin`, champ gardant « 1.250 » ; « 12'34 » → message illisible (« … 4 décimales au plus », M10c) et borne
   omise ; « -5 » → **illisible** (la grammaire du journal n'admet aucun signe : la borne n'atteint jamais le serveur) ; « 1'250 » → `amountMin: '1250'` et `?amountMin=1250` ; URL
   `?amountMin=1.250` → message au chargement, borne non envoyée ; « 100 » → `amountMin: '100'` ; mêmes cas pour *Montant max*.
5. **AC5 — Rien de juste n'est refusé** : pour chacun des quatre sites, un test par forme — « 1.25 », « 1250 », « 1'250.50 », « 1’250 », « 1,25 », « 0.50 » — sans message,
   charge envoyée normalisée ; une saisie bordée de blancs (`' 1250 '`, une espace avant et après) sans message, envoyée `1250` (les quatre sites rognent) ; « 1250.0000 » (zéros de fin) accepté à l'article, au
   minimum et aux filtres ; les formes rouvertes d'AC1 à AC3 sans message. Seules deviennent refusées la forme de la règle et ce que la grammaire du journal ne lit pas —
   `1e3`, `.5`, `5.`, signe, blanc intérieur, **plus de quatre décimales, même nulles** — : un test le constate aux frais (« 10.01000 », que `Number` lisait). À l'article, un prix de **1 à 999 francs, non groupé, à trois décimales** (« 12.125 ») se saisit « 12.1250 » (V2-8) : « 12.1250 » est accepté ; « 0.125 », « 1234.125 » et « 1'234.125 » restent acceptés tels quels.
6. **AC6 — E2E** (`frontend/tests/e2e/montants-ambigus.spec.ts`, créé par la 36-18a, dont l'en-tête annonce ce complément : un `test` de plus par écran, mêmes aides
   `countWrites`, `waitForWrite`, `capture`) : « 1.250 » → message visible ; l'écouteur `page.on('request')`, armé avant le clic, compte les seules requêtes d'écriture
   de l'écran, par **méthode et route** — `POST /api/v1/products` et `PUT /api/v1/products/{id}` (`products.api.ts:40`, `:47`), `POST /api/v1/dunning-levels` et
   `PUT /api/v1/dunning-levels/{id}` (`dunning.api.ts:41`, `:49`), `PUT /api/v1/company/invoice-settings` (`invoices.api.ts:106`) — et en attend **zéro** ; la preuve est
   la relecture par `authedApiContext` de l'état d'avant. Puis « 1'250 » → le corps porte `1250`, réponse 2xx, relecture comparée **en valeur** (`new Big(lu).eq(1250)`),
   jamais en chaîne. Filtres : la requête `GET /api/v1/journal-entries` (`journal-entries.api.ts:31`) porte `amountMin=1250` pour « 1'250 », aucune borne pour « 1.250 ». **L'en-tête du fichier** (`:15-16`, « La 36-18b complètera ce fichier ») est réécrit au nom de la 48-10 (V1-11).
7. **AC7 — Aucun montant enregistré n'est relu ni modifié** : `git diff <base> --stat` ne nomme ni `crates/` hors `kesh-i18n/locales/`, ni `balance.ts`,
   `amount-entry-message.ts`, `unit-price.ts`, `group-settlement-max.ts`, ni `InvoiceForm`, ni `invoices/[id]` ; un test de page constate que **rouvrir** un article, un
   niveau et les réglages sans toucher au montant n'ajoute **aucun message** et renvoie la valeur d'origine (jamais un montant reformaté qui change de valeur).
8. **AC8 — Textes** : une clé × quatre locales (D5) ; `lint-i18n-ownership` vert sans entrée neuve ; `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` **recompté
   à la source** après la fusion (le compteur est partagé avec les autres stories de l'Epic 48), sans addition à la main.
9. **AC9 — Manuels** (`docs/manual/fr/*.tex` ; PDF à l'intégration, jamais commités) : une phrase par écran, qui renvoie à la règle de *Saisie d'une écriture en partie
   double* (`user-manual.tex:561`), porte la chaîne exacte `refusé comme ambigu`, l'exemple `1.250`, la forme `1'250` et la limite de décimales — `user-manual.tex`
   item *Prix unitaire HT* (`:1424`, qui dit aussi qu'un prix de 1 à 999 francs à trois décimales s'écrit « 12.1250 »), § *Le montant minimum d'une facture* (`:1495`), § *Recherche d'écritures*, item *Montant* (`:901`, la borne non appliquée) ;
   `admin-manual.tex` item *Frais* (`:1348`) et § *Montant minimum d'une facture* (`:3911`). **Contrôle de clôture, sur le `.tex`** :
   `/usr/bin/grep -n 'refusé comme ambigu' docs/manual/fr/user-manual.tex docs/manual/fr/admin-manual.tex` rend, en plus des **quatre lignes de départ** (`user-manual.tex:1029`,
   `:1456`, `:1770`, `:2105`), une ligne dans chacune des cinq sections citées. **Contrôle d'intégration seulement** (PDF régénéré après l'union) :
   `pdftotext <f>.pdf - | tr '\n' ' ' | tr -s ' ' | /usr/bin/grep -o 'refusé comme ambigu' | wc -l` pour chaque manuel, comparé à sa valeur de l'étape 0. Les manuels DE, IT
   et EN ne décrivent pas la règle (`git grep -il "ambigu\|mehrdeutig\|ambiguous" docs/manual/{de,it,en}` vide à `af4e0e313`) : rien à y changer.
10. **AC10 — Rien d'autre ne change** : leurs tests verts sans modification (hormis `product-helpers.test.ts`, dont les tests de `isValidPrice` et de `PRICE_RE` partent avec lui, ce que le Dev Agent Record dit).
11. **AC11 — Captures** (recette visuelle, thème clair, base jetable `kesh_e2e_e4810`, `kesh-gate-logs/e4810/captures/`) : chaque écran à « 1.250 » (message, bordure, saisie
    conservée), à « 12'34 », et la valeur « 1'250 » enregistrée et relue ; un article rouvert à `1.2340` sans message ; le filtre ambigu et sa phrase « pas appliqué » ; le
    signet `?amountMin=1.250` rouvert.
12. **AC12 — Accessibilité des refus (D3 de la 36-18a)** : aux quatre sites, un test constate, sur le champ fautif, `aria-invalid="true"` et un `aria-describedby` qui **nomme l'`id`** d'un
    élément portant le message (le prix sous le champ ; les frais sous le champ, **hors** de `formError` ; le minimum ; chaque borne du filtre) ; une fois la saisie corrigée, `aria-invalid` vaut `"false"` (attribut présent, comme le patron `groupMax`, `+page.svelte:771`) ou est absent, et `aria-describedby` ne nomme plus l'`id` du message ; **prix : Entrée sur « 1.250 », puis correction en « 1'250 » → aucun message rouge au pied** (V2-3).

## Tests à écrire, et la preuve du rouge

Écrits d'abord, sur le code non corrigé, journal `/home/gcorbaz/devel/kesh-gate-logs/e4810/rouges.log`.

- **Rouges attendus** : AC1 (`classifyPriceInput('1.250') === 'ok'` ; « 1'250 » `invalid`) ; AC2 (`feeAmount: '1.250'` part ; « 1'250 » refusé) ; AC3
  (`minimumInvoiceAmount: '1.250'`) ; AC4 (`amountMin: '1.250'`) ; AC6 (chaque requête part avec `1.250`).
- **Verts au départ, et déclarés filets** (ils ne prouvent rien du correctif) : le prix d'article rouvert (jamais ambigu par construction), les montants rouverts aux frais et au
  minimum, « 1.25 » et « 1250 » partout. Les **réouvertures** ont chacune leur mutation : M3 (article), M12 (frais), M13 (minimum) ; « 1.25 » et « 1250 » nus restent des filets **sans mutation** ; M2, M5, M9, M14 et M15 ne visent pas les réouvertures.
- **Mocks** : `products-page.test.ts:30-32` et `settings-invoicing-page.test.ts:18` mockent `i18nMsg` **sans interpolation** : un `{ $max }` y resterait invisible et les
  mutations M10a à M10c survivraient. Le mock est aligné sur celui de `journal-entries-list-page.test.ts:20-22` (interpolation) dans le test et le nouveau fichier du dunning. Pour le cas « URL `?amountMin=1.250` » (AC4), le mock de `page.url` de `journal-entries-list-page.test.ts:12-13` est **figé** : il devient mutable (`vi.hoisted`, un objet dont l'`URL` se règle par test), et l'URL écrite se lit dans les appels du mock `goto` (`:15`) (V1-9). Un test du bouton : voir AC3 (V1-7).
- **E2E rouge** : sources remises à la base par `git restore --source=<base> --worktree -- <fichiers>` après commit, frontend construit, spec rouge, sources restaurées
  (`git restore --worktree` puis `touch`), journal `e2e-rouges.log`.

**Mutations** (par l'outil **Edit**, jamais `git stash` ; code commité d'abord ; après restauration, `touch` du fichier ; journaux `mut/M*.log`, résumé `mut/resume.log` ;
Vitest seul ; dossier `/home/gcorbaz/devel/kesh-gate-logs/e4810/mut/`) :

| # | Mutation | Tuée par |
|---|---|---|
| M1 | article : `classifyPriceInput` sans le cas `ambiguous` | AC1 |
| M2 | article : `normalizePriceInput` reprend `replace(',', '.')` | AC1 (`1250`), AC5 |
| M3 | article : `openEdit` passe par une lecture qui refuse `1.234` | AC1 (rouvert) |
| M4a | frais : `feeAmount: fFee` brut à la **création** (`createDunningLevel`) | AC2 (« 1'250 » à la création) |
| M4b | frais : `feeAmount: fFee` brut à la **modification** (`updateDunningLevel`) | AC2 (« 1'250 » à la modification) |
| M5 | frais : la lecture sans `trim()` | AC2 (« 12.50 » rogné) |
| M6 | minimum : `minimumAmount.trim()` envoyé brut | AC3 |
| M7 | filtres : `buildQuery` envoie la borne brute | AC4 |
| M8 | filtres : la phrase « pas appliqué » retirée | AC4 |
| M9 | minimum : la lecture sans `trim()` | AC5 (« 1250 » bordé de blancs) |
| M10a | frais : `amountEntryMessage(problem, 2)` devenu `(problem, 4)` | AC2 (« 12'34 », « 2 décimales au plus ») |
| M10b | minimum : `amountEntryMessage(problem, 2)` devenu `(problem, 4)` | AC3 (« 12'34 ») |
| M10c | filtres : `amountEntryMessage(problem, 4)` devenu `(problem, 2)` | AC4 (« 12'34 », « 4 décimales au plus ») |
| M11 | filtres : le `$effect` d'URL (`+page.svelte:201-208`) construit sa propre requête à partir de la saisie brute (l'URL garde `amountMin=1.250` pendant que la requête est propre) | AC4 (assertion d'URL seule ; la mutation « `buildQuery` brut » est M7, l'URL en dérivant : pas de mutation distincte) |
| M12 | frais : `openEdit` rouvre `new Big(level.feeAmount).toFixed(3)` (trois décimales, « 10.000 ») | AC7 (réouverture sans message) |
| M13 | minimum : le minimum se rouvre à trois décimales | AC7 |
| M14 | article : `normalizePriceInput` / la lecture sans `trim()` | AC5 (« 1250 » bordé de blancs) |
| M15 | filtres : `buildQuery` lit la saisie sans `trim()` | AC5 (« 1250 » bordé de blancs) |
| M16 | retrait de `aria-invalid` ou de `aria-describedby` sur un champ fautif (un par site) | AC12 |
| M17 | minimum : `disabled` sans le terme `minimumBlocked` (bouton jamais désactivé) | AC3 |
| M18 | minimum : la garde de `save()` retirée | AC3 (soumission par Entrée d'un « 1.250 ») |
| M19 | prix : `formError` non remis à zéro à la correction | AC12 (Entrée puis correction) |
| M20 | prix : le pied ne montre plus `product-error-price-required` | AC1 (prix effacé) |

Une mutation équivalente se dit au Dev Agent Record.

## Tâches

- [x] **Étape 0** : relire les lignes citées à la tête de la branche ; rejouer les motifs d'inventaire (`kesh-gate-logs/e4810/etape0-inventaires.log`) ; AC2 (`curl` des frais) et
      AC3 (forme du minimum rendue) ; valeurs de départ du contrôle des manuels (AC9) ; relire les LOW du dernier rapport de validation. Constater que la 48-15 n'a pas
      changé `formValidation` depuis la fiche, et que la 43-7 n'a pas changé le bloc du plafond.
- [x] Tests d'abord (AC1 à AC5, AC7, AC12) ; `product-helpers.ts` et la page (D1) ; `settings/dunning` (D2) ; `settings/invoicing` (D3) ; `journal-entries/+page.svelte` (D4,
      en dernier) ; quatre `messages.ftl` (D5) ; `i18n-keys.test.ts` recompté (AC8).
- [x] E2E (AC6), rouge d'abord ; manuels (AC9), `.tex` seuls, contrôle `.tex` à la clôture ; captures (AC11) ; mutations M1 à M20.
- [x] Gates (`e35-dev-commun.md` § *Quels gates, quand*, voie B pour le frontend) : en cours, Vitest ciblé, `npm run check`, `npm run lint-i18n-ownership` ; **passage en `review`** :
      bases remises à neuf, `scripts/test-fast.sh --ci`, `scripts/gate-ci.sh kesh_ci_e4810` (voie A, son verrou), frontend complet, **pas d'E2E** ; **remédiations** :
      `test-fast.sh --ci` et frontend complet ; **clôture** : `gate-ci` et E2E complet sur `kesh_e2e_e4810` recréée, au dernier commit de code, échecs nommés contre
      `docs/testing.md` § *Les échecs attendus*. Une commande lourde à la fois, `CARGO_BUILD_JOBS=2`, `scripts/mem-guard.sh --status` avant chacune, verrou `.gate.lock`.
- [x] Fin du développement : `Status: review`, prompt de revue P1 (`48-10-montants-ambigus-reglages-filtres-review-prompt-p1.md`) qui **nomme les manuels (PDF compris,
      aplatis avant de greper) et les captures**. Ne pas écrire `CHANGELOG.md` ni `sprint-status.yaml`.

## Note de version

*Rubrique* **Corrigé** (C-e360-11) :

- Le prix d'un article, les frais d'un niveau de rappel, le montant minimal d'une facture et les bornes du filtre des écritures lisent désormais un montant comme une
  écriture : « 1.250 » n'est plus lu 1,25 franc en silence ; il est refusé comme ambigu (le filtre ne s'applique pas, et l'écran le dit), et le message dit comment
  l'écrire (`1'250` ou `1.25`). L'apostrophe des milliers et la virgule décimale sont acceptées ; la saisie à exposant (`1e3`), sans chiffre avant ou après le point
  (`.5`, `5.`) ou à plus de quatre décimales, même nulles, n'est plus lue. Les frais de rappel ne disent plus seulement « deux décimales au plus » d'un montant ambigu.
  Un prix d'article de 1 à 999 francs, non groupé, à trois décimales s'écrit désormais avec la quatrième, **« 12.1250 »** (et non « 12.125 », ambigu ; « 0.125 » et « 1234.125 » ne changent pas). **Avant la 0.20.0**, un prix d'article ou un montant minimal tapé avec trois chiffres après le point ou la virgule (« 1.250 ») a été lu sans avertissement comme un franc
  vingt-cinq : relisez vos articles et votre montant minimal saisis ainsi, et les factures qui ont repris un tel prix. Aucune donnée enregistrée n'est modifiée par cette
  version. (refs #1173, refs #1175, refs #1176, refs #1177)

## Dev Notes

### Hors périmètre

- Soldes de départ, facture fournisseur, règlement : **36-18a** (livrée).
- Le serveur et l'API (C-e350-19) ; les taux de TVA ; les cours ; les quantités (C-e360-6) ; le relevé manuel (deux décimales, « 1.250 » déjà refusé).
- La convergence des messages des 36-3, 36-3b et 36-2 vers les clés communes : **#1202**.
- Toute **correction des données** déjà saisies (articles, minimum) : la note de version les nomme, aucun script.

### Splitting — quatre modules

(1) articles (`features/products`, `routes/(app)/products`) ; (2) réglages (`routes/(app)/settings/dunning`, `settings/invoicing`, un seul domaine : les réglages) ;
(3) `features/journal-entries` (la page de liste) ; (4) `crates/kesh-i18n/locales`. Cinq si l'on compte les deux écrans de réglages séparément — **dans les deux cas, pas
plus de cinq : la règle ne s'applique pas.** S'y ajoutent les manuels, `montants-ambigus.spec.ts`, `i18n-keys.test.ts` et le test neuf `dunning-page.test.ts` (tests et
documentation, hors décompte). Ni les modules ni la nature des défauts ne se recyclent : quatre fois le même geste mécanique sur un lecteur déjà livré.

### Fichiers communs avec les autres stories de l'Epic 48 (fichier d'épique, tableau « Fichiers communs »)

- `routes/(app)/products/+page.svelte` : **48-15, déjà fusionnée** (nom et motif au pied du formulaire) ; pas de conflit de texte, **mais la 48-10 change la condition du pied** (`:726-730`) que la 48-15 emploie pour le motif du nom : les tests de la 48-15 restent verts (AC10), et le commentaire `:322-325` (« le motif s'affiche au pied du formulaire (`formTouched && formValidation`) ») est mis à jour. Déjà fusionnée dans la base de la branche : à rebaser au code.
- `routes/(app)/journal-entries/+page.svelte` : la **48-2** (détail, `[id]/`) touche un autre fichier de l'espace ; pas de conflit attendu.
- `settings/invoicing/+page.svelte` : 43-7 déjà fusionnée ; la 48-14 (solde) ne le nomme pas ; à recontrôler à l'étape 0.
- `messages.ftl` ×4, manuels, `i18n-keys.test.ts` (`sitesTotal`) : partagés avec les 48-1, 48-2, 48-12, 48-13, 48-15 et l'Epic 49 ; chaque story dans son bloc, un conflit se
  résout en gardant les deux côtés, les compteurs se recomptent après la fusion.
- `montants-ambigus.spec.ts` : créé par la 36-18a (`done`), seul à y écrire.
- `CHANGELOG.md` : non écrit par la branche ; PDF des manuels non commités.
- **Coordination** : aucun des fichiers de Beat (PDF des rapports, `largeurs.rs`) n'est touché.

### Ce qui doit rester vrai

- Un montant juste ne devient jamais refusé (AC5) ; un montant rouvert jamais (AC1 à AC3) ; **aucune saisie ne s'efface** (AC1 à AC4).
- Le serveur reste juge des bornes et de l'échelle ; big.js partout ; fixtures synthétiques seulement, aucune donnée réelle de Guy dans un test, une capture ou un commit.
- **Aucune facture, aucun avoir, aucun PDF existant ne change** (contrainte de production du 2026-11-18) : la story ne touche pas la facture ; seul le prix d'un article **neuf ou
  modifié** est concerné, et un article rouvert sans modification se renvoie à l'identique.

### E2E

Chaque montant part tel que tapé, et le filtre traverse la frontière HTTP : AC6 exigé (un test Vitest ne voit pas le corps réel ni l'URL).

### Références

- Issues #1173, #1175 (et son commentaire de l'orchestrateur), #1176, #1177, #1202 ; fiches `36-18a-montants-ambigus-livres.md`, `36-18-montants-ambigus-autres-ecrans.md` (`split`),
  `36-18b-montants-ambigus-reglages-filtres.md` (reprise) ; fichier d'épique `epic-48-corrections-0-20-0.md` § 48-10 ; Stories 36-3, 36-3b, 43-7 ; C-36-18-1 à C-36-18-10.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`bmad-dev-story`, en autonomie), 2026-11-19, worktree `kesh-48-10`, clé `e4810`, port 3410, bases `kesh_e4810`,
`kesh_e2e_e4810`, `kesh_ci_e4810` sur `kesh-mariadb-dev` (voie 1).

### Debug Log References

Tous sous `/home/gcorbaz/devel/kesh-gate-logs/e4810/` :

- **Étape 0** : `etape0-inventaires.log` (37 champs `inputmode="decimal"`, identiques à l'inventaire de la fiche, sonde positive
  `settings/dunning/+page.svelte:284`) ; `etape0-dunning.log` (`PUT` et `POST /api/v1/dunning-levels` avec `"1.250"` → 400
  « Les frais de rappel ne peuvent avoir plus de 2 décimales. », niveaux relus inchangés, rendus `"0.00"`, `"20.00"`, `"40.00"`) ;
  `etape0-minimum.log` (forme rendue : `5` et `5.00` → `5.0000` ; `1.25` → `1.2500` ; **`1.250` accepté et rendu `1.2500`**, le
  défaut #1176 constaté ; `1250.5` → `1250.5000`) ; `etape0-manuels.log` (`.tex` : quatre lignes de départ ; PDF : 4 occurrences
  au manuel utilisateur, 0 au manuel d'administration ; DE/IT/EN : aucun, relancé depuis la racine avec sonde positive après un
  premier essai à pathspec relative muette, dit au journal).
- **Rouges** : `rouges.log` (commit `3cba47602`, tests non commités) — **59 rouges** sur 163 tests des cinq fichiers ;
  `e2e-rouges.log` — sources remises à `af4e0e313`, frontend construit (`build-front-rouge.log`) : **4 rouges** sur 8, les
  requêtes parties avec `"unitPrice":"1.250"` (deux fois : clic forcé et Entrée), `"feeAmount":"1.250"`, le `PUT` des réglages, et
  la borne envoyée ; sources restaurées par `git restore --worktree` puis `touch`.
- **Verts** : `verts-cibles-1.log` (163/163, cinq fichiers) ; `check-1.log` (`svelte-check` 0 erreur, 29 avertissements tous hors
  des fichiers de la story ; `lint-i18n-ownership` vert ; `i18n-keys.test.ts` rouge « expected 3496 to be 3495 ») ; `check-2.log`
  (11/11 après relevé) ; `e2e-verts-1.log` (7/8 : une assertion fausse de **mon** test, corrigée par `a61314f1f`),
  `e2e-verts-2.log` (**8/8**, captures écrites).
- **Mutations** : `mut/resume.log`, `mut/M*.log`, script `mut/mutations-e4810.py` — **26 mutations jouées (M1 à M20, M4, M10 et
  M16 déclinées), 26 tuées, 0 survivante, 0 équivalente** ; code à `a61314f1f`.
- **Gates du passage en revue** : voir § *Completion Notes*, *Gates*.

### Completion Notes List

**Étape 0.** Lignes citées relues à `3cba47602` : la 48-15 n'a pas changé `formValidation` (`products/+page.svelte:328-340`, nom
puis prix, comme la fiche) ; la 43-7 n'a pas changé le bloc du plafond (`settings/invoicing/+page.svelte:76-100`, `:763-786`).
Les huit LOW de la P3 de validation sont déjà traités dans le texte de la fiche (Change Log, entrée « Validation terminée ») ;
aucun ne demandait de geste au code. Rien de la fiche n'était « à confirmer » hors des deux constats AC2 et AC3, faits.

**Écart à la lettre de D1 (C-e4810-1).** `if (formError === priceMessage) formError = ''` ne marche pas : `oninput` (délégué en
Svelte 5) s'exécute après la mise à jour de `formPrice` par `bind:value`, et `priceMessage` y vaut déjà celui de la saisie
corrigée (`''`). Le test d'AC12 (« Entrée sur 1.250 puis correction ») l'aurait rougi. `onPriceInput` compare donc `formError` aux
**trois** messages du prix ; même intention (le doublon de nom et les taux de TVA restent).

**Le reste suit D1 à D6** : `normalizePriceInput(raw): string | null` = `amountEntryPayload(raw.trim())`, garde `if (unitPrice ===
null) return;` ; `isValidPrice` et `PRICE_RE` retirés avec leurs tests (AC10) ; le prix illisible garde « Format de prix
invalide », le négatif son message ; frais, minimum et bornes par `readJournalAmount(….trim())` ; minimum `minimumBlocked` au
bouton et garde de `save()` ; bornes omises de la requête et de l'URL par `amountBound`, la saisie jamais réécrite. Une clé neuve
(D5), quatre locales. Écrans hors périmètre, aides communes, serveur : intacts (`git diff --stat af4e0e313`, AC7).

**Usage réel de Guy (aucune forme juste refusée, aucune saisie perdue, aucun montant enregistré modifié).** AC5 : douze formes
justes au prix, sept aux frais, huit au minimum et aux bornes, acceptées sans message et envoyées normalisées (« 1'250 » → `1250`,
« 1,25 » → `1.25`, « 1250.0000 » gardé) ; deviennent refusées, comme la fiche le prévoit, la forme ambiguë et ce que la grammaire
du journal ne lit pas (`1e3`, `.5`, `5.`, signe, blanc intérieur, plus de quatre décimales — testé aux frais et au prix). Saisie
conservée après refus : testée aux quatre écrans (Vitest et E2E). Réouvertures : article à `1.2340`, `1.2500`, `1500.0000`,
`0.1230`, niveau à `10.00`, minimum à `5.00`, `5.0000`, `1.2500`, `1250.5000` (formes réelles d'`etape0-minimum.log`) — aucun
message, renvoyés **à la même valeur** (tuées M3, M12, M13). Aucune donnée relue ni réécrite : la story ne touche ni facture, ni
avoir, ni PDF.

**Manuels (AC9).** Cinq phrases (`user-manual.tex` : item *Montant* de la recherche, item *Prix unitaire HT*, § *Le montant
minimum d'une facture* ; `admin-manual.tex` : item *Frais*, § *Montant minimum d'une facture*). Contrôle de clôture sur le `.tex` :
`/usr/bin/grep -n 'refusé comme ambigu'` rend neuf lignes (quatre de départ, `:1029`, `:1456`, `:1770`, `:2105`, plus `:901`,
`:1424`, `:1495` et admin `:1348`, `:3911`). PDF **non régénérés** (intégration). Le garde `textes_coherents`
(`les_comptes_cites_en_exemple…`) a pris l'exemple `1234.125` pour un compte (rouge de `test-fast`, ci-dessous) : réécrit
`1'234.125`.

**E2E (AC6)** : quatre tests ajoutés à `montants-ambigus.spec.ts`, en-tête réécrit au nom de la 48-10 ; rouges sur les sources de
la base, verts au code (8/8). **E2E complet non lancé** (clôture).

**Captures (AC11)** — `captures/`, douze `e4810-*.png`, thème clair, base jetable `kesh_e2e_e4810` : `article-prix-ambigu`,
`article-prix-illisible`, `article-prix-1250-enregistre`, `article-rouvert-1.2340`, `rappel-frais-ambigu`,
`rappel-frais-illisible`, `rappel-frais-1250-enregistre`, `minimum-ambigu`, `minimum-illisible`, `minimum-1250-enregistre-relu`,
`filtre-ambigu`, `filtre-signet-ambigu`. Regardées : message rouge sous le champ, bordure rouge, saisie conservée. (Les
`e3618a-*.png` du même dossier viennent des tests de la 36-18a rejoués par la même spec.)

**Écarts à la consigne Edit.** Le bloc de tests des filtres a été **ajouté par `cat >>`** à la fin de
`journal-entries-list-page.test.ts` (un ajout en fin de fichier, relu ensuite et vert), au lieu de l'outil Edit : l'outil n'y
manquait de rien, c'est un geste d'habitude. Les mutations passent par un script (permis).

**Gates.**
- `test-fast --ci` (`review-test-fast.log`, commit `90a6e08b0`, bases remises à neuf, `bases.log`) : 6155 tests, **6154 verts,
  1 rouge** — `kesh-api::textes_coherents::les_comptes_cites_en_exemple_existent_dans_les_plans_livres`, causé par l'exemple
  `1234.125` du manuel. Corrigé par `7d8c779ca` (documentation seule) ; binaire rejoué seul : `textes-coherents-1.log` (rouge,
  avant que l'édition ne soit appliquée), `textes-coherents-2.log` (30/30). `gate-ci` rejoue toute la suite à `7d8c779ca`.
- `gate-ci.sh kesh_ci_e4810` (`review-gate-ci.log`, commit `7d8c779ca`, base de CI recréée) : **6172 passés, 0 échoué,
  18 ignorés**, code 0.
- Frontend complet (`review-front.log`, commit `7d8c779ca`) : `check` 0 erreur (29 avertissements, aucun dans les fichiers de la
  story), `lint-i18n-ownership` vert, `build` vert, **`test:unit` 1 rouge** : `e2e-selecteurs-traduits.test.ts` (garde KF-043) —
  mes quatre tests E2E visaient « Créer » et « Modifier » par leur libellé traduit. Corrigé par `e4bf86564` (sélecteurs de
  structure) ; puis (`review-front-2.log`, commit `e4bf86564`) **`test:unit` 4750/4750** et `check` 0 erreur ; garde rejoué seul
  (`garde-selecteurs.log`, vert) ; spec E2E rejouée (`e2e-verts-3.log`, **8/8**). `lint-i18n-ownership` et `build` non rejoués à
  `e4bf86564` (le commit ne touche que la spec E2E, hors de leur champ) ; `test-fast` et `gate-ci` non plus (aucun code Rust ni
  fichier lu par un test Rust).
- **Non lancés** : E2E complet (clôture), régénération des PDF (intégration).

**Commits.** `e076b9913` (code, tests, catalogues, registre), `a61314f1f` (spec E2E), `90a6e08b0` (manuels, prompt de revue),
`7d8c779ca` (manuel), `e4bf86564` (spec E2E, sélecteurs — **dernier commit de code**, tests seulement ; dernier code de production :
`e076b9913`), puis le commit de la fiche (statut `review`). Diff de la revue : `git diff 3cba47602 e4bf86564`.

**Fichiers communs.** `i18n-keys.test.ts` (`sitesTotal` 3495 → 3496, relevé par la garde) et les quatre `messages.ftl` : à
recompter et à unir à la fusion avec les autres stories de l'Epic 48. Aucun fichier de Beat touché.

**Statut à poser au `sprint-status.yaml`** (par l'orchestrateur) : `48-10-montants-ambigus-reglages-filtres: review`.

#### Remédiation P1 (2026-11-19)

Revue P1 (trois lentilles Opus) : 0/0/0/14 (`review-p1-{B,E,A}.md`). LOW corrigés dans la story (règle du 2026-11-14), sur
demande de l'orchestrateur :

- **B1, E1** (C-e4810-7) : la saisie du prix efface tout message rouge du pied sauf ceux du nom et des taux de TVA
  (`nonPriceMessages()`) ; doc-comment réécrit.
- **B2, B3** : rappels — `onFeeInput()` efface le bloc commun (sauf le refus du délai) ; le refus des frais est rendu dans le
  bloc du champ *Frais*.
- **B4, B5** (C-e4810-6) : lecture au centime des frais et du minimum (`amount-entry-centimes.ts`, neuf) — « 0.125 »,
  « 10.1250 » illisibles avec « 2 décimales au plus » ; « 10.0100 » part `10.01` ; un minimum rouvert à `5.0000` repart `5.00`
  (même valeur ; les tests de réouverture comparent désormais aussi en valeur) ; un seul prédicat `minimumProblem`.
- **Tests** : E2 (nom vide et « 1.250 »), bouton désactivé sur `12'34` au minimum, borne « ␣1.250␣ », B1, E1, doublon de nom
  conservé, unités de `readCentimesEntry`. L'aide `entree()` des tests de l'article passe de `requestSubmit()` à
  `fireEvent.submit` : `requestSubmit()` n'émet rien dans jsdom quand le bouton par défaut est désactivé, si bien que les tests
  « Entrée » sur une saisie refusée ne soumettaient rien (constaté par l'échec du test B1).
- **E2E** (E3, A3, A1, A2) : le filtre porte sur deux écritures créées par l'API (100.00 et 2000.00) ; « 1'250 » n'en montre
  qu'une, « 1.250 », « 12'34 » et le signet les deux ; capture `e4810-filtre-illisible.png` ajoutée ; l'aide `capture` prend
  `animations: 'disabled'`.
- **A4** : la prémisse de C-e4810-1 est constatée — la mutation `M19-litteral` (formule littérale de D1) rougit le test
  d'AC12 (`mut/M19-litteral.log`).
- **A5, A6** : bases remises à neuf, commandes et sorties journalisées (`bases-remed-p1.log`) ; spec E2E rejouée après le
  commit, en-tête de commit au journal (`remed-p1-e2e.log`).

**Commits** : `531397f5a` (code et tests de la remédiation), `d1afc9384` (compteur `sitesTotal` 3496 → 3502, six sites de clés
existantes, relevé par la garde) — **écart** à « un seul commit de code » : le compteur ne s'est révélé qu'au `test:unit`
complet, joué après le commit. Puis le commit de la fiche, du registre et du prompt P2.

**Ce qui a tourné** (journaux sous `kesh-gate-logs/e4810/`) :
- Vitest ciblé : `remed-p1-vitest-1.log` (4 rouges : trois dus à mon exemple « 10.125 », ambigu et non illisible, remplacé
  par « 10.1250 » ; un dû à `requestSubmit`), `remed-p1-vitest-2.log` (**185/185**) ; `remed-p1-check-pre.log` (0 erreur).
- Mutations (code à `531397f5a`, `mut/mutations-e4810-p1.py`, `mut/resume.log`) : **8 jouées, 8 tuées** — `M19-litteral`,
  `M24` (effacement borné aux trois refus), `M25` (pied `!priceMessage`, E2), `M21` (borne des deux décimales retirée), `M26`
  (charge non ramenée), `M22` (refus sous la rangée), `M23` (bloc commun non effacé), `M17u` (bouton seulement sur l'ambigu).
- `test-fast --ci` (`remed-p1-test-fast.log`, commit `531397f5a`, bases remises à neuf) : **6155/6155**, 14 ignorés.
- Frontend (`remed-p1-front.log`, commit `531397f5a`) : `check` 0 erreur, `lint-i18n-ownership` vert, `build` vert,
  `test:unit` 1 rouge (`sitesTotal`) ; puis `remed-p1-front-2.log` (commit `d1afc9384`) : **`test:unit` 4772/4772**.
- Spec E2E de la story : `remed-p1-e2e.log` — RESULTAT_E2E.
- **Non lancés** : `gate-ci` et E2E complet (clôture) ; PDF (intégration).

### File List

- `frontend/src/lib/features/products/product-helpers.ts` (modifié)
- `frontend/src/lib/features/products/product-helpers.test.ts` (modifié)
- `frontend/src/routes/(app)/products/+page.svelte` (modifié)
- `frontend/src/routes/(app)/products/products-page.test.ts` (modifié)
- `frontend/src/routes/(app)/settings/dunning/+page.svelte` (modifié)
- `frontend/src/routes/(app)/settings/dunning/dunning-page.test.ts` (neuf)
- `frontend/src/routes/(app)/settings/invoicing/+page.svelte` (modifié)
- `frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts` (modifié)
- `frontend/src/routes/(app)/journal-entries/+page.svelte` (modifié)
- `frontend/src/routes/(app)/journal-entries/journal-entries-list-page.test.ts` (modifié)
- `frontend/src/lib/shared/i18n-keys.test.ts` (modifié, `sitesTotal`)
- `frontend/tests/e2e/montants-ambigus.spec.ts` (modifié)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` (une clé chacun)
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/admin-manual.tex` (modifiés ; PDF non commités)
- `frontend/src/lib/features/journal-entries/amount-entry-centimes.ts` et `.test.ts` (neufs, remédiation P1)
- `_bmad-output/implementation-artifacts/e0200-choix-autonomes-e4810.md` (neuf, C-e4810-1 à 7)
- `_bmad-output/implementation-artifacts/48-10-montants-ambigus-reglages-filtres-review-prompt-p1.md` (neuf)
- `_bmad-output/implementation-artifacts/48-10-montants-ambigus-reglages-filtres-review-prompt-p2-ciblee.md` (neuf)

## Change Log

- 2026-11-19 — Fiche créée (Sonnet 5.5) **par reprise de la 36-18b**, code relu à `af4e0e313` : quatre sites, décisions D1 à D6 (les communes renvoyées à la 36-18a, désormais dans
  `main`), AC1 à AC11, mutations M1 à M11 ; quatre modules ; aucune migration. Changements par rapport à la 36-18b : (a) l'aide `amountEntryMessage` existe, les écrans ne
  composent plus leurs messages ; (b) le modèle `groupMax` de l'Epic 43 sur la page des réglages de facturation sert de patron au montant minimal ; (c) la 48-15 a changé le
  pied de formulaire de la page des articles, d'où l'affichage du message du prix sous le champ ; (d) tous les numéros de ligne de code et de manuel relus (le manuel a bougé de
  `:1126`, `:1197`, `:881`, `:1338`, `:3856` à `:1424`, `:1495`, `:901`, `:1348`, `:3911`) ; (e) inventaire refait par le symptôme sur le code actuel (trente-sept champs, un site neuf
  déjà corrigé, aucun hors des quatre issues) ; (f) AC7 « aucun montant enregistré modifié » ajoutée, note de version et sortie à la coupure alignées sur la 0.20.0 ; (g) mocks
  d'`i18nMsg` sans interpolation relevés (M10). Validations : voir les entrées suivantes.
- 2026-11-19 — **Validation terminée** (boucle de trois passes). **Tendance** : P1 (Opus 5.5) 0 CRITICAL / 0 HIGH / 2 MEDIUM / 9 LOW → P2 (Sonnet 5.5) 0/0/2/6 → P3 ciblée (Opus 5.5, sur le seul commit de la dernière remédiation) 0/0/0/8. **MEDIUM nés de la remédiation** : ceux de la P2 sont nés de la remédiation P1 (AC5 emportée par une édition, prix vide sans lieu d'affichage) ; ceux de la P1 portaient sur la D3 non réemployée (frais, prix) et sur la preuve du rouge (M10, M4). Aucun MEDIUM né de la remédiation P2 : la sévérité est retombée. **Remédiations** : P1 — D3 aux quatre sites (message sous le champ, `aria-*`, AC12), cas illisible aux AC3 et AC4, M4 et M10 déclinées, M12 et M13, filets déclarés, `formValidation` et `normalizePriceInput` fixés, numéros de ligne, inventaire reclassé, « -5 » illisible, bouton du minimum, angles morts (quantités C-e360-6, cours, serveur et API C-e350-19) sans issue, mock de `page.url`, en-tête de l'E2E ; P2 — AC5 rétablie, pied du prix vide, `formError`, `aria-invalid="false"`, M11 redéfinie et M14 à M20, « ni claire ni vide », « 12.1250 » ; **les huit LOW de la P3 corrigés après la dernière passe, dans le texte de la fiche** (terme `minimumBlocked`, `formError` borné à `priceMessage`, `priceMessage` défini et scénarios « Entrée » défensifs, règle « 12.1250 » bornée à 1 à 999 non groupé, exemple `' 1250 '`, Dev Note du pied de formulaire, garde `:363-364`, filets sans mutation) — texte seul, aucun code de production : la règle n'exige pas de passe de plus. **Reclassement** : #1173 et #1176 passent P1 (décision de Guy du 2026-11-19). Compte : 12 AC, 20 mutations (23 lignes, M4 et M10 déclinées), D1 à D6.
- 2026-11-19 — **Développement** (Opus 5.5, `bmad-dev-story`), statut `review`. Étape 0 : constats AC2 (400 serveur sur « 1.250 »)
  et AC3 (minimum rendu à quatre décimales, « 1.250 » accepté et lu `1.2500`), inventaire rejoué identique (37 champs), lignes
  relues (48-15 et 43-7 inchangées). Rouges : 59 Vitest, 4 E2E. Code : D1 à D6 ; **un écart à la lettre de D1** (C-e4810-1 :
  remise à zéro de `formError` comparée aux trois messages du prix, la formule de la fiche échouant à cause de l'ordre des écouteurs
  Svelte 5). Mutations : 26 jouées (M16 déclinée en quatre en plus de M4 et M10), 26 tuées. Deux rouges de gate nés de la story et
  corrigés dans la story : garde du manuel `textes_coherents` (exemple `1234.125` → `1'234.125`, `7d8c779ca`) et garde des sélecteurs
  E2E traduits (`e4bf86564`). Gates : voir le Dev Agent Record. E2E complet : à la clôture. Écart à la consigne Edit : un ajout par
  `cat >>` (bloc de tests des filtres).
