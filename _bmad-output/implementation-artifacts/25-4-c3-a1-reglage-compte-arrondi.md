# Story 25.4-c3-a1 : Le réglage du compte de différences d'arrondi

Status: done

**Issue : [#476]** — ⛔ la PR porte `refs #476` (la sœur **25-4-c3-b** la fermera).

**Née du découpage de la 25-4-c3** (règle de splitting : sept modules). Arbitrages de Guy le
2026-09-30 : l'écart d'arrondi au centime se **passe en écriture** sur un **compte de différences
d'arrondi** ; ce compte est désigné par **un réglage dans les paramètres** (pas un rôle de compte) ; il
doit pouvoir être **créé dans le plan comptable et choisi dans les paramètres** ; les plans livrés en
proposeront un (« 6940 Différences d'arrondi », charge), désigné d'office à l'onboarding — c'est la
sœur **25-4-c3-a2**. La sœur **25-4-c3-b** fera l'arrondi et passera l'écart en écriture, en lisant ce
réglage.

**Empilée sur la 25-4-c2** (PR #485, non mergée) : branche `story/25-4-c3-arrondi-centime`.

## Story

En tant qu'administrateur de la comptabilité,
je veux désigner, dans les paramètres de facturation, le compte qui reçoit les différences d'arrondi,
afin que Kesh sache où passer l'écart d'un demi-centime quand un paiement arrondi solde une facture.

## Ce qui existe, vérifié dans le code

- **Le précédent : les comptes de TVA** (Story 18-1a). Trois colonnes facultatives de
  `company_invoice_settings` (`default_vat_payable_account_id`, `default_vat_recoverable_account_id`,
  `default_vat_decompte_account_id`), ajoutées par `20260614000001_vat_accounts_config.sql:53-62` (FK
  `ON DELETE RESTRICT`), que l'onboarding ne remplit pas et qui se règlent dans *Paramètres →
  Facturation*. C'est le patron à suivre, à la différence près des types et de la postabilité (plus bas).
- **L'entité** : `crates/kesh-db/src/entities/company_invoice_settings.rs:20-40` (+ la structure de mise
  à jour qui suit, remplacement intégral).
- **Le dépôt** : `crates/kesh-db/src/repositories/company_invoice_settings.rs` — liste des colonnes
  (`:28`), JSON d'audit (`:41`), comparaison « rien n'a changé » (`:120`), `update` (`:131`, `:174-186`),
  `insert_with_defaults` (`:277`, projection `:358`) et son miroir `insert_with_defaults_in_tx` (`:403`,
  projection `:476`) — ⚠️ commentaire `MIRROR` : les deux se tiennent à la main.
- **La route** : `crates/kesh-api/src/routes/company_invoice_settings.rs` — `validate_account`
  (`:94-124`) contrôle société, existence, `active` et **un** type attendu ; les appels (`:~175-222`).
  ⚠️ Elle **ne contrôle pas `postable`**.
- **L'écran** : `frontend/src/routes/(app)/settings/invoicing/+page.svelte` — listes filtrées par type
  (`:51-57`, `a.active && a.postable && a.accountType === …`), `withCurrentAccount` (`:70`) pour garder
  visible un compte déjà choisi, chargement (`:91`), envoi (`:124`), relecture après conflit (`:146`),
  sélecteur (`:325-328`). Type : `frontend/src/lib/features/invoices/invoices.types.ts`.
- **L'export de souveraineté** : `crates/kesh-api/src/exports/csv_tables.rs:905-946`
  (`serialize_company_invoice_settings_csv`) — la garde d'exhaustivité de la 25-5-a le fera rougir si la
  colonne n'y est pas.
- **La sauvegarde** : `crates/kesh-db/src/backup.rs:66` liste la table ; un `.keshbackup` **antérieur**
  n'aura pas la colonne.
- **Le manuel** : `docs/manual/fr/admin-manual.tex:1994` (*Configuration des comptes TVA*, depuis
  *Paramètres → Facturation*).

## Acceptance Criteria

**AC 1 — La colonne.** Migration **non breaking** : `company_invoice_settings.default_rounding_account_id
BIGINT NULL`, FK vers `accounts(id)` `ON DELETE RESTRICT` (patron `20260614000001`). DDL seul, aucune
donnée écrite (⇒ pas de triage P7). Garde-fous de la § *Migration breaking policy* : ligne à
`docs/migrations-idempotence-audit.md` et **ses compteurs recomptés** (P5) ; `grep -rn
"migrations.len()\|apply_migrations_up_to" crates/` inspecté (P6) ; squash `test-schema` régénéré et
`test_schema_guard` vert ; tout manifeste de sommes de contrôle des migrations mis à jour
(`migrations.sha384` si présent, précédent 24-5). ⛔ P8 : une fois appliquée, la migration ne se modifie
plus.

**AC 2 — L'entité, le dépôt, l'API.** Le champ `default_rounding_account_id: Option<i64>` (JSON
`defaultRoundingAccountId`) traverse l'entité, la mise à jour, **toutes** les projections du dépôt
(`update`, les deux `insert_with_defaults*`, `get_or_create_default*`), le JSON d'audit et la comparaison
« rien n'a changé ». `GET`/`PUT /company/invoice-settings` le portent. `insert_with_defaults*` le laissent
`NULL` (la 25-4-c3-a2 le posera à l'onboarding).

**AC 3 — La validation.** Le compte choisi doit appartenir à la société, être **actif**, être
**imputable** (`postable`) — les écritures automatiques passent `enforce_postable = false`, c'est donc
ici que la garde doit tenir (`invoice_settlements_write.rs:136-161`) —, et être de type **charge
(`Expense`) ou produit (`Revenue`)** : un écart d'arrondi est un résultat, dans un sens ou dans
l'autre. `validate_account` est étendue en conséquence (liste de types acceptés, contrôle de
`postable`) **sans changer** le comportement des champs existants — ⚠️ ne pas imposer `postable` aux
comptes TVA sans le dire : c'est hors périmètre, à signaler en issue si c'est un trou.

**AC 4 — L'écran.** *Paramètres → Facturation* : un sélecteur « Compte de différences d'arrondi »,
facultatif, listant les comptes actifs, imputables, de charge ou de produit ; un compte déjà choisi
reste visible même s'il sort du filtre (`withCurrentAccount`). Un texte d'aide dit à quoi il sert et
qu'on peut le **créer dans le plan comptable** puis le choisir ici. Libellés dans les 4 locales ;
`lint-i18n-ownership` et la garde `i18n-keys.test.ts` (compteur de sites recompté).

**AC 5 — Les gardes voisines.** Un compte désigné ici ne peut pas disparaître en silence :
- **suppression** : **sans objet** — aucune route ne supprime un compte (`crates/kesh-api/src/lib.rs:340-352` :
  création, modification, archivage, réactivation seulement) ; la FK `ON DELETE RESTRICT` reste le
  filet en base ;
- **archivage** : **aucune garde n'existe aujourd'hui pour les comptes TVA désignés** (validation P1).
  Ne pas l'inventer ici : ouvrir une issue qui couvre les **quatre** comptes désignés dans les
  réglages (trois TVA et l'arrondi), et le dire dans le Dev Agent Record. La c3-b refusera de toute
  façon d'écrire sur un compte archivé ou non imputable (garde au moment de l'écriture).

**AC 6 — Export et sauvegarde.** L'export CSV de la table porte la colonne (garde d'exhaustivité verte).
La sauvegarde, elle, lit ses colonnes dans `INFORMATION_SCHEMA` (`export_table`, `column_constraints`) :
elle suit d'elle-même. Un `.keshbackup` produit **avant** cette migration s'importe toujours
(`check_schema_compat` : colonne absente, facultative) et la colonne vaut `NULL` — le prouver par un
test, patron `full_import_without_company_column_merges_archive_entries_as_null`
(`crates/kesh-api/tests/admin_full_import_e2e.rs:2167`, technique `strip_column`). Un `.keshbackup`
postérieur importé par un binaire antérieur est refusé en `400 IMPORT_SCHEMA_MISMATCH` (mécanisme
générique, déjà testé) : rien à faire.

**AC 7 — Tests.** Dépôt : le champ survit à `update` et à la relecture ; validation API : refus d'un
compte d'une autre société, archivé, **non imputable**, de type actif/passif ; acceptation d'une charge
et d'un produit ; Vitest de l'écran (options filtrées, compte courant conservé) ; E2E Playwright :
choisir un compte, enregistrer, relire.

**AC 8 — Textes.** `admin-manual.tex` : à côté des comptes TVA, un paragraphe *Compte de différences
d'arrondi* — à quoi il sert (renvoi à l'écart d'un demi-centime qu'un paiement arrondi laisse ; la
25-4-c3-b le passera en écriture), qu'il est facultatif **tant qu'aucun écart ne se présente**, qu'on
peut le créer dans le plan comptable (charge ou produit, imputable) puis le choisir. PDF régénéré et
contrôlé aplati. `docs/api-external.md` si les réglages de facturation y sont décrits. CHANGELOG
`[0.12.1]` *Added*.

