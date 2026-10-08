# Story 15.12a : Clôturer les exercices dans l'ordre — l'invariant, ses trois transitions, son écran

Status: ready-for-dev

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
     en P3** le sont sur `5e4bec50` et le disent. -->

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
⚠️ **Publication** : le message neutre de l'AC 9 (« rien ne peut être enregistré… ») n'est vrai de l'état
hérité qu'avec la 15-12b — **la v0.13.0 ne se tague pas sans la 15-12b** (qui ferme #543, P1).

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
  plan) — la 15-1a lit sans verrou les `id` triés par `start_date`, puis les verrouille **un par un par
  clé primaire** dans cet ordre, et relit sous verrou ce que ses fantômes pourraient changer (C114, à
  réviser par l'orchestrateur dans la fiche 15-1a).
- Dans l'**état hérité**, l'invariant ne tient pas : un groupe entièrement dans un exercice ouvert N, sous
  un N+1 clos, passe la règle « au moins une ligne sur un exercice ouvert ». Ni cette fiche ni la 15-12b
  ne le refusent ; la 15-1a l'a tranché (C113 : elle **garde** l'état hérité, patron 15-8a).
- ⛔ **Cette fiche ne modifie pas la 15-1a** (en validation) : le point est porté à l'orchestrateur.

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
   **immuable** (seuls `close`, `reopen` et `update_name` écrivent `fiscal_years`, et `update_name` ne
   touche que `name` : `fiscal_years.rs:400`) ; absente → `NotFound` ;
   (b) **liste des antérieurs, sans verrou** — constante `LIST_EARLIER_SQL` : `SELECT id FROM
   fiscal_years WHERE company_id = ? AND start_date < ? ORDER BY start_date ASC`, **tous statuts** (un
   antérieur clos dans la vue peut être en cours de réouverture) ;
   (b') **chacun verrouillé, un par un, dans cet ordre**, par une boucle Rust — constante
   `LOCK_EARLIER_BY_ID_SQL` : `SELECT id FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE`.
   Un exercice listé en (b) et disparu depuis (`onboarding::reset` efface les exercices de la société)
   est ignoré : la relecture (d) fait foi ;
   (c) `SELECT … WHERE id = ? AND company_id = ? FOR UPDATE` sur Y (statut courant), par
   `fetch_optional` — Y disparu depuis (a) → `NotFound`, jamais une panique (F6) ; verdict « déjà clos » ;
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
     le lit ; ce qui viendrait après attend ;
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
   verrou de la pièce, dans `supplier_invoices::cancel_in_tx` (`:952-956`, puis `reverse_owned_in_tx`
   `:979`), `supplier_invoices::cancel_settlement_in_tx` (`:1159`, puis `:1188`),
   `invoice_settlements_write::cancel_settlement_in_tx` (`:749`, puis `:773`) et
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
   `Rejouee`). Les requêtes (b), (b') et (d) sont des **constantes**, pas des chaînes écrites deux fois.

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
   nom en double) et avant l'`INSERT`. HTTP **`400 LATER_FISCAL_YEAR_CLOSED`** (mapping global ;
   `map_create_error` le laisse passer par son bras `other`), message de l'**AC 9** (même fiche).
   `create_for_seed` (une seule société neuve, un seul exercice : `kesh-seed/src/lib.rs:168`) et
   `create_if_absent_in_tx` (n'insère que si la société n'a **aucun** exercice) ne peuvent pas produire
   l'état fautif et ne changent pas — écrit à leur doc-comment. **Test HTTP** (`fiscal_years_e2e.rs`) :
   `POST /fiscal-years` sous un exercice clos → `400 LATER_FISCAL_YEAR_CLOSED`, corps complet
   (`details.fiscalYearId` / `fiscalYearName`, message de l'AC 9) ; paire de précédence : une demande
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
     `onboarding::finalize`.
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
   (`kesh-api/src/errors.rs:2815-2842`) lit une **clé neuve** `error-later-fiscal-year-closed`, aux
   quatre locales et dans le repli Rust (qui remplace celui de `:2823-2825`). Texte fr-CH de
   référence : « L'exercice « { $name } », postérieur, est clôturé, et son bilan reprend tout ce qui le
   précède : rien ne peut être enregistré, modifié ou supprimé avant sa date de début tant qu'il l'est.
   Une écriture se corrige alors par une contre-passation ; sinon, un administrateur rouvre les
   exercices clôturés, en commençant par le plus récent. »
   - La formulation ne présuppose **pas** que l'exercice visé existe (elle vaut à la création d'un
     exercice, F12) ;
   - elle **garde le conseil** que le `PUT` / `DELETE` donnait (la contre-passation, F5) ;
   - elle ne prescrit **jamais** de rouvrir « { $name } » : ce serait refusé par la garde LIFO dès qu'un
     exercice plus récent est clos (C100) ; « en commençant par le plus récent » est le seul ordre que
     LIFO accepte, et il vaut aussi à la création d'un exercice (état sain, où le bandeau de la 15-12b ne
     s'affiche pas : le message porte lui-même la marche à suivre — F5).
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
   fiscal year|wieder eröffnen;|riaprire quell"` hors fiches et PDF) : `crates/kesh-i18n/locales/fr-CH/messages.ftl:372`,
   `de-CH/messages.ftl:378`, `it-CH/messages.ftl:378`, `en-CH/messages.ftl:378`, repli de
   `blocker-messages.ts:94` — et le repli Rust `errors.rs:2824`, qui disparaît avec le passage à la clé
   neuve. Le `grep` se refait après correctif et ne doit plus rien rendre hors des fiches de la 15-8a.
   Code, statut (`400`) et `details` inchangés.

