# Story 29.4 : Une saisie concurrente ne passe plus sur un compte qu'on clôt

## Status

**29-4a : done** (développée le 2026-10-14, commit `e718221b` ; revue de code P1 remédiée le même jour après
rebase sur `b4abae5d`, commit `bc9cf85e` ; passe ciblée P2 faite, 0/0/0/5, **revue close** ; cinq points reportés à la 29-4b, T6-bis) — **29-4b :
done** (développée le 2026-10-14 sur `fe404b11`, branche `story/29-4b` ; commits de code `dba87062` — étape 0,
T6-bis — et `6c8513a6` ; revue de code P1 remédiée, passe ciblée P2 remédiée, **revue close** ; rebasée le
2026-10-14 sur `e889346c` (29-6), gates finaux rejoués au HEAD rebasé — Change Log, dernière entrée).

*Relevé d'origine :* **ready-for-dev** (deux clés) — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `16eeed21`
(kickoff de l'Epic 29), worktree `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4`. Validation P1 faite (Sonnet,
lentilles R et F : 0/0/6/7 et 0/0/2/7) et remédiée le 2026-10-14 ; validation P2 faite (Opus, lentilles R et F :
0/1/3/6 et 0/1/3/7) et remédiée le même jour — **signal D5 levé** (un HIGH convergent né de la remédiation P1), déclaré
au Project Lead (Change Log) ; validation P3 à lancer (`29-4-cloture-compte-concurrente-validate-prompt-p3.md`,
**Sonnet**, lentilles R et F) — le fichier d'epic demande pour cette story une **validation complète Opus et Sonnet**, et
aucune simplification sans relecture des verrous et de la naissance de la vue (B5 de la rétrospective 28).

⛔ **Deux stories, une fiche, deux PR successives** (C-29-4-6, sur le précédent C-28-4-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`29-4a-reverification-import-lot`** | la revérification verrouillante (deux fonctions et deux SQL : `closed_on` seul, et `closed_on` **et** `archived`), la seconde posée sur l'**import** (W22) et la **création de lot** (W17) **après** leur `INSERT` à clé étrangère — verrou déjà tenu, **aucun verrou neuf** ; le **rejeu** de la clôture, de la réouverture et du `PATCH` du compte ; le troisième inventaire du registre (`bank_account_closed_inventory.rs`) ; une ligne `### Corrigé` du CHANGELOG | AC1 à AC5, AC9 (part a), AC10 *(AC11 retiré en P2)* | **`refs #651`** — ⛔ jamais `closes` |
| **`29-4b-reverification-annulations`** | la revérification `closed_on` seul posée, en **dernière acquisition de verrou**, sur les cinq annulations (W9, W24, W25, W28, W29) ; tests d'entrelacement, du cycle neuf et de l'annulation sur un compte archivé ; manuel utilisateur (PDF), `docs/api-external.md`, doc-comments de L1, CHANGELOG complété | AC6 à AC8, AC9 (part b), AC10 | **`closes #651`** |

**Ordre** : la 29-4a part d'`origin/main` **après le merge de la 29-2 et de la 29-3** (fichier d'epic, § *Ordre*) ; la
29-4b part d'`origin/main` après le merge de la 29-4a. **Aucune release entre les deux PR** : la v0.13.4 attend la
29-4b (à écrire au Dev Agent Record de la 29-4a). La clé mère `29-4-cloture-compte-concurrente` passe au statut `split`.

## Story

**As a** indépendant, PME ou association qui clôt un compte bancaire pendant qu'une autre opération est en cours sur ce
compte (un second onglet, une intégration par clé d'API, un import dont le fichier est en cours d'analyse),
**I want** que cette opération, si elle n'est pas validée **avant** la clôture, soit **refusée** au lieu d'aboutir sur
le compte clos — et, si elle l'est avant, que la clôture la voie,
**so that** la promesse de la 28-4 (#642), « un compte clos est en lecture seule », tienne **sans fenêtre**, et que
plus rien ne m'oblige à rouvrir, traiter, puis clore à nouveau.

Issue : [#651](https://github.com/guycorbaz/kesh/issues/651) (`bug`, `P2`) — limite **L1** de la Story 28-4 (fiche
`28-4-cloture-compte-bancaire.md`, AC7, choix C-28-4-21). Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md`
§ 29-4. Cible : **v0.13.4**.

## Prémisses de #651, établies au code (`16eeed21`)

Chaque prémisse est vérifiée **avant** la conception. Deux sont **fausses en partie** : l'orchestrateur commentera l'issue
(rapport de la fiche).

| # | Prémisse de #651 | Verdict | Preuve |
|---|---|---|---|
| P-a | La garde « compte clos » des chemins d'écriture est une lecture cohérente non verrouillante (C-28-4-21) | **vraie** | `ENSURE_OPEN_SQL` et `ENSURE_OPEN_UNARCHIVED_SQL`, `crates/kesh-db/src/repositories/bank_accounts.rs:750-754`, sans `FOR UPDATE` ni `LOCK IN SHARE MODE` (T-a45, l. 1521-1529) |
| P-b | Une écriture dont la transaction a pris sa vue avant le commit de la clôture peut encore aboutir | **vraie**, mais la fenêtre se borne à la **naissance de la vue** de l'écrivain, non au « début de sa transaction » (`sqlx::begin()` n'émet qu'un `BEGIN`) | `close_in_tx`, l. 1003-1056 : son seul verrou est la ligne du compte ; un écrivain qui ne tient aucun verrou sur cette ligne ne la fait pas attendre |
| P-c | Chemins exposés : annulation de rapprochement (W9), création de lot (W17), import CAMT et CSV (W22), suppression d'import (W24), annulation de règlement client (W25) et fournisseur (W29), annulation d'une facture fournisseur payée (W28), **annulation de rejet (W30)** | **fausse pour W30, partielle pour W24** | ci-dessous |
| P-d | « Un import qui tombe dans la fenêtre laisse des transactions `pending` sur un compte clos » ; « un lot de paiement créé dans la fenêtre reste attaché à ce compte » | **vraie** — et c'est par l'`INSERT` à clé étrangère que ces deux chemins se ferment sans verrou neuf (AC2) | `bank_imports.rs:84` (`INSERT INTO bank_imports`), `payment_batches.rs:144` (`INSERT INTO payment_batches`). Deux prémisses InnoDB, de statut différent : (1) **l'existence** d'un verrou conflictuel avec l'exclusif de la clôture sur la ligne du compte, tenu par l'écrivain après ses `INSERT` — **vérifiée en pratique** par T-a31 de la 28-4 (`bank_accounts_repository.rs:2296`, `une_cloture_attend_un_import_en_cours_puis_le_voit`, qui voit la clôture attendre ; il ne dit ni quel `INSERT` l'a posé, ni son mode) ; (2) **la redemande sans effet** — une lecture `LOCK IN SHARE MODE` par égalité sur la clé primaire, quand l'écrivain tient déjà ce verrou et qu'une clôture **attend déjà** en exclusif, est satisfaite sans file ni attente — **raisonnée** (`lock_rec_lock_slow` teste `lock_rec_has_expl` avant tout contrôle de conflit ; contrôle de clé étrangère et recherche unique sur la clé primaire posent tous deux `LOCK_S \| LOCK_REC_NOT_GAP`), **tant que T29a-6 et T29a-7 (AC3) ne l'ont pas établie** — indice constaté hors dépôt le 2026-10-14 (P3, AC3) : un détenteur partagé qui redemande une lecture `LOCK IN SHARE MODE` pendant qu'un exclusif attend la ligne n'attend pas ; le contrôle de clé étrangère d'un `INSERT` n'a pas été essayé. Si (2) était fausse : la fenêtre resterait fermée (la lecture verrouillante lit le dernier état commité), mais chaque course formerait un cycle écrivain / clôture dont la clôture, rejouée (AC4), serait la victime — « aucun verrou neuf » serait alors à réécrire |
| P-f | *(prémisse de la fiche, non de l'issue)* Une lecture verrouillante « lit le dernier état commité » | **vraie sous `innodb_snapshot_isolation = OFF` seulement** | Constaté sur la base de dev le 2026-10-14 : `SELECT @@version, @@innodb_snapshot_isolation` rend `10.11.16-MariaDB`, `0`. À partir de MariaDB **11.6**, le défaut passe à `ON` : une lecture verrouillante qui rencontre une version postérieure à la vue ouverte rend `ER_CHECKREAD` (**1020**) — exactement la course que la revérification traite. Le dépôt le sait déjà (`bank_imports.rs:439-446`, `routes/reconciliation.rs:2166-2170`, `reconciliation_e2e.rs:7556`, `:8401`) et ne le traite pas : `map_db_error` ne connaît pas 1020 (`kesh-db/src/errors.rs:1459-1494`), qui sort en `DbError::Sqlx` → `500 INTERNAL_ERROR` (`kesh-api/src/errors.rs:4114-4120`). ⛔ **Non traité par la story** (validation P2, R1 = F1 ; décision de l'orchestrateur) : l'épinglage que la P1 avait placé dans `create_pool` n'aurait atteint aucune connexion — `create_pool` (`kesh-db/src/pool.rs:28`) n'a **aucun appelant**, la production construit son pool dans `kesh-api/src/main.rs:74`, sans `after_connect`. La question (réglages de session du pool réel : `sql_mode`, `time_zone`, `innodb_snapshot_isolation`) est portée par **#669** (bug, P3) ; angle mort écrit (Dev Notes § *Angles morts*). ⚠️ **Version du NAS : inconnue** — `docker-compose.prod.yml:8` et `ci.yml:48` supposent la parité 10.11 avec le paquet MariaDB de Synology, sans relevé ; **fait à constater** par Guy (`SELECT @@version, @@innodb_snapshot_isolation` sur la base du NAS) |
| P-e | Piste : « ajouter le rejeu sur interblocage de part et d'autre de chaque cycle » | **insuffisante seule** : la fenêtre n'est pas un interblocage, c'est **l'absence de conflit** entre deux transactions. Un rejeu ne ferme rien s'il n'y a d'abord un verrou commun ; il ne sert qu'à neutraliser les cycles que ce verrou crée (C-29-4-1) | — |

**W30 n'a pas de fenêtre (invariant I1).** L'annulation de rejet n'agit que sur une transaction **rejetée**
(`classify_unreject`, `routes/reconciliation.rs:4747`), donc `pending` (le rejet ne pose que `auto_match_rejected_at`
sur une ligne `status = 'pending'`, `reconciliation.rs:3448-3452`) ; elle laisse la ligne `pending`
(`reconciliation.rs:4855-4858`). La clôture compte les `pending` **dans l'état commité** (`close_in_tx`,
`bank_accounts.rs:1025-1036`) : tant que la rejetée existe, la clôture rend `412 BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS`.
Aucune clôture ne peut donc réussir pendant une annulation de rejet du même compte — sauf si I1 est déjà défait, cas où
le pré-vol R8 (lecture fraîche sur le pool) voit la clôture commitée. Le test existant
`annuler_un_rejet_pendant_une_cloture_refuse_par_w30` (`bank_accounts_closed_e2e.rs:1366`) le confirme à sa façon : il
**ne peut** construire la course qu'en posant `closed_on` **par SQL direct**. Même raisonnement, déjà écrit par la 28-4,
pour W1 à W6 (acceptation, rejet, manuel, ventilé : la transaction traitée est `pending` jusqu'au commit).

**W24 n'a de fenêtre que pour un import vide.** La suppression d'un import ne vise qu'un import **sans** transaction
rapprochée (`HasReconciled`, `bank_imports.rs:505-510`) : toutes ses transactions sont `pending` (rejetées comprises).
Tant que la suppression n'est pas commitée, la clôture les voit `pending` et refuse. La fenêtre ne reste ouverte que si
l'import n'a **aucune** transaction (relevé vide). Le défaut est réel, mais borné — la story le ferme quand même (AC6),
la revérification coûtant le même geste que sur les autres annulations.

**Fenêtres réelles, donc** : W9, W17, W22, W25, W28, W29 ; W24 pour un import vide. W8 (`settle_invoice`), W27
(`pay_in_tx`, seul ou par `confirm_batch`, W20) et les gestes du compte sur lui-même (W13-W15) **n'en ont pas** : ils
lisent `closed_on` sous un `FOR UPDATE` de la ligne, qui lit le dernier état commité (28-4, AC7).

## Périmètre minimal — ce qui ferme #651, et ce qui en est exclu

**Dedans** :

1. une **revérification verrouillante** de `closed_on` — `SELECT closed_on … LOCK IN SHARE MODE` sur la clé primaire —
   posée sur les sept chemins à fenêtre réelle (W9, W17, W22, W24, W25, W28, W29), à une **position** qui n'ajoute aucun
   verrou (W17, W22) ou qui fait du verrou neuf la **dernière** acquisition de la transaction (les cinq annulations) ;
2. le **rejeu sur interblocage** de la clôture, de la réouverture et du `PATCH` du compte — chacun, demandeur d'un
   exclusif sur la ligne du compte, peut être la victime d'un cycle que la revérification ferme par la file d'attente
   (AC4 ; C-29-4-5, C-29-4-14 et C-29-4-19, décisions de l'orchestrateur en P2) ;
3. le troisième inventaire du registre statique (les sites à garde non verrouillante, chacun résolu) ;
4. la lecture d'`archived` par la revérification de l'**import** et du **lot** — une seconde variante, que les
   annulations n'appellent **jamais** (AC1, AC2 ; C-29-4-16 renversé et C-29-4-20, décision de l'orchestrateur en P2) :
   elle ferme du même geste la course archivage / import (L5 de la route d'archivage) et archivage / lot ;
5. les textes qui décrivent L1 : doc-comments, manuel utilisateur (PDF), `docs/api-external.md`, CHANGELOG.

**Dehors**, et pourquoi :

| Exclu | Raison |
|---|---|
| Retirer les gardes non verrouillantes de la 28-4 | Elles donnent le **refus précoce** et l'**ordre des refus** que la 28-4 a fixés (T-a27, W24 a/b) ; la revérification ne mord que dans la course (C-29-4-3) |
| Rejouer l'import (`POST /bank-imports`) ou la création de lot | Aucun cycle neuf à deux ne les atteint (leur revérification ne demande aucun verrou) ; seul un cycle à trois passant par un demandeur d'exclusif en file — la clôture, ou le `PATCH` du compte (F2 de la P2) — le pourrait, et ce demandeur, plus léger, y est la victime, rejouée (C-29-4-7, C-29-4-19). Rejouer l'import exige d'extraire ~160 lignes de deux chemins (CAMT, CSV) : hors périmètre minimal, résidu écrit |
| Prendre la sentinelle `companies`, un verrou nommé (`GET_LOCK`) ou les lignes enfants dans la clôture | Écartés au registre (C-29-4-1) : cycles avec les routes de gestion, cycles mixtes non détectés par InnoDB, ou ensemble de verrous dont l'analyse rejoue celle que la 28-4 a dû abandonner |
| Réparer les états laissés par la fenêtre de la v0.13.3 (une `pending` ou un lot `generated` sur un compte clos) | Le remède manuel reste (rouvrir, traiter, clore) ; `cancel_batch` (W21) reste la sortie autorisée. Aucune donnée réelle ne le requiert (aucun signalement) |
| L'inversion `companies` → `bank_accounts` de W8 / W27 contre les routes de gestion (28-4, AC7, « au sol, inchangée ») | Préexistante, non touchée par la story : les routes de gestion ne sont pas rejouées, la story ne les enveloppe pas |
| Manuels DE / IT / EN | `docs/manual/{de,en,it}/` ne portent qu'un `README.md` |
| Traiter `innodb_snapshot_isolation` (épinglage de session, ou traduction de 1020) | *(Réécrit en P2, R1 = F1.)* Le seul endroit du dépôt qui pose des réglages de session, `create_pool`, n'a aucun appelant ; poser le réglage sur le pool réel (`main.rs:74`) touche **toute** la base — avec les réglages `sql_mode` et `time_zone` que la doc de `create_pool` décrit et que la production n'a jamais eus. Hors #651, porté par **#669** (P3). Angle mort écrit : sous MariaDB 11.6+, la revérification rendrait `1020` (`500`) dans la course — du même ordre que deux sites préexistants, sans la même issue : `accept_one_invoice` (`routes/reconciliation.rs:2166-2170`) rendrait `DATABASE_ERROR` dans `failed[]`, en `200` ; `bank_imports.rs:439-446` raisonne l'effet sur la suppression d'import sans dire ce qu'elle rendrait *(rectifié en P3, R6 = F6 b)*. Les points sur 11.6+ (manuel d'administration l. 108 et 2708, `docker-compose.prod.yml:8`, version du NAS) sont versés à #669 (commentaire de l'orchestrateur, P3) |
| Fermer l'interblocage **préexistant** import ou lot / route de gestion (l'`INSERT` prend le compte en partagé **avant** `companies` ; une route de gestion prend `companies` en exclusif **avant** le compte — F5 de la P2) | Préexistant au sol, non aggravé par la story, ni l'un ni l'autre rejoués : rien ne passe sur un archivé, mais l'une des deux requêtes peut échouer en `500`. Angle mort écrit (Dev Notes) |

**Décompte des modules** (§ *Règle de splitting préventif*), **recompté en P1** (R6 = F7), **puis en P2** (retrait de
`pool.rs`, R1 = F1 ; deux enveloppes neuves et la variante « archivé » dans des modules déjà comptés), **puis relu en P3**
(R4 = F4 : la fixture `attendre_un_verrou_en_attente` que la P2 ajoutait à `kesh-db/src/test_fixtures.rs` — module compilé
en permanence, hors `cfg(test)`, donc de code — portait la 29-4a à **cinq** sans que le décompte le dise ; la P3 la
**retire**, la sonde existante `sonde_verrou_nowait` la remplaçant, et le chiffre de quatre redevient vrai ; R8 de la P3
ajoute deux fichiers de doc-comments seuls) — un module de code est un
fichier de production dont le **code** change ; un fichier dont seuls des doc-comments changent se compte **à part** ;
les tests, `docs/` et le CHANGELOG aussi :

| | Modules de code | Doc-comments seuls | Documents | Tests |
|---|---|---|---|---|
| **29-4a** | `kesh-db` : `repositories/bank_accounts` (deux fonctions et deux SQL neufs), `repositories/bank_imports` (appel), `repositories/payment_batches` (appel et conversion de l'archivé) ; `kesh-api` : `routes/bank_accounts` (trois enveloppes : clôture, réouverture, `PATCH`) — **quatre** | `kesh-db/src/errors.rs:1247-1249`, `kesh-api/src/routes/bank_imports.rs:57-61` (P3, R8) | `CHANGELOG.md` | `kesh-db/tests/bank_accounts_repository.rs`, `kesh-api/tests/{rejeu_interblocage_e2e,bank_account_closed_inventory,audit_route_registry}.rs` |
| **29-4b** | `kesh-db` : `repositories/reconciliation_cancel`, `repositories/invoice_settlements_write`, `repositories/supplier_invoices`, `repositories/bank_imports` — **quatre** | `kesh-db/src/repositories/bank_accounts.rs` (§ L1, l. 28, 807-823, 863-865), `kesh-api/src/routes/reconciliation.rs:4908` | `docs/manual/fr/user-manual.tex` (+ PDF), `docs/api-external.md`, `CHANGELOG.md` | `kesh-db/tests/{bank_accounts_repository,invoice_settlement,supplier_invoices_repository,reconciliation_cancel}.rs` (répartition au choix du dev), `kesh-api/tests/{rejeu_interblocage_e2e,bank_account_closed_inventory,bank_accounts_closed_e2e}.rs` |

Au total (les deux parties) : **sept** modules de code distincts (`repositories/bank_imports` est dans les deux) — huit
avant la P2, `pool` compris —, plus de cinq, d'où le découpage (C-29-4-6). La 29-4a en touche **quatre**, sous la borne
de la règle (qui découpe au-delà de cinq) — la P2 avait montré qu'un épinglage réellement branché sur le pool de
production l'aurait portée à six (`main.rs`) : le retrait l'éloigne de la borne, aucune dérogation n'est à écrire pour
elle **au décompte de code** ; comptée largement — **règle (P4, L4) : fichiers de production, code ou doc-comments, sans les documents ni les
tests** —, elle touche **six** fichiers de production — la dérogation ci-dessous la couvre par le même argument (ce qui dépasse cinq n'est que du texte) ;
la 29-4b **quatre** de code ; comptée largement par la même règle, **six** (quatre de code, deux de doc-comments
seuls), et **neuf** si l'on y ajoute ses trois documents (`docs/manual`, `docs/api-external.md`, `CHANGELOG.md`) — le
« sept » de la P1 comptait le manuel sans les deux autres documents et ne suivait aucune règle (P4, L4) — plus de cinq : voir § *Dérogation règle de
splitting*. Le découpage suit aussi le **risque** : la 29-4a n'ajoute aucun verrou d'écrivain ; la 29-4b porte seule le
verrou neuf et le cycle qu'il crée, et c'est elle que la validation doit relire le plus durement.

### Dérogation règle de splitting (29-4b)

*(Écrite en P1, R6 = F7 : l'écriture d'origine, « cinq, la borne », était un sous-décompte. Relue en P2 : inchangée —
le retrait de l'épinglage ne touche que la 29-4a, et le test neuf de la 29-4b, T29b-5, va dans un fichier de tests déjà
compté.)* Comptée largement, la 29-4b
touche six fichiers de production (neuf avec ses documents — P4, L4) ; elle n'est **pas** redécoupée :

- **Ce qui dépasse cinq n'est que du texte** : les deux modules ajoutés au décompte (`bank_accounts.rs`,
  `routes/reconciliation.rs`) n'y changent que des doc-comments, et `docs/manual` est un document. Le code de la 29-4b est
  un **rollout mécanique** du patron posé par la 29-4a (un appel identique, cinq sites), ce que la règle désigne comme la
  forme sûre d'une sous-story.
- **Redécouper coûterait plus que la dérogation** : séparer les textes de L1 du code qui les rend vrais ferait vivre sur
  `main` des doc-comments, un manuel et un `api-external.md` faux pendant une PR de plus (déjà le cas, et assumé, entre la
  29-4a et la 29-4b — R12 de la P1) ; déplacer en 29-4a la réécriture de `bank_accounts.rs` la rendrait fausse tant que les
  annulations restent ouvertes.
- **Risque accepté** : la relecture de la 29-4b se fait au fichier près pour les cinq appels (position contre la table
  *Position par chemin*), et en passes adversariales pour le cycle neuf (T29b-3) seulement.
- **La 29-4a, comptée largement** (P3, R8) : quatre modules de code et deux fichiers de doc-comments seuls
  (`kesh-db/src/errors.rs`, `kesh-api/src/routes/bank_imports.rs`), soit six. Elle n'est pas redécoupée pour la même
  raison : les deux fichiers ajoutés ne changent qu'une phrase de doc-comment que la revérification rend incomplète, et
  la séparer du code qui la motive ferait vivre un doc-comment faux sur `main`.

*(Annotation datée du 2026-10-14, revue de code P1 de la 29-4b, A-3 — le relevé ci-dessus est laissé tel qu'écrit.)*
Depuis l'étape 0 (T6-bis, `dba87062`), l'argument « **ce qui dépasse cinq n'est que du texte** » ne vaut plus : la 29-4b
touche **six modules de code** (`reconciliation_cancel`, `invoice_settlements_write`, `supplier_invoices`,
`bank_imports`, `repositories/bank_accounts` — fonction neuve `primary_flag_locked_in_tx` —, `routes/bank_accounts` —
son appel), sept fichiers de production comptés largement (plus `routes/reconciliation.rs`, doc-comment seul), dix
avec les trois documents. La story est **maintenue non redécoupée, sur décision de l'orchestrateur en autonomie** ; motif :
l'étape 0 est la reprise des reports de la 29-4a (R-1, R-2, R-4, R-5 de sa passe ciblée P2), dont le code change d'une
fonction et d'un appel dans deux modules que la 29-4a touchait déjà. Signalé au Project Lead, à qui l'arbitrage revient.

## Critères d'acceptation

### Le critère décidable de la story (remplace, pour les chemins touchés, le premier volet de l'AC7 de la 28-4)

⛔ **Trois volets**, qui se prouvent chemin par chemin dans la table *Position par chemin* (Dev Notes), recopiée au
Dev Agent Record avec la ligne de code retenue :

1. **Verrou** — *chaque verrou que la story ajoute est **déjà tenu** par la transaction au point où il est demandé
   (W17, W22 : le verrou partagé que l'`INSERT` à clé étrangère vient de poser), ou bien il est la **dernière
   acquisition de verrou** de la transaction : seules le suivent l'insertion d'audit (`audit_log`, sans clé étrangère)
   et des lectures non verrouillantes* — avec la borne que la 28-4 avait posée (fiche 28-4, l. 368-372 ; R7a de sa
   validation P3) et que la P1 de la 29-4 a fait recopier (R9) : l'`INSERT` d'audit porte deux sous-`SELECT` sur `users`
   (`audit_log.rs:94`, nom et société de l'acteur), dont le caractère verrouillant **n'est pas établi** (sous `REPEATABLE
   READ` avec journal binaire, un sous-`SELECT` d'un `INSERT` peut poser un partagé sur la ligne `users`). Inoffensif ici
   pour la même raison qu'en 28-4 — aucun chemin ne tient `users` en exclusif en attendant `bank_accounts` —, mais la
   revérification n'est « la dernière acquisition » qu'à cette borne près ;
2. **Cycle** — *tout cycle que ces verrous ferment a **tous** ses participants rejoués sur interblocage* (enveloppes
   `retry_on_deadlock` / `retry_app_on_deadlock` / `retry_with`), ou passe par un demandeur d'exclusif sur la ligne du
   compte que la story fait rejouer (clôture, réouverture, `PATCH` — AC4) — à deux résidus près, écrits (Dev Notes
   § *Cycles*) : le cycle à trois où l'import, non rejoué, serait victime (théorique, C-29-4-7 — il est le plus lourd du
   cycle) ; le cycle à quatre où une route de gestion du compte, non rejouée, serait victime — quel que soit le
   demandeur d'exclusif en file qu'il traverse (clôture, réouverture, `PATCH`, W8, W27) : à égalité de poids avec la
   clôture et la réouverture, **victime désignée** quand ce demandeur est le `PATCH`, W8 ou W27, plus lourds qu'elle
   (quatre gestes concurrents, négligeable — F3 de la P2, réécrit en P3, F2) ;
3. **Vue** — *aucune naissance de vue déplacée* (critère hérité de l'AC7 de la 28-4) : la story n'ajoute **aucune
   lecture non verrouillante** et ne déplace aucune des gardes non verrouillantes de la 28-4 ; la revérification est
   une lecture **verrouillante**, posée **après** la première lecture non verrouillante de chaque chemin, si bien que la
   vue est née avant elle quelle que soit la façon dont InnoDB traite une lecture verrouillante.

### Partie a — `29-4a-reverification-import-lot`

**AC1 — La revérification : deux fonctions, deux SQL, une forme de verrou contrôlée.**

*(Réécrit en P2 — R2 = F4, décision de l'orchestrateur : la revérification de l'import et du lot lit aussi `archived`,
celle des annulations jamais. Symétrie exacte des deux gardes de la 28-4, `ENSURE_OPEN_SQL` /
`ENSURE_OPEN_UNARCHIVED_SQL`, `bank_accounts.rs:750-754`.)*

- Neufs, dans `crates/kesh-db/src/repositories/bank_accounts.rs`, à côté d'`ensure_open_in_tx` (l. 839) :
  - `#[doc(hidden)] pub const RECHECK_OPEN_LOCKED_SQL: &str = "SELECT closed_on FROM bank_accounts WHERE id = ? AND company_id = ? LOCK IN SHARE MODE";`
    et `pub async fn recheck_open_locked_in_tx(tx, company_id, bank_account_id) -> Result<(), DbError>` : `None` →
    `DbError::NotFound` ; `Some(Some(_))` → `DbError::BankAccountClosed { bank_account_id }` ; `Some(None)` → `Ok(())`
    — **la revérification des annulations** (W9, W24, W25, W28, W29 ; 29-4b) ;
  - `#[doc(hidden)] pub const RECHECK_OPEN_UNARCHIVED_LOCKED_SQL: &str = "SELECT closed_on, archived FROM bank_accounts WHERE id = ? AND company_id = ? LOCK IN SHARE MODE";`
    et `pub async fn recheck_open_unarchived_locked_in_tx(tx, company_id, bank_account_id) -> Result<(), DbError>` :
    `None` ou archivé → `DbError::NotFound` ; clos → `DbError::BankAccountClosed { bank_account_id }` ; sinon `Ok(())`
    — **dans l'ordre « archivé avant clos »** de la garde de tête `ensure_open_unarchived_in_tx` (l. 883) ; **la
    revérification de l'import (W22) et du lot (W17)**, seuls chemins dont la garde de tête lit `archived`.
  — constantes publiques et cachées pour que T29a-1 les lise depuis `tests/` (précédent : `LOCK_IMPORT_TRANSACTIONS_SQL`,
  `bank_imports.rs:394`, et son test `lock_query_uses_the_import_index`).
  ⛔ **`LOCK IN SHARE MODE`, jamais `FOR SHARE`** (erreur de syntaxe sur MariaDB 10.11, `bank_accounts.rs:1241`) ;
  ⛔ **`recheck_open_locked_in_tx` ne lit jamais `archived`, et aucune annulation n'appelle la variante** : un compte
  archivé peut porter des règlements et des lots, et il ne se rouvre pas — refuser l'annulation d'un de ses règlements
  la rendrait définitivement impossible (l'interdit d'`ensure_open_in_tx`, `bank_accounts.rs:825-830`, étendu aux deux
  revérifications ; tenu par T29b-5 et par le registre, AC5 et AC8).
- **La variante ne change ni le verrou ni les cycles** (lentille F de la P2, raisonné) : même égalité sur la clé
  primaire, même verrou d'enregistrement quelle que soit la liste des colonnes lues ; l'archivage prend bien
  l'exclusif de la ligne (`archive_for_company`, `FOR UPDATE` l. 691, puis `UPDATE … SET archived = TRUE` l. 719, sous la
  sentinelle `companies` X, `routes/bank_accounts.rs:908`), si bien que la revérification de W22 / W17 le voit comme
  elle voit une clôture (AC2).
