# Story 53-3 : L'ouverture ne se passe pas sur un compte de résultat — le contrôle des types, sous verrou (#513)

Status: review

*Fiche rédigée le 2026-11-25 (Sonnet 5.5), worktree `kesh-53-3`, branche `story/53-3-ouverture-sous-verrou` partie de
`origin/epic/53-ecritures-justes` à `4d540a6ea`, par le workflow `bmad-create-story`, en autonomie. Epic 53 « Écritures justes »,
troisième story ; release **v0.21.0** (vendredi 2026-12-04). **Aucune migration.** Priorité **P1** (une écriture d'ouverture sur un
compte de produits ou de charges fausse le résultat du premier exercice). Validation P1 (Opus) : 0/0/3/8, remédiée le 2026-11-25 (Change Log) ; validation P2 (Sonnet) : 0/0/4/8, remédiée le 2026-11-25 ; validation P3 (Opus) : 0/0/3/5, remédiée le 2026-11-25 ; validation P4 (Sonnet) : 0/0/0/6, remédiée le 2026-11-25 (V4-1 à V4-6). Q1 (D3) : **tranchée « oui »** par le Project Lead le 2026-11-25, sur la description exacte ; Q2 : issue #1580. Story unique : voir § *Dérogation règle de splitting*.*

## Story

En tant que comptable qui reprend ses soldes de départ, je veux que Kesh refuse une écriture d'ouverture qui touche un compte de produits
ou de charges **au moment même où il l'écrit**, et non quelques millisecondes avant, afin qu'un compte retypé par un collègue pendant la
génération ne fausse pas le résultat du premier exercice.

## Étape 0 — prémisses relues au code (`4d540a6ea`)

**P1 — le contrôle est bien avant l'écriture, dans une transaction jetée (confirmée).**
`crates/kesh-api/src/routes/opening_balances.rs:447-454` : `let mut tx = state.pool.begin()` puis
`let types = accounts::find_types_by_ids_in_tx(&mut tx, company.id, &account_ids).await?;` puis `let _ = tx.rollback().await;` ; le refus
suit (`:455-463`, `Revenue`/`Expense` → `AppError::Validation`, clé `error-opening-balances-non-balance-account`). L'écriture est créée plus loin,
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

