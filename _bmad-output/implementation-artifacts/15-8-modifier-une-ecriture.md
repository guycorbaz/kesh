# Story 15.8 : Modifier une écriture tant que son exercice est ouvert

Status: ready-for-dev

**Issue : [#532]** — CR du Project Lead, 2026-10-08, constat de recette de la v0.12.1. ⚠️ **URGENTE.**
**Révise la Story 24-4b** (`24-4b-gel-ecriture.md`, gel inconditionnel, #380).
Choix autonomes : **C-15-8-1 à C-15-8-9** (`epic-15-choix-autonomes.md`).

## Story

**As a** personne qui tient les livres,
**I want** pouvoir corriger ou supprimer une écriture saisie à la main — l'écriture d'ouverture comprise — tant que
son exercice n'est pas clôturé et que sa période n'est pas verrouillée,
**so that** une inversion de débit et de crédit se corrige là où elle est, sans trois écritures pour une, et sans que
la correction cesse d'être apparente.

## Le déclencheur, et ce qui change

Guy a saisi ses soldes de départ (25-7) avec débit et crédit inversés. Depuis la 24-4b, toute écriture est gelée dès
son insertion : `PUT` et `DELETE /api/v1/journal-entries/{id}` rendent **409 `ENTRY_IS_POSTED`**, et la seule
correction est la contre-passation datée du jour — trois écritures (l'ouverture fausse, son inverse, une OD juste), et
les comptes deviennent « mouvementés », donc **non complétables** (25-7).

> « La majorité des logiciels comptables permettent ces modifications tant que l'exercice comptable n'est pas
> bouclé : je veux cette fonctionnalité dans Kesh. » — Guy, #532

**Cette story rouvre la modification ET la suppression** (C-15-8-2) d'une écriture, dans un cadre unique.

## Ce qui ne change pas — les motifs de la 24-4b qui restent valables (normatif, #532)

| motif de la 24-4b | ce qu'il devient ici |
|---|---|
| **La correction doit être apparente** (art. 958f CO, Olico art. 3) | chaque modification écrit `journal_entry.updated` avec l'instantané **avant et après, lignes comprises**, l'utilisateur (ou la clé) et l'horodatage ; chaque suppression écrit `journal_entry.deleted` avec l'instantané (déjà le cas, `delete_in_tx` étape 5) |
| **Les écritures d'une pièce restent gelées** — leur montant est lu ailleurs (`invoice_settlements.amount`, reste dû) | refus pour **toute** écriture référencée par une table métier — l'inventaire est celui de `reversal_blockers` (D2), fermé par un garde-fou (D3) |
| **Contre-passée et contre-passation restent gelées** | `ENTRY_IS_REVERSED` (contre-passée) et `IS_A_REVERSAL` (contre-passation) |
| **Période verrouillée et exercice clôturé intouchables** | `FISCAL_YEAR_CLOSED` (400) ; `PERIOD_LOCKED` (400) sur l'**ancienne ET la nouvelle** date |
| **Mêmes contrôles qu'à la saisie** | équilibre (`kesh_core::accounting::validate`), comptes **actifs et imputables** sans exemption (C-15-8-4), nouvelle date dans l'exercice **de l'écriture** (C-15-8-3) |

## Décisions

### D1 — Le cadre : ce qui se modifie, ce qui ne se modifie pas

Une écriture est **modifiable et supprimable** par la route si, et seulement si, toutes ces conditions tiennent :

1. son exercice est **ouvert** ;
2. elle n'est **pas contre-passée** et n'est **pas une contre-passation** ;
3. **aucune pièce** ne la référence (facture, avoir, facture fournisseur — achat ou règlement —, règlement client —
   solde compris —, transaction bancaire rapprochée) ;
4. sa date (et, pour une modification, la **nouvelle** date) est **postérieure** à `books_locked_through`.

⇒ En pratique : les **écritures saisies à la main**, l'**écriture d'ouverture** (25-7) et les **compléments de soldes
de départ** (25-7) — qu'aucune table ne référence, cf. D6.

⚠️ **Un compte archivé ne gèle PAS l'écriture** (contrairement à la contre-passation, où `ACCOUNT_ARCHIVED` masque
le bouton) : on peut remplacer le compte. Mais l'enregistrement refuse tant qu'une ligne vise un compte inactif ou
non imputable (C-15-8-4) — **y compris une ligne qu'on n'a pas touchée**.

### D2 — Où vit la garde : au repository, et l'inventaire n'est PAS réécrit

⛔ **Un garde-fou posé à la route ne protège que la route** (24-4b D2) : les refus vivent dans
`crates/kesh-db/src/repositories/journal_entries.rs`.

⛔ **L'inventaire des propriétaires existe déjà, une seule fois** : `reversal_blockers` (`journal_entries.rs:1265`),
une requête à sous-requêtes corrélées sur les **huit** références. La garde de modification le réutilise, filtré :

| motif de `reversal_blockers` | modification / suppression |
|---|---|
| `IsAReversal` | refus `IS_A_REVERSAL` (409) |
| `AlreadyReversed` | refus `ENTRY_IS_REVERSED` (409, **existant** — contrat de la 24-4a, testé) |
| `OwnedByInvoice`, `OwnedByCreditNote`, `OwnedBySupplierInvoice`, `OwnedBySettlement`, `MatchedBankTransaction` | refus sous le **code du motif** (409), `details.documentId` / `details.documentNumber` |
| `AccountArchived` | ⛔ **ignoré** — ce n'est pas un gel (D1) |

Nouveau `DbError::EntryNotModifiable { blocker: ReversalBlocker, document_id: Option<i64>, document_label: Option<String> }`
(`crates/kesh-db/src/errors.rs`), mappé côté `kesh-api` **exactement** comme `EntryNotReversable` (`errors.rs:2698`) :
même code (celui du motif), même corps `details`, **mêmes clés i18n** — leurs messages nomment déjà le chemin de la
pièce (« dévalidez la facture, ou corrigez-la par un avoir »). ⛔ **DRY** : extraire le corps du mappage dans une
fonction partagée par les deux variantes, ne pas le recopier. Code machine : `DbError::code()` (`errors.rs:759`) rend
`blocker.code()` pour la nouvelle variante comme pour l'ancienne.

**Sérialisation** : chaque propriétaire pose sa référence par une **clé étrangère vers `journal_entries`**, dont le
contrôle prend un verrou **partagé** sur la ligne de l'écriture. La garde s'évalue **sous** le `FOR UPDATE` de
l'écriture (étape 2 de `delete_in_tx`, étape 3 de `update`) : une pièce qui s'attache en concurrence attend, et l'une
des deux transactions voit l'autre. *(À citer dans le doc-comment ; ne pas lire `reversal_blockers` sur le pool hors
transaction dans le chemin d'écriture.)*

### D3 — Le garde-fou d'inventaire : un ensemble clos, pas une énumération

⛔ **Une nouvelle table qui référence `journal_entries` passerait sous la garde sans que rien ne rougisse** — c'est
exactement la classe d'un test muet. Règle du `CLAUDE.md` § *Inventorier les sites NON RÉSOLUS* : on inventorie
l'ensemble clos, on n'énumère pas les formes connues.

Test d'intégration (`#[sqlx::test]`, base **migrée par `MIGRATOR`**, pas le squash — c'est le schéma réel qu'il
contrôle ; ⚠️ il doit alors figurer dans la liste en dur de `crates/kesh-db/tests/test_schema_guard.rs`, cf.
`CLAUDE.md` § *Gate rapide*) :

1. `information_schema.KEY_COLUMN_USAGE` où `REFERENCED_TABLE_NAME = 'journal_entries'` doit égaler **exactement**
   l'ensemble suivant — relevé au sol le 2026-10-08 sur `crates/kesh-db/test-schema/0001_schema_squash.sql` :

   | table.colonne | trié par |
   |---|---|
   | `journal_entry_lines.entry_id` | enfant (CASCADE) — réécrit par la modification |
   | `journal_entries.reverses_entry_id` | `IsAReversal` / `AlreadyReversed` |
   | `invoices.journal_entry_id` | `OwnedByInvoice` |
   | `credit_notes.journal_entry_id` | `OwnedByCreditNote` |
   | `supplier_invoices.purchase_journal_entry_id` | `OwnedBySupplierInvoice` |
   | `supplier_invoices.settlement_journal_entry_id` | `OwnedBySupplierInvoice` |
   | `invoice_settlements.journal_entry_id` | `OwnedBySettlement` (règlements **et** soldes `write_off`) |
   | `bank_transactions.matched_entry_id` | `MatchedBankTransaction` |

2. les **colonnes** de `journal_entry_lines` doivent égaler `{id, entry_id, account_id, line_order, debit, credit,
   project_id}` — car la modification **supprime et réinsère** les lignes : toute colonne neuve y serait perdue en
   silence.

Le message d'échec dit quoi faire : « trier la référence dans `reversal_blockers` (gel) ou la déclarer ici avec sa
justification ».

⚠️ **Il rougira à coup sûr avec la 15-1a (lettrage)**, qui ajoute `journal_entry_lines.lettering_id`
(`15-1a-socle-lettrage.md:67`). C'est voulu : une ligne lettrée ne doit pas être réécrite, et la seconde des deux
stories à merger devra trancher (vraisemblablement un motif `LETTERED` qui gèle l'écriture). **L'orchestrateur doit
le signaler à la spécification de la 15-1a.**

### D4 — `PUT /api/v1/journal-entries/{id}` : le contrat

**Corps** : celui de `POST` plus la version —
`{ entryDate, journal, description, lines: [{ accountId, debit, credit, projectId? }], version }`.
**Réponse** : `200` + `JournalEntryResponse` (forme du `POST`).

**Le handler** (`routes/journal_entries.rs`, `update_journal_entry` `:596`, réécrit) :

1. extracteur `Json<UpdateJournalEntryRequest>` rétabli — ⚠️ le doc-comment actuel (`:585-595`) justifiait son
   absence par le refus inconditionnel ; il tombe ;
2. ⛔ **DRY** : la préparation de `create_journal_entry` (`:469-545` — trim et longueur du libellé, borne de lignes,
   parse des montants, `accounting::validate`, construction du `NewJournalEntry`) est **extraite** dans une fonction
   commune aux deux handlers, jamais recopiée ;
3. **pas** de pré-contrôle `find_covering_date` (contrairement au `POST`) : l'exercice est celui **de l'écriture**,
   connu seulement sous le verrou (C-15-8-3) ;
4. appel de `journal_entries::update`, erreurs propagées par `AppError::from`.

⚠️ Les refus de **forme** (400 du corps, de `accounting::validate`) précèdent le 404, comme au `POST` : ils ne
dépendent d'aucune donnée de la base, ils ne révèlent donc pas l'existence d'une ressource d'une autre société.

**Le repository** — `journal_entries::update(pool, company_id, id, version, user_id, updated: NewJournalEntry)`,
**rétablie** depuis `git show d2910022:crates/kesh-db/src/repositories/journal_entries.rs` (`:870-1208`, avec
`is_no_op_change`), puis amendée. Ordre, **verrous d'abord, refus dans l'ordre de précédence** :

| étape | geste | refus |
|---|---|---|
| 0 | projets par ligne validés **hors** projets déjà présents sur l'écriture (*grandfathering* de l'ancien `update`, conservé — C-15-8-4) — **avant** tout verrou, ordre global `companies → projects → fiscal_years` | 404 / 409 projet |
| 0-bis | lecture **non verrouillante** de `books_locked_through` (patron de l'étape 0-bis de `create_in_tx_inner`, `:250-275`) | — |
| 1 | `SELECT je.version, je.entry_date, fy.status, fy.start_date, fy.end_date … FOR UPDATE` scopé `company_id` | `NotFound` → 404 |
| 2 | exercice clos | `FiscalYearClosed` → **400** |
| 3 | `reversed_by` (`:1397`) | `EntryIsReversed` → 409 `ENTRY_IS_REVERSED` |
| 4 | garde D2 (`reversal_blockers` filtré, **dans la transaction**) | `EntryNotModifiable` → 409 `<code du motif>` |
| 5 | `version` | `OptimisticLockConflict` → 409 `OPTIMISTIC_LOCK_CONFLICT` |
| 6 | nouvelle date ∈ `[fy.start_date, fy.end_date]` | `DateOutsideFiscalYear` → 400 |
| 7 | `validate_lines_accounts_in_tx(…, enforce_postable = true)` — ⛔ **sans** l'exemption D-A1 de l'ancien `update` (C-15-8-4) ; le paramètre `exempt_ids` disparaît (Dev Notes) | `InactiveOrInvalidAccounts` → 400 |
| 8 | verrou de période : **ancienne** date, puis **nouvelle** date, seuil **inclusif** | `PeriodLocked { attempted }` → 400 |
| 9 | instantané « avant » ; **court-circuit no-op** (`is_no_op_change`) : rollback, rend l'état, ni audit ni `version` | — |
| 10 | `DELETE` lignes, `UPDATE` en-tête (`entry_date`, `journal`, `description`, `version = version + 1`, `updated_at`), `INSERT` lignes (`line_order` séquentiel) | — |
| 11 | recontrôle `SUM(debit) = SUM(credit)` | `Invariant` → 500 |
| 12 | instantané « après » ; audit `journal_entry.updated`, `details = { before, after }` (`entry_snapshot_json`, `:906`) ; commit | — |

⛔ **Ni `entry_number`, ni `fiscal_year_id`, ni `id`, ni `created_at`, ni `reverses_entry_id` ne sont jamais
écrits.** Le numéro ne change pas (C-15-8-3) ; ⚠️ la date peut, dans l'exercice : l'ordre des numéros peut alors
cesser de suivre l'ordre des dates — c'est déjà le cas du complément de 25-7, et le manuel le dit (§ Numérotation).

⚠️ **Pourquoi le no-op vient APRÈS toutes les gardes** (héritage KF-004) : un `PUT` identique sur une écriture gelée
doit rendre le refus, pas un 200 trompeur.

⚠️ **La précédence est figée et testée** (AC 7) : un exercice clos parle avant tout 409 ; une écriture contre-passée
répond `ENTRY_IS_REVERSED`, jamais un code de pièce ; le verrou de version vient **après** le gel (le motif réel prime
sur « rechargez ») ; le verrou de période parle **en dernier** des refus de date, comme à la création (`:306-326`).

### D5 — `DELETE /api/v1/journal-entries/{id}` : le même cadre

`delete_in_tx` (`:1005`) garde ses deux appelants. Le paramètre **`enforce_immutability` est renommé
`enforce_ownership`** et change de sens :

| appelant | valeur | effet |
|---|---|---|
| `delete_by_id` (la route) | `true` | étape 3-ter : garde D2 → `EntryNotModifiable` |
| `invoices::unvalidate` (`invoices.rs:1650`) | `false` | la facture supprime **sa** propre écriture — la garde `OwnedByInvoice` n'a pas de sens ici |

Précédence (inchangée hormis 3-ter) : `FiscalYearClosed` (400) → `EntryIsReversed` (409) → **garde D2** (409, si
`enforce_ownership`) → `PeriodLocked` (400) → instantané, audit `journal_entry.deleted`, `DELETE` (CASCADE).

⛔ **La contre-passation elle-même (`IS_A_REVERSAL`) est refusée par la garde D2** : la clé étrangère `RESTRICT` ne
protège que l'**origine** — rien ne s'opposerait à l'effacement d'une contre-passation. C'est le seul cas où la garde
apporte un refus que la base n'apporterait pas déjà ; le test l'exerce (AC 10).

⚠️ **Le numéro n'est jamais réattribué** (compteur de la 25-2-c) : la suppression creuse un **trou**, expliqué par
l'instantané `journal_entry.deleted`. Le manuel (§ Numérotation) doit le dire — il affirme aujourd'hui qu'« une
écriture ne se supprime plus ».

### D6 — L'écriture d'ouverture et le mode « Compléter » (25-7)

Vérifié au sol : **rien ne marque l'écriture d'ouverture.** C'est une OD datée du premier jour du premier exercice
(`opening_balances.rs`, `create_opening_entry` `journal_entries.rs:562`), qu'aucune table ne référence. Elle tombe donc
sous D1 comme toute écriture manuelle — **aucun traitement particulier n'est à écrire**, et c'est le résultat voulu.

Conséquences sur 25-7, toutes **sans code neuf**, à tester (AC 13) et à dire au manuel :

| geste | effet sur l'écran « Soldes de départ » |
|---|---|
| **modifier** l'ouverture (débit/crédit inversés) | aucun : la société garde ses écritures, l'écran reste en mode « compléter ». Le contrôle « aucune écriture » de la génération (`count_by_company`, `:495`) ne regarde que le **nombre** d'écritures. |
| modifier l'ouverture en **retirant** la ligne d'un compte | ce compte redevient « jamais mouvementé » → **complétable** (`opening_complement.rs`, même règle qu'après une dévalidation, manuel `:681`) |
| **supprimer** l'ouverture, seule écriture de la société | la société redevient vierge → **la génération est de nouveau proposée** (statut `READY`) — exactement la procédure que la 24-4b avait fermée (24-4b D7) |
| supprimer l'ouverture quand d'autres écritures existent | génération toujours refusée ; ses comptes sans autre mouvement deviennent complétables, et le complément est daté du premier jour du premier exercice s'il est ouvert et hors période verrouillée (25-7, arbitrage 2) |

⚠️ **Limite assumée, à dire au manuel** : faute de marqueur, rien n'empêche d'ajouter à l'écriture d'ouverture une
ligne de **résultat** (la génération le refuse, `D4` de 14-4) — elle est devenue une écriture manuelle comme une autre.

**Concurrence avec le complément** : la modification insère des lignes, donc prend le verrou **partagé** de clé
étrangère sur chaque compte visé — elle sérialise avec le complément, qui tient ces comptes en **exclusif** (25-7,
étape 2). Elle verrouille l'écriture **puis** son exercice, comme `delete_in_tx` et la contre-passation : elle
appartient aux classes de cycles **déjà nommées** par la 25-7 (« par les exercices, avec la contre-passation et
`delete_in_tx` »). Rien de neuf à tenir ; **aucun `retry_with`** n'est exigé (ni `create` ni `delete` n'en ont).

### D7 — Concurrence : le verrou optimiste `version` existant

`journal_entries.version` (`INT NOT NULL DEFAULT 1`) existe, n'est bougée par **aucun** autre chemin (vérifié :
`grep -rn "UPDATE journal_entries" crates` ne rend que le `NULL` de `delete_all_by_company`, tests seuls) et figure
déjà dans `JournalEntryResponse`. Le `PUT` l'exige ; périmée → `409 OPTIMISTIC_LOCK_CONFLICT`. La `DELETE` ne
l'exige pas (comme avant la 24-4b, et comme la contre-passation).

⚠️ **Corollaire utile** : `version > 1` ⇔ l'écriture a été modifiée (par Kesh ≥ 0.13, ou avant le gel). C'est ce que
la fiche affiche (D8).

`docs/optimistic-locking-patterns.md` : la ligne 8 (`:49`) et « Variant A protégée » (`:75`) redeviennent vraies —
rétablir `journal_entries.rs` / `update` (variant B : `SELECT … FOR UPDATE`, pas de race), en le disant.

### D8 — L'écran : depuis la fiche, et l'historique au journal d'audit

Choix C-15-8-6 et C-15-8-7. **La liste ne change pas** (son lien vers la fiche, 24-4b D4, reste le seul geste de
ligne).

**Détail (`GET /journal-entries/{id}`)** — `JournalEntryDetailResponse` (`routes/journal_entries.rs:129`) gagne trois
champs, sur le patron de `reversable` / `reversalBlockedBy` / `reversalBlockedLabel` :

- `modifiable: bool` ;
- `modificationBlockedBy: string | null` — un **code** : `FISCAL_YEAR_CLOSED`, `ALREADY_REVERSED`, `IS_A_REVERSAL`,
  `OWNED_BY_INVOICE`, `OWNED_BY_CREDIT_NOTE`, `OWNED_BY_SUPPLIER_INVOICE`, `OWNED_BY_SETTLEMENT`,
  `MATCHED_BANK_TRANSACTION`, `PERIOD_LOCKED` — dans **cet** ordre de précédence (celui de D4) ;
- `modificationBlockedLabel: string | null` — numéro de pièce, ou la borne du verrou pour `PERIOD_LOCKED`.

Calcul : une fonction `journal_entries::modification_blocker(executor, company_id, id)` au repository, **réutilisée**
par la garde d'écriture pour la partie D2 (⛔ une seule logique, pas deux qui divergent). Elle ne voit que l'état
**présent** : la nouvelle date d'un `PUT` reste contrôlée à l'écriture seule.

**Fiche (`frontend/src/routes/(app)/journal-entries/[id]/+page.svelte`)** :

- boutons **« Modifier »** (`data-testid="edit-entry"`) et **« Supprimer »** (`data-testid="delete-entry"`) à côté de
  « Contre-passer » — ⛔ **absents, pas grisés**, quand `modifiable` est faux (convention de la 24-4a), le motif traduit
  affiché à leur place (`data-testid="modification-blocked-reason"`) ; ⚠️ si le motif de contre-passation et celui de
  modification sont **le même code**, ne l'afficher **qu'une fois** ;
- motifs traduits par un `switch` **exhaustif** typé (`_exhaustif: never`, et chaîne vide au `default` — cf. la leçon
  écrite dans `blockedLabel`, `[id]/+page.svelte`) ; type `ModificationBlocker` dans `journal-entries.types.ts` ;
  ⛔ **réutiliser** les clés `journal-entries-reverse-blocked-*` existantes pour les sept codes communs, n'ajouter que
  `FISCAL_YEAR_CLOSED` et `PERIOD_LOCKED` ;
- **« Modifier »** remplace la vue par `JournalEntryForm` en **mode édition**, pré-rempli ; enregistrer → toast, la
  fiche se recharge (même `id`, même numéro) ; annuler → retour à la vue ;
- **« Supprimer »** ouvre une confirmation (dialogue `role="dialog"`, patron de la confirmation de contre-passation de
  la même page) qui dit que le numéro **ne sera pas réattribué** et que la suppression est **inscrite au journal
  d'audit** ; confirmé → toast, retour à `/journal-entries` ;
- **historique** : mention « Modifiée » quand `version > 1` (`data-testid="entry-modified"`), et lien « Historique »
  (`data-testid="entry-history-link"`) vers `/audit-log?entityType=journal_entry&entityId={id}` — l'écran du journal
  d'audit lit déjà ces filtres depuis l'URL (`audit-log/+page.svelte:120-125`). ⚠️ Le journal d'audit est **refusé
  au rôle Consultation** (403, `lib.rs:707-723`) : masquer le lien pour ce rôle (`authState`).

**Formulaire (`JournalEntryForm.svelte`)** — le mode édition est **rétabli** depuis `d2910022`
(`initialEntry`, `version`, branche `updateJournalEntry`, `case 'OPTIMISTIC_LOCK_CONFLICT'`), avec :

- `form-helpers.ts` : `amountToFieldValue`, `lineResponseToDraft`, `fromJournalEntryResponse` rétablies, **et leur
  fichier de tests** (`form-helpers.test.ts`, 7 tests, retiré par la 24-4b) ;
- `journal-entries.api.ts` : `updateJournalEntry(id, req)` et `deleteJournalEntry(id)` rétablies ; le commentaire de
  fin de fichier (`:56-62`) tombe ;
- conflit de version : un toast qui dit que l'écriture a changé entre-temps, et la fiche se recharge (pas de modale :
  la fiche se recharge d'elle-même) ;
- ⚠️ une ligne pré-remplie sur un compte **archivé ou non imputable** n'est pas sélectionnable dans
  `AccountAutocomplete` : l'afficher avec son numéro et son nom (la fiche charge `fetchAccounts(true)`) et un
  avertissement « à remplacer » (`data-testid="line-account-unusable"`) — le refus faisant autorité reste le 400 du
  serveur ;
- les props manquantes de la fiche (`booksLockedThrough`, `recoverableAccountId`, comptes **actifs** pour la
  sélection) se chargent au clic sur « Modifier », comme la page de liste les charge (`+page.svelte:95-161`).

### D9 — Rapports, export complet, `.keshbackup` : aucune incohérence introduite — vérifié

- **Rapports** : calculés à la volée depuis les lignes. Modifier une écriture d'une période **non verrouillée**
  change un rapport déjà tiré : **c'est le sens même de la CR**, et le verrou de période (24-4c) est l'outil qui fige
  une période déclarée. Le manuel doit le dire en toutes lettres (§ verrou et § modification).
- **Export ZIP de souveraineté** : `journal_entries.csv` porte `version` et `updated_at` (`csv_tables.rs:301-346`) ;
  `audit_log.csv` porte les instantanés avant/après (`exports/global.rs:61`, `:151`). Une modification donne donc
  l'état présent **et** sa trace. Aucun changement de format.
- **`.keshbackup`** : `audit_log` y est (`backup.rs:64`) ; aucune migration, aucune colonne neuve → inventaire de tables
  inchangé, compatibilité des sauvegardes intacte. **P1–P8 muets** (aucun fichier de migration). ⛔ P8 reste un piège
  de grep : `20260729000001_invoice_lines_revenue_account_backfill.sql:54` cite `journal_entries::update` — publiée
  depuis v0.9.0, **ne pas la toucher** (24-4b, R2-2).

### D10 — Les clés API

`PUT` et `DELETE` restent dans `comptable_routes` (`lib.rs:366-368`), donc ouverts aux clés `read-write` (C-15-8-8).
À écrire dans `docs/api-external.md` (§ 7 et une sous-section « Modifier / supprimer une écriture » : corps, réponse,
**tableau des refus dans l'ordre de précédence de D4**, `ENTRY_IS_POSTED` retiré) et dans le manuel administrateur
(liste « ce qu'une clé `read-write` peut faire », `admin-manual.tex:1796`).

## Critères d'acceptation

1. **Modifier une écriture manuelle.** `PUT` d'une écriture manuelle d'un exercice ouvert, hors période verrouillée,
   avec la bonne `version` → **200** ; date, journal, libellé et lignes sont ceux du corps ; `id`, `entry_number`,
   `fiscal_year_id`, `created_at` **inchangés** ; `version` + 1 ; débit = crédit.
2. **Le cas déclencheur.** L'écriture **d'ouverture** générée par `POST /opening-balances` se modifie par `PUT` en
   **inversant débit et crédit** de deux lignes → 200, mêmes garanties que l'AC 1. ⛔ C'est le test qui dit si la
   story a atteint son but.
3. **Audit avant/après.** Toute modification effective écrit **une** entrée `journal_entry.updated` (entité
   `journal_entry`, id de l'écriture, utilisateur ou clé) dont `details.before` égale l'état antérieur et
   `details.after` l'état final, **lignes comprises** (compte, débit, crédit, ordre, projet). Un `PUT` identique à
   l'état présent → 200, **aucune** entrée, `version` inchangée.
4. **Mêmes contrôles qu'à la saisie** : déséquilibre → 400 `ENTRY_UNBALANCED` ; compte archivé, non imputable ou d'une
   autre société — **y compris sur une ligne inchangée** → 400 `INACTIVE_OR_INVALID_ACCOUNTS` ; nouvelle date hors de
   l'exercice **de l'écriture**, même si un autre exercice ouvert la couvre → 400 `DATE_OUTSIDE_FISCAL_YEAR` ; et rien
   n'a changé en base.
5. **Période verrouillée** : ancienne date ≤ borne → 400 `PERIOD_LOCKED` ; ancienne date libre mais **nouvelle** date
   ≤ borne → 400 `PERIOD_LOCKED`. Seuil inclusif testé (borne = date).
6. **Écritures gelées** : pour **chacune** des pièces — facture, avoir, facture fournisseur (achat **et** règlement),
   règlement client, **solde** (`write_off`), transaction bancaire rapprochée — `PUT` et `DELETE` rendent **409 sous le
   code du motif**, avec `details.documentId` ; une écriture contre-passée → **409 `ENTRY_IS_REVERSED`** ; une
   contre-passation → **409 `IS_A_REVERSAL`**. ⚠️ Réutiliser le montage de `every_document_owned_entry_is_refused`
   (`journal_entry_reversal_e2e.rs:485`), ne pas en réinventer un.
7. **Précédence** — chaque paire testée en montant **les deux** causes : exercice clos **et** contre-passée → 400 ;
   contre-passée **et** version périmée → `ENTRY_IS_REVERSED` ; pièce **et** version périmée → code de la pièce ;
   pièce **et** période verrouillée → code de la pièce ; version périmée **et** déséquilibre… ⚠️ le déséquilibre est un
   400 de **forme**, rendu par le handler avant la base : ce cas se teste comme « 400 avant 404 » (AC 12).
8. **Concurrence** : `PUT` avec une `version` périmée → 409 `OPTIMISTIC_LOCK_CONFLICT`, rien n'a changé.
9. **Supprimer une écriture manuelle** → **204** ; l'écriture et ses lignes ont disparu ; une entrée
   `journal_entry.deleted` porte l'instantané complet ; la **création suivante** du même exercice ne reprend **pas** le
   numéro (compteur 25-2-c).
10. **Supprimer une contre-passation** → 409 `IS_A_REVERSAL` (D5 : la base ne l'aurait pas refusé).
11. **Dévalidation inchangée** : `invoices::unvalidate` (`enforce_ownership = false`) supprime toujours l'écriture de
    sa facture — ses tests existants restent verts, sans réécriture.
12. **IDOR et rôles** : `PUT`/`DELETE` sur un `id` d'une autre société ou inexistant → **404**, jamais 409 ;
    Consultation → **403** avant tout autre contrôle ; un corps malformé → 400/422 de l'extracteur, sans lecture de la
    base.
13. **Soldes de départ** (D6) : après **modification** de l'ouverture, `GET /opening-balances/status` rend toujours
    `ALREADY_HAS_ENTRIES` ; après modification qui **retire** la ligne d'un compte, ce compte figure dans
    `completableAccounts` ; après **suppression** de l'ouverture seule écriture de la société, le statut redevient
    `READY` et une nouvelle génération réussit.
14. **Détail** : `GET /journal-entries/{id}` rend `modifiable` / `modificationBlockedBy` / `modificationBlockedLabel`,
    cohérents avec la route d'écriture pour **chaque** motif de l'AC 6, plus `FISCAL_YEAR_CLOSED` et `PERIOD_LOCKED` ;
    un compte archivé ne rend **pas** l'écriture non modifiable.
15. **Garde-fou d'inventaire** (D3) : le test existe, passe, et ⛔ **sa mutation est tuée** — ajouter en test une
    colonne ou une clé étrangère factice vers `journal_entries` le fait rougir (à exécuter une fois, déclarer au Dev
    Agent Record).
16. **Écran** : depuis la fiche, « Modifier » ouvre le formulaire pré-rempli ; enregistrer met à jour la fiche (même
    numéro) ; « Supprimer » demande confirmation puis ramène à la liste ; les deux boutons sont **absents** avec le motif
    quand l'écriture n'est pas modifiable ; « Modifiée » et « Historique » apparaissent après une modification ; le lien
    ouvre le journal d'audit filtré sur l'écriture. Couvert par **Playwright** (le seul test qui voit la valeur traverser
    la frontière HTTP).
17. **Retraits** : `DbError::EntryIsPosted`, le code `ENTRY_IS_POSTED`, la clé `journal-entries-blocked-posted` (quatre
    catalogues) n'existent plus ; ⛔ `grep -rn "ENTRY_IS_POSTED\|EntryIsPosted\|enforce_immutability" crates frontend docs`
    ne rend plus que les story files historiques et le `CHANGELOG` (qui le dit retiré) — **à greper sur le dépôt
    entier et trier à la main** (leçon de la 24-4b).
18. **Audit lisible** : `journal_entry.updated` revient dans `ACTIONS` (`audit_labels.rs:130-132`) avec sa clé
    `audit-log-action-journal-entry-updated` dans les **quatre** locales (la garde `tests/audit_label_registry.rs`
    l'exige) ; `audit_route_registry.rs:82` passe le `PUT` à `Traced`.
19. **Documentation** : manuel utilisateur et administrateur, `docs/api-external.md`,
    `docs/optimistic-locking-patterns.md`, `CHANGELOG.md` et `README.md` disent le nouveau cadre (cf. Dev Notes) ; PDF
    régénérés et **contrôlés aplatis** (`pdftotext f.pdf - | tr '\n' ' ' | tr -s ' '`).

## Invariants testables

- **I1 — L'identité ne bouge pas.** Pour tout `PUT` réussi : `id`, `entry_number`, `fiscal_year_id`, `created_at`,
  `reverses_entry_id` identiques avant et après.
- **I2 — Rien ne change sans trace.** Le nombre d'entrées `journal_entry.updated` d'une écriture égale `version − 1`
  pour une écriture créée par Kesh ≥ 0.13 (no-op exclus des deux côtés).
- **I3 — Aucune écriture d'une pièce ne bouge.** Après tout `PUT`/`DELETE` refusé, lignes et en-tête identiques
  (comptes, montants, ordre, projets, `version`).
- **I4 — La correction reste possible.** Une écriture modifiable reste contre-passable (si aucun compte archivé) : les
  deux voies coexistent, la modification ne retire pas la contre-passation.

## Tâches

- [ ] **T1 — Erreurs** (AC 6, 17)
  - [ ] `DbError::EntryNotModifiable { blocker, document_id, document_label }` + `code()` ; retrait d'`EntryIsPosted`
  - [ ] `kesh-api/src/errors.rs` : mappage partagé avec `EntryNotReversable` (fonction extraite) ; retrait de la branche
        `EntryIsPosted` (`:3042`) **et** du commentaire de la 24-4b resté au-dessus du verrou de période (`:2986-2992`)
  - [ ] message d'`ENTRY_IS_REVERSED` élargi (« … ne peut plus être modifiée ni supprimée »), clé
        `journal-entries-delete-blocked-reversed` **conservée** (consommée par `errors.rs:2982`), texte revu dans les
        quatre locales
- [ ] **T2 — `update`** (AC 1–8) : rétablie depuis `d2910022`, amendée selon D4 ; `modification_blocker` (D8) partagé
- [ ] **T3 — `delete_in_tx`** (AC 9–11) : `enforce_immutability` → `enforce_ownership`, étape 3-ter = garde D2 ;
      doc-comments de `delete_by_id` (`:924-949`) et `delete_in_tx` (`:967-1004`) **réécrits** (⛔ la 24-4b a payé un
      MEDIUM pour un doc-comment resté à l'octet près) ; commentaires d'`invoices.rs:1442` et `:1650` ; en-tête de module
      (`:1-40`, « corriger par contre-passation plutôt que par suppression ») revu
- [ ] **T4 — Route et détail** (AC 1, 12, 14) : handler `PUT` (D4, préparation extraite du `POST`) ; trois champs du
      détail ; doc-comment du `DELETE` (`:609-610`, « asymétrie volontaire avec UPDATE ») revu
- [ ] **T5 — Garde-fou d'inventaire** (AC 15, D3) + inscription dans `test_schema_guard.rs` si monté sur `MIGRATOR`
- [ ] **T6 — Audit** (AC 18) : `ACTIONS`, clé i18n ×4, `audit_route_registry.rs:82`
- [ ] **T7 — Écran** (AC 16) : fiche, formulaire en mode édition, `form-helpers` + tests, api, types ; clés i18n ×4 ;
      `i18n-keys.test.ts` (`ATTENDU.sitesTotal`) mis à jour **avec sa ventilation** dans le doc-comment ;
      `e2e-selecteurs-traduits.test.ts` si un sélecteur traduit est ajouté
- [ ] **T8 — Tests qui changent de sens** (liste ci-dessous) : réécrits, **pas** supprimés en bloc ; chaque test retiré
      nommé au Dev Agent Record avec son remplaçant
- [ ] **T9 — Documentation** (AC 19) — cf. Dev Notes ; `make fr` dans `docs/manual/`, PDF commités
- [ ] **T10 — Gates** (⛔ complets — exception `kesh-db` : ciblage interdit dès qu'un repository est touché)
  - [ ] base remise à zéro (KF-039), `scripts/test-fast.sh` sous `mem-guard`
  - [ ] `npm run check` · `lint-i18n-ownership` · `test:unit` · `build`
  - [ ] suite Playwright **complète au dernier commit de code**, jugée fichier par fichier contre `docs/testing.md`
        § « Les échecs attendus » (D7 de la rétro Epic 25 : « rejouée au push » interdit)

## Hors périmètre

- **Statut brouillon** : non (24-4b D1 tient ; la CR rend la modification possible sans état intermédiaire).
- **Écran de versions** avec différentiel : non (C-15-8-6) ; le journal d'audit porte l'information.
- **Marqueur d'écriture d'ouverture** : non (D6, limite assumée).
- **Modifier une écriture de pièce** : non, par construction (#532 point 2) — elle se corrige par le chemin de sa pièce.
- **Déplacer une écriture d'exercice** : non (C-15-8-3).

## Dev Notes

### Règle de découpage — examinée, non déclenchée (C-15-8-9)

Cinq zones (`kesh-db`, `kesh-api`, `kesh-i18n`, `frontend`, `docs`) : le seuil est « **plus de** cinq ». Un seul
mécanisme, aucune migration, aucune règle métier neuve hors du cadre de #532. **Ligne de découpe pré-déclarée** si la
validation montre une non-convergence **par recyclage** (`CLAUDE.md`, amendement D5 de la rétro Epic 25) : la
suppression (D5, AC 9–11, T3, bouton et E2E de suppression) part en **15-8b** ; la modification de l'écriture manuelle
et d'ouverture reste en **15-8a**, et passe en premier.

### Les sites qui supposent aujourd'hui l'immuabilité — inventaire au sol (2026-10-08)

⛔ **Recontrôler par grep sur le dépôt ENTIER avant de déclarer la liste close** — la 24-4b a raté quatre fois ce geste
(périmètre `frontend/` seul, arrêt au premier résultat, `crates/` seul, report incomplet). Commandes :

```sh
grep -rn "ENTRY_IS_POSTED\|EntryIsPosted\|enforce_immutability\|blocked-posted" . --exclude-dir={node_modules,target,.git}
grep -rnE "ne se (modifie|supprime|réécri)|gel(ée|é)?\b|imposée|Story 24-4b" crates frontend/src frontend/tests docs/*.md
grep -nE "écritur" docs/manual/fr/*.tex | grep -E "modifi|supprim|réécri|éditab|imposé|gel"
```

**Code (`crates/`)**

| site | ce qu'il dit / fait | à faire |
|---|---|---|
| `kesh-db/src/errors.rs:626-638`, `:647`, `:768` | variante `EntryIsPosted`, son doc, sa mention dans le doc de `PeriodLocked` | retirer ; doc de `PeriodLocked` reformulé |
| `kesh-db/src/repositories/journal_entries.rs:1-40` (module), `:65-82` (doc de `validate_lines_accounts_in_tx`, cite `update` et D-A1), `:292` (« symétrique à `update` (:671) »), `:924-1132` (`delete_by_id`, `delete_in_tx`) | gel, renvois à une fonction absente | réécrire ; ⛔ vérifié au sol : `exempt_ids` n'a qu'**un** appelant, qui passe `&[]` (`:340`), et `update` passera `&[]` aussi (C-15-8-4) — **retirer le paramètre et sa branche SQL** plutôt que laisser une exemption morte qui paraît vivante ; doc-comment repris |
| `journal_entries.rs` `mod tests` : `le_gel_parle_avant_le_verrou_de_periode` (~`:2509`) et le helper qui passe `enforce_immutability` (~`:2396-2444`), commentaire `:2120` | test du gel | réécrire : avec `enforce_ownership = true`, une écriture manuelle de période verrouillée rend `PeriodLocked` |
| `kesh-db/src/repositories/invoices.rs:1442`, `:1650` | commentaires `enforce_immutability = false` | renommer, reformuler |
| `kesh-api/src/errors.rs:2986-2992`, `:3042-3050` | commentaire et branche du gel | retirer |
| `kesh-api/src/routes/journal_entries.rs:585-606`, `:609-623` | handler `PUT` refusant ; commentaire du `DELETE` | réécrire |
| `kesh-api/src/audit_labels.rs:130-132`, `:192` | `journal_entry.updated` absent d'`ACTIONS`, « qu'aucun site n'écrit plus » | rétablir, reformuler |
| `kesh-api/tests/audit_route_registry.rs:82` | `NoMatter("… 409 ENTRY_IS_POSTED")` | `Traced` |
| `kesh-api/tests/admin_full_import_e2e.rs:1386-1388` | « ne se réécrit plus du tout (24-4b) » | reformuler : le repointage reste nécessaire (une écriture **de facture** ne se modifie pas) |

**Tests de la 24-4b qui changent de sens** — `crates/kesh-api/tests/journal_entry_reversal_e2e.rs` :

| test | aujourd'hui | devient |
|---|---|---|
| `putting_a_posted_entry_is_refused_and_changes_nothing` (`:1148`) | PUT → 409 | AC 1 + AC 3 (modification réussie, audit) |
| `putting_with_an_empty_or_broken_body_is_refused_the_same_way` (`:1209`) | corps ignoré → 409 | AC 12 (corps malformé → 400/422) |
| `deleting_a_posted_entry_is_refused` (`:1231`) | DELETE → 409 | AC 9 (204, audit, numéro non réattribué) |
| `put_and_delete_never_leak_the_existence_of_a_foreign_entry` (`:1254`) | 404 | **reste** — mais le `PUT` doit porter un corps **valide** |
| `a_reversed_entry_answers_reversed_not_posted` (`:1317`) | `ENTRY_IS_REVERSED` ≠ `ENTRY_IS_POSTED` | reste, sur `PUT` **et** `DELETE`, sans mention de `ENTRY_IS_POSTED` |
| `a_closed_fiscal_year_answers_before_both_conflicts` (`:1341`) | 400 avant 409 | reste (AC 7) |
| `consultation_can_neither_rewrite_nor_delete` (`:1365`) | 403 | reste (AC 12) |
| `the_opening_entry_is_frozen_but_still_correctable` (`:1386`) | ouverture gelée | ⛔ **inversé** : AC 2 + AC 13 |
| `an_entry_of_a_closed_year_stays_correctable` (`:1445`) | contre-passable en exercice clos | reste ; ajouter `PUT`/`DELETE` → 400 |

`deleting_a_reversed_entry_is_refused_but_bulk_delete_still_works` (`:727`) et `every_document_owned_entry_is_refused`
(`:485`) **restent** ; le second sert de montage à l'AC 6.

**Frontend**

| site | à faire |
|---|---|
| `src/lib/features/journal-entries/JournalEntryForm.svelte:55-59` (« CRÉATION SEULE ») | mode édition rétabli (D8) |
| `src/lib/features/journal-entries/form-helpers.ts:1-13` | fonctions rétablies, doc revu ; `form-helpers.test.ts` rétabli |
| `src/lib/features/journal-entries/journal-entries.api.ts:56-62` | `updateJournalEntry`, `deleteJournalEntry` |
| `src/lib/features/journal-entries/journal-entries.types.ts` | `UpdateJournalEntryRequest`, champs du détail, `ModificationBlocker` |
| `src/routes/(app)/journal-entries/+page.svelte:34` (commentaire « plus de mode 'edit' ») | reformuler : l'édition vit sur la fiche |
| `src/routes/(app)/journal-entries/[id]/+page.svelte` | D8 |
| `src/routes/(app)/settings/opening-balances/+page.svelte` (clés ci-dessous) | textes |
| `src/lib/shared/i18n-keys.test.ts:165-182` | `sitesTotal` + ventilation |
| `tests/e2e/journal-entries.spec.ts:274-…` (bloc « le gel (Story 24-4b) ») | les deux tests de liste **restent vrais** (la liste n'a toujours ni ✎ ni 🗑) — renommer le bloc, reformuler le JSDoc ; ajouter le parcours AC 16 |
| `tests/e2e/fiscal-years.spec.ts:108-118` (JSDoc) | `update` existe de nouveau et lève `FISCAL_YEAR_CLOSED` (400) — ses neuf tests ne passent toujours que par la création |

**i18n** (`crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl`) — en **quatre** locales :

| clé | à faire |
|---|---|
| `journal-entries-blocked-posted` (fr `:356`) | **retirer** (seul consommateur : `errors.rs:3046`, retiré) |
| `journal-entries-delete-blocked-reversed` (fr `:355`) | texte : « modifiée ni supprimée » |
| `opening-balances-locked-already-has-entries` (fr `:929`) | ajouter la modification de l'écriture d'ouverture comme chemin de correction |
| `opening-balances-complete-confirm` (fr `:962`) | « Elle ne se modifie plus ensuite » devient faux |
| `error-opening-complement-account-moved` (fr `:979`) | ajouter « en modifiant l'écriture » |
| `audit-log-action-journal-entry-updated` | **ajouter** |
| clés neuves de l'écran (D8) | reprendre, quand le sens est le même, les noms retirés par la 24-4b (`journal-entry-edit`, `journal-entry-delete`, `journal-entry-delete-confirm-{title,message,cancel,delete}`, `journal-entry-deleted`) — lisibles dans `git show d2910022:crates/kesh-i18n/locales/fr-CH/messages.ftl` ; ⚠️ le message de confirmation doit être **réécrit** (numéro non réattribué, trace d'audit) |

⚠️ **Une clé lue par le backend ne se retire pas sans greper `crates/`** : `t(key, default)` rend **la clé brute** si
le bundle est chargé et la clé absente (`kesh-i18n/src/loader.rs`, `format_unknown_key_returns_key`) — le piège S1-C1
de la 24-4b. `i18n-keys.test.ts` ne scanne que le frontend.

**Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) — ⛔ aucun gate ne le lit ; passages relevés :

| ligne | ce qu'il dit | à faire |
|---|---|---|
| `:472-476` § « Cycle de vie » | « immédiatement comptabilisée » | reste vrai ; ajouter qu'elle reste **modifiable** jusqu'à la clôture (cadre D1) |
| `:478-494` § « Une écriture enregistrée ne se modifie plus » | gel inconditionnel | ⛔ **section réécrite et renommée** (ex. « Modifier ou supprimer une écriture ») : le cadre D1, la trace d'audit avant/après, le lien « Historique », la date qui reste dans l'exercice, la suppression qui laisse un trou ; ⚠️ le *keshtip* « Pourquoi il n'y a pas de brouillon » reste vrai, mais « ce que vous validez part directement aux livres » appelle « … et reste corrigeable tant que l'exercice est ouvert » |
| `:506-510` § verrou de période | « aucune écriture déjà passée dans la période ne pourra plus en disparaître » | ajouter « ni être modifiée » ; et dire que **verrouiller est le geste qui fige un trimestre déclaré** (D9) |
| `:532-570` § contre-passation, *keshnote* `:554` | « La contre-passation est **imposée** : la modification et la suppression d'une écriture sont refusées » | ⛔ faux — la contre-passation devient **le** chemin pour une écriture d'exercice clos, de période verrouillée ou de pièce ; « en usage courant, une seule voie fait encore disparaître une écriture » devient faux |
| `:572-585` § Numérotation | « Une écriture ne se supprime plus » ; « si la dernière écriture disparaît — par la dévalidation de sa facture » | la suppression d'une écriture manuelle creuse aussi un trou ; le numéro n'est jamais repris |
| `:631` § clôture | « La modification et la suppression, elles, sont refusées en permanence — clôture ou non » | ⛔ inversé : la clôture est **ce qui** les ferme |
| `:642` § réouverture | « L'exercice rouvert redevient modifiable » | redevient **vrai** pour les écritures manuelles (cadre D1) — vérifier la formulation |
| `:667` *keshnote* soldes de départ | « L'écriture d'ouverture, comme toutes les autres, ne se modifie ni ne se supprime ensuite » | ⛔ faux — c'est le cas déclencheur : dire comment corriger une inversion (fiche → Modifier), et que la supprimer quand elle est seule rouvre la génération (D6) |
| `:681` *keshnote* compléter | « On le corrige … en contre-passant l'écriture fautive, ou par une écriture de correction » | ajouter **la modification** en premier chemin, et la ligne retirée qui redevient complétable |
| `:2115-2140` FAQ « revenir en arrière » | « Écriture : ni modification ni suppression, jamais » | ⛔ faux ; la puce « exercice clôturé » reste vraie pour la contre-passation |

Et `docs/manual/fr/admin-manual.tex` : `:1796` (clés `read-write`), `:1835-1838` (« n'est plus ni modifiable ni
supprimable… L'immutabilité n'est donc plus seulement tracée, elle est imposée » → faux ; la conformité 958f repose
désormais sur la **trace avant/après**, la clôture et le verrou de période), `:2004` (Olico art. 3 : « les corrections
passent par contre-passation » → par contre-passation **ou** par modification tracée). Brochure : rien (`:393` parle des
avoirs). DE/EN/IT : `README.md` seuls, rien à propager.

**Autres documents**

- `docs/api-external.md` : § 7 tableau (`:212`, ajouter `PUT`/`DELETE /journal-entries/{id}`), nouvelle sous-section
  avec le tableau des refus (D4/D5) ; vérifier les autres mentions d'`ENTRY_IS_REVERSED` (`:232`).
- `docs/optimistic-locking-patterns.md:49`, `:75` (D7).
- `CHANGELOG.md` : ⛔ **aucune section `## [0.13.0] — Non publié` n'existe** (en tête : `## [0.12.1] — 2026-10-07`) —
  la **créer en tête**, avec `### Modifié` : *Une écriture se modifie et se supprime tant que son exercice est ouvert
  (#532)* — cadre, trace avant/après, ce qui reste gelé, l'API (`PUT` rétabli, corps + `version`, codes de refus ;
  `ENTRY_IS_POSTED` **retiré** — à dire, c'est un changement de contrat pour une intégration).
  ⚠️ Si une autre story de l'epic l'a créée entre-temps, **y ajouter** — ne pas dupliquer l'en-tête.
- `README.md:29` (« écritures validées ») : ajouter « modifiables tant que l'exercice est ouvert ».

**À NE PAS toucher** : `crates/kesh-db/migrations/20260729000001_invoice_lines_revenue_account_backfill.sql:54` (P8) ;
les story files historiques de `_bmad-output/` (`3-3`, `24-4a`, `24-4b`, …) ; `docs/known-failures.md` et
`docs/change_request.md` (archivés).

### Ce que la story réutilise — ne pas réinventer

| besoin | existant |
|---|---|
| corps de `update`, no-op, *grandfathering* projets | `git show d2910022:crates/kesh-db/src/repositories/journal_entries.rs:870-1208` |
| mode édition du formulaire, helpers, tests | `git show d2910022:frontend/src/lib/features/journal-entries/{JournalEntryForm.svelte,form-helpers.ts,form-helpers.test.ts}` |
| inventaire des propriétaires | `reversal_blockers` (`journal_entries.rs:1265`) |
| mappage d'erreur à motif | `EntryNotReversable` (`kesh-api/src/errors.rs:2698`) |
| instantané d'audit | `entry_snapshot_json` (`:906`) |
| verrou de période, seuil inclusif | `create_in_tx_inner` `:250-326`, `delete_in_tx` `:1059-1089` |
| validation des comptes | `validate_lines_accounts_in_tx` (`:84`) |
| montage de chaque pièce | `every_document_owned_entry_is_refused` (`journal_entry_reversal_e2e.rs:485`) |
| filtres d'URL du journal d'audit | `audit-log/+page.svelte:120-125` (`parseQueryFromUrl`) |

### Pièges

- ⛔ **La garde D2 se lit DANS la transaction**, sous le `FOR UPDATE` de l'écriture — `reversal_blockers` est générique
  sur `Executor` : lui passer `&mut **tx`.
- ⚠️ `reversal_blockers` rend `NotFound` sur une écriture absente : dans la transaction, elle vient **après** le
  `FOR UPDATE` qui a déjà établi l'existence — ne pas transformer ce cas en 500.
- ⚠️ **`AccountArchived` est le DERNIER motif de `reversal_blockers`** : le filtrer ne change pas le premier motif
  restant. Mais ne pas réordonner la liste — sa précédence est celle de la contre-passation (24-4a D6).
- ⚠️ **Le `INSERT` des lignes doit omettre toute colonne absente des schémas anciens** — plusieurs tests montent un
  schéma antérieur puis exercent le vrai chemin (commentaire `:358-366`). Les colonnes de `LINE_COLUMNS` existent
  toutes depuis 19-2 : sans objet aujourd'hui, à garder en tête si la 15-1a passe d'abord.
- ⚠️ **Base de gate piégée** (KF-039) : la remettre à zéro **avant** chaque gate complet, inconditionnellement.
- ⚠️ **Haiku et les diffs multi-commits** : en revue, fournir le diff aplati `main..HEAD`.

### Références

- Issue **#532** ; Story **24-4b** (gel) — D1, D2, D4, D5, D7 ; Story **24-4a** (contre-passation) — D3, D6 ;
  Story **24-4c** (verrou de période) ; Story **25-2-b-zero** (#443, verrou sur la suppression) ; Story **25-2-c**
  (#381, compteur) ; Story **25-7** (#445, soldes de départ, verrous du complément) ; Story **15-1a** (lettrage,
  `lettering_id`).
- `CLAUDE.md` §§ *Review Iteration Rule* (exception `kesh-db`), *Inventorier les sites NON RÉSOLUS*, *Propagation
  post-patch*, *Recompter ses propres comptes rendus*, *Le prompt d'une passe doit NOMMER le manuel*, *Migration
  breaking policy* (P8).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log

| date | ce qui s'est passé |
|---|---|
| 2026-10-08 | **Spec** (`bmad-create-story`, Opus 5.5, autonomie Epic 15). Choix C-15-8-1 à C-15-8-9. Aucune migration. Découpage examiné, non déclenché (cinq zones), ligne de découpe pré-déclarée. |
