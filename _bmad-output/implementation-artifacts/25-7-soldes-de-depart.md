# Story 25.7 : Soldes de départ — avertir si le report à-nouveau manque, compléter un compte oublié

Status: ready-for-dev

**Issue : [#445]**, que cette story **ferme** : la PR porte `closes #445` dans le **titre ET le corps**. Branche
`story/25-7-soldes-de-depart`, partie de `chore/epic-25-retrospective` (la rétrospective de l'Epic 25 voyage dans la
même PR). **Dernière story de l'Epic 25** (décision D1 de la rétrospective du 2026-10-06) : le jalon « Vague 1 » ne se
clôt qu'avec elle. Elle précède la release v0.12.1, après laquelle Guy installe Kesh **à partir de zéro** et passe donc
par cet écran (décisions D2, D3).

## Arbitrages proposés (retenus par défaut, révisables — à soumettre à Guy)

1. **« Compte oublié » = compte de bilan JAMAIS MOUVEMENTÉ** (aucune ligne d'écriture, dans aucun exercice), et non
   « solde net nul ». L'issue dit « solde actuel nul » mais justifie la restriction par « un compte déjà mouvementé ne
   peut pas voir son ouverture réécrite » : seul le critère « jamais mouvementé » tient cette promesse. Un compte
   mouvementé dont le net retombe à zéro a un historique ; lui ajouter un solde de départ après coup réécrirait cet
   historique.
2. **Date de l'ajustement** : le **premier jour du premier exercice** (la date de l'écriture d'ouverture) si cet
   exercice est **ouvert** et que cette date n'est pas dans la période verrouillée ; sinon **la date du jour**, dans
   l'exercice ouvert qui la couvre (une régularisation, comme le dit l'issue) ; si aucun exercice ouvert ne couvre la
   date du jour, ou si elle tombe elle-même dans la période verrouillée → refus nommé.
3. **La contrepartie est calculée par le serveur** sur le compte de rôle *Bénéfice reporté* (`RetainedEarnings`) :
   l'utilisateur ne saisit que les comptes oubliés. Si aucun compte actif et postable ne porte ce rôle → refus nommé.
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

## Les faits, vérifiés dans le code (cartographie du 2026-10-06)

- **Routes** (`crates/kesh-api/src/lib.rs:384-393`, bloc `comptable_routes`) : `POST /api/v1/opening-balances`
  (`routes::opening_balances::generate_opening_balances`) et `GET /api/v1/opening-balances/status`.
- **Status** (`routes/opening_balances.rs:93-101`) : `{ fiscalYear: {id, name, startDate, status} | null, canEnter,
  reason }`, raisons `READY`, `NO_FISCAL_YEAR`, `FIRST_YEAR_CLOSED`, `ALREADY_HAS_ENTRIES`.
- **POST, ordre des gardes** (`:239-337`) : plafond de lignes ; parse `Decimal` ; premier exercice
  (`fiscal_years::find_first_by_company`, `:268`) ; `journal_entries::count_by_company > 0` → 409
  `error-opening-balances-already-has-entries` (`:282-288`) ; premier exercice `Closed` → 400 (`:292-297`) ; refus des
  comptes `Revenue`/`Expense` (`:302-319`, **lecture dans une transaction séparée, hors des verrous** — voir Limites) ;
  libellé dans la langue comptable (`:324-327`) ; `accounting::validate` (`:337`) ; `NewJournalEntry { journal: OD,
  entry_date: fiscal_year.start_date }` (`:340-357`) ; `journal_entries::create_opening_entry` (`:361`). Le corps ne
  porte **ni journal ni date** : le serveur les impose (`:71-73`).
- **Aucun contrôle du report à-nouveau** : le handler ne regarde jamais le rôle `RetainedEarnings`. C'est le défaut 1
  de l'issue.
- **`create_opening_entry`** (`kesh-db/src/repositories/journal_entries.rs:562-644`) : sentinelle
  `companies … FOR UPDATE`, puis exercice `FOR UPDATE`, puis `count_by_company` **sous verrou**, puis statut, puis
  `create_in_tx(…, enforce_postable = true)`. **Ordre des verrous du dépôt : `companies → projects → fiscal_years`.**
- **`create_in_tx`** (`:187` et suivantes) : lit `companies.books_locked_through` (`:228-236`), reverrouille l'exercice
  (`Closed` → `FiscalYearClosed` ; date hors bornes → `DateOutsideFiscalYear`, `:239-264`), refuse une date
  `<= books_locked_through` → `PeriodLocked` (`:283-290`), vérifie les comptes (société, actifs, postables), écrit
  l'audit `journal_entry.created` dans la transaction (`:470-481`), numéro par compteur. **Ne prend pas la sentinelle
  `companies`** : un appelant qui veut sérialiser doit la prendre lui-même, en tête.
- **Rôle `RetainedEarnings`** : `kesh_db::entities::AccountRole` (`entities/account.rs:89-110`), singleton
  (`is_singleton()`), colonne générée `accounts.singleton_role` (vaut le rôle si le compte est actif, sinon `NULL` ;
  unicité `uq_accounts_company_singleton_role`). **Aucune fonction générique de recherche par rôle** : le motif existant
  est du SQL en ligne (`company_invoice_settings.rs:563-566` : `SELECT id FROM accounts WHERE company_id = ? AND
  singleton_role = ? ORDER BY id LIMIT 1 FOR UPDATE`) ; `accounts.rs:690` `find_singleton_role_holder` est privé. Les
  trois plans désignent **2970 « Bénéfice/perte reporté »**, type `Liability`, postable (`pme.json:39`,
  `independant.json:37`, `association.json:35` ; `kesh-core/src/chart_of_accounts/mod.rs:517-519`).
- **Bilan vs résultat** : `kesh-report/src/opening.rs:42` `is_bilan(t) = Asset | Liability` — **il n'y a pas de type
  `Equity`**, les capitaux propres sont `Liability`. Aucune fonction « solde actuel d'un compte » dans
  `kesh-db/repositories` ; le seul calcul est l'agrégat de `trial_balance::generate` (`kesh-report`).
- **Exercices** : `find_first_by_company` (`fiscal_years.rs:690`), `find_covering_date` (`:469`, tous statuts, `Open`
  en premier), `find_open_covering_date` (`:543`, en transaction, `FOR UPDATE`). Pas de notion d'« exercice courant ».
- **Écran** (`frontend/src/routes/(app)/settings/opening-balances/+page.svelte`, 403 lignes, 26 sites `i18nMsg(`) :
  états chargement / erreur / verrou (`data-testid="opening-balances-locked"`, `data-reason`) / grille. Grille =
  comptes actifs, postables, `Asset|Liability` (`:76-84`) ; totaux `computeBalance(rows)` (big.js) →
  `totalDebit`, `totalCredit`, `diff`, `isBalanced` (`:93`). Message du verrou `ALREADY_HAS_ENTRIES` (`:222-225`) :
  « Corrigez l'écriture d'ouverture directement dans le journal, ou supprimez toutes les écritures… ». Feature
  `lib/features/opening-balances/opening-balances.{api,types}.ts` (union `OpeningBalancesReason`, 4 valeurs). Menu
  dans `comptableOnly` (`routes/(app)/+layout.svelte:128-135`). **L'écran n'est pas proposé à l'onboarding** : on y
  accède par *Administration → Soldes de départ*.
- **Tests existants** : `crates/kesh-api/tests/opening_balances_e2e.rs` (21), `crates/kesh-db/tests/
  opening_balances_repository.rs` (10), `opening-balances-page.test.ts` (19 Vitest), `frontend/tests/e2e/
  opening-balances.spec.ts` (2 Playwright).
- **Manuel** (`docs/manual/fr/user-manual.tex`) : `:648` § « Reprise de comptabilité — soldes de départ »
  (`\label{sec:soldes-depart}`), étapes `:654-659`, **verrou « définitif » et remède « contre-passez l'écriture
  d'ouverture… puis saisissez une écriture manuelle… trois écritures au lieu d'une » `:661-664`** ; `:383-404` § des
  comptes 9000/9100/9200 (reprise par cet écran, report sur 2970 ; le à-nouveau passé par le 9000 « partait en
  charges », `:396` ; contre-passer et ressaisir `:400-402`). `admin-manual.tex` : aucune mention.

## Acceptance Criteria

**AC 1 — L'avertissement à la saisie (défaut 1 de l'issue).** Dans la grille du mode « ouverture », quand la saisie est
**équilibrée** et qu'**aucun montant** ne porte sur le compte de rôle `RetainedEarnings`, l'écran affiche un
avertissement (`data-testid="opening-balances-no-retained-earnings"`) qui dit **pourquoi** : le report à-nouveau —
la différence entre actifs et passifs de l'ancien bilan — doit être porté sur ce compte, nommé (numéro et libellé).
**Il signale, il ne bloque pas** : le bouton « Générer » reste actif (doctrine #301). Si la société n'a **aucun** compte
actif portant ce rôle, l'avertissement le dit (« aucun compte n'est désigné *Bénéfice reporté* »). Le rôle se lit côté
client sur les comptes chargés (`account.role` ou équivalent — vérifier le champ exact dans `accounts.types.ts`).

