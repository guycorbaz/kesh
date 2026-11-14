# Story 35-3a : écritures en devise — modèle, validation, API et garde des chemins automatiques

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`. Epic 35, C-35-3, C-35-4, C-35-5. Suit 35-1
et 35-2. **Migration non breaking** (colonnes `NULL` ajoutées). **Découpée** à la rédaction (règle de splitting
préventif : plus de cinq modules) : 35-3a = le serveur (`kesh-core`, `kesh-db`, `kesh-api`, `kesh-reconciliation`,
`kesh-seed`) et les sélecteurs serveur ; 35-3b = soldes d'ouverture, formulaire et écrans.*

## Story

En tant que comptable en multi-devises, je veux saisir une écriture dont certaines lignes sont en devise — montant en
USD, cours officiel du mois proposé, montant en CHF calculé — et, pour une conversion, saisir moi-même les cours et
passer l'écart de change, afin que mes comptes en devise portent leur solde en devise et en CHF.

## Critères d'acceptation

1. **AC-1 — Migration** `20261114000003_journal_lines_foreign_currency.sql` : sur `journal_entry_lines`,
   `currency CHAR(3) NULL`, `amount_currency DECIMAL(19,4) NULL`, `exchange_rate DECIMAL(20,10) NULL`,
   `rate_source VARCHAR(12) NULL` ; contraintes **nommées** (C-35-3), chacune tolérante à la ligne CHF (quatre `NULL`) :
   - `chk_jel_fx_all_or_none` : les quatre `NULL`, ou les quatre non `NULL` ;
   - `chk_jel_fx_currency` : `currency IS NULL OR (currency REGEXP BINARY '^[A-Z]{3}$' AND currency <> 'CHF')`
     (symétrique de 35-1 : la devise principale s'écrit `NULL`) ;
   - `chk_jel_fx_rate_source` : `rate_source IN ('afc','manual','revaluation')` (en `BINARY`) ;
   - `chk_jel_fx_amount` : `rate_source = 'revaluation'` ⇒ `amount_currency = 0` ; sinon `amount_currency > 0` ;
   - `chk_jel_fx_rate` : `exchange_rate > 0` ;
   - `chk_jel_fx_chf` (filet structurel, C-35-4) : `currency IS NULL OR rate_source = 'revaluation' OR debit + credit =
     ROUND(amount_currency * exchange_rate, 2)` — le `ROUND` de MariaDB sur `DECIMAL` arrondit l'équidistant loin de
     zéro, comme `MidpointAwayFromZero` ; il couvre les chemins qui n'appellent pas `validate()` (contre-passation,
     restauration `.keshbackup`, 35-5).
   Chaque `chk_jel_fx_*` a son entrée dans la correspondance des violations de `CHECK` de `crates/kesh-api/src/errors.rs`
   (code métier, jamais une 500 muette) ; le refus 400 ordinaire reste celui de `kesh-core` (AC-3).
   Inscriptions habituelles (sha384, audit, squash, P6 inspecté, P7 sans objet : DDL seul). Colonnes `NULL` : la fenêtre
   d'import ne se ferme pas, une sauvegarde 0.17.6 se réimporte (testé, Tâches).
2. **AC-2 — Modèle Rust.** `ForeignAmount { currency, amount, rate, source }` (`kesh-core`) ;
   `NewJournalEntryLine.foreign: Option<ForeignAmount>` **et** `JournalEntryLineDraft.foreign: Option<ForeignAmount>`
   (`balance.rs:72`, le type que lit `validate()`), **sans** `Default` : chaque site de construction est revu
   (inventaire de l'échec de compilation, écrit au Change Log, site par site : `None` motivé ou extension) ;
   `JournalEntryLine` relu avec les quatre colonnes. Sites SQL de `journal_entries.rs`, liste close (production ; `mod
   tests` commence à `:3177`) :
   - `INSERT` : `:505` (`create_in_tx_inner`) et `:1667` (`update_in_tx`) — les deux seuls `INSERT INTO
     journal_entry_lines` de production de tout `crates/` ;
   - lectures : `LINE_COLUMNS` (`:103`) ; colonnes en dur de `list_all_lines_by_company` (`:2047`, source de l'export
     CSV) ; lecture `OriginLine` de la contre-passation (`:2896`), puis recopie `:2917` ;
   - re-sommes d'équilibre (CHF, inchangées) : `:520-537` et celle d'`update_in_tx`.
3. **AC-3 — Validation `kesh-core`** (`accounting::validate`, `balance.rs:150`, ou fonction voisine appelée par lui),
   sur `JournalEntryLineDraft.foreign`. Pour une ligne en devise, les contrôles FX **précèdent** la règle d'exclusivité
   débit/crédit (`balance.rs:170`), dans cet ordre :
   1. échelle et bornes : au plus 4 décimales sur le montant, 10 sur le cours ; montant ≤ 10¹³, cours ≤ 10⁶ ; sinon
      400 `FX_VALUE_OUT_OF_RANGE` ; le produit montant × cours se calcule par `checked_mul` (dépassement → même code) ;
   2. signe : `amount > 0` et `rate > 0` (hors `revaluation`, ci-dessous) ;
   3. CHF = `round_to_centimes(amount × rate)` (`money.rs:66`) ; s'il vaut 0 → `FX_LINE_ROUNDS_TO_ZERO` ;
   4. le montant CHF fourni (`debit` ou `credit`) doit **égaler** ce calcul, sur le côté non nul, sinon
      `FX_AMOUNT_MISMATCH` (attendu et fourni en `details`).
   - **`rate_source = 'revaluation'`** : exemptée de l'égalité CHF = round(montant × cours) et de
     `FX_LINE_ROUNDS_TO_ZERO` ; exige `amount = 0`, `rate > 0` et un CHF > 0. Refusée par l'API (AC-6), admise pour
     35-5 et pour la contre-passation.
   - l'équilibre reste Σ débit CHF = Σ crédit CHF, **strict** ; déséquilibre → `EntryUnbalanced` avec l'écart.
   - **Vecteurs d'arrondi partagés** : `crates/kesh-core/tests/fixtures/fx_rounding.json` (créé ici ; tableau de
     `{ amount, rate, chf }` en chaînes décimales, ou `{ amount, rate, error: "FX_VALUE_OUT_OF_RANGE" }`), avec au moins
     les cas `x.xx5` (équidistant, arrondi loin de zéro), JPY (cours par unité < 0.01), cours à 10
     décimales, montant à 15 chiffres significatifs, et hors bornes ; un test Rust le lit et l'applique à
     `kesh-core` ; **35-3b le lit dans Vitest** pour `big.js` (même fichier, aucune copie).
4. **AC-4 — Contrôle central du dépôt** (`create_in_tx_inner`, `update_in_tx` — les deux seuls chemins d'`INSERT`).
   **Ordre des refus** (une seule raison, la plus bloquante d'abord ; la devise d'un compte ne se lit qu'après le
   contrôle d'appartenance, garde KF-002) :
   1. `InactiveOrInvalidAccounts` → `AccountsNotPostable` (existants, `validate_lines_accounts_in_tx` :135) ;
   2. société en mono-devise et ligne en devise → **409** `MULTI_CURRENCY_DISABLED` ;
   3. ligne **sans** devise sur un compte en devise → `ACCOUNT_REQUIRES_CURRENCY` ; ligne en devise sur un compte d'une
      autre devise ou CHF → `FX_CURRENCY_MISMATCH` (première ligne fautive, son numéro en `details`) ;
   4. ligne `rate_source = 'afc'` : le cours doit égaler `rate_per_unit` de `_kesh_exchange_rates` (35-2) pour
      (devise, mois de `entry_date`), lu dans la transaction ; sinon 400 `FX_RATE_NOT_AFC` (cours attendu en
      `details`) ; cours absent → `RATE_UNAVAILABLE {currency, month}`. Contrôle à la création **et** au `PUT` (date
      changée comprise). **Exemptée** : la contre-passation (`reverse_in_tx_inner`), qui recopie le cours d'origine à
      une autre date.
   - **Concurrence** : la devise des comptes se lit **`FOR SHARE`** (dans la requête de validation des comptes ou une
     requête dédiée verrouillante) ; 35-1 prend `FOR UPDATE` sur le compte avant de compter ses lignes. Sans cela, un
     `PUT` de 35-1 validé entre la lecture et l'`INSERT` laisserait une ligne CHF sur un compte en devise. Test de
     concurrence, ou argument écrit au Change Log.
   - Chaque refus est une variante de `DbError` avec son `error_code()`, et a son bras dans la correspondance
     `DbError → AppError` de `errors.rs`.
5. **AC-5 — Chemins automatiques.** Factures, avoirs, règlements, factures fournisseur, lots de paiement,
   réconciliation (manuelle, éclatement, règles, propositions), — les soldes d'ouverture et leur complément exceptés
   (étendus par 35-3b ; d'ici là, ils refusent un compte en devise comme les autres) :
   - aucun ne construit de ligne en devise ; s'ils visent un compte en devise, le refus central de l'AC-4 s'applique
     et remonte en erreur claire (`ACCOUNT_REQUIRES_CURRENCY`, traduite) ;
   - **lots (`accept_batch`)** : chaque nouvelle variante a son bras dans **chaque** correspondance
     `DbError → FailedProposal` de `crates/kesh-api/src/routes/reconciliation.rs` — `project_error_to_failed_proposal`
     (`:743`, dont le repli `_ => "DATABASE_ERROR"` `:801-803` est exactement ce qu'il faut éviter : précédent
     `PeriodLocked`, commentaire `:773-780`), la correspondance des refus de comptes (`:3143-3260`, bras
     `InactiveOrInvalidAccounts` `:3169`) et `:675-690` (`AccountsNotPostable`) ; inventaire des `match` au Change
     Log. Test : un lot dont une proposition vise un compte en devise rend **200**, un `failed[]` avec
     `ACCOUNT_REQUIRES_CURRENCY`, les autres propositions acceptées (règle *Pattern batch*) ;
   - **configurations** — 35-1 refuse des deux côtés : poser une devise sur un compte désigné, lié ou porteur d'un
     rôle → 409 `ACCOUNT_CURRENCY_IN_USE` (35-1) ; lier ou désigner un compte en devise → 400
     `ACCOUNT_IN_FOREIGN_CURRENCY` (validations serveur de chaque configuration, ici). Liste **close** des colonnes qui
     désignent un compte, lue au schéma (`crates/kesh-db/test-schema/0001_schema_squash.sql`) :
     `bank_accounts.journal_account_id` (sans FK) ; les dix `company_invoice_settings.default_{bad_debt, bank_fees,
     discount, payable, receivable, revenue, rounding, vat_decompte, vat_payable, vat_recoverable}_account_id` ;
     `products.default_revenue_account_id` ; `invoice_lines.revenue_account_id` ; `credit_note_lines.revenue_account_id` ;
     `supplier_invoice_lines.expense_account_id` ; `supplier_invoices.settlement_account_id` ;
     `invoice_settlements.settlement_account_id` ; `reconciliation_rules.counterparty_account_id` ; plus
     `companies.fx_gain_account_id` / `fx_loss_account_id` (35-1, en CHF par construction) et les rôles de compte.
     Le **contrôle central de l'AC-4 reste le filet** : un état hérité ou une colonne oubliée ne produit jamais une
     ligne CHF sur un compte en devise, seulement un refus. Les sélecteurs du frontend masquent ces comptes (35-3b) ;
   - inventaire **clos** des sites, écrit au Change Log (méthode « inventorier les sites non résolus », `CLAUDE.md`).
6. **AC-6 — API des écritures.** `CreateJournalEntryLineRequest` prend `foreign?: { currency, amount, rate,
   rateSource: "afc"|"manual" }` (`revaluation` refusé ici : réservé à 35-5) ; `debit`/`credit` restent les montants
   CHF ; réponses (`JournalEntryLineResponse`, détail, liste) exposent `foreign` ; `MAX_LINES_PER_ENTRY` inchangé ;
   erreurs traduites (quatre locales). `MULTI_CURRENCY_DISABLED` est un **409** partout.
7. **AC-7 — Modification et contre-passation.**
   - le `PUT` (remplacement complet des lignes : `DELETE` `:1646`, `INSERT` `:1667`) prend `foreign` comme le `POST`,
     avec les mêmes contrôles (AC-3, AC-4, dont `FX_RATE_NOT_AFC`) ; l'audit avant/après (`entry_snapshot_json`
     `:999`) porte les quatre champs ;
   - une écriture qui porte une ligne `rate_source = 'revaluation'` **n'est pas modifiable** : `PUT` → 409
     `ENTRY_IS_FX_REVALUATION` ;
   - la contre-passation (repository `reverse_in_tx_inner` `:2796`, lecture `:2896`, recopie `:2917` ; handlers
     `routes/journal_entries.rs` `:448` et `:479`), y compris d'une réévaluation, recopie `foreign` **et**
     `rate_source` en inversant les côtés ; elle est exemptée du contrôle `FX_RATE_NOT_AFC`.
8. **AC-8 — Sauvegarde, exports, MCP.**
   - CSV des lignes (`csv_tables.rs` :356) avec les quatre colonnes ; `amount_currency` et `exchange_rate` s'exportent
     à leur échelle de colonne (4 et 10), **jamais** par `fmt_decimal` / `fmt_opt_decimal` (`csv_tables.rs:108`,
     `format!("{d:.2}")`, qui les ramènerait à deux décimales) ; test de sérialisation à 10 décimales ;
   - `entry_snapshot_json` (audit) les inclut ;
   - MCP : `kesh_ecritures` / `kesh_ecriture` les donnent — `crates/kesh-api/src/services/ecritures.rs`
     (`list_by_company_paginated` `:201`, `find_by_id` `:243`) et `crates/kesh-api/src/mcp/outils/ecritures.rs`
     (struct `:126`, construction `:187`) ;
   - aller-retour `.keshbackup` (restauration par l'`INSERT` générique, `backup.rs:834`, sous `chk_jel_fx_*`).
9. **AC-9 — Limite écrite : le lettrage.** Le lettrage manuel sur un compte en devise ne raisonne qu'en CHF : un
   débiteur USD soldé en USD avec un écart de change ne se lettre pas. Limite assumée, écrite au CHANGELOG, renvoyée au
   registre (C-35-8) pour une issue à l'ouverture de la PR.
## Tâches

- [ ] Migration + inscriptions, contraintes nommées et leur correspondance dans `errors.rs` (AC-1) ; test : une
      sauvegarde 0.17.6 se réimporte ; test : `chk_jel_fx_chf` refuse une ligne incohérente insérée en SQL brut.
- [ ] `kesh-core` (AC-2, AC-3) : `ForeignAmount`, `JournalEntryLineDraft.foreign`, ordre des contrôles, bornes et
      `checked_mul` ; fixture `crates/kesh-core/tests/fixtures/fx_rounding.json` et son test Rust ; tests de propriété
      (arrondi) et de chaque refus (`FX_VALUE_OUT_OF_RANGE`, `FX_LINE_ROUNDS_TO_ZERO`, `FX_AMOUNT_MISMATCH`,
      exemption `revaluation`).
- [ ] `kesh-db` : entités, colonnes, sites SQL de l'AC-2, contrôle central dans l'ordre de l'AC-4 (`FOR SHARE`),
      `FX_RATE_NOT_AFC`, contre-passation (AC-2, AC-4, AC-7) ; tests de chacun des codes de l'AC-4, de la concurrence
      (ou argument écrit), de la recopie à la contre-passation (réévaluation comprise), du 409
      `ENTRY_IS_FX_REVALUATION`.
- [ ] Revue site par site des constructeurs de `NewJournalEntryLine` et de `JournalEntryLineDraft`, des
      correspondances `FailedProposal` et des validations de configuration (AC-5) — inventaire au Change Log.
- [ ] API (AC-6), MCP, exports, audit (AC-8) ; test de l'export CSV à 10 décimales.
- [ ] Tests d'API : écriture USD au cours AFC et au cours manuel, `FX_RATE_NOT_AFC`, conversion EUR → USD avec ligne
      d'écart, refus d'un règlement de facture sur un compte USD, lot de réconciliation en `FailedProposal` (200),
      `MULTI_CURRENCY_DISABLED` en 409.
- [ ] CHANGELOG, avec la limite du lettrage (AC-9) (le manuel et l'E2E vont à 35-3b).

## Notes de développement

- Lignes : `crates/kesh-db/src/entities/journal_entry.rs` (`JournalEntryLine` :151, `NewJournalEntryLine` :203) ;
  `repositories/journal_entries.rs` (`LINE_COLUMNS` :103, `validate_lines_accounts_in_tx` :135, `create_in_tx_inner`
  :281, `INSERT` :505 et :1667, re-sommes :520-537 et dans `update_in_tx` :1472, `entry_snapshot_json` :999,
  `list_all_lines_by_company` :2047, contre-passation :2558-3122 dont lecture `OriginLine` :2896 et recopie :2917).
  ⚠️ `:5290` est un `INSERT` **de test** (`mod tests` dès `:3177`), non un site de production.
- Validation : `crates/kesh-core/src/accounting/balance.rs` (`JournalEntryLineDraft` :72, `validate` :150, exclusivité
  :170 — appelée seulement par `routes/journal_entries.rs:566`, `routes/opening_balances.rs:448`,
  `opening_complement.rs:699`, d'où le `CHECK` `chk_jel_fx_chf`) ; arrondi `crates/kesh-core/src/types/money.rs`
  (`round_to_centimes` :66). Constructeurs de `JournalEntryLineDraft` : `routes/journal_entries.rs:548`,
  `routes/opening_balances.rs:369`, `opening_complement.rs:669` et :682.
- API : `crates/kesh-api/src/routes/journal_entries.rs` (DTO :68-152, construction du brouillon :515-595).
- Frontend : `frontend/src/lib/features/journal-entries/` (`JournalEntryForm.svelte`, `balance.ts`).
- Constructeurs de `NewJournalEntryLine` en production, recomptés le 2026-11-14 (littéraux hors `mod tests`) :
  **30 dans 11 fichiers** — `repositories/invoice_settlements.rs` 7, `supplier_invoices.rs` 5, `credit_notes.rs` 4,
  `invoices.rs` 4, `opening_complement.rs` 1, `journal_entries.rs` 1 (contre-passation :2920) ;
  `kesh-reconciliation/src/manual.rs` 2, `split.rs` 2 ; `kesh-seed/src/pieges.rs` 2 ; `routes/journal_entries.rs` 1 ;
  `routes/opening_balances.rs` 1. `repositories/accounts.rs` n'en construit qu'en tests. La liste fait foi **après**
  l'échec de compilation, pas avant.
- Correspondances `DbError → FailedProposal` : `routes/reconciliation.rs` :675-690, :743-803, :3143-3260.

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Opus) : 0 CRITICAL, 1 HIGH, 9 MEDIUM, 7 LOW ; remédiés.
