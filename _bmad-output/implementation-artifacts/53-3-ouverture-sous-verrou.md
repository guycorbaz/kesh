# Story 53-3 : L'ouverture ne se passe pas sur un compte de résultat — le contrôle des types, sous verrou (#513)

Status: drafted

*Fiche rédigée le 2026-11-25 (Sonnet 5.5), worktree `kesh-53-3`, branche `story/53-3-ouverture-sous-verrou` partie de
`origin/epic/53-ecritures-justes` à `4d540a6ea`, par le workflow `bmad-create-story`, en autonomie. Epic 53 « Écritures justes »,
troisième story ; release **v0.21.0** (vendredi 2026-12-04). **Aucune migration.** Priorité **P1** (une écriture d'ouverture sur un
compte de produits ou de charges fausse le résultat du premier exercice). Validation P1 (Opus) : 0/0/3/8, remédiée le 2026-11-25 (Change Log) ; validation P2 (Sonnet) :
`53-3-ouverture-sous-verrou-validation-prompt-p2.md`. Décisions du Project Lead du 2026-11-25 : Q1 acceptée (D3), Q2 tracée en issue.*

## Story

En tant que comptable qui reprend ses soldes de départ, je veux que Kesh refuse une écriture d'ouverture qui touche un compte de produits
ou de charges **au moment même où il l'écrit**, et non quelques millisecondes avant, afin qu'un compte retypé par un collègue pendant la
génération ne fausse pas le résultat du premier exercice.

## Étape 0 — prémisses relues au code (`4d540a6ea`)

**P1 — le contrôle est bien avant l'écriture, dans une transaction jetée (confirmée).**
`crates/kesh-api/src/routes/opening_balances.rs:449-454` : `let mut tx = state.pool.begin()` puis
`let types = accounts::find_types_by_ids_in_tx(&mut tx, company.id, &account_ids).await?;` puis `let _ = tx.rollback().await;` ; le refus
suit (`:455-464`, `Revenue`/`Expense` → `AppError::Validation`, clé `error-opening-balances-non-balance-account`). L'écriture est créée plus loin,
`:506-514`, par `journal_entries::create_opening_entry` dans **une autre** transaction.

**P2 — l'écriture ne relit pas le type (confirmée).** `crates/kesh-db/src/repositories/journal_entries.rs:867` `pub async fn create_opening_entry` :
sentinelle `companies FOR UPDATE` (`:886`), `fiscal_years FOR UPDATE`, garde « société vierge » `count_by_company(&mut *tx, …)` (`:921`), statut
`Closed` (`:931`), puis `create_opening_in_tx` (`:942`). Aucune lecture de `account_type`. `validate_lines_accounts_in_tx`
(`journal_entries.rs:263`) lit `id, number, active, postable, currency` — **pas** le type — en `LOCK IN SHARE MODE` (`:276-278`).

**P3 — le seul geste qui change `account_type` après la création est `accounts::update` (confirmée, inventaire par le symptôme).**
`git grep -n "UPDATE accounts SET"` sur `crates/*/src` : le seul `SET … account_type = ?` est `accounts.rs:1271` (les autres posent `postable`,
`active`, `parent_id`, `role`) ; `INSERT INTO accounts` : `accounts.rs:280`, `:1780`, `:1922`, `products.rs:1247` — des créations, un compte neuf
n'est dans aucune requête d'ouverture déjà lancée.

**P4 — ce que prend `accounts::update` (relu, à ne pas déduire).** `accounts.rs:1132` `lock_company_currency_in_tx(&mut tx, company_id,
company_lock)` : `companies` en **`LOCK IN SHARE MODE`** (`FOR UPDATE` seulement avec comptes de différence propres, `accounts.rs:2359`,
`:2363`), puis les réglages de facturation en partage, puis la ligne du compte `FOR UPDATE` (`:1137`). Le retypage d'un compte **mouvementé**
exige `confirm_retype` et est audité (`:1240-1262`) ; celui d'un compte **sans écriture** passe sans confirmation — c'est le cas de la
course : le compte cible d'une ouverture n'a, par définition, aucune écriture.

