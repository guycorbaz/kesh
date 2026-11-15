# Story 38-4 : critères neufs des règles d'affectation (boutique = préfixe du numéro de facture) et filtres sens, devise, compte

Status: ready-for-dev

*Fiche rédigée le 2026-11-15 (Opus), branche `epic/38-paypal`, numéros de ligne relevés sur `8b3d548d`. Epic 38
(`_bmad-output/planning-artifacts/epic-38-paypal.md`) : E4, E14, E18, E19, C-38-0, C-38-6, C-38-12 ; décisions de Beat
B5, B19, B22, B24 (`epics-38-40-cadrage.md`) ; arbitrages O1, O2, O3, O4 de l'epic. **Dépend de 38-2** (O1 : **la**
migration breaking de l'epic, qui passe `reconciliation_rules.match_type` en `VARCHAR(40)` `ascii_bin` avec `CHECK` de
format et relève `kesh_version_min_required`) **et de 38-3a** (C-38-4 : colonnes `counterparty_email`, `invoice_ref`,
`item_names`, `event_code` de `bank_transactions`, accesseur `item_names()`) et de l'**Epic 37** (37-1 : `accounts.currency`, liste ISO 4217 de `kesh-core`,
`CURRENCY_UNKNOWN`, `MULTI_CURRENCY_DISABLED` 409, verrou des deux côtés C-37-9, liste close des absences admises de
37-1 AC-6). **Migration non breaking** (DDL seul). Aucune route neuve.*

⚠️ **Repris de l'ancienne 38-3 (D-P1-1, liste inter-fiches, point 1)** : le décodage tolérant arrive avec la première
écriture d'une valeur neuve ; pour les critères, c'est **cette story**. Elle porte donc ce que 38-3 « déclarait » :
`ReconciliationMatchType` gagne ses quatre variantes neuves et `Other(String)` (`FromStr` tolérant,
`Serialize`/`Deserialize` à la main, perte de `Copy` — relevé au commit : 53 occurrences hors du fichier, dans
`rules.rs`, `routes/reconciliation_rules.rs` et un fichier de tests ; `match rule.match_type` devient `match
&rule.match_type`, sans `match` qui refuse l'inconnu) et le code `RULE_CRITERION_UNKNOWN` (AC-7, quatre locales) ;
`errors.rs:1150-1159, 3304-3316` relus et adaptés à `Other`. Il n'y a plus de « refus provisoire » des quatre types
neufs à lever. 38-3a ne touche que `BankImportSourceFormat`.

## Story

En tant que comptable d'une société qui vend par plusieurs boutiques en ligne et en plusieurs devises, je veux qu'une
règle d'affectation reconnaisse une transaction par le courriel du payeur ou du marchand, par la **boutique** (préfixe
du numéro de facture), par le nom d'un article ou par le code d'événement du prestataire, et qu'elle se restreigne au
sens, à la devise et au compte bancaire de la transaction, afin que chaque vente soit proposée au sous-compte de ventes
de sa boutique et de sa devise — sans que les règles que j'ai déjà changent de comportement.

## Exigences

Reprises de la fiche d'epic (§ 38-4), complétées là où un AC l'exige.

- **FR**
  1. `FR-38-4-1` — Quatre types neufs de critère : `counterparty_email_exact`, `invoice_prefix` (boutique),
     `item_name_contains`, `event_code_prefix`. → AC-2, AC-3, AC-4.
  2. `FR-38-4-2` — Trois filtres facultatifs, applicables à tout critère : sens, devise, compte bancaire. → AC-1,
     AC-3, AC-4.
  3. `FR-38-4-3` — Une règle par boutique et par devise est admise ; deux règles de mêmes type, valeur et filtres sont
     refusées (`RECONCILIATION_RULE_DUPLICATE`). → AC-1, AC-5.
  4. `FR-38-4-4` — Un compte de règle en devise ne s'applique qu'aux transactions de cette devise. → AC-6.
  5. `FR-38-4-5` — Une règle d'un type inconnu ne s'applique jamais et ne se modifie pas (`RULE_CRITERION_UNKNOWN`).
     → AC-7.
  6. `FR-38-4-6` *(ajoutée par la fiche)* — Une règle peut désigner un compte de contrepartie **en devise** (levée,
     pour les seules règles, du refus `ACCOUNT_IN_FOREIGN_CURRENCY` posé par 37-1). → AC-6.
- **NFR**
  1. `NFR-38-4-1` — Non-régression des quatre critères existants et de leur priorité. → AC-8.
  2. `NFR-38-4-2` — Normalisation unique (une fonction) pour l'écriture et l'évaluation. → AC-2.
  3. `NFR-38-4-3` — Migration non breaking. → AC-1.

## Écarts à l'epic

