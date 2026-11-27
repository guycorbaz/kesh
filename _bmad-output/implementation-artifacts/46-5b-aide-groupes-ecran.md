# Story 46-5b : groupes du plan — propositions, écran et manuel

Status: done

Fiche mère : `46-5-aide-groupes-plan.md` (décision du 2026-11-22, **contrat commun**, **règle des propositions**).
Epic 46, D12, FR-46-5, NFR-46-5. Release visée **0.21.0** (prévision). *(Annoté le 2026-11-26 : la release est celle de l'Epic 46, dont la PR porte désormais la 46-5 — numéro attribué au tag, la v0.20.0 visée.)* **Aucune migration.** **Suit la 46-5a** (route
`POST /api/v1/accounts/groups`). Développée directement sur `story/46-5`, après la 46-5a (consigne de la tâche ; la branche `story/46-5b` prévue n'a pas été créée) ;
**aucune fusion dans `epic/46-rapports-groupes` avant l'accord de Guy** (fiche mère).

## Story

En tant que comptable qui range un plan existant,
je veux cocher des comptes, voir les groupes que Kesh propose d'après leurs numéros, les accepter, les corriger ou les
écarter un par un, et voir l'arbre qui en résultera avant de confirmer,
afin de structurer mon plan en quelques gestes, sans rien voir créé ni déplacé d'office.

## Modules touchés (règle de splitting préventif)

Trois : `frontend` (calcul pur `features/accounts/group-proposals.ts`, page `routes/(app)/accounts/groups/`, bouton de
l'écran du plan, appel d'API dans `accounts.api.ts`, types, spec E2E), `kesh-i18n` (textes de l'écran, quatre locales),
`docs/manual/fr/user-manual.tex`.

## Critères d'acceptation

1. **AC-1 — Entrée.** L'écran du plan (`routes/(app)/accounts/+page.svelte`) porte un bouton **« Créer des groupes »**
   à côté de « Nouveau compte », visible aux mêmes rôles (Administrateur, Comptable ; `+page.svelte:543`), qui ouvre la
   page `/accounts/groups`. La page **refuse** l'accès aux autres rôles par un `+page.ts`
   qui redirige vers `/accounts` (sur le modèle de `routes/(app)/bank-import/manual/+page.ts`, qui admet `Admin` et `Comptable` et redirige vers
   l'accueil ; ici vers `/accounts`) —
   l'écran du plan, lui, ne refuse rien, il masque ses gestes. Aucun autre point d'entrée
   (Q9).

2. **AC-2 — Calcul pur des propositions.** `group-proposals.ts` (sans I/O, documenté en JSDoc) applique la **règle des
   propositions** de la fiche mère sur la liste de `GET /accounts?includeArchived=true` (archivés compris, pour les
   numéros pris ; seuls les actifs s'affichent et se sélectionnent) et la sélection : modes « numéro
   sans son dernier chiffre » et « un seul groupe pour la sélection » ; préfixe d'un seul compte sans proposition sauf
   groupe existant ; nature (créer, rattacher, réutiliser sur coche, refus du compte mouvementé — `currencyLocked` — ou
   archivé) ; nom des mots de tête communs ; parent commun, à choisir sinon, racine avec avertissement ; comptes de
   groupe (au moins un sous-compte actif) jamais membres. Contrôles de chaque proposition, au niveau **erreur** :
   numéro vide ou de plus de 10 octets, nom de plus de 255 octets (règles du serveur), nom vide (créer), types mêlés, aucun membre, groupe existant d'un autre
   type, groupe existant qui porte des écritures, parent choisi qui porte des écritures (règle du `PUT`, #1351), numéro d'un compte **de la même proposition** (`numberIsMember`), numéro pris par une autre proposition acceptée, compte d'un `rattacher` ou
   d'un `réutiliser` membre d'une autre proposition acceptée (« un niveau à la fois »), parent manquant sans parent
   commun ; au niveau de la **demande** (pied de page, *Confirmer* inactif), plus de 100 propositions acceptées ou de
   1000 comptes à ranger (`tooMany`) ; deux propositions acceptées qui visent le même compte existant (le motif
   `groupInTwoProposals` du serveur) tombent sous « numéro pris par une autre proposition acceptée », le compte visé
   étant désigné par son numéro ; au niveau **avertissement** : racine, deux noms de tête différents réunis sous un préfixe (un bloc de dix qui
   chevauche une dizaine), réutilisation qui rend le compte non imputable ; au niveau **information** : comptes déjà
   dans le groupe, devises mêlées (« le groupe s'affichera en CHF », 46-1 AC-6), comptes de même préfixe non cochés,
   comptes seuls de leur préfixe. Le même module construit le corps de la demande (fiche mère) depuis les seules
   propositions **acceptées et sans erreur**, et l'arbre de l'aperçu. Aucune règle n'est recopiée dans le composant.
   *(Amendé le 2026-11-23, revue de code P1, L5 : la fiche disait « numéro d'un compte de la sélection ». Le numéro d'un
   compte membre d'une **autre** proposition n'est pas refusé à ce titre ; si les deux propositions sont acceptées, il
   tombe sous « compte d'un `rattacher` ou d'un `réutiliser` membre d'une autre proposition acceptée » (`groupIsMember`),
   comme au serveur (`check_groups_shape`). Même revue, M3 : un parent choisi devenu membre de sa proposition est remis
   à vide, d'où l'erreur « parent manquant ».)*

3. **AC-3 — Page.** Trois zones, comme la maquette : (1) **sélection** — le plan en arbre, une case par compte non
   groupe, raccourcis « Tout le plan » et « Aucun » ; (2) **propositions** — choix du mode, une carte par proposition
   (numéro modifiable ; nom et parent modifiables pour un groupe **à créer** seulement — un rattachement ou une
   réutilisation ne change ni le nom, ni le type, ni le parent du compte visé, contrat commun ; coche « réutiliser » quand le numéro est un compte sans écriture ; membres
   décochables un par un, ce qui permet d'exclure d'un clic les comptes d'un type ; messages par niveau ; gestes
   **Accepter**, **Écarter**, **Revenir**) et le rappel des comptes seuls de leur préfixe ; (3) **aperçu de l'arbre** —
   le plan après confirmation, marques *nouveau*, *déplacé*, *devient groupe*, case « seulement les branches
   modifiées ». Pied : décompte (groupes à créer, comptes réutilisés, comptes à déplacer, propositions sans réponse) et
   **Confirmer**, inactif tant qu'aucune proposition n'est acceptée ou qu'une acceptée porte une erreur. Le bouton
   « Enregistrer mes choix » de la maquette **n'existe pas** dans le produit. Rien n'est écrit avant *Confirmer*.

4. **AC-4 — Désactivation de chaque proposition (remarque du 2026-11-22).** Chaque proposition se désactive seule
   (*Écarter*) et se réactive (*Revenir*), sans toucher les autres ; une proposition écartée n'entre ni dans l'aperçu ni
   dans la demande, et ses erreurs ne bloquent pas *Confirmer*. Changer de mode ou de sélection recalcule les
   propositions en **gardant** l'état et les modifications de celles qui subsistent (même clé de préfixe).
   *(Précisé le 2026-11-23, revue de code P1, M1 : seuls les champs que l'utilisateur a modifiés sont gardés, appliqués
   aux valeurs par défaut recalculées ; une proposition **acceptée** dont les membres changent repasse en attente ; une
   proposition écartée le reste.)*

5. **AC-5 — Confirmation.** *Confirmer* envoie la demande ; en `200`, un message dit combien de groupes ont été créés
   de comptes réutilisés et de comptes déplacés, et l'utilisateur revient à l'écran du plan, rechargé. En refus, **rien n'a été écrit** : la
   page garde la sélection et les propositions, et affiche le message de l'erreur avec la proposition et, s'il est
   donné, le compte que nomme l'erreur (`groupStep` des codes existants, `groupIndex`/`accountId` d'`ACCOUNT_GROUP_INVALID`) (`404 NOT_FOUND` d'un compte disparu et `409 ILLEGAL_STATE_TRANSITION` d'un compte archivé invitent aussi à
   recharger) ; `409 OPTIMISTIC_LOCK_CONFLICT` (compte modifié ou retypé depuis l'aperçu) invite à recharger le plan
   (les `version` sont relues). Les codes existants sont traduits par les messages qu'en donne déjà l'écran du plan (relever au T0 où ils vivent),
   `ACCOUNT_GROUP_INVALID` par motif (`reason`, onze messages), `groupStep` désignant la carte et le compte.
   *(Amendé le 2026-11-23, revue de code P1, M2 : *Recharger le plan* s'offre sur **tout** refus, et non sur trois codes ;
   bien des refus — `ACCOUNT_PARENT_HAS_ENTRIES`, `ACCOUNT_GROUP_INVALID` de `numberTaken`, `notAGroup`, `notReusable`,
   `groupTypeMismatch`, `RESOURCE_CONFLICT` — viennent aussi d'un plan changé depuis l'aperçu.)*
   *(Recontrôle de l'étape 0, 2026-11-22 : les onze messages d'`ACCOUNT_GROUP_INVALID` sont ceux que le serveur rend
   déjà traduits dans les quatre locales — `account_group_invalid_message`, `errors.rs`, « Proposition { $position } : … » —
   l'écran affiche donc `message` tel quel, sans recopier les onze textes, et désigne la carte par `details.groupIndex`
   et le compte par `details.accountId` ; pour les codes existants, `details.groupStep: { groupIndex, accountId? }`.
   L'écran du plan ne traduit lui-même que `OPTIMISTIC_LOCK_CONFLICT` (`accounts-error-stale`) et `RESOURCE_CONFLICT`
   (`accounts-error-number-exists`, rendu aussi par l'index unique sur un numéro que la collation tient pour égal) ;
   les autres codes portent un message traduit, affiché tel quel.)*

6. **AC-6 — i18n.** Toutes les chaînes dans un bloc propre `accounts-groups-*` des quatre locales ; `i18n-keys`,
   `i18n-libelle-en-dur` et `lint-i18n-ownership` verts ; `sitesTotal` **recompté** par la garde.

7. **AC-7 — Manuel.** `docs/manual/fr/user-manual.tex` : une sous-section après *Modification d'un compte*
   (`sec:modification-compte`), « Créer des groupes d'après les numéros » : les deux modes, les trois cas d'un numéro
   pris, l'exclusion des types mêlés, *Accepter* / *Écarter* / *Revenir*, l'aperçu, le **tout ou rien**, l'ordre pour
   deux niveaux (les liquidités d'abord, puis une banque par groupe), le déplacement d'un groupe existant laissé à
   *Modifier le compte*, et que la fonction n'est pas ouverte aux clés d'API. `.tex` seul, aucun PDF commité.

8. **AC-8 — Tests et captures.**
   - Vitest du calcul pur (`group-proposals.test.ts`), un cas au moins par règle de l'AC-2, dont les cas de la
     maquette : `10200`/`10201` → `1020` mouvementé refusé puis renommé `102` ; `10400`/`10401` → `1040` sans écriture,
     erreur puis réutilisation sur coche ; `1081` seul → rattachement à `108` ; `1050`, `1051`, `1058` (trou, sans
     erreur) ; `1068`–`1070` (bloc qui chevauche, avertissement) ; `6900`, `6901`, `6905` types mêlés, valides après
     exclusion de `6905` ; mode « un seul groupe » sur `1000`, `1010`, `1020` → `100` sous `10`, et `100` déjà pris →
     numéro vide ; numéro d'un compte archivé → refus ; groupe existant mouvementé → refus ; parent mouvementé → refus ; corps de la demande sans les propositions écartées ni en attente.
   - Vitest de la page (motif `accounts-page.test.ts`) : *Confirmer* inactif, puis actif ; *Écarter* et *Revenir* ;
     erreur du serveur affichée sans perdre l'état.
   - E2E neuf `frontend/tests/e2e/accounts-groups.spec.ts` : création de deux groupes et rangement (l'arbre de l'écran
     du plan les montre, `GET /accounts` les confirme par l'API : `parentId`, `postable`) ; refus d'un numéro de compte
     mouvementé à l'aperçu ; une proposition écartée n'est pas créée ; accès refusé au rôle Consultation (redirection).
   - Mutations sur le calcul pur (une à la fois, copie de `frontend/` à son propre `node_modules`) : règle du dernier
     chiffre, exclusion des comptes de groupe, refus des types mêlés, filtre des propositions acceptées.
   - Captures (thème clair) de chaque état touché : bouton de l'écran du plan, page vide, propositions avec chacun des
     trois cas, proposition écartée, mode « un seul groupe », erreur du serveur, plan après confirmation —
     dans `kesh-gate-logs/s46-5b/captures/`, jointes au Dev Agent Record et au prompt de la revue.

## Tâches / sous-tâches

- [x] T0 — Relire la 46-5a livrée (forme exacte de la demande, de la réponse et des erreurs) et l'écran du plan.
- [x] T1 — `group-proposals.ts` et ses tests (AC-2).
- [x] T2 — Page, bouton, appel d'API (AC-1, AC-3, AC-4, AC-5) ; i18n (AC-6).
- [x] T3 — E2E, mutations, captures (AC-8) ; manuel (AC-7) ; grep du symptôme sur les quatre locales et le manuel.

## Dev Notes

- **Maquette de référence** : `kesh-gate-logs/s46-5/maquette/maquette-46-5.html` (hors dépôt, plan synthétique) ; ses
  fonctions `buildProposals` et `check` sont le point de départ du calcul pur, à réécrire en TypeScript typé et testé,
  sans son plan, sans « Enregistrer mes choix » ni questions.
- **« Mouvementé »** : `currencyLocked` de `GET /accounts` (`accounts::currency_locked_account_ids` : au moins une
  ligne d'écriture, archivés compris). L'aperçu peut dater ; le serveur rejuge sous verrou (46-5a).
- **Fichiers communs** avec l'Epic 47 (47-2, écran du plan) : `+page.svelte`, `features/accounts/` — blocs distincts ;
  catalogues i18n : garder les deux côtés ; `sitesTotal` recompté.

## Note de version

- **Added** — Plan comptable : **Créer des groupes**. Cochez des comptes : Kesh propose un compte de groupe par numéro
  sans son dernier chiffre, ou un seul groupe pour toute la sélection ; vous acceptez, corrigez ou écartez chaque
  proposition, voyez l'arbre qui en résultera, puis confirmez — tout est créé et rangé en une fois, ou rien.

## Dev Agent Record

### Agent Model Used

Claude Opus (agent de développement), contexte unique, sur `story/46-5` après la clôture de la 46-5a (`ef94fe01a`).

### Debug Log References

Journaux hors dépôt, `kesh-gate-logs/s46-5b/` : `gate-backend-g3.log` (gate backend du dernier commit de code),
`front-*.log` (gate frontend), `e2e-2.log` et `e2e-backend-2.log` (E2E de la spec neuve), `mutations.console.log`,
`manuel.log` / `manuel.txt` (PDF compilé dans une copie), `captures/*.png`.

### Completion Notes List

- **Étape 0** (T0) — fiche recontrôlée contre le code de la 46-5a : `GroupRequest` (`routes/accounts.rs:323`, `kind`
  `create`/`attach`/`reuse`, `parentId` absent ou `null` = racine, `accounts[].{id, version}`), réponse
  `{ created, reused, moved }`, `details.groupStep: { groupIndex, accountId? }` (`group_step_refusal`), et
  `ACCOUNT_GROUP_INVALID` dont les onze messages sont déjà traduits par le serveur (`account_group_invalid_message`,
  « Proposition { $position } : … »). Amendements portés à la fiche : l'écran affiche `message` tel quel (aucune
  recopie des onze textes), `tooMany` jugé au niveau de la demande, nom et parent modifiables pour un groupe à créer
  seulement (contrat commun), `groupInTwoProposals` couvert par « numéro demandé par une autre proposition
  acceptée », captures dans `kesh-gate-logs/s46-5b/captures/`, développement sur `story/46-5`.
- **Calcul pur** (T1, AC-2) — `features/accounts/group-proposals.ts` : `proposalSkeletons` (deux modes, comptes seuls de
  leur préfixe, groupe existant dont tous les comptes sont déjà les sous-comptes : aucune proposition),
  `checkProposals` (seize codes d'erreur, trois avertissements, quatre informations ; contrôles entre propositions
  acceptées seulement), `buildGroupsRequest` (avec la clé de chaque rang), `proposalSummary`, `previewTree`,
  `parentOptionsFor`, `readGroupRefusal`. Numéros comparés sans casse comme le serveur ; longueurs en octets UTF-8.
- **Page** (T2, AC-1, AC-3 à AC-5) — `routes/(app)/accounts/groups/` : trois zones, pied collant (décompte,
  *Confirmer*), garde de rôle (`+page.ts`, redirection vers `/accounts`), bouton *Créer des groupes* de l'écran du
  plan (mêmes rôles). Refus : message (codes existants traduits comme à l'écran du plan, `OPTIMISTIC_LOCK_CONFLICT` et
  `RESOURCE_CONFLICT`), proposition et compte désignés, *Recharger le plan* pour `OPTIMISTIC_LOCK_CONFLICT`,
  `NOT_FOUND`, `ILLEGAL_STATE_TRANSITION` ; l'état (sélection, mode, choix) est gardé.
- **i18n** (AC-6) — 72 clés `accounts-groups-*` dans les quatre locales (recomptées : `grep -c '^accounts-groups-'`,
  72 par catalogue) ; compteurs relevés par les gardes, non additionnés : `sitesTotal` 3517 → 3599,
  `CANDIDATES_ATTENDUES` 132 → 135 (`conforme` 111 → 114). Exemple de la règle pris sur des comptes des plans livrés
  (1170, 1171 → 117) après le rouge de la garde G4-bis (`textes_coherents`, premier gate g2), dans les quatre locales,
  le repli et le manuel.
- **Manuel** (AC-7) — `user-manual.tex`, `\subsection{Créer des groupes d'après les numéros}` (`sec:creer-groupes`)
  après *Modification d'un compte* ; `.tex` seul. PDF compilé dans une copie hors dépôt (`latexmk -xelatex` rc 0,
  aucune référence indéfinie au dernier passage), aplati et grepé : les sept jetons attendus présents.
- **Tests** (AC-8) — Vitest : `group-proposals.test.ts` **36** tests (cas de la maquette, chaque règle de l'AC-2),
  `groups-page.test.ts` **5** tests ; E2E neuf `tests/e2e/accounts-groups.spec.ts`, **2** tests. Décomptes de
  `ef94fe01a` à `HEAD`. Mutations hors dépôt (copie de `frontend/` à son propre `node_modules`, une à la fois, témoin
  41/41 vert) : **7 tuées sur 7**, chacune par une assertion nommée — règle du dernier chiffre, comptes de groupe
  membres, types mêlés, filtre des acceptées, mode « un seul groupe » (`100` pris), préfixe d'un seul compte, *Confirmer*
  malgré une erreur (`mutations.console.log`).
- **Gates au dernier commit de code** (`6862fe31c`) : `scripts/test-fast.sh --ci` (base remise à zéro ;
  `KESH_NO_MEMGUARD=1`, le bus systemd utilisateur refusant la connexion au passage g1, sans résultat) — fmt, clippy
  workspace verts, nextest **6283 exécutés, 6283 passés, 14 ignorés** (1 instable au premier essai,
  `admin_auto_backup_e2e::disque_sature_pendant_l_ecriture`, passé au second) ; frontend : `check` 0 erreur,
  `lint-i18n-ownership` vert, Vitest **4783 passés (285 fichiers)**, `build` vert. Le passage g2 (au commit
  `388a832ad`) avait rendu 1 échec, G4-bis (exemple 1030 absent des plans livrés), corrigé par `6862fe31c`.
  E2E strictement partiel (règle de la nuit du 22.11) : la seule spec ajoutée, `accounts-groups.spec.ts` —
  **2/2 passés** au dernier commit de code (frontend et binaire reconstruits, `e2e-2.log`), sept captures
  (thème clair) dans `kesh-gate-logs/s46-5b/captures/` : bouton de l'écran du plan, page vide, propositions avec les
  trois cas d'un numéro pris, proposition écartée et acceptées, mode « un seul groupe », erreur du serveur, plan après
  confirmation. Pas de `gate-ci.sh` (fin d'epic).

### File List

- `frontend/src/lib/features/accounts/group-proposals.ts` (neuf), `group-proposals.test.ts` (neuf)
- `frontend/src/lib/features/accounts/accounts.api.ts` (`applyGroups`)
- `frontend/src/routes/(app)/accounts/groups/+page.svelte`, `+page.ts`, `messages.ts`, `groups-page.test.ts` (neufs)
- `frontend/src/routes/(app)/accounts/+page.svelte` (bouton *Créer des groupes*)
- `frontend/src/lib/shared/i18n-keys.test.ts`, `i18n-libelle-en-dur.test.ts` (compteurs)
- `frontend/tests/e2e/accounts-groups.spec.ts` (neuf)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl` (bloc `accounts-groups-*`)
- `docs/manual/fr/user-manual.tex`

## Change Log

- 2026-11-22 — Fiche écrite sur la décision du responsable de l'epic (fiche mère), découpage de la 46-5 ; validation
  complète à suivre (`kesh-gate-logs/s46-5/validation/`).
- 2026-11-22 — Validation P1 à P3 : remédiations portées ici (page sur la liste archivés compris, groupe et parent
  mouvementés refusés à l'aperçu, `+page.ts` de redirection, `tooMany`, `groupStep`, longueurs en octets) ; tendance
  et détail au Change Log de la fiche mère.
- 2026-11-22 — Validation P4 (ciblée, Sonnet) et P5 (ciblée, Opus) : remédiations portées ici ; validation close au
  plafond de cinq passes, avec recontrôle obligatoire à l'étape 0 du développement (Change Log de la fiche mère).
- 2026-11-23 — **Développement** (`bmad-dev-story`) sur `story/46-5`, après la clôture de la 46-5a : étape 0 (recontrôle
  de la fiche contre la 46-5a, amendements portés aux AC-2, AC-3, AC-5 et AC-8), calcul pur, page, bouton, i18n des
  quatre locales, manuel, tests (36 + 5 Vitest, 2 E2E), sept mutations tuées ; commits `388a832ad` (écran) et
  `6862fe31c` (exemple de la règle sur des comptes des plans livrés, garde G4-bis). Gates et E2E au Dev Agent Record.
  Statut `review` ; prompt de la revue P1 : `kesh-gate-logs/s46-5b/revue-p1-prompt.md`.
- 2026-11-23 — **Revue de code P1** (complète, Opus, contexte frais) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 6 LOW
  (`kesh-gate-logs/s46-5b/revue-p1-rapport.md`). Remédiation, commit `aec15733a` :
  - **M1** — `edits` ne garde plus que les champs que l'utilisateur a touchés (`ProposalPatch`, rangés par
    `applyEdit`), appliqués par `editOf` sur les valeurs par défaut recalculées ; une acceptation note ses membres
    (`acceptedFor`), et une proposition acceptée dont les membres changent repasse en attente (une écartée le reste).
    AC-4 précisé.
  - **M2** — *Recharger le plan* s'offre sur tout refus (champ `reload` retiré) ; le manuel le disait pour un compte
    modifié, il le dit désormais pour tout refus, écriture reçue comprise. AC-5 amendé. *(La phrase du Dev Agent
    Record « Recharger le plan pour `OPTIMISTIC_LOCK_CONFLICT`, `NOT_FOUND`, `ILLEGAL_STATE_TRANSITION` » vaut au
    commit `6862fe31c`.)*
  - **M3** — choix le plus simple : un parent choisi devenu membre de sa proposition est **remis à vide** dans
    `editOf` ; l'écran montre « (à choisir) », l'erreur `parentMissing` bloque *Accepter*, rien ne part.
  - **L3** — renvois de la JSDoc vers `routes/(app)/accounts/groups/messages.ts`. **L5** — AC-2 amendé
    (« numéro d'un compte de la même proposition »). **L6** — test de page : écartée après un aller-retour de mode,
    puis après *Recharger le plan*.
  - **L1, L4** laissés, versés à la liste des issues à ouvrir de l'epic (enhancement, P4) ; **L2** déjà connu (46-5a, L2).
  - Manuel (`.tex`) : choix gardés champ par champ, acceptée remise en attente, parent remis « à choisir », *Recharger
    le plan* après tout refus. Aucune clé i18n ajoutée ni changée (pas de `textes_coherents` à rejouer).
  - Tests ajoutés (de `0b3f11487` à `aec15733a`) : **5** Vitest du calcul pur, **2** de page (dont l'attente de
    `account-groups-reload` sur `ACCOUNT_GROUP_INVALID` inversée, M2). Mutations (`kesh-gate-logs/s46-5b/p1-mut/`,
    une à la fois, témoin vert) : **4 tuées sur 4** par des assertions nommées — sans remise en attente, parent membre
    gardé, bouton limité à un code, choix effacés au rechargement.
  - Gates au commit `aec15733a` : `npm run check` 0 erreur (29 avertissements, comme avant, aucun dans
    `accounts/groups`), `lint-i18n-ownership` vert, Vitest **4790 passés (285 fichiers)**, `build` vert
    (`front-p1-*.log`) ; E2E strictement partiel, `accounts-groups.spec.ts` **2/2 passés**, frontend et binaire
    reconstruits (`e2e-3.log`, captures rafraîchies). Aucun code Rust touché : backend non rejoué.
  - Prompt de la passe P2 (complète, Sonnet, braquée sur la remédiation) : `kesh-gate-logs/s46-5b/revue-p2-prompt.md`.

### 2026-11-23 — Clôture
Revue de code : P1 (Opus) 0 CRITICAL / 0 HIGH / 3 MEDIUM / 6 LOW → remédiation `aec15733a` → P2 (Sonnet, passe complète) 0/0/0/4, boucle close sur une passe sans finding au-dessus de LOW. Les quatre LOW de la P2 (parent remis à vide plutôt qu'au défaut recalculé ; ré-acceptation implicite non écrite au manuel ; « dans la sélection » à préciser au manuel ; clé `one` partagée entre sélections) sont versés en issues, à ouvrir avec la 46-5. Gates : ceux du dernier commit de code `aec15733a` (frontend complet, E2E de la spec) ; backend à `6862fe31c`, aucun code Rust changé depuis.

### 2026-11-24 — Demandes du mainteneur à l'approbation de la 46-5 (P2 L2 et L3 corrigés dans la story)
- **Manuel** (`user-manual.tex`, `sec:creer-groupes`) : la remise en attente vaut pour un changement de la
  **sélection** (`editOf`, `group-proposals.ts` : `acceptedFor !== membersSignature(skeleton.members)`) ; si la sélection
  revient exactement aux comptes acceptés, la proposition **redevient acceptée** sans nouveau clic, champs gardés (même
  ligne ; `applyEdit` pose `acceptedFor` à l'acceptation) ; décocher un compte dans la proposition la laisse acceptée
  (`+page.svelte`, `toggleMember` → `update(p, { excluded })`, qui ne touche ni `state` ni `acceptedFor`). PDF compilé
  hors dépôt et grepé aplati (`kesh-gate-logs/s46-5b/manuel-p3.*`), non commité.
- **`docs/api-external.md`** : `POST /accounts/groups` disait déjà « tout ou rien » et « ce n'est pas le motif
  `{ accepted, failed }` des lots, et c'est voulu » ; ajouté le motif — un plan à moitié rangé ferait mentir l'aperçu
  que l'utilisateur a confirmé. Le commentaire de la route (`routes/accounts.rs`, `apply_groups`) le disait déjà : inchangé.
- Entrée « (46-5b, revue P2 L2+L3) » de la liste des issues à ouvrir barrée : corrigée dans la story.
- Texte seul, aucun code changé. Gates ciblés : `textes_coherents` et `npm run check` (`kesh-gate-logs/s46-5b/p3-*.log`).
  Prompt de la passe ciblée P3 : `kesh-gate-logs/s46-5b/revue-p3-ciblee-prompt.md`.