**P5 — conséquence, établie par les verrous (non mesurée à la fiche, **à mesurer au développement**, AC-6).** Comme `update` prend
`companies` en partage **avant** tout et que `create_opening_entry` le prend en `FOR UPDATE` **avant** tout, les deux gestes se sérialisent
déjà sur la ligne de la société : tout retypage commité avant le sentinelle est visible de la transaction d'écriture, tout retypage qui
démarre après attend sa fin. **La fenêtre de #513 est donc entièrement en amont du sentinelle** (entre le `rollback` du handler et le `FOR
UPDATE` de la société) : il suffit de relire le type **dans** `create_opening_entry`, après le sentinelle. Mais **voir le piège P6**.

**P6 — piège : la place de la lecture compte (instantané REPEATABLE READ).** `accounts.rs:444` (doc de `count_by_company`) le dit :
« sous REPEATABLE READ, la première lecture cohérente d'une transaction fige son instantané ». Dans `create_opening_entry`, la première
lecture cohérente est `count_by_company` (`journal_entries.rs:921`), **après** les deux verrous. Deux conséquences, qui fixent la
**place** et la **forme** de la lecture des types :
(i) **forme** : une lecture ordinaire des types lirait l'instantané et ne verrait pas un retypage commité entre-temps par un écrivain qui
ne prend pas la société (aucun aujourd'hui ; demain, un nouveau site) : elle doit être **verrouillante** (`LOCK IN SHARE MODE`, jamais
`FOR SHARE` : erreur de syntaxe sur MariaDB 10.11, `accounts.rs:627`), qui lit la dernière version commitée et attend un écrivain en
cours ;
(ii) **place** (validation P1, V1-3) : **juste après `fiscal_years FOR UPDATE`, avant `count_by_company`**, donc **vue non ouverte**.
C'est la règle de la 25-7 que l'issue prescrit (« verrous d'abord, puis lectures, puis refus », `opening_complement.rs:454-461`). Placée
après `count_by_company`, la lecture verrouillante s'exécuterait vue ouverte : avec `innodb_snapshot_isolation` (actif par défaut depuis
MariaDB 11.6), un commit sur la ligne du compte depuis l'ouverture de la vue rend `ER_CHECKREAD` (1020), que `map_db_error` ne traduit pas
et qu'aucun rejeu ne reprend : **500**, rien d'écrit (angle mort documenté, #669 ; `bank_imports.rs:536-545`). Sans effet sous 10.11
(`docker-compose.yml`, CI), mais le dépôt vise déjà l'après. Placée avant, la lecture est un verrou, non une lecture de vue : le défaut ne
peut pas naître ici. C'est aussi le verrou que `validate_lines_accounts_in_tx` prendra ensuite sur les mêmes lignes : **verrou avancé,
non neuf**. Les refus restent évalués dans l'ordre D1, **après** `count_by_company` et le statut (le verrou d'abord, le refus à sa place).

**P7 — l'ancien argument de report ne tient plus (`14-4-bilan-ouverture-soldes-depart.md:381`, ECH-1, reporté à #274).** Il disait :
« `accounts::update` autorise le retypage d'un compte **avec écritures** à tout moment (verrou optimiste seul) → l'état final est atteignable
légalement en séquentiel ». Faux depuis la 25-2-a (#382) : ce retypage exige `confirm_retype` et laisse un `account.retyped` audité
(`accounts.rs:1240-1262`). Un retypage **dans la fenêtre** n'a ni l'un ni l'autre (le compte n'avait pas d'écriture au moment où il passe).
La fenêtre produit donc un état que rien ne signale. Le correctif est justifié.

**P8 — l'issue dit : « Il n'y a pas de données de production à protéger ». Périmé** : Kesh est en production depuis le 2026-10-31 (livres réels
sur le NAS). Sans effet sur la solution ; sur la priorité, elle reste P1.

## Inventaire clos des chemins qui écrivent une ouverture

Un chemin « écrit une ouverture » s'il crée une écriture dont le sens est de reprendre des soldes de départ. Le contrôle lexical
`crates/kesh-db/tests/chf_line_lexical.rs` fige déjà que `create_opening_in_tx` (`journal_entries.rs:408`) n'a que deux appelants.
Commande de l'inventaire : `git grep -n "create_opening_in_tx\|create_opening_entry\|create_opening_complement"` hors tests, puis `git grep -n "journal_entries::update"` (W10), puis
`git grep -n "ouverture\|opening" -- crates/kesh-api/src/mcp crates/kesh-seed crates/kesh-db/src/repositories/fiscal_years.rs`.

| # | Chemin | Contrôle du type de compte | État |
|---|---|---|---|
| W1 | `POST /api/v1/opening-balances` → `journal_entries::create_opening_entry` | **avant**, transaction jetée (`opening_balances.rs:449-464`) | **DÉFAUT — corrigé par la story** |
| W2 | `POST /api/v1/opening-balances/complete` → `opening_complement::create_opening_complement` (portée `Screen`) | **sous verrou** : comptes lus `FOR UPDATE` par clé primaire (`opening_complement.rs:600-611`), refus `NotBalanceAccount` (`check_lines`, `:778-783`, tout ce qui n'est ni `Asset` ni `Liability`) | couvert ; test séquentiel `opening_complement_repository.rs:364`. Le refus est **plus strict** que W1 (qui ne refuse que `Revenue`/`Expense`) : sans effet, il n'existe que quatre types (`entities/account.rs:12-17`) |
| W3 | création et modification d'un compte bancaire avec solde d'ouverture → `create_opening_complement_in_tx` (portée `SingleBankLedger`, `crates/kesh-api/src/routes/bank_accounts.rs:1061-1076`, appel `:1068`) | même fonction que W2, mêmes verrous | couvert |
| W4 | `POST /api/v1/journal-entries` (écriture manuelle, journal OD) | aucun — **hors sujet** : ce n'est pas une ouverture, un OD peut légitimement toucher un compte de résultat | non concerné |
| W5 | import d'installation (`.keshbackup`, `admin_backup/import.rs`) | aucun : restauration brute d'une base entière | **angle mort assumé** : une sauvegarde restaure ce qui fut commité ; aucune ouverture n'est « écrite » |
| W6 | clôture d'exercice / report à nouveau | sans objet : `fiscal_years.rs` ne crée aucune écriture (le bilan est cumulatif depuis l'origine, `journal_entries.rs` commentaire de l'étape 1-a) ; recherche `create_in_tx` dans `fiscal_years.rs` : seulement des doc-comments | non concerné |
| W7 | serveur MCP et clés d'API | aucun outil d'écriture d'ouverture (`mcp/outils/grand_livre.rs`, `exercices.rs`, `rapports.rs` : lecture) ; une clé d'API qui atteint `POST /opening-balances` passe par W1 | couvert par W1 |
| W8 | données de démonstration (`kesh-seed`) | recherche `ouverture\|opening` : aucune ligne | non concerné |
| W9 | assistant d'installation (`routes/onboarding.rs`) | recherche `ouverture\|Opening` : aucune ligne ; le compte bancaire de l'assistant passe par W3 | non concerné |
| W10 | **modification** de l'écriture d'ouverture (`PUT /api/v1/journal-entries/{id}`) | aucun refus de type propre à l'ouverture : « rien n'empêche donc d'y ajouter une ligne de résultat, ce que la génération refuse » (`docs/manual/fr/user-manual.tex:1065`, retrouvé dans le PDF aplati) | **angle mort assumé, documenté au manuel** : la modification est un geste d'écriture ordinaire (toute écriture peut toucher un compte de résultat) ; ce que #513 ferme est la **génération** ; hors périmètre, la fiche ne le traite pas (voir Q3 si le Project Lead veut l'étendre) |
| W11 | soldes d'ouverture de **relevés** (`opening_balance` de `bank_statements_manual.rs`, `bank_imports.rs`) | soldes de relevé, non des écritures | non concernés |

Sites **non résolus** de la classe « lire un type de compte avant d'écrire, hors du verrou » : `find_types_by_ids_in_tx` n'a **qu'un seul
appelant de production**, le handler W1 (`git grep -n find_types_by_ids_in_tx` : handler `:452`, définition `accounts.rs:474`, trois tests
`opening_balances_repository.rs:302`, `:325`, `:260`). Aucun autre site de la classe. Angle mort assumé : un compte **créé** après le
contrôle ne peut être visé par une requête déjà lancée (identifiant inconnu au client).

## Décisions

- **D1 — la lecture des types se fait dans `create_opening_entry`, juste après `fiscal_years FOR UPDATE` et avant `count_by_company` (P6, V1-3) ;
  les refus se jugent ensuite dans l'ordre.** Ordre d'exécution : sentinelle `companies X` → `fiscal_years X` → **lecture verrouillante des
  types** → `count_by_company` → statut. Ordre des refus : `NotFound` → `ALREADY_HAS_ENTRIES` (409) → `FIRST_YEAR_CLOSED` (400) →
  **`NON_BALANCE_ACCOUNT` (400, neuf)** → les refus de `create_in_tx_inner`, dans leur ordre : `FiscalYearClosed`,
  `LATER_FISCAL_YEAR_CLOSED`, `DATE_OUTSIDE_FISCAL_YEAR`, `PERIOD_LOCKED`, puis `INACTIVE_OR_INVALID_ACCOUNTS` / `ACCOUNT_NOT_POSTABLE`, puis
  le contrôle des devises. C'est l'ordre actuel des verdicts, à l'exception de D3. *Écartée 1* : garder la lecture au handler et la
  **répéter** dans la transaction (double lecture : voir D3). *Écartée 2* : la lecture après `count_by_company` (vue ouverte, P6 ii).
- **D2 — la lecture est verrouillante : `LOCK IN SHARE MODE`, `ORDER BY id`, scopée à la société** (P6). On **transforme**
  `accounts::find_types_by_ids_in_tx` (seul appelant de production : le handler) en lecture verrouillante plutôt que d'en ajouter une
  seconde ; on la **renomme** `lock_types_by_ids_in_tx` pour que son nom dise le verrou (un nom qui promet « lecture » aurait fait
  régresser la garde). `ids` vide → `Ok(vec![])` sans requête (inchangé). *Écartée 1* : `FOR UPDATE` — il bloquerait les écritures
  ordinaires qui lisent le compte en partage (`validate_lines_accounts_in_tx`) pour rien ; le partage suffit contre un écrivain, qui
  prend `FOR UPDATE`. *Écartée 2* : ajouter `account_type` au `SELECT` de `validate_lines_accounts_in_tx` et refuser là — ce helper sert
  les **dix-neuf** routes qui créent une écriture ; y mettre une règle d'ouverture fait porter à toutes un paramètre qui ne regarde que
  deux (`create_opening_in_tx` et son complément, déjà gardé en W2).
- **D3 — le handler perd sa transaction jetée et son pré-contrôle des types ; un verdict change d'ordre, **accepté par le Project Lead le
  2026-11-25 (Q1)**.** Aujourd'hui le contrôle des types (`opening_balances.rs:449-464`) précède `accounting::validate` (`:482`). Après la
  story, `validate` précède la transaction : une requête qui cumule un compte de résultat **et** un refus de `validate` rendra l'erreur de
  `validate`, non « comptes de bilan ». Refus de `validate` concernés (`kesh-core/src/accounting/balance.rs:184-219` et le contrôle des
  parties en devise, commentaire `opening_balances.rs:390-391`) : description vide, moins de deux lignes, montant négatif, débit et crédit à
  la fois, écriture déséquilibrée, bornes, signe, arrondi et égalité au CHF d'une partie en devise. Même statut HTTP (400
  `VALIDATION_ERROR`) ; le message diffère. Le changement est **épinglé par un test** (AC-5 e). *Alternative écartée* : conserver, comme
  le fait le handler pour l'exercice et la société vierge (`:410-411`, pré-contrôle UX lock-free, autorité sous verrou), un pré-contrôle
  des types **en plus** du contrôle sous verrou : il garderait l'ordre, au prix d'une double lecture et d'un site de plus pour la même
  règle (DRY) ; écartée pour cette raison seulement, non parce que le motif serait un défaut.
- **D4 — nouvelle clé `FY_OPENING_NON_BALANCE_ACCOUNT_KEY = "fiscal_year:opening-non-balance-account"`** à côté des deux clés de la 14-4
  (`fiscal_years.rs:269`, `:275`), émise en `DbError::Invariant(<KEY>)`, re-mappée par `map_opening_balances_error`
  (`opening_balances.rs:221`) en `AppError::Validation(t("error-opening-balances-non-balance-account", …))` — **même message, mêmes
  quatre locales, aucune clé i18n neuve**. Une clé non mappée retomberait dans `AppError::from(DbError::Invariant)` = **500**.
- **D5 — aucune arête neuve dans le graphe des attentes, avec la place de D1.** Les verrous **détenus** quand la lecture des types demande
  `accounts` en partage sont `companies X` et `fiscal_years X` ; ce sont exactement ceux qui sont détenus quand `validate_lines_accounts_in_tx`
  le demande aujourd'hui (entre les deux, `books_locked_through` et `find_later_closed` sont des lectures ordinaires, le second
  `fiscal_years FOR UPDATE` est ré-entrant, `validate_taggable_in_tx` sort tôt sans projet : `projects.rs:94-96`) ; `count_by_company` est
  une lecture non verrouillante. L'arête `{companies X, fiscal_years X} → accounts S` existe donc déjà : la story la **déplace** de
  quelques instructions, sans la créer. Les gestes qui écrivent `accounts` prennent tous `companies` en partage d'abord (`update` `:1132`,
  `archive_guarded` `:1453`, `reactivate` `:1640`, complément `opening_complement.rs:454-457`) : aucun cycle avec le sentinelle exclusif.
- **D6 — pas de migration.** Aucun schéma, aucune donnée. `kesh_version_min_required` inchangé ; P5–P8 du § *Migration breaking policy* sans
  objet. Release `c` si elle partait seule ; elle sort avec les migrations de l'Epic 53 (v0.21.0).

## Verrous : ordre et interblocage (cités, relus au code)

| Geste | Ordre des verrous | Source |
|---|---|---|
| **`create_opening_entry` (W1), après la story** | `companies` **X** → `fiscal_years` **X** → `accounts` **S** (types, avancé) → [`count_by_company`, lecture ordinaire] → `accounts` **S** (`validate`, ré-entrant) → compteur de numéros | `journal_entries.rs:886`, `:900`, `:276-278` |
| `accounts::update` (le retypage) | `companies` **S** → réglages de facturation **S** → `accounts` (la ligne) **X** → liens bancaires, écritures (lecture verrouillante) | `accounts.rs:1132-1137`, doc `:1090-1112` |
| `create_opening_complement_in_tx` (W2, W3) | `companies` **S** → comptes `FOR UPDATE` `ORDER BY id` → compte de report **S** → `fiscal_years` **X** (premier, puis candidat) | doc de `opening_complement.rs:450-461` |
| `create_in_tx_inner` (toute écriture) | `fiscal_years` **X** → … → `accounts` **S** (`validate_lines_accounts_in_tx`) | `journal_entries.rs:542`, `:276-278` |
| flux « exercice avant société » (rapprochements par règle, ventilation, manuel à projet) | `fiscal_years` **X** puis `companies` | doc de `create_opening_entry` (`journal_entries.rs:839-856`) : cycle **connu**, rejoué par `retry_on_deadlock` |

Lecture : `update` (société en **S**) et W1 (société en **X**) s'excluent à la première ligne : aucun des deux ne tient autre chose quand
il attend l'autre. W1 et W2/W3 s'excluent de même (société **S** contre **X**). Le seul cycle possible reste celui, déjà documenté et rejoué
(`retry_on_deadlock("opening_balances::generate", …)`, `opening_balances.rs:506`), des flux qui prennent l'exercice avant la société. Une
`DbError::Invariant` n'est jamais rejouée (`retry.rs:116`, prédicat `is_deadlock_error`).

⚠️ **À mesurer, pas à déduire** (la consigne de l'issue, et ce que la 25-7 a établi), sur la base de dev, **avec au moins deux
identifiants** (un seul donnerait `const`, non `range`) : (a) `EXPLAIN` **des deux requêtes**, la neuve (`lock_types_by_ids_in_tx`) et celle de
`validate_lines_accounts_in_tx` (`journal_entries.rs:277-278`, même prédicat `company_id = ? AND id IN (…)`, même transaction) : accès
`const` ou `range` sur `PRIMARY`, sans `Using filesort` ; sinon la lecture verrouille davantage de lignes que les comptes visés (voir
`opening_complement.rs:311-327`) — réécrire la seule requête neuve ne gagne rien si l'ancienne surverrouille déjà ; (b) sonde à deux
sessions : session 1 tient la transaction de W1 ouverte après la lecture des types ; session 2 `SELECT id FROM accounts WHERE id = <un
compte non visé de la même société> FOR UPDATE NOWAIT` **réussit**, et sur un compte visé elle échoue — **sous MariaDB l'erreur est 1205
(`ER_LOCK_WAIT_TIMEOUT`, délai nul), non `ER_LOCK_NOWAIT` (code de MySQL)** ; (c) une sonde `FOR UPDATE NOWAIT` sur une ligne existante ne
voit pas les verrous d'**intervalle** : si le plan n'est pas sur `PRIMARY`, ajouter une sonde d'`INSERT` d'un compte de la société sous
`SET SESSION innodb_lock_wait_timeout = 1`, qui doit réussir ; (d) **ne pas sonder un compte d'une autre société nommé dans la requête** :
InnoDB verrouille sous REPEATABLE READ les lignes lues puis écartées par le `WHERE` ; le comportement est le même qu'avant la story, et la
sonde serait bloquée sans que ce soit une régression. Sorties dans `/home/gcorbaz/devel/kesh-gate-logs/e533/`.

## Acceptance Criteria

**AC-1 — le contrôle est dans la transaction qui écrit.** `create_opening_entry` lit, juste après `fiscal_years FOR UPDATE` et **avant** `count_by_company` (D1,
P6 ii), le type de **chaque** compte visé par une lecture `LOCK IN SHARE MODE` (D2), puis, ses gardes « société vierge » et de statut passées, refuse `Revenue` et `Expense` par
`DbError::Invariant(FY_OPENING_NON_BALANCE_ACCOUNT_KEY)` (D1, D4), sans écrire ni auditer. Un identifiant absent du résultat (inconnu, autre
société) n'est **pas** refusé ici : il retombe dans `INACTIVE_OR_INVALID_ACCOUNTS` de `create_in_tx` (contrat AC-B de la 14-4). Le handler
n'ouvre plus de transaction et ne lit plus de type (`git grep -n find_types_by_ids_in_tx crates/kesh-api/src` vide).

**AC-2 — le verdict HTTP ne change pas.** `POST /api/v1/opening-balances` avec une ligne `Revenue` — et avec une ligne `Expense` — rend
**400 `VALIDATION_ERROR`**, message `error-opening-balances-non-balance-account` (« … comptes de bilan … ») ; aucune écriture, aucune ligne
d'audit. Le test existant `post_revenue_account_400_non_balance_account` (`opening_balances_e2e.rs:571`) reste vert, un jumeau `Expense`
s'y ajoute.

**AC-3 — la course de #513 est fermée, et le test rougit sans le correctif.** Test d'entrelacement forcé (patron `PROCESSLIST` de
`opening_complement_repository.rs` / `credit_card_repository.rs:150`, jamais `INNODB_TRX`, jamais un `sleep`) : une connexion
`tenue` ouvre une transaction, prend `companies` en partage comme `accounts::update`, retype le compte cible en `Revenue` **sans commiter** ;
`create_opening_entry` est lancée (tâche) et **vue en attente** sur la société ; `tenue` commite ; l'appel doit rendre
`Invariant(FY_OPENING_NON_BALANCE_ACCOUNT_KEY)`, **aucune** écriture. Sans le correctif, l'appel rend `Ok` et une ligne d'ouverture est
posée sur un compte de produits : le test est rouge. (Variante `Expense` en table.)

**AC-4 — la lecture verrouille vraiment (P6).** Second entrelacement, **sans** `companies` : la connexion `tenue` fait seulement
`UPDATE accounts SET account_type = 'Revenue' WHERE id = ?` (un écrivain qui ne prendrait pas la société) et ne commite pas ;
`create_opening_entry` est vue en attente **sur la lecture des types** (motif `PROCESSLIST` sur le texte exact du `SELECT`, `?` compris) ;
`tenue` commite ; refus `NON_BALANCE`. Tue la mutation « lecture ordinaire » (M4) que l'AC-3 ne tue pas, la société étant déjà un verrou
commun.

**AC-5 — l'ordre des verdicts est conservé, hors D3.** Tests **au niveau du dépôt** (`opening_balances_repository.rs`, appel direct de
`create_opening_entry`) : les pré-contrôles lock-free du handler (`opening_balances.rs:423`, `:434`) répondent avant la transaction, un
test par l'API ne voit donc pas l'ordre sous verrou (V1-1) : (a) société avec une écriture + ligne `Revenue` →
`Invariant(FY_OPENING_ALREADY_HAS_ENTRIES_KEY)` ; (b) premier exercice clos (société vierge) + ligne `Revenue` →
`Invariant(FY_OPENING_FIRST_YEAR_CLOSED_KEY)` ; (c) ligne `Revenue` + une autre ligne sur un compte **archivé** → `NON_BALANCE` (le contrôle
précède `INACTIVE_OR_INVALID_ACCOUNTS`) ; (d) ligne sur un compte `Revenue` **d'une autre société** → `INACTIVE_OR_INVALID_ACCOUNTS` (non
`NON_BALANCE` : anti-énumération KF-002, contrat 14-4). Par l'API (`opening_balances_e2e.rs`), contrôle d'absence de régression : 409 et
`first-year-closed` inchangés ; (e) **épinglage de D3** : une ligne `Revenue` **et** une écriture déséquilibrée rendent le message
d'équilibre (`map_core_error`), non « comptes de bilan » ; mêmes quatre refus de `validate` listés en D3 par un cas au moins hors
équilibre (une seule ligne).

**AC-6 — ni sur-verrouillage ni nouvelle arête d'interblocage (D5).** Les mesures (a) à (d) du § *Verrous* sont faites sur la base de
dev, avec au moins deux identifiants, leurs sorties versées à `kesh-gate-logs/e533/`, leur lecture citée au Dev Agent Record. Si l'accès
n'est ni `const` ni `range` sur `PRIMARY` — pour l'**une ou l'autre** des deux requêtes —, elle est réécrite (`FORCE INDEX (PRIMARY)` ou lecture
par clé primaire, une ligne à la fois, `ORDER BY id`) et la mesure refaite. La doc de `create_opening_entry` (liste des verrous, § « Portée
exacte ») est mise à jour : le verrou `accounts` y paraît, avec sa place et la raison (vue non ouverte, #669).

**AC-7 — les chemins couverts le restent (inventaire W2, W3).** Les tests existants de `opening_complement_repository.rs` et
`bank_account_opening_balance_e2e.rs` passent sans modification ; **aucun** de leurs fichiers de production n'est touché. Le contrôle
lexical `chf_line_lexical.rs` reste vert (`create_opening_in_tx` : deux appelants).

**AC-8 — le concept ne se perd pas.** Test lexical `crates/kesh-db/tests/` (patron `chf_line_lexical.rs`) : le texte de
`opening_balances.rs` (handler) ne contient pas `lock_types_by_ids_in_tx` ni `find_types_by_ids_in_tx` ; celui de `create_opening_entry`
(`journal_entries.rs`) contient `lock_types_by_ids_in_tx`. Sonde positive : la fonction qui lit le fichier source échoue bruyamment si le
fichier est introuvable. Limite écrite : le test est lexical (un renommage en alias le contourne) ; il garde le geste, non l'intention.

**AC-9 — gates.** `cargo fmt`, `clippy -D warnings`, gate ciblé `binary(opening_balances_repository)` + `binary(opening_balances_e2e)` entre
deux passes ; gate complet (`scripts/gate-ci.sh`) et E2E Playwright complet au dernier commit de code (aucun changement d'écran : l'E2E
n'est qu'un contrôle d'absence de régression). Le story file n'affirme que ce qui a tourné.

## Tâches

- [ ] **T0 — étape 0 du développement (avant tout code).** Rejouer à la main : `git grep -n "find_types_by_ids_in_tx"` (appelants) et
  `git grep -n "SET .*account_type"` sur `crates/*/src` (un seul écrivain, `accounts.rs:1271`) ; relire les verrous de
  `accounts::update` (`:1132-1137`) et de `create_opening_entry` ; **recontrôler P5** par une sonde sur la base de dev (une session
  `UPDATE accounts SET account_type='Revenue'` sous `companies LOCK IN SHARE MODE`, non commitée ; une seconde exécute
  `create_opening_entry` : elle doit attendre sur `companies`) ; **revérifier l'absence d'arête neuve** (D5) avec la place de D1. Écart avec la fiche → l'écrire au Change Log.
- [ ] **T1 — test rouge d'abord** (AC-3 et AC-4) dans `crates/kesh-db/tests/opening_balances_repository.rs` ; prouver le rouge **avant**
  le correctif (journal dans `kesh-gate-logs/e533/`).
- [ ] **T2 — `accounts.rs`** : `find_types_by_ids_in_tx` → `lock_types_by_ids_in_tx`, `ORDER BY id … LOCK IN SHARE MODE`, doc réécrite
  (« sous verrou », renvoi à P6) ; mettre à jour les deux tests existants (`opening_balances_repository.rs:260`, `:321`).
- [ ] **T3 — `fiscal_years.rs`** : constante `FY_OPENING_NON_BALANCE_ACCOUNT_KEY` ; **`journal_entries.rs`** : l'étape de contrôle dans
  `create_opening_entry` (D1), doc de la fonction mise à jour (étapes, verrous) ; garde `rollback` comme les autres refus.
- [ ] **T4 — `routes/opening_balances.rs`** : retirer la transaction jetée et le pré-contrôle des types (`:445-464`) ; arm de
  `map_opening_balances_error` pour la clé neuve (même `t(...)`, même repli) ; doc-comments à reprendre : `:362-365` (point 2 de la liste des
  gardes), `:442-445` (commentaire du pré-contrôle, supprimé avec lui), `:216-217` (« Toute autre erreur retombe vers le mapping global » :
  citer la clé neuve). **Ne pas toucher** `:261` (doc de `opening_balances_status`) ni `:411` (pré-check 1 UX) : toujours vraies.
- [ ] **T5 — tests d'API** (AC-2, AC-5) dans `crates/kesh-api/tests/opening_balances_e2e.rs`.
- [ ] **T6 — garde lexicale** (AC-8).
- [ ] **T7 — mesures** (AC-6), journal dans `kesh-gate-logs/e533/`.
- [ ] **T8 — mutations** (tableau ci-dessous), un journal chacune ; **greper le symptôme** après le patch : `find_types_by_ids_in_tx`,
  `non-balance`, `Revenue.*Expense` sur tout le dépôt (code, fiches, doc-comments, tests, quatre locales i18n, manuels) — la valeur, non la
  formulation. Sites à reprendre, relus au code : `opening_balances.rs:216-217`, `:362-365` (T4) ; `opening_balances_e2e.rs:548`
  (`/// dans \`find_types_by_ids_in_tx\`, P3-ECH-LOW-1).`) ; le doc-comment de `create_opening_entry` (`journal_entries.rs:~839-856`, liste
  des verrous). `journal_entries.rs:793` (doc de `count_by_company`) et `opening_balances.rs:261`, `:411` **ne changent pas**.
- [ ] **T9 — Dev Agent Record, Change Log, Notes de version**, `sprint-status.yaml` → `review`.

## Tests et mutations

| # | Mutation (appliquée à la main, restaurée, `touch` du fichier après restauration) | Doit rougir |
|---|---|---|
| M1 | supprimer l'étape de contrôle de `create_opening_entry` | AC-3, AC-4, AC-2 (via l'API) |
| M2 | lire les types **avant** le sentinelle `companies` (lecture simple ou verrouillante) | AC-3 (lecture simple) ; variante verrouillante : par l'attente seule, voir ci-dessous |
| M3 | ne refuser que `Revenue` (retirer `Expense`) | variante `Expense` d'AC-2 et d'AC-3 |
| M4 | retirer `LOCK IN SHARE MODE` (lecture ordinaire, instantané de `count_by_company`) | **AC-4 seul** (AC-3 reste vert : la société est déjà un verrou commun — à écrire au Dev Agent Record) |
| M5 | retirer l'arm de `map_opening_balances_error` (la clé tombe en 500) | AC-2 |
| M6 | placer le **refus** avant la garde « société vierge » | AC-5 (a), **au niveau du dépôt** (V1-1) |
| M6-bis | placer le **refus** avant le contrôle du statut `Closed` | AC-5 (b), au niveau du dépôt |
| M7 | refuser aussi l'identifiant absent (`non-balance` au lieu de `INACTIVE_OR_INVALID_ACCOUNTS`) | AC-5 (d) |
| M8 | placer le contrôle **après** `create_opening_in_tx` | AC-5 (c) : `INACTIVE` passe alors avant `NON_BALANCE` (la ligne archivée est une autre ligne que le compte `Revenue`) |
| M9 | lecture **non scopée par la société** (`WHERE id IN …` seul) | test renommé de `opening_balances_repository.rs:260` (compte `foreign` absent du résultat) ; AC-5 (d) |
| M10 | placer la lecture verrouillante **après** `count_by_company` (vue ouverte, P6 ii) | **aucun test sous 10.11** : angle mort écrit, protégé par le doc-comment et par AC-6 (la place) ; ne devient visible que sous ≥ 11.6 (#669) |

**Comment M2 (variante verrouillante) et M4 meurent** : uniquement par la **panique de l'attente `PROCESSLIST`** (la requête attendue ne se
montre jamais en attente), non par une assertion sur le résultat. À écrire au Dev Agent Record, et à **vérifier dans le journal de chaque
mutation** que le rouge vient de l'attente. **Lecture sur le pool** (au lieu de la transaction) : la signature `&mut Transaction`
l'interdit ; angle mort gardé par le type.

Une mutation **vivante** n'est pas un test réussi : l'écrire comme angle mort, avec sa raison. Angles morts **assumés d'avance** : W5
(restauration d'une sauvegarde) ; un second écrivain de `account_type` qui n'existe pas (AC-4 le garde en puissance) ; la durée d'attente
d'un verrou (`innodb_lock_wait_timeout`, hors périmètre).

## Dev Notes

- **Fichiers touchés (deux crates)** : `kesh-db` (`repositories/accounts.rs`, `journal_entries.rs`, `fiscal_years.rs`,
  trois fichiers de tests), `kesh-api` (`routes/opening_balances.rs`, un fichier de tests). Pas de frontend. **Splitting préventif : non
  déclenché** (deux modules). Recoupement d'epic : `journal_entries.rs` avec l'Epic 51 (48-2) et l'Epic 47 d'un contributeur — la
  modification est **localisée** à `create_opening_entry` ; reprendre `main` avant la revue de code (§ *Travailler à plusieurs développeurs*).
  `accounts.rs` et `fiscal_years.rs` : **non déclarés** au fichier d'epic (`epic-53-ecritures-justes.md:31` ne cite que `opening_balances.rs` et `journal_entries.rs`) et aucun contrôle de recoupement ne les couvre : à faire à la reprise de `main` (`git diff --name-only main...<branche>` des branches d'epic ouvertes sur ces deux fichiers). **Ligne à ajouter au fichier d'epic** (écrite par l'orchestrateur, « Fichiers et modules ») : `- 53-3 : kesh-api/src/routes/opening_balances.rs ; kesh-db/src/repositories/journal_entries.rs (create_opening_entry), accounts.rs (lecture des types), fiscal_years.rs (clé d'erreur) ; tests kesh-db et kesh-api.`
- **`LOCK IN SHARE MODE`, jamais `FOR SHARE`** (MariaDB 10.11 : erreur de syntaxe) ; **jamais de lecture ordinaire** pour un contrôle sous
  REPEATABLE READ (P6).
- **Le test d'entrelacement attend, il ne dort pas** : copier le patron `attendre_requete_en_cours` de
  `crates/kesh-db/tests/credit_card_repository.rs:150` (dix secondes au plus, `PROCESSLIST`, `INFO LIKE ?`). Le helper est **déjà dupliqué**
  dans plusieurs fichiers de tests (voir l'issue du § *Décisions du Project Lead*) : ne pas en refaire une copie sans le dire, et le noter. L'issue est ouverte par
  l'orchestrateur, jamais par l'agent.
- **Aucune clé i18n neuve** : `error-opening-balances-non-balance-account` existe dans les quatre locales (`kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl`, `:1311` pour le français).
- **Manuel (`docs/manual/`)** : relu, **PDF compris** (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' '`).
  `user-manual.tex:1051` « la grille liste vos comptes de bilan actifs et postables (actifs et passifs — jamais les comptes de produits ou de
  charges, qui fausseraient le résultat de l'exercice) » est dans le PDF et **reste vraie** ; le comportement visible ne change que dans une
  course que l'écran ne peut pas produire seul. **Aucune modification de manuel** ; le développement le **recontrôle** (grep du symptôme :
  `produits ou de charges`, `comptes de bilan`, `soldes de départ`) et écrit le résultat. Seul le français existe (`docs/manual/{de,en,it}/`
  ne portent qu'un `README.md`).
- **Recette visuelle** : **aucun contrôle visible touché** (écran *Soldes de départ* inchangé, `frontend/` non touché) ; la règle de
  `CLAUDE.md` § *Recette visuelle* ne s'applique pas, déclaré.
- **Audit** : aucun événement d'audit neuf ; un refus n'écrit rien (la transaction est annulée avant `journal_entry.created`).
- **Documentation à synchroniser** : `README.md` mentionne déjà #513 dans la ligne de la v0.21.0 (rien à changer) ; `CHANGELOG.md` n'est
  pas écrit dans la story (assemblé à l'intégration depuis les Notes de version ci-dessous) ; migrations : `docs/migrations-idempotence-audit.md`
  inchangé (aucune migration).
- **Sécurité de la relecture** : la requête est paramétrée ; la clause `IN` se construit par placeholders (patron déjà en place), jamais
  par interpolation d'identifiants.

## Notes de version (pour l'assemblage du CHANGELOG)

**v0.21.0 — Corrigé.** *Soldes de départ* : le contrôle qui refuse un compte de produits ou de charges dans le bilan d'ouverture se fait
désormais **au moment même de l'écriture**, sous verrou, et non avant ; un compte retypé par un autre utilisateur pendant la génération ne
peut plus recevoir une ligne d'ouverture. Aucun changement pour l'utilisateur dans le cas courant : même message de refus. Aucune migration.
`refs #513`.

## Décisions du Project Lead (2026-11-25) et issue à tracer

- **Q1 — tranchée** : le changement d'ordre des verdicts sur une requête doublement fautive (D3) est **accepté**.
- **Q2 — tranchée** : l'issue du helper de test se **trace sans question** (CLAUDE.md § *Priorités des défauts*, « on trace toujours »).
  Ouverte par l'orchestrateur (modèle `known_failure.yml` n'est pas le bon : c'est de la dette, `technical-debt`, P3). Texte proposé :

  **Titre** : `La sonde d'attente PROCESSLIST des tests de course est recopiée dans cinq fichiers de tests`
  **Labels** : `technical-debt`, `P3`, `triage`
  **Corps** : Les tests d'entrelacement forcé attendent qu'une autre connexion exécute une requête précise avant de rendre la main à celle
  qui tient le verrou : sonde `information_schema.PROCESSLIST` (`INFO LIKE ?`, `DB = DATABASE()`, dix secondes au plus). La requête
  `FROM information_schema.PROCESSLIST` est écrite dans **cinq** fichiers (`grep -rln "FROM information_schema.PROCESSLIST" crates/*/tests`,
  2026-11-25) : `crates/kesh-api/tests/credit_card_e2e.rs` (`attendre_requete_en_cours`), `crates/kesh-api/tests/inbox_import_e2e.rs`
  (`attendre_requete_bloquee`), `crates/kesh-db/tests/credit_card_repository.rs` (`attendre_requete_en_cours`),
  `crates/kesh-api/tests/mcp_transport_e2e.rs` (`lectures_du_plan_en_cours`) et `crates/kesh-db/tests/pool_lecture_seule.rs`
  (`sommeils_en_cours`) — les deux derniers pour une mesure voisine, non pour attendre un verrou. D'autres fichiers attendent par un autre
  moyen (`supplier_document_attachment.rs`, `attendre_une_requete_en_cours` ; `supplier_invoice_duplicate_lock.rs`,
  `attendre_puis_commiter` ; `bank_accounts_repository.rs:2501`, `attendre_l_ecrivain`) : à regarder avant de factoriser. **Piste** : un
  module `common` par crate de tests (celui de `kesh-api/tests/common/` existe ; `kesh-db/tests/common/` aussi), un seul helper paramétré
  par le motif. Révélée par la validation P1 de la fiche 53-3 (`refs #513`) ; la story 53-3 ajoute un test d'attente et n'en refait pas une
  copie sans le dire. *Le compte se refait par la commande ci-dessus à l'ouverture.*
