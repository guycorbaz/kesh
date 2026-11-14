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
   `rate_source VARCHAR(12) NULL` ; `CHECK` (C-35-3) :
   - tout-ou-rien : les quatre `NULL`, ou les quatre non `NULL` ;
   - `rate_source IN ('afc','manual','revaluation')` (en `BINARY`) ;
   - `rate_source = 'revaluation'` ⇒ `amount_currency = 0` ; sinon `amount_currency > 0` ;
   - `exchange_rate > 0`.
   Inscriptions habituelles (sha384, audit, squash, P6, P7 sans objet). Une sauvegarde antérieure se réimporte.
2. **AC-2 — Modèle Rust.** `ForeignAmount { currency, amount, rate, source }` (`kesh-core`) ;
   `NewJournalEntryLine.foreign: Option<ForeignAmount>` **sans** `Default` : chaque site de construction est revu
   (inventaire de l'échec de compilation, écrit au Change Log, site par site : `None` motivé ou extension) ;
   `JournalEntryLine` relu avec les quatre colonnes (`LINE_COLUMNS`, toutes les requêtes brutes, dont l'`INSERT` de
   `journal_entries.rs` :5290).
3. **AC-3 — Validation `kesh-core`** (`accounting::validate`, ou fonction voisine appelée par lui) :
   - ligne en devise : `amount > 0` (sauf `revaluation`), `rate > 0`, CHF = `round_to_centimes(amount × rate)` — le
     montant CHF fourni doit **égaler** ce calcul, sinon `FX_AMOUNT_MISMATCH` ; CHF arrondi à 0 →
     `FX_LINE_ROUNDS_TO_ZERO` ; au plus 4 décimales sur le montant en devise, 10 sur le cours ;
   - l'équilibre reste Σ débit CHF = Σ crédit CHF, **strict** ; déséquilibre → `EntryUnbalanced` avec l'écart.
4. **AC-4 — Contrôle central du dépôt** (`create_in_tx_inner`, `update_in_tx`, et tout chemin qui insère des lignes) :
   devise de la ligne = devise du compte, sinon `FX_CURRENCY_MISMATCH` ; ligne **sans** devise sur un compte en devise
   → `ACCOUNT_REQUIRES_CURRENCY` ; ligne en devise sur un compte CHF → `FX_CURRENCY_MISMATCH` ; société en mono-devise
   et ligne en devise → `MULTI_CURRENCY_DISABLED`. Lecture des devises dans la même transaction que la validation des
   comptes (`validate_lines_accounts_in_tx` :135).
5. **AC-5 — Chemins automatiques.** Factures, avoirs, règlements, factures fournisseur, lots de paiement,
   réconciliation (manuelle, éclatement, règles, propositions), — les soldes d'ouverture et leur complément exceptés (étendus par 35-3b ; d'ici là, ils
   refusent un compte en devise comme les autres) :
   - aucun ne construit de ligne en devise ; s'ils visent un compte en devise, le refus central de l'AC-4 s'applique
     et remonte en erreur claire (`ACCOUNT_REQUIRES_CURRENCY`, traduite) ; dans un lot (`accept_batch`), en
     `FailedProposal` par proposition, HTTP 200 (règle *Pattern batch*) ;
   - leurs sélecteurs de comptes (frontend et validations serveur de configuration : compte de produit d'une ligne de
     facture, compte de charge fournisseur, compte lié d'un compte bancaire, comptes par défaut de facturation…)
     **refusent** les comptes en devise (`ACCOUNT_IN_FOREIGN_CURRENCY`) ; les sélecteurs du frontend les masquent (35-3b) ; une configuration existante qui en désigne un (devise posée avant le
     lien est impossible : verrou 35-1 ; l'inverse — lier un compte déjà en devise — est refusé) ;
   - inventaire **clos** des sites, écrit au Change Log (méthode « inventorier les sites non résolus », `CLAUDE.md`).
6. **AC-6 — API des écritures.** `CreateJournalEntryLineRequest` prend `foreign?: { currency, amount, rate,
   rateSource: "afc"|"manual" }` (`revaluation` refusé ici : réservé à 35-5) ; `debit`/`credit` restent les montants
   CHF ; réponses (`JournalEntryLineResponse`, détail, liste) exposent `foreign` ; `MAX_LINES_PER_ENTRY` inchangé ;
   erreurs traduites (quatre locales).
7. **AC-7 — Extourne et modification** : l'extourne (`reverse*` :448/:479) recopie `foreign` en inversant les côtés ; la dévalidation/modification conserve
   les champs.
8. **AC-8 — Sauvegarde, exports, MCP.** CSV des lignes (`csv_tables.rs` :356) avec les quatre colonnes ;
    `entry_snapshot_json` (audit) les inclut ; `kesh_ecritures` / `kesh_ecriture` les donnent ; aller-retour
    `.keshbackup`.
## Tâches

- [ ] Migration + inscriptions (AC-1).
- [ ] `kesh-core` (AC-2, AC-3) avec tests de propriété (arrondi) et de refus.
- [ ] `kesh-db` : entités, colonnes, contrôle central, tous les `INSERT` de lignes, extourne (AC-2, AC-4, AC-7).
- [ ] Revue site par site des constructeurs de `NewJournalEntryLine` et des sélecteurs (AC-5) — inventaire au Change Log.
- [ ] API (AC-6), MCP, exports, audit (AC-8).
- [ ] Tests d'API : écriture USD au cours AFC et au cours manuel, conversion EUR → USD avec ligne d'écart, refus d'un
      règlement de facture sur un compte USD, lot de réconciliation en `FailedProposal`.
- [ ] CHANGELOG (le manuel et l'E2E vont à 35-3b).

## Notes de développement

- Lignes : `crates/kesh-db/src/entities/journal_entry.rs` (`JournalEntryLine` :151, `NewJournalEntryLine` :203) ;
  `repositories/journal_entries.rs` (`LINE_COLUMNS` :103, `create_in_tx_inner` :281, boucle d'`INSERT` :502-518,
  re-somme SQL :520-537, `update_in_tx` :1472, `entry_snapshot_json` :999, extourne :2558-3122, `INSERT` brut :5290).
- Validation : `crates/kesh-core/src/accounting/balance.rs` (`validate` :150) ; arrondi
  `crates/kesh-core/src/types/money.rs` (`round_to_centimes` :66).
- API : `crates/kesh-api/src/routes/journal_entries.rs` (DTO :68-152, construction du brouillon :515-595).
- Frontend : `frontend/src/lib/features/journal-entries/` (`JournalEntryForm.svelte`, `balance.ts`).
- Constructeurs connus : `repositories/{invoices,credit_notes,invoice_settlements,supplier_invoices,accounts,
  opening_complement}.rs`, `kesh-reconciliation/src/{manual,split}.rs`, `routes/opening_balances.rs`,
  `kesh-seed/src/pieges.rs` — la liste fait foi **après** l'échec de compilation, pas avant.

## Change Log

- 2026-11-14 — fiche rédigée.
