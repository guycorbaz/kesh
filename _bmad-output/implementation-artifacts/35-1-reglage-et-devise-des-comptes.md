# Story 35-1 : réglage multi-devises de la société et devise des comptes

Status: ready-for-dev

*Fiche rédigée le 2026-11-14 (Opus 5.5), branche `epic/35-multi-devises`, numéros de ligne de `67efc51f`. Epic 35
(`_bmad-output/planning-artifacts/epic-35-multi-devises.md`), registre C-35-1 à C-35-8. **Migration non breaking**.*

## Story

En tant que comptable d'une société qui encaisse et paie en USD et en EUR, je veux passer ma comptabilité en mode
multi-devises, désigner mes comptes de différences de change et donner une devise à certains comptes du plan, afin d'y
passer ensuite des écritures en devise (35-3), sans rien changer pour qui reste en mono-devise.

## Décisions qui s'appliquent

D1, D2 (fichier d'epic) ; C-35-2 (réglage sur `companies`, sans FK, quatre gardes), C-35-5 (verrous).

## Critères d'acceptation

1. **AC-1 — Migration** `20261114000001_multi_currency_settings.sql`, non breaking (DDL seul, aucune écriture de
   données) :
   - `companies` : `multi_currency_enabled BOOLEAN NOT NULL DEFAULT FALSE`, `fx_gain_account_id BIGINT NULL`,
     `fx_loss_account_id BIGINT NULL`, **sans clé étrangère** (commentaire SQL qui dit pourquoi : cycle
     `companies` ↔ `accounts`, réinitialisation ; intégrité par le dépôt, C-35-2) ;
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
   - Test de montée sur base peuplée (`migrations_upgrade_path.rs`, précédent de la 34-11a1) : comptes et société
     existants prennent les défauts.
2. **AC-2 — Lecture du réglage.** `GET /api/v1/companies/current` expose `multiCurrencyEnabled`, `fxGainAccountId`,
   `fxLossAccountId`, `baseCurrency: "CHF"` (constante, C-35-2 ; la devise principale ne change pas).
3. **AC-3 — Écriture du réglage.** `PUT /api/v1/companies/current/currency-settings`
   `{ multiCurrencyEnabled, fxGainAccountId, fxLossAccountId, version }` (verrou optimiste comme les autres `PUT` de
   la société) :
   - **Administrateur seul**, comme les trois `PUT /companies/current/*` et `PUT /company/invoice-settings`
     (`lib.rs:277-278, 446-461`) : route dans `admin_routes`, **au-dessus** du marqueur « TOUTE ROUTE S'AJOUTE
     AU-DESSUS DE CETTE LIGNE » (`require_admin_role` + `require_not_pat`) ; une clé d'API reçoit
     `API_KEY_ADMIN_FORBIDDEN` ; la route s'ajoute à la liste de `admin_pat_denied_e2e.rs` (:85). **Conséquence
     assumée** : la recette par l'API (clé d'API) ne peut pas activer le mode multi-devises ; il s'active à l'écran,
     par un Administrateur ;
   - fonction dédiée `companies::update_currency_settings` (ne passe **pas** par `CompanyUpdate`/`update_in_tx`, qui
     sont en remplacement complet : chaque route e-mail, coordonnées, adresse et `company_update_of` de l'assistant
     devrait sinon recopier trois champs de plus) ;
   - activer exige les deux comptes (le même compte est admis pour les deux) ; tant que le mode est actif, aucun des
     deux ne peut être vidé ;
   - chaque compte : de la société, imputable, actif, de type `Expense` ou `Revenue`, **sans devise** ; **ni** au rôle
     `DefaultRevenue`, **ni** désigné `default_revenue_account_id` (réglages de facturation ou articles) — les gains de
     change ne vont pas au chiffre d'affaires (CO 959b) ; lu `FOR UPDATE` ; sinon 400 `FX_ACCOUNT_INVALID` (champ et
     `reason` en `details`, p. ex. `reason: "revenueAccount"`). Les autres désignations (arrondi 6940, frais bancaires
     6900) restent admises. **Motif C4** du précédent (`validate_account_of`, `company_invoice_settings.rs:219,231`) :
     ces contrôles ne portent que sur un compte **changé** ; un compte inchangé devenu invalide n'empêche pas
     d'enregistrer l'autre (la revalidation à l'usage, C-35-2 (3), reste le filet) ;
   - désactiver est refusé (409 `MULTI_CURRENCY_IN_USE`) tant qu'un compte porte une devise, **comptes archivés
     compris** ; désactiver remet les deux comptes à `NULL` ;
   - **verrous, dans l'ordre global du dépôt** (`companies` → réglages → `accounts` → `bank_accounts`,
     `accounts.rs:446-448`) : ligne `companies` `FOR UPDATE` (version jugée dessus, et non sur une lecture sans verrou
     comme `update_in_tx` :849), puis les comptes désignés `FOR UPDATE`, puis le `COUNT` des comptes en devise ;
   - audit `company.currency_settings_updated` (avant/après) ; libellés d'audit dans les quatre locales.
