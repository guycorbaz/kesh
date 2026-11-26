# Story 46-5a : groupes du plan — transaction et route

Status: done

Fiche mère : `46-5-aide-groupes-plan.md` (décision du 2026-11-22, **contrat commun** : demande, réponse, refus). Epic
46, D12, FR-46-5. Release visée **0.21.0** (prévision). **Aucune migration.** Développée directement sur `story/46-5` (consigne de
la tâche ; la branche `story/46-5a` prévue n'a pas été créée) ; **aucune fusion dans `epic/46-rapports-groupes` avant l'accord de Guy** (fiche mère).

## Story

En tant que comptable,
je veux que la création de plusieurs groupes et le rangement de leurs comptes se fassent en une seule opération, tout
ou rien, avec les contrôles de la création et du changement de parent,
afin qu'un rangement interrompu ne laisse jamais mon plan à moitié modifié.

## Modules touchés (règle de splitting préventif)

Cinq au grain de la fiche mère : `kesh-db/repositories/accounts` (variantes `_in_tx`, fonction de groupement, erreur neuve de `DbError`),
`kesh-api` (route et handler dans `routes/accounts.rs`, câblage dans `lib.rs`, correspondance dans `errors.rs`),
`kesh-i18n` (message du code neuf, quatre locales), le manuel d'administration (`docs/manual/fr/admin-manual.tex`), la
documentation technique (`docs/api-external.md`, `docs/MULTI-TENANT-SCOPING-PATTERNS.md`). (La fiche mère compte six modules pour la
story entière, les deux sous-fiches réunies.)

⚠️ **Exception `kesh-db`** (`CLAUDE.md`) : le patch touche un repository — **nextest complet même en boucle de revue**,
jamais un gate ciblé seul.

## Critères d'acceptation

1. **AC-1 — Variantes `_in_tx`, sans copie.** `accounts::create` et `accounts::update` deviennent
   `begin()` → `create_in_tx` / `update_in_tx` → `commit()` ; **tout le corps** (verrous de la société et des réglages
   de facturation, gardes, `INSERT`/`UPDATE`, normalisation de `postable`, copie des cours, audit) passe dans la
   variante, qui prend `&mut Transaction<'_, MySql>`. `update_in_tx` reçoit le `company_id` de l'appelant (la lecture
   `company_of_account` sur le pool reste dans `update`, hors transaction, comme aujourd'hui ; la variante garde la
   comparaison au `company_id` relu sous `FOR UPDATE` → `NotFound`). Les `tx.rollback()` du corps **qui précèdent un
   `return Err`** deviennent de simples retours d'erreur (la transaction de l'appelant s'annule à l'abandon) ; leur
   ordre et leurs erreurs ne changent pas. ⚠️ **Le court-circuit no-op d'`update` n'est pas une erreur** : son
   `tx.rollback()` précède `return Ok(before)` ; dans `update_in_tx`, il devient `return Ok(before)` **sans rien
   annuler** (c'est `update`, la fonction qui possède la transaction, qui peut l'annuler sur un no-op) — sinon il
   effacerait les groupes déjà écrits dans la transaction de l'appelant. Un test le garde (AC-7 : membre déjà sous son
   groupe au milieu d'une demande). **Aucun comportement visible ne change** pour `POST /accounts` et `PUT /accounts/{id}` : les tests
   existants (`accounts_repository.rs`, `account_parent_change.rs`, `account_currency_repository.rs`,
   `accounts_e2e.rs`, `account_currency_e2e.rs` et les tests du module) passent sans modification.