## Tasks / Subtasks

- [x] **T1 — migration et squash** (AC 1).
- [x] **T2 — entité, dépôt, route, validation** (AC 2, 3, 5).
- [x] **T3 — export et sauvegarde** (AC 6).
- [x] **T4 — écran et i18n** (AC 4).
- [x] **T5 — tests** (AC 7).
- [x] **T6 — textes** (AC 8).
- [x] **T7 — gates** : backend complet (base remise à zéro ; ⛔ `kesh-db` touché ⇒ gate complet même en
  cours de boucle), frontend complet, **E2E complet**.

## Dev Notes

### Ce qu'il ne faut pas faire

- ⛔ **Un rôle de compte** : arbitrage de Guy — c'est un réglage. Un rôle serait de plus une migration
  breaking (un binaire antérieur ne décode pas un rôle inconnu).
- ⛔ **Écrire des données dans la migration** (créer le compte 6940 chez les sociétés existantes) : les
  sociétés existantes le créent dans le plan et le choisissent (arbitrage) ; les plans livrés, c'est la
  c3-a2.
- ⛔ **Un numéro de compte dans le code applicatif** (`14-3a`, migration `20260722000001:3-8`).
- ⛔ **Utiliser le réglage pour écrire** : c'est la c3-b.
- ⚠️ **Les deux `insert_with_defaults*` sont miroirs** : les modifier ensemble.
- **Sans objet** : `docs/optimistic-locking-patterns.md:48` énumère les champs comparés par `update` —
  instantané de la Story 7-3, jamais tenu à jour depuis (il omet déjà les trois comptes TVA,
  `credit_note_number_format` et `default_payable_account_id`). Ne pas le mettre à jour ici.
