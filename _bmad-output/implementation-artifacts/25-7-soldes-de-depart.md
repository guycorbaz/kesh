# Story 25.7 : Soldes de départ — avertir si le report à-nouveau manque, compléter un compte oublié

Status: ready-for-dev

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
    détruisent aussi des écritures. Le compte
     redevient alors complétable s'il n'avait pas d'autre mouvement ;
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
     l'exercice **ouvert** qui couvre cette date (`find_open_covering_date`) — une régularisation, comme le dit
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
  fournisseur — prend d'abord la sentinelle `companies … FOR UPDATE` (`projects.rs:99` →
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
ne s'évaluent qu'en branche (b), validation P3 R-8) : `NO_ENTRIES` (y compris aucun exercice : rien à compléter) >
`NO_OPEN_FISCAL_YEAR` > `DATE_LOCKED` >
`NO_RETAINED_EARNINGS` > `RETAINED_EARNINGS_NOT_POSTABLE` > `NO_COMPLETABLE_ACCOUNT` > `READY`. Lecture sans verrou :
le POST fait autorité.

**AC 4 — La route de complément.** `POST /api/v1/opening-balances/complete`, corps `{ lines: [{ accountId, debit,
credit }] }` — **sans** la contrepartie.

**Dans le handler**, avant toute transaction (contrôles de forme, sans lecture de la base) : au moins une ligne ;
plafond `MAX_LINES_PER_ENTRY − 1` (une ligne réservée à la contrepartie) ; montants parsés en `Decimal`, strictement
positifs, une seule colonne par ligne ; comptes distincts. Chacun a son code (AC 5).

**Dans `kesh-db`** — fonction nouvelle, p. ex. `journal_entries::create_opening_complement(pool, company_id, user_id,
actor_api_key_id, lines, description, today)` —, **une seule transaction**, dans cet ordre, qui est **l'ordre global
du dépôt** (`companies → fiscal_years`, Pattern 5), la société étant prise **en partagé** (validation P3, R-1/H1/H2) :
1. **la société, en partagé** : `SELECT books_locked_through FROM companies WHERE id = ? FOR SHARE` — compatible
   avec le partagé que prend l'en-tête d'une écriture sans projet ; attend (sans rien tenir) une écriture taguée, une
   génération d'ouverture ou un `lock_books` en vol, qui la prennent en exclusif ; la borne est **figée** jusqu'au
   commit ;
2. **les exercices, toujours les deux** (validation P3, H2) : le premier exercice (variante en transaction,
   `ORDER BY start_date, id LIMIT 1 FOR UPDATE` ; aucun → refus `NO_ENTRIES` : sans exercice, il n'y a rien à
   compléter), **puis** l'exercice ouvert qui couvre `today` (`find_open_covering_date`, `FOR UPDATE` ; un seul verrou
   s'il est le premier), dans l'ordre des dates de début — **avant** de choisir la branche. La **date** se décide
   ensuite (arbitrage 2), avec la borne de l'étape 1 : jamais un exercice verrouillé après coup ;
3. **`NO_ENTRIES`**, par une lecture **ordinaire** (`SELECT EXISTS (SELECT 1 FROM journal_entries WHERE company_id =
   ?)`), **non verrouillante** (validation P3, H1) : un `FOR SHARE` sur une ligne d'écriture formerait un cycle avec
   la contre-passation et `delete_in_tx` (dévalidation), qui verrouillent l'écriture **puis** l'exercice
   (`journal_entries.rs:1019`, `:1530`). La génération d'ouverture ne peut pas être en vol (étape 1). ⚠️ Cette lecture
   **ouvre l'instantané** de la transaction (REPEATABLE READ) : toute lecture qui suit et doit voir le dernier état
   est donc **verrouillante** ;
4. **les comptes saisis et le compte de report** :
   - comptes saisis : `SELECT … FROM accounts WHERE company_id = ? AND id IN (…) ORDER BY id FOR UPDATE` — chacun de
     la société (un identifiant absent du résultat → `ACCOUNT_INVALID`), actif, postable, de bilan
     (`Asset`/`Liability`), et ne portant **pas** le rôle `RetainedEarnings` ; l'exclusif est **nécessaire** : il
     arrête la première écriture concurrente sur le compte, dont les lignes demandent un partagé (clé étrangère) ;
   - compte de report, de la société : `… WHERE company_id = ? AND singleton_role = 'RetainedEarnings' FOR SHARE` —
     **partagé** (validation P3, R-3) : il suffit à le figer contre un archivage, et il reste compatible avec le
     partagé que prennent les lignes d'une écriture ordinaire sur 2970 ; présent et postable, sinon refus ;
