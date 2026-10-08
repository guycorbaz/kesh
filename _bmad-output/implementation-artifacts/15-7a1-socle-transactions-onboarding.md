# Story 15.7a1 : Le socle transactionnel de l'onboarding

Status: review

<!-- Née le 2026-10-08 du découpage de la 15-7a (choix C-15-7-19), à la passe de validation P2.
     Patron « story-zéro + rollout » du CLAUDE.md (§ Règle de splitting préventif) : cette fiche
     pose les variantes `_in_tx` et les primitives `kesh-db` que la 15-7a2 emploie, SANS changement
     de comportement ni d'audit. Choix applicables : C-15-7-6 (une transaction par route, variantes
     `_in_tx`), C-15-7-15 (helpers société, fin du MIRROR), C-15-7-17 (taux TVA ligne à ligne),
     C-15-7-19 (découpage), C-15-7-20 (emplacement des helpers), C-15-7-21 (`clear_stub_in_tx`),
     C-15-7-26 (test 8, exception assumée — corrigé par C-15-7-30), C-15-7-30 (test 8 complété par
     `finalize`, séquence d'audit avec `user.created`, SQL du verrou, décompte de six modules).
     Validation : née de la P2 de la 15-7a ; P1 (2 MEDIUM), P2 (1 HIGH, 2 MEDIUM) et P3 (1 MEDIUM, fiche seule)
     appliquées ; **close** après la P3. Choix C-15-7-33 (P3). -->

**Issue** : `refs #434` — partout, PR comprise. Cette fiche ne ferme rien.

**Dépendances** : aucune en amont. **La 15-7a2 dépend de celle-ci** et ne commence qu'après son merge ;
la 15-7b1 dépend de la 15-7a2, la 15-7b2 de la 15-7b1 (découpage de la 15-7b, C-15-7-31).

## Story

En tant que **développeur de la 15-7a2, de la 15-7b1 et de la 15-7b2**,
je veux que **chaque écriture de l'onboarding existe en variante qui s'exécute dans la transaction de
l'appelant et rend ce qu'elle a réellement fait**,
afin que **les routes puissent ensuite tenir la mutation, l'étape et la trace dans une seule
transaction**, sans qu'aucune de ces extractions ne change ce que l'application fait aujourd'hui.

## Décompte de modules — signal D5 déclaré

À la granularité retenue par C-15-7-8 (celle des exemples de la règle, `kesh-api/routes/invoices`) :
`kesh-db/repositories/{accounts, bank_accounts, company_invoice_settings, vat_rates, companies,
onboarding}` — **six modules de code**. `kesh-api/routes/onboarding` n'est **pas** touché : ses deux
appels (`:721`, `:745`) compilent inchangés malgré les types de retour changés (R-6/F-11 de la P2 —
établi à la lecture, pas au compilateur : `let _settings = match … { Ok(s) => s, … }` absorbe le tuple,
`if let Err(e) = …` ignore le `Vec`). Le test 8 touche un fichier de test de `kesh-api`, non compté.
Le seuil de cinq est franchi. **Le signal est déclaré au Project Lead.** Motif de ne pas découper
davantage (F-5 de la P2, motif rectifié) : c'est la **story-zéro** du patron du CLAUDE.md, qui pose les
primitives que la 15-7a2, la 15-7b1 et la 15-7b2 emploient ; **quatre** modules y font une extraction mécanique
(`accounts`, `bank_accounts`, `company_invoice_settings` : « commit » retiré de la fonction, enveloppe
pool conservée ; `vat_rates` en plus change d'algorithme — quatre `INSERT` d'une ligne au lieu d'un
`INSERT` multi-lignes), **deux** reçoivent un ajout ponctuel de quelques lignes (`companies`,
`onboarding`) ; aucune règle métier, aucun comportement neuf hors l'exception assumée de l'AC 1.

## Acceptance Criteria

**1. Aucun changement de comportement observable.** Toutes les enveloppes pool gardent leur signature
et leur effet (commit compris) ; aucune entrée d'audit n'est ajoutée ni retirée ; aucune route ne
change de code de retour. Les tests existants des six repositories, `onboarding_e2e`,
`onboarding_path_b_e2e` et `fiscal_years_e2e` restent verts **sans modification d'assertion** — seuls
deux des trois appels directs de `insert_with_defaults_in_tx` dans
`crates/kesh-db/tests/company_invoice_settings_repository.rs` s'adaptent au type de retour (AC 4) :
`:1084` et `:1174`, qui emploient ensuite les réglages, prennent `.0` ; `:953` ne fait que
`matches!(result, Err(DbError::InactiveOrInvalidAccounts))` et compile **inchangé** (vérifié, pas
adapté). Le test 8 **ajoute** à `full_path_b_flow` une requête (`POST /api/v1/onboarding/finalize`)
et des assertions ; il ne modifie aucune assertion existante.

**Exception assumée au « sans changement de comportement »** (F-3 de la P1) : là où une enveloppe pool
annule aujourd'hui par `tx.rollback().await.map_err(map_db_error)?` sur une sortie d'**erreur**
(`company_invoice_settings.rs:666`, `bank_accounts.rs:195`, `accounts.rs:991` — ce dernier dans
`bulk_create_from_chart`, sur `last_insert_id == 0`, inatteignable en pratique mais inventorié, R-4/F-4
de la P2), elle passe au rollback **best-effort**
(`let _ = tx.rollback().await;`) suivi de l'**erreur d'origine**. Seul diffère le cas où le rollback
lui-même échoue : l'erreur de base remplaçait jusqu'ici l'erreur métier — ce qui, sur
`InactiveOrInvalidAccounts`, cassait la boucle de retry de `seed_demo`. Durcissement voulu, déjà de
règle sur la branche voisine (`company_invoice_settings.rs:596-600`, « P6-M2 »).

**Comportement des enveloppes pool, branche par branche** (R2 de la P1) — chaque enveloppe est
`begin` + variante + l'issue ci-dessous, et rien d'autre :

| La variante rend | L'enveloppe pool |
|---|---|
| un succès d'écriture (`Created`, `Updated`, `(s, true)`, `(s, false)`, `Vec` de comptes ou de taux) | `commit`, puis la valeur projetée sur l'ancien type de retour |
| `Ok(Unchanged(a))` (`upsert_primary`) | `rollback` comme aujourd'hui (`bank_accounts.rs:170-177`, « rien n'a été modifié » ; son erreur éventuelle reste propagée par `map_db_error`, sortie non-erreur inchangée), puis `Ok(a)` |
| `Err(OptimisticLockConflict)` (`upsert_primary`) | rollback best-effort, puis `Err(OptimisticLockConflict)` |
| `Err(e)` quelconque | rollback best-effort, puis `Err(e)` **telle quelle** |
| — (`bulk_create_from_chart`, `entries.is_empty()`) | le court-circuit `return Ok(vec![])` reste **dans l'enveloppe, avant `begin`** (`accounts.rs:941-943`) ; la variante, appelée avec une liste vide, rend `Ok(vec![])` sans requête |

