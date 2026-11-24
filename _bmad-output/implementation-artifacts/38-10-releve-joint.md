# Story 38-10 : Relevé joint à la main — solde de clôture, écart aux contrôles de clôture

Status: deferred

*(2026-11-20 — reportée en phase 2 par la décision du responsable de l'epic (phase 1 KISS, § « Phase 1 KISS » du fichier d'epic) ; validée le 2026-11-15, gardée telle quelle comme base de la phase 2. Ce qu'en reprend la phase 1 est dit dans les fiches 38-11a à 38-11d.)*

- **Epic** : 38 (`_bmad-output/planning-artifacts/epic-38-paypal.md`, § *38-10*, E17, E18, E20 (b), C-38-10) ; décisions du
  responsable de l'epic B10 et B20 (`epics-38-40-cadrage.md`), qui priment.
- **Dépend de** : **38-2** et **38-3a** (ordre de l'epic : 38-10 « dès 38-3a » — rien de 38-3b n'est lu ; la story elle-même vaut pour **tout** compte
  bancaire, prestataire ou non — C-38-10-1) ; Epic 37 : devise d'un compte (37-1), solde en devise (37-4 AC-1, fragment
  de `repositories/account_balances.rs`), liste close des absences admises à l'import (37-1 AC-6, R1).
- **Prépare** : Epic 39 (Wise y range le PDF obtenu par l'API, `source = 'provider_api'`), Epic 40 (rapports Stripe joints
  à la main).
- **Migration** : **une, non breaking** — `CREATE TABLE bank_account_documents` (NFR-38-10-3).

## Story

En tant que **Comptable**,
je veux **joindre le relevé PDF d'un compte bancaire** (le relevé annuel PayPal, qui n'a pas d'API) à son compte et à son
exercice, **avec le solde de clôture qu'il annonce**,
afin que **ce solde serve de somme de contrôle** : Kesh me montre l'écart avec ses livres, et me le rappelle avant de
clôturer l'exercice.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-38-10-1` | Joindre un relevé PDF à un compte bancaire, pour un exercice ou une période. | AC-2, AC-3 |
| `FR-38-10-2` | Saisir le solde de clôture et la devise du relevé (facultatif). | AC-2, AC-4 |
| `FR-38-10-3` | Afficher l'écart entre ce solde et le solde comptable à sa date. | AC-5, AC-9 |
| `FR-38-10-4` | Ajouter cet écart aux contrôles de clôture de l'exercice (avertissement non bloquant). | AC-6 |
| `FR-38-10-5` *(ajoutée)* | Lister, télécharger, corriger et retirer une pièce jointe ; corriger ou retirer est refusé dans un exercice clôturé. | AC-3, AC-4 |
| `NFR-38-10-1` | PDF seul, taille bornée, fichier rangé par `document_storage`. | AC-2 |
| `NFR-38-10-2` | Pièces emportées et restaurées par la sauvegarde complète. | AC-7 |
| `NFR-38-10-3` | Migration non breaking. | AC-1 |
| `NFR-38-10-4` *(ajoutée)* | Gestes tracés au journal d'audit ; quatre locales ; manuel utilisateur et PDF à jour ; captures de chaque état. | AC-8, AC-9, AC-10 |

## Ce que le code dit (prémisses vérifiées le 2026-11-15)

- **Stockage des pièces** : `document_storage::store_document(dir, bytes, ext, original_filename, mime_type) ->
  DocumentMeta` (`crates/kesh-api/src/document_storage.rs:90-131`) range sous `{sha256}.{ext}` (nom d'origine jamais dans
  un chemin, `:1-6`), refuse une extension non sûre et un contenu vide (`:97-110`), écrit atomiquement (`write_atomically`, `:166-190`) ;
  `read_document` distingue l'absent (`NotFound`, rendu 410) de l'erreur d'E/S (`:142-157`). Le téléchargement d'un
  document fournisseur (`serve_document`, `crates/kesh-api/src/routes/imported_supplier_invoices.rs:733-750`) pose
  `Content-Type` et un `Content-Disposition` robuste aux accents (`content_disposition`, `:768`), et rend
  `SOURCE_DOCUMENT_GONE` (410) pour un fichier absent (`crates/kesh-api/src/errors.rs:3506`). Le PDF se reconnaît à
  `%PDF` (`crates/kesh-api/src/inbox_import.rs:900`).
- **Taille** : l'inbox borne un fichier à 25 Mio par défaut (`crates/kesh-api/src/config.rs:979-1000`,
  `KESH_INBOX_MAX_FILE_BYTES` ; `:530` est la configuration de test) ; les routes
  d'import bancaire posent leur borne par `DefaultBodyLimit::max` (`crates/kesh-api/src/lib.rs:832-838`) et lisent le
  multipart par `parse_multipart` / `map_multipart_err` (`crates/kesh-api/src/routes/bank_imports.rs:448`, `:872`).
- **La sauvegarde emporte les pièces que la base désigne** : `read_referenced_files` (`crates/kesh-db/src/backup.rs:423`)
  lit **deux** sources (`imported_supplier_invoices`, `invoices.pdf_storage_path`) ; `ParsedBackup::referenced_files`
  (`crates/kesh-api/src/admin_backup/import.rs:115-140`) fait de même côté import, et tolère une table ou une colonne
  absente (`:109-114`, `:121-127`) ; le test `colonnes_storage_path_inventaire_clos` (`backup.rs:2456-2490`) **rougit** sur toute
  colonne `*storage_path*` neuve. ⚠️ `read_referenced_files` lit ses sources **sans condition** (deux `SELECT` fixes,
  `:423-450`), dans `export_installation_in` (`:558-584`) — que la **sauvegarde avant mise à jour** appelle avec
  `Inventaire::SchemaPresent` (`crates/kesh-api/src/pre_upgrade_backup.rs:299`), c'est-à-dire **sur le schéma de la
  version précédente, avant les migrations** : une source qui lit une table neuve y échoue (table absente).
- **Export CSV** : la garde `chaque_colonne_du_schema_est_exportee_ou_ecartee`
  (`crates/kesh-api/src/exports/csv_tables.rs:2377`) ne voit que les tables de **sa liste écrite à la main** (`:2403…`)
  et exige que chaque colonne absente de l'en-tête soit à `COLONNES_HORS_EXPORT` (`:2558`) avec son motif.
- **Tables** : `TABLES_TO_TRUNCATE` ordonne enfants → parents (`backup.rs:46`, `bank_transactions`/`bank_imports`/
  `bank_accounts` `:100-104`, `fiscal_years` `:109`) ; `RESET_PRESERVED_TABLES` (`:137-145`) ne contient pas les tables
  bancaires (vidées à la remise à zéro). L'export de souveraineté exige qu'une table soit dans `TABLES_EXPORTEES` ou
  `TABLES_HORS_EXPORT` (`crates/kesh-api/src/exports/global.rs:100-120`, garde `export_couvre_toutes_les_tables`).
