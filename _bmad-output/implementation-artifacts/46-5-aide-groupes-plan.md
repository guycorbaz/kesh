# Story 46-5 : aide à la création des groupes d'un plan existant

Status: split

Epic : `_bmad-output/planning-artifacts/epic-46-rapports-groupes.md` — D12, FR-46-5, NFR-46-5 ; Q9 (maquette avant
décision). Release visée **0.21.0** (prévision). **Aucune migration.** Esquissée le 2026-11-18, reprise le 2026-11-20,
**décidée le 2026-11-22** (section ci-dessous), **découpée** en **46-5a** (transaction et route) et **46-5b** (écran,
calcul des propositions, manuel) par la règle de splitting préventif.

> ⚠️ **Confirmation du chef de projet attendue.** Guy doit encore confirmer la décision du 2026-11-22. La story se
> développe sur sa propre branche (`story/46-5`, d'où partent et où reviennent `story/46-5a` et `story/46-5b`) et
> **n'est fusionnée dans `epic/46-rapports-groupes` qu'après son accord**. Sans cet accord, rien n'entre dans l'epic ;
> si Guy refuse, la story sort de l'epic (tableau, ordre, FR-46-5 et fichiers communs du fichier d'epic à corriger).

## Story

En tant que comptable qui range un plan existant,
je veux que Kesh me propose des comptes de groupe d'après les numéros des comptes que je choisis, et que je confirme,
afin de structurer mon plan (une banque = un groupe, puis les liquidités) sans créer chaque groupe et déplacer chaque
compte à la main.

## Décision du 2026-11-22

Le responsable de l'epic a tranché sur la maquette (`kesh-gate-logs/s46-5/maquette/`, hors dépôt : `maquette-46-5.html`,
trois captures, `questions-decision.md`) ; ses réponses Q1 à Q10 sont **toutes les recommandations (a)**, avec une
remarque :

| Question | Décision |
|---|---|
| Q1 — faire la story ? | **Oui, telle que maquettée** : sélection, propositions, mode « un seul groupe », aperçu de l'arbre, confirmation. |
| Q2 — règle de proposition | **Numéro sans son dernier chiffre**, **plus** un mode **« un seul groupe pour la sélection »** (numéro = plus long préfixe commun ; si ce préfixe est le parent commun, ce préfixe suivi de `0`, s'il est libre). Numéro modifiable dans les deux modes. |
| Q3 — atomicité | **Une transaction, tout ou rien** : variantes `_in_tx` de `accounts::create` et `accounts::update`, appelées par les fonctions actuelles, enchaînées par une route neuve dans un seul `begin()`. |
| Q4 — numéro déjà pris | **Trois cas** : groupe existant → rattacher ; compte sans écriture → le réutiliser comme groupe sur coche expresse ; compte mouvementé → refus, autre numéro à saisir. |
| Q5 — types mêlés | **Refus de la proposition, avec exclusion d'un clic** (décocher les comptes d'un type). |
| Q6 — préfixe d'un seul compte | **Aucun groupe proposé** (le mode « un seul groupe » permet de le faire) ; exception : préfixe qui est un groupe existant → rattachement. |
| Q7 — parent du groupe | **Parent commun par défaut**, modifiable ; sans parent commun, à choisir ; racine permise avec avertissement. |
| Q8 — sur quoi proposer | **Sur la sélection cochée** ; « Tout le plan » n'est qu'un raccourci de sélection. |
| Q9 — où vit l'outil | **Écran du plan seulement**, pour les rôles qui modifient déjà le plan ; **ni clé d'API ni MCP**. |
| Q10 — audit | **L'audit existant** de la création et de la modification, une entrée par compte créé ou déplacé. |

**Remarque du responsable de l'epic** (sur Q2) : **chaque proposition de groupement peut être désactivée
individuellement** (écartée), et réactivée. Une proposition écartée n'entre ni dans l'aperçu ni dans la demande.

Conséquences inscrites ici et dans les deux sous-fiches :

- **À confirmer par Guy avec la décision** : Q9 se lit « ni clé d'API » ; mais sa conséquence écrite (« la route suit
  les droits des routes du plan existantes ; aucun accès par clé n'est ouvert de plus ») est ambiguë, ces routes étant
  ouvertes aux clés `read-write`. La fiche retient la lettre de la décision : **route fermée aux clés**.