**AC 2 — Totaux actif / passif et écart attendu.** La grille affiche, en plus des totaux débit / crédit existants, le
**total des actifs** (soldes débiteurs des comptes `Asset`), le **total des passifs et capitaux** (soldes créditeurs
des comptes `Liability`, report exclu) et l'**écart que le report à-nouveau devrait porter**. Montants en big.js,
jamais en `Number`. Un compte `Asset` saisi au crédit ou `Liability` au débit (contre-nature : un découvert, une
créance d'impôt) est compté avec son signe — l'écart doit rester juste.

**AC 3 — Le status annonce le mode « compléter ».** `GET /api/v1/opening-balances/status` gagne un champ `canComplete`
(booléen) et, si besoin, les raisons qui l'en empêchent. `canComplete` est vrai quand la société **porte déjà des
écritures** (raison actuelle `ALREADY_HAS_ENTRIES`, y compris quand le premier exercice est clos) **et** qu'un compte
actif et postable porte le rôle `RetainedEarnings` **et** qu'une date d'ajustement est possible (arbitrage 2). Les
champs existants (`canEnter`, `reason`, `fiscalYear`) sont **inchangés** : le mode « ouverture » ne bouge pas.

**AC 4 — La route de complément.** `POST /api/v1/opening-balances/complete`, corps `{ lines: [{ accountId, debit,
credit }] }` comme la génération — **sans** la contrepartie, que le serveur calcule. Gardes, dans cet ordre, **toutes
sous verrou dans une seule transaction** (fonction `kesh-db` nouvelle, p. ex. `journal_entries::create_opening_complement`) :
1. sentinelle `companies … FOR UPDATE` en tête (ordre des verrous du dépôt) ;
2. au moins une ligne, plafond `MAX_LINES_PER_ENTRY − 1` (la contrepartie en prend une), montants positifs, une seule
   colonne par ligne, comptes distincts ;
3. chaque compte : de la société, actif, postable, **de bilan** (`Asset` ou `Liability`) — **lu sous verrou**, pas
   dans une transaction préalable — et **jamais mouvementé** (aucune ligne d'écriture, arbitrage 1), vérifié **sous le
   même verrou** ; ⛔ le compte `RetainedEarnings` lui-même est refusé comme ligne saisie ;
4. le compte `RetainedEarnings` : actif, postable — sinon refus nommé ;
5. la date (arbitrage 2) et l'exercice qui la couvre, ouvert ;
6. la contrepartie : une ligne sur le compte `RetainedEarnings`, au crédit si les lignes saisies sont nettement
   débitrices, au débit sinon, du montant de l'écart — **refus si l'écart est nul** (rien à équilibrer : la saisie est
   déjà équilibrée, elle n'a pas besoin du report… — ⚠️ à vérifier : un écart nul reste une écriture valide ; décider
   si on l'accepte sans contrepartie, voir Limites) ;
7. `create_in_tx(…, enforce_postable = true)` — qui applique exercice clos, bornes, verrou de période, audit.
Réponse `201 + JournalEntryResponse`, journal `OD`, libellé « Complément des soldes de départ » dans la langue
comptable de la société.

**AC 5 — Les erreurs.** Chaque refus porte un **code nommé** et un message dans les 4 locales : compte déjà mouvementé
(qui le nomme), compte de résultat, compte `RetainedEarnings` saisi, aucun compte de report désigné, aucune date
possible (aucun exercice ouvert couvrant la date, ou période verrouillée), société sans écriture (le mode
« compléter » n'a pas lieu d'être : utiliser la génération). Les erreurs existantes de `create_in_tx` remontent par
`map_core_error` / le mapping existant.

**AC 6 — L'écran du mode « compléter ».** Quand le status rend `canEnter = false` et `canComplete = true`, l'écran
remplace le message de verrou par une grille de **complément** (`data-testid="opening-balances-complete"`) :
- elle ne propose **que les comptes de bilan jamais mouvementés** — la liste vient du serveur (un champ du status, ou
  une route de lecture ; ne pas la recalculer côté client) ;
- elle montre la contrepartie calculée sur le compte de report, en direct ;
- elle dit la **date** de l'ajustement et pourquoi (exercice d'ouverture, ou régularisation datée du jour) ;
- un bouton « Compléter » (`data-testid="opening-balances-complete-submit"`) avec confirmation ;
- après succès, l'écran se recharge et le compte complété disparaît de la liste.
Quand `canComplete = false`, le message de verrou dit **pourquoi**, et ne propose plus la contre-passation intégrale
comme remède par défaut.

**AC 7 — Tests.** Chacun aurait échoué avant le patch :
- `kesh-db` : complément sur un compte jamais mouvementé → écriture équilibrée par la contrepartie sur 2970, datée
  selon l'arbitrage 2 (deux cas : premier exercice ouvert / clos) ; refus d'un compte mouvementé ; refus d'un compte
  de résultat ; refus du compte de report saisi ; refus sans compte de report ; refus en période verrouillée ;
  **course** : un compte mouvementé par une autre transaction entre la lecture et l'écriture n'est pas complété
  (test d'entrelacement, patron `test_fixtures::attendre_une_requete_en_cours`) — mutation « vérification hors
  verrou » tuée ;
- `kesh-api` : la route (201, refus nommés, RBAC Consultation 403, isolation entre sociétés) ; le status
  (`canComplete` vrai/faux selon les cas) ;
- Vitest : avertissement présent / absent (AC 1), non bloquant ; totaux actif/passif et écart (AC 2), y compris un
  compte contre-nature ; grille de complément (AC 6) ;
- Playwright : un parcours — générer l'ouverture **sans** le compte 1020, puis le compléter, et voir la balance
  porter le 1020.
- Mutations tuées et nommées (au moins : garde « jamais mouvementé » retirée, sens de la contrepartie inversé,
  avertissement toujours masqué).

**AC 8 — Le manuel et le CHANGELOG.** `user-manual.tex` :
- `:661-664` : le remède « contre-passer puis ressaisir » est remplacé par le **complément** ; la contre-passation
  intégrale reste citée pour un montant **faux** sur un compte déjà saisi (le complément ne le corrige pas) ;
- `:654-659` : l'avertissement du report à-nouveau, les totaux actif/passif et l'écart ;
- `:400-402` (comptes 9000) : vérifier que le remède cité reste juste ;
- greper la **valeur** « contre-pass » aux 4 locales et dans les manuels.
PDF régénéré, contrôlé **aplati**. CHANGELOG `[0.12.1]` : `Added` (le complément, l'avertissement).

**AC 9 — Gardes structurelles.** La route POST neuve entre au registre des routes d'audit
(`crates/kesh-api/tests/audit_route_registry.rs`, `Traced` — l'audit vient de `create_in_tx`) et ses **trois totaux**
sont recalés **en recomptant la source** ; `sitesTotal` de `frontend/src/lib/shared/i18n-keys.test.ts` recompté par
`grep -o "i18nMsg("` aux deux bornes, avec une note d'historique ; `lint-i18n-ownership` vert (clés `opening-balances-*`).

## Tasks / Subtasks

- [ ] **T1 — `kesh-db`** (AC 4, 7) : recherche du compte de rôle (`accounts::find_id_by_singleton_role` ou nom
  équivalent, en transaction) ; liste des comptes de bilan jamais mouvementés d'une société ; choix de la date et de
  l'exercice (arbitrage 2) ; `create_opening_complement` (sentinelle, gardes sous verrou, contrepartie,
  `create_in_tx`) ; tests de dépôt dont l'entrelacement.
- [ ] **T2 — `kesh-api`** (AC 3, 4, 5, 9) : status étendu (`canComplete`, liste des comptes complétables, date
  prévue) ; route `POST …/complete` ; erreurs nommées × 4 locales ; registre des routes d'audit ; tests e2e.
- [ ] **T3 — l'écran** (AC 1, 2, 6) : avertissement, totaux actif/passif et écart, grille de complément ; types et API
  de la feature ; Vitest ; `sitesTotal`.
- [ ] **T4 — Playwright** (AC 7).
- [ ] **T5 — manuel, CHANGELOG** (AC 8).
- [ ] **T6 — gates** : backend complet (touche des repositories `kesh-db` : gate complet même en cours de boucle),
  frontend complet, **E2E complet au dernier commit de code** (décision D7).

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Ne pas lire « jamais mouvementé » ni le type des comptes hors du verrou** : c'est un TOCTOU (cf. la leçon de
  l'Epic 25, REPEATABLE READ). Tout se relit **dans** la transaction de création, après la sentinelle `companies`.
- **Ne pas toucher au mode « ouverture »** : `create_opening_entry`, ses gardes et ses tests restent tels quels.
- **Ne pas recalculer côté client** la liste des comptes complétables ni la date : le serveur les donne.
- **Ne pas bloquer** sur l'avertissement de l'AC 1.
- **Ne pas créer d'action d'audit neuve** (arbitrage 4) — sinon `audit_label_registry` l'exigerait × 4 locales.

### Limites assumées

- **TOCTOU préexistant du mode « ouverture »** : le refus des comptes de résultat (`opening_balances.rs:302-319`) lit
  les types hors du verrou de `create_opening_entry`. Fenêtre étroite (changer le type d'un compte pendant une
  génération), antérieure à la story, **hors périmètre** — à tracer en issue si la validation le confirme.
- **Un montant faux** sur un compte déjà saisi ne se corrige pas par le complément (le compte est mouvementé) : il
  reste une écriture de correction manuelle, ou la contre-passation de l'ouverture.
- **Écart nul** dans un complément (des lignes qui s'équilibrent entre elles) : AC 4 étape 6 à trancher en
  validation — accepter sans contrepartie, ou refuser.

### Modules

`kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` (écran des soldes de départ) — **quatre modules de code**, sous le seuil
de découpage (+ `docs`, `CHANGELOG`).

### References

- Issue [#445] ; cartographie du 2026-10-06 (agent de lecture seule) ; rétrospective `epic-25-retro-2026-10-06.md`
  (D1-D3).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-06** — Créée (rétrospective de l'Epic 25, décision D1). Cinq arbitrages proposés, retenus par défaut.

[#445]: https://github.com/guycorbaz/kesh/issues/445