**P6 — la forme et la place de la lecture des types (REPEATABLE READ, `innodb_snapshot_isolation`).** `accounts::count_by_company` (doc,
`accounts.rs:444`) le rappelle : sous REPEATABLE READ, la première lecture cohérente d'une transaction fige son instantané. Dans
`create_opening_entry`, `BEGIN` n'ouvre pas de vue (sqlx 0.8.6, `transaction.rs:279`), les deux verrous de tête (`companies`,
`fiscal_years`) sont des lectures verrouillantes, et la première lecture cohérente est `journal_entries::count_by_company`
(`journal_entries.rs:921`, `SELECT COUNT(*) FROM journal_entries` : **autre fonction**, même nom). Deux conséquences :
(i) **forme** : la lecture des types doit être **verrouillante** (`LOCK IN SHARE MODE`, jamais `FOR SHARE` : erreur de syntaxe sur MariaDB
10.11, `accounts.rs:627`). Placée avant `count_by_company`, une lecture ordinaire serait elle-même la première lecture cohérente et verrait
tout retypage commité avant elle ; ce que la lecture verrouillante apporte **en plus** : (α) elle **attend** un écrivain non commité qui ne
prendrait pas la société (aucun aujourd'hui ; demain, un nouveau site) puis lit la dernière version commitée ; (β) elle **tient** le verrou
partagé jusqu'au commit, ce qui garde la propriété (aucun retypage entre le contrôle et l'écriture) **même contre un écrivain qui ne prendrait
pas la société** ; aujourd'hui, c'est la sentinelle `companies` qui ferme #513 (P5), et M4 laisse AC-3 vert ;
(ii) **place** (validation P1, V1-3) : **juste après `fiscal_years FOR UPDATE`, avant `count_by_company`**, donc **vue non ouverte**.
C'est la règle de la 25-7 que l'issue prescrit (« verrous d'abord, puis lectures, puis refus », `opening_complement.rs:454-461`). Placée
après `count_by_company`, la lecture verrouillante s'exécuterait vue ouverte : avec `innodb_snapshot_isolation` (actif par défaut depuis
MariaDB 11.6), un commit sur la ligne du compte depuis l'ouverture de la vue rend `ER_CHECKREAD` (1020), que `map_db_error` ne traduit pas
et qu'aucun rejeu ne reprend : **500**, rien d'écrit (angle mort documenté, #669 ; `bank_imports.rs:536-545`). Sans effet sous 10.11
(`docker-compose.yml`, CI). C'est aussi le verrou que `validate_lines_accounts_in_tx` prendra ensuite sur les mêmes lignes : **verrou avancé,
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
| W1 | `POST /api/v1/opening-balances` → `journal_entries::create_opening_entry` | **avant**, transaction jetée (`opening_balances.rs:447-463`) | **DÉFAUT — corrigé par la story** |
| W2 | `POST /api/v1/opening-balances/complete` → `opening_complement::create_opening_complement` (portée `Screen`) | **sous verrou** : comptes lus `FOR UPDATE` par clé primaire (`opening_complement.rs:600-611`), refus `NotBalanceAccount` (`check_lines`, `:778-783`, tout ce qui n'est ni `Asset` ni `Liability`) | couvert ; test séquentiel `opening_complement_repository.rs:364`. Le refus est **plus strict** que W1 (qui ne refuse que `Revenue`/`Expense`) : sans effet, il n'existe que quatre types (`entities/account.rs:12-17`) |
| W3 | création et modification d'un compte bancaire avec solde d'ouverture → `create_opening_complement_in_tx` (portée `SingleBankLedger`, `crates/kesh-api/src/routes/bank_accounts.rs:1061-1076`, appel `:1068`) | même fonction que W2, mêmes verrous | couvert |
| W4 | `POST /api/v1/journal-entries` (écriture manuelle, journal OD) | aucun — **hors sujet** : ce n'est pas une ouverture, un OD peut légitimement toucher un compte de résultat | non concerné |
| W5 | import d'installation (`.keshbackup`, `admin_backup/import.rs`) | aucun : restauration brute d'une base entière | **angle mort assumé** : une sauvegarde restaure ce qui fut commité ; aucune ouverture n'est « écrite » |
| W6 | clôture d'exercice / report à nouveau | sans objet : `fiscal_years.rs` ne crée aucune écriture (le bilan est cumulatif depuis l'origine, `journal_entries.rs` commentaire de l'étape 1-a) ; recherche `create_in_tx` dans `fiscal_years.rs` : seulement des doc-comments | non concerné |
| W7 | serveur MCP et clés d'API | aucun outil d'écriture d'ouverture (`mcp/outils/grand_livre.rs`, `exercices.rs`, `rapports.rs` : lecture) ; une clé d'API qui atteint `POST /opening-balances` passe par W1 | couvert par W1 |
| W8 | données de démonstration (`kesh-seed`) | recherche `ouverture\|opening` : aucune ligne | non concerné |
| W9 | assistant d'installation (`routes/onboarding.rs`) | recherche `ouverture\|Opening` : aucune ligne ; le compte bancaire de l'assistant passe par W3 | non concerné |
| W10 | **modification** de l'écriture d'ouverture (`PUT /api/v1/journal-entries/{id}`) | aucun refus de type propre à l'ouverture : « rien n'empêche donc d'y ajouter une ligne de résultat, ce que la génération refuse » (`docs/manual/fr/user-manual.tex:1064`, retrouvé dans le PDF aplati) | **angle mort assumé, documenté au manuel** : la modification est un geste d'écriture ordinaire (toute écriture peut toucher un compte de résultat) ; ce que #513 ferme est la **génération** ; hors périmètre, la fiche ne le traite pas (issue à ouvrir, texte au § *Décisions du Project Lead*) |
| W11 | soldes d'ouverture de **relevés** (`opening_balance` de `bank_statements_manual.rs`, `bank_imports.rs`) | soldes de relevé, non des écritures | non concernés |

Sites **non résolus** de la classe « lire un type de compte avant d'écrire, hors du verrou » : `find_types_by_ids_in_tx` n'a **qu'un seul
appelant de production**, le handler W1 (`git grep -n find_types_by_ids_in_tx` : handler `:452`, définition `accounts.rs:474`, deux tests
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
- **D2 — la lecture est verrouillante : `FORCE INDEX (PRIMARY)`, `ORDER BY id`, `LOCK IN SHARE MODE`, scopée à la société, posée d'emblée**
  (P6). `FORCE INDEX (PRIMARY)` n'est **pas** conditionnel à l'`EXPLAIN` de la base de dev : la revue 15-5d (B-1) l'a établi, sur une table
  réelle l'optimiseur pourrait choisir un index secondaire commençant par `company_id` et y poser des verrous d'intervalle qui bloqueraient la
  création d'un compte de la société (`company_invoice_settings.rs`, `lock_designated_accounts_in_tx`, `:1200-1204`) ; un `EXPLAIN` de dev ne
  dit rien du plan sur le NAS (AC-6 prouve le plan **épinglé**, il ne le décide pas). On **transforme** `accounts::find_types_by_ids_in_tx`
  (seul appelant de production : le handler) en cette lecture et on la **renomme** `lock_types_by_ids_in_tx` (le nom dit le verrou). `ids`
  vide → `Ok(vec![])` (inchangé). *Écartée 1 — réemployer `lock_designated_accounts_in_tx`* (Story 35-3, #537 : « seul helper de verrou d'une
  liste de comptes », lit `account_type` dans la même requête verrouillante, même classe de défaut fermée pour quatre comptes de réglage) :
  son patron `owned_account_ids` (C88) ouvre d'abord une **lecture ordinaire** des identifiants de la société — qui ouvrirait la vue **avant**
  la lecture verrouillante, contre la place de D1 (P6 ii) — et ses sorties sont des `LockedDesignatedAccount`, propres aux réglages. La
  conséquence assumée de ne pas appliquer C88 : un identifiant d'une **autre** société est verrouillé en partage avant d'être refusé
  (`INACTIVE_OR_INVALID_ACCOUNTS`) ; c'est déjà le comportement de `validate_lines_accounts_in_tx`, qui lit sans pré-filtre
  (`journal_entries.rs:276-278`) : aucune régression. *Écartée 2* : `FOR UPDATE` — il bloquerait les écritures ordinaires qui lisent le compte
  en partage ; le partage suffit contre un écrivain, qui prend `FOR UPDATE`. *Écartée 3* : ajouter `account_type` au `SELECT` de
  `validate_lines_accounts_in_tx` — ce helper sert les dix-neuf routes qui créent une écriture ; y mettre une règle d'ouverture fait porter à
  toutes un paramètre qui ne regarde que deux. Sa requête **n'est pas réécrite** par la story (hors périmètre) : si l'`EXPLAIN` de AC-6 montre
  qu'elle surverrouille, c'est une **issue**, non un patch ici.
- **D3 — le handler perd sa transaction jetée et son pré-contrôle des types ; l'ordre de deux familles de verdicts change. Décision Q1 du
  Project Lead du 2026-11-25, sur cette description exacte : OUI, accepté.** Aujourd'hui le contrôle des types (`opening_balances.rs:443-463`)
  précède `accounting::validate` (`:481`). Après la story, `validate` précède la transaction : une requête qui cumule un compte de résultat
  **et** un refus de `validate` rendra le refus de `validate`. **Statut HTTP inchangé (400). Le code machine change pour deux familles** :
  (1) le **déséquilibre** rendra **`ENTRY_UNBALANCED`** (`errors.rs:2745`) ; (2) les refus de partie en devise **que juge
  `accounting::validate`** (`check_foreign_line`, `kesh-core/src/accounting/fx.rs:216-247` ; il faut un **cours et** un débit ou un crédit
  fournis : cours hors bornes, montant en devise ≤ 0, `round(montant × cours, 2)` nul, débit différent du CHF calculé) rendront leur code
  **`FX_*`** (`errors.rs:3246-3250`). **Ne changent pas** : les refus de l'**analyse de la ligne**, `parse_line_amounts`
  (`crates/kesh-api/src/routes/journal_entries.rs:790-846` : somme hors bornes, `chf` fourni contredisant le montant, cours déduit hors bornes, ligne sans cours ni CHF de référence, ligne sans débit ni crédit),
  qui précèdent déjà le contrôle des types (`opening_balances.rs:391-405`) et **gardent leur code** (le plus souvent `FX_*` ; `VALIDATION_ERROR` pour « cours ou montant CHF requis » et « côté requis ») ; et libellé vide, moins de deux lignes,
  montant négatif, débit et crédit à la fois, total hors intervalle (restent `VALIDATION_ERROR`). Le **message** change dans tous les cas.
  Pour un client de l'API (clé REST), la requête « compte de résultat + déséquilibre » lisait `VALIDATION_ERROR` et lira `ENTRY_UNBALANCED`.
  **Qualification au regard de CLAUDE.md § *Versions de l'API*** : la précédence des refus de `POST /opening-balances` n'est **pas documentée**
  (`docs/api-external.md` ne dit rien de `non-balance-account` ni de son ordre) ; ce n'est donc pas une rupture du contrat, mais un changement
  de précédence, **annoncé ⚠️ en tête des notes de version**, sous `/api/v1`, release `c` si elle partait seule ; précédent assumé : C-15-8-16
  (`routes/journal_entries.rs`, `create_journal_entry` : « un corps déséquilibré ET sans exercice rend désormais `ENTRY_UNBALANCED` avant
  `NO_FISCAL_YEAR` »). Le changement est **épinglé par un test** qui asserte le **code** (AC-5 e).
- **D4 — nouvelle clé `FY_OPENING_NON_BALANCE_ACCOUNT_KEY = "fiscal_year:opening-non-balance-account"`** à côté des deux clés de la 14-4
  (`fiscal_years.rs:269`, `:275`), émise en `DbError::Invariant(<KEY>)`, re-mappée par `map_opening_balances_error`
  (`opening_balances.rs:221`) en `AppError::Validation(t("error-opening-balances-non-balance-account", …))` — **même message, mêmes
  quatre locales, aucune clé i18n neuve**. Une clé non mappée retomberait dans `AppError::from(DbError::Invariant)` = **500**.
- **D5 — aucune arête neuve dans le graphe des attentes, avec la place de D1.** Les verrous **détenus** quand la lecture des types demande
  `accounts` en partage sont `companies X` et `fiscal_years X` ; ce sont exactement ceux qui sont détenus quand `validate_lines_accounts_in_tx`
  le demande aujourd'hui (entre les deux, `books_locked_through` et `find_later_closed` sont des lectures ordinaires, le second
  `fiscal_years FOR UPDATE` est ré-entrant, `validate_taggable_in_tx` sort tôt sans projet : `projects.rs:94-96`) ; `count_by_company` est
  une lecture non verrouillante. L'arête `{companies X, fiscal_years X} → accounts S` existe donc déjà : la story la **déplace** de
  quelques instructions, sans la créer. Les gestes qui **écrivent** `accounts` prennent tous `companies` en partage d'abord (`create` `:199`, `update` `:1132`,
  `archive_guarded` `:1453`, `reactivate` `:1640`, complément `opening_complement.rs:454-457`) : aucun cycle avec le sentinelle exclusif.
- **D6 — pas de migration.** Aucun schéma, aucune donnée. `kesh_version_min_required` inchangé ; P5–P8 du § *Migration breaking policy* sans
  objet. Release `c` si elle partait seule ; elle sort dans la release de l'Epic 53 (v0.21.0 au fichier d'epic ; `b` si la 53-2 porte sa migration).