- ⚠️ `migrations_upgrade_path.rs` porte `assert_eq!(total, 69, …)` : il rougira — c'est son rôle
  (P6) ; le passer à 70 **et** vérifier l'assertion de montage qui l'accompagne.
- ⚠️ Aucun E2E n'est dédié à *Paramètres → Facturation* : le spec de l'AC 7 est à créer.

### Où regarder

| Fichier | Pourquoi |
|---|---|
| `crates/kesh-db/migrations/20260614000001_vat_accounts_config.sql` | patron de la colonne |
| `crates/kesh-db/src/entities/company_invoice_settings.rs` | entité et mise à jour |
| `crates/kesh-db/src/repositories/company_invoice_settings.rs` | projections, `update`, audit |
| `crates/kesh-api/src/routes/company_invoice_settings.rs` | `validate_account`, `update_invoice_settings` |
| `crates/kesh-api/src/exports/csv_tables.rs:905-946` | export |
| `crates/kesh-db/src/backup.rs` | sauvegarde, import d'une sauvegarde antérieure |
| `frontend/src/routes/(app)/settings/invoicing/+page.svelte` | écran |
| `crates/kesh-db/tests/company_invoice_settings_repository.rs`, `crates/kesh-api/tests/idor_multi_tenant_e2e.rs` | tests existants qui énumèrent les champs |
| `docs/manual/fr/admin-manual.tex:1994` | manuel |

