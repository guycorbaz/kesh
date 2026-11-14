# Story 35-4 : rapports dans les deux devises

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, D2, C-35-3. Suit 35-3a.
**Aucune migration attendue.***

## Story

En tant que comptable en multi-devises, je veux voir, pour chaque compte en devise, le solde et les mouvements en
devise à côté du CHF, dans le grand livre, la balance, le bilan et le compte de résultat, à l'écran et dans les
exports, afin de rapprocher mes comptes PayPal ou Wise de leurs relevés et de contrôler ma réévaluation.

## Critères d'acceptation

1. **AC-1 — Règle unique.** Le solde en devise d'un compte = Σ `amount_currency` au débit − Σ au crédit (signé comme
   le CHF par le type du compte, `opening.rs`), sur les lignes `currency = devise du compte` ; les lignes
   `revaluation` (montant 0) n'y changent rien. Les sommes CHF existantes **ne changent pas** (aucun rapport ne lit les
   colonnes en devise pour le CHF). Une fonction partagée (`kesh-report`) calcule les deux soldes ; règle
   d'ouverture identique à `opening_from` (:75).
2. **AC-2 — Grand livre** (`general_ledger.rs` `generate` :644) : pour un compte en devise, colonnes *Montant (devise)*,
   *Cours*, *Solde (devise)* en plus du CHF ; report d'ouverture en devise ; une ligne `revaluation` affichée « Réévaluation
   au cours X » sans montant en devise ; comptes CHF : inchangés.
3. **AC-3 — Balance** (`trial_balance.rs` :129) : devise et solde en devise pour les comptes en devise.
4. **AC-4 — Bilan** (`balance_sheet.rs` :166) et **compte de résultat** (`income_statement.rs` :30) : à côté du CHF,
   mention « USD 1'233.57 » pour un compte en devise (comme les livres de référence de Beat) ; totaux en CHF
   seulement.
5. **AC-5 — Exports** : PDF et CSV des quatre rapports portent les mêmes informations ; formatage suisse des montants
   (`format_swiss_amount`).
6. **AC-6 — Écran et MCP** : les vues (`GeneralLedgerView`, `TrialBalanceView`, `BalanceSheetView`,
   `IncomeStatementView`) et les outils `kesh_grand_livre` / rapports donnent les champs en devise ; mono-devise : rien
   de visible ne change (test).
7. **AC-7 — Tests** : jeu à deux exercices (compte USD, achats et ventes à des cours différents, conversion,
   réévaluation posée à la main en ligne `revaluation`) ; solde en devise et CHF attendus calculés à la main dans le
   test ; non-régression des rapports en mono-devise (mêmes sorties qu'avant, octet pour octet pour le CSV).
8. **AC-8 — Manuel** : lecture des rapports en devise ; i18n ; PDF.

## Tâches

- [ ] `kesh-report` : fonction de solde à deux devises ; les quatre générateurs ; PDF, CSV.
- [ ] `services/rapports.rs`, routes, MCP.
- [ ] Frontend : quatre vues, types ; Vitest ; E2E (grand livre d'un compte USD).
- [ ] Manuel, CHANGELOG.

## Notes de développement

- `crates/kesh-report/src/` : `general_ledger.rs` (sommes :338-340, :465-468), `trial_balance.rs` (:151-161),
  `balance_sheet.rs` (:289-290, :340), `income_statement.rs` (:63-64), `opening.rs`, `pdf.rs`, `csv.rs`.
- `crates/kesh-api/src/services/rapports.rs` (partagé avec MCP).

## Change Log

- 2026-11-14 — fiche rédigée.
