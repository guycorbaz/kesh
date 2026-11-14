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
   - `accounts` : `currency CHAR(3) NULL` (NULL = devise principale), `CHECK (currency IS NULL OR currency REGEXP
     '^[A-Z]{3}$')` et `CHECK (currency IS NULL OR currency <> 'CHF')` (la devise principale s'écrit NULL, une seule
     graphie).
   - Les colonnes des lignes d'écriture **ne sont pas** dans cette migration (35-3).
   - Inscrite à `migrations.sha384`, au tableau de `docs/migrations-idempotence-audit.md` (compteurs recomptés, P5),
     squash régénéré (`scripts/regen-test-schema.sh`), P6 inspecté, P7 : aucune donnée écrite (le test de triage passe
     sans entrée).
   - `kesh_version_min_required` inchangé (0.17.6).
2. **AC-2 — Lecture du réglage.** `GET /api/v1/companies/current` expose `multiCurrencyEnabled`, `fxGainAccountId`,
   `fxLossAccountId`, `baseCurrency: "CHF"` (constante, C-35-2 ; la devise principale ne change pas).
3. **AC-3 — Écriture du réglage.** `PUT /api/v1/companies/current/currency-settings`
   `{ multiCurrencyEnabled, fxGainAccountId, fxLossAccountId, version }` (verrou optimiste comme les autres `PUT` de
   la société, rôle Administrateur ou Comptable selon le précédent des réglages comptables) :
   - activer exige les deux comptes (le même compte est admis pour les deux) ;
   - chaque compte : de la société, imputable, actif, de type `Expense` ou `Revenue`, **sans devise** ; lu
     `FOR UPDATE` dans la transaction ; sinon 400 `FX_ACCOUNT_INVALID` (champ en `details`) ;
   - désactiver est refusé (409 `MULTI_CURRENCY_IN_USE`) tant qu'un compte porte une devise ; désactiver remet les deux
     comptes à `NULL` ;
   - audit `company.currency_settings_updated` (avant/après) ; libellés d'audit dans les quatre locales.
4. **AC-4 — Devise d'un compte.** `CreateAccountRequest` et `UpdateAccountRequest` prennent `currency` facultatif
   (code ISO 4217 à trois lettres majuscules, `null` ou absent = CHF) ; `AccountResponse` l'expose :
   - refus 400 `MULTI_CURRENCY_DISABLED` si la société est en mono-devise et qu'une devise est donnée ;
   - code inconnu de la liste ISO 4217 embarquée (`kesh-core`) → 400 `CURRENCY_UNKNOWN` ; `CHF` donné → normalisé à
     `NULL` ;
   - changer la devise d'un compte qui porte **au moins une ligne d'écriture** → 409 `ACCOUNT_CURRENCY_LOCKED`
     (jamais de confirmation possible, à la différence du changement de type) ;
   - un compte désigné gain/perte ne peut recevoir de devise, ni être archivé, ni changer de type hors
     `Expense`/`Revenue` → 409 `ACCOUNT_IS_FX_DIFFERENCE_ACCOUNT` ;
   - un compte de groupe (non imputable) peut porter une devise (informatif) ; un enfant n'hérite de rien.
5. **AC-5 — Réinitialisation de la société** (`onboarding/reset`, et tout chemin qui vide les données d'une société) :
   les trois colonnes reprennent leur défaut ; test dédié.
6. **AC-6 — Sauvegarde.** Un `.keshbackup` écrit par la 0.17.6 se réimporte (colonnes à leur défaut) ; l'aller-retour
   d'une sauvegarde 35-1 conserve réglage et devises ; l'export CSV des comptes porte la colonne `currency`.
7. **AC-7 — Écrans.**
   - *Paramètres → Comptabilité* : une case « Comptabilité multi-devises » et, cochée, deux sélecteurs (compte de gains
     de change, compte de pertes de change ; mêmes filtres que l'AC-3), enregistrer, erreurs affichées ; le texte dit que
     la devise principale reste le CHF et qu'on ne revient en mono-devise que sans compte en devise.
   - *Plan comptable* : en multi-devises, un champ « Devise » (sélecteur ISO 4217, vide = CHF) à la création et à la
     modification, désactivé avec l'explication si le compte a des écritures ; une colonne « Devise » dans la liste
     (vide pour CHF). En mono-devise : **aucun** changement visible.
   - Captures d'écran des états touchés (recette visuelle, thème clair), hors du dépôt.
8. **AC-8 — MCP et services.** `kesh_plan_comptable` (`services/plan_comptable.rs`) donne la devise d'un compte en
   devise.
9. **AC-9 — i18n.** Toutes les chaînes neuves dans les quatre locales (`fr-CH`, `de-CH`, `it-CH`, `en-CH`),
   `lint-i18n-ownership` vert.
10. **AC-10 — Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) : section « Comptabilité multi-devises » —
    activer, comptes de différences de change, devise d'un compte et son verrou ; PDF régénéré et contrôlé
    (`pdftotext | tr`).

## Tâches

- [ ] Migration + `migrations.sha384` + audit d'idempotence + squash (AC-1).
- [ ] `kesh-core` : module `currency` — liste ISO 4217 (codes actifs, trois lettres), `CurrencyCode` validé,
      `BASE_CURRENCY = "CHF"` ; tests (AC-4).
- [ ] `kesh-db` : `Company`, `CompanyUpdate`/réglage dédié, `Account`/`NewAccount`/`AccountUpdate` + macro `COLUMNS` ;
      `companies::update_currency_settings` (gardes C-35-2) ; `accounts::create/update/archive*` (verrou, gardes) ;
      réinitialisation (AC-5) ; `test_fixtures.rs`, seed.
- [ ] `kesh-api` : routes, DTO, codes d'erreur (`errors.rs` + i18n), audit (`audit_labels.rs`), MCP (AC-2 à AC-8).
- [ ] Frontend : `settings` (section Comptabilité), `accounts` (champ, colonne), types, API, Vitest (AC-7).
- [ ] Manuel (AC-10), CHANGELOG `[Unreleased]`.
- [ ] Tests : dépôt (gardes, verrou, reset), API (codes), sauvegarde (AC-6), E2E Playwright (activer le mode, créer un
      compte USD, verrou après une écriture passée par l'API).

## Notes de développement

- Comptes : `crates/kesh-db/src/repositories/accounts.rs` (`COLUMNS` :34, `create` :153, `update` :622, garde
  `AccountHasEntries` vers :575-588, `archive_guarded` :859) ; `crates/kesh-api/src/routes/accounts.rs`
  (`CreateAccountRequest` :36, `UpdateAccountRequest` :77, `AccountResponse` :139).
- Société : `crates/kesh-db/src/entities/company.rs` (`Company` :137), `repositories/companies.rs` (`update_in_tx`
  :842) ; `routes/companies.rs` (`get_current` :448, `PUT` par champ :112/:250/:344/:497 — suivre leur forme) ;
  écran `frontend/src/routes/(app)/settings/+page.svelte` (section Comptabilité :881-890).
- Précédent de garde d'archivage d'un compte désigné : #486 (Story 33-8, `compte-designe-archive`).
- La liste ISO 4217 est **embarquée** (pas de dépendance réseau) ; une devise absente des cours AFC reste admise (cours
  manuel, 35-3).
- `CHECK` en `BINARY`/`REGEXP` : suivre le style des migrations récentes (`20261110000001`).

## Change Log

- 2026-11-14 — fiche rédigée.