### Gardes-fous du dépôt

- Modules : `kesh-db`, `kesh-api`, `frontend`, `kesh-i18n` (+ `docs`) — sous le seuil.
- Migration : P5, P6, P8 ; P7 sans objet (DDL seul) ; squash et `test_schema_guard`.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (`claude-opus-5-5`).

### Debug Log References

- Mutations (restaurées, fichier touché ensuite) : garde `postable` neutralisée → `settings_rounding_account_is_validated`
  rouge (« charge non imputable » acceptée) ; préservation de l'absent remplacée par `None` →
  `settings_rounding_account_absent_preserves_null_clears` rouge. Le test d'import antérieur est
  discriminant par construction : sans le retrait de la colonne, l'import restaurerait le compte.
- Le jeu E2E `with-company` ne porte que deux comptes de résultat (3000, 4000) : le Playwright choisit
  la charge 4000, pas un compte du plan PME.
- Deux gardes frontend ont rougi et ont été suivies : le compteur de sites i18n (1752 → 1756, recompté
  32 → 36 dans la page) et `e2e-selecteurs-traduits` (le spec visait « Enregistrer » par son libellé) →
  `data-testid="settings-invoicing-save"`.
- Gate E2E : 229 passed / 19 skipped / 7 failed — les 7 KF-029 (#97), rien d'autre (run à 15:55 UTC).

### Completion Notes List

- **Migration** `20260930000001_invoice_settings_rounding_account.sql` : `ADD COLUMN
  default_rounding_account_id BIGINT NULL` + FK `fk_cis_rounding` `ON DELETE RESTRICT`, DDL seul, non
  breaking. P5 : ligne d'audit `tracked-by-sqlx`, compteurs **recomptés** (70 fichiers = 70 lignes ;
  8 + 62 + 0). P6 : `migrations_upgrade_path` 69 → 70 **et** soustracteur 35 → 36 (frontière 34
  inchangée), valeur grepée dans tout le fichier (sept sites mis à jour, les historiques laissés). P7
  sans objet. P8 : ligne ajoutée à `migrations.sha384`. Squash régénéré (`scripts/regen-test-schema.sh`,
  rejeu vérifié).
- **Dépôt et entité** : le champ traverse `COLUMNS`, le JSON d'audit, `is_no_op_change`, l'`UPDATE` et
  les deux projections miroirs de `insert_with_defaults*` (qui le laissent `NULL`).
- **API** : `GET`/`PUT /company/invoice-settings` portent `defaultRoundingAccountId`. **Absent du corps
  d'un `PUT` : préservé** (comme `creditNoteNumberFormat`, #216) ; **à `null` : effacé** — par
  `double_option`, déplacé de `reconciliation_rules.rs` vers `crate::helpers` (partagé, DRY).
  `validate_account_of` généralise `validate_account` (plusieurs types, postabilité exigée ou non) sans
  changer les champs existants ; le compte d'arrondi exige charge **ou** produit, actif, **imputable**,
  de la société.
- **Export** : colonne ajoutée au CSV. **Sauvegarde** : dynamique (`INFORMATION_SCHEMA`), rien à changer ;
  l'import d'un backup antérieur est prouvé par test.
- **Écran** : section *Différences d'arrondi* dans *Paramètres → Facturation* — sélecteur filtré (actifs,
  imputables, charges et produits), compte courant conservé (`withCurrentAccount`), texte d'aide qui
  renvoie au plan comptable ; trois clés dans les quatre catalogues.