- **Forme du verrou** : la recherche est une **égalité sur la clé primaire** — verrou d'enregistrement partagé, **sans
  intervalle**, exactement celui que pose un `INSERT` à clé étrangère vers `bank_accounts(id)` (InnoDB : contrôle de clé
  étrangère sur l'index référencé, `LOCK_S | LOCK_REC_NOT_GAP`). C'est ce qui rend la demande **sans effet** quand le
  verrou est déjà tenu (W17, W22) — prémisse **raisonnée**, que T29a-6 et T29a-7 établissent (P-d (2)). Test T29a-1 :
  `EXPLAIN` de l'instruction rend `key = PRIMARY` et un accès `const` ; l'`EXPLAIN` exige des valeurs, le test les
  substitue en remplaçant chaque `?` de la constante, dans l'ordre, par l'identifiant d'un compte et d'une société de
  sa base (`id`, puis `company_id`), et vérifie avant de l'exécuter qu'il ne reste aucun `?` — **pour chacune des deux
  constantes**.
- **T-a45 de la 28-4, étendu** (`bank_accounts.rs:1521`, `mod tests`) : les deux SQL de garde restent sans verrou ; les
  deux neufs **doivent** contenir `LOCK IN SHARE MODE` et ni `FOR UPDATE` ni `FOR SHARE` (tue M29-3 et M29-19) ;
  `RECHECK_OPEN_LOCKED_SQL` ne contient **pas** `archived`, `RECHECK_OPEN_UNARCHIVED_LOCKED_SQL` le contient (tue
  M29-22 dans sa forme SQL, et la mutation inverse — `archived` ajouté à la revérification des annulations).
- Doc-comment des deux fonctions : la règle de position (critère, volet 1, avec la borne de l'audit), les deux usages
  (verrou déjà tenu / dernière acquisition), l'interdit d'appeler la variante depuis une annulation (et sa raison),
  l'interdit de la poser avant une écriture qui prend un autre verrou, sa dépendance **non épinglée** à
  `innodb_snapshot_isolation = OFF` (sous `ON`, défaut de MariaDB 11.6+, elle rendrait `1020` dans la course — angle mort
  renvoyé à #669), l'argument de poids qui fait des demandeurs d'exclusif les victimes de leurs cycles (F11 de la P2,
  Dev Notes § *Cycles*) **avec ses deux réserves** (P3 : raisonné, établi au dépôt à deux participants seulement — F3 ;
  sans effet contre une route de gestion en file sur la sentinelle — F2), les deux résidus (C-29-4-7, le cycle à
  quatre), et la raison du choix (C-29-4-1, C-29-4-2).

**AC2 — L'import (W22) et la création de lot (W17) : la revérification après l'`INSERT` à clé étrangère.**

- **W22** — `bank_imports::create_with_transactions` (`bank_imports.rs:67`) : appel de
  `recheck_open_unarchived_locked_in_tx(tx, new.company_id, new.bank_account_id)` **immédiatement après** l'étape 1
  (`INSERT INTO bank_imports`, l. 82-104, et ses contrôles de `last_insert_id`, l. 106-113), **avant** l'étape 2
  (`INSERT INTO bank_transactions`). La garde non verrouillante de tête (l. 79, `ensure_open_unarchived_in_tx`)
  **reste** : refus précoce, archivé compris. Un archivé rend `DbError::NotFound`, que la route traduit déjà en `404
  BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND` (`import_create_error`, comme pour la garde de tête) : rien à ajouter.
- **W17** — `payment_batches::create_batch` (`payment_batches.rs:69`) : appel de `recheck_open_unarchived_locked_in_tx`
  **immédiatement après** l'`INSERT INTO payment_batches` (l. 142-154), avant l'`UPDATE` du `msg_id` (l. 159) et les
  `INSERT` d'articles. La lecture de l'étape (0) (l. 85-108, `closed_on` et `archived`) **reste**. **Conversion au site
  d'appel** : `DbError::NotFound` → `DbError::IllegalStateTransition("le compte bancaire source est archivé".into())`,
  le refus même de l'étape (0) (l. 99-103) — sûre : après l'`INSERT` à clé étrangère la ligne existe, et sa société a
  été contrôlée à l'étape (0) (`WHERE id = ? AND company_id = ?`) ; un `NotFound` ne peut plus vouloir dire
  qu'« archivé ». Les autres erreurs passent telles quelles.
- **Pourquoi c'est sans verrou neuf** : l'`INSERT` d'une ligne dont `bank_account_id` référence `bank_accounts(id)` a
  posé, et tient jusqu'au commit, un verrou partagé sur la ligne du compte (existence : T-a31 de la 28-4 le vérifie en
  pratique). La revérification redemande ce verrou-là : InnoDB le trouve tenu, n'en crée pas et n'attend pas — y compris
  quand une clôture attend déjà en exclusif (**raisonné**, P-d (2), établi par T29a-6 et T29a-7). Elle lit, comme toute
  lecture verrouillante sous `innodb_snapshot_isolation = OFF` (P-f ; défaut de MariaDB 10.11, **non épinglé** — #669),
  **le dernier état commité**.
- **Pourquoi cela ferme la fenêtre** : (i) si la clôture tient déjà la ligne en exclusif, l'`INSERT` l'attend ; au
  commit de la clôture, il passe, et la revérification lit `closed_on` posé → `DbError::BankAccountClosed` → la route
  rend `409 BANK_ACCOUNT_CLOSED` (import : `import_create_error`, `routes/bank_imports.rs` ; lot : `?` existant) et la
  transaction est abandonnée, **rien d'écrit** ; (ii) si l'écrivain a inséré d'abord, la clôture attend son commit,
  puis le voit (I1 : transactions `pending` ; I2 : lot `generated`) et refuse — c'est le sol (T-a31).
- **Et la course avec l'archivage** (P2, R2 = F4 ; raisonné) : (iii) archivage commité avant l'`INSERT` → l'`INSERT`
  attend la ligne du compte (ou `companies`, que l'archivage tient en exclusif par la sentinelle), puis la
  revérification lit `archived = TRUE` → refus, rien d'écrit (W22 : `404` ; W17 : `IllegalStateTransition`, comme son
  étape (0)) ; (iv) `INSERT` d'abord → l'archivage attend la sentinelle `companies` (l'`INSERT` y tient un partagé),
  puis son décompte, dont la vue naît après la sentinelle, voit les transactions de l'import → `412
  BANK_ACCOUNT_HAS_TRANSACTIONS` (`routes/bank_accounts.rs:909-916`). ⚠️ **Ce que cela répare, et ce que non** : pour
  W22, la story empêche un état que l'archivage interdit (un archivé porteur de transactions). Pour W17, c'est de la
  **cohérence** — le refus de l'étape (0) tenu jusqu'au commit —, non une réparation : l'archivage ne compte pas les
  lots (seules les `bank_transactions`), si bien qu'« un lot créé, puis le compte archivé » est un enchaînement permis
  au sol hors course. Reste l'interblocage **préexistant** import ou lot / route de gestion (F5 de la P2, table
  *Dehors*).
- **Ordre des refus inchangé** hors course : la garde de tête répond la première ; la revérification ne répond que si
  une clôture a été commitée entre la naissance de la vue et l'`INSERT`.
- Ce que cela coûte à l'import : un relevé dont l'analyse a commencé avant la clôture est refusé **après** analyse (le
  travail est perdu, il faut rouvrir le compte et réimporter) — c'est le comportement voulu, écrit au manuel en 29-4b.

**AC3 — Les tests de la fenêtre, rouges d'abord (W22, W17).**

Au dépôt (`crates/kesh-db/tests/bank_accounts_repository.rs`, module `compte_clos`, patron de T-a31), pour chacun des
deux chemins : (1) une transaction `C` exécute `close_in_tx` **sans commiter** (elle tient la ligne en exclusif) ;
(2) l'écrivain démarre (`tokio::spawn`) dans sa propre transaction — pour l'import, après une **lecture non
verrouillante** préalable dans sa transaction (la recherche de doublons de la route : `find_by_company_and_hash`), afin
que sa vue naisse **avant** le commit de la clôture, comme en production ; (3) `attendre_une_requete_en_cours(&pool,
&["INSERT INTO bank_imports"], …)` (resp. `["INSERT INTO payment_batches"]`, `test_fixtures.rs:561`) le voit en attente ;
(4) `C.commit()` ; (5) l'écrivain doit rendre `DbError::BankAccountClosed` ; pour W22, dont la tâche tient la
transaction, elle fait alors `tx.rollback().await` **explicite** avant de rendre son résultat (l'abandon par `drop` est
asynchrone, `routes/bank_imports.rs:1196-1201`) ; pour W17, `create_batch` possède sa transaction
(`payment_batches.rs:80`) et l'annule lui-même avant de rendre l'erreur (`let _ = tx.rollback().await;`, l. 231 et 236) :
la tâche n'a rien à faire *(précisé en P3, R10)* ; et seulement ensuite le test vérifie qu'aucune ligne `bank_imports` / `bank_transactions` (resp. `payment_batches` /
`payment_batch_items`) n'existe. **Rouge au sol** : l'écrivain aboutit. Tests **T29a-2** (W22) et **T29a-3** (W17) ;
tuent M29-1 et M29-2 (et M29-3 : la lecture redeviendrait celle de la vue). ⚠️ Au sol, l'étape (3) passe aussi
(l'`INSERT` attend déjà la clôture) : le test rougit à l'étape (5), pour la bonne raison.

**La course avec l'archivage** (T29a-11 pour W22, T29a-12 pour W17 ; P2, R2 = F4) — même patron, l'archivage à la place
de la clôture : (1) une transaction `A` prend la sentinelle (`acquire_company_sentinel_lock`, comme la route) puis
exécute `archive_for_company` sur un compte **sans transaction**, **sans commiter** ; (2) l'écrivain démarre, vue née
avant le commit (pour l'import, `find_by_company_and_hash` ; pour le lot, l'étape (0) de `create_batch`, qui lit le
compte non archivé) ; (3) la sonde le voit en attente sur son `INSERT` ; (4) `A.commit()` ; (5) l'écrivain doit rendre
`DbError::NotFound` (W22) — resp. `DbError::IllegalStateTransition` (W17, la conversion de l'AC2) —, `rollback`
(explicite dans la tâche pour W22, par `create_batch` lui-même pour W17 — R10 de la P3), puis aucune ligne écrite. **Rouge au sol** : l'écrivain aboutit sur un compte archivé. Tuent M29-22
(`archived` retiré de la variante, SQL ou `match`), M29-23 (W22 ou W17 appelle `recheck_open_locked_in_tx`) et, pour
T29a-12, M29-25 (conversion retirée : `NotFound` au lieu d'`IllegalStateTransition`).

**L'autre ordre — l'écrivain a inséré, la clôture attend, puis l'écrivain revérifie** (T29a-6 pour W22, T29a-7 pour W17 ;
R5 = F2 de la P1). C'est le seul ordre qui éprouve la prémisse P-d (2), et AC3 ci-dessus ne le monte pas (à l'étape (4)
la clôture a déjà commité, plus d'exclusif en file). Patron de T-a31 de la 28-4, prolongé : (1) l'écrivain, dans sa
transaction, exécute son `INSERT` d'entête (`INSERT INTO bank_imports` — resp. `INSERT INTO payment_batches`) **sans** la
revérification — **par le SQL de l'`INSERT`**, seule voie une fois la revérification posée (`create_batch` ouvre sa
propre transaction, `payment_batches.rs:80`) ; (1 bis) **sonde de référence, avant tout lancement de la clôture** *(P4, M1)* : le test appelle une fois la
sonde de l'étape (3) — elle **doit** rendre `true` (le seul partagé de l'écrivain ne la fait pas échouer), faute de quoi
le test échoue avec un message de montage ; placée avant l'étape (2), elle ne peut pas être battue par la clôture, dont
la demande exclusive n'est pas encore en file ; (2) la clôture démarre en tâche (`close_in_tx`,
**sans** enveloppe de rejeu, commit si `Done`) ; (3) le test attend que la demande exclusive de la clôture soit **dans
la file** de la ligne du compte, par la sonde existante `sonde_verrou_nowait` (`kesh-db/src/test_fixtures.rs:607`)
appelée en boucle sur une connexion tierce du pool — `sonde_verrou_nowait(&pool, "SELECT id FROM bank_accounts WHERE id
= ? LOCK IN SHARE MODE NOWAIT", compte)`, tous les 10 ms, bornée à 10 s, la boucle interrompue (échec avec message) si
la tâche de la clôture se termine sans attendre — jusqu'à ce qu'elle rende `false` (la boucle ne sert qu'à
attendre ce `false` ; la garde du montage à vide est la sonde de référence de (1 bis), non un premier tour de boucle). Tant que seul le partagé de
l'écrivain est tenu, la demande partagée de la sonde est compatible et réussit ; dès que la demande exclusive de la
clôture est en file, la sonde échoue aussitôt en `1205`, une demande neuve étant jugée contre les demandes en attente
comme contre les verrous accordés ; et comme rien d'autre ne tient la ligne dans un mode incompatible, ce `false` ne
peut venir que de la clôture. La boucle est locale au fichier de test : **aucune fixture neuve**, **aucun privilège**
(la sonde lit `bank_accounts`, que l'utilisateur de dev lit déjà ; ni `information_schema.INNODB_TRX` ni le privilège
`PROCESS`). *(Réécrit en P2, R4 = F7 : un délai de 500 ms ne prouvait pas que la demande exclusive était dans la file ;
réécrit à nouveau en P3, R1 = F1, décision de l'orchestrateur, C-29-4-21 annoté : la sonde de `INNODB_TRX` prescrite
en P2 exige le privilège `PROCESS`, que les droits versionnés de dev ne donnent pas —
`scripts/mariadb-init/01-dev-grants.sql`, `reconciliation_e2e.rs:7546-7553`.)* **Constaté le 2026-10-14** sur la
MariaDB de dev (`10.11.16`), base `kesh_294`, table jetable à clé primaire, trois connexions du client `mariadb`
(utilisateur `kesh`) : la sonde partagée `NOWAIT` **réussit** quand une autre transaction tient la ligne en partagé, et
**échoue aussitôt en `1205`** dès qu'une troisième y demande l'exclusif en file ; après les deux commits, elle réussit
de nouveau. Le même essai a vu le détenteur partagé **redemander** son partagé pendant que l'exclusif attendait, sans
attendre ni interblocage — indice pour P-d (2), sur une lecture `LOCK IN SHARE MODE` et non sur le contrôle de clé
étrangère d'un `INSERT` : la prémisse reste à établir par T29a-6 et T29a-7. Ce constat est hors dépôt : le dev le
**rejoue** dans le test et l'écrit au Dev Agent Record. ⚠️ **La forme `LOCK IN SHARE MODE NOWAIT` n'a aucun précédent
dans le dépôt** — tous les sites existants (`grep -rnF "FOR UPDATE NOWAIT" crates/`) sont en `FOR UPDATE NOWAIT`, aucun en `LOCK IN SHARE MODE NOWAIT`, et le
doc-comment du helper (`test_fixtures.rs:607`) raisonne l'exclusif ; son comportement n'a été constaté que hors dépôt, sur
MariaDB 10.11.16 (remédiation P3), et **s'établit au T0 du développement**, avec le repli ci-dessous si elle ne se
comportait pas ainsi (le helper panique sur toute erreur autre que `1205` : une syntaxe refusée échouerait bruyamment).
Le doc-comment du helper se généralise à cette occasion (`FOR UPDATE NOWAIT` ou `LOCK IN SHARE MODE NOWAIT` ; `false` =
`1205`) *(P4, L1)*. *Repli*, si la sonde ne se comportait pas ainsi sur la branche
rebasée : le précédent du dépôt (`reconciliation_e2e.rs:7538-7553`, le motif « 500 ms » aux l. 7538-7540) — attente bornée constatée par
`attendre_une_requete_en_cours`, deux échantillons à 500 ms d'écart, puis l'issue après le commit — avec, écrit au Dev
Agent Record, le constat que le test ne distingue plus « prémisse vraie » de « montage arrivé trop tôt » ; jamais une
exigence de privilège. La sonde éprouve du même geste la prémisse « file InnoDB » sur laquelle repose l'AC4 ; (4) l'écrivain
appelle `recheck_open_unarchived_locked_in_tx` sous `tokio::time::timeout(Duration::from_secs(2), …)` : il **doit**
rendre `Ok(())` dans le délai ; (4 bis) **W22 seulement** — l'écrivain insère une transaction `pending` de son import
(`INSERT INTO bank_transactions`, par SQL, sous le même `timeout`), dans l'ordre de production « entête →
revérification → transactions » *(ajouté en P2, R3 = F6 : sans elle, la clôture ne comptait aucune `pending`, rendait
`Done`, et le test rougissait quand la prémisse est vraie)* ; l'`INSERT` redemande le partagé déjà tenu, même prémisse ;
(5) l'écrivain commite ; (6) la clôture **doit** rendre `Refused(PendingTransactions { pending_count: 1 })` (W22, la
transaction de l'étape (4 bis)) — resp. `Refused(OpenPaymentBatches { batch_count: 1 })` (W17, le lot inséré à
l'étape (1) est `generated`, `payment_batches.rs:144-146`). **Ce qui rougit si la prémisse est fausse** : si la redemande se mettait en file
derrière l'exclusif, InnoDB verrait le cycle écrivain / clôture et en tuerait un — la clôture, plus légère : elle rendrait
alors `Err(DbError::Sqlx(1213))` à l'étape (6) au lieu du refus (le test n'enveloppe pas `close_in_tx` dans un rejeu,
précisément pour que l'interblocage se voie) ; si l'écrivain était la victime, l'étape (4) rendrait l'erreur. Si
T29a-6 ou T29a-7 rougit pour cette raison, la conséquence s'écrit au Dev Agent Record (P-d (2) fausse : la fenêtre reste
fermée, mais « aucun verrou neuf » est à réécrire et chaque course coûte un rejeu de la clôture), la conception n'est
pas à refaire. Tue M29-19 (revérification en `FOR UPDATE` : la demande exclusive d'un détenteur partagé, derrière une
autre demande exclusive en file, attendrait — cycle).

**AC4 — Le rejeu de la clôture, de la réouverture et du `PATCH` du compte.**

*(Étendu en P2 sur décisions de l'orchestrateur : la réouverture **est** rejouée — F3, la décision (2) de la P1 reposait
sur un argument faux, un cycle à quatre existe dont elle est victime (C-29-4-14 renversé) ; le `PATCH` du compte **est**
rejoué — F2, cycle à trois W24 / `PATCH` / import (C-29-4-19). Une enveloppe chacun, même patron que la clôture.)*

- `close_bank_account` (`routes/bank_accounts.rs:1005`) : la section `begin` … `commit` passe dans une fonction « une
  tentative » (`close_once`), enveloppée par `crate::retry::retry_app_on_deadlock("bank_accounts::close", …)` — patron de
  `bank_imports::delete` → `delete_once` (`routes/bank_imports.rs:1461-1490`). Les contrôles de forme (version, borne de
  `closedOn`) et `assert_onboarding_complete` restent **devant** l'enveloppe. Le refus métier (`412`, `409`) n'est pas
  rejoué ; aucune conversion ne fait sortir un 1213 d'`AppError::Database(DbError::Sqlx(_))` avant le prédicat —
  aujourd'hui `close_in_tx` rend ses erreurs par `Err(e) => AppError::Database(e)`, à garder. **Sur refus**, `close_once`
  fait `tx.rollback().await` **explicite** avant de rendre l'erreur (patron de `delete_once`, `routes/bank_imports.rs:1477-1482` pour la règle, `:1549` pour le geste) :
  la transaction tient le `FOR UPDATE` du compte, et un abandon implicite ne le rendrait qu'au retour de la connexion au
  pool — autant d'attente en plus pour les écrivains partagés en file derrière la clôture.
- **La réouverture est rejouée** (P2, F3 ; C-29-4-14 renversé) : `reopen_bank_account` (`routes/bank_accounts.rs:1082`)
  → `reopen_once`, enveloppée par `retry_app_on_deadlock("bank_accounts::reopen", …)` ; contrôles de forme et
  `assert_onboarding_complete` devant l'enveloppe ; `rollback` explicite sur refus (`409 BANK_ACCOUNT_NOT_CLOSED`, `409`
  de version). ⚠️ **L'argument écrit en P1 pour ne pas la rejouer était faux** (R9 = F3 de la P2) : « toute écriture sur
  un clos est refusée **avant** de prendre ce partagé » — or W22 et W17 prennent le partagé du compte par leur `INSERT`
  **avant** que la revérification les refuse, et les annulations de la 29-4b le prennent **par** la revérification ; et
  une demande partagée attend dès qu'une demande exclusive est **en attente** devant elle, y compris celle de la
  réouverture. Cycle à quatre (raisonné, F3) : `Q` (annulation, contre-passée, `companies` S) → `R` (réouverture, `A` X
  en attente derrière `I`) → `I` (import, `A` S par son `INSERT`, `companies` S en attente derrière `M`) → `M` (route de
  gestion, `companies` X en attente derrière `Q`) → `Q` (revérification, `A` S derrière `R`). Quatre gestes
  concurrents : négligeable, mais non « impossible par construction » ; une enveloppe le rend invisible quand la
  réouverture est la victime (Dev Notes § *Cycles* pour le résidu : `M`, à égalité de poids avec la réouverture ou la
  clôture, victime désignée quand le demandeur en file est le `PATCH`, W8 ou W27 — F2 de la P3).
- **Le `PATCH` du compte est rejoué** (P2, F2 ; C-29-4-19) : `patch_bank_account_journal_link`
  (`routes/bank_accounts.rs:1142`) → `patch_once` (la transaction : `claims_for_target`, partagés sur
  `company_invoice_settings` ; `set_journal_account_id_for_company`, `FOR UPDATE` du compte, `bank_accounts.rs:462` ;
  audit ; commit), enveloppée par `retry_app_on_deadlock("bank_accounts::patch", …)` ; la version et
  `validate_journal_account_id` (sur le pool) restent devant l'enveloppe ; `rollback` explicite sur refus (`404`, `409`,
  `400 ACCOUNT_NOT_POSTABLE`, `400 BANK_ACCOUNT_LEDGER_IS_CLAIM_ACCOUNT`). Les erreurs de dépôt y sortent déjà en
  `AppError::Database(e)` (`Err(e) => return Err(AppError::Database(e))`, et `?` sur `DbError` pour les partagés des
  réglages), donc visibles du prédicat. **Pourquoi** : cycle à trois (raisonné, F2) — un import `I` sur `A` a passé sa
  revérification (il tient `A` en partagé) et attend l'intervalle d'`idx_bank_transactions_import` que tient une
  suppression d'import `D` (W24, import le plus récent de l'installation) ; le `PATCH` demande `A` en exclusif derrière
  `I` ; `D` arrive à sa revérification et attend derrière le `PATCH` en file : `D → PATCH → I → D`. Le `PATCH`, qui n'a
  rien écrit, est le plus léger → victime — `500` sans enveloppe. Rejoué, il ferme aussi l'angle mort « PATCH » par
  `accounts` que la P1 avait écrit (Dev Notes § *Cycles*).
- Doc-comment de la route `close_bank_account` (`routes/bank_accounts.rs:1000-1004`, F3 de la P1) : « Pas de rejeu sur
  interblocage : un seul verrou, en tenant rien, la route ne peut pas fermer de cycle » devient faux — réécrit : la
  route ne tient rien **comme détenteur**, mais sa demande exclusive en attente peut fermer un cycle à trois, dont elle
  est la victime désignée ; d'où le rejeu. Même ajout aux doc-comments de `reopen_bank_account` (l. 1073-1081) et de
  `patch_bank_account_journal_link` (l. 1130-1141), chacun avec son cycle.
- **Pourquoi** : la clôture ne tient **rien** pendant qu'elle attend sa ligne ; sa demande exclusive **en attente** fait
  pourtant attendre derrière elle toute demande partagée ultérieure sur la même ligne (file InnoDB : une demande neuve
  attend un verrou incompatible **en attente** comme un verrou accordé — remarque R7 b de la validation P3 de la 28-4).
  Elle peut donc fermer un **cycle à trois** (Dev Notes § *Cycles*) ; elle y est la transaction la plus légère (aucune
  écriture, aucun verrou accordé), donc la **victime** désignée — prémisse que le dépôt tient déjà pour établie et sur
  laquelle reposent ses tests de rejeu (`rejeu_interblocage_e2e.rs:9-11`, « InnoDB choisit pour victime la
  transaction la plus légère »). La rejouer rend ce cas invisible à l'utilisateur. ⚠️ *(P3, F3)* Le dépôt ne l'établit
  que pour des cycles **à deux** (tous les tests « route victime » du fichier) ; que la victime d'un cycle **à trois**
  soit le plus léger de tout le cycle, et non de la seule paire qui le ferme, est **raisonné** — T29a-4 et T29a-9 sont
  les premiers tests à trois, et leur contingence est écrite ci-dessous.
- `audit_route_registry.rs`, point (vi) du doc-comment (l. 136) : « **sept** routes `SansEcritureAuJournal` sont
  rejouées » → **dix**, et `close_bank_account`, `reopen_bank_account`, `patch_bank_account_journal_link` nommées avec
  leur raison ; leurs classements (`Traced`, `SansEcritureAuJournal`, l. 333, 339, 340) ne changent pas. Sept recomptées
  au sol par les deux lentilles de la P1 et la lentille R de la P2 (`onboarding::finalize`, `update_invoice_settings`,
  `create_fiscal_year`, `close_fiscal_year`, `onboarding::seed_demo`, `onboarding::reset`, `bank_imports::delete`) ; les
  recompter **à nouveau** sur la branche rebasée avant d'écrire dix. Le commentaire des l. 336-338 (« un seul verrou …
  — aucune enveloppe de rejeu (C-28-4-21) ») devient faux : réécrit (« clore et rouvrir : rejouées, chacune victime
  possible d'un cycle par la file d'attente — Story 29-4 ») ; un commentaire d'une ligne s'ajoute au-dessus du `PATCH`
  (l. 333) — R6 de la P2.
- Doc-comment de `close_in_tx` (`bank_accounts.rs:987-1002`) : « il ne peut fermer aucun cycle » reste vrai **comme
  détenteur** ; ajouter qu'en attente il peut être la victime d'un cycle à trois, d'où le rejeu de la route ; même
  ajout pour `reopen_in_tx` et `set_journal_account_id_for_company`.
- **Test T29a-4** (`rejeu_interblocage_e2e.rs`, patron du fichier : transactions de test alourdies de 500 lignes de lest,
  témoin `warn!` de `kesh_db::retry` capturé par `common::capture_rejeu`). ⛔ **Séquence réécrite en P1 (R1 = F1)** : la
  séquence d'origine attendait `T1` en ligne **avant** d'annuler `T2` ; or la victime (la clôture) ne tient rien, son
  annulation ne libère pas `R`, et `T2` — dont la demande partagée est accordée dès que l'exclusif en file disparaît —
  garde `R` : le test se serait bloqué sur lui-même jusqu'au `1205`. Le patron `victime()` (`rejeu_interblocage_e2e.rs:347-372`)
  ne s'applique pas tel quel : il suppose que la victime **est** la route qui tient la ressource demandée. **Montage**
  *(P3, R2)* : le montage du fichier (`seed_accounting_company`, `spawn_app`, `forge_jwt`) ne termine pas l'onboarding,
  or `close_bank_account` et `reopen_bank_account` l'exigent (`assert_onboarding_complete`,
  `routes/bank_accounts.rs:269-275` ; le `PATCH` non) : le test appelle `mark_onboarding_complete(&pool)`
  (`kesh_db::test_fixtures`), comme `bank_accounts_closed_e2e.rs:162`, sans quoi la route rend l'erreur d'onboarding et
  jamais le cycle. Séquence réalisable :
  1. `T1` (alourdie) tient la ligne du compte en partagé (`SELECT … FROM bank_accounts WHERE id = ? LOCK IN SHARE MODE`) ;
  2. la clôture (`POST /bank-accounts/{id}/close`) est lancée en tâche et vue en attente (`["FROM bank_accounts",
     "archived = FALSE FOR UPDATE"]`) ;
  3. `T2` (alourdie) tient `R` — la ligne `companies` de la société en exclusif (`SELECT id FROM companies WHERE id = ?
     FOR UPDATE`) —, puis demande la ligne du compte en partagé **en tâche** : elle attend derrière la demande exclusive
     de la clôture (prémisse « file InnoDB », raisonnée) ;
  4. `T1` demande `R` **en tâche** (`tokio::spawn`) : cycle `T1 → T2 → clôture → T1` ;
  5. le test attend l'événement de rejeu `bank_accounts::close` — sondage de `capture.operations()`
     (`common/capture_rejeu.rs:94`) jusqu'à ce qu'il le contienne, borné à 10 s, puis `exiger_un_rejeu` (l. 105) — c'est
     l'**issue** qui établit l'attente de `T2`, la sonde ne distinguant pas une requête bloquée d'une requête en cours
     (P5 de la 28-4) ;
  6. la tâche de `T2` rend sa demande partagée accordée ; **`T2` annule** (`rollback`), ce qui libère `R` ;
  7. la tâche de `T1` est jointe avec `.expect` : `T1` a obtenu `R` — preuve qu'elle n'était pas la victime ;
  8. `T1` annule ; la clôture rejouée obtient sa ligne et rend `200`.
  Tue M29-4. ⚠️ Si `T2` n'attend pas derrière la clôture (prémisse « file InnoDB » fausse), sa demande est accordée à
  l'étape (3), `T1` attend `R` sans cycle, et aucun rejeu ne vient à l'étape (5) : le test échoue sur la borne avec un
  message qui le dit. Cela s'écrit alors au Dev Agent Record — la conséquence serait que le rejeu de la clôture n'a pas
  d'objet, non qu'il est nuisible ; le test se réécrit en garde de non-régression de l'enveloppe, sans cycle.
  ⚠️ **Seconde contingence — une victime autre que la clôture** *(P3, F3)* : si InnoDB désigne `T1` ou `T2` (alourdies)
  — `T1` reçoit `1213` à l'étape (7), ou `T2` à l'étape (6), et aucun événement `bank_accounts::close` ne vient —, le
  test rougit, et le Dev Agent Record consigne la victime **observée** (l'erreur rendue à `T1` ou `T2`, le message de
  rejeu ou son absence). La conséquence est plus large que le test : en production, la victime d'un cycle à trois
  pourrait être l'import ou une annulation, que la story ne rejoue pas ; l'argument de poids et le résidu C-29-4-7
  sont alors à réécrire, et le dev le **remonte à l'orchestrateur** avant de poursuivre — la conception n'est pas à
  refaire d'office.