**2. `accounts`.**
- `bulk_create_from_chart_in_tx(tx, company_id, chart, lang) -> Result<Vec<Account>, DbError>` : les
  comptes **réellement insérés** ; **aucun** `rollback` ni `commit` interne (`accounts.rs:991` sort de
  la variante) — sur erreur, la variante rend l'erreur et c'est l'appelant qui annule.
  `bulk_create_from_chart` (pool) = `begin` + variante + `commit` ; ses appelants (`kesh-seed:159`,
  `routes/onboarding.rs:382`, `admin_full_import_e2e:760`, tests de `company_invoice_settings_repository`)
  sont inchangés.
- `count_by_company` devient **générique sur l'exécuteur** (`<'e, E: Executor<'e, Database = MySql>>`,
  patron `journal_entries::count_by_company`, `journal_entries.rs:495`) : un seul appelant
  (`routes/onboarding.rs:377`), donc **pas** d'enveloppe pool distincte — la 15-7a2 l'appellera avec
  `&mut *tx` (R2-6).

**3. `bank_accounts`.** `upsert_primary_in_tx(tx, new) -> Result<UpsertPrimaryOutcome, DbError>`,
`enum UpsertPrimaryOutcome { Created(BankAccount), Updated { before: BankAccount, after: BankAccount },
Unchanged(BankAccount) }`. Le court-circuit no-op existant (`is_no_op_change`, KF-004) rend
`Unchanged` **sans** `rollback` (`bank_accounts.rs:176`, `:195` sortent de la variante : « inchangé »
n'annule rien, l'appelant doit pouvoir écrire son étape) ; le conflit de version rend
`OptimisticLockConflict` sans annuler. `upsert_primary` (pool) garde `-> Result<BankAccount, DbError>`
(le compte de `Created`, `after` ou `Unchanged`) ; ses appelants — `routes/onboarding.rs:531`, les trois
tests E2E de PDF et d'e-mail (`invoice_pdf_e2e:165`, `invoice_frozen_pdf_e2e:161`,
`invoice_send_email_e2e:203`) et les six appels de `crates/kesh-db/tests/bank_accounts_repository.rs`
(`:101`, `:116`, `:145`, `:160`, `:189`, `:203`) — sont inchangés.