- **AC 5** : aucune route ne supprime un compte ; l'archivage non gardé des comptes désignés (TVA et
  arrondi) → **issue #486**.
- **Tests** (périmètre : `e52cde9f` → cette branche) : +1 dépôt, +2 API (validation sur sept cas,
  absent / `null`), +1 import de sauvegarde, +3 Vitest (nouveau fichier), +1 Playwright (nouveau spec).
- **Textes** : `admin-manual.tex` — paragraphe *Compte de différences d'arrondi*, qui dit que le réglage
  n'est encore lu par aucune écriture ; PDF régénéré, contrôlé aplati. CHANGELOG `[0.12.1]` *Added*.
  `docs/api-external.md` ne décrit pas ces réglages : sans objet.

### File List

- `CHANGELOG.md`
- `crates/kesh-api/src/exports/csv_tables.rs`
- `crates/kesh-api/src/helpers.rs`
- `crates/kesh-api/src/routes/company_invoice_settings.rs`
- `crates/kesh-api/src/routes/reconciliation_rules.rs`
- `crates/kesh-api/tests/admin_full_import_e2e.rs`
- `crates/kesh-api/tests/idor_multi_tenant_e2e.rs`
- `crates/kesh-db/migrations.sha384`
- `crates/kesh-db/migrations/20260930000001_invoice_settings_rounding_account.sql`
- `crates/kesh-db/src/entities/company_invoice_settings.rs`
- `crates/kesh-db/src/repositories/company_invoice_settings.rs`
- `crates/kesh-db/test-schema/0001_schema_squash.sql`
- `crates/kesh-db/tests/company_invoice_settings_repository.rs`
- `crates/kesh-db/tests/migrations_upgrade_path.rs`
- `crates/kesh-i18n/locales/de-CH/messages.ftl`
- `crates/kesh-i18n/locales/en-CH/messages.ftl`
- `crates/kesh-i18n/locales/fr-CH/messages.ftl`
- `crates/kesh-i18n/locales/it-CH/messages.ftl`
- `docs/manual/fr/admin-manual.pdf`
- `docs/manual/fr/admin-manual.tex`
- `docs/migrations-idempotence-audit.md`
- `frontend/src/lib/features/invoices/invoices.types.ts`
- `frontend/src/lib/shared/i18n-keys.test.ts`
- `frontend/src/routes/(app)/settings/invoicing/+page.svelte`
- `frontend/src/routes/(app)/settings/invoicing/settings-invoicing-page.test.ts`
- `frontend/tests/e2e/settings-rounding-account.spec.ts`
- `_bmad-output/implementation-artifacts/25-4-c3-a1-reglage-compte-arrondi.md`
- `_bmad-output/implementation-artifacts/25-4-c3-a1-validate-prompt-p1.md`
- `_bmad-output/implementation-artifacts/25-4-c3-a1-validate-prompt-p2.md`
- `_bmad-output/implementation-artifacts/25-4-c3-a1-review-prompt-p1.md`
- `_bmad-output/implementation-artifacts/25-4-c3-a1-review-prompt-p2.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`

## Change Log

- **2026-09-30** — Créée au découpage de la 25-4-c3 (sept modules), selon les arbitrages de Guy : réglage
  dans les paramètres, compte créable dans le plan et choisi ici ; les plans livrés et l'onboarding à la
  c3-a2 ; l'arrondi et l'écriture à la c3-b.
- **2026-09-30** — Validation P1 (Sonnet, prompt `25-4-c3-a1-validate-prompt-p1.md`) : **2 MEDIUM**,
  vérifiés. AC 5 visait une suppression de compte que l'API n'offre pas → sans objet ; l'archivage non
  gardé des comptes TVA → issue couvrant les quatre comptes des réglages. `optimistic-locking-patterns.md:48`
  → écrit sans objet (instantané figé). Faits utiles reportés : sauvegarde dynamique, précédent de test
  `strip_column`, `assert_eq!(total, 69)`, pas d'E2E de l'écran. Toutes les références exactes ; P5
  sain (69 = 69, 8 + 61 + 0).
