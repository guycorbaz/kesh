# Story 15.12a : Clôturer les exercices dans l'ordre — l'invariant, ses trois transitions, son écran

Status: done

<!-- Créée le 2026-10-09 par DÉCOUPAGE de la Story 15-12 à la remédiation de sa validation P2 (décision de
     l'orchestrateur, choix C107 de `epic-15-choix-autonomes.md`). Version complète de la 15-12 avant
     découpage : commit `dae3a618`
     (`git show dae3a618:_bmad-output/implementation-artifacts/15-12-cloture-dans-l-ordre.md`).
     Choix applicables : C89, révisé par C100 (validation P1), C107 à C112 (validation P2) et C119, C121
     à C123 (validation P3 ; C119 révise C110).
     ⚠️ **Numérotation des AC conservée de la 15-12** — d'où des trous (8, 10-12, 15, 16, 18, 20 sont
     dans la 15-12b) : les références croisées (« AC 13 b », « mutation (ii) ») restent stables d'une
     fiche à l'autre et avec les rapports de validation. Un AC partagé porte « part A ».
     ⚠️ **Numéros de ligne** relevés sur `origin/main` `8f9811d8` (le code Rust y est identique à
     `9cb5083b` hors `kesh-api/config.rs`, `main.rs`, `lib.rs` et `configuration_transmise.rs`) ; le
     manuel administrateur, décalé de 30 lignes par la 15-11a, est recalé sur `8f9811d8`. T0 les refait
     sur le `main` du moment, par leur texte. ⚠️ **`main` a avancé depuis** : `5e4bec50` (15-5d mergée)
     décale notamment `kesh-api/src/errors.rs` (+28 lignes : bras `LaterFiscalYearClosed` `:2847`),
     `invoices.rs` (+1 à +79 lignes) et les catalogues (+4 lignes : fr-CH `:376`). Les relevés **refaits
     en P3** le sont sur `5e4bec50` et le disent.
     ✅ **T0 (2026-10-09, développement)** : tous les numéros de ligne cités sont **relocalisés sur
     `5e4bec50`** (base de la branche `story/15-12a-cloture-dans-l-ordre`), par leur texte ; ceux qui
     avaient bougé sont corrigés en place (R1, R2, AC 9, AC 23), les autres ont été retrouvés
     identiques. Les LOW de la validation P4 (rapports `target/gate-logs/15-12a-p4-{R,F}.md`) sont
     appliqués, sauf R7 (fiche 15-1a, à l'orchestrateur). Mesures T0 au Dev Agent Record. -->


**Issues** : **refs #543** (P1 — « on peut écrire dans un exercice ouvert alors qu'un exercice postérieur
est déjà clos — le bilan clos change en silence »). Cette fiche rend l'état fautif **inatteignable** à
partir d'un état sain ; le **filet** pour les données qui le portent déjà est la **15-12b**, qui
**ferme** #543. Mot-clé dans la PR (`refs #543`), le dépôt merge en squash.

**Dépend de** la 15-8a et de la 15-8b (`DbError::LaterFiscalYearClosed`, code
`LATER_FISCAL_YEAR_CLOSED`, `find_later_closed_in_tx` / `find_later_closed`, garde du `PUT` et du
`DELETE`) et de la 15-5e1/15-5e2 (enveloppes `kesh_db::retry` / `kesh_api::retry`, registre à deux
colonnes). **Toutes mergées.**

**Bloque** : la **15-12b** (elle lit `EarlierFiscalYearOpen`, la clé du message de l'AC 9, la clôture
ordonnée pour la réparation de l'AC 16) ; la **15-1a**, dont c'est le **prérequis réel** (C112).

**Touche des fichiers que d'autres stories en vol touchent** : `docs/manual/fr/user-manual.tex` et
`admin-manual.{tex,pdf}` (la 15-11a, mergée, a ajouté 195 lignes au manuel administrateur et régénéré son
PDF — la branche de planification est en retard sur `main`), `CHANGELOG.md` (rubriques de `[0.13.0]`),
`crates/kesh-db/src/repositories/journal_entries.rs` (ici : le seul doc-comment de `reverse_in_tx_inner`,
AC 14), et la 15-12b (mêmes fichiers). En cas de rebase : garder les deux côtés, **régénérer** les PDF
après le rebase (conflit binaire).

## Story

En tant que **comptable** qui clôture ses exercices,
je veux que **Kesh refuse de clôturer un exercice tant qu'un exercice antérieur est encore ouvert**, et
refuse de créer un exercice antérieur à un exercice déjà clôturé,
afin que **l'état « exercice ouvert suivi d'un exercice clôturé » ne puisse plus être atteint** — le
bilan est cumulatif depuis l'origine (`crates/kesh-report/src/balance_sheet.rs:8-11`) : toute écriture de
N figure dans le bilan de N+1.

### Le défaut, établi au code

- `fiscal_years::close` (`crates/kesh-db/src/repositories/fiscal_years.rs:773`) est un `UPDATE … SET
  status = 'Closed' WHERE id = ? AND status = 'Open'` : il ne regarde **aucun** autre exercice. Clôturer
  2027 avec 2026 ouvert est accepté.
- `fiscal_years::create` (`:156`) ne contrôle que le chevauchement et le nom : créer 2026 **après** avoir
  clôturé 2027 donne un exercice ouvert antérieur à un exercice clos.
- La réouverture (`reopen`, `:872`) a déjà sa garde LIFO (Story 14-2) : elle ne crée pas l'état fautif.
- Dans l'état « N ouvert, N+1 clos », dix-neuf routes écrivent dans N sans contrôle : c'est l'objet de
  la **15-12b**.

### Conception retenue (C89, C100, C107)

- **Invariant I** : *pour chaque société, les exercices clôturés forment un préfixe de l'ordre
  chronologique* — aucun exercice ouvert ne précède un exercice clôturé.
- **Clôture** : refusée tant qu'un exercice antérieur est ouvert (le préfixe ne s'allonge que par son
  premier exercice ouvert). C'est le cœur : elle ferme tous les flux d'un coup pour toute installation
  qui part d'un état sain.
- **Création** : refusée si un exercice postérieur à la date de début demandée est clôturé.
- **Réouverture** : garde LIFO existante (le préfixe ne raccourcit que par son dernier exercice).

⚠️ **La portée de l'invariant, déclarée (F11, C112)** : I est vrai de tout état **obtenu par ces trois
transitions à partir d'un état sain**. Il ne l'est **pas** des installations qui portent déjà l'état
fautif — données d'une v0.12.x, restauration d'une sauvegarde (`admin::full_import`) — ni d'un état
écrit hors de `close` / `reopen` (SQL direct, future transition). Ces états-là ne repassent par aucune
transition et ne peuvent que se résorber (clôture du plus ancien ouvert, réouvertures LIFO). Tout
consommateur de l'invariant doit **tolérer** l'état hérité ou l'écrire comme limite : le filet de la
15-12b couvre les écritures du journal (création, suppression), **pas** le lettrage de la 15-1a ni la
vue « au » d'une date de la 15-1b.

Écartées (détail en C89) : refuser seulement l'écriture sans ordonner la clôture ; une détection au
démarrage ; refuser d'importer une sauvegarde fautive ; une migration qui « répare ».

### Frontière avec la 15-12b

La 15-12b généralise la garde `LATER_FISCAL_YEAR_CLOSED` aux deux points de passage du journal
(`create_in_tx_inner`, `delete_in_tx`), au lot de rapprochement, et signale l'état hérité à l'écran (bandeau,
réparation). **Rien ici ne lit ni ne cite le filet comme existant** : les doc-comments de l'AC 7 et de
l'AC 14 nomment ce qui contourne l'invariant ; la 15-12b les complète en y décrivant son filet. Livrée
seule, la 15-12a est complète et testable : dans un état sain, l'état fautif est inatteignable ; dans un
état hérité, le comportement des écritures est celui de `main` (seuls `PUT` et `DELETE` refusent).
⚠️ **Publication** : **la v0.13.0 ne se tague pas sans la 15-12b** (qui ferme #543, P1). Le message neutre
de l'AC 9 ne nomme, depuis la revue P1 (B-1, C-15-12a-3), que ce que le code de la 15-12a garde — la
modification et la suppression d'une écriture ; la 15-12b l'élargira à la saisie avec sa garde.

### Frontière avec la 15-1a (R6, F11, C112)

- La 15-12a est le **prérequis réel** de la 15-1a : elle tient l'invariant I sur lequel reposent la
  règle des exercices du lettrage et la vue « au » d'une date.
- **La clôture devient rejouée** (AC 6) : la 15-1a, qui la disait « non rejouée », a été corrigée par sa
  propre remédiation P2 (elle dit désormais que la 15-12a l'enveloppe — `15-1a-socle-lettrage.md`, § des
  verrous, points 2 et 3, relu au 2026-10-09). Le **principe** que la 15-1a en tire (verrouiller les exercices en mode
  `Manual` dans l'ordre chronologique, comme la clôture ; aucun verrou d'exercice en mode `System`)
  **reste juste** : l'ordre ascendant évite le cycle, le rejeu ne fait que l'absorber s'il survient.
  ⚠️ **La forme** de ce verrouillage s'aligne sur l'AC 3 (C119, validation P3) : un parcours
  d'intervalle `… ORDER BY start_date ASC FOR UPDATE` ne fixe pas l'ordre des verrous (il dépend du
  plan). **Fait (C125)** — la 15-1a a depuis été découpée en **15-1a-i** (marque du lettrage) et
  **15-1a-ii** (gardes) : verrous par clé primaire, un par un dans l'ordre de `start_date`, bornés aux
  exercices qui portent une ligne du groupe ; l'exercice postérieur clos est lu sans verrou par
  `find_later_closed`. Aucun fantôme n'en change le verdict : le lettrage n'a pas de relecture sous
  verrou à faire.
- Dans l'**état hérité**, l'invariant ne tient pas : un groupe entièrement dans un exercice ouvert N, sous
  un N+1 clos, passe la règle « au moins une ligne sur un exercice ouvert ». Ni cette fiche ni la 15-12b
  ne le refusent ; la 15-1a l'a tranché (C113 : elle **garde** l'état hérité, patron 15-8a).
- ⛔ **Cette fiche ne modifie pas les fiches 15-1a-i / 15-1a-ii** : le point est porté à l'orchestrateur.
- **Ordre** : 15-12a → 15-12b → 15-1a-i → 15-1a-ii.

## Acceptance Criteria

### La clôture dans l'ordre

1. **Refus de la clôture.** `fiscal_years::close(pool, user_id, company_id, Y)` refuse, **sans rien
   écrire** (ni statut, ni ligne d'audit `fiscal_year.closed`), dès qu'un exercice **de la même
   société** dont `start_date < Y.start_date` est `Open`. L'erreur est une **variante neuve**
   `DbError::EarlierFiscalYearOpen { fiscal_year_id, fiscal_year_name }` qui nomme le **plus ancien**
   exercice antérieur ouvert (`ORDER BY start_date ASC LIMIT 1`) — celui qu'il faut clôturer
   d'abord (en nommer un plus proche enverrait l'utilisateur sur un exercice que la même garde
   refuserait à son tour). Les exercices d'une autre société sont ignorés (test dédié).

2. **Précédence des refus de `close`**, testée paire par paire : `NotFound` (exercice inexistant ou
   d'une autre société) → **déjà clos** (`IllegalStateTransition`, comportement actuel, 409
   `ILLEGAL_STATE_TRANSITION`) → `EarlierFiscalYearOpen`. ⚠️ Un exercice déjà clos **et** précédé d'un
   exercice ouvert (état fautif hérité) répond « déjà clos » : l'état de l'exercice lui-même parle
   d'abord, comme `FISCAL_YEAR_CLOSED` avant `LATER_FISCAL_YEAR_CLOSED` au `PUT`.

