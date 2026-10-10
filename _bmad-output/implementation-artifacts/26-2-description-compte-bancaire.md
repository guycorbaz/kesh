# Story 26.2 : Une courte description pour chaque compte bancaire

## Status

**ready-for-dev** (deux clés) — fiche créée le 2026-10-10 en autonomie (`bmad-create-story`), sur `fe162504` (kickoff
de l'Epic 26), branche `story/26-2` ; **découpée** par l'orchestrateur le même jour (registre **C-26-2-15**) ;
validation **P1** (Sonnet, lentilles R et F : 0 HIGH, 8 MEDIUM, 15 LOW) **remédiée** le 2026-10-10 — validation P2
(Opus) due.

⛔ **Deux stories, une fiche, deux PR successives** (C-26-2-15, qui remplace la proposition C-26-2-1) :

| Clé au registre de sprint | Contenu | AC | PR |
|---|---|---|---|
| **`26-2a-description-socle`** | colonne, dépôt, API, validation, export CSV et `.keshbackup`, fonctions de libellé (TS et Rust), saisie et écran *Comptes bancaires*, i18n | AC1 à AC8, AC13 (a), AC14 (a) | **`refs #622`** — ⛔ jamais `closes` |
| **`26-2b-description-ecrans`** | les autres écrans, `GET /companies/current`, message serveur du refus de désignation, garde d'inventaire clos, manuel, CHANGELOG, `docs/api-external.md` | AC9 à AC12, AC13 (b), AC14 (b) | **`closes #622`** |

**Ordre** : 26-2a fusionnée **avant** que la 26-2b ne commence ; la 26-2b part de `origin/main` après cette
fusion. La clé mère `26-2-description-compte-bancaire` est au statut `split`. Revue de la 26-2b **fichier par
fichier** (rollout mécanique, § *Règle de splitting préventif*). **Aucune release entre les deux** : entre les deux
PR, le champ se saisit sans que manuel ni CHANGELOG le disent — à écrire au Dev Agent Record de la 26-2a.

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

- Migration neuve **`crates/kesh-db/migrations/20261011000002_bank_accounts_description.sql`** — version **fixée
  par l'orchestrateur** (C-26-2-25, remplace C-26-2-14) : la 26-1 garde `20261011000001` et **fusionne la
  première** ; la 26-2a se développe **après rebase sur la 26-1 fusionnée** (ou, si elle doit commencer avant, se
  rebase dessus avant son gate final et refait tous les recomptes ci-dessous). **Une seule instruction** :
  `ALTER TABLE bank_accounts ADD COLUMN description VARCHAR(60) NULL, ALGORITHM=INSTANT, LOCK=NONE;` — **en fin de
  table, sans `AFTER`** (C-26-2-16 : patron de `20260531000001_bank_accounts_archived.sql` ; `AFTER` + `INSTANT` a
  aussi un précédent, `20260507200001_bank_account_journal_link.sql` (`AFTER qr_iban`, l. 21-22), mais la position
  n'apporte rien — l'ordre de lecture du CSV est fixé par le sérialiseur, pas par la table), précédée d'un en-tête de commentaires (story, #622, non-breaking, pourquoi 60). Aucune contrainte, aucun
  index (C-26-2-3, C-26-2-4).
- **P1/P3** : `ADD COLUMN` nullable → **non breaking** ; aucun `UPDATE _kesh_version`, aucun bump Cargo **par
  cette migration**. Une 0.13.0 qui lirait la base l'ignore (ses `SELECT` nomment leurs colonnes). En revanche
  une sauvegarde d'installation **prise en 0.13.1** est refusée par une 0.13.0 (`check_schema_compat`, colonne
  source inconnue, `crates/kesh-api/src/admin_backup/import.rs:216-223`, `400` avant toute écriture) — et elle
  l'est **de toute façon** : la 26-1 porte `kesh_version_min_required` à `0.13.1`. Ce refus est **attendu** ; le
  retour arrière d'une 0.13.1 passe par la sauvegarde de la base prise avant la mise à jour (F7, C-26-2-24).