- **Q9 et `docs/api-external.md`.** Les routes du plan (`POST /accounts`, `PUT /accounts/{id}`) sont ouvertes aux clés
  `read-write` ; la route neuve, elle, **refuse toute clé** (`403 API_KEY_OPERATION_FORBIDDEN`, comme la suppression
  d'un import). `docs/api-external.md` § 7 énumère « quatre exceptions » fermées aux clés : la route en devient la
  cinquième, et la phrase change (46-5a). La conséquence « aucun changement de `docs/api-external.md` » de
  `questions-decision.md` est donc **corrigée** : elle supposait la route ouverte aux clés comme celles du plan.
- **Q3, motif des lots.** La route est **tout ou rien** : une erreur refuse toute la demande, rien n'est écrit. Ce
  n'est **pas** le motif `{ accepted, failed }` des endpoints batch (`CLAUDE.md` § *Pattern batch*), et c'est voulu :
  un plan à moitié rangé dans des livres en production ferait mentir l'aperçu. Aucune revue ne doit l'exiger.
- **Q3, `kesh-db`.** La 46-5a touche un repository : **gate complet même en boucle de revue** (`CLAUDE.md` §
  *Exception `kesh-db`*).

## Découpage (règle de splitting préventif)

Modules touchés par la story entière — **six** au grain des crates et dossiers de premier niveau :
`kesh-db` (variantes `_in_tx`, fonction de groupement, erreur), `kesh-api` (route, `errors.rs`, `lib.rs`, registre
`tests/audit_route_registry.rs`), `kesh-i18n` (quatre locales), `frontend` (calcul des propositions, page), les
**manuels** (`docs/manual`, utilisateur et administration) et la **documentation technique** (`docs/api-external.md`,
`docs/MULTI-TENANT-SCOPING-PATTERNS.md`). Cinq si `docs/` compte pour un seul dossier : le découpage se justifie alors
aussi par la taille des stories, que `CLAUDE.md` § *Numérotation des versions* (dernière puce) place avant tout, et par
l'exception `kesh-db`. **Plus de cinq**, d'où le découpage, qui sépare aussi le travail soumis à
l'exception `kesh-db` du travail d'écran.

| Clé | Titre | Modules | Dépend de | Fiche |
|---|---|---|---|---|
| 46-5a | Groupes du plan : transaction et route | `kesh-db`, `kesh-api`, `kesh-i18n` (messages d'erreur), manuel d'administration, documentation technique (`api-external.md`, `MULTI-TENANT-SCOPING-PATTERNS.md`) — cinq | — | `46-5a-aide-groupes-transaction-route.md` |
| 46-5b | Groupes du plan : propositions, écran, manuel | `frontend`, `kesh-i18n` (textes de l'écran), `docs/manual` — trois | 46-5a | `46-5b-aide-groupes-ecran.md` |

Ordre : 46-5a puis 46-5b, sur `story/46-5`. La 46-7 (solde de l'epic) est déjà `done` : la dette des 46-5a et 46-5b
ne va pas à une story de solde — un LOW de revue qui touche la production se corrige dans la story (boucle rouverte
sur une passe ciblée), sinon il s'ouvre en issue ; puis, **après l'accord de Guy**, fusion de `story/46-5` dans
`epic/46-rapports-groupes`, suivie de la revue indépendante des défauts d'interaction (avec la 46-7 et la 47-2/47-3 si
`epic/47` y est entré entre-temps : mêmes fichiers `routes/accounts.rs`, `repositories/accounts.rs`, écran du plan).

## Le contrat commun (que les deux sous-fiches citent)

### Vocabulaire

- **Compte de groupe** : un compte actif qui a au moins un sous-compte actif (`has_active_children`) — la définition
  de la 46-1 (D1). Il n'est pas imputable (`effective_postable`).
- **Proposition** : un groupe à créer ou à réutiliser, et les comptes à y ranger. Trois natures : **créer** (numéro
  libre), **rattacher** (le numéro est un compte de groupe existant), **réutiliser** (le numéro est un compte actif
  sans écriture et sans sous-compte, que l'utilisateur a choisi expressément de faire devenir groupe).
- **Comptes membres** : les comptes de la sélection qui vont sous le groupe ; jamais un compte de groupe (les comptes
  de groupe ne se cochent pas, ils servent de parent).

### Demande — `POST /api/v1/accounts/groups` (46-5a)

```json
{
  "groups": [
    { "kind": "create", "number": "103", "name": "Banque A", "parentId": 12,
      "accounts": [ { "id": 31, "version": 4 }, { "id": 32, "version": 1 } ] },
    { "kind": "attach", "accountId": 44, "accounts": [ { "id": 47, "version": 2 } ] },
    { "kind": "reuse",  "accountId": 40, "accounts": [ { "id": 41, "version": 3 }, { "id": 42, "version": 3 } ] }
  ]
}
```

- `create` : `number` et `name` aux règles de `POST /accounts` (non vides après `trim`, 10 et 255 octets au plus, comme aujourd'hui) ;
  `parentId` facultatif (`null` = racine). Type du groupe **déduit** de ses membres (tous du même type, sinon refus) ;
  devise principale (`currency = NULL`), aucun rôle, non imputable.
- `attach` / `reuse` : `accountId` du compte qui devient ou est déjà le groupe ; **son nom, son type et son parent ne
  changent pas** (ils se changent à l'écran du plan, #1351).
- `accounts[].version` : la `version` lue à l'aperçu, comparée dans la passe de verrous en tête de la transaction
  (46-5a, AC-2), avant tout contrôle d'état → `409 OPTIMISTIC_LOCK_CONFLICT` si le compte a changé depuis (retypé
  compris) ; le contrôle d'`update`, sous le même verrou, n'est plus qu'un filet.

### Réponse

`200` : `{ "created": [Account…], "reused": [Account…], "moved": [Account…] }` — les groupes créés, les comptes
réutilisés comme groupe (devenus non imputables) et les comptes réellement déplacés, au format de `GET /accounts` (le
compte déjà sous son groupe n'est ni déplacé ni audité : court-circuit no-op d'`update`). Un groupe rattaché, déjà
groupe, ne change pas et n'y figure pas.

### Refus — tout ou rien

Toute erreur annule la transaction entière : aucun compte créé, déplacé ni rendu non imputable, aucune entrée d'audit.
Codes :

- les **codes existants** de `POST /accounts` et `PUT /accounts/{id}` pour l'étape qui échoue, inchangés
  (`ACCOUNT_PARENT_HAS_ENTRIES`, `ACCOUNT_PARENT_CYCLE`, `ACCOUNT_PARENT_ARCHIVED`, `OPTIMISTIC_LOCK_CONFLICT`,
  `ILLEGAL_STATE_TRANSITION` (membre archivé),
  `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`, `ACCOUNT_FOREIGN_PARENT_HAS_ENTRIES`, `NOT_FOUND`…), avec **`details.groupStep: { groupIndex, accountId? }`**
  (rang de la proposition dans `groups` et, quand l'étape porte sur un membre, son `id`) — le reste des `details` du
  code inchangé ;
- un code neuf, **`400 ACCOUNT_GROUP_INVALID`**, pour ce que ni `create` ni `update` ne contrôlent, avec
  `details: { groupIndex, reason, accountId? }` et `reason` parmi : `mixedTypes` (membres de types différents),
  `groupTypeMismatch` (groupe rattaché ou réutilisé d'un autre type que ses membres), `numberTaken` (`create` sur un
  numéro déjà porté, archivé compris, ou demandé par deux créations de la même demande ; le doublon de la demande se
  compare **sans casse, pas sans accents** (`10x` après `10X` rend `numberTaken`) ; `10é` après `10e`, que la collation
  `utf8mb4_unicode_ci` de `accounts.number` tient pour égaux, passe ce contrôle et bute sur l'index unique
  `uq_accounts_company_number` : `409 RESOURCE_CONFLICT` (`DbError::UniqueConstraintViolation` côté base) avec `groupStep`, rien d'écrit — revue P2 de la
  46-5a, L2), `notAGroup` (`attach` sur un compte sans sous-compte actif ou archivé), `notReusable` (`reuse` sur un compte qui a des sous-comptes actifs ou n'est pas actif ;
  ses écritures sont jugées sous verrou par `update`, ci-dessous), `accountInTwoGroups` (un membre deux fois dans la demande, dans une même proposition ou dans deux), `groupIsMember` (le compte d'un `attach` ou d'un
  `reuse` est aussi membre d'une proposition de la même demande — un niveau à la fois), `groupInTwoProposals` (le
  compte d'un `attach` ou d'un `reuse` déjà visé par une proposition précédente : deux `attach`, deux `reuse`, ou l'un
  et l'autre — ajouté à la revue P1 de la 46-5a), `memberIsOwnGroup`
  (un membre est le compte même de l'`attach` ou du `reuse` de sa proposition), `empty` (proposition sans membre), `tooMany` (plus de 100 propositions ou de
  1000 membres dans la demande) ;
- un **compte réutilisé qui a reçu une écriture** entre l'aperçu et la confirmation est refusé par le contrôle sous
  verrou d'`update` (`ACCOUNT_PARENT_HAS_ENTRIES`), non par l'aperçu ;
- **le parent d'un groupe créé** passe le contrôle du `PUT` (#1351, refus d'un parent mouvementé,
  `ACCOUNT_PARENT_HAS_ENTRIES`), **non** celui, plus large, de la création (qui l'accepte : écart A-1 de
  `fix-api-parent-compte`) ;
- un identifiant (membre, compte d'un `attach` ou d'un `reuse`) inconnu ou d'une **autre société** → `404 NOT_FOUND`
  avec `groupStep`, jugé avant tout autre contrôle sur ce compte (jamais `mixedTypes`, `notAGroup` ni
  `notReusable` pour un compte étranger) ; un `parentId` de `create` inconnu, étranger ou archivé → `400
  VALIDATION_ERROR`, par le contrôle de forme de `POST /accounts` (`check_parent_in_company`) ;
- `403 API_KEY_OPERATION_FORBIDDEN` pour une clé `read-write` (une clé `read` est arrêtée avant, en `403
  API_KEY_READ_ONLY`, par la couche d'authentification commune) ; `403` du rôle Consultation (couche des routes Comptable+).

### Règle des propositions (46-5b)

- **Mode « numéro sans son dernier chiffre »** (par défaut) : chaque compte membre de la sélection de numéro long d'au
  moins deux caractères va au groupe de numéro `numéro[..len-1]` (numéro sans son dernier caractère) ; un préfixe qui ne réunit qu'**un** compte ne donne
  **aucune** proposition, sauf si ce préfixe est un compte de groupe existant (rattachement).
- **Mode « un seul groupe pour la sélection »** : une seule proposition, numéro = plus long préfixe commun des numéros ;
  si ce préfixe est le numéro du parent commun (`10`), Kesh propose ce préfixe suivi de `0` (`100`) s'il est libre,
  sinon un numéro vide à saisir.
- **Nature** selon le numéro (modifiable), jugée sur la liste **archivés compris** (`GET /accounts?includeArchived=true`,
  les archivés n'étant ni affichés ni sélectionnables) : libre → créer ; compte de groupe existant → rattacher, sauf
  s'il porte des écritures (refus : le serveur le refuserait comme parent mouvementé) ; compte sans écriture ni
  sous-compte → refus tant que la coche « réutiliser » n'est pas cochée (décochée par défaut), puis réutiliser ;
  compte mouvementé (`currencyLocked` de `GET /accounts`, qui vaut « porte au moins une ligne d'écriture ») ou archivé →
  refus, autre numéro à saisir.
- **Nom** proposé : les mots de tête communs aux noms des membres (« Banque A »), modifiable ; à saisir s'il est vide.
- **Parent** : le parent commun des membres, modifiable ; sans parent commun, à choisir (erreur tant que vide) ;
  « racine » permise avec un avertissement ; un parent qui porte des écritures est une erreur (règle du `PUT`, #1351).
- **États** : chaque proposition est *en attente* à l'ouverture (rien d'office), puis *acceptée* ou *écartée*
  (désactivée), et *Revenir* la remet en attente ; seules les propositions **acceptées et sans erreur** entrent dans
  l'aperçu et dans la demande. *Confirmer* est inactif tant qu'une proposition acceptée porte une erreur ou qu'aucune
  n'est acceptée.
- **Pour ranger sur deux niveaux** (10 › 100 › 103 › comptes) : créer d'abord le groupe des liquidités en mode « un
  seul groupe » (`100` sous `10`), puis les groupes par banque, dont le parent commun est alors `100`. Déplacer un
  groupe existant reste un geste de l'écran du plan (#1351).

## Ce qui ne change pas

Aucune migration ; aucun outil MCP ; `POST /accounts` et `PUT /accounts/{id}` gardent leur contrat et leurs refus
(l'extraction `_in_tx` est sans effet visible, ce que les tests existants gardent) ; l'audit n'a aucune action neuve
(`account.created`, `account.updated`).

## Note de version

*(Écrite dans les sous-fiches : 46-5a et 46-5b.)*

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
- 2026-11-22 — **Décision du responsable de l'epic** sur la maquette (Q1 à Q10 : recommandations (a) ; remarque :
  chaque proposition peut être écartée). Fiche réécrite : contrat commun (demande, réponse, refus tout ou rien, règle
  des propositions), découpage en **46-5a** et **46-5b** (six modules), statut `split` ; confirmation de Guy attendue
  avant toute fusion dans l'epic. Les points « à trancher » de la validation allégée sont tranchés ici. Validation
  complète à suivre (P1 Sonnet ou Opus), rapports dans `kesh-gate-logs/s46-5/validation/`.
- 2026-11-22 — validation P1 (complète, Opus, lecture seule) : **0 C, 0 H, 10 M, 8 L**
  (`kesh-gate-logs/s46-5/validation/P1-rapport.md`). Remédiés : M1 (statut des sous-fiches `drafted` jusqu'à la fin de
  la validation), M2 (le no-op d'`update_in_tx` rend `Ok` sans `rollback`), M3 (garde du parent seul extraite de
  `guard_parent_change_in_tx`), M4 (pas d'enveloppe dans `DbError` : erreur propre d'`apply_groups`, rejeu sur `source`,
  `groupIndex` ajouté par le handler), M5 (lectures filtrées par société, `404` d'un compte étranger, `400` d'un
  `parentId` étranger, tests), M6 (aperçu : groupe existant et parent mouvementés refusés), M7 (aperçu sur la liste
  archivés compris), M8 (tous les sites de la fermeture aux clés : `api-external.md` § 4, 7, 9, 10, `admin-manual.tex`,
  commentaire d'`errors.rs`), M9 (fichier d'epic : lignes « conditionnelle », fichiers communs, ordre après la 46-7 et
  sort de la dette), M10 (`+page.ts` qui redirige) ; L1 (`API_KEY_READ_ONLY` d'une clé `read`), L2 (une passe de
  verrous en tête, ordre stable sur toute la demande), L3 (décompte des modules nuancé), L4 (membre deux fois dans la
  demande), L5 (réponse calculée en une requête), L6 (lecture de Q9 à confirmer par Guy), L7 (`reused` dans la
  réponse), L8 (bloc i18n `accounts-groups-*` au fichier d'epic).
- 2026-11-22 — validation P2 (complète, Sonnet, contexte frais) : **0 C, 0 H, 4 M, 12 L**
  (`kesh-gate-logs/s46-5/validation/P2-rapport.md`). Remédiés : M1 (rejeu : la fermeture de `retry_app_on_deadlock`
  convertit l'interblocage en `AppError::Database`, le reste en variante neuve d'`AppError` rendue par la correspondance
  existante ; `groupStep` ajouté aux `details`), M2 (registre `audit_route_registry.rs` et tableau des verrous de
  `MULTI-TENANT-SCOPING-PATTERNS.md` au T3), M3 (garde du parent généralisée par `moved: Option<&Account>`, ordre
  inchangé, au lieu d'un découpage), M4 (mutations observables) ; LOW remédiés : phrase dupliquée, `groupStep` au lieu
  d'`accountId` au premier niveau, verrou d'intervalle noté, tests de motifs ajoutés, `tooMany` à l'écran, source des
  messages à relever au T0, redirection sans faux renvoi au rechargement, longueurs en octets, `reuse` aligné sur #1351
  écrit, décompte des modules expliqué. Non retenu : le prénom du chef de projet dans les fiches (la règle de dépôt vise
  le contributeur et sa société ; le chef de projet est nommé dans tout le dépôt).
- 2026-11-22 — validation P3 (complète, Opus, contexte frais) : **0 C, 0 H, 2 M, 9 L**
  (`kesh-gate-logs/s46-5/validation/P3-rapport.md`). Remédiés : M1 (`groupStep` par une variante
  `AccountGroupStep { error: Box<AppError> }` déléguée comme `ConversionEntry`, puis `refus_avec_detail` après le rejeu —
  la correspondance de `DbError` n'est pas touchée ; `ACCOUNT_GROUP_INVALID` non enveloppé), M2 (fichiers communs du
  fichier d'epic et liste des modules : registre des routes, `MULTI-TENANT-SCOPING-PATTERNS.md`, `kesh-db/src/errors.rs`,
  `admin-manual.tex`) ; LOW : classement `SansEcritureAuJournal` et liste (vi) recomptée, motif renommé
  `memberIsOwnGroup`, mutation du filtre de société rendue tuable, précédent `bank-import/manual/+page.ts`, longueurs en
  octets partout, renvoi à `CLAUDE.md` corrigé, contrat de la route au § 7 d'`api-external.md`, Change Logs des
  sous-fiches, `version` des membres jugée avant les contrôles d'état (`409` plutôt que `mixedTypes`).
- 2026-11-22 — validation P4 (passe ciblée, Sonnet, sur `3c07d8d8b`) : **0 C, 0 H, 1 M, 5 L**
  (`kesh-gate-logs/s46-5/validation/P4-rapport.md`) ; les trois questions posées (mécanisme `AccountGroupStep` et
  `refus_avec_detail`, `version` dans la passe de verrous, classement `SansEcritureAuJournal`) jugées justes au code.
  Remédiés : M1 (test du membre retypé → `409`, mutation de la comparaison de la passe de verrous, contrôle d'`update`
  déclaré filet) ; L1 (modèle de la redirection décrit tel qu'il est), L2 (seule l'arme d'`AccountGroupInvalid`
  s'ajoute), L3 (`groupStep` seulement avec un `group_index` connu), L4 (ordre 404 / archivé / 409 ; contrat de
  `version` de la fiche mère mis à jour), L5 (deux sites du décompte du registre, modules recomptés, longueur du nom à
  l'écran).
- 2026-11-22 — validation P5 (passe ciblée, Opus, sur `462e6fdcd`) : **0 C, 0 H, 2 M, 4 L**
  (`kesh-gate-logs/s46-5/validation/P5-rapport.md`) ; les trois réponses de P4 tiennent. Remédiés : M1 (le test du
  tout ou rien porte sur une erreur née après des écritures — `create` écrit, puis `reuse` d'un compte mouvementé —, et
  la mutation « commiter avant la dernière proposition » redevient tuable), M2 (ordre des refus de la passe de verrous
  limité aux membres ; comptes d'`attach` et de `reuse` inactifs → `notAGroup` / `notReusable`, tests par nature) ;
  L1 (`ILLEGAL_STATE_TRANSITION` aux codes et à l'écran), L2 (ordre « compte par compte »), L3 (la 46-5a compte cinq
  modules au grain de la fiche mère), L4 (Change Logs des sous-fiches).
- 2026-11-22 — **Validation close au plafond de cinq passes** (consigne de la tâche ; `CLAUDE.md` § *Review Iteration
  Rule*, clôture au plafond sur des MEDIUM corrigés). **Tendance** : P1 0/0/10/8 → P2 0/0/4/12 → P3 0/0/2/9 →
  P4 0/0/1/5 → P5 0/0/2/4 (C/H/M/L) ; aucun CRITICAL ni HIGH en cinq passes ; P2 à P5 n'ont trouvé, au-dessus de LOW,
  que des précisions de la mécanique (erreurs, verrous, tests) ou des effets des remédiations précédentes, aucun défaut
  de la décision. Modèles : P1 Opus (complète), P2 Sonnet (complète), P3 Opus (complète), P4 Sonnet (ciblée), P5 Opus
  (ciblée). **Condition de la clôture** : les deux MEDIUM de P5 ont été corrigés après la dernière passe permise, sans
  être relus ; **à l'étape 0 du développement de la 46-5a, le développeur recontrôle au code** l'ordre des refus de la
  passe de verrous (AC-2 : membres, puis comptes d'`attach`/`reuse`) et le test du tout ou rien (AC-7), et déclare ce
  recontrôle à son Change Log, ligne lue citée. Statut : 46-5a et 46-5b `ready-for-dev` ; **aucune fusion dans
  l'epic avant l'accord de Guy**.
- 2026-11-26 — **Accord du mainteneur, entrée dans la PR de l'epic.** Le mainteneur accepte la 46-5 : elle entre
  dans la PR de l'Epic 46 (branche `pr/46-rapports-groupes-v2`), pour que l'epic n'entre qu'une fois dans `main`. Diff repris de `story/46-5` depuis sa base dans l'epic (`dc8786541`), rien des
  Epics 38, 39 ou 40 ; conflits résolus en gardant les deux côtés (registre d'audit, `docs/api-external.md`, manuel
  d'administration, gardes i18n), compteurs recomptés. Les mentions « depuis la release qui suit la v0.19.0 » de la
  46-5a deviennent « Epic 46 » dans `docs/api-external.md` et « depuis la v0.20.0 » au manuel d'administration.
