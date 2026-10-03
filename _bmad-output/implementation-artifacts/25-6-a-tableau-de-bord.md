# Story 25.6-a : Le tableau de bord dit vrai — dernières écritures, factures ouvertes, solde comptable

Status: review

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
sont `null` quand le compte n'a aucun relevé avec solde ; **sans compte de grand livre lié**, le relevé
(`statementClosingBalance`, `statementDate`) est **rendu** — c'est un fait bancaire — et seuls `currentBalance` et
`ledgerBalanceAtStatement` sont `null` (*amendé le 2026-10-03, revue de code P1, C-2 ; la première rédaction voulait les
trois champs à `null`*) ; `ledgerBalanceAtStatement` (donc l'écart) est aussi `null` quand **plusieurs comptes bancaires partagent le
compte de grand livre** — son solde ne s'attribue pas à l'un d'eux. « Dernier » = `period_to` le plus récent **parmi les
relevés qui portent un solde** (un relevé sans `closing_balance` plus récent ne masque pas un plus ancien), départagé par
`imported_at` puis `id`. **Signe** : `débit − crédit` du compte lié, quel que soit son type — c'est la convention du
relevé (un découvert est négatif au CAMT, `DBIT`, comme le solde d'un compte de passif lié à une ligne de crédit tirée).
**Date de valeur** (arbitrage de Guy, 2026-10-03, voie (a)) : le solde du relevé CAMT (`CLBD`, `camt053/mod.rs:541`)
vaut à la fin de la période du relevé (`period_to`, lu de `FrToDt/ToDtTm`, `:336` — la date propre du bloc `<Bal>`
n'est pas lue) et reflète les mouvements par date de **comptabilisation bancaire**, alors que les écritures de rapprochement sont datées à la date
de **valeur** (`reconciliation.rs:1958`, `:2295`, `kesh-reconciliation/src/manual.rs:35`). `ledgerBalanceAtStatement`
est donc **corrigé** par les transactions bancaires **rapprochées** du compte (`bank_transactions.status = 'reconciled'`,
`matched_entry_id` non nul) dont les deux dates tombent de part et d'autre de `period_to` : **+** le mouvement que
l'écriture `matched_entry_id` porte **sur le compte de grand livre lié actuel** (`SUM(jel.debit − jel.credit)`,
`jel.entry_id = matched_entry_id AND jel.account_id = <compte lié>`, une fois par écriture) pour les transactions dont
`booking_date ≤ period_to` et dont l'écriture (`journal_entries.entry_date`) est **postérieure** ; **−** ce même
mouvement pour celles dont `booking_date > period_to` et dont l'écriture est **antérieure ou égale**. La date de
l'écriture est **lue**, jamais déduite de `value_date` (un rapprochement manuel peut la choisir). On somme la **ligne de
l'écriture**, pas `bank_transactions.amount` (validation P3) : une écriture posée sur un **autre** compte — lien du compte
bancaire **changé** après des rapprochements, `set_journal_account_id_for_company` ne le garde pas — sort d'elle-même de
la correction, et la correction ne suppose plus que la ligne bancaire vaille `amount` (vrai aujourd'hui sur les cinq
chemins de rapprochement, `reconciliation.rs:1575, 2022, 2386, 3082, 3547`, mais rien ne le garantit demain). Une transaction **non rapprochée** n'entre pas dans la correction : son absence du
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
MariaDB 10.11). La **correction de date de valeur** se calcule dans une **table dérivée (ou sous-requête corrélée) par
compte bancaire**, jamais par une jointure `bank_transactions ⋈ journal_entries` dans le même `FROM` que les lignes du
grand livre — elle multiplierait ces lignes par le nombre de transactions et fausserait les deux `SUM`. Le test de
non-régression de `currentBalance` tourne **avec des transactions rapprochées présentes**, ce qui détecte ce produit
cartésien. ⚠️ **Le calcul actuel n'a aucun test** : un test de **non-régression** est écrit et vert **contre le code
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
  d'écriture diffère de `value_date` : la date **lue** fait foi ; le chemin **facture** (écriture datée
  `value_date.unwrap_or(booking_date)`, `reconciliation.rs:1270`) ; un **lien changé** après rapprochement (écritures sur
  l'ancien compte) : hors de la correction ; un rapprochement **annulé** (`reconciliation_cancel.rs:331` —
  `matched_entry_id` à `NULL`, `pending`) : hors de la correction ;
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
  dépôt de l'arrondi bancaire ; et `"-100.005"` contre `"-100.01"` — l'écart est **signé**, et la référence
  `vat-purchase.ts:12` ne vaut que pour des montants positifs) ; boutons d'action absents pour Consultation ; total en `big.js` (`0.1 + 0.2`) ;
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
**aplati**. Le manuel dit que l'écart tient compte des **dates de valeur** des mouvements rapprochés, et ses limites (une écriture
saisie à la main n'est pas reliée au relevé). Le manuel renvoie à l'**import bancaire** pour l'origine du relevé (le solde de clôture du fichier CAMT). La capture
`dashboard.png` (`% TODO capture`, `:220`) ne correspondra plus : limite assumée, déjà marquée. Le glossaire (« Actif :
liquidités, créances… », `:2065`) parle de liquidités au sens comptable : **inchangé**. Manuels DE/IT/EN : un README
chacun, rien à traduire. CHANGELOG, section `## [0.12.1] — Non publié` : `Fixed` (#388, #389), sous la section `### Changed` **existante**
(« Total liquidités » → « Total (solde comptable) »). Le § *Tableau de bord* dit que **seuls les relevés CAMT.053 portent
un solde de clôture** — un import CSV n'en a pas ; la section *Import bancaire* le dit aussi. README et site relus
(`grep -rni "tableau de bord\|liquidit" README.md website/`).

## Tasks / Subtasks

- [x] **T1 — le dernier relevé et le solde à sa date** (AC 4, 6), `kesh-db` `bank_accounts.rs`.
- [x] **T2 — le DTO** (AC 4, 6), `kesh-api` `routes/bank_accounts.rs`.
- [x] **T3 — les montants en chaînes** (AC 5), `bank-accounts.api.ts`, `format.ts`, page des comptes bancaires, fixtures.
- [x] **T4 — l'accueil** (AC 1, 2, 3, 4) : trois composants extraits dans `features/homepage/`, la page qui les
  alimente ; l'i18n (4 locales, parité, `sitesTotal`, relevé des libellés en dur). Les composants appellent `i18nMsg`
  **directement** (la garde `lint-i18n-ownership` ne lit que `i18nMsg(`, `scripts/lint-i18n-ownership.js:151` — le
  relais `msg()` de la page lui échappe). **Clés neuves** : `homepage-entries-unavailable`,
  `homepage-invoices-unavailable`, `homepage-invoices-open` (« { $n } facture(s) ouverte(s) — { $amount } »),
  `homepage-invoices-overdue` (« dont { $n } échue(s) — { $amount } »), `homepage-bank-ledger-balance` (« Solde
  comptable »), `homepage-bank-ledger-help`, `homepage-bank-total-ledger`, `homepage-bank-shared-ledger-note`,
  `homepage-bank-statement` (« Relevé du { $date } : { $amount } », date au format suisse), `homepage-bank-gap`
  (« Écart : { $amount } ») ; `homepage-invoices-empty-guided` **réécrite** sans « première » ;
  `homepage-bank-total-liquidity` **retirée**.
- [x] **T5 — tests** (AC 7).
- [x] **T6 — manuel, CHANGELOG** (AC 8).
- [x] **T7 — gates** : backend complet, frontend complet, **E2E complet**.

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
- **Résidu après annulation de rapprochement** : une transaction comptabilisée par la banque **après** `period_to`, dont
  l'écriture d'origine était **antérieure ou égale** et dont le rapprochement a été annulé (la contre-passation est
  datée du jour), redevient `pending` et sort de la correction — le grand livre à `period_to` porte l'écriture, le
  relevé non : écart fictif. Rare.
- La correction parcourt toutes les transactions rapprochées du compte antérieures à `period_to` à chaque ouverture de
  l'accueil (`idx_bank_transactions_pending` sert la recherche). Acceptable à cette échelle ; `EXPLAIN` relevé à la mise
  en œuvre.

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

Claude Opus 5.5 (`claude-opus-5-5`).

### Debug Log References

- Deux erreurs de montage dans les tests `kesh-db`, corrigées avant toute conclusion : l'archivage d'un compte bancaire
  est la colonne `archived` (pas `active`) ; `seed_accounting_company` ne se pose qu'une fois par base (`admin` est
  unique) — la seconde société est montée en SQL brut.
