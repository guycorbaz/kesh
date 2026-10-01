# Story 25.4-d2a : Solder le reste — l'écriture et son annulation

Status: ready-for-dev

**Issues : [#384], [#490]** — ⛔ la PR porte `refs #384, refs #490` : la **25-4-d2b** (le bouton) fermera #490, la
**25-4-d2c** (le rapport TVA) fermera #384.

**Première des trois stories de la 25-4-d2.** Empilée sur la 25-4-d1 (branche `story/25-4-d2a-ecriture-de-solde`).

## Arbitrages (Project Lead)

Rendus le 2026-10-01 (25-4-d, cf. `25-4-d1-comptes-de-solde.md`) :

1. Quatre natures : **escompte accordé**, **frais bancaires**, **perte sur débiteur**, **reste d'arrondi** (#490).
2. L'utilisateur **choisit la nature** ; **aucun seuil**.
3. Un compte par nature, réglé dans les paramètres (posé par la 25-4-d1 ; le reste d'arrondi utilise le compte de
   différences d'arrondi).
4. **TVA corrigée au prorata des taux** de la facture pour l'escompte et la perte ; pas pour les frais ni l'arrondi.
5. Un bouton « Solder le reste » sur la fiche, **annulable par contre-passation**.
6. Toute facture **validée non soldée** peut être soldée, réglée en partie ou pas du tout.
7. Le rapport TVA retranche la TVA des soldes.

**Retenus par défaut le 2026-10-01** (Guy : « pousse puis continue », sans trancher ; révisables) :

- **Découpage de la d2 en trois** : d2a (cette story — l'écriture, l'annulation, côté serveur), d2b (le bouton, le
  dialogue, la liste, le manuel utilisateur), d2c (le rapport TVA). La d2 entière touchait six modules de code.
- **Le solde est une ligne de `invoice_settlements`**, de type `write_off` : le reste dû, l'échéancier, la balance
  âgée, le rapprochement, `paid_at` et l'annulation existante le suivent sans modification. Comme un règlement, il
  bloque l'avoir et la dévalidation tant qu'il n'est pas annulé.
- **Rapport TVA** (d2c) : retrancher la TVA des soldes, par taux, dans la période du solde ; #390 reste à part. D'où,
  ici, la **ventilation par taux figée** sur la ligne de solde.
- Le solde porte sur le **reste exact** (brut, quatre décimales) ; la créance tombe à zéro.

## Story

En tant que comptable,
je veux solder ce qui reste dû sur une facture en choisissant la nature de l'écart,
afin que la créance se ferme et que l'écart — et la TVA qu'il corrige — soit passé au bon compte.

## Les faits, vérifiés dans le code (inventaire du 2026-10-01)

- **`invoice_settlements`** (`20260827000001`, `20260828000001`) : `settlement_type VARCHAR(20)`,
  `settlement_bank_account_id`, `settlement_account_id` ; `chk_invoice_settlements_type` (`bank_transfer`,
  `internal_account`) et `chk_invoice_settlements_counterparty` (une référence par mode) ; `amount > 0` ;
  `UNIQUE (journal_entry_id)`. Le reste dû retranche **toute** ligne de la table (`INVOICE_AMOUNT_DUE_DERIVED_SQL`,
  `invoice_settlements.rs:119`), quel que soit son type.
- **`settle_invoice`** (`invoice_settlements_write.rs:45-323`) est le patron : verrou `FOR UPDATE` de la facture,
  statut `validated`, borne `settled_on >= date − 1 jour`, créance = **première ligne de débit** de l'écriture de vente,
  exercice ouvert couvrant la date, `journal_entries::create_in_tx` (verrou de période, exercice clos, comptes),
  `invoice_settlements::create_in_tx`, `paid_at` + **`version + 1`** (invariant écrit à `:251-262`), audit dans la
  transaction.
- **`cancel_settlement_in_tx`** (`:412-560`) contre-passe **toutes** les lignes de l'écriture, supprime la ligne de
  règlement, remet `paid_at` à `NULL` si un reste réapparaît, `version + 1`, audit. Il ne regarde pas le type.
- **`SettlementChoice`** est partagé avec le fournisseur (`entities/supplier_invoice.rs:104-110`) : ⛔ ne pas y ajouter
  de variante — le solde a son propre chemin.
- **Ventilation de TVA** : `vat_breakdown_by_rate` (`kesh-core/src/accounting/vat.rs:123-153`), TVA arrondie par ligne,
  taux > 0 seulement. **Aucune fonction de prorata.** L'écriture de vente crédite la TVA due par taux sur
  `default_vat_payable_account_id` ; l'avoir la débite de même (`credit_notes.rs:187-257`) — précédent direct.
- **Comptes de nature** : seuls `rounding_account_for_write` (`company_invoice_settings.rs:400-427`) relit un compte
  désigné au moment d'écrire (actif, imputable, charge ou produit, `FOR UPDATE`).
- **Lignes d'écriture** en `DECIMAL(19,4)` : une écriture peut créditer la créance au reste exact (la c3-b le fait).
- **#490** : un reste brut inférieur au demi-centime (`10.0040` après un avoir de `10.00`) est **insoldable** —
  tout paiement positif est un trop-perçu.

## Acceptance Criteria

**AC 1 — La migration** (`20261001000004_invoice_settlements_write_off.sql`).
- `ADD COLUMN write_off_nature VARCHAR(20) NULL`, `ADD COLUMN write_off_vat JSON NULL` (la ventilation figée, AC 4).
- `chk_invoice_settlements_type` recréée avec `'write_off'` ; `chk_invoice_settlements_counterparty` recréée avec une
  troisième branche : `write_off` ⇒ `settlement_account_id IS NOT NULL AND settlement_bank_account_id IS NULL`.
- Nouvelle `chk_invoice_settlements_write_off_nature` : `(settlement_type = 'write_off') = (write_off_nature IS NOT
  NULL)` et `write_off_nature IN ('discount', 'bank_fees', 'bad_debt', 'rounding')`.
- **DDL seul** (P7 sans objet) ; **non breaking** à démontrer dans l'en-tête (un binaire antérieur lit
  `settlement_type` en chaîne, ignore les deux colonnes, contre-passe un solde comme un règlement ; `DROP CONSTRAINT`
  n'est pas une opération listée en P3) — la passe de revue le confirme. P5 (audit, 74), P6 (`migrations_upgrade_path.rs`
  73 → 74, 39 → 40), squash, `migrations.sha384`, export CSV des deux colonnes, sauvegarde antérieure → `NULL`.

**AC 2 — Le prorata** (`kesh-core/src/accounting/vat.rs`). `write_off_vat_shares(lines, total_ttc, amount) ->
Vec<VatRateShare { rate_percent, base_ht, vat_amount }>` : pour chaque taux > 0 de `vat_breakdown_by_rate(lines)`,
`vat_amount = round_centime(amount × vat_r / total_ttc)` et `base_ht = round_centime(amount × base_r / total_ttc)`, où
`total_ttc` est le TTC **figé** de la facture (lignes + `rounding_amount`). Tests : un taux, plusieurs taux, lignes à 0 %
(aucune part), arrondi figé (la part sans TVA ne porte rien), `amount == total_ttc` (la TVA corrigée égale la TVA
facturée), montant à quatre décimales, `amount` minuscule (parts nulles omises).

**AC 3 — Le compte au moment d'écrire.** `write_off_account_for_write(conn, company_id, nature)` généralise
`rounding_account_for_write` (**un seul code**, le compte d'arrondi en devient un cas) : la colonne de la nature
(`rounding` → `default_rounding_account_id`), actif, imputable, charge ou produit, `FOR UPDATE`. Absent ou invalide →
`DbError::WriteOffAccountNotConfigured { nature }` → **400 `WRITE_OFF_ACCOUNT_NOT_CONFIGURED`**, message qui nomme la
nature et renvoie à *Paramètres → Facturation* (4 locales). `rounding_account_for_write` garde son erreur et ses
messages.

**AC 4 — L'écriture de solde.** `write_off_invoice(pool, user_id, company_id, invoice_id, nature, settled_on) ->
WriteOffOutcome { journal_entry_id, amount }`, sur le patron de `settle_invoice` :
1. verrou de la facture `FOR UPDATE`, statut `validated`, borne de date ;
2. reste brut `A = amount_due` ; `A <= 0` → refus `InvalidInput("nothingToWriteOff")` ;
3. nature `rounding` : seulement si `A < 0.05` (un reste d'arrondi ne dépasse pas l'unité de 5 centimes), sinon
   `InvalidInput("writeOffRoundingTooLarge")` ;
4. compte de la nature (AC 3) ; créance = première ligne de débit de l'écriture de vente ;
5. escompte et perte : parts de TVA (AC 2) ; si une part est non nulle, compte `default_vat_payable_account_id`
   **courant** (comme l'avoir), absent → `ConfigurationRequired` ;
6. exercice ouvert couvrant `settled_on` ; écriture au journal **OD**, libellé « Solde facture {numéro} — {nature} »,
   `project_id` de la facture, lignes : **débit** du compte de la nature `A − Σ TVA`, **débit** de la TVA due par taux,
   **crédit** de la créance `A` ;
7. ligne `invoice_settlements` : `settlement_type = 'write_off'`, `settlement_account_id` = compte de la nature,
   `amount = A`, `write_off_nature`, `write_off_vat` = les parts (JSON `[{ratePercent, baseHt, vatAmount}]`, `[]` sans
   TVA) ;
8. `paid_at = settled_on`, **`version + 1`** ;
9. audit `invoice.written_off` (nature, montant, TVA corrigée, écriture), libellé dans `audit_labels.rs` (4 locales).

**AC 5 — La route.** `POST /api/v1/invoices/{id}/write-off` `{ nature, settledOn }` — rôle Comptable+, **sans montant**
(le serveur solde le reste exact). Réponse `{ invoice, journalEntryId, amount }`. Nature inconnue → 400. Toutes les
erreurs de l'AC 4 mappées (clés i18n, 4 locales). Inscrite au registre d'audit des routes.

**AC 6 — La liste et l'annulation.** `GET …/settlements` : `settlementType = "write_off"` et un champ
`writeOffNature` (`null` hors solde). L'annulation existante (`…/settlements/{id}/cancel`) contre-passe le solde,
**TVA comprise**, rouvre la facture (`paid_at = NULL`), `version + 1` — prouvé par test, sans code nouveau si
possible.

**AC 7 — Le CHANGELOG** `[0.12.1]` *Added* : le solde du reste par l'API (`POST …/write-off`), l'écran suit (d2b).
Le manuel utilisateur vient avec la d2b.

**AC 8 — Tests.** Chacun aurait échoué avant le patch :
- prorata (AC 2) ;
- une facture **non réglée** et une facture **réglée en partie** soldées, pour chaque nature : écriture équilibrée,
  comptes et montants, TVA par taux (escompte, perte), aucune TVA (frais, arrondi), reste dû nul, `paid_at`, `version`,
  audit ;
- **#490** : un reste de `0.0040` soldé en nature `rounding`, créance à zéro ;
- refus : brouillon, facture soldée, compte non configuré ou archivé, arrondi ≥ 0.05, date avant la facture, exercice
  clos, période verrouillée, compte TVA absent ;
- annulation : reste rétabli, TVA contre-passée, `paid_at = NULL` ; l'avoir et la dévalidation sont refusés tant que
  le solde existe ;
- API : rôle (Consultation refusée), nature inconnue, liste avec `writeOffNature` ;
- sauvegarde antérieure sans les colonnes.

## Tasks / Subtasks

- [ ] **T1 — migration** (AC 1).
- [ ] **T2 — prorata** (AC 2).
- [ ] **T3 — compte au moment d'écrire** (AC 3).
- [ ] **T4 — écriture de solde** (AC 4).
- [ ] **T5 — route, liste** (AC 5, AC 6).
- [ ] **T6 — CHANGELOG** (AC 7).
- [ ] **T7 — tests** (AC 8).
- [ ] **T8 — gates** : backend complet (migration), frontend (non touché : `check` et `test:unit` pour la forme),
  **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ Ajouter une variante à `SettlementChoice` : partagé avec le fournisseur, dont le CHECK n'admet que deux valeurs.
- ⛔ Créer une table pour la ventilation : une table neuve rend **toutes** les sauvegardes antérieures inimportables
  (l'inventaire des tables doit être identique) — d'où la colonne JSON.
- ⛔ Recopier `rounding_account_for_write` : le généraliser.
- ⛔ Un numéro de compte dans le code.
- ⛔ Toucher le frontend : la d2b. (Conséquence assumée : jusqu'à la d2b, l'écran affiche un solde fait par l'API comme
  « Espèces ou autre compte ».)

### Modules

`kesh-core` (prorata), `kesh-db`, `kesh-api`, `kesh-i18n` (+ `CHANGELOG`) — quatre modules de code.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-01** — Créée après l'inventaire ; d2 découpée en d2a / d2b / d2c ; recommandations retenues par défaut
  (Guy : « pousse puis continue »).

[#384]: https://github.com/guycorbaz/kesh/issues/384
[#490]: https://github.com/guycorbaz/kesh/issues/490
