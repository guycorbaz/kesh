# Story 25.7 : Soldes de départ — avertir si le report à-nouveau manque, compléter un compte oublié

Status: review

**Issue : [#445]**, que cette story **ferme** : la PR porte `closes #445` dans le **titre ET le corps**. Branche
`story/25-7-soldes-de-depart`, partie de `chore/epic-25-retrospective` (la rétrospective de l'Epic 25 voyage dans la
même PR). **Dernière story de l'Epic 25** (décision D1 de la rétrospective du 2026-10-06) : le jalon « Vague 1 » ne se
clôt qu'avec elle. Elle précède la release v0.12.1, après laquelle Guy installe Kesh **à partir de zéro** et passe donc
par cet écran (décisions D2, D3).

## Arbitrages proposés (retenus par défaut le 2026-10-06 — Guy : « ok, continue » ; révisables)

1. **« Compte oublié » = compte de bilan JAMAIS MOUVEMENTÉ** : aucune ligne d'écriture sur ce compte, dans aucun
   exercice, au moment du complément. **Écart à l'issue**, qui dit « solde actuel nul » mais justifie la restriction par
   « un compte déjà mouvementé ne peut pas voir son ouverture réécrite » : seul « jamais mouvementé » tient cette
   promesse. Conséquences, à dire au manuel :
   - une écriture **contre-passée** garde ses lignes (`reverses_entry_id`) : le compte reste mouvementé. En
     particulier, après la contre-passation de toute l'écriture d'ouverture, **plus aucun compte de l'ouverture n'est
     complétable** — le remède est alors une écriture manuelle ;
   - une écriture **détruite** retire ses lignes : côté utilisateur, le seul chemin qui détruise une écriture est la
     **dévalidation d'une facture** (`invoices::unvalidate` → `delete_in_tx(…, false)`, `invoices.rs:1654`) — une
     écriture comptabilisée ne se supprime plus depuis le gel (refus `EntryIsPosted` dans `delete_in_tx`,
     `journal_entries.rs:1005`) ; hors usage courant, la remise à zéro de la démo et la restauration d'une sauvegarde
     détruisent aussi des écritures. Le compte redevient alors complétable s'il n'avait pas d'autre mouvement ;
   - un compte **archivé** n'est pas proposé (il doit être actif et postable) ; réactivé, il l'est s'il n'a jamais été
     mouvementé.
2. **Date de l'ajustement** — « aujourd'hui » est la **date UTC** du serveur (`Utc::now().date_naive()`, comme
   `crates/kesh-db/src/repositories/settlement_cancellation.rs:131` ; ⚠️ entre minuit et une ou deux heures du matin, heure suisse, c'est encore la
   veille — le 1ᵉʳ janvier, une régularisation peut tomber dans l'exercice précédent, ou être refusée s'il est clos ;
   le manuel le dit),
   passée en **paramètre** à la fonction `kesh-db` (testable) et **recalculée sous verrou** par le POST :
   - (a) si le **premier exercice** (variante **en transaction et verrouillante** de `find_first_by_company`,
     `fiscal_years.rs:690` n'étant ni l'un ni l'autre) est **ouvert** et que son premier jour est **après**
     `books_locked_through` : le **premier jour du premier exercice**, la date de l'écriture d'ouverture. **Écart à
     l'issue**, qui ne demande que le cas (b) : un complément antidaté au début de l'exercice d'ouverture modifie les
     soldes d'ouverture et tout rapport déjà tiré sur cet exercice — c'est voulu (le compte oublié faisait partie de
     l'ouverture), et le manuel le dit ; ⚠️ `fiscal_years::close` n'impose pas d'ordre de clôture (`:740-755`) : si
     l'exercice suivant est déjà clos, son bilan reporté en est aussi modifié — propriété de toute écriture dans le
     premier exercice, écrite au manuel ;
   - (b) sinon (premier exercice clos, ou son premier jour dans la période verrouillée) : **aujourd'hui**, dans
     l'exercice **ouvert** qui couvre cette date (sous verrou, par la requête à une ligne de l'AC 4 étape 3 ; le status, sans verrou) — une régularisation, comme le dit
     l'issue ;
   - (c) si aucun exercice ouvert ne couvre aujourd'hui → refus `OPENING_COMPLEMENT_NO_OPEN_FISCAL_YEAR` ; si
     aujourd'hui est lui-même dans la période verrouillée → refus `OPENING_COMPLEMENT_DATE_LOCKED` — **défensif** :
     `lock_books` refuse une borne `>= aujourd'hui` (`crates/kesh-db/src/repositories/companies.rs:343`), le cas n'est atteignable qu'en test, par le
     paramètre `today`.
   La date affichée par le status (AC 3) est **indicative** : le POST fait autorité (un verrou posé entre-temps, ou
   minuit UTC franchi, peut la changer).
3. **La contrepartie est calculée par le serveur** sur le compte de rôle *Bénéfice reporté* (`RetainedEarnings`),
   cherché **dans la société** : l'utilisateur ne saisit que les comptes oubliés. **Si les lignes saisies
   s'équilibrent entre elles** (deux comptes oubliés qui se compensent), **aucune contrepartie** : l'écriture porte les
   seules lignes saisies — une ligne à zéro serait refusée par la contrainte `chk_jel_debit_credit_exclusive`. (Une
   ligne unique, de montant strictement positif, n'a jamais un écart nul : il y a alors forcément au moins deux
   lignes, sans contrôle à écrire.) Sans compte actif portant ce rôle, ou s'il n'est pas postable → refus nommé.
4. **Audit** : l'écriture d'ajustement est tracée comme toute écriture (`journal_entry.created`, dans la transaction de
   `create_in_tx`) ; pas d'action d'audit neuve. Libellé distinct : « Complément des soldes de départ ».
5. **Droits** : ceux de l'écran actuel — Comptable et Administrateur (`comptable_routes`), Consultation refusé ; une
   clé d'API `read-write` peut l'appeler, comme la génération actuelle.

## Story

En tant que **personne qui met Kesh en service** en reprenant le bilan de son ancien logiciel,
je veux être **avertie** si mes soldes de départ s'équilibrent sans passer par le report à-nouveau, et pouvoir
**compléter** un compte oublié **sans tout ressaisir**,
afin que la mise en service — le moment le plus fragile — ne produise ni un bilan faux en silence, ni une réparation
disproportionnée.

## Les faits, vérifiés dans le code (cartographie du 2026-10-06, recontrôlée en validation P1)

- **Routes** (`crates/kesh-api/src/lib.rs:384-393`, bloc `comptable_routes` ouvert en `:346`) :
  `POST /api/v1/opening-balances` (`routes::opening_balances::generate_opening_balances`) et
  `GET /api/v1/opening-balances/status`.
- **Status** : `OpeningBalancesStatusResponse { fiscal_year, can_enter, reason }` (`routes/opening_balances.rs:93-101`),
  raisons `READY`, `NO_FISCAL_YEAR`, `FIRST_YEAR_CLOSED`, `ALREADY_HAS_ENTRIES`.
- **POST, ordre des gardes** (`:239-337`) : plafond de lignes (`MAX_LINES_PER_ENTRY = 500`, importé de
  `crate::routes::journal_entries`, `opening_balances.rs:51`) ; parse `Decimal` ; premier exercice (`find_first_by_company`, `:268`) ;
  `count_by_company > 0` → 409 `error-opening-balances-already-has-entries` (`:282-288`) ; premier exercice `Closed` →
  400 (`:292-297`) ; refus des comptes `Revenue`/`Expense` via `find_types_by_ids_in_tx` **dans une transaction
  rollbackée, hors des verrous** (`:300-319`, voir Limites) ; libellé en langue comptable (`:324-327`) ;
  `accounting::validate` (`:337`) ; `NewJournalEntry { journal: OD, entry_date: fiscal_year.start_date }`
  (`:340-357`) ; `create_opening_entry` (`:361`). Le corps ne porte **ni journal ni date** (`:71-73`).
- **Aucun contrôle du report à-nouveau** : le handler ne regarde jamais le rôle `RetainedEarnings` (défaut 1 de
  l'issue).
- **Mapping d'erreurs** : `map_opening_balances_error` (`:119-135`) ne traite que deux clés `DbError::Invariant`
  (`FY_OPENING_ALREADY_HAS_ENTRIES_KEY`, `FY_OPENING_FIRST_YEAR_CLOSED_KEY`, `fiscal_years.rs:102-108`) et délègue le
  reste à `AppError::from` ; **tout autre `DbError::Invariant` devient une 500 `INTERNAL_ERROR`** (`errors.rs:3208-3215`).
  `map_core_error` (`routes/journal_entries.rs:195-220`) ne connaît que les `CoreError` d'écriture.
- **`create_opening_entry`** (`journal_entries.rs:562-644`) : sentinelle `companies … FOR UPDATE`, exercice
  `FOR UPDATE`, `count_by_company` sous verrou, statut, puis `create_in_tx(…, enforce_postable = true)`. Ordre des
  verrous du dépôt : `companies → projects → fiscal_years`.
- **`create_in_tx`** (`:187` et suivantes) : lit `companies.books_locked_through` (`:269-275`) ; reverrouille
  **son seul** exercice (`:279-302` : `Closed` → `FiscalYearClosed`, `:290` ; hors bornes → `DateOutsideFiscalYear`,
  `:301`) ; refuse une date `<= books_locked_through` → `PeriodLocked` (`:319-325`) ; vérifie les comptes par
  `validate_lines_accounts_in_tx` (`:85-120`), **lecture NON verrouillante** de `accounts` ; audit
  `journal_entry.created` dans la transaction (`:470-481`). Ne prend pas `companies` `FOR UPDATE` — mais son
  insertion d'en-tête la prend en partagé (voir plus bas).
- ⛔ **Qui sérialise quoi (validation P2, F1 — réfute la prémisse du CRITICAL B1 de P1).** Une écriture ordinaire
  **prend bien un verrou sur `companies`** : `journal_entries.company_id` porte la clé étrangère
  `fk_journal_entries_company` (`migrations/20260412000001_journal_entries.sql:27` ; de même `fk_jens_company` pour le
  compteur), et **InnoDB pose un verrou partagé sur la ligne parente** à chaque contrôle de clé étrangère — à
  l'insertion de l'en-tête (`journal_entries.rs:366-398`), avant les lignes (`:412-427`). Précédent du dépôt :
  `crates/kesh-api/tests/reconciliation_e2e.rs:4420-4422`. Les lignes prennent de même un verrou partagé sur le compte
  (`fk_jel_account`). **Ordre de fait d'une écriture ordinaire** : son exercice `FOR UPDATE` (`:279-287`), le compteur,
  puis `companies` (partagé, en-tête), puis ses comptes (partagé, lignes).
- ⛔ **Deux familles d'écritures ordinaires, deux ordres (validation P3, R-1).** Une écriture **sans projet** prend son
  exercice, puis `companies` en partagé (clé étrangère). Une écriture **taguée projet** — et les flux facture et
  fournisseur **quand ils portent un projet** (`validate_invoice` ne la prend jamais, `invoices.rs:1978-1983` ;
  `supplier_invoices.rs:282-284` seulement avec `project_id`) — prend d'abord la sentinelle `companies … FOR UPDATE` (`projects.rs:99` →
  `bank_accounts.rs:592`), puis l'exercice : c'est l'ordre global déclaré du dépôt, `companies → projects →
  fiscal_years` (Pattern 5, `journal_entries.rs:256-259`). Les deux familles forment déjà un cycle entre elles — c'est
  préexistant. Un interblocage finit en 500 (`map_db_error`, `crates/kesh-db/src/errors.rs:781-813`, ne traite pas
  1213), sur la victime que choisit InnoDB.
- ⛔ **Conséquence (validation P2 F2, puis P3 R-1/H1)** : le complément ne prend `companies` **ni en exclusif** (cycle
  avec l'écriture sans projet, P2), **ni après l'exercice** (cycle avec l'écriture taguée, la génération, P3). Il la
  prend **en partagé, en premier** — l'ordre global du dépôt, compatible avec le partagé de l'écriture sans projet.
- **Rôle `RetainedEarnings`** : `AccountRole` (`entities/account.rs:89-110`, 10 rôles), singleton (`is_singleton()`,
  `:157`), colonne générée `accounts.singleton_role` (le rôle si le compte est actif, sinon `NULL` ; unicité
  `uq_accounts_company_singleton_role`). **Aucune fonction générique de recherche par rôle** : motif SQL en ligne
  (`company_invoice_settings.rs:563-566`, `… WHERE company_id = ? AND singleton_role = ? … FOR UPDATE`) ;
  `accounts.rs:690` `find_singleton_role_holder` est privé. Les trois plans désignent **2970 « Bénéfice/perte
  reporté »**, `Liability`, postable (`pme.json:39`, `independant.json:37`, `association.json:35`).
- **Bilan vs résultat** : `kesh-report/src/opening.rs:42` `is_bilan(t) = Asset | Liability` ; **pas de type `Equity`**
  (les capitaux propres sont `Liability`).
- **Exercices** : `find_first_by_company` (`fiscal_years.rs:690`), `find_covering_date` (`:469`),
  `find_open_covering_date` (`:543`, en transaction, `FOR UPDATE`).
- **Écran** (`frontend/src/routes/(app)/settings/opening-balances/+page.svelte`, 403 lignes, 26 sites `i18nMsg(`) :
  états chargement / erreur / verrou (`data-testid="opening-balances-locked"`, `data-reason`) / grille. Grille =
  comptes `active && postable && Asset|Liability` (`:82-91`) ; totaux `computeBalance(rows)` (big.js, `:101`). Le
  champ de rôle existe côté client : `role: AccountRole | null` (`accounts.types.ts:54`). Message du verrou
  `ALREADY_HAS_ENTRIES` (`:230-234`, repli `:233`). Feature `lib/features/opening-balances/opening-balances.{api,types}.ts`
  (union `OpeningBalancesReason`, 4 valeurs). Menu `comptableOnly` (`routes/(app)/+layout.svelte:128-135`). **L'écran
  n'est pas proposé à l'onboarding.**
- **Messages qui conseillent un remède périmé** (validation P1, C2) : `opening-balances-locked-already-has-entries`
  (fr-CH `messages.ftl:927` : « **Corrigez l'écriture d'ouverture directement dans le journal**, **ou supprimez
  toutes les écritures** pour recommencer » — deux gestes impossibles depuis le gel de la 24-4a : une écriture
  comptabilisée ne se modifie ni ne se supprime ; validation P2, F8) et `error-opening-balances-already-has-entries`
  (fr-CH `:935`), dans les **4 locales** ; repris en repli Rust (`opening_balances.rs:~120`) et Svelte (`:233`).
- **Tests existants** : `crates/kesh-api/tests/opening_balances_e2e.rs` (21), `crates/kesh-db/tests/
  opening_balances_repository.rs` (10), `opening-balances-page.test.ts` (19 Vitest, dont `:206-216`, `:360-384`,
  `:400-419` qui simulent `ALREADY_HAS_ENTRIES` et affirment le verrou), `frontend/tests/e2e/opening-balances.spec.ts`
  (2 Playwright ; `:107-112` attend le verrou après génération).
- **Manuel** (`docs/manual/fr/user-manual.tex`) : `:383-404` § comptes 9000/9100/9200 (avertissement `:394`,
  contre-passer et ressaisir `:400-402`) ; `:648` § « Reprise de comptabilité — soldes de départ »
  (`sec:soldes-depart`), étapes `:654-659`, **verrou « définitivement » et remède « contre-passez… trois écritures au
  lieu d'une » `:661-664`** ; `:1682` (balance : « l'ouverture… datée du premier jour… apparaît en mouvements ») — un
  complément daté du jour (arbitrage 2 b) y fait exception. `admin-manual.tex` : aucune mention de l'écran.

## Acceptance Criteria

**AC 1 — L'avertissement à la saisie (défaut 1 de l'issue).** Dans la grille du mode « ouverture », quand la saisie est
**équilibrée** et qu'**aucun montant** ne porte sur le compte de rôle `RetainedEarnings`, l'écran affiche un
avertissement (`data-testid="opening-balances-no-retained-earnings"`) qui nomme ce compte (numéro, libellé) et dit
pourquoi : le report à-nouveau — la différence entre actifs et passifs de l'ancien bilan — doit y être porté ; une
saisie équilibrée sans lui signifie que l'écart a été porté sur un autre compte, ou que le report vaut réellement
zéro ; il invite à **comparer les totaux Actifs et Passifs affichés (AC 2) à ceux de l'ancien bilan**. **Il signale, il ne bloque pas** (doctrine #301) : le bouton « Générer » reste actif. Cas distincts, chacun son
texte : aucun compte actif ne porte le rôle ; le compte qui le porte n'est pas postable (il n'est alors pas dans la
grille). Le rôle se lit sur `account.role` (`accounts.types.ts:54`).

**AC 2 — Totaux actif / passif et écart attendu.** La grille affiche, en plus des totaux débit / crédit existants :
- **Actifs** = Σ (débit − crédit) des lignes sur des comptes `Asset` ;
- **Passifs et capitaux** = Σ (crédit − débit) des lignes sur des comptes `Liability`, **le compte `RetainedEarnings`
  exclu** ;
- **Montant à porter au compte de report pour équilibrer** = Actifs − Passifs et capitaux : positif → **au crédit**
  du compte de report (bénéfice reporté), négatif → **au débit** (perte reportée) ;
- **Report saisi** = (crédit − débit) sur le compte `RetainedEarnings`, et l'**écart restant** = montant à porter −
  report saisi, qui vaut 0 quand la saisie est équilibrée.
⚠️ **Ce montant aide la saisie, il ne contrôle rien une fois la saisie équilibrée** (validation P2, F7) : équilibrée
sans rien sur le compte de report, il vaut 0 **par construction** — le montant mal placé est compté dans « Passifs et
capitaux ». Le vrai contrôle est de **comparer les totaux Actifs et Passifs affichés à ceux de l'ancien bilan** : l'écran
le dit à côté des totaux, l'AC 1 le dit dans l'avertissement, et le manuel aussi.
Un compte saisi contre sa nature (un `Asset` au crédit, un `Liability` au débit) compte avec son signe. Montants en
big.js, jamais en `Number`.

**AC 3 — Le status annonce le mode « compléter ».** `GET /api/v1/opening-balances/status` gagne, **sans toucher aux
champs existants** (`canEnter`, `reason`, `fiscalYear`) :
- `canComplete` (booléen) et `completeReason` (`READY`, `NO_ENTRIES`, `NO_RETAINED_EARNINGS`,
  `RETAINED_EARNINGS_NOT_POSTABLE`, `NO_OPEN_FISCAL_YEAR`, `DATE_LOCKED`, `NO_COMPLETABLE_ACCOUNT`) ;
- `completableAccounts` : les comptes de bilan actifs, postables, **jamais mouvementés**, hors compte de report
  (`id`, `number`, `name`, `accountType`) ;
- `complementDate` (date prévue, **indicative**) et l'exercice qui la porte ;
- `retainedEarningsAccount` (`id`, `number`, `name`) quand il existe.
`canComplete` est vrai **seulement** si `completeReason = READY` — donc s'il existe au moins un compte complétable.
**Priorité d'évaluation** (validation P2, R-5 ; une seule raison rendue, la première qui s'applique — l'ordre suit
celui du POST ; la branche (a)/(b) de l'arbitrage 2 est choisie d'abord, et `NO_OPEN_FISCAL_YEAR` comme `DATE_LOCKED`
ne s'évaluent qu'en branche (b), validation P3 R-8 ; le POST évalue ses refus dans ce même ordre, **après** toutes
ses lectures, AC 4 étape 5 ; `NO_COMPLETABLE_ACCOUNT` n'a pas de refus POST propre : le POST nomme le compte fautif,
`ACCOUNT_MOVED` ou un autre code par compte) : `NO_ENTRIES` (y compris aucun exercice : rien à compléter) >
`NO_OPEN_FISCAL_YEAR` > `DATE_LOCKED` >
`NO_RETAINED_EARNINGS` > `RETAINED_EARNINGS_NOT_POSTABLE` > `NO_COMPLETABLE_ACCOUNT` > `READY`. Lecture sans verrou :
le POST fait autorité.

**AC 4 — La route de complément.** `POST /api/v1/opening-balances/complete`, corps `{ lines: [{ accountId, debit,
credit }] }` — **sans** la contrepartie.

**Dans le handler**, avant toute transaction (contrôles de forme, sans lecture de la base) : au moins une ligne ;
plafond `MAX_LINES_PER_ENTRY − 1` (une ligne réservée à la contrepartie) ; montants parsés en `Decimal`, strictement
positifs, **au plus quatre décimales** (l'échelle de `DECIMAL(19,4)` : au-delà, l'arrondi à l'écriture
déséquilibrerait la contrepartie calculée avant lui), une seule colonne par ligne — l'autre vaut `"0"` ou est absente,
comme l'envoie l'écran (`+page.svelte:143-144`) ; comptes distincts. Chacun a son code (AC 5).

**Dans `kesh-db`** — fonction nouvelle, p. ex. `journal_entries::create_opening_complement(pool, company_id, user_id,
lines, description, today)` (l'audit vient de `create_in_tx`, qui n'enregistre que l'utilisateur, `journal_entries.rs:472-480`),
**une seule transaction**.

⛔ **Principe (validation P4, après trois passes dont tous les défauts graves venaient de l'ordre des verrous)** : le
dépôt n'a **pas** d'ordre de verrouillage unique — une écriture sans projet prend l'exercice puis ses comptes, le
règlement et la passation en perte prennent un compte en exclusif puis l'exercice (`invoice_settlements_write.rs:157`
puis `:200` ; `supplier_invoices.rs:639` puis `:662` ; `company_invoice_settings.rs:468`), la contre-passation
l'écriture puis les exercices. **Aucun ordre ne rend le complément exempt de cycle**, et la fiche ne le prétend pas.
Elle garantit trois choses, qui ne dépendent pas de l'ordre : (i) **aucun complément sur un compte mouvementé**,
archivé ou de résultat ; (ii) **aucune écriture** dans un exercice clos ou une période verrouillée ; (iii) **les cycles résiduels sont
nommés** (section « Cycles ») : ils passent par un compte visé, ou appartiennent à des classes **déjà présentes** entre
les flux existants (contre-passation et dévalidation contre toute écriture). Le complément y est rejoué. Elle s'obtient par une règle
simple : **tous les verrous d'abord, une ligne par requête ; les lectures ordinaires ensuite ; les refus évalués en
dernier, dans l'ordre de priorité de l'AC 3** — l'ordre des refus n'est pas celui des lectures (patron de l'étape 0-bis
de `create_in_tx_inner`, `journal_entries.rs:250-262`).

1. **la société, en partagé** : `SELECT books_locked_through FROM companies WHERE id = ? LOCK IN SHARE MODE` — ⛔ **pas
   `FOR SHARE`, erreur de syntaxe 1064 sur MariaDB 10.11** (validation P5, R5-1, vérifié sur l'image du dépôt ; aucun
   précédent de verrou partagé dans le code) ; compatible
   avec le partagé que prend l'en-tête d'une écriture sans projet ; attend une écriture taguée, une génération
   d'ouverture ou un `lock_books` en vol, qui la prennent en exclusif ; la borne est **figée** jusqu'au commit ;
2. **les comptes** :
   - comptes saisis, **par clé primaire** : `SELECT … FROM accounts WHERE id IN (…) AND company_id = ? ORDER BY id
     FOR UPDATE` — l'accès se fait **par la clé primaire**, le filtre `company_id` évitant de verrouiller les comptes
     d'une autre société (validation P5, R5-4/F5-9) ; un parcours d'un index commençant par `company_id`
     verrouillerait tous les comptes de la société (validation P4, R4-6) : l'`EXPLAIN` de cette requête **et des deux
     requêtes d'exercice** (étape 3, F5-5) est relevé au développement et cité au Dev Agent Record ; un identifiant
     absent du résultat → `ACCOUNT_INVALID` ; chacun de la société (sinon `ACCOUNT_INVALID`), actif, postable, de bilan (`Asset`/`Liability`), et ne
     portant **pas** le rôle `RetainedEarnings` ; l'exclusif est **nécessaire** : il arrête la première écriture
     concurrente sur le compte, dont les lignes demandent un partagé (clé étrangère) ;
   - compte de report, de la société : `… WHERE company_id = ? AND singleton_role = 'RetainedEarnings' LOCK IN SHARE MODE`
     (index unique, une ligne) — **partagé** : il suffit à le figer contre un archivage, et reste compatible avec le
     partagé que prennent les lignes d'une écriture ordinaire sur 2970 ;
3. **les exercices, une ligne chacun** (validation P4, R4-1/F-2 : `find_open_covering_date` n'est **pas** réutilisé —
   ses colonnes `end_date` et `status` hors index lui font verrouiller tous les exercices parcourus) :
   - le premier : `SELECT … FROM fiscal_years WHERE company_id = ? ORDER BY start_date LIMIT 1 FOR UPDATE` —
     ⛔ **sans** le départage `, id` de `find_first_by_company` (`fiscal_years.rs:690`) : **mesuré** sur la base de dev
     (validation P6), `ORDER BY start_date, id` donne un plan `Using filesort`, qui lit — donc verrouille — **tous** les
     exercices de la société avant de trier ; `ORDER BY start_date` suit l'index `uq_fiscal_years_company_start_date`
     sans tri. L'unicité `(company_id, start_date)` rend le départage inutile ;
   - le candidat du jour : `… WHERE company_id = ? AND start_date <= ? ORDER BY start_date DESC LIMIT 1 FOR UPDATE`
     — les exercices ne se chevauchent pas, c'est le seul qui puisse couvrir `today` ; sa fin et son statut sont
     contrôlés **dans le code** ; un seul verrou s'il est le premier ;
   les deux sont toujours verrouillés, **avant** de choisir la branche de l'arbitrage 2 ;
4. **les lectures ordinaires**, après tous les verrous — l'instantané de la transaction (REPEATABLE READ) s'ouvre donc
   **ici**, et tout ce que lisent ensuite les lectures ordinaires de `create_in_tx` (`books_locked_through`, les
   comptes de `validate_lines_accounts_in_tx`) est cohérent avec les verrous tenus (validation P4, R4-4) :
   - **« jamais mouvementé »** : `SELECT account_id FROM journal_entry_lines WHERE account_id IN (…) LIMIT 1` — une
     écriture non validée sur ces comptes a été attendue à l'étape 2, et plus aucune ne peut y écrire ; aucun verrou
     d'intervalle sur `idx_jel_account` ;
   - **`NO_ENTRIES`** : `SELECT EXISTS (SELECT 1 FROM journal_entries WHERE company_id = ?)` — ne verrouille aucune
     ligne d'écriture (cycle avec la contre-passation et la dévalidation, validation P3 H1) ;
5. **les refus**, dans l'ordre de priorité de l'AC 3 : `NO_ENTRIES` (dont « aucun exercice ») ; puis la **date**
   (arbitrage 2 : branche (a) ou (b), avec la borne de l'étape 1) et ses refus `NO_OPEN_FISCAL_YEAR`, `DATE_LOCKED` ;
   puis `NO_RETAINED_EARNINGS`, `RETAINED_EARNINGS_NOT_POSTABLE` ; puis les refus par compte, **par catégorie dans cet
   ordre** — `ACCOUNT_INVALID`, `RETAINED_EARNINGS_LINE`, `NOT_BALANCE_ACCOUNT`, `ACCOUNT_MOVED` — et, dans une
   catégorie, le plus petit `id` (validation P5, R5-6 : un test à plusieurs comptes fautifs est ainsi décidable) ;
6. **la contrepartie** (arbitrage 3) : écart = Σ débit − Σ crédit des lignes saisies ; > 0 → une ligne **au crédit**
   du compte de report de ce montant ; < 0 → **au débit** ; = 0 → aucune ligne ;
7. `accounting::validate` sur l'écriture finale, comme la génération (`opening_balances.rs:337`, validation P5,
   F5-7), puis `create_in_tx(…, enforce_postable = true)` — exercice clos, bornes, période verrouillée, numérotation,
   audit `journal_entry.created`. ⚠️ `create_in_tx` prend **ses propres** lectures verrouillantes après l'instantané
   (exercice `journal_entries.rs:279`, compteur et plancher `MAX(entry_number) … FOR UPDATE`,
   `journal_entry_number_sequences.rs:63`, `:131-145`) : sans incohérence, puisque l'exercice est tenu **depuis
   l'étape 3** et que tout écrivain de ces lignes doit le tenir — invariant à garder : **le verrou d'exercice ne se
   déplace jamais après l'ouverture de l'instantané**.

**Ce qui sérialise** (et que les tests prouvent) : une écriture en vol **sur un compte visé**, de quelque exercice
que ce soit → l'exclusif de l'étape 2 contre son partagé de clé étrangère ; un archivage du compte → l'étape 2 ; deux
compléments du même compte → l'étape 2 ; une écriture du même exercice ne touchant **aucun** compte visé → l'exercice
de l'étape 3 ; une génération d'ouverture, une écriture taguée, `lock_books` → l'étape 1.

**Cycles : ceux qui restent** (validation P3 R-1/R-3/H1, P4 R4-1/R4-3, P5 R5-2/R5-3/R5-4/F5-1) :
- **par un compte visé** : une transaction concurrente qui tient l'exercice voulu par le complément (étape 3) et
  demande ensuite un compte qu'il tient en exclusif (étape 2) — une écriture du même exercice qui mouvemente **pour la
  première fois** un compte visé ; ou un second complément qui saisit (à tort) le compte de report du premier, pris en
  exclusif avant le contrôle de rôle. Le règlement et la passation en perte, qui prennent le compte **puis**
  l'exercice, suivent le même ordre que le complément : pas de cycle ;
- **par les exercices, avec la contre-passation et `delete_in_tx`** : elles verrouillent l'écriture d'origine **et son
  exercice** (jointure `FOR UPDATE`, `journal_entries.rs:1527-1531`, `:1012-1019`), puis la contre-passation cherche
  l'exercice du jour par `find_open_covering_date`, qui parcourt les exercices depuis le premier. Contre-passer une
  écriture de l'exercice du jour quand un exercice antérieur existe forme un cycle avec le complément (qui tient le
  premier et veut celui du jour). Inverser l'ordre du complément ferait diverger celui-ci des flux qui parcourent en
  ordre croissant (règlement, validation, rapprochement) : pire. **Classe préexistante** — deux contre-passations
  d'exercices différents se bloquent déjà ainsi ;
- **par la dernière écriture de l'exercice** : le plancher du compteur (`journal_entry_number_sequences.rs:131-145`)
  verrouille la dernière écriture de l'exercice tenu ; la contre-passation ou la dévalidation de cette écriture, qui la
  tient avant l'exercice, forme un cycle. **Classe préexistante** pour toute écriture ordinaire ;
- **à trois, par la file d'attente** : InnoDB fait attendre une demande partagée derrière une demande exclusive **en
  attente** (écriture taguée sur `companies`). La classe existe déjà (une transaction qui tient `companies` en
  partagé par clé étrangère, puis demande un exercice) ; le complément, qui tient ce partagé **dès son début**, en
  **élargit la fenêtre**.

**Rejeu** : la route s'enveloppe dans `kesh_db::retry::retry_with` sur `is_deadlock_error` (précédents
`onboarding.rs:614`, `invoices.rs:1343`, `reconciliation.rs:850`). Il ne protège que le complément : si InnoDB choisit
l'autre transaction pour victime, celle-ci rend une 500, comme le font déjà aujourd'hui les cycles entre écritures
ordinaires. ⚠️ Au moment d'un cycle, le complément n'a modifié **aucune** ligne (toutes ses écritures sont à
l'étape 7), ce qui en fait en pratique la victime la plus légère pour InnoDB — une tendance, non une garantie. Le rejeu est sûr, chaque tentative refaisant toutes les gardes.

Réponse `201 + JournalEntryResponse`, journal `OD`, libellé « Complément des soldes de départ » (clé i18n, langue
comptable de la société).

**AC 5 — Les erreurs.** Une variante de `DbError` portant une énumération de refus (p. ex.
`DbError::OpeningComplementRefused { reason, account_id }`), mappée vers une variante d'`AppError` à **code par cause**
(patron `errors.rs:939`, `error_code: &'static str` ; ce patron ne rend qu'un 400 : la variante neuve
**dérive le statut HTTP de la raison**, selon la table) — ⛔ **jamais** un `DbError::Invariant` non mappé, qui
deviendrait une 500 :

| Refus | HTTP | Code | Clé i18n |
|---|---|---|---|
| Société sans écriture (utiliser la génération) | 409 | `OPENING_COMPLEMENT_NO_ENTRIES` | `error-opening-complement-no-entries` |
| Compte déjà mouvementé (nommé) | 409 | `OPENING_COMPLEMENT_ACCOUNT_MOVED` | `error-opening-complement-account-moved` |
| Compte de résultat | 400 | `OPENING_COMPLEMENT_NOT_BALANCE_ACCOUNT` | `error-opening-complement-not-balance-account` |
| Compte inconnu, d'une autre société, archivé ou non postable | 400 | `OPENING_COMPLEMENT_ACCOUNT_INVALID` | `error-opening-complement-account-invalid` |
| Compte de report saisi comme ligne | 400 | `OPENING_COMPLEMENT_RETAINED_EARNINGS_LINE` | `error-opening-complement-retained-earnings-line` |
| Aucun compte actif ne porte le rôle | 409 | `OPENING_COMPLEMENT_NO_RETAINED_EARNINGS` | `error-opening-complement-no-retained-earnings` |
| Compte de report non postable | 409 | `OPENING_COMPLEMENT_RETAINED_EARNINGS_NOT_POSTABLE` | `error-opening-complement-retained-earnings-not-postable` |
| Aucun exercice ouvert ne couvre la date | 409 | `OPENING_COMPLEMENT_NO_OPEN_FISCAL_YEAR` | `error-opening-complement-no-open-fiscal-year` |
| Date dans la période verrouillée (défensif) | 409 | `OPENING_COMPLEMENT_DATE_LOCKED` | `error-opening-complement-date-locked` |
| Aucune ligne | 400 | `OPENING_COMPLEMENT_NO_LINES` | `error-opening-complement-no-lines` |
| Trop de lignes | 400 | `OPENING_COMPLEMENT_TOO_MANY_LINES` | `error-opening-complement-too-many-lines` |
| Montant invalide, nul, négatif, ou deux colonnes | 400 | `OPENING_COMPLEMENT_INVALID_AMOUNT` | `error-opening-complement-invalid-amount` |
| Même compte sur deux lignes | 400 | `OPENING_COMPLEMENT_DUPLICATE_ACCOUNT` | `error-opening-complement-duplicate-account` |

Les quatre dernières viennent du handler (contrôles de forme, AC 4) ; les autres de la variante `kesh-db`. Les erreurs
que `create_in_tx` peut encore rendre (`FiscalYearClosed`, `DateOutsideFiscalYear`, `PeriodLocked`,
`InactiveOrInvalidAccounts`) sont **défensives** — les gardes précédentes les rendent inatteignables — et remontent par `AppError::from`. Messages dans
les **4 locales**.

**AC 6 — L'écran du mode « compléter ».** Quand `canEnter = false`, l'écran garde **toujours** le bandeau de verrou
existant (`data-testid="opening-balances-locked"`, `data-reason`, liens vers le bilan et le journal) — ce qui a été
généré reste dit, et le Playwright existant (`opening-balances.spec.ts:107-112`) reste vrai. **En dessous, et
seulement si `reason = ALREADY_HAS_ENTRIES`** (validation P4, F-4 : sous `NO_FISCAL_YEAR` ou `FIRST_YEAR_CLOSED`,
`completeReason = NO_ENTRIES` conseillerait une génération impossible ; ces bandeaux restent seuls, Vitest le prouve) :
- si `canComplete = true`, une grille de **complément** (`data-testid="opening-balances-complete"`) :
  - les seuls `completableAccounts` du status (la liste n'est **pas** recalculée côté client) ;
  - la contrepartie calculée en direct sur le compte de report (sens et montant, arbitrage 3 — y compris « aucune
    contrepartie » quand les lignes s'équilibrent) ;
  - la date prévue et sa raison (exercice d'ouverture, ou régularisation datée du jour), avec la mention qu'elle est
    confirmée à l'enregistrement ;
  - ses lignes ont leurs propres `data-testid` (`opening-balances-complete-debit-{number}`, `…-credit-{number}`, numéro
    du compte, comme le code actuel `+page.svelte:322`, `:334`) — **jamais** ceux de la grille d'ouverture
    (`opening-balances-debit-{number}`, `opening-balances-grid`), que le Playwright existant
    affirme absents après génération ;
  - un bouton « Compléter » (`data-testid="opening-balances-complete-submit"`) avec confirmation ;
  - après succès, rechargement : le compte complété disparaît de la liste ;
- si `canComplete = false`, un texte (`data-testid="opening-balances-complete-unavailable"`, `data-reason` =
  `completeReason`) qui dit pourquoi le complément n'est pas possible.
Le message du bandeau ne propose plus de « supprimer toutes les écritures » ; il renvoie au complément ou, à défaut,
dit pourquoi il est impossible.

**AC 7 — Tests.** Chacun aurait échoué avant le patch :
- **`kesh-db`** :
  - complément d'un compte jamais mouvementé → écriture équilibrée par la contrepartie sur 2970, sens correct (actif
    → crédit du report ; passif → débit) ;
  - **les quatre branches de l'arbitrage 2**, `today` passé en paramètre : (a) premier exercice ouvert, non verrouillé
    → date d'ouverture ; (b) premier exercice clos → aujourd'hui ; (b) premier exercice ouvert mais son premier jour
    verrouillé → aujourd'hui ; (c) aucun exercice ouvert couvrant → refus ; (c) aujourd'hui verrouillé → refus ;
  - écart nul (deux lignes qui se compensent) → aucune contrepartie ;
  - refus : compte mouvementé ; compte de résultat ; compte archivé ; compte d'une autre société ; compte de report
    saisi ; sans compte de report ; compte de report non postable ; société sans écriture ;
  - après **contre-passation** d'une écriture sur un compte → toujours mouvementé ; après **dévalidation d'une
    facture** (`invoices::unvalidate`), le compte débiteurs dont c'était le seul mouvement → redevenu complétable ;
  - **entrelacements** (patron `test_fixtures::attendre_une_requete_en_cours`, réglé sur la requête où le complément
    **attend réellement**) :
    (1) une écriture ordinaire, **lignes insérées non validées sur un compte visé** (d'un autre exercice, pour isoler
    le compte de l'exercice) → le complément attend sur la requête `accounts … FOR UPDATE` (étape 2), puis, l'écriture
    validée, refuse `ACCOUNT_MOVED` ;
    (2) une écriture ordinaire **du même exercice**, ne touchant **aucun** compte visé, figée après son verrou
    d'exercice et avant son en-tête — montage : une transaction de test prend l'exercice `FOR UPDATE`, attend que le
    complément soit vu en attente sur la requête `fiscal_years`, puis appelle `create_in_tx` (`pub`, `:187`) et
    valide → le complément attend à l'étape 3, puis **réussit** ; aucun interblocage ;
    (3) un **archivage** du compte saisi, en vol → le complément attend à l'étape 2, puis refuse `ACCOUNT_INVALID` ;
    (4) deux compléments du même compte → un seul réussit ;
    (5) **deux exercices**, une contre-passation d'une écriture de l'exercice du jour en vol (validation P5, R5-2) →
    le complément est rejoué ou la contre-passation échoue ; dans les deux cas, **aucun état incohérent** (l'écriture
    et sa contre-passation, ou ni l'une ni l'autre ; le complément au plus une fois). Montage : une transaction de
    test tient l'exercice du jour `FOR UPDATE` ; le complément est lancé et vu en attente sur la requête `fiscal_years`
    (il tient alors le premier exercice) ; la contre-passation est lancée et vue en attente ; la transaction de test
    est annulée. L'ordre d'obtention n'est pas maîtrisé, et l'assertion n'en dépend pas : elle porte sur l'**état
    final**, qu'il y ait eu interblocage ou non.
    **Mutations tuées** (validation P4) : « comptes saisis sans `FOR UPDATE` » par (1) — l'instantané s'ouvrirait
    avant la validation de l'écriture concurrente — et par (3) ; « lecture « jamais mouvementé » avant l'étape 2 » par
    (1) ; « société prise en exclusif » par (2) — l'écriture figée demande ensuite `companies` en partagé pour son
    en-tête : interblocage observé (complément rejoué ou écriture en erreur, le test l'affirme).
- **`kesh-api`** : la route (201, chaque code de l'AC 5, RBAC Consultation 403, isolation entre sociétés) ; le status
  (`canComplete` et chaque `completeReason`, `completableAccounts`).
- **Vitest** : avertissement présent / absent / non bloquant, et ses deux variantes (AC 1) ; totaux, montant à porter et
  écart restant, y compris un compte contre-nature, **et le montant à porter qui vaut 0 sur une saisie équilibrée
  sans compte de report** (AC 2, épingle la limite) ; grille de complément, contrepartie en direct, liste vide
  impossible (`canComplete` faux), verrou avec `completeReason` (AC 6).
- **Tests existants adaptés** (validation P1 C3, P4 F-5) : **les six** mocks `OpeningBalancesStatus` de
  `opening-balances-page.test.ts` — `readyStatus()`, `:181` (`NO_FISCAL_YEAR`), `:194` (`FIRST_YEAR_CLOSED`), `:209`,
  `:365`, `:399` (`ALREADY_HAS_ENTRIES`) — reçoivent les champs neufs, **obligatoires** dans le type ; l'assertion du
  bandeau de verrou est conservée (AC 6 le garde toujours) ; les tests e2e du status vérifient la nouvelle forme ; le
  Playwright `:107-112` reste vrai tel quel — le preset `with-company` n'a que cinq comptes, **sans rôle**
  (`test_fixtures.rs:126-133`, `opening-balances.spec.ts:40-43`) : après génération, `completeReason =
  NO_RETAINED_EARNINGS`, et c'est le texte `complete-unavailable` qui s'affiche sous le bandeau.
- **Playwright** (validation P4 F-1, P5 F5-10) : **avant** la génération, créer par l'API un compte 2970 de rôle
  `RetainedEarnings` (`POST /api/v1/accounts` accepte `role`, `routes/accounts.rs:37`) — il sert alors de
  contrepartie à la génération (`getGridAccounts`, `opening-balances.spec.ts:36-63`) ; générer l'ouverture sur `1000`
  et `2000` seulement, **sans** `1100 Banque CI` ; compléter 1100 ; voir la balance le porter. L'exercice seedé couvre 2020-2030 : branche (a), date 2020-01-01.
- **Mutations** tuées et nommées : garde « jamais mouvementé » retirée ; sens de la contrepartie inversé ; avertissement
  toujours masqué ; les mutations de verrou ci-dessus ; rejeu retiré (si un montage d'interblocage déterministe existe,
  sur le patron `accept_replays_the_batch_when_it_is_the_deadlock_victim`, `reconciliation_e2e.rs:4408-4425` — un
  autre montage, cité pour sa technique de transaction lestée ; sinon la limite est écrite).

**AC 8 — Le manuel et le CHANGELOG.** `user-manual.tex` :
- `:654-659` : l'avertissement du report à-nouveau, les totaux et le montant à porter — avec sa limite (il ne
  contrôle rien une fois la saisie équilibrée ; comparer aux totaux de l'ancien bilan) ;
- `:661-664` : « se verrouille **définitivement** » devient faux — l'écran passe en mode « compléter » ; le remède
  « contre-passer puis ressaisir » est remplacé par le **complément**, avec ses règles (comptes jamais mouvementés,
  date, contrepartie) ; la contre-passation reste citée pour un **montant faux** sur un compte déjà saisi, et le manuel
  dit qu'après la contre-passation de toute l'ouverture plus rien n'est complétable (arbitrage 1) ;
- `:400-402` (comptes 9000) : vérifier que le remède cité reste juste ;
- `:1682` (balance) : l'exception du complément daté du jour.
- `:667` (« datée au premier jour du premier exercice existant ») : la branche (a) et sa limite (exercice antérieur
  créé après la génération) ; `:1633` (report d'ouverture, reprise de comptabilité) : vérifier (validation P5, F5-3) ;
- en branche (a), le complément porte le **numéro le plus haut** et la **date la plus ancienne** de l'exercice : la
  numérotation cesse d'y être chronologique ; le manuel le dit (validation P5, F5-8) ;
- les bords de la date (arbitrage 2) : minuit UTC, clôture dans le désordre ;
- `README.md` § « Fonctionnalités » (`:46`) : le complément. Regardés et sans effet : `README.md:214` (feuille de route),
  `docs/i18n-glossaire.md:112`, `docs/kesh-specifications.txt` (FR62) — à reconfirmer au patch.
Greper **sans casse** (`grep -i`) les **valeurs** « définitivement », « supprimez toutes » (« delete all », « Löschen Sie alle », « elimina
tutte »), « directement dans le journal », « via le journal » (« directly in the journal », « im Journal », « nel
giornale », « tramite il giornale »), « contre-pass » dans les manuels, les **4 catalogues** et les replis Rust
(`opening_balances.rs:124` et `:286`) et Svelte (`+page.svelte:233`). PDF régénéré, contrôlé **aplati**. CHANGELOG
`[0.12.1]` : `Added` (le complément, l'avertissement, les totaux).

**AC 9 — Gardes structurelles.** La route POST neuve entre au registre des routes d'audit
(`crates/kesh-api/tests/audit_route_registry.rs`, `Traced` — l'audit vient de `create_in_tx`) : les constantes
codées en dur passent de un : `assert_eq!(LIB_ROUTES.len(), 111)` → 112 (`:464`), le nombre de routes tracées, et le
total combiné avec `TEST_ENDPOINT_ROUTES` (`:479`) — **recomptés depuis le registre**, messages d'assertion réécrits
(validation P5, F5-6). `sitesTotal` (et ses voisins `sitesNonResolus`, `relais`, `sitesGabarit`) de
`frontend/src/lib/shared/i18n-keys.test.ts` recalé sur le **relevé du test** (un `grep -o "i18nMsg("` ne donne qu'un
delta par fichier : 1717 occurrences littérales pour 1830 sites relevés), avec une note d'historique ;
`lint-i18n-ownership` vert (clés `opening-balances-*` et `error-opening-complement-*`) ; le Playwright neuf n'utilise
que des `data-testid` (garde `e2e-selecteurs-traduits`).

## Tasks / Subtasks

- [x] **T1 — `kesh-db`** (AC 4, 5, 7) : recherche du compte de rôle par société ; liste des comptes complétables ;
  choix de la date (arbitrage 2, `today` en paramètre) ; `create_opening_complement` (société `LOCK IN SHARE MODE` ; comptes `FOR UPDATE` par clé
  primaire ; compte de report `LOCK IN SHARE MODE` ; deux exercices, une ligne chacun ; lectures ordinaires « jamais
  mouvementé » et `NO_ENTRIES` ; refus dans l'ordre de l'AC 3 ; contrepartie ; `accounting::validate` ;
  `create_in_tx`) ; variante d'erreur ; tests de dépôt dont les **cinq** entrelacements de l'AC 7 ; `EXPLAIN` relevés.
- [x] **T2 — `kesh-api`** (AC 3, 4, 5, 9) : status étendu ; route `POST …/complete` ; variante d'`AppError` et ses
  codes ; messages × 4 locales ; registre des routes d'audit ; tests e2e (neufs et adaptés).
- [x] **T3 — l'écran** (AC 1, 2, 6) : avertissement, totaux, grille de complément ; types et API de la feature ;
  Vitest (neufs et adaptés) ; `sitesTotal` ; messages de verrou réécrits × 4 locales et replis.
- [x] **T4 — Playwright** (AC 7) : parcours de complément ; spec existante adaptée.
- [ ] **T5 — manuel, CHANGELOG** (AC 8) ; issue du TOCTOU préexistant (Limites), ouverte avec l'accord de Guy et
  citée à la PR.
- [ ] **T6 — gates** : backend complet (repositories `kesh-db` : gate complet même en cours de boucle), frontend
  complet, **E2E complet au dernier commit de code** (décision D7).

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Ne pas verrouiller `companies` en exclusif**, ni **après** les exercices : le premier forme un cycle avec une
  écriture sans projet, le second avec une écriture taguée et la génération. Suivre l'ordre de l'AC 4.
- ⛔ **Ne pas réutiliser `find_open_covering_date`** sous verrou : il verrouille tous les exercices qu'il parcourt.
- ⛔ **Ne pas intercaler de lecture ordinaire entre les verrous** : elle ouvrirait l'instantané avant eux.
- ⛔ **Ne pas verrouiller une ligne d'écriture** (`journal_entries … LOCK IN SHARE MODE`) pour `NO_ENTRIES` : cycle avec la
  contre-passation et la dévalidation.
- ⛔ **Ne pas lire « jamais mouvementé » avant le verrou des comptes** : l'instantané s'ouvrirait avant la validation
  d'une écriture concurrente, et la lecture ne la verrait pas.
- **Ne pas toucher au mode « ouverture »** : `create_opening_entry`, ses gardes et ses tests restent tels quels.
- **Ne pas recalculer côté client** la liste des comptes complétables ni la date.
- **Ne pas bloquer** sur l'avertissement de l'AC 1.
- **Ne pas créer d'action d'audit neuve** (arbitrage 4) — sinon `audit_label_registry` l'exigerait × 4 locales.
- **Ne pas émettre de `DbError::Invariant`** pour un refus métier : 500 garantie.

### Limites assumées

- **TOCTOU préexistant du mode « ouverture »** : le refus des comptes de résultat (`opening_balances.rs:300-319`) lit
  les types hors du verrou de `create_opening_entry`. Fenêtre étroite (changer le type d'un compte pendant une
  génération), antérieure à la story, **hors périmètre** — à tracer en issue.
- **Un montant faux** sur un compte déjà saisi ne se corrige pas par le complément (le compte est mouvementé) : une
  écriture de correction manuelle.
- **Après contre-passation de toute l'ouverture**, plus aucun compte de l'ouverture n'est complétable (arbitrage 1).
- **La date du status est indicative** (arbitrage 2).
- **Cycles résiduels** (AC 4, section « Cycles ») : par un compte visé ; par les exercices avec la contre-passation et
  `delete_in_tx` ; par la dernière écriture de l'exercice ; à trois par la file d'attente — les trois derniers de
  classes déjà présentes. La victime autre que le complément rend une 500, comme aujourd'hui entre flux existants.
- **`NO_ENTRIES` non verrouillé** : si l'unique écriture de la société est dévalidée pendant le complément, celui-ci
  passe et devient la première écriture. Cas sans dommage comptable (l'écriture reste équilibrée).
- **MariaDB ≥ 11.6** (`innodb_snapshot_isolation = ON`, `reconciliation.rs:1692-1696`) : sans effet ici : les lectures
  verrouillantes qui suivent l'instantané (celles de `create_in_tx`) portent sur des lignes que seul le détenteur de
  l'exercice peut modifier, et le complément le tient depuis l'étape 3 (validation P5, R5-5/F5-1) ; l'image reste figée en 10.11
  (`docker-compose.yml:4`, `ci.yml:50`).
- **Branche (a) et ouverture** (validation P4, F-10) : « le premier jour du premier exercice » est la date de
  l'écriture d'ouverture **si elle existe** ; un exercice antérieur créé après la génération (`fiscal_years::create` ne
  contrôle que le chevauchement), ou une société qui a des écritures sans ouverture, datent le complément ailleurs.
  Écrit au manuel.

### Modules

`kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` (écran des soldes de départ) — **quatre modules de code**, sous le seuil
de découpage (+ `docs`, `CHANGELOG`, hors décompte des modules de code).

### References

- Issue [#445] ; cartographie du 2026-10-06 (agent de lecture seule) ; rétrospective `epic-25-retro-2026-10-06.md`
  (D1-D3, D6, D7).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (orchestrateur), 2026-10-06.

### Debug Log References

- `target/gate-logs/backend-257-dev.log` — gate backend complet (`scripts/test-fast.sh --ci`, base remise à zéro
  avant) : **2735 tests, 2735 passés**, 4 ignorés ; fmt et clippy `-D warnings` verts.
- Gate frontend complet : `npm run check` 0 erreur ; `lint-i18n-ownership` PASS ; `test:unit` **956/956** (104
  fichiers) ; `build` vert.
- Playwright ciblé `opening-balances.spec.ts` : **3/3**. E2E complet sur le commit de dev
  (`target/gate-logs/e2e-257-dev.log`, départ 10:07 UTC) : **238 passés, 9 échoués, 19 ignorés** — les 9 sont la
  liste attendue (`docs/testing.md`) : KF-029 ×7 (`mode-expert:26/:41`, `onboarding-path-b:65/:92`,
  `onboarding:57/:77/:150`) et KF-051 ×2 (`invoices:415/:439`, run avant 12:00 UTC) ; aucun hors liste. À rejouer au
  dernier commit de code de la revue (D7).

### Completion Notes List

- **`kesh-db`** — module neuf `repositories/opening_complement.rs` : `create_opening_complement` suit la règle de
  l'AC 4 (société `LOCK IN SHARE MODE` ; comptes saisis `FOR UPDATE` par clé primaire, filtre `company_id` ; compte de
  report `LOCK IN SHARE MODE` ; premier exercice puis candidat du jour, une ligne chacun ; lectures ordinaires « jamais
  mouvementé » et « a des écritures » ; refus dans l'ordre ; contrepartie ; `accounting::validate` ; `create_in_tx`).
  `complement_status` (lectures sans verrou) et `decide_date` (arbitrage 2, pure). Variante
  `DbError::OpeningComplementRefused { reason, account_id, account_number }` — le compte fautif est **nommé** par son
  numéro quand il appartient à la société (jamais pour un compte d'une autre société).
- **`EXPLAIN` relevés** (MariaDB 10.11.16, base de dev) : comptes saisis `range` sur `PRIMARY` ; premier exercice `ref`
  sur `uq_fiscal_years_company_start_date`, **sans** `filesort` ; candidat du jour `range` sur le même index ; compte de
  report par l'index unique `uq_accounts_company_singleton_role`. Sonde à deux sessions de la validation P6 :
  `25-7-validate-p6-lockprobe.sh`.
- **`kesh-api`** — `POST /api/v1/opening-balances/complete` (`comptable_routes`), contrôles de forme dans le handler,
  appel enveloppé dans `retry_with` ; status étendu (`canComplete`, `completeReason`, `completableAccounts`,
  `complementDate`, `complementFiscalYear`, `retainedEarningsAccount`).
  **Écarts à la fiche, assumés** : (1) un champ `complementDateKind` (`OPENING_DAY` / `TODAY`) s'ajoute au status —
  l'AC 6 demande d'afficher « la date prévue **et sa raison** », que la date seule ne donne pas ; (2) les refus de
  `kesh-db` sont mappés dans le `match` global de `AppError::Database` (statut et code dérivés de la raison, comme
  `EntryIsPosted`), et les refus de forme du handler par une variante `AppError::OpeningComplementInvalid` — l'AC 5
  envisageait une variante unique, le contrat (code par cause, statut de la table) est le même.
- **i18n** — 43 clés neuves × 4 locales (`opening-balances-*` de l'écran, `opening-balances-complement-description`,
  13 `error-opening-complement-*`) ; `opening-balances-locked-already-has-entries` et
  `error-opening-balances-already-has-entries` réécrits × 4 locales, replis Rust (`opening_balances.rs`, deux sites) et
  Svelte compris.
- **Écran** — calculs dans `opening-balances-totals.ts` (pur, big.js) : totaux et montant à porter (AC 2),
  avertissement à trois variantes (AC 1), contrepartie du complément (AC 6). Section de complément **seulement** sous
  `ALREADY_HAS_ENTRIES`. `sitesTotal` 1830 → **1864** (relevé du test ; `grep -o` aux deux bornes : 26 → 60).
- **Tests** (périmètre `HEAD` de validation → commit de dev) : `kesh-db` **22** d'intégration neufs + **7** unitaires ;
  `kesh-api` e2e 21 → **26**, unitaires 3 → **5** ; Vitest page 19 → **33**, calculs **6** neufs ; Playwright 2 →
  **3**. Le double d'`i18nMsg` du test de page interpole désormais ses variables.
- **Mutations jouées et tuées** (chacune restaurée, octet vérifié par `cmp`) : comptes saisis sans `FOR UPDATE` → (1)
  et (3) ; garde « jamais mouvementé » retirée → `refus_compte_mouvemente` ; sens de la contrepartie inversé →
  `la_contrepartie_va_au_report_dans_le_bon_sens` ; société prise en exclusif → (2), **interblocage 1213 observé** ;
  lecture « jamais mouvementé » avant le verrou des comptes → (1) ; avertissement toujours masqué → 3 tests Vitest ;
  contrepartie inversée côté écran → 2 tests Vitest. **Non jouée** : « rejeu retiré » — aucun montage d'interblocage
  déterministe dont le complément soit la victime (l'entrelacement (5) accepte les deux issues) ; limite écrite.
- **Reste** : l'issue du TOCTOU préexistant (Limites, T5) attend l'accord de Guy.

### File List

- `crates/kesh-db/src/repositories/opening_complement.rs` (neuf)
- `crates/kesh-db/src/repositories/mod.rs`
- `crates/kesh-db/src/errors.rs`
- `crates/kesh-db/tests/opening_complement_repository.rs` (neuf)
- `crates/kesh-api/src/routes/opening_balances.rs`
- `crates/kesh-api/src/errors.rs`
- `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/tests/opening_balances_e2e.rs`
- `crates/kesh-api/tests/audit_route_registry.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `frontend/src/lib/features/opening-balances/opening-balances.types.ts`
- `frontend/src/lib/features/opening-balances/opening-balances.api.ts`
- `frontend/src/lib/features/opening-balances/opening-balances-totals.ts` (neuf)
- `frontend/src/lib/features/opening-balances/opening-balances-totals.test.ts` (neuf)
- `frontend/src/routes/(app)/settings/opening-balances/+page.svelte`
- `frontend/src/routes/(app)/settings/opening-balances/opening-balances-page.test.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/tests/e2e/opening-balances.spec.ts`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `CHANGELOG.md`, `README.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- **2026-10-06** — Développement (`bmad-dev-story`, Opus 5.5) : T1 à T4 et le manuel / CHANGELOG / README de T5 ;
  gates backend 2735/2735, frontend 956/956, E2E complet 238 passés et 9 échecs tous attendus. Deux écarts à la fiche,
  assumés (Completion Notes). Reste : l'issue du TOCTOU (accord de Guy).

- **2026-10-06** — Validation P7, **passe ciblée** (Haiku, prompt `25-7-validate-prompt-p7-ciblee.md`, braquée sur
  `0a8e2ae6..c6819583`) : **0 finding**, axes déclarés. Vérifié par l'orchestrateur (CLAUDE.md : un « 0 » se vérifie) :
  aucun résidu `FOR SHARE` hors Change Log, aucun « quatre entrelacements » ; `accounting::validate` existe
  (`kesh-core/src/accounting/balance.rs:150`, `JournalEntryDraft → BalancedEntry`). **Un écart** : la passe déclare
  l'entrelacement (5) « testé avec un montage d'attente » alors que la fiche n'en décrivait aucun — montage écrit (LOW).
  **Validation close** : P1 1 C / 4 H → P2 4 H → P3 2 H / 6 M → P4 3 H / 6 M → P5 1 H / 3 M → P6 1 M (mesurée) → P7 0
  (+ 1 LOW de l'orchestrateur). Modèles : Sonnet ×3, Opus ×2, Sonnet ×2, Opus ×2, Sonnet ×2, mesure, Haiku ciblé.
  Signal de recyclage (D5) levé en P4, déclaré ; traité par un changement de méthode (AC 4 par règle et propriétés,
  puis mesure sur la base) plutôt que par découpage, la zone tenant dans une fonction `kesh-db`.

- **2026-10-06** — Validation P6 (orchestrateur, **mesurée** et non lue : les requêtes de l'AC 4 exécutées sur
  `kesh-mariadb-dev` 10.11.16, `EXPLAIN` puis transaction annulée) : **1 MEDIUM**, retenu. Syntaxe de toutes les
  requêtes vérifiée ; comptes saisis par `PRIMARY` en `range` ; compte de report par l'index unique ; exercice du jour
  par `uq_fiscal_years_company_start_date` en `range`, une ligne ; **le premier exercice, avec le départage `, id`,
  passait par `Using filesort`** — toutes les lignes de la société lues sous verrou — corrigé en `ORDER BY start_date`,
  plan sans tri vérifié. **Sonde à deux sessions** (`25-7-validate-p6-lockprobe.sh`, versionnée : quatre exercices
  2024-2027 insérés puis supprimés ; une session tient la requête, l'autre tente chaque exercice en `FOR UPDATE
  NOWAIT`) : premier exercice `ORDER BY start_date` → **1** verrouillé ; avec `, id` → **4** ; candidat du jour
  `DESC` → **1** ; `find_open_covering_date` → **3** (du premier à celui du jour). Les constats R4-1 et F-2 de P4,
  jusque-là déduits, sont **établis**.

- **2026-10-06** — Validation P5 (Sonnet ×2, prompt `25-7-validate-prompt-p5.md`) : **1 HIGH, 3 MEDIUM, ~10 LOW**,
  tous retenus.
  - **HIGH** : `FOR SHARE` est une erreur de syntaxe sur MariaDB 10.11 (R5-1, **vérifié** : `ERROR 1064` sur
    `kesh-mariadb-dev` 10.11.16, `LOCK IN SHARE MODE` passe) → `LOCK IN SHARE MODE` partout. Né de la remédiation P3.
  - **MEDIUM** : cycle par les exercices avec la contre-passation (R5-2) et par la dernière écriture de l'exercice,
    verrouillée par le plancher du compteur de `create_in_tx` (F5-1) → la propriété (iii) ne prétend plus que tout
    cycle passe par un compte visé ; les classes préexistantes sont nommées, l'entrelacement (5) vérifie l'absence
    d'état incohérent ; T1 contredisait l'AC 4 (F5-2).
  - **LOW** : file d'attente à trois, fenêtre élargie (R5-3) ; cycle entre deux compléments par le compte de report et
    verrou inter-sociétés → filtre `company_id` (R5-4, F5-9) ; justification 11.6 erronée (R5-5) ; ordre des refus par
    compte (R5-6) ; manuel `:667`, `:1633` (F5-3) ; chemins de crate (F5-4, non corrigés un à un : les `fichier:ligne`
    se relisent au développement) ; `EXPLAIN` des requêtes d'exercice (F5-5) ; constantes de l'AC 9 (F5-6) ;
    `accounting::validate` (F5-7) ; numérotation non chronologique (F5-8) ; ordre du Playwright (F5-10).
  - Trend : P1 1 C / 4 H → P2 4 H → P3 2 H / 6 M → P4 3 H / 6 M → P5 1 H / 3 M. La sévérité **décroît** ; le HIGH de P5
    est un fait de syntaxe vérifiable, qu'aucune lecture ne pouvait trancher — d'où la passe suivante : **les requêtes
    de l'AC 4 exécutées contre la base de dev** (syntaxe, `EXPLAIN`, verrous posés), en lecture et en transaction
    annulée.

- **2026-10-06** — Validation P4 (Opus ×2 : R = ordre des verrous de chaque transaction concurrente, F = périmètre
  complet ; prompt `25-7-validate-prompt-p4.md`) : **3 HIGH, 6 MEDIUM, 12 LOW**, tous retenus.
  - **HIGH** : `find_open_covering_date … FOR UPDATE` verrouille tous les exercices parcourus (`end_date`, `status`
    hors index) → cycle avec toute contre-passation (R4-1, F-2) ; l'entrelacement (2) pouvait ne tuer aucune mutation
    (R4-2) ; le preset Playwright n'a ni 2970 ni rôle, le parcours prescrit était impossible (F-1, vérifié
    `test_fixtures.rs:126-133`).
  - **MEDIUM** : cycle avec le règlement et la passation en perte, qui prennent le compte puis l'exercice (R4-3) ;
    instantané ouvert avant des verrous, relu par `create_in_tx` (R4-4) ; refus de date avant `NO_ENTRIES` (F-3) ;
    section de complément sous des bandeaux où la génération est impossible (F-4) ; six mocks à adapter, non trois
    (F-5).
  - **LOW** : verrous d'intervalle (R4-5, disparus) ; plan de l'étape des comptes (R4-6) ; ordre des refus distinct de
    celui des lectures (R4-7) ; paramètre `actor_api_key_id` sans destination (F-6) ; grep sans casse (F-7) ;
    `data-testid` par numéro (F-8) ; statut HTTP de la variante (F-9) ; branche (a) sans ouverture (F-10) ; issue du
    TOCTOU (F-11, en T5) ; montage de l'entrelacement (F-12) ; précisions de contrat dont l'échelle décimale (F-13) ;
    mise en forme (F-14).
  - **Remède de principe** : l'AC 4 ne cherche plus un ordre sans cycle, qui n'existe pas dans ce dépôt ; il suit une
    règle — tous les verrous d'abord, une ligne par requête ; les lectures ordinaires ensuite ; les refus en dernier —
    et garantit trois propriétés indépendantes de l'ordre. Les cycles résiduels passent tous par un compte visé.
  - Trend : P1 1 C / 4 H → P2 4 H → P3 2 H / 6 M → P4 3 H / 6 M. ⚠️ **Signal de découpage (D5) : recyclage.** Trois
    passes de suite, les défauts graves viennent de la remédiation précédente, dans la même zone (l'ordre des
    verrous). Déclaré à Guy. La zone est confinée à une fonction `kesh-db` ; le remède change de méthode plutôt que de
    retoucher l'ordre une quatrième fois. Passe P5 : complète sur l'AC 4 et l'AC 7.

- **2026-10-06** — Validation P3 (Sonnet ×2 : R = la remédiation `e25e8bb4`, C = complétude ; prompt
  `25-7-validate-prompt-p3.md`) : **2 HIGH, 6 MEDIUM, ~8 LOW**, tous retenus, vérifiés au code.
  - **HIGH** : `NO_ENTRIES` en `FOR SHARE` sur une ligne d'écriture → cycle avec la contre-passation et la
    dévalidation, qui verrouillent l'écriture puis l'exercice (C-H1) → lecture ordinaire ; l'exercice « du jour »
    verrouillé selon une branche décidée **après** la société → cycle (C-H2) → les deux exercices toujours verrouillés,
    la branche choisie ensuite.
  - **MEDIUM** : l'ordre « exercices d'abord » de P2 contredit l'ordre global du dépôt (`companies → projects →
    fiscal_years`, `journal_entries.rs:256`) et cycle avec toute écriture **taguée projet**, qui prend la sentinelle
    `companies` en exclusif (R-1, vérifié `projects.rs:99`, `bank_accounts.rs:592`) → **société en partagé, en
    premier** ; entrelacement (2) attendant à l'étape 5 au lieu de l'étape des comptes (R-2, C-M1) ; compte de report
    en exclusif → cycle avec les lignes ordinaires sur 2970 (R-3) → partagé, cycle résiduel sur deux comptes saisis
    écrit ; Dev Note inversée (R-4) ; entrelacement (1) sans point d'arrêt et mutation non tuable (C-M2) ; absence
    d'exercice au POST (C-M3) → `NO_ENTRIES`.
  - **LOW** : T1 « deux » entrelacements (R-5) ; citations (`crates/kesh-db/src/errors.rs:781-813`,
    chemins complets, `delete_in_tx`, « côté utilisateur ») (R-6, C-L2) ; verrous next-key (R-7) ; priorité de
    `completeReason` selon la branche (R-8) ; repli `opening_balances.rs:286` (C-L1) ; docs regardées (C-L3). Non
    retenu en patch : liste des clés i18n de l'écran (C-L4), comptées par `sitesTotal` et `lint-i18n-ownership`.
  - Trend : P1 1 CRITICAL / 4 HIGH → P2 4 HIGH → P3 2 HIGH / 6 MEDIUM. **Tous les défauts de P3 viennent de la
    remédiation P2** (l'ordre des verrous), aucun de la conception d'origine. Pas de signal de découpage (D5 : défauts
    distincts, non recyclés). La remédiation réécrit encore l'AC 4 : passe P4 complète.

- **2026-10-06** — Validation P2 (Opus ×2 : R = la remédiation `bede9278`, F = périmètre complet ; prompt
  `25-7-validate-prompt-p2.md`) : **4 HIGH, ~10 MEDIUM, ~10 LOW**, tous retenus. **Les HIGH viennent de la remédiation
  de P1** :
  - **F1 réfute la prémisse du CRITICAL B1 de P1** : une écriture ordinaire prend bien `companies` en partagé, par la
    clé étrangère `fk_journal_entries_company`, à l'insertion de son en-tête. La sentinelle sérialisait donc déjà ; le
    verrou des comptes protège contre une modification du compte, pas contre une écriture.
  - **F2 / R-1** : l'ordre de P1 (société exclusive → comptes → exercice) croisait celui d'une écriture ordinaire
    (exercice → société) : interblocage dans le cas courant, en 500 ; le test prescrit l'évitait. Corrigé : l'AC 4 suit
    l'ordre d'une écriture ordinaire (exercices → société en partagé → comptes), plus un rejeu défensif.
  - **F3 / R-2** : les mutations de verrou prescrites étaient intuables. Remplacées par celles que l'architecture rend
    tuables, chacune avec son entrelacement.
  - **MEDIUM** : `find_first_by_company` hors transaction (F4, R-3) ; les écritures « brouillon » n'existent pas, seule
    la dévalidation d'une facture détruit une écriture (F5, R-4) ; `NO_ENTRIES` non placé (F6, R-3) ; le « report
    attendu » de l'AC 2 vaut 0 par construction quand la saisie est équilibrée (F7) → renommé, limite dite ; « via le
    journal » ×4 locales et replis (F8) ; priorité des `completeReason` (R-5) ; contrôles de forme impossibles à
    rapporter au mapping de la génération (R-6) → dans le handler, avec codes, dont `DUPLICATE_ACCOUNT`.
  - **LOW** : README (F9) ; `company_id` au verrou des comptes (F10, L-3) ; `DATE_LOCKED` défensif (F11) ; minuit UTC et
    clôture dans le désordre au manuel (F12) ; numéros de ligne Svelte (L-1) ; `today_utc` n'est pas une fonction (L-2) ;
    « au moins deux lignes » automatique (L-4) ; erreurs de `create_in_tx` défensives (L-5) ; `data-testid` de la grille
    de complément (L-6) ; motif de l'audit (L-7) ; `MAX_LINES_PER_ENTRY` importé de `routes::journal_entries`.
  - Trend : P1 1 CRITICAL / 4 HIGH → P2 4 HIGH (nés de la remédiation P1). Sévérité décroissante, défauts nés du
    correctif : pas de signal de découpage (D5) ; passe suivante complète (la remédiation réécrit l'AC 4).
- **2026-10-06** — Validation P1 (Sonnet ×3, prompt `25-7-validate-prompt-p1.md`) : **1 CRITICAL, 4 HIGH, ~10 MEDIUM**,
  LOW, tous retenus.
  - **CRITICAL B1** (convergent avec A1 HIGH et C5) : la sentinelle `companies` ne sérialise pas le complément contre
    une écriture ordinaire (qui ne la prend pas, ne verrouille pas `accounts`, et ne verrouille que son exercice) ;
    « jamais mouvementé » lu « sous ce verrou » laissait passer une écriture en vol. Corrigé : verrou `FOR UPDATE` des
    comptes saisis — l'insertion d'une ligne prend un verrou partagé sur le compte parent par la clé étrangère
    `fk_jel_account`, vérifiée au schéma — puis lecture verrouillante ; deux entrelacements prescrits.
  - **HIGH** : les refus neufs auraient fini en 500 par `DbError::Invariant` (B2, A3) → table de codes et variante
    d'erreur (AC 5) ; écart nul laissé ouvert (B3, C1) → aucune contrepartie, au moins deux lignes (arbitrage 3) ; date
    du status et du POST divergentes, bords non définis (B4, C4) → date UTC en paramètre, recalculée sous verrou,
    status indicatif, quatre branches testées.
  - **MEDIUM** : contre-passée / supprimée / archivée (B5) ; formule des totaux et du report attendu (B6) ; variantes
    de l'avertissement (B7) ; `canComplete` vrai sans compte complétable (B8) → `completeReason` et
    `NO_COMPLETABLE_ACCOUNT` ; messages de verrou périmés × 4 locales et replis, qui conseillent une suppression
    impossible (C2, A5) ; tests existants à adapter (C3) ; numéros de ligne de `create_in_tx` périmés (A2) ; écarts à
    l'issue non dits (C7) → écrits dans les arbitrages 1 et 2.
  - **LOW** : recomptage de `sitesTotal` par le relevé du test (A4) ; registre d'audit, totaux qui bougent (A4, C6) ;
    `:1682` du manuel (A5) ; « définitivement » au grep (C8) ; champ `role` côté client confirmé (C9) ; compte de report
    cherché par société (B9) ; plafond (B10) ; décompte des modules (C10).
- **2026-10-06** — Créée (rétrospective de l'Epic 25, décision D1). Cinq arbitrages proposés, retenus par défaut.

[#445]: https://github.com/guycorbaz/kesh/issues/445
