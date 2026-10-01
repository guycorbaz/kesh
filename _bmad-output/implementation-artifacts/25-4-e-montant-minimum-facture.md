# Story 25.4-e : Un montant minimum configurable sous lequel une facture n'est pas émise

Status: ready-for-dev

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
- Dépôt : `CompanyInvoiceSettingsUpdate.minimum_invoice_amount: Option<Decimal>`, `UPDATE`, `is_no_op_change` ;
  l'audit suit (`COLUMNS`).
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

- [ ] **T1 — la migration** (AC 1) et ses garde-fous.
- [ ] **T2 — le refus** (AC 2, 3).
- [ ] **T3 — le réglage** (AC 4) : dépôt, route, écran.
- [ ] **T4 — textes** (AC 5).
- [ ] **T5 — tests et mutations** (AC 6).
- [ ] **T6 — gates** : backend complet (migration), frontend complet, **E2E complet**.

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

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-01** — Créée : réglage `minimum_invoice_amount` (`NULL` = aucun seuil), refus dédié à la validation
  sur le total arrondi, avoirs et rappels hors champ, API qui préserve l'absent, écran, manuels.

[#495]: https://github.com/guycorbaz/kesh/issues/495
