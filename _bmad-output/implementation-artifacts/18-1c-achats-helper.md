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

**18-1c est une story FRONTEND PURE — aucun changement backend.** Un achat TVA récupérable est saisissable
intégralement via l'endpoint **existant** `POST /journal-entries` en 3 lignes équilibrées
(`D charge 6xxx / D impôt préalable 1171 / C fournisseur TTC`). Tout le backend nécessaire existe déjà :

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

- **DC-c1 (umbrella DC3=B)** — **frontend pur, zéro backend, zéro migration, zéro nouvel endpoint, zéro
  nouvelle entité.** L'écriture d'achat est postée via `POST /journal-entries` existant.
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
- **DC-c7 (équilibre par construction)** — `débit charge = HT`, `débit impôt préalable = vat`,
  `crédit contrepartie = HT + vat` (somme exacte, PAS de re-arrondi). `SUM(debit) = HT + vat = SUM(credit)`.
  Si `vat == 0` (taux exempt/0) → **2 lignes seulement** (charge + contrepartie), pas de ligne impôt
  préalable (cohérent F-OPUS-1 / contrainte `chk_jel_debit_credit_exclusive` qui interdit `debit = 0`).
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
- **AC2 — Calcul TVA avec parité backend (DC-c2)** : un helper TS `lineVatAmount(ht: string, ratePercent:
  string): string` calcule `round(ht × rate ÷ 100, 2, half-up)` via `big.js`. Parité prouvée par test :
  `lineVatAmount("1000", "8.10") === "81.00"`, `("1000","2.60") === "26.00"`, `("1000","3.80") === "38.00"`,
  `("1000","0") === "0.00"`, et un cas d'arrondi half-up (`("100.05","8.10")` → valeur identique à
  `line_vat_amount` backend = round(8.10405) = `"8.10"`).
- **AC3 — Génération de l'écriture (DC-c6/DC-c7)** : « Insérer les lignes » génère, depuis (compte charge,
  HT, taux, compte contrepartie) :
  - `D charge` = HT sur le compte de charge ;
  - `D impôt préalable` = `lineVatAmount(HT, taux)` sur `defaultVatRecoverableAccountId` — **émise
    seulement si > 0** (DC-c7) ;
  - `C contrepartie` = `HT + vat` (TTC) sur le compte de contrepartie ;
  puis injecte ces lignes dans `lines`, met `journal = 'Achats'`, et pré-remplit `description` (ex.
  « Achat — TVA <taux> % récupérable », éditable). L'écriture résultante est **équilibrée**
  (`balance.isBalanced === true`).
- **AC4 — Taux exempt/0 (DC-c7)** : si le taux choisi donne `vat == 0.00` → **2 lignes** générées (charge
  + contrepartie, montants égaux), **aucune ligne d'impôt préalable** (évite `debit = 0` rejeté par
  `chk_jel_debit_credit_exclusive`).
- **AC5 — Config requise (DC-c3)** : si `defaultVatRecoverableAccountId` est `null`, le bouton « Insérer
  les lignes » est **désactivé** et un message indique de configurer le compte d'impôt préalable dans
  **Paramètres → Facturation** (lien). Aucune génération possible sans compte cible.
- **AC6 — Confirmation avant remplacement (DC-c9)** : si le tableau des lignes contient déjà au moins une
  ligne **non vide** saisie à la main, « Insérer les lignes » demande confirmation (modale/`confirm`) avant
  de remplacer le brouillon. Sur un formulaire vierge (cas nominal), insertion directe sans confirmation.
- **AC7 — Soumission via flux existant (DC-c5)** : après insertion, l'utilisateur peut éditer les lignes,
  puis soumet via le bouton « Valider » existant → `POST /journal-entries`. **Aucune route ni payload
  nouveaux.** L'écriture postée respecte les contraintes backend (équilibre, comptes actifs, journal
  Achats) — vérifié par un test E2E round-trip (AC10).
- **AC8 — Validation des entrées de l'assistant** : « Insérer les lignes » est désactivé tant que (compte
  charge sélectionné **ET** HT > 0 valide (≤ 4 décimales, format `balance.ts` `isValidAmount`) **ET** taux
  choisi **ET** compte contrepartie sélectionné **ET** compte charge ≠ compte contrepartie). Messages
  d'erreur inline cohérents avec le style du formulaire.
