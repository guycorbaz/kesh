# Story 25.6-a : Le tableau de bord dit vrai — dernières écritures, factures ouvertes, solde comptable

Status: ready-for-dev

**Issues : [#388], [#389]**, que cette story **ferme** : la PR porte `closes #388, closes #389` dans le **titre ET le
corps**. Branche `story/25-6-a-tableau-de-bord`, partie de `main` (`d7c74f02`).

⛔ **Issue du DÉCOUPAGE de la 25-6** (2026-10-03, recommandation non contestée de Guy) : la 25-6 réunissait trois issues
isolées. #388 et #389 portent sur la page d'accueil ; **#387** (figer le PDF d'une facture) touche stockage, migration
et rendu — elle part en **25-6-b**. Ensemble, elles dépassaient le seuil de cinq modules de la § *Règle de splitting
préventif*.

## Story

En tant que **personne qui ouvre Kesh**,
je veux que **la page d'accueil montre ce que contiennent réellement mes livres** — mes dernières écritures, mes factures
ouvertes et leur montant, et un solde bancaire nommé pour ce qu'il est,
afin de **ne pas lire « Aucune écriture » sur une comptabilité pleine**, ni prendre un solde comptable pour l'argent en
banque.

## Les faits, vérifiés dans le code

- **La tuile « Dernières écritures » n'appelle rien** (`frontend/src/routes/(app)/+page.svelte:82-98`) : elle affiche
  inconditionnellement `homepage-entries-empty` (« Aucune écriture. ») ou sa variante guidée. Le endpoint existe :
  `GET /api/v1/journal-entries` (`crates/kesh-api/src/routes/journal_entries.rs:240-405`), **tout rôle authentifié**
  (`lib.rs:710-715`, `authenticated_routes`), tri par défaut `entry_date` **décroissant** (`kesh-core/src/listing/mod.rs`),
  `limit` borné à 500. Client frontend : `features/journal-entries/journal-entries.api.ts:27`.
- **La tuile « Factures ouvertes » n'affiche aucun montant** (`+page.svelte:100-128`) : seulement « N facture(s) à
  rappeler », **pour Admin/Comptable**, depuis `/dunning/reminders` (Comptable+) — un rôle Consultation voit « Aucune
  facture ouverte » en permanence. Le résumé existe : `GET /api/v1/invoices/due-dates` (`lib.rs:751`, **tout rôle**) rend
  `DueDatesSummary { unpaidCount, unpaidTotal, overdueCount, overdueTotal }` (`kesh-db/src/repositories/invoices.rs:870-905`)
  — des **restes dus** depuis la 25-4-b1 (#416), factures `validated` non soldées. Client : `invoices.api.ts:118`.
- **Le « solde bancaire » est le solde comptable** : `bank_accounts::list_by_company_with_balances`
  (`kesh-db/src/repositories/bank_accounts.rs:620-690`) rend `SUM(debit) − SUM(credit)` du compte de grand livre lié,
  toutes dates. L'accueil l'affiche sous « Comptes bancaires » et le totalise en « **Total liquidités** »
  (`+page.svelte:130-185`) — rien ne dit qu'il s'agit du solde **comptable**.
- **Le solde du relevé est déjà stocké** : `bank_imports.closing_balance DECIMAL(18,2) NULL` et `period_to DATE` (une
  ligne par import, `idx_bank_imports_company_account_imported`) — jamais lu par l'accueil.
- ⚠️ **Montants en `Number`** : `bank-accounts.api.ts:55-69` convertit `currentBalance` (chaîne décimale du serveur) en
  `Number`, et l'accueil somme ces nombres (`+page.svelte:62-65`). Contraire à la règle du dépôt (`big.js` pour tout
  montant). Second consommateur : `routes/(app)/bank-accounts/+page.svelte:397`.
- **E2E existants de l'accueil** : `homepage-reminders.spec.ts`, `homepage-settings.spec.ts`.
- **Manuel** : `docs/manual/fr/user-manual.tex:215-235` (§ *Tableau de bord*) — « Dernières écritures — accès rapide au
  journal », « Comptes bancaires — soldes calculés » : à réécrire.

## Acceptance Criteria

**AC 1 — Dernières écritures, branchées.** La tuile appelle `GET /api/v1/journal-entries?limit=5` (tri par défaut :
date décroissante) et liste au plus **cinq** écritures : date, numéro, libellé, montant (somme des débits de
l'écriture, en `big.js`). Chaque ligne mène à l'écriture (`/journal-entries/{id}`). Sans écriture, le texte vide actuel
(et sa variante guidée) reste. Échec de l'appel : la tuile le dit (« indisponible ») au lieu de prétendre qu'il n'y a
rien. Le bouton « Saisir une écriture » n'apparaît que pour Admin/Comptable (Consultation ne saisit pas).

**AC 2 — Factures ouvertes, chiffrées, pour tous les rôles.** La tuile appelle `GET /api/v1/invoices/due-dates` (sans
filtre) et affiche **N factures ouvertes — reste dû total** et, s'il y en a, **dont M échues — montant**. Les montants
passent par `big.js` et le formatage suisse du dépôt. Le compteur « à rappeler » (Admin/Comptable) reste. Le bouton
« Créer une facture » n'apparaît que pour Admin/Comptable. Sans facture ouverte, le texte vide actuel reste ; échec de
l'appel : « indisponible ».

**AC 3 — Le solde est nommé « solde comptable ».** Dans la tuile « Comptes bancaires », chaque solde est libellé
**« Solde comptable »** et le total **« Total (solde comptable) »** au lieu de « Total liquidités ». Le libellé d'aide dit
d'où vient le chiffre : le grand livre, non la banque.

**AC 4 — Le solde du dernier relevé, et l'écart.** `GET /api/v1/bank-accounts` gagne, par compte, le **dernier relevé
importé** qui porte un solde de clôture : `statementClosingBalance` (`Decimal` | `null`), `statementDate` (`period_to`) et
`ledgerBalanceAtStatement` (le solde comptable du compte lié **au `period_to` inclus**). L'accueil affiche, quand un relevé
existe : « Relevé du {date} : {montant} » et, si `ledgerBalanceAtStatement ≠ statementClosingBalance`, **« Écart :
{différence} »** (solde comptable à cette date − relevé), mis en évidence. Sans relevé, rien de plus. Les champs neufs
sont `null` quand le compte n'a pas de compte de grand livre lié (le solde comptable ne se calcule pas) ou aucun relevé
avec solde. « Dernier » = `period_to` le plus récent, départagé par `imported_at` puis `id`.

**AC 5 — Les montants ne passent plus par `Number`.** `currentBalance` et les trois champs neufs restent des **chaînes
décimales** côté frontend (`BankAccountSummary`), le total et l'écart se calculent en `big.js`, l'affichage par le
formateur existant (adapté à une chaîne si nécessaire). Les deux consommateurs (accueil, page des comptes bancaires)
sont mis à jour.

**AC 6 — L'API et la doc.** Les champs neufs dans la réponse de `GET /api/v1/bank-accounts` (et des routes qui rendent
le même DTO, s'il y en a — à inventorier), documentés au DTO. Le calcul du solde comptable à une date partage la
requête existante (pas de seconde copie du `SUM(debit) − SUM(credit)`).

**AC 7 — Tests.** Chacun aurait échoué avant le patch :
- `kesh-db` : relevé le plus récent choisi (deux imports, `period_to` différents ; égalité départagée) ; relevé sans
  `closing_balance` ignoré ; solde comptable **à la date du relevé** (une écriture postérieure n'y entre pas, une écriture
  du jour même y entre) ; compte sans grand livre lié → `null` ; isolation par société ;
- `kesh-api` : forme de la réponse (`statementClosingBalance`, `statementDate`, `ledgerBalanceAtStatement` en chaînes) ;
- Vitest de l'accueil (`+page.svelte` ou composants extraits) : dernières écritures listées / vide / échec ; factures
  ouvertes chiffrées pour un rôle **Consultation** ; « dont M échues » absent à zéro ; « Solde comptable » et le total
  renommé ; écart affiché seulement s'il est non nul ; boutons d'action absents pour Consultation ; total calculé en
  `big.js` (cas `0.1 + 0.2`) ;
- **E2E** : une écriture postée par l'API apparaît dans la tuile ; une facture validée apparaît dans « Factures
  ouvertes » avec son montant (sélecteurs `data-testid`, jamais un libellé traduit — garde `e2e-selecteurs-traduits`).

**AC 8 — Le manuel et le CHANGELOG.** `user-manual.tex` § *Tableau de bord* réécrit : les trois tuiles telles qu'elles
sont, le solde **comptable** expliqué (le grand livre, pas la banque ; il ne reflète un encaissement qu'une fois
celui-ci comptabilisé), le dernier relevé et l'écart, et ce que voit un rôle Consultation. PDF régénéré, contrôlé
**aplati**. CHANGELOG `[0.12.1]` : `Fixed` (#388, #389), `Changed` (« Total liquidités » → « Total (solde comptable) »).
README et site relus (`grep -rni "tableau de bord\|liquidit" README.md website/`).

## Tasks / Subtasks

- [ ] **T1 — le dernier relevé et le solde à sa date** (AC 4, 6), `kesh-db` `bank_accounts.rs`.
- [ ] **T2 — le DTO** (AC 4, 6), `kesh-api` `routes/bank_accounts.rs`.
- [ ] **T3 — les montants en chaînes** (AC 5), `bank-accounts.api.ts`, page des comptes bancaires.
- [ ] **T4 — l'accueil** (AC 1, 2, 3, 4) et l'i18n (4 locales, `sitesTotal`, relevé des libellés en dur).
- [ ] **T5 — tests** (AC 7).
- [ ] **T6 — manuel, CHANGELOG** (AC 8).
- [ ] **T7 — gates** : backend complet, frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- **Ne pas** présenter le solde du relevé comme « le solde bancaire actuel » : c'est le solde **au `period_to`** du
  dernier relevé importé, daté à l'écran.
- **Ne pas** créer d'endpoint « tableau de bord » agrégé : les trois endpoints existent, sont ouverts à tout rôle, et un
  agrégat recopierait leurs règles (DRY).
- **Ne pas** recopier le calcul du reste dû : `due-dates` le porte déjà (25-4-b1).
- **Ne pas** afficher « Aucune … » quand l'appel a échoué — c'est le défaut même de #388.

### Limites assumées

- Le solde comptable ne reflète un encaissement qu'une fois celui-ci **comptabilisé** (rapprochement ou règlement) : c'est
  précisément ce que l'écart avec le relevé rend visible.
- Le relevé ne vaut que pour sa date : entre deux imports, l'écart n'est pas recalculé contre la banque réelle.

### Modules

`kesh-db` (dernier relevé, solde à une date), `kesh-api` (DTO), `frontend` (accueil, API des comptes bancaires, page des
comptes), `kesh-i18n` (+ `docs`, `CHANGELOG`) — **quatre modules de code**, sous le seuil.

### References

- Issues [#388], [#389] ; audit du 2026-08-26 (DAF § 1.4).
- `frontend/src/routes/(app)/+page.svelte` ; `crates/kesh-db/src/repositories/bank_accounts.rs:620-690` ;
  `crates/kesh-db/src/repositories/invoices.rs:870-905` (`DueDatesSummary`).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-03** — Créée (Guy : « ok, merge et continue » ; découpage 25-6-a / 25-6-b non contesté).

[#388]: https://github.com/guycorbaz/kesh/issues/388
[#389]: https://github.com/guycorbaz/kesh/issues/389