- Garde-fous i18n recomptés depuis la source, pas ajustés : la page d'accueil pesait 17 sites (15 appels par son relais
  `msg()`, sa déclaration, son corps) et en pèse 24 (1 + 23 dans les trois tuiles) — `sitesTotal` 1758 → 1765 ; le relais
  disparaît — `relais` 7 → 6, `sitesNonResolus` 34 → 32.

### Completion Notes List

- **T1, le dernier relevé et le solde à sa date** — `list_by_company_with_balances` rend désormais une
  `BankAccountBalances` par compte : `current_balance` et `last_transaction_date` **inchangés** (même expression), plus le
  dernier relevé portant un solde (`ROW_NUMBER()` sur `period_to`, `imported_at`, `id`), le solde comptable à sa date et
  sa **correction de date de valeur** (deux sous-requêtes corrélées sur les lignes de l'écriture rapprochée, sur le compte
  lié actuel, dédoublonnées par `je.id IN (…)`), et le décompte du partage (tous les comptes de la société, archivés
  compris). Une requête agrégée par appel, chaque somme dans sa propre table dérivée ou sous-requête — aucune jointure
  `bank_transactions` dans le `FROM` des lignes du grand livre.
- **Non-régression, commit séparé et antérieur** (`8713c9de`) : verte contre le code d'origine, puis contre le code
  réécrit — JSON `"380.0000"`, `"0"`, `null`, ordre de la liste, avec une transaction rapprochée présente.