5. **« jamais mouvementé »**, par une lecture **verrouillante** : `SELECT account_id FROM journal_entry_lines WHERE
   account_id IN (…) LIMIT 1 FOR SHARE`. Une écriture **non validée** sur ces comptes a déjà été attendue à l'étape 4
   (son partagé de clé étrangère contre l'exclusif) ; ce verrou est donc là pour **voir** celle qui a été validée
   pendant cette attente — une lecture ordinaire lirait l'instantané de l'étape 3 et la manquerait ;
6. **la contrepartie** (arbitrage 3) : écart = Σ débit − Σ crédit des lignes saisies ; > 0 → une ligne **au crédit**
   du compte de report de ce montant ; < 0 → **au débit** ; = 0 → aucune ligne ;
7. `create_in_tx(…, enforce_postable = true)` — exercice clos, bornes, période verrouillée, audit
   `journal_entry.created`.

**Ce qui sérialise** (et que les tests prouvent) : deux compléments → l'exercice de l'étape 2 ; un complément et une
écriture ordinaire **du même exercice** → ce même exercice ; **d'un autre exercice**, lignes insérées non validées →
l'exclusif de l'étape 4 contre le partagé de clé étrangère, puis la lecture verrouillante de l'étape 5 ; une
génération d'ouverture, une écriture taguée, `lock_books` → l'étape 1 ; un archivage du compte → l'étape 4.

**Cycles : ceux qui restent, et pourquoi on les garde** (validation P3, R-1/R-3/H1) :
- avec la génération, une écriture taguée, `lock_books`, la contre-passation, la dévalidation : **aucun** cycle à deux
  dans l'ordre ci-dessus ;
- **à trois, par la file d'attente** : InnoDB fait attendre une demande partagée derrière une demande exclusive
  **en attente**. Si une écriture taguée attend `companies` en exclusif pendant que le complément la tient en partagé,
  une écriture sans projet qui tient l'exercice voulu par le complément attend derrière elle. C'est le cycle **déjà
  présent** entre écritures taguées et non taguées ; le complément n'en crée pas de nouvelle classe ;
- **sur les comptes** : une écriture ordinaire d'un autre exercice qui mouvemente, au même instant, **deux** comptes
  que le complément veut compléter, dans l'ordre inverse de leurs `id`. Il faut deux comptes jamais mouvementés,
  touchés pour la première fois par une même écriture, pendant le complément : le cas est étroit, et il est **dit**.

**Rejeu** : la route s'enveloppe dans `kesh_db::retry::retry_with` sur `is_deadlock_error` (précédents
`onboarding.rs:614`, `invoices.rs:1343`, `reconciliation.rs:850`). Il ne protège que le complément : si InnoDB choisit
l'autre transaction pour victime, celle-ci rend une 500, comme le font déjà aujourd'hui les cycles entre écritures
ordinaires. Le rejeu est sûr, chaque tentative refaisant toutes les gardes.

Réponse `201 + JournalEntryResponse`, journal `OD`, libellé « Complément des soldes de départ » (clé i18n, langue
comptable de la société).

**AC 5 — Les erreurs.** Une variante de `DbError` portant une énumération de refus (p. ex.
`DbError::OpeningComplementRefused { reason, account_id }`), mappée vers une variante d'`AppError` à **code par cause**
(patron `errors.rs:939`, `error_code: &'static str`) — ⛔ **jamais** un `DbError::Invariant` non mappé, qui
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
que `create_in_tx` peut encore rendre (`FiscalYearClosed`, `DateOutsideFiscalYear`, `PeriodLocked`) sont
**défensives** — les gardes précédentes les rendent inatteignables — et remontent par `AppError::from`. Messages dans
les **4 locales**.