**4. `company_invoice_settings` — fin du MIRROR.** `insert_with_defaults_in_tx` rend
`(CompanyInvoiceSettings, bool)`, le booléen valant « inséré » (`rows_affected == 1` de l'`INSERT
IGNORE`, `repositories/company_invoice_settings.rs:743-763` ; la branche `rows == 0` est `:765-791`).
`insert_with_defaults` (pool) garde sa signature et **appelle** la variante : `begin`, variante,
`commit` ; sur erreur, `rollback` *best-effort* puis **l'erreur d'origine rendue telle quelle** — la
boucle de retry de `seed_demo` reconnaît `DbError::InactiveOrInvalidAccounts` à cette variante exacte
(`crates/kesh-seed/src/lib.rs:208`, `Err(DbError::InactiveOrInvalidAccounts) if retries < max_retries` ;
le rollback best-effort existant est commenté « P6-M2 » à `company_invoice_settings.rs:596-600`). La
branche `rows == 0` de la variante est à `:768`. La duplication et ses marqueurs `MIRROR` (`:522-524`, `:558`, `:694`) disparaissent ;
le docstring dit la délégation. L'appel de `finalize` (`routes/onboarding.rs:721`) **compile inchangé** :
`let _settings = match … { Ok(s) => s, … }` lie le tuple sans l'employer (vérifié à la lecture, R-6/F-11
de la P2) — ne pas le toucher ; l'usage du booléen est à la 15-7a2.

**5. `vat_rates` — taux ligne à ligne** (C-15-7-17). `seed_default_swiss_rates_in_tx` rend
`Vec<VatRate>` — les taux **réellement** insérés, **dans l'ordre du seed** (`normal`, `special`,
`reduced`, `exempt`). L'`INSERT IGNORE` multi-lignes (`vat_rates.rs:289-303`, signature `:285`) devient **quatre `INSERT
IGNORE` d'une ligne** ; une ligne est insérée ssi `rows_affected == 1`, son id est `last_insert_id`,
la ligne rendue est relue par id. L'assertion `count >= 4` (`:305-318`) est conservée. L'enveloppe pool
`seed_default_swiss_rates` garde `-> Result<(), DbError>` ; l'appel de `finalize` (`:744-745`,
`if let Err(e) = …`) **compile inchangé** avec `Result<Vec<VatRate>, _>` (vérifié à la lecture).

**6. `companies::clear_stub_in_tx(tx, company_id) -> Result<bool, DbError>`** (C-15-7-21) —
**SQL fixé** : `UPDATE companies SET is_stub = FALSE, version = version + 1 WHERE id = ? AND is_stub =
TRUE` ; rend `rows_affected == 1`. Effet sur `version` : **+1 si et seulement si** le drapeau était
levé ; rien sinon. Aucun appelant dans cette fiche (fonction publique de bibliothèque : pas
d'avertissement `dead_code`) ; la 15-7a2 (`coordinates`) et la 15-7b1 (`seed_demo`) l'emploient.
**Changement de sémantique assumé, porté par la 15-7b1** (F-3 de la P3) : le site qu'elle remplacera
(`routes/onboarding.rs:211`, `UPDATE companies SET is_stub = FALSE WHERE is_stub = TRUE`) n'est borné à
aucune société et **ne** bumpe **pas** `version` ; `clear_stub_in_tx` est borné à `id = ?` et bumpe —
comme le fait déjà `update_company_coordinates`. Sur la voie démo, la version de la société bouge donc
désormais : écrit dans la fiche 15-7b1 (AC 1).

**7. `onboarding::lock_state_in_tx(tx) -> Result<Option<OnboardingState>, DbError>`** (C-15-7-20) —
le verrou aujourd'hui écrit trois fois dans `routes/onboarding.rs` (`:250`, `:653`, `:806`, toutes
`WHERE singleton = TRUE FOR UPDATE`). Primitive **sans** comparaison d'étape ni `rollback` : elle rend
la ligne verrouillée, ou `None`. **SQL littéral fixé** (F-3/R-3 de la P2) — une constante neuve du
module, **`pub const LOCK_SQL: &str`** (R3-3/F-4 de la P3 : le test 7, binaire d'intégration externe,
l'exécute sur sa seconde connexion ; une constante privée y serait invisible, une copie littérale
dériverait ; son doc-comment dit « partagée avec le test 7 du dépôt — n'est pas une API à
étendre ») :

```sql
SELECT id, step_completed, is_demo, ui_mode, version, created_at, updated_at
FROM onboarding_state WHERE singleton = TRUE FOR UPDATE
```

lue par **`fetch_optional`**. Les colonnes sont celles de `SELECT_SQL` (`repositories/onboarding.rs:12`),
**sans** `singleton`, que l'entité ne porte pas ; mais **`SELECT_SQL` n'est pas réutilisée** : elle
finit par `LIMIT 1` et n'a pas de `WHERE`, et `SELECT_SQL` + `FOR UPDATE` serait une autre requête
(balayage + `LIMIT` au lieu de l'index unique `uq_onboarding_singleton`, squash `:841` — autres
verrous de trou sur table vide, précisément le cas `None`). Les copies `:250` et `:653` lisent
aujourd'hui par `fetch_one` (ligne absente ⇒ `RowNotFound` ⇒ 500) : le sort de `None` est décidé par
chaque appelant. La comparaison d'étape (`lock_state_at_step`, `kesh-api`) est à la 15-7a2, qui
remplace **deux** des trois copies (`:653`, `:806`) ; la troisième (`:250`, `reset`) est remplacée par
la 15-7b2 (R3-3 de la P3 de la 15-7a2) ; `kesh-seed` (15-7b1 et 15-7b2) emploie la primitive directement (DRY —
finding L-4 de la P2 de la 15-7b). Aucun appelant dans cette fiche.
**Ordre d'appel, écrit dans le doc-comment** (F-2 de la P3) : `lock_state_in_tx` s'appelle **en
premier** dans la transaction, avant toute lecture cohérente (non verrouillante). Sous REPEATABLE READ,
la première lecture cohérente fige l'instantané ; une garde fondée sur une lecture ultérieure (le
`count_by_company(&mut *tx) == 0` de la 15-7a2) n'est exacte que si le verrou d'état l'a précédée. Ordre
des verrous de référence : `docs/MULTI-TENANT-SCOPING-PATTERNS.md:322` (onboarding_state, puis company,
puis accounts).

**8. Les doc-comments que la story rend faux disent le nouvel état.** Greps exécutés, sorties collées au
Dev Agent Record (F-9 de la P2 : l'ancien motif `tx\.rollback` rendait 42 lignes presque toutes hors
périmètre) :
`grep -nE "MIRROR|intentionally duplicated|Duplication note|count_by_company" crates/kesh-db/src/repositories/{accounts,bank_accounts,company_invoice_settings,vat_rates}.rs`,
puis, pour les annulations, `grep -nF "rollback"` **restreint aux fonctions du périmètre** par leurs
plages (`sed -n 920,1010p accounts.rs`, `sed -n 150,240p bank_accounts.rs`, `sed -n 510,700p
company_invoice_settings.rs`, `sed -n 740,800p company_invoice_settings.rs`) — tri manuel, déclaré.
Sites connus : `company_invoice_settings.rs:522-524` (note de duplication), `:558` et `:694`
(marqueurs `MIRROR` de `insert_with_defaults`). ⚠️ Le `MIRROR` de `:69`/`:101` et l'en-tête de module
`:11-17` portent sur `get_or_create_default{,_in_tx}` — **hors périmètre**, à laisser. Les sites qui
deviennent faux seulement quand un appelant **écrit** une trace (« ne génère PAS d'entrées d'audit »,
« Pas d'audit log ») sont à la 15-7a2 ; ceux qui nomment `seed_demo` comme appelant d'une enveloppe pool
(`vat_rates.rs:323-324`, `company_invoice_settings.rs:516`, `:598`) sont à la 15-7b1 — l'attribution
complète est dans la fiche index `15-7-trace-onboarding.md`.
**Doc-comments des fonctions dont le contrat change ici** (R5 de la P1) — chacun dit ce que la fonction
rend et qu'elle ne commite pas : `bulk_create_from_chart_in_tx` (neuve ; l'en-tête de
`bulk_create_from_chart`, `accounts.rs:925-934`, « dans une transaction unique », dit la délégation),
`count_by_company` (exécuteur générique ; voit les écritures non commitées de la transaction ; la ligne
orpheline « Liste les comptes d'une company, triés par numéro » de `accounts.rs:258` **disparaît**, F-8
de la P2),
`upsert_primary_in_tx` et `UpsertPrimaryOutcome` (neufs ; `upsert_primary`, `bank_accounts.rs:152-155`,
dit la projection sur `BankAccount` et le rollback sur `Unchanged`), `insert_with_defaults_in_tx` (le
booléen « inséré »), `seed_default_swiss_rates_in_tx` (`vat_rates.rs:278-284` : « insère les 4 taux »
devient « rend les taux réellement insérés, dans l'ordre du seed »), `clear_stub_in_tx` et
`lock_state_in_tx` (neufs).

## Tasks / Subtasks

- [x] **T1 — `accounts`** (AC 2) — `bulk_create_from_chart_in_tx`, enveloppe, `count_by_company` générique.
- [x] **T2 — `bank_accounts`** (AC 3) — `UpsertPrimaryOutcome`, `upsert_primary_in_tx`, enveloppe.
- [x] **T3 — `company_invoice_settings`, `vat_rates`** (AC 4, 5) — fin du MIRROR, taux ligne à ligne, adaptation de deux appels de test (`:1084`, `:1174`) ; les deux appels de `finalize` et le troisième appel de test (`:953`) compilent inchangés.
- [x] **T4 — `companies::clear_stub_in_tx`, `onboarding::lock_state_in_tx`** (AC 6, 7) ; doc-comments (AC 8).
- [x] **T5 — Tests et gates** (Dev Notes § Tests) — `kesh-db` touché ⇒ **gate complet même en cours de boucle** ; base de gate remise à zéro avant ; le frontend n'est pas touché, mais l'E2E complet tourne au dernier commit de code (règle D7) et se juge contre les échecs attendus de `docs/testing.md`.

## Dev Notes

### Ce que la story ne fait pas

- **Aucune route ne change de transaction**, aucune entrée d'audit n'est écrite : c'est la 15-7a2.
- **`record_step_completed_in_tx` n'est pas ici** (C-15-7-20) : il écrit l'action
  `installation.step_completed`, que la garde bilatérale `crates/kesh-api/tests/audit_label_registry.rs`
  exige déclarée dans `ACTIONS` et libellée dans les quatre `.ftl` dès qu'un site de production l'écrit
  — c'est-à-dire le travail d'audit de la 15-7a2.
- Ni manuel, ni CHANGELOG : rien de visible ne change.

### Points de vigilance

- **Les tests « par les deux variantes »** de `company_invoice_settings_repository.rs` (`:1063-1093`,
  `:1149-1180`, docstring `:933-938`) deviennent **tautologiques** dès que l'enveloppe délègue : ils ne
  prouvent plus une parité, seulement la délégation. Réécrire leur docstring en ce sens ; ne pas les
  compter comme garde de parité.
- **Ordre de rendu des taux** : la 15-7a2 assertera `vat_rate.created` dans l'ordre du seed ; la
  variante rend donc les taux dans cet ordre, et non par `ORDER BY id`.
- **`count_by_company` générique** : la forme `&mut *tx` voit les insertions non commitées de la même
  transaction — c'est ce que la 15-7a2 exige pour la garde `existing == 0`. Son doc-comment (AC 8) le
  dit, et ajoute la condition de validité de la garde (F-2 de la P3) : appelée **après**
  `lock_state_in_tx`, jamais avant (cf. AC 7, « Ordre d'appel »).

### Tests

Attribut squash **dans la graphie exacte qu'admet le garde-fou** `crates/kesh-db/tests/test_schema_guard.rs:62`
(`SQUASH_SPELLINGS`) : `#[sqlx::test(migrations = "./test-schema")]` dans `kesh-db`,
`#[sqlx::test(migrations = "../kesh-db/test-schema")]` dans `kesh-api` (R3-1/F-1 de la P3 : la graphie
`"test-schema"` sans `./` y est refusée et rougit le garde-fou au gate complet). Fichiers, un par test :

| Tests | Fichier |
|---|---|
| 1, 2 | **`crates/kesh-db/tests/accounts_repository.rs`, à créer** — il n'existe aujourd'hui que `accounts_role_backfill.rs` (vrai `MIGRATOR`, hors sujet) ; les tests de `mod tests` d'`accounts.rs` sont des `#[tokio::test]` sur la base de dev partagée, que la story ne prolonge pas |
| 3 | `crates/kesh-db/tests/bank_accounts_repository.rs` |
| 4 | `crates/kesh-db/tests/company_invoice_settings_repository.rs` |
| 5 | `crates/kesh-db/tests/vat_rates_repository.rs` |
| 6 | `crates/kesh-db/tests/companies_repository.rs` |
| 7 | `crates/kesh-db/tests/onboarding_repository.rs` |
| 8 | `crates/kesh-api/tests/onboarding_path_b_e2e.rs` (graphie `kesh-api`, déjà celle du fichier) |

| # | Test | AC |
|---|---|---|
| 1 | `bulk_create_from_chart_in_tx` : rend N comptes, N = `COUNT(*)` vu **dans** la transaction ; `tx.rollback()` après l'appel ⇒ `COUNT(*) = 0` (aucun commit interne). Et l'appel avec une liste **vide** (`&[]`) rend `Ok(vec![])`, la transaction restant utilisable (une requête suivante réussit) — sans le court-circuit, `IN ()` est une erreur SQL (R3-6 de la P3) | 2 |
| 2 | `count_by_company` générique : après une insertion non commitée dans une transaction, **assertion croisée** — par `&mut *tx` le compte vaut n + 1, par `&pool` (autre connexion) il vaut n (F-5 de la P3 : c'est la visibilité transactionnelle qui est prouvée, pas seulement la signature) | 2 |
| 3 | `upsert_primary_in_tx` : `Created` ; `Updated { before, after }` (`before` = valeurs d'avant, `after.version = before.version + 1`) ; `Unchanged` (version inchangée, transaction toujours utilisable : une requête suivante réussit) ; `rollback` après `Created` ⇒ aucune ligne | 3 |
| 4 | `insert_with_defaults_in_tx` : `(s, true)` puis, sur la même société, `(s, false)` ; l'enveloppe pool rend les mêmes réglages ; `InactiveOrInvalidAccounts` traverse l'enveloppe **inchangée** (société sans compte débiteurs) | 4 |
| 5 | `seed_default_swiss_rates_in_tx` : société vide ⇒ quatre taux dans l'ordre `normal`, `special`, `reduced`, `exempt`, ids = ceux de la base ; société dotée au montage de **deux taux qui heurtent la clé unique `(company_id, rate, valid_from)` des défauts** (8.10 et 2.60 au 2024-01-01) ⇒ deux taux rendus (`special`, `exempt`) ; second appel ⇒ `Vec` vide | 5 |
| 6 | `clear_stub_in_tx` — montage : `companies::create` (société non stub), puis `UPDATE companies SET is_stub = TRUE WHERE id = ?` en SQL brut pour la société stub (aucune fonction de `kesh-db` n'insère de stub ; `insert_stub_company` est privée à `kesh-api`, `bootstrap.rs:347`). Société stub ⇒ `true`, `is_stub = FALSE`, `version + 1` ; second appel ⇒ `false`, `version` inchangée ; société non stub ⇒ `false` | 6 |
| 7 | `lock_state_in_tx` — montage : `onboarding::init_state(&pool)` pose le singleton (le squash ne l'insère pas). La fonction rend la ligne ; tant que sa transaction est ouverte, une **connexion dédiée hors du pool** (`MySqlConnection::connect_with(&pool.connect_options())`, fermée en fin de test — **pas** de `SET SESSION` sur une connexion qui retournerait au pool) exécute `SET SESSION innodb_lock_wait_timeout = 1` puis la requête `LOCK_SQL` de l'AC 7, qui échoue en **1205** — le verrou est tenu ; après `rollback`, la même requête réussit. Puis `onboarding::delete_state` ⇒ `None` | 7 |
| 8 | **Parcours de production complet, état après `finalize`** — dans `crates/kesh-api/tests/onboarding_path_b_e2e.rs::full_path_b_flow` (`:216`), qui s'arrête aujourd'hui à `bank-account` (étape 7) et **n'appelle pas `finalize`** (`grep -nF "finalize"` sur le fichier : aucune sortie). Le test **ajoute**, après l'étape 7, `POST /api/v1/onboarding/finalize` ⇒ **200**, `stepCompleted = 8` (le plan `independant.json` porte `Receivable` et `DefaultRevenue` : pas de `InactiveOrInvalidAccounts`), puis les assertions d'état : `vat_rates` de la société = **4** lignes (`normal`, `special`, `reduced`, `exempt`) ; `company_invoice_settings` = **une** ligne ; `audit_log` (`ORDER BY id`) = **exactement `["user.created", "fiscal_year.created"]`** — la première écrite par le bootstrap (`ensure_admin_user`, cas `(0, true)` à `auth/bootstrap.rs:83`, `audit_log::insert_in_tx` de `user.created` à `:148` ; appelé par le test à `onboarding_path_b_e2e.rs:219`), la seconde par `fiscal_years::create_if_absent_in_tx` (`fiscal_years.rs:314`) ; aucune autre (vérifié : `companies::create` n'audite pas, `routes/auth.rs` n'audite que le mot de passe, `routes/onboarding.rs` n'écrit aucune trace, cf. #434). Ainsi complété, c'est le **seul** test qui traverse les quatre fonctions touchées (`bulk_create_from_chart` à `accounting-language`, `count_by_company == 0` ; `upsert_primary` à `bank-account` ; `insert_with_defaults_in_tx` et `seed_default_swiss_rates_in_tx` à `finalize`) : ceux de `fiscal_years_e2e.rs` appellent `finalize` mais pré-insèrent les comptes (`seed_minimal_chart`) et passent par `skip-bank`. La 15-7a2 remplacera cette séquence par celle de son AC 2, préfixée de `user.created` | 1, 4, 5 |

⚠️ **Prouver que les tests mordent** — chaque mutation appliquée seule, puis le fichier **touché**
(cargo garde sinon le binaire muté) :

| Test | Mutation | Attendu |
|---|---|---|
| 1 | `sqlx::query("COMMIT").execute(&mut **tx).await.map_err(map_db_error)?;` ajouté en fin de `bulk_create_from_chart_in_tx` (un `tx.commit()` ne compile pas : la variante ne reçoit que `&mut Transaction`, et `commit` consomme `self` ; pas de `From<sqlx::Error> for DbError`, d'où le `map_err`, R3-4 de la P3) | rouge : le `rollback` du test est sans effet, `COUNT(*) = N` |
| 1b | court-circuit `entries.is_empty()` retiré de la variante | rouge : l'appel à vide rend une erreur SQL |
| 2 | signature de `count_by_company` repassée sur `&MySqlPool` | rouge **par non-compilation** (l'appel par `&mut *tx` ne compile plus) — preuve de signature, non de comportement ; le comportement est prouvé par l'assertion croisée du test 2 (F-5 de la P3) |
| 3 | dans `upsert_primary_in_tx`, la branche no-op rend `Ok(UpsertPrimaryOutcome::Updated { before: account.clone(), after: account })` au lieu de `Unchanged(account)` (la variable est `account`, `bank_accounts.rs:175` ; `existing` est l'`Option`) | rouge (`Unchanged` attendu) |
| 4 | la variante rend `true` en dur au lieu de `rows_affected == 1` | rouge (`(s, false)` attendu au second appel) |
| 5 | variante TVA rendant toujours les quatre taux | rouge |
| 6 | `AND is_stub = TRUE` retiré de `clear_stub_in_tx` | rouge |
| 7 | `FOR UPDATE` retiré de `LOCK_SQL` | rouge (la seconde connexion n'attend plus — elle exécute la même constante, sans verrou elle non plus) |
| 7b | `lock_state_in_tx` lit `SELECT_SQL` au lieu de `LOCK_SQL` (constante intacte) | rouge : pas de 1205 sur la seconde connexion (F-4 de la P3 : distingue « fonction qui n'emploie pas la constante » de « constante modifiée ») |
| 8 | dans `finalize`, le bloc `let _settings = match …insert_with_defaults_in_tx(…) { … };` (`routes/onboarding.rs:720-738`) retiré | rouge : zéro ligne de réglages au lieu d'une. **Mord, prouvé à la lecture** : sur ce parcours, aucun autre site n'écrit `company_invoice_settings` (`grep -n "company_invoice_settings\|get_or_create_default" routes/onboarding.rs auth/bootstrap.rs` : `:721` est le seul appel, `:682` est un commentaire — R3-5 de la P3 ; `upsert_primary` n'y touche pas), et le test 8 appelle désormais `finalize` |

Consigner au Dev Agent Record.

### References

- Fiche index `15-7-trace-onboarding.md` ; fiche sœur `15-7a2-trace-installation-production.md`.
- Rapports P2 de la 15-7a : `target/gate-logs/15-7a-p2-{R,F}.md` (non versionnés) ; prompt versionné
  `15-7a-validate-prompt-p2.md`.
- Passes de cette fiche : prompts versionnés `15-7a1-validate-prompt-p{1,2,3}.md` ; rapports
  `target/gate-logs/15-7a1-p{1,2,3}-{R,F}.md` (non versionnés).
- Choix C-15-7-26 (corrigé par C-15-7-30), C-15-7-30 et C-15-7-33.
- `CLAUDE.md` § Règle de splitting préventif (patron story-zéro + rollout, amendement D5).

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`), agent de développement en autonomie (Epic 15), worktree
`/home/gcorbaz/devel/kesh-15-7a1`, cible cargo propre, bases dédiées `kesh_157a1` / `kesh_e2e_157a1`.

### Debug Log References

Journaux non versionnés sous `target/gate-logs/` : `15-7a1-gate-complet.log`, `15-7a1-e2e.log`,
`15-7a1-e2e-backend.log`, `15-7a1-mut-{1,1b,2,3,4,5,6,7,7b,8}.log`.

### Completion Notes List

- **T0 — alignement sur le livré** : dérives de lignes seulement, aucun AC changé (C-15-7a1-1, Change Log).
- **Compilation** : après le changement des types de retour, `cargo build --workspace --all-targets`
  n'a cassé que les deux appels de test prévus (`company_invoice_settings_repository.rs:1084`,
  `:1174`) ; `:953` et les deux appels de `finalize` (`routes/onboarding.rs:717`, `:741`) ont compilé
  **inchangés** — constaté au compilateur, pas seulement à la lecture. `routes/onboarding` n'est pas
  touché (`git diff 9cb5083b..HEAD -- crates/kesh-api/src` : vide).
- **Tests neufs** (périmètre `9cb5083b..HEAD`, recompté par `grep -c 'sqlx::test'` aux deux bornes) :
  **7** fonctions de test neuves (`accounts_repository` 0 → 2, `bank_accounts_repository` 23 → 24,
  `companies_repository` 18 → 19, `company_invoice_settings_repository` 22 → 23,
  `onboarding_repository` 7 → 8, `vat_rates_repository` 18 → 19) ; le test 8 **prolonge**
  `full_path_b_flow` (7 → 7 fonctions dans `onboarding_path_b_e2e.rs`), sans modifier aucune
  assertion existante.
- **Mutations** — dix, chacune seule, fichier restauré puis touché (`os.utime`), restauration
  vérifiée octet pour octet ; chaque rouge lu à son message :

  | # | rouge observé |
  |---|---|
  | 1 | `aucun commit interne` : 86 au lieu de 0 |
  | 1b | erreur SQL 1064 près de `) ORDER BY number` |
  | 2 | non-compilation `E0308` à l'appel `count_by_company(&mut *tx, …)` (preuve de signature) |
  | 3 | `Unchanged attendu, obtenu Updated` |
  | 4 | `second appel : la ligne existait déjà` |
  | 5 | `second appel : rien d'inséré` (quatre taux rendus) |
  | 6 | second `clear_stub_in_tx` rend `true` |
  | 7 | pas de 1205 : la seconde connexion lit la ligne |
  | 7b | idem (fonction sur `SELECT_SQL`, constante intacte) |
  | 8 | `une ligne de réglages de facturation` : 0 au lieu de 1 |

- **Greps de l'AC 8** (sorties relevées, tri manuel) :
  `grep -nE "MIRROR|intentionally duplicated|Duplication note|count_by_company"` sur les quatre
  fichiers → `company_invoice_settings.rs:15`, `:69`, `:101` (portent sur
  `get_or_create_default{,_in_tx}`, **hors périmètre**, laissés) ; `:527` (le docstring neuf qui dit
  la disparition du MIRROR) ; `accounts.rs:261`, `:267`, `:270` (doc et signature neuves) ; `:411`
  (doc de `retype_impact`, qui cite `journal_entries::count_by_company` — reste vrai). Rollbacks par
  plage : `bulk_create_from_chart` → `:1005` seul (`let _ = tx.rollback().await;`, best-effort) ;
  `bulk_create_from_chart_in_tx` → 0 ; `upsert_primary` → `:195` (`Unchanged`, rollback propagé comme
  avant) et `:203` (best-effort) ; `upsert_primary_in_tx` → 0 ; `insert_with_defaults` → `:571`
  (best-effort) ; `insert_with_defaults_in_tx` → 0. Les sites « pas d'audit » et ceux qui nomment
  `seed_demo` sont laissés à la 15-7a2 et à la 15-7b1, comme l'attribue la fiche index.
- **Gates réellement exécutés, au dernier commit de code `a76a6dbc`** : base `kesh_157a1` remise à
  zéro (DROP/CREATE, migrations, seed) ; `scripts/test-fast.sh` (fmt + clippy `-D warnings` +
  nextest) **vert — 2844 exécutés, 2844 passés, 4 ignorés** ; frontend (non touché) : `npm run
  check` 0 erreur, `lint-i18n-ownership` PASS, `test:unit` 111 fichiers / 1086 tests verts, `build`
  vert ; **E2E complet** (backend `:3009`, base `kesh_e2e_157a1` migrée, secrets générés,
  `KESH_COOKIE_SECURE=false`) : **249 passés, 7 échecs, 17 ignorés** — les sept échecs sont
  exactement les sept KF-029 (#97) de `docs/testing.md` § « Les échecs attendus »
  (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`,
  `:77`, `:150`) ; pas de huitième.
- Ni manuel, ni CHANGELOG : rien de visible ne change (Dev Notes). Choix C-15-7a1-2.

### File List

- `crates/kesh-db/src/repositories/accounts.rs` — `bulk_create_from_chart_in_tx`, enveloppe, `count_by_company` générique
- `crates/kesh-db/src/repositories/bank_accounts.rs` — `UpsertPrimaryOutcome`, `upsert_primary_in_tx`, enveloppe
- `crates/kesh-db/src/repositories/company_invoice_settings.rs` — fin du MIRROR, `(réglages, inséré)`
- `crates/kesh-db/src/repositories/vat_rates.rs` — taux ligne à ligne, `Vec<VatRate>`
- `crates/kesh-db/src/repositories/companies.rs` — `clear_stub_in_tx`
- `crates/kesh-db/src/repositories/onboarding.rs` — `LOCK_SQL`, `lock_state_in_tx`
- `crates/kesh-db/tests/accounts_repository.rs` — **neuf** (tests 1, 2)
- `crates/kesh-db/tests/bank_accounts_repository.rs` — test 3
- `crates/kesh-db/tests/company_invoice_settings_repository.rs` — test 4 ; deux appels adaptés ; docstrings « délégation »
- `crates/kesh-db/tests/vat_rates_repository.rs` — test 5
- `crates/kesh-db/tests/companies_repository.rs` — test 6
- `crates/kesh-db/tests/onboarding_repository.rs` — test 7
- `crates/kesh-api/tests/onboarding_path_b_e2e.rs` — test 8 (`finalize` ajouté à `full_path_b_flow`)
- `_bmad-output/implementation-artifacts/sprint-status.yaml`, `epic-15-choix-autonomes.md`, cette fiche

## Change Log

- 2026-10-08 — Née du découpage de la 15-7a à la passe de validation P2 (findings R2-3 / F-1 : la 15-7a
  franchissait elle-même le seuil de cinq modules ; décision de l'orchestrateur, choix C-15-7-19). Reprend
  la tâche T1 et les parties « repository » des AC 4, 5 et 8 de la 15-7a, remédiées selon les findings
  P2 : R2-2/F-5 (SQL et effet de `clear_stub_in_tx` fixés), R2-6 (`count_by_company` générique, appelants
  réels de `upsert_primary`), R2-7 (primitive de verrou sans `rollback`), R2-8 (montage du test des taux),
  R2-9 (lignes de l'`INSERT IGNORE`), F-6 (appels directs adaptés, tests devenus tautologiques, MIRROR au
  grep), L-4 de la 15-7b (`lock_state_in_tx` dans `kesh-db`). Décompte de modules : **sept**, signal D5
  déclaré. Recompte : **8 AC, 5 tâches, 8 tests**.
- 2026-10-08 — **Passe de validation P1** (prompt versionné `15-7a1-validate-prompt-p1.md` ; deux
  lentilles Sonnet en contexte frais, R auditeur d'acceptation et F full-scope adversary ; rapports
  `target/gate-logs/15-7a1-p1-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **2 MEDIUM, 5 LOW** ; F **1 MEDIUM, 4 LOW**. Après fusion (R1 = F-1 ; R2 absorbe F-2 et F-3 ; R7 =
  F-5) : **0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW distincts**.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | R1 = F-1 | MEDIUM | R, F | test 8 vide (`[] == []`), dans un fichier qui n'emprunte pas le code touché, sans mutation | test 8 remplacé : assertions d'état sur `onboarding_path_b_e2e.rs::full_path_b_flow` (quatre taux, une ligne de réglages, séquence `["fiscal_year.created"]` exactement) + mutation — C-15-7-26 |
  | R2 (+ F-2, F-3) | MEDIUM | R, F | comportement des enveloppes pool hors succès non fixé ; rollback best-effort non dit comme changement | AC 1 : tableau branche par branche (`Err`, `Unchanged`, `OptimisticLockConflict`, court-circuit `entries.is_empty()` avant `begin`) ; **exception assumée** écrite |
  | R3 | LOW | R | `:953` n'a pas à s'adapter | AC 1 : deux appels adaptés, `:953` vérifié inchangé |
  | R4 | LOW | R | références de l'AC 4 | `kesh-seed/src/lib.rs:208`, `:596-600`, `:768` |
  | R5 | LOW | R | doc-comments des fonctions dont le contrat change hors AC 8 | AC 8 : liste des doc-comments à réécrire |
  | R6 | LOW | R | tests 2, 4, 7 sans mutation | tableau des mutations, un par test |
  | R7 = F-5 | LOW | R, F | test 7 : `SET SESSION` sur le pool, montage du singleton | connexion dédiée hors pool, `init_state`, `delete_state` pour `None` |
  | F-4 | LOW | F | colonnes et mode de lecture de `lock_state_in_tx` non fixés | AC 7 : `SELECT_SQL` sans `singleton`, `fetch_optional` |

  **Trouvé pendant la remédiation** : la consigne « `audit_log` à zéro » était fausse sur le code
  actuel — `finalize` écrit déjà `fiscal_year.created` (C-15-7-26). Et R3-3 de la P3 de la 15-7a2 :
  l'AC 7 disait que la 15-7a2 remplace les **trois** copies du verrou d'état, elle en remplace deux
  (`:653`, `:806`), la 15-7b la troisième — corrigé. Propagation : grep des symptômes sur les trois
  fiches, l'index, le registre. Recompte : **8 AC, 5 tâches, 8 tests**.
- 2026-10-08 — **Passe de validation P2** (prompt versionné `15-7a1-validate-prompt-p2.md` ; deux
  lentilles Opus en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7a1-p2-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **1 HIGH, 2 MEDIUM, 5 LOW** ; F **3 MEDIUM, 8 LOW**. Après fusion (R-1 = F-1, R-2 = F-2, R-3 = F-3,
  R-4 = F-4, R-8 = F-6 ; F-11 se partage entre R-5 et R-6) : **0 CRITICAL, 1 HIGH, 2 MEDIUM, 10 LOW
  distincts**.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | R-1 = F-1 | HIGH | R, F | le test 8 de la P1 visait `full_path_b_flow`, qui n'appelle jamais `finalize` : assertions rouges sur le code inchangé, mutation 8 sans prise | test 8 : `POST /onboarding/finalize` **ajouté** à `full_path_b_flow` (200, étape 8) puis assertions ; seul test qui traverse alors les quatre fonctions ; mutation 8 prouvée à la lecture — C-15-7-30 |
  | R-2 = F-2 | MEDIUM | R, F | séquence `["fiscal_year.created"]` fausse : le bootstrap écrit `user.created` (`bootstrap.rs:148`) | séquence exacte `["user.created", "fiscal_year.created"]`, chaque entrée nommée par son écrivain ; propagée à la 15-7a2 (`Points de vigilance`, test 1) — C-15-7-30 |
  | R-3 = F-3 | MEDIUM | R, F | AC 7 : « `SELECT_SQL` suivie de `FOR UPDATE` » ≠ `WHERE singleton = TRUE` | SQL littéral fixé, constante neuve `LOCK_SQL`, `SELECT_SQL` non réutilisée ; test 7 aligné |
  | R-4 = F-4 | LOW | R, F | exception assumée sans `accounts.rs:991` | ajouté à l'inventaire |
  | R-5 (+ F-11) | LOW | R, F | lignes décalées | `accounts.rs:925-934`, `vat_rates.rs:278-284`, `:285`, `:289-303`, `:305-318` |
  | R-6 (+ F-11) | LOW | R, F | `:745` (et `:721`) n'ont rien à adapter | les deux appels de `finalize` compilent inchangés ; `routes/onboarding` n'est plus touché ⇒ **six** modules |
  | R-7 | LOW | R | en-tête sans C-15-7-26 | ajouté, avec C-15-7-30 |
  | R-8 = F-6 | LOW | R, F | mutation 1 non compilable | `sqlx::query("COMMIT")` brut ; mutation 3 écrite en Rust |
  | F-5 | LOW | F | motif D5 inexact (« rollout ») | story-zéro ; quatre extractions, deux ajouts ponctuels |
  | F-7 | LOW | F | montage du test 6 non fixé | `companies::create` puis `UPDATE … is_stub = TRUE` en SQL brut |
  | F-8 | LOW | F | doc-comment orphelin `accounts.rs:258` | AC 8 : la ligne disparaît |
  | F-9 | LOW | F | grep de l'AC 8 bruité (42 lignes) | motif resserré, rollbacks relevés par plage, tri manuel déclaré |
  | F-10 | LOW | F | hors périmètre : `user-manual.tex:780`, `:1772` annoncent 2.5 % et 3.7 % (taux d'avant 2024), le seed pose 2.60 et 3.80 | rien dans la fiche ; **signalé à l'orchestrateur** pour une issue |

  **Signal D5 constaté** : R-1 (HIGH) **recycle** R1 = F-1 de la P1 — un défaut né de la remédiation
  P1 (C-15-7-26), à sévérité supérieure. C'est le signal de découpage sous sa forme « recyclage ». **Suivi
  ainsi** : déclaré au Project Lead ; l'orchestrateur a fait corriger le test 8 sans découper — le
  défaut recyclé tient dans le texte d'un seul test, et la fiche est déjà la story-zéro d'un découpage
  (C-15-7-19), sans axe de coupe restant (C-15-7-30). La remédiation de cette passe touche des tests et
  du texte de spec, aucune ligne de code de production n'étant encore écrite. Propagation : grep des
  symptômes (`fiscal_year.created"]`, `SELECT_SQL`, `adaptation mécanique`, `sept modules`, `:922`,
  `:277`, `:285-293`, `:308`) sur les trois fiches, l'index et le registre ; C-15-7-26 corrigé par une
  entrée neuve (C-15-7-30), non réécrit. Recompte : **8 AC, 5 tâches, 8 tests, 8 mutations**.
- 2026-10-08 — **Passe de validation P3** (prompt versionné `15-7a1-validate-prompt-p3.md` ; deux
  lentilles Sonnet en contexte frais, R regression hunter et F full-scope adversary ; rapports
  `target/gate-logs/15-7a1-p3-{R,F}.md`, non versionnés). Bruts, recomptés depuis les rapports : R
  **1 MEDIUM, 6 LOW** ; F **1 MEDIUM, 4 LOW**. Après fusion (R3-1 = F-1 pour la graphie du squash, R3-2
  absorbé par F-1 pour le fichier des tests 1-2, R3-3 = F-4) : **0 CRITICAL, 0 HIGH, 1 MEDIUM, 8 LOW
  distincts**.

  | finding | sévérité | lentilles | objet | sort |
  |---|---|---|---|---|
  | R3-1 = F-1 (+ R3-2) | MEDIUM | R, F | `migrations = "test-schema"` refusé par `test_schema_guard.rs:62` ; tests 1-2 sans fichier | graphies `"./test-schema"` / `"../kesh-db/test-schema"` ; tableau d'un fichier par test ; `crates/kesh-db/tests/accounts_repository.rs` à créer — C-15-7-33 |
  | R3-3 = F-4 | LOW | R, F | visibilité de `LOCK_SQL` non dite ; mutation 7 ne distingue pas fonction et constante | `pub const LOCK_SQL` ; mutation 7b (`SELECT_SQL` lu au lieu de `LOCK_SQL`) — C-15-7-33 |
  | R3-4 | LOW | R | mutation 1 non compilable (`?` sans `From<sqlx::Error>`) | `.await.map_err(map_db_error)?` |
  | R3-5 | LOW | R | sortie du grep de la mutation 8 inexacte | `:721` seul appel, `:682` commentaire |
  | R3-6 | LOW | R | court-circuit `entries.is_empty()` sans test | appel à vide au test 1, mutation 1b |
  | R3-7 | LOW | R | références : `bootstrap.rs` `:83`/`:148`, test `:219`, `:743-763`, variable `account` | corrigées |
  | F-2 | LOW | F | condition de validité de la garde `existing == 0` (instantané REPEATABLE READ) | AC 7 et Points de vigilance : verrou d'état **en premier**, doc-comments de `lock_state_in_tx` et `count_by_company` — C-15-7-33 |
  | F-3 | LOW | F | `clear_stub_in_tx` borné et bumpant ≠ site `:211` qu'il remplacera | AC 6 : changement assumé, porté par la 15-7b1 (écrit dans son AC 1) |
  | F-5 | LOW | F | mutation 2 de compilation, pas de comportement | assertion croisée pool / transaction au test 2 ; la mutation 2 dite « de signature » |

  **Validation close après cette passe.** La remédiation ne touche que la fiche (graphie d'attribut,
  fichiers de test, visibilité d'une constante, texte des mutations, doc-comments à écrire) : aucun
  changement de conception, aucune ligne de code de production n'étant écrite. Les deux MEDIUM de la
  passe étaient le même défaut, distinct de ceux des passes précédentes et non né de la remédiation P2
  (la graphie `"test-schema"` remonte à la création de la fiche) : pas de recyclage, pas de signal D5
  neuf. Une lecture ciblée de cette remédiation est attendue à 0 au-dessus de LOW. Propagation : grep de
  `test-schema"`, `.await?;`, `LOCK_SQL`, `:744`, `existing` sur les quatre fiches et l'index ; la
  sémantique de `clear_stub_in_tx` propagée à la 15-7b1. Trend : P1 2 MEDIUM → P2 1 HIGH, 2 MEDIUM →
  P3 1 MEDIUM. Recompte : **8 AC, 5 tâches, 8 tests, 10 mutations** (1, 1b, 2 à 7, 7b, 8).
- 2026-10-08 — **Passe de validation P4 ciblée — clôture** (prompt versionné
  `15-7a1-validate-prompt-p4-ciblee.md` ; une lentille Haiku en contexte frais, réservée à la passe
  ciblée — décision D6 ; braquée sur le seul commit de la remédiation P3, `327ea9df`, fiche seulement ;
  rapport `target/gate-logs/15-7a1-p4-ciblee.md`, non versionné). Bilan : **0 CRITICAL, 0 HIGH,
  0 MEDIUM, 0 LOW**. Axes déclarés exercés : cohérence des hunks, renvois de ligne revérifiés au code,
  graphies d'attribut contre `test_schema_guard.rs:62`, mutations 1b et 7b, recomptes ; non exercés :
  compilation, exécution, audit, i18n, manuels (hors du périmètre d'une passe ciblée sur une
  remédiation de fiche). **Vérification de l'orchestrateur** (le « 0 » se vérifie comme un finding) :
  les trois occurrences de graphie d'attribut de la fiche (`"./test-schema"` et
  `"../kesh-db/test-schema"` aux Tests, `"test-schema"` citée comme forme refusée) sont conformes à
  `SQUASH_SPELLINGS` (`crates/kesh-db/tests/test_schema_guard.rs:62`, sortie de `grep -n` relue). La
  remédiation relue ne touchait aucune ligne de code de production : **validation close**.
  **Trend** : P1 2 MEDIUM → P2 1 HIGH, 2 MEDIUM → P3 1 MEDIUM → P4 ciblée 0. **Modèles** : deux lentilles
  (R, F) en contexte frais, Sonnet en P1, Opus en P2, Sonnet en P3 ; une lentille Haiku en P4 ciblée. **Reclassements** : aucun ;
  signal D5 de la P2 (recyclage R-1) déclaré et suivi sans découpage (C-15-7-30). Recompte inchangé :
  **8 AC, 5 tâches, 8 tests, 10 mutations**.
- 2026-10-08 — **Alignement sur le livré (T0 du développement)**, sur `HEAD` `0f6dfabb` (= `origin/main`
  `9cb5083b` + report de la planification). Chaque référence relocalisée par le texte. **Aucun écart ne
  change une règle ni un AC** — choix C-15-7a1-1. Dérives de lignes (fiche → `HEAD`) : `accounts.rs`
  `:925-934` → `:967-976`, `:941-943` → `:983-985`, `:991` → `:1033` ; `routes/onboarding.rs` `:653` →
  `:649`, `:806` → `:802`, `:721` → `:717`, `:720-738` → `:716-734`, `:745` → `:741`, `:682` → `:678` ;
  `fiscal_years.rs:314` → `:320` ; `journal_entries.rs:495` → `:527` ;
  `MULTI-TENANT-SCOPING-PATTERNS.md:322` → `:298`/`:317`. Fait nouveau sans effet ici : `finalize` est
  enveloppé par `retry_app_on_deadlock("onboarding::finalize", …)` (15-5e2) — la route n'est pas touchée
  par cette fiche.
- 2026-10-08/09 — **Développement** (`bmad-dev-story`, commit `a76a6dbc`) : T1 à T5 faits ; 7 tests neufs
  + le test 8 prolongeant `full_path_b_flow` ; 10 mutations, toutes rouges pour la raison attendue ;
  gate complet vert (2844/2844, 4 ignorés) et E2E complet aux seuls 7 échecs KF-029 attendus, au dernier
  commit de code. Choix C-15-7a1-2. Statut → `review`.
