# Story 26.2 : Une courte description pour chaque compte bancaire

## Status

**ready-for-dev** — fiche créée le 2026-10-10 en autonomie (`bmad-create-story`), sur `fe162504` (kickoff de
l'Epic 26), branche `story/26-2`. Validation (`bmad-create-story validate`) à venir.

⚠️ **Découpage proposé, non encore matérialisé** (registre **C-26-2-1**) : l'inventaire touche bien plus de cinq
modules au sens du `CLAUDE.md` (§ *Règle de splitting préventif*). La fiche est écrite pour se scinder sans
réécriture — chaque AC porte sa partie :

| Partie | Contenu | AC | PR |
|---|---|---|---|
| **26-2a — socle** | colonne, dépôt, API, validation, export CSV et `.keshbackup`, fonctions de libellé (TS et Rust), saisie et écran *Comptes bancaires*, i18n | AC1 à AC8, AC13 (a), AC14 | `refs #622` |
| **26-2b — rollout** | les autres écrans, `GET /companies/current`, message serveur du refus de désignation, garde d'inventaire clos, manuel, CHANGELOG | AC9 à AC12, AC13 (b), AC14 | `closes #622` |

26-2b suppose 26-2a fusionnée. Sa revue se fait **fichier par fichier** (rollout mécanique). La décision de
découper — nouvelles clés au registre de sprint, branches, PR — revient à l'orchestrateur ; tant qu'elle n'est
pas prise, la clé `26-2-description-compte-bancaire` porte les deux parties, et la PR unique porte `closes #622`.

## Story

**As a** indépendant ou PME qui tient plusieurs comptes dans la même banque,
**I want** donner à chaque compte bancaire une courte description facultative (« Personnel », « Épargne ») et la
voir partout où Kesh me montre ou me propose un compte (« UBS — Personnel »),
**so that** je distingue mes comptes sans lire l'IBAN.

Issue : [#622](https://github.com/guycorbaz/kesh/issues/622) (CR, recette v0.13.0). Décision du Project Lead du
2026-10-10, commentaire de l'issue : **facultative** ; sans elle, l'affichage reste celui d'aujourd'hui, et les
comptes existants n'ont rien à saisir. Epic : `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md`
(décision 4). Cible : **v0.13.1**.

## Critères d'acceptation

### Partie a — socle

**AC1 — La colonne, non breaking, et ses garde-fous.**

- Migration neuve `crates/kesh-db/migrations/20261011000001_bank_accounts_description.sql` (version à confirmer
  contre `origin/main` au développement, C-26-2-14), **une seule instruction** :
  `ALTER TABLE bank_accounts ADD COLUMN description VARCHAR(60) NULL AFTER bank_name, ALGORITHM=INSTANT, LOCK=NONE;`
  précédée d'un en-tête de commentaires (story, #622, non-breaking, pourquoi 60). Aucune contrainte, aucun index
  (C-26-2-3, C-26-2-4).
- **P1/P3** : `ADD COLUMN` nullable → **non breaking** ; aucun `UPDATE _kesh_version`, aucun bump Cargo. Une 0.13.0
  qui lirait la base l'ignore (ses `SELECT` nomment leurs colonnes).
- **P5** : une ligne à sa place chronologique **dans** le tableau de `docs/migrations-idempotence-audit.md`, verdict
  `tracked-by-sqlx` (`ADD COLUMN` sans `IF NOT EXISTS` → erreur 1060 au rejeu hors sqlx), non-breaking, pas
  d'écriture de données. Les **cinq** compteurs **recomptés depuis le tableau** : en-tête `## Table d'audit (N
  migrations)`, ligne `Total` (avec sa ventilation : `+ 1 Story 26-2`), `yes`, `tracked-by-sqlx`, `no` — sur
  `fe162504` : 78 / 78 / 10 / 68 / 0, donc **79 / 79 / 10 / 69 / 0** si la 26-1 n'a rien fusionné entre-temps.
  Commandes, à exécuter et à recopier au Dev Agent Record :
  ```sh
  ls crates/kesh-db/migrations/*.sql | wc -l
  grep -c '^| `20' docs/migrations-idempotence-audit.md
  grep -c '^| `20.*| yes |' docs/migrations-idempotence-audit.md
  grep -c '^| `20.*| tracked-by-sqlx |' docs/migrations-idempotence-audit.md
  grep -c '^| `20.*| no |' docs/migrations-idempotence-audit.md
  ```
- **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` inspecté site par site. Sur `fe162504`,
  un seul site positionnel : `crates/kesh-db/tests/migrations_upgrade_path.rs` (`total, 78` l. 110 ; `total - 44`
  l. 192), frontière figée à **34** → `total` **79**, soustracteur **45**, et **toutes** les mentions de la valeur
  dans le fichier (commentaires l. 38, 40, 57-58, 107, 111, 115, 123, 130, 133, 185, 195, 264), greppées **par
  jeton** : `grep -nE "\b(78|44)\b" crates/kesh-db/tests/migrations_upgrade_path.rs` doit ne plus rendre que des
  généalogies historiques. Les autres sites (`lettering_documents_backfill.rs`, `audit_log_company_id_backfill.rs`,
  `journal_entry_number_sequences_bootstrap.rs`, `accounts_role_backfill.rs`,
  `invoice_lines_revenue_account_backfill.rs`) résolvent par version (`migrations_before`) : rien à faire, à
  confirmer par le gate.
- **P7** : la migration n'écrit aucune donnée (pas d'`UPDATE`, `INSERT`, `REPLACE`, `DELETE`) → **ni registre
  `POST_RESTORE_BACKFILLS`, ni `EXEMPT_MIGRATIONS`** ; `every_data_backfill_migration_is_triaged` reste vert sans
  retouche — **à vérifier au gate**, pas à supposer. Les commentaires de l'en-tête évitent les mots-clés SQL
  d'écriture en tête de ligne (le détecteur travaille sur du SQL décommenté, mais la prudence ne coûte rien).
- **P8** : la migration n'est éditée **plus du tout** une fois appliquée à une base de dev persistante
  (`kesh_e2e_<clé>` notamment) ; toute précision ultérieure va dans la ligne d'audit ou les Dev Notes.
- **Squash de test** régénéré par `scripts/regen-test-schema.sh` (jamais édité à la main) :
  `crates/kesh-db/tests/test_schema_guard.rs` vert.

**AC2 — Entité et dépôt : une seule liste de colonnes, et trois chemins qui ne touchent pas la description.**

- `BankAccount` (`crates/kesh-db/src/entities/bank_account.rs`) gagne `pub description: Option<String>` (doc
  `///` : facultative, normalisée par la route, `None` = absente), sérialisée `description`.
  `NewBankAccount` **ne change pas** (C-26-2-7).
- Les **onze** `SELECT` qui énumèrent les colonnes vers `BankAccount` (dix dans
  `crates/kesh-db/src/repositories/bank_accounts.rs` : `FIND_BY_ID_SQL` l. 25, `find_primary` l. 77,
  `find_by_id_for_company` l. 107, `list_by_company` l. 129 et 132, `upsert_primary_in_tx` l. 227,
  `set_journal_account_id_for_company` l. 353, `update_for_company` l. 468, et l. 541, l. 606 ; un dans
  `crates/kesh-api/src/routes/bank_accounts.rs` l. 520-522) passent par **une seule constante** de colonnes
  (`const BANK_ACCOUNT_COLUMNS: &str`) — DRY, et c'est ce qui empêche un `SELECT` oublié de faire échouer
  `FromRow` à l'exécution. Le `SELECT` inline de la route (relecture après `INSERT`) est remplacé par une fonction
  publique du dépôt (`find_by_id_in_tx`) ou par la constante exportée. Recompte de contrôle :
  `grep -rnF "is_primary, journal_account_id" crates/*/src` ne rend plus que la constante.
- `update_for_company` reçoit `new_description: Option<&str>` (paramètre séparé, patron de
  `new_journal_account_id`) et l'écrit ; ses appelants (route, et les tests de
  `crates/kesh-db/tests/bank_accounts_repository.rs`) suivent. Sa doc-comment énumère le champ.
- **Ne touchent pas** la colonne : `upsert_primary_in_tx` (ni l'`INSERT`, ni l'`UPDATE`, ni
  `is_no_op_change`), `set_journal_account_id_for_company` (PATCH), `archive_for_company` (DELETE),
  `bank_accounts::create`. Chacun a un test qui pose une description (par SQL) puis exerce le chemin et la
  retrouve intacte (l'upsert : changement de `bank_name` → `Updated`, description conservée).
- `invoice_pdf_service.rs` (l. 659 et 845, littéraux de test) gagne `description: None` ; le PDF et le QR-Bill
  **n'affichent pas** le nom de la banque (`grep -rn "bank_name" crates/kesh-qrbill crates/kesh-payment
  crates/kesh-report` → aucun résultat sur `fe162504`) : rien à y changer.

**AC3 — Validation et normalisation serveur** (C-26-2-3, C-26-2-11).

`validate_bank_input` (`crates/kesh-api/src/routes/bank_accounts.rs` l. 230) reçoit la description et rend sa
forme normalisée :

- absente, `null`, vide ou faite d'espaces → `None` ;
- sinon `trim()` ; plus de **60 caractères** (`chars().count()`) → `400 VALIDATION_ERROR`, message en dur
  « La description ne peut pas dépasser 60 caractères » ; un caractère de contrôle (`char::is_control`, dont `\n`,
  `\t`) → `400`, « La description ne peut pas contenir de saut de ligne ni de caractère de contrôle » ;
- **ordre** des refus de forme : banque vide → description → IBAN → QR-IBAN.
- Tests aux bornes : 60 caractères acceptés, 61 refusés ; 60 caractères **accentués** (`é` ×60) acceptés —
  `chars`, pas octets ; `"  Épargne  "` stocké `"Épargne"` ; `"   "` stocké `NULL` ; `"a\nb"` refusé.

**AC4 — L'API.**

- `POST /api/v1/bank-accounts` : `CreateBankAccountBody` gagne `#[serde(default)] description: Option<String>` ;
  l'`INSERT` de la route la nomme. Réponse `201` avec `description`.
- `PUT /api/v1/bank-accounts/{id}` : `UpdateBankAccountBody` idem ; **remplacement complet** — absente ⇒ effacée,
  comme `qrIban` (C-26-2-8). La doc-comment du handler le dit.
- `GET /api/v1/bank-accounts` : `description` présente sur chaque élément (`null` si absente) — gratuit par le
  `#[serde(flatten)]` de `BankAccountWithBalance`, mais **testé**.
- `PATCH` et `DELETE` : réponse avec la description inchangée (testé).
- Audit (C-26-2-10) : `bank_account.created` porte `"description"` ; `bank_account.updated` (`trigger:
  full_update`) la porte dans `before` et `after`. `audit_route_registry.rs` reste vert. L'audit de l'onboarding
  (`routes/onboarding.rs` l. 600-660) ne change pas : la description n'y bouge pas.
- Contrat multi-tenant inchangé : rien de neuf dans les gardes (le champ vit dans la ligne déjà scopée).

**AC5 — Export CSV et sauvegarde d'installation.**

- `serialize_bank_accounts_csv` (`crates/kesh-api/src/exports/csv_tables.rs` l. 623) : colonne `description`
  **juste après** `bank_name` (C-26-2-6), valeur `fmt_opt_str(&b.description)`. La garde
  `chaque_colonne_du_schema_est_exportee_ou_ecartee` (même fichier, l. 2147) — qui lit le squash — rougirait sans
  elle : elle doit être verte **après** régénération du squash, et un test de valeur (ligne avec et sans
  description) s'ajoute aux tests du sérialiseur.
- `.keshbackup` : l'export est générique (`kesh_db::backup::export_table`, colonnes lues dans
  `information_schema`) — la colonne y entre seule. Deux tests dans `crates/kesh-api/tests/admin_full_import_e2e.rs` :
  (1) aller-retour : un compte décrit, export, import → description restituée ; (2) **sauvegarde antérieure** :
  `strip_column(&mut manifest, "bank_accounts", "description")` (helper existant, l. 925), import → `200`, compte
  restauré, description `NULL` (`is_required()` est faux pour une colonne nullable).
- `post_restore.rs` : rien (AC1, P7).

**AC6 — Le libellé, en une fonction de chaque côté** (C-26-2-5).

- **Frontend** — `frontend/src/lib/features/bank-accounts/format.ts` (qui porte déjà `shortIban`) exporte, avec
  JSDoc :
  - `bankAccountName(a: { bankName: string; description?: string | null }): string` → `"UBS — Personnel"` ou
    `"UBS"` ; une description `null`, `undefined`, vide ou d'espaces vaut absente ; séparateur `" — "` (tiret
    cadratin U+2014 entouré d'espaces, celui des sélecteurs actuels) ;
  - `bankAccountOptionLabel(a: { bankName: string; description?: string | null; iban: string }): string` →
    `` `${bankAccountName(a)} — ${a.iban}` ``.
  - Tests Vitest (`format.test.ts`, neuf ou existant) : avec, sans, `null`, `undefined`, `"  "` ; sans
    description, le libellé d'option est **identique** à l'ancien `` `${bankName} — ${iban}` ``.
- **Serveur** — `kesh_db::entities::bank_account` exporte `pub fn bank_account_display_name(bank_name: &str,
  description: Option<&str>) -> String` et `impl BankAccount { pub fn display_name(&self) -> String }`, mêmes
  règles, mêmes exemples en test unitaire (`#[cfg(test)]` du module). Elle sert à AC10.

**AC7 — Saisie et écran *Administration → Comptes bancaires*.**

`frontend/src/routes/(app)/bank-accounts/+page.svelte` :

- formulaires de **création** et de **modification** : un champ « Description (facultative) » placé **juste après
  « Banque »**, `data-testid="form-description"` / `"edit-description"`, `maxlength="60"`, aide sous le champ
  (`bank-accounts-help-description`) ; état `formDescription` remis à zéro par `resetForm`, prérempli par
  `openEditFull` (`ba.description ?? ''`) ; envoyé `description: formDescription.trim() === '' ? null :
  formDescription` dans les **deux** payloads (`NewBankAccountPayload`, `UpdateBankAccountPayload` gagnent
  `description?: string | null`) ;
- tableau : la cellule « Banque » (l. 413) affiche `bankAccountName(ba)` — badges *Principal* et *Archivé*
  inchangés ;
- `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte` l. 142 : l'en-tête
  `{bankAccount.bankName} — {bankAccount.iban}` devient `bankAccountOptionLabel(bankAccount)` ;
- `bank-accounts.api.ts` : `BankAccountSummary` et `BankAccountSummaryRaw` gagnent `description: string | null`
  (**obligatoire** dans le type public, pour que `npm run check` signale chaque fixture à compléter) ;
  `parseBankAccount` mappe `raw.description ?? null`.
- Tests Vitest de l'écran (`bank-accounts-page.test.ts`) : création avec et sans description (payload), édition
  qui préremplit puis efface (payload `null`), cellule avec et sans description. Les fixtures de
  `BankAccountSummary` des huit fichiers de test qui en construisent (cf. Dev Notes) gagnent `description: null`.

**AC8 — i18n, quatre locales.**

- Clés neuves dans `crates/kesh-i18n/locales/{fr-CH,de-CH,it-CH,en-CH}/messages.ftl`, à côté des
  `bank-accounts-labels-*` : `bank-accounts-labels-description` (« Description (facultative) » /
  « Beschreibung (optional) » / « Descrizione (facoltativa) » / « Description (optional) ») et
  `bank-accounts-help-description` (« Par exemple « Personnel » ou « Épargne » : pour distinguer deux comptes de la
  même banque. 60 caractères au plus. » et ses traductions). Le repli de chaque `i18nMsg` est **identique** au
  texte fr-CH.
- Gardes : `npm run lint-i18n-ownership` ; `frontend/src/lib/shared/i18n-keys.test.ts` — `ATTENDU.sitesTotal` et
  bornes associées **recomptés depuis la source** (`grep -o "i18nMsg("` sur `main` contre l'arbre, aux deux bornes),
  avec le commentaire de généalogie d'usage ; `parity_between_locales` (kesh-i18n) vert → **gate backend complet**
  (les FTL sont dans `kesh-i18n`).

### Partie b — rollout

**AC9 — Chaque site qui affiche ou propose un compte bancaire passe par AC6.** L'inventaire clos est en Dev Notes
(§ *Inventaire clos*) ; ses sites **D1 à D10** sont tous résolus ainsi :

| Site (sur `fe162504`) | Aujourd'hui | Après |
|---|---|---|
| D3 `routes/(app)/reconciliation/+page.svelte:65` | `{ba.bankName} — {ba.iban}` | `bankAccountOptionLabel(ba)` |
| D4 `routes/(app)/payment-batches/+page.svelte:133` | `{b.bankName} — {b.iban}` | `bankAccountOptionLabel(b)` |
| D5 `routes/(app)/supplier-invoices/[id]/+page.svelte:344` | `{b.bankName} — {b.iban}` | `bankAccountOptionLabel(b)` |
| D6 `lib/features/invoices/SettleInvoiceDialog.svelte:250` | `{b.bankName} — {b.iban}` | `bankAccountOptionLabel(b)` |
| D7 `lib/features/bank-import/BankImportUpload.svelte:212` | `{account.bankName} ({account.iban})` | `bankAccountOptionLabel(account)` — **seul changement visible sans description** (C-26-2-5) |
| D8 `lib/features/homepage/BankAccountsCard.svelte:38` | `{account.bankName}` (+ `shortIban` dessous, inchangé) | `bankAccountName(account)` |
| D9 `routes/(app)/settings/+page.svelte:470` | `{account.bankName}` (+ IBAN dessous, inchangé) | `bankAccountName(account)` |

(D1, D2 — écran *Comptes bancaires* et formulaire de lien — sont en AC7 ; D10 — message serveur — en AC10.)

Les types en ligne `BankAccountListItem` de `routes/(app)/reconciliation/+page.svelte:11` et
`routes/(app)/bank-import/+page.svelte:10`, le type de prop de `BankImportUpload.svelte:24` et `BankAccountJson`
de `lib/features/settings/settings.types.ts:44` gagnent `description: string | null` — ou, mieux, se réduisent à
un type partagé exporté de `format.ts` (`BankAccountLabelSource`), au choix du développeur, consigné.

Tests Vitest : `SettleInvoiceDialog.test.ts` (option avec description), `HomepageCards.test.ts` (tuile avec et
sans), `supplier-settlement-page.test.ts` (option avec description) ; les autres sites sont couverts par AC11 et
l'E2E.

**AC10 — `GET /companies/current` et le refus de désignation nomment le compte.**

- `BankAccountJson` (`crates/kesh-api/src/routes/companies.rs` l. 316-332) gagne `description: Option<String>` —
  c'est **la source** des sélecteurs de l'import bancaire et de la réconciliation (D3, D7) et de *Paramètres* (D9).
  Test : la réponse porte la description.
- Refus `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT` (C-26-2-9) : `first_active_bank_account_linked_to`
  (`repositories/bank_accounts.rs` l. 723-738) sélectionne `bank_name, description` **sans toucher** au
  `LOCK IN SHARE MODE` ni au texte `FROM bank_accounts` (un test de sérialisation l'attend, cf. sa doc) et rend
  `(id, bank_account_display_name(..))` ; le champ `bank_name` de `DbError::ClaimAccountLinkedToBankAccount`
  (`crates/kesh-db/src/errors.rs` l. 1078) devient `bank_label`, propagé à `company_invoice_settings.rs` l. 217-229,
  `crates/kesh-api/src/errors.rs` l. 3260-3270 et 3994-4023, et aux tests (`errors.rs` l. 1578 et 4834,
  `crates/kesh-db/tests/company_invoice_settings_repository.rs` l. 1497). Le message (`$bank` des quatre locales,
  textes FTL **inchangés**) et `details.bankName` (clé conservée) portent « UBS — Personnel ».
- Tests : le refus nomme « UBS — Personnel » quand le compte lié a une description, « UBS » sans (route
  `PUT /api/v1/company/invoice-settings` ; test e2e existant de la 15-6c,
  `crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs`, étendu).

**AC11 — Garde d'inventaire clos : aucun `.bankName` affiché hors des fonctions de libellé.**

Un test Vitest statique (dans `frontend/src/lib/features/bank-accounts/format.test.ts` ou un fichier de garde
voisin) lit **tous** les `.svelte` de `frontend/src` (`import.meta.glob(..., { query: '?raw', eager: true })` ou
`fs`) et relève chaque occurrence de `bankName` dans une **expression de gabarit** (`{…bankName…}`). L'ensemble
attendu est **clos et nommé** : seuls subsistent les sites de **saisie** listés en Dev Notes (*Exclus*, E1 à E4)
— chacun avec son motif dans la constante du test. Toute occurrence hors de cette liste fait rougir en nommant
`fichier:ligne` ; une entrée de la liste qui ne se trouve plus fait rougir aussi (liste périmée). **Borne
anti-test-muet** : le nombre de fichiers lus est ≥ un minimum mesuré, et la collecte trouve bien les sites
exclus (un motif cassé ne doit pas rendre la garde verte). Mutation exigée : rétablir `{b.bankName} — {b.iban}`
dans `SettleInvoiceDialog.svelte` → rouge.

**AC12 — Manuel utilisateur et CHANGELOG.**

- `docs/manual/fr/user-manual.tex` : nouvelle `\section{Comptes bancaires}\label{sec:comptes-bancaires}` **juste
  avant** `\section{Import bancaire}` (C-26-2-13), qui dit : où se gèrent les comptes (*Administration → Comptes
  bancaires*) ; leurs champs (banque, **description facultative** de 60 caractères au plus, IBAN, QR-IBAN
  éventuel, compte comptable lié, compte principal) ; ce que fait la description — « UBS — Personnel » dans
  **les listes de comptes** (la page elle-même, l'accueil, *Paramètres*) et **dans les sélecteurs** (import
  bancaire, réconciliation, règlement d'une facture, règlement d'une facture fournisseur, lot de paiement), où
  l'IBAN reste affiché à la suite ; qu'elle n'est pas unique ; qu'un compte sans description s'affiche comme
  avant ; qu'elle ne se saisit pas à l'onboarding mais ensuite ; qu'elle ne figure **ni sur la QR-facture ni dans
  le fichier de paiement** ; qu'elle est dans l'export CSV et la sauvegarde. L'item « Compte bancaire » de
  l'onboarding (l. 202) renvoie à cette section (`\S\ref{sec:comptes-bancaires}`).
- PDF régénéré (`make fr` dans `docs/manual/`) et **contrôlé aplati** :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "description facultative[^.]*"`.
- Le manuel d'administration n'est pas touché (sa phrase « le message nomme le compte bancaire », l. 2472, reste
  vraie). Manuels DE/IT/EN : inexistants (README seuls) — rien.
- `CHANGELOG.md` : section `## [0.13.1] — Non publié` (créée si absente, **fusion par union** avec la 26-1 et la
  26-3), `### Ajouté` : la description facultative, où elle s'affiche, le seul changement visible sans elle
  (sélecteur de l'import : « UBS — CH… » au lieu de « UBS (CH…) »), lien #622. `README.md` « Feuille de route » :
  vérifié, a priori inchangé.

### Les deux parties

**AC13 — E2E Playwright** (`frontend/tests/e2e/bank-accounts-crud.spec.ts`, seed `with-company`).

- **(a)** créer un compte « UBS E2E » avec la description « Épargne E2E » → la ligne de la liste montre
  « UBS E2E — Épargne E2E » ; le modifier en effaçant la description → « UBS E2E » seul ; le champ porte
  `maxlength="60"` (attribut vérifié ; le refus serveur de 61 caractères est couvert par AC3, pas par l'E2E —
  `fill()` ne garantit pas l'effet de `maxlength`).
- **(b)** deux comptes de la même banque, l'un décrit, l'autre non → sur `/bank-import`, le sélecteur
  `[data-testid="bank-account-select"]` propose « UBS E2E — Épargne E2E — CH… » et « UBS E2E — CH… » (texte des
  `<option>`, sélecteurs par `data-testid`, jamais par libellé traduit) ; sur `/reconciliation`, même
  sélecteur, même constat ; sur l'accueil `/`, la tuile montre « UBS E2E — Épargne E2E ».
- Le test (a) crée ses comptes avec des IBAN distincts de ceux des tests voisins du fichier ; s'il laisse des
  comptes derrière lui, l'ordre des tests du fichier n'en dépend pas (vérifier en lançant la spec seule **et**
  dans la suite).
- **Suite E2E complète** au dernier commit de code de chaque partie (D7 de la rétro Epic 25) ; « vert » =
  les échecs attendus de `docs/testing.md` § *Les échecs attendus*, jugés fichier par fichier.

**AC14 — Mutations et gates.**

Chaque mutation est appliquée, le test qui doit rougir **rougit** (sortie recopiée), puis restaurée — **et le
fichier touché** (`touch`) pour que cargo ne garde pas le binaire muté :

| # | Mutation | Doit rougir |
|---|---|---|
| M1 | retirer `description` de la constante de colonnes du dépôt | tests de dépôt / route (`FromRow`) |
| M2 | `update_for_company` ne lie plus la description | test `PUT` qui modifie la description |
| M3 | supprimer le `trim()` | test `"  Épargne  "` |
| M4 | borne `> 61` au lieu de `> 60` | test des 61 caractères |
| M5 | `upsert_primary_in_tx` écrit `description = NULL` | test de préservation à l'onboarding |
| M6 | `bankAccountName` ignore la description | `format.test.ts` |
| M7 | `bankAccountName` rend `"UBS — "` sur description vide | `format.test.ts` |
| M8 | rétablir `{b.bankName} — {b.iban}` dans un site de AC9 | garde AC11 |
| M9 | retirer `description` de `BankAccountJson` (companies) | test de `/companies/current` + E2E (b) |
| M10 | `first_active_bank_account_linked_to` rend `bank_name` seul | test du refus de désignation |
| M11 | retirer la colonne de `serialize_bank_accounts_csv` | `chaque_colonne_du_schema_est_exportee_ou_ecartee` |

M1-M5, M11 relèvent de la partie a ; M6-M7 aussi ; M8-M10 de la partie b.

Gates (CLAUDE.md § *Test Locally First*, consignes de l'epic) : `cargo fmt --check`, `clippy -D warnings`,
**gate backend complet** (story `kesh-db` : migration et dépôt — **pas** de gate ciblé, même en boucle de revue),
base de gate **remise à zéro avant** ; frontend `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ;
E2E complet (AC13). Le Dev Agent Record n'affirme que ce qui a tourné, avec ses chiffres.

## Tasks / Subtasks

- [ ] **T1 — Migration et garde-fous** (AC1) — partie a
  - [ ] T1.1 Vérifier la dernière migration de `origin/main` (course avec la 26-1) ; écrire la migration.
  - [ ] T1.2 `scripts/regen-test-schema.sh` sur une base jetable (jamais `kesh-mariadb-dev` à redémarrer).
  - [ ] T1.3 Ligne P5 + recompte des cinq compteurs ; P6 `migrations_upgrade_path.rs` (79 / 45, grep par jeton).
- [ ] **T2 — Entité et dépôt** (AC2)
  - [ ] T2.1 Champ `description`, constante de colonnes, `find_by_id_in_tx`, `update_for_company`.
  - [ ] T2.2 `bank_account_display_name` + `display_name()` + tests unitaires.
  - [ ] T2.3 Tests de préservation (upsert, PATCH, archive) dans `bank_accounts_repository.rs`.
- [ ] **T3 — Route** (AC3, AC4)
  - [ ] T3.1 `validate_bank_input` étendu ; bodies POST/PUT ; INSERT ; audit.
  - [ ] T3.2 Tests `crates/kesh-api/tests/bank_accounts_e2e.rs` : POST/PUT/GET/PATCH/DELETE, bornes, audit.
- [ ] **T4 — Exports** (AC5) : CSV + test de valeur ; deux tests `admin_full_import_e2e.rs`.
- [ ] **T5 — Frontend socle** (AC6, AC7, AC8)
  - [ ] T5.1 `format.ts` + `format.test.ts`.
  - [ ] T5.2 Types et `parseBankAccount` ; fixtures des huit fichiers de test.
  - [ ] T5.3 Écran *Comptes bancaires* + `BankAccountJournalLinkForm` + tests.
  - [ ] T5.4 Clés i18n ×4, recompte `ATTENDU`.
- [ ] **T6 — E2E (a)** (AC13 a) ; mutations M1-M7, M11 (AC14) ; gates complets. *Fin de 26-2a.*
- [ ] **T7 — Rollout** (AC9) : D3 à D9, types partagés, tests Vitest. — partie b
- [ ] **T8 — Serveur** (AC10) : `BankAccountJson` ; refus de désignation (renommage `bank_label`) + tests.
- [ ] **T9 — Garde d'inventaire** (AC11).
- [ ] **T10 — Manuel, PDF, CHANGELOG** (AC12).
- [ ] **T11 — E2E (b)** (AC13 b) ; mutations M8-M10 ; gates complets ; E2E complet au dernier commit de code.

## Dev Notes

### Inventaire clos — les sites qui affichent ou proposent un compte bancaire

Méthode (CLAUDE.md § *Inventorier les sites NON RÉSOLUS* ; mémoire « un détecteur part du symptôme ») : le
symptôme est **un compte bancaire montré à un humain sans sa description**. Trois détecteurs, chacun relancé par
le développeur sur sa base et comparé à ce relevé de `fe162504` :

1. **le champ** — `grep -rn "bankName" frontend/src --include=*.svelte --include=*.ts | grep -v "\.test\.ts"` ;
2. **l'IBAN affiché** — `grep -rn "\.iban\b\|iban}" frontend/src --include=*.svelte | grep -v test` (un site
   pourrait montrer l'IBAN sans le nom) ;
3. **les consommateurs des listes** — `grep -rln "bankAccountId\|bankAccounts\|listBankAccounts" frontend/src
   --include=*.svelte --include=*.ts | grep -v "\.test\.ts"` (un site pourrait résoudre un compte par son id) ;
4. **le serveur** — `grep -rn "bank_name" crates --include=*.rs | grep -v "/tests/"`, et
   `grep -rn "bank_name\|bankName" crates/kesh-qrbill crates/kesh-payment crates/kesh-report
   crates/kesh-reconciliation crates/kesh-core crates/kesh-seed` (PDF, pain.001, rapports).

**Sortie du détecteur 1, copiée (35 lignes) :**

```
src/routes/(app)/payment-batches/+page.svelte:133:	<option value={b.id}>{b.bankName} — {b.iban}</option>
src/routes/(app)/reconciliation/+page.svelte:11:	type BankAccountListItem = { id: number; bankName: string; iban: string };
src/routes/(app)/reconciliation/+page.svelte:65:	<option value={ba.id}>{ba.bankName} — {ba.iban}</option>
src/routes/(app)/settings/+page.svelte:470:	<p class="font-medium">{account.bankName}</p>
src/routes/(app)/bank-accounts/+page.svelte:152:	formBankName = ba.bankName;
src/routes/(app)/bank-accounts/+page.svelte:209:	bankName: formBankName,
src/routes/(app)/bank-accounts/+page.svelte:233:	bankName: formBankName,
src/routes/(app)/bank-accounts/+page.svelte:413:	{ba.bankName}
src/routes/(app)/supplier-invoices/[id]/+page.svelte:344:	<option value={b.id}>{b.bankName} — {b.iban}</option>
src/routes/(app)/bank-import/+page.svelte:10:	type BankAccountListItem = { id: number; bankName: string; iban: string };
src/routes/onboarding/+page.svelte:121:	let bankName = $state('');
src/routes/onboarding/+page.svelte:168:	if (!bankName.trim() || !bankIban.trim()) {
src/routes/onboarding/+page.svelte:174:	bankName.trim(),
src/routes/onboarding/+page.svelte:373:	<Input id="bank-name" bind:value={bankName} required />
src/lib/features/bank-accounts/bank-accounts.api.ts:16:	bankName: string;
src/lib/features/bank-accounts/bank-accounts.api.ts:57:	bankName: string;
src/lib/features/bank-accounts/bank-accounts.api.ts:75:	bankName: raw.bankName,
src/lib/features/bank-accounts/bank-accounts.api.ts:124:	bankName: string;
src/lib/features/bank-accounts/bank-accounts.api.ts:148:	bankName: string;
src/lib/features/onboarding/onboarding.svelte.ts:134:	async setBankAccount(bankName: string, iban: string, qrIban: string | null) {
src/lib/features/onboarding/onboarding.svelte.ts:137:	_state = await api.setBankAccount(bankName, iban, qrIban);
src/lib/features/bank-import/BankProfileList.svelte:68:	<td>{profile.bankName}</td>
src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte:142:	{bankAccount.bankName} — {bankAccount.iban}
src/lib/features/settings/settings.types.ts:44:	bankName: string;
src/lib/features/onboarding/onboarding.api.ts:64:	bankName: string,
src/lib/features/onboarding/onboarding.api.ts:69:	bankName,
src/lib/features/bank-import/BankProfileForm.svelte:26:	let bankName = $state(existing?.bankName ?? '');
src/lib/features/bank-import/BankProfileForm.svelte:68:	bankName,
src/lib/features/bank-import/BankProfileForm.svelte:103:	bind:value={bankName}
src/lib/features/bank-import/bank-profile.types.ts:14:	bankName: string;
src/lib/features/invoices/SettleInvoiceDialog.svelte:250:	<option value={b.id}>{b.bankName} — {b.iban}</option>
src/lib/features/homepage/BankAccountsCard.svelte:38:	<p class="font-medium">{account.bankName}</p>
src/lib/features/bank-import/BankImportUpload.svelte:24:	bankAccounts: Array<{ id: number; bankName: string; iban: string }>;
src/lib/features/bank-import/BankImportUpload.svelte:212:	<option value={account.id}>{account.bankName} ({account.iban})</option>
src/lib/features/bank-import/BankProfileSelector.svelte:53:	{profile.bankName}{#if autoMatchedId === profile.id}
```
*(chemins relatifs à `frontend/`, indentation réduite ; 35 lignes, recomptées.)*

**Classement — chaque ligne a exactement une case :**

*Sites d'affichage ou de proposition — tous résolus :*

| # | Site | Résolution | Partie |
|---|---|---|---|
| D1 | `routes/(app)/bank-accounts/+page.svelte:413` (cellule) | `bankAccountName` (AC7) | a |
| D2 | `lib/features/bank-accounts/BankAccountJournalLinkForm.svelte:142` | `bankAccountOptionLabel` (AC7) | a |
| D3 | `routes/(app)/reconciliation/+page.svelte:65` | `bankAccountOptionLabel` (AC9) | b |
| D4 | `routes/(app)/payment-batches/+page.svelte:133` | idem | b |
| D5 | `routes/(app)/supplier-invoices/[id]/+page.svelte:344` | idem | b |
| D6 | `lib/features/invoices/SettleInvoiceDialog.svelte:250` (aussi monté par `invoices/[id]` et `invoices/due-dates`) | idem | b |
| D7 | `lib/features/bank-import/BankImportUpload.svelte:212` | idem, format harmonisé (C-26-2-5) | b |
| D8 | `lib/features/homepage/BankAccountsCard.svelte:38` | `bankAccountName` (AC9) | b |
| D9 | `routes/(app)/settings/+page.svelte:470` | `bankAccountName` (AC9) | b |
| D10 | serveur : message de `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT` (`crates/kesh-api/src/errors.rs:3994-4023`, alimenté par `repositories/bank_accounts.rs:730`) | `bank_account_display_name` (AC10) | b |

*Porteurs de données (types, parse, JSON) — étendus, ils n'affichent rien :* `bank-accounts.api.ts:16, 57, 75,
124, 148` (a) ; `reconciliation/+page.svelte:11`, `bank-import/+page.svelte:10`, `BankImportUpload.svelte:24`,
`settings.types.ts:44` (b) ; côté serveur `BankAccount`, `BankAccountJson` (`companies.rs:318`),
`serialize_bank_accounts_csv` (`csv_tables.rs:633, 648`), l'audit (`routes/bank_accounts.rs:666, 674`).

*Exclus — saisie de la banque, pas affichage d'un compte (liste fermée de la garde AC11) :*

- **E1** `routes/(app)/bank-accounts/+page.svelte:152, 209, 233` — préremplissage et payloads du formulaire ;
- **E2** `routes/onboarding/+page.svelte:121, 168, 174, 373`, `onboarding.svelte.ts:134, 137`,
  `onboarding.api.ts:64, 69` — saisie à l'onboarding, sans description (C-26-2-2) ;
- **E3** profils d'import, **homonymes et non comptes bancaires** : `BankProfileList.svelte:68`,
  `BankProfileForm.svelte:26, 68, 103`, `BankProfileSelector.svelte:53`, `bank-profile.types.ts:14` (le champ
  `bank_profiles.bank_name` est le nom du profil CSV) ;
- **E4** côté serveur, mêmes homonymes : `routes/bank_imports.rs:738, 1481, 1578, 1671`,
  `routes/bank_profiles.rs`, `errors.rs:772-773, 1083` (`BankProfileSummary`), `csv_tables.rs:1046, 1062`,
  `kesh-import/src/csv/*`, `repositories/bank_profiles.rs`, `entities/bank_profile.rs`.

*Détecteur 2 (IBAN affiché)* : `settings/+page.svelte:471`, `bank-accounts/+page.svelte:425`,
`BankAccountsCard.svelte:39` (`shortIban`) — l'IBAN **à côté** du nom de D9, D1, D8 : inchangés.
`supplier-invoices/[id]/+page.svelte:276` et `supplier-invoices/import/+page.svelte:443` montrent l'IBAN **du
créancier**, pas un compte de la société : hors sujet. Les sites `— {…iban}` sont D2-D7.

*Détecteur 3 (consommateurs)* — ne montrent pas de compte : `account-options.ts` (filtre),
`ReconciliationProposals.svelte` (lit `journalAccountId`), `ManualMatchModal.svelte`, `TransactionSplitModal.svelte`
(id seul), `routes/(app)/+page.svelte` (passe la liste à D8), `invoices/[id]` et `invoices/due-dates` (passent la
liste à D6), `settings/invoicing/+page.svelte` (filtre d'ids). **Angles morts assumés** (C-26-2-12) :
`BankImportList.svelte`, `routes/(app)/bank-import/[id]/+page.svelte`, `routes/(app)/payment-batches/[id]/+page.svelte`
n'affichent **aucun** compte — la story ne leur en ajoute pas.

*Détecteur 4 (serveur)* : hors E4 et des porteurs, un seul site d'affichage — D10. `kesh-qrbill`, `kesh-payment`,
`kesh-report`, `kesh-reconciliation`, `kesh-core`, `kesh-seed` : **aucune** occurrence (le QR-Bill prend l'IBAN du
compte principal par `find_primary`, jamais son nom ; le pain.001 lit `iban` seul,
`repositories/payment_batches.rs:714`). L'audit de l'onboarding (`routes/onboarding.rs:643`) photographie le compte
sans la description : la description n'y change pas (C-26-2-2).

*Documentation* : le manuel utilisateur n'a **aucune** section sur *Administration → Comptes bancaires* (AC12 en
crée une) ; `docs/api-external.md` l. 218 liste les routes sans leurs champs — rien à y ajouter ;
`docs/optimistic-locking-patterns.md` l. 46 décrit les colonnes de `upsert_primary` — inchangées (C-26-2-7).

**Total** : 10 sites d'affichage résolus (D1-D10), 0 non résolu ; 3 angles morts assumés (écrans sans compte) ;
la saisie à l'onboarding exclue par choix.

### État actuel des fichiers modifiés — ce qui doit être préservé

- **`routes/bank_accounts.rs`** : ordre des erreurs documenté (forme → compte lié → sentinelle → créance →
  démotion du principal → écriture) ; la validation de forme de la description s'insère **dans** l'étape
  « forme », avant tout accès base. Transitions de principal, audit dans la même transaction, garde
  `assert_onboarding_complete` : intacts.
- **`upsert_primary_in_tx`** : court-circuit KF-004 sur `(bank_name, iban, qr_iban)` — ne pas y ajouter la
  description (elle n'est pas dans le payload de l'onboarding, la comparer rendrait `Updated` à chaque appel).
- **`first_active_bank_account_linked_to`** : verrou `LOCK IN SHARE MODE` (pas `FOR SHARE`, MariaDB 10.11) et
  ordre des verrous `companies → réglages → bank_accounts` — on n'ajoute qu'une colonne au `SELECT`.
- **Sélecteurs** : `data-testid` inchangés (`bank-account-select`, `settle-bank`, `pay-bank-account`) ; seules
  les **valeurs** d'`<option>` changent de texte. Aucun E2E existant ne sélectionne un compte bancaire par
  libellé (`grep -rn "selectOption({ *label" frontend/tests/e2e` → un seul site, sur un compte du plan comptable).
- **Écran *Comptes bancaires*** : modes exclusifs (`create`, `edit-full`, `edit-journal`, `archive-confirm`),
  validation client du QR-IBAN (#155) — le champ neuf ne s'en mêle pas.

### Fixtures de test à compléter (`description: null`)

Vitest — fichiers qui construisent des comptes bancaires : `routes/(app)/supplier-invoices/[id]/supplier-settlement-page.test.ts`,
`routes/(app)/bank-accounts/bank-accounts-page.test.ts`, `lib/features/reconciliation/ReconciliationProposals.test.ts`,
`lib/features/bank-accounts/bank-accounts.api.test.ts`, `lib/features/bank-accounts/BankAccountJournalLinkForm.test.ts`,
`lib/features/homepage/HomepageCards.test.ts`, `lib/features/homepage/homepage.test.ts`,
`lib/features/invoices/SettleInvoiceDialog.test.ts`. Rust : les littéraux `BankAccount { … }` de
`invoice_pdf_service.rs` (l. 659, 845). Les `INSERT INTO bank_accounts (…)` des tests (sans la colonne) restent
valides : elle est nullable.

### Coordination avec les autres stories de l'epic

- **26-1** (#620, #621) peut ajouter une migration (`bank_profiles`) et régénérer le squash : collisions sur le
  squash, la ligne d'audit, ses compteurs et `migrations_upgrade_path.rs` — résolues par **celle qui fusionne la
  seconde** (C-26-2-14), en recomptant, jamais en additionnant de confiance.
- **26-3** (#616, #617, #619) réécrit deux lignes de la section *Import bancaire* du manuel, voisine de la section
  neuve (C-26-2-13) ; les deux régénèrent `user-manual.pdf` (conflit binaire : régénérer après rebase).
- **CHANGELOG** : les trois stories écrivent dans `## [0.13.1] — Non publié` — union.

### Pièges connus du dépôt, applicables ici

- **Story `kesh-db`** : gate complet même en boucle de revue (migration, dépôt) ; base de gate remise à zéro avant
  chaque gate complet ; P8 (ne plus toucher la migration une fois appliquée à `kesh_e2e_<clé>`).
- **Mutation restaurée, binaire périmé** : après `cp`/`mv` d'une mutation, `touch` le fichier.
- **Greper la valeur, pas la formulation** : après chaque patch, greper `\b78\b`, `\b44\b` (upgrade path), `60`
  (longueur : migration, validation, `maxlength`, i18n, manuel), `bank_name` (renommage `bank_label`).
- **Manuel** : contrôler le PDF aplati, pas seulement le `.tex`.
- Montage E2E : `KESH_COOKIE_SECURE=false`, `KESH_HOST=127.0.0.1`, `KESH_TEST_MODE=true`,
  `PLAYWRIGHT_HOST_PLATFORM_OVERRIDE=ubuntu24.04-x64`, port et bases propres au worktree (consignes de l'epic).

### Project Structure Notes

- Fonctions de libellé dans le dossier propriétaire (`features/bank-accounts/format.ts`) — les autres dossiers
  l'importent, comme ils importent déjà `shortIban`. Pas de module `shared/` neuf.
- Fonction serveur dans l'entité (`kesh_db::entities::bank_account`), pas dans `kesh-core` (formatage d'affichage
  d'une entité persistée, sans règle métier).
- Clés i18n `bank-accounts-*` : propriété du dossier `bank-accounts` (route et feature), conforme à
  `lint-i18n-ownership`.

### References

- Issue #622 et son commentaire de décision (`gh issue view 622 --comments`).
- `_bmad-output/planning-artifacts/epic-26-recette-v0-13.md` — § 26-2, décision 4.
- `crates/kesh-db/migrations/20260410000001_bank_accounts.sql`, `20260507200001_bank_account_journal_link.sql`,
  `20260531000001_bank_accounts_archived.sql` (patron `ADD COLUMN … ALGORITHM=INSTANT, LOCK=NONE`).
- `crates/kesh-db/src/backup.rs` (export générique, `is_required`), `crates/kesh-db/test-schema/README.md`.
- `CLAUDE.md` § *Migration breaking policy* (P1-P8), § *Inventorier les sites NON RÉSOLUS*, § *Règle de splitting
  préventif*, § *Test Locally First*.
- Registre : `_bmad-output/implementation-artifacts/epic-26-choix-autonomes.md`, C-26-2-1 à C-26-2-14.

## Dev Agent Record

### Agent Model Used

(à remplir par le développeur)

### Debug Log References

### Completion Notes List

- Fiche créée le 2026-10-10 (`bmad-create-story`, autonome) : analyse exhaustive du code sur `fe162504` —
  inventaire clos des sites, choix C-26-2-1 à 14 au registre de l'epic.

### File List

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-10 | `bmad-create-story` | Fiche créée : 14 AC, inventaire clos (10 sites d'affichage, 3 angles morts assumés), découpage 26-2a / 26-2b proposé (C-26-2-1). |