2. **AC-2 — Fonction de groupement.** Une fonction du dépôt (p. ex. `accounts::apply_groups(pool, company_id,
   user_id, groups)`) ouvre **une** transaction et, pour chaque proposition dans l'ordre reçu :
   - `create` : contrôle du parent du groupe par la règle du `PUT` (#1351 : parent de la société, actif, ni compte de
     différence de change, **refus d'un parent qui porte des écritures** `ACCOUNT_PARENT_HAS_ENTRIES` — non la règle
     plus large de la création, écart A-1). `guard_parent_change_in_tx` exige le compte déplacé (`&Account`), qu'un
     groupe pas encore inséré n'a pas : **généraliser** la fonction plutôt que la couper : `guard_parent_change_in_tx` prend
     `moved: Option<&Account>` (et `company_id`), et **saute** pour `None` les seules étapes qui dépendent du compte
     déplacé — lui-même comme parent, cycle par les ancêtres (et leur verrouillage), exclusion de ses propres
     désignations, comptes propres — en gardant pour `Some` **l'ordre exact** de ses refus et de ses verrous (un parent
     à la fois descendant et mouvementé rend toujours `ACCOUNT_PARENT_CYCLE`) ; un découpage en deux fonctions
     changerait cet ordre et n'est pas permis ; puis `create_in_tx` avec `account_type` = le type commun des membres,
     `currency: None`, `role: None`, `postable: false`, `own_fx_accounts: None` ; numéro déjà porté (archivé compris)
     → `ACCOUNT_GROUP_INVALID` `numberTaken`, jugé avant l'`INSERT` sous verrou (`SELECT … FOR UPDATE` sur
     `(company_id, number)`), la contrainte `uq_accounts_company_number` restant le filet ;
   - `attach` : le compte existe dans la société, est actif et a au moins un sous-compte actif, sinon `notAGroup` ;
   - `reuse` : le compte existe dans la société, est actif et n'a aucun sous-compte actif, sinon `notReusable` ; ses
     écritures sont jugées par la garde du changement de parent de chaque membre (`guard_parent_change_in_tx`,
     `ACCOUNT_PARENT_HAS_ENTRIES`), sous verrou ;
   - puis, pour chaque membre, dans l'ordre croissant des `id`,
     `update_in_tx(company_id, id, version, user_id, changes, false)` avec `changes` construit **depuis la ligne du
     membre lue sous `FOR UPDATE`** — nom, type, rôle, postabilité inchangés, `currency` et `own_fx_accounts`
     `Unchanged`, `parent: ParentChange::Under(group_id)` ; un membre déjà sous ce groupe est un no-op (ni `version`,
     ni audit) ; aucun membre n'est retypé (`confirm_retype = false`).
   Les contrôles de forme de la demande (`accountInTwoGroups`, `groupIsMember`, `groupInTwoProposals`,
   `memberIsOwnGroup`, `empty`, `tooMany`) se font **avant** `begin()` — `groupInTwoProposals` (revue P1, L2) : un
   même compte visé par deux propositions, deux `attach`, deux `reuse` ou l'un et l'autre, est refusé à la seconde,
   ses comptes se réunissant en une seule proposition. Dans la transaction, **ordre des verrous** : d'abord `companies` en partage et
   les réglages de facturation en partage (comme `create` et `update`, qui les reprennent ensuite sans effet), puis
   **tous** les membres et comptes d'`attach`/`reuse` de la demande `FOR UPDATE`, **en une passe, dans l'ordre
   croissant des `id` sur toute la demande**, par une lecture **filtrée par `company_id`** : un identifiant absent de la
   société → `NotFound` (`404`, avec `groupStep`) avant tout autre contrôle. Puis, **compte par compte dans l'ordre
   croissant des `id`** : pour un **membre**, archivé → le refus d'`update` (`409 ILLEGAL_STATE_TRANSITION`, avec
   `groupStep`), puis `version` différente de celle de la demande → `409 OPTIMISTIC_LOCK_CONFLICT` (avec `groupStep`) —
   **avant** les contrôles d'état : un membre retypé depuis l'aperçu rend `409` (recharger), non `mixedTypes` ; pour le
   compte d'un **`attach`** ou d'un **`reuse`** (qui n'a pas de `version` dans la demande), inactif → `notAGroup` ou
   `notReusable`, jugés avec les contrôles d'état ci-dessous. Le contrôle de `version` d'`update_in_tx`, sous le même
   verrou, ne peut plus échouer : c'est un filet, non un contrôle à tester. Puis les contrôles d'état
   (`mixedTypes`, `groupTypeMismatch`, `notAGroup`, `notReusable`, `numberTaken`) sur ces lignes verrouillées — un
   retypage concurrent ne passe pas entre le contrôle et l'écriture —, **en une phase entière avant toute écriture**,
   proposition par proposition dans l'ordre reçu (fixé à l'étape 0, L2 de P6) : `mixedTypes`, puis `numberTaken` d'un
   `create` (numéro porté, ou demandé par une création précédente de la même demande, comparé **sans casse** comme la
   collation de la colonne — revue P1, L1), `notAGroup` ou `notReusable`
   d'un `attach` ou d'un `reuse`, puis leur `groupTypeMismatch` ; les écritures (parent du groupe créé, `create_in_tx`,
   `update_in_tx` des membres) viennent ensuite, proposition par proposition ; les comptes réutilisés **et déplacés**
   sont relus après la dernière écriture, avant le commit (revue P1, M1 : un compte déplacé devenu parent d'un groupe
   créé ensuite y est non imputable, sans hausse de `version`) ; toute erreur rend `Err` et rien n'est commité.

