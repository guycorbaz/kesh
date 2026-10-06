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
   - une écriture **supprimée** (brouillon, ou dévalidation qui détruit l'écriture) retire ses lignes : le compte
     redevient complétable ;
   - un compte **archivé** n'est pas proposé (il doit être actif et postable) ; réactivé, il l'est s'il n'a jamais été
     mouvementé.
2. **Date de l'ajustement** — « aujourd'hui » est la **date UTC** du serveur (comme `today_utc`, `invoices.rs:249`),
   passée en **paramètre** à la fonction `kesh-db` (testable) et **recalculée sous verrou** par le POST :
   - (a) si le **premier exercice** (`find_first_by_company`) est **ouvert** et que son premier jour est **après**
     `books_locked_through` : le **premier jour du premier exercice**, la date de l'écriture d'ouverture. **Écart à
     l'issue**, qui ne demande que le cas (b) : un complément antidaté au début de l'exercice d'ouverture modifie les
     soldes d'ouverture et tout rapport déjà tiré sur cet exercice — c'est voulu (le compte oublié faisait partie de
     l'ouverture), et le manuel le dit ;
   - (b) sinon (premier exercice clos, ou son premier jour dans la période verrouillée) : **aujourd'hui**, dans
     l'exercice **ouvert** qui couvre cette date (`find_open_covering_date`) — une régularisation, comme le dit
     l'issue ;
   - (c) si aucun exercice ouvert ne couvre aujourd'hui → refus `OPENING_COMPLEMENT_NO_OPEN_FISCAL_YEAR` ; si
     aujourd'hui est lui-même dans la période verrouillée → refus `OPENING_COMPLEMENT_DATE_LOCKED`.
   La date affichée par le status (AC 3) est **indicative** : le POST fait autorité (un verrou posé entre-temps, ou
   minuit UTC franchi, peut la changer).
3. **La contrepartie est calculée par le serveur** sur le compte de rôle *Bénéfice reporté* (`RetainedEarnings`),
   cherché **dans la société** : l'utilisateur ne saisit que les comptes oubliés. **Si les lignes saisies
   s'équilibrent entre elles** (deux comptes oubliés qui se compensent), **aucune contrepartie** : l'écriture porte les
   seules lignes saisies (il en faut alors au moins deux) — une ligne à zéro serait refusée par la contrainte
   `chk_jel_debit_credit_exclusive`. Sans compte actif portant ce rôle, ou s'il n'est pas postable → refus nommé.
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
- **POST, ordre des gardes** (`:239-337`) : plafond de lignes (`MAX_LINES_PER_ENTRY = 500`,
  `journal_entries.rs:412`) ; parse `Decimal` ; premier exercice (`find_first_by_company`, `:268`) ;
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
  `journal_entry.created` dans la transaction (`:470-481`). **Ne prend pas la sentinelle `companies`.**
- ⛔ **Conséquence (validation P1, B1 CRITICAL)** : une écriture ordinaire ne prend ni la sentinelle `companies`, ni de
  verrou sur `accounts`, ni d'autre exercice que le sien. La sentinelle **ne sérialise donc pas** un complément contre
  une écriture ordinaire. **Ce qui la sérialise** : `journal_entry_lines` porte la clé étrangère
  `fk_jel_account → accounts(id)` (schéma, `0001_schema_squash.sql`), et **InnoDB pose un verrou partagé sur la ligne
  parente** `accounts` à chaque insertion d'une ligne d'écriture. Un `SELECT … FROM accounts … FOR UPDATE` sur les
  comptes à compléter attend donc toute écriture en vol sur ces comptes, et bloque celles qui viendraient après.
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
  comptes `active && postable && Asset|Liability` (`:76-84`) ; totaux `computeBalance(rows)` (big.js, `:93`). Le
  champ de rôle existe côté client : `role: AccountRole | null` (`accounts.types.ts:54`). Message du verrou
  `ALREADY_HAS_ENTRIES` (`:222-225`, repli `:233`). Feature `lib/features/opening-balances/opening-balances.{api,types}.ts`
  (union `OpeningBalancesReason`, 4 valeurs). Menu `comptableOnly` (`routes/(app)/+layout.svelte:128-135`). **L'écran
  n'est pas proposé à l'onboarding.**
- **Messages qui conseillent un remède périmé** (validation P1, C2) : `opening-balances-locked-already-has-entries`
  (fr-CH `messages.ftl:927` : « Corrigez l'écriture d'ouverture directement dans le journal, **ou supprimez toutes les
  écritures** pour recommencer » — impossible depuis la 24-4a) et `error-opening-balances-already-has-entries`
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
zéro. **Il signale, il ne bloque pas** (doctrine #301) : le bouton « Générer » reste actif. Cas distincts, chacun son
texte : aucun compte actif ne porte le rôle ; le compte qui le porte n'est pas postable (il n'est alors pas dans la
grille). Le rôle se lit sur `account.role` (`accounts.types.ts:54`).

**AC 2 — Totaux actif / passif et écart attendu.** La grille affiche, en plus des totaux débit / crédit existants :
- **Actifs** = Σ (débit − crédit) des lignes sur des comptes `Asset` ;
- **Passifs et capitaux** = Σ (crédit − débit) des lignes sur des comptes `Liability`, **le compte `RetainedEarnings`
  exclu** ;
- **Report à-nouveau attendu** = Actifs − Passifs et capitaux : positif → à porter **au crédit** du compte de report
  (bénéfice reporté), négatif → **au débit** (perte reportée) ;
- **Report saisi** = (crédit − débit) sur le compte `RetainedEarnings`, et l'**écart restant** = attendu − saisi, qui
  vaut 0 quand la saisie est équilibrée.
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
Lecture sans verrou : le POST fait autorité.

**AC 4 — La route de complément.** `POST /api/v1/opening-balances/complete`, corps `{ lines: [{ accountId, debit,
credit }] }` — **sans** la contrepartie. Fonction `kesh-db` nouvelle (p. ex. `journal_entries::create_opening_complement
(pool, company_id, user_id, actor_api_key_id?, lines, description, today)`), **une seule transaction**, dans cet ordre :
1. **sentinelle** `SELECT id FROM companies WHERE id = ? FOR UPDATE` (sérialise contre un autre complément, la
   génération et `lock_books`) ; **relire `books_locked_through` après elle** ;
2. contrôles de forme : au moins une ligne ; plafond `MAX_LINES_PER_ENTRY − 1` (une ligne réservée à la contrepartie) ;
   montants strictement positifs, une seule colonne par ligne ; comptes distincts ;
3. **verrouiller les comptes saisis** : `SELECT id, company_id, account_type, active, postable, singleton_role FROM
   accounts WHERE id IN (…) ORDER BY id FOR UPDATE` — lecture verrouillante, donc à jour ; chacun doit être de la
   société, actif, postable, de bilan (`Asset`/`Liability`), et **ne pas** porter le rôle `RetainedEarnings` ;
4. **« jamais mouvementé », par une lecture verrouillante** :
   `SELECT account_id FROM journal_entry_lines WHERE account_id IN (…) LIMIT 1 FOR SHARE` (ou équivalent) — après le
   verrou de l'étape 3, qui a attendu toute écriture en vol ; ⛔ **aucune lecture non verrouillante avant cette
   étape** (sous REPEATABLE READ, elle figerait l'instantané) ;
5. le compte `RetainedEarnings` **de la société** : `… WHERE company_id = ? AND singleton_role = 'RetainedEarnings'
   FOR UPDATE` ; présent et postable, sinon refus ;
6. la date et l'exercice (arbitrage 2), l'exercice verrouillé `FOR UPDATE` (ordre `companies → … → fiscal_years`) ;
7. la contrepartie (arbitrage 3) : écart = Σ débit − Σ crédit des lignes saisies ; > 0 → une ligne **au crédit** du
   compte de report de ce montant ; < 0 → **au débit** ; = 0 → aucune ligne (au moins deux lignes saisies) ;
8. `create_in_tx(…, enforce_postable = true)` — exercice clos, bornes, période verrouillée, audit
   `journal_entry.created`.
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
| Date dans la période verrouillée | 409 | `OPENING_COMPLEMENT_DATE_LOCKED` | `error-opening-complement-date-locked` |

Les erreurs de forme (montants, lignes) réutilisent le mapping existant de la génération. Messages dans les **4
locales**.

**AC 6 — L'écran du mode « compléter ».** Quand `canEnter = false`, l'écran garde **toujours** le bandeau de verrou
existant (`data-testid="opening-balances-locked"`, `data-reason`, liens vers le bilan et le journal) — ce qui a été
généré reste dit, et le Playwright existant (`opening-balances.spec.ts:107-112`) reste vrai. **En dessous** :
- si `canComplete = true`, une grille de **complément** (`data-testid="opening-balances-complete"`) :
  - les seuls `completableAccounts` du status (la liste n'est **pas** recalculée côté client) ;
  - la contrepartie calculée en direct sur le compte de report (sens et montant, arbitrage 3 — y compris « aucune
    contrepartie » quand les lignes s'équilibrent) ;
  - la date prévue et sa raison (exercice d'ouverture, ou régularisation datée du jour), avec la mention qu'elle est
    confirmée à l'enregistrement ;
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
  - après **contre-passation** d'une écriture sur un compte → toujours mouvementé ; après **suppression** d'un
    brouillon → redevenu complétable ;
  - **deux entrelacements** (patron `test_fixtures::attendre_une_requete_en_cours`) : (1) une écriture **ordinaire**
    sur le compte, dans **un autre exercice** que celui du complément, en vol pendant le complément → le complément
    attend puis refuse `ACCOUNT_MOVED` ; (2) deux compléments du même compte → un seul réussit. Mutations tuées :
    « comptes non verrouillés à l'étape 3 », « contrôle de l'étape 4 en lecture non verrouillante avant l'étape 3 ».
- **`kesh-api`** : la route (201, chaque code de l'AC 5, RBAC Consultation 403, isolation entre sociétés) ; le status
  (`canComplete` et chaque `completeReason`, `completableAccounts`).
- **Vitest** : avertissement présent / absent / non bloquant, et ses deux variantes (AC 1) ; totaux, report attendu et
  écart restant, y compris un compte contre-nature (AC 2) ; grille de complément, contrepartie en direct, liste vide
  impossible (`canComplete` faux), verrou avec `completeReason` (AC 6).
- **Tests existants adaptés** (validation P1, C3) : les mocks Vitest de `ALREADY_HAS_ENTRIES` (`:206-216`, `:360-384`,
  `:400-419`) reçoivent les champs neufs (`canComplete: false`, `completeReason`) ; leur assertion du bandeau de verrou
  est conservée (AC 6 le garde toujours) ; les tests e2e du status vérifient la nouvelle forme ; le Playwright
  `:107-112` reste vrai tel quel (le bandeau reste affiché ; la société `with-company` porte 2970 et des comptes non
  mouvementés : la grille de complément apparaît en plus, sans casser l'assertion).
- **Playwright** : générer l'ouverture **sans** un compte de bilan (p. ex. 1020), puis le compléter, et voir la balance
  le porter.
- **Mutations** tuées et nommées : garde « jamais mouvementé » retirée ; sens de la contrepartie inversé ; avertissement
  toujours masqué ; les deux mutations de verrou ci-dessus.

**AC 8 — Le manuel et le CHANGELOG.** `user-manual.tex` :
- `:654-659` : l'avertissement du report à-nouveau, les totaux et le report attendu ;
- `:661-664` : « se verrouille **définitivement** » devient faux — l'écran passe en mode « compléter » ; le remède
  « contre-passer puis ressaisir » est remplacé par le **complément**, avec ses règles (comptes jamais mouvementés,
  date, contrepartie) ; la contre-passation reste citée pour un **montant faux** sur un compte déjà saisi, et le manuel
  dit qu'après la contre-passation de toute l'ouverture plus rien n'est complétable (arbitrage 1) ;
- `:400-402` (comptes 9000) : vérifier que le remède cité reste juste ;
- `:1682` (balance) : l'exception du complément daté du jour.
Greper les **valeurs** « définitivement », « supprimez toutes » (et « delete all », « Löschen Sie alle », « elimina
tutte »), « contre-pass » dans les manuels et les **4 catalogues**. PDF régénéré, contrôlé **aplati**. CHANGELOG
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
  choix de la date (arbitrage 2, `today` en paramètre) ; `create_opening_complement` (sentinelle, comptes
  `FOR UPDATE`, « jamais mouvementé » verrouillant, contrepartie, `create_in_tx`) ; variante d'erreur ; tests de
  dépôt dont les deux entrelacements.
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

- ⛔ **Ne pas lire les comptes ni « jamais mouvementé » hors verrou**, ni par une lecture non verrouillante avant le
  verrou des comptes (REPEATABLE READ fige l'instantané). La sentinelle `companies` **ne suffit pas** contre une
  écriture ordinaire : c'est le verrou `FOR UPDATE` des **comptes** (que l'insertion d'une ligne prend en partagé, par
  la clé étrangère) qui sérialise.
- **Ne pas toucher au mode « ouverture »** : `create_opening_entry`, ses gardes et ses tests restent tels quels.
- **Ne pas recalculer côté client** la liste des comptes complétables ni la date.
- **Ne pas bloquer** sur l'avertissement de l'AC 1.
- **Ne pas créer d'action d'audit neuve** (arbitrage 4).
- **Ne pas émettre de `DbError::Invariant`** pour un refus métier : 500 garantie.

### Limites assumées

- **TOCTOU préexistant du mode « ouverture »** : le refus des comptes de résultat (`opening_balances.rs:300-319`) lit
  les types hors du verrou de `create_opening_entry`. Fenêtre étroite (changer le type d'un compte pendant une
  génération), antérieure à la story, **hors périmètre** — à tracer en issue.
- **Un montant faux** sur un compte déjà saisi ne se corrige pas par le complément (le compte est mouvementé) : une
  écriture de correction manuelle.
- **Après contre-passation de toute l'ouverture**, plus aucun compte de l'ouverture n'est complétable (arbitrage 1).
- **La date du status est indicative** (arbitrage 2).

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
