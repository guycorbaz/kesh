# Story 53-3 : L'ouverture ne se passe pas sur un compte de résultat — le contrôle des types, sous verrou (#513)

Status: drafted

*Fiche rédigée le 2026-11-25 (Sonnet 5.5), worktree `kesh-53-3`, branche `story/53-3-ouverture-sous-verrou` partie de
`origin/epic/53-ecritures-justes` à `4d540a6ea`, par le workflow `bmad-create-story`, en autonomie. Epic 53 « Écritures justes »,
troisième story ; release **v0.21.0** (vendredi 2026-12-04). **Aucune migration.** Priorité **P1** (une écriture d'ouverture sur un
compte de produits ou de charges fausse le résultat du premier exercice). Validation : prompt versionné à côté
(`53-3-ouverture-sous-verrou-validation-prompt-p1.md`), passe complète, Opus.*

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

**P6 — piège : une lecture simple ne suffit pas (REPEATABLE READ).** `accounts.rs:444` (doc de `count_by_company`) le dit : « sous
REPEATABLE READ, la première lecture cohérente d'une transaction fige son instantané ». Dans `create_opening_entry`, la première lecture
cohérente est `count_by_company` (`journal_entries.rs:921`), **après** les deux verrous. Une relecture des types par un `SELECT` ordinaire,
placée après elle, lirait l'instantané — et ne verrait pas un retypage qui commite entre-temps **par un écrivain qui ne prend pas la société**
(aucun aujourd'hui ; demain, un nouveau site). Elle doit être une **lecture verrouillante** (`LOCK IN SHARE MODE`, jamais `FOR SHARE` : erreur
de syntaxe sur MariaDB 10.11, `accounts.rs:627`), qui lit la dernière version commitée et attend un écrivain en cours. C'est aussi le
verrou que `validate_lines_accounts_in_tx` prend déjà sur les mêmes lignes un peu plus loin : pas de verrou neuf, un verrou **avancé**.

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
Commande de l'inventaire : `git grep -n "create_opening_in_tx\|create_opening_entry\|create_opening_complement"` hors tests, puis
`git grep -n "ouverture\|opening" -- crates/kesh-api/src/mcp crates/kesh-seed crates/kesh-db/src/repositories/fiscal_years.rs`.