- **Test T29a-9** (réouverture, P2) — la séquence de T29a-4 à l'identique, sur un compte **clos** au montage, avec la
  réouverture (`POST /bank-accounts/{id}/reopen`) à la place de la clôture (même motif de sonde : `close_in_tx` et
  `reopen_in_tx` partagent le `FOR UPDATE` de `bank_accounts.rs:931`) ; événement `bank_accounts::reopen` attendu ;
  la réouverture rejouée rend `200`. Tue M29-20 (enveloppe de la réouverture retirée). Le cycle à quatre de F3 n'est
  **pas** monté : le cycle à trois de la séquence suffit à prouver que la réouverture, victime, est rejouée. Même
  montage (`mark_onboarding_complete`, R2 de la P3) et mêmes deux contingences que T29a-4, la réouverture à la place de
  la clôture (F3 de la P3).
- **Test T29a-10** (`PATCH`, P2) — patron `victime()` (`rejeu_interblocage_e2e.rs:347-372`) tel quel, car le `PATCH`
  **tient** la ressource que la transaction de test demande : `T` (alourdie) tient la ligne du compte (`SELECT … FROM
  bank_accounts WHERE id = ? FOR UPDATE`) ; le `PATCH` (corps avec un `journalAccountId`, pour que `claims_for_target`
  prenne ses partagés sur `company_invoice_settings`) est lancé en tâche et vu en attente (`["FROM bank_accounts",
  "archived = FALSE FOR UPDATE"]`) ; fermeture : `SELECT … FROM company_invoice_settings WHERE company_id = ? FOR
  UPDATE` — cycle à deux ; le `PATCH`, plus léger, est la victime ; `T` obtient sa ligne (`.expect`), annule ; le
  `PATCH` rejoué rend `200`, événement `bank_accounts::patch` capturé. Tue M29-21 (enveloppe du `PATCH` retirée). Le
  cycle à trois de F2 n'est pas monté (même raison que T29a-9). **Précondition** *(P3, F5)* : la société porte sa ligne
  `company_invoice_settings` — `seed_accounting_company` l'insère (`test_fixtures.rs:157`), le fichier est donc servi ;
  sans elle, le partagé du `PATCH` et l'exclusif de la transaction de test seraient des verrous d'**intervalle**,
  compatibles entre eux : aucun cycle, et le test échouerait sur l'événement de rejeu pour une raison que sa
  description ne dit pas.
- Les points (ii) et (iii bis) du doc-comment d'`audit_route_registry.rs` (l. 56-80, routes « à preuve dynamique »)
  gagnent les trois routes et leurs tests T29a-4, T29a-9, T29a-10 — par une phrase propre à la famille `AppError`
  (`retry_app_on_deadlock`) : les « sept » de ces deux points (l. 58-62, l. 73) comptent la famille `DbError` et
  **restent** sept ; seul le point (vi) passe de sept à dix *(précisé en P3, R9)*.

**AC5 — Le registre statique : l'inventaire clos des gardes non verrouillantes.**

Test existant `crates/kesh-api/tests/bank_account_closed_inventory.rs` (31 sites d'écriture, 27 entrées au `REGISTRE`,
3 au `APPELANTS_REGLEMENT` — recomptés en P1, R7) : il gagne un **troisième inventaire**, tenu dans les deux sens (comme
les deux autres), test **T29a-5** :

- **extraction des appels** : tout appel `recheck_open_locked_in_tx(` **ou** `recheck_open_unarchived_locked_in_tx(`
  (préfixés de `bank_accounts::` ou `super::bank_accounts::`, nom entier comparé — aucun n'est préfixe de l'autre) hors
  tests et hors commentaires dans `kesh-db/src` et `kesh-api/src`, rattaché à sa fonction englobante (analyseur partagé
  de C-28-4-29) **avec le nom appelé** (P2, R2 = F4) ;
- **extraction des gardes non verrouillantes** (C-29-4-15, R3 de la P1 — depuis le symptôme, non depuis le texte des
  raisons du registre principal) : l'ensemble `G` des fonctions englobantes, hors tests et hors commentaires, dans
  `kesh-db/src` et `kesh-api/src`, qui (a) appellent `bank_accounts::ensure_open_in_tx(` ou
  `bank_accounts::ensure_open_unarchived_in_tx(` (copie masquée ; **qualifiés**, comme l'extraction des appels
  ci-dessus — la forme `super::bank_accounts::` les contient ; sans préfixe, les deux définitions,
  `bank_accounts.rs:839` et `:871`, seraient comptées et `G` vaudrait onze — R12 de la P3 ; au sol, les sept appelants
  sont tous qualifiés), ou (b) portent un **littéral** contenant `closed_on` et `FROM bank_accounts` et ni `FOR UPDATE` ni `LOCK IN
  SHARE MODE` — **hors** `repositories/bank_accounts.rs`, dont les lectures de `closed_on` sont les gardes elles-mêmes
  (vues par (a) chez leurs appelants), les gestes du compte sur lui-même (sous `FOR UPDATE`) et les lectures de détail
  qu'emploient les pré-vols. Au sol, `G` vaut **neuf** fonctions (inventaire de la lentille R de la P1, recoupé ici :
  `grep -rn "closed_on" crates/kesh-db/src crates/kesh-api/src` hors `bank_accounts.rs` ne rend comme SQL que
  `payment_batches.rs:90` et `:439`, sans verrou, et `invoice_settlements_write.rs:138`, `supplier_invoices.rs:760`, sous
  `FOR UPDATE`) : W9 `reconciliation_cancel::cancel_in_tx`, W17 `create_batch`, W20 `confirm_batch`, W22
  `create_with_transactions`, W24 `delete_unreconciled_in_tx`, W25 `cancel_settlement_in_tx`
  (`invoice_settlements_write`), W28 `supplier_invoices::cancel_in_tx`, W29 `supplier_invoices::cancel_settlement_in_tx`,
  W30 `unreject_once` ;
- **registre** `FENETRE: &[(fichier, fonction, Fenetre)]`, une ligne par fonction de `G`, avec un statut **typé** :
  - `Reverifiee(Variante, "<position>")` — l'appel existe dans la fonction, et `Variante` dit lequel des deux
    (`Variante::Clos` → `recheck_open_locked_in_tx`, `Variante::ClosOuArchive` →
    `recheck_open_unarchived_locked_in_tx` ; ajouté en P2) ; en 29-4a : W17 (`create_batch`) et W22
    (`create_with_transactions`), tous deux `ClosOuArchive` ; en 29-4b, les cinq annulations, toutes `Clos` ;
  - `SansFenetre("<invariant ou verrou>")` — W20 `confirm_batch` (« chaque facture par `pay_in_tx`, compte lu sous
    `FOR UPDATE` ; et I2 »), W30 `unreject_once` (en 29-4a : « I1, sous réserve de la 29-4b — W9 peut encore remettre une
    transaction `pending` sur un compte clos » ; en 29-4b : « I1 : la transaction est `pending`, la clôture refuse » —
    R12 b de la P1) ;
  - `Ouverte("29-4b")` — en 29-4a seulement : W9, W24, W25, W28, W29. ⛔ La 29-4b **retire** cette variante (plus
    aucun site), sans quoi elle serait morte sous `clippy -D warnings`.
  ⚠️ W1-W6 et `accept_one_*` **n'y figurent pas** (correction de la P1) : ce ne sont pas des gardes non verrouillantes
  en transaction mais des `Invariant("I1", …)` du registre principal, que la story ne touche pas.
- **assertions** : (i) l'ensemble des couples (fonction, nom appelé) extraits **égale** l'ensemble des couples
  (fonction, `Variante`) des `Reverifiee` — une annulation qui appellerait la variante « archivé », ou l'import qui
  appellerait celle des annulations, rougit (tue M29-23 et M29-24 statiquement) ; (i bis) une fonction de `FENETRE`
  dont le chemin est une annulation (W9, W24, W25, W28, W29) ne peut porter que `Variante::Clos` — assertion écrite en
  dur, avec sa raison (un archivé ne se rouvre pas), pour que la déclaration elle-même ne puisse pas dériver ; (ii) l'ensemble `G` **égale** l'ensemble des fonctions de `FENETRE` — égalité, dans les deux sens : une
  garde non verrouillante neuve sans statut rougit, une entrée sans garde rougit ; (iii) **ordre textuel** : les
  positions se lisent **dans la source de la fonction**, en mêlant deux sources de positions — les jetons de **code**
  dans la copie masquée (l'appel de revérification, `reverse_in_tx(`, …) et les jetons **SQL** dans les **littéraux**
  (`Litteral.debut` de `lexique::decouper`, `crates/kesh-db/tests/lexique/mod.rs:17-26`), la copie masquée effaçant le
  contenu des chaînes (R2 de la P1). Pour W22, l'appel suit le littéral qui contient `INSERT INTO bank_imports` et
  précède celui qui contient `INSERT INTO bank_transactions` (le `QueryBuilder::new("INSERT INTO bank_transactions …")`
  est un littéral) ; pour W17, il suit le littéral qui contient `INSERT INTO payment_batches`. Tue M29-6 et M29-7 — un
  appel déplacé avant l'`INSERT` prendrait un verrou explicite **avant** l'`INSERT`, à une position que l'analyse n'a
  pas faite, et aucun test fonctionnel ne le verrait.
- **Volet inverse, par cas construit** : chaque assertion est éprouvée sur une source fabriquée qui doit la faire
  rougir (une garde non verrouillante sans entrée ; un appel avant l'`INSERT` ; une annulation qui appelle
  `recheck_open_unarchived_locked_in_tx`), sans quoi un extracteur qui ne voit rien la ferait passer à vide.
- Les textes de raison du registre principal qui citent L1 (l. 89 — doc de la variante `SortieAutorisee` —, 115, 233,
  290) restent exacts en 29-4a ; la 29-4b les réécrit (AC9).
- Doc-comment du test : ce qu'il voit (présence, ensemble des gardes non verrouillantes, ordre textuel), ce qu'il ne
  voit pas — que la revérification s'exécute (affaire des tests d'entrelacement) ; une écriture par une fonction appelée
  **après** l'appel, que l'ordre textuel d'une seule fonction ne suit pas (d'où, pour les annulations, l'ordre exigé par
  rapport aux appels nommés, AC8) ; une lecture non verrouillante de `closed_on` dont le SQL ne porte pas littéralement
  `FROM bank_accounts` (requête assemblée, `format!`) ; les **pré-vols** de route sur le pool (`is_closed()`,
  `routes/bank_imports.rs:994`, `:1151`, `routes/reconciliation.rs:1051`, `:3262`, `:3661`, `:4110`, `:4934`,
  `routes/bank_accounts.rs:892`), hors transaction, que suit chacun une garde ou un invariant du registre principal.

**AC11 — retiré en validation P2** (R1 = F1, HIGH convergent ; décision de l'orchestrateur ; C-29-4-13 annoté).

La P1 avait prescrit d'épingler `innodb_snapshot_isolation = OFF` dans `create_pool` (`kesh-db/src/pool.rs:28`), « à
côté du `SET SESSION sql_mode …` existant ». **`create_pool` n'a aucun appelant** : la production construit son pool
dans `kesh-api/src/main.rs:74` (`MySqlPoolOptions::new()`, sans `after_connect`), et ce depuis la Story 1.2 — le
réglage n'aurait atteint aucune connexion, et son test, qui appelait la fonction de configuration directement, aurait
certifié la prémisse fausse. Le brancher sur le pool réel n'est pas un geste de la story : il ferait passer en
production les réglages `sql_mode` et `time_zone` que la doc de `create_pool` décrit et que la production n'a jamais eus
(changement de comportement hors #651), ou exigerait un `after_connect` neuf dans `main.rs` (un module de plus, au-delà
de la borne de la 29-4a). **Retrait plutôt qu'ajout** : la configuration de session, `configurer_session`, le test
T29a-8 et la mutation M29-5 sortent de la story. La question entière — `create_pool` mort, `sql_mode`, `time_zone`,
`innodb_snapshot_isolation` du pool réel — est portée par **#669** (bug, P3).

Ce qui reste, écrit comme **angle mort renvoyé à #669** (Dev Notes § *Angles morts*, doc-comment des deux fonctions de
revérification) : sous `innodb_snapshot_isolation = ON` — défaut de MariaDB à partir de la **11.6** —, une lecture
verrouillante qui rencontre une version de ligne postérieure à la vue ouverte rend `ER_CHECKREAD` (**1020**) : dans la
course même qu'elle traite, la revérification rendrait `500 INTERNAL_ERROR` (`map_db_error` ignore 1020,
`kesh-db/src/errors.rs:1459-1494` ; rendu `kesh-api/src/errors.rs:4114-4120`) — l'écrivain resterait refusé, rien
d'écrit, mais par un `500` que rien ne distingue d'une panne. **Du même ordre que deux sites préexistants, sans la
même issue** *(rectifié en P3, R6 = F6 b — la fiche écrivait « comme les deux sites préexistants »)* :
`accept_one_invoice` (`routes/reconciliation.rs:2166-2170`) rendrait `DATABASE_ERROR` par proposition, dans `failed[]`,
en `200` ; `bank_imports.rs:439-446` raisonne l'effet sur la suppression d'import (« sans effet sur le code nominal …
à revoir à la montée de version ») sans dire ce qu'elle rendrait. Leurs doc-comments ne sont pas retouchés **au titre
de l'AC11** ; `bank_imports.rs:444-445` se réécrit pourtant en 29-4b, pour une autre raison — W24 y ouvre sa vue avant
la revérification (AC9 ; R3 de la P3). Constaté sur la base de dev : `10.11.16`, variable à `OFF` ; les
tests `#[sqlx::test]` sur un serveur 11.6+ le signaleraient (T29a-2, T29a-3, comme `reconciliation_e2e.rs:7556` et
`:8401`). Le texte que l'AC9 prescrit au manuel (« elle est refusée avec le message ci-dessus ») vaut sous
`innodb_snapshot_isolation = OFF` — MariaDB 10.11, celle de la CI (`ci.yml:48`) et de la base de dev. Les points sur
11.6+ — le manuel d'administration qui annonce « MariaDB 10.11+ » (l. 108) et suggère « 11.5+ » (l. 2708),
`docker-compose.prod.yml:8` qui admet ces versions, la version et le réglage du serveur du NAS — ont été **versés en
commentaire à #669** par l'orchestrateur (validation P3, F6 = R5) : la story ne prescrit aucun texte de manuel pour
11.6+ et ne touche pas le manuel d'administration.

### Partie b — `29-4b-reverification-annulations`

**AC6 — Les cinq annulations : la revérification en dernière acquisition de verrou.**

À chaque site, **après la dernière écriture** de la fonction de dépôt et **avant** son insertion d'audit, si le
chemin porte un compte bancaire (mêmes conditions que la garde non verrouillante de la 28-4) :

| Site | Fonction | Position (sur `16eeed21`) | Compte |
|---|---|---|---|
| W9 | `reconciliation_cancel::cancel_in_tx` (`reconciliation_cancel.rs:279`) | après l'étape (6) (W25 imbriqué ou `reverse_in_tx`, l. 360-386), avant (7) (l. 388) | `bt.bank_account_id` |
| W25 | `invoice_settlements_write::cancel_settlement_in_tx` (`:826`) | après l'étape (6) (`UPDATE invoices`, l. 953-986), avant (7) (l. 988) | `settlement_bank_account_id`, si non nul |
| W28 | `supplier_invoices::cancel_in_tx` (`:1061`) | après l'étape (4) (`UPDATE supplier_invoices`, l. 1150-1168), avant (5) (l. 1169) | `inv.settlement_bank_account_id`, si non nul (lu avant l'`UPDATE` qui le remet à `NULL`) |
| W29 | `supplier_invoices::cancel_settlement_in_tx` (`:1292`) | après l'étape (4) (l. 1395-1413), avant (5) (l. 1414) | idem |
| W24 | `bank_imports::delete_unreconciled_in_tx` (`:472`) | après le `DELETE` de l'étape 4 (l. 525-534), avant le `Ok(Deleted…)` ; l'audit est écrit par la route (`delete_once`) | `import.bank_account_id` |

- **Toujours `recheck_open_locked_in_tx`, jamais la variante** (`closed_on` seul ; AC1) : une annulation sur un compte
  **archivé** passe, revérification comprise — tenu par T29b-5 et par l'assertion (i bis) du registre.
- **W9 attend sous le verrou nommé du compte** (F10 de la P2, transitoire, à écrire au doc-comment de W9) :
  `cancel_reconciliation_once` (`routes/reconciliation.rs:4617`) exécute `cancel_in_tx` dans `with_account_lock` (l. 4631) ; pendant
  qu'une clôture en file attend derrière un import long, la revérification de W9 attend en gardant `GET_LOCK(A)`, et les
  autres gestes de réconciliation du compte échouent en `ReconciliationAccountLocked` au bout de `LOCK_TIMEOUT_SECS`.
  Aucun cycle mixte (raisonné, F10) : aucun détenteur d'un partagé sur `A` ne demande ensuite `GET_LOCK(A)`, les routes
  de réconciliation le prenant en tête.
- **La garde non verrouillante de la 28-4 reste à sa place** à chaque site (refus précoce, ordre des refus de la 28-4 :
  W24 « `HasReconciled` d'abord », motifs d'exercice clos après la garde, etc.). La revérification ne répond que si une
  clôture a été commitée après la naissance de la vue de l'annulation. Le commentaire de chaque garde
  (`reconciliation_cancel.rs:320-324`, `invoice_settlements_write.rs:892-894`, `supplier_invoices.rs:1097` et `:1338`,
  `bank_imports.rs:516-522`) gagne une demi-ligne qui renvoie à la revérification posée plus bas *(P3, R8)*.