- **2026-09-30** — Validation P2 **ciblée** (Haiku, prompt `25-4-c3-a1-validate-prompt-p2.md`, sur
  `06c075be`) : **0 finding**, preuves jointes sur les quatre axes (routes d'archivage relues,
  `INFORMATION_SCHEMA` et `is_required()` lus, test `strip_column` et `assert_eq!(total, 69)` trouvés,
  P5 recompté). La remédiation ne touchait que la fiche : **validation close, 0 > LOW.**

  **Bilan** — P1 Sonnet 2 MEDIUM → P2 Haiku ciblée 0. Modèles : Sonnet, Haiku.
- **2026-09-30** — Implémentée (`bmad-dev-story`). Gates **réellement exécutés** : backend complet sur base
  remise à zéro **2520/2520** (4 ignorés), fmt, clippy ; frontend `check`, `lint-i18n-ownership`,
  **839/839**, build ; **E2E 229 / 19 / 7** (les 7 KF-029). Issue #486 ouverte. Statut → `review`.
- **2026-09-30** — Revue de code P1 (3 lentilles Sonnet, prompt `25-4-c3-a1-review-prompt-p1.md`, sur
  `55d98555`) : Blind Hunter 2 MEDIUM / 9 LOW, Edge Case Hunter 1 HIGH / 1 MEDIUM, Acceptance Auditor **0**
  (décomptes recomptés depuis la source). Vérifiés. **Corrigés** : HIGH (Edge) — le compte d'arrondi était
  revalidé à chaque `PUT` même reconduit tel quel : devenu archivé ailleurs (#486), il aurait bloqué tout
  enregistrement des paramètres, format de numérotation compris → **validé seulement s'il change** ; test
  `settings_unchanged_archived_rounding_account_does_not_block_other_changes`, éprouvé par mutation ; la
  même face du défaut pour les comptes TVA, hors périmètre, **commentée sur #486**. MEDIUM (Blind + Edge) —
  le Playwright laissait le réglage posé s'il échouait en route → `try/finally`, patron de
  `company-contact-details.spec.ts`. LOW retenus : cas « produit non imputable / archivé » ajoutés ;
  `double_option` en `pub(crate)`. **Laissés LOW** : message « A ou B » pour un troisième type futur ; type
  SQL `BIGINT` (identique au patron TVA, squash vérifié par Edge) ; `toMatchObject` du Vitest ; pas de test
  du 400 ni du 409 côté écran. Gates ciblés : `idor_multi_tenant_e2e` 3/3 sur l'arrondi, Playwright vert,
  garde des sélecteurs E2E verte, fmt, clippy ; **`kesh-db` non touché** par la remédiation.
- **2026-09-30** — Revue de code P2 **ciblée** (Haiku, prompt `25-4-c3-a1-review-prompt-p2.md`, sur
  `aa76eb2c`) : **1 LOW** (le `finally` du Playwright pourrait masquer l'erreur d'origine s'il échouait
  lui-même), 0 > LOW, quatre axes prouvés. Recoupée par l'orchestrateur sur la course entre deux `PUT` :
  une valeur reconduite est celle en place, donc validée quand elle fut posée ; un `PUT` concurrent qui la
  change bouge `version`, et le nôtre sort en 409. **Revue close.** Gate complet au dernier commit : backend
  **2521/2521** (base remise à zéro), frontend **839/839**, **E2E 228 / 19 / 8** — les 7 KF-029, et
  `products.spec.ts:109`, **vert rejoué seul deux fois** (pollution d'état, famille déjà relevée à la liste
  des échecs attendus : `products.spec.ts:166`).

  **Bilan de la revue** — P1 Sonnet ×3 : 1 HIGH / 2 MEDIUM corrigés, LOW triés → P2 Haiku ciblée : 0 > LOW.
  Statut → `done`.

[#476]: https://github.com/guycorbaz/kesh/issues/476