## Verrous : ordre et interblocage (cités, relus au code)

| Geste | Ordre des verrous | Source |
|---|---|---|
| **`create_opening_entry` (W1), après la story** | `companies` **X** → `fiscal_years` **X** → `accounts` **S** (types, avancé) → [`count_by_company`, lecture ordinaire] → `accounts` **S** (`validate`, ré-entrant) → compteur de numéros | `journal_entries.rs:886`, `:900`, `:276-278` |
| `accounts::update` (le retypage) | `companies` **S** → réglages de facturation **S** → `accounts` (la ligne) **X** → liens bancaires, écritures (lecture verrouillante) | `accounts.rs:1132-1137`, doc `:1090-1112` |
| `create_opening_complement_in_tx` (W2, W3) | `companies` **S** → comptes `FOR UPDATE` `ORDER BY id` → compte de report **S** → `fiscal_years` **X** (premier, puis candidat) | doc de `opening_complement.rs:450-461` |
| `create_in_tx_inner` (toute écriture) | `fiscal_years` **X** → … → `accounts` **S** (`validate_lines_accounts_in_tx`) | `journal_entries.rs:542`, `:276-278` |
| flux « exercice avant société » (rapprochements par règle, ventilation, manuel à projet) | `fiscal_years` **X** puis `companies` | doc de `create_opening_entry` (`journal_entries.rs:839-856`) : cycle **connu**, rejoué par `retry_on_deadlock` |

Lecture : `update` (société en **S**) et W1 (société en **X**) s'excluent à la première ligne : aucun des deux ne tient autre chose quand
il attend l'autre. W1 et W2/W3 s'excluent de même (société **S** contre **X**). Les cycles **préexistants** restent, tous rejoués
(`retry_on_deadlock("opening_balances::generate", …)`, `opening_balances.rs:506`) : (1) les flux qui prennent l'exercice avant la société ;
(2) les cycles « par un compte visé » (25-7, `25-7-soldes-de-depart.md` § *Cycles résiduels*), formés par les gestes qui tiennent une ligne
`accounts FOR UPDATE` **sans l'écrire** et sans prendre `companies` d'abord — **inventaire clos** (relu par la P2) :
`supplier_invoices.rs:2233`, `:2668`, `invoice_settlements_write.rs:264`, `grouped_settlements.rs:361`, `company_invoice_settings.rs:743`, `:871`,
`:918`, `:1540`. La story ne les crée pas : l'arête `{companies X, fiscal_years X} → accounts S` est celle de
`validate_lines_accounts_in_tx` ; seules les requêtes refusées en `ALREADY_HAS_ENTRIES` / `FIRST_YEAR_CLOSED` prennent désormais ce verrou
avant de refuser (le handler les écarte presque toujours par deux pré-contrôles lock-free). T0 rejoue cet inventaire. Une
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
n'ouvre plus de transaction et ne lit plus de type (AC-8 : le handler ne contient ni `lock_types_by_ids_in_tx` ni `find_types_by_ids_in_tx`).

**AC-2 — le verdict HTTP ne change pas.** `POST /api/v1/opening-balances` avec une ligne `Revenue` — et avec une ligne `Expense` — rend
**400 `VALIDATION_ERROR`**, message `error-opening-balances-non-balance-account` (« … comptes de bilan … ») ; aucune écriture, aucune ligne
d'audit. Le test existant `post_revenue_account_400_non_balance_account` (`opening_balances_e2e.rs:571`) reste vert, un jumeau `Expense`
s'y ajoute.

**AC-3 — la course de #513 est fermée, et le test rougit sans le correctif.** Test d'entrelacement forcé (assistant partagé `kesh_db::test_fixtures::attendre_une_requete_en_cours`, `test_fixtures.rs:593`, motifs
`["FROM companies WHERE id = ?", "FOR UPDATE"]`, jamais `INNODB_TRX`, jamais un `sleep`) : une connexion
`tenue` ouvre une transaction, prend `companies` en partage comme `accounts::update`, retype le compte cible en `Revenue` **sans commiter** ;
`create_opening_entry` est lancée (tâche) et **vue en attente** sur la société ; `tenue` commite ; l'appel doit rendre
`Invariant(FY_OPENING_NON_BALANCE_ACCOUNT_KEY)`, **aucune** écriture. Sans le correctif, l'appel rend `Ok` et une ligne d'ouverture est
posée sur un compte de produits : le test est rouge. (Variante `Expense` en table.)