- **Pourquoi cela ferme la fenêtre** : l'annulation tient désormais, de sa revérification à son commit, un verrou
  partagé sur la ligne du compte. (i) Clôture déjà en exclusif : la revérification l'attend, puis lit `closed_on` posé
  → `BankAccountClosed`, transaction abandonnée, rien d'écrit ; (ii) revérification passée d'abord : la clôture attend
  le commit de l'annulation, puis la voit — pour W9, la transaction redevenue `pending` → `412
  BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS` ; pour W24, l'import supprimé ; pour W25, W28, W29, le règlement annulé, puis
  la clôture passe : l'annulation a eu lieu **sur un compte encore ouvert**, ce qui est l'ordre juste.
- **W9 imbrique W25** (étape (6)) : la revérification de W25 s'exécute dans W9, sur le compte du règlement — le même
  que celui de la transaction pour un règlement né d'un rapprochement (`reconciliation.rs`, `BankTransfer {
  bank_account_id }` d'`accept_one_invoice`) ; la revérification de W9, juste après, redemande alors un verrou déjà
  tenu. Si les deux comptes devaient différer (question F11 de la validation P2 de la 28-4, **non établie**), deux
  verrous partagés se prennent en fin de transaction, sur deux lignes : la position « dernière acquisition » vaut pour
  les deux, et la table des cycles ne change pas (aucun partenaire n'y demande ces lignes dans l'ordre inverse en
  exclusif sans être rejoué).
- **Ce qui rend la position juste, et pourquoi elle n'est pas « au plus tôt »** : posée **juste après le verrou du
  document** (avant l'exercice), la revérification tiendrait la ligne du compte **avant** que l'annulation prenne
  `companies` en partagé (`INSERT INTO journal_entries` de la contre-passation, clé étrangère `company_id`) — ordre
  inverse de celui des routes de gestion (`companies` en exclusif, puis le compte), qui **ne sont pas rejouées** : un
  `500` sur `PUT`, `POST` ou `DELETE /bank-accounts` (le cycle R1 de la validation P2 de la 28-4). Posée **en dernier**,
  elle prend le compte **après** `companies`, dans l'ordre des routes de gestion : aucun cycle avec elles. Le cycle
  qu'elle ferme à la place est celui de la validation P1 de la 28-4 (F1 : compte → exercice contre exercice → compte),
  avec des partenaires **tous rejoués** (Dev Notes § *Cycles*). C-29-4-2.

**AC7 — Les tests de la fenêtre, rouges d'abord (les cinq annulations), et l'autre entrelacement.**

- **Fenêtre** (au dépôt, patron de l'AC3) — pour chaque site, sur un compte **ouvert** portant l'objet à annuler,
  réglé **par virement** (sans quoi la garde ne passe pas) : (1) `C` exécute `close_in_tx` sans commiter — ce qui exige
  que le compte n'ait ni `pending` ni lot `generated` au moment de la clôture : pour W9, un rapprochement de type
  `Entry` (toutes les transactions de l'import rapprochées) ; pour W24, un **import vide** ; pour W25, W28, W29, un
  règlement par virement (aucune transaction en attente) ; (2) l'annulation démarre dans sa transaction ; (3) la sonde la
  voit en attente sur la revérification (`["FROM bank_accounts", "LOCK IN SHARE MODE"]`) — ⚠️ **au sol, l'annulation
  n'attend pas** : le test attend donc « l'annulation vue en attente **ou** terminée » (borne 10 s) avant l'étape
  suivante ; (4) `C.commit()` ; (5) l'annulation doit rendre `BankAccountClosed` ; sa transaction est annulée par
  `rollback().await` **explicite** (AC3) avant tout comptage ; elle ne doit avoir rien écrit (règlement présent,
  transaction toujours `reconciled`, aucune contre-passation, import présent). **Rouge au sol** à l'étape (5). Cinq
  tests, **T29b-1** (W9, W24, W25, W28, W29) ; tuent M29-8 à M29-12.
- **Autre entrelacement**, **T29b-2** — W9 et W25 : l'annulation, appelée dans une transaction de test **non
  commitée**, a passé sa revérification ; la clôture lancée en parallèle est vue en attente (`archived = FALSE FOR
  UPDATE`) ; commit de l'annulation ; la clôture rend, pour W9, `PendingTransactions { pending_count: 1 }`, pour W25,
  `Done`. **M29-13** (revérification non verrouillante, qui ne fait pas attendre la clôture) est tuée par la moitié
  **W9** : sans attente, la clôture rend `Done` au lieu de `PendingTransactions` — et M29-8 à M29-12 la tuent aussi
  (l'annulation ne serait pas refusée dans le premier test). ⚠️ La moitié **W25** ne la tue pas, et ne le prétend plus
  (R5 de la P2) : la clôture ne compte ni règlements ni écritures (`bank_accounts.rs:1023-1046`), si bien que, sous la
  mutation, elle rend `Done` et le règlement est absent **aussi** ; cette moitié garde seulement que l'annulation
  revérifiée ne bloque pas la clôture qui la suit.
- **Compte archivé**, **T29b-5** (P2, R2 = F4) — W25 : un règlement par virement vers un compte `A`, puis `A` archivé
  (possible : l'archivage ne compte que les `bank_transactions`, `routes/bank_accounts.rs:909-916`) ; l'annulation du
  règlement **doit** réussir, revérification comprise. Tue M29-24 (W25 appelle `recheck_open_unarchived_locked_in_tx` :
  l'annulation rendrait `NotFound`). Un test sur W25 seul, non cinq : les cinq sites appellent la même fonction, que
  l'assertion (i bis) du registre tient pour chacun (C-29-4-22).
- **Les tests qui gardent la naissance de la vue restent verts, sans retouche** : T-a48 (a)-(c) de la 28-4
  (`invoice_settlement.rs`, `supplier_invoices_repository.rs`) — ils exigent que les gardes non verrouillantes de la
  28-4 restent après le verrou d'exercice (volet 3 du critère). *(Correction de la P1, F9 : la fiche rangeait ici
  `a_concurrent_close_waits_for_the_unreconciliation`, `crates/kesh-db/tests/reconciliation_cancel.rs:128` ; ce test porte
  sur la clôture d'**exercice** — il garde le verrou d'exercice de l'étape (3) de W9, non la garde du compte, et refuse
  avant la revérification. Il reste vert sans retouche, pour cette raison-là.)* M29-14 (déplacer une revérification
  **avant** le verrou d'exercice) : tuée par l'ordre textuel de l'AC8 ; elle ne déplace pas la vue (lecture
  verrouillante), et les tests de vue ne la voient pas — dit tel.

**AC8 — Le registre : `Ouverte` disparaît, l'ordre textuel s'étend aux annulations, et le cycle neuf se prouve.**

- Registre `FENETRE` de l'AC5 : W9, W24, W25, W28, W29 passent en `Reverifiee(Variante::Clos, …)` ; la variante
  `Ouverte` est retirée ; les assertions (i), (i bis) et (ii) restent — (i bis) prend alors son sens plein (tue M29-24
  statiquement).
- **Ordre textuel** (assertion (iii) étendue, test **T29b-4**), dans la source de chaque fonction, positions mêlées
  comme à l'AC5 (R2 de la P1) : l'appel de revérification (copie masquée) **suit** la dernière occurrence, dans le corps,
  de tout geste qui prend un verrou — **jetons SQL, lus dans les littéraux** (`Litteral.debut`) : `INSERT INTO`,
  `UPDATE `, `DELETE FROM`, `FOR UPDATE`, `LOCK IN SHARE MODE` ; **jetons de code, lus dans la copie masquée** :
  `reverse_in_tx(`, `reverse_owned_in_tx(`, `cancel_settlement_in_tx(` (W9) — et **précède** `audit_log::insert_in_tx(`
  (copie masquée) quand la fonction en porte un. ⚠️ Sans les littéraux, l'assertion ne voyait que les jetons de code :
  pour W24 (`DELETE FROM bank_imports`, un littéral, dernier geste), W28 et W29 (`UPDATE supplier_invoices`, un
  littéral, après `reverse_owned_in_tx(`), un appel déplacé **avant** la dernière écriture passait — le volet 1 n'aurait
  été gardé que pour W9 et W25. Liste des jetons recopiée au doc-comment, avec son angle mort (un geste verrouillant
  appelé par une fonction au nom absent de la liste, ou un SQL assemblé hors littéral). Tue M29-14, M29-15, M29-16,
  M29-17, M29-18.
- **Le cycle neuf, prouvé une fois** (**T29b-3**, `rejeu_interblocage_e2e.rs`, patron du fichier) : `T` (alourdie) tient la
  ligne du compte en exclusif — c'est ce que fait `settle_invoice` (W8) à son étape (3) — ; l'annulation d'un
  règlement par virement (`POST /invoices/{id}/settlements/{sid}/cancel`) est vue en attente sur sa revérification ;
  `T` demande l'exercice de ce règlement en exclusif (`SELECT … FROM fiscal_years WHERE id = ? FOR UPDATE`), que
  l'annulation tient depuis (2-bis) : cycle ; InnoDB annule l'annulation (plus légère) ; `T` **doit** obtenir
  l'exercice ; `T` annule ; l'annulation rejouée rend `200` et l'événement `invoices::cancel_settlement` est capturé.
  Il prouve que le cycle compte → exercice / exercice → compte que la story **crée** est détecté (`1213`, non `1205`) et
  rejoué. Les quatre autres annulations ferment le même cycle, avec des partenaires rejoués (table des cycles) : un
  test chacune n'est **pas** exigé — choix C-29-4-12. Réalisable tel quel (les deux lentilles de la P1) : la victime,
  l'annulation, tient l'exercice que `T` demande ; le patron `victime()` (`rejeu_interblocage_e2e.rs:347-372`)
  s'applique avec `motifs = ["FROM bank_accounts", "LOCK IN SHARE MODE"]`.

**AC9 — Les textes de L1, réécrits ; le CHANGELOG ; et ce qui ne se réécrit pas.**

- **29-4a** : doc-comments qui décrivent l'import ou la création de lot comme exposés à L1 —
  `bank_imports.rs:72-78`, `payment_batches.rs:85-88` — réécrits (« revérifié sous le verrou de clé étrangère, après
  l'`INSERT`, AC2 de la 29-4 ») ; le § *Angle mort assumé — L1* d'`ensure_open_in_tx` (`bank_accounts.rs:807-823`)
  retire l'import et la création de lot de ses chemins et renvoie à la revérification — son décompte écrit (« Huit
  chemins ») se refait depuis la liste qu'il énumère, non depuis le chiffre (R7 de la P3) ; le doc-comment de
  `DbError::BankAccountClosed` (`kesh-db/src/errors.rs:1247-1249`, « Produite par les gardes du dépôt … ») et celui
  d'`import_create_error` (`routes/bank_imports.rs:57-61`, `NotFound` « ne naît que de la garde
  `ensure_open_unarchived_in_tx` ») nomment aussi la revérification `recheck_open_unarchived_locked_in_tx` (R8 de la
  P3 ; doc-comments seuls, décompte ci-dessus) ; `CHANGELOG.md`, section
  **`## [0.13.4] — Non publié`** (créée, ou complétée **par union** si la 29-2 ou la 29-3 l'ont ouverte), `### Corrigé` :
  « **Un import ou un lot de paiement commencé juste avant la clôture — ou l'archivage — d'un compte bancaire ne passe
  plus sur ce compte** ([#651](https://github.com/guycorbaz/kesh/issues/651)) : il est refusé, et rien n'est
  enregistré. » *(Libellé élargi en P2 à l'archivage, AC2 ; le message cité « Ce compte bancaire est clos » retiré : un
  archivé rend un autre refus.)* S'y ajoutent, en 29-4a, les doc-comments de l'AC4 (routes `close_bank_account`,
  `reopen_bank_account`, `patch_bank_account_journal_link` ; `close_in_tx`, `reopen_in_tx`,
  `set_journal_account_id_for_company`) ; le doc-comment d'`ensure_open_unarchived_in_tx` (`bank_accounts.rs:866-867`,
  « Ne ferme pas la course archivage / import (L5 de la route d'archivage) : la lecture d'`archived` est non
  verrouillante ») → « La course archivage / import est fermée par la revérification de l'import,
  `recheck_open_unarchived_locked_in_tx` (Story 29-4) » ; le commentaire de la sentinelle de l'archivage
  (`routes/bank_accounts.rs:904-907`, « bank_imports n'acquiert pas ce lock — L5 limitation v0.1 documentée »), resté
  littéralement vrai, complété : « … ; la course qu'il laissait est fermée par la revérification de l'import et du lot
  (Story 29-4) » (R2 de la P2). *(Les doc-comments de `create_pool`, `bank_imports.rs:439-446` et
  `routes/reconciliation.rs:2166-2170` que prescrivait l'AC11 ne se touchent plus **au titre de l'AC11** : AC11 retiré,
  #669 — `bank_imports.rs:444-445` se réécrit en 29-4b pour une autre raison, ci-dessous ; R3 de la P3.)*
  ⚠️ **État transitoire assumé** (R12 a de la P1) : entre le merge de la 29-4a et celui de la 29-4b, la puce du manuel
  (« pour un import dès l'envoi du fichier — peut encore aboutir ») et `docs/api-external.md:241` restent écrits pour
  les sept chemins, donc **trop larges** pour l'import et le lot ; le § L1 d'`ensure_open_in_tx`, lui, est réécrit en
  29-4a (ci-dessus) et ne figure plus dans cette liste *(R7 de la P3 : la fiche le disait à la fois réécrit et laissé
  trop large)*. C'est la
  règle d'inclusion du `CLAUDE.md` (« batched dans la PR de la dernière story » quand plusieurs stories y contribuent) ;
  aucune release ne tombe entre les deux PR (Status). À écrire tel au Dev Agent Record de la 29-4a.
- **29-4b** :
  - § *Angle mort assumé — L1* d'`ensure_open_in_tx` **remplacé** par un § « *La fenêtre de la clôture — fermée par la
    revérification (Story 29-4, #651)* » : pourquoi la garde non verrouillante seule laissait passer, la revérification
    et sa position, les deux chemins sans fenêtre par invariant (W30, W1-W6) ; le renvoi à #651 reste, comme histoire.
    Le doc-comment d'`ensure_open_unarchived_in_tx` (l. 863-865, « angle mort L1 (#651) ») suit, ainsi que le
    doc-comment de module (`bank_accounts.rs:28`, « … les trois invariants et l'angle mort L1 » → « … et la fenêtre de la
    clôture, fermée par la revérification » — R8 de la P1).
  - `bank_imports.rs:444-445` : « sans effet sur le code nominal (aucune vue n'est ouverte avant les verrous) »
    devient faux pour W24, dont la garde de la 28-4 ouvre la vue avant la revérification — réécrit : la revérification
    de W24 s'exécute vue ouverte ; sous `innodb_snapshot_isolation = ON` (11.6+) elle rendrait `1020` (`500`) dans la
    course — #669 (R4 de la P1 ; réécrit en P2, l'épinglage étant retiré ; « comme les deux sites préexistants » retiré
    en P3, R6).
  - Registre (`bank_account_closed_inventory.rs`) : les raisons des l. 115, 233, 290 — « sous L1 (#651), le pré-vol
    R1 refuse » devient « I1 ; et le pré-vol R1 refuse un compte clos » ; W21 : « annuler un lot `generated` laissé sur
    un compte clos par la fenêtre de la v0.13.3 (#651, fermée en v0.13.4) » ; **l. 89**, doc de la variante
    `SortieAutorisee` (« …réparer ce que l'angle mort L1 (#651) a laissé passer ») → « …réparer ce que la fenêtre de la
    clôture (#651, fermée en v0.13.4) a pu laisser passer jusqu'à la v0.13.3 » (R8 de la P1 : l. 89 n'est pas une raison
    mais un doc-comment, qui n'avait pas de texte prescrit) ; la raison de W30 à `FENETRE` perd sa réserve « sous réserve
    de la 29-4b » (AC5).
  - `routes/reconciliation.rs:4908` (doc-comment de `closed_account_failed`, le `failed[]` des lots d'acceptation R1 et
    de rejet R2 : « sous l'angle mort L1 (#651), c'est ce pré-vol … qui les arrête » — libellé corrigé en P2, R8 ; le
    pré-vol de l'annulation de rejet, lui, est l. 4933-4934), `bank_accounts_closed_e2e.rs:18` (en-tête) et
    `bank_accounts_closed_e2e.rs:1577` (doc de `annuler_un_lot_reste_permis_sur_un_compte_clos` : « le geste qui répare
    ce que L1 a laissé passer » — R8 de la P1) : la mention de L1 se met au passé ou au conditionnel (« comme l'aurait
    fait la fenêtre de #651 »). ⚠️ `routes/reconciliation.rs` est partagé avec la 29-2 et la 29-3 : lire après rebase.
  - **Manuel utilisateur** `docs/manual/fr/user-manual.tex`, § *Clore un compte bancaire*, puce l. 1748-1752
    (« Une opération déjà en cours … peut encore aboutir sur le compte (issue \#651). Le refus des écritures vaut donc
    hors de cette fenêtre … ») **remplacée** par : « Une opération **déjà en cours** au moment de la clôture — dans un
    autre onglet ou par une intégration — ne passe pas sur le compte clos : ou bien elle se termine **avant**, et la
    clôture en tient compte (elle peut alors être refusée, par exemple si l'opération a remis une transaction à
    rapprocher), ou bien elle est **refusée** avec le message ci-dessus. Pour un import, le refus peut venir après
    l'analyse du relevé : rouvrir le compte, puis réimporter. » Toute autre mention greppée (`grep -n "651\|fenêtre\|en
    cours" docs/manual/fr/user-manual.tex`). PDF régénéré (`make fr` dans `docs/manual/`, sous verrou), commité,
    **contrôlé aplati** : `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '` ne contient plus « hors
    de cette fenêtre » et contient « ne passe pas sur le compte clos ».
  - `docs/api-external.md:241` (puce « ⚠️ **Angle mort : une écriture déjà en cours au moment de la clôture** »)
    **remplacée** par une puce « *Depuis la v0.13.4* » : une écriture en cours au moment de la clôture est refusée en
    `409 BANK_ACCOUNT_CLOSED` (rien d'écrit), ou se termine avant et la clôture la voit ; pour une intégration, un import
    peut être refusé **après** l'envoi du fichier ; la clôture peut **attendre** quelques instants qu'une écriture en
    cours se termine. La phrase « Le refus ci-dessus vaut donc **hors de cette fenêtre** » disparaît ; la note de la
    puce précédente (« **Exception** : l'annulation d'un lot généré… ») reste.
  - `docs/api-external.md:555` (*Compte bancaire clos*, lots d'acceptation et de rejet ; R7 de la P2) : « … ce refus
    n'est atteint que par une course — un lot préparé avant la clôture — ou dans la fenêtre de [#651](…) (note ⁹ du
    §7) » → « … ce refus n'est atteint que par une course — un lot préparé avant la clôture. » *(La clause « la note ⁹
    du §7, si elle ne sert plus qu'à cette phrase, se relit » est retirée en P3, R11 : la note ⁹, l. 235-241, est celle de
    la clôture, appelée aux l. 220 et 719 ; elle ne dépend pas de cette phrase.)*
  - `docs/api-external.md:594` (*Annuler le rejet d'une transaction bancaire* ; F7 de la P3) : « Une transaction rejetée
    comptant comme en attente, la clôture d'un tel compte est elle-même refusée (`412
    BANK_ACCOUNT_HAS_PENDING_TRANSACTIONS`) ; ce refus n'est donc atteint que par une course (note ⁹ du §7). » → « … ;
    le 409 de l'annulation du rejet n'est donc atteint que sur un compte clos que la fenêtre de la v0.13.3 a laissé
    dans cet état ([#651](…), fermée en v0.13.4 ; note ⁹ du §7). » *(P4, L2 : « ce refus » désignait ambigument le
    412 de la clôture ou le 409 de l'annulation.)* — la course qui remettait une transaction `pending` sur un compte clos (W9, W22) est
    fermée par la story.
  - `CHANGELOG.md`, `## [0.13.4] — Non publié`, `### Corrigé` : la ligne de la 29-4a **complétée** (par union) : « …
    ni une annulation (rapprochement, règlement client ou fournisseur, facture fournisseur payée), ni la suppression
    d'un import vide : l'opération est refusée, ou se termine avant la clôture, qui en tient compte. La remarque de la
    v0.13.3 (« une opération déjà en cours au moment de la clôture peut encore aboutir ») ne vaut plus. » *(R11 de la
    P1 : la suppression d'un import n'est pas une « annulation » — le bouton s'appelle « Supprimer l'import ».)*
  - ⛔ **Ne se réécrivent pas** : la section `## [0.13.3] — 2026-10-14` du CHANGELOG (relevé daté d'une release
    publiée, l. 37) ; la fiche et le registre de la 28-4 (relevés de leurs passes) ; l'issue #651 (l'orchestrateur la
    commente).
- **Propagation** (§ *Propagation post-patch*, greper la **valeur**) : `grep -rn "651\|\bL1\b" crates docs CHANGELOG.md
  frontend/src website README.md` à chaque PR, **et la formulation** `grep -rn "par une course\|hors de cette fenêtre"`
  sur les mêmes chemins (F7 de la P3 : le jeton `651` ne voyait pas `api-external.md:594`), chaque site trié (réécrit,
  ou laissé avec sa raison) au Dev Agent Record.

**AC10 — Gates (les deux parties : la 29-4a comme la 29-4b ; R13 de la P1).**

- **`kesh-db` touché** : **gate complet** (`scripts/test-fast.sh`, base `kesh_294` remise à zéro avant) à chaque commit
  de code, **même en boucle de revue** — jamais de gate ciblé (CLAUDE.md, exception `kesh-db`).
- **E2E complet** au dernier commit de code de chaque partie (frontend inchangé, mais la règle vaut pour toute story ;
  `KESH_PORT=3045`, base `kesh_e2e_294`, recette de `docs/testing.md` à la lettre) ; échecs attendus jugés **fichier par
  fichier** contre `docs/testing.md` § *Les échecs attendus*.
- Frontend : `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` — inchangés, à rejouer au push (aucune clé
  i18n neuve : la story n'ajoute aucun texte visible hors du manuel).
- Campagne de **mutations** (§ *Mutations*), motif vérifié unique, restauration `git checkout --` **puis `touch`**.

## Tasks / Subtasks

### 29-4a

- [ ] **T0** — Rebase sur `origin/main` après la 29-2 et la 29-3 ; relire **sur la branche rebasée** chaque numéro de
  ligne cité (fichiers de dépôt, `routes/bank_accounts.rs`, `audit_route_registry.rs`, le registre). Bases `kesh_294`,
  `kesh_e2e_294`, port 3045, `CARGO_TARGET_DIR=<worktree>/target`.
- [ ] **T1** — `RECHECK_OPEN_LOCKED_SQL` et `RECHECK_OPEN_UNARCHIVED_LOCKED_SQL` (`#[doc(hidden)] pub`),
  `recheck_open_locked_in_tx` et `recheck_open_unarchived_locked_in_tx`, doc-comments ; T-a45 de la 28-4 étendu (verrou
  et `archived`) ; T29a-1 (`EXPLAIN`, les deux constantes) (AC1).
- [ ] **T1-bis** — *retirée en P2* (AC11 sorti de la story, R1 = F1 ; #669).
- [ ] **T2** — Tests de fenêtre T29a-2 (W22) et T29a-3 (W17), et de la course avec l'archivage T29a-11 (W22) et
  T29a-12 (W17), **d'abord** (rouges au sol, journal du rouge au Dev Agent Record), puis l'appel de la variante après
  chaque `INSERT` et la conversion de W17 (AC2, AC3) ; puis T29a-6 et T29a-7 (l'autre ordre, prémisse P-d (2), sonde
  `sonde_verrou_nowait` en `LOCK IN SHARE MODE NOWAIT`, en boucle locale au test, avec sa sonde de référence (1 bis) avant le
  lancement de la clôture — P3, R1 = F1 ; P4, M1) — leur verdict, vert ou rouge, s'écrit au Dev Agent Record et met à jour le libellé
  de P-d.
- [ ] **T3** — `close_once`, `reopen_once`, `patch_once` et leurs enveloppes (rollback explicite sur refus) ;
  doc-comments des trois routes et de `close_in_tx`, `reopen_in_tx`, `set_journal_account_id_for_company` ; point (vi)
  d'`audit_route_registry.rs` recompté (sept → dix), points (ii) et (iii bis) (une phrase pour la famille `AppError`,
  leurs « sept » inchangés), commentaire des l. 333 et 336-338 ; T29a-4, T29a-9 (montage `mark_onboarding_complete`),
  T29a-10 (AC4).
- [ ] **T4** — Troisième inventaire du registre (T29a-5) : extraction `G`, extraction des appels **avec le nom appelé**,
  `FENETRE` à neuf entrées, `Reverifiee(Variante, …)`, statut `Ouverte("29-4b")` pour les cinq annulations,
  assertions (i) et (i bis), ordre textuel par positions de littéraux, volet inverse (AC5).
- [ ] **T5** — Doc-comments et CHANGELOG de la partie a (AC9, dont `bank_accounts.rs:866-867`,
  `routes/bank_accounts.rs:904-907`, `errors.rs:1247-1249` et `routes/bank_imports.rs:57-61`) ; propagation greppée.
- [ ] **T6** — Mutations M29-1 à M29-4, M29-6, M29-7, M29-19 à M29-23 et M29-25 ; table *Position par chemin*
  recopiée (lignes W17, W22, clôture, réouverture, `PATCH`) ; gate complet + E2E complet au dernier commit de code ;
  Dev Agent Record (« aucune release avant la 29-4b », état transitoire des textes) ; corps de PR **demandé** à
  l'orchestrateur : `refs #651` — ⛔ ni `closes` ni `fixes` dans aucun message de commit, même en prose.

### 29-4b

- [x] **T6-bis (étape 0, reports de la passe ciblée P2 de la 29-4a, 2026-10-14)** — avant tout autre geste, reprendre
  les cinq points LOW reportés (`kesh-gate-logs/29-4a-review-p2-ciblee.md`) :
  **R-1** — T29a-15 joue un cycle **synthétique** (transaction de test en X sur `company_invoice_settings`, route en S
  dessus après la sentinelle : aucune route réelle ne tient ce verrou en X puis demande `companies`) ; le dire dans le
  doc-comment du test et au point (vi) d'`audit_route_registry.rs` (« même mécanique de rejeu, la démotion n'est pas
  jouée »), ou ajouter un quatrième test `isPrimary=true` contre une ligne de compte tenue.
  **R-2** — `was_primary` est lu au pré-vol (`bank_accounts.rs`, `archive`) et **réutilisé à chaque tentative** de
  `archive_once` : le relire dans la transaction de chaque tentative (sous le `FOR UPDATE`), avec un test (promotion
  concurrente du compte entre deux tentatives) ; défaut préexistant que le rejeu élargit.
  **R-4** — doc-comments de cycles (`create_bank_account`, point (vi), `ensure_*`) : distinguer le **mesuré** du
  **supposé** — balayage de l'index secondaire `fk_bank_accounts_company` à la démotion du principal (non vérifié par
  EXPLAIN) et ordre des contrôles de clé étrangère d'InnoDB (note N-1 : squash et migrations ne rangent pas les FK dans
  le même ordre) ; écrire « raisonné, non mesuré », ou mesurer.
  **R-5** — mutation du `rollback` de `conclure` sur les trois routes neuves (`conclure` sans `tx.rollback()`), tuée
  par un test qui sonde le verrou (`sonde_verrou_nowait`), ou déclarée non couverte.
  **`docs/api-external.md`** (§ *Interblocages*, l. ~746) — la liste des routes rejouées est trop courte : y ajouter
  `close`, `reopen`, `PATCH`, `PUT`, `DELETE`, `POST /bank-accounts` et `DELETE /bank-imports/{id}`.
- [x] **T7** — Partir d'`origin/main` après la fusion de la 29-4a ; relire les numéros de ligne de l'AC6.
- [x] **T8** — Tests de fenêtre des cinq annulations (T29b-1) **d'abord** (rouges au sol), puis les cinq appels de
  `recheck_open_locked_in_tx` (AC6, AC7) ; les deux tests de l'autre entrelacement (T29b-2) ; l'annulation sur un compte
  archivé (T29b-5) ; doc-comment de W9 (attente sous le verrou nommé).
- [x] **T9** — Registre (T29b-4) : `Reverifiee(Variante::Clos, …)` pour les cinq, `Ouverte` retiré, raison de W30 sans
  réserve, ordre textuel étendu par positions de littéraux (AC8) ; T29b-3 (cycle neuf).
- [x] **T10** — Textes de L1 (doc-comments dont `bank_accounts.rs:28` et `bank_imports.rs:444-445`, registre dont
  l. 89, `routes/reconciliation.rs:4908`, `bank_accounts_closed_e2e.rs:18` et `:1577`), manuel et PDF contrôlé aplati,
  `api-external.md` (l. 241, 555 et 594), commentaires des cinq gardes (AC6), CHANGELOG complété (AC9) ; propagation
  greppée (jeton et formulation).
- [x] **T11** — Mutations M29-8 à M29-18 et M29-24 ; table *Position par chemin* et table des cycles recopiées ; gate
  complet + E2E complet au dernier commit de code ; corps de PR **demandé** : `closes #651` (sur la PR seulement).

## Dev Notes

### La conception — pourquoi une revérification verrouillante, et pourquoi à ces positions

La 28-4 a écarté une garde **verrouillante** pour les cycles qu'elle ouvrait (validations P1 et P2 de la 28-4,
`kesh-gate-logs/28-4-validate-p{1,2}-{R,F}.md`) : posée **tôt** (P0/P1 : avant l'exercice, ou au point de la garde),
elle prenait le compte en exclusif, puis l'écrivain demandait `companies` en partagé (clé étrangère des écritures, des
imports, des lots) — ordre inverse des routes de gestion, non rejouées ; ou l'exercice après le compte, contre
`accept_one_invoice` ; ou les transactions d'un import après le compte, contre la suppression d'import et `unreject`.
Puis, rendue non verrouillante à la même place (P2), elle déplaçait la naissance de la vue (P3, R1 = F1, HIGH).

Ce que la 29-4 change, et qui fait la différence :

1. **Partagé, non exclusif.** La clôture prend l'exclusif ; un écrivain n'a besoin que du partagé pour l'en empêcher.
   Deux écrivains ne se gênent pas entre eux (S/S compatibles) : les cycles W9 / W24, W30 / W24, W22 / W24 de la 28-4
   (qui naissaient de deux demandes **exclusives** du compte) **n'existent pas** ici.
2. **Là où le verrou est déjà tenu, aucun verrou.** L'import et la création de lot tiennent la ligne du compte en
   partagé depuis leur `INSERT` (clé étrangère) — la 28-4 le savait (T-a31) mais lisait `closed_on` **avant**, dans la
   vue. Lire après, sous ce verrou, ferme la fenêtre pour rien (redemande sans effet : raisonnée, P-d (2), établie par
   T29a-6 et T29a-7).
3. **Ailleurs, en dernier.** Les annulations ne tiennent aucun verrou du compte : la revérification en ajoute un, et sa
   **position** décide des cycles. En dernier, le compte vient après `companies` (pas de cycle avec les routes de
   gestion, non rejouées) et après l'exercice (cycle avec W8 / W27, **tous rejoués** — table ci-dessous).
4. **Aucune vue déplacée.** La revérification est une lecture verrouillante (dernier état commité), posée après la
   naissance de la vue de chaque chemin ; les gardes non verrouillantes de la 28-4 ne bougent pas.

Pistes du prompt, départagées (détail : C-29-4-1) :

| Piste | Ferme la fenêtre ? | Cycles | Vue | Coût | Verdict |
|---|---|---|---|---|---|
| (a) un verrou commun pris par la clôture **et** chaque écrivain, dans un ordre global unique | oui | le dépôt **n'a pas** d'ordre global (`journal_entries.rs:1358-1359`, « le dépôt n'a pas d'ordre de verrouillage unique ») ; l'ordre « document → compte → exercice » (P1-F1 de la 28-4) crée le cycle avec les routes de gestion **non rejouées** (`companies` X → compte X contre compte S → `companies` S) | sans effet si verrouillant | 5 enveloppes neuves (`POST`, `PUT`, `DELETE /bank-accounts`, import, lot), cycles à revérifier sur toute la doctrine | **écartée** sous sa forme pure ; retenue dans sa **variante** (c) |
| (b) rejeu sur interblocage des deux côtés | **non**, seul : il n'y a pas de cycle tant qu'il n'y a pas de verrou commun | neutralise ceux que le verrou crée | — | faible par route | **retenue en complément** : la clôture, la réouverture et le `PATCH` du compte (AC4 ; la réouverture, écartée en P1, rétablie en P2 — F3 ; le `PATCH`, ajouté en P2 — F2) ; les annulations et W8 / W27 le sont déjà |
| (c) revérification sous verrou au moment de l'écriture finale | **oui** | W17, W22 : aucun ; annulations : un cycle avec W8 / W27 (rejoués des deux côtés) ; cycles par la file d'attente des demandeurs d'exclusif (clôture, réouverture, `PATCH`, W8, W27 — tous rejoués) | aucun déplacement | 2 fonctions, 7 appels, 3 enveloppes, tests | **retenue** |
| (d1) verrou nommé `GET_LOCK` du compte, pris aussi par la clôture | non : `with_account_lock` relâche le verrou **avant** le commit de l'appelant (`kesh-reconciliation/src/mutex.rs:131-136`, `RELEASE_LOCK` dans la transaction, avant que l'appelant commite) ; et un cycle verrou nommé / verrou de ligne échappe à la détection d'InnoDB (attente jusqu'au délai, `1205`, non rejoué) | — | — | changer la sémantique de toutes les routes de réconciliation | **écartée** |
| (d2) la clôture verrouille les lignes enfants du compte (transactions, règlements, factures fournisseurs) | oui | ensemble de verrous large (index `bank_account_id`, intervalles), ordres de parcours à prouver contre la suppression d'import, l'import, `accept_one_invoice`, `pay_in_tx` — exactement l'analyse que la 28-4 a dû abandonner | — | élevé | **écartée** |
| (d3) refuser la clôture tant qu'une écriture est en cours, ou réduire et documenter | — | — | — | — | **inutile** : (c) ferme la fenêtre ; la sortie n'est pas nécessaire |

### Position par chemin — la preuve du critère (à recopier au Dev Agent Record avec la ligne retenue)

« Vue » = première lecture non verrouillante de la transaction, au sol puis avec la story.

| Chemin | Verrous au sol, dans l'ordre (28-4, table *Position par chemin*, relue sur `16eeed21`) | Ce que la story ajoute | Verrou neuf ? | Position | Vue : au sol → avec la story | Vue déplacée ? |
|---|---|---|---|---|---|---|
| W22 import (CAMT, CSV) | [doublons sans verrou, route] → `INSERT bank_imports` (compte S, `companies` S, `users` S) → `INSERT bank_transactions` → relecture → audit | revérification **avec `archived`** (`recheck_open_unarchived_locked_in_tx`) après `INSERT bank_imports` | **non** (compte S déjà tenu) | après l'`INSERT` d'entête | `find_by_company_and_hash` (route) → inchangée | non |
| W17-W19 `create_batch` | compte sans verrou (0) → factures fournisseur X → `INSERT payment_batches` (compte S, `companies` S) → `UPDATE` → `INSERT` articles → audit | revérification **avec `archived`** après `INSERT payment_batches`, `NotFound` converti | **non** | après l'`INSERT` du lot | étape (0) → inchangée | non |
| W9 `cancel_in_tx` | [verrou nommé, hors InnoDB] → transaction X (1) → règlement ou intervalle de `uq_invoice_settlements_entry`, facture X (2) → écriture et exercice X (3) → [garde 28-4, vue] → `UPDATE bank_transactions` (5) → (6) W25 imbriqué (… `companies` S) ou `reverse_in_tx` (`companies` S) → audit | revérification après (6) | **oui** : compte S | dernière acquisition | garde 28-4 entre (3) et (4) → inchangée | non |
| W25 `cancel_settlement_in_tx` | facture X (1) → règlement X (2) → écriture et exercice X (2-bis) → [garde 28-4] → lettrage → `reverse_owned_in_tx` (`companies` S) → `DELETE` règlement → `UPDATE invoices` → audit | revérification après (6) | **oui** : compte S | dernière acquisition | garde 28-4 après (2-bis) → inchangée | non |
| W28 `supplier_invoices::cancel_in_tx` | facture X → écriture d'achat et exercice X → [garde 28-4] → lettrage → contre-passation (`companies` S) → `UPDATE` → audit | revérification après (4) | **oui** | dernière acquisition | garde 28-4 → inchangée | non |
| W29 `supplier_invoices::cancel_settlement_in_tx` | facture X → (1-bis) écriture de règlement et exercice X → [garde 28-4] → lettrage → contre-passation (`companies` S) → `UPDATE` → audit | revérification après (4) | **oui** | dernière acquisition | garde 28-4 → inchangée | non |
| W24 `delete_unreconciled_in_tx` | entête X → transactions de l'import X et intervalle (`LOCK_IMPORT_TRANSACTIONS_SQL`) → [comptage] → [garde 28-4, vue] → `DELETE` (cascade) → audit (route) | revérification après le `DELETE` | **oui** | dernière acquisition | garde 28-4 → inchangée | non |
| clôture | compte X (première instruction) → décomptes sans verrou → `UPDATE` → audit | enveloppe de rejeu | non | — | premier décompte → inchangée | non |
| réouverture | compte X → `UPDATE` → audit | enveloppe de rejeu (P2, F3) | non | — | inchangée | non |
| W13 `PATCH` | `company_invoice_settings` S (`claims_for_target`, première instruction) → compte X (`bank_accounts.rs:462`) → `accounts` lu sans verrou → `UPDATE` (`accounts` S, clé étrangère) → audit | enveloppe de rejeu (P2, F2) | non | — | lecture non verrouillante d'`accounts`, après le `FOR UPDATE` → inchangée (chaque tentative rejouée a sa propre transaction) | non |
| W30, W1-W6, `accept_one_*`, W20, W8, W27, W14-W15, W21, W26, gestion | inchangés | rien | non | — | inchangée | non |

### Cycles — chaque paire que le verrou neuf peut fermer

Le verrou neuf est un **partagé** sur la ligne du compte `A`, pris **en dernier** par W9, W24, W25, W28, W29. Un cycle à
deux exige un partenaire qui tient `A` en **exclusif** (ou l'y attend) et attend une ressource que l'annulation tient.
Détenteurs d'exclusif sur `A` au sol (extraction de toutes les lectures verrouillantes de `kesh-db/src` et
`kesh-api/src` par table, refaite pour la fiche ; `bank_accounts.rs:334, 462, 607, 691, 931, 1107`,
`invoice_settlements_write.rs:139`, `supplier_invoices.rs:761` — **complète**, recoupée par la lentille F de la P1, qui y
a ajouté les **écritures** de `bank_accounts` sans `FOR UPDATE` : `bank_accounts.rs:63` (ligne neuve), `:380`, `:516`,
`:646`, `:719`, `:954`, `:1121`, `routes/bank_accounts.rs:644`, chacune précédée du `FOR UPDATE` de sa fonction ou portant
sur une ligne neuve). ⚠️ **Méthode** (F4 de la P1) : une extraction par `FOR UPDATE` ne voit pas les exclusifs
**implicites** d'un `UPDATE` / `DELETE` ; toute extraction d'exclusifs, ici et pour `accounts` ci-dessous, porte aussi
sur `(UPDATE|DELETE FROM|INSERT INTO) <table>`. ⚠️ `flip_primary_off_for_company` (`:1107`, `WHERE company_id = ? AND
is_primary = TRUE AND archived = FALSE AND id != ? FOR UPDATE`) peut, sous `REPEATABLE READ`, poser l'exclusif sur
**plusieurs lignes de comptes** de la société s'il balaie l'index de société — sans effet sur la conclusion : il ne
s'exécute que sous la sentinelle `companies` X, donc jamais pendant qu'une annulation tient `companies` S. Le tableau
dit donc « un exclusif sur des lignes de comptes, dont `A` » pour ce détenteur. **S'y ajoute la restauration** (`backup.rs`, `DELETE` des lignes de `bank_accounts` sous `X` ; F9 de la P2), détenteur implicite trouvé par l'extraction `DELETE FROM`.

| Partenaire (exclusif sur `A`) | Ce qu'il demande **après** `A` | L'annulation le tient-elle ? | Cycle | Rejoué des deux côtés |
|---|---|---|---|---|
| W8 `settle_invoice` (virement vers `A`) | [comptes], **exercice** X, `companies` S, `INSERT invoice_settlements` (intervalle de `uq_invoice_settlements_entry`) | **oui** : l'exercice (W9 (3), W25 (2-bis), W28, W29) ; l'intervalle de `uq_invoice_settlements_entry` (W9 « écriture », F13 de la P2 de la 28-4) | **oui** — le cycle F1 de la P1 de la 28-4, et F13 | **oui** : `invoices::settle` ; `reconciliation::cancel`, `invoices::cancel_settlement`, `supplier_invoices::cancel`, `…::cancel_settlement` |
| W27 `pay_in_tx` (par `pay` ou `confirm_batch`) | [comptes X, `supplier_invoices.rs:808`], **exercice** X, `companies` S | oui : l'exercice | **oui** | **oui** : `supplier_invoices::pay`, `payment_batches::confirm` |
| W13 `PATCH` | `accounts` en lecture **non verrouillante** (`ensure_postable_if_active_in_tx`), puis l'`UPDATE` (clé étrangère `journal_account_id` → `accounts` S) | non **à deux** : aucune annulation ne tient `accounts` en exclusif (les seuls exclusifs sur `accounts` sont dans `company_invoice_settings` — réglages, TVA, défauts —, `invoice_settlements_write.rs:181` (règlement, compte interne), `supplier_invoices.rs:433` (création) et `:808` (paiement) ; aucun dans la chaîne de contre-passation, `journal_entries.rs` n'en porte aucun ; exclusifs implicites recherchés aussi — `(UPDATE|DELETE FROM|INSERT INTO) accounts` : seuls `accounts.rs`, routes de gestion des comptes, et `credit_notes.rs:479`, hors chaîne — lentille F de la P1). **À trois**, voir l'angle mort « PATCH » et la file d'attente ci-dessous | non à deux ; **à trois, oui** (F4 de la P1, F2 de la P2) | **rejoué depuis la P2** (C-29-4-19) |
| W14 `PUT`, `POST /bank-accounts`, W15 archivage | `companies` X **avant** `A` ; après `A` : `accounts` S, audit | l'annulation (W9, W25, W28, W29) tient `companies` S **avant** de demander `A` : la route de gestion l'attend sans rien tenir ; W24 ne prend pas `companies`, et la route ne demande rien qu'il tienne | non avec les annulations ; ⚠️ **oui, préexistant, avec l'import et le lot** (revue de code P1, E-1) : entre ses deux contrôles de clé étrangère, l'`INSERT` tient `A` S et demande `companies` S — cycle `I ↔ M` | **rejouées depuis la revue de code P1** (C-29-4-28 ; T29a-13 à T29a-15) |
| `upsert_primary_in_tx` (onboarding) | — | — | non (et `onboarding::finalize` est rejouée) | — |
| clôture | **rien** (décomptes non verrouillants, `UPDATE` de la même ligne, audit sans clé étrangère) | — | non à deux ; **à trois**, par la file d'attente (AC4) | la story la fait rejouer (AC4) |
| réouverture | **rien** | — | non à deux ; **à quatre**, par la file d'attente (F3 de la P2 ; l'argument de la P1, « aucun détenteur partagé réaliste d'un compte clos », était faux) | **rejouée depuis la P2** (C-29-4-14 renversé) |

**Cycles par la file d'attente — tout demandeur d'exclusif sur `A`** *(généralisé en P2, F2 et F3 ; la P1 ne le
raisonnait que pour la clôture)* : un demandeur d'exclusif `E` attend `A` derrière un détenteur partagé `H` ; un
demandeur partagé `R` attend derrière `E` (une demande neuve attend une demande incompatible **en attente** comme un
verrou accordé) ; si `H` attend, directement ou par un tiers, une ressource que `R` tient, le cycle passe par `E`. Les
demandeurs d'exclusif sur `A` (extraction ci-dessus) et leur sort :

| `E` | Peut-il être en file derrière un partagé de la course ? | Rejoué ? |
|---|---|---|
| clôture | oui (AC4, T29a-4) | **oui** (AC4) |
| réouverture | oui, sur un compte clos — le partagé d'un écrivain que la revérification va refuser (W22, W17 après leur `INSERT`, les annulations par leur revérification) ; cycle à quatre de F3 (AC4) | **oui** (P2, F3) |
| `PATCH` (W13) | oui — cycle à trois `D → PATCH → I → D` (AC4, F2) | **oui** (P2, F2) |
| W8, W27 | oui | oui (au sol) |
| `PUT`, `POST`, archivage | ⚠️ **oui** (revue de code P1, E-1 — la P1 de validation disait « non ici », à tort) : ils prennent `companies` X **avant** `A`, mais l'import et le lot, entre les deux contrôles de clé étrangère de leur `INSERT`, tiennent `A` S **sans** tenir encore `companies` S. Cycle préexistant `I ↔ M`, et `I → M → E → I` quand un demandeur d'exclusif `E` (clôture, réouverture, `PATCH`) est en file entre eux — c'est le cycle à trois **atteignable en 29-4a**, celui que T29a-4 et T29a-9 jouent (`t2` = `M`). La revérification de l'import et du lot, redemande d'un partagé tenu, n'y fait jamais la queue (T29a-6, T29a-7) | **oui, depuis la revue de code P1** (C-29-4-28) |
| restauration (`backup.rs`, `DELETE` des lignes de `bank_accounts` ; F9 de la P2) | oui, en théorie : une annulation en revérification derrière ce `X` ferme un cycle avec elle | l'annulation, plus légère que la restauration (qui a écrit massivement), est la victime, **rejouée** sur les cinq routes |

**L'argument de poids, écrit au lieu d'être invoqué** (F11 de la P2, raisonné) : InnoDB désigne pour victime la
transaction de moindre poids, `undo_no` + nombre de verrous (attentes comprises). Un demandeur d'exclusif dont la
demande est la **première instruction** (clôture, réouverture) porte deux verrous (l'intention de table et sa demande
en attente) et aucune écriture ; tout autre participant d'un cycle **hors route de gestion** (`M` n'en tient aucun, réserve (2)) tient au moins un
verrou d'enregistrement, son intention de table et une demande en attente — trois au moins. Il est donc **strictement** le plus léger de tout cycle où
il n'entre qu'avec des transactions qui tiennent un verrou d'enregistrement ; le `PATCH`, qui tient déjà les partagés
des réglages, n'a rien écrit et reste plus léger qu'un import ou une suppression d'import qui ont écrit. ⚠️ **Deux
réserves** *(P3)* : (1) **F3** — le dépôt n'établit la désignation du plus léger que pour des cycles à deux ; qu'InnoDB
compare **tout** le cycle, et non la seule paire qui le ferme, est raisonné, éprouvé pour la première fois par T29a-4
et T29a-9 (seconde contingence, AC4) ; (2) **F2** — l'argument ne vaut pas contre une route de gestion `M` en file sur
la sentinelle `companies` : elle ne tient aucun verrou d'enregistrement (intention de table et demande en attente,
deux), si bien qu'elle est **à égalité** avec la clôture et la réouverture, et **strictement plus légère** que le
`PATCH` (qui tient les partagés des réglages), W8 et W27. Cet argument est recopié au doc-comment des deux fonctions de
revérification (AC1) **avec ces deux réserves** — non sous la forme « strictement le plus léger de tout cycle ».

**Résidus** :

- (C-29-4-7, réécrit en P1, F8 b ; **étendu en P2**, F2) le cycle à trois où l'import (W22, `H` : partagé depuis son
  `INSERT`, en attente de l'intervalle d'`idx_bank_transactions_import` tenu par une suppression d'import W24 = `R`)
  serait désigné victime — que le demandeur d'exclusif soit la clôture ou le `PATCH`. **Théorique** : l'import et la
  suppression ont écrit, le demandeur d'exclusif non ; il est la victime, rejouée — sous la réserve F3 de la P3 (choix
  de la victime sur un cycle entier, raisonné ; seconde contingence de T29a-4). Écrit au doc-comment de la
  revérification, sans « à surveiller » ; aucune issue.
- (P2, F3 ; **réécrit en P3**, F2) **le cycle à quatre** `Q → E → I → M → Q` : une annulation `Q` qui a contre-passé
  (`companies` S) demande `A` en partagé derrière un demandeur d'exclusif `E` en file ; `E` attend `A` derrière un
  import `I` (ou un lot) qui tient `A` S par son `INSERT` et demande `companies` S entre ses deux contrôles de clé
  étrangère ; `I` attend derrière une route de gestion `M` (`PUT`, `POST` ou `DELETE /bank-accounts`) en file sur la
  sentinelle `companies` X, que `Q` tient en partagé. Le cycle **ne dépend pas de ce qu'est `E`** : il se forme avec
  la clôture, la réouverture, le `PATCH`, W8 ou W27. `M` n'y tient que son intention de table et sa demande en
  attente : **à égalité de poids** avec la clôture et la réouverture (raisonné ; InnoDB peut désigner l'une ou
  l'autre) ; **strictement plus légère**, donc **victime désignée**, quand `E` est le `PATCH` (qui tient les partagés
  des réglages), W8 ou W27. Désignée, `M`, non rejouée, rend `500`, rien d'écrit. Quatre gestes concurrents sur un
  même compte et une même société, `I` dans l'intervalle de ses deux contrôles de clé étrangère : **négligeable**,
  écrit au doc-comment de la revérification, sans issue. Rejouer les routes de gestion fermerait ce résidu, au prix de
  trois enveloppes de plus pour un cas que rien n'a montré : écarté (C-29-4-14, annotations de la P2 et de la P3).
  ⛔ **Fermé en revue de code P1 de la 29-4a** (E-1, décision de l'orchestrateur, C-29-4-28) : les routes de gestion
  sont rejouées, non pour ce cycle à quatre mais pour le cycle préexistant `I ↔ M`, atteignable dès la 29-4a, dont
  `M` était la victime en `500` ; le résidu tombe avec lui.

**Angle mort « PATCH », cycle à trois par `accounts`** (F4 de la P1) — ⛔ **fermé en P2** par le rejeu du `PATCH`
(C-29-4-19) : une annulation par virement (W25) tient l'exercice ; un règlement interne W8 tient un exclusif sur un
compte de caisse `Z'` et attend l'exercice ; un `PATCH` du compte `A` vers `Z'` tient `A` en exclusif et demande `Z'` en
partagé derrière W8 ; l'annulation, en dernière acquisition, demande `A` : cycle annulation → `PATCH` → W8 →
annulation. Ses trois participants sont désormais rejoués ; la phrase « aucun partenaire … sans être rejoué » de l'AC6
vaut à deux **et** à trois, aux résidus ci-dessus près.

**Ce qui ne revient pas, de la 28-4** : le cycle avec les routes de gestion (P2 R1 / F5), le cycle W30 / W24 (P2 F1,
HIGH), le cycle W22 / W24 par l'intervalle (P1 R2 / F4), le cycle `accept_one_invoice` (P2 F2 : partagé contre partagé)
— tous naissaient d'un exclusif pris par l'écrivain, ou d'un verrou pris **avant** `companies`.

### État actuel des fichiers modifiés — ce qui doit être préservé

- `bank_accounts.rs` : `ensure_open_in_tx` et `ensure_open_unarchived_in_tx` **inchangés dans leur SQL** (T-a45) ; seuls
  leurs doc-comments changent (AC9). `close_in_tx` / `reopen_in_tx` inchangés dans leur séquence.
- `bank_imports.rs` : l'ordre « entête → transactions » de l'import, et celui de la suppression (« aucune vue n'est
  ouverte avant les verrous », l. 389-391, 420-425) ; la garde W24 de la 28-4 après le comptage (C-28-4-24).
- `reconciliation_cancel.rs`, `invoice_settlements_write.rs`, `supplier_invoices.rs` : les règles « aucune lecture
  non verrouillante avant le verrou d'exercice » (l. 244-255 ; `:1040-1046` ; `:855-861` ; `:1290-1294`) — la
  revérification est verrouillante et vient après.
- `payment_batches.rs` : l'étape (0) et ses refus (`IllegalStateTransition` pour l'archivé, `BankAccountClosed`).
- `routes/bank_accounts.rs` : l'ordre des refus de la clôture et de la réouverture (28-4, AC3/AC4), la borne de
  `closedOn` avant toute transaction.
- `bank_account_closed_inventory.rs` : ses deux inventaires (écrivains, appelants de `invoice_settlements::create_in_tx`)
  et leurs **27 + 3 entrées** (31 sites d'écriture vus pour les 27 entrées du `REGISTRE`, `:491`), inchangés.
  *(Recompté en P1, R7 : la fiche écrivait « 31 + 3 entrées », confondant sites et entrées.)*
- `pool.rs` : **non touché** (AC11 retiré en P2 ; `create_pool` sans appelant, #669).
- `routes/bank_accounts.rs` : les contrôles hors transaction des trois routes rejouées (version, borne de `closedOn`,
  `assert_onboarding_complete`, `validate_journal_account_id`) restent **devant** leur enveloppe ; l'ordre des refus du
  `PATCH` (doc-comment l. 1130-1141) est inchangé.

### Fichiers communs avec les autres stories de l'epic

| Fichier | Stories |
|---|---|
| `CHANGELOG.md` `[0.13.4]` | 29-2, 29-3 (union) |
| `docs/manual/fr/user-manual.tex` / `.pdf` | 29-3 (« la meilleure »), 29-2 si texte |
| `docs/api-external.md` | 29-2, 29-3 possibles |
| `crates/kesh-api/src/routes/reconciliation.rs` | 29-2, 29-3 — la 29-4 n'y touche **qu'un doc-comment** (l. 4908, en 29-4b ; celui des l. 2166-2170 n'est plus touché depuis le retrait de l'AC11) |
| `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` | — (aucune autre story ne le prévoit) |
| `sprint-status.yaml`, `epic-29-choix-autonomes*.md` | toutes (union) |

La 29-1 ne touche ni la réconciliation ni les comptes bancaires (fichier d'epic). Aucune migration n'est prévue
(prémisse : la revérification lit une colonne existante) — ⚠️ si une autre story de l'epic en ajoute une, P6 se rejoue.

### Tests (récapitulatif)

*Identifiants renumérotés en P1 (R7)* : `T-a1` à `T-a5` existent déjà dans le module `compte_clos`
(`bank_accounts_repository.rs:1947`, `:1978`, tests de la 28-4), `T-b1` à `T-b4` côté frontend
(`bank-accounts.api.test.ts:199`), et `M1` à `M15` recouvrent les mutations de la 28-4 que citent des doc-comments
(T-a31 cite « M6 ») : les tests de la 29-4 sont **`T29a-n` / `T29b-n`**, ses mutations **`M29-n`**. `T-a31`, `T-a45`,
`T-a48`, `T-a49` désignent toujours les tests de la 28-4.

**Partie a** : T29a-1 `EXPLAIN` des deux revérifications (PRIMARY, `const`) ; T-a45 (28-4) étendu (verrou et `archived`) ;
T29a-2 fenêtre W22 ; T29a-3 fenêtre W17 ; T29a-4 cycle à trois par la clôture ; T29a-5 registre (troisième inventaire,
deux sens, ensemble `G`, variante appelée, ordre textuel) et son volet inverse par un cas construit ; T29a-6 et T29a-7
l'autre ordre (écrivain inséré, clôture **en file** établie par `sonde_verrou_nowait` en partagé — P3, R1 = F1 —, revérification sans attente —
prémisse P-d (2)) ; ~~T29a-8~~ *retiré en P2* (AC11, #669) ; T29a-9 cycle à trois par la réouverture ; T29a-10 le
`PATCH` victime, rejoué ; T29a-11 et T29a-12 la course avec l'archivage (W22, W17). Soit **onze** tests neufs de la
partie a (T29a-1 à T29a-12, moins T29a-8), plus T-a45 étendu.

**Partie b** : T29b-1 fenêtres W9, W24 (import vide), W25, W28, W29 (cinq) ; T29b-2 autre entrelacement W9 (412) et W25
(`Done`) ; T29b-3 cycle neuf annulation / exclusif du compte, rejoué ; T29b-4 registre (`Ouverte` retiré,
`Variante::Clos` pour les cinq, ordre textuel des annulations) ; T29b-5 annulation d'un règlement sur un compte
archivé (W25). Restent verts **sans retouche** : T-a31, T-a48 (a)-(c),
`a_concurrent_close_waits_for_the_unreconciliation` (clôture d'exercice, refuse avant la revérification),
`annuler_un_rejet_pendant_une_cloture_refuse_par_w30`, le reste de `bank_accounts_closed_e2e.rs`,
`rejeu_interblocage_e2e.rs` — dont `settlement_cancellation_is_replayed_when_it_is_the_deadlock_victim` (l. 627) et
`accept_and_settlement_cancel_interleave` (l. 1545) : **relus en P1** par les deux lentilles, leurs règlements annulés
sont en `internal_account`, la revérification ne s'y exécute pas (la branche `if let Some(bank_account_id)` n'est pas
prise). Un rouge s'y diagnostiquerait quand même avant toute retouche.

### Mutations

| # | Mutation | Tuée par |
|---|---|---|
| M29-1 | W22 : appel de revérification retiré | T29a-2 |
| M29-2 | W17 : appel retiré | T29a-3 |
| M29-3 | `LOCK IN SHARE MODE` retiré de `RECHECK_OPEN_LOCKED_SQL` | T-a45 (28-4, étendu) ; et T29a-2, T29a-3 (la lecture redevient celle de la vue) |
| M29-4 | enveloppe de la clôture retirée | T29a-4 |
| ~~M29-5~~ | *retirée en P2* avec l'AC11 (elle visait l'épinglage de `configurer_session` ; #669). Le numéro n'est pas réattribué | — |
| M29-6 | W22 : appel déplacé avant l'`INSERT INTO bank_imports` | T29a-5 (ordre textuel) — fonctionnellement équivalente, dite telle |
| M29-7 | W17 : appel déplacé avant l'`INSERT INTO payment_batches` | T29a-5 — idem |
| M29-8 à M29-12 | W9, W24, W25, W28, W29 : appel retiré | T29b-1 (un test chacune) |
| M29-13 | revérification non verrouillante (M29-3 jouée en 29-4b) | T29b-2, moitié W9 (la moitié W25 ne la tue pas, R5 de la P2) ; M29-8 à M29-12 |
| M29-14 | W25 : appel déplacé juste après (2-bis) | T29b-4 (ordre textuel) — fonctionnellement équivalente, dite telle |
| M29-15 | W9 : appel déplacé avant l'étape (6) | T29b-4 |
| M29-16 | W24 : appel déplacé avant le `DELETE FROM bank_imports` (ajoutée en P1, R2) | T29b-4 (position du littéral) |
| M29-17 | W28 : appel déplacé entre `reverse_owned_in_tx(` et l'`UPDATE supplier_invoices` (ajoutée en P1, R2) | T29b-4 (position du littéral) |
| M29-18 | W29 : même déplacement (ajoutée en P1, R2) | T29b-4 |
| M29-19 | une revérification en `FOR UPDATE` au lieu de `LOCK IN SHARE MODE` (ajoutée en P1, R5 = F2) | T-a45 (28-4, étendu, les deux constantes) ; T29a-6 / T29a-7 pour la variante (cycle écrivain / clôture) |
| M29-20 | enveloppe de la réouverture retirée (ajoutée en P2, F3) | T29a-9 |
| M29-21 | enveloppe du `PATCH` retirée (ajoutée en P2, F2) | T29a-10 |
| M29-22 | `archived` retiré de la variante — du SQL, ou du `match` qui rend `NotFound` (ajoutée en P2, R2 = F4) | T29a-11, T29a-12 ; T-a45 (forme SQL) |
| M29-23 | W22 ou W17 appelle `recheck_open_locked_in_tx` au lieu de la variante (ajoutée en P2) | T29a-11 / T29a-12 ; T29a-5 (assertion (i), variante appelée) |
| M29-24 | W25 appelle `recheck_open_unarchived_locked_in_tx` (une annulation qui refuserait un archivé ; ajoutée en P2) | T29b-5 ; T29b-4 (assertions (i) et (i bis)) |
| M29-25 | W17 : conversion `NotFound` → `IllegalStateTransition` retirée (ajoutée en P2) | T29a-12 |

### Angles morts assumés (à écrire aux doc-comments)

- Le cycle à trois où l'import serait désigné victime (C-29-4-7) — **théorique** depuis la P1 : le demandeur
  d'exclusif (clôture, ou `PATCH` depuis la P2) y est toujours plus léger (raisonné : réserve F3 de la P3).
- Le cycle à quatre de F3 (P2, réécrit en P3, F2) où une route de gestion du compte, non rejouée, serait victime —
  à égalité de poids avec la clôture ou la réouverture, victime désignée quand le demandeur d'exclusif traversé est le
  `PATCH`, W8 ou W27 — négligeable (quatre gestes concurrents).
- ~~Le cycle à trois par `accounts` où le `PATCH`, non rejoué, serait victime (F4 de la P1)~~ — **fermé en P2** : le
  `PATCH` est rejoué (C-29-4-19).
- L'interblocage **préexistant** import ou lot / route de gestion (l'`INSERT` prend le compte en partagé avant
  `companies`, la route de gestion `companies` en exclusif avant le compte ; F5 de la P2) : ni l'un ni l'autre rejoués,
  `500` possible, rien ne passe sur un archivé ; au sol, non aggravé.
- L'ordre textuel du registre ne suit pas un geste verrouillant appelé sous un nom absent de sa liste de jetons, ni un
  SQL assemblé hors littéral (AC8) ; l'ensemble `G` ne voit pas une lecture de `closed_on` dont le SQL ne porte pas
  littéralement `FROM bank_accounts`, ni les pré-vols de route sur le pool (AC5).
- La question F11 de la 28-4 (compte du règlement ≠ compte de la transaction dans W9) reste non établie ; la position
  « dernière acquisition » la rend sans effet sur les cycles (AC6).
- ~~La revérification ne lit pas `archived`~~ — **fermé en P2** : la variante de l'import et du lot lit `archived`
  (AC1, AC2, C-29-4-16 renversé) ; aucune issue L5 à ouvrir.
- **`innodb_snapshot_isolation` non épinglé** (AC11 retiré en P2, R1 = F1) : sous MariaDB 11.6+ (`ON` par défaut), les
  deux revérifications rendraient `1020` → `500` dans la course qu'elles traitent — rien d'écrit, mais un refus
  indiscernable d'une panne ; du même ordre que `accept_one_invoice` (`routes/reconciliation.rs:2166-2170`, qui rendrait
  `DATABASE_ERROR` dans `failed[]`) et que le raisonnement de `bank_imports.rs:439-446` (R6 de la P3). Renvoyé à
  **#669** (réglages de session du pool réel ; les points sur 11.6+ du manuel d'administration, de
  `docker-compose.prod.yml` et du NAS y sont versés) ; la CI et la base de dev tournent sous MariaDB 10.11 (`OFF`).
- Les états laissés par la fenêtre de la v0.13.3 ne sont pas réparés (table *Dehors*).

### Pièges connus du dépôt, applicables ici

- `LOCK IN SHARE MODE`, pas `FOR SHARE` (MariaDB 10.11).
- `attendre_une_requete_en_cours` voit une requête **en cours**, non une requête **bloquée** (leçon de la validation P5
  de la 28-4, T-a49) : chaque test d'entrelacement doit établir l'attente par son **issue** (le résultat après le commit
  de `C`), pas par la seule sonde — ou par une sonde de file de verrou quand l'issue ne départage rien (T29a-6,
  T29a-7 ; R4 = F7 de la P2) : `sonde_verrou_nowait` en `LOCK IN SHARE MODE NOWAIT` sur la ligne tenue en partagé, qui
  échoue en `1205` dès qu'une demande exclusive est en file (AC3, constaté le 2026-10-14). ⛔ **Pas
  d'`information_schema.INNODB_TRX`** : elle exige le privilège `PROCESS`, que les droits versionnés de dev ne donnent
  pas (`scripts/mariadb-init/01-dev-grants.sql` ; `reconciliation_e2e.rs:7546-7553`) — R1 = F1 de la P3. ⚠️ Le conteneur
  de dev de la station porte, le 2026-10-14, un `GRANT ALL PRIVILEGES ON *.*` à `kesh` qui n'est pas celui du fichier
  versionné : une sonde à privilège y passerait, et rougirait sur un conteneur rejoué depuis ce fichier. Ne pas s'y fier.
- Tests de rejeu : alourdir la transaction de test (500 lignes de lest), exiger qu'elle obtienne son verrou (`.expect`),
  capturer le `warn!` de rejeu (`rejeu_interblocage_e2e.rs:5-23`).
- Mutations : `git checkout -- <fichier>` **puis `touch`** ; base de gate remise à zéro avant tout gate complet.
- Registre de sprint et registre des choix : par **union**.
- Données réelles : fixtures synthétiques (IBAN de test) ; rien du relevé de Guy.

### Project Structure Notes

Aucun fichier neuf de production ; tests ajoutés aux fichiers existants (`bank_accounts_repository.rs`,
`invoice_settlement.rs`, `supplier_invoices_repository.rs`, `reconciliation_cancel.rs` côté `kesh-db/tests`,
`rejeu_interblocage_e2e.rs`, `bank_account_closed_inventory.rs` côté `kesh-api/tests`) — choix laissé au dev pour la
répartition des tests de fenêtre, à écrire au Dev Agent Record.

### References

- Issue [#651](https://github.com/guycorbaz/kesh/issues/651) ; fiche `28-4-cloture-compte-bancaire.md` (AC6, AC7,
  § *Concurrence*, *Position par chemin*, L1) ; registre `epic-28-choix-autonomes.md`, C-28-4-14, -16, -17, -21, -24.
- Rapports de validation de la 28-4 : `/home/gcorbaz/devel/kesh-gate-logs/28-4-validate-p1-F.md` (F1, F4),
  `…-p1-R.md` (R2), `…-p2-R.md` (R1), `…-p2-F.md` (F1, F2, F5, F12, F13, table de l'axe obligatoire), `…-p3-R.md` (R1),
  `…-p5-F.md` (F1).
- Rétrospective de l'Epic 28, § 4.2 et § 4.3 (`epic-28-retro-2026-10-14.md`).
- Code : `crates/kesh-db/src/repositories/bank_accounts.rs:746-1093` ; `bank_imports.rs:67-180, 394-534` ;
  `payment_batches.rs:69-230` ; `reconciliation_cancel.rs:240-418` ; `invoice_settlements_write.rs:826-1000` ;
  `supplier_invoices.rs:1061-1190, 1292-1430` ; `crates/kesh-api/src/routes/bank_accounts.rs:1005-1140` ;
  `routes/bank_imports.rs:1461-1550` ; `crates/kesh-api/src/retry.rs` ; `crates/kesh-reconciliation/src/mutex.rs` ;
  `crates/kesh-db/src/repositories/journal_entries.rs:1344-1380` (ordre des verrous et cycles hérités) ;
  `crates/kesh-db/src/pool.rs:28-49` (`create_pool`, sans appelant) et `crates/kesh-api/src/main.rs:74` (le pool réel) —
  issue **#669** ; `crates/kesh-db/src/errors.rs:1459-1494` (`map_db_error`) ;
  `crates/kesh-api/src/errors.rs:4114-4120` (rendu de `DbError::Sqlx`) ; `crates/kesh-api/tests/common/capture_rejeu.rs`.
- Rapports de validation de la 29-4 : `/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p1-R.md` (R1-R13),
  `…-p1-F.md` (F1-F9, table de l'axe obligatoire), `…-p2-R.md` (R1-R10), `…-p2-F.md` (F1-F12, table des verrous et des
  cycles), `…-p3-R.md` (R1-R12), `…-p3-F.md` (F1-F7, table des verrous et des cycles).

## Dev Agent Record

*Partie a seulement (`29-4a-reverification-import-lot`), 2026-10-14. La partie b reste à développer.*

### Agent Model Used

Opus 5.5 (`bmad-dev-story`, autonomie), worktree `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4` sur `089fcd80`
(Stories 29-2 et 29-3 mergées). Commit de code : `e718221b`.

### Debug Log References

Journaux sous `/home/gcorbaz/devel/kesh-gate-logs/` : `294-rouge-sol-T2.log` (rouge au sol), `294-gate-1.log` (gate
interrompu par `clippy`), `294-gate-2.log` (gate complet vert), `294-mutations.log` et `294-M29-*.log` (campagne),
`294-front-{check,lint,unit,build}.log`, `294-e2e-1.log` et `294-e2e-backend.log` (E2E complet).

### Completion Notes List

**T0 — établi au code, avant la conception des tests.** Rebase vérifié (`git merge-base --is-ancestor 089fcd80 HEAD`).
Bases `kesh_294` et `kesh_e2e_294` recréées depuis les migrations du worktree, seed `scripts/seed-dev-db.sql` ; serveur
`10.11.16-MariaDB`, `@@innodb_snapshot_isolation = 0`. **La sonde `LOCK IN SHARE MODE NOWAIT` se comporte comme la
fiche le prévoit** — rejoué sur `kesh_294`, deux tables jetables `parent_294` / `enfant_294` (clé étrangère), client
`mariadb` sous `kesh` : sonde **réussie** quand une transaction A ne tient le parent qu'en partagé **par le contrôle de
clé étrangère d'un `INSERT`** ; **échec immédiat `ERROR 1205`** dès qu'une transaction C demande l'exclusif du parent
en file ; réussie de nouveau après les commits. Le même essai **a établi le cas que la P3 n'avait pas essayé** : A,
détenteur du partagé de clé étrangère, a redemandé `SELECT … LOCK IN SHARE MODE` pendant que C attendait — réponse en
**0,9 ms**, puis un second `INSERT` enfant en 0,5 ms, sans attente ni interblocage ; C n'a obtenu la ligne qu'au commit
de A, et y a vu les deux lignes *(annoté le 2026-10-14, revue de code P1, A-7 : ces mesures au client `mariadb` ne sont
pas journalisées)*. Aucun repli n'a été nécessaire. Tables supprimées ensuite. **Aucune** lecture
d'`information_schema.INNODB_TRX` ni du privilège `PROCESS`.

**Tests rouges d'abord** (`294-rouge-sol-T2.log`) — T29a-2, T29a-3, T29a-11, T29a-12 écrits avant le code, contre le
sol ; les quatre rougissent **à l'étape (5)**, après avoir été vus en attente sur leur `INSERT` (étape (3) passée) :

```
⛔ l'import commencé avant la clôture devait être refusé — reçu Ok(())
⛔ l'import commencé avant l'archivage devait être refusé — reçu Ok(())
⛔ le lot commencé avant l'archivage devait être refusé — reçu Ok(true)
⛔ le lot commencé avant la clôture devait être refusé — reçu Ok(true)
Summary [   0.192s] 4 tests run: 0 passed, 4 failed, 59 skipped
```

Après les appels : `bank_accounts_repository` et `payment_batches_repository`, **63/63** *(annoté le 2026-10-14,
revue de code P1, A-7 : run non journalisé ; le gate complet `294-gate-2.log` le couvre)*. T29a-6, T29a-7 (l'autre
ordre), T29a-1 (`EXPLAIN`) et T-a45 étendu ont été écrits **après** les fonctions (ils les appellent) et sont verts du
premier coup.

**Prémisse P-d (2) — établie au dépôt** par T29a-6 et T29a-7 : sonde de référence réussie (seul l'écrivain tient le
compte), sonde de file échouée une fois la clôture lancée (demande exclusive en file), revérification rendue `Ok(())`
sous `timeout` de 2 s, `INSERT` de la transaction `pending` sans attente, puis la clôture rend
`Refused(PendingTransactions { pending_count: 1 })` (resp. `OpenPaymentBatches { batch_count: 1 }`) — **sans**
interblocage. « Aucun verrou neuf » tient pour W22 et W17.

**Contingences de T29a-4 et T29a-9 : aucune ne s'est produite.** Les deux tests, premiers du dépôt à monter un cycle
**à trois** par la file d'attente, sont verts : la victime observée est la clôture (resp. la réouverture) — événement
`bank_accounts::close` (resp. `::reopen`) capturé, `t2` et `t1` ont obtenu leurs verrous (`.expect`). La réserve (1)
de l'argument de poids reçoit donc un premier cas favorable ; elle reste écrite comme raisonnée au-delà de ce cas.
T29a-10 (`PATCH`, cycle à deux) vert, événement `bank_accounts::patch`.

**Table *Position par chemin*, avec les lignes retenues** (sur `e718221b`) :

| Chemin | Garde de tête (vue) | `INSERT` à clé étrangère | Revérification | Suite | Verrou neuf ? | Vue déplacée ? |
|---|---|---|---|---|---|---|
| W22 import (CAMT, CSV) | `bank_imports.rs:85` (`ensure_open_unarchived_in_tx`) ; vue née avant, dans la route (`find_by_company_and_hash`) | `bank_imports.rs:90` (`INSERT INTO bank_imports`) | `bank_imports.rs:131`, `recheck_open_unarchived_locked_in_tx` | `bank_imports.rs:151` (`INSERT INTO bank_transactions`) | non (compte S déjà tenu) | non |
| W17 `create_batch` | `payment_batches.rs:92` (étape (0)) | `payment_batches.rs:146` (`INSERT INTO payment_batches`) | `payment_batches.rs:167`, `recheck_open_unarchived_locked_in_tx`, `NotFound` → `IllegalStateTransition` | `payment_batches.rs:184` (`UPDATE … msg_id`), articles, audit | non | non |
| clôture | — | — | enveloppe `routes/bank_accounts.rs:1040` → `close_once` (`:1055`), `conclure` (`:1110`) | — | non | non |
| réouverture | — | — | enveloppe `:1157` → `reopen_once` (`:1167`) | — | non | non |
| `PATCH` (W13) | — | — | enveloppe `:1255` → `patch_once` (`:1267`) ; `claims_for_target` reste première instruction de la tentative | — | non | non |

SQL et fonctions : `bank_accounts.rs:907` (`RECHECK_OPEN_LOCKED_SQL`), `:913` (`RECHECK_OPEN_UNARCHIVED_LOCKED_SQL`),
`:999`, `:1038`.

**Gates réellement exécutés, au commit de code `e718221b`** (le commit qui suit ne porte que la fiche, le registre de
sprint et le prompt de revue) :

- `scripts/test-fast.sh` (fmt + clippy workspace + nextest), sous verrou, base `kesh_294` remise à zéro avant *(annoté
  le 2026-10-14, revue de code P1, A-7 : la remise à zéro n'est pas journalisée)* :
  **3637 passés, 6 ignorés** (`294-gate-2.log`, 155 s). Le premier essai (`294-gate-1.log`) a échoué sur `clippy`
  (`enum_variant_names` : `Fenetre::SansFenetre`), corrigé avant le commit — écart à la fiche ci-dessous.
- Frontend, sous verrou : `npm run check` 0 erreur (29 avertissements préexistants), `lint-i18n-ownership` vert,
  `test:unit` **1617/1617** (146 fichiers), `build` vert. Frontend non touché par la story.
- E2E complet, sous verrou, recette de `docs/testing.md` (port 3045, base `kesh_e2e_294` recréée, `KESH_TEST_MODE`
  des deux côtés) : **282 passés, 7 échecs, 19 ignorés** (11,4 min). Les sept, jugés fichier par fichier contre
  `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
  `onboarding.spec.ts:57`, `:77`, `:150` — **les sept KF-029 (#97)** ; aucun autre.

**Mutations** (`294-mutations.log`, motif vérifié unique, restauration `git checkout --` puis `touch`, arbre vérifié
propre après la campagne) — **15 jouées, 15 tuées, aucune équivalente** :

| # | Mutation | Tuée par |
|---|---|---|
| M29-1 | W22 : appel retiré | T29a-2, T29a-11, T29a-5 |
| M29-2 | W17 : appel (et conversion) retirés | T29a-3, T29a-12, T29a-5 |
| M29-3 | `LOCK IN SHARE MODE` retiré de `RECHECK_OPEN_LOCKED_SQL` | T-a45 étendu (seul consommateur en 29-4a ; T29a-2 / T29a-3 appellent la variante — voir M29-3 bis) |
| M29-3 bis | idem sur `RECHECK_OPEN_UNARCHIVED_LOCKED_SQL` (ajoutée) | T-a45 étendu, T29a-2, T29a-3, T29a-11, T29a-12 |
| M29-4 | enveloppe de la clôture retirée | T29a-4 |
| M29-6 | W22 : appel déplacé avant l'`INSERT INTO bank_imports` | T29a-5 (ordre textuel) ; et T29a-2 / T29a-11 — ⚠️ **non** parce que le comportement diffère, mais parce que leur sonde attend la requête `INSERT INTO bank_imports` et voit l'écrivain bloqué sur la revérification : fonctionnellement, le refus aurait lieu (équivalence de la fiche confirmée), la tuerie de ces deux-là dépend du montage |
| M29-7 | W17 : appel déplacé avant l'`INSERT INTO payment_batches` | T29a-5 ; T29a-3 / T29a-12 par la même dépendance de montage que M29-6 |
| M29-19 | la variante en `FOR UPDATE` | T-a45 étendu, T29a-6, T29a-7 (le cycle écrivain / clôture) |
| M29-19 bis | `RECHECK_OPEN_LOCKED_SQL` en `FOR UPDATE` (ajoutée) | T-a45 étendu |
| M29-20 | enveloppe de la réouverture retirée | T29a-9 |
| M29-21 | enveloppe du `PATCH` retirée | T29a-10 |
| M29-22 | `archived` ignoré par le `match` de la variante | T29a-11, T29a-12 (la forme SQL est tenue par T-a45 étendu) |
| M29-23 | W22 appelle `recheck_open_locked_in_tx` | T29a-11, T29a-5 |
| M29-23 bis | W17 appelle `recheck_open_locked_in_tx` (ajoutée) | T29a-12, T29a-5 |
| M29-25 | W17 : conversion retirée | T29a-12 |

**État transitoire assumé, à tenir jusqu'à la 29-4b** (AC9) : la puce du manuel utilisateur (`user-manual.tex`, § *Clore
un compte bancaire*, « une opération déjà en cours … peut encore aboutir ») et `docs/api-external.md:241` restent écrits
pour les sept chemins, donc **trop larges** pour l'import et le lot ; ni le manuel ni son PDF ne sont touchés par la
partie a. **Aucune release entre la PR de la 29-4a et celle de la 29-4b** : la v0.13.4 attend la 29-4b.

**Propagation** (AC9) : `grep -rnE "651|\bL1\b"` et `grep -rnE "par une course|hors de cette fenêtre"` sur `crates docs
CHANGELOG.md frontend/src website README.md`. Sites triés : réécrits en 29-4a (`bank_imports.rs` tête de
`create_with_transactions` et son doc-comment, `payment_batches.rs` étape (0), § L1 d'`ensure_open_in_tx` — « Huit
chemins » refait depuis la liste : **six** —, doc-comment d'`ensure_open_unarchived_in_tx`, `errors.rs`
(`BankAccountClosed`), `routes/bank_imports.rs` (`import_create_error`), commentaire de la sentinelle
`routes/bank_accounts.rs`) ; **laissés pour la 29-4b**, comme la fiche le prescrit : `bank_accounts.rs:28` et le renvoi
« angle mort L1 » du doc-comment d'`ensure_open_unarchived_in_tx`, le registre l. 95 / 121 / 239 / 296,
`routes/reconciliation.rs:5021` (ex-`:4908`), `bank_accounts_closed_e2e.rs:18` et `:1577`, `user-manual.tex:1750-1751`,
`docs/api-external.md:241`, `:557`, `:596` ; **laissé, relevé daté** : `CHANGELOG.md:48` (§ 0.13.3). Les autres
occurrences de `L1` désignent d'autres limitations (sans rapport).

**Écarts à la fiche** (chacun au registre — *annoté le 2026-10-14, revue de code P1, A-5 : le renommage `Fermee` et les
trois mutations ajoutées n'y étaient pas ; versés en C-29-4-30*) : W17 testé dans `payment_batches_repository.rs` (C-29-4-23) ; position
structurée `Position { apres, avant }` (C-29-4-24) ; `Fenetre::SansFenetre` renommée **`Fenetre::Fermee`** (refus
`clippy::enum_variant_names`, noté au doc-comment de la variante) ; `conclure` partagée, rendu `Internal` de `begin` /
`commit` inchangé (C-29-4-25) ; rendez-vous sur la demande de `t2` dans T29a-4 / T29a-9 (C-29-4-26) ; tâche d'import
qui commite sur succès (C-29-4-27) ; trois mutations ajoutées (M29-3 bis, M29-19 bis, M29-23 bis). Points (ii) et
(iii bis) d'`audit_route_registry.rs` : leurs « sept » de la famille `DbError` inchangés, une phrase ajoutée pour la
famille `AppError` ; point (vi) recompté sur la branche rebasée : sept au sol (`onboarding::finalize`,
`update_invoice_settings`, `create_fiscal_year`, `close_fiscal_year`, `onboarding::seed_demo`, `onboarding::reset`,
`bank_imports::delete`) + trois = **dix**.

**Décomptes** (recomptés, périmètre `089fcd80..e718221b`) : tests neufs **13** — `kesh-db` 8 (T29a-1, -2, -6, -7, -11
au module `compte_clos` ; T29a-3, -12 dans `payment_batches_repository.rs` ; T-a45 étendu en `mod tests`, test neuf
`les_reverifications_verrouillent_en_partage`), `kesh-api` 5 (T29a-4, -9, -10 ; T29a-5 et son volet inverse) — soit
les onze identifiants T29a de la fiche plus le volet inverse de T29a-5 et T-a45 étendu ; choix ajoutés **5**
(C-29-4-23 à C-29-4-27 ; registre à 27) ; fichiers de production touchés **6** (quatre de code : `bank_accounts.rs`,
`bank_imports.rs`, `payment_batches.rs`, `routes/bank_accounts.rs` ; deux de doc-comments seuls : `errors.rs`,
`routes/bank_imports.rs`), conforme à la fiche.

**Demandé à l'orchestrateur** : la revue de code P1 (`29-4a-reverification-import-lot-review-prompt-p1.md`, Opus,
lentilles B/E/A) ; puis le push et la PR, corps **`refs #651`** — ⛔ ni `closes` ni `fixes`, la 29-4b fermera #651.

### File List

- `crates/kesh-db/src/repositories/bank_accounts.rs` — deux SQL, deux fonctions de revérification, doc-comments (L1,
  `ensure_open_unarchived_in_tx`, `close_in_tx`, `reopen_in_tx`, `set_journal_account_id_for_company`), T-a45 étendu
- `crates/kesh-db/src/repositories/bank_imports.rs` — appel W22, commentaires
- `crates/kesh-db/src/repositories/payment_batches.rs` — appel W17, conversion de l'archivé, commentaire
- `crates/kesh-db/src/errors.rs` — doc-comment de `BankAccountClosed`
- `crates/kesh-api/src/routes/bank_accounts.rs` — `close_once`, `reopen_once`, `patch_once`, `conclure`, doc-comments,
  commentaire de la sentinelle
- `crates/kesh-api/src/routes/bank_imports.rs` — doc-comment d'`import_create_error`
- `crates/kesh-db/tests/bank_accounts_repository.rs` — T29a-1, -2, -6, -7, -11
- `crates/kesh-db/tests/payment_batches_repository.rs` — T29a-3, -12
- `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` — T29a-4, -9, -10
- `crates/kesh-api/tests/bank_account_closed_inventory.rs` — troisième inventaire (T29a-5) et volet inverse
- `crates/kesh-api/tests/audit_route_registry.rs` — points (ii), (iii bis), (vi), commentaires
- `CHANGELOG.md` — `[0.13.4]`, `### Corrigé`
- `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-4.md` — C-29-4-23 à C-29-4-27
- `_bmad-output/implementation-artifacts/29-4a-reverification-import-lot-review-prompt-p1.md` — prompt de la revue P1
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — clé `29-4a-reverification-import-lot` en `review`

### Partie b — `29-4b-reverification-annulations` (2026-10-14)

**Agent** : Opus 5.5 (`bmad-dev-story`, autonomie), worktree `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4b`
sur `fe404b11` (29-4a mergée). Commits de code : `dba87062` (étape 0, T6-bis) et `6c8513a6` (29-4b). Journaux sous
`/home/gcorbaz/devel/kesh-gate-logs/` : `294b-rouge-R2.log`, `294b-vert-R2.log`, `294b-rouge-sol-T8.log`,
`294b-vert-T8.log`, `294b-gate-t6bis.log`, `294b-gate-code.log`, `294b-make-fr.log`, `294b-mutations.log` (script
`294b-mutations.py`), `294b-fe-*.log`, `294b-e2e.log`, `294b-e2e-backend.log`, `294b-front-e2e-main.log`.

**T0 — l'état transitoire laissé par la 29-4a, relu et rendu juste** : la puce du manuel (`user-manual.tex`,
§ *Clore un compte bancaire*, « peut encore aboutir … hors de cette fenêtre »), `docs/api-external.md:242` (puce
« Angle mort »), `:571` et `:610` (« par une course »), et `bank_imports.rs` (`innodb_snapshot_isolation`, « aucune vue
n'est ouverte avant les verrous ») sont réécrits ci-dessous ; § *Interblocages* d'`api-external.md` complété en
étape 0.

**Étape 0 (T6-bis), commit `dba87062`** :

- **R-2** — rouge d'abord (`294b-rouge-R2.log`) : `archive_rereads_the_primary_flag_inside_its_transaction`
  (`bank_accounts_e2e.rs`) — `B` principal au pré-vol, transfert du principal à `A` pendant que l'archivage attend la
  sentinelle — rendait au sol

  ```
  left: (412, Some("BANK_ACCOUNT_CANNOT_ARCHIVE_PRIMARY"))
  right: (409, Some("OPTIMISTIC_LOCK_CONFLICT"))
  ```

  Correctif : `bank_accounts::primary_flag_locked_in_tx` (`FOR UPDATE`), appelée dans `archive_once` à chaque
  tentative (C-29-4-33). Constat : au sol, l'effet est un mauvais code de refus, non un archivage indu (le verrou
  optimiste refusait déjà). Vert ensuite (`294b-vert-R2.log`, 81/81 des trois binaires `bank_accounts_e2e`,
  `rejeu_interblocage_e2e`, `bank_accounts_closed_e2e`). Le « M29-34 » de cette correction est le rouge au sol
  lui-même : la mutation qui la défait est le retour à la valeur du pré-vol, qui ne s'écrit pas dans `archive_once`
  seule.
- **R-5** — T29b-7 (`bank_account_closed_inventory.rs`), garde textuelle de `conclure` et des six tentatives ; le
  verrou rendu **avant** la réponse est déclaré **non couvert dynamiquement** (sqlx 0.8.6 rend la connexion par une
  tâche lancée à part : une sonde ferait la course ; C-29-4-34). M29-35 tuée.
- **R-1, R-4** — doc-comments : T29a-15 dit synthétique (au test et au point (vi)) ; « mesuré » et « raisonné, non
  mesuré » distingués (`recheck_open_locked_in_tx`, routes `create_bank_account`, `update_bank_account`,
  `close_bank_account`, point (vi)). T29a-14 : la ligne attendue est désormais celle de `primary_flag_locked_in_tx`
  (même motif de sonde).
- **`docs/api-external.md` § Interblocages** : `POST /bank-accounts`, `PUT`, `PATCH`, `DELETE /bank-accounts/{id}`,
  `close`, `reopen`, et `DELETE /bank-imports/{id}` (réservée à l'interface web) ajoutées ; l'import et la création
  de lot dits non rejoués.
- Gate complet au commit (`294b-gate-t6bis.log`, base remise à zéro dans le même journal) : **3686 passés, 6
  ignorés**.

**Tests rouges d'abord (29-4b)** — `294b-rouge-sol-T8.log`, contre le sol (avant les cinq appels) :

```
⛔ la suppression de l'import vide commencée avant la clôture devait être refusée — reçu Ok(())
⛔ la clôture devait attendre l'annulation, qui tient le compte en partagé — reçu Ok(Ok(()))   (T29b-2, W9)
⛔ l'annulation du rapprochement commencée avant la clôture devait être refusée — reçu Ok(())
⛔ la clôture devait attendre l'annulation, qui tient le compte en partagé — reçu Ok(Ok(()))   (T29b-2, W25)
⛔ l'annulation de la facture payée commencée avant la clôture devait être refusée — reçu Ok(())
⛔ l'annulation du règlement commencée avant la clôture devait être refusée — reçu Ok(())
⛔ l'annulation du règlement fournisseur commencée avant la clôture devait être refusée — reçu Ok(())
Summary [   1.268s] 8 tests run: 1 passed, 7 failed, 20 skipped
la route devait attendre sur ["SELECT closed_on FROM bank_accounts", "LOCK IN SHARE MODE"] — le montage ne prouve rien sinon   (T29b-3)
```

Le seul vert au sol est T29b-5 (annulation sur un compte archivé), garde contre M29-24 par nature. Après les appels :
`bank_accounts_closed_e2e` et `rejeu_interblocage_e2e`, **46/46** (`294b-vert-T8.log`) ; inventaire **8/8**.

**T29b-3 — la victime observée** : l'annulation du règlement par virement (événement `invoices::cancel_settlement`
capturé ; la transaction de test a obtenu l'exercice, `victime()`), rejouée en `200`, une seule contre-passation.
Le cycle neuf compte → exercice est donc détecté (`1213`), non subi jusqu'au délai (`1205`).

**Table *Position par chemin*, lignes retenues** (sur `6c8513a6`) :

| Chemin | Garde 28-4 (vue) | Dernière écriture | Revérification | Suite | Verrou neuf ? | Vue déplacée ? |
|---|---|---|---|---|---|---|
| W9 `reconciliation_cancel::cancel_in_tx` | `reconciliation_cancel.rs:326` | (6) `cancel_settlement_in_tx` `:368` ou `reverse_in_tx` `:384` | `:405`, `recheck_open_locked_in_tx(…, bt.bank_account_id)` | audit `:410`, relecture `find(…, false)` | oui, compte S (déjà tenu si W25 imbriqué l'a pris sur le même compte) | non |
| W25 `invoice_settlements_write::cancel_settlement_in_tx` | `:897` | `UPDATE invoices` `:968` | `:997`, si `settlement_bank_account_id` | audit `:1003` | oui, compte S | non |
| W28 `supplier_invoices::cancel_in_tx` | `:1100` | `UPDATE supplier_invoices … 'cancelled'` `:1154` | `:1176`, sur `inv.settlement_bank_account_id` lu en (1) | audit `:1190`, relectures | oui | non |
| W29 `supplier_invoices::cancel_settlement_in_tx` | `:1351` | `UPDATE supplier_invoices … 'open'` `:1409` | `:1431`, idem | audit `:1435`, relectures | oui | non |
| W24 `bank_imports::delete_unreconciled_in_tx` | `:554` | `DELETE FROM bank_imports` `:557` | `:574` | audit par la route (`delete_once`) | oui | non |
| archivage (R-2) | — | — | `primary_flag_locked_in_tx` (`bank_accounts.rs:1449`), appelée `routes/bank_accounts.rs:1038` | `archive_for_company` (redemande du X) | non (X de la ligne avancé, après la sentinelle) | non |

**Cycles** (table de la fiche, Dev Notes § *Cycles*, relue sur le code) : le verrou neuf est un partagé pris **après**
`companies` (clé étrangère des contre-passations) et après l'exercice ; les partenaires qui tiennent le compte en
exclusif puis demandent l'exercice — `settle_invoice` (`invoices::settle`, `routes/invoices.rs:1282`), `pay_in_tx`
(`supplier_invoices::pay`, `:388` ; `payment_batches::confirm`, `routes/payment_batches.rs:251`) — sont rejoués, et
les cinq annulations aussi (`reconciliation::cancel`, `routes/reconciliation.rs:4723` ; `invoices::cancel_settlement`,
`routes/invoices.rs:1497` ; `supplier_invoices::cancel`, `:419` ; `supplier_invoices::cancel_settlement`, `:451` ;
`bank_imports::delete`, `routes/bank_imports.rs:1494`). Les demandeurs d'exclusif en file (clôture, réouverture,
`PATCH`, routes de gestion) le sont depuis la 29-4a. Mesuré : T29b-3 (un cycle à deux, l'annulation d'un règlement) ;
le reste, raisonné, comme la fiche l'écrit.

**Mutations** (`294b-mutations.log`, motif vérifié unique, restauration `git checkout --` puis `touch`, arbre
`crates` propre après la campagne, sur le commit `6c8513a6`) — **14 jouées, 14 tuées**, aucune erreur de compilation :

| # | Mutation | Tuée par |
|---|---|---|
| M29-8 | W9 : appel retiré | T29b-1 (W9) |
| M29-9 | W24 : appel retiré | T29b-1 (W24) |
| M29-10 | W25 : appel retiré | T29b-1 (W25) |
| M29-11 | W28 : appel retiré | T29b-1 (W28) |
| M29-12 | W29 : appel retiré | T29b-1 (W29) |
| M29-13 | `LOCK IN SHARE MODE` retiré de `RECHECK_OPEN_LOCKED_SQL` | T29b-2, **les deux moitiés** (la clôture n'attend plus : montage) — C-29-4-37 |
| M29-14 | W25 : appel déplacé juste après (2-bis) | T29a-5 / T29b-4 (assertion (iii), « en dernier ») — fonctionnellement équivalente, dite telle |
| M29-15 | W9 : appel déplacé avant l'étape (6) | idem |
| M29-16 | W24 : appel déplacé avant le `DELETE` | idem (position du littéral) |
| M29-17 | W28 : appel déplacé entre `reverse_owned_in_tx(` et l'`UPDATE` | idem |
| M29-18 | W29 : même déplacement | idem |
| M29-24 | W25 appelle la variante « archivé » | T29b-5 (`404 NOT_FOUND`) ; et, rejouée seule sur le registre, T29a-5 (assertions (i), (i bis)) |
| M29-35 | `conclure` sans `rollback` explicite | T29b-7 |

⚠️ Pour M29-14 à M29-18, le journal ne recopie que la tête du message (« ⛔ fenêtre de la clôture ») : la faute est
nécessairement de (iii), un déplacement dans la même fonction ne changeant ni (i) ni (ii) — raisonné, non relu ligne à
ligne.

**Écarts à la fiche** (chacun au registre, C-29-4-32 à C-29-4-37) : T29b-1, T29b-2, T29b-5 dans
`bank_accounts_closed_e2e.rs`, l'annulation appelée au dépôt (C-29-4-32) ; R-2 par une fonction de dépôt neuve
(C-29-4-33) ; R-5 en garde textuelle, effet dynamique non couvert (C-29-4-34) ; `Position` énumération, deux jetons de
dissolution ajoutés (C-29-4-35) ; « Deux limites » → « Deux précisions » au manuel (C-29-4-36) ; la moitié W25 de
T29b-2 tue M29-13 par son montage (C-29-4-37). ⚠️ **Décompte des modules** (dérogation de splitting) : la fiche
annonçait pour la 29-4b quatre modules de code ; l'étape 0, prescrite par l'orchestrateur, en ajoute **deux**
(`repositories/bank_accounts.rs`, dont le code change — fonction neuve —, et `routes/bank_accounts.rs`, R-2) : **six
modules de code**, **sept** fichiers de production comptés largement (plus `routes/reconciliation.rs`, doc-comment
seul), **dix** avec les trois documents. Au-delà de la borne de cinq ; les deux ajoutés étaient déjà ceux de la 29-4a,
et le geste y est d'une fonction et d'un appel. Signalé à l'orchestrateur, non redécoupé.

**Textes de L1** (AC9 part b), réécrits : doc-comment de module et § « La fenêtre de la clôture — fermée par la
revérification » d'`ensure_open_in_tx` (remplace « Angle mort assumé — L1 »), renvoi d'`ensure_open_unarchived_in_tx`,
doc de `recheck_open_locked_in_tx` (appelants, cycle neuf mesuré), `bank_imports.rs` (étape 5 ; `1020` vue ouverte,
#669), registre (doc de `SortieAutorisee` ; raisons d'`accept_one_invoice` ×2 ; W21 ; W30 sans réserve),
`routes/reconciliation.rs` (`closed_account_failed`), `bank_accounts_closed_e2e.rs` (en-tête, W21), commentaires des
cinq gardes (« Revérifiée sous verrou après … »). Manuel (`user-manual.tex`, puce et intitulé) et **PDF régénéré**
(`make fr`, sous verrou, `294b-make-fr.log` ; admin et brochure régénérés aussi, **non commités** : sources
inchangées) — **contrôle aplati** : « hors de cette fenêtre » 0, « ne passe pas sur le compte clos » 1, « Deux
limites » 0, « puis réimporter » 1, « peut encore aboutir » 0. `api-external.md` l. 242, 571, 610 ; CHANGELOG
§ `[0.13.4]`, ligne #651 complétée par union (§ `[0.13.3]` non touchée).

**Propagation** (`grep -rnE "651|\bL1\b"` et `grep -rnE "par une course|hors de cette fenêtre|peut encore aboutir"` sur
`crates docs CHANGELOG.md frontend/src website README.md`) — sites restants, triés : `CHANGELOG.md:55` (§ 0.13.3,
relevé daté) ; `frontend/src/lib/features/reconciliation/failed-proposal-label.ts:146` (« par une course seulement » :
le lot préparé avant la clôture, toujours vrai) ; `docs/api-external.md:571` (« par une course — un lot préparé avant
la clôture », texte prescrit) ; `crates/kesh-db/tests/invoice_frozen_pdf.rs:6` (sans rapport) ; les mentions « Story
29-4a/29-4b, #651 » des doc-comments et des tests (historique, voulues) ; les autres `L1` désignent d'autres
limitations.

**Décomptes** (recomptés, périmètre `fe404b11..6c8513a6`) : tests neufs **12** (attributs de test `crates` : 3503 →
3515, `git grep -cE '#\[(sqlx::)?test'` aux deux bornes) — étape 0 : 2 (R-2, T29b-7) ; 29-4b : 10 (T29b-1 ×5, T29b-2
×2, T29b-3, T29b-5, volet inverse de T29b-4) ; tests modifiés : T29a-5 (sept appels, statut des annulations), son
volet inverse (`Position::…`), doc-comments de T29a-14 et T29a-15. Choix ajoutés **6** (C-29-4-32 à C-29-4-37 ;
registre à 37). Fichiers de production **7** (six de code, un de doc-comment seul).

**Gates réellement exécutés, au commit de code `6c8513a6`** (le commit qui suit ne porte que la fiche, le registre de
sprint, le registre des choix et le prompt de revue) :

- `scripts/test-fast.sh`, sous verrou, base `kesh_294` remise à zéro dans le même journal (`294b-gate-code.log`,
  « RESET_OK » puis le gate) : **3696 passés, 6 ignorés**.
- Frontend, sous verrou (`294b-fe-*.log`) : `npm run check` 0 erreur (29 avertissements préexistants),
  `lint-i18n-ownership` vert, `test:unit` **1640/1640** (147 fichiers), `build` vert. Frontend non touché.
- E2E complet, sous verrou, recette de `docs/testing.md` (port 3045, base `kesh_e2e_294` recréée, `KESH_TEST_MODE`
  des deux côtés, backend tué par PID ; `294b-front-e2e-main.log`, `294b-e2e.log`) : **285 passés, 7 échecs, 19
  ignorés** (11,5 min). Les sept, jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus* :
  `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150` —
  **les sept KF-029 (#97)** ; aucun autre.

**File List (partie b)** : `crates/kesh-db/src/repositories/{reconciliation_cancel,invoice_settlements_write,supplier_invoices,bank_imports,bank_accounts}.rs`,
`crates/kesh-api/src/routes/{bank_accounts,reconciliation}.rs`,
`crates/kesh-api/tests/{bank_accounts_closed_e2e,rejeu_interblocage_e2e,bank_account_closed_inventory,bank_accounts_e2e,audit_route_registry}.rs`,
`docs/manual/fr/user-manual.tex` et `.pdf`, `docs/api-external.md`, `CHANGELOG.md` ; `_bmad-output/implementation-artifacts/`
: cette fiche, `epic-29-choix-autonomes-29-4.md` (C-29-4-32 à 37), `sprint-status.yaml` (clé en `review`),
`29-4b-reverification-annulations-review-prompt-p1.md`.

**Demandé à l'orchestrateur** : la revue de code P1 (`29-4b-reverification-annulations-review-prompt-p1.md`, Opus,
lentilles B/E/A) ; puis le push et la PR, corps **`closes #651`** (sur la PR seulement ; aucun commit ne le porte).

#### Remédiation de la revue de code P1 de la 29-4b (2026-10-14)

*(Sous-section ajoutée ; ce qui précède est le relevé du développement, laissé tel qu'écrit à sa date — les annotations
ci-dessous le complètent sans le réécrire.)* Opus 5.5, worktree `/home/gcorbaz/devel/kesh-29-4`, commits `7dabbfcf`
(test, B-1) et `9848de33` (code et `docs/api-external.md`) ; rapports `kesh-gate-logs/29-4b-review-p1-{A,B,E}.md`.

**B-1 — prouvé** (`294b-mutations.log`, section « Remédiation revue P1 ») : test neuf
`the_last_acquisition_rule_sees_each_locking_token`, dix cas construits (table écrite à part des listes), décompte
anti-muet, couverture dans les deux sens (C-29-4-38). Avant lui, retirer `INSERT INTO` laissait verts le volet inverse
existant et T29a-5 (constaté au même jeu). **M-B1a** (`INSERT INTO` retiré de `GESTES_VERROUILLANTS_SQL`) : rouge,
`jeton « INSERT INTO » — left: [« aucun geste verrouillant dans la fonction »]`, 8 passés / 1 échec ; restauré
(`git checkout --` puis `touch`), recompilé, 9/9. **M-B1b** (`reverse_in_tx(` retiré de `GESTES_VERROUILLANTS_CODE`) :
rouge de même, 8/1 ; restauré, recompilé, 9/9.

**E-4 / A-9** : `annuler_pendant_une_cloture` exige désormais que la sonde rende `true` ; `false` annule la clôture
puis fait échouer le test en citant l'issue de l'annulation (C-29-4-39). C'était le seul des trois appels
d'`attendre_une_requete_en_cours` ajoutés par la 29-4b (`git diff fe404b11..4862cd0e -- crates/kesh-api/tests`) qui en
ignorait le résultat ; les deux autres l'assertent (patron de T29b-2, `bank_accounts_closed_e2e.rs` ; test de R-2,
`bank_accounts_e2e.rs`). T29b-3 passe par la sonde préexistante de `rejeu_interblocage_e2e.rs`, qui l'asserte aussi.

**A-4 — table des cycles, recopiée** de la lentille E de la P1 (§ 2.2, relue au code par elle ; raisonnée sauf mention) :

| Verrou neuf / avancé | Partenaire | Ce que le partenaire tient → demande | Cycle | Participants rejoués ? | Victime |
|---|---|---|---|---|---|
| `A` S des annulations W9, W25, W28, W29 | W8 `settle_invoice` (virement vers `A`) | `A` X → exercice X ; pour W9 « écriture », aussi l'intervalle de `uq_invoice_settlements_entry` | à deux, neuf : annulation (exercice X) ↔ W8 (`A` X) | oui : `invoices::settle` ; les quatre annulations | W8 au réel (raisonné) ; l'annulation dans T29b-3, partenaire alourdi (mesuré) |
| idem | W27 `pay_in_tx` (`pay`, `confirm_batch`) | facture X → `A` X → exercice X | à deux, neuf | oui : `supplier_invoices::pay`, `payment_batches::confirm` | le plus léger des deux |
| idem | `PATCH` (W13) | réglages S → `A` X → `UPDATE` (`accounts` S, FK) | non à deux ; à trois par W8 interne (angle mort « PATCH », fermé) | oui | `PATCH` |
| idem | routes de gestion `M` (`PUT`, `POST`, `DELETE`) | `companies` X → `A` X | non : l'annulation tient `companies` S avant `A` | oui (29-4a) | — |
| idem | clôture, réouverture | `A` X → rien | non à deux ; par la file : annulation → clôture en file → détenteur S de `A` (import, lot) | oui | clôture / réouverture |
| idem | à quatre `Q → E → I → M → Q` | `I` tient `A` S, attend `companies` S derrière `M` ; `M` attend la sentinelle que `Q` tient | oui (résidu (b), fermé) | oui, sauf `I` | `M` ou `E` |
| idem | deux annulations entre elles | `Q2`, revérification passée, ne demande plus que l'audit | non | — | — |
| idem | restauration | `DELETE` massif, dont `bank_accounts` | en théorie | annulation oui ; restauration non | l'annulation (non relu) |
| `A` S de W24 | W8, W27 ; `M` | exercice, intervalle de règlement ; `companies` X → `A` X | non : W24 ne tient ni exercice, ni `companies`, ni intervalle de règlement | — | — |
| idem | import `I` + clôture en file (à trois) | `I` tient `A` S, attend l'intervalle d'`idx_bank_transactions_import` tenu par W24 ; W24 attend `A` S derrière la clôture | oui, neuf — résidu (a) (C-29-4-7) | clôture et W24 oui ; `I` non | la clôture |
| `A` S de W9 sous `GET_LOCK` | tout geste de réconciliation du compte | tous prennent `GET_LOCK` en tête, avant toute ligne | aucun cycle mixte | — | effet : `409 RECONCILIATION_ACCOUNT_LOCKED` au bout de 5 s pendant l'attente (E-6, écrit à `api-external.md:242`) |
| `A` X avancé de l'archivage (R-2) | import / lot `I` | `I` tient `A` S, demande `companies` S | `I ↔ M` préexistant, inchangé | archivage oui ; `I` non | l'archivage |
| idem | annulations | `companies` S avant `A` ; W24 tient `A` S en dernier sans rien demander que l'archivage tienne | non | — | — |

**A-5** : écart à la prescription de T6-bis écrit au registre (C-29-4-33, annotation) — le test de R-2 joue une
démotion commitée entre le pré-vol et la **première** tentative, non une promotion entre deux tentatives ; et (B-2) il
prouve la relecture dans la transaction, non le `FOR UPDATE`.

**A-6** (annotation du relevé « 14 jouées, 14 tuées ») : **14 jeux joués pour 13 mutations** — M29-24 est jouée deux
fois, la seconde rejouée seule sur le registre (`BILAN M29-24 (registre) TUÉE`). Le total était juste au journal ; la
table ne le reconstituait qu'à la lecture de la cellule de M29-24.

**B-8 — angle mort assumé, sans test ajouté** : le chemin imbriqué W9 → W25 (règlement client rapproché : deux
revérifications, celle de W25 puis celle de W9) n'est exercé par aucun entrelacement — T29b-1 ne joue W9 qu'en `Entry` ;
et le `GET_LOCK` de W9 n'est pas sur le chemin testé (les annulations sont appelées au dépôt, hors
`with_account_lock`). La fonction appelée est commune et la position contrôlée par l'inventaire (T29b-4) ; la limite
« deux comptes différents » du chemin imbriqué (E-5) est écrite au commentaire (6 bis) de W9.

**Gates au dernier commit de code `9848de33`** : voir l'entrée du Change Log « Revue de code P1 de la 29-4b ».

## Change Log

- 2026-10-14 — Fiche créée (`bmad-create-story`, Opus 5.5, autonomie). Prémisses de #651 établies au code : W30 sans
  fenêtre (I1), W24 seulement pour un import vide, piste « rejeu seul » insuffisante. Conception : revérification
  verrouillante partagée (C-29-4-1, -2) ; découpage 29-4a / 29-4b (C-29-4-6). Choix C-29-4-1 à C-29-4-12 au registre
  `epic-29-choix-autonomes-29-4.md`.
- 2026-10-14 — **Validation P1** (Sonnet 5.5, deux lentilles en contexte frais, lecture seule, sur `cd61980b` ;
  prompt `29-4-cloture-compte-concurrente-validate-prompt-p1.md`). Lentille R : **0 CRITICAL / 0 HIGH / 6 MEDIUM / 7 LOW**
  (`kesh-gate-logs/29-4-validate-p1-R.md`) ; lentille F : **0 / 0 / 2 / 7** (`…-p1-F.md`), table de l'axe obligatoire
  fournie, axes exercés et non exercés déclarés par les deux. Les deux concluent que la conception tient (aucun cycle non
  rejoué, aucune écriture hors fenêtre, aucune vue déplacée) ; les MEDIUM sont des défauts de preuve et de rédaction.
  Convergences : R1 = F1, R4 = F6, R5 = F2, R6 = F7. **Remédiation** (Opus 5.5, même jour), tous findings traités :
  - **R1 = F1** (MEDIUM) — séquence de T29a-4 réécrite : la demande de `R` par `T1` en tâche, attente de l'événement de
    rejeu par sondage borné, annulation de `T2`, puis jointure de `T1` (AC4).
  - **R5 = F2** (MEDIUM) — tests **T29a-6** (W22) et **T29a-7** (W17), « l'écrivain a inséré, la clôture attend, puis
    l'écrivain revérifie », avec délai de 500 ms et `timeout` de 2 s ; P-d scindée : existence du verrou vérifiée
    (T-a31), redemande sans effet **raisonnée** jusqu'à ces tests ; mutation M29-19 (AC1, AC2, AC3).
  - **R4 = F6** (MEDIUM / LOW) — **décision de l'orchestrateur** appliquée : la revérification ne dépend plus en silence
    d'`innodb_snapshot_isolation`. Établi au code : une erreur de base inconnue sort en `DbError::Sqlx` → `500
    INTERNAL_ERROR` (`map_db_error` ignore 1020) ; constaté sur la base de dev : `10.11.16`, variable présente, `OFF`.
    Retenu : **épinglage `OFF` par session** dans `create_pool` (AC11, prémisse P-f, choix C-29-4-13), testé par
    T29a-8, mutation M29-5 ; version du NAS **à constater** par Guy.
  - **R6 = F7** (MEDIUM / LOW) — décompte des modules refait (tableau, doc-comments seuls comptés à part) : 29-4a cinq
    modules de code, 29-4b quatre de code mais sept comptée largement → section *Dérogation règle de splitting (29-4b)*
    (C-29-4-18).
  - **R2** (MEDIUM) — l'ordre textuel lit les jetons SQL aux positions des **littéraux**, mêlées aux jetons de code de
    la copie masquée ; mutations M29-16 (W24), M29-17 (W28), M29-18 (W29) ajoutées (AC5, AC8).
  - **R3** (MEDIUM) — l'assertion (ii) ne lit plus le texte des raisons : ensemble `G` **extrait** (appelants
    d'`ensure_open*` ∪ littéraux non verrouillants lisant `closed_on`), égal à `FENETRE` ; W1-W6 et `accept_one_*`
    retirés de `FENETRE` (ce sont des `Invariant`) ; neuf entrées au sol (C-29-4-15).
  - **F3** — doc-comment de la route `close_bank_account` ajouté à l'AC4. **F4** — angle mort « PATCH » (cycle à trois
    par `accounts`) écrit ; méthode d'extraction étendue aux exclusifs implicites. **F5** — la **réouverture n'est plus
    rejouée** (aucun cycle réaliste ; C-29-4-5 amendé) : point (vi) passe de sept à **huit**, M5 remplacée. **F8** —
    (a) la course archivage / import et lot (L5) nommée, non fermée, issue demandée (C-29-4-16) ; (b) résidu C-29-4-7
    réécrit comme théorique. **F9** — `a_concurrent_close_waits_for_the_unreconciliation` nommé pour ce qu'il garde.
  - **R7** — « 31 + 3 entrées » → 27 + 3 ; tests et mutations renumérotés `T29a-n` / `T29b-n` / `M29-n` (C-29-4-17).
    **R8** — trois sites de L1 ajoutés à l'AC9 (`bank_accounts.rs:28`, `bank_accounts_closed_e2e.rs:1577`, registre
    l. 89 avec son texte). **R9** — borne des sous-`SELECT` d'audit recopiée au volet 1. **R10** — `rollback` explicite
    avant les comptages et dans `close_once` ; constante `#[doc(hidden)] pub` et substitution des `?` pour l'`EXPLAIN`.
    **R11** — puce du manuel l. 1748-1752, `UPDATE` du `msg_id` l. 159, libellé CHANGELOG « suppression d'un import
    vide ». **R12** — (a) état transitoire des textes écrit à l'AC9 ; (b) raison de W30 « sous réserve de la 29-4b » en
    29-4a ; (c) aucune barrière outillée contre une release entre les deux PR : **retenu tel** — la variante `Ouverte`
    disparaît du code en 29-4b, l'interdit reste de procédure (Status, Dev Agent Record). **R13** — AC10 rattaché aux
    deux parties.
  - Propagation : chaque valeur corrigée greppée sur la fiche, son registre et son prompt (`T-a[1-5]`, `T-b`, `M[0-9]`,
    `31 + 3`, `1747`, `l. 158`, `reopen_once`, `sept`) — résidus restants : les seules mentions historiques, voulues
    (« l'ancienne M5 », « la fiche écrivait … »). Choix ajoutés : C-29-4-13 à C-29-4-18 ; C-29-4-5, -6, -7 amendés
    (annotation datée). Prompt de la validation P2 : `29-4-cloture-compte-concurrente-validate-prompt-p2.md` (Opus).
- 2026-10-14 — **Validation P2** (Opus 5.5, deux lentilles en contexte frais, lecture seule, sur `2fdcfb5b` ; prompt
  `29-4-cloture-compte-concurrente-validate-prompt-p2.md`). Lentille R : **0 CRITICAL / 1 HIGH / 3 MEDIUM / 6 LOW**
  (`kesh-gate-logs/29-4-validate-p2-R.md`, R1-R10) ; lentille F : **0 / 1 / 3 / 7** (`…-p2-F.md`, table des verrous et
  des cycles fournie) — ⚠️ recompté : le rapport F annonce 7 LOW et en numérote **huit** (F5 à F12), F12 étant marqué
  « hors périmètre » ; il est traité ci-dessous avec les autres. Axes exercés et non exercés déclarés par les deux.
  Convergences : R1 = F1 (HIGH), R2 = F4, R3 = F6, R4 = F7, R9 = F3. **Origine** : R1 = F1, R3 = F6, R4 = F7, R8 (deux
  citations), R9 = F3 et F8 sont nés de la **remédiation P1** ; F2, R5, R6, R7, F9 à F11 de la conception d'origine ;
  R2 = F4 de décisions de l'orchestrateur postérieures à la fiche, non encore propagées ; F5, F12 de l'héritage.

  ⛔ **Signal D5 — déclaré au Project Lead** (§ *Règle de splitting préventif*, amendement du 2026-10-06) : la P1 rendait
  **0 / 0 / 8** (six MEDIUM pour R, deux pour F, tous défauts de preuve ou de rédaction) ; la P2 rend un **HIGH
  convergent né de la remédiation P1** (l'épinglage de l'AC11, posé dans une fonction que la production n'appelle pas)
  — sévérité en hausse **et** défaut issu du correctif précédent : c'est le **recyclage** que la règle désigne, non une
  revue qui trouve des défauts d'origine neufs. S'y ajoutent trois MEDIUM de la P2 eux aussi issus de la remédiation P1
  (R3 = F6, R4 = F7, F3). Réponse retenue par l'orchestrateur : **retirer** ce que la remédiation P1 avait ajouté
  (l'AC11), plutôt que l'étendre au pool réel — ce qui supprime le défaut au lieu de le corriger et fait **redescendre**
  la 29-4a de cinq à quatre modules de code ; aucun redécoupage. La décision de continuer la boucle sans découper reste
  celle du Project Lead, à qui le signal est remonté par le rapport de cette remédiation.

  **Remédiation** (Opus 5.5, même jour), décisions de l'orchestrateur consignées comme telles :
  - **R1 = F1** (HIGH) — **retrait de l'épinglage de session** (décision 1) : `create_pool` n'a aucun appelant, la
    production construit son pool dans `main.rs:74`. Sortent de la story : l'AC11 (remplacé par une note de retrait),
    `configurer_session`, T29a-8, M29-5 (numéros non réattribués), T1-bis ; C-29-4-13 **annoté**, non réécrit. Porté par
    **#669** (bug, P3 : `create_pool` mort, `sql_mode`, `time_zone`, `innodb_snapshot_isolation`). Angle mort écrit :
    sous MariaDB 11.6+, la revérification rendrait `1020` → `500`, comme les deux sites préexistants (P-f, AC11, Dev
    Notes). Les doc-comments de `bank_imports.rs:439-446` et `routes/reconciliation.rs:2166-2170` ne se touchent plus ;
    celui de `bank_imports.rs:444-445` (29-4b) renvoie à #669. R10 et F8, qui portaient sur l'épinglage, **tombent**.
    Modules recomptés : 29-4a **quatre** (cinq avant), 29-4b quatre de code et sept comptée largement (inchangée) ; sept
    distincts au total (huit avant) ; la dérogation de la 29-4b reste, relue ; aucune pour la 29-4a.
  - **R2 = F4** (MEDIUM) — `archived` (décision 4) : **deux SQL, deux fonctions** (`RECHECK_OPEN_LOCKED_SQL` /
    `recheck_open_locked_in_tx`, `closed_on` seul, pour les annulations ; `RECHECK_OPEN_UNARCHIVED_LOCKED_SQL` /
    `recheck_open_unarchived_locked_in_tx`, ordre « archivé avant clos », pour W22 et W17), symétrie de la 28-4 ; W22 rend
    `NotFound` → `404` existant ; W17 convertit `NotFound` en `IllegalStateTransition` au site d'appel ; les annulations
    ne lisent **jamais** `archived`. Tests T29a-11, T29a-12 (course avec l'archivage), T29b-5 (annulation sur un compte
    archivé) ; mutations M29-22 à M29-25 ; T-a45 étendu à `archived` ; registre : `Reverifiee(Variante, …)`,
    assertions (i) sur le nom appelé et (i bis) en dur pour les annulations. AC1, AC2, table *Dehors*, angles morts,
    doc-comment `bank_accounts.rs:866-867` (et commentaire `routes/bank_accounts.rs:904-907`) corrigés ; CHANGELOG
    élargi à l'archivage ; **plus d'issue L5** ; C-29-4-16 annoté (renversé) ; C-29-4-20, C-29-4-22.
  - **F3 = R9** (MEDIUM) — **la réouverture est rejouée** (décision 2 de la P1 renversée : l'argument « toute écriture sur
    un clos est refusée avant de prendre ce partagé » était faux ; cycle à quatre dont elle est victime) : `reopen_once`
    et son enveloppe, T29a-9, M29-20 ; C-29-4-14 annoté (renversé). Résidu écrit : dans ce cycle à quatre, une route de
    gestion non rejouée est à égalité de poids avec la réouverture.
  - **F2** (MEDIUM) — **le `PATCH` du compte est rejoué** (décision 3 ; cycle à trois W24 / `PATCH` / import créé par la
    29-4b) : `patch_once` et son enveloppe, T29a-10 (patron `victime()`), M29-21 ; l'angle mort « PATCH » par `accounts`
    de la P1 **se ferme** du même geste ; C-29-4-19. Point (vi) d'`audit_route_registry.rs` : sept → **dix**.
  - **R3 = F6** — T29a-6 insère une transaction `pending` (étape (4 bis), ordre de production). **R4 = F7** — le délai de
    500 ms remplacé par une sonde d'état de verrou (`INNODB_TRX.trx_state = 'LOCK WAIT'` pour le `CONNECTION_ID()` de la
    clôture, fixture `attendre_un_verrou_en_attente`) ; C-29-4-21.
  - LOW : **R5** — T29b-2 ne prétend plus que la moitié W25 tue M29-13 (tuée par W9 et M29-8 à M29-12). **R6** —
    commentaire `audit_route_registry.rs:336-338` et ligne du `PATCH` à l'AC4. **R7** — `api-external.md:555` à l'AC9.
    **R8** — `routes/bank_imports.rs:1196-1201` (au lieu de `:1464-1467`) ; `routes/reconciliation.rs:4908` libellé
    `closed_account_failed` ; la citation de `:2166` tombe avec l'AC11. **F5** — interblocage préexistant import ou lot /
    route de gestion écrit (table *Dehors*, angles morts). **F9** — la restauration ajoutée aux détenteurs d'exclusif.
    **F10** — attente de W9 sous le verrou nommé, au doc-comment de W9 (AC6). **F11** — l'argument de poids écrit (Dev
    Notes § *Cycles*, doc-comment des revérifications), et § *Cycles par la file d'attente* généralisé à tout demandeur
    d'exclusif sur le compte. **F12** — porté par #669.
  - **Refusé** : aucun finding. Écarté, au registre : rejouer aussi les routes de gestion pour fermer le résidu du cycle
    à quatre (C-29-4-14, annotation de la P2).
  - **Recomptes** (depuis la fiche) : tests neufs de la partie a **onze** (T29a-1 à T29a-12, T29a-8 retiré), plus T-a45
    étendu ; partie b **cinq** identifiants (T29b-1 en cinq tests, T29b-2 en deux moitiés, T29b-3 à T29b-5) ; mutations
    **24** identifiants (M29-1 à M29-25, M29-5 retiré ; M29-8 à M29-12 une par site) ; choix **22** (C-29-4-1 à 22) ;
    routes `SansEcritureAuJournal` rejouées au point (vi) : **dix** (sept au sol, à recompter au rebase).
  - Propagation : greppés sur la fiche, le registre et `sprint-status.yaml` — `AC11`, `T29a-8`, `M29-5`,
    `configurer_session`, `create_pool`, `épingl`, `huit`, `n'est pas rejouée`, `non rejouée`, `ne lit pas .archived`,
    `issue à ouvrir`, `500 ms`, `L5`, `clôture seule` ; résidus restants : les relevés datés (Change Log de création et
    de la P1, choix d'origine annotés, prompts P1 et P2) et les mentions de la note de retrait, voulues. Prompt de la
    validation P3 : `29-4-cloture-compte-concurrente-validate-prompt-p3.md` (Sonnet).
- 2026-10-14 — **Validation P3** (Sonnet 5.5, deux lentilles en contexte frais, lecture seule, sur `ffad9a23` ; prompt
  `29-4-cloture-compte-concurrente-validate-prompt-p3.md`). Lentille R : **0 CRITICAL / 0 HIGH / 1 MEDIUM / 11 LOW**
  (`kesh-gate-logs/29-4-validate-p3-R.md`, R1-R12) ; lentille F : **0 / 1 / 0 / 6** (`…-p3-F.md`, F1-F7, table des
  verrous et des cycles fournie). Axes exercés et non exercés déclarés par les deux. Convergences : R1 = F1 (le **même
  défaut**, classé MEDIUM par R et HIGH par F), R4 = F4, R5 = F6 (a), R6 = F6 (b). **Origine** : R1 = F1, R3, R4 = F4,
  R5, R6, R8 (deux premiers sites), R9, R11, F2, F5 et T29a-12 de R10 sont nés de la **remédiation P2** ; R7, R12 et
  T29a-3 de R10 de la remédiation P1 ; R2 (T29a-4), F7 et R8 (commentaires des annulations) de la conception d'origine ;
  F3 de l'héritage. **Trend** : P1 R 0/0/6/7, F 0/0/2/7 → P2 R 0/1/3/6, F 0/1/3/7 → P3 R 0/0/1/11, F 0/1/0/6. Le HIGH de
  la P3 naît, comme celui de la P2, du correctif précédent (une prescription de la P2 contredite par deux commentaires
  du dépôt) : le motif « la remédiation introduit le défaut suivant » est observé une seconde fois, sur un défaut de
  preuve (un test irréalisable sur le poste du gate), non de conception ; la sévérité totale baisse (8 au-dessus de LOW
  en P2, 1 défaut en P3). **Remédiation** (Opus 5.5, même jour), décisions de l'orchestrateur consignées comme telles :
  - **R1 = F1** (MEDIUM / HIGH) — **décision de l'orchestrateur** : la sonde `INNODB_TRX`, qui exige le privilège
    `PROCESS`, est remplacée par le helper existant `sonde_verrou_nowait` (`SELECT id FROM bank_accounts WHERE id = ?
    LOCK IN SHARE MODE NOWAIT`, connexion tierce, en boucle locale au test, bornée à 10 s), qui ne demande aucun
    privilège ; la fixture `attendre_un_verrou_en_attente` est retirée ; garde (1 bis) contre le montage à vide (la
    sonde réussit avant la clôture). **Établi** le 2026-10-14 sur la MariaDB de dev (`10.11.16`, base `kesh_294`,
    table jetable `sonde_294`, trois connexions `mariadb` sous `kesh`) : la sonde réussit avec un partagé tenu, échoue
    aussitôt en `1205` avec un exclusif en file, réussit après les commits ; le détenteur partagé y a redemandé son
    partagé sans attendre (indice pour P-d (2), écrit à la table des prémisses). Repli écrit (précédent
    `reconciliation_e2e.rs:7546-7553`). Constaté au passage : le conteneur de dev de la station accorde ce jour
    `ALL PRIVILEGES ON *.*` à `kesh`, contrairement à `01-dev-grants.sql` — une sonde à privilège y aurait passé ; écrit
    aux Pièges. AC3, T2, Dev Notes (tests, pièges), C-29-4-21 annoté.
  - **R4 = F4** — la fixture retirée, la 29-4a revient à **quatre** modules de code ; le « quatre » de la P2 était faux
    à sa date, d'un module (C-29-4-6 annoté).
  - **F6 = R5** — **décision de l'orchestrateur** : les points sur MariaDB 11.6+ (manuel d'administration l. 108 et
    2708, `docker-compose.prod.yml:8`, version du NAS) sont versés en commentaire à **#669** ; la fiche y renvoie
    (AC11 retiré, table *Dehors*, angles morts) et ne prescrit aucun texte de manuel pour 11.6+. **R6 = F6 (b)** — « comme
    les deux sites préexistants » rectifié partout où la fiche le prescrit : `accept_one_invoice`
    (`routes/reconciliation.rs:2166-2170`) rendrait `DATABASE_ERROR` dans `failed[]`, la revérification un `500`.
  - LOW : **R2** — `mark_onboarding_complete` au montage de T29a-4 et T29a-9. **R3** — `bank_imports.rs:444-445` dit
    réécrit en 29-4b pour une autre raison que l'AC11 (AC11 retiré, AC9) ; la phrase de l'entrée P2 ci-dessus (« ne se
    touchent plus ; celui de `bank_imports.rs:444-445` (29-4b) renvoie à #669 ») se lit ainsi : 439-446 non touché au
    titre de l'AC11, sauf 444-445 au titre de W24. **R7** — le § L1 d'`ensure_open_in_tx`, réécrit en 29-4a, sort de
    la liste de l'état transitoire ; son décompte « Huit chemins » se refait depuis la liste. **R8** —
    `errors.rs:1247-1249` et `routes/bank_imports.rs:57-61` à l'AC9 (29-4a, doc-comments seuls : la 29-4a, comptée
    largement, passe à six fichiers, dérogation étendue — C-29-4-18 annoté) ; commentaires des cinq gardes à l'AC6.
    **R9** — points (ii) et (iii bis) : une phrase pour la famille `AppError`, leurs « sept » (famille `DbError`)
    inchangés. **R10** — W17 : `create_batch` annule lui-même ; rollback explicite pour W22 seulement (T29a-3,
    T29a-12). **R11** — la clause sur la note ⁹ retirée. **R12** — extraction `G` (a) qualifiée (`bank_accounts::`).
    **F2** — le résidu du cycle à quatre réécrit pour tout demandeur d'exclusif en file (clôture, réouverture,
    `PATCH`, W8, W27) : `M` à égalité avec la clôture et la réouverture, **victime désignée** contre le `PATCH`, W8 et
    W27 ; critère volet 2, AC4, § *Cycles*, angles morts, doc-comment prescrit ; C-29-4-14 annoté. **F3** — seconde
    contingence de T29a-4 et T29a-9 (victime autre que la clôture ou la réouverture : consignée, remontée) ; « raisonné »
    devant l'argument de poids, recopié au doc-comment avec ses deux réserves ; C-29-4-7 annoté. **F5** — précondition
    `company_invoice_settings` de T29a-10 écrite. **F7** — `docs/api-external.md:594` ajouté à l'AC9 (29-4b), et la
    propagation greppe la formulation (« par une course », « hors de cette fenêtre ») en plus du jeton.
  - **Refusé** : aucun finding.
  - **Recomptes** (depuis la fiche et le registre) : tests neufs de la partie a **onze** (inchangé) ; partie b **cinq**
    identifiants (inchangé) ; mutations **24** (inchangé) ; choix **22** (`grep -c '^### C-29-4-'`, aucun neuf ;
    annotés en P3 : C-29-4-6, -7, -14, -18, -21) ; modules de code : 29-4a **quatre**, 29-4b **quatre**, **sept**
    distincts ; comptés largement : 29-4a **six**, 29-4b **sept**.
  - Propagation : greppés sur la fiche et le registre — `INNODB_TRX`, `attendre_un_verrou_en_attente`, `PROCESS`,
    `LOCK WAIT`, `CONNECTION_ID`, `comme les deux sites`, `égalité`, `strictement`, `toujours plus léger`, `note ⁹`,
    `par une course`, `58-6`, `quatre modules` ; résidus restants : les relevés datés (Change Log de la P2, annotation P2
    de C-29-4-6, texte d'origine de C-29-4-21) et les mentions de retrait, voulues. Prompt de la passe ciblée P4 :
    `29-4-cloture-compte-concurrente-validate-prompt-p4-ciblee.md` (Haiku, une lentille sur le seul diff de cette
    remédiation).

### Validation P4 (passe ciblée) — 2026-10-14

**Passe ciblée** (Haiku 5.5, une lentille, sur le seul diff de la remédiation P3, commits `e1f8e17f` et `540ee118` ;
prompt `29-4-cloture-compte-concurrente-validate-prompt-p4-ciblee.md`, rapport
`/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p4-ciblee.md`) : **CRITICAL 0 / HIGH 0 / MEDIUM 1 / LOW 5**. Chaque
constat a été vérifié au texte (`grep -nF`) avant d'être traité ; tous sont réels. **Trend complet** : P1 R 0/0/6/7,
F 0/0/2/7 → P2 R 0/1/3/6, F 0/1/3/7 → P3 R 0/0/1/11, F 0/1/0/6 → **P4 ciblée 0/0/1/5**. Origine du MEDIUM : la garde
(1 bis) ajoutée par la remédiation P3 (le motif « la remédiation introduit le défaut suivant » se confirme une dernière
fois, sur une garde de test).
**Remédiation** (Opus 5.5, même jour) :
  - **M1** (MEDIUM) — la garde (1 bis) devient une **sonde de référence appelée entre les étapes (1) et (2)**, avant le
    lancement de la clôture : elle doit rendre `true` (seul l'écrivain tient la ligne en partagé) ; la boucle de
    l'étape (3) ne sert plus qu'à attendre le `false`. Écrite à l'AC3 (donc pour T29a-6 et T29a-7) et à T2.
  - **L1** — la fiche écrit que `LOCK IN SHARE MODE NOWAIT` n'a aucun précédent dans le dépôt, que son comportement a été
    constaté hors dépôt (10.11.16, remédiation P3) et s'établit au T0 du développement, avec le repli déjà écrit ;
    le doc-comment du helper se généralise.
  - **L2** — la prescription de `docs/api-external.md:594` nomme « le 409 de l'annulation du rejet ». **L3** — « tout
    autre participant … hors route de gestion » (Dev Notes, argument de poids). **L4** — la règle du décompte large est
    énoncée (fichiers de production, code ou doc-comments, sans documents ni tests) : 29-4a six, 29-4b six (neuf avec ses
    trois documents) ; le « sept » de la 29-4b de la P1 n'obéissait à aucune règle. **L5** — `reconciliation_e2e.rs:7538-7553`
    cité pour le motif « 500 ms » du repli.
  - **Refusé** : aucun finding. **Recomptes** (depuis la fiche et le registre) : choix **22**, mutations **24**, T29a
    **onze**, T29b **cinq**, modules de code 29-4a **quatre** / 29-4b **quatre** / **sept** distincts (inchangés).
    Les entrées P1 à P3 ci-dessus ne sont pas réécrites : leur « sept » de la 29-4b comptée largement est annoté ici.
  - Propagation : greppés — `(1 bis)`, `sonde de l'étape (3)`, `ce refus`, `trois au moins`, `sept`, `7546-7553`
    (les relevés datés P3 le gardent, voulu), `LOCK IN SHARE MODE NOWAIT`.
  - **Validation close : la remédiation de la passe ciblée ne touche aucune ligne de production** (fiche et registre
    seulement ; aucun code). La 29-4a et la 29-4b passent en développement, après la 29-3.

### Développement de la 29-4a — 2026-10-14

`bmad-dev-story` (Opus 5.5, autonomie) sur `089fcd80`. T0 établi au code (sonde `LOCK IN SHARE MODE NOWAIT` et redemande
du partagé de clé étrangère, sur MariaDB 10.11.16, aucun repli) ; quatre tests de fenêtre rouges au sol, puis les deux
revérifications et leurs appels (W22, W17) ; P-d (2) établie au dépôt (T29a-6, T29a-7) ; trois enveloppes de rejeu,
victimes observées conformes (T29a-4, T29a-9, T29a-10) ; troisième inventaire du registre (T29a-5, volet inverse).
Commit de code `e718221b` (`refs #651`). Gates au commit de code : `test-fast.sh` 3637/3637 (6 ignorés), Vitest
1617/1617, E2E 282 passés + 7 KF-029 ; 15 mutations, 15 tuées. Choix C-29-4-23 à C-29-4-27. Détail au Dev Agent
Record. Revue de code P1 à lancer : `29-4a-reverification-import-lot-review-prompt-p1.md` (Opus, B/E/A).

### Revue de code P1 de la 29-4a — 2026-10-14

**Passe** : Opus, trois lentilles en contexte frais (`29-4a-reverification-import-lot-review-prompt-p1.md`), sur
`3d7851be`. Rapports `kesh-gate-logs/29-4a-review-p1-{B,E,A}.md` : **B 0/0/2/6, E 0/0/0/4, A 0/0/1/8** (CRITICAL /
HIGH / MEDIUM / LOW) — 21 findings (8 + 4 + 9), dont trois paires convergentes (A-1 = B-1, B-3 = A-6, E-3 = B-6),
soit **18 distincts** ; **deux MEDIUM distincts** (A-1 = B-1, B-3), tous deux **d'origine** — aucun ne naît d'une
remédiation. **Remédiation** (Opus 5.5), commit de code
`bc9cf85e`, après **rebase** de la branche sur `b4abae5d` (29-1a mergée).

**Rebase** : conflits réunis par union — `sprint-status.yaml` (clés de `main` et de la 29-4, en-têtes de la branche
renumérotés 145 à 149 au-dessus de la plus haute de `main`, A-9), `CHANGELOG.md` § `[0.13.4]` (les deux entrées de
`### Corrigé`), `crates/kesh-db/src/repositories/bank_accounts.rs` (`mod tests` : le test de `statement_gap` de la 29-1a
et `les_reverifications_verrouillent_en_partage`, entiers). **Interaction révélée par le gate ciblé** : le troisième
inventaire a vu la constante `EXAMINED_ACCOUNTS_SQL` de `closing_checks.rs` (29-1a), lecture de `closed_on` sans verrou
hors fonction — classée `Fermee` (lecture seule), `G` à dix (C-29-4-29).

**Décisions de l'orchestrateur, appliquées comme telles** :

- **A-1 = B-1 (MEDIUM, convergents)** — les doc-comments nommaient la revérification de l'import et du lot comme la
  demande partagée en file derrière la clôture, ce que T29a-6 et T29a-7 réfutent. Réécrits d'après ce qui est prouvé
  (`close_in_tx`, `set_journal_account_id_for_company`, les deux revérifications, `close_bank_account`,
  `reopen_bank_account`, `patch_bank_account_journal_link`, point (vi)) : la redemande du partagé tenu n'attend pas ;
  ce qui attend derrière un exclusif en file, c'est une demande **neuve** (l'exclusif d'une route de gestion, le
  contrôle de clé étrangère d'un second `INSERT`, et en 29-4b la revérification d'une annulation). Cycle atteignable en
  29-4a, écrit : `I → M → E → I` (import ou lot `I` entre ses deux contrôles de clé étrangère, route de gestion `M` sur
  la sentinelle, demandeur d'exclusif `E` en file), et `I ↔ M` à deux.
- **E-1 (LOW, relevé par l'orchestrateur)** — les routes de gestion du compte sont **rejouées** : `create_once`,
  `update_once`, `archive_once` sous `retry_app_on_deadlock` (`bank_accounts::create`, `::update`, `::archive`), patron
  de la réouverture et du `PATCH`, `conclure` commune ; T29a-13 (`PUT`), T29a-14 (archivage), T29a-15 (création), patron
  `victime()` et capture du rejeu ; **rouges d'abord** (`294-rouge-p1-E1.log` : `3 tests run: 0 passed, 3 failed`, les
  trois en `INTERNAL_ERROR`), verts ensuite (`294-vert-p1-E1.log`, 17/17 du binaire) ; mutations M29-26 à M29-28. Point
  (vi) d'`audit_route_registry.rs` recompté : dix + trois = **treize** ; points (ii) et (iii bis) : six routes `AppError`
  à preuve dynamique. Table *Cycles* de la fiche (deux lignes « `PUT`, `POST`, archivage ») et résidu du cycle à quatre
  annotés ; C-29-4-14 annoté, C-29-4-28. Le commentaire de la sentinelle de l'archivage (« la course … est fermée par la
  revérification ») est laissé : il dit vrai.
- **B-3 = A-6 (MEDIUM)** — le volet inverse de T29a-5 éprouve désormais les trois autres fautes de (iii) : « suit
  `avant` », littéral `apres` absent, littéral `avant` absent, chacune seule, plus un cas de contrôle conforme sans
  faute ; mutations M29-30 à M29-32.
- **E-3 = B-6 (LOW)** — `rollback` explicite sur le refus de l'import après son `INSERT` : aide
  `create_import_or_roll_back` aux deux sites (CAMT, CSV) ; T-a47 réécrit en conséquence (un appel de
  `create_with_transactions`, dans l'aide ; deux appels de l'aide) ; mutation M29-29.
- **E-2 (LOW)** — la phrase `innodb_snapshot_isolation` élargie à tout commit sur la ligne du compte depuis la vue, avec
  renvoi à #669 (commentaire versé par l'orchestrateur).
- **LOW traités** : A-2 et B-2 (cycles de la 29-4b marqués « à partir de la 29-4b » ; `recheck_open_locked_in_tx` dite
  sans appelant de production en 29-4a) ; A-3 (M29-3 bis aux doc-comments de T29a-2 et T29a-3) ; A-4 (doc de
  `sonde_verrou_nowait` généralisé à `LOCK IN SHARE MODE NOWAIT`) ; A-5 (écarts versés au registre, C-29-4-30 ; C-29-4-8
  annoté) ; A-7 (trois déclarations du Dev Agent Record annotées « non journalisé » — la remise à zéro de cette passe,
  elle, est journalisée) ; A-8 (`T29a-1` retiré de l'interdit `FOR UPDATE`) ; A-9 (en-têtes renumérotés) ; B-4 (borne
  haute de W17 : `avant: Some("UPDATE payment_batches SET msg_id")`, mutation M29-33) ; B-5 (« cinq chemins », l'annulation
  de rejet sortie avec la raison I1 ; « six » du Dev Agent Record laissé, relevé daté) ; B-7 (attente de `t2` exigée à
  l'étape (3)) ; B-8 (ligne du CHANGELOG mise en forme, phrase API, et la ligne des routes de gestion rejouées) ; E-4
  (« aucun verrou d'enregistrement accordé », « la redemande passe sans attente »).
- **Refusé** : aucun.

⚠️ **État transitoire neuf** : `docs/api-external.md` § *Interblocages* ne liste pas les routes du compte bancaire
désormais rejouées (clôture, réouverture, `PATCH` depuis le développement ; création, `PUT`, archivage depuis cette
remédiation) — à reprendre en 29-4b avec le reste d'`api-external.md` (la liste est trop **courte**, pas fausse dans
l'autre sens).

**Mutations** (`kesh-gate-logs/294-mutations-review-p1.log`, script `294-mutations-review-p1.py`, motif vérifié unique,
restauration `git checkout --` puis `touch`, arbre propre après campagne) — **8 jouées, 8 tuées**, chacune par
l'assertion attendue (aucune erreur de compilation) :

| # | Mutation | Tuée par |
|---|---|---|
| M29-26 | `PUT` : enveloppe retirée | T29a-13 (`INTERNAL_ERROR`) |
| M29-27 | archivage : enveloppe retirée | T29a-14 (`INTERNAL_ERROR`) |
| M29-28 | création : enveloppe retirée | T29a-15 (`INTERNAL_ERROR`) |
| M29-29 | import refusé : `rollback` explicite retiré | T-a47 |
| M29-30 | inventaire (iii) : faute « suit `avant` » jamais produite | volet inverse de T29a-5 (cas « suit ») |
| M29-31 | inventaire (iii) : faute « littéral `avant` absent » jamais produite | volet inverse (cas « avant absent ») |
| M29-32 | inventaire (iii) : faute « littéral `apres` absent » jamais produite | volet inverse (cas « apres absent ») |
| M29-33 | W17 : revérification déplacée après l'`UPDATE` du `msg_id` | T29a-5 (borne haute neuve) |

**Gates au commit de code `bc9cf85e`** (le commit qui suit ne porte que la fiche, le registre de sprint et le prompt) :

- `scripts/test-fast.sh`, sous verrou, base `kesh_294` **remise à zéro dans le même journal** (`294-gate-review-p1.log`,
  « RESET_OK » puis le gate) : **3668 passés, 6 ignorés**.
- Frontend, sous verrou : `npm run check` 0 erreur (29 avertissements préexistants), `lint-i18n-ownership` vert,
  `test:unit` **1640/1640** (147 fichiers), `build` vert (`294-p1r-front-*.log`).
- E2E complet, sous verrou, recette de `docs/testing.md` (port 3045, base `kesh_e2e_294` recréée, backend tué par PID ;
  `294-p1r-e2e.log`, `294-p1r-e2e-run.log`) : **285 passés, 7 échecs, 19 ignorés** (11,5 min). Les sept, jugés fichier par
  fichier contre `docs/testing.md` § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`,
  `:92`, `onboarding.spec.ts:57`, `:77`, `:150` — **les sept KF-029 (#97)** ; aucun autre.

**Décomptes** (périmètre `daab782f..bc9cf85e`) : tests neufs **3** (T29a-13 à T29a-15 ; attributs de test
`crates` : 3484 → 3487, `git grep -cE '#\[(sqlx::)?test'` aux deux bornes) ; tests modifiés **4** (volet inverse de T29a-5,
T29a-5 — décompte de `G` —, T-a47, T29a-10 — aides extraites) et l'aide `cycle_a_trois_par_la_file` (étape (3), qui
sert T29a-4 et T29a-9) ; choix ajoutés **3** (C-29-4-28 à C-29-4-30 ;
registre à 30) ; fichiers de production touchés **4** (`routes/bank_accounts.rs`, `routes/bank_imports.rs` : code ;
`repositories/bank_accounts.rs`, `test_fixtures.rs` : doc-comments seuls).

**Suite** : la remédiation touche du code de production — la boucle n'est pas close. Passe ciblée P2 à lancer :
`29-4a-review-prompt-p2-ciblee.md` (**Sonnet**, une lentille sur le seul diff de la remédiation, table des verrous des
trois enveloppes neuves). PR toujours `refs #651`.

### Revue de code P2 (passe ciblée) de la 29-4a — 2026-10-14

**Passe** : Sonnet, une lentille (R) sur le seul diff de la remédiation P1 (`bc9cf85e`), prompt
`29-4a-review-prompt-p2-ciblee.md`, rapport `kesh-gate-logs/29-4a-review-p2-ciblee.md`. **0/0/0/5** (CRITICAL / HIGH /
MEDIUM / LOW : R-1 à R-5). **Trend** : P1 B 0/0/2/6, E 0/0/0/4, A 0/0/1/8 → P2 ciblée Sonnet 0/0/0/5. Aucun défaut de
code de production nouveau dans les trois enveloppes ni dans `create_import_or_roll_back` ; R-2 est préexistant (élargi
par le rejeu), R-1, R-3, R-4, R-5 naissent de la remédiation P1.

**Traité ici** : R-3 — la ligne #651 du CHANGELOG (§ 0.13.4) ne promet plus l'absence d'erreur interne : après trois
tentatives d'interblocage, la route rend encore `500`. Seul changement de la remédiation : un `.md`, aucune ligne de code.

**Reporté à la 29-4b (T6-bis, étape 0)** : R-1 (cycle de T29a-15 synthétique), R-2 (`was_primary` à relire dans la
transaction de chaque tentative d'`archive_once`, avec test), R-4 (doc-comments de cycles : mesuré / supposé), R-5
(mutation du `rollback` de `conclure`), et la liste des routes rejouées de `docs/api-external.md`.

**Revue close** : la remédiation ne touche aucune ligne de code de production (règle de la passe ciblée). Clé
`29-4a-reverification-import-lot` : `done`.

### Développement de la 29-4b — 2026-10-14

`bmad-dev-story` (Opus 5.5, autonomie) sur `fe404b11`, branche `story/29-4b`. **Étape 0 (T6-bis)**, commit `dba87062` :
R-2 corrigé (l'archivage relit `is_primary` sous verrou à chaque tentative ; test rouge d'abord), R-5 en garde
textuelle (effet dynamique non couvert, déclaré), R-1 et R-4 aux doc-comments, § *Interblocages* d'`api-external.md`
complété. **29-4b**, commit `6c8513a6` : huit tests rouges au sol (T29b-1 ×5, T29b-2 ×2, T29b-3), puis les cinq
revérifications en dernière acquisition (W9, W24, W25, W28, W29) ; T29b-5 (archivé) ; registre `EnDernier` et son volet
inverse (T29b-4), `Ouverte` retiré ; textes de L1, manuel et PDF, `api-external.md`, CHANGELOG. Gates : `test-fast.sh`
3686 (étape 0) puis 3696 (6 ignorés) ; 14 mutations, 14 tuées. Choix C-29-4-32 à C-29-4-37. Détail au Dev Agent
Record, § *Partie b*. Revue de code P1 à lancer : `29-4b-reverification-annulations-review-prompt-p1.md` (Opus,
B/E/A). PR : **`closes #651`** dans le corps de la PR, jamais dans un commit.

### Revue de code P1 de la 29-4b — 2026-10-14

**Passe** : Opus, trois lentilles en contexte frais (`29-4b-reverification-annulations-review-prompt-p1.md`), sur
`4862cd0e`. Rapports `kesh-gate-logs/29-4b-review-p1-{A,B,E}.md` : **A 0/0/1/8, B 0/0/1/7, E 0/0/1/5** (CRITICAL /
HIGH / MEDIUM / LOW) — 23 findings (9 + 8 + 6), dont les paires convergentes A-1 = B-5, A-2 = E-1, A-7 = B-7,
A-8 ⊂ B-5, A-9 = E-4, B-4 = E-3, soit **17 distincts** ; **trois MEDIUM distincts** : B-1 (test), A-1 (doc), E-1
(doc). Aucun ne naît d'une remédiation ; aucun ne touche le comportement du code de production. **Remédiation**
(Opus 5.5) : commits `7dabbfcf` (test) et `9848de33` (code : doc-comments, une assertion de test,
`docs/api-external.md`). Principe : retirer plutôt qu'ajouter, le texte ramené à l'exactitude du code, du mécanisme
seulement là où un test était aveugle (B-1, E-4).

**Corrigé** :

- **B-1 (MEDIUM)** — test neuf `the_last_acquisition_rule_sees_each_locking_token` : un cas par jeton des deux listes
  (dix), table écrite à part des listes, couverture dans les deux sens (C-29-4-38). Prouvé par deux mutations, M-B1a
  (`INSERT INTO`) et M-B1b (`reverse_in_tx(`), rouges puis vertes après restauration et `touch`.
- **A-1 / B-5 (MEDIUM)** — étape « 6 bis » à la liste ordonnée du doc-comment de `reconciliation_cancel::cancel_in_tx`,
  avec l'attente sous `GET_LOCK` (AC6, T8) ; (4 bis) de W28 sans ambiguïté (valeur lue en (1), remise à `NULL` par
  l'`UPDATE`) — A-8.
- **E-1 / A-2 (MEDIUM)** — § *Angle mort* de `recheck_open_locked_in_tx` réécrit pour ses appelants réels (les cinq
  annulations), tout commit sur la ligne du compte, `1020` ni traduit ni rejoué, sans effet sous 10.11, #669 ;
  `bank_imports.rs` (étape 5) élargi de « une clôture commitée » à tout commit sur la ligne ; renvoi court aux quatre
  autres sites (W9, W25, W28, W29).
- **E-4 / A-9** — T29b-1 exige que l'annulation soit vue en attente (C-29-4-39).
- **LOW** : B-3 (archivage : le verrou est pris par `primary_flag_locked_in_tx`, deux doc-comments et un commentaire
  de `routes/bank_accounts.rs`, plus le renvoi de `recheck_open_unarchived_locked_in_tx`) ; B-4 / E-3 (W24 ne ferme pas
  le cycle compte → exercice : doc de T29b-3 et de `recheck_open_locked_in_tx`) ; E-2 (la victime de T29b-3 tient au
  montage ; contre un `settle_invoice` réel, `settle_invoice`, rejoué) ; E-5 (limite W9 imbriquant W25 sur deux comptes,
  au commentaire (6 bis)) ; E-6 (`api-external.md:242` : attente de W9 sous le verrou de réconciliation, `409
  RECONCILIATION_ACCOUNT_LOCKED` au bout de 5 s) ; A-7 / B-7 (`api-external.md:571` : l'état laissé par la v0.13.3) ;
  B-2 (doc de `primary_flag_locked_in_tx` : le test prouve la relecture, non le `FOR UPDATE` ; la sentinelle sérialise
  les bascules des routes de gestion ; l'intégration, seule exception, non jouée) ; B-6 (`NotFound` dit inatteignable
  par construction) ; A-3 (annotation datée de la *Dérogation*) ; A-4 (table des cycles recopiée au Dev Agent Record) ;
  A-5 (écart à la prescription de T6-bis, annotation de C-29-4-33) ; A-6 (14 jeux pour 13 mutations, annoté) ; B-8
  (angle mort assumé, écrit au Dev Agent Record).

**Laissé** : aucun finding sans traitement. Restent, déclarés : B-8 sans test (angle mort assumé, sur instruction) ;
B-6 traité au texte, non par un `DbError::Invariant` ; la mise à jour de #669 pour nommer les cinq annulations
(commande `gh` d'écriture, interdite à l'agent — demandée à l'orchestrateur). Le manuel n'est pas touché : aucun texte
visible de l'interface ne change.

**Propagation** (grep des valeurs sur `crates`, `docs/api-external.md`, `docs/manual/fr/*.tex`, `CHANGELOG.md`,
`frontend/src`) : « un import ou un lot » — restent quatre sites où l'import et le lot sont les écrivains `I` des
cycles, justes, et `CHANGELOG.md:25` ; « clôture commitée » — restent des sites où la clôture est la course traitée,
justes ; « même cycle » — T29a-9 (sans rapport) et le doc corrigé de T29b-3 ; « hors de cette fenêtre » — 0 ;
`archive_for_company` comme lieu du verrou — corrigé à `recheck_open_unarchived_locked_in_tx` en plus des sites signalés ;
« par une course » — `api-external.md:571` (complété), `failed-proposal-label.ts:146` (toujours vrai),
`invoice_frozen_pdf.rs:6` (sans rapport).

**Décomptes** (recomptés) : attributs de test `crates`, `git grep -cE '#\[(sqlx::)?test'` : 3515 à `4862cd0e`, **3516**
à `9848de33` (un test neuf, B-1). Choix ajoutés : **2** (C-29-4-38, C-29-4-39 ; registre à 39), un annoté (C-29-4-33).

**Gates au dernier commit de code `9848de33`** (le commit qui suit ne porte que la fiche, le registre et le prompt de
la P2) :

- `scripts/test-fast.sh`, sous verrou, base `kesh_294` remise à zéro dans le même journal (`294b-gate-p1-remed.log`,
  « RESET_OK » puis le gate) : **3697 passés, 6 ignorés** (3696 + le test neuf).
- Frontend, sous verrou (`294b-p1-fe-*.log`) : `check` 0 erreur, 29 avertissements (préexistants) ;
  `lint-i18n-ownership` vert ; `test:unit` **1640/1640** (147 fichiers) ; `build` vert.
- E2E complet, sous verrou, recette de `docs/testing.md` (port 3045, base `kesh_e2e_294` recréée, backend tué par
  PID ; `294b-p1-front-e2e-main.log`, `294b-p1-e2e.log`) : **285 passés, 7 échecs, 19 ignorés** (11,6 min). Les sept,
  jugés fichier par fichier : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
  `onboarding.spec.ts:57`, `:77`, `:150` — **les sept KF-029 (#97)** ; aucun autre.

**Suite** : passe ciblée P2, une lentille, sur le seul diff de la remédiation —
`29-4b-reverification-annulations-review-prompt-p2-ciblee.md` (rapport `kesh-gate-logs/29-4b-review-p2-ciblee.md`).
La remédiation de la P1 a touché du code de production (commentaires seulement) : la boucle n'est pas close avant
elle.

### 2026-10-14 — Revue de code P2 (passe ciblée, Sonnet) : 0/0/1/4, remédiée par l'orchestrateur ; revue close

Prompt `29-4b-reverification-annulations-review-prompt-p2-ciblee.md`, rapport `kesh-gate-logs/29-4b-review-p2-ciblee.md`.
Les cinq findings viennent de la remédiation P1 (récidive « la remédiation introduit le défaut suivant »), tous sur des
commentaires ; sortie par **retrait**.

- **R-1 (MEDIUM)** : le doc-comment de `primary_flag_locked_in_tx` disait que le `FOR UPDATE` couvre « la seule promotion
  qui ne prend pas la sentinelle — l'intégration ». Faux : `routes/onboarding.rs:585` prend `lock_company` avant
  `upsert_primary_in_tx` (`:586`). Phrase retirée, remplacée par le constat ; annotation datée à C-29-4-33.
- **R-2 (LOW)** : « aucun geste ne supprime une ligne de `bank_accounts` hors de la restauration » — faux,
  `kesh_seed::reset_demo` vide aussi la table (`TABLES_TO_TRUNCATE`, `backup.rs:79`). Corrigé.
- **R-4 (LOW)** : `docs/api-external.md:242` — l'annulation d'un autre rapprochement ajoutée aux gestes qui peuvent rendre
  `409 RECONCILIATION_ACCOUNT_LOCKED`.
- **R-5 (LOW)** : renvoi à l'angle mort `innodb_snapshot_isolation` (#669) ajouté au site W24 (`bank_imports.rs`).
- **R-3 (LOW), laissé** : le motif écrit de l'assertion de T29b-1 (et de C-29-4-39) décrit un cas que le montage ne
  produit pas (la sonde panique à 10 s plutôt que de rendre « borne expirée ») ; l'assertion reste un durcissement juste.
- Non rejoué par la passe : les mutations M29-8 à M29-12, antérieures à l'assertion neuve de T29b-1 ; par lecture,
  chacune meurt désormais aussi à cette assertion.
- **Clôture** : la remédiation ne touche que des commentaires et un paragraphe de documentation d'API, aucune ligne
  exécutable ; trend P1 (A 0/0/1/8, B 0/0/1/7, E 0/0/1/5) → P2 ciblée 0/0/1/4 → 0 au-dessus de LOW après retrait.
  Gate complet et E2E rejoués au commit de cette remédiation (fichiers de `kesh-db` touchés).

### 2026-10-14 — Rebase de la 29-4b sur `e889346c` (29-6) et gates finaux au HEAD rebasé ; clé `done`

- **Rebase** : `story/29-4b`, partie de `fe404b11`, rejouée sur `origin/main` `e889346c` (Story 29-6). Sept commits
  rejoués (`ba32b215` … `047bdcbf`, anciens `dba87062` … `f2aff17c`).
- **Conflits, réunis par union** : deux seulement.
  - `docs/manual/fr/user-manual.pdf` (binaire, au rejeu de l'ancien `6c8513a6`) : le PDF de main repris au conflit,
    puis régénéré par `make fr` (sous verrou et `mem-guard`, 2 passes, stable) depuis le `.tex`, qui avait fusionné
    sans conflit ; commit `869b4402`. Contrôle du PDF aplati : phrases de la 29-6 présentes (« quelques centimes de
    trop », « ni facture désignée que le paiement dépasse », « Aucune correspondance » ×3) et de la 29-4b (« Deux
    précisions », « ne passe pas sur le compte clos », « rouvrir le compte, puis réimporter ») ; aucun « ?? ».
  - `sprint-status.yaml` (en-têtes `# last_updated`, au rejeu de l'ancien `4862cd0e`) : l'en-tête de la branche,
    n° 153, renuméroté **159** et placé au-dessus des 153 à 158 de main (29-6) ; les clés ont fusionné seules (29-4b
    de la branche, 29-6 de main). `yaml.safe_load` sans erreur ; aucun marqueur de conflit dans le worktree.
  - `CHANGELOG.md`, `docs/api-external.md`, `routes/reconciliation.rs`, `user-manual.tex` : fusion automatique.
- **Gates au HEAD rebasé `869b4402`** (arbre propre ; `kesh_294` remise à zéro dans le même journal,
  `kesh-gate-logs/294b-gate-rebase.log`) :
  - `scripts/test-fast.sh` : fmt, clippy, **3705 tests, 3705 passés, 6 ignorés** (3697 avant le rebase ; +8, ceux de
    la 29-6).
  - Frontend : `npm run check` 0 erreur (29 avertissements, préexistants), `lint-i18n-ownership` PASS,
    `npm run test:unit` **1645 / 1645** (147 fichiers), `npm run build` vert.
  - E2E complet (backend sur 3045, base `kesh_e2e_294` neuve, répertoires `/tmp/kesh-e2e-294-rb` neufs) :
    **287 passés, 7 échoués, 19 ignorés** — les sept KF-029 (#97) de `docs/testing.md` § *Les échecs attendus* :
    `mode-expert.spec.ts:26` et `:41`, `onboarding-path-b.spec.ts:65` et `:92`, `onboarding.spec.ts:57`, `:77` et
    `:150` ; ni KF-052 ni pollution sur ce run. Journal `kesh-gate-logs/294b-rb-e2e.log`, script `294b-rb-run.sh`.
- **Clé** `29-4b-reverification-annulations` passée à `done` (en-tête n° 160).