3. **AC-3 — Erreurs.** Une variante neuve de `DbError` (p. ex. `AccountGroupInvalid { group_index, reason,
   account_id }`) donne `400 ACCOUNT_GROUP_INVALID`, `details: { groupIndex, reason, accountId? }` (`reason` parmi
   les dix de la fiche mère, onze avec `groupInTwoProposals`, ajouté à la revue P1). Une erreur **existante** levée par l'étape d'une proposition garde son code HTTP, son code
   et ses `details`, **augmentés** d'un objet **`groupStep: { groupIndex, accountId? }`** (objet à part : une clé
   `accountId` au premier niveau écraserait celle que portent déjà `ACCOUNT_ROLE_ALREADY_ASSIGNED` ou
   `ACCOUNT_IN_FOREIGN_CURRENCY`) ; l'objet `details` est créé quand le code n'en a pas. Mécanique, sans recopie des
   correspondances existantes :
   - `apply_groups` rend un type d'erreur propre (p. ex. `GroupStepError { group_index: Option<usize>, account_id:
     Option<i64>, source: DbError }`) ; **pas d'enveloppe dans `DbError`** ;
   - dans la fermeture passée à `retry_app_on_deadlock` (seule enveloppe de rejeu permise dans `src/routes/`,
     `audit_route_registry.rs` volet c bis interdisant `retry_with`), le handler convertit : `source` interblocage
     (`is_deadlock_error`) → `AppError::Database(source)`, que le prédicat `is_app_deadlock` reconnaît et rejoue ;
     sinon → une variante neuve d'`AppError`, `AccountGroupStep { group_index, account_id, error: Box<AppError> }`,
     dont `into_response` **délègue** à l'erreur interne (précédent exact : `AppError::ConversionEntry`,
     `errors.rs:654` et `:2384`) ;
   - **après** le rejeu, le handler rend la réponse par le mécanisme commun **`refus_avec_detail(*error, "groupStep",
     json!({ "groupIndex": …, "accountId": … }))`** (`errors.rs:39`, asynchrone : il relit le corps de l'erreur, crée
     `details` s'il manque, y ajoute la clé ; précédent `conversion_refusal`, `routes/journal_entries.rs:1001`) — les
     correspondances existantes de `DbError` (~1700 lignes) ne sont ni touchées ni recopiées ; seule s'y ajoute celle
     de la variante neuve `AccountGroupInvalid` ;
   - `groupStep` n'est ajouté que si l'erreur se rattache à une proposition (`group_index` connu, `usize`) ; `accountId`
     n'y figure que pour une étape qui porte sur un compte ; une erreur hors proposition (verrou de la société, p. ex.)
     est rendue telle quelle ;
   - un `ACCOUNT_GROUP_INVALID` porte déjà `groupIndex` dans ses `details` : il **n'est pas** enveloppé.
   Le message du code neuf, et celui des onze motifs s'il est rendu par le serveur, existent dans les quatre locales.

4. **AC-4 — Route.** `POST /api/v1/accounts/groups` dans `comptable_routes` (Administrateur et Comptable ;
   Consultation `403`), **au-dessus** du `route_layer` du bloc ; en tête du handler, une clé `read-write` est refusée
   `403 API_KEY_OPERATION_FORBIDDEN` (motif `bank_imports.rs:1689`, test `current_user.api_key_id`) ; une clé `read`
   l'est avant, en `403 API_KEY_READ_ONLY`, par la couche d'authentification (`middleware/auth.rs`, méthode mutante). Forme du corps (`serde`, `camelCase`, champs
   inconnus refusés comme les autres corps du plan s'ils le sont) ; `number` et `name` contrôlés par **la même
   fonction** que `create_account` (extraite, non recopiée ; longueurs en octets, comme aujourd'hui), `groups` vide →
   `400 VALIDATION_ERROR` ; `parentId` d'un `create` par `check_parent_in_company`.
   L'appel au dépôt est **rejoué sur interblocage** (`retry_app_on_deadlock("accounts::apply_groups", …)`), la
   transaction entière à chaque tentative. Réponse `200 { created, reused, moved }` au format `AccountResponse` de
   `GET /accounts`, lettrable et devise verrouillée calculés **en une requête pour toute la réponse**
   (`currency_locked_account_ids` et l'équivalent pour la lettrabilité), non par compte.

5. **AC-5 — Audit (Q10).** Aucune action neuve : une entrée `account.created` par groupe créé et une
   `account.updated` par compte déplacé, écrites **dans la transaction** par `create_in_tx` et `update_in_tx`. Un
   groupe réutilisé ou rattaché, qui passe non imputable sans hausse de `version`, n'a pas d'entrée propre (comme le
   parent d'un `POST /accounts` aujourd'hui).

6. **AC-6 — Documentation de la fermeture aux clés et de la route.** Tous les sites qui énumèrent les opérations
   fermées aux clés, et ceux qui décrivent les routes du plan :
   - `docs/api-external.md` : § 4 (un paragraphe par exception fermée, comme la suppression d'un import), § 7 (« à
     quatre exceptions près, toutes quatre fermées aux clés API » devient cinq, avec `POST /accounts/groups`), § 9
     (une ligne au tableau des limitations, comme le journal d'audit), § 10 (ligne `API_KEY_OPERATION_FORBIDDEN` ; ligne
     neuve `400 ACCOUNT_GROUP_INVALID` et ses motifs ; ligne `ACCOUNT_PARENT_HAS_ENTRIES`, qui vaut aussi pour la
     route groupée), le paragraphe des routes du plan de § 7 (`POST /accounts`, `PUT /accounts/{id}`) gagne le contrat de
     `POST /accounts/groups` (corps, réponse, refus tout ou rien, `groupStep`), et le paragraphe des routes **rejouées
     sur interblocage** s'il les énumère ;
   - `docs/manual/fr/admin-manual.tex` : la liste des opérations fermées aux clés (itemize de la section des clés API,
     « Suppression d'un import bancaire », « Changement de mot de passe ») gagne un point ;
   - le commentaire de la variante `AppError::ApiKeyOperationForbidden` (`errors.rs`, « Aujourd'hui : … ») ;
   - greper le nombre (`\bquatre\b`, `\b4\b` près de « exceptions ») et `API_KEY_OPERATION_FORBIDDEN` sur `docs/`, les
     manuels, le code et les gardes de textes (`crates/kesh-api/tests/textes_coherents.rs`) avant de conclure ; `.tex`
     seul, aucun PDF commité.

7. **AC-7 — Tests (sondes négatives vérifiées par mutation, une à la fois, copie à sa propre cible cargo).**
   - `kesh-db`, `#[sqlx::test]` (fichier neuf, p. ex. `crates/kesh-db/tests/account_groups.rs`, sur le schéma de test
     comme ses voisins) : création de deux groupes et déplacement de leurs membres ; **tout ou rien** — sur une erreur qui
     ne peut naître **qu'après des écritures** : une demande dont la première proposition (`create`) écrit son groupe et
     déplace ses membres, et dont la seconde (`reuse`) vise un compte qui porte une écriture →
     `ACCOUNT_PARENT_HAS_ENTRIES` au changement de parent de son premier membre, et **rien n'est écrit** (nombre de
     comptes, `parent_id`, `postable`, `version` et nombre d'entrées d'audit identiques avant et après) ; un conflit de
     `version` du dernier membre, lui, est refusé dans la passe de verrous, avant toute écriture ; un compte d'`attach`
     archivé → `notAGroup`, de `reuse` archivé → `notReusable`, un membre archivé → `ILLEGAL_STATE_TRANSITION` ; un membre **retypé** après la lecture de sa `version` → `409` avec
     `groupStep`, jamais `mixedTypes` ; un membre déjà sous son groupe **au milieu** d'une
     demande ne fait rien annuler (les groupes écrits avant lui restent au commit : garde du no-op, AC-1) ; `reuse` (le compte devient non imputable, garde
     son nom) ; `reuse` d'un compte qui a reçu une écriture avant l'appel → `ACCOUNT_PARENT_HAS_ENTRIES` ; `attach` ;
     `attach` sur un compte sans sous-compte → `notAGroup` ; `mixedTypes` ; `groupTypeMismatch` ; `numberTaken` (dont
     un numéro archivé) ; parent mouvementé d'un groupe créé → `ACCOUNT_PARENT_HAS_ENTRIES` ; membre déjà sous le
     groupe → aucune entrée d'audit pour lui ; **décompte d'audit** n créations + m déplacements ; membre, compte d'`attach`
     et compte de `reuse` d'une autre société → `NotFound` (et non `mixedTypes`, `notAGroup`, `notReusable`), rien
     écrit ; `POST /accounts` et `PUT /accounts/{id}` inchangés sur un parent mouvementé (garde extraite, AC-2).
   - `kesh-api` (`crates/kesh-api/tests/accounts_e2e.rs` ou fichier voisin) : `200` et forme de la réponse ;
     `403 API_KEY_OPERATION_FORBIDDEN` d'une clé `read-write` et `403 API_KEY_READ_ONLY` d'une clé `read` (sonde :
     la même demande par session passe) ; `400 VALIDATION_ERROR` d'un `parentId` d'une autre société ; `403` Consultation ; `400` de forme (motifs
     `accountInTwoGroups`, `groupIsMember`, `groupInTwoProposals`, `empty`, `tooMany`) ; une erreur existante porte `groupStep` en plus de
     ses `details`.
   - Tests de motifs en plus : `notReusable` (compte qui a un sous-compte actif), `memberIsOwnGroup`,
     `ACCOUNT_PARENT_CYCLE` (parent d'un `create` qui est membre de la demande, sous l'un de ses membres), `attach` sur
     un groupe mouvementé → `ACCOUNT_PARENT_HAS_ENTRIES` ; le refus d'une erreur existante porte `groupStep` sans
     perdre ses propres `details` (p. ex. `ACCOUNT_PARENT_HAS_ENTRIES` garde `parentNumber`).
   - Les mutations à rejouer (chacune doit faire échouer un test nommé sur une assertion) : retirer le refus du parent
     mouvementé du `create` groupé ; retirer le contrôle `mixedTypes` ; retirer la comparaison de `version` de la passe de verrous (le
     test du membre retypé rend alors `mixedTypes` au lieu de `409`) ; supprimer le
     refus des clés `read-write` en tête du handler ; faire rendre une erreur au no-op d'`update_in_tx` (le test du
     membre déjà rangé au milieu d'une demande rougit) ; commiter avant la dernière proposition (le test du tout ou rien rougit) ; filtrer la passe de
     verrous sans `company_id` (tuée seulement par un test dont le compte étranger donnerait, sans le filtre,
     `notAGroup`, `notReusable` ou `mixedTypes` avant qu'`update_in_tx` ne le refuse : choisir les comptes en
     conséquence).

