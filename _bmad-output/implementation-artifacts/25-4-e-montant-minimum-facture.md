# Story 25.4-e : Un montant minimum configurable sous lequel une facture n'est pas émise

Status: review

**Issue : [#495]** (CR) — ⛔ la PR porte `closes #495`, titre ET corps.

**Empilée sur la 25-4-c4** (branche `story/25-4-e-montant-minimum-facture`, partie de
`story/25-4-c4-arrondi-facture-5-centimes`) : elle lit le total **arrondi** que la c4-a calcule à la validation,
et elle touche le même écran de réglages que la c4-b.

## Arbitrages (Project Lead, 2026-10-01, #495)

1. **Le montant minimum est configurable** : un réglage par société dans *Paramètres → Facturation*.

Retenus par défaut, sur recommandation, **révisables** :

2. **Aucun seuil par défaut** : la règle ne s'applique qu'une fois un montant fixé.
3. **Refus à la validation**, message qui donne le seuil ; le brouillon reste enregistrable.
4. **Les avoirs ne sont pas soumis au seuil.**

Deux points de spécification, tranchés ici sur recommandation (révisables) :
- le seuil se compare au **total TTC arrondi**, celui que le client paiera (brut si l'arrondi est désactivé) ;
- les **rappels** ne sont pas concernés : ils réclament une facture déjà émise.

## Story

En tant que petite entreprise,
je veux fixer un montant sous lequel Kesh refuse d'émettre une facture,
afin de ne pas envoyer de factures qui coûtent plus à traiter qu'elles ne rapportent.

## Les faits, vérifiés dans le code

- **La validation** (`crates/kesh-db/src/repositories/invoices.rs`, `validate_invoice`) calcule à l'étape « 2 bis' »
  (`:1993-2005`, c4-a) le TTC brut, l'arrondi et le **total arrondi**, et refuse un total arrondi nul
  (`InvalidInput("invoiceTotalZero")`) **avant** de réclamer un compte d'arrondi.
- **Les réglages** : `company_invoice_settings` (entité, `COLUMNS`, `CompanyInvoiceSettingsUpdate`, `UPDATE`,
  `is_no_op_change`, instantané d'audit), route `routes/company_invoice_settings.rs` (corps à champs **préservés
  s'ils sont absents** pour les ajouts récents : compte d'arrondi, `roundTo5Centimes`), écran
  `settings/invoicing/+page.svelte` (section *Différences d'arrondi*).
- **L'avoir** (`credit_notes::create_credit_note`) ne passe pas par `validate_invoice` : il échappe au seuil par
  construction.

## Acceptance Criteria

**AC 1 — La migration** (non breaking) : `company_invoice_settings.minimum_invoice_amount DECIMAL(19,4) NULL`,
`NULL` = aucun seuil (le défaut, arbitrage 2). Garde-fous du dépôt :
- **P5** : ligne d'audit, compteurs recomptés (72) ;
- **P6** : `migrations_upgrade_path.rs` 71 → 72 et 37 → 38, valeurs grepées sur tout le fichier ;
- squash régénéré, `migrations.sha384` ;
- export de souveraineté (`csv_tables.rs`, garde `chaque_colonne_du_schema_est_exportee_ou_ecartee`) ;
- sauvegarde antérieure sans la colonne → `NULL`.

**AC 2 — Le refus à la validation.** Juste après le refus du total arrondi nul, avant le compte d'arrondi : si
`minimum_invoice_amount` est posé et que le **total TTC arrondi** lui est **strictement inférieur**, la
validation est refusée sans rien écrire, la facture reste brouillon. Variante dédiée
`DbError::InvoiceBelowMinimum { total, minimum }` → **400** `INVOICE_BELOW_MINIMUM`, message qui **nomme les deux
montants** au centime (« Le total de cette facture, CHF 4.50, est inférieur au montant minimum fixé dans
Paramètres → Facturation, CHF 5.00. ») — clé `error-invoice-below-minimum` dans les 4 locales. Total égal au
seuil : accepté.

**AC 3 — Les avoirs et les rappels** ne sont pas touchés (aucun code ; un test le prouve pour l'avoir d'une
facture émise avant qu'un seuil plus haut soit fixé).

**AC 4 — Le réglage, API et écran.**
- Dépôt (`crates/kesh-db/src/repositories/company_invoice_settings.rs`) — ⛔ **chaque site qui énumère les
  colonnes à la main** (validation P1) :
  - l'entité `CompanyInvoiceSettings` et `COLUMNS` (`:28-34`, qui sert `get_or_create_default*` et les lectures
    `before`/`after` de `update`) ;
  - les **deux `SELECT` préfixés `cis.`** de `insert_with_defaults` et `insert_with_defaults_in_tx` (branche
    `rows == 0`, `:493` et `:617`), qui **ne dérivent pas** de `COLUMNS` : un oubli y fait échouer en
    `ColumnNotFound` le chemin idempotent de l'onboarding et du seed (test `company_invoice_settings_repository.rs:58`) ;
  - l'instantané d'audit `settings_snapshot_json` (`:36-53`), manuscrit : un oubli y rend la piste d'audit
    **silencieusement** incomplète sur le changement de seuil ;
  - `CompanyInvoiceSettingsUpdate.minimum_invoice_amount: Option<Decimal>`, l'`UPDATE`, `is_no_op_change`.
- Hors du dépôt, les autres sites qui énumèrent les champs des réglages : l'export de souveraineté
  (`crates/kesh-api/src/exports/csv_tables.rs`, en-tête et valeurs de `company_invoice_settings`, cf. AC 1) ; les
  types frontend `InvoiceSettingsResponse` et `UpdateInvoiceSettingsRequest`
  (`frontend/src/lib/features/invoices/invoices.types.ts`) ; les montages de test qui construisent
  `CompanyInvoiceSettingsUpdate` (`crates/kesh-db/tests/company_invoice_settings_repository.rs`) ou une réponse de
  réglages (`settings-invoicing-page.test.ts`).
- Route : `minimumInvoiceAmount` en `GET` et `PUT` — **absent du corps : préservé ; présent à `null` : effacé**
  (`double_option`, patron du compte d'arrondi). Validation : **strictement positif**, au plus deux décimales
  (`scale_within(&v.normalize(), 2)`), sinon 400.
- Écran : dans *Paramètres → Facturation*, une section ou un champ **« Montant minimum d'une facture »**
  (montant, vide = aucun), avec une aide (« une facture dont le total est inférieur ne peut pas être validée ;
  les avoirs ne sont pas concernés »). Libellés dans les 4 locales, `sitesTotal` recompté, `data-testid`.

**AC 5 — Textes.** `user-manual.tex` (validation d'une facture : le refus et le réglage), `admin-manual.tex`
(paramètres de facturation), PDF régénérés et contrôlés aplatis ; CHANGELOG `[0.12.1]` *Added* (#495).

**AC 6 — Tests.** Chacun aurait échoué avant le patch :
- seuil 5.00 : facture de 4.50 refusée (code, message nommant 4.50 et 5.00, brouillon intact) ; 5.00 accepté ;
  total **brut** 4.98 arrondi 5.00 accepté (comparaison au total arrondi) ;
- aucun seuil : 0.05 accepté ;
- avoir d'une facture émise sous un seuil fixé ensuite : accepté ;
- API : `PUT` qui pose, préserve (absent), efface (`null`) ; valeur nulle, négative ou à trois décimales refusée ;
- Vitest de l'écran ; E2E : fixer le seuil, voir la validation refusée avec le message, l'effacer ;
- sauvegarde : import d'une sauvegarde sans la colonne ;
- mutations : comparaison au brut au lieu de l'arrondi ; `<` remplacé par `<=` ; seuil ignoré.

## Tasks / Subtasks

- [x] **T1 — la migration** (AC 1) et ses garde-fous.
- [x] **T2 — le refus** (AC 2, 3).
- [x] **T3 — le réglage** (AC 4) : dépôt, route, écran.
- [x] **T4 — textes** (AC 5).
- [x] **T5 — tests et mutations** (AC 6).
- [x] **T6 — gates** : backend complet (migration), frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Comparer au TTC brut** : le client paie le total arrondi ; 4.98 arrondi 5.00 n'est pas sous un seuil de 5.00.
- ⛔ **Bloquer l'enregistrement du brouillon** : seul l'acte d'émission est refusé.
- ⛔ **Appliquer le seuil à l'avoir** : il annule une facture déjà émise, quel que soit son montant.
- ⚠️ **Fixer un seuil ne touche aucune facture émise** — elles le sont déjà.

### Modules

`kesh-db`, `kesh-api`, `frontend`, `kesh-i18n` (+ `docs`) — sous le seuil.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`).

### Debug Log References

- Clippy `collapsible_if` sur la validation du seuil dans la route (fusionné en `if let … &&`).
- Le premier gate n'a pas tourné : la chaîne de remise à zéro a échoué avant `test-fast.sh` (base pas encore
  prête) ; rejouée pas à pas.

### Completion Notes List

- **Migration** `20261001000002` (`minimum_invoice_amount DECIMAL(19,4) NULL`) : somme de contrôle, squash, audit
  (72 lignes, 8 + 64, recomptés), `migrations_upgrade_path.rs` 71 → 72 et 37 → 38.
- **Refus** à l'étape « 2 bis'' » de `validate_invoice`, sur le total arrondi, `<` strict ; variante
  `DbError::InvoiceBelowMinimum { total, minimum }` → 400 `INVOICE_BELOW_MINIMUM`, message à deux montants (clé
  `error-invoice-below-minimum`, 4 locales). L'avoir n'y passe pas.
- **Réglage** à tous les sites nommés par l'AC 4 : entité (deux structs), `COLUMNS`, instantané d'audit, les deux
  `SELECT` `cis.`-préfixés, `UPDATE`, `is_no_op_change` ; route (`double_option`, validation positif et au centime) ;
  export CSV (`fmt_opt_decimal`) ; types frontend ; montages de test.
- **Écran** : section *Montant minimum*, champ texte (vide → `null`) et aide ; `data-testid` posés sur le bouton
  « Valider », sa confirmation et sa zone d'erreur (la spec E2E ne pouvait pas les cibler autrement). `sitesTotal`
  1766 → 1769 recompté (réglages 38 → 41).
- **Textes** : manuel utilisateur (paragraphe sous la validation), manuel admin (paramètres), PDF régénérés et
  contrôlés aplatis, CHANGELOG *Added*.
- **Tests** (périmètre : ce commit contre `87d022ee`) : **7** Rust (4 `invoices_validate_vat.rs`, 1
  `idor_multi_tenant_e2e.rs`, 1 `invoice_echeancier_e2e.rs`, 1 `admin_full_import_e2e.rs`), **1** Vitest, **1** spec
  Playwright (`invoice-minimum-amount.spec.ts`).
- **Mutations**, toutes tuées : comparaison au brut (1 rouge), `<` → `<=` (2), seuil ignoré (1).
- **Gates** : backend complet sur base remise à zéro — **2576/2576** ; frontend complet — 0 erreur, lint PASS,
  **851/851**, build ; **E2E complet** sur `kesh_e2e` reconstruite, 14:12 UTC — **230 passés, 19 ignorés,
  8 échecs** : les 7 KF-029, et `product-revenue-account:133`, pollution d'état (verte rejouée seule, avec la spec
  neuve).

### File List

- `crates/kesh-db/migrations/20261001000002_invoice_minimum_amount.sql` (neuf), `migrations.sha384`, squash
- `crates/kesh-db/src/errors.rs`, `crates/kesh-db/src/entities/company_invoice_settings.rs`,
  `crates/kesh-db/src/repositories/{company_invoice_settings,invoices}.rs`
- `crates/kesh-db/tests/{invoices_validate_vat,company_invoice_settings_repository,migrations_upgrade_path}.rs`
- `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/company_invoice_settings.rs`,
  `crates/kesh-api/src/exports/csv_tables.rs`
- `crates/kesh-api/tests/{idor_multi_tenant_e2e,invoice_echeancier_e2e,admin_full_import_e2e}.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `frontend/src/lib/features/invoices/invoices.types.ts`, `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/src/routes/(app)/settings/invoicing/{+page.svelte,settings-invoicing-page.test.ts}`
- `frontend/src/routes/(app)/invoices/[id]/+page.svelte` (`data-testid`)
- `frontend/tests/e2e/invoice-minimum-amount.spec.ts` (neuf)
- `docs/migrations-idempotence-audit.md`, `docs/manual/fr/{admin,user}-manual.tex` + `.pdf`, `CHANGELOG.md`

## Change Log

- **2026-10-01** — Créée : réglage `minimum_invoice_amount` (`NULL` = aucun seuil), refus dédié à la validation
  sur le total arrondi, avoirs et rappels hors champ, API qui préserve l'absent, écran, manuels.
- **2026-10-01** — Validation P1 (Sonnet) : 1 HIGH, 1 MED, retenus. La formule « l'audit suit (`COLUMNS`) » laissait
  croire à un site unique : les deux `SELECT` `cis.`-préfixés des `insert_with_defaults*` (échec `ColumnNotFound`
  au chemin idempotent) et l'instantané `settings_snapshot_json` (audit incomplet sans signal) énumèrent les
  colonnes à la main — nommés à l'AC 4.
- **2026-10-01** — Validation P2 ciblée (Haiku) : 1 MED rendu, **reclassé LOW** — l'export `csv_tables.rs`, que l'AC 1
  nommait déjà et que sa garde automatique imposerait ; rendu explicite à l'AC 4, avec les types frontend et les
  montages de test. **Boucle close** : 1 HIGH/1 MED → 0 au-dessus de LOW ; Sonnet → Haiku ; remédiation sur la fiche
  seule.
- **2026-10-01** — Implémentée (T1–T6) : réglage `minimum_invoice_amount`, refus `INVOICE_BELOW_MINIMUM` sur le total
  arrondi, écran, manuels. 7 tests Rust, 1 Vitest, 1 spec Playwright neufs ; 3 mutations tuées. Gates : backend
  2576/2576, frontend 851/851, E2E 230/19/8 expliqués.

[#495]: https://github.com/guycorbaz/kesh/issues/495
