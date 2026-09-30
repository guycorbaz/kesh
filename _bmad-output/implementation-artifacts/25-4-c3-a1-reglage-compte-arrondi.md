# Story 25.4-c3-a1 : Le réglage du compte de différences d'arrondi

Status: ready-for-dev

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
- **suppression** : bloquée par la FK (vérifier que l'API rend un refus lisible, pas un 500) ;
- **archivage** : vérifier comment l'archivage d'un compte TVA désigné est traité aujourd'hui, et
  faire de même pour ce compte — s'il n'y a **aucune** garde pour les comptes TVA non plus, le dire et
  ouvrir une issue plutôt que de l'inventer ici.

**AC 6 — Export et sauvegarde.** L'export CSV de la table porte la colonne (garde d'exhaustivité verte).
Un `.keshbackup` produit **avant** cette migration s'importe toujours : la colonne manquante vaut `NULL`
— test existant à étendre ou test neuf (`admin_backup_e2e` / `admin_full_import_e2e`).

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

- [ ] **T1 — migration et squash** (AC 1).
- [ ] **T2 — entité, dépôt, route, validation** (AC 2, 3, 5).
- [ ] **T3 — export et sauvegarde** (AC 6).
- [ ] **T4 — écran et i18n** (AC 4).
- [ ] **T5 — tests** (AC 7).
- [ ] **T6 — textes** (AC 8).
- [ ] **T7 — gates** : backend complet (base remise à zéro ; ⛔ `kesh-db` touché ⇒ gate complet même en
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

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-09-30** — Créée au découpage de la 25-4-c3 (sept modules), selon les arbitrages de Guy : réglage
  dans les paramètres, compte créable dans le plan et choisi ici ; les plans livrés et l'onboarding à la
  c3-a2 ; l'arrondi et l'écriture à la c3-b.

[#476]: https://github.com/guycorbaz/kesh/issues/476
