# Story 38-10 : Relevé joint à la main — solde de clôture, écart aux contrôles de clôture

Status: ready-for-dev

- **Epic** : 38 (`_bmad-output/planning-artifacts/epic-38-paypal.md`, § *38-10*, E17, E18, E20 (b), C-38-10) ; décisions de
  Beat B10 et B20 (`epics-38-40-cadrage.md`), qui priment.
- **Dépend de** : **38-2** et **38-3** (ordre de l'epic : 38-10 « dès 38-3 » ; la story elle-même vaut pour **tout** compte
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
- **Taille** : l'inbox borne un fichier à 25 Mio par défaut (`crates/kesh-api/src/config.rs:530`) ; les routes
  d'import bancaire posent leur borne par `DefaultBodyLimit::max` (`crates/kesh-api/src/lib.rs:832-838`) et lisent le
  multipart par `parse_multipart` / `map_multipart_err` (`crates/kesh-api/src/routes/bank_imports.rs:448`, `:872`).
- **La sauvegarde emporte les pièces que la base désigne** : `read_referenced_files` (`crates/kesh-db/src/backup.rs:423`)
  lit **deux** sources (`imported_supplier_invoices`, `invoices.pdf_storage_path`) ; `ParsedBackup::referenced_files`
  (`crates/kesh-api/src/admin_backup/import.rs:115-140`) fait de même côté import, et tolère une table ou une colonne
  absente (`:109-114`, `:121-127`) ; le test `colonnes_storage_path_inventaire_clos` (`backup.rs:2456-2490`) **rougit** sur toute
  colonne `*storage_path*` neuve.
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
  (chaque solde arrondi au centime avant soustraction, `:93-99` ; `repositories/bank_accounts.rs:2019`) ; toutes les
  requêtes du module sont des constantes de `ALL_SQL`, sans mot de verrou (garde T-a2, `:30-40`, `:550`). Consommateurs
  du nombre : `crates/kesh-db/tests/closing_checks.rs:1679` (`len(), 8`), `docs/api-external.md:266-267` (« toujours
  huit »), `docs/manual/fr/user-manual.tex:890` (« huit contrôles préalables »), le front
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
- **C-38-10-4 — L'écart se calcule à la lecture.** Solde comptable du compte du grand livre lié à `period_to` inclus :
  en CHF, `SUM(debit) − SUM(credit)` (convention de `BALANCES_SQL`, signe libre, une dette de carte est négative) ; en
  devise, le **solde en devise** de 37-4 (AC-1 : `Σ amount_currency` au débit − au crédit, fragment de
  `repositories/account_balances.rs`, réévaluations sans effet). Écart = règle de `statement_gap` (deux soldes arrondis
  au centime, puis différence ; `null` si nul) — par la même fonction ou une fonction commune extraite, jamais une
  seconde règle. Compte sans grand livre lié, ou grand livre partagé (`bank_accounts::is_shared`) → `notComparable: true`,
  écart `null`.
- **C-38-10-5 — Contrôle de clôture : K9 en dernier.** Un neuvième contrôle `ATTACHED_STATEMENT_GAP` (`warning`), ajouté
  **après** K8 pour que K1–K8 gardent leur rang (précédent v017-4a) : pour chaque compte bancaire non archivé ayant au
  moins un document à solde dont `period_to` tombe dans Y, le **dernier** (plus grand `period_to`, puis plus grand `id`)
  est comparé (C-38-10-4) ; écart non nul → compte en `attention`. K3 est **inchangé** (un compte sans CAMT y reste
  `NO_BALANCE_STATEMENT`) : écrire au manuel que K9 est le contrôle des comptes sans relevé importé.
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
   (date au développement, postérieure à toutes les migrations de l'Epic 37 et de 38-1 à 38-5) : `CREATE TABLE
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
   31.12.2024, document au 31.12.2024 avec le solde comptable exact → `gap: null` ; solde différent de 12.34 → `gap:
   "-12.34"` (signe : relevé − livres, comme `statement_gap`) ; compte PayPal EUR lié à un compte du plan EUR (37-1),
   lignes en devise → l'écart se lit **en EUR** sur le solde en devise (une réévaluation au 31.12 ne le change pas) ;
   carte de crédit liée à un passif endetté : solde saisi négatif, écart nul ; compte non lié ou grand livre partagé →
   `notComparable: true`, `gap: null` ; document sans solde → `ledgerBalanceAtPeriodTo` rendu, `gap: null`. Lectures sans
   verrou ; une lecture par page de documents (pas une par document : regroupées par compte et date, test du compteur
   ou justification au Change Log).
6. **AC-6 — Contrôle de clôture K9 (FR-38-10-4).** `GET /api/v1/fiscal-years/{id}/closing-checks` rend **neuf**
   contrôles ; le neuvième, en dernier, `ATTACHED_STATEMENT_GAP`, `severity: warning`, `status` `attention` si au moins un
   compte a un écart (C-38-10-5), `count` = nombre de comptes en écart, `details: { accounts: [{ bankAccountId, name,
   documentId, periodTo, currency, statementClosingBalance, ledgerBalanceAtStatement, gap }], notComparable: [{
   bankAccountId, name }] }` (comptes non comparables sans effet sur le statut, comme K3). La clôture **n'est pas
   bloquée** (`fiscal_years::close` inchangé). Requêtes : constantes ajoutées à `ALL_SQL`, sans verrou (garde T-a2
   verte), dans l'instantané de la route. Mises à jour du nombre (prémisse, **tous** les sites) : doc du module (« neuf »,
   table, `count`), `ClosingChecks::checks` (« toujours neuf »), `CheckCode`/`CheckDetails`/`Severity` (variant neuf),
   `crates/kesh-db/tests/closing_checks.rs:1679`, `crates/kesh-api/tests/closing_checks_e2e.rs`, `docs/api-external.md`
   (« toujours neuf », contrôle décrit, changement de contrat annoncé comme la ligne `:267` l'exige), front
   (`closing-checks-labels.ts`, `FiscalYearChecks.svelte` avec lien vers le dialogue des relevés du compte,
   `fiscal-years.types.ts`, fixtures, tests Vitest), `user-manual.tex:890` ; grep du jeton `\bhuit\b` et `\b8\b` sur ces
   fichiers, résultat trié au Change Log. Tests : écart → `attention` ; aucun document → `ok` ; document daté hors Y
   ignoré ; deux documents dans Y → le dernier compte ; dialogue de clôture (Vitest) qui rappelle K9.
7. **AC-7 — Sauvegarde et restauration (NFR-38-10-2, E18).** (a) `bank_account_documents` entre à `TABLES_TO_TRUNCATE`
   **avant** `bank_transactions` (enfant de `bank_accounts`, `fiscal_years`, `users`, `companies`), donc vidée à la remise
   à zéro de la société ; (b) `read_referenced_files` lit une **troisième** source (`SELECT storage_path, sha256 FROM
   bank_account_documents`), `ParsedBackup::referenced_files` aussi (`("bank_account_documents", "storage_path",
   "sha256")`), et `colonnes_storage_path_inventaire_clos` liste la colonne neuve ; (c) la table entre à la **liste close
   des absences admises** de 37-1 AC-6 (R1, `Absence::Table("bank_account_documents")` avec sa version de migration) :
   un `.keshbackup` antérieur se réimporte, table vide ; (d) l'export CSV de souveraineté la porte (`TABLES_EXPORTEES`,
   sérialiseur `serialize_bank_account_documents_csv` : `id, bank_account_id, fiscal_year_id, kind, source, period_from,
   period_to, original_filename, sha256, byte_size, closing_balance, closing_currency, created_by_user_id, created_at` —
   sans `storage_path`), garde `export_couvre_toutes_les_tables` verte. Tests : export complet puis import dans une
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
   API cliente dans `bank-accounts.api.ts`. Vitest : liste, écart, formulaire (champs exigés, PDF seul côté écran),
   rôles, compte clos. **E2E Playwright** (`frontend/tests/e2e/bank-account-documents.spec.ts`) : joindre un PDF de
   fixture avec un solde qui diffère des livres → écart affiché → panneau *Préparer la clôture* : K9 en attention →
   corriger le solde → écart nul, K9 `ok` → télécharger (contenu identique) → retirer. **Recette visuelle** : captures
   (thème clair) de la liste vide, de la liste avec écart, du formulaire, d'une erreur, du panneau de clôture avec K9,
   jointes au Dev Agent Record et au prompt de revue (hors dépôt, `kesh-gate-logs/`).
10. **AC-10 — Manuels et API (NFR-38-10-4).** `docs/manual/fr/user-manual.tex` : section *Joindre un relevé à un compte
    bancaire* (près de *Comptes bancaires* et de *Tenir une carte de crédit*) : pourquoi (le relevé annuel PayPal n'a pas
    d'API ; le solde de clôture est la somme de contrôle), comment (période, exercice choisi par la date de fin, un PDF
    par compte et par devise), l'écart, le contrôle K9, exercice clos (on joint, on ne corrige plus) ; § *Préparer la
    clôture* : « neuf contrôles », K9 décrit. `admin-manual.tex` : les relevés joints sont dans le dossier des documents
    et dans la sauvegarde complète ; la sauvegarde d'avant mise à jour les emporte. PDF régénérés (`make fr` dans
    `docs/manual/`) et vérifiés aplatis (`pdftotext … | tr '\n' ' '`). `docs/api-external.md` : les cinq routes, leurs
    codes, K9. **MCP** : aucun outil (E23) — `kesh_pieces` (`mcp/outils/pieces.rs`) liste les pièces fournisseur
    importées et n'est pas étendu ; les relevés joints ne sont pas exposés au serveur MCP (décision de l'epic, aucune
    donnée de prestataire exposée).