4. **AC-4 — Devise d'un compte.** `CreateAccountRequest.currency` est facultatif (`null` ou absent = CHF).
   `UpdateAccountRequest.currency` est `Option<Option<String>>` désérialisé par `present_or_null`, **comme `role`**
   (contrat full-replace, `routes/accounts.rs:69-76`) : **absent → 400**, `null` = CHF, valeur = devise. Code ISO
   4217 à trois lettres, mis en majuscules après `trim` avant validation. `AccountResponse` expose `currency` et
   `currencyLocked: bool` (le compte porte au moins une ligne d'écriture ; calculé pour la liste en **une** requête,
   `SELECT DISTINCT account_id FROM journal_entry_lines …`, jamais une par compte).
   - refus **409** `MULTI_CURRENCY_DISABLED` si la société est en mono-devise et qu'une devise est donnée (409 partout,
     décision d'epic) ;
   - code inconnu de la liste ISO 4217 embarquée (`kesh-core`) → 400 `CURRENCY_UNKNOWN` ; `CHF` donné → normalisé à
     `NULL`. La liste ne valide qu'**à l'écriture** : la lecture décode la colonne en `String`, de sorte qu'un code
     retiré plus tard de la liste ne fasse jamais échouer la lecture d'un compte ;
   - changer la devise d'un compte qui porte **au moins une ligne d'écriture** → 409 `ACCOUNT_CURRENCY_LOCKED`
     (jamais de confirmation possible, à la différence du changement de type). Un `PUT` qui renvoie la devise en place
     est un no-op sur ce champ et ne déclenche pas ce refus (`currency` entre dans `is_no_op_change`) ;
   - **un compte que les chemins automatiques débitent en CHF ne peut recevoir de devise** → 409
     `ACCOUNT_CURRENCY_IN_USE` (le site en `details`) : compte qui porte un rôle (`AccountRole`, les dix) ; désigné
     dans les réglages de facturation (`DESIGNATED_ACCOUNT_COLUMNS`, `company_invoice_settings.rs:46-57`) ; lié à un
     compte bancaire (`bank_accounts.journal_account_id`) ; compte de produit par défaut d'un article
     (`products.default_revenue_account_id`) ; contrepartie d'une règle de réconciliation
     (`reconciliation_rules.counterparty_account_id`) ; compte par défaut des factures fournisseur. **Inversement**,
     lier à un compte bancaire, désigner dans les réglages de facturation, choisir comme compte d'un article, d'une
     règle de réconciliation ou de facture fournisseur, ou poser un rôle sur un compte **en devise** → 400
     `ACCOUNT_IN_FOREIGN_CURRENCY`. **C'est la 35-1 qui pose ces refus des deux côtés** ; la 35-3a (AC-5) s'y appuie.
     Inventaire **clos** des sites, lu sur les FK `REFERENCES accounts` des migrations et sur les liens sans FK
     (`bank_accounts.journal_account_id`), écrit au Change Log (méthode « inventorier les sites non résolus ») ;
   - un compte désigné gain/perte ne peut recevoir de devise, ni être archivé, ni changer de type hors
     `Expense`/`Revenue`, ni recevoir le rôle `DefaultRevenue`, ni passer à `postable = false`, ni recevoir un
     sous-compte (`accounts::create` :197-207 passe le parent à non imputable sans garde) → 409
     `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT`. À l'archivage, ce refus est **sans recours** — à la différence du précédent
     #486 (`AccountDesignatedInSettings`, confirmable) : `confirm_designated` ne le lève pas. Ordre des refus
     d'`archive_guarded` : sous-comptes actifs → gain/perte → désignation de facturation → version ;
   - `reactivate` d'un compte en devise alors que la société est en mono-devise → 409 `MULTI_CURRENCY_DISABLED`
     (défensif : le décompte de la désactivation inclut déjà les archivés) ;
   - **verrous** : `accounts::create`, `update`, `archive_guarded` et `reactivate` lisent la ligne `companies`
     `LOCK IN SHARE MODE` **avant** le `FOR UPDATE` du compte (même ordre que l'AC-3, pas d'interblocage) ;
     `multi_currency_enabled` et les deux colonnes FX se lisent sous ce verrou. Le compte des lignes d'écriture se fait
     **après** le `FOR UPDATE` du compte. La 35-3a lit la devise des comptes `LOCK IN SHARE MODE`
     (`validate_lines_accounts_in_tx` :135 lit aujourd'hui sans verrou). Un test de course par paire : désactivation
     contre création en devise, désignation contre archivage/retypage, changement de devise contre première ligne ;
   - un compte de groupe (non imputable) peut porter une devise (informatif) ; un enfant n'hérite de rien.
5. **AC-5 — Réinitialisation de la société.** Seul chemin qui vide les comptes en gardant la société :
   `kesh_seed::reset_demo` (`onboarding/reset`) — `accounts` est dans `reset_cleared_tables`, `companies` est
   préservée (`backup.rs:136-144`) et remise à zéro en place par `companies::reset_to_stub_in_tx`
   (`companies.rs:214`). Aucun autre `DELETE FROM accounts` hors tests (grep vérifié) ; l'import complet remplace
   `companies` entière. `reset_to_stub_in_tx` écrit `multi_currency_enabled = FALSE, fx_gain_account_id = NULL,
   fx_loss_account_id = NULL`. Test : les trois colonnes entrent dans `PERTURBED` (**pas** dans `NOT_COMPARED`) de
   `reset_restores_the_stub_columns_in_place` (`onboarding_audit_e2e.rs:2342`) et sont perturbées par son `UPDATE`
   (:2344).
6. **AC-6 — Sauvegarde.** Un `.keshbackup` écrit par la 0.17.6 se réimporte (colonnes à leur défaut) ; l'aller-retour
   d'une sauvegarde 35-1 conserve réglage et devises ; l'export CSV des comptes (`serialize_accounts_csv`,
   `csv_tables.rs:265`, en-tête écrit à la main) porte `currency` ; `company.csv` (`serialize_company_csv`, :163)
   porte les trois colonnes.
7. **AC-7 — Écrans.**
   - *Paramètres → Comptabilité* : une case « Comptabilité multi-devises » et, cochée, deux sélecteurs (compte de gains
     de change, compte de pertes de change ; mêmes filtres que l'AC-3), enregistrer, erreurs affichées ; le texte dit que
     la devise principale reste le CHF et qu'on ne revient en mono-devise que sans compte en devise. Section éditable
     pour `isAdmin` seulement (`settings/+page.svelte:228`), lecture seule pour les autres rôles.
   - *Plan comptable* : en multi-devises, un champ « Devise » (sélecteur ISO 4217, vide = CHF) à la création et à la
     modification, désactivé d'avance avec l'explication si `currencyLocked` ; une colonne « Devise » dans la liste
     (vide pour CHF). En mono-devise : **aucun** changement visible.
   - Captures d'écran des états touchés (recette visuelle, thème clair), hors du dépôt.
8. **AC-8 — MCP et services.** `kesh_plan_comptable` (`services/plan_comptable.rs`) donne la devise d'un compte en
   devise.
9. **AC-9 — i18n.** Toutes les chaînes neuves dans les quatre locales (`fr-CH`, `de-CH`, `it-CH`, `en-CH`),
   `lint-i18n-ownership` vert.
10. **AC-10 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) : section « Comptabilité multi-devises » —
    activer, comptes de différences de change, devise d'un compte et son verrou ; PDF régénéré et contrôlé
    (`pdftotext | tr`). Aucun plan livré n'a de compte « Différences de change » (`pme.json:76-78` : 6900, 6940,
    6950) : le manuel dit quoi choisir ou créer (p. ex. 6840). Si le manuel cite la route
    `PUT /api/v1/companies/current/currency-settings`, ajouter l'entrée à `MULTI_SOCIETE_ASSUME` de
    `textes_coherents.rs` (motif `\bcompanies\b`, précédent :1014-1017).