- **T2, le DTO** — trois champs neufs, documentés.
- **T3, les montants en chaînes** — `currentBalance` et les champs neufs restent des chaînes ; `formatChfBalance` prend une
  chaîne ou un `Big` (arrondi `big.js` demi loin de zéro, puis `Number` d'un montant déjà au centime) ; champs neufs
  optionnels côté `Raw`, normalisés à `null` ; fixtures complétées.
- **T4, l'accueil** — trois tuiles extraites dans `features/homepage/` (`RecentEntriesCard`, `OpenInvoicesCard`,
  `BankAccountsCard`), calculs purs dans `homepage.ts` (`entryAmount`, `ledgerTotal` dédoublonné, `statementGap` au
  centime). La page charge les quatre sources en parallèle. Dix clés neuves × 4 locales, `homepage-invoices-empty-guided`
  réécrite, `homepage-bank-total-liquidity` retirée (parité : 28 clés `homepage-*` par locale).
- ⚠️ **Écart à la fiche** : le testid `homepage-invoices-open-total` est remplacé par un attribut `data-amount` sur
  `homepage-invoices-open-count` — un élément `sr-only` portant le montant brut l'aurait fait lire deux fois aux lecteurs
  d'écran.
- ⚠️ **Couverture E2E du relevé et de l'écart** : non exercée en E2E (il faudrait un import CAMT réel) ; tenue par les dix
  tests `kesh-db` et par le Vitest de `BankAccountsCard`. Le passage axe porte sur l'accueil peuplé d'une écriture, d'une
  facture et d'un compte bancaire, sans relevé.
- **T5, tests** — **périmètre : de `91ca244f` (validation close, aucun code) à l'arbre de travail** :
  - `kesh-db` **+10** (`bank_account_statement_gap.rs`, neuf) ;
  - `kesh-api` **+2** (`bank_accounts_e2e.rs` : non-régression, forme des champs neufs) ;
  - Vitest **+20** (`homepage.test.ts` 9, `HomepageCards.test.ts` 11) ;
  - E2E **+4** (`homepage-dashboard.spec.ts`, neuf) et le helper `createJournalEntryViaApi`.
