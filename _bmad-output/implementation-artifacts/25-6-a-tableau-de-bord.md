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
  `DueDatesSummary { unpaidCount, unpaidTotal, overdueCount, overdueTotal }` (`kesh-db/src/repositories/invoices.rs`, `DueDatesSummary` et `due_dates_summary`)
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
`entry_date DESC, entry_number DESC`) et liste au plus **cinq** écritures : date, numéro, libellé, montant (somme des
débits de l'écriture, calculée côté client en `big.js` depuis les lignes que la liste rend déjà — la liste lit les
lignes par écriture, cinq requêtes de plus : acceptable à cette taille, aucun champ d'API ajouté). Une contre-passation
s'affiche comme toute écriture, sous son propre numéro. Chaque ligne mène à l'écriture (`/journal-entries/{id}`). Sans écriture, le texte vide actuel
(et sa variante guidée) reste. Échec de l'appel : la tuile le dit (« indisponible ») au lieu de prétendre qu'il n'y a
rien — et l'échec **prime sur la variante guidée**. Le bouton « Saisir une écriture » n'apparaît que pour Admin/Comptable (Consultation ne saisit pas).

**AC 2 — Factures ouvertes, chiffrées, pour tous les rôles.** La tuile appelle `GET /api/v1/invoices/due-dates?limit=1`
(seul le `summary` sert ; la liste paginée qui l'accompagne est réduite à une ligne) et affiche **N factures ouvertes — reste dû total** et, s'il y en a, **dont M échues — montant**. Les montants
passent par `big.js` et le formatage suisse du dépôt. Le compteur « à rappeler » (Admin/Comptable) reste. Le bouton
« Créer une facture » n'apparaît que pour Admin/Comptable. Sans facture ouverte, le texte vide actuel reste ; échec de
l'appel : « indisponible ».

**AC 3 — Le solde est nommé « solde comptable », et compté une fois.** Dans la tuile « Comptes bancaires », chaque solde est libellé
**« Solde comptable »** et le total **« Total (solde comptable) »** au lieu de « Total liquidités ». Le libellé d'aide dit
d'où vient le chiffre : le grand livre, non la banque. **Le total ne compte chaque compte de grand livre qu'une fois** :
tous les comptes liés au même compte de grand
livre ont **par construction** le même solde (la somme porte sur le même `account_id`) : on en garde un, sans autre
règle ; rien n'empêche deux comptes bancaires d'être liés au même compte (`idx_bank_accounts_journal_account` n'est pas unique,
`validate_journal_account_id` ne contrôle pas l'usage), et le total les compterait deux fois ; une note le signale
quand des comptes partagent un même compte lié. La clé `homepage-bank-total-liquidity` est **remplacée** par
`homepage-bank-total-ledger` (4 locales) et le `data-testid` `homepage-bank-total-liquidity` par `homepage-bank-total`
(cité par aucun test).

**AC 4 — Le solde du dernier relevé, et l'écart.** `GET /api/v1/bank-accounts` gagne, par compte, le **dernier relevé
importé** qui porte un solde de clôture : `statementClosingBalance` (`Decimal` | `null`), `statementDate` (`period_to`) et
`ledgerBalanceAtStatement` (le solde comptable du compte lié **au `period_to` inclus**). L'accueil affiche, quand un relevé
existe : « Relevé du {date} : {montant} » et, si `ledgerBalanceAtStatement ≠ statementClosingBalance`, **« Écart :
{différence} »** (solde comptable à cette date − relevé), mis en évidence. Sans relevé, rien de plus. Les champs neufs
sont `null` quand le compte n'a pas de compte de grand livre lié (le solde comptable ne se calcule pas) ou aucun relevé
avec solde ; `ledgerBalanceAtStatement` (donc l'écart) est aussi `null` quand **plusieurs comptes bancaires partagent le
compte de grand livre** — son solde ne s'attribue pas à l'un d'eux. « Dernier » = `period_to` le plus récent **parmi les
relevés qui portent un solde** (un relevé sans `closing_balance` plus récent ne masque pas un plus ancien), départagé par
`imported_at` puis `id`. **Signe** : `débit − crédit` du compte lié, quel que soit son type — c'est la convention du
relevé (un découvert est négatif au CAMT, `DBIT`, comme le solde d'un compte de passif lié à une ligne de crédit tirée).
**Date de valeur** (arbitrage de Guy, 2026-10-03, voie (a)) : le solde du relevé CAMT (`CLBD`) est arrêté à la date
de **comptabilisation bancaire** (`camt053/mod.rs:541`), alors que les écritures de rapprochement sont datées à la date
de **valeur** (`reconciliation.rs:1958`, `:2295`, `kesh-reconciliation/src/manual.rs:35`). `ledgerBalanceAtStatement`
est donc **corrigé** par les transactions bancaires **rapprochées** du compte (`bank_transactions.status = 'reconciled'`,
`matched_entry_id` non nul) dont les deux dates tombent de part et d'autre de `period_to` : **+ Σ `amount`** des
transactions dont `booking_date ≤ period_to` et dont l'écriture (`journal_entries.entry_date` de `matched_entry_id`) est
**postérieure** ; **− Σ `amount`** de celles dont `booking_date > period_to` et dont l'écriture est **antérieure ou
égale**. La date de l'écriture est **lue**, jamais déduite de `value_date` (un rapprochement manuel peut la choisir).
Le montant de la transaction est signé comme le relevé (`amount` du CAMT) : la correction s'ajoute au solde `débit −
crédit` sans changement de signe. Une transaction **non rapprochée** n'entre pas dans la correction : son absence du
grand livre est précisément l'écart que la tuile doit montrer.

**Échelle** : l'écart se calcule **côté client**, en `big.js`, au centime — les deux soldes arrondis à 2 décimales
(`.round(2, Big.roundHalfUp)`, l'arrondi du dépôt — `vat-purchase.ts:12` ; **pas** `round_dp`, qui est l'arrondi
bancaire) avant la soustraction ; le relevé est en `DECIMAL(18,2)`, le grand livre à 4 décimales. **Partage** : le
nombre de comptes bancaires liés à un même compte de grand livre se compte **en SQL, sur tous les comptes de la
société, archivés compris** (l'historique d'un compte archivé alimente le même compte de grand livre) — la réponse ne
dépend donc pas de `includeArchived`. **Imports CSV** : ils ne portent jamais de solde de clôture
(`kesh-import/src/csv/parser.rs:342`) — un compte alimenté par CSV seul n'a pas de relevé, et un import CSV plus récent
ne masque pas un relevé CAMT plus ancien.

**AC 5 — Les montants ne passent plus par `Number`.** `currentBalance` et les trois champs neufs restent des **chaînes
décimales** côté frontend (`BankAccountSummary`), le total et l'écart se calculent en `big.js`. `formatChfBalance`
(`features/bank-accounts/format.ts:21`, aujourd'hui `(balance: number)`) prend une **chaîne ou un `Big`** et formate
`new Big(v).toFixed(2)` — TypeScript refuse une chaîne nue dans `Intl.NumberFormat.format`. Les deux lecteurs de
`currentBalance` (accueil, page des comptes bancaires) sont mis à jour. Forme imposée :
`format(new Big(v).round(2, Big.roundHalfUp).toNumber())` — un `string` (même issu de `toFixed`) ne compile pas, et le
`Number` d'un montant **déjà arrondi au centime** est exact jusqu'à ~9·10¹³ CHF. Côté `Raw`, les champs neufs sont **optionnels**
et normalisés à `null` aux quatre sites de `parseBankAccount` (`bank-accounts.api.ts:101,125,149,164` — les routes de
mutation rendent un `BankAccount` nu) ; les fixtures de `bank-accounts.api.test.ts` et de
`BankAccountJournalLinkForm.test.ts` complétées.

**AC 6 — L'API et la doc.** Les champs neufs dans la réponse de `GET /api/v1/bank-accounts` — seule route qui rend
`BankAccountWithBalance` (`routes/bank_accounts.rs:345-356`, inventaire fait) —, documentés au DTO. **Contrat** : une
seule requête par appel, **par compte bancaire** (et non plus par compte de grand livre) : `LEFT JOIN` du dernier relevé
avec solde du compte bancaire, et deux sommes sur le compte lié — `SUM(debit) − SUM(credit)` toutes dates (le
`currentBalance` d'aujourd'hui, **inchangé**) et la même somme bornée par `CASE WHEN je.entry_date <= period_to` —, pas
de seconde copie du calcul ; le dernier relevé par compte se réduit à une ligne avant les sommes (`ROW_NUMBER()`,
MariaDB 10.11). ⚠️ **Le calcul actuel n'a aucun test** : un test de **non-régression** est écrit et vert **contre le code
actuel, dans un commit séparé, avant la réécriture** — compte lié avec écritures, compte lié **sans** écriture
(`currentBalance` sérialisé `"0"`, pas `"0.0000"` : asserté sur le **JSON** de la route, une assertion `Decimal` ne
verrait rien), compte non lié (`null`), ordre de la liste. Il est **exempté** de la règle « aurait échoué avant ».

**AC 7 — Tests.** Chacun aurait échoué avant le patch :
- `kesh-db` : relevé le plus récent choisi — deux imports dont le **plus récent importé est le plus ancien en date**
  (l'ordre d'import ne décide pas), et une égalité de `period_to` départagée ; un relevé sans `closing_balance` **plus
  récent** qu'un relevé avec solde ne le masque pas ; solde comptable **à la date du relevé** (une écriture postérieure
  n'y entre pas, une écriture du jour même y entre ; un relevé daté **après** la dernière écriture rend le solde
  courant) ; compte sans grand livre lié → `null` ; **deux comptes bancaires sur le même compte de grand livre** →
  `ledgerBalanceAtStatement` `null` ; un compte lié de **passif** (ligne de crédit tirée) → solde et relevé de même
  signe, écart nul ; isolation par société ; non-régression de `currentBalance` / `lastTransactionDate` ;
  **date de valeur** — un crédit de 200 comptabilisé par la banque le jour du relevé, rapproché par une écriture datée
  du lendemain : écart **nul** ; le cas inverse (comptabilisé le lendemain, écriture du jour) : écart nul ; une
  transaction **non rapprochée** du jour du relevé : écart égal à son montant ; un rapprochement manuel dont la date
  d'écriture diffère de `value_date` : la date **lue** fait foi ;
- `kesh-api` : forme de la réponse (`statementClosingBalance`, `statementDate`, `ledgerBalanceAtStatement` en chaînes) ;
- **Vitest, par composant** — les trois tuiles sont **extraites** en composants (`features/homepage/`
  `RecentEntriesCard.svelte`, `OpenInvoicesCard.svelte`, `BankAccountsCard.svelte`, données en props, appels dans la
  page) : aucune page n'est testée aujourd'hui, et monter `+page.svelte` exigerait cinq mocks. ⚠️ Le dossier est
  `homepage/`, pas `dashboard/` : `lint-i18n-ownership` n'admet dans `features/X/` que les clés `X-*`, et les clés de
  l'accueil sont `homepage-*`. Cas : dernières écritures
  listées / vide / vide **guidé** / échec (l'échec prime sur le guidé) ; factures ouvertes chiffrées pour un rôle
  **Consultation** ; « dont M échues » absent à zéro ; vide guidé — sans « première » (une facture émise puis soldée
  laisse la tuile vide : « Créez votre première facture » serait faux) ; un rôle **Consultation** voit le texte vide
  **non guidé**, sans injonction à une action qu'il ne peut faire ; « Solde comptable » et le total renommé ; total
  **dédoublonné** par compte lié, avec sa note ; écart affiché seulement s'il est non nul **au centime** (`"100.0049"`
  contre `"100.00"` : pas d'écart ; `"100.005"` contre `"100.01"` : pas d'écart — le cas qui départage l'arrondi du
  dépôt de l'arrondi bancaire) ; boutons d'action absents pour Consultation ; total en `big.js` (`0.1 + 0.2`) ;
  `formatChfBalance` sur une chaîne à 4 décimales ;
- **E2E** (sélecteurs `data-testid`, jamais un libellé traduit — garde `e2e-selecteurs-traduits`) — testids :
  `homepage-entry-row`, `homepage-entries-unavailable`, `homepage-invoices-open-count`, `homepage-invoices-open-total`,
  `homepage-invoices-overdue`, `homepage-bank-statement-{id}`, `homepage-bank-gap-{id}`, `homepage-bank-total` :
  - une écriture postée par l'API apparaît dans la tuile — helper **neuf** `createJournalEntryViaApi` dans
    `tests/e2e/helpers/api-fixtures.ts` (aucun n'existe) ;
  - une facture validée apparaît dans « Factures ouvertes » avec son montant ;
  - un rôle **Consultation**, créé par l'API sur le patron de `homepage-reminders.spec.ts:59-88`, voit les factures
    ouvertes chiffrées et aucun bouton d'action — la frontière HTTP n'est vérifiée que là ;
  - un passage **axe** sur l'accueil **peuplé** (écriture, facture, compte avec relevé et écart) — celui de
    `homepage-settings.spec.ts:65-70` tourne sur un accueil vide et ne voit rien de ce que la story ajoute. Les
    écritures sont une liste (`ul`/`li`), chaque lien a un nom accessible, l'écart a un libellé (pas la couleur seule) ;
  - montage déterministe : `seedTestState('with-company')` en `beforeAll` (patron `homepage-reminders.spec.ts:19`),
    l'écriture datée pour être la plus récente.
- Les E2E existants de l'accueil (`homepage-reminders.spec.ts`, `homepage-settings.spec.ts`) gardent leurs sélecteurs :
  les `data-testid` `homepage-card-*` et `homepage-reminders-count`, et le titre de la tuile « Comptes bancaires », sont
  conservés.

**AC 8 — Le manuel et le CHANGELOG.** `user-manual.tex` § *Tableau de bord* réécrit : les trois tuiles telles qu'elles
sont, le solde **comptable** expliqué (le grand livre, pas la banque ; il ne reflète un encaissement qu'une fois
celui-ci comptabilisé), le dernier relevé et l'écart, et ce que voit un rôle Consultation. PDF régénéré, contrôlé
**aplati**. Le manuel renvoie à l'**import bancaire** pour l'origine du relevé (le solde de clôture du fichier CAMT). La capture
`dashboard.png` (`% TODO capture`, `:220`) ne correspondra plus : limite assumée, déjà marquée. Le glossaire (« Actif :
liquidités, créances… », `:2065`) parle de liquidités au sens comptable : **inchangé**. Manuels DE/IT/EN : un README
chacun, rien à traduire. CHANGELOG, section `## [0.12.1] — Non publié` : `Fixed` (#388, #389), sous la section `### Changed` **existante**
(« Total liquidités » → « Total (solde comptable) »). Le § *Tableau de bord* dit que **seuls les relevés CAMT.053 portent
un solde de clôture** — un import CSV n'en a pas ; la section *Import bancaire* le dit aussi. README et site relus
(`grep -rni "tableau de bord\|liquidit" README.md website/`).

## Tasks / Subtasks

- [ ] **T1 — le dernier relevé et le solde à sa date** (AC 4, 6), `kesh-db` `bank_accounts.rs`.
- [ ] **T2 — le DTO** (AC 4, 6), `kesh-api` `routes/bank_accounts.rs`.
- [ ] **T3 — les montants en chaînes** (AC 5), `bank-accounts.api.ts`, `format.ts`, page des comptes bancaires, fixtures.
- [ ] **T4 — l'accueil** (AC 1, 2, 3, 4) : trois composants extraits dans `features/homepage/`, la page qui les
  alimente ; l'i18n (4 locales, parité, `sitesTotal`, relevé des libellés en dur). Les composants appellent `i18nMsg`
  **directement** (la garde `lint-i18n-ownership` ne lit que `i18nMsg(`, `scripts/lint-i18n-ownership.js:151` — le
  relais `msg()` de la page lui échappe). **Clés neuves** : `homepage-entries-unavailable`,
  `homepage-invoices-unavailable`, `homepage-invoices-open` (« { $n } facture(s) ouverte(s) — { $amount } »),
  `homepage-invoices-overdue` (« dont { $n } échue(s) — { $amount } »), `homepage-bank-ledger-balance` (« Solde
  comptable »), `homepage-bank-ledger-help`, `homepage-bank-total-ledger`, `homepage-bank-shared-ledger-note`,
  `homepage-bank-statement` (« Relevé du { $date } : { $amount } », date au format suisse), `homepage-bank-gap`
  (« Écart : { $amount } ») ; `homepage-invoices-empty-guided` **réécrite** sans « première » ;
  `homepage-bank-total-liquidity` **retirée**.
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

- **Dépendance avec la 25-4-d** (chaîne non mergée) : la tuile suit `due-dates`, qui compte les factures `validated`
  sans `paid_at`. Une facture **soldée** par la 25-4-d y pose `paid_at` — elle sort donc des factures ouvertes. Si ce
  contrat changeait, la tuile compterait une facture au reste nul.
- `due-dates` classe « échue » sur `UTC_DATE()`, non sur la date suisse : entre minuit et 1 h / 2 h, une facture peut
  être classée un jour trop tôt. Hérité, non corrigé ici.
- À date égale, « dernières » suit `entry_number`, unique **par exercice** : le départage n'est pas chronologique entre
  deux exercices.

- Le solde comptable ne reflète un encaissement qu'une fois celui-ci **comptabilisé** (rapprochement ou règlement) : c'est
  précisément ce que l'écart avec le relevé rend visible.
- Le relevé ne vaut que pour sa date : entre deux imports, l'écart n'est pas recalculé contre la banque réelle.
- La correction de date de valeur ne connaît que les écritures **issues d'un rapprochement** (`matched_entry_id`). Une
  écriture saisie à la main pour un mouvement bancaire, à une date différente de sa comptabilisation par la banque,
  n'est pas corrigée — l'écart le montre, et c'est juste : rien ne relie cette écriture au relevé.

### Modules

`kesh-db` (dernier relevé, solde à une date), `kesh-api` (DTO), `frontend` (accueil et trois composants
`features/homepage/`, API et formateur des comptes bancaires, page des comptes, helper E2E), `kesh-i18n` (+ `docs`,
`CHANGELOG`) — **quatre modules de code**, sous le seuil.

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

- **2026-10-03** — Arbitrage de Guy (« ok, vas-y ») : **H1 par la voie (a)** — l'écart est corrigé par les
  transactions rapprochées de part et d'autre de la date du relevé, la date de l'écriture étant lue via
  `matched_entry_id` ; **pas de découpage**.
- **2026-10-03** — Validation P2 (Opus, prompt `25-6-a-validate-prompt-p2.md`) : **1 HIGH, 6 MED, 8 LOW**.
  - Retenus :
    - l'arrondi de l'écart en `big.js` côté client (`round_dp` est bancaire) (M1) ;
    - la non-régression écrite contre le code actuel, avant la réécriture, JSON `"0"` (M2) ;
    - le partage compté en SQL, archivés compris (M3) ;
    - les textes guidés (« première ») et la variante Consultation (M4) ;
    - axe sur l'accueil peuplé (M5) ;
    - les imports CSV sans solde (M6) ;
    - `formatChfBalance` qui compile (L1), `CLES_RELEVEES` retiré (L2), `### Changed` existante (L3), `i18nMsg`
      direct (L4), clés nommées (L5), règle de dédoublonnage écrite (L6), E2E déterministes (L7).
  - ⚠️ **H1 (HIGH) — date de valeur contre date de comptabilisation** : le relevé CAMT (`CLBD`) est un solde par date de
    comptabilisation ; les écritures de rapprochement sont datées à la date de **valeur**
    (`reconciliation.rs:1958`). Sur des livres entièrement rapprochés, un mouvement de fin de mois à valeur décalée
    produit un écart fictif. Deux voies soumises à Guy — **voie (a) retenue**, appliquée à l'AC 4.
  - ⚠️ **Signal de découpage** : sévérité maximale HIGH → HIGH (P1 → P2). Défauts **distincts** (P1 : partage,
    solde à une date ; P2 : date de valeur), aucun recyclé. **Arbitrage de Guy : pas de découpage.**
- **2026-10-03** — Validation P1 (Sonnet ×3, prompt `25-6-a-validate-prompt-p1.md`) : **4 HIGH, 7 MED**, LOW.
  - **Réfuté — A1 (HIGH)**, « signe de l'écart faux pour un compte de passif ». `débit − crédit` est la convention du
    relevé quel que soit le type : un passif lié à une ligne de crédit tirée de 1000 vaut −1000, comme le relevé
    (`DBIT`). Écrit à l'AC 4, avec un test.
  - **Retenus** :
    - deux comptes bancaires sur un même compte de grand livre : total dédoublonné, écart `null` (A2) ;
    - le test de l'accueil : trois composants extraits (C1) ;
    - les cas du solde à une date : ordre d'import inversé, relevé postérieur, relevé sans solde plus récent, compte de
      passif, échelle (C2, A6) ;
    - le contrat de la requête et la non-régression (B1) ;
    - `formatChfBalance` sur une chaîne (B2) ;
    - le helper E2E et les testids (C3) ;
    - un E2E pour le rôle Consultation (C4) ;
    - le mode guidé (C5) ;
    - les sites du manuel (C6) ;
    - la clé et le testid du total (C7) ;
    - `due-dates?limit=1` et la limite `UTC_DATE()` (A4) ;
    - la dépendance avec la 25-4-d (A3).
  - **LOW** : tri et montant (A5, C9, B5), les sites `Raw` (B3), les références (B4), `CLES_RELEVEES` (B6).
- **2026-10-03** — Créée (Guy : « ok, merge et continue » ; découpage 25-6-a / 25-6-b non contesté).

[#388]: https://github.com/guycorbaz/kesh/issues/388
[#389]: https://github.com/guycorbaz/kesh/issues/389