- **Aucune suppression** d'exercice (`crates/kesh-api/src/routes/fiscal_years.rs:18`) ni de compte bancaire (archivage
  seul, `crates/kesh-api/src/routes/bank_accounts.rs:16`, `:1583-1640`) : la question de l'epic « suppression d'un
  exercice ou d'un compte » n'a pas d'objet (FK `RESTRICT` suffit).
- **Écran** : il n'existe **pas** d'écran de détail d'un compte bancaire — seulement la liste
  (`frontend/src/routes/(app)/bank-accounts/+page.svelte`, 1 295 lignes ; `frontend/src/lib/features/bank-accounts/`).
- **Contrôles de clôture** : `closing_checks::for_fiscal_year` rend **exactement huit** contrôles, K8 ajouté **en
  dernier** pour ne pas déplacer K1–K7 (`crates/kesh-db/src/repositories/closing_checks.rs:1-20`) ; K3
  `BANK_STATEMENT_AT_YEAR_END` ne compare que les relevés **à solde** (`bank_imports.closing_balance`, CAMT) et classe
  sinon le compte `NO_BALANCE_STATEMENT` (`:453-480`, `:692-770`) ; l'écart est celui de `bank_accounts::statement_gap`
  (chaque solde arrondi au centime avant soustraction, `:93-99` ; `repositories/bank_accounts.rs:2019-2024` : **livres −
  relevé**, `None` si nul ; `docs/api-external.md:266` : « le même que `statementGap` de `GET /bank-accounts` ») ; toutes les
  requêtes du module sont des constantes de `ALL_SQL`, sans mot de verrou (garde T-a2, `:30-40`, `:550`) ; les fonctions
  de dépôt **externes** qu'il appelle sont un angle mort assumé de T-a2, **listé** au doc du module (`:51-60`) et vérifié
  en revue. Consommateurs du nombre : `crates/kesh-db/tests/closing_checks.rs:1679` (`len(), 8`), `docs/api-external.md:266-267` (« toujours
  huit »), `docs/manual/fr/user-manual.tex:890` (« huit contrôles préalables »), `docs/manual/fr/admin-manual.tex:1866`
  (« les huit contrôles »), le front
  (`frontend/src/lib/features/fiscal-years/closing-checks-labels.ts`, `FiscalYearChecks.svelte`, `fiscal-years.types.ts`,
  `closing-checks.test-fixtures.ts`).

## Écarts à l'epic

1. **« Onglet *Relevés* de l'écran du compte bancaire »** (§ 38-10 de l'epic) : cet écran n'existe pas (prémisse). Les
   pièces se gèrent dans un **dialogue** ouvert depuis la ligne du compte, dans la liste *Comptes bancaires* (AC-9). Si
   38-9a crée une fiche de compte, le dialogue s'y monte tel quel.
2. **« Suppression d'un exercice ou d'un compte (refus tant qu'une pièce y est rangée, ou suppression de la pièce : à
   trancher) »** : sans objet, aucun des deux ne se supprime (prémisse) ; FK `ON DELETE RESTRICT` par principe.
3. **Pas de restriction aux comptes prestataires** : le relevé joint vaut pour tout compte bancaire (une banque sans CAMT,
   une carte de crédit dont le PDF n'avait aucun moyen d'être joint — C-e341-26 de 34-11b). Rien dans l'epic ne le
   restreint ; la fiche le dit.

## Choix