3. **Ordre des verrous de `close`**, écrit à son doc-comment et tenu par le **code**, non par le plan
   de l'optimiseur *(réécrit en P3 : F2, F5, F6, R3 ; C119, qui révise C110)* :
   (a) lecture **non verrouillante** de `start_date` de Y (scopée `(id, company_id)`) — `start_date` est
   **immuable** : aucun `UPDATE` du code de production n'écrit `start_date` (les `UPDATE fiscal_years`
   hors `#[cfg(test)]` sont ceux de `close`, `reopen` — statut — et `update_name` — `name` seul,
   `fiscal_years.rs:400`) ; une ligne supprimée puis recréée (`kesh-seed` `reset_demo`,
   `lib.rs:275`) prend un autre `id` et sort de (b) ; la restauration d'une sauvegarde est l'angle
   mort déclaré (Dev Notes) *(F5 de P4)* ; absente → `NotFound` ;
   (b) **liste des antérieurs, sans verrou** — constante `LIST_EARLIER_SQL` : `SELECT id FROM
   fiscal_years WHERE company_id = ? AND start_date < ? ORDER BY start_date ASC`, **tous statuts** (un
   antérieur clos dans la vue peut être en cours de réouverture) ;
   (b') **chacun verrouillé, un par un, dans cet ordre**, par une boucle Rust — constante
   `LOCK_EARLIER_BY_ID_SQL` : `SELECT id FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE`.
   Un exercice listé en (b) et disparu depuis (`onboarding::reset` efface les exercices de la société)
   est ignoré : la relecture (d) fait foi ;
   (c) verrou de Y — constante **`LOCK_IN_COMPANY_SQL`** : `SELECT id, company_id, name, start_date,
   end_date, status, created_at, updated_at FROM fiscal_years WHERE id = ? AND company_id = ? FOR
   UPDATE`, c'est-à-dire **le texte que `reopen` et `update_name` écrivent déjà chacun à la main**,
   extrait en une constante partagée par les trois (DRY ; F1 = R5 de P4). Sa liste de colonnes est
   **la condition du motif** de l'AC 13 (`["SELECT id, company_id", "WHERE id = ", "FOR UPDATE"]`),
   qui en est tiré : une forme réduite (`SELECT status …`) le rendrait aveugle. Par `fetch_optional`
   — Y disparu depuis (a) → `NotFound`, jamais une panique (F6) ; verdict « déjà clos » ;
   (d) **relecture verrouillante** — constante `FIND_EARLIER_OPEN_SQL` : `SELECT … FROM fiscal_years
   WHERE company_id = ? AND start_date < ? AND status = 'Open' ORDER BY start_date ASC LIMIT 1 FOR
   UPDATE`. Lecture verrouillante, donc sur l'état **validé le plus récent** et non sur la vue fixée en
   (a) : c'est **elle** qui rend le verdict `EarlierFiscalYearOpen` (le plus ancien antérieur ouvert,
   AC 1) ;
   (e) `UPDATE` existant, audit, `COMMIT`.
   **Pourquoi un par un (F2)** : une lecture verrouillante par intervalle (`… ORDER BY start_date ASC
   FOR UPDATE`, la forme de la P2) ne fixe que l'ordre du **résultat**. InnoDB verrouille dans l'ordre du
   **parcours**, que choisit l'optimiseur : l'index `uq_fiscal_years_company_start_date` (ascendant),
   mais aussi l'autre index préfixé par `company_id`, `uq_fiscal_years_company_name` (deux index, la
   contrainte de clé étrangère réutilisant le premier — `test-schema/0001_schema_squash.sql:548-550`,
   R3), ou la clé primaire, suivis d'un `filesort`. Le dépôt l'a **mesuré** :
   `opening_complement.rs:278-281` (« avec `, id`, le plan passe par `Using filesort`, qui lit — donc,
   en `FOR UPDATE`, verrouille — tous les exercices de la société »). Or l'ordre des `id` diverge de
   celui des `start_date` dès qu'un exercice antérieur est créé après un postérieur, ce que l'AC 5 laisse
   permis sous un exercice ouvert. Le verrou par clé primaire dans une boucle a un ordre que **seul le
   code** fixe : aucune mesure n'en dépend.
   **Ce que cette forme perd, et ce qui le rend** :
   - **les verrous de clé suivante et d'intervalle** d'un parcours : un exercice antérieur à Y **créé**
     après la vue de (a) — un fantôme — n'est ni listé en (b) ni verrouillé en (b'). La relecture (d) le
     voit : elle lit l'état validé. Et une telle création ne peut plus se valider une fois Y tenu par
     (c) : sa garde `find_later_closed_in_tx` (AC 5) parcourt les exercices postérieurs à X et
     **examine** Y — elle le demande, et attend la clôture —, sauf si un exercice **clos** s'interpose
     entre X et Y, auquel cas la création est refusée de toute façon. Ce qui s'est validé avant (c), (d)
     le lit ; ce qui viendrait après attend. *(Revue P1, A1 : dans les tests, la création bute même plus
     tôt, dans `find_overlapping` ; le verrou de la garde elle-même — une création dont la garde a
     passé **tient** Y, la clôture l'attend en (c) et la voit en (d) — est prouvé par le 13 b3. Le cas du
     fantôme validé entre (a) et (c) n'a pas de test : aucun montage simple ne le force, B-2.)* ;
   - **des requêtes** : une par exercice antérieur (une par année, quelques-unes par société) ;
   - **(d) reste un parcours d'intervalle**, dont les acquisitions **nouvelles** dépendent du plan. Mais
     elle vient quand la clôture tient déjà Y et tous les antérieurs connus : ce qu'elle peut encore
     acquérir, ce sont les fantômes (ouverts — ils font refuser la clôture) et, sous un plan qui ne
     suit pas l'index, d'autres lignes de la société. Un cycle qui y naîtrait est absorbé par le rejeu
     de la clôture (AC 6) ; l'invariant I n'en dépend pas, (d) étant verrouillante quel que soit le plan.
     `EXPLAIN` de (d) mesuré en T0 **à titre descriptif**, dans les deux régimes (base éphémère à une
     société ; base de dev à plusieurs sociétés) — il ne fonde rien.
   **Écartée (C119)** : `FORCE INDEX (uq_fiscal_years_company_start_date)` sur la requête de parcours,
   avec un test de régression sur son `EXPLAIN`. La garantie y reste une **mesure** — un indice que
   l'optimiseur suit et qu'un `EXPLAIN` confirme pour la version de MariaDB testée —, là où la boucle
   n'en demande aucune ; et l'indice fige un nom d'index dans une requête. Le dépôt emploie `FORCE
   INDEX (PRIMARY)` (`company_invoice_settings.rs:622`, 15-5d) pour **borner** des verrous partagés à
   des clés primaires, non pour ordonner des verrous exclusifs.
   **Pourquoi cet ordre** : la clôture acquiert les antérieurs connus **avant** Y, dans l'ordre
   chronologique. Verrouiller Y d'abord l'exposerait au cycle avec une réouverture de Y-1 (qui tient
   Y-1 puis demande les postérieurs, Y compris) — et `reopen` n'est **pas** rejouée : elle serait la
   victime, en 500. ⚠️ L'argument suppose que la réouverture ne demande, après Y-1, que des
   postérieurs : vrai sous le plan par l'index de `FIND_LATER_CLOSED_SQL` (`reopen`, garde 15-8a, et la
   création de l'AC 5) ; sous un autre plan, ce parcours peut verrouiller toute la société. Limite
   **préexistante** de `reopen` et de la garde 15-8a, dont le code ne change pas ici : `EXPLAIN` de
   `FIND_LATER_CLOSED_SQL` mesuré en T0 dans les deux régimes, à titre descriptif ; un plan qui ne suit
   pas l'index est écrit au Dev Agent Record et signalé à l'orchestrateur (issue), non corrigé ici.
   ⚠️ **`OPEN_COVERING_DATE_SQL` reste sans `ORDER BY`** (C119, **révise C110**) : la requête rend au
   plus une ligne (les exercices ne se chevauchent pas) ; un `ORDER BY` n'en changerait pas le résultat
   et ne fixerait pas l'ordre de ses verrous, pour la raison ci-dessus. Rien dans cette fiche ne dépend
   de l'ordre dans lequel `find_open_covering_date` verrouille : son doc-comment dit déjà que la place de
   l'exercice est une convention de fréquence, la défense étant le rejeu (`fiscal_years.rs:543-549`).
   ⚠️ **Ce n'est PAS l'ordre de tous les flux** (C100). Les flux qui verrouillent l'exercice d'une
   **origine** puis cherchent l'exercice **du jour** prennent l'ordre inverse : `reverse_in_tx_inner`
   (`journal_entries.rs:2175`) verrouille l'origine et son exercice par clé primaire (`:2183-2187`),
   puis `find_open_covering_date(today)` (`:2300`) parcourt un intervalle qui contient les exercices
   **antérieurs** à l'origine — depuis le premier exercice de la société sous le plan par l'index, toute
   la société sous un autre plan — et les verrouille **après** elle. Mêmes deux temps, précédés du
   verrou de la pièce, dans `supplier_invoices::cancel_in_tx` (`:1008`, puis `reverse_owned_in_tx`
   `:1033`), `supplier_invoices::cancel_settlement_in_tx` (`:1213`, puis `:1242`),
   `invoice_settlements_write::cancel_settlement_in_tx` (`:753`, puis `:777`) *(relocalisés sur
   `5e4bec50` en T0, R1 de P4)* et
   `reconciliation_cancel::cancel_in_tx` (`:310`, puis `reverse_in_tx` `:370`). Un cycle avec la
   clôture est donc **possible, quel que soit le plan** : `close(N)` tient un antérieur M (étape b') et
   demande N (étape c) ; la contre-passation d'une écriture de N tient N et demande M. Il se résout par
   le **rejeu** : les cinq routes de ces flux sont `Rejouee` au registre (`reverse_journal_entry`,
   `cancel_supplier_invoice`, `cancel_supplier_invoice_settlement`, `cancel_invoice_settlement_handler`,
   `post_cancel_reconciliation`), et la clôture le devient (AC 6) — test 13 d.
   **Le même cycle existe avec la création** (F5) : `create(X)` tient les exercices antérieurs à X
   (pré-contrôle `find_overlapping … FOR UPDATE`, un parcours qui contient tous les exercices de début
   antérieur à la fin de X), puis demande les postérieurs (`find_later_closed_in_tx`, AC 5) ; une
   contre-passation ou une annulation d'une écriture d'un exercice N postérieur à X tient N, puis
   parcourt depuis le premier exercice. Résolu par le rejeu des deux côtés (création : AC 6 ; flux :
   `Rejouee`). Les requêtes (b), (b'), (c) et (d) sont des **constantes**, pas des chaînes écrites deux
   fois.
   ⚠️ **« Se résout par le rejeu » a une borne** (F6 de P4) : l'enveloppe fait au plus
   **trois tentatives** (`DEFAULT_MAX_DEADLOCK_ATTEMPTS = 3`, `kesh-db/src/retry.rs:71`), et un
   dépassement du délai d'attente de verrou (`1205`, `innodb_lock_wait_timeout` = 50 s) n'est **pas**
   rejoué (`retry.rs:96-99`). Conséquence nouvelle pour la clôture : elle attend désormais en (b')
   tout écrivain en vol qui tient un exercice antérieur (les écrivains verrouillent par parcours
   depuis le premier exercice, `find_open_covering_date`) ; une attente de plus de 50 s rend un 500.
   Préexistant pour la contre-passation, nouveau pour la clôture ; écrit aux Dev Notes.

4. **HTTP.** `POST /api/v1/fiscal-years/{id}/close` rend **`409`** code **`EARLIER_FISCAL_YEAR_OPEN`**,
   `details.fiscalYearId` / `details.fiscalYearName` (l'exercice à clôturer d'abord), message
   localisé `error-fiscal-year-close-earlier-open` avec `$name` (« Clôturez d'abord l'exercice
   « { $name } », plus ancien et encore ouvert : le bilan est cumulatif, et un exercice ne se clôt
   qu'après tous ceux qui le précèdent. »). Mapping dans `kesh-api/src/errors.rs`, à côté de
   `LaterFiscalYearClosed`. **Code dédié, et non `ILLEGAL_STATE_TRANSITION` comme la garde LIFO de la
   réouverture** : l'écran des exercices traduit **tout** `ILLEGAL_STATE_TRANSITION` de la clôture en
   « Cet exercice est déjà clôturé » (`+page.svelte`, `submitClose`) — réutiliser ce code y afficherait
   un mensonge ; et une intégration par clé d'API doit pouvoir distinguer les deux refus.

5. **Création.** `fiscal_years::create` refuse un exercice dont la `start_date` précède celle d'un
   exercice **clôturé** de la société : `DbError::LaterFiscalYearClosed { fiscal_year_id,
   fiscal_year_name }` (le **plus proche** postérieur clos, `find_later_closed_in_tx` — lecture
   **verrouillante**, cf. AC 13 b), après les pré-contrôles existants (longueur du nom, chevauchement,
   nom en double) et avant l'`INSERT`. HTTP **`400 LATER_FISCAL_YEAR_CLOSED`**, mêmes `details` que le
   mapping global, mais **message propre à la création** *(T0, F3 de P4, choix C-15-12a-1)* : le texte
   de l'AC 9 conseille la contre-passation d'**une écriture**, ce qui ne répond à rien de ce que tente
   l'écran de création. `map_create_error` intercepte donc `DbError::LaterFiscalYearClosed` et le rend
   par une variante d'`AppError` dédiée (`FiscalYearBeforeClosedYear { fiscal_year_id,
   fiscal_year_name }`), clé neuve **`error-fiscal-year-create-later-closed`** — fr-CH : « L'exercice
   « { $name } », postérieur, est clôturé, et son bilan reprend tout ce qui le précède : aucun exercice
   ne peut être créé avant sa date de début tant qu'il l'est. Pour créer celui-ci, un administrateur
   rouvre d'abord les exercices clôturés, en commençant par le plus récent. » (textes DE/IT/EN : AC 9).
   Code, statut et `details` restent ceux de l'AC 9.
   `create_for_seed` (une seule société neuve, un seul exercice : `kesh-seed/src/lib.rs:168`) et
   `create_if_absent_in_tx` (n'insère que si la société n'a **aucun** exercice) ne peuvent pas produire
   l'état fautif et ne changent pas — écrit à leur doc-comment. **Test HTTP** (`fiscal_years_e2e.rs`) :
   `POST /fiscal-years` sous un exercice clos → `400 LATER_FISCAL_YEAR_CLOSED`, corps complet
   (`details.fiscalYearId` / `fiscalYearName`, message de création ci-dessus) ; paire de précédence : une demande
   qui **chevauche** un exercice et précède un exercice clos rend **`400 VALIDATION_ERROR`**, message
   `error-fiscal-year-overlap` (pré-contrôle existant, qui parle d'abord : `FY_OVERLAP_KEY` est la clé
   interne de `DbError::Invariant`, que `map_create_error` traduit en `AppError::Validation` —
   `routes/fiscal_years.rs:102`, `errors.rs:1277-1278` sur `5e4bec50` ; R7). Le test existant
   `create_overlap` (`fiscal_years_e2e.rs:229`) asserte ce message.

6. **Rejeu des deux routes, et sa preuve.** `create_fiscal_year` et `close_fiscal_year`
   (`routes/fiscal_years.rs:218`, `:279` ; aucune enveloppe aujourd'hui) appellent leur dépôt dans
   l'enveloppe `kesh_db::retry::retry_on_deadlock("fiscal_years::create" | "fiscal_years::close", …)`
   (transaction unique, sûre à relancer) : la course création/clôture peut se résoudre par un
   interblocage (verrous de la relecture (d) de la clôture, ou des verrous par clé primaire de (b')/(c),
   contre les parcours et l'intention d'insertion de la création, cf. AC 13 b), et les cycles
   clôture/contre-passation et création/contre-passation (AC 3, F5) en sont. Elles restent
   `SansEcritureAuJournal` au registre `crates/kesh-api/tests/audit_route_registry.rs` (point (vi) de
   son doc-comment : la colonne ne dit rien de la présence d'une enveloppe) ; les volets (c) et (c bis)
   restent verts (enveloppe nommée, jamais la primitive `retry_with`).
   **La preuve de l'enveloppe** (R2, C109) — le volet (c) n'examine pas les `SansEcriture…`, et un test
   `kesh-db` écrirait l'enveloppe lui-même :
   - **clôture** : test HTTP `fiscal_year_close_is_replayed_when_it_is_the_deadlock_victim` dans
     `crates/kesh-api/tests/rejeu_interblocage_e2e.rs`, patron des tests 2 à 5 du fichier
     (`transaction_lourde`, `attendre_une_requete_en_cours`, témoin `common::capture_rejeu`). Montage :
     M **clos**, N **ouvert**, M < N ; la transaction lourde tient N (`SELECT … FROM fiscal_years WHERE
     id = N FOR UPDATE`) ; `POST /fiscal-years/{N}/close` lancée en tâche ; attente de la clôture à
     l'étape **(c)** (motif de l'AC 13) — l'étape (b') a verrouillé M **par sa clé primaire**,
     explicitement et quel que soit son statut ; la transaction demande M (`.expect` : elle doit
     l'obtenir, sinon c'est elle la victime et le test ne prouve rien) ; elle annule ; la route rejoue et
     rend **200**, N clos, et l'événement `warn!` de `kesh_db::retry` nomme `fiscal_years::close`.
     *(L'hypothèse de la P2 — « la requête de parcours garde le verrou d'une ligne examinée qu'elle ne
     retient pas » — n'a plus d'objet depuis C119 : le verrou de M est posé par (b'), non déduit d'un
     parcours ; et le point d'arrêt n'est plus « (b) ou (c) selon la mesure » : (b') ne bloque pas sur M,
     libre, si bien que la clôture ne peut buter qu'en (c), sur N.)*
   - **création** : même patron si la mesure de l'AC 13 b montre un interblocage dont la victime se
     laisse forcer (`fiscal_year_creation_is_replayed_when_it_is_the_deadlock_victim`) ; **sinon**, la
     mutation (x) est écrite « tenue par revue » au Dev Agent Record et au point (vi) du registre, comme
     `onboarding::finalize`. ✅ **Mesuré en T0 : la victime se laisse forcer** — montage M clos, Y
     ouvert, M < X < Y ; la transaction lourde tient Y par clé primaire ; `POST /fiscal-years` (X) passe
     son pré-contrôle `find_overlapping` (qui lit, donc verrouille, M) et bute sur Y dans
     `find_later_closed_in_tx` (motif `["start_date > ", "FOR UPDATE"]`) — **à la main** ; dans le test
     HTTP 10, sur une base `#[sqlx::test]` de quelques lignes, elle bute dès `find_overlapping`, d'où son
     motif large `["fiscal_years", "FOR UPDATE"]` (revue P1, A1) ; la transaction demande M :
     interblocage, la création (plus légère) est annulée, la transaction obtient M, annule ; la route
     rejoue et rend **201**. Le test est donc écrit, et il tue la mutation (x).
   **Le doc-comment du registre est mis à jour** (propagation) : le point (iv) (`:94-98`) retire
   `POST /fiscal-years/{id}/close` des routes non rejouées exposées au cycle (il y garde `/reopen`, en
   nommant le cycle réouverture/contre-passation, préexistant et hors de cette story) ; le point (vi)
   (`:103`) passe de « **deux** routes `SansEcritureAuJournal` sont rejouées » à **quatre**, en nommant
   `create_fiscal_year` et `close_fiscal_year`, leur enveloppe **et leur preuve** (test HTTP, ou revue).
   **Et les deux autres énumérations des preuves dynamiques du même doc-comment** (F4) : le point
   **(ii)** (« Seuls les tests 2 à 5 et 7 de `rejeu_interblocage_e2e.rs` le prouvent dynamiquement, pour
   **cinq** routes de la famille `DbError` », `:58-60`) et le point **(iii bis)** (« Ont une preuve
   dynamique : les **cinq** routes des tests 2 à 5 et 7… », `:72-73`) gagnent le test de la clôture —
   et celui de la création si T0 l'a rendu possible —, leur famille (`DbError`, enveloppe
   `kesh_db::retry`) et le nombre de routes. Chaque décompte est **refait depuis le fichier de tests et
   le code des routes**, non incrémenté (`grep -n "^async fn\|^#\[sqlx::test" rejeu_interblocage_e2e.rs`).

7. **Réouverture inchangée, invariant écrit.** `reopen` ne change pas de code. Le doc-comment du
   module `fiscal_years.rs` énonce l'invariant I, les trois transitions qui le tiennent (clôture : AC 1 ;
   création : AC 5 ; réouverture : garde LIFO), **sa portée** (états atteints depuis un état sain) et ce
   qui le contourne : la restauration d'une sauvegarde (`admin::full_import`), les données d'une version
   antérieure, toute écriture de `fiscal_years.status` hors de `close` / `reopen` (si une future
   transition le fait, la preuve tombe). La 15-12b y ajoute la description de son filet. Le doc-comment
   de `DbError::LaterFiscalYearClosed` (`kesh-db/src/errors.rs:553-561`), qui dit « `fiscal_years::close`
   ne regarde pas les exercices antérieurs », est réécrit : la variante sert au `PUT`, au `DELETE` et à
   la création d'un exercice (la 15-12b y ajoutera ses deux points de passage).

