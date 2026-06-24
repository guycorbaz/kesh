---
status: ready-for-dev
epic: 18
story: 18-1c
type: feature
parent: 18-1
issue: 180
created: 2026-06-24
depends_on: [18-1a]
baseline_commit: 79fb482
stepsCompleted: []
---

# Story 18-1c — Achats avec TVA récupérable (saisie manuelle assistée)

> Extraite de l'umbrella convergée [`18-1-comptabilisation-tva-achats.md`](18-1-comptabilisation-tva-achats.md)
> (validate 5 passes, DC1-DC9 figés). **Axe (c)** — **DC3=B** : pas de nouvelle entité « facture d'achat ».
> Un **helper UI d'écriture manuelle assistée** (journal `Achats`) permet de saisir un achat avec TVA
> récupérable en pré-remplissant la ligne d'**impôt préalable** depuis un taux TVA. Réutilise
> `POST /journal-entries` + `GET /vat-rates` + `GET /company/invoice-settings` (tous existants).
> Dépend de **18-1a** (`done` — compte impôt préalable `1171` + `default_vat_recoverable_account_id`).

## User Story

**En tant que** comptable/fiduciaire utilisant Kesh pour une PME suisse,
**je veux** saisir un achat avec TVA récupérable sans calculer la TVA à la main, via un assistant qui
pré-remplit l'écriture (charge HT + impôt préalable + contrepartie TTC),
**afin de** comptabiliser correctement la TVA déductible sur le compte d'impôt préalable — base du décompte
AFC (18-1d) où le solde de ce compte alimentera `total_vat_recoverable`.

## Contexte ground-truth (vérifié `main` @ `79fb482`, après 18-1a/18-1b)

### Décision de scope décisive (prouvée par exploration)

**18-1c est une story FRONTEND — aucun changement de code backend de PRODUCTION.** Un achat TVA récupérable
est saisissable intégralement via l'endpoint **existant** `POST /journal-entries` en 3 lignes équilibrées
(`D charge 6xxx / D impôt préalable 1171 / C fournisseur TTC`).

> **⚠️ Exception test-fixture (corrigée Pass 1)** : la fixture de test partagée
> `crates/kesh-db/src/test_fixtures.rs` (`seed_accounting_company` + `seed_accounting_company_no_fy`) **ne
> configure PAS** `default_vat_recoverable_account_id` aujourd'hui (vérifié ground-truth : l'`INSERT
> company_invoice_settings` ne pose que `default_receivable/revenue/vat_payable` — `test_fixtures.rs:152-165`
> + `:259-271`). Sans ce champ, l'assistant tomberait en mode « config requise » (AC5) et l'E2E AC10
> échouerait. **18-1c DOIT donc ajouter une ligne de test-fixture** : poser `default_vat_recoverable_account_id`
> = **compte `1000` réutilisé** (Asset existant « Caisse CI », `test_fixtures.rs:124`) — **PAS** de 6e compte
> (compteur `("accounts", 5)` inchangé, exactement le pattern 18-1b qui a réutilisé `2000` pour payable).
> `create_in_tx` ne valide que `active=TRUE` + `company_id` (pas le type), donc `1000` (Asset, actif) est une
> cible valide. C'est l'unique changement backend (test-only), couvert par T-C5. Le code de production
> backend reste **strictement inchangé**.

Tout le backend de production nécessaire existe déjà :

- **`POST /api/v1/journal-entries`** (`crates/kesh-api/src/routes/journal_entries.rs:62-80` requête,
  `:388-495` handler) — accepte `{ entryDate, journal, description, lines: [{accountId, debit, credit}] }`
  (camelCase). Valide l'équilibre débit=crédit (`accounting::validate`), 2..500 lignes, comptes actifs +
  company-owned (`create_in_tx`, `repositories/journal_entries.rs:113-143`), journal ∈
  {Achats,Ventes,Banque,Caisse,OD} (CHECK SQL `20260412000001_journal_entries.sql`), exercice ouvert.
  Le journal `Achats` est autorisé. **Rien à ajouter côté backend.**
- **`GET /api/v1/vat-rates`** (`routes/vat.rs:160-180`) → `VatRateResponse[]` (`{id, category, label,
  rate: "8.10", validFrom, validTo, active, version}`). Taux actifs par défaut. Front déjà câblé :
  `vat-rates.api.ts:26 listVatRates()` + store de session `vat-rates.store.svelte.ts getVatRates()`.
- **`GET /api/v1/company/invoice-settings`** → `InvoiceSettingsResponse` avec
  `defaultVatRecoverableAccountId: number | null` (posé en 18-1a). Front déjà câblé :
  `invoices.api.ts:67 getInvoiceSettings()` + type `invoices.types.ts:49-56`.
- **`GET /api/v1/accounts`** → `AccountResponse[]` (`{id, number, name, accountType, active, ...}`). Front
  déjà câblé : `accounts.api.ts:9 fetchAccounts()`, déjà chargé par la page journal-entries.