- **C-38-10-1 — Tout compte bancaire.** Non archivé ; clos → les gestes d'écriture sont refusés en `409
  BANK_ACCOUNT_CLOSED` (patron 28-4a), la lecture et le téléchargement restent.
- **C-38-10-2 — Un document = un compte × une période.** `period_from ≤ period_to` ; l'exercice est **celui qui couvre
  `period_to`** (la date du solde de clôture, donc la clôture qu'il contrôle), sinon `400 BANK_DOCUMENT_NO_FISCAL_YEAR`.
  Un relevé à cheval sur deux exercices est rangé dans celui de sa fin. Le même PDF peut se joindre à plusieurs comptes
  (le relevé PayPal couvre trois soldes : un document par compte, chacun avec le solde de sa devise) ; jamais deux fois
  au même compte (`UNIQUE (bank_account_id, sha256)`, `409 BANK_DOCUMENT_DUPLICATE`).
- **C-38-10-3 — La devise est celle du compte.** `closing_currency` = la devise du compte du grand livre lié (37-1 ; CHF
  pour un compte CHF ou non lié) ; un corps qui en porte une autre → `400 BANK_DOCUMENT_CURRENCY_MISMATCH`. `closingCurrency`
  absent avec `closingBalance` présent → la devise du compte. Les deux colonnes sont tout-ou-rien (`CHECK`).
- **C-38-10-4 — L'écart se calcule à la lecture.** Solde comptable du compte du grand livre lié à `period_to` inclus,
  **par une seule fonction pour les deux cas** — `balances_as_of(…, Some(&[compte]))` rend `chf_net` et `devise_net` :
  en CHF, `chf_net` = `SUM(debit) − SUM(credit)` (convention de `BALANCES_SQL`, signe libre, une dette de carte est
  négative) ; en
  devise, le **solde en devise** de 37-4 (AC-1 : `Σ amount_currency` au débit − au crédit, par
  `kesh_db::repositories::account_balances::balances_as_of(conn, company_id, as_of, Some(&[compte]))`, 37-4 AC-1 quater :
  borne inclusive, débit net brut CHF et en devise, réévaluations sans effet) — **la même fonction que le contrôle de
  solde de 38-9a (AC-8 b)**, jamais une seconde somme. Écart = règle de `statement_gap` (deux soldes arrondis
  au centime, puis différence ; `null` si nul) — par la même fonction ou une fonction commune extraite, jamais une
  seconde règle. Compte sans grand livre lié, ou grand livre partagé (`bank_accounts::is_shared`) → `notComparable: true`,
  écart `null`.
- **C-38-10-5 — Contrôle de clôture : K10 en dernier.** *(Décision du 2026-11-16 : l'Epic 37, qui sort avant celui-ci,
  prend K9, « comptes en devise non réévalués », bloquant ; le relevé joint passe de K9 à K10 et le nombre de contrôles
  de neuf à dix. Les nombres « huit » des sites existants décrivent l'état avant l'Epic 37 : relus après la reprise de
  `main`, ils valent neuf, et cette story les passe à dix.)* Un dixième contrôle `ATTACHED_STATEMENT_GAP` (`warning`),
  ajouté **après** K9 pour que K1–K9 gardent leur rang (précédent v017-4a) : pour chaque compte bancaire non archivé ayant au
  moins un document à solde dont `period_to` tombe dans Y, le **dernier** (plus grand `period_to`, puis plus grand `id`)
  est comparé (C-38-10-4) ; écart non nul → compte en `attention`. K3 est **inchangé** (un compte sans CAMT y reste
  `NO_BALANCE_STATEMENT`) : écrire au manuel que K10 est le contrôle des comptes sans relevé importé.
- **C-38-10-5 bis — Exercice clos.** Joindre une pièce à un exercice clos est **admis** (la pièce justifie, elle n'écrit
  rien au journal) ; la **corriger** ou la **retirer** est refusé (`400 FISCAL_YEAR_CLOSED`, code existant,
  `errors.rs:1887`) — la pièce d'une clôture faite ne bouge plus (CO 958f).
- **C-38-10-6 — Colonnes à domaine ouvert (E4).** `kind` (`statement` seul aujourd'hui) et `source` (`manual` ; Wise :
  `provider_api`) sont des `VARCHAR(32)` **sans `CHECK`**, lus en enum à variante `Other(String)` : les Epics 39 et 40
  n'ont ni migration breaking ni relèvement à faire.
- **C-38-10-7 — Le fichier reste sur le disque au retrait.** Le stockage est adressé par contenu et peut être partagé
  (même PDF sur trois comptes, ou pièce fournisseur identique) ; retirer une ligne ne supprime aucun fichier. Un fichier
  orphelin n'est ni sauvegardé ni restauré (la sauvegarde ne lit que les fichiers désignés), comme pour une pièce
  fournisseur écartée.

## Critères d'acceptation

1. **AC-1 — Migration (NFR-38-10-3).** Fichier `crates/kesh-db/migrations/<AAAAMMJJ>000001_bank_account_documents.sql`
   (date au développement, postérieure à **toutes les migrations présentes dans la branche de l'epic** à ce moment —
   38-10 peut se développer « dès 38-3a », avant 38-4, 38-5 ou 38-6c2 — et de version **distincte** de celles que toutes
   les autres fiches de l'epic écrivent en `<date>000001` (38-1, 38-2, 38-3a, 38-6c2, 38-4, 38-5) ; collision vérifiée par `ls
   crates/kesh-db/migrations/` au commit, citée au Change Log) : `CREATE TABLE
   bank_account_documents` — `id BIGINT AUTO_INCREMENT PK`, `company_id` (FK `companies`, `RESTRICT`), `bank_account_id`
   (FK `bank_accounts`, `RESTRICT`), `fiscal_year_id` (FK `fiscal_years`, `RESTRICT`), `kind VARCHAR(32) NOT NULL DEFAULT
   'statement'`, `source VARCHAR(32) NOT NULL DEFAULT 'manual'`, `period_from DATE NOT NULL`, `period_to DATE NOT NULL`,
   `storage_path VARCHAR(255) NOT NULL`, `sha256 CHAR(64) NOT NULL`, `original_filename VARCHAR(255) NOT NULL`, `mime_type
   VARCHAR(100) NOT NULL`, `byte_size BIGINT NOT NULL`, `closing_balance DECIMAL(19,4) NULL`, `closing_currency CHAR(3)
   NULL`, `created_by_user_id` (FK `users`, `RESTRICT`), `created_at`, `updated_at`, `version INT NOT NULL DEFAULT 1` ;
   `UNIQUE (bank_account_id, sha256)` ; `INDEX (company_id, fiscal_year_id)` ; `CHECK (period_to >= period_from)`, `CHECK
   (byte_size > 0)`, `CHECK ((closing_balance IS NULL) = (closing_currency IS NULL))` ; `ENGINE=InnoDB`, jeu de caractères
   des tables voisines. Aucune écriture de données (P7 sans objet : `every_data_backfill_migration_is_triaged` reste
   vert sans entrée), pas de relèvement de `kesh_version_min_required` (P1–P3 : `CREATE TABLE`). **P5** : une ligne au
   tableau de `docs/migrations-idempotence-audit.md` (verdict `tracked-by-sqlx`, DDL), en-tête `## Table d'audit (N
   migrations)` et ligne `Total` recomptés (`ls crates/kesh-db/migrations/*.sql | wc -l`, `grep -c '^| \`20'`), compteurs de
   partition recomptés. **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` exécuté, chaque site
   inspecté, résultat au Change Log. **P8** : aucune migration existante touchée. **Squash** :
   `crates/kesh-db/test-schema/0001_schema_squash.sql` reçoit la table (garde `test_schema_guard.rs` verte).
   **Montée** : `migrations_fresh_install` et la recette de montée (`scripts/recette-montee.sh`) passent.
2. **AC-2 — Joindre (FR-38-10-1, FR-38-10-2, NFR-38-10-1).** `POST /api/v1/bank-accounts/{id}/documents` (Comptable+,
   monté avec les mutations de `/bank-accounts` ; clés d'API `read-write` admises, `read` refusées par le middleware
   existant), `multipart/form-data` : `file` (exigé), `periodFrom`, `periodTo` (`AAAA-MM-JJ`, exigés), `closingBalance`
   (chaîne décimale signée, ≤ 4 décimales, facultative), `closingCurrency` (facultatif). Borne `DefaultBodyLimit::max`
   à `BANK_DOCUMENT_MAX_BYTES + 64 Kio` (constante neuve, **25 Mio**, comme l'inbox) ; fichier au-delà → `413
   BANK_DOCUMENT_TOO_LARGE`. Contrôles, dans l'ordre (une raison, rien d'écrit avant le dernier) : compte de la société,
   non archivé (`404 BANK_IMPORT_BANK_ACCOUNT_NOT_FOUND`, variante existante `AppError::BankAccountNotFound`) ; non clos
   (`409 BANK_ACCOUNT_CLOSED`) ; champs (`400 VALIDATION_ERROR`, champ nommé) ; fichier non vide qui commence par
   `%PDF-` (`400 BANK_DOCUMENT_NOT_PDF` — ni l'extension ni le type annoncé ne suffisent ; le fichier est rangé avec
   l'extension `pdf` et le type `application/pdf`) ; période (`400 BANK_DOCUMENT_INVALID_PERIOD`) ; exercice couvrant
   `periodTo` (`400 BANK_DOCUMENT_NO_FISCAL_YEAR`) ; devise (`400 BANK_DOCUMENT_CURRENCY_MISMATCH`) ; doublon (`409
   BANK_DOCUMENT_DUPLICATE`, `details.documentId`). Le fichier est écrit par `store_document` **avant** l'`INSERT` (un
   échec d'`INSERT` laisse un fichier orphelin inoffensif, C-38-10-7) ; nom d'origine tronqué à 255 octets comme
   `bank_imports.rs:365`. Réponse `201` : le document (AC-3). Trace `bank_account_document.attached`.
3. **AC-3 — Lister et télécharger (FR-38-10-5).** `GET /api/v1/bank-accounts/{id}/documents?fiscalYearId` (tout rôle
   authentifié, monté avec `GET /bank-accounts`, `lib.rs:1228-1231` ; clés `read` admises) → `{ documents: [{ id,
   bankAccountId, fiscalYearId, fiscalYearName, kind, source, periodFrom, periodTo, originalFilename, byteSize,
   closingBalance, closingCurrency, ledgerBalanceAtPeriodTo, gap, notComparable, createdAt, createdBy, version }] }`,
   triés par `periodTo` décroissante puis `id` ; `kind`/`source` inconnus rendus tels quels (C-38-10-6) ; `storage_path` et
   `sha256` ne sont jamais rendus. `GET /api/v1/bank-accounts/{id}/documents/{docId}` (mêmes rôles) → le PDF
   (`Content-Type: application/pdf`, `Content-Disposition` par la fonction de `imported_supplier_invoices.rs`, rendue
   partagée — DRY —, `X-Content-Type-Options: nosniff`) ; ligne absente ou d'un autre compte ou d'une autre société → `404
   BANK_DOCUMENT_NOT_FOUND` ; fichier absent du disque → `410 BANK_DOCUMENT_GONE`. Tests : isolation par société
   (KF-002 : même `docId` d'une autre société → 404).
4. **AC-4 — Corriger et retirer (FR-38-10-2, FR-38-10-5).** `PATCH /api/v1/bank-accounts/{id}/documents/{docId}`
   (Comptable+) `{ periodFrom?, periodTo?, closingBalance?, closingCurrency?, version }` — `closingBalance: null` retire
   le solde (et la devise) ; mêmes contrôles qu'AC-2 sur les champs présents ; l'exercice se recalcule sur `periodTo`.
   `DELETE /api/v1/bank-accounts/{id}/documents/{docId}?version=` (Comptable+) → `204`, la ligne seule (C-38-10-7).
   Refus : `404 BANK_DOCUMENT_NOT_FOUND`, `409 BANK_ACCOUNT_CLOSED`, `409 OPTIMISTIC_LOCK_CONFLICT` (version), `400
   FISCAL_YEAR_CLOSED` si l'exercice **actuel** ou **visé** du document est clôturé (C-38-10-5 bis). Traces
   `bank_account_document.updated` (avant/après des champs changés) et `bank_account_document.deleted` (métadonnées).
5. **AC-5 — Écart au solde comptable (FR-38-10-3).** Tests d'intégration : compte CHF lié à 1020, écritures jusqu'au
   31.12.2024, document au 31.12.2024 avec le solde comptable exact → `gap: null` ; solde du relevé **supérieur** de
   12.34 aux livres → `gap: "-12.34"` (signe : **livres − relevé**, comme `statement_gap`,
   `repositories/bank_accounts.rs:2019-2024`, et comme `gap` de K3 et `statementGap` de la liste) ; relevé inférieur de
   12.34 → `"12.34"` ; compte PayPal EUR lié à un compte du plan EUR (37-1),
   lignes en devise → l'écart se lit **en EUR** sur le solde en devise (une réévaluation au 31.12 ne le change pas) ;
   carte de crédit liée à un passif endetté : solde saisi négatif, écart nul ; compte non lié ou grand livre partagé →
   `notComparable: true`, `gap: null` ; document sans solde → `ledgerBalanceAtPeriodTo` rendu, `gap: null`. Lectures sans
   verrou ; une lecture par page de documents (pas une par document : regroupées par compte et date, test du compteur
   ou justification au Change Log).
6. **AC-6 — Contrôle de clôture K10 (FR-38-10-4).** `GET /api/v1/fiscal-years/{id}/closing-checks` rend **dix**
   contrôles (K9 est celui de l'Epic 37, repris de `main` : le nombre se relit à la source après chaque reprise) ; le dixième, en dernier, `ATTACHED_STATEMENT_GAP`, `severity: warning`, `status` `attention` si au moins un
   compte a un écart (C-38-10-5), `count` = nombre de comptes en écart, `details: { accounts: [{ bankAccountId, name,
   documentId, periodTo, currency, statementClosingBalance, ledgerBalanceAtStatement, gap }], notComparable: [{
   bankAccountId, name }] }` (comptes non comparables sans effet sur le statut, comme K3). La clôture **n'est pas
   bloquée** (`fiscal_years::close` inchangé). Requêtes : constantes ajoutées à `ALL_SQL`, sans verrou (garde T-a2
   verte), dans l'instantané de la route ; le solde comptable vient de `account_balances::balances_as_of` (C-38-10-4),
   appelée sur la connexion de l'instantané — fonction **externe**, donc ajoutée à la liste de l'angle mort de T-a2 au
   doc du module (`closing_checks.rs:51-60`), son SQL relu sans verrou en revue. Mises à jour du nombre (prémisse, **tous** les sites) : doc du module (« dix »,
   table, `count`), `ClosingChecks::checks` (« toujours dix »), `CheckCode`/`CheckDetails`/`Severity` (variant neuf),
   `crates/kesh-db/tests/closing_checks.rs:1679`, `crates/kesh-api/tests/closing_checks_e2e.rs`, `docs/api-external.md`
   (« toujours dix », contrôle décrit, changement de contrat annoncé comme la ligne `:267` l'exige), front
   (`closing-checks-labels.ts`, `FiscalYearChecks.svelte` avec lien vers le dialogue des relevés du compte,
   `fiscal-years.types.ts`, `fiscal-years.api.ts`, `closing-checks.test-fixtures.ts`, `FiscalYearChecks.test.ts`,
   **`frontend/src/routes/(app)/settings/fiscal-years/+page.svelte` et `fiscal-years-page.test.ts:461`
   (`toHaveLength(8)`)**, `i18n-keys.test.ts` et `i18n-libelle-en-dur.test.ts` si les libellés du contrôle y entrent),
   **E2E `frontend/tests/e2e/fiscal-year-closing-journey.spec.ts:158,165` (`toHaveCount(8)` : attend huit lignes) et
   `frontend/tests/e2e/fiscal-years.spec.ts:499-500` (même assertion, commentaire « huit contrôles, K8 en dernier »)**,
   `fiscal-years-page.test.ts:447` (titre « les huit lignes »), `FiscalYearChecks.test.ts:48`, doc-comments
   `closing_checks.rs:123` (« Toujours huit ») et `:566`, `docs/manual/fr/marketing-brochure.tex:437` (« Huit contrôles
   préalables… » devenu « Neuf » par l'Epic 37, « Dix » ici ; `.tex` seul, PDF non commité),
   `user-manual.tex:890`, `admin-manual.tex:1866` ; grep du jeton `\bhuit\b` et `\b8\b` sur ces fichiers (liste relevée par `grep -rln
   "closing_checks\|closing-checks\|DRAFT_SUPPLIER_INVOICES" crates frontend/src frontend/tests docs`), résultat trié au
   Change Log. **Nom** : K10 est ici `ATTACHED_STATEMENT_GAP` ; K9 est celui de l'Epic 37 (« comptes en devise non réévalués »,
   bloquant, qui sort avant : `37-5-reevaluation-fin-exercice.md`, à relire après la reprise de `main`). Tests : écart → `attention` ; aucun document → `ok` ; document daté hors Y
   ignoré ; deux documents dans Y → le dernier compte ; dialogue de clôture (Vitest) qui rappelle K10.
7. **AC-7 — Sauvegarde et restauration (NFR-38-10-2, E18).** (a) `bank_account_documents` entre à `TABLES_TO_TRUNCATE`
   **avant** `bank_transactions` (enfant de `bank_accounts`, `fiscal_years`, `users`, `companies`), donc vidée à la remise
   à zéro de la société — `backup_inventory_matches_schema` vert, et **sites de dénombrement** (même grep que 38-1,
   38-2 et 38-6c2 AC-1, à relire à la source) : `crates/kesh-api/tests/admin_full_export_e2e.rs:~281` (`data_count`,
   44 aujourd'hui, **+1** cumulé avec les tables des autres fiches de l'epic) et `onboarding_audit_e2e.rs:2046`
   (`EMPTY_BEFORE_RESET`, liste fermée : la table y entre) ; (b) `read_referenced_files` lit une **troisième** source (`SELECT storage_path, sha256 FROM
   bank_account_documents`) **seulement si la table est au schéma présent** — lue dans le même instantané
   (`information_schema.TABLES` de `DATABASE()`, comme `tables_du_schema_present`), sinon aucune référence : la
   sauvegarde avant mise à jour (#847) est écrite par le binaire neuf sur le schéma **précédent**, où la table n'existe
   pas encore (prémisse) ; un `SELECT` sans condition y échouerait, et Kesh **refuserait de migrer** à la première mise à
   jour vers cette version. `ParsedBackup::referenced_files` aussi (`("bank_account_documents", "storage_path",
   "sha256")`, déjà tolérant à une table absente), et `colonnes_storage_path_inventaire_clos` liste la colonne neuve.
   **Test** au patron de `pre_upgrade_backup/tests.rs` : sauvegarde avant mise à jour prise sur une base migrée
   jusqu'à la migration **précédant** celle de l'AC-1 (index résolu par version, P6) → écrite, aucune référence de
   relevé ; mutation « condition retirée » → rouge ; (c) la table entre à la **liste close
   des absences admises** de 37-1 AC-6 (R1, `Absence::Table("bank_account_documents")` avec sa version de migration) :
   un `.keshbackup` antérieur se réimporte, table vide ; (d) l'export CSV de souveraineté la porte (`TABLES_EXPORTEES`,
   sérialiseur `serialize_bank_account_documents_csv` : `id, bank_account_id, fiscal_year_id, kind, source, period_from,
   period_to, original_filename, sha256, byte_size, closing_balance, closing_currency, created_by_user_id, created_at` —
   sans `storage_path`), garde `export_couvre_toutes_les_tables` verte ; **la table entre à la liste de
   `chaque_colonne_du_schema_est_exportee_ou_ecartee`** (`csv_tables.rs:2377`, liste écrite à la main : sans cette
   ligne la garde ne la voit pas) et chaque colonne hors de l'en-tête — `company_id`, `storage_path`, `mime_type`,
   `updated_at`, `version` — entre à `COLONNES_HORS_EXPORT` (`:2558`) avec son motif (`storage_path` : patron de
   `invoices.pdf_storage_path`, dérivé de `sha256`), **ou** à l'en-tête (`company_id`, comme `bank_imports`) — choix au
   Change Log. Tests : export complet puis import dans une
   instance neuve → le PDF se télécharge, empreinte identique (`admin_full_import_e2e` étendu) ; la sauvegarde d'avant
   mise à jour (#847, `pre_upgrade_backup`) emporte le fichier ; sauvegarde sans la table → import accepté.
8. **AC-8 — Audit et registre (NFR-38-10-4).** Actions `bank_account_document.attached`, `.updated`, `.deleted` à
   `crates/kesh-api/src/audit_labels.rs` (ordre alphabétique), entité `bank_account_document` ; clés
   `audit-log-action-bank-account-document-*` et `audit-log-entity-bank-account-document` aux quatre locales
   (`tests/audit_label_registry.rs` vert). Registre `LIB_ROUTES` (`crates/kesh-api/tests/audit_route_registry.rs`) :
   `("post", "bank_accounts::attach_document", Traced, SansEcritureAuJournal)`, `("patch",
   "bank_accounts::update_document", …)`, `("delete", "bank_accounts::delete_document", …)` (noms au développement ;
   module `routes/bank_account_documents.rs` admis) — **+3 routes mutantes, +3 tracées** ; les deux `GET` n'y entrent pas.
   Décomptes du manuel d'administration qui citent le registre, recomptés. Codes neufs et leur message aux quatre
   locales : `BANK_DOCUMENT_NOT_PDF`, `BANK_DOCUMENT_TOO_LARGE`, `BANK_DOCUMENT_INVALID_PERIOD`,
   `BANK_DOCUMENT_NO_FISCAL_YEAR`, `BANK_DOCUMENT_CURRENCY_MISMATCH`, `BANK_DOCUMENT_DUPLICATE`, `BANK_DOCUMENT_NOT_FOUND`,
   `BANK_DOCUMENT_GONE`.
9. **AC-9 — Écran (FR-38-10-3, NFR-38-10-4).** Dans *Comptes bancaires*, chaque ligne gagne l'action *Relevés joints*
   (tout rôle ; les gestes d'écriture masqués pour Consultation et sur un compte clos) qui ouvre
   `BankAccountDocumentsDialog.svelte` (`frontend/src/lib/features/bank-accounts/`) : liste (période, exercice, nom du
   fichier, solde de clôture et devise, **écart** au solde comptable — mis en évidence quand non nul, « non comparable »
   sinon), téléchargement, formulaire *Joindre un relevé* (fichier PDF, période, solde facultatif — la devise est
   affichée, celle du compte, non saisie), correction, retrait avec confirmation ; erreurs de l'API traduites par code.
   API cliente dans `bank-accounts.api.ts`. Clés i18n neuves au préfixe **`bank-accounts-`** pour le dialogue
   (`lint-i18n-ownership.js:177-186` : sous `features/bank-accounts/`, nom à tiret → `startsWith('bank-accounts-')`)
   et **`fiscal-years-checks-`** pour le contrôle de clôture (`closing-checks-labels.ts`, patron existant), `error-`
   pour les messages serveur. `+page.svelte` des comptes bancaires (1 295 lignes, même fichier que
   38-9a AC-9) **ne reçoit que l'appel du composant** et le bouton de la ligne, sans logique inline. Le refus du compte
   clos (`BANK_ACCOUNT_CLOSED`) est tenu par les tests de route : `bank_account_closed_inventory.rs` ne lit que les
   sept tables de 28-4a, dont `bank_account_documents` ne fait pas partie (rien à y inscrire, dit au Change Log).
   Vitest : liste, écart, formulaire (champs exigés, PDF seul côté écran),
   rôles, compte clos. **E2E Playwright** (`frontend/tests/e2e/bank-account-documents.spec.ts`) : joindre un PDF de
   fixture avec un solde qui diffère des livres → écart affiché → panneau *Préparer la clôture* : K10 en attention →
   corriger le solde → écart nul, K10 `ok` → télécharger (contenu identique) → retirer. **Recette visuelle** : captures
   (thème clair) de la liste vide, de la liste avec écart, du formulaire, d'une erreur, du panneau de clôture avec K10,
   jointes au Dev Agent Record et au prompt de revue (hors dépôt, `kesh-gate-logs/`).
10. **AC-10 — Manuels et API (NFR-38-10-4).** `docs/manual/fr/user-manual.tex` : section *Joindre un relevé à un compte
    bancaire* (près de *Comptes bancaires* et de *Tenir une carte de crédit*) : pourquoi (le relevé annuel PayPal n'a pas
    d'API ; le solde de clôture est la somme de contrôle), comment (période, exercice choisi par la date de fin, un PDF
    par compte et par devise), l'écart, le contrôle K10, exercice clos (on joint, on ne corrige plus) ; § *Préparer la
    clôture* : « dix contrôles », K10 décrit. `admin-manual.tex` : les relevés joints sont dans le dossier des documents
    et dans la sauvegarde complète ; la sauvegarde d'avant mise à jour les emporte ; « les huit contrôles » (`:1866`,
    répétition d'une clôture ; « neuf » après l'Epic 37) devient « les dix contrôles » ; `marketing-brochure.tex:437` (« Huit contrôles préalables à la clôture… ») devient
    « Dix », avec la mention de la livraison de K10 (numéro de version écrit seulement s'il est connu au développement
    — Q1 de l'epic —, jamais un gabarit `<…>` dans le `.tex`) ; **`README.md:48`** (§ *Fonctionnalités*, « huit contrôles
    en lecture seule avant de clôturer un exercice ») devient « dix contrôles » — le README est un support de la liste
    de contrôle pré-push de `CLAUDE.md`, hors du grep de la P3 (`crates`, `frontend`, `docs`) ; `CHANGELOG.md:486`
    (entrée publiée) reste tel ; `website/` et manuels DE/IT/EN relus sans occurrence (P4). **Les `.tex` seuls sont modifiés ; les PDF se régénèrent à la release** (contrôle
    local du rendu aplati permis — `make fr`, `pdftotext … | tr '\n' ' '` —, PDF non commités). `docs/api-external.md` : les cinq routes, leurs
    codes, K10. **MCP** : aucun outil (E23) — `kesh_pieces` (`mcp/outils/pieces.rs`) liste les pièces fournisseur
    importées et n'est pas étendu ; les relevés joints ne sont pas exposés au serveur MCP (décision de l'epic, aucune
    donnée de prestataire exposée).

## Tâches

- [ ] T1 — Migration, ligne d'audit d'idempotence (P5), grep P6, squash `test-schema` (AC-1).
- [ ] T2 — `kesh-db` : entité `BankAccountDocument` (`kind`, `source` tolérants), dépôt (créer, lister avec soldes
      comptables, lire, modifier avec `version`, retirer), calcul de l'écart par la règle de `statement_gap` (AC-2 à AC-5).
- [ ] T3 — `kesh-api` : routes (`lib.rs`, bornes de corps), multipart, contrôle PDF, `store_document`, téléchargement
      (fonction `content_disposition` partagée), codes d'erreur, audit (AC-2 à AC-4, AC-8).
- [ ] T4 — `closing_checks` : K10, tous les consommateurs du nombre (AC-6).
- [ ] T5 — Sauvegarde : `TABLES_TO_TRUNCATE`, `read_referenced_files` **conditionnée à la présence de la table** et
      son test de sauvegarde avant mise à jour sur le schéma précédent, `ParsedBackup::referenced_files`, inventaire
      clos, absences admises (R1), export CSV et garde de colonnes (AC-7).
- [ ] T6 — Frontend : dialogue, API, i18n, Vitest, E2E, captures (AC-9).
- [ ] T7 — Manuels FR (`.tex` seuls), brochure, `README.md:48` (« dix contrôles »), `api-external.md` (AC-10).
- [ ] T8 — Gate complet **même en cours de boucle** (migration et dépôts touchés) ; runtime : `admin_backup_e2e`,
      `admin_full_import_e2e`, `migrations_fresh_install` ; E2E complet au dernier commit de code.

## Note de version

*(Texte prêt à assembler au `CHANGELOG.md` à la release.)*

- **Ajouté** : on joint à un compte bancaire le relevé PDF d'un prestataire sans API (relevé annuel PayPal), avec son
  solde de clôture par devise ; l'écart avec les livres s'affiche, et un dixième contrôle de clôture (K10, avertissement)
  le signale. Le relevé joint voyage avec la sauvegarde complète. Migration non breaking, mais le retour à la version
  précédente passe par la sauvegarde d'avant la mise à jour.
- **Changé** : l'avant-clôture compte dix contrôles (K9 étant celui des comptes en devise non réévalués).

**Compteurs** (audit des migrations — total aux deux sites et partition —, schéma de test, `LIB_ROUTES`, `data_count`,
`EMPTY_BEFORE_RESET`, nombre de contrôles de clôture et ses assertions front et E2E, `Cargo.lock`) : recomptés depuis la
source après chaque reprise de `main`.

## Dev Notes

- **Pourquoi un K10 et non K3 étendu** : K3 compare au grand livre en CHF les relevés **importés** ; y mêler les relevés
  joints changerait son contrat (`NO_BALANCE_STATEMENT`) et sa règle de précédence, publiés (`api-external.md:266`).
  K10, ajouté en dernier, ne déplace rien (précédent K8, v017-4a).
- **Solde en devise** : ne pas recoder la somme `amount_currency` ; employer le fragment de 37-4
  (`repositories/account_balances.rs`, AC-1 quater de 37-4). Avant la fusion de l'Epic 37, la story n'est pas
  développable (dépendance).
- **Multipart** : reprendre `parse_multipart`/`map_multipart_err` de `bank_imports.rs` (extraire un utilitaire commun si
  la forme diffère trop, sans recopie) ; un champ inconnu → `400 VALIDATION_ERROR`.
- **Sécurité** : le nom d'origine n'entre jamais dans un chemin (`store_document`) ; le téléchargement force
  `application/pdf` et `nosniff` (un fichier commençant par `%PDF-` mais polyglotte ne s'exécute pas comme HTML).
- **Wise (39)** : écrira `source = 'provider_api'` et l'utilisateur de la connexion en `created_by_user_id` ; aucune
  migration de plus.
- **Références** : epic E17, E18, E20 (b), C-38-10 ; 31-1a (pièces dans la sauvegarde) ; 29-1a et v017-4a (contrôles de
  clôture) ; 28-4a (compte clos) ; 34-11b C-e341-26 (PDF de relevé non joignable avant cette story) ; 37-1 AC-6 (R1) ;
  37-4 AC-1.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- 2026-11-15 — validation P2 (Opus) : 7 findings (0 CRITICAL, 1 HIGH, 2 MEDIUM, 4 LOW) ; remédiés dans la fiche, aucun
  inter-fiches. HIGH P2-1 : la troisième source de `read_referenced_files` était un `SELECT` sans condition ; or la
  sauvegarde avant mise à jour lit le **schéma précédent** (`Inventaire::SchemaPresent`, `pre_upgrade_backup.rs:299` ;
  `export_installation_in`, `backup.rs:558-584`, appelle `read_referenced_files` sans condition, `:423-450`) : table
  absente → sauvegarde en échec → **migration refusée** à la première mise à jour vers la release — lecture conditionnée
  à la présence de la table, test sur le schéma précédent, mutation. MEDIUM : P2-2 sens de l'écart faux —
  `statement_gap` est **livres − relevé** (`bank_accounts.rs:2019-2024`), la fiche disait « relevé − livres, comme
  `statement_gap` » ; P2-3 l'export CSV omettait la garde `chaque_colonne_du_schema_est_exportee_ou_ecartee`
  (`csv_tables.rs:2377`, liste à la main) et `COLONNES_HORS_EXPORT` (`:2558`) pour les colonnes non exportées. LOW :
  P2-4 note « K9/K10 » à jour (37-5 aligné, `:543-544`) ; P2-5 `admin-manual.tex:1866` (« les huit contrôles ») ajouté
  aux sites et à l'AC-10 ; P2-6 version de migration : postérieure à la branche au moment du développement, distincte
  des `000001` de 38-4, 38-5, 38-6c ; P2-7 rédaction — `config.rs:530` est la configuration de test (défaut réel
  `:979-1000`), solde CHF et en devise par la seule `balances_as_of` (DRY), fonction externe ajoutée à la liste de
  l'angle mort de T-a2 (`closing_checks.rs:51-60`). Relu au code : `document_storage.rs:90`, `:142`, `:166`,
  `imported_supplier_invoices.rs:733`, `:768`, `errors.rs:1885-1888`, `:3504-3508`, `inbox_import.rs:900`,
  `backup.rs:46`, `:98-112`, `:137`, `:423-450`, `:470-481`, `:558-584`, `:2460`, `import.rs:108-140`,
  `bank_imports.rs:872-885`, `lib.rs:828-862`, `:930-960`, `:1222-1235`, `closing_checks.rs:28-66`, `:88-100`,
  `tests/closing_checks.rs:1676-1681`, `fiscal-years-page.test.ts:459-463`, `fiscal-year-closing-journey.spec.ts:156-166`,
  `api-external.md:264-268`, `user-manual.tex:890`, `admin-manual.tex:1866`. Trend : P1 = 5 (2 MEDIUM) → P2 = 7 (1 HIGH,
  2 MEDIUM). Axes non exercés : code de 37-4 (`balances_as_of`, non écrit) ; PDF du manuel (non régénéré) ; B27/B28
  sans effet sur cette fiche (vérifié : aucun lecteur de groupe, solde lu par `balances_as_of`).
- 2026-11-15 — validation P1 (Sonnet) : 5 findings (0 CRITICAL, 0 HIGH, 2 MEDIUM, 3 LOW) ; prémisses `fichier:ligne`
  toutes relues au code et justes (`document_storage.rs`, `backup.rs`, `import.rs`, `global.rs`, `closing_checks.rs`,
  `errors.rs`, `lib.rs`, `bank_imports.rs`) ; remédiés dans la fiche sauf l'inter-fiche 38-10-P1-2 (K9 de 37-5 → K10).
  Axes non exercés : code de 37-4 (`balances_as_of`, non écrit) ; PDF du manuel.
- 2026-11-15 — rédaction (Opus) : 10 AC, 5 FR (4 de l'epic + 1 ajoutée), 4 NFR (3 + 1) ; une migration non breaking
  (`bank_account_documents`) ; +3 routes mutantes, +2 lectures ; contrôle de clôture K9 neuf ; écarts : pas d'écran de
  détail de compte (dialogue depuis la liste), suppression d'exercice ou de compte sans objet, story valable pour tout
  compte bancaire.
- 2026-11-15 — alignement inter-fiches P2 (Opus) : renvois 38-6c → 38-6c2 (la migration du groupe, D-P2-3) à l'AC-1.
- 2026-11-15 — validation P3 (Sonnet) : 4 findings (0 CRITICAL, 0 HIGH, 2 MEDIUM, 2 LOW) ; remédiés dans la fiche, aucun
  inter-fiches. MEDIUM : P3-1 AC-7 omettait les **sites de dénombrement** que 38-2 AC-1 et 38-6c2 AC-1 listent pour
  toute table neuve — `admin_full_export_e2e.rs:~281` (`data_count == 44`, +1 cumulé) et `onboarding_audit_e2e.rs:2046`
  (`EMPTY_BEFORE_RESET`, liste fermée), relus à la source ; faute de quoi le gate runtime rougit sur deux tests sans
  rapport apparent ; P3-2 le recensement des consommateurs du nombre de contrôles manquait `fiscal-years.spec.ts:499-500`
  (`toHaveCount(8)`, E2E qui rougirait avec K9), `marketing-brochure.tex:437` (« Huit contrôles préalables », brochure et
  PDF), `fiscal-years-page.test.ts:447`, `FiscalYearChecks.test.ts:48` et les doc-comments `closing_checks.rs:123`,
  `:566` — trouvés par `grep -rniE "huit (contr|ligne)|toujours huit"` sur `crates`, `frontend`, `docs`. LOW : P3-3 la
  garde `backup_inventory_matches_schema` n'était pas nommée à l'AC-7 ; P3-4 la liste des versions de migration à
  distinguer (AC-1) ne citait que 38-6c2, 38-4 et 38-5 — toutes celles de l'epic. Relu au code : `document_storage.rs:90`,
  `:142`, `:166`, `imported_supplier_invoices.rs:733`, `:768`, `inbox_import.rs:900`, `bank_imports.rs:365`, `:448`, `:872`,
  `lib.rs:832-838`, `:1228-1231`, `backup.rs:423`, `:539`, `pre_upgrade_backup.rs:299`, `import.rs:115-140`,
  `closing_checks.rs:51-60`, `tests/closing_checks.rs:1679`, `csv_tables.rs:2377`, `:2558`, `errors.rs:1887`, `:3506`,
  `api-external.md:266-267`, `user-manual.tex:890`, `admin-manual.tex:1866` ; 37-1 AC-6 (`Absence::Table`), 37-4 AC-1 quater
  (`balances_as_of`), 37-5 `:543-544` (K10) — tous concordent. Trend : P1 = 5 → P2 = 7 → P3 = 4 (0 HIGH, 2 MEDIUM).
  Axes non exercés : code de 37-4 (non écrit) ; PDF du manuel (non régénéré) ; B27/B28 sans effet sur cette fiche.
- 2026-11-15 — validation P4 ciblée (Opus) : 2 findings (0 CRITICAL, 0 HIGH, 1 MEDIUM, 1 LOW), remédiés dans la fiche,
  aucun inter-fiches. MEDIUM : P4-1 propagation de P3-2 incomplète — `README.md:48` (§ *Fonctionnalités* : « huit
  contrôles en lecture seule avant de clôturer un exercice ») restait hors de l'inventaire, le grep de la P3 ne
  couvrant que `crates`, `frontend`, `docs` ; ajouté à l'AC-10 (`CHANGELOG.md:486`, publié, reste tel ; `website/` et
  manuels DE/IT/EN sans occurrence, relus par `grep -rniE "huit contr|eight|acht|otto"`). LOW : P4-2 la brochure
  « avec la mention de la livraison de K9 » appelait un numéro de version que Q1 n'a pas fixé — écrit seulement s'il est
  connu, jamais de gabarit `<…>`. Remédiations P3 relues à la source et confirmées : `admin_full_export_e2e.rs:281-283`
  (`data_count, 44`), `onboarding_audit_e2e.rs:2046` (`EMPTY_BEFORE_RESET`), `fiscal-years.spec.ts:499-500`,
  `fiscal-years-page.test.ts:447`, `FiscalYearChecks.test.ts:48`, `kesh-db/src/repositories/closing_checks.rs:123`,
  `marketing-brochure.tex:437` ; liste des migrations de l'epic (AC-1) recoupée aux fiches 38-1, 38-2, 38-3a, 38-4, 38-5,
  38-6c2 (les autres : aucune migration) ; `truncate_all` couvert par `TABLES_TO_TRUNCATE` (`test_fixtures.rs:377-395`).
  Axes non exercés : PDF, code de 37-4. Trend : P3 = 4 (2 MEDIUM) → P4 = 2 (1 MEDIUM) : une passe de plus est due.
- 2026-11-15 — validation P5 ciblée (Sonnet) : 1 finding (0 CRITICAL, 0 HIGH, 0 MEDIUM, 1 LOW), remédié dans la fiche,
  aucun inter-fiches. LOW : P5-1 la tâche T7 ne nommait pas `README.md:48`, ajouté à l'AC-10 en P4 (liste de tâches
  alignée sur l'AC). Remédiation P4 relue à la source : `README.md:48` et `CHANGELOG.md:486` portent bien « Huit
  contrôles » ; `grep -rniE "huit contr|…"` sur le dépôt (hors `_bmad-output`, `target`, `node_modules`) ne rend que les
  sites déjà inventoriés (`closing_checks.rs:566`, `tests/closing_checks.rs:1679`, `closing-checks.test-fixtures.ts:24`,
  `fiscal-years.spec.ts:499`, `admin-manual.tex:1866`, `marketing-brochure.tex:437`, README, CHANGELOG), aucun dans
  `website/` ni les manuels DE/IT/EN ; Q1 de l'epic (`epic-38-paypal.md:14`, `:339`) : numéro de release non fixé, la
  règle « jamais un gabarit » est juste. Axes non exercés : PDF, code de 37-4. Trend : P4 = 2 (1 MEDIUM) → P5 = 1 (0
  MEDIUM) : aucun finding au-dessus de LOW, boucle close.
- 2026-11-15 — préfixes i18n (constat de la validation de l'Epic 40, Opus) : AC-9 ne nommait aucun préfixe ;
  précisé contre `lint-i18n-ownership.js:177-186` — `bank-accounts-` (dialogue), `fiscal-years-checks-` (contrôle de
  clôture), `error-` (global). Rédaction seule.
- 2026-11-15 — validation ciblée des préfixes i18n (Sonnet) : 0 finding au-dessus de LOW, 0 LOW. `BankAccountDocumentsDialog.svelte`
  (`features/bank-accounts/`, `bank-accounts-` : règle à tiret, `:177-186`) et `closing-checks-labels.ts` /
  `FiscalYearChecks.svelte` (`features/fiscal-years/`, `fiscal-years-checks-`, déjà celui des clés existantes) relus ;
  `error-` global (`:16`). Aucun inter-fiches.
- 2026-11-16 — alignement sur les règles de travail à plusieurs (Sonnet) : contrôle de clôture du relevé joint **K9 → K10**
  (l'Epic 37, qui sort avant, prend K9 « comptes en devise non réévalués », bloquant), « neuf contrôles » → « dix
  contrôles » à tous les sites actifs (AC-6, AC-10, tâches, Dev Notes) ; les entrées datées plus haut gardent K9, relevé
  daté de leur passe. PDF des manuels non commités (`.tex` seuls, régénération à la release) ; section *Note de version*
  ajoutée ; compteurs « recomptés après chaque reprise de `main` ».