- **Gates** (T7) : base remise à zéro, `scripts/test-fast.sh` **2559/2559, 4 ignorés** (fmt, clippy, nextest) ; frontend
  `check` 0 erreur, `lint-i18n-ownership` PASS, `test:unit` **865/865**, `build` OK ; **E2E complet** sur `kesh_e2e`
  reconstruite (run de 16:04 UTC) : **232 passés, 8 échoués, 19 ignorés** — les 8 attendus (KF-029 ×7,
  `sidebar-navigation:75` KF-052) ; les 4 tests de `homepage-dashboard.spec.ts` verts.
- **Mutations tuées** (`kesh-db`) : tri par ordre d'import ; imports sans solde admis ; borne `<` au lieu de `<=` (2
  tests) ; correction de date de valeur retirée ; partage compté sur les seuls comptes actifs.
- **T6, manuel et CHANGELOG** — § *Tableau de bord* réécrit, sous-section *Solde comptable, relevé et écart* (dates de
  valeur, leurs deux limites, CAMT seul), `\label{sec:import-bancaire}` et une phrase à l'import bancaire ; PDF régénéré
  et contrôlé aplati. CHANGELOG `[0.12.1]` : `Fixed` (#388, #389) et `Changed`. README relu : seule mention, une
  évolution prévue en v0.5+, toujours vraie.

### File List

- `CHANGELOG.md`
- `crates/kesh-db/src/repositories/bank_accounts.rs`
- `crates/kesh-db/tests/bank_account_statement_gap.rs` (neuf)
- `crates/kesh-api/src/routes/bank_accounts.rs`
- `crates/kesh-api/tests/bank_accounts_e2e.rs`
- `crates/kesh-i18n/locales/{de-CH,en-CH,fr-CH,it-CH}/messages.ftl`
- `docs/manual/fr/user-manual.{tex,pdf}`
- `frontend/src/lib/features/bank-accounts/{bank-accounts.api.ts,format.ts,bank-accounts.api.test.ts,BankAccountJournalLinkForm.test.ts}`
- `frontend/src/lib/features/homepage/{homepage.ts,RecentEntriesCard.svelte,OpenInvoicesCard.svelte,BankAccountsCard.svelte,homepage.test.ts,HomepageCards.test.ts}` (neufs)
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/src/routes/(app)/+page.svelte`
- `frontend/src/routes/(app)/bank-accounts/+page.svelte`
- `frontend/tests/e2e/helpers/api-fixtures.ts`
- `frontend/tests/e2e/homepage-dashboard.spec.ts` (neuf)

## Change Log

- **2026-10-03** — Revue de code P2 (Opus, prompt `25-6-a-review-prompt-p2.md`) : **0 CRITICAL/HIGH, 2 MED, 4 LOW**,
  retenus. **M-1** — le filtre `status` existe en **deux** exemplaires (`booked_before_entered_after`,
  `booked_after_entered_before`) et le test de P1 n'en tuait qu'un : le symptôme de P1 reproduit sur la copie jumelle,
  faute d'avoir grepé le symptôme. Test miroir `seule_une_transaction_rapprochee_corrige_le_miroir` ; mutation de la
  seconde copie **exécutée et tuée**. **M-2** — l'AC 4 contredisait encore le comportement retenu en P1 (relevé sans
  compte lié) : amendé et daté. LOW : « Trois limites » au manuel (L-1) ; repli Svelte de `bank-accounts-labels-balance`
  à « Solde comptable » (L-2) ; test négatif « compte non lié avec relevé : pas de non calculable » (L-3) ; Vitest du
  câblage de la page — un rejet de `listBankAccounts` mène à la tuile en échec, mutation tuée (L-4). La P2 a confirmé le
  reclassement de F1 (aucun autre écrivain de `matched_entry_id`) et le SQL borné (5 `?` / 5 `.bind`, `currentBalance`
  inchangé y compris pour un compte lié archivé). Gate ciblé : `fmt`, `clippy` workspace, `binary(bank_account_statement_gap)`
  12/12, frontend `check` 0 erreur, `test:unit` **872/872**. **Gate complet au push.**
- **2026-10-03** — Revue de code P1 (Sonnet ×3, prompt `25-6-a-review-prompt-p1.md`) : **0 CRITICAL/HIGH, 4 MED**, LOW.
  - **Retenus** :
    - un test qui ne tuait pas la mutation revendiquée (`status`) : test neuf avec une transaction `pending` liée, qui
      la tue (C-1, mutation exécutée) ;
    - le relevé rendu **sans compte lié**, contraire à l'AC 4 et non déclaré : comportement **gardé** (le relevé est un
      fait bancaire), documenté au DTO, au manuel et ici (C-2) ;
    - aucun test du mappage TypeScript des champs neufs : deux Vitest, réponse de liste avec valeurs et réponse de
      mutation sans les champs (C-3).
  - **Reclassé** : **F1 (MED → LOW), plusieurs transactions sur une écriture**. Non atteignable : les cinq chemins de
    rapprochement lient une écriture qu'ils viennent de créer (`reconciliation.rs:1520, 2016, 2381, 3075, 3541`).
    L'invariant est écrit à la doc de `list_by_company_with_balances`.
  - **LOW corrigés** :
    - la tuile bancaire dit son échec au lieu de disparaître (A-2, F4) ;
    - « écart non calculable » pour un compte du grand livre partagé (F3) ;
    - `-0.00 CHF` évité (F2) ;
    - la requête agrégée bornée aux comptes liés (F5) ;
    - commentaires périmés (C-4) ;
    - le chemin de menu du manuel aligné sur l'interface, « Administration → Comptes bancaires », aux trois sites, et
      la limite « résidu après annulation » écrite (C-5) ;
    - « Solde comptable » aussi sur la page des comptes bancaires (C-6) ;
    - E2E : présence avant absence, date locale au lieu d'UTC (C-8, F6).
  - **Écartés** :
    - `canManage` lu au montage : le layout attend l'authentification (A-3, vérifié par la lentille B) ;
    - l'isolation par société, sans conséquence aujourd'hui (C-7).
  - **Gates de la remédiation** :
    - backend complet sur base remise à zéro, **2560/2560**. Un premier run s'était arrêté au premier échec, un test
      de performance de `kesh-core` hors périmètre (226 ms > 200 ms sous charge) ; il est passé au second run ;
    - frontend `check` 0 erreur, `test:unit` **870/870**, `sitesTotal` 1765 → 1767 ;
    - E2E de l'accueil **9/9**. **E2E complet au push.**
- **2026-10-03** — Implémentée (dev-story) : T1–T7, gates ci-dessus. Statut → `review`.
- **2026-10-03** — Validation P4 ciblée (Haiku, prompt `25-6-a-validate-prompt-p4.md`) : **0 finding**, preuves des
  quatre vérifications fournies. ⚠️ Son **cas 2 chiffré est mal posé** (il garde le grand livre à 800 alors que
  l'écriture du 30 y figure, d'où un « écart −400 volontaire ») — **refait par l'orchestrateur** : grand livre au 30 = 1000,
  relevé = 800 (mouvement comptabilisé le 31), correction −200, écart **nul** ; la règle de la fiche est juste. **Boucle
  close** : P1 4H/7M (Sonnet ×3) → P2 1H/6M (Opus) → P3 2M (Sonnet) → P4 0 (Haiku).
- **2026-10-03** — Validation P3 (Sonnet, prompt `25-6-a-validate-prompt-p3.md`) : **0 CRITICAL/HIGH, 2 MED, 4 LOW**,
  retenus. La correction somme la **ligne de l'écriture sur le compte lié actuel**, pas `amount` — juste après un
  changement de lien (M1) ; le contrat de requête impose une table dérivée par compte bancaire, non-régression avec
  transactions rapprochées présentes (M2) ; tests annulation, lien changé, chemin facture, arrondi négatif (L1) ;
  résidu après annulation et performance en limites (L2, L4) ; `period_to` et `CLBD` reformulés (L3) ; manuel :
  correction et limites de la date de valeur (LOW). La P3 a vérifié que la ligne bancaire vaut `amount` sur les cinq
  chemins de rapprochement et le sens de la correction (chiffré). Trend : P1 4H/7M → P2 1H/6M → P3 2M.
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