### Ce qui EXISTE côté frontend (réutiliser, ne pas réinventer)

- **Formulaire d'écriture manuelle** : `frontend/src/lib/features/journal-entries/JournalEntryForm.svelte`.
  - État runes : `journal = $state<Journal>('Achats')` (défaut Achats, l.65), `description` (l.67),
    `lines = $state<LineDraft[]>` (l.69-76, 2 lignes vides à la création), `entryDate` (l.63).
  - `LineDraft` (`form-helpers.ts:9-13`) : `{ accountId: number | null, debit: string, credit: string }`
    (montant 0 ⇒ string vide `''`).
  - Équilibre dérivé : `balance = $derived(computeBalance(lines))` (l.84, `balance.ts:47`) →
    `{ totalDebit, totalCredit, diff, isBalanced, hasInvalidAmount }`. `canSubmit` (l.92-100) exige
    description non vide + ≥2 lignes `valid` + 0 ligne `partial` + `balance.isBalanced`.
  - Payload POST (l.114-127) : `{ entryDate, journal, description: description.trim(),
    lines: nonEmptyLines.map(...) }` avec `debit/credit` vides → `'0'`, virgule → point.
  - Sélection de compte : `AccountAutocomplete.svelte` (props `accounts: AccountResponse[]`,
    `value: number|null`, `onSelect`), filtre `active === true` (l.32), recherche number/name.
  - Page parent : `routes/(app)/journal-entries/+page.svelte` charge `accounts` (l.91) et instancie
    `<JournalEntryForm {accounts} {accountsLoadError} ...>` (mode create l.378, edit l.385).
- **Arithmétique décimale** : `big.js` (`package.json:44`), déjà utilisé dans `balance.ts:13`. **Jamais
  `parseFloat`.**
- **i18n** : `i18nMsg(key, fallback)` (`shared/utils/i18n.svelte.ts:14` ; ré-exporté via
  `features/onboarding/onboarding.svelte`). Clés du formulaire préfixées `journal-entry-form-*`.
  Le helper TVA utilisera le namespace **`vat-purchase-*`**.
- **IDs DOM HTTP-LAN-safe** : `$props.id()` (jamais `crypto.randomUUID` — `undefined` en contexte non
  sécurisé sur déploiement HTTP NAS, cf. bug #145 / `feedback_no_secure_context_apis_http_lan`). Précédent :
  `routes/(app)/settings/invoicing/+page.svelte` (`const uid = $props.id()`).
- **Tests unitaires** : vitest, fichiers `*.test.ts` dans la feature (`balance.test.ts`,
  `form-helpers.test.ts`, …). Lancés via `npm run test:unit`.

### Calcul TVA — parité backend obligatoire (DC-c2)

`crates/kesh-core/src/accounting/vat.rs:39-43` (réexporté `accounting/mod.rs:22`) :
```rust
pub fn line_vat_amount(base_ht: Decimal, rate_percent: Decimal) -> Decimal {
    Money::new(base_ht * rate_percent / dec!(100)).round_to_centimes().amount()
}
```
`round_to_centimes` = **half-up away-from-zero** (`MidpointAwayFromZero`, `money.rs:66-71`), 2 décimales.
**Aucun helper TS équivalent n'existe** (grep `vatAmount|lineVat|round.*centime` sur `invoices`/`products`/
`shared` = 0 résultat). Le front DOIT répliquer ce calcul avec parité exacte (sinon écart d'un centime →
écriture déséquilibrée rejetée par le backend). Les montants d'achat HT sont **positifs** ⇒ half-up
(`Big.roundHalfUp`, mode 1) ≡ away-from-zero. Implémentation : `new Big(ht).times(rate).div(100).round(2,
Big.roundHalfUp).toFixed(2)`.

## Décisions figées (héritées umbrella + tranchées par exploration — NE PAS re-litiger)

- **DC-c1 (umbrella DC3=B)** — **zéro code backend de PRODUCTION, zéro migration, zéro nouvel endpoint, zéro
  nouvelle entité.** L'écriture d'achat est postée via `POST /journal-entries` existant. **Unique exception
  (Pass 1)** : une ligne de **test-fixture** (`default_vat_recoverable_account_id` = compte `1000` réutilisé)
  pour rendre l'E2E exécutable — réutilise un compte existant (compteur inchangé), pattern 18-1b. Voir T-C5.
- **DC-c2 (parité calcul)** — la TVA récupérable est calculée côté front avec **parité exacte** de
  `line_vat_amount` : `round(ht × rate ÷ 100, 2, half-up)` via `big.js`. Test de parité obligatoire
  (valeurs de référence identiques au backend).