## Tâches

- [ ] Migration + `migrations.sha384` + audit d'idempotence + squash (AC-1).
- [ ] `kesh-core` : module `currency` — liste ISO 4217 (codes actifs, trois lettres), `CurrencyCode` validé,
      `BASE_CURRENCY = "CHF"` ; tests (AC-4).
- [ ] `kesh-db` : `Company` + `FIND_BY_ID_SQL`/`LIST_SQL` (`companies.rs:24,29`) ; `Account`/`NewAccount`/
      `AccountUpdate` + macro `columns!` ; `companies::update_currency_settings` (gardes C-35-2, verrous AC-3) ;
      `accounts::create/update/archive_guarded/reactivate` (verrous, gardes AC-4) ; `is_no_op_change` (:533-540) et
      `account_snapshot_json` (:57) avec `currency` ; `bulk_create` (:1146) et `bulk_create_from_chart_in_tx` (:1256)
      lient `currency` (leurs `INSERT` l'ignoreraient sinon en silence ; plans livrés et assistant : `NULL`) ;
      gardes croisées de l'AC-4 dans les réglages de facturation, comptes bancaires, articles, règles de
      réconciliation, factures fournisseur ; `reset_to_stub_in_tx` (AC-5) ; `test_fixtures.rs`, seed.
- [ ] `kesh-api` : `CompanyJson` (`companies.rs:30`, miroir écrit à la main) ; `serialize_accounts_csv` et
      `serialize_company_csv` (AC-6).
- [ ] `kesh-api` : routes, DTO, codes d'erreur (`errors.rs` + i18n), audit (`audit_labels.rs`), MCP (AC-2 à AC-8).
- [ ] Frontend : `settings` (section Comptabilité), `accounts` (champ, colonne), types, API, Vitest (AC-7).
- [ ] Manuel (AC-10), CHANGELOG `[Unreleased]`.
- [ ] Tests : dépôt (gardes, verrou, reset), API (codes), sauvegarde (AC-6), E2E Playwright (activer le mode, créer un
      compte USD, verrou après une écriture passée par l'API).

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
  manuel, 35-3).
- `CHECK` : collation binaire de la colonne **et** `BINARY` dans les `CHECK` (AC-1) — `20261110000001` ne met
  `BINARY` que sur son `IN`, son `REGEXP` ne porte que sur des chiffres : ne pas le recopier tel quel.

## Change Log

- 2026-11-14 — fiche rédigée.
- 2026-11-14 — validation P1 (Opus) : 0 CRITICAL, 2 HIGH, 9 MEDIUM, 5 LOW ; tous remédiés.
