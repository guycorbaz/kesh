# Story 29.4 : Une saisie concurrente ne passe plus sur un compte qu'on clôt

## Status

**ready-for-dev** (deux clés) — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `16eeed21`
(kickoff de l'Epic 29), worktree `/home/gcorbaz/devel/kesh-29-4`, branche `story/29-4`. Validation P1 faite (Sonnet,
lentilles R et F : 0/0/6/7 et 0/0/2/7) et remédiée le 2026-10-14 (Change Log) ; validation P2 à lancer
(`29-4-cloture-compte-concurrente-validate-prompt-p2.md`, **Opus**, lentilles R et F) — le fichier d'epic demande pour
cette story une **validation complète Opus et Sonnet**, et aucune simplification sans relecture des verrous et de la
naissance de la vue (B5 de la rétrospective 28).

⛔ **Deux stories, une fiche, deux PR successives** (C-29-4-6, sur le précédent C-28-4-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`29-4a-reverification-import-lot`** | la revérification verrouillante (fonction et SQL), posée sur l'**import** (W22) et la **création de lot** (W17) **après** leur `INSERT` à clé étrangère — verrou déjà tenu, **aucun verrou neuf** ; le **rejeu** de la clôture ; `innodb_snapshot_isolation` épinglé à `OFF` par session (AC11) ; le troisième inventaire du registre (`bank_account_closed_inventory.rs`) ; une ligne `### Corrigé` du CHANGELOG | AC1 à AC5, AC11, AC9 (part a), AC10 | **`refs #651`** — ⛔ jamais `closes` |
| **`29-4b-reverification-annulations`** | la revérification posée, en **dernière acquisition de verrou**, sur les cinq annulations (W9, W24, W25, W28, W29) ; tests d'entrelacement et du cycle neuf ; manuel utilisateur (PDF), `docs/api-external.md`, doc-comments de L1, CHANGELOG complété | AC6 à AC8, AC9 (part b), AC10 | **`closes #651`** |

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
| P-d | « Un import qui tombe dans la fenêtre laisse des transactions `pending` sur un compte clos » ; « un lot de paiement créé dans la fenêtre reste attaché à ce compte » | **vraie** — et c'est par l'`INSERT` à clé étrangère que ces deux chemins se ferment sans verrou neuf (AC2) | `bank_imports.rs:84` (`INSERT INTO bank_imports`), `payment_batches.rs:144` (`INSERT INTO payment_batches`). Deux prémisses InnoDB, de statut différent : (1) **l'existence** d'un verrou conflictuel avec l'exclusif de la clôture sur la ligne du compte, tenu par l'écrivain après ses `INSERT` — **vérifiée en pratique** par T-a31 de la 28-4 (`bank_accounts_repository.rs:2296`, `une_cloture_attend_un_import_en_cours_puis_le_voit`, qui voit la clôture attendre ; il ne dit ni quel `INSERT` l'a posé, ni son mode) ; (2) **la redemande sans effet** — une lecture `LOCK IN SHARE MODE` par égalité sur la clé primaire, quand l'écrivain tient déjà ce verrou et qu'une clôture **attend déjà** en exclusif, est satisfaite sans file ni attente — **raisonnée** (`lock_rec_lock_slow` teste `lock_rec_has_expl` avant tout contrôle de conflit ; contrôle de clé étrangère et recherche unique sur la clé primaire posent tous deux `LOCK_S \| LOCK_REC_NOT_GAP`), **tant que T29a-6 et T29a-7 (AC3) ne l'ont pas établie**. Si (2) était fausse : la fenêtre resterait fermée (la lecture verrouillante lit le dernier état commité), mais chaque course formerait un cycle écrivain / clôture dont la clôture, rejouée (AC4), serait la victime — « aucun verrou neuf » serait alors à réécrire |
| P-f | *(prémisse de la fiche, non de l'issue)* Une lecture verrouillante « lit le dernier état commité » | **vraie sous `innodb_snapshot_isolation = OFF` seulement** | Constaté sur la base de dev le 2026-10-14 : `SELECT @@version, @@innodb_snapshot_isolation` rend `10.11.16-MariaDB`, `0`. À partir de MariaDB **11.6**, le défaut passe à `ON` : une lecture verrouillante qui rencontre une version postérieure à la vue ouverte rend `ER_CHECKREAD` (**1020**) — exactement la course que la revérification traite. Le dépôt le sait déjà (`bank_imports.rs:439-446`, `routes/reconciliation.rs:2166-2170`, `reconciliation_e2e.rs:7556`, `:8401`) et ne le traite pas : `map_db_error` ne connaît pas 1020 (`kesh-db/src/errors.rs:1459-1494`), qui sort en `DbError::Sqlx` → `500 INTERNAL_ERROR` (`kesh-api/src/errors.rs:4114-4120`). D'où l'AC11 (C-29-4-13). ⚠️ **Version du NAS : inconnue** — `docker-compose.prod.yml:8` et `ci.yml:48` supposent la parité 10.11 avec le paquet MariaDB de Synology, sans relevé ; **fait à constater** par Guy (`SELECT @@version, @@innodb_snapshot_isolation` sur la base du NAS) |
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
2. le **rejeu sur interblocage** de la clôture (elle devient victime possible d'un cycle à trois, AC4) — **non** celui de
   la réouverture, qui n'a pas de cycle réaliste (C-29-4-5, amendé en P1) ;
3. le troisième inventaire du registre statique (les sites à garde non verrouillante, chacun résolu) ;
4. `innodb_snapshot_isolation` épinglé à `OFF` sur chaque connexion du pool de production (AC11, C-29-4-13) — sans
   quoi la revérification, sous MariaDB 11.6 ou plus, rendrait un `500` dans la course même qu'elle traite (P-f) ;
5. les textes qui décrivent L1 : doc-comments, manuel utilisateur (PDF), `docs/api-external.md`, CHANGELOG.

**Dehors**, et pourquoi :

| Exclu | Raison |
|---|---|
| Retirer les gardes non verrouillantes de la 28-4 | Elles donnent le **refus précoce** et l'**ordre des refus** que la 28-4 a fixés (T-a27, W24 a/b) ; la revérification ne mord que dans la course (C-29-4-3) |
| Rejouer l'import (`POST /bank-imports`) ou la création de lot | Aucun cycle neuf à deux ne les atteint (leur revérification ne demande aucun verrou) ; seul un cycle à trois passant par la clôture le pourrait, et la clôture, plus légère, y est la victime (C-29-4-7). Rejouer l'import exige d'extraire ~160 lignes de deux chemins (CAMT, CSV) : hors périmètre minimal, résidu écrit |
| Prendre la sentinelle `companies`, un verrou nommé (`GET_LOCK`) ou les lignes enfants dans la clôture | Écartés au registre (C-29-4-1) : cycles avec les routes de gestion, cycles mixtes non détectés par InnoDB, ou ensemble de verrous dont l'analyse rejoue celle que la 28-4 a dû abandonner |
| Réparer les états laissés par la fenêtre de la v0.13.3 (une `pending` ou un lot `generated` sur un compte clos) | Le remède manuel reste (rouvrir, traiter, clore) ; `cancel_batch` (W21) reste la sortie autorisée. Aucune donnée réelle ne le requiert (aucun signalement) |
| L'inversion `companies` → `bank_accounts` de W8 / W27 contre les routes de gestion (28-4, AC7, « au sol, inchangée ») | Préexistante, non touchée par la story : les routes de gestion ne sont pas rejouées, la story ne les enveloppe pas |
| Manuels DE / IT / EN | `docs/manual/{de,en,it}/` ne portent qu'un `README.md` |
| Fermer la course **archivage** / import et archivage / création de lot (la revérification ne lit pas `archived`, F8 de la P1) | Ce n'est pas #651 (clôture) ; elle est **déjà nommée** au dépôt (`ensure_open_unarchived_in_tx`, `bank_accounts.rs:866-867` : « Ne ferme pas la course archivage / import (L5 de la route d'archivage) ») ; la fermer n'est pas mécanique (W17 rend `IllegalStateTransition` pour un archivé, W22 `NotFound` : deux traductions, une seconde forme de revérification). Angle mort écrit (Dev Notes) ; **issue à ouvrir** par l'orchestrateur, aucune n'existant pour L5 (C-29-4-16) |
| Rejeu de la réouverture | Aucun cycle réaliste : pour qu'elle en soit victime, il faudrait un détenteur partagé de la ligne d'un compte **clos** en attente d'une ressource ; toute écriture sur un clos est refusée avant de prendre ce partagé (garde non verrouillante, ou lecture sous `FOR UPDATE`), et les gestes permis sur un clos (W21, W30) ne touchent pas la ligne du compte (F5 de la P1, C-29-4-5 amendé) |
| Traiter `innodb_snapshot_isolation` ailleurs que par l'épinglage de session (traduction de 1020 chemin par chemin, ou adaptation des autres sites qui en dépendent) | L'épinglage couvre, d'un geste, la revérification **et** les sites existants qui supposent `OFF` (`routes/reconciliation.rs:2166`, `bank_imports.rs:439`) ; leurs doc-comments se mettent à jour (AC11), leur code non |

**Décompte des modules** (§ *Règle de splitting préventif*), **recompté en P1** (R6 = F7) — un module de code est un
fichier de production dont le **code** change ; un fichier dont seuls des doc-comments changent se compte **à part** ;
les tests, `docs/` et le CHANGELOG aussi :

| | Modules de code | Doc-comments seuls | Documents | Tests |
|---|---|---|---|---|
| **29-4a** | `kesh-db` : `repositories/bank_accounts` (fonction neuve), `repositories/bank_imports` (appel), `repositories/payment_batches` (appel), `pool` (épinglage, AC11) ; `kesh-api` : `routes/bank_accounts` (enveloppe) — **cinq** | `kesh-api/src/routes/reconciliation.rs:2166` (AC11) | `CHANGELOG.md` | `kesh-db/tests/bank_accounts_repository.rs`, `kesh-api/tests/{rejeu_interblocage_e2e,bank_account_closed_inventory,audit_route_registry}.rs` |
| **29-4b** | `kesh-db` : `repositories/reconciliation_cancel`, `repositories/invoice_settlements_write`, `repositories/supplier_invoices`, `repositories/bank_imports` — **quatre** | `kesh-db/src/repositories/bank_accounts.rs` (§ L1, l. 28, 807-823, 863-865), `kesh-api/src/routes/reconciliation.rs:4908` | `docs/manual/fr/user-manual.tex` (+ PDF), `docs/api-external.md`, `CHANGELOG.md` | `kesh-db/tests/{bank_accounts_repository,invoice_settlement,supplier_invoices_repository,reconciliation_cancel}.rs` (répartition au choix du dev), `kesh-api/tests/{rejeu_interblocage_e2e,bank_account_closed_inventory,bank_accounts_closed_e2e}.rs` |

Au total (les deux parties) : **huit** modules de code — plus de cinq, d'où le découpage (C-29-4-6). La 29-4a en touche
**cinq**, la borne de la règle (qui découpe au-delà de cinq) ; la 29-4b **quatre** de code, mais **sept** si l'on compte
comme la fiche le faisait avant la P1 (doc-comments et `docs/manual` compris) — plus de cinq : voir § *Dérogation règle de
splitting*. Le découpage suit aussi le **risque** : la 29-4a n'ajoute aucun verrou d'écrivain ; la 29-4b porte seule le
verrou neuf et le cycle qu'il crée, et c'est elle que la validation doit relire le plus durement.

### Dérogation règle de splitting (29-4b)

*(Écrite en P1, R6 = F7 : l'écriture d'origine, « cinq, la borne », était un sous-décompte.)* Comptée largement, la 29-4b
touche sept modules ; elle n'est **pas** redécoupée :

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
   `retry_on_deadlock` / `retry_app_on_deadlock` / `retry_with`), ou passe par la clôture, que la story fait rejouer
   (AC4) — à deux angles morts près, écrits et sans cas réaliste (cycle à trois par `accounts` où le `PATCH` serait
   victime ; résidu théorique C-29-4-7) ;
3. **Vue** — *aucune naissance de vue déplacée* (critère hérité de l'AC7 de la 28-4) : la story n'ajoute **aucune
   lecture non verrouillante** et ne déplace aucune des gardes non verrouillantes de la 28-4 ; la revérification est
   une lecture **verrouillante**, posée **après** la première lecture non verrouillante de chaque chemin, si bien que la
   vue est née avant elle quelle que soit la façon dont InnoDB traite une lecture verrouillante.

### Partie a — `29-4a-reverification-import-lot`

**AC1 — La revérification : une fonction, un SQL, une forme de verrou contrôlée.**

- Neuf, dans `crates/kesh-db/src/repositories/bank_accounts.rs`, à côté d'`ensure_open_in_tx` (l. 839) :
  `#[doc(hidden)] pub const RECHECK_OPEN_LOCKED_SQL: &str = "SELECT closed_on FROM bank_accounts WHERE id = ? AND company_id = ? LOCK IN SHARE MODE";`
  — publique et cachée pour que T29a-1 la lise depuis `tests/` (précédent : `LOCK_IMPORT_TRANSACTIONS_SQL`,
  `bank_imports.rs:394`, et son test `lock_query_uses_the_import_index`) ;
  et `pub async fn recheck_open_locked_in_tx(tx, company_id, bank_account_id) -> Result<(), DbError>` : `None` →
  `DbError::NotFound` ; `Some(Some(_))` → `DbError::BankAccountClosed { bank_account_id }` ; `Some(None)` → `Ok(())`.
  ⛔ **`LOCK IN SHARE MODE`, jamais `FOR SHARE`** (erreur de syntaxe sur MariaDB 10.11, `bank_accounts.rs:1241`) ;
  ⛔ **ne lit pas `archived`** (les annulations doivent passer sur un compte archivé, table *Dehors* de la 28-4).
- **Forme du verrou** : la recherche est une **égalité sur la clé primaire** — verrou d'enregistrement partagé, **sans
  intervalle**, exactement celui que pose un `INSERT` à clé étrangère vers `bank_accounts(id)` (InnoDB : contrôle de clé
  étrangère sur l'index référencé, `LOCK_S | LOCK_REC_NOT_GAP`). C'est ce qui rend la demande **sans effet** quand le
  verrou est déjà tenu (W17, W22) — prémisse **raisonnée**, que T29a-6 et T29a-7 établissent (P-d (2)). Test T29a-1 :
  `EXPLAIN` de l'instruction rend `key = PRIMARY` et un accès `const` ; l'`EXPLAIN` exige des valeurs, le test les
  substitue en remplaçant chaque `?` de la constante, dans l'ordre, par l'identifiant d'un compte et d'une société de
  sa base (`id`, puis `company_id`), et vérifie avant de l'exécuter qu'il ne reste aucun `?`.
- **T-a45 de la 28-4, étendu** (`bank_accounts.rs:1521`, `mod tests`) : les deux SQL de garde restent sans verrou ; le
  nouveau **doit** contenir `LOCK IN SHARE MODE` et ni `FOR UPDATE` ni `FOR SHARE` (tue M29-3 et M29-19).
- Doc-comment de la fonction : la règle de position (critère, volet 1, avec la borne de l'audit), les deux usages
  (verrou déjà tenu / dernière acquisition), l'interdit d'y lire `archived` (et l'angle mort qui en découle pour W17 /
  W22, C-29-4-16), l'interdit de la poser avant une écriture qui prend un autre verrou, sa dépendance à
  `innodb_snapshot_isolation = OFF` et l'épinglage qui la tient (AC11), le résidu C-29-4-7, et la raison du choix
  (C-29-4-1, C-29-4-2).

**AC2 — L'import (W22) et la création de lot (W17) : la revérification après l'`INSERT` à clé étrangère.**

- **W22** — `bank_imports::create_with_transactions` (`bank_imports.rs:67`) : appel de `recheck_open_locked_in_tx(tx,
  new.company_id, new.bank_account_id)` **immédiatement après** l'étape 1 (`INSERT INTO bank_imports`, l. 82-104, et ses
  contrôles de `last_insert_id`, l. 106-113), **avant** l'étape 2 (`INSERT INTO bank_transactions`). La garde non
  verrouillante de tête (l. 79, `ensure_open_unarchived_in_tx`) **reste** : refus précoce, archivé compris.
- **W17** — `payment_batches::create_batch` (`payment_batches.rs:69`) : appel **immédiatement après** l'`INSERT INTO
  payment_batches` (l. 142-154), avant l'`UPDATE` du `msg_id` (l. 159) et les `INSERT` d'articles. La lecture de
  l'étape (0) (l. 85-108, `closed_on` et `archived`) **reste**.
- **Pourquoi c'est sans verrou neuf** : l'`INSERT` d'une ligne dont `bank_account_id` référence `bank_accounts(id)` a
  posé, et tient jusqu'au commit, un verrou partagé sur la ligne du compte (existence : T-a31 de la 28-4 le vérifie en
  pratique). La revérification redemande ce verrou-là : InnoDB le trouve tenu, n'en crée pas et n'attend pas — y compris
  quand une clôture attend déjà en exclusif (**raisonné**, P-d (2), établi par T29a-6 et T29a-7). Elle lit, comme toute
  lecture verrouillante sous `innodb_snapshot_isolation = OFF` (P-f, AC11), **le dernier état commité**.
- **Pourquoi cela ferme la fenêtre** : (i) si la clôture tient déjà la ligne en exclusif, l'`INSERT` l'attend ; au
  commit de la clôture, il passe, et la revérification lit `closed_on` posé → `DbError::BankAccountClosed` → la route
  rend `409 BANK_ACCOUNT_CLOSED` (import : `import_create_error`, `routes/bank_imports.rs` ; lot : `?` existant) et la
  transaction est abandonnée, **rien d'écrit** ; (ii) si l'écrivain a inséré d'abord, la clôture attend son commit,
  puis le voit (I1 : transactions `pending` ; I2 : lot `generated`) et refuse — c'est le sol (T-a31).
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
(4) `C.commit()` ; (5) l'écrivain doit rendre `DbError::BankAccountClosed` ; sa tâche fait alors `tx.rollback().await`
**explicite** avant de rendre son résultat (l'abandon par `drop` est asynchrone, `routes/bank_imports.rs:1464-1467`), et
seulement ensuite le test vérifie qu'aucune ligne `bank_imports` / `bank_transactions` (resp. `payment_batches` /
`payment_batch_items`) n'existe. **Rouge au sol** : l'écrivain aboutit. Tests **T29a-2** (W22) et **T29a-3** (W17) ;
tuent M29-1 et M29-2 (et M29-3 : la lecture redeviendrait celle de la vue). ⚠️ Au sol, l'étape (3) passe aussi
(l'`INSERT` attend déjà la clôture) : le test rougit à l'étape (5), pour la bonne raison.

**L'autre ordre — l'écrivain a inséré, la clôture attend, puis l'écrivain revérifie** (T29a-6 pour W22, T29a-7 pour W17 ;
R5 = F2 de la P1). C'est le seul ordre qui éprouve la prémisse P-d (2), et AC3 ci-dessus ne le monte pas (à l'étape (4)
la clôture a déjà commité, plus d'exclusif en file). Patron de T-a31 de la 28-4, prolongé : (1) l'écrivain, dans sa
transaction, exécute son `INSERT` d'entête (`INSERT INTO bank_imports` — resp. `INSERT INTO payment_batches`) **sans** la
revérification — par le SQL de l'`INSERT` ou par une fonction de fixture qui s'arrête là ; (2) la clôture démarre en tâche
(`close_in_tx`, **sans** enveloppe de rejeu, commit si `Done`) ; (3) la sonde la voit en attente (`["FROM bank_accounts",
"archived = FALSE FOR UPDATE"]`) ; puis un **délai** (500 ms) pour laisser la demande exclusive s'inscrire dans la file de
la ligne — la sonde voit une requête en cours, non une requête bloquée (P5 de la 28-4) ; (4) l'écrivain appelle
`recheck_open_locked_in_tx` sous `tokio::time::timeout(Duration::from_secs(2), …)` : il **doit** rendre `Ok(())` dans le
délai ; (5) l'écrivain commite ; (6) la clôture **doit** rendre
`Refused(PendingTransactions { pending_count: 1 })` (W22, une transaction insérée par l'écrivain avant le commit) — resp.
`Refused(OpenPaymentBatches { batch_count: 1 })` (W17). **Ce qui rougit si la prémisse est fausse** : si la redemande se mettait en file
derrière l'exclusif, InnoDB verrait le cycle écrivain / clôture et en tuerait un — la clôture, plus légère : elle rendrait
alors `Err(DbError::Sqlx(1213))` à l'étape (6) au lieu du refus (le test n'enveloppe pas `close_in_tx` dans un rejeu,
précisément pour que l'interblocage se voie) ; si l'écrivain était la victime, l'étape (4) rendrait l'erreur. Si
T29a-6 ou T29a-7 rougit pour cette raison, la conséquence s'écrit au Dev Agent Record (P-d (2) fausse : la fenêtre reste
fermée, mais « aucun verrou neuf » est à réécrire et chaque course coûte un rejeu de la clôture), la conception n'est
pas à refaire. Tue M29-19 (revérification en `FOR UPDATE` : la demande exclusive d'un détenteur partagé, derrière une
autre demande exclusive en file, attendrait — cycle).

**AC4 — Le rejeu de la clôture.**

- `close_bank_account` (`routes/bank_accounts.rs:1005`) : la section `begin` … `commit` passe dans une fonction « une
  tentative » (`close_once`), enveloppée par `crate::retry::retry_app_on_deadlock("bank_accounts::close", …)` — patron de
  `bank_imports::delete` → `delete_once` (`routes/bank_imports.rs:1461-1490`). Les contrôles de forme (version, borne de
  `closedOn`) et `assert_onboarding_complete` restent **devant** l'enveloppe. Le refus métier (`412`, `409`) n'est pas
  rejoué ; aucune conversion ne fait sortir un 1213 d'`AppError::Database(DbError::Sqlx(_))` avant le prédicat —
  aujourd'hui `close_in_tx` rend ses erreurs par `Err(e) => AppError::Database(e)`, à garder. **Sur refus**, `close_once`
  fait `tx.rollback().await` **explicite** avant de rendre l'erreur (patron de `delete_once`, `routes/bank_imports.rs:1477-1482` pour la règle, `:1549` pour le geste) :
  la transaction tient le `FOR UPDATE` du compte, et un abandon implicite ne le rendrait qu'au retour de la connexion au
  pool — autant d'attente en plus pour les écrivains partagés en file derrière la clôture.
- ⛔ **La réouverture n'est pas rejouée** (C-29-4-5, amendé en P1 sur F5) : aucun cycle réaliste ne peut la désigner
  victime — il faudrait un détenteur partagé de la ligne d'un compte **clos** en attente d'une ressource, or toute
  écriture sur un clos est refusée **avant** de prendre ce partagé (garde non verrouillante, ou lecture sous `FOR
  UPDATE`), et les gestes permis sur un clos (W21 `cancel_batch`, W30) ne touchent pas la ligne du compte. Une
  revérification de la 29-4b qui lit un compte clos tient son partagé jusqu'à son `rollback`, mais **n'attend rien** :
  elle ne peut fermer aucun cycle. `reopen_bank_account` reste tel quel, son doc-comment aussi.
- Doc-comment de la route `close_bank_account` (`routes/bank_accounts.rs:1000-1004`, F3 de la P1) : « Pas de rejeu sur
  interblocage : un seul verrou, en tenant rien, la route ne peut pas fermer de cycle » devient faux — réécrit : la
  route ne tient rien **comme détenteur**, mais sa demande exclusive en attente peut fermer un cycle à trois, dont elle
  est la victime désignée ; d'où le rejeu.
- **Pourquoi** : la clôture ne tient **rien** pendant qu'elle attend sa ligne ; sa demande exclusive **en attente** fait
  pourtant attendre derrière elle toute demande partagée ultérieure sur la même ligne (file InnoDB : une demande neuve
  attend un verrou incompatible **en attente** comme un verrou accordé — remarque R7 b de la validation P3 de la 28-4).
  Elle peut donc fermer un **cycle à trois** (Dev Notes § *Cycles*) ; elle y est la transaction la plus légère (aucune
  écriture, aucun verrou accordé), donc la **victime** désignée — prémisse que le dépôt tient déjà pour établie et sur
  laquelle reposent ses tests de rejeu (`rejeu_interblocage_e2e.rs:9-11`, « InnoDB choisit pour victime la
  transaction la plus légère »). La rejouer rend ce cas invisible à l'utilisateur.
- `audit_route_registry.rs`, point (vi) du doc-comment (l. 136) : « **sept** routes `SansEcritureAuJournal` sont
  rejouées » → **huit**, et `close_bank_account` nommée avec sa raison ; son classement (`Traced`,
  `SansEcritureAuJournal`, l. 339) ne change pas. Sept recomptées au sol par les deux lentilles de la P1
  (`onboarding::finalize`, `update_invoice_settings`, `create_fiscal_year`, `close_fiscal_year`, `onboarding::seed_demo`,
  `onboarding::reset`, `bank_imports::delete`) ; les recompter **à nouveau** sur la branche rebasée avant d'écrire huit.
- Doc-comment de `close_in_tx` (`bank_accounts.rs:987-1002`) : « il ne peut fermer aucun cycle » reste vrai **comme
  détenteur** ; ajouter qu'en attente il peut être la victime d'un cycle à trois, d'où le rejeu de la route.
- **Test T29a-4** (`rejeu_interblocage_e2e.rs`, patron du fichier : transactions de test alourdies de 500 lignes de lest,
  témoin `warn!` de `kesh_db::retry` capturé par `common::capture_rejeu`). ⛔ **Séquence réécrite en P1 (R1 = F1)** : la
  séquence d'origine attendait `T1` en ligne **avant** d'annuler `T2` ; or la victime (la clôture) ne tient rien, son
  annulation ne libère pas `R`, et `T2` — dont la demande partagée est accordée dès que l'exclusif en file disparaît —
  garde `R` : le test se serait bloqué sur lui-même jusqu'au `1205`. Le patron `victime()` (`rejeu_interblocage_e2e.rs:347-372`)
  ne s'applique pas tel quel : il suppose que la victime **est** la route qui tient la ressource demandée. Séquence
  réalisable :
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

**AC5 — Le registre statique : l'inventaire clos des gardes non verrouillantes.**

Test existant `crates/kesh-api/tests/bank_account_closed_inventory.rs` (31 sites d'écriture, 27 entrées au `REGISTRE`,
3 au `APPELANTS_REGLEMENT` — recomptés en P1, R7) : il gagne un **troisième inventaire**, tenu dans les deux sens (comme
les deux autres), test **T29a-5** :

- **extraction des appels** : tout appel `bank_accounts::recheck_open_locked_in_tx(` (ou `super::bank_accounts::…`)
  hors tests et hors commentaires dans `kesh-db/src` et `kesh-api/src`, rattaché à sa fonction englobante (analyseur
  partagé de C-28-4-29) ;
- **extraction des gardes non verrouillantes** (C-29-4-15, R3 de la P1 — depuis le symptôme, non depuis le texte des
  raisons du registre principal) : l'ensemble `G` des fonctions englobantes, hors tests et hors commentaires, dans
  `kesh-db/src` et `kesh-api/src`, qui (a) appellent `ensure_open_in_tx(` ou `ensure_open_unarchived_in_tx(` (copie
  masquée), ou (b) portent un **littéral** contenant `closed_on` et `FROM bank_accounts` et ni `FOR UPDATE` ni `LOCK IN
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
  - `Reverifiee("<position>")` — l'appel existe dans la fonction ; en 29-4a : W17 (`create_batch`), W22
    (`create_with_transactions`) ;
  - `SansFenetre("<invariant ou verrou>")` — W20 `confirm_batch` (« chaque facture par `pay_in_tx`, compte lu sous
    `FOR UPDATE` ; et I2 »), W30 `unreject_once` (en 29-4a : « I1, sous réserve de la 29-4b — W9 peut encore remettre une
    transaction `pending` sur un compte clos » ; en 29-4b : « I1 : la transaction est `pending`, la clôture refuse » —
    R12 b de la P1) ;
  - `Ouverte("29-4b")` — en 29-4a seulement : W9, W24, W25, W28, W29. ⛔ La 29-4b **retire** cette variante (plus
    aucun site), sans quoi elle serait morte sous `clippy -D warnings`.
  ⚠️ W1-W6 et `accept_one_*` **n'y figurent pas** (correction de la P1) : ce ne sont pas des gardes non verrouillantes
  en transaction mais des `Invariant("I1", …)` du registre principal, que la story ne touche pas.
- **assertions** : (i) l'ensemble des fonctions qui appellent la revérification **égale** l'ensemble des
  `Reverifiee` ; (ii) l'ensemble `G` **égale** l'ensemble des fonctions de `FENETRE` — égalité, dans les deux sens : une
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
  rougir (une garde non verrouillante sans entrée ; un appel avant l'`INSERT`), sans quoi un extracteur qui ne voit rien
  la ferait passer à vide.
- Les textes de raison du registre principal qui citent L1 (l. 89 — doc de la variante `SortieAutorisee` —, 115, 233,
  290) restent exacts en 29-4a ; la 29-4b les réécrit (AC9).
- Doc-comment du test : ce qu'il voit (présence, ensemble des gardes non verrouillantes, ordre textuel), ce qu'il ne
  voit pas — que la revérification s'exécute (affaire des tests d'entrelacement) ; une écriture par une fonction appelée
  **après** l'appel, que l'ordre textuel d'une seule fonction ne suit pas (d'où, pour les annulations, l'ordre exigé par
  rapport aux appels nommés, AC8) ; une lecture non verrouillante de `closed_on` dont le SQL ne porte pas littéralement
  `FROM bank_accounts` (requête assemblée, `format!`) ; les **pré-vols** de route sur le pool (`is_closed()`,
  `routes/bank_imports.rs:994`, `:1151`, `routes/reconciliation.rs:1051`, `:3262`, `:3661`, `:4110`, `:4934`,
  `routes/bank_accounts.rs:892`), hors transaction, que suit chacun une garde ou un invariant du registre principal.

**AC11 — La revérification ne dépend pas en silence d'`innodb_snapshot_isolation` (partie a ; R4 = F6 de la P1,
C-29-4-13).**

*Constat* (P-f) : sous `innodb_snapshot_isolation = ON` — défaut de MariaDB à partir de la **11.6** —, une lecture
verrouillante qui rencontre une version de ligne postérieure à la vue ouverte rend `ER_CHECKREAD` (1020). C'est la course
même que la revérification traite (la vue est née **avant** le commit de la clôture). Le dépôt ne traduit pas 1020 :
`map_db_error` le laisse en `DbError::Sqlx`, que `AppError` rend en `500 INTERNAL_ERROR` avec un `tracing::error!`
(`kesh-db/src/errors.rs:1459-1494`, `kesh-api/src/errors.rs:4114-4120`). L'écrivain resterait refusé, rien d'écrit — mais
par un `500` que rien ne distingue d'une panne. Et ce n'est pas propre à la story : `routes/reconciliation.rs:2166-2170`
et `bank_imports.rs:439-446` portent déjà la même réserve, sans traitement.

*Retenu* : **épingler `innodb_snapshot_isolation = OFF` sur chaque connexion du pool de production**, dans
`kesh-db/src/pool.rs`, `create_pool`, à côté du `SET SESSION sql_mode …, time_zone …` existant (`pool.rs:36-45`) :

- la configuration de session sort dans une fonction `pub async fn configurer_session(conn: &mut MySqlConnection) ->
  Result<(), sqlx::Error>` (appelée par `after_connect`), pour être testable ;
- elle exécute `SET SESSION innodb_snapshot_isolation = OFF` **dans une instruction à part** ; si le serveur ne connaît
  pas la variable — erreur **1193** (`ER_UNKNOWN_SYSTEM_VARIABLE`) —, l'erreur est ignorée avec un `tracing::debug!` :
  un serveur qui ne connaît pas la variable se comporte comme `OFF`. Toute autre erreur remonte (la connexion échoue,
  visiblement). ⚠️ **Raisonné, non vérifié** : de mémoire de la documentation MariaDB, la variable existe depuis les
  versions correctives de 2024 des branches 10.6, 10.11 et 11.x (10.11.8 pour la 10.11), défaut `OFF`, et le défaut passe
  à `ON` en 11.6 ; **constaté** : `10.11.16` la connaît, à `OFF` (base de dev, 2026-10-14). Le chemin 1193 n'est exercé
  par aucun test du dépôt (aucun serveur antérieur disponible) — angle mort écrit au doc-comment.
- doc-comment de `create_pool` : la liste des réglages de session gagne celui-ci, avec sa raison (les lectures
  verrouillantes du dépôt lisent le dernier état commité **par conception** : revérification de la 29-4, relecture
  verrouillante du rejet, `routes/reconciliation.rs:2166`), et ce qu'il ne couvre **pas** : les pools des tests
  `#[sqlx::test]`, qui ne passent pas par `create_pool` — sur un serveur 11.6+, les tests verraient `ON`, et c'est voulu :
  `reconciliation_e2e.rs:7556` et `:8401` sont écrits pour signaler la montée de version, et T29a-2 / T29a-3 le feront
  aussi.
- doc-comments qui disent « à revoir à la montée de version » mis à jour (doc-comments seuls, 29-4a) :
  `bank_imports.rs:439-446` et `routes/reconciliation.rs:2166-2170` — « épinglé à `OFF` par `create_pool` (Story 29-4) ;
  sans cet épinglage, … ». ⚠️ La phrase de `bank_imports.rs:444-445`, « sans effet sur le code nominal (aucune vue n'est
  ouverte avant les verrous) », deviendra fausse en 29-4b pour W24 (la garde de la 28-4 ouvre la vue avant la
  revérification) : la 29-4b la réécrit (AC9).

*Tests* — **T29a-8** (`kesh-db/tests/bank_accounts_repository.rs`, module `compte_clos`), sur la base éphémère du test :

- (a) **la prémisse, observée** : sur une connexion où `SET SESSION innodb_snapshot_isolation = ON` a été exécuté
  (possible sur 10.11.16 : la variable existe), monter la course de T29a-2 (lecture non verrouillante, clôture commitée,
  puis `INSERT` d'entête et revérification) : la revérification — ou l'`INSERT`, dont le contrôle de clé étrangère est
  aussi une lecture verrouillante — rend `1020`. Le test **constate** lequel des deux, et l'écrit au Dev Agent Record :
  c'est ce qui dit si l'épinglage était nécessaire pour l'`INSERT` aussi (non établi à la fiche) ;
- (b) **l'épinglage** : sur une connexion passée en `ON`, `configurer_session(&mut conn)` puis
  `SELECT @@session.innodb_snapshot_isolation` rend `0` ; puis la même course rend `DbError::BankAccountClosed`.
  Tue **M29-5** (le `SET … innodb_snapshot_isolation` retiré de `configurer_session`).
- ⛔ Aucune traduction de 1020 n'est ajoutée à `map_db_error` : avec l'épinglage, 1020 ne peut plus naître de la course ;
  s'il naissait d'ailleurs, `500` reste le bon rendu d'un état que le code ne prévoit pas.

*Fait à constater, hors du dépôt* : la version de MariaDB du NAS (paquet de Synology) et son réglage. Avec l'épinglage,
la story est juste quelle que soit la réponse ; la réponse dit seulement si le NAS **aurait** rendu des `500` sans lui.
Question posée à Guy par l'orchestrateur (rapport de la remédiation P1).

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

- **La garde non verrouillante de la 28-4 reste à sa place** à chaque site (refus précoce, ordre des refus de la 28-4 :
  W24 « `HasReconciled` d'abord », motifs d'exercice clos après la garde, etc.). La revérification ne répond que si une
  clôture a été commitée après la naissance de la vue de l'annulation.
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
  `Done` — et le règlement est absent : la clôture a bien attendu (tue M29-13 : revérification non verrouillante, qui ne
  fait pas attendre la clôture — et qui ne refuserait pas l'annulation dans le premier test, M29-8 à M29-12 le tuant
  aussi).
- **Les tests qui gardent la naissance de la vue restent verts, sans retouche** : T-a48 (a)-(c) de la 28-4
  (`invoice_settlement.rs`, `supplier_invoices_repository.rs`) — ils exigent que les gardes non verrouillantes de la
  28-4 restent après le verrou d'exercice (volet 3 du critère). *(Correction de la P1, F9 : la fiche rangeait ici
  `a_concurrent_close_waits_for_the_unreconciliation`, `crates/kesh-db/tests/reconciliation_cancel.rs:128` ; ce test porte
  sur la clôture d'**exercice** — il garde le verrou d'exercice de l'étape (3) de W9, non la garde du compte, et refuse
  avant la revérification. Il reste vert sans retouche, pour cette raison-là.)* M29-14 (déplacer une revérification
  **avant** le verrou d'exercice) : tuée par l'ordre textuel de l'AC8 ; elle ne déplace pas la vue (lecture
  verrouillante), et les tests de vue ne la voient pas — dit tel.

**AC8 — Le registre : `Ouverte` disparaît, l'ordre textuel s'étend aux annulations, et le cycle neuf se prouve.**

- Registre `FENETRE` de l'AC5 : W9, W24, W25, W28, W29 passent en `Reverifiee` ; la variante `Ouverte` est retirée ;
  les assertions (i) et (ii) restent.
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
  retire l'import et la création de lot de ses chemins et renvoie à la revérification ; `CHANGELOG.md`, section
  **`## [0.13.4] — Non publié`** (créée, ou complétée **par union** si la 29-2 ou la 29-3 l'ont ouverte), `### Corrigé` :
  « **Un import ou un lot de paiement commencé juste avant la clôture d'un compte bancaire ne passe plus sur ce
  compte** ([#651](https://github.com/guycorbaz/kesh/issues/651)) : il est refusé (« Ce compte bancaire est clos »), et
  rien n'est enregistré. » S'y ajoutent, en 29-4a, les doc-comments de l'AC4 (route `close_bank_account`, `close_in_tx`)
  et de l'AC11 (`create_pool`, `bank_imports.rs:439-446`, `routes/reconciliation.rs:2166-2170`).
  ⚠️ **État transitoire assumé** (R12 a de la P1) : entre le merge de la 29-4a et celui de la 29-4b, la puce du manuel
  (« pour un import dès l'envoi du fichier — peut encore aboutir »), `docs/api-external.md:241` et le § L1
  d'`ensure_open_in_tx` restent écrits pour les sept chemins, donc **trop larges** pour l'import et le lot. C'est la
  règle d'inclusion du `CLAUDE.md` (« batched dans la PR de la dernière story » quand plusieurs stories y contribuent) ;
  aucune release ne tombe entre les deux PR (Status). À écrire tel au Dev Agent Record de la 29-4a.
- **29-4b** :
  - § *Angle mort assumé — L1* d'`ensure_open_in_tx` **remplacé** par un § « *La fenêtre de la clôture — fermée par la
    revérification (Story 29-4, #651)* » : pourquoi la garde non verrouillante seule laissait passer, la revérification
    et sa position, les deux chemins sans fenêtre par invariant (W30, W1-W6) ; le renvoi à #651 reste, comme histoire.
    Le doc-comment d'`ensure_open_unarchived_in_tx` (l. 863-865, « angle mort L1 (#651) ») suit, ainsi que le
    doc-comment de module (`bank_accounts.rs:28`, « … les trois invariants et l'angle mort L1 » → « … et la fenêtre de la
    clôture, fermée par la revérification » — R8 de la P1).
  - `bank_imports.rs:444-445` (AC11) : « sans effet sur le code nominal (aucune vue n'est ouverte avant les verrous) »
    devient faux pour W24, dont la garde de la 28-4 ouvre la vue avant la revérification — réécrit : la revérification
    de W24 s'exécute vue ouverte, et c'est l'épinglage de `create_pool` qui la tient (R4 de la P1).
  - Registre (`bank_account_closed_inventory.rs`) : les raisons des l. 115, 233, 290 — « sous L1 (#651), le pré-vol
    R1 refuse » devient « I1 ; et le pré-vol R1 refuse un compte clos » ; W21 : « annuler un lot `generated` laissé sur
    un compte clos par la fenêtre de la v0.13.3 (#651, fermée en v0.13.4) » ; **l. 89**, doc de la variante
    `SortieAutorisee` (« …réparer ce que l'angle mort L1 (#651) a laissé passer ») → « …réparer ce que la fenêtre de la
    clôture (#651, fermée en v0.13.4) a pu laisser passer jusqu'à la v0.13.3 » (R8 de la P1 : l. 89 n'est pas une raison
    mais un doc-comment, qui n'avait pas de texte prescrit) ; la raison de W30 à `FENETRE` perd sa réserve « sous réserve
    de la 29-4b » (AC5).
  - `routes/reconciliation.rs:4908` (doc-comment du pré-vol R8), `bank_accounts_closed_e2e.rs:18` (en-tête) et
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
  - `CHANGELOG.md`, `## [0.13.4] — Non publié`, `### Corrigé` : la ligne de la 29-4a **complétée** (par union) : « …
    ni une annulation (rapprochement, règlement client ou fournisseur, facture fournisseur payée), ni la suppression
    d'un import vide : l'opération est refusée, ou se termine avant la clôture, qui en tient compte. La remarque de la
    v0.13.3 (« une opération déjà en cours au moment de la clôture peut encore aboutir ») ne vaut plus. » *(R11 de la
    P1 : la suppression d'un import n'est pas une « annulation » — le bouton s'appelle « Supprimer l'import ».)*
  - ⛔ **Ne se réécrivent pas** : la section `## [0.13.3] — 2026-10-14` du CHANGELOG (relevé daté d'une release
    publiée, l. 37) ; la fiche et le registre de la 28-4 (relevés de leurs passes) ; l'issue #651 (l'orchestrateur la
    commente).
- **Propagation** (§ *Propagation post-patch*, greper la **valeur**) : `grep -rn "651\|\bL1\b" crates docs CHANGELOG.md
  frontend/src website README.md` à chaque PR, chaque site trié (réécrit, ou laissé avec sa raison) au Dev Agent Record.

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
- [ ] **T1** — `RECHECK_OPEN_LOCKED_SQL` (`#[doc(hidden)] pub`), `recheck_open_locked_in_tx`, doc-comment ; T-a45 de la
  28-4 étendu ; T29a-1 (`EXPLAIN`) (AC1).
- [ ] **T1-bis** — `configurer_session` et l'épinglage d'`innodb_snapshot_isolation` ; T29a-8 (a) **d'abord** (constat
  de la prémisse, écrit au Dev Agent Record), puis (b) ; doc-comments de `create_pool`, `bank_imports.rs:439-446`,
  `routes/reconciliation.rs:2166-2170` (AC11).
- [ ] **T2** — Tests de fenêtre T29a-2 (W22) et T29a-3 (W17) **d'abord** (rouges au sol, journal du rouge au Dev Agent
  Record), puis l'appel après chaque `INSERT` (AC2, AC3) ; puis T29a-6 et T29a-7 (l'autre ordre, prémisse P-d (2)) —
  leur verdict, vert ou rouge, s'écrit au Dev Agent Record et met à jour le libellé de P-d.
- [ ] **T3** — `close_once` et son enveloppe (rollback explicite sur refus) ; doc-comment de la route
  `close_bank_account` ; point (vi) d'`audit_route_registry.rs` recompté (sept → huit) ; doc-comment de `close_in_tx` ;
  T29a-4 (AC4).
- [ ] **T4** — Troisième inventaire du registre (T29a-5) : extraction `G`, `FENETRE` à neuf entrées, statut
  `Ouverte("29-4b")` pour les cinq annulations, ordre textuel par positions de littéraux, volet inverse (AC5).
- [ ] **T5** — Doc-comments et CHANGELOG de la partie a (AC9) ; propagation greppée.
- [ ] **T6** — Mutations M29-1 à M29-7 et M29-19 ; table *Position par chemin* recopiée (lignes W17, W22, clôture) ; gate
  complet + E2E complet au dernier commit de code ; Dev Agent Record (« aucune release avant la 29-4b », état
  transitoire des textes) ; corps de PR **demandé** à l'orchestrateur : `refs #651` — ⛔ ni `closes` ni `fixes` dans
  aucun message de commit, même en prose.

### 29-4b

- [ ] **T7** — Partir d'`origin/main` après la fusion de la 29-4a ; relire les numéros de ligne de l'AC6.
- [ ] **T8** — Tests de fenêtre des cinq annulations (T29b-1) **d'abord** (rouges au sol), puis les cinq appels (AC6,
  AC7) ; les deux tests de l'autre entrelacement (T29b-2).
- [ ] **T9** — Registre (T29b-4) : `Reverifiee` pour les cinq, `Ouverte` retiré, raison de W30 sans réserve, ordre
  textuel étendu par positions de littéraux (AC8) ; T29b-3 (cycle neuf).
- [ ] **T10** — Textes de L1 (doc-comments dont `bank_accounts.rs:28` et `bank_imports.rs:444-445`, registre dont
  l. 89, pré-vol R8, `bank_accounts_closed_e2e.rs:18` et `:1577`), manuel et PDF contrôlé aplati, `api-external.md`,
  CHANGELOG complété (AC9) ; propagation greppée.
- [ ] **T11** — Mutations M29-8 à M29-18 ; table *Position par chemin* et table des cycles recopiées ; gate complet + E2E
  complet au dernier commit de code ; corps de PR **demandé** : `closes #651` (sur la PR seulement).

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
| (b) rejeu sur interblocage des deux côtés | **non**, seul : il n'y a pas de cycle tant qu'il n'y a pas de verrou commun | neutralise ceux que le verrou crée | — | faible par route | **retenue en complément** : la clôture (AC4 ; la réouverture, sans cycle réaliste, ne l'est pas — amendement P1) ; les annulations et W8 / W27 le sont déjà |
| (c) revérification sous verrou au moment de l'écriture finale | **oui** | W17, W22 : aucun ; annulations : un cycle avec W8 / W27 (rejoués des deux côtés) ; cycles à trois par la clôture (rejouée) | aucun déplacement | 1 fonction, 7 appels, 1 enveloppe, l'épinglage de session (AC11), tests | **retenue** |
| (d1) verrou nommé `GET_LOCK` du compte, pris aussi par la clôture | non : `with_account_lock` relâche le verrou **avant** le commit de l'appelant (`kesh-reconciliation/src/mutex.rs:131-136`, `RELEASE_LOCK` dans la transaction, avant que l'appelant commite) ; et un cycle verrou nommé / verrou de ligne échappe à la détection d'InnoDB (attente jusqu'au délai, `1205`, non rejoué) | — | — | changer la sémantique de toutes les routes de réconciliation | **écartée** |
| (d2) la clôture verrouille les lignes enfants du compte (transactions, règlements, factures fournisseurs) | oui | ensemble de verrous large (index `bank_account_id`, intervalles), ordres de parcours à prouver contre la suppression d'import, l'import, `accept_one_invoice`, `pay_in_tx` — exactement l'analyse que la 28-4 a dû abandonner | — | élevé | **écartée** |
| (d3) refuser la clôture tant qu'une écriture est en cours, ou réduire et documenter | — | — | — | — | **inutile** : (c) ferme la fenêtre ; la sortie n'est pas nécessaire |

### Position par chemin — la preuve du critère (à recopier au Dev Agent Record avec la ligne retenue)

« Vue » = première lecture non verrouillante de la transaction, au sol puis avec la story.

| Chemin | Verrous au sol, dans l'ordre (28-4, table *Position par chemin*, relue sur `16eeed21`) | Ce que la story ajoute | Verrou neuf ? | Position | Vue : au sol → avec la story | Vue déplacée ? |
|---|---|---|---|---|---|---|
| W22 import (CAMT, CSV) | [doublons sans verrou, route] → `INSERT bank_imports` (compte S, `companies` S, `users` S) → `INSERT bank_transactions` → relecture → audit | revérification après `INSERT bank_imports` | **non** (compte S déjà tenu) | après l'`INSERT` d'entête | `find_by_company_and_hash` (route) → inchangée | non |
| W17-W19 `create_batch` | compte sans verrou (0) → factures fournisseur X → `INSERT payment_batches` (compte S, `companies` S) → `UPDATE` → `INSERT` articles → audit | revérification après `INSERT payment_batches` | **non** | après l'`INSERT` du lot | étape (0) → inchangée | non |
| W9 `cancel_in_tx` | [verrou nommé, hors InnoDB] → transaction X (1) → règlement ou intervalle de `uq_invoice_settlements_entry`, facture X (2) → écriture et exercice X (3) → [garde 28-4, vue] → `UPDATE bank_transactions` (5) → (6) W25 imbriqué (… `companies` S) ou `reverse_in_tx` (`companies` S) → audit | revérification après (6) | **oui** : compte S | dernière acquisition | garde 28-4 entre (3) et (4) → inchangée | non |
| W25 `cancel_settlement_in_tx` | facture X (1) → règlement X (2) → écriture et exercice X (2-bis) → [garde 28-4] → lettrage → `reverse_owned_in_tx` (`companies` S) → `DELETE` règlement → `UPDATE invoices` → audit | revérification après (6) | **oui** : compte S | dernière acquisition | garde 28-4 après (2-bis) → inchangée | non |
| W28 `supplier_invoices::cancel_in_tx` | facture X → écriture d'achat et exercice X → [garde 28-4] → lettrage → contre-passation (`companies` S) → `UPDATE` → audit | revérification après (4) | **oui** | dernière acquisition | garde 28-4 → inchangée | non |
| W29 `supplier_invoices::cancel_settlement_in_tx` | facture X → (1-bis) écriture de règlement et exercice X → [garde 28-4] → lettrage → contre-passation (`companies` S) → `UPDATE` → audit | revérification après (4) | **oui** | dernière acquisition | garde 28-4 → inchangée | non |
| W24 `delete_unreconciled_in_tx` | entête X → transactions de l'import X et intervalle (`LOCK_IMPORT_TRANSACTIONS_SQL`) → [comptage] → [garde 28-4, vue] → `DELETE` (cascade) → audit (route) | revérification après le `DELETE` | **oui** | dernière acquisition | garde 28-4 → inchangée | non |
| clôture | compte X (première instruction) → décomptes sans verrou → `UPDATE` → audit | enveloppe de rejeu | non | — | premier décompte → inchangée | non |
| réouverture | compte X → `UPDATE` → audit | rien (amendement P1) | non | — | inchangée | non |
| W30, W1-W6, `accept_one_*`, W20, W8, W27, W13-W15, W21, W26, gestion | inchangés | rien | non | — | inchangée | non |

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
dit donc « un exclusif sur des lignes de comptes, dont `A` » pour ce détenteur.

| Partenaire (exclusif sur `A`) | Ce qu'il demande **après** `A` | L'annulation le tient-elle ? | Cycle | Rejoué des deux côtés |
|---|---|---|---|---|
| W8 `settle_invoice` (virement vers `A`) | [comptes], **exercice** X, `companies` S, `INSERT invoice_settlements` (intervalle de `uq_invoice_settlements_entry`) | **oui** : l'exercice (W9 (3), W25 (2-bis), W28, W29) ; l'intervalle de `uq_invoice_settlements_entry` (W9 « écriture », F13 de la P2 de la 28-4) | **oui** — le cycle F1 de la P1 de la 28-4, et F13 | **oui** : `invoices::settle` ; `reconciliation::cancel`, `invoices::cancel_settlement`, `supplier_invoices::cancel`, `…::cancel_settlement` |
| W27 `pay_in_tx` (par `pay` ou `confirm_batch`) | [comptes X, `supplier_invoices.rs:808`], **exercice** X, `companies` S | oui : l'exercice | **oui** | **oui** : `supplier_invoices::pay`, `payment_batches::confirm` |
| W13 `PATCH` | `accounts` en lecture **non verrouillante** (`ensure_postable_if_active_in_tx`), puis l'`UPDATE` (clé étrangère `journal_account_id` → `accounts` S) | non **à deux** : aucune annulation ne tient `accounts` en exclusif (les seuls exclusifs sur `accounts` sont dans `company_invoice_settings` — réglages, TVA, défauts —, `invoice_settlements_write.rs:181` (règlement, compte interne), `supplier_invoices.rs:433` (création) et `:808` (paiement) ; aucun dans la chaîne de contre-passation, `journal_entries.rs` n'en porte aucun ; exclusifs implicites recherchés aussi — `(UPDATE|DELETE FROM|INSERT INTO) accounts` : seuls `accounts.rs`, routes de gestion des comptes, et `credit_notes.rs:479`, hors chaîne — lentille F de la P1). **À trois**, voir l'angle mort « PATCH » ci-dessous | non à deux ; à trois, sans cas réaliste (angle mort) | — (non rejouée) |
| W14 `PUT`, `POST /bank-accounts`, W15 archivage | `companies` X **avant** `A` ; après `A` : `accounts` S, audit | l'annulation (W9, W25, W28, W29) tient `companies` S **avant** de demander `A` : la route de gestion l'attend sans rien tenir ; W24 ne prend pas `companies`, et la route ne demande rien qu'il tienne | non | — (non rejouées, et n'en ont pas besoin ici) |
| `upsert_primary_in_tx` (onboarding) | — | — | non (et `onboarding::finalize` est rejouée) | — |
| clôture | **rien** (décomptes non verrouillants, `UPDATE` de la même ligne, audit sans clé étrangère) | — | non à deux ; **à trois**, par la file d'attente (AC4) | la story la fait rejouer (AC4) |
| réouverture | **rien** | — | non à deux ; à trois, aucun détenteur partagé réaliste d'un compte **clos** (AC4, amendement P1) | non rejouée |

**Cycles à trois par la file d'attente** : la clôture attend `A` (exclusif) derrière un détenteur partagé `H` ; un
demandeur partagé `R` attend derrière la clôture ; si `H` attend une ressource que `R` tient, le cycle est
`R → clôture → H → R`. La clôture n'y tient rien et n'a rien écrit : victime désignée (AC4), rejouée. (Prémisses
« file » et « victime la plus légère » : raisonnées pour ce cas ; la seconde est vérifiée en pratique pour les couples
« transaction alourdie / route » des tests 2 à 10 de `rejeu_interblocage_e2e.rs` ; T29a-4 éprouve les deux pour la
clôture.) **Résidu** (C-29-4-7, **réécrit en P1**, F8 b) : on avait écrit qu'un cycle à trois où l'import (W22, `H` :
partagé depuis son `INSERT`, en attente de l'intervalle de `idx_bank_transactions_import` tenu par une suppression
d'import W24 = `R`) serait désigné victime laisserait un `500` sur la route d'import, non rejouée, « si la transaction la
plus légère n'était pas désignée ». C'était **surestimé** : dans ce cycle, la clôture a un poids d'environ 1 (une demande
en attente, rien de modifié), l'import et la suppression ont écrit ; InnoDB désigne la plus légère, la clôture n'est
dépassée qu'à égalité de poids, qui ne peut survenir ici. Le résidu est **théorique** : écrit comme tel au doc-comment de
`recheck_open_locked_in_tx`, sans « à surveiller » ; aucune issue.

**Angle mort « PATCH », cycle à trois par `accounts`** (F4 de la P1, raisonné) : une annulation par virement (W25) tient l'exercice ; un règlement interne W8 tient un exclusif
sur un compte de caisse `Z'` et attend l'exercice ; un `PATCH` du compte `A` vers `Z'` tient `A` en exclusif et demande
`Z'` en partagé derrière W8 ; l'annulation, en dernière acquisition, demande `A` : cycle annulation → PATCH → W8 →
annulation, dont le `PATCH` (non rejoué) pourrait être la victime → `500`. Il exige au même instant un lien de `A` vers
un compte de caisse, un règlement interne sur ce compte et une annulation par virement dans le même exercice :
**négligeable**, écrit, sans issue. La phrase « aucun partenaire … sans être rejoué » de l'AC6 vaut **à deux** ; à trois,
à cet angle mort près.

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
- `pool.rs` : le `SET SESSION sql_mode …, time_zone …` existant, inchangé ; l'épinglage s'y ajoute en instruction à part
  (AC11).

### Fichiers communs avec les autres stories de l'epic

| Fichier | Stories |
|---|---|
| `CHANGELOG.md` `[0.13.4]` | 29-2, 29-3 (union) |
| `docs/manual/fr/user-manual.tex` / `.pdf` | 29-3 (« la meilleure »), 29-2 si texte |
| `docs/api-external.md` | 29-2, 29-3 possibles |
| `crates/kesh-api/src/routes/reconciliation.rs` | 29-2, 29-3 — la 29-4 n'y touche **que deux doc-comments** (l. 2166-2170 en 29-4a, l. 4908 en 29-4b) |
| `crates/kesh-db/src/pool.rs` | — (à vérifier au rebase) |
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

**Partie a** : T29a-1 `EXPLAIN` de la revérification (PRIMARY, `const`) ; T-a45 (28-4) étendu ; T29a-2 fenêtre W22 ;
T29a-3 fenêtre W17 ; T29a-4 cycle à trois par la clôture ; T29a-5 registre (troisième inventaire, deux sens, ensemble
`G`, ordre textuel) et son volet inverse par un cas construit ; T29a-6 et T29a-7 l'autre ordre (écrivain inséré, clôture
en file, revérification sans attente — prémisse P-d (2)) ; T29a-8 `innodb_snapshot_isolation` (constat de 1020 sous
`ON`, épinglage).

**Partie b** : T29b-1 fenêtres W9, W24 (import vide), W25, W28, W29 (cinq) ; T29b-2 autre entrelacement W9 (412) et W25
(`Done`, règlement absent) ; T29b-3 cycle neuf annulation / exclusif du compte, rejoué ; T29b-4 registre (`Ouverte`
retiré, ordre textuel des annulations). Restent verts **sans retouche** : T-a31, T-a48 (a)-(c),
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
| M29-5 | `SET SESSION innodb_snapshot_isolation = OFF` retiré de `configurer_session` (remplace l'ancienne M5, enveloppe de la réouverture, sans objet depuis la P1) | T29a-8 (b) |
| M29-6 | W22 : appel déplacé avant l'`INSERT INTO bank_imports` | T29a-5 (ordre textuel) — fonctionnellement équivalente, dite telle |
| M29-7 | W17 : appel déplacé avant l'`INSERT INTO payment_batches` | T29a-5 — idem |
| M29-8 à M29-12 | W9, W24, W25, W28, W29 : appel retiré | T29b-1 (un test chacune) |
| M29-13 | revérification non verrouillante (M29-3 jouée en 29-4b) | T29b-2 |
| M29-14 | W25 : appel déplacé juste après (2-bis) | T29b-4 (ordre textuel) — fonctionnellement équivalente, dite telle |
| M29-15 | W9 : appel déplacé avant l'étape (6) | T29b-4 |
| M29-16 | W24 : appel déplacé avant le `DELETE FROM bank_imports` (ajoutée en P1, R2) | T29b-4 (position du littéral) |
| M29-17 | W28 : appel déplacé entre `reverse_owned_in_tx(` et l'`UPDATE supplier_invoices` (ajoutée en P1, R2) | T29b-4 (position du littéral) |
| M29-18 | W29 : même déplacement (ajoutée en P1, R2) | T29b-4 |
| M29-19 | `RECHECK_OPEN_LOCKED_SQL` en `FOR UPDATE` au lieu de `LOCK IN SHARE MODE` (ajoutée en P1, R5 = F2) | T-a45 (28-4, étendu) ; T29a-6 / T29a-7 (cycle écrivain / clôture) |

### Angles morts assumés (à écrire aux doc-comments)

- Le cycle à trois où l'import serait désigné victime (C-29-4-7) — **théorique** depuis la P1 : la clôture y est
  toujours la plus légère.
- Le cycle à trois par `accounts` où le `PATCH`, non rejoué, serait victime (F4 de la P1) — négligeable.
- L'ordre textuel du registre ne suit pas un geste verrouillant appelé sous un nom absent de sa liste de jetons, ni un
  SQL assemblé hors littéral (AC8) ; l'ensemble `G` ne voit pas une lecture de `closed_on` dont le SQL ne porte pas
  littéralement `FROM bank_accounts`, ni les pré-vols de route sur le pool (AC5).
- La question F11 de la 28-4 (compte du règlement ≠ compte de la transaction dans W9) reste non établie ; la position
  « dernière acquisition » la rend sans effet sur les cycles (AC6).
- La revérification ne lit pas `archived` : un **archivage** commité entre la garde de tête de W22 / W17 et leur
  `INSERT` laisse passer l'import ou le lot sur un compte archivé — la course « archivage / import » que le dépôt nomme
  déjà L5 (`bank_accounts.rs:866-867`), étendue au lot. Hors #651, non mécanique à fermer (deux traductions de
  l'archivé) ; **issue à ouvrir** (C-29-4-16, F8 a de la P1).
- `configurer_session` ignore l'erreur 1193 (variable inconnue d'un serveur ancien) : chemin non exercé par aucun test
  (AC11).
- Les états laissés par la fenêtre de la v0.13.3 ne sont pas réparés (table *Dehors*).

### Pièges connus du dépôt, applicables ici

- `LOCK IN SHARE MODE`, pas `FOR SHARE` (MariaDB 10.11).
- `attendre_une_requete_en_cours` voit une requête **en cours**, non une requête **bloquée** (leçon de la validation P5
  de la 28-4, T-a49) : chaque test d'entrelacement doit établir l'attente par son **issue** (le résultat après le commit
  de `C`), pas par la seule sonde.
- Tests de rejeu : alourdir la transaction de test (500 lignes de lest), exiger qu'elle obtienne son verrou (`.expect`),
  capturer le `warn!` de rejeu (`rejeu_interblocage_e2e.rs:5-23`).
- Mutations : `git checkout -- <fichier>` **puis `touch`** ; base de gate remise à zéro avant tout gate complet.
- Registre de sprint et registre des choix : par **union**.
- Données réelles : fixtures synthétiques (IBAN de test) ; rien du relevé de Guy.

### Project Structure Notes

Aucun fichier neuf de production (l'épinglage vit dans `pool.rs`, existant) ; tests ajoutés aux fichiers existants (`bank_accounts_repository.rs`,
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
  `crates/kesh-db/src/pool.rs:28-49` ; `crates/kesh-db/src/errors.rs:1459-1494` (`map_db_error`) ;
  `crates/kesh-api/src/errors.rs:4114-4120` (rendu de `DbError::Sqlx`) ; `crates/kesh-api/tests/common/capture_rejeu.rs`.
- Rapports de validation de la 29-4 : `/home/gcorbaz/devel/kesh-gate-logs/29-4-validate-p1-R.md` (R1-R13),
  `…-p1-F.md` (F1-F9, table de l'axe obligatoire).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

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