### La concurrence

13. **Tests à deux connexions** (`crates/kesh-db/tests/fiscal_years_repository.rs`, base éphémère
    `#[sqlx::test]`, `attendre_une_requete_en_cours` de `test_fixtures.rs`) — chacun asserte **l'état
    final** et l'issue de **chaque** côté, et **force** l'entrelacement qu'il teste : une course libre
    (`tokio::spawn` × 2) ne tombe qu'au hasard dans la fenêtre qui compte (C100). La session W est une
    transaction de test qui tient un verrou.
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
      (sérialisation, sans interblocage). Dans les deux cas, la propriété ne dépend pas du mécanisme :
      une création validée avant que la clôture ne tienne Y est lue par (d) ; une création postérieure
      attend Y (AC 3, « ce que cette forme perd »). Un test par configuration ; chacun écrit le mécanisme
      observé (interblocage rejoué, ou sérialisation) dans son doc-comment, et la phrase de l'AC 6 est
      corrigée si aucune configuration n'interbloque. La mesure décide aussi de la preuve de
      l'enveloppe de la création (AC 6, mutation (x)).
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
    L'ancien **13 c** de la création (« un écrivain dans N contre `close(N+1)` ») est **retiré** (C100).
    Chaque test nomme, dans son doc-comment, la **mutation qu'il tue** (AC 19).

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
    l'ordre, pas une garantie ; (iv) retirer la garde de `create` → AC 5 rouge ; (viii) faire passer
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
      modification et la suppression. ⚠️ Seuls **restent** la parenthèse « pour la modification et la
      suppression seulement… limite suivie par l'issue #543 » de ce même item et l'avertissement des
      conditions de modification (`:494-500`) : ils décrivent l'état hérité, que seule la 15-12b ferme.
    - **Manuel admin** (`admin-manual.tex`, recalé sur `8f9811d8`) : paragraphe « Réouverture d'un
      exercice clôturé » (`:1381-1382`) — la clôture dans l'ordre, symétrique de la garde de réouverture.
      Les lignes `:1920` et `:1959` (« si aucun exercice postérieur n'est clôturé ») restent vraies.
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

- [ ] **T0 — Relevés au sol sur le `main` du moment** (AC 3, 6, 9, 13, 14, 21) *(ex-T0, part A)*
  - [ ] Refaire les numéros de ligne cités, par leur texte.
  - [ ] Vérifier que `start_date` n'est écrite nulle part après l'`INSERT` (AC 3 a).
  - [ ] Mesurer en MariaDB 10.11 (deux sessions `mariadb` à la main) le mécanisme de l'AC 13 b dans
        ses **deux** configurations (b1, b2), le cycle de l'AC 13 d et le moyen d'en forcer la victime ;
        `EXPLAIN` **descriptif** (C119 : rien n'en dépend) de `FIND_EARLIER_OPEN_SQL` (relecture (d)) et
        de `FIND_LATER_CLOSED_SQL`, **dans les deux régimes** — base éphémère `#[sqlx::test]` à une
        société, base de dev à plusieurs sociétés —, écrit au Dev Agent Record ; un plan de
        `FIND_LATER_CLOSED_SQL` qui ne suit pas `uq_fiscal_years_company_start_date` est signalé à
        l'orchestrateur (AC 3, limite préexistante de `reopen` et de la garde 15-8a).
  - [ ] Constater la forme de `PROCESSLIST.INFO` d'une requête préparée (`?` ou valeurs) ; vérifier les
        motifs de l'AC 13 et ceux de `journal_entries_modification.rs:410`, `:511` (AC 21).
  - [ ] Refaire le `grep` des sites de l'ancienne clé (AC 9) et chercher toute assertion de test sur
        son texte ou sur celui du repli Rust.
- [ ] **T1 — Clôture dans l'ordre (dépôt)** (AC 1, 2, 3, 7, 14) *(ex-T1)*
  - [ ] `DbError::EarlierFiscalYearOpen { fiscal_year_id, fiscal_year_name }` (+ `error_code()` si la
        famille l'exige — suivre `LaterFiscalYearClosed`, `errors.rs:198/210/221/562/993`).
  - [ ] Constantes `LIST_EARLIER_SQL`, `LOCK_EARLIER_BY_ID_SQL`, `FIND_EARLIER_OPEN_SQL` ; `close`
        réordonné en (a)-(b)-(b')-(c)-(d)-(e), la boucle (b') en Rust (C119) ; `OPEN_COVERING_DATE_SQL`
        **inchangée** (C119 révise C110) ; doc-comments (module, `close`, `FIND_LATER_CLOSED_SQL` et
        `find_later_closed_in_tx` — R5 —, invariant I et sa portée, `DbError::LaterFiscalYearClosed`,
        `reverse_in_tx_inner`) ; `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (AC 14).
  - [ ] Tests : refus, précédence (paires), autre société, rien d'écrit (statut + audit) ; **deux
        exercices antérieurs aux `id` inversés** par rapport à leurs dates (le plus ancien créé en
        second), le plus ancien ouvert → `EarlierFiscalYearOpen` le nomme ; exercice disparu entre (a)
        et (c) → `NotFound` (F6 : W tient Y, la clôture bute en (c), W supprime Y — sans écriture —
        et valide ; jamais de panique).
- [ ] **T2 — Création** (AC 5) *(ex-T2)* — garde, doc-comments de `create_for_seed` /
      `create_if_absent_in_tx`, tests de dépôt ; test HTTP `POST /fiscal-years` → `400
      LATER_FISCAL_YEAR_CLOSED` et paire chevauchement (`400 VALIDATION_ERROR`) (avec T3).
- [ ] **T3 — HTTP, message et rejeu** (AC 4, 6, 9) *(ex-T3, et la part « message » de l'ex-T4)* —
      mapping 409 dans `kesh-api/src/errors.rs` ; clé `error-later-fiscal-year-closed` ×4 + repli Rust ;
      ancienne clé alignée ×4 + repli de `blocker-messages.ts` (C111) ; enveloppes des deux routes ;
      tests `fiscal_years_e2e.rs` (corps complet : code, message, `details`) ; test HTTP de rejeu de la
      clôture (et de la création si T0 le permet) dans `rejeu_interblocage_e2e.rs` ; registre vert et son
      doc-comment (points (ii), (iii bis), (iv) et (vi) — F4).
- [ ] **T4 — Concurrence** (AC 13) *(ex-T6)* — tests (a), (b1), (b2), (c), (d), entrelacements forcés,
      motifs de l'AC 13 ; `reopen_close_concurrent_is_serialized` remplacé par le 13 a.
- [ ] **T5 — Écran des exercices : la clôture** (AC 17) *(ex-T7, part A)* — bouton désactivé,
      `earliestEarlierOpen`, `submitClose` ; clés `fiscal-year-close-blocked-earlier-open`,
      `error-fiscal-year-close-earlier-open` ×4 ; `npm run lint-i18n-ownership` ; Vitest
      (`fiscal-years-page.test.ts`).
- [ ] **T6 — Tests existants** (AC 21, part A) *(ex-T9, part A)* — triage écrit, test par test, `mod
      tests` de `src/` compris ; doc-comments des tests 15-8a/15-8b.
- [ ] **T7 — E2E** (AC 22) *(ex-T10)*.
- [ ] **T8 — Mutations** (AC 19, part A) *(ex-T11, part A)*.
- [ ] **T9 — Documentation** (AC 23, part A) *(ex-T12, part A)* — manuels + PDF aplatis,
      `api-external.md`, CHANGELOG.
- [ ] **T10 — Gates** *(ex-T13)* — base remise à zéro (DROP/CREATE de **ses** bases, jamais de
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
  (`kesh-api/src/errors.rs:2978`, `:3509`, `:3546`) et frontend (`settlement-cancel-blocked.ts:39`,
  `reconciliation-cancel.ts:60`, `invoice-cancel.ts:43`) ; manuel utilisateur (`:1405`, `:1427`,
  `:1714`, `:2242`). **Issue à ouvrir par l'orchestrateur** (P3 : texte qui décrit un geste que le code
  refuse dans un cas), qui les alignera sur la prescription de C111 — « en commençant par le plus
  récent ».

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

### Debug Log References

### Completion Notes List

### File List

## Change Log

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