**AC-4 — la lecture verrouille vraiment (P6 i).** Second entrelacement, **sans** `companies` : la connexion `tenue` fait seulement
`UPDATE accounts SET account_type = 'Revenue' WHERE id = ?` (un écrivain qui ne prendrait pas la société) et ne commite pas ;
`create_opening_entry` est vue en attente **sur la lecture des types** (assistant partagé, motifs `["account_type FROM accounts", "LOCK IN
SHARE MODE"]` — le motif doit distinguer cette lecture de celle de `validate_lines_accounts_in_tx`, qui attendrait aussi sur le même
compte sous M4) ; `tenue` commite ; refus `NON_BALANCE`. Tue la mutation « lecture ordinaire » (M4), que l'AC-3 ne tue pas (la société est un
verrou commun, et la lecture ordinaire ouvre sa vue **après** le commit de `tenue`) : M4 meurt par (α) et (β) de P6 (i) — la lecture ne
**attend** pas, puis ne **tient** pas le verrou.

**AC-5 — l'ordre des verdicts est conservé, hors D3.** Tests **au niveau du dépôt** (`opening_balances_repository.rs`, appel direct de
`create_opening_entry`) : les pré-contrôles lock-free du handler (`opening_balances.rs:426`, `:436`) répondent avant la transaction, un
test par l'API ne voit donc pas l'ordre sous verrou (V1-1) : (a) société avec une écriture + ligne `Revenue` →
`Invariant(FY_OPENING_ALREADY_HAS_ENTRIES_KEY)` ; (b) premier exercice clos (société vierge) + ligne `Revenue` →
`Invariant(FY_OPENING_FIRST_YEAR_CLOSED_KEY)` ; (c) ligne `Revenue` + une autre ligne sur un compte **archivé** → `NON_BALANCE` (le contrôle
précède `INACTIVE_OR_INVALID_ACCOUNTS`) ; (d) ligne sur un compte `Revenue` **d'une autre société** → `INACTIVE_OR_INVALID_ACCOUNTS` (non
`NON_BALANCE` : anti-énumération KF-002, contrat 14-4). Par l'API (`opening_balances_e2e.rs`), contrôle d'absence de régression : 409 et
`first-year-closed` inchangés ; (e) **épinglage de D3, sur le code**, chaque cas **rouge à `4d540a6ea`** (journal à l'appui) : une ligne `Revenue` **et** (1) une écriture déséquilibrée
→ **`ENTRY_UNBALANCED`** ; (2) une partie en devise **jugée par `validate`** — cours **et** débit fournis, `foreign.amount = "-5"` — →
**`FX_VALUE_OUT_OF_RANGE`** (une ligne en devise dont `chf` contredit le débit, ou dont le cours déduit sort des bornes, est refusée par `parse_line_amounts` et rendait
déjà `FX_*` avant la story : elle n'épingle rien) ; (3) une seule ligne → **`VALIDATION_ERROR`** (message de `validate`, non « comptes de bilan »).

**AC-6 — ni sur-verrouillage ni nouvelle arête d'interblocage (D5).** (i) La requête neuve porte `FORCE INDEX (PRIMARY)` **d'emblée** (D2) ;
les mesures (a) à (d) du § *Verrous* (sur la base de dev, au moins deux identifiants) **constatent** le plan épinglé (`const` ou `range` sur
`PRIMARY`, sans `Using filesort`), leurs sorties versées à `kesh-gate-logs/e533/`, leur lecture citée au Dev Agent Record. (ii) Pour la requête de
`validate_lines_accounts_in_tx` (`journal_entries.rs:276-278`, dix-neuf routes), qui n'est **pas** réécrite par la story : un plan autre que
`const` ou `range` sur `PRIMARY` ouvre une **issue**, citée au Dev Agent Record, et la sonde d'`INSERT` (c) devient **obligatoire** ; jamais une
réécriture ici. La doc de `create_opening_entry` (liste des verrous, § « Portée exacte ») est mise à jour : le verrou `accounts` y paraît, avec sa
place et la raison (vue non ouverte, #669).

**AC-7 — les chemins couverts le restent (inventaire W2, W3).** Les tests existants de `opening_complement_repository.rs` et
`bank_account_opening_balance_e2e.rs` passent sans modification ; **aucun** de leurs fichiers de production n'est touché. Le contrôle
lexical `chf_line_lexical.rs` reste vert (`create_opening_in_tx` : deux appelants).

**AC-8 — le concept ne se perd pas.** Test lexical `crates/kesh-db/tests/` sur le patron de `chf_line_lexical.rs`, **sur le texte masqué**
(`lexique::decouper` : commentaires, chaînes et tests masqués — sinon le doc-comment de `create_opening_entry`, qui nommera la fonction,
fait passer la garde à vide après suppression de l'appel) et le corps de `create_opening_entry` délimité par `fonction_englobante` : il
**contient** un appel à `lock_types_by_ids_in_tx` ; le handler `opening_balances.rs` n'en contient ni de `lock_types_by_ids_in_tx` ni de
`find_types_by_ids_in_tx`.
Sondes : positive (le fichier source est lu, sinon échec bruyant) et **négative** (un appel retiré, une copie du corps sans l'appel, rougit).
Limite écrite : lexical, un renommage en alias le contourne ; il garde le geste, non l'intention.

**AC-9 — gates.** `cargo fmt`, `clippy -D warnings`, gate ciblé `binary(opening_balances_repository)` + `binary(opening_balances_e2e)` entre
deux passes ; gate complet (`scripts/gate-ci.sh`) et E2E Playwright complet au dernier commit de code (aucun changement d'écran : l'E2E
n'est qu'un contrôle d'absence de régression). Le story file n'affirme que ce qui a tourné.

## Tâches

- [x] **T0 — étape 0 du développement (avant tout code).** Rejouer à la main : `git grep -n "find_types_by_ids_in_tx"` (appelants) et
  `git grep -n "SET .*account_type"` sur `crates/*/src` (un seul écrivain, `accounts.rs:1271`) ; relire les verrous de
  `accounts::update` (`:1132-1137`) et de `create_opening_entry` ; **recontrôler P5** par une sonde sur la base de dev (une session
  `UPDATE accounts SET account_type='Revenue'` sous `companies LOCK IN SHARE MODE`, non commitée ; une seconde exécute
  `create_opening_entry` : elle doit attendre sur `companies`) ; **revérifier l'absence d'arête neuve** (D5) avec la place de D1. Écart avec la fiche → l'écrire au Change Log.
- [x] **T1 — test rouge d'abord** (AC-3, AC-4 **et AC-5 (e) (1)(2)(3)**, ces derniers rouges à `4d540a6ea` **avant** T4) (AC-3, AC-4 dans `crates/kesh-db/tests/opening_balances_repository.rs` ; AC-5 (e) dans `crates/kesh-api/tests/opening_balances_e2e.rs`), **en réemployant
  `kesh_db::test_fixtures::attendre_une_requete_en_cours`** (`test_fixtures.rs:593`, Story 25-3-a-1, crochet `interrompre`), non en en copiant une ; prouver le rouge **avant**
  le correctif (journal dans `kesh-gate-logs/e533/`).
- [x] **T2 — `accounts.rs`** : `find_types_by_ids_in_tx` → `lock_types_by_ids_in_tx`, `ORDER BY id … LOCK IN SHARE MODE`, doc réécrite
  (« sous verrou », renvoi à P6) ; mettre à jour les deux tests existants (`opening_balances_repository.rs:260`, `:321`, dont les doc-comments `:4` et `:257`).
- [x] **T3 — `fiscal_years.rs`** : constante `FY_OPENING_NON_BALANCE_ACCOUNT_KEY` ; **`journal_entries.rs`** : l'étape de contrôle dans
  `create_opening_entry` (D1), doc de la fonction mise à jour (étapes, verrous) ; garde `rollback` comme les autres refus.
- [x] **T4 — `routes/opening_balances.rs`** : retirer la transaction jetée et le pré-contrôle des types (`:443-463`) ; arm de
  `map_opening_balances_error` pour la clé neuve (même `t(...)`, même repli) ; doc-comments à reprendre : `:360-363` (point 2 de la liste des
  gardes), `:443-445` (commentaire du pré-contrôle, supprimé avec lui), `:219-220` (« Toute autre erreur retombe vers le mapping global » :
  citer la clé neuve). **Ne pas toucher** `:261` (doc de `opening_balances_status`) ni `:411` (pré-check 1 UX) : toujours vraies.
- [x] **T5 — tests** : AC-2 et AC-5 (e) par l'API (`crates/kesh-api/tests/opening_balances_e2e.rs`) ; **AC-5 (a) à (d) au niveau du dépôt**
  (`opening_balances_repository.rs`, appel direct de `create_opening_entry`, sur le montage de `create_opening_entry_rejected_if_company_has_entries_in_any_fy`,
  `…_first_year_closed`, `…_not_found_for_missing_fy`).
- [x] **T6 — garde lexicale** (AC-8).
- [x] **T7 — mesures** (AC-6), journal dans `kesh-gate-logs/e533/`.
- [x] **T8 — mutations** (tableau ci-dessous), un journal chacune ; **greper le symptôme** après le patch : `find_types_by_ids_in_tx`,
  `non-balance`, `Revenue.*Expense` sur tout le dépôt (code, fiches, doc-comments, tests, quatre locales i18n, manuels) — la valeur, non la
  formulation. Sites à reprendre, relus au code : `opening_balances.rs:216-217`, `:360-363` (T4) ; `opening_balances_e2e.rs:548`
  (`/// dans \`find_types_by_ids_in_tx\`, P3-ECH-LOW-1).`) ; le doc-comment de `create_opening_entry` (`journal_entries.rs:~839-856`, liste
  des verrous). `journal_entries.rs:793` (doc de `count_by_company`) et `opening_balances.rs:261`, `:411` **ne changent pas**.
- [x] **T9 — Dev Agent Record, Change Log, Notes de version**, `sprint-status.yaml` → `review`.

## Tests et mutations

| # | Mutation (appliquée à la main, restaurée, `touch` du fichier après restauration) | Doit rougir |
|---|---|---|
| M1 | supprimer l'étape de contrôle de `create_opening_entry` | AC-3, AC-4, AC-2 (via l'API : sans contrôle sous verrou, la requête `Revenue` rend 201) |
| M2 | lire les types **avant** le sentinelle `companies` (lecture simple ou verrouillante) | AC-3 (lecture simple) ; variante verrouillante : par l'attente seule, voir ci-dessous |
| M3 | ne refuser que `Revenue` (retirer `Expense`) | variante `Expense` d'AC-2 et d'AC-3 |
| M4 | retirer `LOCK IN SHARE MODE` (lecture ordinaire des types) | **AC-4 seul** (AC-3 reste vert : la société est déjà un verrou commun, et la lecture ordinaire ouvre sa vue après le commit de `tenue`) ; meurt par (α) et (β) de P6 (i) — à écrire au Dev Agent Record |
| M5 | retirer l'arm de `map_opening_balances_error` (la clé tombe en 500) | AC-2 (le handler n'a plus de pré-contrôle : la requête `Revenue` atteint l'arm et rend 500) |
| M6 | placer le **refus** avant la garde « société vierge » | AC-5 (a), **au niveau du dépôt** (V1-1) |
| M6-bis | placer le **refus** avant le contrôle du statut `Closed` | AC-5 (b), au niveau du dépôt |
| M7 | refuser aussi l'identifiant absent (`non-balance` au lieu de `INACTIVE_OR_INVALID_ACCOUNTS`) | AC-5 (d) |
| M8 | placer le contrôle **après** `create_opening_in_tx` | AC-5 (c) : `INACTIVE` passe alors avant `NON_BALANCE` (la ligne archivée est une autre ligne que le compte `Revenue`) |
| M9 | lecture **non scopée par la société** (`WHERE id IN …` seul) | test renommé de `opening_balances_repository.rs:260` (compte `foreign` absent du résultat) ; AC-5 (d) |
| M11 | retirer `FORCE INDEX (PRIMARY)` ou `ORDER BY id` de `lock_types_by_ids_in_tx` | **aucun test** (le plan de la base de test reste `PRIMARY`, l'ordre de verrouillage n'est pas observé) : **angle mort écrit**, gardé par la revue et par la mesure (a) d'AC-6 ; la garde lexicale d'AC-8 peut en outre exiger les deux motifs dans les **littéraux** de `lock_types_by_ids_in_tx` (premier retour de `lexique::decouper`, la fonction délimitée par `fonction_englobante` : la requête est une chaîne, donc absente du texte masqué) |
| M12 | rétablir au handler un pré-contrôle des types **avant** `accounting::validate` | AC-5 (e) (1)(2)(3) : les codes redeviennent `VALIDATION_ERROR` |
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
  déclenché** (deux modules). Recoupements : `journal_entries.rs` avec **la story 53-1 de la même epic** (`epic-53-ecritures-justes.md:27`), l'Epic 51 (48-2) et
  l'Epic 47 d'un contributeur — la modification est **localisée** à `create_opening_entry` ; le second arrivé reprend l'epic avant sa revue de
  code (§ *Travailler à plusieurs développeurs*). À ajouter à la ligne « Recoupements » de l'epic : « 53-1 et 53-3 sur `journal_entries.rs`
  (fonctions distinctes) ».
  `accounts.rs` et `fiscal_years.rs` : **non déclarés** au fichier d'epic (`epic-53-ecritures-justes.md:31` ne cite que `opening_balances.rs` et `journal_entries.rs`) et aucun contrôle de recoupement ne les couvre : à faire à la reprise de `main` (`git diff --name-only main...<branche>` des branches d'epic ouvertes sur ces deux fichiers). **Ligne à ajouter au fichier d'epic** (écrite par l'orchestrateur, « Fichiers et modules ») : `- 53-3 : kesh-api/src/routes/opening_balances.rs ; kesh-db/src/repositories/journal_entries.rs (create_opening_entry), accounts.rs (lecture des types), fiscal_years.rs (clé d'erreur) ; tests kesh-db et kesh-api.`
- **`LOCK IN SHARE MODE`, jamais `FOR SHARE`** (MariaDB 10.11 : erreur de syntaxe) ; **jamais de lecture ordinaire** pour un contrôle sous
  REPEATABLE READ (P6).
- **Le test d'entrelacement attend, il ne dort pas** : **réemployer** `kesh_db::test_fixtures::attendre_une_requete_en_cours`
  (`test_fixtures.rs:593` ; motifs du texte préparé, `?` compris ; `interrompre` rend la vraie issue si la tâche finit sans attendre ;
  emplois : `bank_account_opening_balance_e2e.rs:24`, `bank_accounts_closed_e2e.rs:1397`, …). **Ne pas copier** les assistants locaux
  (`credit_card_*`, `inbox_import_e2e.rs`) : leur migration vers l'assistant partagé est l'objet de l'issue **#1580**.
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
peut plus recevoir une ligne d'ouverture. Aucun changement pour l'utilisateur dans le cas courant : même message de refus.
**⚠️ Pour un client de l'API** (`POST /api/v1/opening-balances`) : une requête qui porte **à la fois** un compte de résultat **et** un
déséquilibre rend désormais `ENTRY_UNBALANCED` (au lieu de `VALIDATION_ERROR`), et une partie en devise jugée invalide par la validation de
l'écriture (cours **et** débit ou crédit CHF fournis) rend son code `FX_*` ; le statut reste 400, la précédence de ces refus n'était pas documentée. Aucune
migration. `refs #513`.

## Décisions du Project Lead (2026-11-25) et issues

- **Q1 — tranchée OUI (2026-11-25), sur la description exacte de D3** : le changement d'ordre des verdicts est accepté (`ENTRY_UNBALANCED` et les
  `FX_*` jugés par `accounting::validate`, au lieu de `VALIDATION_ERROR` ; statut 400 inchangé).
- **Q2 — tracée** : issue **#1580** (helper d'attente `PROCESSLIST`, migration vers `kesh_db::test_fixtures::attendre_une_requete_en_cours`).
- **W10 — issue à ouvrir** par l'orchestrateur (« on trace toujours », CLAUDE.md § *Priorités des défauts*). Texte proposé :
  **Titre** : `La modification de l'écriture d'ouverture peut y poser une ligne de compte de résultat`
  **Labels** : `enhancement`, `P3`, `triage` (le Project Lead peut la requalifier `bug` P2 : la génération refuse ce que la modification admet).
  **Corps** : `POST /api/v1/opening-balances` refuse les comptes de produits et de charges (le bilan d'ouverture ne touche que des comptes de
  bilan). Mais l'écriture d'ouverture, une fois générée, « se modifie comme toute écriture saisie à la main » (`docs/manual/fr/user-manual.tex:1064`,
  « rien n'empêche donc d'y ajouter une ligne de résultat, ce que la génération refuse ») : `PUT /api/v1/journal-entries/{id}` n'a aucun refus de
  type propre à l'ouverture. C'est une **limite documentée**, non un contrôle. À décider : refuser un compte de résultat dans la modification
  de l'écriture d'ouverture (comment la reconnaître : journal OD à la date de début du premier exercice, description de la langue comptable ?),
  ou maintenir la limite et la garder au manuel. Hors de la story 53-3 (`refs #513`), qui ne ferme que la fenêtre entre contrôle et
  écriture de la **génération**.

## Dérogation règle de splitting

Signal déclaré au Project Lead : la validation P3 a rendu 3 MEDIUM **recyclés** (V3-1 et V3-2 nés du correctif de V2-1, V3-3 reliquat de V2-2),
après P1 (3 MEDIUM) et P2 (4 MEDIUM, dont deux nés de correctifs). **La story reste unique** (décision du Project Lead du 2026-11-25) :
(1) elle touche **deux crates** (`kesh-db`, `kesh-api`), loin du plafond de cinq modules, et elle est minimale (une lecture verrouillante, une
clé d'erreur, un arm de mapping, des tests) ; (2) **un découpage ne supprimerait pas la cause** : les défauts recyclés étaient des précisions de
**texte** autour d'une seule décision, D3 (Q1), non une taille ni une dépendance ; (3) **Q1 est tranchée** : la branche « non », qui
doublait D3, AC-1, AC-5, AC-8, T4, les mutations et les Notes de version, est **retirée** — la branche qui recyclait n'existe plus.

## Dev Agent Record

*Développement par `bmad-dev-story` (Sonnet 5.5), 2026-11-25, base `kesh_533` (MariaDB 10.11.19). Journaux : `kesh-gate-logs/e533/`.*

### T0 — mesures avant tout code (journaux `t0-sonde.sh`, `t0-mesures.txt`, `t0-sonde2.sh`, `t0-mesures-bis.txt`)

- **Inventaire rejoué** : `git grep find_types_by_ids_in_tx` : un seul appelant de production (`opening_balances.rs:452`), définition `accounts.rs:474` ;
  `git grep "SET .*account_type"` sur `crates/*/src/*` : un seul écrivain, `accounts.rs:1271`. Conforme à P3.
- **P5 mesurée** : session A `companies LOCK IN SHARE MODE` + `UPDATE accounts SET account_type='Revenue'` non commité ; session B `companies FOR UPDATE`
  (le sentinelle) : **attend 4,04 s** (la prise de A durait 6 s moins 2 s de décalage), puis lit `Revenue` dans la lecture verrouillante des types. Les deux gestes
  se sérialisent bien sur la société.
- **AC-4 mesuré** : écrivain **sans** la société (`UPDATE accounts` seul, non commité) ; `SELECT … FORCE INDEX (PRIMARY) … LOCK IN SHARE MODE` : **attend 3,03 s**
  et lit `Revenue` (la lecture ordinaire ne l'aurait pas attendu).
- **(a) EXPLAIN, quatre identifiants** : requête neuve (`FORCE INDEX (PRIMARY)`, `ORDER BY id`) : `range` sur `PRIMARY`, sans `Using filesort`. Requête de
  `validate_lines_accounts_in_tx` : **`range` sur `PRIMARY`** statistiques fraîches (table de 11 comptes, puis de ~300), mais **`ALL`** au premier passage, sur une table
  dont les statistiques n'avaient pas été recalculées après l'ajout de six comptes. Sans `FORCE INDEX`, la requête neuve choisit aussi `range` (avec cinq clés candidates).
- **(b)(c)** sur les deux requêtes séparément, 11 puis ~300 comptes : un compte non visé (13) en `FOR UPDATE NOWAIT` **réussit** ; un compte visé (9, 4) échoue en
  **1205** ; l'`INSERT` d'un compte de la société (timeout 1 s) **réussit**. Avec le plan `ALL` du premier passage, tout échouait en 1205, `INSERT` compris (balayage complet
  de `accounts` sous verrou partagé) : c'est le défaut pressenti de la requête de `validate`, **antérieur à la story**.
- **Conclusion** : aucune prémisse démentie ; la story procède. **Constat à tracer** (AC-6 (ii)) : le plan de la requête de `validate_lines_accounts_in_tx` dépend des
  statistiques (`ALL` mesuré, donc surverrouillage de toute la table et blocage des créations de comptes) ; une issue est demandée à l'orchestrateur, aucune réécriture ici.

### Commits (branche `story/53-3-ouverture-sous-verrou`, base `4d540a6ea`)

`4b61e8c04` (étape 0 : V4-1 à V4-6, T0) ; `ff2c982dd` (tests rouges + clé `FY_OPENING_NON_BALANCE_ACCOUNT_KEY` seule) ; `607fb31e8` (**dernier commit de
code** : correctif et garde lexicale) ; `98e2051af` (fusion de `origin/main` = `fbfa64a27`, qui ne touche aucun fichier de la story) ; `a4e7c2f47` (prompt de la
revue P1) ; ce commit (Dev Agent Record, statut).

### Ce qui a été écrit

- `accounts::lock_types_by_ids_in_tx` (ex-`find_types_by_ids_in_tx`) : `FORCE INDEX (PRIMARY)`, `ORDER BY id`, `LOCK IN SHARE MODE`, scopée société.
- `create_opening_entry` : étape 2-bis (lecture sous verrou, après `fiscal_years FOR UPDATE`, avant `count_by_company`), étape 4-bis (refus `Revenue`/`Expense`
  après société vierge et statut) ; doc mise à jour (étapes, ordre des verrous : `companies` X → `fiscal_years` X → `accounts` S).
- `fiscal_years.rs` : `FY_OPENING_NON_BALANCE_ACCOUNT_KEY` ; handler : transaction jetée et pré-contrôle retirés, arm de `map_opening_balances_error`, doc-comments
  repris, import `accounts` retiré. Aucune clé i18n neuve, aucune migration, aucun frontend, aucune modification de manuel.
- Tests : `opening_balances_repository.rs` (AC-3 et AC-4 en deux variantes, refus séquentiel, AC-5 a à d, renommage des deux tests de lecture) ;
  `opening_balances_e2e.rs` (jumeau Expense d'AC-2, AC-5 e (1)(2)(3)) ; `opening_types_sous_verrou_lexical.rs` (AC-8, sondes positive et négative, motifs de M11).

### Rouges au code de base (journaux `rouge-kesh-db.log`, `rouge-kesh-api.log`)

Production inchangée sauf la constante : `opening_balances_repository` 6 rouges sur 19 (AC-3 Revenue et Expense : l'appel rend `Ok` ; AC-4 Revenue et Expense : panique de
l'attente, requête de lecture des types jamais vue ; refus séquentiel : `Ok` ; AC-5 c : `InactiveOrInvalidAccounts` au lieu de `NON_BALANCE`) ; `opening_balances_e2e` 3 rouges sur
35 (AC-5 e (1)(2)(3) : code `VALIDATION_ERROR`, message « comptes de bilan »). Les cas AC-5 a, b, d sont des gardes de non-régression, verts dès le départ.

### Mutations (pilote `mutations.py`, journaux `mutations/<M>-db.log`, `-api.log` ; codes 100 = tuée)

| Mutation | Résultat | Tuée par |
|---|---|---|
| M1 | tuée (db, api) | AC-3, AC-4, refus séquentiel, AC-2 / e2e |
| M2 simple (lecture ordinaire avant le sentinelle) | tuée (db) | AC-3 |
| M2 verrouillante (avant le sentinelle) | tuée (db) | **panique de l'attente** `["FROM companies WHERE id = ?", "FOR UPDATE"]`, vérifiée au journal (les deux courses) |
| M3 (sans `Expense`) | tuée (db, api) | variantes Expense |
| M4 (sans `LOCK IN SHARE MODE`) | tuée (db) | **panique de l'attente** `["account_type FROM accounts", "LOCK IN SHARE MODE"]` (AC-4, deux variantes), vérifiée au journal ; la garde lexicale rougit aussi (motif retiré du littéral). AC-3 seul reste vert, comme prévu : la lecture ne **attend** pas, puis ne **tient** pas le verrou — (α) et (β) de P6 i |
| M5 (arm de mapping retiré) | tuée (api) | AC-2 |
| M6, M6-bis | tuées (db) | AC-5 a, b |
| M7 | tuée (db) | AC-5 d |
| M8 | tuée (db) | AC-5 c |
| M9 | tuée (db) | test de portée par société, AC-5 d |
| M10 (lecture après `count_by_company`) | **vivante** (25 sur 25 verts) | **angle mort écrit** : aucun test sous 10.11 ; ne devient visible que sous MariaDB ≥ 11.6 (#669) ; protégée par le doc-comment et la place |
| M11 (sans `FORCE INDEX`, sans `ORDER BY id`) | tuées (db) | la garde lexicale `la_requete_epingle_index_ordre_et_verrou` (la fiche annonçait « aucun test » : la garde AC-8 en ajoute un) |
| M12 (pré-contrôle rétabli au handler) | tuée (api) | AC-5 e (1)(2)(3) |

Incident de procédure : la première campagne s'est arrêtée à la première mutation sur un motif périmé (rustfmt avait reformaté l'appel) ; le pilote ouvrait le fichier avant de
calculer la mutation et l'avait tronqué ; restauré par `git checkout` et `touch`, motifs rendus uniques par fonction, quinze motifs vérifiés avant relance. Rien n'a été commité muté.

### Gates (au dernier commit de code `607fb31e8`, puis après fusion de `main`)

- Avant fusion : `scripts/test-fast.sh --ci` **6580 sur 6580** passés (1 lent, 17 ignorés) ; `scripts/gate-ci.sh kesh_ci_533` **6599 passés, 0 échoué, 21 ignorés**.
- Après fusion (`98e2051af`, migration Stripe appliquée à `kesh_533`) : `scripts/test-fast.sh --ci` **6721 sur 6721** passés (1 lent, 17 ignorés), `gate-test-fast-2.log`.
- Frontend : `check`, `lint-i18n-ownership`, `build` : code 0 ; `test:unit` **5261 tests sur 308 fichiers** passés, `gate-front.log`.
- `cargo fmt --check` et `clippy --workspace --all-targets -D warnings` : verts (`clippy-1.log`).
- **E2E Playwright complet** (backend après `cd` dans le worktree, `KESH_LOCALES_DIR` absolu, ligne « 4 locales chargées » vérifiée, `LANG=fr_CH.UTF-8`, `KESH_TEST_MODE=true`
  des deux côtés, port 3533, base `kesh_e2e_533`) : **527 passés, 20 ignorés, 7 échecs en 24,0 min**, journaux `e2e-run.log`, `e2e-backend.log`, `e2e-playwright.log`. Les sept
  échecs, jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus*, sont exactement ceux de la KF-029 (#97) : `mode-expert.spec.ts:26` et `:41`,
  `onboarding-path-b.spec.ts:65` et `:92`, `onboarding.spec.ts:57`, `:77` et `:150`. Aucun autre échec, aucun échec de pollution. Aucun écran touché : pas de capture.
  (L'E2E a tourné sur le code de `607fb31e8` fusionné avec `main` ; les commits suivants ne touchent que des comptes rendus.)

### Mesures et lecture de l'AC-6

Voir § T0. Lecture citée : requête neuve `type = range`, `key = PRIMARY`, `rows = 4`, sans `Using filesort` (`t0-mesures.txt`, `t0-mesures-bis.txt`) ; un compte non visé en
`FOR UPDATE NOWAIT` réussit, un compte visé échoue en 1205, l'`INSERT` réussit (sonde (c) faite, puisque le plan de `validate` a été vu en `ALL` une fois).
**Issue à ouvrir (AC-6 ii)** : le plan de `validate_lines_accounts_in_tx` dépend des statistiques (`ALL` mesuré sur statistiques périmées, `range` après `ANALYZE`) ; texte prêt :
`kesh-gate-logs/e533/issue-plan-validate.md`. Issue W10 : texte au § *Décisions du Project Lead*. Les deux, à ouvrir par l'orchestrateur.

### Recontrôles

- Grep du symptôme : `find_types_by_ids_in_tx` ne subsiste que dans la constante de la garde lexicale et dans la prose historique de `_bmad-output` (14-4, 25-7) ; `non-balance` : un seul
  mapping, quatre locales, aucun site périmé. Manuel (PDF aplati) : « jamais les comptes de produits ou de charges » (`user-manual.tex:1051`) reste vrai ; **aucune modification de manuel**.
- Mots-clés de fermeture dans les commits de la branche : aucun (`refs #513` seulement).
- Recoupements déclarés : `journal_entries.rs` avec la 53-1 (fonctions distinctes), `accounts.rs` et `fiscal_years.rs` non déclarés au fichier d'epic (ligne à ajouter par l'orchestrateur, § Dev Notes).
- Limite : AC-3 et AC-4 prouvent la fermeture sur MariaDB 10.11 ; le comportement sous ≥ 11.6 (`ER_CHECKREAD`) n'est protégé que par la place de la lecture (M10 vivante).


## Change Log

- 2026-11-25 — **validation terminée** (entrée unique, CLAUDE.md § *Review Iteration Rule* ; rapports `kesh-gate-logs/e533/validation-p1.md` à `-p4.md`).
  **Tendance** (CRITICAL/HIGH/MEDIUM/LOW) : P1 0/0/3/8 → P2 0/0/4/8 → P3 0/0/3/5 → P4 0/0/0/6 ; arrêt sur la P4 (aucun finding au-dessus de LOW ; la
  dernière remédiation ne touche que la fiche, sans passe ciblée). **Modèles** : P1 Opus, P2 Sonnet, P3 Opus, P4 Sonnet. **Reclassements** : aucun.
  **Remédiations** — P1 : AC-5 au niveau du dépôt et M6-bis (V1-1), AC-5 « hors D3 » (V1-2), lecture verrouillante placée avant `count_by_company`
  (V1-3 ; D1, D5, P6, M10, #669), V1-4 à V1-11 (citations, mesures de AC-6, W10/W11, Q2 → #1580, ligne d'epic) ; P2 : D3 exacte sur les **codes** (V2-1),
  `FORCE INDEX (PRIMARY)` d'emblée et réemploi de `lock_designated_accounts_in_tx` écarté par écrit (V2-2), assistant partagé
  `test_fixtures::attendre_une_requete_en_cours` (V2-3), raisons (α) et (β) de P6 (V2-4), V2-5 à V2-12 ; P3 : Q1 tranchée « oui » et branche « non » retirée
  (V3-1), refus de `accounting::validate` seuls modifiés (V3-2), AC-6 aligné sur D2 (V3-3), V3-4 à V3-8 ; P4 : V4-1 (D3 : plage `routes/journal_entries.rs:790-846`,
  « gardent leur code »), V4-2 (Notes de version : « cours **et** débit ou crédit CHF fournis »), V4-3 (T1 porte le rouge de AC-5 (e) avant T4 ; M12), V4-4
  (M11 : motifs dans les littéraux, non le texte masqué), V4-5 (AC-1 : contrôle par AC-8), V4-6 (D6 : « dans la release de l'Epic 53 »).
  **Signal de recyclage** levé en P2 et P3 (défauts nés des correctifs, tous de précision de texte) ; dérogation au splitting : § *Dérogation règle de splitting*.
- 2026-11-25 — fiche rédigée (Sonnet 5.5). Prémisses P1-P4 lues au code ; P5 (sérialisation par la société) déduite des verrous puis **mesurée à T0** (Dev Agent Record).
- 2026-11-25 — **développement : T0 exécuté, aucune prémisse démentie** (Dev Agent Record § *T0*) ; statut `in-progress`.
- 2026-11-26 — **développement terminé** : correctif, tests, mutations (M10 vivante, angle mort écrit), gates et E2E verts (Dev Agent Record) ; statut `review`. Revue de code P1 : prompt versionné `53-3-ouverture-sous-verrou-code-review-prompt-p1.md`.