- **DC-c3 (compte impôt préalable)** — la ligne d'impôt préalable est postée sur
  `InvoiceSettingsResponse.defaultVatRecoverableAccountId` (compte `1171`, posé 18-1a). Si ce champ est
  `null`, l'assistant est **désactivé** avec un message renvoyant vers **Paramètres → Facturation**
  (parallèle frontend de l'AC5 backend de 18-1b : config requise).
- **DC-c4 (taux)** — l'assistant propose les **taux actifs** issus de `getVatRates()` (`category` + `rate`).
  Le taux `exempt`/0 produit une TVA nulle. **Le taux choisi est libre** (l'utilisateur sélectionne) — pas
  de re-lookup temporel `find_for_category_at_date` côté front (l'assistant n'a pas de notion de période :
  l'utilisateur choisit le taux applicable, comme pour une facture saisie).
- **DC-c5 (l'assistant peuple le formulaire existant)** — l'assistant **génère 3 lignes équilibrées** et
  les **injecte dans `lines` $state** du `JournalEntryForm`, met `journal = 'Achats'` et pré-remplit la
  `description`. L'utilisateur **revoit, édite et soumet** via le flux POST existant. La validation
  d'équilibre existante (`canSubmit`) reste le garde-fou. **Aucune nouvelle validation de soumission.**
- **DC-c6 (comptes saisis vs auto)** — dans l'assistant : le **compte de charge** (Expense, 6xxx) et le
  **compte de contrepartie** (fournisseur/banque/caisse — le crédit TTC) sont **choisis par l'utilisateur**
  (réutiliser `AccountAutocomplete`). Le **compte d'impôt préalable** est **automatique** (DC-c3).
- **DC-c7 (équilibre par construction — VERROU NUMÉRIQUE Pass 3 Opus)** — **les 3 lignes sont toutes
  arrondies à 2 décimales** : `débit charge = round₂(HT)`, `débit impôt préalable = vat` (déjà 2 déc. via
  `lineVatAmount`), `crédit contrepartie = round₂(HT) + vat` (= `round₂(HT + vat)`, **identique** car `vat`
  a exactement 2 décimales). `SUM(debit) = round₂(HT) + vat = SUM(credit)` **exactement**, prouvé par
  brute-force Opus (0 contre-exemple sur 20 000 HT à 4 décimales + tie-breaks).
  - **⚠️ Piège (F-OPUS-C1)** : `isValidAmount` autorise un HT à **3-4 décimales** (`AMOUNT_RE`,
    `balance.ts:23`). Si le dev émet `débit charge = HT brut` (non arrondi, ex. `"100.005"`) alors que
    `crédit = round₂(HT + vat)`, l'écriture est **déséquilibrée** (`100.005 + 8.10 = 108.105 ≠ 108.11`) →
    `POST` rejeté `ENTRY_UNBALANCED` (le backend compare en **égalité décimale EXACTE**, sans epsilon —
    `balance.rs`, `journal_entry_lines DECIMAL(19,4)`). **OBLIGATION** : `débit charge` DOIT être
    `round₂(HT)` (pas le HT brut). « PAS de re-arrondi » ne concerne que `vat` (ne pas réarrondir la somme
    des TVA), PAS le HT.
  - Si `vat == 0` (taux exempt/0) → **2 lignes seulement** (charge `round₂(HT)` + contrepartie `round₂(HT)`,
    montants égaux), pas de ligne impôt préalable (cohérent F-OPUS-1 / `chk_jel_debit_credit_exclusive` qui
    interdit `debit = 0`).
- **DC-c8 (HTTP-LAN-safe)** — IDs DOM via `$props.id()`, jamais `crypto.randomUUID`.
- **DC-c9 (sémantique d'injection)** — l'assistant agit sur un formulaire de **création** ; il **remplace**
  les lignes du brouillon (les 2 lignes vides initiales) par les lignes générées. S'il existe déjà des
  lignes non vides saisies à la main, l'assistant **demande confirmation** avant de remplacer (évite une
  perte de saisie silencieuse).

## Acceptance Criteria

- **AC1 — Assistant présent** : le `JournalEntryForm` (mode création) expose un panneau « Assistant TVA
  achat » (repliable, replié par défaut) au-dessus du tableau des lignes. Champs : compte de charge
  (`AccountAutocomplete`), montant HT (input décimal), taux TVA (`<Select>` des taux actifs), compte de
  contrepartie (`AccountAutocomplete`), bouton « Insérer les lignes ».
  - **Affichage des options du `<Select>` taux (M2)** : chaque option affiche le **taux formaté**
    (`VatRateResponse.rate`, ex. `8.10 %`) suivi du libellé de catégorie. **Fallback robuste exact**
    (`category` est `String` non-null mais peut être `""`) :
    ```ts
    const label = r.category && r.category.length > 0
      ? i18nMsg(`vat-category-${r.category}`, r.label)
      : (r.label || `${r.rate} %`);
    ```
    (évite d'interpoler la clé `vat-category-` vide → i18n key miss). La **valeur** de l'option est `r.rate`
    (string, passée telle quelle à `lineVatAmount`). Ne PAS dépendre de `category` pour le calcul (seul
    `rate` compte).
  - **Liste de taux vide (M3)** : si `getVatRates()` retourne `[]`, le `<Select>` n'a aucune option ; le
    bouton « Insérer les lignes » reste désactivé avec un message **dédié** (« Aucun taux TVA configuré —
    voir Paramètres → Taux TVA »), **distinct** du message config-requise compte (AC5).
- **AC2 — Calcul TVA avec parité backend (DC-c2)** : un helper TS `lineVatAmount(ht: string, ratePercent:
  string): string` calcule `round(ht × rate ÷ 100, 2, half-up)` via `big.js`. **Normalisation virgule (H1)** :
  `ht` et `ratePercent` sont normalisés `,→.` **avant** `new Big(...)` — réutiliser `parseAmount`
  (`balance.ts:32`, `new Big(raw.replace(',','.'))`, DRY) plutôt que `new Big` brut (sinon `new
  Big("1000,50")` throw alors que la saisie virgule passe `isValidAmount`). Parité prouvée par test :
  `lineVatAmount("1000", "8.10") === "81.00"`, `("1000","2.60") === "26.00"`, `("1000","3.80") === "38.00"`,
  `("1000","0") === "0.00"`, **cas virgule** `("1000,50","8.10") === "81.04"` (1000.50 × 8.10 / 100 =
  81.0405 → 81.04), et un **vrai cas tie-break half-up** `("0.10","5") === "0.01"` (0.10 × 5 / 100 = 0.0050
  exactement → half-up away-from-zero → `"0.01"`, alors qu'un half-even donnerait `"0.00"` — ce cas
  discrimine réellement le mode `Big.roundHalfUp`, contrairement à un produit non-milieu). Chaque valeur
  attendue est figée contre `line_vat_amount` backend (même arrondi `MidpointAwayFromZero`).
- **AC3 — Génération de l'écriture (DC-c6/DC-c7)** : « Insérer les lignes » génère, depuis (compte charge,
  HT, taux, compte contrepartie) :
  - `D charge` = HT sur le compte de charge ;
  - `D impôt préalable` = `lineVatAmount(HT, taux)` sur `defaultVatRecoverableAccountId` — **émise
    seulement si > 0** (DC-c7) ;
  - `C contrepartie` = `HT + vat` (TTC) sur le compte de contrepartie ;
  puis injecte ces lignes dans `lines` (strings via `Big...toFixed(2)`, L1), met `journal = 'Achats'`
  (DC-c5 — forcé car l'achat est sémantiquement un journal Achats, voir H3), et pré-remplit `description`.
  - **Format `description` (H2)** : clé i18n `vat-purchase-description` avec interpolation
    (`i18nMsg('vat-purchase-description', 'Achat — TVA {$rate} % récupérable', { rate })` — `i18nMsg`
    supporte l'interpolation `{$var}`, `i18n.svelte.ts:17`). Le `rate` interpolé = `VatRateResponse.rate`
    brut (ex. `"8.10"`). Si `vat == 0` (taux exempt), clé `vat-purchase-description-exempt`
    (`'Achat — sans TVA'`, sans interpolation). Description **éditable** après insertion.
  - **Écrasement journal/description (H3)** : `journal` est **toujours** écrasé à `'Achats'` (documenté,
    pas de confirmation — c'est le sens de l'assistant). La `description` n'est écrasée **que si elle est
    vide** (`description.trim() === ''`) ; si l'utilisateur a déjà saisi une description, elle est
    **préservée** (l'écriture générée ne l'écrase pas). Une description manuelle non vide compte aussi dans
    le déclencheur de confirmation AC6.
  L'écriture résultante est **équilibrée** (`balance.isBalanced === true`).
- **AC4 — Taux exempt/0 (DC-c7)** : si le taux choisi donne `vat == 0.00` → **2 lignes** générées (charge
  + contrepartie, montants égaux), **aucune ligne d'impôt préalable** (évite `debit = 0` rejeté par
  `chk_jel_debit_credit_exclusive`).
- **AC5 — Config requise (DC-c3)** : si `defaultVatRecoverableAccountId` est `null`, le bouton « Insérer
  les lignes » est **désactivé** et un message indique de configurer le compte d'impôt préalable dans
  **Paramètres → Facturation** (lien). Aucune génération possible sans compte cible.
- **AC6 — Confirmation avant remplacement (DC-c9)** : si le tableau des lignes contient déjà au moins une
  ligne **non vide** saisie à la main, OU une `description` non vide, « Insérer les lignes » demande
  confirmation avant de remplacer le brouillon. Sur un formulaire vierge (cas nominal), insertion directe
  sans confirmation.
  - **Pattern de modale (Pass 2, figé)** : réutiliser le **pattern de modale inline existant** du
    `JournalEntryForm` (la modale de conflit 409, `JournalEntryForm.svelte:395-422` : `<div role="dialog"
    aria-modal="true">` + carte `bg-card border border-border rounded-lg p-6` + `h2`/`p`/boutons
    `Annuler`/action). **NE PAS** introduire de composant `AlertDialog` (absent du projet — seul
    `components/ui/dialog` shadcn existe). Cohérence visuelle + HTTP-LAN-safe. `window.confirm()` est un
    fallback acceptable mais la modale inline est préférée (cohérence projet).
  - **Prédicat « ligne non vide » figé (M1)** : une `LineDraft` est non vide ssi
    `accountId !== null || debit.trim() !== '' || credit.trim() !== ''` (le `LineDraft` initial à la
    création est `{accountId: null, debit: '', credit: ''}`, donc vierge). Implémenter en helper testable
    `isDraftLineNonEmpty(line: LineDraft): boolean` (T-C1, couvert AC10) pour éviter toute divergence.
- **AC7 — Soumission via flux existant (DC-c5)** : après insertion, l'utilisateur peut éditer les lignes,
  puis soumet via le bouton « Valider » existant → `POST /journal-entries`. **Aucune route ni payload
  nouveaux.** L'écriture postée respecte les contraintes backend (équilibre, comptes actifs, journal
  Achats) — vérifié par un test E2E round-trip (AC10).
- **AC8 — Validation des entrées de l'assistant** : « Insérer les lignes » est désactivé tant que (compte
  charge sélectionné **ET** HT > 0 valide (≤ 4 décimales, format `balance.ts` `isValidAmount`) **ET** taux
  choisi **ET** compte contrepartie sélectionné **ET** compte charge ≠ compte contrepartie **ET** (Pass 3
  F-OPUS-C2) **compte charge ≠ compte impôt préalable** (`recoverableAccountId`) **ET** **compte
  contrepartie ≠ compte impôt préalable**). Rationale F-OPUS-C2 : si l'utilisateur choisissait le compte
  d'impôt préalable comme charge/contrepartie, l'écriture aurait 2 lignes sur le même compte `1171` —
  équilibrée mais **polluant le solde que 18-1d lira pour `total_vat_recoverable`** (un montant HT viendrait
  gonfler la TVA récupérable). Messages d'erreur inline dédiés, cohérents avec le style du formulaire.
- **AC9 — i18n** : tous les libellés de l'assistant via `i18nMsg('vat-purchase-*', '<fallback FR>')` :
  titre du panneau, labels des 4 champs (charge, HT, taux, contrepartie), bouton « Insérer les lignes »,
  `vat-purchase-description` (avec interpolation `{$rate}`) + `vat-purchase-description-exempt`, message
  config-requise compte (AC5), message liste de taux vide (M3, distinct), message de confirmation de
  remplacement (AC6), messages de validation inline (AC8 : charge = contrepartie, HT invalide). Fallbacks FR
  fournis dans le code. `npm run lint-i18n-ownership` passe. (Le format d'affichage des options de taux
  réutilise la clé existante `vat-category-*` si présente, cf. AC1/M2.)
- **AC10 — Tests** :
  - **Unitaires (vitest)** : helper `lineVatAmount` (parité AC2 : ≥6 cas dont virgule `("1000,50","8.10")`,
    tie-break half-up `("0.10","5")`, et 0) ; `buildPurchaseVatLines(...)` (3 lignes si vat>0 / 2 lignes si
    vat==0 ; équilibre `Σdebit == Σcredit` ; compte impôt préalable correct ; ordre charge→impôt→
    contrepartie ; **cas HT à 3-4 décimales `htAmount="100.005"` (F-OPUS-C1)** asserting `Σdebit === Σcredit`
    ET `charge` à 2 décimales exactement) ; `isDraftLineNonEmpty` (ligne vierge initiale → false ;
    accountId/debit/credit → true).
  - **E2E (Playwright)** : `frontend/tests/e2e/` — saisir un achat via l'assistant (HT 1000 @ 8.10 %),
    insérer, soumettre, vérifier que l'écriture créée a 3 lignes (`débit charge = 1000.00` / `débit impôt
    préalable = 81.00` / `crédit contrepartie = 1081.00`) et apparaît dans la liste des écritures.
    **Pré-requis seed (corrigé Pass 1)** : `default_vat_recoverable_account_id` **doit être configuré dans la
    fixture** — ce n'est PAS le cas aujourd'hui (cf. exception test-fixture + T-C5). Le test ne peut passer
    qu'après le patch fixture de T-C5.
- **AC11 — Quality gate** « Test Locally First » frontend : `npm run check` 0 err, `npm run
  lint-i18n-ownership` PASS, `npm run test:unit` vert, `npm run build` OK, `npm run test:e2e` vert (si suite
  lancée localement). Backend **non touché** → checks Rust inchangés (no-op).

## Tasks (T-C1..T-C6)

- **T-C1 — Helpers (parité + génération)** : créer `frontend/src/lib/features/journal-entries/vat-purchase.ts`
  exportant :
  - `lineVatAmount(ht: string, ratePercent: string): string` (big.js, half-up 2 déc., **normalisation
    virgule via `parseAmount`** — H1/DRY) ;
  - `buildPurchaseVatLines(params): LineDraft[]` où `params = { chargeAccountId, htAmount, ratePercent,
    counterpartyAccountId, recoverableAccountId }` → `LineDraft[]` (DC-c6/DC-c7 : 3 lignes si vat>0, 2 si 0,
    équilibre par construction). **Verrou F-OPUS-C1** : les 3 montants émis en strings via
    `Big(...).round(2, Big.roundHalfUp).toFixed(2)` — `débit charge = round₂(HT)` (PAS le HT brut),
    `débit impôt préalable = vat`, `crédit contrepartie = round₂(HT).plus(vat).toFixed(2)`. Test unitaire
    **obligatoire avec HT à 3-4 décimales** (ex. `htAmount = "100.005"`) asserting `Σdebit === Σcredit`
    **et** que `charge` n'a que 2 décimales (sinon déséquilibre `ENTRY_UNBALANCED`) ;
  - `isDraftLineNonEmpty(line: LineDraft): boolean` (M1 : `accountId !== null || debit.trim() !== '' ||
    credit.trim() !== ''`).
  Doc JSDoc + invariant de parité backend (`line_vat_amount`).
- **T-C2 — Composant assistant** : créer
  `frontend/src/lib/features/journal-entries/VatPurchaseAssistant.svelte` (panneau repliable). Props :
  `accounts: AccountResponse[]`, `recoverableAccountId: number | null`, `onApply: (lines: LineDraft[]) =>
  void`. État runes pour les 4 champs ; charge les taux via `getVatRates()` ; valide les entrées (AC8) ;
  bouton « Insérer les lignes » → `onApply(buildPurchaseVatLines(...))`. Config-requise si
  `recoverableAccountId == null` (AC5). IDs DOM via `$props.id()` (DC-c8). i18n `vat-purchase-*` (AC9).
- **T-C3 — Branchement dans `JournalEntryForm`** : importer/instancier `VatPurchaseAssistant` **en mode
  création uniquement** — détecté via la rune **existante** `isEdit = $derived(initialEntry !== null)`
  (`JournalEntryForm.svelte:43`) → rendre l'assistant sous `{#if !isEdit}`.
  - **Chargement du compte récupérable (Pass 2 H, figé)** : `recoverableAccountId: number | null` est une
    **nouvelle prop du `JournalEntryForm`**, chargée par la **page parent** `journal-entries/+page.svelte`
    via `getInvoiceSettings()` au montage — **strictement parallèle au chargement existant de `accounts`**
    (`+page.svelte:91 fetchAccounts()` passé en prop). En cas d'échec réseau de `getInvoiceSettings()`,
    traiter comme `null` (assistant en mode config-requise, AC5). NE PAS charger dans le form lui-même
    (cohérence avec le pattern props du form).
  - `onApply` : si lignes non vides (`isDraftLineNonEmpty`) OU description non vide → confirmation (AC6) ;
    sinon remplacer `lines`, set `journal='Achats'`, pré-remplir `description` si vide. Ne PAS modifier le
    payload POST ni la validation existante.
- **T-C4 — i18n (précisé Pass 3 F-OPUS-C3)** : ajouter les clés `vat-purchase-*` aux **4 fichiers**
  `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` (FR canonique + traductions DE/IT/EN, syntaxe
  Fluent `{ $rate }`). **Indispensable** : `all_messages()` (`loader.rs:130-133`) charge **FR-CH comme base**
  puis overlaye la locale ; si les clés ne sont ajoutées qu'à FR-CH, une instance DE/IT/EN afficherait
  l'assistant **en français** (la clé étant présente avec la valeur FR, le fallback inline du code
  `i18nMsg` **ne se déclenche jamais**). ⚠️ `lint-i18n-ownership` (frontend) ne vérifie PAS la parité des
  `.ftl` backend → un oubli DE/IT/EN passerait le quality gate **silencieusement**. **Réutiliser les clés
  `vat-category-*` existantes** (`fr-CH/messages.ftl:1064-1068`, déjà traduites 4 langues) pour l'affichage
  du Select (M2). Fallbacks FR inline dans le code (`i18nMsg(key, '<FR>')`) en filet de sécurité. (Note :
  les `.ftl` sont des **ressources de traduction**, pas du code de production — la story reste « zéro
  logique backend ».)
- **T-C5 — Fixture + tests** :
  - **(a) Patch fixture (test-only, C1)** : dans `crates/kesh-db/src/test_fixtures.rs`, ajouter
    `default_vat_recoverable_account_id` à l'`INSERT company_invoice_settings` des **deux** fixtures
    (`seed_accounting_company` `:152-165` **et** `seed_accounting_company_no_fy` `:259-271`), en
    **réutilisant le compte `1000`** (`accounts["1000"]`, Asset existant) — **PAS** de 6e compte (compteur
    `("accounts", 5)` inchangé, aucune assertion cassée — vérifier `test_endpoints_e2e.rs`,
    `exports_global_e2e.rs`, self-tests fixture). Étendre le self-test fixture (`:548-556` zone
    `default_vat_payable`) avec une assertion `default_vat_recoverable_account_id == accounts["1000"]`.
    Lancer `cargo test --workspace -j1 -- --test-threads=1` pour confirmer 0 régression.
  - **(b) Tests unitaires** : `vat-purchase.test.ts` (parité `lineVatAmount` + `buildPurchaseVatLines`,
    AC2/AC10 ; inclure le cas virgule `("1000,50","8.10")` et le cas tie-break half-up).
  - **(c) E2E Playwright** `vat_purchase_assistant.spec.ts` (AC10) — round-trip insertion→soumission→liste,
    avec `seedTestState('with-company')` (désormais `default_vat_recoverable_account_id` configuré via (a)).
    Vérifier qu'aucune assertion E2E existante de `journal-entries` ne casse.
- **T-C6 — Quality gate** « Test Locally First » frontend (AC11) + Change Log. **Pré-requis E2E** :
  `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64` (cf. `reference_playwright_ubuntu26`), MariaDB up +
  seed CI.

## Hors-scope (→ stories suivantes)

- **Remplissage `total_vat_recoverable` / `vat_balance` (18-1d)** : le solde du compte `1171` (alimenté par
  les écritures de cet assistant) sera lu du grand livre en 18-1d. 18-1c ne touche PAS `VatReport`.
- **Réconciliation rapport ↔ grand livre (18-1e)**.
- **Entité « facture d'achat » / workflow fournisseurs / paiements** : explicitement exclu (DC3=B). Pas de
  gestion d'échéances fournisseurs, pas d'OCR, pas de pièces jointes.
- **Sélection temporelle automatique du taux** (`find_for_category_at_date`) : l'utilisateur choisit le
  taux ; pas d'inférence par date d'écriture en 18-1c.
- **Doc utilisateur/CHANGELOG/README** : différés à **18-1f** (mi-epic, pas de release tant que le décompte
  AFC n'est pas livré).

### Migration / doc / backend (vérifié exploration)

- **Aucune migration**, **aucun changement backend**, **aucun changement d'API**. Story 100 % frontend.
- **Doc-sync différée 18-1f** (split umbrella).

## Risques

- **Parité d'arrondi (DC-c2)** : c'est le principal piège. Si le calcul TS dévie de `line_vat_amount` d'un
  centime, le crédit contrepartie ne sera pas équilibré et le `POST /journal-entries` sera rejeté
  (`ENTRY_UNBALANCED`). Mitigation : `buildPurchaseVatLines` calcule `vat` **une fois** et pose
  `crédit = HT + vat` (somme exacte) → équilibre par construction quelle que soit la valeur de `vat`. Le
  test de parité verrouille la valeur de `vat` vs backend.
- **`big.js` roundingMode** : utiliser explicitement `Big.roundHalfUp` (mode 1). Ne PAS se fier au
  `Big.RM` global (qui pourrait être configuré ailleurs) — passer le mode en 2e argument de `.round()`.
- **Compte impôt préalable non configuré** : géré par AC5 (assistant désactivé + message). Ne PAS générer
  une écriture incomplète.
- **Confusion charge ≠ contrepartie** : AC8 interdit le même compte des deux côtés (sinon écriture absurde
  mais techniquement équilibrée).
- **Échec réseau `getInvoiceSettings`/`getVatRates`** : dégrader proprement (assistant en mode
  config-requise / liste de taux vide + message), ne pas casser le formulaire d'écriture manuelle existant.
- **Double-clic « Insérer » (L2)** : bénin (l'injection remplace, donc idempotente) ; après une 1ʳᵉ
  insertion les lignes générées deviennent « non vides » → un 2ᵉ clic re-déclencherait la confirmation AC6.
  Acceptable ; optionnellement désactiver le bouton pendant l'injection.
- **Borne du HT (L3)** : `isValidAmount` borne les décimales (≤4) mais pas la magnitude ; un HT démesuré
  serait rejeté côté backend (`DECIMAL(19,4)`). Pas de garde front spécifique requis (cohérent avec le
  formulaire manuel existant), noté pour complétude.
- **Format string des 3 lignes (L1 + verrou F-OPUS-C1)** : `buildPurchaseVatLines` produit `debit`/`credit`
  via `Big(...).round(2, Big.roundHalfUp).toFixed(2)` (2 décimales). `débit charge = round₂(HT)` (PAS le HT
  brut — cf. DC-c7), `crédit = round₂(HT).plus(vat)`. L'équilibre `Σdebit == Σcredit` tient **exactement**
  par construction (le backend compare en égalité décimale exacte, sans epsilon).

## Prochaine étape

`bmad-create-story validate 18-1c` (rotation Sonnet→Haiku→Opus→…, contexte frais) avant la `dev-story`.

## Change Log

### `bmad-create-story validate 18-1c` — cycle adversarial (CLAUDE.md Review Iteration Rule)

| Passe | Modèle | Findings > LOW | Points clés |
|-------|--------|----------------|-------------|
| 1 | Sonnet 4.6 | 7 (1C+3H+3M) | **Ground-truth : 24/25 claims confirmées** — décision « frontend pur » prouvée valide (POST /journal-entries + GET /vat-rates + GET /company/invoice-settings + line_vat_amount existent). **C1 CRITICAL (vérifié ground-truth orchestrateur)** : la fixture `seed_accounting_company`/`_no_fy` ne configure PAS `default_vat_recoverable_account_id` (`test_fixtures.rs:152-165`/`:259-271` ne pose que receivable/revenue/payable) → E2E AC10 casserait + « zéro backend » faux. **Fix** : patch test-fixture réutilisant le compte `1000` existant (pattern 18-1b 2000-pour-payable, compteur inchangé) → T-C5(a) ; DC-c1 reformulé « zéro code production, 1 ligne test-fixture ». H1 : normaliser virgule via `parseAmount` avant `new Big` (sinon throw sur `"1000,50"`). H2 : figer clé i18n `vat-purchase-description` + interpolation `{$rate}` (i18nMsg supporte `{$var}`). H3 : journal forcé `Achats` (documenté), description écrasée seulement si vide. M1 : prédicat `isDraftLineNonEmpty` figé. M2 : affichage `<Select>` taux (rate formaté + fallback category vide). M3 : message dédié liste de taux vide. F1+L1-L4 : vrai cas tie-break half-up `("0.10","5")`, format `toFixed(2)`, double-clic, borne HT, libellé « 1081 »→montant. |

| 2 | Haiku 4.5 | 3 (1H+2M) | **Ground-truth 11/11 confirmé, 0 hallucination** (reviewer GT) : fix C1 valide (fixture ne pose pas recoverable, compte `1000` réutilisable, compteur intact, `create_in_tx` sans type-check), `parseAmount`/`i18nMsg {$var}`/`MidpointAwayFromZero` confirmés, cas tie-break `("0.10","5")` discriminant validé. **3 findings de clarification (gap hunter)** : H (T-C3) chargement `getInvoiceSettings()` ambigu → **figé : prop `recoverableAccountId` chargée par page parent** (parallèle `accounts`), mode création via `isEdit = $derived(initialEntry !== null)` (`:43`). M (AC1/M2) fallback `category` vide → **code TS exact** figé. M (AC6) type de modale → **réutiliser la modale inline existante** du form (conflit 409 `:395-422`), PAS `AlertDialog` (absent du projet — corrigé suggestion Haiku erronée). |

| 3 | Opus 4.8 | 3 (1H+2M) | **Catch-architectural** — design prouvé fondamentalement sain (axes 3/5 prouvés : réutilisation compte `1000` sans collision de solde, couplage 18-1d cohérent). **F-OPUS-C1 (HIGH)** : condition d'équilibre latente non figée — DC-c7 « débit charge = HT » contredisait L1 « toFixed(2) » ; sur un HT à 3-4 décimales (autorisé par `isValidAmount`), émettre le HT brut → déséquilibre `ENTRY_UNBALANCED` (backend = égalité décimale EXACTE, sans epsilon). Brute-force Opus : `round₂(HT) + vat == round₂(HT+vat)` toujours (0 contre-exemple). **Fix** : verrou `débit charge = round₂(HT)` + test HT 3-4 déc. **F-OPUS-C2 (MEDIUM)** : AC8 ne gardait pas `charge/contrepartie ≠ recoverable` → 2 lignes sur `1171`, équilibré mais **polluant le solde lu par 18-1d**. **Fix** : étendre AC8. **F-OPUS-C3 (MEDIUM)** : `all_messages` charge FR-CH en base+overlay (`loader.rs:130`) → clés `vat-purchase-*` ajoutées seulement à FR-CH ⟹ DE/IT/EN affichent l'assistant **en FR** (fallback code inopérant) ; `lint-i18n-ownership` ne couvre pas les `.ftl`. **Fix** : T-C4 exige les **4** `.ftl` ; réutiliser `vat-category-*` (déjà 4 langues). |

**Trend findings > LOW** : Pass 1 (Sonnet) 7 (1C+3H+3M) → Pass 2 (Haiku) 3 (1H+2M) → Pass 3 (Opus) 3 (1H+2M). Rotation Sonnet→Haiku→Opus, contexte frais. **Catch décisif Pass 3 Opus** : verrou numérique d'équilibre (F-OPUS-C1, bug prod latent sur HT 3-4 décimales) + garde anti-pollution solde 1171 (F-OPUS-C2) + parité i18n 4 locales (F-OPUS-C3). Prochaine : Pass 4 (Sonnet) contexte frais.