| # | Chemin | Contrôle du type de compte | État |
|---|---|---|---|
| W1 | `POST /api/v1/opening-balances` → `journal_entries::create_opening_entry` | **avant**, transaction jetée (`opening_balances.rs:449-464`) | **DÉFAUT — corrigé par la story** |
| W2 | `POST /api/v1/opening-balances/complete` → `opening_complement::create_opening_complement` (portée `Screen`) | **sous verrou** : comptes lus `FOR UPDATE` par clé primaire (`opening_complement.rs:600-611`), refus `NotBalanceAccount` (`check_lines`, `:778-783`, tout ce qui n'est ni `Asset` ni `Liability`) | couvert ; test séquentiel `opening_complement_repository.rs:364`. Le refus est **plus strict** que W1 (qui ne refuse que `Revenue`/`Expense`) : sans effet, il n'existe que quatre types (`entities/account.rs:12-17`) |
| W3 | création et modification d'un compte bancaire avec solde d'ouverture → `create_opening_complement_in_tx` (portée `SingleBankLedger`, `bank_accounts.rs:1061-1076`) | même fonction que W2, mêmes verrous | couvert |
| W4 | `POST /api/v1/journal-entries` (écriture manuelle, journal OD) | aucun — **hors sujet** : ce n'est pas une ouverture, un OD peut légitimement toucher un compte de résultat | non concerné |
| W5 | import d'installation (`.keshbackup`, `admin_backup/import.rs`) | aucun : restauration brute d'une base entière | **angle mort assumé** : une sauvegarde restaure ce qui fut commité ; aucune ouverture n'est « écrite » |
| W6 | clôture d'exercice / report à nouveau | sans objet : `fiscal_years.rs` ne crée aucune écriture (le bilan est cumulatif depuis l'origine, `journal_entries.rs` commentaire de l'étape 1-a) ; recherche `create_in_tx` dans `fiscal_years.rs` : seulement des doc-comments | non concerné |
| W7 | serveur MCP et clés d'API | aucun outil d'écriture d'ouverture (`mcp/outils/grand_livre.rs`, `exercices.rs`, `rapports.rs` : lecture) ; une clé d'API qui atteint `POST /opening-balances` passe par W1 | couvert par W1 |
| W8 | données de démonstration (`kesh-seed`) | recherche `ouverture\|opening` : aucune ligne | non concerné |
| W9 | assistant d'installation (`routes/onboarding.rs`) | recherche `ouverture\|Opening` : aucune ligne ; le compte bancaire de l'assistant passe par W3 | non concerné |

Sites **non résolus** de la classe « lire un type de compte avant d'écrire, hors du verrou » : `find_types_by_ids_in_tx` n'a **qu'un seul
appelant de production**, le handler W1 (`git grep -n find_types_by_ids_in_tx` : handler `:452`, définition `accounts.rs:474`, trois tests
`opening_balances_repository.rs:302`, `:325`, `:260`). Aucun autre site de la classe. Angle mort assumé : un compte **créé** après le
contrôle ne peut être visé par une requête déjà lancée (identifiant inconnu au client).

## Décisions

- **D1 — le contrôle des types se fait dans `create_opening_entry`, après les deux verrous et la garde « société vierge » et le statut de
  l'exercice, avant `create_opening_in_tx`.** Ordre des refus dans la transaction : `NotFound` → `ALREADY_HAS_ENTRIES` (409) →
  `FIRST_YEAR_CLOSED` (400) → **`NON_BALANCE_ACCOUNT` (400, neuf)** → `INACTIVE_OR_INVALID_ACCOUNTS` / `ACCOUNT_NOT_POSTABLE` (de
  `create_in_tx`). C'est l'ordre actuel des verdicts du handler (`opening_balances.rs:398-464`, puis `create_in_tx`), à une exception
  près, D3. *Écartée* : garder la lecture au handler et la **répéter** dans la transaction — deux sites pour une règle, et un pré-contrôle
  qui laisse croire que la garde est là.
- **D2 — la lecture est verrouillante : `LOCK IN SHARE MODE`, `ORDER BY id`, scopée à la société** (P6). On **transforme**
  `accounts::find_types_by_ids_in_tx` (seul appelant de production : le handler) en lecture verrouillante plutôt que d'en ajouter une
  seconde ; on la **renomme** `lock_types_by_ids_in_tx` pour que son nom dise le verrou (un nom qui promet « lecture » aurait fait
  régresser la garde). `ids` vide → `Ok(vec![])` sans requête (inchangé). *Écartée 1* : `FOR UPDATE` — il bloquerait les écritures
  ordinaires qui lisent le compte en partage (`validate_lines_accounts_in_tx`) pour rien ; le partage suffit contre un écrivain, qui
  prend `FOR UPDATE`. *Écartée 2* : ajouter `account_type` au `SELECT` de `validate_lines_accounts_in_tx` et refuser là — ce helper sert
  les **dix-neuf** routes qui créent une écriture ; y mettre une règle d'ouverture fait porter à toutes un paramètre qui ne regarde que
  deux (`create_opening_in_tx` et son complément, déjà gardé en W2).
- **D3 — le handler perd sa transaction jetée et son pré-contrôle ; un verdict change d'ordre, et c'est écrit.** Une requête qui cumule
  un compte de résultat **et** une écriture déséquilibrée (ou à moins de deux lignes, ou à montant négatif) rendait `non-balance-account` ;
  elle rendra l'erreur de `accounting::validate`, qui précède la transaction. Même statut HTTP (400, `VALIDATION_ERROR`) ; le message
  diffère. Soumis au Project Lead (Q1) ; sans réponse contraire, le déplacement complet (la piste de l'issue) est retenu. *Écartée* :
  conserver un pré-contrôle lock-free sur le pool pour garder l'ordre — c'est le défaut même, deux sites pour une règle (D1).
- **D4 — nouvelle clé `FY_OPENING_NON_BALANCE_ACCOUNT_KEY = "fiscal_year:opening-non-balance-account"`** à côté des deux clés de la 14-4
  (`fiscal_years.rs:269`, `:275`), émise en `DbError::Invariant(<KEY>)`, re-mappée par `map_opening_balances_error`
  (`opening_balances.rs:221`) en `AppError::Validation(t("error-opening-balances-non-balance-account", …))` — **même message, mêmes
  quatre locales, aucune clé i18n neuve**. Une clé non mappée retomberait dans `AppError::from(DbError::Invariant)` = **500**.
- **D5 — aucun changement de l'ordre des verrous** (P5, AC-6). Le verrou ajouté est pris **après** `fiscal_years FOR UPDATE` et sur les
  mêmes lignes que `validate_lines_accounts_in_tx` prendra ensuite en partage (même transaction : ré-entrant, sans attente). Le graphe
  des attentes ne gagne aucune arête : `companies(X) → fiscal_years(X) → accounts(S)` existait déjà.
- **D6 — pas de migration.** Aucun schéma, aucune donnée. `kesh_version_min_required` inchangé ; P5–P8 du § *Migration breaking policy* sans
  objet. Release `c` si elle partait seule ; elle sort avec les migrations de l'Epic 53 (v0.21.0).

## Verrous : ordre et interblocage (cités, relus au code)

| Geste | Ordre des verrous | Source |
|---|---|---|
| **`create_opening_entry` (W1), après la story** | `companies` **X** → `fiscal_years` **X** → `accounts` **S** (types, nouveau) → `accounts` **S** (`validate`, ré-entrant) → compteur de numéros | `journal_entries.rs:886`, `:900`, `:276-278` |
| `accounts::update` (le retypage) | `companies` **S** → réglages de facturation **S** → `accounts` (la ligne) **X** → liens bancaires, écritures (lecture verrouillante) | `accounts.rs:1132-1137`, doc `:1090-1112` |
| `create_opening_complement_in_tx` (W2, W3) | `companies` **S** → comptes `FOR UPDATE` `ORDER BY id` → compte de report **S** → `fiscal_years` **X** (premier, puis candidat) | doc de `opening_complement.rs:445-461` |
| `create_in_tx_inner` (toute écriture) | `fiscal_years` **X** → … → `accounts` **S** (`validate_lines_accounts_in_tx`) | `journal_entries.rs:542`, `:276-278` |
| flux « exercice avant société » (rapprochements par règle, ventilation, manuel à projet) | `fiscal_years` **X** puis `companies` | doc de `create_opening_entry` (`journal_entries.rs:839-856`) : cycle **connu**, rejoué par `retry_on_deadlock` |

Lecture : `update` (société en **S**) et W1 (société en **X**) s'excluent à la première ligne : aucun des deux ne tient autre chose quand
il attend l'autre. W1 et W2/W3 s'excluent de même (société **S** contre **X**). Le seul cycle possible reste celui, déjà documenté et rejoué
(`retry_on_deadlock("opening_balances::generate", …)`, `opening_balances.rs:506`), des flux qui prennent l'exercice avant la société. Une
`DbError::Invariant` n'est jamais rejouée (`retry.rs:116`, prédicat `is_deadlock_error`).

⚠️ **À mesurer, pas à déduire** (la consigne de l'issue, et ce que la 25-7 a établi) : (a) `EXPLAIN` du `SELECT … LOCK IN SHARE MODE` neuf
sur la base de dev : l'accès doit être `range` sur `PRIMARY`, sans `Using filesort` ; sinon la lecture verrouille davantage de lignes que
les comptes visés (voir `opening_complement.rs:311-327` : un `ORDER BY` hors index a déjà verrouillé tous les exercices) ; (b) sonde à deux
sessions : session 1 tient la transaction de W1 ouverte après la lecture des types ; session 2 `SELECT id FROM accounts WHERE id = <un
compte non visé de la société> FOR UPDATE NOWAIT` **réussit** (aucun sur-verrouillage), et sur un compte visé elle échoue
(`ER_LOCK_NOWAIT`). Sorties dans `/home/gcorbaz/devel/kesh-gate-logs/e533/`.

## Acceptance Criteria

**AC-1 — le contrôle est dans la transaction qui écrit.** `create_opening_entry` lit, après le sentinelle, l'exercice, la garde « société
vierge » et le statut, le type de **chaque** compte visé par une lecture `LOCK IN SHARE MODE` (D2) et refuse `Revenue` et `Expense` par
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

**AC-5 — l'ordre des verdicts est conservé (D1).** Quatre tests : société non vierge + ligne `Revenue` → **409** `already-has-entries` ;
premier exercice clos (société vierge) + ligne `Revenue` → **400** message `first-year-closed` (et non `non-balance-account`) ; ligne
`Revenue` + compte archivé → `non-balance-account` ; ligne sur un compte d'une **autre société** → `INACTIVE_OR_INVALID_ACCOUNTS`
(non `non-balance-account`, anti-énumération KF-002 et contrat 14-4).

**AC-6 — ni sur-verrouillage ni nouvelle arête d'interblocage (D5).** Les deux mesures du § *Verrous* (`EXPLAIN` sur `PRIMARY` ;
sonde `NOWAIT` à deux sessions) sont faites sur la base de dev, leurs sorties versées à `kesh-gate-logs/e533/`, leur lecture citée au Dev
Agent Record. Si l'accès n'est pas `range` sur `PRIMARY`, la requête est réécrite (`FORCE INDEX (PRIMARY)` ou lecture par clé primaire,
une ligne à la fois, `ORDER BY id`) et la mesure refaite. La doc de `create_opening_entry` (liste des verrous, § « Portée exacte ») est
mise à jour : le verrou `accounts` y paraît.

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
  `create_opening_entry` : elle doit attendre sur `companies`). Écart avec la fiche → l'écrire au Change Log.
- [ ] **T1 — test rouge d'abord** (AC-3 et AC-4) dans `crates/kesh-db/tests/opening_balances_repository.rs` ; prouver le rouge **avant**
  le correctif (journal dans `kesh-gate-logs/e533/`).
- [ ] **T2 — `accounts.rs`** : `find_types_by_ids_in_tx` → `lock_types_by_ids_in_tx`, `ORDER BY id … LOCK IN SHARE MODE`, doc réécrite
  (« sous verrou », renvoi à P6) ; mettre à jour les deux tests existants (`opening_balances_repository.rs:260`, `:321`).
- [ ] **T3 — `fiscal_years.rs`** : constante `FY_OPENING_NON_BALANCE_ACCOUNT_KEY` ; **`journal_entries.rs`** : l'étape de contrôle dans
  `create_opening_entry` (D1), doc de la fonction mise à jour (étapes, verrous) ; garde `rollback` comme les autres refus.
- [ ] **T4 — `routes/opening_balances.rs`** : retirer la transaction jetée et le pré-contrôle (`:445-464`) ; arm de
  `map_opening_balances_error` pour la clé neuve (même `t(...)`, même repli) ; doc-comment de `generate_opening_balances` (liste des gardes,
  `:355-368`) réécrite ; le commentaire de `opening_balances_status` n'est pas touché.
- [ ] **T5 — tests d'API** (AC-2, AC-5) dans `crates/kesh-api/tests/opening_balances_e2e.rs`.
- [ ] **T6 — garde lexicale** (AC-8).
- [ ] **T7 — mesures** (AC-6), journal dans `kesh-gate-logs/e533/`.
- [ ] **T8 — mutations** (tableau ci-dessous), un journal chacune ; **greper le symptôme** après le patch : `find_types_by_ids_in_tx`,
  `non-balance`, `Revenue.*Expense` sur tout le dépôt (code, fiches, doc-comments, tests, quatre locales i18n, manuels) — la valeur, non la
  formulation ; traiter les doc-comments qui décrivent l'ancien flux (`journal_entries.rs:793`, `opening_balances.rs:261`, `:411`).
- [ ] **T9 — Dev Agent Record, Change Log, Notes de version**, `sprint-status.yaml` → `review`.

## Tests et mutations

| # | Mutation (appliquée à la main, restaurée, `touch` du fichier après restauration) | Doit rougir |
|---|---|---|
| M1 | supprimer l'étape de contrôle de `create_opening_entry` | AC-3, AC-4, AC-2 (via l'API) |
| M2 | lire les types **avant** le sentinelle `companies` (lecture simple ou verrouillante) | AC-3 |
| M3 | ne refuser que `Revenue` (retirer `Expense`) | variante `Expense` d'AC-2 et d'AC-3 |
| M4 | retirer `LOCK IN SHARE MODE` (lecture ordinaire, instantané de `count_by_company`) | **AC-4 seul** (AC-3 reste vert : la société est déjà un verrou commun — à écrire au Dev Agent Record) |
| M5 | retirer l'arm de `map_opening_balances_error` (la clé tombe en 500) | AC-2 |
| M6 | placer le contrôle **avant** la garde « société vierge » | AC-5 (409 attendu) |
| M7 | refuser aussi l'identifiant absent (`non-balance` au lieu de `INACTIVE_OR_INVALID_ACCOUNTS`) | AC-5 (compte d'une autre société) |