- **P5** : une ligne à sa place chronologique **dans** le tableau de `docs/migrations-idempotence-audit.md`, verdict
  `tracked-by-sqlx` (`ADD COLUMN` sans `IF NOT EXISTS` → erreur 1060 au rejeu hors sqlx), non-breaking, pas
  d'écriture de données. Les **cinq** compteurs **recomptés depuis le tableau** : en-tête `## Table d'audit (N
  migrations)`, ligne `Total` (avec sa ventilation : `+ 1 Story 26-2`), `yes`, `tracked-by-sqlx`, `no` — sur
  `fe162504` : 78 / 78 / 10 / 68 / 0. **Après la 26-1** (une migration, verdict attendu `tracked-by-sqlx` — à relire
  sur `origin/main`, jamais à supposer) : 79 / 79 / 10 / 69 / 0 ; **après la 26-2a** : **80 / 80 / 10 / 70 / 0**.
  Les valeurs qui font foi sont celles que rendent les commandes, exécutées **après le rebase** et recopiées au
  Dev Agent Record :
  ```sh
  ls crates/kesh-db/migrations/*.sql | wc -l
  grep -c '^| `20' docs/migrations-idempotence-audit.md
  grep -c '^| `20.*| yes |' docs/migrations-idempotence-audit.md
  grep -c '^| `20.*| tracked-by-sqlx |' docs/migrations-idempotence-audit.md
  grep -c '^| `20.*| no |' docs/migrations-idempotence-audit.md
  ```
- **P6** : `grep -rn "migrations.len()\|apply_migrations_up_to" crates/` inspecté site par site. Sur `fe162504`,
  un seul site positionnel : `crates/kesh-db/tests/migrations_upgrade_path.rs` (`total, 78` l. 110 ; `total - 44`
  l. 192), frontière figée à **34**. La 26-1 l'aura porté à `total` 79 / soustracteur 45 ; la 26-2a le porte à
  **`total` 80, soustracteur 46** (80 − 46 = 34) — valeurs à relire **sur la branche rebasée**, et **toutes** les
  mentions de la valeur
  dans le fichier (sur `fe162504` : commentaires l. 38, 40, 57-58, 107, 111, 115, 123, 130, 133, 185, 195, 264),
  greppées **par jeton** : `grep -nE "\b(79|45)\b" crates/kesh-db/tests/migrations_upgrade_path.rs` (les valeurs
  laissées par la 26-1) doit ne plus rendre que des généalogies historiques. Le bloc de généalogie du fichier (l. 185 sur `fe162504` : « Story 15-1a2-ii : 42 → 44,
  frontière inchangée (78 - 44 = 34) », suivie de celle de la 26-1) gagne sa ligne **« Story 26-2a : 45 → 46,
  frontière inchangée (80 - 46 = 34) »**. Les autres sites (`lettering_documents_backfill.rs`, `audit_log_company_id_backfill.rs`,
  `journal_entry_number_sequences_bootstrap.rs`, `accounts_role_backfill.rs`,
  `invoice_lines_revenue_account_backfill.rs`, `closing_accounts_backfill.rs`) résolvent par version
  (`migrations_before`) : rien à faire, à confirmer par le gate.
- **P7** : la migration n'écrit aucune donnée (pas d'`UPDATE`, `INSERT`, `REPLACE`, `DELETE`) → **ni registre
  `POST_RESTORE_BACKFILLS`, ni `EXEMPT_MIGRATIONS`** ; `every_data_backfill_migration_is_triaged` reste vert sans
  retouche — **à vérifier au gate**, pas à supposer. Les commentaires de l'en-tête évitent les mots-clés SQL
  d'écriture en tête de ligne (le détecteur travaille sur du SQL décommenté, mais la prudence ne coûte rien).
- **P8** : la migration n'est éditée **plus du tout** une fois appliquée à une base de dev persistante
  (`kesh_e2e_<clé>` notamment) ; toute précision ultérieure va dans la ligne d'audit ou les Dev Notes.
- **Squash de test** régénéré par `scripts/regen-test-schema.sh` (jamais édité à la main) **après le rebase sur la
  26-1** (un squash régénéré avant ne porte pas sa migration) : `crates/kesh-db/tests/test_schema_guard.rs` vert.
- **Registre des checksums** `crates/kesh-db/migrations.sha384` (garde P8 outillé, Story 22-5) : **une ligne
  ajoutée** `20261011000002 <sha384>`, à la suite de celle de la 26-1, **jamais** une ligne modifiée. La valeur
  est celle qu'imprime `published_migrations_keep_their_checksums` (`test_schema_guard.rs`, « à coller dans
  crates/kesh-db/migrations.sha384 ») ou `sha384sum` du fichier — calculée sur la **version définitive** de la
  migration ; toute retouche ultérieure de la migration avant publication impose de recalculer cette ligne (et
  de reconstruire les bases de dev, P8). Contrôle : `grep -c '^20' crates/kesh-db/migrations.sha384` égale le
  nombre de migrations (`ls … | wc -l`).

**AC2 — Entité et dépôt : une seule liste de colonnes, et cinq chemins qui ne touchent pas la description.**

- `BankAccount` (`crates/kesh-db/src/entities/bank_account.rs`) gagne `pub description: Option<String>` (doc
  `///` : facultative, normalisée par la route, `None` = absente), sérialisée `description`.
  `NewBankAccount` **ne change pas** (C-26-2-7).
- Les **onze** `SELECT` qui énumèrent les colonnes vers `BankAccount` (dix dans
  `crates/kesh-db/src/repositories/bank_accounts.rs` : `FIND_BY_ID_SQL` l. 25, `find_primary` l. 77,
  `find_by_id_for_company` l. 107, `list_by_company` l. 129 et 132, `upsert_primary_in_tx` l. 227,
  `set_journal_account_id_for_company` l. 353, `update_for_company` l. 468, et l. 541, l. 606 ; un dans
  `crates/kesh-api/src/routes/bank_accounts.rs` l. 520-522) passent par **une seule liste** de colonnes — DRY, et
  c'est ce qui empêche un `SELECT` oublié de faire échouer `FromRow` à l'exécution. **Mécanisme fixé**
  (C-26-2-20) : un `macro_rules! columns` **privé au module du dépôt**, sur le précédent exact de
  `crates/kesh-db/src/repositories/accounts.rs:21-42` (« un `macro_rules!` plutôt qu'un `const` pour rester
  utilisable dans `concat!`, qui n'accepte que des littéraux ») ; chaque `SELECT` s'écrit
  `concat!("SELECT ", columns!(), " FROM bank_accounts WHERE …")`, `FIND_BY_ID_SQL` compris. **Aucune** liste de
  colonnes ne sort de `kesh-db` : le `SELECT` inline de la route (relecture après `INSERT`) est remplacé par une
  fonction publique neuve du dépôt, `pub async fn find_by_id_in_tx(tx, id) -> Result<BankAccount, DbError>`.
  Contrôles, à exécuter et recopier :
  ```sh
  grep -rnE "SELECT id, company_id, bank_name" crates/*/src          # → aucune sortie
  grep -rnF "id, company_id, bank_name, iban" crates/*/src            # → la seule ligne du corps de la macro
  ```
  (les `INSERT INTO bank_accounts (company_id, bank_name, …)` ne commencent pas par `id,` et ne sont pas visés ;
  l'`INSERT` de la route est étendu à part, AC4.)
- `update_for_company` reçoit `new_description: Option<&str>` (paramètre séparé, patron de
  `new_journal_account_id`) et l'écrit ; ses appelants (route, et les tests de
  `crates/kesh-db/tests/bank_accounts_repository.rs`) suivent. Sa doc-comment énumère le champ.
- **Ne touchent pas** la colonne — les **cinq** autres écrivains de `bank_accounts` (recomptés :
  `grep -n "^pub async fn" crates/kesh-db/src/repositories/bank_accounts.rs` → `create` l. 29,
  `upsert_primary_in_tx` l. 222, `set_journal_account_id_for_company` l. 340, `update_for_company` l. 458,
  `archive_for_company` l. 534, `flip_primary_off_for_company` l. 600 ; seul `update_for_company` l'écrit) :
  `upsert_primary_in_tx` (ni l'`INSERT`, ni l'`UPDATE`, ni `is_no_op_change`), `set_journal_account_id_for_company`
  (PATCH), `archive_for_company` (DELETE), `bank_accounts::create`, et **`flip_primary_off_for_company`** —
  l'ancien principal démoté quand un autre compte est promu, **typiquement un compte décrit**. Chacun a un test
  qui pose une description (par SQL) puis exerce le chemin et la retrouve intacte (l'upsert : changement de
  `bank_name` → `Updated`, description conservée ; la démotion : `PUT isPrimary=true` sur un second compte, l'ancien
  principal relu `isPrimary=false`, description intacte).
- **Onboarding** (C-26-2-22) : refaire l'étape « Compte bancaire » avec une **autre** banque garde la description
  du compte principal (« UBS2 — Personnel ») — conséquence assumée de C-26-2-2, dite au manuel (AC12).
- `crates/kesh-api/src/routes/invoice_pdf_service.rs` (l. 659 et 845, littéraux de test) gagne `description: None` ; le PDF et le QR-Bill
  **n'affichent pas** le nom de la banque (`grep -rn "bank_name" crates/kesh-qrbill crates/kesh-payment
  crates/kesh-report` → aucun résultat sur `fe162504`) : rien à y changer.

**AC3 — Validation et normalisation serveur** (C-26-2-3, C-26-2-11).

`validate_bank_input` (`crates/kesh-api/src/routes/bank_accounts.rs` l. 230) reçoit la description et rend sa
forme normalisée :

- absente, `null`, vide ou faite d'espaces → `None` ;
- sinon `trim()` ; plus de **`BANK_ACCOUNT_DESCRIPTION_MAX_CHARS`** caractères (`chars().count()`) →
  `400 VALIDATION_ERROR`, message en dur « La description ne peut pas dépasser 60 caractères » (le nombre du
  message est formaté depuis la constante) ;
- un caractère **invisible ou de mise en page** → `400`, « La description ne peut pas contenir de saut de ligne ni
  de caractère invisible » — l'ensemble refusé est **fermé et nommé** (C-26-2-17) : `char::is_control` (catégorie
  Cc, dont `\n`, `\r`, `\t`), **plus** U+2028 et U+2029 (séparateurs de ligne et de paragraphe), U+200B à U+200F
  (largeur nulle, marques LRM/RLM), U+202A à U+202E (enchâssements et inversion bidi), U+2060 à U+2064, U+2066 à
  U+2069 (isolats bidi) et U+FEFF — une fonction `is_forbidden_in_description(c: char) -> bool`, testée sur chaque
  plage (une borne de chaque plage, et un caractère voisin accepté) ;
- **ordre** des refus de forme : banque vide → description → IBAN → QR-IBAN — **épinglé par un test** (corps
  qui viole les quatre : le refus nomme la banque ; puis banque valide : la description ; etc.).
- Tests aux bornes : 60 caractères acceptés, 61 refusés ; 60 caractères **accentués précomposés** (`é` U+00E9
  ×60) acceptés — `chars`, pas octets ; `"  Épargne  "` stocké `"Épargne"` ; `"   "` stocké `NULL` ; `"a\nb"`,
  `"a\u{2028}b"`, `"a\u{202E}b"` refusés. **Angle mort assumé** (C-26-2-17) : un accent **décomposé** (`e` +
  U+0301) compte pour deux caractères — Kesh ne normalise pas en NFC ; les champs de saisie des navigateurs
  produisent du précomposé.
- **Source unique de la borne** (C-26-2-21) : `pub const BANK_ACCOUNT_DESCRIPTION_MAX_CHARS: usize = 60` dans
  `kesh_db::entities::bank_account`, lue par la validation ; un test unitaire de `kesh-db` lit le squash
  (`include_str!("../../test-schema/0001_schema_squash.sql")`, patron de `csv_tables.rs:2148`) et vérifie que la
  colonne `description` de `bank_accounts` y est `varchar(<la constante>)`. Côté frontend,
  `BANK_ACCOUNT_DESCRIPTION_MAX` exportée de `format.ts` alimente les deux `maxlength`. Le nombre reste écrit en
  toutes lettres dans les textes (FTL ×4, manuel, CHANGELOG) et dans la migration : greper `\b60\b` après tout
  changement.
- **Pas d'unicité** (C-26-2-4) **épinglée par un test** : deux `POST` de la même société avec la même banque et la
  même description → `201` et `201`.

**AC4 — L'API.**

- `POST /api/v1/bank-accounts` : `CreateBankAccountBody` gagne `#[serde(default)] description: Option<String>` ;
  l'`INSERT` de la route la nomme. Réponse `201` avec `description`.
- `PUT /api/v1/bank-accounts/{id}` : `UpdateBankAccountBody` idem ; **remplacement complet** — absente ⇒ effacée,
  comme `qrIban` (C-26-2-8). La doc-comment du handler le dit. **C'est un changement de contrat** pour une
  intégration par clé d'API (`PUT` est ouvert aux clés `read-write`, `docs/api-external.md:218`) : un corps écrit
  pour la 0.13.0, sans le champ, **efface** une description saisie à l'écran (C-26-2-19). Il est donc **figé par un
  test** (compte décrit, `PUT` dont le corps omet `description` → `200`, description `null` en réponse et en base)
  **et par la mutation M13**, et **documenté** en 26-2b (AC12 : `docs/api-external.md` et CHANGELOG `### Modifié`).
  Les **deux** formulaires du frontend envoient toujours le champ (AC7, testé).
- `GET /api/v1/bank-accounts` : `description` présente sur chaque élément (`null` si absente) — gratuit par le
  `#[serde(flatten)]` de `BankAccountWithBalance`, mais **testé**.
- `PATCH` et `DELETE` : réponse avec la description inchangée (testé).
- Audit (C-26-2-10) : `bank_account.created` porte `"description"` ; `bank_account.updated` (`trigger:
  full_update`) la porte dans `before` et `after`. `audit_route_registry.rs` reste vert. L'audit de l'onboarding
  (`routes/onboarding.rs` l. 600-660) ne change pas : la description n'y bouge pas.
- Contrat multi-tenant inchangé : rien de neuf dans les gardes (le champ vit dans la ligne déjà scopée).

**AC5 — Export CSV et sauvegarde d'installation.**

- `serialize_bank_accounts_csv` (`crates/kesh-api/src/exports/csv_tables.rs` l. 623) : colonne `description`
  **juste après** `bank_name` (C-26-2-6 ; la place en base est indifférente, C-26-2-16), valeur
  `fmt_opt_str(&b.description)` (passe par `csv_sanitize` : pas d'injection de formule). La garde
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
  - le type `BankAccountLabelSource = { bankName: string; description: string | null }` et
    `BankAccountOptionSource = BankAccountLabelSource & { iban: string }` — `description` **obligatoire**
    (`null` permis), pour que `npm run check` refuse un objet qui l'aurait oubliée (C-26-2-18) ;
  - `bankAccountName(a: BankAccountLabelSource): string` → `"UBS — Personnel"` ou
    `"UBS"` ; une description `null`, `undefined`, vide ou d'espaces vaut absente ; séparateur `" — "` (tiret
    cadratin U+2014 entouré d'espaces, celui des sélecteurs actuels) ;
  - `bankAccountOptionLabel(a: BankAccountOptionSource): string` →
    `` `${bankAccountName(a)} — ${a.iban}` ``.
  - la constante `BANK_ACCOUNT_DESCRIPTION_MAX = 60` (AC3, C-26-2-21) ;
  - Tests Vitest (`format.test.ts`, neuf) : avec, sans, `null`, `"  "` ; sans
    description, le libellé d'option est **identique** à l'ancien `` `${bankName} — ${iban}` ``.
- **Serveur** — `kesh_db::entities::bank_account` exporte `pub fn bank_account_display_name(bank_name: &str,
  description: Option<&str>) -> String` et `impl BankAccount { pub fn display_name(&self) -> String }`, mêmes
  règles, mêmes exemples en test unitaire (`#[cfg(test)]` du module). Elle sert à AC10 (26-2b) : en 26-2a, elle
  n'a d'appelant que ses tests — attendu, elle est `pub` d'une bibliothèque, aucun avertissement.

**AC7 — Saisie et écran *Administration → Comptes bancaires*.**

`frontend/src/routes/(app)/bank-accounts/+page.svelte` :

- formulaires de **création** et de **modification** : un champ « Description (facultative) » placé **juste après
  « Banque »**, `data-testid="form-description"` / `"edit-description"`,
  `maxlength={BANK_ACCOUNT_DESCRIPTION_MAX}`, aide sous le champ
  (`bank-accounts-help-description`) ; état `formDescription` remis à zéro par `resetForm`, prérempli par
  `openEditFull` (`ba.description ?? ''`) ; envoyé `description: formDescription.trim() === '' ? null :
  formDescription` dans les **deux** payloads (`NewBankAccountPayload`, `UpdateBankAccountPayload` gagnent
  `description?: string | null`) ;
- tableau : la cellule « Banque » (l. 413) affiche `bankAccountName(ba)` dans un
  `<span data-testid="bank-account-name-{ba.id}">` — badges *Principal* et *Archivé* inchangés, hors du `span` ;
- `frontend/src/lib/features/bank-accounts/BankAccountJournalLinkForm.svelte` l. 142 : l'en-tête
  `{bankAccount.bankName} — {bankAccount.iban}` devient `bankAccountOptionLabel(bankAccount)` ;
- `bank-accounts.api.ts` : `BankAccountSummary` et `BankAccountSummaryRaw` gagnent `description: string | null`
  (**obligatoire** dans le type public, pour que `npm run check` signale chaque fixture à compléter) ;
  `parseBankAccount` mappe `raw.description ?? null`.
- Tests Vitest de l'écran (`bank-accounts-page.test.ts`) : création avec et sans description (payload : le
  champ `description` est **présent** dans les deux, `null` si vide), édition qui préremplit puis efface (payload
  `null`), cellule avec et sans description ; `BankAccountJournalLinkForm.test.ts` : en-tête avec description (D2). Les fixtures de
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

**Types — tranché** (C-26-2-18) : les types en ligne `BankAccountListItem` de
`routes/(app)/reconciliation/+page.svelte:11` et `routes/(app)/bank-import/+page.svelte:10`, et le type de prop de
`BankImportUpload.svelte:24`, **disparaissent** au profit de `BankAccountOptionSource & { id: number }` (AC6) —
ils ne contiennent plus le jeton `bankName`. `BankAccountJson` de `lib/features/settings/settings.types.ts:44`
gagne `description: string | null` (**obligatoire**) ; c'est le type de la réponse de `GET /companies/current`
(`settings.api.ts:10`), source de D3, D7 et D9.

Dans D8 (`BankAccountsCard`) et D9 (*Paramètres*), le nom passe dans un élément à `data-testid`
(`homepage-bank-name-{id}`, `settings-bank-name-{id}`), pour l'E2E et les tests.

**Un test par site** (aucun site n'est couvert par la seule garde AC11, qui prouve une absence et non un appel
correct) :

| Site | Test de comportement |
|---|---|
| D1, D2 | Vitest `bank-accounts-page.test.ts`, `BankAccountJournalLinkForm.test.ts` (AC7) + E2E (a) |
| D3 | E2E (b), `/reconciliation` |
| D4 | **Vitest neuf** `routes/(app)/payment-batches/payment-batches-page.test.ts` : `listBankAccounts` simulé (un compte décrit et lié au plan comptable) → l'`<option>` porte « UBS — Personnel — CH… » |
| D5 | Vitest `supplier-settlement-page.test.ts` (option avec description) |
| D6 | Vitest `SettleInvoiceDialog.test.ts` (option avec description) |
| D7 | E2E (b), `/bank-import` |
| D8 | Vitest `HomepageCards.test.ts` (avec et sans) + E2E (b), accueil |
| D9 | **Vitest neuf** `routes/(app)/settings/settings-page.test.ts` (patron `settings/invoicing/settings-invoicing-page.test.ts`) : la réponse simulée de `/api/v1/companies/current` porte `description` → « UBS — Personnel » rendu ; sans → « UBS » |
| D10 | Rust, AC10 |

La chaîne complète de D9 (serveur → JSON → type → rendu) est tenue par deux tests qui se rejoignent : le test
serveur de `BankAccountJson` (AC10) et ce test Vitest, dont la réponse simulée est typée `CompanyCurrentResponse`
(`description` obligatoire, donc `npm run check` refuse une simulation qui l'omet).

**AC10 — `GET /companies/current` et le refus de désignation nomment le compte.**

- `BankAccountJson` (`crates/kesh-api/src/routes/companies.rs` l. 316-332) gagne `description: Option<String>` —
  c'est **la source** des sélecteurs de l'import bancaire et de la réconciliation (D3, D7) et de *Paramètres* (D9).
  Test : la réponse porte la description.
- Refus `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT` (C-26-2-9) : `first_active_bank_account_linked_to`
  (`repositories/bank_accounts.rs` l. 723-738) sélectionne `bank_name, description` **sans toucher** au
  `LOCK IN SHARE MODE` ni au texte `FROM bank_accounts` (un test de sérialisation l'attend, cf. sa doc) et rend
  `(id, bank_account_display_name(..))` ; le champ `bank_name` de `DbError::ClaimAccountLinkedToBankAccount`
  (`crates/kesh-db/src/errors.rs` l. 1078) devient `bank_label`, propagé à `company_invoice_settings.rs` l. 217-229,
  `crates/kesh-api/src/errors.rs` l. 3260-3270 et 3994-4023, et aux tests (`crates/kesh-db/src/errors.rs` l. 1578,
  `crates/kesh-api/src/errors.rs` l. 4834, `crates/kesh-db/tests/company_invoice_settings_repository.rs` l. 1497).
  Le message (`$bank` des quatre locales, textes FTL **inchangés**) et `details.bankName` (clé conservée, C-26-2-23)
  portent « UBS — Personnel ». Lire le message une fois à l'écran (« … au compte bancaire UBS — Personnel : il ne
  peut pas … ») ; s'il se lit mal, guillemets « » autour de `{ $bank }` dans les **quatre** FTL, consigné.
- Tests : le refus nomme « UBS — Personnel » quand le compte lié a une description, « UBS » sans (route
  `PUT /api/v1/company/invoice-settings` ; test e2e existant de la 15-6c,
  `crates/kesh-api/tests/company_invoice_settings_postable_e2e.rs`, étendu).

**AC11 — Garde d'inventaire clos : le jeton `bankName` n'apparaît que là où il est attendu, et autant de fois.**

Définition **unique**, sans numéros de ligne (C-26-2-18) — un inventaire des **non-résolus** par décompte :

- **Collecte** : tous les fichiers `.svelte` et `.ts` de `frontend/src`, **hors** `*.test.ts` et `*.spec.ts`,
  lus en entier (`<script>`, balisage, attributs, `{@const}`, `.ts` — aucune distinction de zone) ; dans chacun,
  le nombre d'occurrences du **jeton** `bankName` (expression `/\bbankName\b/g` — `formBankName` n'est pas
  compté, la casse diffère).
- **Attendu** : une table **fermée** `{ fichier → nombre d'occurrences, motif }` écrite dans le test. Sur l'arbre
  final, elle ne contient que : `lib/features/bank-accounts/format.ts` (les fonctions de libellé et leurs types),
  `lib/features/bank-accounts/bank-accounts.api.ts` et `lib/features/settings/settings.types.ts` (porteurs),
  `routes/(app)/bank-accounts/+page.svelte` (E1 : préremplissage et payloads du formulaire — la cellule D1 n'y
  contribue plus), les trois fichiers de l'onboarding (E2), les quatre fichiers des profils d'import (E3,
  homonymes). Les nombres sont **mesurés sur l'arbre final** et recopiés au Dev Agent Record avec la commande
  `grep -row bankName frontend/src --include=*.svelte --include=*.ts | grep -v "\.test\.ts:\|\.spec\.ts:" | cut -d: -f1 | sort | uniq -c`
  (occurrences et non lignes : `grep -c` compterait 5 et non 6 pour `bank-accounts.api.ts`, dont la l. 75 porte
  deux jetons).
- **Rougit** : un fichier hors table avec ≥ 1 occurrence (nommé, avec ses lignes) ; un fichier de la table dont
  le nombre diffère ; une entrée de la table dont le fichier n'existe plus ou compte 0 (table périmée).
- **Anti-test-muet** : nombre de fichiers lus ≥ un minimum mesuré (posé à la valeur mesurée, pas « moins une
  marge ») ; **somme** des occurrences égale à la somme de la table ; et un **cas synthétique par forme**, sur la
  fonction de décompte elle-même : balisage `<p>{a.bankName}</p>`, attribut `title={a.bankName}`,
  `{@const l = `${a.bankName}`}`, libellé construit en script `items.map((a) => ({ label: a.bankName }))`,
  déstructuration `const { bankName } = a`, et un `.ts` — chacun compté 1 ; `formBankName` compté 0.
- **Ce que la garde ne voit pas** (angle mort assumé, C-26-2-18) : un libellé construit **sans** le jeton
  (`a['bank' + 'Name']`, ou un champ serveur neuf qui porterait le nom de la banque sous un autre nom). Le
  symptôme reste alors couvert par les tests de site (AC9) et l'E2E.
- **Ajout ou déplacement légitime** d'un `bankName` (par exemple la 26-1 qui retouche un profil d'import) : se
  traduit par une mise à jour **motivée** de la table, visible en revue — c'est le but.
- Mutations : M8 (rétablir `{b.bankName} — {b.iban}` dans `payment-batches/+page.svelte`, site D4) → rouge ;
  M8-bis (`title={b.bankName}` ajouté dans `SettleInvoiceDialog.svelte`) → rouge.

**AC12 — Manuel utilisateur et CHANGELOG.**

- `docs/manual/fr/user-manual.tex` : nouvelle `\section{Comptes bancaires}\label{sec:comptes-bancaires}` **juste
  avant** `\section{Import bancaire}` (C-26-2-13), qui dit : où se gèrent les comptes (*Administration → Comptes
  bancaires*) ; leurs champs (banque, **description facultative** de 60 caractères au plus, IBAN, QR-IBAN
  éventuel, compte comptable lié, compte principal) ; ce que fait la description — « UBS — Personnel » dans
  **les listes de comptes** (la page elle-même, l'accueil, *Paramètres*) et **dans les sélecteurs** (import
  bancaire, réconciliation, règlement d'une facture, règlement d'une facture fournisseur, lot de paiement), où
  l'IBAN reste affiché à la suite ; qu'elle n'est pas unique ; qu'un compte sans description s'affiche comme
  avant ; qu'elle ne se saisit pas à l'onboarding mais ensuite, et qu'elle **survit** à un changement de banque
  refait à l'onboarding (C-26-2-22) ; qu'elle ne figure **ni sur la QR-facture ni dans le fichier de paiement** ;
  qu'elle est dans l'export CSV et la sauvegarde ; et, pour qui modifie un compte par l'API, qu'un `PUT` sans la
  description l'efface. L'item « Compte bancaire » de
  l'onboarding (l. 202) renvoie à cette section (`\S\ref{sec:comptes-bancaires}`).
- PDF régénéré (`make fr` dans `docs/manual/`) et **contrôlé aplati** :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "description facultative[^.]*"`.
- Le manuel d'administration n'est pas touché (sa phrase « le message nomme le compte bancaire », l. 2472, reste
  vraie). Manuels DE/IT/EN : inexistants (README seuls) — rien.
- `CHANGELOG.md` : section `## [0.13.1] — Non publié` (créée si absente, **fusion par union** avec la 26-1 et la
  26-3) :
  - `### Ajouté` : la description facultative, où elle s'affiche, lien #622 ;
  - `### Modifié` (C-26-2-19, C-26-2-23) : le sélecteur de l'import (« UBS — CH… » au lieu de « UBS (CH…) »,
    seul changement visible sans description) ; **« Changement de contrat pour une intégration par clé d'API »**
    (formule du précédent de la 0.13.0) : `PUT /bank-accounts/{id}` remplace aussi `description` — l'omettre
    l'efface ; `PATCH` ne la touche pas ; et `details.bankName` du refus `CLAIM_ACCOUNT_LINKED_TO_BANK_ACCOUNT`
    porte désormais le nom complet du compte (« UBS — Personnel »).
  `README.md` « Feuille de route » : vérifié, a priori inchangé.
- `docs/api-external.md` (C-26-2-19) : à la ligne des comptes bancaires (l. 218) ou dans une note qui la suit,
  « *Depuis la v0.13.1* : les comptes bancaires portent un champ `description` (facultatif, 60 caractères au
  plus) ; `PUT /bank-accounts/{id}` le remplace comme les autres champs — **l'omettre l'efface** ; `PATCH` (lien
  au grand livre) ne le touche pas. » Patron des annotations « Depuis la v0.13.0 » du même fichier (l. 649-650).

### Les deux parties

**AC13 — E2E Playwright** (`frontend/tests/e2e/bank-accounts-crud.spec.ts`, seed `with-company`).

- **(a)** créer un compte « UBS E2E » avec la description « Épargne E2E » →
  `[data-testid="bank-account-name-{id}"]` montre « UBS E2E — Épargne E2E » (ou `toContainText` sur
  `bank-account-row-{id}`, qui porte aussi les badges) ; le modifier en effaçant la description → « UBS E2E »
  seul ; le champ porte
  `maxlength="60"` (attribut vérifié ; le refus serveur de 61 caractères est couvert par AC3, pas par l'E2E —
  `fill()` ne garantit pas l'effet de `maxlength`).
- **(b)** deux comptes de la même banque, l'un décrit, l'autre non → sur `/bank-import`, le sélecteur
  `[data-testid="bank-account-select"]` propose « UBS E2E — Épargne E2E — CH… » et « UBS E2E — CH… » (texte des
  `<option>`, sélecteurs par `data-testid`, jamais par libellé traduit) ; sur `/reconciliation`, même
  sélecteur, même constat ; sur l'accueil `/`, `[data-testid="homepage-bank-name-{id}"]` montre
  « UBS E2E — Épargne E2E ».
- **Isolement** (base `kesh_e2e` partagée entre specs) : d'autres specs choisissent le compte bancaire **par
  position** (`selectOption({ index: 1 })` dans `bank-import*.spec.ts`, `bank-csv-import.spec.ts`,
  `reconciliation*.spec.ts`) ou raisonnent sur le nombre de comptes (`payment-batches.spec.ts:75`). Chaque test de
  la 26-2 **archive** en fin de test (`afterEach`/`afterAll`, `DELETE /api/v1/bank-accounts/{id}` par le contexte
  API authentifié) les comptes qu'il a créés, crée des comptes **non principaux** (`isPrimary` décoché), avec des
  IBAN distincts de ceux des tests voisins. Vérifier en lançant la spec seule **et** dans la suite complète, et
  comparer les échecs des specs voisines à la liste attendue.
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
| M8 | rétablir `{b.bankName} — {b.iban}` dans `payment-batches/+page.svelte` (D4) | garde AC11 **et** `payment-batches-page.test.ts` |
| M9 | retirer `description` de `BankAccountJson` (companies) | test de `/companies/current` + E2E (b) |
| M10 | `first_active_bank_account_linked_to` rend `bank_name` seul | test du refus de désignation |
| M11 | retirer la colonne de `serialize_bank_accounts_csv` | `chaque_colonne_du_schema_est_exportee_ou_ecartee` |
| M12 | `flip_primary_off_for_company` écrit `description = NULL` | test de préservation à la démotion (AC2) |
| M13 | le `PUT` conserve la description absente (`COALESCE(?, description)`) | test « `PUT` sans le champ l'efface » (AC4) |
| M14 | retirer U+202E de l'ensemble refusé | test des caractères invisibles (AC3) |
| M15 | `BANK_ACCOUNT_DESCRIPTION_MAX_CHARS = 61` | test de la borne contre le squash (AC3) et test des 61 caractères |
| M16 | retirer `description` de `BankAccountJson` **du frontend** (`settings.types.ts`) et de la simulation | `npm run check` ou `settings-page.test.ts` (AC9, D9) |
| M8-bis | `title={b.bankName}` ajouté dans `SettleInvoiceDialog.svelte` | garde AC11 |

**Partie a** : M1-M7, M11-M15. **Partie b** : M8, M8-bis, M9, M10, M16. (M8 vise désormais D4, site sans E2E.)

Gates (CLAUDE.md § *Test Locally First*, consignes de l'epic) : `cargo fmt --check`, `clippy -D warnings`,
**gate backend complet** (story `kesh-db` : migration et dépôt — **pas** de gate ciblé, même en boucle de revue),
base de gate **remise à zéro avant** ; frontend `npm run check`, `lint-i18n-ownership`, `test:unit`, `build` ;
E2E complet (AC13). Le Dev Agent Record n'affirme que ce qui a tourné, avec ses chiffres.

## Tasks / Subtasks

- [ ] **T1 — Migration et garde-fous** (AC1) — partie a
  - [ ] T1.1 Rebaser sur `origin/main` **avec la 26-1 fusionnée** ; vérifier que `20261011000001` est la sienne ;
    écrire `20261011000002_bank_accounts_description.sql`.
  - [ ] T1.2 `scripts/regen-test-schema.sh` sur une base jetable (jamais `kesh-mariadb-dev` à redémarrer).
  - [ ] T1.3 Ligne P5 + recompte des cinq compteurs (80 / 80 / 10 / 70 / 0 attendus, la commande fait foi) ; P6
    `migrations_upgrade_path.rs` (80 / 46, généalogie, grep par jeton `\b(79|45)\b`) ; ligne ajoutée à
    `crates/kesh-db/migrations.sha384`.
- [ ] **T2 — Entité et dépôt** (AC2)
  - [ ] T2.1 Champ `description`, macro `columns!` (onze `SELECT`), `find_by_id_in_tx`, `update_for_company`,
    `BANK_ACCOUNT_DESCRIPTION_MAX_CHARS` + test contre le squash.
  - [ ] T2.2 `bank_account_display_name` + `display_name()` + tests unitaires.
  - [ ] T2.3 Tests de préservation (upsert, PATCH, archive, création par le dépôt, **démotion du principal**) dans
    `bank_accounts_repository.rs`.
- [ ] **T3 — Route** (AC3, AC4)
  - [ ] T3.1 `validate_bank_input` étendu (`is_forbidden_in_description`) ; bodies POST/PUT ; INSERT ; audit.
  - [ ] T3.2 Tests `crates/kesh-api/tests/bank_accounts_e2e.rs` : POST/PUT/GET/PATCH/DELETE ; bornes ; caractères
    invisibles ; ordre des refus ; non-unicité ; **`PUT` sans le champ efface** ; démotion ; audit.
- [ ] **T4 — Exports** (AC5) : CSV + test de valeur ; deux tests `admin_full_import_e2e.rs`.
- [ ] **T5 — Frontend socle** (AC6, AC7, AC8)
  - [ ] T5.1 `format.ts` (types, fonctions, `BANK_ACCOUNT_DESCRIPTION_MAX`) + `format.test.ts`.
  - [ ] T5.2 Types et `parseBankAccount` ; fixtures des huit fichiers de test.
  - [ ] T5.3 Écran *Comptes bancaires* + `BankAccountJournalLinkForm` + tests.
  - [ ] T5.4 Clés i18n ×4, recompte `ATTENDU`.
- [ ] **T6 — E2E (a)** (AC13 a, isolement) ; mutations M1-M7, M11-M15 (AC14) ; gates complets. *Fin de
  26-2a — PR `refs #622`.*
- [ ] **T7 — Rollout** (AC9) : D3 à D9, types partagés, `data-testid`, tests Vitest (dont les deux neufs D4,
  D9). — partie b, partie de `origin/main` après la fusion de la 26-2a
- [ ] **T8 — Serveur** (AC10) : `BankAccountJson` ; refus de désignation (renommage `bank_label`) + tests.
- [ ] **T9 — Garde d'inventaire** (AC11).
- [ ] **T10 — Manuel, PDF, CHANGELOG (`### Ajouté` et `### Modifié`), `docs/api-external.md`** (AC12).
- [ ] **T11 — E2E (b)** (AC13 b) ; mutations M8, M8-bis, M9, M10, M16 ; gates complets ; E2E complet au dernier
  commit de code. *PR `closes #622`.*

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
*(chemins relatifs à `frontend/`, indentation réduite ; 35 lignes dans **19 fichiers**, recomptées
(`grep -rln … | grep -v "\.test\.ts" | wc -l` → 19). L'issue dit « 20 fichiers » : son grep comptait sans doute un
fichier de test ; avec les tests, 27 — aucun site d'affichage n'est dans un fichier de test.)*

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

*Exclus — saisie de la banque, pas affichage d'un compte (ce sont les fichiers de la table de la garde AC11, qui
compte par fichier et non par ligne ; les numéros ci-dessous ne valent que pour `fe162504`) :*

- **E1** `routes/(app)/bank-accounts/+page.svelte:152, 209, 233` — préremplissage et payloads du formulaire ;
- **E2** `routes/onboarding/+page.svelte:121, 168, 174, 373`, `onboarding.svelte.ts:134, 137`,
  `onboarding.api.ts:64, 69` — saisie à l'onboarding, sans description (C-26-2-2) ;
- **E3** profils d'import, **homonymes et non comptes bancaires** : `BankProfileList.svelte:68`,
  `BankProfileForm.svelte:26, 68, 103`, `BankProfileSelector.svelte:53`, `bank-profile.types.ts:14` (le champ
  `bank_profiles.bank_name` est le nom du profil CSV) ;
- **E4** côté serveur, mêmes homonymes : `routes/bank_imports.rs:738, 1481, 1578, 1671`,
  `routes/bank_profiles.rs`, `crates/kesh-api/src/errors.rs:772-773, 1083` (`BankProfileSummary`), `csv_tables.rs:1046, 1062`,
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
crée une) ; `docs/api-external.md` l. 218 liste les routes sans leurs champs, mais le **changement de contrat du
`PUT`** s'y annote (AC12, C-26-2-19) ;
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

- **26-1** (#620, #621) ajoute la migration `20261011000001` et **fusionne la première** (C-26-2-25). La 26-2a
  part de — ou se rebase sur — `origin/main` qui la contient, puis **recompte** : cinq compteurs P5, `total` et
  soustracteur de `migrations_upgrade_path.rs` (et sa généalogie), squash régénéré, ligne de
  `migrations.sha384` ajoutée **après** celle de la 26-1. Jamais en additionnant de confiance. La 26-1 retouche
  aussi les profils d'import (E3) : la table de la garde AC11 (26-2b) se mesure sur l'arbre final.
- **26-3** (#616, #617, #619) réécrit deux lignes de la section *Import bancaire* du manuel, voisine de la section
  neuve (C-26-2-13) ; les deux régénèrent `user-manual.pdf` (conflit binaire : régénérer après rebase).
- **CHANGELOG** : les trois stories écrivent dans `## [0.13.1] — Non publié` — union.

### Pièges connus du dépôt, applicables ici

- **Story `kesh-db`** : gate complet même en boucle de revue (migration, dépôt) ; base de gate remise à zéro avant
  chaque gate complet ; P8 (ne plus toucher la migration une fois appliquée à `kesh_e2e_<clé>`).
- **Mutation restaurée, binaire périmé** : après `cp`/`mv` d'une mutation, `touch` le fichier.
- **Greper la valeur, pas la formulation** : après chaque patch, greper `\b79\b`, `\b45\b` (upgrade path laissé
  par la 26-1), `\b60\b` (longueur : migration, constantes, FTL, manuel, CHANGELOG, `api-external.md`),
  `bank_name` (renommage `bank_label`), `20261011000002` (migration, audit P5, `migrations.sha384`).
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
- Registre : `_bmad-output/implementation-artifacts/epic-26-choix-autonomes.md`, C-26-2-1 à C-26-2-25 (C-26-2-1 et
  C-26-2-14 remplacés par C-26-2-15 et C-26-2-25 ; C-26-2-6 complété par C-26-2-16).
- Rapports de validation : `/home/gcorbaz/devel/kesh-gate-logs/26-2-validate-p1-R.md`, `…-p1-F.md` ; prompts
  versionnés `26-2-validate-prompt-p1.md`, `26-2-validate-prompt-p2.md`.

## Dev Agent Record

### Agent Model Used

(à remplir par le développeur)

### Debug Log References

### Completion Notes List

- Fiche créée le 2026-10-10 (`bmad-create-story`, autonome) : analyse exhaustive du code sur `fe162504` —
  inventaire clos des sites, choix C-26-2-1 à 14 au registre de l'epic.
- Validation P1 remédiée le 2026-10-10 (même auteur que la fiche) : choix C-26-2-16 à 25 ; bilan au Change Log.

### File List

## Change Log

| Date | Étape | Résumé |
|---|---|---|
| 2026-10-10 | `bmad-create-story` | Fiche créée : 14 AC, inventaire clos (10 sites d'affichage, 3 angles morts assumés), découpage 26-2a / 26-2b proposé (C-26-2-1). |
| 2026-10-10 | découpage (orchestrateur) | Clés `26-2a-description-socle` (AC1-8, `refs #622`) et `26-2b-description-ecrans` (AC9-12, `closes #622`) au registre de sprint, clé mère `split` ; fiche unique en deux parties (C-26-2-15). |
| 2026-10-10 | validation P1 (Sonnet ×2, R et F) remédiée | Bilan ci-dessous. |

### Validation P1 — bilan finding par finding

Rapports : `kesh-gate-logs/26-2-validate-p1-R.md` (0 HIGH, 4 MEDIUM, 7 LOW) et `…-p1-F.md` (0 HIGH, 4 MEDIUM,
8 LOW) — **8 MEDIUM, 15 LOW**, dont quatre paires convergentes (R1 = F5, R3 = F1, R4 = F2, R10 = F9), soit 19 constats distincts (19 lignes ci-dessous). Tous traités ; aucun
reclassé en dette ; un **fait** réfuté (F6 : le précédent existe), sa conclusion appliquée quand même. Complément de l'orchestrateur en cours de remédiation : version de migration
`20261011000002` et registre `migrations.sha384` (C-26-2-25).

| Finding | Sév. | Traitement |
|---|---|---|
| R1 / F5 — fiche « non découpée », PR unique `closes` | M / L | Status et tableau réécrits : deux clés, deux PR, ordre, pas de release entre les deux (C-26-2-15). |
| R2 — D4 et D9 sans test de comportement | M | AC9 : tableau « un test par site » ; deux Vitest neufs (`payment-batches-page.test.ts`, `settings-page.test.ts`) ; `BankAccountJson` frontend à `description` obligatoire ; M8 déplacée sur D4, M16 neuve. |
| R3 / F1 — garde AC11 ambiguë, numéros de ligne, formes non vues | M / M | AC11 redéfinie : décompte du **jeton** par fichier (`.svelte` et `.ts`, tout le fichier), table fermée `{fichier → nombre, motif}` sans numéros de ligne, cas synthétiques par forme (balisage, `title=`, `@const`, script, déstructuration, `.ts`), anti-muet par nombre de fichiers et somme ; types en ligne remplacés par `BankAccountOptionSource` (tranché) ; angle mort nommé (C-26-2-18). |
| R4 / F2 — `PUT` qui efface : contrat non documenté, non testé | M / M | AC4 : test et M13 ; AC12 : `docs/api-external.md` « Depuis la v0.13.1 » et CHANGELOG `### Modifié` (C-26-2-19). |
| F3 — `flip_primary_off_for_company` oublié | M | AC2 : cinq écrivains recomptés, test de démotion, M12. |
| F4 — constante inimplémentable, grep de contrôle faux | M | AC2 : `macro_rules! columns` sur le précédent `accounts.rs:21-42`, `find_by_id_in_tx` pour la route, deux greps de contrôle justes (C-26-2-20). |
| R5 — pas de `data-testid` sur les noms | L | `bank-account-name-{id}`, `homepage-bank-name-{id}`, `settings-bank-name-{id}` (AC7, AC9, AC13). |
| R6 — P6 : `closing_accounts_backfill.rs` omis ; généalogie | L | Ajoutés à AC1. |
| R7 — chemins `errors.rs` ambigus | L | Qualifiés (AC10, Dev Notes E4). |
| R8 — `MODIFY COLUMN` dit non breaking | L | C-26-2-3 rectifié au registre : élargir est un `MODIFY COLUMN` (P3) — bump ou `-- breaking-skip-bump` justifié (P4). |
| R9 — ordre des refus et non-unicité non épinglés | L | Deux tests (AC3). |
| R10 / F9 — Unicode Cf, bidi, U+2028 ; accents décomposés | L / L | Ensemble refusé fermé et nommé, `is_forbidden_in_description`, M14 ; NFC en angle mort (C-26-2-17). |
| R11 — « 20 fichiers » de l'issue contre 35 lignes | L | 19 fichiers hors tests, 27 avec, écart expliqué (Dev Notes). |
| F6 — `AFTER` + `INSTANT` sans précédent | L | **Fait réfuté** : le précédent existe, `20260507200001_bank_account_journal_link.sql` l. 21-22 (`AFTER qr_iban,` puis `ALGORITHM=INSTANT` à la ligne suivante — le grep mono-ligne de la lentille ne pouvait pas le voir ; R l'a relevé). Conclusion gardée quand même : colonne **en fin de table**, la position n'apportant rien (C-26-2-16) ; place dans le CSV inchangée. |
| F7 — sauvegarde 0.13.1 refusée par une 0.13.0 | L | Dit en AC1 — attendu de toute façon, la 26-1 bumpe `min_required` à 0.13.1 (C-26-2-24). |
| F8 — pollution inter-specs E2E, assertion de ligne | L | AC13 : archivage des comptes créés, comptes non principaux, `toContainText` / `data-testid`. |
| F10 — 60 écrit en six endroits | L | Constantes Rust et TS, test du squash contre la constante, M15 (C-26-2-21). |
| F11 — description qui survit à un changement de banque à l'onboarding | L | AC2 et manuel ; C-26-2-22. |
| F12 — `details.bankName` change de sens | L | CHANGELOG `### Modifié` ; lecture du message à l'écran (AC10) ; C-26-2-23. |

Décomptes après remédiation (recomptés) : **14 AC** ; mutations **M1-M16 + M8-bis = 17** ; sites d'affichage
**10** (D1-D10), chacun avec un test de comportement.

Propagation (grep de la valeur après correction) : `20261011000001` → ne reste que comme migration de la 26-1 ;
`\b78\b`/`\b44\b` → seulement l'état de `fe162504` et la généalogie citée ; `AFTER` → seulement la phrase qui
l'écarte ; `C-26-2-14` → seulement « remplacé par C-26-2-25 » ; `maxlength="60"` → seulement l'attribut **rendu**
vérifié par l'E2E.
