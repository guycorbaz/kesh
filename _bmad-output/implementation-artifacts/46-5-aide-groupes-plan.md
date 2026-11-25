# Story 46-5 : aide à la création des groupes d'un plan existant (conditionnelle)

Status: ready-for-dev

*Revue du responsable de l'epic (2026-11-19) : approuvée ; **proposition d'interface (maquette) avant de décider** de la story (Q9) ; approuvée par Guy le 2026-11-19 (revue 1).*

*Esquisse du 2026-11-18, reprise le 2026-11-20. Epic 46, D12. **Conditionnelle** (Q9) : ni validée en boucle ni
développée avant la maquette de l'AC-0 et la décision du responsable de l'epic et de Guy. `fix/api-parent-compte` est fusionné dans
`main` (#1351, `d29aa557`) : `PUT /accounts/{id}` accepte `parentId`, refuse un cycle et un parent qui porte des
écritures. Release visée 0.21.0 si décidée ; sinon la story sort de l'epic, qui compte alors six stories. Aucune
migration attendue.*

**Modules touchés** (règle de splitting préventif) : `kesh-api/routes/accounts` (ou appels existants enchaînés),
`kesh-db/repositories/accounts` (transaction), `frontend` (écran du plan), `kesh-i18n`, manuel — cinq ; à recompter
à la validation selon la maquette.

## Story

En tant que comptable qui range un plan existant, je veux que Kesh me propose des comptes de groupe d'après les numéros
de mes comptes, afin de structurer mon plan (une banque = un groupe, puis les liquidités) sans tout créer à la main.

## Critères d'acceptation (esquisse)

0. **AC-0 — Maquette avant décision (Q9, responsable).** Une maquette de l'interface (sélection, proposition, aperçu de
   l'arbre) est présentée au responsable de l'epic et à Guy ; **la story n'est décidée, ni développée, qu'après cette maquette**. La règle
   « numéro sans son dernier chiffre » y est confirmée ou changée.

1. **AC-1 — Proposition.** Pour une sélection de comptes, Kesh propose un groupe = « numéro sans son dernier chiffre »
   (règle à confirmer, Q9), avec nom à saisir et parent éventuel ; aperçu de l'arbre résultant.
2. **AC-2 — Rien d'office.** Aucun compte créé ni déplacé sans confirmation ; un numéro déjà pris, un compte qui porte
   des écritures (refus de #1351 : un compte déjà mouvementé ne devient pas un groupe — le groupe d'une banque est un
   compte neuf) ou un cycle sont signalés dans l'aperçu et exclus.
3. **AC-3 — Écriture.** À la confirmation : création des comptes de groupe puis changement de parent, par les mêmes
   contrôles que `POST /accounts` et `PUT /accounts/{id}`, dans une transaction ; audit.
4. **AC-4 — Tests et manuel.** Tests d'intégration ; E2E ; captures ; manuel FR (`.tex`).

## Points à trancher avec la maquette (validation P1, allégée — la fiche est conditionnelle)

La fiche n'est **pas validée en boucle** (Q9) : cette passe ne la confronte qu'au code de `origin/main` et lui
inscrit ce que la maquette doit trancher ; elle se **revalide en entier** après la décision du responsable de l'epic et de Guy.

1. **Atomicité (AC-3).** `accounts::create` (`:189`) et `accounts::update` (`:1079`) prennent le **pool** et ouvrent
   chacun leur transaction (`pool.begin()`, `:192`, `:1091`) ; les enchaîner n'est donc **pas** « dans une transaction ».
   Soit on extrait des variantes `_in_tx` (précédent : `bulk_create_from_chart_in_tx`, `:1847`) que les fonctions
   actuelles appellent — touche `kesh-db`, **gate complet même en boucle de revue** (`CLAUDE.md` § *Exception
   `kesh-db`*) ; soit on admet un enchaînement non atomique avec reprise et on le dit à l'aperçu. À décider.
2. **Le compte de groupe a un type, une devise, une postabilité.** `NewAccount` exige un `account_type` : la
   proposition le **déduit** des comptes sélectionnés quand ils sont tous du même type, et **refuse** une sélection
   de types mêlés (le groupe serait, de toute façon, dissous par le bilan, 46-1 D4) ; devise `None` ; `postable`
   normalisé par le dépôt (un compte qui a des sous-comptes actifs n'est pas imputable, `entities/account.rs:233`).
   `PUT /accounts/{id}` remplace tous les champs et exige la `version` du compte : un déplacement = un `PUT` par
   compte, `version` relue à l'aperçu ; un parent archivé est refusé (`routes/accounts.rs:83`).
3. **Numéro d'un groupe qui existe déjà.** Le cas du responsable de l'epic (10 numéros par banque, groupe = numéro sans son dernier
   chiffre) tombe sur un numéro **déjà pris** quand le plan est celui de la PME (`1020 Banque`, mouvementé : refusé
   comme parent, #1351). L'aperçu doit le dire et laisser saisir un autre numéro.
4. **Parent du groupe proposé.** Par défaut, le **parent commun** des comptes sélectionnés (p. ex. `10` pour des
   comptes bancaires de la PME) : sans lui, le groupe neuf serait une racine et sortirait les comptes de leur place
   dans l'arbre (et du sous-total de `10`). Une sélection sans parent commun : parent à choisir dans l'aperçu.
5. **Modules** : cinq (`routes/accounts` ou appels enchaînés, `repositories/accounts`, frontend, `kesh-i18n`, manuel) ;
   un point 1 « extraction » ajoute des tests de `kesh-db`, non un module. Recompter après la maquette.

## Tâches principales

- [ ] Calcul de la proposition (pur, testé) ; aperçu à l'écran du plan
- [ ] Route de création groupée (ou appels existants enchaînés, à décider à la validation) ; réutiliser les contrôles
      de `accounts::create` et du changement de parent de #1351 (`crates/kesh-db/src/repositories/accounts.rs:189`,
      `:1079`), sans les recopier
- [ ] Tests, captures, manuel

## Fichiers

`crates/kesh-api/src/routes/accounts.rs` ; `crates/kesh-db/src/repositories/accounts.rs` ;
`frontend/src/routes/(app)/accounts/+page.svelte`, `frontend/src/lib/features/accounts/` ; locales i18n ;
`docs/manual/fr/user-manual.tex`.

## Note de version

*(à écrire au développement)*

## Change Log

- 2026-11-20 — Reprise : #1351 fusionnée, refus d'un parent mouvementé rappelé, modules déclarés, statut `drafted`
  (conditionnelle, maquette avant décision).
- 2026-11-20 — validation P1 (Sonnet 5.5), **allégée** (fiche conditionnelle) : 0 C, 0 H, 1 M, 1 L. M : l'AC-3
  « dans une transaction » n'est pas réalisable en enchaînant `create` et `update`, qui ouvrent chacun la leur — section
  « Points à trancher » (extraction `_in_tx`, type et devise du groupe, `version`, numéro déjà pris). L : le statut
  reste `drafted` (conditionnelle), à relire entièrement après la maquette. Axes exercés : `repositories/accounts.rs`,
  `routes/accounts.rs`, `entities/account.rs` ; non exercés : maquette (absente), manuel, i18n.
- 2026-11-20 — validation P2 (Claude Opus 5.5), allégée (fiche conditionnelle) : 0 C, 0 H, 0 M, 1 L, remédié. L : le
  parent du groupe proposé (parent commun de la sélection, sinon les comptes sortent de leur place dans l'arbre) ajouté
  aux points à trancher. Remédiation P1 (variantes `_in_tx`, `:189`, `:192`, `:1079`, `:1091`, `:1847` ;
  `entities/account.rs:233` ; `routes/accounts.rs:83`) confrontée au code : juste. Rapport :
  `kesh-gate-logs/epic-46/validation/46-5-P2.md`.