## Tâches

- [ ] T1 — Migration, ligne d'audit d'idempotence (P5), grep P6, squash `test-schema` (AC-1).
- [ ] T2 — `kesh-db` : entité `BankAccountDocument` (`kind`, `source` tolérants), dépôt (créer, lister avec soldes
      comptables, lire, modifier avec `version`, retirer), calcul de l'écart par la règle de `statement_gap` (AC-2 à AC-5).
- [ ] T3 — `kesh-api` : routes (`lib.rs`, bornes de corps), multipart, contrôle PDF, `store_document`, téléchargement
      (fonction `content_disposition` partagée), codes d'erreur, audit (AC-2 à AC-4, AC-8).
- [ ] T4 — `closing_checks` : K9, tous les consommateurs du nombre (AC-6).
- [ ] T5 — Sauvegarde : `TABLES_TO_TRUNCATE`, `read_referenced_files`, `ParsedBackup::referenced_files`, inventaire
      clos, absences admises (R1), export CSV (AC-7).
- [ ] T6 — Frontend : dialogue, API, i18n, Vitest, E2E, captures (AC-9).
- [ ] T7 — Manuels FR (PDF régénérés), `api-external.md` (AC-10).
- [ ] T8 — Gate complet **même en cours de boucle** (migration et dépôts touchés) ; runtime : `admin_backup_e2e`,
      `admin_full_import_e2e`, `migrations_fresh_install` ; E2E complet au dernier commit de code.

## Dev Notes

- **Pourquoi un K9 et non K3 étendu** : K3 compare au grand livre en CHF les relevés **importés** ; y mêler les relevés
  joints changerait son contrat (`NO_BALANCE_STATEMENT`) et sa règle de précédence, publiés (`api-external.md:266`).
  K9, ajouté en dernier, ne déplace rien (précédent K8, v017-4a).
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

- 2026-11-15 — rédaction (Opus) : 10 AC, 5 FR (4 de l'epic + 1 ajoutée), 4 NFR (3 + 1) ; une migration non breaking
  (`bank_account_documents`) ; +3 routes mutantes, +2 lectures ; contrôle de clôture K9 neuf ; écarts : pas d'écran de
  détail de compte (dialogue depuis la liste), suppression d'exercice ou de compte sans objet, story valable pour tout
  compte bancaire.