## Tâches / sous-tâches

- [x] T0 — Relire au code les lignes citées (`repositories/accounts.rs` `create` `:189`, `update` `:1079`,
      `guard_parent_change_in_tx` `:932`, `has_journal_line_locked` `:2704` ; `routes/accounts.rs` `create_account`
      `:373`, `update_account` `:452`, `check_parent_in_company` `:76` ; `lib.rs` `comptable_routes` `:576`) ; les
      numéros de ligne datent de `dc8786541` et ont pu bouger.
- [x] T1 — Extraction `create_in_tx`, `update_in_tx` (AC-1) ; gate nextest complet **avant** d'aller plus loin (les
      tests existants sont le filet de l'extraction).
- [x] T2 — `apply_groups`, `DbError` neuve et enveloppe (AC-2, AC-3) ; tests `kesh-db`.
- [x] T3 — Route, câblage, `errors.rs`, refus des clés (AC-4) ; **entrée au registre des routes** de
      `crates/kesh-api/tests/audit_route_registry.rs`, classée **`SansEcritureAuJournal`** (elle n'écrit aucune
      écriture comptable ; `Rejouee` veut dire « écrit au journal ») et ajoutée à la liste (vi) des routes
      `SansEcritureAuJournal` rejouées sur interblocage, dont les décomptes datés (« trente-deux » à `:163` et à
      `:354-355`, avec « vingt-neuf handlers ») sont **recomptés** depuis la liste, non incrémentés ; ligne de la route dans le tableau des verrous de `docs/MULTI-TENANT-SCOPING-PATTERNS.md`
      (renvoi au doc-comment d'`apply_groups`, « Verrous », comme `PUT /accounts/{id}`) ; tests `kesh-api`.
- [x] T4 — i18n (quatre locales), `docs/api-external.md` (AC-3, AC-6) ; grep du symptôme.
- [x] T5 — Mutations (AC-7) ; gates de fin de story (règles en vigueur au moment du développement).

## Dev Notes

- **Pourquoi une passe de verrous en tête** : `update` prend le compte `FOR UPDATE`, puis le nouveau parent
  `FOR UPDATE` et ses ancêtres en partage ; verrouiller d'abord tous les comptes de la demande, dans un ordre stable
  après `companies`, fait que deux demandes croisées s'interbloquent moins, et la route rejoue l'interblocage restant.
- **Le compte de groupe créé est non imputable dès l'`INSERT`** (`postable: false`), puis le premier membre le
  reconfirme (`UPDATE accounts SET postable = FALSE` du changement de parent) — sans effet de bord.
- **Ce qui n'est pas contrôlé au serveur** : la règle des propositions (dernier chiffre, préfixe commun, « pas de
  groupe d'un seul compte »). Le serveur reçoit des groupes explicites ; un groupe d'un seul compte est permis (mode
  « un seul groupe » de l'écran).
- **Verrou d'intervalle** : le `SELECT … FOR UPDATE` d'un numéro absent (`numberTaken`) pose un verrou d'intervalle ;
  deux demandes sur des numéros voisins peuvent s'interbloquer — l'interblocage est rejoué (AC-3, AC-4).
- **`reuse` n'ajoute aucun contrôle** au changement de parent de #1351 (rôle, lien bancaire, désignation) : un compte
  réutilisé devient non imputable exactement comme un parent rattaché par `PUT /accounts/{id}` aujourd'hui.
- **Membres qui sont des groupes** : non proposés par l'écran (fiche mère) ; le serveur ne les refuse pas (le `PUT`
  les accepte), il les range comme les autres.
- **Fichiers communs** avec l'Epic 47 (miroir) : `routes/accounts.rs`, `repositories/accounts.rs` — blocs distincts ;
  le second arrivé résout, et la fusion a sa revue des défauts d'interaction.

### References

- `46-5-aide-groupes-plan.md` § *Le contrat commun*.
- `fix-api-parent-compte.md` (#1351) : garde du changement de parent, écart A-1 (création plus large).
- `CLAUDE.md` § *Exception `kesh-db`*, § *Pattern batch* (non appliqué, motif à la fiche mère).

## Note de version

- **Added** — `POST /api/v1/accounts/groups` : crée des comptes de groupe et y range des comptes, en une seule
  opération tout ou rien, avec les contrôles de la création et du changement de parent ; réservée à l'interface web
  (`403 API_KEY_OPERATION_FORBIDDEN` pour toute clé d'API) ; refus propres `400 ACCOUNT_GROUP_INVALID`.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`bmad-dev-story`).

### Debug Log References

Journaux hors dépôt, `kesh-gate-logs/s46-5a/` : `clippy-t3.log`, `fmt-t3.log` (fmt et clippy workspace),
`gate-backend-g1.log` (sans résultat : bus systemd utilisateur refusé, relancé avec `KESH_NO_MEMGUARD=1`),
`gate-backend-g2.log` (9 échecs, trois causes, ci-dessous), `gate-backend-g3.log` (vert), `front-*.log`,
`mut/*.log` (mutations), `manuel.txt` (PDF aplati).

### Completion Notes List

- **Étape 0** : recontrôle déclaré au Change Log, lignes lues citées.
- **T1** : `create` et `update` deviennent `begin()` → `create_in_tx` / `update_in_tx` → `commit()`. Écart à la
  tâche : le gate nextest de T1 n'a pas été joué seul, avant T2 — l'extraction et `apply_groups` sont au même commit
  `94ccec862`, et le premier gate complet (g2) couvre les deux ; les tests existants des comptes
  (`accounts_repository`, `account_parent_change`, `account_currency_repository`, `accounts_e2e`,
  `account_currency_e2e`) y passent **sans modification** (aucun de ces fichiers n'est touché par la story :
  `git diff 46fd2f49f..HEAD --stat`).
- **Phase des contrôles d'état** (L2 de P6, fixé ici) : une phase entière avant toute écriture, proposition par
  proposition dans l'ordre reçu (`apply_groups`, `accounts.rs` « 3. L'état »), puis les écritures.
- **Choix de la story** : `numberTaken` vaut aussi pour deux créations de même numéro dans la demande (sans
  `accountId`), au lieu de laisser la contrainte unique rendre un `409` générique ; les refus `VALIDATION_ERROR` du
  numéro, du nom et du `parentId` d'un `create` portent `details.groupStep: { groupIndex }` ; la forme de la demande
  est jugée par la route (avant les lectures du `parentId`) puis de nouveau par `apply_groups` (fonction pure).
- **Gardes rencontrées au gate g2** (9 échecs, aucune régression du code) : `audit_acteur_garde` (les sites
  d'acteur `User` de `create`/`update` sont passés dans `create_in_tx`/`update_in_tx` : entrées renommées, appelants
  complétés), `audit_route_registry::the_replay_partition_is_what_the_story_declares` (161 → 162),
  `date_du_jour_garde` ×7 (le montage E2E lisait `Utc::now().date_naive()` : remplacé par
  `common::jour::aujourd_hui_test()`). Corrigé au commit `d88771e04`.
- **Registre des routes** : 157 → 158 routes, 149 → 150 tracées, 128 → 129 `SansEcritureAuJournal`, 161 → 162 avec
  les endpoints de test ; liste (vi) **trente-trois** (trente handlers), recomptée depuis le code : 28 handlers
  `SansEcritureAuJournal` dont le corps appelle une enveloppe, plus `acknowledge_notices` et `apply_groups` par leur
  corps délégué, plus les trois rejouées hors de `src/routes/`. Manuel : « 150 des 158 routes », « 150 + 4 + 4 = 158 ».
- **Tests ajoutés** (recomptés, de `46fd2f49f` à `HEAD`) : 20 — `crates/kesh-db/tests/account_groups.rs` 15 (14
  `#[sqlx::test]`, 1 `#[test]` de forme), `crates/kesh-api/tests/account_groups_e2e.rs` 5.
- **Gates au dernier commit de code** (`d88771e04`) : fmt et clippy workspace verts ; nextest complet
  (`scripts/test-fast.sh --ci`, base remise à zéro) **6281 exécutés, 6281 passés, 14 ignorés** ; frontend (les
  quatre `.ftl` changent) : `check`, `lint-i18n-ownership`, `build` verts, Vitest **4742 passés (283 fichiers)**. **Aucun E2E** (aucune spec Playwright ajoutée : règle de la nuit du
  22.11) ; `gate-ci.sh` à la fin de l'epic.
- **Mutations** (copie `git archive`, cible `kesh-targets/s46-5a-mut`, une à la fois ; copie intacte verte d'abord,
  20/20) : **sept tuées sur sept**, chacune par une assertion nommée (`panicked at` + test en `FAIL`) —
  `parent-mouvemente` (refus du parent mouvementé du `create` retiré) par
  `parent_mouvemente_d_un_groupe_cree_est_refuse` (et `erreur_existante_portee_avec_son_etape`) ; `mixed-types` par
  `types_meles_et_type_du_groupe` ; `version-passe` (comparaison de `version` de la passe retirée) par
  `membre_retype_depuis_l_apercu_rend_409_et_non_mixed_types` ; `cle-api` (refus des clés `read-write` retiré) par
  `toute_cle_d_api_est_refusee` ; `no-op-erreur` (le no-op d'`update_in_tx` rend une erreur) par
  `membre_deja_range_au_milieu_n_annule_rien` ; `commit-anticipe` (commit avant la dernière proposition) par
  `tout_ou_rien_sur_un_refus_ne_apres_des_ecritures` ; `societe-passe` (passe de verrous sans `company_id`) par
  `compte_d_une_autre_societe_rend_not_found`. ⚠️ Le premier passage de `cle-api` réutilisait un `kesh-db` encore
  muté par `version-passe` (les fichiers restaurés par `git archive` gardent leur date ancienne, cargo les a crus à
  jour) : rejoué seul après un `touch` de tous les `.rs` de la copie, tué de même par `toute_cle_d_api_est_refusee`
  (`mut/cle-api.log`) ; les autres passages mutaient `kesh-db`, que cargo recompilait avec `kesh-api`.
- **PDF** du manuel d'administration produit dans une copie et aplati (`latexmk` vert) : « 150 des 158 routes », « 150 + 4 + 4 = 158 »,
  « Création de groupes du plan » et « POST /api/v1/accounts/groups » y figurent chacun une fois ; aucun PDF commité.
- **Fusion** : la story reste sur `story/46-5`, **aucune fusion dans `epic/46-rapports-groupes` avant l'accord de
  Guy** (fiche mère).

### File List

- `crates/kesh-db/src/repositories/accounts.rs` — `create_in_tx`, `update_in_tx`, `guard_parent_change_in_tx`
  généralisée (`company_id`, `moved: Option<&Account>`), `check_groups_shape`, `apply_groups`
- `crates/kesh-db/src/entities/account.rs` — `GroupKind`, `GroupMember`, `GroupProposal`, `GroupsOutcome`,
  `MAX_GROUP_PROPOSALS`, `MAX_GROUP_MEMBERS`
- `crates/kesh-db/src/errors.rs` — `AccountGroupReason`, `DbError::AccountGroupInvalid`, `GroupStepError`
- `crates/kesh-db/tests/account_groups.rs` (neuf)
- `crates/kesh-api/src/routes/accounts.rs` — `apply_groups`, `apply_groups_checked`, `group_step_error`,
  `group_step_refusal`, `checked_name`, `checked_number_and_name`, DTO de la route
- `crates/kesh-api/src/errors.rs` — `AppError::AccountGroupStep`, correspondance d'`AccountGroupInvalid`,
  commentaire d'`ApiKeyOperationForbidden`
- `crates/kesh-api/src/lib.rs` — `POST /api/v1/accounts/groups` dans `comptable_routes`
- `crates/kesh-api/tests/account_groups_e2e.rs` (neuf), `crates/kesh-api/tests/audit_route_registry.rs`,
  `crates/kesh-api/tests/audit_acteur_garde.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` — onze messages `error-account-group-invalid-*` (dix au développement, `group-in-two-proposals` à la revue P1)
- `docs/api-external.md`, `docs/manual/fr/admin-manual.tex`, `docs/MULTI-TENANT-SCOPING-PATTERNS.md`
- `_bmad-output/implementation-artifacts/46-5-aide-groupes-plan.md` (L1 de P6), cette fiche, `sprint-status.yaml`

## Change Log

- 2026-11-22 — Fiche écrite sur la décision du responsable de l'epic (fiche mère), découpage de la 46-5 ; validation
  complète à suivre (`kesh-gate-logs/s46-5/validation/`).
- 2026-11-22 — Validation P1 à P3 : remédiations portées ici (no-op sans `rollback`, garde du parent généralisée,
  erreurs par `AccountGroupStep` et `refus_avec_detail`, passe de verrous filtrée par société avec la `version`,
  registre des routes et tableau des verrous, sites de la fermeture aux clés, tests et mutations) ; tendance et détail
  au Change Log de la fiche mère.
- 2026-11-22 — Validation P4 (ciblée, Sonnet) et P5 (ciblée, Opus) : remédiations portées ici ; validation close au
  plafond de cinq passes, avec recontrôle obligatoire à l'étape 0 du développement (Change Log de la fiche mère).
- 2026-11-22 — **Étape 0, recontrôle au code des deux MEDIUM de P5 et des LOW de P6** (`kesh-gate-logs/s46-5/validation/P6-rapport.md`) :
  (M1, tout ou rien) l'erreur du test ne naît qu'après des écritures — `guard_parent_change_in_tx`,
  `repositories/accounts.rs:1008` (numérotation après développement), lu : `if has_journal_line_locked(tx,
  new_parent_id).await? { return Err(DbError::AccountParentHasEntries { … }) }`, appelée par `update_in_tx` au
  changement de parent du premier membre de la proposition `reuse` ; les contrôles d'état de `reuse` ne jugent que
  `active` et les sous-comptes actifs (`accounts.rs:1635`, `let refused = match g.kind`) ; (M2, ordre des refus) celui
  d'`update` lu à `accounts.rs:1168` (`Some(a) if !a.active` → `IllegalStateTransition`) puis `:1173`
  (`Some(a) if a.version != version` → `OptimisticLockConflict`), repris tel quel par la passe des membres
  d'`apply_groups` ; les comptes d'`attach`/`reuse` inactifs vont à `notAGroup`/`notReusable` dans la phase d'état.
  (L1 de P6) « ou archivé » ajouté à `notAGroup` dans la fiche mère. (L2 de P6) fixé : contrôles d'état en une phase
  entière avant toute écriture (AC-2, `accounts.rs:1597` « 3. L'état… » puis `:1659` « 4. Les écritures »).
- 2026-11-22 — Développement (`bmad-dev-story`) : commits `94ccec862` (code, tests, i18n, documentation) et
  `d88771e04` (gardes de l'audit, du registre et de la date du jour) ; gates et mutations au Dev Agent Record.
  Statut `review` ; prompt de la revue P1 : `kesh-gate-logs/s46-5a/revue-p1-prompt.md`.
- 2026-11-22 — **Revue de code P1** (complète, Opus, `kesh-gate-logs/s46-5a/revue-p1-rapport.md`) : 0 CRITICAL, 0 HIGH,
  1 MEDIUM, 5 LOW. Remédiation au commit `9b8e497b9` :
  (M1) `apply_groups` relit les comptes déplacés, comme les réutilisés, après la dernière écriture et avant le commit
  (`reread_in_tx`) : un compte déplacé devenu parent d'un groupe créé par une proposition suivante est rendu non
  imputable — test `moved_relu_apres_la_derniere_ecriture` (la sonde de la revue) ;
  (L1, choix de l'orchestrateur) deux créations du même numéro à la casse près rendent `numberTaken` (clé du contrôle
  en `to_lowercase()`, la colonne étant sans casse ; les autres équivalences de la collation restent à l'index unique,
  tout ou rien tenu) — cas `10a`/`10A` ajouté à `numero_deja_porte` ;
  (L2, choix de l'orchestrateur) un même compte visé par deux propositions est refusé à la forme, avant toute
  écriture, motif neuf **`groupInTwoProposals`** (onze motifs, message dans les quatre locales) ; l'agent l'étend à
  `attach` puis `reuse` du même compte, déjà refusé avant sous verrou (`notAGroup`/`notReusable`), pour une règle
  unique — cas ajoutés à `forme_de_la_demande` et `refus_de_forme` ; contrat écrit ici, dans la fiche mère et
  `docs/api-external.md` § 7 et § 10 ;
  (L3) doc-comment de `coded_response` remis au-dessus de sa fonction ; (L4) § 7 de `docs/api-external.md` : le
  `VALIDATION_ERROR` d'une création porte `groupStep: { groupIndex }` ; (L5) `create` appelle
  `check_role_account_type` avant `begin()` (`create_in_tx` le refait en tête, ordre des refus inchangé).
  Gates au commit `9b8e497b9` : gate ciblé des deux binaires 21/21 ; `scripts/test-fast.sh --ci` (base remise à
  zéro ; `KESH_NO_MEMGUARD=1`, le bus systemd utilisateur ayant refusé la connexion au premier passage g4, sans
  résultat) — fmt, clippy workspace verts, nextest **6282 exécutés, 6282 passés, 14 ignorés** (`gate-backend-g5.log`) ;
  frontend (les `.ftl` changent) : `check`, `lint-i18n-ownership`, `build` verts, Vitest **4742 passés (283
  fichiers)**. Aucun E2E (aucune spec ajoutée). Mutations (copie `git archive`, cible `kesh-targets/s46-5a-mut`, une à
  la fois, sources touchées à chaque copie ; témoin intact 21/21) : **trois tuées sur trois** par une assertion nommée
  — `p1-m1-relecture` par `moved_relu_apres_la_derniere_ecriture` (« moved rend l'état du commit », premier passage
  non compilé, motif corrigé et rejoué : `mut/p1-m1-rejeu.console.log`), `p1-l1-casse` par `numero_deja_porte`,
  `p1-l2-cible` par `forme_de_la_demande` (et `refus_de_forme`). Tests de la story recomptés (de `46fd2f49f` à
  `HEAD`) : 21 — `account_groups.rs` 16, `account_groups_e2e.rs` 5. Prompt de la revue P2 (complète, Sonnet) :
  `kesh-gate-logs/s46-5a/revue-p2-prompt.md`.
- 2026-11-22 — **Revue de code P2** (complète, Sonnet, `kesh-gate-logs/s46-5a/revue-p2-rapport.md`) : 0 CRITICAL,
  0 HIGH, 0 MEDIUM, 3 LOW — **boucle close** (aucun finding au-dessus de LOW). Suites données, sans code de
  production :
  (L1) `AccountGroupReason::ALL` n'avait aucun usage : gardé, et lié aux messages par le test
  `errors::tests::motifs_de_groupe_ont_leur_cle_dans_les_quatre_catalogues` (codes d'API distincts, clé de chaque motif
  présente dans les quatre `messages.ftl`, `{ $position }` partout sauf `tooMany`, et de même `{position}` du repli) ;
  mutation hors dépôt (clé `…-group-in-two-proposals` retirée du catalogue `de-CH`, copie `git archive`, cible
  propre) : témoin vert, mutant tué par l'assertion « de-CH : clé … absente » (`mut/l1-mutation.console.log`) ;
  (L2) la fiche mère dit exactement que le doublon de la demande se compare « sans casse, pas sans accents », et que
  `10é` après `10e` bute sur l'index unique en `409 RESOURCE_CONFLICT` (le rapport de la revue écrit `UNIQUE_CONSTRAINT_VIOLATION`, code de `DbError` côté base, que `errors.rs:4589` rend en `RESOURCE_CONFLICT`) avec `groupStep`, rien d'écrit ;
  (L3) jugé acceptable par la revue, versé à la liste des issues à ouvrir (enhancement, P4 : message de
  `groupInTwoProposals` pour un `attach` puis un `reuse` du même compte).
  Gates de clôture : `cargo fmt --check`, clippy workspace `-D warnings` verts ; nextest
  `binary(account_groups) | binary(account_groups_e2e) | test(motifs_de_groupe_ont_leur_cle)` (base remise à zéro)
  **22/22** (`cible-clot.log`).
  **Tendance de la revue de code** : P1 (complète, Opus) 0/0/1/5 → P2 (complète, Sonnet) 0/0/0/3. Statut `done` ;
  fusion dans `epic/46-rapports-groupes` après l'accord de Guy (fiche mère).