Une mutation **vivante** n'est pas un test réussi : l'écrire comme angle mort, avec sa raison. Angles morts **assumés d'avance** : W5
(restauration d'une sauvegarde) ; un second écrivain de `account_type` qui n'existe pas (AC-4 le garde en puissance) ; la durée d'attente
d'un verrou (`innodb_lock_wait_timeout`, hors périmètre).

## Dev Notes

- **Fichiers touchés (deux crates)** : `kesh-db` (`repositories/accounts.rs`, `journal_entries.rs`, `fiscal_years.rs`,
  trois fichiers de tests), `kesh-api` (`routes/opening_balances.rs`, un fichier de tests). Pas de frontend. **Splitting préventif : non
  déclenché** (deux modules). Recoupement d'epic : `journal_entries.rs` avec l'Epic 51 (48-2) et l'Epic 47 d'un contributeur — la
  modification est **localisée** à `create_opening_entry` ; reprendre `main` avant la revue de code (§ *Travailler à plusieurs développeurs*).
  `accounts.rs` : aucun epic ouvert connu n'y touche (contrôle du 2026-11-23, `epic-53-ecritures-justes.md`).
- **`LOCK IN SHARE MODE`, jamais `FOR SHARE`** (MariaDB 10.11 : erreur de syntaxe) ; **jamais de lecture ordinaire** pour un contrôle sous
  REPEATABLE READ (P6).
- **Le test d'entrelacement attend, il ne dort pas** : copier le patron `attendre_requete_en_cours` de
  `crates/kesh-db/tests/credit_card_repository.rs:150` (dix secondes au plus, `PROCESSLIST`, `INFO LIKE ?`). Le helper est **déjà dupliqué**
  dans cinq fichiers de tests : ne pas en faire une sixième copie sans le dire — voir Q2 (une issue `technical-debt` à ouvrir par
  l'orchestrateur, jamais par l'agent).
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

## Questions pour le Project Lead

- **Q1 (D3)** — le déplacement complet du contrôle change l'ordre de deux verdicts 400 sur une requête **doublement** fautive (compte de
  résultat **et** écriture déséquilibrée : on lira l'erreur d'équilibre, non plus « comptes de bilan »). Acceptable ? Sinon la seule voie qui
  garde l'ordre est un pré-contrôle lock-free supplémentaire, que la fiche écarte.
- **Q2** — ouvrir une issue `technical-debt` pour `attendre_requete_en_cours`, dupliqué dans cinq fichiers de tests
  (`credit_card_e2e.rs:788`, `credit_card_repository.rs:150`, `inbox_import_e2e.rs:1465`, `supplier_invoice_duplicate_lock.rs:104`,
  `bank_accounts_repository.rs:2501`) ? Le défaut voisin n'est pas mécanique à corriger dans cette story (cinq fichiers, deux crates).

## Dev Agent Record

*(à remplir au développement)*

## Change Log

- 2026-11-25 — fiche rédigée (Sonnet 5.5). Prémisses P1-P4 lues au code ; P5 (sérialisation par la société) **déduite des verrous cités et à
  mesurer** (T0, AC-6) ; P6 (instantané REPEATABLE READ) lu à la doc de `accounts.rs`. Validation P1 (Opus) : à venir.