- **Q3 (ouverte, mineure)** — W10 : la **modification** d'une écriture d'ouverture peut y poser une ligne de résultat (manuel
  `user-manual.tex:1065`, assumé). La story ne la traite pas. À étendre un jour ?

## Dev Agent Record

*(à remplir au développement)*

## Change Log

- 2026-11-25 — validation P1 remédiée (Opus 0/0/3/8 ; rapport `kesh-gate-logs/e533/validation-p1.md`) : V1-1 (AC-5 au niveau du dépôt, M6-bis), V1-2 (AC-5 renommé « hors D3 », test e épinglant D3, liste des refus de `validate`), V1-3 (lecture verrouillante placée avant `count_by_company`, D1/D5/P6 réécrits, M10 et #669), V1-4 (D3, alternative présentée sans « défaut »), V1-5 (T4/T8 : doc-comments exacts), V1-6 (AC-6 : deux requêtes, `const`/`range`, 1205, intervalles, autre société), V1-7 (M8, M9, M10, lecture sur le pool, mort par l'attente), V1-8 (W10, W11), V1-9 (Q2 → issue, comptes), V1-10 (ligne d'epic à ajouter), V1-11 (liste D1 complète, crate de W3, `:450`). Q1 et Q2 tranchées par le Project Lead.
- 2026-11-25 — fiche rédigée (Sonnet 5.5). Prémisses P1-P4 lues au code ; P5 (sérialisation par la société) **déduite des verrous cités et à
  mesurer** (T0, AC-6) ; P6 (instantané REPEATABLE READ) lu à la doc de `accounts.rs`. Validation P1 (Opus) : à venir.