- **AC9 — i18n** : tous les libellés de l'assistant via `i18nMsg('vat-purchase-*', '<fallback FR>')`
  (titre, labels des 4 champs, bouton, message config-requise, message confirmation). Fallbacks FR fournis.
  `npm run lint-i18n-ownership` passe.
- **AC10 — Tests** :
  - **Unitaires (vitest)** : helper `lineVatAmount` (parité AC2, ≥5 cas dont arrondi half-up et 0) ; helper
    de génération de lignes `buildPurchaseVatLines(...)` (3 lignes si vat>0 / 2 lignes si vat==0 ;
    équilibre `Σdebit == Σcredit` ; compte impôt préalable correct ; ordre charge→impôt→contrepartie).
  - **E2E (Playwright)** : `frontend/tests/e2e/` — saisir un achat via l'assistant (HT 1000 @ 8.10 %),
    insérer, soumettre, vérifier que l'écriture créée a 3 lignes (D 1000 charge / D 81 impôt préalable /
    C 1081 contrepartie) et apparaît dans la liste des écritures. Pré-requis seed : `defaultVatRecoverable
    AccountId` configuré (le seed CI a le compte `1171` via 18-1a + settings).
- **AC11 — Quality gate** « Test Locally First » frontend : `npm run check` 0 err, `npm run
  lint-i18n-ownership` PASS, `npm run test:unit` vert, `npm run build` OK, `npm run test:e2e` vert (si suite
  lancée localement). Backend **non touché** → checks Rust inchangés (no-op).

## Tasks (T-C1..T-C6)

- **T-C1 — Helper de calcul TVA (parité)** : créer `frontend/src/lib/features/journal-entries/vat-purchase.ts`
  exportant `lineVatAmount(ht: string, ratePercent: string): string` (big.js, half-up 2 déc.) **et**
  `buildPurchaseVatLines(params): LineDraft[]` où `params = { chargeAccountId, htAmount, ratePercent,
  counterpartyAccountId, recoverableAccountId }` → `LineDraft[]` (DC-c6/DC-c7 : 3 lignes si vat>0, 2 si 0,
  équilibre par construction, débit charge HT + débit impôt préalable vat + crédit contrepartie TTC). Doc
  JSDoc + invariant de parité backend.
- **T-C2 — Composant assistant** : créer
  `frontend/src/lib/features/journal-entries/VatPurchaseAssistant.svelte` (panneau repliable). Props :
  `accounts: AccountResponse[]`, `recoverableAccountId: number | null`, `onApply: (lines: LineDraft[]) =>
  void`. État runes pour les 4 champs ; charge les taux via `getVatRates()` ; valide les entrées (AC8) ;
  bouton « Insérer les lignes » → `onApply(buildPurchaseVatLines(...))`. Config-requise si
  `recoverableAccountId == null` (AC5). IDs DOM via `$props.id()` (DC-c8). i18n `vat-purchase-*` (AC9).
- **T-C3 — Branchement dans `JournalEntryForm`** : importer/instancier `VatPurchaseAssistant` (mode
  création uniquement, pas en édition). Charger `defaultVatRecoverableAccountId` via `getInvoiceSettings()`
  (au montage de la page parent ou du form ; gérer l'échec réseau en désactivant l'assistant comme
  config-requise). `onApply` : si lignes non vides présentes → confirmation (AC6) ; sinon remplacer `lines`,
  set `journal='Achats'`, pré-remplir `description`. Ne PAS modifier le payload POST ni la validation.
- **T-C4 — i18n** : ajouter les clés `vat-purchase-*` (FR + miroirs DE/IT/EN selon convention du projet —
  vérifier le mécanisme de messages i18n et `lint-i18n-ownership`). Fallbacks FR dans le code.
- **T-C5 — Tests unitaires** : `vat-purchase.test.ts` (parité `lineVatAmount` + `buildPurchaseVatLines`,
  AC2/AC10). + E2E Playwright `vat_purchase_assistant.spec.ts` (AC10) — round-trip insertion→soumission→
  liste, avec `seedTestState('with-company')` (le seed a `1171` + settings via 18-1a). Vérifier qu'aucune
  assertion E2E existante de `journal-entries` ne casse.
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

## Prochaine étape

`bmad-create-story validate 18-1c` (rotation Sonnet→Haiku→Opus→…, contexte frais) avant la `dev-story`.