**AC 6 — L'écran du mode « compléter ».** Quand `canEnter = false`, l'écran garde **toujours** le bandeau de verrou
existant (`data-testid="opening-balances-locked"`, `data-reason`, liens vers le bilan et le journal) — ce qui a été
généré reste dit, et le Playwright existant (`opening-balances.spec.ts:107-112`) reste vrai. **En dessous** :
- si `canComplete = true`, une grille de **complément** (`data-testid="opening-balances-complete"`) :
  - les seuls `completableAccounts` du status (la liste n'est **pas** recalculée côté client) ;
  - la contrepartie calculée en direct sur le compte de report (sens et montant, arbitrage 3 — y compris « aucune
    contrepartie » quand les lignes s'équilibrent) ;
  - la date prévue et sa raison (exercice d'ouverture, ou régularisation datée du jour), avec la mention qu'elle est
    confirmée à l'enregistrement ;
  - ses lignes ont leurs propres `data-testid` (`opening-balances-complete-debit-{n}`, `…-credit-{n}`) — **jamais**
    ceux de la grille d'ouverture (`opening-balances-debit-{n}`, `opening-balances-grid`), que le Playwright existant
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
    (1) une écriture **ordinaire du même exercice, sans projet**, en vol (figée par une transaction tenue à la main
    **après** son verrou d'exercice et **avant** son en-tête, montage de `reconciliation_e2e.rs:4408-4425`) → le
    complément attend sur l'exercice (étape 2), puis refuse `ACCOUNT_MOVED` — **aucun interblocage** ;
    (2) une écriture ordinaire d'**un autre exercice**, lignes insérées non validées → le complément attend à
    l'**étape 4** (exclusif contre le partagé de clé étrangère), puis, l'écriture validée, refuse `ACCOUNT_MOVED` par
    la lecture verrouillante de l'étape 5 ;
    (3) un **archivage** du compte saisi, en vol → le complément attend à l'étape 4, puis refuse `ACCOUNT_INVALID` ;
    (4) deux compléments du même compte → un seul réussit.
    **Mutations tuées** (validation P2 F3/R-2, révisées en P3 R-2/M1/M2) : « comptes saisis sans `FOR UPDATE` » par
    (3) (en (2), la lecture de l'étape 5 attendrait encore les lignes non validées) ; « étape 5 sans `FOR SHARE` » par (2) — l'instantané, ouvert à l'étape 3 avant l'attente, ne voit pas
    l'écriture validée pendant celle-ci ; « société prise **après** les exercices » par (1) n'est **pas** promise :
    elle ne cycle qu'avec une écriture taguée, dont la victime n'est pas déterministe — limite écrite, sans test.
- **`kesh-api`** : la route (201, chaque code de l'AC 5, RBAC Consultation 403, isolation entre sociétés) ; le status
  (`canComplete` et chaque `completeReason`, `completableAccounts`).
- **Vitest** : avertissement présent / absent / non bloquant, et ses deux variantes (AC 1) ; totaux, montant à porter et
  écart restant, y compris un compte contre-nature, **et le montant à porter qui vaut 0 sur une saisie équilibrée
  sans compte de report** (AC 2, épingle la limite) ; grille de complément, contrepartie en direct, liste vide
  impossible (`canComplete` faux), verrou avec `completeReason` (AC 6).
- **Tests existants adaptés** (validation P1, C3) : les mocks Vitest de `ALREADY_HAS_ENTRIES` (`:206-216`, `:360-384`,
  `:400-419`) reçoivent les champs neufs (`canComplete: false`, `completeReason`) ; leur assertion du bandeau de verrou
  est conservée (AC 6 le garde toujours) ; les tests e2e du status vérifient la nouvelle forme ; le Playwright
  `:107-112` reste vrai tel quel (le bandeau reste affiché ; la société `with-company` porte 2970 et des comptes non
  mouvementés : la grille de complément apparaît en plus, sans casser l'assertion).
- **Playwright** : générer l'ouverture **sans** un compte de bilan (p. ex. 1020), puis le compléter, et voir la balance
  le porter.
- **Mutations** tuées et nommées : garde « jamais mouvementé » retirée ; sens de la contrepartie inversé ; avertissement
  toujours masqué ; les mutations de verrou ci-dessus ; rejeu retiré (si un montage d'interblocage déterministe existe,
  sur le patron `accept_replays_the_batch_when_it_is_the_deadlock_victim`, `reconciliation_e2e.rs:4408-4425` ; sinon la
  limite est écrite).

**AC 8 — Le manuel et le CHANGELOG.** `user-manual.tex` :
- `:654-659` : l'avertissement du report à-nouveau, les totaux et le montant à porter — avec sa limite (il ne
  contrôle rien une fois la saisie équilibrée ; comparer aux totaux de l'ancien bilan) ;
- `:661-664` : « se verrouille **définitivement** » devient faux — l'écran passe en mode « compléter » ; le remède
  « contre-passer puis ressaisir » est remplacé par le **complément**, avec ses règles (comptes jamais mouvementés,
  date, contrepartie) ; la contre-passation reste citée pour un **montant faux** sur un compte déjà saisi, et le manuel
  dit qu'après la contre-passation de toute l'ouverture plus rien n'est complétable (arbitrage 1) ;
- `:400-402` (comptes 9000) : vérifier que le remède cité reste juste ;
- `:1682` (balance) : l'exception du complément daté du jour.
- les bords de la date (arbitrage 2) : minuit UTC, clôture dans le désordre ;
- `README.md` § « Fonctionnalités » (`:46`) : le complément. Regardés et sans effet : `README.md:214` (feuille de route),
  `docs/i18n-glossaire.md:112`, `docs/kesh-specifications.txt` (FR62) — à reconfirmer au patch.
Greper les **valeurs** « définitivement », « supprimez toutes » (« delete all », « Löschen Sie alle », « elimina
tutte »), « directement dans le journal », « via le journal » (« directly in the journal », « im Journal », « nel
giornale », « tramite il giornale »), « contre-pass » dans les manuels, les **4 catalogues** et les replis Rust
(`opening_balances.rs:124` et `:286`) et Svelte (`+page.svelte:233`). PDF régénéré, contrôlé **aplati**. CHANGELOG
`[0.12.1]` : `Added` (le complément, l'avertissement, les totaux).

**AC 9 — Gardes structurelles.** La route POST neuve entre au registre des routes d'audit
(`crates/kesh-api/tests/audit_route_registry.rs`, `Traced` — l'audit vient de `create_in_tx`) : `LIB_ROUTES.len()` et
le nombre de routes tracées gagnent un, et le total combiné aussi — **recomptés par le test lui-même**, messages
d'assertion réécrits. `sitesTotal` (et ses voisins `sitesNonResolus`, `relais`, `sitesGabarit`) de
`frontend/src/lib/shared/i18n-keys.test.ts` recalé sur le **relevé du test** (un `grep -o "i18nMsg("` ne donne qu'un
delta par fichier : 1717 occurrences littérales pour 1830 sites relevés), avec une note d'historique ;
`lint-i18n-ownership` vert (clés `opening-balances-*` et `error-opening-complement-*`) ; le Playwright neuf n'utilise
que des `data-testid` (garde `e2e-selecteurs-traduits`).

## Tasks / Subtasks

- [ ] **T1 — `kesh-db`** (AC 4, 5, 7) : recherche du compte de rôle par société ; liste des comptes complétables ;
  choix de la date (arbitrage 2, `today` en paramètre) ; `create_opening_complement` (société en partagé, exercices, comptes
  `FOR UPDATE`, « jamais mouvementé » verrouillant, contrepartie, `create_in_tx`) ; variante d'erreur ; tests de
  dépôt dont les **quatre** entrelacements de l'AC 7.
- [ ] **T2 — `kesh-api`** (AC 3, 4, 5, 9) : status étendu ; route `POST …/complete` ; variante d'`AppError` et ses
  codes ; messages × 4 locales ; registre des routes d'audit ; tests e2e (neufs et adaptés).
- [ ] **T3 — l'écran** (AC 1, 2, 6) : avertissement, totaux, grille de complément ; types et API de la feature ;
  Vitest (neufs et adaptés) ; `sitesTotal` ; messages de verrou réécrits × 4 locales et replis.
- [ ] **T4 — Playwright** (AC 7) : parcours de complément ; spec existante adaptée.
- [ ] **T5 — manuel, CHANGELOG** (AC 8).
- [ ] **T6 — gates** : backend complet (repositories `kesh-db` : gate complet même en cours de boucle), frontend
  complet, **E2E complet au dernier commit de code** (décision D7).

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Ne pas verrouiller `companies` en exclusif**, ni **après** les exercices : le premier forme un cycle avec une
  écriture sans projet, le second avec une écriture taguée et la génération. Suivre l'ordre de l'AC 4.
- ⛔ **Ne pas verrouiller une ligne d'écriture** (`journal_entries … FOR SHARE`) pour `NO_ENTRIES` : cycle avec la
  contre-passation et la dévalidation.
- ⛔ **Ne pas lire « jamais mouvementé » par une lecture ordinaire** : elle lit l'instantané, ouvert par la lecture de
  `NO_ENTRIES`, et ne voit pas une écriture validée pendant l'attente de l'étape 4.
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
- **Cycles résiduels** (AC 4) : à trois par la file d'attente d'InnoDB (préexistant entre écritures taguées et non
  taguées), et sur deux comptes saisis mouvementés pour la première fois par une même écriture concurrente. La victime
  autre que le complément rend une 500, comme aujourd'hui entre écritures ordinaires.
- **Verrous d'intervalle** (validation P3, R-7) : le `FOR SHARE` de l'étape 5 pose des verrous next-key sur
  `idx_jel_account` ; une écriture d'un autre exercice sur un compte **voisin dans l'index** peut attendre le commit du
  complément. Une attente, pas un cycle.
- **`NO_ENTRIES` non verrouillé** : si l'unique écriture de la société est dévalidée pendant le complément, celui-ci
  passe et devient la première écriture. Cas sans dommage comptable (l'écriture reste équilibrée).

### Modules

`kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` (écran des soldes de départ) — **quatre modules de code**, sous le seuil
de découpage (+ `docs`, `CHANGELOG`, hors décompte des modules de code).

### References

- Issue [#445] ; cartographie du 2026-10-06 (agent de lecture seule) ; rétrospective `epic-25-retro-2026-10-06.md`
  (D1-D3, D6, D7).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

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