1. **Valeur inconnue de `matchType` aujourd'hui : 422, non 400.** E14 écrit « aujourd'hui par la désérialisation de
   l'enum en 400 ». Les routes de règles prennent le corps par `axum::Json` (`routes/reconciliation_rules.rs:26`,
   `:262`), dont le rejet de données est le `422` natif d'Axum 0.8 (`crates/kesh-api/src/extractors.rs:11-13` le dit ;
   seules les routes qui passent par `impl_validation_extractor` rendent 400). Avec cette story (décodage repris de
   l'ancienne 38-3), `matchType` se décode en `Other(String)` : une **chaîne** inconnue n'est plus un rejet de données et cette fiche la refuse en **400
   `RULE_CRITERION_UNKNOWN`** (AC-7) ; un corps mal formé (type JSON faux, champ requis absent) reste en 422, comme
   aujourd'hui — l'extracteur des règles n'est pas changé (hors périmètre, aucun motif).
2. **Le compte d'une règle en devise contredit 37-1.** 37-1 AC-4 refuse en 400 `ACCOUNT_IN_FOREIGN_CURRENCY` le choix
   d'un compte en devise comme contrepartie d'une règle de réconciliation, et en 409 `ACCOUNT_CURRENCY_IN_USE` la devise
   donnée à un compte qu'une règle désigne (C-37-9, « verrou des deux côtés »). E14 admet un compte de règle en devise
   sans dire qu'il lève ce refus. Cette fiche lève **le sens « désigner »** pour les seules règles (FR-38-4-6, AC-6) et
   **garde le sens « recevoir une devise »** : un compte que désigne une règle ne change pas de devise — la règle
   changerait sinon en silence de transactions visées. Choix à confirmer en validation.
3. **Tolérance étendue aux filtres.** E4 range « sens » parmi les colonnes lues avec `Other(String)`. La colonne
   `direction` ne porte donc **aucun `CHECK` d'énumération** (format seul, précédent A28 de 37-2) ; une valeur inconnue
   lue rend la règle inerte et non modifiable, comme un type inconnu (AC-7).
4. **Le code par proposition `RULE_ACCOUNT_CURRENCY_MISMATCH`** est rangé par C-38-8 sous 38-6a ; E14 le donne aux
   règles « par proposition ». Cette fiche le pose sur le seul chemin d'acceptation existant (`accept_one_rule`, CHF) ;
   38-6a le reprend pour la variante `provider`.

## Critères d'acceptation

1. **AC-1 — Migration** `<AAAAMMJJ>000001_reconciliation_rule_filters.sql` (version strictement postérieure à la
   migration breaking de 38-2, à la migration non breaking de 38-3a et à toute migration de la branche au moment de la
   fusion), **non breaking**, DDL seul,
   une seule instruction `ALTER TABLE reconciliation_rules` :
   - `ADD COLUMN direction VARCHAR(16) CHARACTER SET ascii COLLATE ascii_bin NULL` + `CONSTRAINT
     chk_reconciliation_rules_direction CHECK (direction IS NULL OR BINARY direction REGEXP '^[a-z_]{1,16}$')` —
     format seul (Écart 3) ; valeurs écrites par l'API : `in`, `out` ;
   - `ADD COLUMN currency CHAR(3) CHARACTER SET ascii COLLATE ascii_bin NULL` + `CONSTRAINT
     chk_reconciliation_rules_currency CHECK (currency IS NULL OR BINARY currency REGEXP '^[A-Z]{3}$')` — même
     définition qu'`accounts.currency` (37-1 AC-1), CHF **admis** (filtre « transactions en CHF ») ;
   - `ADD COLUMN bank_account_id BIGINT NULL` + `CONSTRAINT fk_reconciliation_rules_bank_account FOREIGN KEY
     (bank_account_id) REFERENCES bank_accounts(id) ON DELETE RESTRICT` (un compte bancaire ne se supprime jamais,
     il s'archive : aucun `DELETE FROM bank_accounts` hors tests) ;
   - `ADD COLUMN filter_key VARCHAR(48) CHARACTER SET ascii COLLATE ascii_bin GENERATED ALWAYS AS
     (CONCAT(COALESCE(direction, '-'), '|', COALESCE(currency, '-'), '|', COALESCE(CAST(bank_account_id AS CHAR),
     '-'))) VIRTUAL` — jamais `NULL` par construction (contrairement à `active_uniq`, il n'exclut rien de l'unicité) ;
   - `DROP INDEX uq_reconciliation_rules_match_active, ADD CONSTRAINT uq_reconciliation_rules_match_active UNIQUE
     (company_id, match_type, active_uniq, filter_key)` — **même nom**, pour que
     `is_duplicate_rule_constraint` (`repositories/reconciliation_rules.rs:37-43`, test sur le nom) reste vrai ; index
     d'origine `20260513000001_reconciliation_rules.sql:65-66` sur `active_uniq` (`:54`, `IF(active, match_value,
     NULL)`, `VIRTUAL` : précédent d'un index unique sur colonne virtuelle) — **reconstruit une première fois par
     l'`ALTER` de 38-2** (`MODIFY COLUMN match_type`), le nom et les colonnes inchangés (test de 38-2) ; longueur de clé à vérifier au développement (BIGINT + `match_type` `ascii` de 38-2 + 255 × 4 + 48
     octets, sous la limite InnoDB de 3072). `filter_key` dépend de `bank_account_id`, colonne de base d'une colonne
     générée, et MariaDB restreint les actions référentielles sur une telle colonne : seule `RESTRICT` (écrite ici)
     est sûre — un test de migration pose la FK **et** la colonne générée dans l'ordre écrit sur la version de
     MariaDB du dépôt, et vérifie la valeur de `filter_key` à l'octet (`'in|EUR|12'`, charset `ascii` de la colonne,
     `CAST(… AS CHAR)` compris).
   **Non breaking** (P1) : ajouts de colonnes `NULL`, d'une colonne générée et remplacement d'un index unique par un
   index plus large. `DROP INDEX` n'est pas dans la liste P3 ; un binaire antérieur ne démarre de toute façon
   plus sur une base passée par la migration breaking de 38-2 (C-38-0, O1) — aucun relèvement ici. **Aucun `UPDATE`** : P7 sans objet (le
   détecteur classe sur le premier mot-clé, `ALTER`) — vérifié par `every_data_backfill_migration_is_triaged`.
   **P5** : une ligne à `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx` : DDL non rejouable — `ADD
   COLUMN` sans `IF NOT EXISTS`), en-tête `## Table d'audit (N migrations)` et ligne `Total` recomptés à la source
   (`ls crates/kesh-db/migrations/*.sql | wc -l` et `grep -c '^| \`20' docs/migrations-idempotence-audit.md`),
   compteurs de partition recomptés depuis le tableau. **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to"
   crates/` exécuté, chaque site inspecté, résultat au Change Log. **P8** : aucune migration appliquée retouchée.
   Squash de test régénéré (`scripts/regen-test-schema.sh`, `test_schema_guard.rs` vert). Gate **complet** même en
   cours de boucle (exception `kesh-db` de *Test Locally First*).
2. **AC-2 — Normalisation unique** (`NFR-38-4-2`). Une fonction pure `kesh_reconciliation::rules::match_key(match_type:
   &ReconciliationMatchType, value: &str) -> String`, **seule** définition de la forme comparée, appelée à
   l'écriture (route) **et** à l'évaluation (des deux côtés : valeur de la règle et champ de la transaction) :

   | Type | `match_key` | Stocké à l'écriture |
   |---|---|---|
   | `counterparty_contains`, `counterparty_exact`, `reference_contains`, `item_name_contains` | `trim` + minuscules (la fonction `normalize` actuelle, `rules.rs:102`) | tel que saisi (inchangé pour les trois existants, NFR-38-4-1) ; `item_name_contains` : tel que saisi |
   | `iban_exact` | espaces retirés + majuscules (`normalize_iban`, `rules.rs:112`) | `match_key` (comme aujourd'hui) |
   | `counterparty_email_exact`, `invoice_prefix` | **tout** blanc retiré + minuscules | `match_key` |
   | `event_code_prefix` | tout blanc retiré + majuscules | `match_key` |
   | `Other(_)` | identité (jamais évaluée, AC-7) | — (refusée, AC-7) |

   Propriété testée : `match_key(t, &match_key(t, v)) == match_key(t, v)` pour tout type et un jeu de valeurs (idempotence,
   ce qui rend sûr le stockage de la forme normalisée). La copie `normalize_iban_canonical` de la route
   (`routes/reconciliation_rules.rs:162-167`, doublon de `rules.rs:112-117`) est **supprimée** au profit de
   `match_key` (DRY ; grep du symptôme `to_uppercase()` dans les deux fichiers, au Change Log). `normalize` et
   `normalize_iban` deviennent privées derrière `match_key`. Validation de forme à l'écriture : `counterparty_email_exact`
   — après `match_key`, exactement un `@`, au moins un caractère de part et d'autre, sinon 400 `VALIDATION_ERROR`
   (« courriel invalide ») ; tout type — `match_key` non vide (le contrôle `trim` actuel,
   `validate_match_value`, `routes/reconciliation_rules.rs:181-191`, s'applique à la valeur saisie, puis le
   non-vide se recontrôle sur la forme normalisée). Aucun contrôle de forme sur `event_code_prefix` ni
   `invoice_prefix` (les Epics 39 et 40 y mettront d'autres codes).
3. **AC-3 — Évaluation** (`rule_matches`, `kesh-reconciliation/src/rules.rs:46-70`) : le critère, **puis** les
   filtres ; tous doivent être vrais.
   - `counterparty_email_exact` : `tx.counterparty_email` (colonne de 38-3a) présent et `match_key` égal ;
   - `invoice_prefix` : `tx.invoice_ref` (38-3a ; PayPal : `invoice_id`, B24) présent et
     `match_key(invoice_ref).starts_with(match_key(valeur))` ;
   - `item_name_contains` : au moins un nom de `tx.item_names` (38-3a) dont le `match_key` contient celui de la
     valeur ; chaque nom est comparé **séparément** (jamais la liste jointe : une valeur ne doit pas « contenir » la
     fin d'un nom et le début du suivant) — par l'accesseur que 38-3a fournit pour lire la liste, quelle que soit sa
     forme stockée ;
   - `event_code_prefix` : `tx.event_code` (38-3a) présent et `match_key(event_code).starts_with(match_key(valeur))` ;
   - les quatre existants : inchangés (NFR-38-4-1) ;
   - `Other(_)` : **faux**, toujours (AC-7) ;
   - filtre `direction` : `in` ⇔ `tx.amount > 0`, `out` ⇔ `tx.amount < 0` (montant nul : aucun des deux ; E14 « signe
     de `amount` » — pour une transaction de prestataire, `amount` est le net, E7) ; `Other(_)` : faux ;
   - filtre `currency` : `tx.currency == currency` (octets ; les deux sont en majuscules ASCII) ;
   - filtre `bank_account_id` : `tx.bank_account_id == bank_account_id`.
   Type d'évaluation : `RuleDirection { In, Out, Other(String) }` dans `kesh-db::entities::reconciliation_rule`
   (décodage tolérant, E4 ; `as_str` rend la valeur brute de `Other`). Tests unitaires (`rules.rs`, sans base) : chaque
   type neuf vrai et faux, champ absent (`None`) faux, préfixe plus long que la valeur faux, casse et blancs ignorés,
   `item_names` « à cheval » sur deux noms faux, chaque filtre seul vrai et faux, combinaison critère + trois filtres,
   montant nul avec `direction`, `Other` dans le type et dans le sens.
4. **AC-4 — Routes** (`routes/reconciliation_rules.rs` ; formes inchangées, rôles inchangés — écriture Comptable+,
   lecture tout rôle, clés d'API comme aujourd'hui) :
   - `ReconciliationRuleResponse` (`:58-75`) gagne `direction: Option<String>`, `currency: Option<String>`,
     `bankAccountId: Option<i64>`, **toujours présents** (`null` si absent : ce ne sont pas des montants en devise,
     la règle C-37-9 de JSON identique en mono-devise ne les vise pas) ; `matchType` d'une règle `Other` est sa chaîne
     brute ;
   - `CreateRuleRequest` (`:97-109`, `deny_unknown_fields`) gagne `direction`, `currency`, `bankAccountId`, facultatifs
     (absent = `null`) ; `matchType` admet les **huit** valeurs connues (le type se décode en `Other(String)`
     pour une neuvième, en-tête — AC-7) ;
   - `UpdateRuleRequest` (`:115-133`) gagne les trois filtres en `Option<Option<…>>` par
     `crate::helpers::double_option` (patron de `default_project_id`, `:131-132`) : absent = inchangé, `null` =
     retiré, valeur = posée. `matchType` reste **non modifiable** (absent de la struct, `deny_unknown_fields` : si un client
     l'envoie, le rejet de désérialisation d'`axum::Json` rend **422** (`routes/reconciliation_rules.rs:368`, sans
     conversion en 400, cf. Écart 1) — comportement actuel, gardé : changer le type briserait la lecture des
     traces `reconciliation_rule.applied`, `entities/reconciliation_rule.rs:150-152`) ;
   - validation, **avant** la transaction, dans cet ordre : forme existante (libellé, valeur, priorité) → `matchType`
     inconnu (AC-7) → forme du courriel (AC-2) → `direction` ∉ {`in`, `out`} → 400 `VALIDATION_ERROR` →
     `currency` : `trim` + majuscules ASCII, hors liste ISO 4217 de `kesh-core` (37-1) → 400 `CURRENCY_UNKNOWN` →
     compte bancaire : `bankAccountId ≤ 0` → 400, absent de la société → 404 `BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`
     (`AppError::BankAccountNotFound`, `errors.rs:770`, `:2553`), archivé **et changé** → même 404 (un filtre
     inchangé sur un compte archivé depuis ne bloque pas un `PATCH` sans rapport, patron « validé seulement s'il
     change » de `company_invoice_settings.rs:240-250`) → compte de contrepartie (`validate_counterparty_account`,
     `:205-228`) ;
   - dans la transaction : `currency` ≠ `CHF` alors que la société est en mono-devise → **409
     `MULTI_CURRENCY_DISABLED`** (37-1 : 409 partout ; lu dans la transaction, comme le réglage de 37-1) ; puis la
     garde de devise du compte (AC-6) ; puis les gardes existantes du dépôt (projet, compte imputable,
     `create_in_tx` `repositories/reconciliation_rules.rs:183-204`) ;
   - **audit** : `reconciliation_rule.created` porte `direction`, `currency`, `bank_account_id` ; `.updated` les porte
     dans `before` et `after` (`:390-405`), avec — correction au passage — `counterparty_account_id`, aujourd'hui
     absent du diff (une modification du seul compte laisse `before == after` : à écrire au Change Log comme défaut
     voisin traité). Aucune action neuve à `audit_labels.rs` (les quatre `reconciliation_rule.*` existent, `:192-195`) ;
   - `NewReconciliationRule` et `UpdateReconciliationRule` (`entities/reconciliation_rule.rs:136-169`) gagnent les
     champs ; `COLUMNS` du dépôt (`repositories/reconciliation_rules.rs:27-28`) les lit (jamais `filter_key`, comme
     `active_uniq`) ; l'entité `ReconciliationRule` gagne `direction: Option<RuleDirection>`, `currency:
     Option<String>`, `bank_account_id: Option<i64>`.
   Tests HTTP (`kesh-api/tests/reconciliation_rules_e2e.rs`) : création de chaque type neuf (201, valeur stockée
   normalisée), chaque refus ci-dessus avec son code et son statut, `PATCH` d'un filtre (posé, retiré par `null`,
   inchangé si absent), `PATCH` portant `matchType` → 422 (comportement actuel), filtre sur un compte bancaire d'une autre société → 404,
   devise non CHF en mono-devise → 409.
5. **AC-5 — Unicité** (`FR-38-4-3`). Deux règles **actives** de mêmes société, type, `match_value` stockée et filtres
   (`filter_key`) → 409 `RECONCILIATION_RULE_DUPLICATE` (création, `PATCH`, réactivation) ; les `details` actuels
   (`matchType`, `matchValue`, `errors.rs:3303-3320`) gagnent `direction`, `currency`, `bankAccountId` (lus de la
   requête résolue — `map_create_or_update_db_err`, `routes/reconciliation_rules.rs:233-246`, reçoit la règle
   résolue, non plus le seul couple) ; message `reconciliation-rules-error-duplicate` revu aux quatre locales («… pour
   cette combinaison type, valeur et filtres »). Tests : une règle `invoice_prefix` `LG` en EUR **et** une en USD →
   les deux 201 (B19) ; deux `LG` en EUR → 409 ; une `LG` sans filtre et une `LG` en EUR → 201 (filtres distincts,
   la priorité les départage, AC-8) ; `LG` et `lg` → 409 (forme stockée identique, AC-2) ; une règle archivée ne
   compte pas (comportement actuel de `active_uniq`, test existant `rule_create_succeeds_when_existing_rule_is_inactive`
   gardé vert).
6. **AC-6 — Compte de règle en devise** (`FR-38-4-4`, `FR-38-4-6`).
   - **Écriture** : une règle peut désigner un compte de contrepartie en devise : la garde
     `ACCOUNT_IN_FOREIGN_CURRENCY` que 37-1 AC-4 pose sur `reconciliation_rules.counterparty_account_id` est **retirée
     pour cette colonne seule** (création, changement de compte, réactivation) ; le refus inverse
     `ACCOUNT_CURRENCY_IN_USE` (donner une devise à un compte qu'une règle désigne, archivée comprise) **reste**
     (Écart 2). L'inventaire clos des sites de 37-1 (dix-neuf FK `REFERENCES accounts`) est relu au Change Log : une
     seule colonne change de traitement. Une règle dont le **filtre** `currency` contredit la devise de son compte
     (compte en EUR, filtre USD) → 400 `RULE_ACCOUNT_CURRENCY_MISMATCH` (code neuf, `details: { accountId,
     accountCurrency, filterCurrency }`, quatre locales) : elle ne s'appliquerait jamais ; un compte CHF admet tout
     filtre.
   - **Proposition** : une fonction pure `rule_account_admits(account_currency: Option<&str>, tx_currency: &str) ->
     bool` (`kesh-reconciliation::rules`) — compte en devise X : vrai ssi la transaction est en X ; compte CHF
     (`None`) : vrai pour toute devise (l'applicabilité d'une contrepartie CHF à une transaction en devise relève de
     38-6a). `first_matching_rule` (`rules.rs:85-95`) reçoit, à la place de `active_account_ids: &HashSet<i64>`, une
     carte `eligible_accounts: &HashMap<i64, Option<String>>` (compte → devise ; mêmes comptes : actifs, imputables,
     hors compte de la banque — l'ensemble que construit `get_proposals`, `routes/reconciliation.rs:1230-1258`) et
     saute une règle dont le compte n'admet pas la transaction ; **la règle suivante** est alors proposée (même
     comportement qu'une règle à compte archivé, test `first_matching_rule_skips_inactive_account` comme patron).
     La requête de `get_proposals` (`:1230-1236`) lit `currency` (37-1).
   - **Acceptation** (`accept_one_rule`, `routes/reconciliation.rs:3946`) : à l'étape 5 (`:4068`), la lecture du
     compte de contrepartie lit aussi sa devise ; après l'étape 6 (transaction relue sous verrou), si
     `!rule_account_admits(…)` → `FailedProposal` `RULE_ACCOUNT_CURRENCY_MISMATCH`, `details: { ruleId, accountId,
     accountCurrency, transactionCurrency }`, **avant** le refus hors CHF (étape 6.5, `:4142`) et l'étape 7
     (`rule_matches`, `:4154`) ; HTTP 200 (*Pattern batch*). Le refus `RECONCILIATION_CURRENCY_MISMATCH` des transactions hors CHF (`:4142-4151`) reste en place
     (levé par 38-6a pour la seule variante `provider`).
   Tests : unitaires de `rule_account_admits` et de `first_matching_rule` (compte EUR sauté pour une transaction CHF,
   règle suivante prise) ; HTTP : règle sur compte EUR créée (201, société multi-devises), même règle en mono-devise →
   impossible par construction (37-1 : pas de compte en devise) ; proposition CHF qui saute la règle EUR ; acceptation
   rejouée d'une règle dont le compte a reçu une devise entre-temps — inatteignable (refus inverse gardé) : le test
   pose la devise par SQL et vérifie `failed[0].errorCode == "RULE_ACCOUNT_CURRENCY_MISMATCH"`.
7. **AC-7 — Type ou sens inconnus** (`FR-38-4-5`, E4). `RULE_CRITERION_UNKNOWN` (code **posé par cette story**, repris de l'ancienne
   38-3 — 400 à l'écriture d'un `Other`, 409 à la modification d'une règle `Other` ; en-tête) ; il vaut aussi pour le
   **sens** **pour la seule lecture d'une
   règle stockée** : le 409 d'une modification porte `details: { direction }` à côté de `{ matchType }` — une valeur de
   sens inconnue **écrite** par l'API reste un 400 `VALIDATION_ERROR` (ci-dessous) :
   - **400** à l'écriture d'une valeur inconnue (création ; un `PATCH` ne porte pas de type) — `matchType` décodé en
     `Other(_)`, ou `direction` ∉ {`in`, `out`} rend, lui, 400 `VALIDATION_ERROR` (AC-4 : une valeur de sens n'est pas
     un critère) ;
   - **409** à toute modification (`PATCH`, quels qu'en soient les champs, réactivation comprise) d'une règle stockée
     dont le type **ou** le sens est `Other` ; la **désactivation** (`DELETE`, `routes/reconciliation_rules.rs:444`)
     reste admise : elle ne change pas le critère, et retire de la liste une règle que ce binaire ne sait pas lire
     (choix, au Change Log) ;
   - jamais appliquée : `rule_matches` faux (AC-3) ; `accept_one_rule` la rend
     `RECONCILIATION_RULE_NO_LONGER_MATCHES` (étape 7, code existant) — aucune écriture fausse ;
   - lue sans erreur : `GET` liste et détail, export CSV (valeur brute).
   Test : une ligne posée par SQL avec `match_type = 'future_criterion'` (possible depuis le `VARCHAR` de 38-2) et une
   avec `direction = 'both'` : `GET` 200, `PATCH` 409, `DELETE` 204, proposition sans cette règle (la suivante
   s'applique).
8. **AC-8 — Non-régression** (`NFR-38-4-1`). Tous les tests existants de `rules.rs` (8 tests, `:216-507`) et de
   `reconciliation_rules_e2e.rs` restent verts **sans modification d'assertion** (seuls les littéraux de construction
   gagnent leurs champs — l'échec de compilation inventorie les sites : `ReconciliationRule {` 1 hors entité (`rules.rs:191`),
   `NewReconciliationRule {` 5 et `UpdateReconciliationRule {` 12 hors déclaration, recomptés au développement). Priorité inchangée :
   `priority ASC, id ASC` (`find_active_for_company`, `repositories/reconciliation_rules.rs:73-90`), y compris entre
   une règle large et une étroite — une règle `LG` sans filtre de priorité 10 passe devant une `LGT` de priorité 20 :
   test dédié (préfixe contenu dans un autre, risque de l'epic), et le fait est transmis à 38-9b pour le manuel.
9. **AC-9 — Sauvegarde, export, remise à zéro.**
   - `.keshbackup` : les trois colonnes passent seules (`export_table` lit `information_schema`, colonnes générées
     exclues, `kesh-db/src/backup.rs:179-190`) ; `filter_key` est générée, donc absente du manifeste ; aller-retour
     d'une règle à trois filtres testé (`admin_full_import_e2e.rs`).
   - **Liste close des absences admises** (37-1 AC-6, E18) : `Absence::Colonne("reconciliation_rules", "direction")`,
     `…"currency"`, `…"bank_account_id"`, avec la version de la migration de l'AC-1 ; le test de la liste (colonne au
     schéma, non `is_required()`, ajoutée par une migration de l'epic) les couvre ; le test d'import d'une sauvegarde
     antérieure (patron 0.17.6 de 37-1) retire les trois colonnes → 200, règles sans filtre.
   - Ordre de `TABLES_TO_TRUNCATE` (`backup.rs:46`) : `reconciliation_rules` (`:103`) est déjà **avant**
     `bank_accounts` (`:104`) — la FK neuve est satisfaite au vidage (enfant d'abord) et à la restauration (ordre
     inverse) ; à vérifier par `backup_inventory_matches_schema` et `reset_partition_is_guarded_by_the_schema`, sans
     changement de liste attendu.
   - **Export CSV** (`serialize_reconciliation_rules_csv`, `exports/csv_tables.rs:1015-1059`) : colonnes
     `direction`, `currency`, `bank_account_id` après `default_project_id` ; `filter_key` ajoutée à
     `COLONNES_HORS_EXPORT` (`:2558`) avec son motif (« colonne GÉNÉRÉE (unicité des règles avec leurs filtres) :
     dérivée de `direction`, `currency`, `bank_account_id`, exportés »), sur le patron d'`active_uniq` (`:2584-2588`) ;
     `chaque_colonne_du_schema_est_exportee_ou_ecartee` (`:2377`) vert ; test unitaire d'une ligne à filtres.
10. **AC-10 — Écran (le minimum de cette story ; le formulaire est en 38-9b).** `rules.types.ts` : `MatchType` gagne
    les quatre valeurs, `ReconciliationRule` les trois filtres (`string | null`, `number | null`),
    `UpdateRuleRequest` les filtres facultatifs ; `RulesList.svelte` (`matchTypeLabel`, `:31-52`) : libellé des quatre
    types neufs (`reconciliation-rules-match-type-counterparty-email-exact` « Courriel exact »,
    `…-invoice-prefix` « Boutique (préfixe du numéro de facture) » — le libellé d'E14 —, `…-item-name-contains`
    « Article contient », `…-event-code-prefix` « Code d'événement commence par ») et, pour une valeur inconnue (cas
    `default`, `:50-51`, qui rend aujourd'hui la chaîne brute), « Critère d'une version ultérieure (`x`) »
    (`reconciliation-rules-match-type-unknown`) ; les filtres d'une règle s'affichent dans la ligne (« entrées · EUR ·
    PayPal EUR ») — clés `reconciliation-rules-filter-*`. `RuleFormModal.svelte` **n'est pas modifié** : il n'envoie
    ni `matchType` au `PATCH` ni les filtres (absents = inchangés, AC-4) — un `PATCH` depuis l'écran actuel ne retire
    donc aucun filtre (test Vitest qui l'établit sur le corps envoyé). Vitest de `RulesList` : chaque libellé, le
    libellé inconnu, l'affichage des filtres. **Captures d'écran** (*Recette visuelle*, thème clair) de la liste avec
    une règle de chaque type neuf, une à filtres et une inconnue, hors du dépôt (`kesh-gate-logs/`), jointes au Dev
    Agent Record et au prompt de revue.
11. **AC-11 — i18n.** Toutes les clés neuves (libellés de l'AC-10, message `RULE_ACCOUNT_CURRENCY_MISMATCH`,
    message de doublon revu, message neuf de `RULE_CRITERION_UNKNOWN`, type et sens) dans `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl` ;
    `npm run lint-i18n-ownership` vert ; décompte des clés recompté par `grep -c` aux deux bornes, au Change Log.
12. **AC-12 — MCP, manuels.** **MCP** : aucun outil n'expose les règles (`crates/kesh-api/src/mcp/outils/` : pas de
    module de règles) ; E23 (aucun outil neuf) — rien à faire, écrit au Change Log. **Manuels** : le manuel
    utilisateur décrit les règles par le formulaire (`docs/manual/fr/user-manual.tex`) ; le formulaire ne change
    qu'en 38-9b, qui écrit la section (critères, boutique = préfixe, filtres, priorité entre `LG` et `LGT`, compte en
    devise). Cette story vérifie que rien de ce que le manuel dit des quatre critères existants ne devient faux
    (grep de « règle d'affectation » dans le `.tex` et dans le PDF aplati — `pdftotext … | tr '\n' ' '`), et liste au
    Change Log les faits à reprendre par 38-9b. Aucune régénération de PDF.

## Tâches

- [ ] Migration de l'AC-1 ; ligne d'audit P5 et compteurs recomptés ; grep P6 ; squash régénéré ; liste close R1
      (AC-9).
- [ ] `kesh-db` : `RuleDirection` (décodage tolérant, `as_str`), champs de l'entité, de `New…`, `Update…`,
      `COLUMNS`, `create_in_tx`, `update_in_tx` (SET dynamiques, audit `before`/`after` complet) (AC-3, AC-4).
- [ ] `kesh-reconciliation::rules` : `match_key` (AC-2), évaluation des quatre types et des filtres (AC-3),
      `rule_account_admits`, `first_matching_rule` sur la carte des comptes éligibles (AC-6) ; tests unitaires.
- [ ] `kesh-api` : routes (AC-4, AC-5, AC-7), retrait de `normalize_iban_canonical`, levée de la garde 37-1 pour
      les règles et code `RULE_ACCOUNT_CURRENCY_MISMATCH` (AC-6), `get_proposals` et `accept_one_rule` (AC-6),
      dans `errors.rs` : `RuleAccountCurrencyMismatch` (neuf), `RuleCriterionUnknown` (neuf, type et sens),
      détails du doublon ; `ReconciliationMatchType` tolérant et ses quatre variantes, perte de `Copy` (en-tête).
- [ ] Export CSV et `COLONNES_HORS_EXPORT` (AC-9) ; aller-retour `.keshbackup`.
- [ ] Frontend (AC-10), i18n (AC-11), captures.
- [ ] Tests HTTP et unitaires listés aux AC ; non-régression (AC-8).
- [ ] Gate complet (fmt, clippy, `test-fast.sh`, `scripts/gate-ci.sh` au dernier commit de code), base remise à
      zéro avant ; frontend (`check`, `lint-i18n-ownership`, `test:unit`, `build`) ; E2E complet au dernier commit de
      code (aucun E2E neuf : l'écran des règles ne change qu'en libellés ; les specs de règles existantes doivent
      rester vertes).

## Notes de développement

- **Site de l'évaluation** : `rule_matches` est appelée par `first_matching_rule` (proposition) et par l'étape 7
  d'`accept_one_rule` (revalidation, `routes/reconciliation.rs:4154`). Les deux reçoivent la même règle et la même
  transaction : filtres et critère neufs y sont donc revalidés sous verrou sans code de plus.
- **Ce que 38-2 et 38-3a doivent livrer et que cette fiche lit** (fiche 38-3a relue le 2026-11-15) : **de 38-2** (O1)
  `match_type VARCHAR(40) CHARACTER SET ascii COLLATE ascii_bin` (8 + 40 + 1020 + 48 octets de clé pour l'index de
  l'AC-1) ; **de 38-3a**
  `item_names JSON` tableau de chaînes (`CHECK … JSON_TYPE = 'ARRAY'`, ≤ 50 noms — l'évaluation de l'AC-3 lit le
  tableau décodé, nom par nom, par `BankTransaction::item_names() -> Vec<String>`, posé par 38-3a AC-3), les colonnes
  `counterparty_email`, `invoice_ref`, `item_names`, `event_code` de `BankTransaction` (C-38-4). **Cette story** pose
  `ReconciliationMatchType::{CounterpartyEmailExact, InvoicePrefix, ItemNameContains, EventCodePrefix, Other(String)}`
  (`Copy` perdu : `routes/reconciliation_rules.rs:288`, `:411` copient aujourd'hui `match_type` — `clone()`) ; il n'y a
  plus d'étape intermédiaire où les quatre types neufs seraient déclarés mais refusés.
- **Pourquoi `match_key` stocke la forme normalisée pour les types neufs seulement** : les trois critères textuels
  existants affichent la valeur saisie depuis toujours ; la normaliser au stockage changerait ce que l'utilisateur
  relit (NFR-38-4-1). La comparaison, elle, passe par la même fonction pour tous.
- **Unicité et collation** : `match_value` est en `utf8mb4_unicode_ci` (table, `20260513000001:71`) ; `active_uniq`
  hérite de la collation : `LG` et `lg` seraient déjà égaux pour l'index ; `match_key` rend l'égalité vraie aussi pour
  les blancs (`L G`).
- **Inventaire des lecteurs de `match_type`** (epic, § *Inventaire*) relu : `rules.rs:46-70` ;
  `routes/reconciliation_rules.rs:63, 82, 101, 121, 235, 240, 270, 282, 288, 396, 411` ; `errors.rs:1150-1159,
  3303-3320` ; `exports/csv_tables.rs:1044` ; `routes/reconciliation.rs:1458` (`rule_match_type` d'une proposition),
  `:4359`, `:4397` (traces de l'acceptation) ;
  frontend `RuleFormModal.svelte:26, 104, 154`, `RulesList.svelte:31, 141`.
- **Précédent d'un compte désigné « validé seulement s'il change »** : `company_invoice_settings.rs:240-262`.
- **Ce qui n'est pas fait ici** : formulaire et manuel (38-9b) ; propositions de prestataire et levée du saut hors
  CHF (38-6a) ; catégorie de vente par analytique (Q3, reportée, C-38-13).

## Change Log

- 2026-11-15 — rédaction (Opus).
- 2026-11-15 — validation P1 (Sonnet) : 8 findings (0 CRITICAL, 0 HIGH, 5 MEDIUM, 3 LOW) ; remédiés dans la fiche sauf
  38-4-P1-8 (inter-fiche, 37-1). MEDIUM : P1-1 la fiche attribuait la migration breaking et `match_type VARCHAR(40)` à
  38-3 (O1 les place en 38-2) — en-tête, AC-1 (ordre des migrations, absence de relèvement), notes réalignés ;
  P1-2 « `PATCH` portant `matchType` → 400 » contredisait l'Écart 1 de la fiche (extracteur `axum::Json` des règles :
  rejet de données = 422, `routes/reconciliation_rules.rs:26, 368`, `extractors.rs:11-13`) — corrigé à 422 aux deux
  sites ; P1-3 colonne générée `filter_key` sur une colonne de base à FK : actions référentielles restreintes par
  MariaDB, charset du `CAST` — test de migration ajouté à l'AC-1 ; P1-4 l'« accesseur » d'`item_names` que 38-3 devait
  fournir n'existait pas dans 38-3 — posé à 38-3 AC-3. LOW : P1-5 numéros de ligne de l'index d'origine
  (`:65-66`, non `:68-69`) ; P1-6 position du refus `RULE_ACCOUNT_CURRENCY_MISMATCH` par rapport au refus hors CHF
  (`:4142`) ; P1-7 `RULE_CRITERION_UNKNOWN` « étendu au sens » ambigu contre le 400 `VALIDATION_ERROR` de l'écriture.
  P1-8 (inter-fiche) : 37-1 AC-4 et C-37-9 de l'epic 37 refusent encore « désigner un compte en devise » pour les règles
  (`ACCOUNT_IN_FOREIGN_CURRENCY`) — la levée de l'AC-6 (O3) doit s'y écrire. Axes non exercés : le décompte des
  dix-neuf FK de 37-1 (relu sur la fiche 37-1, non sur le code), l'exécution de la migration (aucun `cargo`), les
  littéraux de test (`ReconciliationRule {` ×1, `NewReconciliationRule {` ×5, `UpdateReconciliationRule {` ×12).
- 2026-11-15 — renvois à 38-3 répartis après le découpage D-P1-1 (Opus) : colonnes et accesseur `item_names()` lus de
  **38-3a** ; le décodage tolérant de `ReconciliationMatchType` (quatre variantes, `Other`, perte de `Copy`) et le code
  `RULE_CRITERION_UNKNOWN` sont **repris par cette story** (liste inter-fiches, point 1 : le décodage arrive avec la
  première écriture d'un critère neuf) — en-tête, Écart 1, AC-4, AC-7, AC-11, tâches et notes réalignés ; plus de
  refus provisoire. Modules inchangés (`kesh-db` déjà touché).
