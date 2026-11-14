# Story 35-1 : réglage multi-devises de la société et devise des comptes

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`, numéros de ligne de `67efc51f`. Epic 35
(`_bmad-output/planning-artifacts/epic-35-multi-devises.md`), registre C-35-1 à C-35-8. **Migration non breaking**.
Révisée le 2026-11-14 pour la revue de Beat (CR-2 : R1, R3, R6 — colonnes seulement —, R7, R10), qui prime sur le
registre antérieur.*

## Story

En tant que comptable d'une société qui encaisse et paie en USD et en EUR, je veux passer ma comptabilité en mode
multi-devises, désigner mes comptes de différences de change et donner une devise à certains comptes du plan, afin d'y
passer ensuite des écritures en devise (35-3), sans rien changer pour qui reste en mono-devise.

## Décisions qui s'appliquent

D1, D2 (fichier d'epic) ; C-35-2 (réglage sur `companies`, sans FK, quatre gardes), C-35-5 (verrous), C-35-9 (409 partout,
verrou des deux côtés, réglage réservé à l'Administrateur). **CR-2 (revue de Beat, prime)** : R1 (une sauvegarde
antérieure à l'Epic 35 se réimporte : liste close des tables et colonnes de l'epic, AC-6) ; R3 (mode mono-devise ou
multi-devises **par exercice**, le réglage de la société en est le défaut, AC-3 bis) ; R6 (colonnes
`opening_rate_kind`/`closing_rate_kind` seulement, AC-1 ; route et écran en 35-2) ; R7 (compte de conversion en
CHF désigné à l'activation, AC-3) ; R10 (constante du début de l'exercice initial de l'assistant, AC-11).

## Critères d'acceptation

1. **AC-1 — Migration** `20261114000001_multi_currency_settings.sql`, non breaking (DDL seul, aucune écriture de
   données) :
   - `companies` : `multi_currency_enabled BOOLEAN NOT NULL DEFAULT FALSE`, `fx_gain_account_id BIGINT NULL`,
     `fx_loss_account_id BIGINT NULL`, `fx_transit_account_id BIGINT NULL` (compte de conversion, R7), **sans clé
     étrangère** (commentaire SQL qui dit pourquoi : cycle `companies` ↔ `accounts`, réinitialisation ; intégrité par le
     dépôt, C-35-2) ;
   - `fiscal_years` (R3, R6) : `multi_currency BOOLEAN NOT NULL DEFAULT FALSE` (mode de l'exercice, AC-3 bis),
     `opening_rate_kind ENUM('day','month') NOT NULL DEFAULT 'day'` et `closing_rate_kind ENUM('day','month') NOT NULL
     DEFAULT 'day'` (genre du cours d'ouverture et de clôture de l'exercice, R6 : lus par 35-3b et 35-5, réglés à
     l'écran *Cours de change* de 35-2). **Décision CR-2** : les trois colonnes d'exercice sont dans **cette** migration,
     non dans celle de 35-2 — l'entité `FiscalYear` et ses quinze listes de colonnes écrites à la main (Tâches) ne se
     touchent qu'une fois, et le test de montée de `fiscal_years` est unique ; 35-2 n'écrit que la route et l'écran qui
     les règlent. Les exercices existants prennent `FALSE`, `'day'`, `'day'` (une base 0.17.6 est mono-devise) ;
   - `accounts` : `currency CHAR(3) CHARACTER SET ascii COLLATE ascii_bin NULL` (NULL = devise principale),
     `CHECK (currency IS NULL OR BINARY currency REGEXP '^[A-Z]{3}$')` et
     `CHECK (currency IS NULL OR BINARY currency <> BINARY 'CHF')` (la devise principale s'écrit NULL, une seule
     graphie). ⚠️ La table est en `utf8mb4_unicode_ci` (`20260411000001_accounts.sql:23`) : sans collation binaire,
     `REGEXP` ignore la casse et `usd` passerait. Test : un `INSERT` de `usd` est refusé par le schéma.
   - Les colonnes des lignes d'écriture **ne sont pas** dans cette migration (35-3).
   - Inscrite à `migrations.sha384`, au tableau de `docs/migrations-idempotence-audit.md` (compteurs recomptés, P5),
     squash régénéré (`scripts/regen-test-schema.sh`), P6 inspecté, P7 : aucune donnée écrite (le test de triage passe
     sans entrée).
   - `kesh_version_min_required` inchangé (0.17.6).
   - Test de montée sur base peuplée (`migrations_upgrade_path.rs`, précédent de la 34-11a1) : comptes, société et
     exercices existants prennent les défauts.
2. **AC-2 — Lecture du réglage.** `GET /api/v1/companies/current` expose `multiCurrencyEnabled`, `fxGainAccountId`,
   `fxLossAccountId`, `fxTransitAccountId`, `baseCurrency: "CHF"` (constante, C-35-2 ; la devise principale ne change
   pas). `GET /api/v1/fiscal-years` et le détail d'un exercice (`FiscalYearResponse`, `routes/fiscal_years.rs:51`, et
   `services/exercices.rs`) exposent `multiCurrency`, `openingRateKind`, `closingRateKind` — toujours présents, comme
   les champs du réglage de `CompanyJson` (ce ne sont pas des champs en devise, C-35-9). L'outil MCP `kesh_exercices`
   n'est pas étendu (sa sortie `sortie!` ne lit pas ces champs : hors périmètre, écrit au Change Log).
3. **AC-3 — Écriture du réglage.** `PUT /api/v1/companies/current/currency-settings`
   `{ multiCurrencyEnabled, fxGainAccountId, fxLossAccountId, fxTransitAccountId, version }` (verrou optimiste comme
   les autres `PUT` de la société) :
   - **Administrateur seul**, comme les trois `PUT /companies/current/*` et `PUT /company/invoice-settings`
     (`lib.rs:277-278, 446-461`) : route dans `admin_routes`, **au-dessus** du marqueur « TOUTE ROUTE S'AJOUTE
     AU-DESSUS DE CETTE LIGNE » (`require_admin_role` + `require_not_pat`) ; une clé d'API reçoit
     `API_KEY_ADMIN_FORBIDDEN` ; la route s'ajoute à la liste de `admin_pat_denied_e2e.rs` (:85). **Conséquence
     assumée** : la recette par l'API (clé d'API) ne peut pas activer le mode multi-devises ; il s'active à l'écran,
     par un Administrateur ;
   - fonction dédiée `companies::update_currency_settings` (ne passe **pas** par `CompanyUpdate`/`update_in_tx`, qui
     sont en remplacement complet : chaque route e-mail, coordonnées, adresse et `company_update_of` de l'assistant
     devrait sinon recopier trois champs de plus) ;
   - activer exige **les trois comptes** (gains, pertes, conversion ; le même compte est admis pour les gains et les
     pertes) ; tant que le mode est actif, aucun des trois ne peut être vidé (400 `FX_ACCOUNT_INVALID`,
     `reason: "missing"`) ; `multiCurrencyEnabled: false` avec un identifiant non nul → 400 `FX_ACCOUNT_INVALID`,
     `reason: "unexpected"` (la désactivation les remet à `NULL`, le corps ne les porte donc pas) ; **no-op** (rien ne
     change) : court-circuit sans bump de `version` ni audit, comme KF-004 ;
   - **compte de conversion** (R7, `fx_transit_account_id`) : compte **de bilan** en CHF par lequel passe une
     conversion entre deux devises (35-3b, assistant *Conversion* : il revient à 0 après les deux écritures) ; mêmes
     gardes que les comptes de gains et de pertes (de la société, imputable, actif, **sans devise**, lu `FOR UPDATE`,
     motif C4, revalidation à l'usage par la route de conversion de 35-3b — la réévaluation de 35-5 n'emploie que les
     comptes de gains et de pertes —, verrous de l'AC-4 sous `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`),
     **sauf le type** : `Asset` ou `Liability` (à créer, p. ex. 1090 *Transferts de fonds* — aucun plan livré n'a de
     compte 109x, grep des `*.json` de plans du 2026-11-14), jamais `Expense`/`Revenue`
     (`reason: "accountType"`) ; il ne peut donc pas être le compte de gains ou de pertes. La contrainte « ni au rôle
     `DefaultRevenue`, ni désigné `default_revenue_account_id` » ne le concerne pas (un compte de bilan n'est pas un
     compte de produit) ; la liste close des `reason` est inchangée ;
   - **exercices à l'activation** (R3) : passer le réglage à `true` passe **tous les exercices ouverts** de la société
     en multi-devises (`fiscal_years.multi_currency = TRUE`) dans la même transaction — sans quoi l'exercice initial de
     l'assistant, créé mono-devise (la société l'était), le resterait ; les exercices clos ne changent pas. La
     **désactivation**, admise seulement sans compte en devise (ci-dessous, donc sans aucune ligne en devise), passe
     **tous** les exercices de la société à `FALSE`, clos compris (ils ne portent rien en devise). Les identifiants
     des exercices changés vont au détail d'audit. Ensuite, chaque exercice se règle seul (AC-3 bis) ;
   - **ordre des refus** (une seule raison) : forme du corps (`missing`/`unexpected`, 400, avant la transaction) →
     version (409, jugée sous le `FOR UPDATE` de `companies`) → no-op → `MULTI_CURRENCY_IN_USE` (désactivation) →
     `FX_ACCOUNT_INVALID` (compte changé, gain puis perte puis conversion) ;
   - chaque compte : de la société, imputable, actif, de type `Expense` ou `Revenue`, **sans devise** ; **ni** au rôle
     `DefaultRevenue`, **ni** désigné `default_revenue_account_id` (réglages de facturation ou articles) — les gains de
     change ne vont pas au chiffre d'affaires (CO 959b) ; lu `FOR UPDATE` ; sinon 400 `FX_ACCOUNT_INVALID` (champ et
     `reason` en `details`, p. ex. `reason: "revenueAccount"` ; **liste close des `reason`**, reprise par la 35-5 : `missing`, `unexpected`, `unknown`
     (introuvable ou d'une autre société), `archived`, `notPostable`, `accountType`, `foreignCurrency`,
     `revenueAccount`). Les autres désignations (arrondi 6940, frais bancaires
     6900) restent admises. **Motif C4** du précédent (`validate_account_of`, `crates/kesh-api/src/routes/company_invoice_settings.rs:219,231` ;
     `DESIGNATED_ACCOUNT_COLUMNS` vit, lui, dans `crates/kesh-db/src/repositories/company_invoice_settings.rs:46-57`) :
     ces contrôles ne portent que sur un compte **changé** ; un compte inchangé devenu invalide n'empêche pas
     d'enregistrer l'autre (la revalidation à l'usage — la réévaluation, 35-5, C-35-9 — reste le filet). ⚠️
     `validate_account_of` vit dans `crates/kesh-api/src/routes/company_invoice_settings.rs` (:200), non dans le
     dépôt du même nom ; le contrôle de la 35-1 est écrit dans `companies::update_currency_settings`, sous verrou ;
   - désactiver est refusé (409 `MULTI_CURRENCY_IN_USE`) tant qu'un compte porte une devise, **comptes archivés
     compris** ; `details` liste (jusqu'à 20) les numéros de ces comptes, chacun avec `locked` (il porte une ligne :
     sa devise ne se retire plus jamais, `update` refusant aussi un compte archivé — l'utilisateur doit le
     réactiver pour en changer la devise) ; désactiver remet les trois comptes à `NULL` ;
   - **verrous, dans l'ordre global du dépôt** (`companies` → réglages → `accounts` → `bank_accounts`,
     `accounts.rs:446-448`) : ligne `companies` `FOR UPDATE` (version jugée dessus, et non sur une lecture sans verrou
     comme `update_in_tx` :849), puis — quand le geste active ou désactive — les exercices de la société `FOR UPDATE`
     dans l'ordre chronologique (CR-2 ; ordre `companies` → exercices → `accounts` de `create_in_tx_inner`, qui prend
     l'exercice `FOR UPDATE` avant de lire les comptes, 35-3a AC-4 : les prendre après les comptes formerait un cycle),
     puis les réglages de facturation `LOCK IN SHARE MODE` (contrôle « désigné
     `default_revenue_account_id` »), puis les comptes désignés `FOR UPDATE`, puis les articles qui les désignent
     (`products.default_revenue_account_id`, lecture verrouillante), puis le `COUNT` des comptes en devise ;
     ⚠️ **ce `COUNT`, comme tout test d'existence de ligne d'écriture de l'AC-4, est une lecture verrouillante**
     (`… LOCK IN SHARE MODE`) et **aucune lecture simple ne le précède dans la transaction** : sous REPEATABLE READ,
     une lecture simple fige l'instantané *avant* l'attente des verrous et ne verrait pas une ligne ou un compte
     commités entre-temps (c'est pourquoi la lecture des comptes bancaires de la 34-11a2 est elle-même verrouillante,
     `accounts.rs:617-620`) ;
   - **rejeu sur interblocage** (35-1-CR2-P1-1) : l'ordre « `companies` → exercices » ci-dessus n'est **pas** celui des
     écritures — `create_in_tx_inner` lit `companies` sans verrou, prend l'exercice `FOR UPDATE`
     (`journal_entries.rs:326-336`), puis son `INSERT` prend `companies` en partage par `fk_journal_entries_company`
     (exercice → société). Une activation ou une désactivation, qui tient `companies` `FOR UPDATE` puis demande les
     exercices, ferme donc un cycle avec toute saisie d'écriture en cours (angle mort (iv) du registre des routes) :
     le handler est sous `kesh_db::retry::retry_on_deadlock("companies::update_currency_settings", …)`, comme
     `create_fiscal_year`, et la route est **rejouée** — la 35-1 ajoute donc **deux** routes `SansEcritureAuJournal`
     rejouées (celle-ci et `currency-mode`, AC-3 bis) : `audit_route_registry.rs` (vi) passe de « vingt-cinq » à
     « vingt-sept » (décompte, liste et « vingt-trois handlers `SansEcritureAuJournal` de `src/routes/` » recomptés au
     code, en deltas), et un test de victime d'interblocage par route dans `rejeu_interblocage_e2e.rs` (patron
     `fiscal_year_reopen_is_replayed_when_it_is_the_deadlock_victim`). **Autres cycles, couverts par le même rejeu**
     (35-1-CR2-P2-2, à écrire au Change Log du développement) : la validation d'une facture, l'avoir et la saisie
     fournisseur prennent les réglages de facturation `FOR UPDATE` **avant** l'exercice (`invoices.rs:1974`,
     `credit_notes.rs:312`, `supplier_invoices.rs:1484`), l'activation les réglages en partage **après** les
     exercices — routes rejouées des deux côtés ; et la bascule `currency-mode` sous `force` (AC-3 bis) tient
     l'exercice et les lignes des comptes en devise en lecture verrouillante puis demande ces comptes `FOR UPDATE`,
     quand une écriture d'un exercice postérieur tient son exercice et le compte en partage (35-3a) avant d'insérer
     sa ligne — c'est ce cycle-là, non celui de `companies`, qui fait rejouer `currency-mode` (elle ne prend
     `companies` qu'en partage) ;
   - audit `company.currency_settings_updated` (avant/après, ajouté à `ACTIONS` de `audit_labels.rs`, liste triée) ;
     libellés d'audit dans les quatre locales ;
   - **réponse** : 200 avec le `CompanyJson` à jour (nouvelle `version`, comme les trois autres `PUT` de la société) ;
     tests : Comptable → 403, clé d'API → `API_KEY_ADMIN_FORBIDDEN` ;
   - **registre des routes** : le `PUT` entre dans `LIB_ROUTES` (`tests/audit_route_registry.rs`) avec le couple
     `Traced, SansEcritureAuJournal` (le quatrième champ du tuple est la partition de rejeu : il écrit au journal
     d'audit, non au journal comptable) ; **deux** tests comptent, et le second est à recompter aussi :
     `the_registry_partition_is_what_the_story_declares` (:992) : **+1 route et +1 tracée**, total du registre
     (`LIB_ROUTES` + `TEST_ENDPOINT_ROUTES`, `:1069-1073`) **+1** ; `the_replay_partition_is_what_the_story_declares`
     (:1450) : `sans_ecriture` **+1** (:1480), `tout.len()` **+1** (:1502 ; `rejouees` et `exemptees` inchangés) ;
     messages d'assertion et commentaires recomptés, dont l'en-tête du module (`//!`, :33-35). **Chiffres en deltas**
     (état au 2026-11-14 : 146 routes, 139 tracées, total 149, `sans_ecriture` 119) : les 35-2 et 35-5 ajoutent aussi
     leurs routes, l'ordre d'intégration décide du chiffre absolu, **recompté au code**. La phrase du manuel
     d'administration (`admin-manual.tex:3217`, « N des M routes… », « N + 4 + 3 = M ») suit le chiffre recompté, PDF
     régénéré, garde `le_manuel_dit_le_compte_du_registre`. *(Alignement de l'orchestrateur sur 35-2-P4-IF3.)*
     **CR-2** : la route de l'AC-3 bis s'y ajoute avec le même couple — la 35-1 compte donc **+2 routes, +2 tracées,
     total +2, `sans_ecriture` +2, `tout.len()` +2**, toujours en deltas.
3. **AC-3 bis — Mode d'un exercice (R3).** Chaque exercice est **mono-devise** ou **multi-devises**
   (`fiscal_years.multi_currency`, AC-1). Le réglage de la société en est le **défaut** :
   - **à la création**, l'exercice prend `companies.multi_currency_enabled`, lu **dans la transaction qui l'insère**
     (`INSERT … SELECT … FROM companies`, ou lecture `LOCK IN SHARE MODE` en tête de transaction, avant les verrous
     d'exercice : ordre `companies` → exercices). **Inventaire clos des chemins qui créent un exercice** (grep
     `INSERT INTO fiscal_years`, relevé au code le 2026-11-14) : `fiscal_years::create` (`repositories/fiscal_years.rs:331`,
     `INSERT` dans `insert_fiscal_year_in_tx` :1360 ; route `POST /api/v1/fiscal-years`, `routes/fiscal_years.rs:259`) ;
     `create_if_absent_in_tx` (:456, `INSERT … SELECT … FROM dual` :470 ; **l'exercice initial de l'assistant**,
     `routes/onboarding.rs:1053`, dont la transaction tient déjà `companies` `FOR UPDATE` par `company_select!` — la
     colonne s'y lit sans verrou de plus) ; `create_for_seed` (:412 ; `kesh-seed/src/lib.rs:286`, `pieges.rs:379`, et
     deux tests de `journal_entries.rs`). Les `INSERT` bruts des tests (`test_fixtures.rs:118`, `invoices.rs:4334`,
     `accounts.rs:1499`, `journal_entries.rs:4022`, `:4220`, `:4353`) prennent le défaut de la colonne (`FALSE`) : admis,
     aucun changement. L'import d'une sauvegarde recopie la colonne (ou la met à son défaut, AC-6) ; la remise à zéro
     recrée l'exercice par l'assistant. Inventaire écrit au Change Log. Test : société multi-devises → exercice créé
     multi-devises par chacun des trois chemins ; société mono-devise → mono-devise ;
   - **bascule** : `PUT /api/v1/fiscal-years/{id}/currency-mode { multiCurrency, force? }`, dans **`comptable_routes`**
     (Administrateur ou Comptable, comme `…/close` ; une clé d'API l'atteint comme `close`), handler sous
     `kesh_db::retry::retry_on_deadlock("fiscal_years::set_currency_mode", …)` comme `create_fiscal_year`
     (`routes/fiscal_years.rs:286`), audit `fiscal_year.currency_mode_updated` (`{ before, after, archivedAccountIds }`,
     `ACTIONS` triée, quatre locales), registre `Traced, SansEcritureAuJournal` (ci-dessus). **Ordre des refus** (une
     seule raison) : corps illisible → 400 (l'extracteur `Json` passe avant le handler, comme pour `…/reopen`) ;
     exercice inexistant ou d'une autre société → 404 (pré-vol sur le pool) ; puis, dans la
     transaction, `companies` `LOCK IN SHARE MODE`, les exercices antérieurs verrouillés par clé primaire dans l'ordre
     chronologique puis l'exercice `FOR UPDATE` (ordre de `fiscal_years::close`, `LOCK_EARLIER_BY_ID_SQL` /
     `LOCK_IN_COMPANY_SQL`, sans ses lectures ordinaires de tête — 35-5 AC-2 le détaille) : `multiCurrency: true` dans
     une société mono-devise → 409 `MULTI_CURRENCY_DISABLED` ; exercice clos → 400 `FISCAL_YEAR_CLOSED` (le mode d'un
     exercice clos est figé) ; no-op → 200 sans audit ni écriture ; **vers mono-devise** :
     - **refus** 409 **`FISCAL_YEAR_HAS_FOREIGN_BALANCES`** (code neuf, quatre locales) tant qu'un compte en devise
       (archivés compris) a un solde cumulé **≠ 0 au début de l'exercice** (lignes datées avant `start_date`, toutes
       années) — en CHF (Σ débit − Σ crédit) **et** en devise (Σ `amount_currency` signé, colonne de 35-3a : **35-3a
       étend cette requête**, la 35-1 ne connaissant que le CHF et l'existence des lignes) — ou **porte une ligne** datée
       dans l'exercice ; `details: { accounts: [{ id, number, reason: "balance" | "lines" }] }` (20 au plus). Lectures
       **verrouillantes** (`LOCK IN SHARE MODE`), après les verrous d'exercice, aucune lecture simple avant (AC-3) ;
     - puis **avertissement** : s'il reste des comptes en devise **actifs** qui ne portent **aucune ligne datée après
       la fin** de l'exercice (ceux qui servent à un exercice postérieur ne sont pas proposés), sans `force: true` → 409
       **`FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE`** (code neuf), `details: { accounts: [{ id, number, currency }] }` : Kesh
       **propose de les archiver** (R3). Avec `force: true`, ces comptes sont archivés dans la même transaction (audit
       `account.archived` de chacun, `version` + 1, verrous `accounts` `FOR UPDATE` après les exercices), puis l'exercice
       passe en mono-devise. **Comment** (35-1-CR2-P1-2) : `archive_guarded` ouvre sa propre transaction sur le pool
       (`accounts.rs:859`) et refuse un compte aux sous-comptes actifs ; le corps de l'archivage est donc **extrait**
       en `archive_in_tx` (mêmes refus, même audit, aucune copie : DRY), appelé par `archive_guarded` et par
       `set_currency_mode`. Contrat (35-1-CR2-P2-8) : `archive_in_tx` reçoit la société attendue et **compare** le
       `company_id` relu sous le `FOR UPDATE` du compte (écart → `NotFound`, comme l'AC-4 pour `archive_guarded`) ;
       sous `force`, la `version` passée est celle relue sous ce verrou ; `companies` et les réglages de facturation
       sont pris en partage **une fois**, par l'appelant, avant le premier compte (ordre `companies` → exercices →
       réglages → `accounts` de l'AC-3). Seuls les comptes **imputables** sont proposés et archivés ; un compte de groupe porteur
       d'une devise (informatif, AC-4) n'est jamais proposé — il ne reçoit aucune ligne — et garde sa devise (il se
       retire par `PUT`, aucune ligne). Un compte imputable qui garde un sous-compte actif (état anormal) n'est pas
       proposé non plus. Les comptes se traitent par identifiant croissant. L'archivage se défait par la réactivation (société multi-devises). Aucun compte n'est
       converti en CHF (R3) ;
     - **vers multi-devises** : aucune autre condition ;
   - **dans un exercice mono-devise**, un compte en devise **n'est pas utilisable** : une ligne en devise est refusée
     (409 `MULTI_CURRENCY_DISABLED`, jugé sur l'exercice de l'écriture, 35-3a AC-4) et une ligne CHF sur un compte en
     devise l'est déjà (`ACCOUNT_REQUIRES_CURRENCY`, 35-3a) ; 35-3a garde aussi un exercice **antérieur** à un exercice
     mono-devise (sans quoi le solde contrôlé ici à la bascule changerait après coup). Le réglage de la société reste
     la condition de tout ce qui est en devise (devise d'un compte, écrans, récupération des cours) : un exercice
     multi-devises n'existe que dans une société multi-devises ;
   - **écran** *Exercices* (société multi-devises seulement) : colonne « Devises » (« mono » / « multi ») et action
     « Passer en mono-devise » / « Passer en multi-devises » (Administrateur, Comptable) ; sur
     `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE`, un dialogue liste les comptes et propose « Archiver et passer en
     mono-devise » (renvoi avec `force: true`) ; sur `FISCAL_YEAR_HAS_FOREIGN_BALANCES`, la liste des comptes et la
     raison. Vitest et E2E ;
   - **tests** : bascule refusée (solde CHF ≠ 0 au début ; ligne dans l'exercice ; compte archivé compris),
     avertissement puis `force` (comptes archivés, audit), compte utilisé par un exercice postérieur non proposé, exercice
     clos, société mono-devise, no-op, Consultation → 403, course bascule contre écriture CHF dans un exercice antérieur
     (sérialisées par le verrou de l'antérieur).
4. **AC-4 — Devise d'un compte.** `CreateAccountRequest.currency` est facultatif (`null` ou absent = CHF).
   `UpdateAccountRequest.currency` est `Option<Option<String>>` désérialisé par `present_or_null` (`routes/accounts.rs:60`) :
   **absent = devise inchangée**, `null` = CHF, valeur = devise. L'absence se résout **dans le dépôt**, sous le
   `FOR UPDATE` du compte : `AccountUpdate.currency` est lui aussi à trois états (inchangé / CHF / devise), jamais
   rempli par une lecture de la route. ⚠️ **Pas de 400 sur l'absence, à la différence de `role`** :
   le motif d'un full-replace est d'éviter l'effacement silencieux d'une donnée (`routes/accounts.rs:69-76`) ; ici
   « absent = inchangé » n'efface rien, et un 400 casserait en mono-devise tout client qui ne connaît pas le champ
   (la recette par l'API de l'epic, l'écran de la version précédente), ce que la story s'interdit (« sans rien changer
   pour qui reste en mono-devise », précédent : la `confirm_account_retype`, `routes/accounts.rs:94-109`). Code ISO
   4217 à trois lettres, `trim` puis **majuscules ASCII** (`to_ascii_uppercase`, jamais Unicode) avant validation ;
   toute valeur non ASCII → 400 `CURRENCY_UNKNOWN`. `AccountResponse` expose `currency` et
   `currencyLocked: bool` (le compte porte au moins une ligne d'écriture ; calculé pour la liste en **une** requête,
   bornée à la société — `SELECT a.id FROM accounts a WHERE a.company_id = ? AND EXISTS (SELECT 1 FROM
   journal_entry_lines l WHERE l.account_id = a.id)`, par l'index `idx_jel_account` (celui de la contrainte
   `fk_jel_account`), plutôt qu'un `DISTINCT` sur
   toutes les lignes —, jamais une par compte ; la forme de `letterings::letterable_account_ids`) ; les réponses à un
   seul compte (création, modification, détail) le portent aussi. **Sérialisation (C-35-9 : le JSON des routes HTTP
   reste identique en mono-devise)** : `currency` porte `#[serde(skip_serializing_if = "Option::is_none")]` (absent
   pour un compte CHF) et `currencyLocked` `#[serde(skip_serializing_if = "std::ops::Not::not")]` (absent = `false`) ;
   type TS `currency?: string; currencyLocked?: boolean`, lu `account.currency ?? null`. Test : la réponse d'un
   compte CHF sans ligne n'a aucune des deux clés. Les champs du réglage de `CompanyJson` (AC-2) ne sont pas des
   champs en devise : toujours présents.
   - refus **409** `MULTI_CURRENCY_DISABLED` si la société est en mono-devise et qu'une devise est donnée (409 partout,
     décision d'epic) ;
   - code inconnu de la liste ISO 4217 embarquée (`kesh-core`) → 400 `CURRENCY_UNKNOWN` ; `CHF` donné → normalisé à
     `NULL` **avant** tout autre contrôle : en mono-devise, `"CHF"` (ou `"chf"`) est la devise principale, jamais un
     `MULTI_CURRENCY_DISABLED` (le refus ne vise qu'une devise **autre** que CHF). La liste ne valide qu'**à l'écriture** : la lecture décode la colonne en `String`, de sorte qu'un code
     retiré plus tard de la liste ne fasse jamais échouer la lecture d'un compte ;
   - changer la devise d'un compte qui porte **au moins une ligne d'écriture** → 409 `ACCOUNT_CURRENCY_LOCKED`
     (jamais de confirmation possible, à la différence du changement de type). Un `PUT` qui renvoie la devise en place
     est un no-op sur ce champ et ne déclenche pas ce refus (`currency` entre dans `is_no_op_change`) ;
   - **un compte que les chemins automatiques débitent en CHF ne peut recevoir de devise** → 409
     `ACCOUNT_CURRENCY_IN_USE` (le site en `details`) : compte qui porte un rôle (`AccountRole`, les dix) ; désigné
     dans les réglages de facturation (`DESIGNATED_ACCOUNT_COLUMNS`, `company_invoice_settings.rs:46-57`) ; lié à un
     compte bancaire (`bank_accounts.journal_account_id`) ; compte de produit par défaut d'un article
     (`products.default_revenue_account_id`) ; contrepartie d'une règle de réconciliation
     (`reconciliation_rules.counterparty_account_id`) (le compte par défaut des factures fournisseur,
     `default_payable_account_id`, est déjà l'une des dix colonnes de `DESIGNATED_ACCOUNT_COLUMNS`). **Inversement**,
     lier à un compte bancaire, désigner dans les réglages de facturation, choisir comme compte d'un article ou d'une
     règle de réconciliation, ou poser un rôle sur un compte **en devise** → 400 `ACCOUNT_IN_FOREIGN_CURRENCY`.
     **C'est la 35-1 qui pose ces refus des deux côtés, pour ces sites de configuration** ; les comptes choisis ligne
     par ligne (`invoice_lines`/`credit_note_lines.revenue_account_id`, `supplier_invoice_lines.expense_account_id`,
     `supplier_invoices`/`invoice_settlements.settlement_account_id`) ne sont **pas** gardés par la 35-1 : la 35-3a (AC-5) les
     refuse à l'enregistrement de la pièce (400 `ACCOUNT_IN_FOREIGN_CURRENCY`) ; un brouillon antérieur dont le compte
     est devenu en devise est refusé par le contrôle central à la validation (`ACCOUNT_REQUIRES_CURRENCY`) — décision
     écrite au Change Log. *(Alignement de l'orchestrateur sur 35-3a-P4-IF1.)* La 35-3a (AC-5) s'appuie
     sur les refus de la 35-1.
     Les liens se comptent **quel que soit l'état** de l'objet qui désigne (compte bancaire archivé, article archivé,
     règle supprimée logiquement compris) : aucune réactivation de ces objets n'a de garde de devise. Limite assumée,
     écrite au manuel : un compte lié à un compte bancaire archivé ne reçoit plus de devise.
     Inventaire **clos** des sites, lu sur les FK `REFERENCES accounts` des migrations — **dix-neuf** à ce jour
     (`grep -rn "REFERENCES accounts" crates/kesh-db/migrations/*.sql`), dont `accounts.parent_id` et
     `journal_entry_lines.account_id` hors configuration, soit dix-sept colonnes : les dix des réglages, l'article, la
     règle, et les cinq colonnes de pièce laissées à la 35-3a — et sur les liens sans FK
     (`bank_accounts.journal_account_id`), écrit au Change Log (méthode « inventorier les sites non résolus ») ;
   - **ordre des refus** de `update` (une seule raison, la plus bloquante) : le pré-vol existant de la route reste en
     tête (`find_by_id_in_company` sur le pool → 404, `routes/accounts.rs:290` ; puis `name`, `role` et `postable`
     → 400), le contrôle `CURRENCY_UNKNOWN` se place après lui, dans la route ; avant la transaction, forme
     (`CURRENCY_UNKNOWN`, `check_role_account_type`) ; puis compte introuvable/archivé → version → no-op →
     `MULTI_CURRENCY_DISABLED` → `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` → `ACCOUNT_CURRENCY_LOCKED` →
     `ACCOUNT_CURRENCY_IN_USE` → `ACCOUNT_IN_FOREIGN_CURRENCY` → refus existants dans leur ordre actuel (rôle singleton
     déjà porté, `AccountLinkedToBankAccount`, `AccountHasEntries`). Jugés sur l'**état résultant** du `PUT` : devise
     posée **et** rôle posé dans la même requête → `ACCOUNT_IN_FOREIGN_CURRENCY` (comme `create`) ; devise posée sur
     un compte qui garde son rôle → `ACCOUNT_CURRENCY_IN_USE` ; rôle retiré et devise posée dans la même requête →
     admis (si aucun autre site ne désigne le compte). Pour `create` : `CURRENCY_UNKNOWN` (avant la transaction) →
     `MULTI_CURRENCY_DISABLED` → `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` (parent désigné) → `ACCOUNT_IN_FOREIGN_CURRENCY`
     (rôle posé en même temps) ;
   - un compte désigné gain/perte ne peut recevoir de devise, ni être archivé, ni changer de type hors
     `Expense`/`Revenue`, ni recevoir le rôle `DefaultRevenue`, ni passer à `postable = false`, ni recevoir un
     sous-compte (`accounts::create` :197-207 passe le parent à non imputable sans garde) → 409
     `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`. **Symétrique de l'AC-3** (les gains de change ne vont pas au chiffre
     d'affaires) : il ne peut pas non plus être **désigné** compte de produit par défaut — `default_revenue_account_id`
     des réglages de facturation (`company_invoice_settings::update`, qui lit alors `companies` `LOCK IN SHARE MODE`
     **avant** son `INSERT IGNORE` / `FOR UPDATE` des réglages, ordre global) ni d'un article (`products::create` /
     `update`, `companies` lu en partage avant le compte) — → 409 `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`, seulement si la
     valeur **change** (motif C4) ; sans quoi la garde de l'AC-3 se contourne en désignant après coup, et la
     revalidation de la 35-5 ne le verrait pas. À l'archivage, ce refus est **sans recours** — à la différence du précédent
     #486 (`AccountDesignatedInSettings`, confirmable) : `confirm_designated` ne le lève pas. Ordre des refus
     d'`archive_guarded` : sous-comptes actifs → gain/perte → désignation de facturation → version ;
   - **compte de conversion** (R7) : mêmes refus, même code (`ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`, `details.designation`
     = `"gain"`, `"loss"` ou `"transit"` — champ ajouté pour les trois), sauf le type : il ne peut pas passer hors
     `Asset`/`Liability` ; le refus de désignation « produit par défaut » ne le vise pas. Le compte de conversion ne
     reçoit ni devise, ni archivage, ni sous-compte, ni `postable = false` ;
   - `reactivate` d'un compte en devise alors que la société est en mono-devise → 409 `MULTI_CURRENCY_DISABLED`
     (défensif : le décompte de la désactivation inclut déjà les archivés) ;
   - **verrous** : `accounts::create`, `update`, `archive_guarded` et `reactivate` lisent la ligne `companies`
     `LOCK IN SHARE MODE` **avant** le `FOR UPDATE` du compte (même ordre que l'AC-3, pas d'interblocage) ;
     `multi_currency_enabled` et les trois colonnes FX se lisent sous ce verrou. ⚠️ **Cela change trois fonctions** :
     `update` lit aujourd'hui le compte `FOR UPDATE` *en premier* (`FIND_BY_ID_FOR_UPDATE_SQL`), et `archive_guarded`
     (:859) et `reactivate` (:1026) ne prennent **aucun** `FOR UPDATE` sur le compte (lecture simple de `company_id`
     ou de la ligne). Elles lisent donc d'abord `company_id` (immuable) **hors transaction**, sur le
     pool (voir ⛔ ci-dessous), puis, dans la transaction, `companies` en partage, puis les réglages en partage quand le geste touche à une désignation
     (archivage, retypage, `postable`, rôle `DefaultRevenue`, **devise posée**), puis `accounts` `FOR UPDATE` : une lecture simple en tête figerait l'instantané REPEATABLE READ avant l'attente
     (AC-3). ⛔ **`company_id` ne se lit pas `LOCK IN SHARE MODE` sur la ligne du compte** : ce verrou partagé suivi du
     `FOR UPDATE` de la même ligne est une promotion S → X, et deux gestes concurrents sur le même compte
     (deux archivages, un `PUT` et un archivage) s'interbloquent (1213), chacun tenant le partage que l'autre attend
     (la même promotion est décrite à `company_invoice_settings.rs:385-387`). `company_id` est **immuable** : il se lit
     **sur le pool, avant `begin()`** (une lecture hors transaction ne fige pas l'instantané de la transaction ; la
     doc d'`archive_guarded` veut que la société soit celle du compte, lue par la fonction, revue P1 A L-5 — l'appelant
     ne la passe donc pas), puis la transaction prend `companies` en partage, les réglages, le compte `FOR UPDATE`
     et **compare** le `company_id` relu sous verrou à celui de la lecture hors transaction (écart → `NotFound`).
     Le test d'existence de ligne d'écriture se fait **après** le `FOR UPDATE` du compte, en lecture
     verrouillante. La 35-3a lit la devise des comptes `LOCK IN SHARE MODE`
     (`validate_lines_accounts_in_tx` :135 lit aujourd'hui sans verrou). **Les sites de configuration** : les
     validateurs actuels lisent le compte **hors transaction**, sur le pool — `validate_account_of`
     (`routes/company_invoice_settings.rs:211`, `accounts::find_by_id(&state.pool, …)`), `validate_revenue_account`
     (`routes/products.rs:351`), `validate_counterparty_account` (`routes/reconciliation_rules.rs:215`) : un `LOCK IN
     SHARE MODE` y serait relâché aussitôt. Les refus `ACCOUNT_IN_FOREIGN_CURRENCY` et `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`
     de ces sites se font donc **dans la transaction d'écriture du dépôt** : `company_invoice_settings::update` (compte
     changé lu `LOCK IN SHARE MODE` après le `FOR UPDATE` des réglages) ; `products::create` / `update` et
     `reconciliation_rules::create_in_tx` / `update_in_tx` (compte lu `LOCK IN SHARE MODE` **avant** tout verrou ou
     écriture de la ligne d'article ou de règle) ; les routes bancaires par `type_in_share_mode` (34-11a2, `currency`
     ajouté à son `SELECT`) ; **l'étape bancaire de l'assistant** (`routes/onboarding.rs:659-696`) lie un compte par
     `validate_journal_account_id` sur le pool puis `upsert_primary_in_tx`, sans relecture verrouillante : le refus
     `ACCOUNT_IN_FOREIGN_CURRENCY` y est posé dans `validate_journal_account_id` (pré-vol, partagé avec les routes
     bancaires), et la course est une **limite assumée** écrite au Change Log (la société d'un assistant en cours
     n'a ni exercice ni compte en devise : la remise à zéro en repart du stub). Côté `update` d'un compte (devise ou rôle posé) : réglages lus en partage **avant** le
     `FOR UPDATE` du compte (ci-dessus — les lire après formerait un cycle avec `company_invoice_settings::update`, qui
     tient les réglages en `FOR UPDATE` puis lit le compte), puis, **sous** le `FOR UPDATE`, `bank_accounts`,
     `products` et `reconciliation_rules` en lecture verrouillante — de sorte que de deux gestes concurrents l'un
     attend l'autre et voit sa valeur commitée, sans cycle. L'ordre de pose de ces verrous par
     site est écrit au Change Log. Un test de course par paire : désactivation contre création en devise, désignation
     contre archivage/retypage, changement de devise contre première ligne, **changement de devise contre
     désignation (réglages de facturation) et contre lien à un compte bancaire**. ⚠️ Toute ligne d'écriture de ces
     tests (et de l'E2E) se passe sur un compte **encore CHF**, puis on tente de lui donner une devise : une ligne CHF
     sur un compte déjà en devise, acceptée tant que la 35-3a n'est pas là, sera refusée par son contrôle central
     (`ACCOUNT_REQUIRES_CURRENCY`) et casserait le test ;
   - un compte de groupe (non imputable) peut porter une devise (informatif) ; un enfant n'hérite de rien.
5. **AC-5 — Réinitialisation de la société.** Seul chemin qui vide les comptes en gardant la société :
   `kesh_seed::reset_demo` (`onboarding/reset`) — `accounts` est dans `reset_cleared_tables`, `companies` est
   préservée (`backup.rs:136-144`) et remise à zéro en place par `companies::reset_to_stub_in_tx`
   (`companies.rs:214`). Aucun autre `DELETE FROM accounts` hors tests (grep vérifié) ; l'import complet remplace
   `companies` entière. `reset_to_stub_in_tx` écrit `multi_currency_enabled = FALSE, fx_gain_account_id = NULL,
   fx_loss_account_id = NULL, fx_transit_account_id = NULL` (les exercices, applicatifs, sont vidés puis recréés
   par l'assistant). Test : les quatre colonnes entrent dans `PERTURBED` (**pas** dans `NOT_COMPARED`) de
   `reset_restores_the_stub_columns_in_place` (`onboarding_audit_e2e.rs:2342`) et sont perturbées par son `UPDATE`
   (:2344).
6. **AC-6 — Sauvegarde.** Un `.keshbackup` écrit par la 0.17.6 se réimporte (colonnes à leur défaut) — **et le
   reste dans la release** (R1, CR-2 : l'annotation CR-1 qui le bornait à cette story est caduque). **Liste close des
   absences admises** (R1) : une sauvegarde antérieure à l'Epic 35 est forcément mono-devise ; l'import tolère
   l'absence des **seules** tables et colonnes ajoutées par l'Epic 35, déclarées dans une constante de
   `kesh-db::backup` (nom au développement, p. ex. `ABSENCES_ADMISES_EPIC_35: &[Absence]`, `Absence::Table(&str)` |
   `Absence::Colonne(&str, &str)`, chaque entrée avec la version de la migration qui l'ajoute). Cette story crée le
   mécanisme et y inscrit ses colonnes (`companies` × 4, `accounts.currency`, `fiscal_years` × 3) ; 35-2 y inscrit la
   table `exchange_rate_overrides`, 35-3a les quatre colonnes de `journal_entry_lines`. Au code :
   - **tables** : la boucle de couverture de `parse_and_verify_avec_inventaire`
     (`crates/kesh-api/src/admin_backup/import.rs:250-253`, « table '…' absente du manifeste ») saute une table de la
     liste absente du manifeste ; toute autre table absente reste refusée (400, test conservé). La restauration la vide
     sans rien y insérer (`restore_body`, `backup.rs:796-803`, saute déjà une table absente de la carte après son
     `DELETE`) : elle est « traitée comme vide ». La réciproque (table hors inventaire → 400) ne change pas ;
   - **fenêtre d'import** : `creates_application_table` (`crates/kesh-db/src/post_restore.rs`, appelée par
     `last_table_creating_migration`) **ignore** les tables de la liste — une table que l'import tolère absente ne
     referme pas la fenêtre ; `registry_entries_are_within_import_window` reste vert, et le registre de rejeu
     (`POST_RESTORE_BACKFILLS`, entrée `20261106000001`) **n'est pas touché** par l'epic ;
   - **colonnes** : déjà tolérées par la règle générique — `missing_required_columns` (`import.rs:625`) ne refuse
     qu'une colonne `is_required()` (ni `NULL` ni `DEFAULT`) ; toutes les colonnes de l'epic en ont un. La liste les
     nomme pour les **garder** : un test vérifie que chaque colonne listée existe au schéma et n'est **pas**
     `is_required()` (une future modification en `NOT NULL` sans défaut rougit), que chaque table listée est dans
     `TABLES_TO_TRUNCATE`, et que chaque entrée est ajoutée par une migration de l'epic (version ≥ `20261114000001`,
     `MIGRATOR`, dont le SQL crée la table ou ajoute la colonne) — la liste ne peut pas servir à autre chose ;
   - **test d'import d'une sauvegarde 0.17.6** (`admin_full_import_e2e.rs`, sur le patron des sauvegardes 0.17.4 et
     0.17.5 de ce fichier, `:5563`, `:5638`, simulées par l'export courant retouché) : export d'une société
     mono-devise, puis retrait des colonnes et des tables de la liste au manifeste et aux NDJSON, `keshVersion` 0.17.6 →
     import 200, réglage et exercices à leur défaut (mono-devise), comptes sans devise (35-2 étend le test à
     `exchange_rate_overrides` absente — aucun cours saisi —, 35-3a aux colonnes des lignes — aucune ligne en devise) ;
     et une table **hors liste** retirée → 400 (le refus d'origine tient). La phase 5 de
     `scripts/recette-montee.sh` (import du `.keshbackup` de l'image 0.17.6, exige un 200, `:651`) se joue normalement
     à la release ; son `--diff-attendu` y liste `migrations` (toujours, `_sqlx_migrations`), `companies`, `accounts`
     et `fiscal_years`, plus les tables des autres stories (l'empreinte lit
     `SELECT *` : les colonnes neuves sont un écart attendu, `scripts/recette-montee.sh:414-440`).
   L'aller-retour d'une sauvegarde 35-1 conserve réglage, devises et mode des exercices ; l'export CSV des comptes (`serialize_accounts_csv`,
   `csv_tables.rs:265`, en-tête écrit à la main) porte `currency` ; `company.csv` (`serialize_company_csv`, :163)
   porte les quatre colonnes ; l'export CSV des exercices (`serialize_fiscal_years_csv`, `csv_tables.rs:230`, écrit à
   la main ; `exports/global.rs:124`) porte les trois colonnes d'exercice, avec son cas de test (le test qui compare
   les en-têtes au schéma rougit sinon). Le `.keshbackup` lui-même n'énumère rien à la main (`export_table` lit les colonnes à
   `information_schema`) : les colonnes neuves y passent seules, le test d'aller-retour le prouve. Le test qui
   compare les en-têtes des CSV aux colonnes du schéma (`csv_tables.rs`, ~:2395, `entete(|w| serialize_*_csv(&[], w))`)
   rougit tant que les deux sérialiseurs ne suivent pas ; ses tests unitaires (`serialize_accounts_csv_*`,
   `serialize_company_csv_*`) portent un cas avec devise et avec réglage actif.
7. **AC-7 — Écrans.**
   - *Paramètres → Comptabilité* : une case « Comptabilité multi-devises » et, cochée, trois sélecteurs (compte de gains
     de change, compte de pertes de change, compte de conversion — comptes de bilan pour ce dernier ; mêmes filtres que
     l'AC-3), enregistrer, erreurs affichées ; le texte dit que la devise principale reste le CHF, que le réglage est le
     défaut des exercices créés et passe les exercices ouverts en multi-devises (AC-3), qu'une conversion entre deux
     devises passe par le compte de conversion (R7), et qu'on ne revient en mono-devise que sans compte en devise. Section éditable
     pour `isAdmin` seulement (`settings/+page.svelte:228`), lecture seule pour les autres rôles.
   - *Plan comptable* : en multi-devises, un champ « Devise » (sélecteur ISO 4217, vide = CHF) à la création et à la
     modification, désactivé d'avance avec l'explication si `currencyLocked` ; une colonne « Devise » dans la liste
     (vide pour CHF). En mono-devise : **aucun** changement visible.
   - Captures d'écran des états touchés (recette visuelle, thème clair), hors du dépôt.
8. **AC-8 — MCP et services.** `kesh_plan_comptable` donne la devise d'un compte. Le service
   (`services/plan_comptable.rs`) rend déjà l'`Account` entier : le changement est dans la sortie de l'outil,
   `sortie! struct Compte` (`crates/kesh-api/src/mcp/outils/plan_comptable.rs:31-43`), champ `devise` (noms français
   de la sortie) de type `Option<…>` admis par `sortie!` — `TexteNonFiable` ou un type scellé dédié, **jamais
   `String`, qui ne compile pas** (`mcp/sortie.rs:92`) ; `null` pour le CHF (la macro ne pose aucun
   `skip_serializing_if` : la clé est toujours présente). La description de l'outil mentionne la devise. Les tailles
   mesurées (`LIMITE_DEFAUT` 40, `LIMITE_MAX_MCP` 70 ; `mcp_outils_e2e.rs:183`) sont re-mesurées avec le champ
   (précédent C-e331-27) ; le test de contrat du protocole MCP de la CI est relu. Test de l'outil avec un compte en
   devise et un compte CHF.
9. **AC-9 — i18n.** Toutes les chaînes neuves dans les quatre locales (`fr-CH`, `de-CH`, `it-CH`, `en-CH`),
   `lint-i18n-ownership` vert.
10. **AC-10 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) : section « Comptabilité multi-devises » —
    activer, comptes de différences de change, devise d'un compte et son verrou — **dit que le verrou est définitif**
    (une écriture sur un compte en devise ne se défait que par sa suppression ; tant qu'un compte porte une devise,
    archivés compris, on ne revient pas en mono-devise) ; PDF régénéré et contrôlé
    (`pdftotext | tr`). Il dit aussi le **mode par exercice** (R3 : défaut, bascule, refus tant qu'un compte en devise
    n'est pas à zéro, archivage proposé), le **compte de conversion** (R7 : un compte de bilan en CHF à créer, p. ex.
    1090, qui revient à 0 après une conversion) et la réimportation des sauvegardes antérieures (R1, mono-devise). Aucun plan livré n'a de compte « Différences de change » (`pme.json:76-78` : 6900, 6940,
    6950) : le manuel dit quoi choisir ou créer (p. ex. 6840, **sous le groupe 6** et non sous 6800 *Amortissements*,
    compte imputable que la création d'un sous-compte rendrait non imputable, `accounts::create` :197-207). Si le manuel cite la route
    `PUT /api/v1/companies/current/currency-settings`, ajouter l'entrée à `MULTI_SOCIETE_ASSUME` de
    `textes_coherents.rs` (motif `\bcompanies\b`, précédent :1014-1017).
11. **AC-11 — Début de l'exercice initial (R10).** L'assistant crée l'exercice initial au 1er janvier de l'année
    courante (`routes/onboarding.rs:1044-1046`, `from_ymd_opt(year, 1, 1)` et `(year, 12, 31)` écrits en ligne). Ces
    dates deviennent des **constantes nommées** du module (nom au développement, p. ex.
    `EXERCICE_INITIAL_DEBUT: (u32, u32) = (1, 1)` et `EXERCICE_INITIAL_FIN: (u32, u32) = (12, 31)`), avec le
    commentaire de l'arbitrage A1 du fichier d'epic (« Ne pas modifier : l'exercice initial créé par l'assistant
    commence le 1er janvier ; un exercice à d'autres dates se crée dans *Exercices* », soumis à Beat ; sa réponse, si
    elle le change, au Change Log) ; le comportement ne change
    pas (test existant de l'assistant vert). Kesh accepte déjà des exercices à dates libres (`POST /fiscal-years`) :
    rien d'autre à implémenter ; 35-2 et 35-5 se testent sur des exercices au 1er juillet et au 1er avril (R6).

## Tâches

- [ ] Migration + `migrations.sha384` + audit d'idempotence + squash (AC-1).
- [ ] `kesh-core` : module `currency` — liste ISO 4217 (codes actifs, trois lettres), `CurrencyCode` validé,
      `BASE_CURRENCY = "CHF"` ; tests (AC-4).
- [ ] `kesh-db` : `Company` + **les quatre listes de colonnes écrites à la main** qui la construisent par
      `query_as::<_, Company>` — `FIND_BY_ID_SQL`/`LIST_SQL` (`companies.rs:24,29`), la macro `company_select!` de
      `routes/onboarding.rs:1273-1287` (`COMPANY_SELECT_FOR_UPDATE`, `COMPANY_BY_ID_FOR_UPDATE`, lues :1292, :1376,
      :1482) et la requête de `kesh-seed/src/lib.rs:210-214` (`seed_demo`). ⚠️ `FromRow` : une colonne manquante
      n'échoue **pas** à la compilation mais à l'exécution (assistant et chargement de la démonstration en 500) ;
      les quatre colonnes de `companies` de l'AC-1 entrent dans les quatre ; `Account`/`NewAccount`/
      `AccountUpdate` + macro `columns!` ; `companies::update_currency_settings` (gardes C-35-2, verrous AC-3) ;
      `accounts::create/update/archive_guarded/reactivate` (verrous, gardes AC-4) ; `is_no_op_change` (:530-537) et
      `account_snapshot_json` (:57) avec `currency` ; `bulk_create` (:1146) et `bulk_create_from_chart_in_tx` (:1256)
      lient `currency` (leurs `INSERT` l'ignoreraient sinon en silence ; plans livrés et assistant : `NULL`) ;
      sites de construction inventoriés par l'échec de compilation (`NewAccount` : une cinquantaine de littéraux
      `NewAccount {` (52 occurrences du motif, définition comprise), dans les tests de `kesh-db` et de `kesh-api`
      surtout, hors `NewAccount::new` qui prend `currency: None` ; `AccountUpdate` : deux sites de production —
      `routes/accounts.rs:315` et la normalisation de `accounts::update` (`accounts.rs:663`, `..changes` : elle
      recopie `currency` sans erreur de compilation, à garder ainsi) — et vingt-quatre littéraux de tests (vingt et un dans
      `accounts.rs`, un dans `credit_card_repository.rs`, `letterings.rs` et `journal_entry_reversal_e2e.rs`) ;
      `kesh-seed` n'en construit aucun, vérifié) ;
      gardes croisées de l'AC-4 dans les transactions du dépôt des réglages de facturation, comptes bancaires,
      articles et règles de réconciliation (les colonnes de pièce, factures fournisseur comprises, sont à la 35-3a) ; `reset_to_stub_in_tx` (AC-5) ; `test_fixtures.rs`, seed.
- [ ] `kesh-api` : `CompanyJson` (`routes/companies.rs:31`, miroir écrit à la main) ; `serialize_accounts_csv` et
      `serialize_company_csv` (AC-6).
- [ ] `kesh-api` : routes, DTO, codes d'erreur (`errors.rs` + i18n), audit (`audit_labels.rs`), MCP (AC-2 à AC-8).
- [ ] Frontend : `settings` (section Comptabilité), `accounts` (champ, colonne), types, API, Vitest (AC-7).
- [ ] `kesh-api` : registre des routes d'audit et phrase du manuel d'administration (AC-3).
- [ ] Manuel (AC-10), CHANGELOG `[Unreleased]` (nouveau champ `currency` du compte, absent = inchangé ; réglage
      réservé à l'Administrateur, hors clé d'API).
- [ ] Tests : dépôt (gardes, verrou, reset), API (codes), sauvegarde (AC-6), E2E Playwright (créer d'abord un compte de
      bilan pour la conversion — aucun plan livré n'en a, AC-3 —, activer le mode avec les trois comptes, créer un
      compte USD, verrou : écriture passée par l'API sur un compte **CHF**, puis devise refusée à l'écran — AC-4 ;
      écran *Exercices* : bascule en mono-devise, dialogue d'archivage puis renvoi avec `force` — AC-3 bis).
- [ ] CR-2 — `kesh-db` : `FiscalYear` (`entities/fiscal_year.rs:73`) gagne `multi_currency`, `opening_rate_kind`,
      `closing_rate_kind` (énumération `RateKindChoice`/nom au développement) ; **quinze listes de colonnes écrites à la
      main** à compléter (`FromRow` échoue à l'exécution, non à la compilation) — quatorze constantes et requêtes de
      `repositories/fiscal_years.rs` (`FIND_BY_ID_SQL` :145, `LIST_BY_COMPANY_SQL` :150, `LOCK_IN_COMPANY_SQL` :162,
      `EARLIER_OPEN_SQL` :186, `NEXT_AFTER_SQL` :207, `OPEN_COVERING_DATE_SQL` :731, `FIND_LATER_CLOSED_SQL` :890…,
      recomptées par `grep -rn "SELECT id, company_id, name, start_date, end_date, status" crates --include=*.rs` : 14 + 1 de
      production et la copie de test de `kesh-db/tests/reversal_next_fiscal_year.rs:554` (texte de verrou, sans
      `FromRow`) ; le motif sans `SELECT` en rend 21, avec cinq `INSERT` de `kesh-report/tests`) et une de
      `repositories/closing_checks.rs` ; `snapshot_json` des exercices ; les trois chemins de création (AC-3 bis) ;
      `fiscal_years::set_currency_mode` (verrous, refus, archivage sous `force`) ; `update_currency_settings` étendu
      (compte de conversion, exercices à l'activation et à la désactivation) ; liste close des absences admises et son
      emploi dans `parse_and_verify_avec_inventaire` et `creates_application_table` (AC-6).
- [ ] CR-2 — tests manquants (35-1-CR2-P1-3) : activation → exercices **ouverts** multi-devises, clos inchangés,
      identifiants au détail d'audit ; désactivation → tous mono-devise, clos compris ; compte de conversion :
      `reason: "accountType"` pour `Expense`/`Revenue`, `details.designation = "transit"` à l'archivage, au retypage, à
      la devise et au sous-compte ; refus de l'activation sans l'un des trois comptes ; archivage sous `force`
      (imputables seuls, groupe laissé, `archive_in_tx` partagé avec `archive_guarded`) ; rejeu sur interblocage des
      deux routes (AC-3).
- [ ] CR-2 — `kesh-api` : route `PUT /fiscal-years/{id}/currency-mode`, codes `FISCAL_YEAR_HAS_FOREIGN_BALANCES` et
      `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE` (409, quatre locales), audit `fiscal_year.currency_mode_updated`,
      `FiscalYearResponse` ; constantes de l'assistant (AC-11) ; frontend *Exercices* (colonne, action, dialogue
      d'archivage) et troisième sélecteur des paramètres ; test d'import d'une sauvegarde 0.17.6 (AC-6).

## Notes de développement

- Comptes : `crates/kesh-db/src/repositories/accounts.rs` (`COLUMNS` :34, `create` :153, `update` :622, garde
  `AccountHasEntries` :726-735, `archive_guarded` :859, `reactivate` :1026) ; `crates/kesh-api/src/routes/accounts.rs`
  (`CreateAccountRequest` :36, `present_or_null` :60, `UpdateAccountRequest` :77, `AccountResponse` :139).
- Société : `crates/kesh-db/src/entities/company.rs` (`Company` :137), `repositories/companies.rs` (`update_in_tx`
  :842, `reset_to_stub_in_tx` :214) ; `routes/companies.rs` (`get_current` :448, `PUT` par champ :112/:250/:344 —
  suivre leur forme ; :497 est `lock_company_books`, un `POST` ouvert au Comptable, pas un précédent de réglage) ;
  écran `frontend/src/routes/(app)/settings/+page.svelte` (section Comptabilité :881-890).
- Précédent de garde d'archivage d'un compte désigné : #486 (Story 33-8, `compte-designe-archive`) — confirmable, à
  la différence du refus gain/perte de l'AC-4.
- Marqueur de plan comptable pour un compte de différences de change (sur le modèle de `roundingDifference`) : hors
  périmètre, à noter au registre pour plus tard.
- La liste ISO 4217 est **embarquée** (pas de dépendance réseau) ; une devise absente des cours AFC reste admise (cours
  manuel, 35-3). Elle ne porte que des **monnaies** : les codes réservés (`XXX`, `XTS`), les métaux (`XAU`, `XAG`,
  `XPT`, `XPD`), les droits de tirage (`XDR`) et les unités de fonds en sont exclus ; `CHF` y figure (normalisé à
  `NULL` par l'AC-4).
- `CHECK` : collation binaire de la colonne **et** `BINARY` dans les `CHECK` (AC-1) — `20261110000001` ne met
  `BINARY` que sur son `IN`, son `REGEXP` ne porte que sur des chiffres : ne pas le recopier tel quel.

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Opus) : 0 CRITICAL, 2 HIGH, 9 MEDIUM, 5 LOW ; tous remédiés.
- 2026-11-14 — validation P2 (Sonnet) : 0 CRITICAL, 1 HIGH, 5 MEDIUM, 4 LOW dans la fiche, plus 3 inter-fiches (1 MEDIUM,
  2 LOW) ; remédiés dans la fiche sauf IF-1 (35-3a, `FOR SHARE` → `LOCK IN SHARE MODE`), IF-2 (35-3a AC-5, « ici »)
  et IF-3 (35-2, décompte du registre des routes). Les cinq choix de conception de cette passe — `currency` absent d'un
  `PUT` de compte = inchangé (et non 400, P2-4) ; lectures verrouillantes avant tout `COUNT` (P2-1) ; verrous des
  sites de configuration (P2-2) ; ordre des refus (P2-5) ; liste des comptes en devise dans `MULTI_CURRENCY_IN_USE`
  (P2-6) — sont de l'orchestrateur de la passe, à confirmer par la P3.
- 2026-11-14 — validation P3 (Opus) : 0 CRITICAL, 0 HIGH, 8 MEDIUM, 7 LOW ; remédiés dans la fiche sauf IF-1 (35-5,
  revalidation des comptes de différences de change : filet de la désignation « produit par défaut », LOW). Choix
  P2 confirmés : P2-4, P2-1, P2-6 ; P2-2 corrigé (P3-1 : validateurs hors transaction ; P3-2 : place de la lecture des
  réglages dans `update`, sans quoi cycle avec `company_invoice_settings::update`) ; P2-5 complété (P3-5). Décisions de
  cette passe : désignation « produit par défaut » d'un compte gain/perte refusée des deux côtés (P3-8) ; liens d'un
  objet archivé comptés (P3-12) ; sortie MCP `devise` toujours présente, `null` pour le CHF (P3-4).
- 2026-11-14 — validation P4 (Sonnet) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW dans la fiche, plus 2 inter-fiches
  (2 MEDIUM) ; remédiés dans la fiche sauf IF-1 (35-2, second test du registre des routes : `sans_ecriture` +1,
  `exemptees` +1, `tout.len()` +2) et IF-2 (35-5, idem : `sans_ecriture` +1, `rejouees` +1, `tout.len()` +2).
  Points : second test de partition du registre (`the_replay_partition_…`, quatrième champ `SansEcritureAuJournal`)
  oublié (P4-1) ; lecture verrouillante de `company_id` = promotion S → X et interblocage entre deux gestes sur le
  même compte, remplacée par une lecture hors transaction (immuable) comparée sous verrou (P4-2, choix de la
  passe, à confirmer par la suivante) ; `"CHF"` en mono-devise (P4-3) ; réponse du `PUT` et test 403 (P4-4) ; sites
  de construction de `NewAccount`/`AccountUpdate` (P4-5) ; chemins des renvois `company_invoice_settings.rs` (P4-6) ;
  liste close des `reason` de `FX_ACCOUNT_INVALID` (P4-7) ; étape bancaire de l'assistant (P4-8, limite assumée).
- 2026-11-14 — validation P5 (Opus) : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW dans la fiche, plus 1 inter-fiches (LOW) ;
  remédiés dans la fiche sauf 35-1-P5-IF1 (35-3b, type TS `currency?` d'`AccountResponse`). Points : listes de
  colonnes de `Company` écrites à la main hors `companies.rs` (onboarding, `kesh-seed`), échec à l'exécution et non
  à la compilation (P5-1) ; `AccountResponse` aligné sur C-35-9, clés omises pour un compte CHF (P5-2) ; sites de
  construction de `NewAccount`/`AccountUpdate` recomptés (P5-3) ; pré-vol 404 de la route en tête de l'ordre des
  refus de `update` (P5-4) ; nom de l'index des lignes (P5-5). Choix P4-2 (`company_id` lu sur le pool puis
  comparé sous verrou) confirmé.
- 2026-11-14 — validation P6 ciblée (Sonnet) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW (P6-1 : littéraux de tests
  d'`AccountUpdate` recomptés, 21 dans `accounts.rs` et non 20, soit 24 et non 23 ; remédié) ; 0 inter-fiches.
  Vérifiés au code : `idx_jel_account`, `routes/accounts.rs:290`/`:315`, `accounts.rs:663`, les quatre listes de colonnes
  de `Company`, 52 `NewAccount {`.
- 2026-11-14 — révision CR-1 (D9, D10, terminologie ; Opus) : AC-6 annotée — la réimportation d'une sauvegarde 0.17.6
  ne vaut que jusqu'à 35-2, dont la table applicative `exchange_rate_overrides` referme la fenêtre d'import et qui
  inverse ce test en refus. Aucun autre AC touché.
- 2026-11-14 — révision CR-2 (revue de Beat R1–R10 ; Opus) : **R1** — une sauvegarde antérieure à l'Epic 35 se
  réimporte (mono-devise) : liste close des tables et colonnes de l'epic dans `kesh-db::backup`, sautée par la boucle de
  couverture de `parse_and_verify_avec_inventaire` et ignorée par `creates_application_table` (la fenêtre d'import ne se
  referme pas, le registre de rejeu reste tel quel), colonnes gardées non `is_required()` par un test, test d'import
  d'une sauvegarde 0.17.6 ; l'annotation CR-1 de l'AC-6 est caduque. **R3** — mode par exercice :
  `fiscal_years.multi_currency` (défaut tiré de la société par les trois chemins de création inventoriés, exercice
  initial de l'assistant compris), activation qui passe les exercices ouverts en multi-devises, désactivation qui les
  passe tous en mono-devise, route `PUT /fiscal-years/{id}/currency-mode` (refus `FISCAL_YEAR_HAS_FOREIGN_BALANCES`,
  avertissement `FISCAL_YEAR_FOREIGN_ACCOUNTS_ACTIVE` et archivage sous `force`), écran *Exercices*. **R6** (colonnes
  seulement) — `opening_rate_kind`, `closing_rate_kind` sur `fiscal_years`, dans **cette** migration (décision : une
  seule retouche de `FiscalYear` et de ses quinze listes de colonnes ; route et écran en 35-2). **R7** — compte de
  conversion `fx_transit_account_id` (bilan, CHF, mêmes gardes que gains/pertes sauf le type, obligatoire pour
  activer, `details.designation`). **R10** — constantes de l'exercice initial de l'assistant (AC-11). Registre des
  routes : +2 routes. Prémisses relues au code : `import.rs:250-253`, `:625` ; `backup.rs:796-803` ;
  `post_restore.rs:939`, `:968`, `:988` ; `fiscal_years.rs:331`, `:412`, `:456`, `:470`, `:1360`, listes de colonnes ;
  `journal_entries.rs:332-336` ; `onboarding.rs:1044-1046`, `:1053` ; `routes/fiscal_years.rs:51`, `:286` ;
  `audit_route_registry.rs:537-539` ; `csv_tables.rs:230`. AC touchés : en-tête, Décisions, AC-1, AC-2, AC-3, AC-3 bis
  (neuf), AC-4 (compte de conversion), AC-5, AC-6, AC-7, AC-10, AC-11 (neuf), Tâches. Fiche à revalider (cycle CR-2).
- 2026-11-14 — validation CR-2 P1 (Sonnet) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 2 LOW ; remédiés dans la fiche, aucun
  inter-fiches. CR2-P1-1 : cycle exercice → société des écritures contre l'activation (`companies` X puis exercices),
  rejeu des deux routes et registre (vi) ; CR2-P1-2 : archivage sous `force` (`archive_in_tx` extrait, imputables
  seuls) ; CR2-P1-3 : tests d'activation, de désactivation et du compte de conversion ; CR2-P1-4 : grep des listes de
  colonnes d'exercice (21 et non 15) ; CR2-P1-5 : `--diff-attendu` de la recette de montée. Prémisses relues au code :
  `import.rs:250-253`, `:625` ; `backup.rs:796-803` ; `post_restore.rs:968-1004` ; `fiscal_years.rs:331`, `:456`,
  `:1360` ; `journal_entries.rs:326-336` ; `accounts.rs:859` ; `audit_route_registry.rs` (vi) ; `onboarding.rs:1044`.
- 2026-11-14 — validation CR-2 P2 (Opus) : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 8 LOW dans la fiche, plus 1 inter-fiches
  (LOW) ; remédiés dans la fiche sauf 35-1-CR2-P2-IF1 (fichier d'epic, arbitrage A6 : « suggestion 1091 » contre 1090
  des fiches 35-1 et 35-3b ; texte proposé : « suggestion 1090 »).
  Points : ordre 400 (corps) avant 404 (extracteur `Json`) de `currency-mode` (P2-1) ; cycles couverts par le rejeu
  — réglages `FOR UPDATE` avant l'exercice des validations de facture, avoir, saisie fournisseur, et cycle propre à
  `currency-mode` sous `force` (P2-2) ; commentaire de la constante repris de l'arbitrage A1 (P2-3) ; R6 à l'en-tête
  et aux Décisions (P2-4) ; revalidation du compte de conversion par 35-3b seule (P2-5) ; `migrations` au
  `--diff-attendu` de la recette (P2-6) ; E2E : compte de bilan à créer avant l'activation, bascule d'exercice
  (P2-7) ; contrat d'`archive_in_tx` (société comparée, version, verrous pris une fois, P2-8). Prémisses relues au
  code : `routes/fiscal_years.rs:286`, `:385-389` (`reopen`, extracteur `Json`) ; `accounts.rs:440-448`, `:859-946`
  (`archive_guarded` : aucun contrôle de solde) ; `fiscal_years.rs:162`, `:178`, `:331`, `:412`, `:456`, `:470`,
  `:1360`, 14 + 1 listes de colonnes ; `journal_entries.rs:281-336` ; `invoices.rs:698`, `:1974` ;
  `credit_notes.rs:312` ; `supplier_invoices.rs:1484` ; `post_restore.rs:938-1004` (fonctions de test) ;
  `audit_route_registry.rs:155`, `:302-308`, `:1450-1502` ; `rejeu_interblocage_e2e.rs:2497` ;
  `admin_pat_denied_e2e.rs:85` ; `onboarding.rs:1043-1053`, `:1273-1287` ; `scripts/recette-montee.sh:414-453`,
  `:651` ; plans livrés (aucun 109x). Axes non exercés : source journalière (aucun GET, la 35-1 n'y touche pas),
  manuels (PDF non lu), i18n au code.