### Le message

9. **Message neutre** *(déplacé de la 15-12b à la découpe, R3/C107 : l'AC 5 le rend dès cette fiche ;
   complété en P2, F5/R4/C111)*. Le mapping global de `DbError::LaterFiscalYearClosed`
   (`kesh-api/src/errors.rs:2843-2872` sur `5e4bec50`) lit une **clé neuve** `error-later-fiscal-year-closed`, aux
   quatre locales et dans le repli Rust (qui remplace celui de `:2851-2853`). Texte fr-CH de
   référence : « L'exercice « { $name } », postérieur, est clôturé, et son bilan reprend tout ce qui le
   précède : aucune écriture datée avant sa date de début ne peut être modifiée ni supprimée tant qu'il
   l'est. Une telle écriture se corrige par une contre-passation ; sinon, un administrateur rouvre les
   exercices clôturés, en commençant par le plus récent. » *(Revue P1, B-1/E4, C-15-12a-3 : la première
   rédaction disait « rien ne peut être enregistré, modifié ou supprimé » — faux de la saisie tant que
   la 15-12b n'a pas posé sa garde, et contraire au manuel `user-manual.tex:712` et à
   `api-external.md`. La 15-12b élargira le texte à la saisie.)*
   - La formulation ne présuppose **pas** que l'exercice visé existe (elle vaut à la création d'un
     exercice, F12) ;
   - elle **garde le conseil** que le `PUT` / `DELETE` donnait (la contre-passation, F5) ;
   - elle ne prescrit **jamais** de rouvrir « { $name } » : ce serait refusé par la garde LIFO dès qu'un
     exercice plus récent est clos (C100) ; « en commençant par le plus récent » est le seul ordre que
     LIFO accepte, et il vaut aussi à la création d'un exercice (état sain, où le bandeau de la 15-12b ne
     s'affiche pas : le message porte lui-même la marche à suivre — F5). ⚠️ **À la création d'un
     exercice, c'est la clé dédiée de l'AC 5 qui parle** (T0, C-15-12a-1) : le conseil de
     contre-passation n'y a pas d'objet.
   **Textes des trois autres locales** *(T0, F3 de P4)* — termes du glossaire `docs/i18n-glossaire.md`
   (ligne *clôture (d'exercice)* : *Abschluss* / *chiusura* / *closing*, jamais « fermer » — KF-041 ;
   *contre-passation* : *Stornobuchung* / *storno* / *reversal*, comme les clés voisines) :
   - `error-later-fiscal-year-closed` — de-CH : « Das spätere Geschäftsjahr „{ $name }“ ist
     abgeschlossen, und seine Bilanz enthält alles, was ihm vorangeht: Eine Buchung vor seinem Beginn
     kann weder geändert noch gelöscht werden, solange es abgeschlossen ist. Eine solche Buchung wird
     durch eine Stornobuchung korrigiert; andernfalls eröffnet eine Administratorin oder ein Administrator
     die abgeschlossenen Geschäftsjahre wieder, beginnend mit dem neuesten. » ; it-CH : « L’esercizio
     successivo « { $name } » è chiuso, e il suo bilancio riprende tutto ciò che lo precede: nessuna
     scrittura datata prima della sua data d’inizio può essere modificata o eliminata finché lo è. Una
     tale scrittura si corregge con uno storno; altrimenti, un amministratore riapre gli esercizi chiusi,
     cominciando dal più recente. » ; en-CH : « The later fiscal year "{ $name }" is closed, and its
     balance sheet includes everything before it: no entry dated before its start date can be changed
     or deleted while it is closed. Such an entry is corrected with a reversal; otherwise, an
     administrator reopens the closed fiscal years, starting with the most recent. »
   - `error-fiscal-year-create-later-closed` (AC 5) — même première proposition, puis de-CH : « Vor
     seinem Beginn kann kein Geschäftsjahr erstellt werden, solange es abgeschlossen ist. Um dieses zu
     erstellen, eröffnet eine Administratorin oder ein Administrator zuerst die abgeschlossenen
     Geschäftsjahre wieder, beginnend mit dem neuesten. » ; it-CH : « nessun esercizio può essere creato
     prima della sua data d’inizio finché lo è. Per crearlo, un amministratore riapre prima gli esercizi
     chiusi, cominciando dal più recente. » ; en-CH : « no fiscal year can be created before its start
     date while it is closed. To create this one, an administrator first reopens the closed fiscal
     years, starting with the most recent. »
   - ancienne clé alignée (ci-dessous), première phrase inchangée, puis de-CH : « Korrigieren Sie sie
     durch eine Stornobuchung; andernfalls eröffnet eine Administratorin oder ein Administrator die
     abgeschlossenen Geschäftsjahre wieder, beginnend mit dem neuesten. » ; it-CH : « Correggetela con
     uno storno; altrimenti, un amministratore riapre gli esercizi chiusi, cominciando dal più
     recente. » ; en-CH : « Correct it with a reversal; otherwise, an administrator reopens the closed
     fiscal years, starting with the most recent. »
   - `error-fiscal-year-close-earlier-open` (AC 4) et `fiscal-year-close-blocked-earlier-open` (AC 17) —
     de-CH : « Schliessen Sie zuerst das Geschäftsjahr „{ $name }“ ab, das älter und noch offen ist[:
     Die Bilanz ist kumulativ, und ein Geschäftsjahr wird erst nach allen vorangehenden
     abgeschlossen]. » ; it-CH : « Chiudi prima l’esercizio « { $name } », più vecchio e ancora
     aperto[: il bilancio è cumulativo, e un esercizio si chiude solo dopo tutti quelli che lo
     precedono]. » ; en-CH : « First close fiscal year "{ $name }", which is earlier and still open[:
     the balance sheet is cumulative, and a fiscal year can only be closed after all those before
     it]. » (entre crochets : la seconde proposition, propre à la clé de l'API).
   Le texte actuel (`journal-entries-modify-blocked-later-fiscal-year-closed`, « elle reste figée…
   corrigez par une contre-passation ») ne vaut que pour la modification ; la clé neuve sert désormais
   au `PUT`, au `DELETE` et à la création d'un exercice (la 15-12b y ajoute la saisie, les règlements, la
   validation d'une facture, la dévalidation). **L'ancienne clé reste**, lue par
   `frontend/src/lib/features/journal-entries/blocker-messages.ts:91-96`, texte d'écran de la fiche
   d'écriture (`modificationBlockedBy`). **Mais sa prescription est alignée** (R4, C111) — elle dit
   aujourd'hui « Un administrateur peut rouvrir cet exercice », le geste que C100 a réfuté, dans l'état
   même (hérité) où elle s'affiche. Texte fr-CH : « L'exercice postérieur { $name } est clôturé, et son
   bilan reprend cette écriture : elle reste figée tant qu'il l'est. Corrigez-la par une
   contre-passation ; sinon, un administrateur rouvre les exercices clôturés, en commençant par le plus
   récent. » **Sites, relevés sur `8f9811d8` par la valeur** (`grep -rnE "rouvrir cet exercice|reopen that
   fiscal year|wieder eröffnen;|riaprire quell"` hors fiches et PDF ; **relocalisés sur `5e4bec50` en T0**) :
   `crates/kesh-i18n/locales/fr-CH/messages.ftl:376`, `de-CH/messages.ftl:382`,
   `it-CH/messages.ftl:382`, `en-CH/messages.ftl:382`, repli de
   `blocker-messages.ts:94` — et le repli Rust `errors.rs:2852`, qui disparaît avec le passage à la clé
   neuve. Le `grep` se refait après correctif et ne doit plus rien rendre hors des fiches de la 15-8a.
   Code, statut (`400`) et `details` inchangés.

### La concurrence

13. **Tests à deux connexions** (`crates/kesh-db/tests/fiscal_years_repository.rs`, base éphémère
    `#[sqlx::test]`, `attendre_une_requete_en_cours` de `test_fixtures.rs`) — chacun asserte **l'état
    final** et l'issue de **chaque** côté, et **force** l'entrelacement qu'il teste : une course libre
    (`tokio::spawn` × 2) ne tombe qu'au hasard dans la fenêtre qui compte (C100). La session W est une
    transaction de test qui tient un verrou.
    ⚠️ **Désignations** *(T0, R4 de P4)* : les **tests** s'écrivent toujours « 13 a » … « 13 d » (et
    « 13 b1 », « 13 b2 » pour les deux configurations du 13 b) ; les **étapes** de `close` s'écrivent
    « étape (a) » … « étape (e) », « étape (b') ». Une lettre seule entre parenthèses, ci-dessous, est
    une étape dans la liste des motifs, un test dans la liste des tests.
    **« Attendre la clôture », ce que l'aide sait voir (R7, F9)** : `attendre_une_requete_en_cours`
    (`test_fixtures.rs:560-590`) compte les **autres** connexions de la base dont `PROCESSLIST.INFO`
    contient tous les motifs : elle voit une requête **en cours d'exécution**, qu'elle soit bloquée ou
    non — elle ne sait **pas** dire « en attente de verrou ». Ce qui fait de « en cours » une attente,
    c'est le montage : W tient la ligne que la requête attendue demande, si bien que la requête reste
    visible tant qu'elle bute. Les doc-comments des tests le disent ainsi, sans prétendre davantage.
    Elle rend `false` (et non une panique) si la tâche observée a fini — `interrompre` — : un test doit
    alors rougir sur l'**état final**, jamais sur le délai de dix secondes.
    **Motifs discriminants**, par étape de `close` (C119 ; toutes portent `fiscal_years` ; les motifs
    évitent `?`, que `INFO` peut montrer substitué ou non — T0 le constate) :
    - **(a)** lecture non verrouillante de `start_date`, et **(b)** liste sans verrou des antérieurs :
      **jamais attendues** (elles ne bloquent pas) ;
    - **(b')** verrou d'un antérieur par clé primaire (`LOCK_EARLIER_BY_ID_SQL`) :
      `["SELECT id FROM fiscal_years WHERE id = ", "FOR UPDATE"]` — la liste de colonnes réduite à `id`
      l'oppose à (c) ;
    - **(c)** verrou de Y par clé primaire : `["SELECT id, company_id", "WHERE id = ", "FOR UPDATE"]` —
      `WHERE id = ` exclut les requêtes par société, `FOR UPDATE` exclut l'étape (a) ;
    - **(d)** relecture verrouillante (`FIND_EARLIER_OPEN_SQL`) : `["start_date < ", "FOR UPDATE"]` —
      l'espace après `<` exclut `start_date <= ` d'`OPEN_COVERING_DATE_SQL` et `start_date > ` de
      `FIND_LATER_CLOSED_SQL` ; `FOR UPDATE` exclut la liste (b).
    Motifs par test : **13 a et 13 c** — la clôture bute en **(b')** sur N, que W tient ; motif large
    `["fiscal_years", "FOR UPDATE"]`, tolérant à l'étape où elle bute : sous la mutation (ii) — (d) non
    verrouillante —, elle bute toujours en (b'), et le test rougit ensuite sur l'état final ou l'issue.
    Ce motif large convient parce qu'aucune autre connexion n'exécute de requête pendant l'attente (W est
    inactive). **13 d** et **test HTTP de l'AC 6** — le motif de **(c)** : la clôture ne peut buter
    qu'en (c), sur N ; (b') a verrouillé M, libre, sans attendre, et l'exécution est séquentielle — voir
    (c) en cours prouve que (b') a rendu et tient M. *(L'aléa de la P2 — « la clôture peut buter dès (b),
    un instant avant d'avoir verrouillé M » — disparaît avec le parcours d'intervalle : C119.)*
    - **(a) Une réouverture de N en cours contre `close(L)`** (N clos, L ouvert, L > N, rien d'autre) —
      **c'est la preuve de l'invariant I, et le test qui tue la mutation (ii)**. W refait les gestes de
      `reopen` sans les valider : `SELECT … WHERE id = N FOR UPDATE`, lecture des postérieurs clos
      `FOR UPDATE` (rend rien), `UPDATE fiscal_years SET status = 'Open' WHERE id = N`. On lance
      `close(L)` (dans son enveloppe), on attend qu'elle soit vue en cours, on valide W ; on asserte
      `EarlierFiscalYearOpen` nommant N, L **ouvert**, aucune ligne d'audit `fiscal_year.closed`.
      Sous la mutation (ii) (relecture (d) non verrouillante), la clôture attend W en (b') puis lit (d)
      dans sa vue, fixée en (a) pendant que W n'était pas validée : N **clos** — la clôture passe, et
      l'état final est « N ouvert, L clos » — rouge sur la propriété elle-même, de façon déterministe. Le test existant `reopen_close_concurrent_is_serialized` (`:1055`), course libre qui
      asserte que la clôture réussit toujours, est **remplacé** par celui-ci.
    - **(b) `create(X)` contre `close(Y)`**, X antérieur à Y, la société n'ayant aucun autre exercice
      ouvert avant Y : **jamais** « X ouvert, Y clos ». Les deux appels passent par leur **enveloppe de
      rejeu** (AC 6) — le test appelle les fonctions du dépôt **dans** `retry_on_deadlock`, comme les
      routes. ⚠️ Mécanisme à **mesurer** en T0 et non à supposer, **dans les deux configurations** :
      (b1) sans aucun exercice antérieur à X ; (b2) avec un exercice **clos** antérieur à X — là, le
      pré-contrôle `find_overlapping` de la création (parcours depuis le premier exercice) et le verrou
      (b') de la clôture portent tous deux sur ce premier exercice et s'y rencontrent probablement
      (sérialisation, sans interblocage). ✅ **Mesuré en T0** (MariaDB 10.11.16, trois sessions à la
      main, la clôture bloquée en étape (c) par une session qui tient Y, la création lancée ensuite) :
      **13 b1 — sérialisation** : la création passe ses pré-contrôles, bute sur Y dans
      `find_later_closed_in_tx`, attend la clôture et lit Y **clos** → `LaterFiscalYearClosed` —
      **à la main** ; **dans les tests** (`#[sqlx::test]`), la création bute dès `find_overlapping`, sur
      Y (b1) ou sur M (b2), et ne lit Y clos dans sa garde qu'après : les 13 b1 / b2 prouvent la
      propriété, **non le verrou de la garde** — une garde non verrouillante y passerait aussi (revue
      P1, A1 = B-2 = E1). D'où le **13 b3** ci-dessous ;
      **13 b2 — interblocage**, contrairement à la supposition : la clôture tient M (étape (b')), la
      création lit M dans `find_overlapping` et l'attend, la clôture, Y obtenu, demande en étape (d) le
      verrou d'index de M que la création a posé — cycle ; victime observée : la création, qui rejouée
      lit Y clos. La phrase de l'AC 6 (« peut se résoudre par un interblocage ») est donc **confirmée**.
      Dans les deux cas, la propriété ne dépend pas du mécanisme :
      une création validée avant que la clôture ne tienne Y est lue par (d) ; une création postérieure
      attend Y (AC 3, « ce que cette forme perd »). Un test par configuration ; chacun écrit le mécanisme
      observé (interblocage rejoué, ou sérialisation) dans son doc-comment, et la phrase de l'AC 6 est
      corrigée si aucune configuration n'interbloque. La mesure décide aussi de la preuve de
      l'enveloppe de la création (AC 6, mutation (x)).
    - **(b3) Le verrou de la garde de `create`** *(ajouté en revue P1, A1)* : W refait les gestes de
      `create` **qui suivent** ses pré-contrôles, sans les valider — la garde `find_later_closed_in_tx`
      (rend rien, Y ouvert), puis, après le lancement de `close(Y)` et son attente en étape (c),
      l'`INSERT` de X — et valide. On asserte : jamais « X ouvert, Y clos » ; la clôture vue en (c) ;
      `EarlierFiscalYearOpen` nommant X ; Y ouvert ; aucun audit `fiscal_year.closed`. W ne rejoue
      **pas** `find_overlapping`, dont le verrou de borne masquerait celui de la garde. **Tue la mutation
      (xi)** (garde non verrouillante), jouée : seul ce test rougit (« X ouvert sous Y clos »).
    - **(c) Une clôture de N en cours contre `close(N+1)`** (N et N+1 ouverts) — garde contre un refus
      **parasite**, sous forme **ordonnée** (C100) : W pose `UPDATE fiscal_years SET status = 'Closed'
      WHERE id = N`, non validé ; on lance `close(N+1)`, on attend qu'elle soit vue en cours (bloquée sur
      N à l'étape (b')), on valide W ; `close(N+1)` **réussit**. Forme ordonnée plutôt que l'ensemble des
      issues admises : lancées librement, `close(N+1)` peut lire N ouvert la première et refuser
      **légitimement** — asserter l'ensemble des issues admettrait ce refus, et le test ne garderait plus
      contre le refus parasite, qui est son objet. Sous la mutation (ii), `close(N+1)` attend W en (b'),
      puis lit (d) dans sa vue fixée en (a) : N **ouvert** → refus parasite → rouge sur l'issue.
    - **(d) `close(N)` contre une contre-passation** (cycle de l'AC 3 ; M clos, N ouvert, T ouvert
      couvrant le jour du serveur, M < N < T) : W refait les deux temps de `reverse_in_tx_inner` —
      verrou de N par clé primaire, puis `fiscal_years::find_open_covering_date(today)` — en lançant
      `close(N)` entre les deux et en attendant de la voir buter en (c) (motif ci-dessus). W demande
      alors M — son parcours le contient quel que soit le plan —, que la clôture tient par (b') :
      interblocage. Assertion : la clôture, dans son enveloppe, **finit
      acceptée** (N clos) quelle que soit la victime ; si c'est W, W reçoit un 1213 (annulée par le test).
      T0 mesure si la victime se laisse forcer (W alourdie d'abord, patron des tests de rejeu). **Objet de
      ce test : le cycle existe et se résout** — il ne prétend **pas** tuer la mutation (ix), l'enveloppe
      y étant écrite par le test (R2) ; c'est le test HTTP de l'AC 6 qui la tue.
      ✅ **Mesuré en T0** : la victime est **la clôture**, W alourdie (500 lignes) **ou non** — W obtient
      M. ⚠️ **Qui libère W** *(F6 de P4)* : la clôture rejouée rebute en étape (b') sur M, que W tient
      désormais ; le test **annule W dès que W a obtenu M, puis seulement** attend la clôture — un `join`
      d'abord attendrait `innodb_lock_wait_timeout` (`1205`, jamais rejoué). Si, contre la mesure, W
      était la victime, W reçoit un 1213 et le test l'annule de même.
    L'ancien **13 c** de la création (« un écrivain dans N contre `close(N+1)` ») est **retiré** (C100).
    Chaque test nomme, dans son doc-comment, la **mutation qu'il tue** (AC 19) — **sauf** le 13 d
    (preuve d'existence et de résolution du cycle, aucune mutation) et les 13 b1 / 13 b2, dont la
    propriété (« jamais X ouvert, Y clos ») est tenue par les gardes (i) et (iv) ensemble ; ils
    l'écrivent ainsi *(T0, R3 de P4)*.

14. **Doc-comments canoniques** *(révisé en P3 : C119, R5, F5)*. Le doc-comment du module
    `fiscal_years.rs` (§ « Lock ordering & audit », `:11-22`) dit que les mutatrices « ne verrouillent
    que `fiscal_years` » — toujours vrai — et recopie la phrase de l'AC 3 : **la clôture** acquiert ses
    antérieurs connus **un par un par clé primaire, dans l'ordre chronologique — ordre fixé par le
    code** —, puis l'exercice, puis relit les antérieurs ouverts sous verrou ; **la réouverture, la
    garde 15-8a et la création** prennent leur exercice (ou leurs pré-contrôles) puis **parcourent** les
    postérieurs (`FIND_LATER_CLOSED_SQL`), dans l'ordre du **plan** — ascendant sous l'index
    `uq_fiscal_years_company_start_date`, mesuré en T0 à titre descriptif ; `find_open_covering_date`
    parcourt un intervalle dans l'ordre du plan, convention de fréquence. Les flux qui verrouillent
    l'exercice d'une origine puis cherchent l'exercice du jour (contre-passation et les quatre
    annulations qui la portent) prennent l'ordre inverse, et leurs cycles avec la clôture **et avec la
    création** se résolvent par le rejeu des deux côtés. ⛔ **Pas** « l'ordre ascendant de toutes les
    acquisitions » (C100), ni « ascendant par construction » d'un parcours d'intervalle (C119) : un
    `ORDER BY` fixe le résultat, pas l'ordre des verrous. Le doc-comment de `reverse_in_tx_inner` nomme
    ces cycles à côté de son étape (4). Le doc-comment de `close` décrit les étapes (a) à (e), ce que la
    forme « un par un » perd et ce qui le rend (AC 3).
    **Les deux doc-comments de `fiscal_years.rs` que la story rend faux** (R5) :
    - `FIND_LATER_CLOSED_SQL` (doc-comment `:664-667`) dit que les tests de la 15-8a reconnaissent la requête
      verrouillante à ses motifs `ORDER BY start_date ASC` et `FOR UPDATE` : après cette fiche,
      `FIND_EARLIER_OPEN_SQL` (AC 3 d) porte aussi ce texte — le doc-comment le dit, et renvoie à la
      vérification de l'AC 21 ;
    - `find_later_closed_in_tx` (`:639-645`) dit son hypothèse de verrouillage (*next-key locking* du
      parcours `start_date > ?`) « testée par le test de course concurrente `reopen`/`close` » — test
      **remplacé** par le 13 a (AC 13), qui n'éprouve pas cette hypothèse-là. Réécrit : l'hypothèse vaut
      sous le plan par l'index (`EXPLAIN` de T0, descriptif), et ce qui la teste aujourd'hui est nommé
      (les tests de concurrence de la 15-8a, `journal_entries_modification.rs:410`, `:511`), ou l'absence
      de test est écrite.
    **`docs/MULTI-TENANT-SCOPING-PATTERNS.md`** (F4 de la P2) — la ligne `fiscal_years::create /
    update_name / close / find_*_locked` (`:323`) décrit l'ordre des verrous que la story change : elle
    devient « `close` : exercices antérieurs lus sans verrou puis verrouillés un par un par clé primaire
    (`start_date` croissant), puis l'exercice, puis relecture verrouillante des antérieurs ouverts ;
    `create` : pré-contrôles puis exercices postérieurs clos (`find_later_closed_in_tx`) ;
    `find_open_covering_date` : parcours, ordre du plan » ; une note nomme les cycles clôture /
    contre-passation et création / contre-passation (et les quatre annulations), résolus par le rejeu des
    deux côtés (Pattern 5 : l'ordre réduit la fréquence, le rejeu est la défense). La ligne du `DELETE`
    (`:327`) ne change pas ici (la 15-12b la relit).
    **Aucun** doc-comment « # Ordre des locks » d'un flux (`validate_invoice`,
    `supplier_invoices::create_in_tx`, règlements) ne change. Vérification en T0 :
    `grep -rnE "find_later_closed(_in_tx)?\(" crates/kesh-db/src` — la version `_in_tx` n'est appelée
    que par `update`, `delete_in_tx`, `reopen` et `create` d'exercice ; la version non verrouillante,
    que par le motif d'écran de `GET /journal-entries/{id}` (`journal_entries.rs:1101` sur `5e4bec50`).

### L'écran des exercices

17. **Bouton « Clôturer ».** Pour un exercice ouvert précédé d'un exercice ouvert, le bouton est
    **désactivé** avec un `title` nommant le plus ancien antérieur ouvert — clé
    `fiscal-year-close-blocked-earlier-open` (« Clôturez d'abord l'exercice « { $name } », plus ancien
    et encore ouvert. »), patron exact du bouton « Réouvrir » (`+page.svelte:362-378`, fonction sœur
    `earliestEarlierOpen` alignée sur la requête serveur `ORDER BY start_date ASC LIMIT 1`). Le serveur
    reste l'autorité : `submitClose` (`:215`) traite `EARLIER_FISCAL_YEAR_OPEN` **avant** sa branche
    `ILLEGAL_STATE_TRANSITION` (`:228`) (message du serveur, la modale se ferme, la liste se recharge) —
    test Vitest qui **rougit** si la branche « déjà clôturé » attrape ce code. La boîte de création
    affiche le message de `LATER_FISCAL_YEAR_CLOSED` (chemin générique actuel — à vérifier, pas à
    supposer).
    ⚠️ **`frontend/src/lib/shared/i18n-keys.test.ts` porte des bornes exactes** *(T0, F2 de P4)* —
    sur `5e4bec50` : `sitesTotal: 1915`, `sitesNonResolus: 31`, `relais: 6`, `sitesGabarit: 10`,
    `litterauxMin: 1050` (`:496-500`). Chaque `msg(…)` / `i18nMsg(…)` ajouté à `+page.svelte` bouge
    `sitesTotal` : la borne se **recompte depuis la source** aux deux bornes du diff (règle de
    l'en-tête du fichier : recompter, ne pas ajuster), avec une ligne d'historique, et le décompte va
    au Dev Agent Record.

### Ce qui doit être prouvé, et le reste

19. **Mutations — part A** (chacune jouée, observée rouge, restaurée — ⚠️ toucher le fichier après
    restauration, cargo garde le binaire muté) : (i) retirer la garde de `close` → AC 1 rouge ; (ii)
    rendre la relecture AC 3 (d) non verrouillante → tests 13 a **et** 13 c rouges, sur l'état final ou
    l'issue ; (iii) verrouiller Y avant ses antérieurs — (c) avant (b'), ou (b') retirée (C119) →
    observation seulement, sur une **variante** du
    13 a où W verrouille N, lance la clôture, puis lit les postérieurs : la clôture mutée tient Y et
    attend N, W demande Y — interblocage attendu, dont la victime peut être W, c'est-à-dire une
    réouverture, non rejouée, en 500. (Dans le 13 a tel que monté, W lit les postérieurs **avant** de
    lancer la clôture : la mutation (iii) n'y forme pas de cycle et le test reste vert.) Justification de
    l'ordre, pas une garantie ; (iv) retirer la garde de `create` → AC 5 rouge ; (xi) *(revue P1, A1)*
    rendre la garde de `create` non verrouillante (`FOR UPDATE` retiré de `find_later_closed_in_tx`) →
    test 13 b3 rouge ; (viii) faire passer
    `EARLIER_FISCAL_YEAR_OPEN` par la branche `ILLEGAL_STATE_TRANSITION` de l'écran → Vitest rouge ; (ix)
    appeler `fiscal_years::close` hors de son enveloppe dans la **route** → test HTTP de l'AC 6 rouge
    (500 ou absence du témoin de rejeu) ; (x) appeler `fiscal_years::create` hors de son enveloppe →
    test HTTP de l'AC 6 rouge **si** T0 l'a rendu possible, sinon « tenue par revue », écrit. L'ordre de
    la boucle (b') n'a pas de mutation propre : il n'a d'effet que sur la formation d'un cycle, ce
    qu'observe (iii). Les mutations (v), (vi), (vii) sont dans la 15-12b. Résultat de chaque mutation au Dev
    Agent Record.

21. **Tests existants à reprendre — part A** (ceux que la clôture ordonnée et la garde de création
    changent de sens) — triage au gate, chaque rouge classé (a) scénario désormais impossible par l'API
    → réécrit (ordre de clôture corrigé, ou état posé par SQL **si** le test vise la donnée héritée), ou
    (b) défaut réel → corrigé. **Inventaire par recherche — la clôture** (relevé sur `8f9811d8`,
    reconfirmé sur `5e4bec50` : mêmes fichiers et lignes) :
    `git grep -nE "fiscal_years::close\(|/close\"" -- 'crates/*.rs'` (⚠️ le chemin `'crates/*.rs'`, où
    `*` traverse les `/` dans un *pathspec* git, couvre `src/lib.rs` ; la forme de la P2,
    `'crates/*/src/**/*.rs'`, exigeait un sous-répertoire et ne le rendait pas — R2), hors code de
    production (`routes/fiscal_years.rs:288`, `lib.rs:602`) :
    `fiscal_years_repository.rs` (`reopen_lifo_three_years_intercalated` `:873` — FY1 clos, FY2 ouvert,
    FY3 clos **par l'API** : à poser par SQL ; `reopen_close_concurrent_is_serialized` `:1055` — remplacé
    par le 13 a ; les autres appels `:264-1080`, un exercice ou un ordre déjà croissant : à vérifier) ;
    `fiscal_years_e2e.rs` (`:674-858`, `:1427`, aide `create_and_close_fy` `:1491`/`:1510` et ses
    appelants) ; `reconciliation_e2e.rs` (`an_invoice_of_a_closed_year_paid_the_next_year_is_unreconcilable`
    `:3823`, `a_closed_year_refuses_with_the_reconciliation_text` `:3891` — exercices insérés par SQL puis
    clos par le dépôt) ; `invoice_settlement.rs` (`:1046`, dans `monter`) ; `supplier_invoices_repository.rs`
    (`:1587`, `:1848`, dans `monter` / `monter_achat`) ; `opening_balances_e2e.rs` (`:673`, `:959`,
    `:1090`, `:1247`), `opening_balances_repository.rs` (`:444`), `invoice_echeancier_e2e.rs` (`:720` :
    clôture du seul exercice, a priori sain) ; **modules de test internes à `src/`**, sur la **base
    partagée** (`test_pool`, « première company ») : `journal_entries.rs` `mod tests`
    (`test_create_rejects_closed_fiscal_year` `:3475`, `update_no_op_in_closed_fy_returns_fiscal_year_closed`
    `:5069` — `fiscal_years::close` sur l'exercice que `setup` garantit ouvert : refusée désormais si un
    exercice antérieur ouvert subsiste pour la société). Les tests qui posent l'état par SQL direct
    relèvent de la 15-12b (le filet ; son AC 21 les inventorie par le symptôme).
    **Inventaire par recherche — la création** (R2 ; relevé sur `5e4bec50`) :
    `git grep -nE "fiscal_years::create\(|exercice_de\(|/fiscal-years\"|NewFiscalYear \{" -- 'crates/*.rs'`
    — 21 fichiers, dont 4 de production (`lib.rs`, `routes/fiscal_years.rs`, `routes/onboarding.rs`,
    `kesh-seed/src/lib.rs`) et l'entité. Un site ne change de sens que s'il crée un exercice **antérieur
    à un exercice déjà clos** dans son test. Lus, avec l'ordre des créations et des clôtures de chaque
    test : `fiscal_years_repository.rs` (toutes les créations précèdent les clôtures du test),
    `fiscal_years_e2e.rs` (idem ; `reopen_lifo_blocked_returns_409_distinct_message` crée 2025 puis 2026,
    chacun clos après sa création — ordre croissant), `journal_entry_reversal_e2e.rs` (aide
    `exercice_de`, `:1353`, qui crée **puis** pose le statut : appels en ordre croissant d'année, ou
    exercices créés ouverts avant toute pose — `:1786`, `:1863-1865`, `:2077`, `:2115-2116`, `:2510`, `:2585`) ;
    les autres fichiers ne clôturent aucun exercice postérieur à une création. **Aucun cas trouvé** ;
    T0 le reconfirme sur le `main` du moment, et un rouge `LATER_FISCAL_YEAR_CLOSED` à la création se
    trie comme les autres.
    Les tests de la 15-8a/15-8b qui posent « N+1 clos » par `UPDATE` direct
    (`journal_entries_modification.rs:395`, `:499` ; `journal_entries.rs:3250-3270`, `:3444-3455` — ce
    dernier, le test C-15-8-29, est **inversé** par la 15-12b, AC 10) **restent** ; leur doc-comment « la clôture **en cours** d'un exercice postérieur » est réécrit
    (« transition simulée par SQL : la clôture la refuse depuis la 15-12a ; le test garde la propriété du
    verrou pour les données héritées »).
    ⚠️ **Motifs existants de `attendre_une_requete_en_cours`** : `["ORDER BY start_date ASC", "FOR UPDATE"]`
    (`journal_entries_modification.rs:410`, `:511`) visent `FIND_LATER_CLOSED_SQL`. Après cette fiche,
    **une** autre requête porte ce texte : `FIND_EARLIER_OPEN_SQL` (AC 3 d) ; `OPEN_COVERING_DATE_SQL`
    n'en reçoit pas (C119 révise C110). Relevé : `journal_entries::update` ne l'appelle pas, et aucune
    autre connexion n'exécute de requête pendant ces attentes — aucune confusion aujourd'hui. T0 le
    vérifie ; si les deux sites restent tels quels, l'écrire au Dev Agent Record, sinon passer au motif
    discriminant `"start_date > "`.
    **Côté frontend** *(T0, F2 de P4)* : `i18n-keys.test.ts` (bornes exactes, AC 17) et
    `fiscal-years-page.test.ts` ; `blocker-messages.test.ts` asserte un fragment du repli
    (« exercice postérieur Exercice 2027 est clôturé ») que la réécriture de l'AC 9 conserve.

22. **E2E Playwright** (`frontend/tests/e2e/fiscal-years.spec.ts`) : le test « crée un exercice 2031,
    le renomme puis le clôture » (`:56-96`) **casse** — l'exercice seedé 2020-2030 est ouvert. Il est
    réécrit : créer et renommer 2031 ; le bouton « Clôturer » de 2031 est **désactivé**, son `title`
    contient le nom **complet** de l'exercice seedé, « Exercice CI 2020-2030 » (`test_fixtures.rs:119`,
    `scripts/seed-dev-db.sql:68` — F11 : asserter le nom complet, ou une correspondance partielle
    **écrite comme telle**, jamais un « Exercice CI » qu'un sélecteur exact ne trouverait pas) ;
    clôturer l'exercice seedé ; clôturer 2031 → « Clôturé ». Sélecteurs par
    `data-testid` (ajouter `fiscal-year-close-{id}` au bouton, patron de `fiscal-year-reopen-{id}`).
    Les autres specs ne clôturent qu'un exercice seul (`closeSeededFiscalYearViaApi`) : vérifié par
    `grep -rn "/close" frontend/tests/e2e`.

23. **Documentation — part A.**
    - **Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) : § « Clôture d'un exercice » (`:699-715`)
      — la clôture se fait **dans l'ordre** (refus et bouton désactivé, nommés) ; § « Réouverture »
      (`:717-723`) — rappel que la réouverture va dans l'ordre inverse (la procédure de réparation d'un
      état hérité est de la 15-12b) ; création d'exercice (`:697`) — un exercice ne se crée pas avant un
      exercice clôturé. **Item « Verrouille l'exercice » (`:708`)** (R8) : la proposition « clôturer
      l'exercice suivant fige aussi celui-ci » présente comme disponible un geste que l'AC 1 **refuse** —
      clôturer l'exercice suivant d'un exercice ouvert. Elle est réécrite : un exercice ne se clôture
      qu'après tous ceux qui le précèdent, si bien qu'une écriture se modifie tant que son exercice est
      ouvert ; et si des données antérieures (installation mise à jour, sauvegarde restaurée) portent
      déjà un exercice clôturé après un exercice ouvert, cet exercice ouvert reste figé pour la
      modification et la suppression. **Deux autres phrases** *(T0, F4 de P4)* : `:752` (complément
      des soldes de départ — « Si un exercice suivant est déjà clôturé, son bilan reporté l'est
      aussi », un état que le premier exercice ouvert ne peut plus avoir sous un exercice clos que par
      des données antérieures) et `:762` (« Un exercice antérieur créé \emph{après} la génération
      devient à son tour le premier », désormais refusé si un exercice postérieur est clôturé) —
      incise à chacune. ⚠️ Seuls **restent** la parenthèse « pour la modification et la
      suppression seulement… limite suivie par l'issue #543 » de ce même item et l'avertissement des
      conditions de modification (`:494-500`) : ils décrivent l'état hérité, que seule la 15-12b ferme.
    - **Manuel admin** (`admin-manual.tex`, recalé sur `8f9811d8`) : paragraphe « Réouverture d'un
      exercice clôturé » (`:1381-1382`) — la clôture dans l'ordre, symétrique de la garde de réouverture.
      Les puces `:1920` (« aucun exercice postérieur clôturé », clé `read-write`) et `:1959` (« si
      aucun exercice postérieur n'est clôturé ») restent vraies *(relocalisées par le texte en T0, F7
      de P4)*.
    - **`docs/api-external.md`** — `:484` (tableau des erreurs) : ligne neuve **`409
      EARLIER_FISCAL_YEAR_OPEN`** (`POST /fiscal-years/{id}/close`, `details`) ; `LATER_FISCAL_YEAR_CLOSED`
      gagne la création d'exercice (`POST /fiscal-years`) parmi ses routes ; changement de contrat signalé
      (une intégration qui clôturait hors d'ordre reçoit un refus). Le retrait de « ne garde pas encore
      les autres chemins (#543) » est de la 15-12b.
    - **PDF régénérés** (`make fr` dans `docs/manual/`), contrôlés **aplatis**
      (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`) : les phrases neuves présentes, PDF utilisateur et
      administrateur.
    - **`CHANGELOG.md` `[0.13.0]`** (F10, C123) : entrée sous `### Corrigé` pour le **défaut** (refs
      #543 : l'état « exercice ouvert suivi d'un exercice clôturé » ne s'atteint plus — clôture dans
      l'ordre, création d'exercice, bouton désactivé, message du refus `LATER_FISCAL_YEAR_CLOSED`) **et**
      entrée sous `### Modifié` pour le **changement de contrat d'API** — une intégration par clé qui
      clôturait hors d'ordre reçoit `409 EARLIER_FISCAL_YEAR_OPEN`, une création sous un exercice clos
      `400 LATER_FISCAL_YEAR_CLOSED` (la clôture est ouverte aux clés, Comptable+) —, patron des entrées
      #532 de la même version, qui rangent sous `Modifié` ce qui change un comportement d'API. La 15-12b
      complète l'entrée `Corrigé` (filet, réparation) et réécrit la parenthèse de l'entrée #532.
    - **README** « Feuille de route » : inchangé a priori (correctif) — vérifié.

## Tasks / Subtasks

- [x] **T0 — Relevés au sol sur le `main` du moment** (AC 3, 6, 9, 13, 14, 21) *(ex-T0, part A)*
  - [x] Refaire les numéros de ligne cités, par leur texte.
  - [x] Vérifier que `start_date` n'est écrite nulle part après l'`INSERT` (AC 3 a).
  - [x] Mesurer en MariaDB 10.11 (deux sessions `mariadb` à la main) le mécanisme de l'AC 13 b dans
        ses **deux** configurations (b1, b2), le cycle de l'AC 13 d et le moyen d'en forcer la victime ;
        `EXPLAIN` **descriptif** (C119 : rien n'en dépend) de `FIND_EARLIER_OPEN_SQL` (relecture (d)) et
        de `FIND_LATER_CLOSED_SQL`, **dans les deux régimes** — base éphémère `#[sqlx::test]` à une
        société, base de dev à plusieurs sociétés —, écrit au Dev Agent Record ; un plan de
        `FIND_LATER_CLOSED_SQL` qui ne suit pas `uq_fiscal_years_company_start_date` est signalé à
        l'orchestrateur (AC 3, limite préexistante de `reopen` et de la garde 15-8a).
  - [x] Constater la forme de `PROCESSLIST.INFO` d'une requête préparée (`?` ou valeurs) ; vérifier les
        motifs de l'AC 13 et ceux de `journal_entries_modification.rs:410`, `:511` (AC 21).
  - [x] Refaire le `grep` des sites de l'ancienne clé (AC 9) et chercher toute assertion de test sur
        son texte ou sur celui du repli Rust.
- [x] **T1 — Clôture dans l'ordre (dépôt)** (AC 1, 2, 3, 7, 14) *(ex-T1)*
  - [x] `DbError::EarlierFiscalYearOpen { fiscal_year_id, fiscal_year_name }` (+ `error_code()` si la
        famille l'exige — suivre `DbError::LaterFiscalYearClosed` : variante `errors.rs:562`,
        `error_code()` `:1015` ; `ModificationBlocker::LaterFiscalYearClosed` (`:198-221`) est une
        autre énumération et **ne change pas** — T0, R2 de P4).
  - [x] Constantes `LIST_EARLIER_SQL`, `LOCK_EARLIER_BY_ID_SQL`, `FIND_EARLIER_OPEN_SQL` ; `close`
        réordonné en (a)-(b)-(b')-(c)-(d)-(e), la boucle (b') en Rust (C119) ; `OPEN_COVERING_DATE_SQL`
        **inchangée** (C119 révise C110) ; doc-comments (module, `close`, `FIND_LATER_CLOSED_SQL` et
        `find_later_closed_in_tx` — R5 —, invariant I et sa portée, `DbError::LaterFiscalYearClosed`,
        `reverse_in_tx_inner`) ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (AC 14).
  - [x] Tests : refus, précédence (paires), autre société, rien d'écrit (statut + audit) ; **deux
        exercices antérieurs aux `id` inversés** par rapport à leurs dates (le plus ancien créé en
        second), le plus ancien ouvert → `EarlierFiscalYearOpen` le nomme ; exercice disparu entre (a)
        et (c) → `NotFound` (F6 : W tient Y, la clôture bute en (c), W supprime Y — sans écriture —
        et valide ; jamais de panique).
- [x] **T2 — Création** (AC 5) *(ex-T2)* — garde, doc-comments de `create_for_seed` /
      `create_if_absent_in_tx`, tests de dépôt ; test HTTP `POST /fiscal-years` → `400
      LATER_FISCAL_YEAR_CLOSED` et paire chevauchement (`400 VALIDATION_ERROR`) (avec T3).
- [x] **T3 — HTTP, message et rejeu** (AC 4, 6, 9) *(ex-T3, et la part « message » de l'ex-T4)* —
      mapping 409 dans `kesh-api/src/errors.rs` ; clé `error-later-fiscal-year-closed` ×4 + repli Rust ;
      ancienne clé alignée ×4 + repli de `blocker-messages.ts` (C111) ; enveloppes des deux routes ;
      tests `fiscal_years_e2e.rs` (corps complet : code, message, `details`) ; test HTTP de rejeu de la
      clôture (et de la création si T0 le permet) dans `rejeu_interblocage_e2e.rs` ; registre vert et son
      doc-comment (points (ii), (iii bis), (iv) et (vi) — F4).
- [x] **T4 — Concurrence** (AC 13) *(ex-T6)* — tests (a), (b1), (b2), (c), (d), entrelacements forcés,
      motifs de l'AC 13 ; `reopen_close_concurrent_is_serialized` remplacé par le 13 a.
- [x] **T5 — Écran des exercices : la clôture** (AC 17) *(ex-T7, part A)* — bouton désactivé,
      `earliestEarlierOpen`, `submitClose` ; clés `fiscal-year-close-blocked-earlier-open`,
      `error-fiscal-year-close-earlier-open` ×4 ; `npm run lint-i18n-ownership` ; Vitest
      (`fiscal-years-page.test.ts`).
- [x] **T6 — Tests existants** (AC 21, part A) *(ex-T9, part A)* — triage écrit, test par test, `mod
      tests` de `src/` compris ; doc-comments des tests 15-8a/15-8b.
- [x] **T7 — E2E** (AC 22) *(ex-T10)*.
- [x] **T8 — Mutations** (AC 19, part A) *(ex-T11, part A)*.
- [x] **T9 — Documentation** (AC 23, part A) *(ex-T12, part A)* — manuels + PDF aplatis,
      `api-external.md`, CHANGELOG.
- [x] **T10 — Gates** *(ex-T13)* — base remise à zéro (DROP/CREATE de **ses** bases, jamais de
      redémarrage du conteneur) ; `scripts/test-fast.sh` complet (story `kesh-db` : gate complet même en
      cours de boucle) ; frontend (`check`, `lint-i18n-ownership`, `test:unit`, `build`) ; E2E complet au
      **dernier commit de code**, jugé fichier par fichier contre `docs/testing.md` § « Les échecs
      attendus ». Ne déclarer que ce qui a tourné.

## Dev Notes

### Pourquoi la clôture, et pas seulement la garde d'écriture

L'issue propose deux voies. Garder seulement l'écriture laisserait l'état fautif atteignable par un clic,
et imposerait une garde sur chaque flux — c'est ce que la 15-8a a fait pour deux routes, et l'issue est
née de ce qu'il en restait dix-neuf. Ordonner la clôture rend l'état **inatteignable** à partir d'un état
sain : la garde d'écriture de la 15-12b n'est plus qu'un filet pour les données héritées.

### Pourquoi la clôture verrouille ses antérieurs (l'invariant sous concurrence)

L'étape (a) de `close` est une lecture cohérente qui fixe la vue de sa transaction ; si une réouverture
de N est en cours (non validée), cette vue montre N **clos** ; une relecture (d) non verrouillante y lirait
« aucun antérieur ouvert », la clôture de L passerait après la validation de la réouverture, et l'état
« N ouvert, L clos » serait atteint **par l'API**. Le verrou (b') attend la réouverture, et la relecture
verrouillante (d) lit N ouvert. Test 13 a, mutation (ii). C'est la propriété (α) sur laquelle la 15-12b fonde la preuve de son
filet non verrouillant.

⚠️ **Angle mort assumé — la restauration en vol** (C100). `admin::full_import` ne se sérialise qu'avec
les autres imports (verrou `_kesh_version id = 1 FOR UPDATE`, `routes/admin.rs:173-179`), **pas** avec la
clôture ni les écrivains : c'est le cas, préexistant et plus large, de toute écriture en vol pendant une
restauration. Écrit au doc-comment du module (la 15-12b l'écrit aussi à la ligne `admin::full_import` du
registre).

### Ordre des verrous — ce qui change, ce qui ne change pas

- **Change** : `close` prend les antérieurs connus un par un par clé primaire, dans l'ordre
  chronologique fixé par le code, puis Y, puis relit les antérieurs ouverts sous verrou (C119) ;
  `create` prend en plus les postérieurs (`find_later_closed_in_tx`) après ses deux pré-contrôles.
- **Ne change pas, et c'est voulu** : `find_open_covering_date` (C119 révise C110 — un `ORDER BY`
  n'aurait fixé que son résultat, déjà unique, non l'ordre de ses verrous).
- **Ne change pas** : `update`, `delete_in_tx`, `reopen`, les flux d'écriture (la 15-12b n'y ajoute
  qu'une lecture non verrouillante et la levée d'une condition).
- **L'ordre réel des autres flux** (C100) : la contre-passation (`reverse_in_tx_inner`) verrouille
  l'origine et son exercice par clé primaire, puis `find_open_covering_date(today)` parcourt l'index
  depuis le premier exercice et verrouille les exercices antérieurs à l'origine **après** elle ; les
  annulations (`supplier_invoices::cancel_in_tx`, `cancel_settlement_in_tx`,
  `invoice_settlements_write::cancel_settlement_in_tx`, `reconciliation_cancel::cancel_in_tx`)
  verrouillent d'abord la pièce, puis l'écriture d'origine et son exercice, puis passent par ce socle.
  Règlements, paiements et saisie prennent l'exercice d'abord (`find_open_covering_date`, parcours dans
  l'ordre du plan, puis l'exercice par clé primaire, déjà tenu) et rien d'autre dans `fiscal_years`.
- Interblocages possibles, tous résolus par le rejeu des **deux** côtés : création contre clôture
  (AC 13 b, si la mesure en montre un) ; **contre-passation et annulations contre clôture** (AC 3,
  test 13 d) ; **contre-passation et annulations contre création** (AC 3, F5) — les cinq routes sont
  `Rejouee`, la clôture et la création le deviennent (AC 6).
- **Hors de cette story, écrit** : la réouverture, non rejouée, peut être la victime d'un cycle avec
  une contre-passation (elle tient Y-1 et demande les postérieurs ; la contre-passation tient une
  origine postérieure et parcourt depuis le premier exercice). Préexistant, sans rapport avec la
  clôture dans l'ordre : nommé au point (iv) du registre (AC 6), qui le porte déjà pour `/reopen`.
- **Hors de cette story, écrit — cycle clôture ↔ renommage** *(T0, R9 de P4)* : `update_name` tient
  l'exercice renommé Y (`LOCK_IN_COMPANY_SQL`), puis verrouille l'exercice **homonyme**
  (`… WHERE company_id = ? AND name = ? AND id <> ? FOR UPDATE`). Renommer Y au nom d'un antérieur M
  tient Y et demande M ; `close(Y)` tient M (étape (b')) et demande Y : cycle, **nouveau** (la clôture
  ne verrouillait pas ses antérieurs). La victime peut être le renommage, route
  `PUT /fiscal-years/{id}` **non rejouée** → 500, sur un renommage que le pré-contrôle aurait refusé
  de toute façon (`FY_NAME_DUPLICATE`). Très rare et bénin ; nommé au point (iv) du registre avec
  `/reopen`, pas de code.
- **Attente bornée** *(T0, F6 de P4)* : la clôture attend désormais en étape (b') tout écrivain en vol
  qui tient un exercice antérieur ; au-delà de `innodb_lock_wait_timeout` (50 s), `1205` → 500, non
  rejoué. Préexistant pour la contre-passation, nouveau pour la clôture ; accepté (une transaction
  d'écriture dure quelques millisecondes).

### Codes et messages

| Situation | Code | HTTP | Clé |
|---|---|---|---|
| Clôture avec un antérieur ouvert | `EARLIER_FISCAL_YEAR_OPEN` (neuf) | 409 | `error-fiscal-year-close-earlier-open` |
| `PUT` / `DELETE` d'une écriture, création d'exercice, sous un postérieur clos | `LATER_FISCAL_YEAR_CLOSED` | 400 | `error-later-fiscal-year-closed` (neuve) |
| Fiche d'écriture, motif du blocage (écran) | — | — | `journal-entries-modify-blocked-later-fiscal-year-closed` (gardée, prescription alignée) |
| Bouton « Clôturer » désactivé | — | — | `fiscal-year-close-blocked-earlier-open` |

409 pour la clôture, comme les autres refus de transition de cet écran (déjà clos, réouverture LIFO) ;
400 pour `LATER_FISCAL_YEAR_CLOSED`, inchangé (15-8a : l'état d'un exercice, pas un conflit sur
l'objet).

### Ce qui doit être préservé

- Le comportement de la 15-8a/15-8b (`PUT`, `DELETE`, écran de la fiche) — codes, `details`, ordre ;
  seul le **texte** du refus change (AC 9).
- Les tests qui posent l'état fautif par SQL : ils deviennent les seuls moyens de l'exercer, ne pas
  les « simplifier » en passant par l'API.
- `create_for_seed`, `create_if_absent_in_tx` : inchangés (AC 5).
- `admin::full_import` : accepte une sauvegarde fautive (C89).

### Hors périmètre, et écrit

- Le filet, le lot de rapprochement, le bandeau d'état hérité, la réparation : **15-12b**.
- Toute clôture « réelle » (écritures de clôture, report physique) — Kesh clôture par verrou.
- Une route de réparation automatique — la réparation est comptable.
- **Les autres textes qui prescrivent une réouverture sans dire dans quel ordre** (F8, C122) — même
  symptôme que celui que C111 corrige pour l'ancienne clé, mais préexistant et hors du défaut de #543 :
  ils disent « un administrateur doit rouvrir l'exercice », geste que la garde LIFO refuse dès qu'un
  exercice plus récent est clos. Relevé par la valeur sur `5e4bec50` (`grep -rnE "rouvrir
  l'exercice|doit d'abord le rouvrir|rouvrez-le"` hors fiches et PDF) : fr-CH
  `settlement-cancel-blocked-fiscal-year-closed` (`:812`), `reconciliation-cancel-blocked-fiscal-year-closed`
  (`:820`), `supplier-invoices-cancel-blocked-fiscal-year-closed` (`:1949`), `error-fiscal-year-reopen-blocked`
  (`:922`, « rouvrez-le d'abord » sans dire lequel — le bouton le nomme, l'API non),
  `error-opening-balances-first-year-closed` (`:959`) et leurs trois autres locales ; replis Rust
  (`kesh-api/src/errors.rs:2978`, `:3509`, `:3546`, et — complétés en T0, R6 de P4 —
  `kesh-api/src/routes/fiscal_years.rs:178`, `routes/opening_balances.rs:205`, `:406`) et frontend
  (`settlement-cancel-blocked.ts:39`, `reconciliation-cancel.ts:60`, `invoice-cancel.ts:43`) ; manuel
  utilisateur (`:719` — « peut rouvrir l'exercice directement », légitime pour la procédure —,
  `:1405`, `:1427`, `:1714`, `:2242`). ⚠️ Des tests Vitest **figent** le texte courant
  (`InvoiceSettlements.test.ts`, `reconciliation-cancel.test.ts`, `CancelReconciliationDialog.test.ts`,
  `settlement-cancel-blocked.test.ts`, `invoice-settlements-page.test.ts`) : l'issue les nommera. **Issue
  [#569](https://github.com/guycorbaz/kesh/issues/569)** (P3 : texte qui décrit un geste que le code
  refuse dans un cas), qui les alignera sur la prescription de C111 — « en commençant par le plus
  récent ». Les trois replis Rust d'`errors.rs` (règlement, rapprochement, facture fournisseur) la
  citent en commentaire depuis la revue P1 (A3/E3).

### Dérogation règle de splitting

*(Section rédigée en P3 — F9, décision de l'orchestrateur, C121.)*

- **Le critère franchi** : le premier critère de la § « Règle de splitting préventif » du `CLAUDE.md`
  (plus de 5 modules). Recompté à la découpe : **8** modules de premier niveau —
  `kesh-db/repositories/fiscal_years`, `kesh-db/errors`, `kesh-db/repositories/journal_entries` (un
  doc-comment, AC 14) ; `kesh-api/routes/fiscal_years`, `kesh-api/errors` (mapping 409, clé de l'AC 9) ;
  `kesh-i18n` ; `frontend/routes/(app)/settings/fiscal-years` ; `frontend/features/journal-entries`
  (repli de `blocker-messages.ts`, AC 9). Hors décompte : tests, doc (dont
  `MULTI-TENANT-SCOPING-PATTERNS.md`). La remédiation P3 (C119) ne change pas ce compte : la clôture
  « un par un » reste dans `fiscal_years`, et `OPEN_COVERING_DATE_SQL` n'est plus touchée.
- **Pourquoi l'exception codifiée s'applique** : la story a **déjà** été découpée une fois (C107), selon
  la seule couture naturelle — l'ordre (cette fiche) et le filet (15-12b). Un second découpage
  séparerait l'ordre de `close` de sa concurrence (AC 3, AC 13 : les tests à deux connexions sont la
  seule preuve de l'ordre), ou la garde de son écran et de son message (AC 1, 4, 9, 17 : le bouton
  désactivé et le refus serveur se testent ensemble, Vitest et E2E compris) : des sous-stories qui ne
  seraient **pas testables isolément** — l'exception que le `CLAUDE.md` écrit (« merges intermédiaires
  impossibles à tester en isolation »). Trois des huit modules ne portent qu'un ou deux sites (un
  doc-comment, un repli, des clés).
- **Risque accepté** : une revue de code qui embrasse huit modules relit moins finement chacun ; la
  remédiation d'une passe peut introduire un défaut dans un module que la passe suivante ne relit pas.
  Mitigation : la passe ciblée sur chaque remédiation (§ *La passe ciblée*), et les inventaires par
  recherche (AC 21, AC 23) refaits à chaque passe.
- **Signal déclaré au Project Lead** (Guy), dans cette fiche et au registre (C121).

### Tests — ce qui rendrait un test vert sans rien prouver

- Un test de concurrence qui n'asserte que « pas de panique » (le test actuel `:1055` en est un : il
  admet l'état fautif comme issue).
- Une attente qui ne distingue pas « en cours » de « bloquée » prise pour une preuve de blocage : c'est
  le montage qui fait l'attente (AC 13).
- Une enveloppe de rejeu écrite par le test lui-même, prise pour la preuve de celle de la route (R2).
- Une mutation restaurée par `cp`/`mv` sans `touch` : le binaire muté reste en cache.

### Fichiers touchés (prévision)

`crates/kesh-db/src/repositories/fiscal_years.rs`, `crates/kesh-db/src/repositories/journal_entries.rs`
(doc-comment de `reverse_in_tx_inner`), `crates/kesh-db/src/errors.rs`,
`crates/kesh-db/tests/fiscal_years_repository.rs`, `crates/kesh-db/tests/journal_entries_modification.rs`
(doc-comments), `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/fiscal_years.rs`,
`crates/kesh-api/tests/fiscal_years_e2e.rs`, `crates/kesh-api/tests/rejeu_interblocage_e2e.rs`,
`crates/kesh-api/tests/audit_route_registry.rs` (doc-comment, points (iv) et (vi)), tests listés à
l'AC 21, `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`,
`frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` et son test,
`frontend/src/lib/features/journal-entries/blocker-messages.ts`,
`frontend/tests/e2e/fiscal-years.spec.ts`, `docs/manual/fr/{user,admin}-manual.{tex,pdf}`,
`docs/api-external.md`, `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, `CHANGELOG.md`.
**Aucune migration** (P1-P8 sans objet).

### References

- Issue #543 et ses deux commentaires (15-8a, 15-8b).
- `crates/kesh-db/src/repositories/fiscal_years.rs` — `OPEN_COVERING_DATE_SQL` `:528`,
  `find_open_covering_date` `:550`, `find_later_closed_in_tx` `:646`, `FIND_LATER_CLOSED_SQL` `:668`,
  `close` `:773`, `reopen` `:872`.
- `crates/kesh-db/src/repositories/journal_entries.rs` — `reverse_in_tx_inner` `:2175` (`:2300`).
- `crates/kesh-api/src/errors.rs:2815-2842` ; `crates/kesh-api/src/routes/fiscal_years.rs:279`
  (`close_fiscal_year`), `:218` (`create_fiscal_year`).
- `crates/kesh-api/tests/audit_route_registry.rs` ; `crates/kesh-api/tests/rejeu_interblocage_e2e.rs`
  (tests 2 à 5, `transaction_lourde`, `capture_rejeu`) ; `crates/kesh-db/src/test_fixtures.rs:539-590`.
- `crates/kesh-report/src/balance_sheet.rs:1-20` (bilan cumulatif).
- Fiches `15-12b-filet-sous-un-bilan-clos.md` (suite), `15-12-cloture-dans-l-ordre.md` (index ; version
  complète au commit `dae3a618`), `15-8a-modifier-une-ecriture.md`, `15-8b-supprimer-une-ecriture.md`,
  `15-5e1-socle-rejeu.md`, `15-1a-socle-lettrage.md` (frontière).
- `CLAUDE.md` § Pattern batch, § Règle de splitting, § Propagation post-patch.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (agent de développement, worktree `/home/gcorbaz/devel/kesh-15-12a`).

### Mesures T0 (2026-10-09, MariaDB 10.11.16, base de mesure `kesh_1512a_t0`)

- **Isolement et délai** : `REPEATABLE-READ`, `innodb_lock_wait_timeout = 50`.
- **`EXPLAIN` (descriptif, C119 — rien n'en dépend)** de `FIND_EARLIER_OPEN_SQL` (étape (d)) et de
  `FIND_LATER_CLOSED_SQL`, chacun en `… FOR UPDATE`, ainsi que de `LIST_EARLIER_SQL` :
  - régime « une société » (quatre exercices, le plus ancien créé en dernier — `id` inversés), avant et
    après `ANALYZE TABLE` : `type = range`, `key = uq_fiscal_years_company_start_date`, `Extra = Using
    index condition; Using where` (liste : `Using where; Using index`) — **pas de `filesort`** ;
  - régime « plusieurs sociétés » (six sociétés, vingt exercices chacune), avant et après `ANALYZE` :
    même plan, `rows` 15-16 (la société seule).
  ⇒ `FIND_LATER_CLOSED_SQL` **suit l'index** dans les deux régimes : la limite préexistante de `reopen`
  et de la garde 15-8a (plan qui ne suivrait pas l'index) n'est **pas** observée ; rien à signaler.
- **AC 13 b — création contre clôture** (trois sessions `mariadb`, la clôture bloquée en étape (c) par
  une session qui tient Y, la création lancée 0,5 s après) :
  - **b1** (aucun exercice antérieur à X) : **sérialisation** — la création bute sur Y dans
    `find_later_closed_in_tx`, attend la clôture, lit Y clos (refus `LaterFiscalYearClosed`).
  - **b2** (M clos antérieur à X) : **interblocage** (1213) — victime observée : la création ; la
    clôture valide. (Mesure faite d'abord sans l'étape (b') par erreur de script : la victime était
    alors la clôture, en étape (d) ; refaite avec (b').)
- **Preuve HTTP de l'enveloppe de création** (montage de l'AC 6) : W alourdie (500 lignes) tient Y ; la
  création bute en `find_later_closed_in_tx` ; W demande M → **la création est la victime, W obtient
  M**. Déterministe sur deux essais. ⇒ test HTTP écrit, mutation (x) testée.
- **AC 13 d — clôture contre contre-passation** (M clos, N ouvert, T ouvert couvrant le jour) : W tient
  N ; la clôture prend M (b') et bute sur N (c) ; W lance `OPEN_COVERING_DATE_SQL … FOR UPDATE` :
  **interblocage, victime la clôture**, que W soit alourdie ou non ; W obtient M.
- **Écritures de `fiscal_years` hors tests** (`git grep`) : `INSERT` (`create`, `create_if_absent_in_tx`,
  `insert_fiscal_year_in_tx`), `UPDATE` de statut (`close`, `reopen`) et de nom (`update_name`),
  `DELETE` (`kesh-seed` `reset_demo`) ; les autres (`accounts.rs`, `invoices.rs`, `journal_entries.rs`)
  sont dans des `mod tests`. Aucun `UPDATE` de `start_date`.
- **`find_later_closed(_in_tx)?`** : la version verrouillante est appelée par `reopen`
  (`fiscal_years.rs:911`), `journal_entries::update` (`:1365`) et `delete_in_tx` (`:1682`) ; la version
  libre par `GET /journal-entries/{id}` (`:1101`). Conforme à l'AC 14.
- **Motifs existants** `["ORDER BY start_date ASC", "FOR UPDATE"]` (`journal_entries_modification.rs:410`,
  `:511`) : `journal_entries::update` / `delete` n'appellent pas `close`, et aucune autre connexion
  n'exécute de requête pendant ces attentes — **laissés tels quels** (AC 21).
- **Ancienne clé (AC 9)** : sites relocalisés (fr-CH `:376`, autres `:382`, `blocker-messages.ts:94`,
  repli Rust `errors.rs:2852`) ; seule assertion de test sur son texte :
  `blocker-messages.test.ts:27`, fragment conservé par la réécriture.
- **Forme de `PROCESSLIST.INFO`** (constatée en T4, dans un test à deux connexions) : le texte de la
  requête préparée **avec ses `?`**, non substitués (« … WHERE company_id = ? AND start_date <= ? AND
  end_date >= ? LIMIT 1 FOR UPDATE »). Les motifs, qui évitent `?`, valent dans les deux cas.
- ⚠️ **Écart de mesure 13 b1** : à la main (T0), la création passait `find_overlapping` et butait dans
  sa garde ; **dans le test** (`#[sqlx::test]`), elle bute dès `find_overlapping`, sur Y (borne de son
  parcours), que la session W tient. Même issue (sérialisation, création refusée en une tentative) ;
  le test fait foi et son doc-comment le dit. En 13 b2 comme dans le montage HTTP de l'AC 6, la
  création bute aussi dans `find_overlapping` (sur M en b2) : les motifs des tests visent ce
  pré-contrôle (`["end_date >= ", "LIMIT 1 FOR UPDATE"]`), ou un motif tolérant au test HTTP.

### Debug Log References

- Gate backend : `target/gate-logs/15-12a-gate-backend.log` (worktree) ; frontend :
  `target/gate-logs/15-12a-gate-frontend.log` ; E2E : `target/gate-logs/15-12a-e2e.log`, backend
  `target/gate-logs/15-12a-backend-e2e.log`.

### Completion Notes List

**Commits** (sur `5e4bec50` + planification `77194098`) : `15144b1f` (T0), `1d17cc2c` (dépôt, T1/T2/T4),
`15814d29` (HTTP, message, rejeu, T3), `a29cc474` (écran, T5), `f0bee2a0` (E2E et doc-comments des tests
15-8a/15-8b, T6/T7), `2c69ce4b` (documentation, T9).

**Ce qui est livré** :
- `DbError::EarlierFiscalYearOpen` ; `close` en étapes (a)-(e) — constantes `START_DATE_IN_COMPANY_SQL`,
  `LIST_EARLIER_SQL`, `LOCK_EARLIER_BY_ID_SQL`, `LOCK_IN_COMPANY_SQL` (partagée avec `reopen` et
  `update_name`, qui l'écrivaient chacune à la main), `FIND_EARLIER_OPEN_SQL` ; `OPEN_COVERING_DATE_SQL`
  inchangée (C119).
- `create` : garde `find_later_closed_in_tx` après les pré-contrôles ; `create_for_seed` et
  `create_if_absent_in_tx` inchangés, leur doc-comment dit pourquoi.
- HTTP : `409 EARLIER_FISCAL_YEAR_OPEN` ; `400 LATER_FISCAL_YEAR_CLOSED` à la création, message propre
  (`AppError::FiscalYearBeforeClosedYear`, clé `error-fiscal-year-create-later-closed` — C-15-12a-1) ;
  clé neuve `error-later-fiscal-year-closed` au mapping global ; corps `LATER_FISCAL_YEAR_CLOSED` construit
  par une seule fonction (`later_fiscal_year_closed_response`) pour les deux messages ; ancienne clé
  alignée aux quatre locales et au repli de `blocker-messages.ts`. `grep` de l'ancienne prescription
  (`rouvrir cet exercice|reopen that fiscal year|wieder eröffnen;|riaprire quell`) hors fiches et PDF :
  **0 résultat**.
- Enveloppes `retry_on_deadlock("fiscal_years::create" | "fiscal_years::close", …)` dans les routes ;
  doc-comment du registre (points (ii), (iii bis), (iv) — cycle renommage/clôture nommé, `/close` retirée
  — et (vi) : **quatre** routes) ; en-tête de `rejeu_interblocage_e2e.rs`.
- Écran : `earliestEarlierOpen`, bouton désactivé (`data-testid="fiscal-year-close-{id}"`, `title`),
  `submitClose` traite `EARLIER_FISCAL_YEAR_OPEN` avant `ILLEGAL_STATE_TRANSITION` ; la boîte de
  création affiche `err.message` (chemin générique **vérifié** dans `submitCreate`).
- Doc-comments : module `fiscal_years.rs` (invariant I, portée, angle mort de la restauration, ordre des
  verrous), `close`, `create`, `FIND_LATER_CLOSED_SQL`, `find_later_closed_in_tx`,
  `DbError::LaterFiscalYearClosed`, `reverse_in_tx_inner` ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md`.

**Tests ajoutés** (recomptés `grep -cE '#\[(sqlx|tokio)::test'` aux deux bornes `5e4bec50` → `HEAD`) :
`fiscal_years_repository.rs` 40 → 53 (+14 neufs, −1 remplacé : `reopen_close_concurrent_is_serialized`
→ 13 a) ; `fiscal_years_e2e.rs` 41 → 44 (+3) ; `rejeu_interblocage_e2e.rs` 7 → 9 (+2, tests 9 et 10) ;
Vitest `fiscal-years-page.test.ts` 6 → 9 (+3). Backend : +18, cohérent avec le gate (2879 selon le
registre de sprint, compté sur l'état rebasé sur `8f9811d8` et **non rejoué** sur `5e4bec50` — précision
de la revue P1, A7 → 2897).

**Mutations (AC 19), jouées et constatées** (chacune restaurée par copie puis `touch`) :
- (i) garde de `close` neutralisée → **3 rouges** : `close_is_refused_while_an_earlier_year_is_open`,
  `close_names_the_oldest_earlier_open_year_whatever_the_ids`, `close_sees_a_concurrent_reopening_of_an_earlier_year`.
- (ii) `FIND_EARLIER_OPEN_SQL` sans `FOR UPDATE` → **2 rouges** : 13 a et 13 c.
- (iii) étape (b') retirée → **observation**, sur une variante temporaire du 13 a (W verrouille N, lance
  la clôture, puis lit les postérieurs) : avec le code réel, aucun interblocage ; sous la mutation, **W
  — la réouverture — reçoit un 1213** (soit un 500 côté route, `reopen` n'étant pas rejouée). Le 13 a tel
  que monté reste vert, comme prévu. Variante retirée.
- (iv) garde de `create` neutralisée → **3 rouges** : `create_is_refused_before_a_closed_year`, 13 b1, 13 b2.
- (viii) `EARLIER_FISCAL_YEAR_OPEN` routé dans la branche « déjà clôturé » → Vitest **rouge**
  (« refus serveur EARLIER_FISCAL_YEAR_OPEN… »).
- (ix) enveloppe de la clôture ramenée à **une** tentative (`retry_on_deadlock_with(…, 1, …)`, c'est-à-dire
  sans rejeu, le nom restant visible du volet (c)) → test HTTP 9 **rouge** ; (x) idem pour la création →
  test HTTP 10 **rouge**. ⚠️ Forme de la mutation choisie pour garder l'appel nommé : un appel nu ferait
  rougir le test **et** changerait le texte que lit le registre ; l'effet mesuré est le même (aucun rejeu).
- Après restauration : fichier de tests du dépôt 53/53, tests HTTP de rejeu 2/2, Vitest 9/9.

**Triage des tests existants (AC 21)** — au gate complet, **aucun rouge** à trier hors des deux tests
repris d'avance : `reopen_lifo_three_years_intercalated` (FY3 clos **par SQL**, catégorie (a), état
hérité) et `reopen_close_concurrent_is_serialized` (remplacé par le 13 a). Les autres sites de
l'inventaire (`fiscal_years_e2e.rs`, `reconciliation_e2e.rs`, `invoice_settlement.rs`,
`supplier_invoices_repository.rs`, `opening_balances_*`, `invoice_echeancier_e2e.rs`, les deux tests du
`mod tests` de `journal_entries.rs` sur la base partagée) clôturent dans l'ordre ou un exercice seul :
verts. Aucune création ne rougit en `LATER_FISCAL_YEAR_CLOSED`. Doc-comments des deux tests 15-8a/15-8b
« clôture **en cours** » réécrits (transition simulée par SQL). Motifs `["ORDER BY start_date ASC", "FOR
UPDATE"]` de `journal_entries_modification.rs` **laissés tels quels** (aucune clôture ne tourne pendant
leurs attentes).

**Documentation** : manuel utilisateur (création, clôture dans l'ordre, item « Verrouille l'exercice »
réécrit, réouverture, `:752`, `:762`), manuel administrateur (paragraphe « Clôture dans l'ordre »),
`api-external.md` (ligne `409 EARLIER_FISCAL_YEAR_OPEN`, création parmi les routes de
`LATER_FISCAL_YEAR_CLOSED`, rejeu des deux routes), CHANGELOG `[0.13.0]` (`Modifié` : contrat d'API ;
`Corrigé` : le défaut). PDF régénérés (`make fr`), **contrôlés aplatis** : les phrases neuves présentes
dans les deux PDF ; la phrase réfutée « clôturer l'exercice suivant fige aussi celui-ci » absente (0).
Un code `\texttt` qui débordait de la marge dans le PDF administrateur (tronqué en
`EARLIER_FISCAL_YEAR_OP`) a été rendu sécable (`\allowbreak`) et revérifié. La brochure, régénérée par
`make fr` sans changement de source, n'est **pas** commitée. README inchangé (correctif, vérifié).

**Gates au dernier commit de code** (`2c69ce4b` porte la doc et les PDF ; le dernier commit de code est
`f0bee2a0` — aucun code n'a changé depuis, la suite E2E a tourné sur `2c69ce4b`) :
- bases `kesh_1512a` / `kesh_e2e_1512a` remises à zéro (DROP/CREATE, migrations, seed) ;
- `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest) : **2897 / 2897, 4 ignorés** ;
- frontend : `npm run check` 0 erreur (27 avertissements préexistants), `lint-i18n-ownership` PASS,
  `test:unit` **1094 / 1094** (112 fichiers), `build` OK ;
- E2E complet (backend `:3012` sur `kesh_e2e_1512a`, secrets `openssl rand`, montage complet de
  `docs/testing.md` — SMTP, inbox, documents ; `smtpConfigured:true`), lancé vers 00:44 UTC (backend
  démarré à 00:18 UTC, journal clos à 00:59 UTC ; heure corrigée en revue P1, A7) : **245
  passés, 9 échecs, 19 ignorés** (14,7 min). Les 9, jugés **fichier par fichier** contre
  `docs/testing.md` § « Les échecs attendus » : sept KF-029 (#97 — `mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) et deux KF-045
  (#421 — `invoices.spec.ts:415`, `:439`, attendus **avant 12:00 UTC**). Aucun huitième variable,
  aucun échec hors liste. Le scénario réécrit `fiscal-years.spec.ts:59` (clôture dans l'ordre) passe.
  Backend arrêté par son PID.

**Points à vérifier en revue** :
- La garde de création (AC 5) n'est pas seule à verrouiller : en 13 b1/b2 et au test HTTP, c'est
  `find_overlapping` qui bute — à lire contre l'AC 13 b (« la création postérieure attend Y »).
- La mutation (ix)/(x) a été jouée par réduction à une tentative, non par appel nu (voir plus haut).
- R7 de P4 (fiche 15-1a) reste à l'orchestrateur ; l'issue des « autres textes de réouverture sans
  ordre » (C122) est **#569**.

### Remédiation de la revue de code P1 (2026-10-09)

Rapports : `target/gate-logs/15-12a-review-p1-{B,E,A}.md` (worktree). Choix : C-15-12a-2, -3, -4 ;
suivi #569 ajouté à C122.

- **A1 (MEDIUM) = B-2 = E1** — test **13 b3** `close_waits_for_a_creation_whose_guard_holds_the_later_year`
  (W : garde de `create` puis `INSERT`, sans `find_overlapping` ; clôture attendue en (c) ;
  `EarlierFiscalYearOpen` nommant X). **Mutation (xi)** — `FOR UPDATE` retiré de
  `find_later_closed_in_tx` — **jouée : 1 rouge, le 13 b3** (« état fautif atteint : X ouvert sous Y
  clos ») ; 13 b1 et 13 b2 **verts** sous elle, ce qui confirme le constat de la revue. Restaurée par
  copie puis `touch`. Fiche (AC 3, AC 6, AC 13 b, AC 19), commentaire du site de la garde
  (`fiscal_years.rs`), doc de `create`, de `close` (« ce que cette forme perd ») et de
  `find_later_closed_in_tx`, doc-comments des 13 b1 / b2 : chacun dit ce qu'il prouve. Branche
  inatteignable de `jamais_x_ouvert_sous_y_clos` commentée ; le fantôme validé entre (a) et (c) écrit
  « sans test ».
- **B-1 (MEDIUM) = E4** — `error-later-fiscal-year-closed` (4 locales + repli Rust) ne nomme plus que la
  modification et la suppression ; assertion ajoutée à
  `a_closed_later_year_freezes_the_entry_until_reopened` de `journal_entry_reversal_e2e.rs` (contient « ne peut être modifiée ni
  supprimée », pas « enregistr »). Grep par la valeur (« enregistr », « recorded », « erfasst »,
  « registrat ») : plus aucun site hors fiches et registre. La 15-12b élargira le texte (C-15-12a-3).
- **B-5** — deux clés voisines gardées, pourquoi écrit au mapping ; registre italien tranché par le
  glossaire (tutoiement des clés de l'écran des exercices), C-15-12a-3.
- **B-3** — infobulle portée par une enveloppe `<span title>` (bouton désactivé = `pointer-events:
  none`), pour « Clôturer » **et** « Réouvrir » ; Vitest et E2E assertent l'enveloppe. Mutation
  « `title` retiré de l'enveloppe » jouée : Vitest rouge.
- **A6** — Vitest « refus serveur LATER_FISCAL_YEAR_CLOSED à la création : message affiché dans la
  boîte » ; mutation « `createError = err.message` remplacé » jouée : rouge.
- **E5** — `create_ignores_the_closed_years_of_another_company` (mutation « `company_id` neutralisé dans
  `FIND_LATER_CLOSED_SQL` » jouée : seul ce test rougit), `create_right_after_a_closed_year_is_allowed`
  (frontière : antérieur clos adjacent ; le cas postérieur adjacent est déjà
  `create_is_refused_before_a_closed_year`), HTTP `both_refusals_reach_a_read_write_api_key` (409 et 400
  par clé `read-write`).
- **A2** — références à `reopen_close_concurrent_is_serialized` (`opening_balances_e2e.rs`,
  `opening_balances_repository.rs` ×2) : « l'ancien …, remplacé par la Story 15-12a ».
- **A3 / E3** — #569 cité dans la fiche (Hors périmètre, Points à vérifier), au registre (C122) et en
  commentaire aux trois replis Rust d'`errors.rs` (règlement, rapprochement, facture fournisseur).
  #568 (prédicteurs, 15-12b) non cité dans `api-external.md` : hors de ce que dit la ligne.
- **A4** — `api-external.md:253` borné aux chemins d'écriture d'une écriture comptable.
- **A7** — heure du run E2E corrigée (vers 00:44 UTC), origine du 2879 précisée.
- **E2** — doc de module : cycle réouverture ↔ contre-passation d'un postérieur nommé (non rejouée, 500
  possible, préexistant). **B-6** — coût de (b') écrit au doc de `close`. **B-4** — assertion
  disjonctive du 13 d expliquée dans son doc-comment.
- **A5 / B-7** — écrits, sans changement : (ix)/(x) jouées par réduction à une tentative, l'appel nu
  reste une variante **non jouée** ; le cycle `reopen` ↔ contre-passation est préexistant (E2).
- **Non traité** : aucun.

**Tests ajoutés par cette remédiation** (recomptés `grep -cE '#\[(sqlx|tokio)::test'`, `1f477526` →
arbre de travail) : `fiscal_years_repository.rs` 53 → 56 (+3), `fiscal_years_e2e.rs` 44 → 45 (+1) ;
Vitest `fiscal-years-page.test.ts` 9 → 10 (+1). Backend +4 (2897 → 2901), frontend +1 (1094 → 1095).

**Gates (arbre de la remédiation, commité tel quel)** :
- `cargo fmt --all -- --check` vert ; `cargo clippy --workspace --all-targets -- -D warnings` vert ;
- bases `kesh_1512a` / `kesh_e2e_1512a` remises à zéro (DROP/CREATE, migrations, seed) ;
  `scripts/test-fast.sh` : **2901 / 2901, 4 ignorés** (`target/gate-logs/15-12a-gate-backend-p1.log`) ;
- frontend : `check` 0 erreur (27 avertissements préexistants), `lint-i18n-ownership` PASS,
  `test:unit` **1095 / 1095** (112 fichiers), `build` OK (`…/15-12a-gate-frontend-p1.log`) ;
- E2E complet (backend `:3012`, `kesh_e2e_1512a`, montage complet, `smtpConfigured:true`), vers 01:34 (fin à
  01:44 UTC) : **245 passés, 9 échecs, 19 ignorés** (10,2 min). Les 9, fichier par fichier contre
  `docs/testing.md` : sept KF-029 (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`,
  `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) et deux KF-045 (`invoices.spec.ts:415`, `:439`, run
  avant 12:00 UTC). Aucun hors liste ; `fiscal-years.spec.ts:59` vert. Backend arrêté par son PID.
- Manuels non touchés (le texte promettait déjà l'infobulle, désormais tenue) : PDF non régénérés.

### Clôture — rebase sur `origin/main` et gates finaux (2026-10-09)

**Rebase** de la branche (11 commits) de `5e4bec50` sur `de285ea8` (porte 15-7a1 et 15-11b ; la
15-6a n'était pas encore sur `main`). Conflits, tous de registre ou de document :
`epic-15-choix-autonomes.md` (union, 276 entrées `## C…`, aucun identifiant en double — vérifié par
`grep -E '^## C' | sort | uniq -d`), `sprint-status.yaml` deux fois (lignes `last_updated` de la
branche renumérotées (27) et (28) derrière les (24)-(26) de `main` ; ligne 15-11b de `main` gardée,
lignes 15-12 / 15-12a / 15-12b de la branche gardées), `admin-manual.pdf` (binaire : `.tex` fusionné
sans conflit, PDF régénéré après rebase, contrôlé aplati — « Comment .env atteint Kesh » de la 15-11b
et « du plus ancien vers le plus récent » de cette story y figurent). `CHANGELOG.md`, `messages.ftl`
et `audit_route_registry.rs` fusionnés sans conflit (`main` n'a pas touché `audit_route_registry.rs`
depuis la base : partition inchangée). Aucun code de la branche ne lit l'environnement
(`git diff origin/main -- '*.rs' | grep 'env::var'` vide) : la règle `config::env_nonempty` de la
15-11b n'est pas en jeu.

**Gates complets sur l'état rebasé** (dernier commit de code : `160ff2f0`, la remédiation P1 rebasée ;
aucun code ne change après) :
- frontend : `check` 0 erreur (27 avertissements préexistants), `lint-i18n-ownership` PASS,
  `test:unit` **1095 / 1095** (112 fichiers), `build` OK (`target/gate-logs/15-12a-gate-frontend-final.log`) ;
- bases `kesh_1512a` / `kesh_e2e_1512a` remises à zéro (DROP/CREATE, migrations, seed), attente des
  gates des autres agents (`wait-kesh.sh`) ; `scripts/test-fast.sh` (fmt + clippy + nextest) :
  **2934 / 2934, 4 ignorés** (`…/15-12a-gate-backend-final.log`) ;
- E2E complet (backend `:3012` sur `kesh_e2e_1512a`, secrets `openssl rand`, SMTP factice,
  `KESH_INBOX_DIR`/`KESH_DOCUMENTS_DIR` sous `target/e2e-dirs` du worktree, `smtpConfigured:true`),
  02:27-02:37 UTC : **244 passés, 10 échecs, 19 ignorés** (10,2 min, `…/15-12a-e2e-final.log`). Fichier
  par fichier contre `docs/testing.md` : sept KF-029 (`mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`), deux KF-045
  (`invoices.spec.ts:415`, `:439`, run avant 12:00 UTC) et **`xss-token-protection.spec.ts:82`, faute
  de montage** : `KESH_TEST_MODE=true` manquait **côté runner** (`docs/testing.md` § prérequis — la spec
  lit la variable dans Playwright). Rejouée avec la variable contre le même backend : **3 / 3 passés**
  (`…/15-12a-e2e-final-xss.log`). Aucun échec hors liste ; les 6 tests de `fiscal-years.spec.ts` verts.
  Backend arrêté par son PID.

### Intégration sur `39b52628` (2026-10-09)

`main` a avancé à `39b52628` (15-6a mergée, #572) après la poussée de la clôture : nouveau rebase des
12 commits de la branche. Conflits :
- `epic-15-choix-autonomes.md` : union (316 entrées `## C…`, aucun identifiant en double, vérifié par
  `sort | uniq -d`) ;
- `sprint-status.yaml`, trois fois : lignes `last_updated` de la branche renumérotées (30), (31), (32)
  derrière les (27)-(29) de la 15-6a, sans doublon ; lignes 15-6a de `main` et 15-12 / 15-12a / 15-12b
  de la branche gardées ;
- **code** — `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` : les deux branches ajoutaient leurs
  tests en fin de fichier (15-6a : test 15, l'avoir victime ; 15-12a : tests 9 et 10, clôture et
  création victimes, et l'aide `exercice_sql`). Les deux intentions gardées : en-tête de module de la
  15-12a, puis le bloc de la 15-6a, puis celui de la 15-12a (fusion vérifiée contre la base commune :
  seuls l'en-tête et les deux ajouts diffèrent) ;
- `admin-manual.pdf`, `user-manual.pdf` : `.tex` fusionnés sans conflit, les deux PDF régénérés après
  rebase et contrôlés aplatis — témoins de la 15-6a (« Compte de différences d'arrondi », « Avant
  d'archiver un compte de produit, pensez aux avoirs », « Avoir refusé : compte débiteurs »), de la
  15-11b (« Comment .env atteint Kesh ») et de cette story (« du plus ancien vers le plus récent »)
  présents.
`CHANGELOG.md` (une rubrique `[0.13.0]` par section), `messages.ftl` (aucune clé en double dans les
quatre locales), `errors.rs` (Rust et dépôt) fusionnés sans conflit.

**Gates complets sur l'état rebasé** :
- frontend : `check` 0 erreur (27 avertissements préexistants), `lint-i18n-ownership` PASS,
  `test:unit` **1095 / 1095**, `build` OK (`target/gate-logs/15-12a-gate-frontend-39b5.log`) ;
- `wait-kesh.sh`, bases `kesh_1512a` / `kesh_e2e_1512a` remises à zéro ; `scripts/test-fast.sh` :
  **2951 / 2951, 4 ignorés** (`…/15-12a-gate-backend-39b5.log`) ;
- E2E complet (backend `:3012`, `kesh_e2e_1512a`, secrets `openssl rand`, SMTP factice, répertoires
  sous `target/e2e-dirs`, `KESH_TEST_MODE=true` **côté runner** cette fois), 02:57-03:07 UTC : **245
  passés, 9 échecs, 19 ignorés** — sept KF-029 (`mode-expert.spec.ts:26`, `:41`,
  `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) et deux KF-045
  (`invoices.spec.ts:415`, `:439`, avant 12:00 UTC). Aucun hors liste ; les 6 tests de
  `fiscal-years.spec.ts` verts. Backend arrêté par son PID.

### File List

- `crates/kesh-db/src/errors.rs`, `crates/kesh-db/src/repositories/fiscal_years.rs`,
  `crates/kesh-db/src/repositories/journal_entries.rs` (doc-comment)
- `crates/kesh-db/tests/fiscal_years_repository.rs`, `crates/kesh-db/tests/journal_entries_modification.rs`
- `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/fiscal_years.rs`
- `crates/kesh-api/tests/fiscal_years_e2e.rs`, `crates/kesh-api/tests/rejeu_interblocage_e2e.rs`,
  `crates/kesh-api/tests/audit_route_registry.rs` (doc-comment)
- `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`
- `frontend/src/routes/(app)/settings/fiscal-years/+page.svelte`, `…/fiscal-years-page.test.ts`
- `frontend/src/lib/features/journal-entries/blocker-messages.ts`, `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/tests/e2e/fiscal-years.spec.ts`
- revue P1 : `crates/kesh-api/tests/journal_entry_reversal_e2e.rs`,
  `crates/kesh-api/tests/opening_balances_e2e.rs`, `crates/kesh-db/tests/opening_balances_repository.rs`
- `docs/manual/fr/{user,admin}-manual.{tex,pdf}`, `docs/api-external.md`,
  `docs/MULTI-TENANT-SCOPING-PATTERNS.md`, `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/{15-12a-cloture-dans-l-ordre.md, 15-12-cloture-dans-l-ordre.md,
  sprint-status.yaml, epic-15-choix-autonomes.md}`

## Change Log

- 2026-10-09 — **Intégration sur `39b52628`** (15-6a) : rebase, conflit de code dans
  `rejeu_interblocage_e2e.rs` résolu en gardant les deux ajouts, PDF régénérés ; gates complets :
  backend 2951/2951, Vitest 1095/1095, E2E 245 / 9 attendus (Dev Agent Record).

- 2026-10-09 — **Clôture** : boucle de revue de code close. Trend : **P1** (Sonnet ×3, lentilles B, E, A)
  0 C / 0 H / **2 MEDIUM** distincts d'origine + LOW → remédiation `37784da4` (rebasée `160ff2f0`) →
  **P2 ciblée** (Haiku, une lentille sur la seule remédiation, prompt `15-12a-review-prompt-p2-ciblee.md`,
  rapport `target/gate-logs/15-12a-review-p2-ciblee.md`) : **0** ; l'orchestrateur a vérifié lui-même le
  test 13 b3 (garde réelle `find_later_closed_in_tx`, attente observée à (c) par
  `attendre_une_requete_en_cours`, verdict `EarlierFiscalYearOpen` nommant X). La passe ciblée n'ayant
  produit aucune remédiation, aucune ligne de production ne bouge après elle : boucle close. Rebase sur
  `de285ea8` (15-7a1, 15-11b), gates complets sur l'état rebasé : backend 2934/2934, Vitest 1095/1095,
  E2E 244 / 10 échecs dont 9 attendus et 1 de montage rejoué vert (Dev Agent Record, § « Clôture »).
  Statut `done`. L'issue #543 reste ouverte : la 15-12b la fermera.

- 2026-10-09 — **Revue de code P1** (Sonnet ×3, contexte frais, lentilles B, E, A ; prompt
  `15-12a-review-prompt-p1.md`). Bruts : B 0 / 0 / 1 MEDIUM / 6 LOW, E 0 / 0 / 0 / 5 LOW, A 0 / 0 / 1
  MEDIUM / 6 LOW ; après dédoublonnage (B-1 = E4 ; A1 = B-2 = E1) : **0 CRITICAL, 0 HIGH, 2 MEDIUM**
  distincts, le reste LOW. Les deux MEDIUM sont d'**origine** (aucun n'est né d'une remédiation) : le
  verrou de la garde de création n'était prouvé par aucun test (A1 → test 13 b3, mutation (xi) rouge) ;
  le message neutre promettait plus que le code (B-1 → texte borné à la modification et à la
  suppression). Tous les LOW traités ou écrits (Dev Agent Record, § « Remédiation de la revue de code
  P1 »). Choix C-15-12a-2 à -4. Gates complets sur l'arbre remédié : backend 2901/2901, Vitest
  1095/1095, E2E 245 / 9 attendus. ⚠️ La remédiation touche du code de production (texte du repli Rust
  et des `.ftl`, enveloppe des boutons de l'écran) : la boucle n'est pas close — passe ciblée suivante.
- 2026-10-09 — **Développement** (T1-T10) : clôture dans l'ordre, garde de création, 409
  `EARLIER_FISCAL_YEAR_OPEN`, message neutre et message de création, rejeu des deux routes, écran,
  E2E, documentation. Gates au dernier commit de code : backend 2897/2897, Vitest 1094/1094, E2E 245 /
  9 attendus (7 KF-029 + 2 KF-045). Mutations (i), (ii), (iv), (viii), (ix), (x) tuées, (iii)
  observée. Statut `review`.
- 2026-10-09 — **T0 (développement)** : 15 des 16 LOW de la validation P4 appliqués à la fiche (F1 = R5
  constante `LOCK_IN_COMPANY_SQL` ; F2 bornes de `i18n-keys.test.ts` ; F3 textes DE/IT/EN, glossaire,
  message propre à la création — C-15-12a-1 ; F4/F7 manuel ; F5 immutabilité de `start_date` ; F6 qui
  libère W au 13 d, trois tentatives, `1205` ; R1, R2 numéros de ligne ; R3 mutations des 13 b / 13 d ;
  R4 désignations ; R6 inventaire hors périmètre ; R8 `sprint-status.yaml` — entrées 15-12 reportées sur
  la branche ; R9 cycle clôture ↔ renommage). R7 (fiche 15-1a) laissé à l'orchestrateur ; frontière 15-1a
  mise à jour sur sa demande (découpage 15-1a-i / 15-1a-ii, C125). Numéros de ligne relocalisés sur
  `5e4bec50`. Mesures T0 au Dev Agent Record : aucune ne change une règle ni un AC sur le fond — b2
  interbloque (l'AC 6 le prévoyait), la création se laisse forcer en victime (test HTTP écrit).

- 2026-10-09 — **Créée au découpage de la 15-12** (remédiation de la validation P2, décision de
  l'orchestrateur, C107). Historique de la 15-12 (création C89 ; validation P1, Opus ×2 : 1 HIGH / 4
  MEDIUM / 14 LOW distincts, C100) : Change Log de l'index `15-12-cloture-dans-l-ordre.md`.
  **Validation P2** (Sonnet ×2, contexte frais ; rapports `target/gate-logs/15-12-p2-{R,F}.md`) : R 0 / 0
  / 2 MEDIUM / 5 LOW, F 0 / 0 / 3 MEDIUM / 8 LOW ; après dédoublonnage (R1 = F1, R5 = F6, R7 = F9 ; R3
  rattaché à F3) : **0 CRITICAL, 0 HIGH, 4 MEDIUM, 10 LOW distincts** (14 sur 18 bruts). Les quatre
  MEDIUM sont d'**origine** ; deux LOW sont des **résidus de la remédiation P1** (R4 : propagation
  incomplète de C100 à l'ancienne clé ; F7 : « 8 modules » resté en tête de la ligne du registre). Trend : P1 1 HIGH / 4 MEDIUM → P2 0 HIGH / 4 MEDIUM. Traités **ici** : **F3** (+ R3) —
  découpage (C107), AC 9 déplacé dans cette fiche ; **R2** — preuve HTTP de l'enveloppe de la clôture,
  mutation (ix) tuée par elle, (x) pour la création (C109) ; **F2** — `ORDER BY start_date ASC` dans
  `OPEN_COVERING_DATE_SQL`, `EXPLAIN` en T0 (C110) ; **R4** et **F5** — message neutre avec la
  marche à suivre, ancienne clé alignée aux cinq sites relevés (C111) ; **R5 = F6** — manuel
  administrateur recalé (`:1381-1382`), fichiers en vol complétés ; **R6** et **F11** — portée de
  l'invariant déclarée, frontière avec la 15-1a écrite (C112 ; la fiche 15-1a n'est **pas** modifiée) ;
  **R7 = F9** — ce que voit `attendre_une_requete_en_cours`, motifs discriminants par étape et par test ;
  **F4** — `MULTI-TENANT-SCOPING-PATTERNS.md` à l'AC 14 ; **F10** (part A) — inventaire des tests par
  recherche à l'AC 21. Traités dans la 15-12b : R1 = F1, F8 (lot), F10 (part B). F7 : `sprint-status.yaml`.
  Comptes : **15 AC** (1-7, 9, 13, 14, 17, 19, 21, 22, 23 — numéros d'origine), **11 tâches** (T0-T10),
  recomptés. Passe suivante : cf. index.
- 2026-10-09 — **Validation P3** (Opus ×2, contexte frais, sur les **deux** fiches : R chasseur de
  régressions, F adversaire plein périmètre ; rapports `target/gate-logs/15-12-p3-R.md` et `…-p3-F.md`).
  Bruts : R 0 / 0 / 1 MEDIUM / 8 LOW, F 0 / 0 / 3 MEDIUM / 8 LOW ; après dédoublonnage (R1 = F3 ; R3
  rattaché à F2) : **0 CRITICAL, 0 HIGH, 3 MEDIUM, 15 LOW distincts**. ⚠️ **Deux des trois MEDIUM sont nés
  de la remédiation P2** — F2 porte sur le « par construction » posé par C110, R1 = F3 sur l'inventaire
  « par recherche » ajouté en P2 (F10) — ; F1 est d'origine. Trend : P1 1 HIGH / 4 MEDIUM → P2 0 HIGH /
  4 MEDIUM → P3 0 HIGH / 3 MEDIUM. Traités **ici** : **F2** (MEDIUM, décision de l'orchestrateur) —
  `close` verrouille ses antérieurs **un par un par clé primaire** dans l'ordre lu sans verrou, puis Y,
  puis relit sous verrou (AC 3 réécrit, étapes (a)-(b)-(b')-(c)-(d)-(e)) ; ce que la forme perd (verrous
  de clé suivante, fantômes) et ce qui le rend, écrits ; `FORCE INDEX` écarté ; `OPEN_COVERING_DATE_SQL`
  **sans** `ORDER BY` (C119, **révise C110**) ; AC 6, 13 (motifs par étape, point d'arrêt (c)
  déterministe), 14, 19 (mutations (ii) et (iii) redéfinies), 21, T0, T1 et Dev Notes alignés ;
  « par construction » retiré partout où il qualifiait un parcours ; **R3** (deux index, non trois) dans
  la même réécriture ; **F5** — cycle création ↔ contre-passation / annulations nommé (AC 3, 6, 14, Dev
  Notes) ; **F6** — Y disparu entre (a) et (c) → `NotFound` par `fetch_optional`, test ; **F4** — points
  (ii) et (iii bis) du doc-comment du registre ajoutés à la propagation de l'AC 6 ; **R2** — inventaire
  de l'AC 21 étendu à la création (aucun cas trouvé, lus) et *pathspec* `'crates/*.rs'` (couvre
  `lib.rs`) ; **R5** — doc-comments de `FIND_LATER_CLOSED_SQL` et `find_later_closed_in_tx` à l'AC 14 ;
  **R7** — chevauchement : `400 VALIDATION_ERROR`, message `error-fiscal-year-overlap` (non
  `FY_OVERLAP`) ; **R8** — `user-manual.tex:708` : la proposition « clôturer l'exercice suivant fige
  aussi celui-ci » réécrite ; **F8** — autres textes qui prescrivent une réouverture sans ordre : hors
  périmètre, sites listés, issue à ouvrir (C122) ; **F9** — section « Dérogation règle de splitting »
  (C121, dérogation acceptée par l'orchestrateur) ; **F10** — CHANGELOG : défaut sous `Corrigé`, contrat
  sous `Modifié` (C123) ; **F11** — nom complet « Exercice CI 2020-2030 » à l'E2E. Frontière 15-1a mise
  à jour (C113 tranche l'état hérité ; C114 à aligner sur C119 ; la 15-1a ne dit plus la clôture « non
  rejouée »). Traités dans la 15-12b : F1, R1 = F3, R4, R6, F7, et la part B de F9. **R9** (la fiche
  15-11a, mergée, renvoie #551/#552 à « story 15-12 » — `15-11a-compose-transmet-la-configuration.md:36`,
  `:1274-1275` —, alors que la 15-12b les attribue à la 15-13) : **résidu historique**, non corrigé —
  la fiche est sur `main`, et la réécrire n'apporterait rien. `main` a avancé à `5e4bec50` (15-5d) : les
  relevés refaits en P3 le disent. Comptes : **15 AC** (1-7, 9, 13, 14, 17, 19, 21, 22, 23), **11
  tâches** (T0-T10), recomptés par la commande de l'index. Passe suivante : cf. index.
