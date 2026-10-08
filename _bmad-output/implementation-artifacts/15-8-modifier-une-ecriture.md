# Story 15.8 : Modifier une écriture tant que son exercice est ouvert

Status: ready-for-dev

**Issue : [#532]** — CR du Project Lead, 2026-10-08, constat de recette de la v0.12.1. ⚠️ **URGENTE.**
**Révise la Story 24-4b** (`24-4b-gel-ecriture.md`, gel inconditionnel, #380).
Choix autonomes : **C-15-8-1 à C-15-8-16** (`epic-15-choix-autonomes.md`) — C-15-8-10 à 16 viennent de la validation P1.

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
| **La correction doit être apparente** (art. 958f CO, Olico art. 3) | chaque modification écrit `journal_entry.updated` avec l'instantané **avant et après, lignes comprises**, l'utilisateur **ou la clé d'API** (`NewAuditLogEntry::for_actor`, D4/D5 — ⚠️ aujourd'hui `delete_in_tx` écrit en `::user` : une suppression par clé serait attribuée au créateur de la clé) et l'horodatage ; chaque suppression écrit `journal_entry.deleted` avec l'instantané (`delete_in_tx` étape 5) |
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
une requête à sous-requêtes corrélées qui rend **huit** motifs : **sept motifs de propriété ou d'état**, lus sur les
**sept** colonnes de clé étrangère vers `journal_entries` autres que `journal_entry_lines.entry_id` (D3 ; la huitième
colonne est l'enfant, non gelant), **plus** le compte archivé. La garde de modification le réutilise, filtré :

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
pour `EntryNotReversable` le **repli générique** `"ENTRY_NOT_REVERSABLE"` (commentaire `:757-758` : « le code EXPOSÉ
est celui du `ReversalBlocker` ») ; la nouvelle variante prend de même un repli `"ENTRY_NOT_MODIFIABLE"`, et le code
**exposé** vient du mappage `kesh-api`, qui rend `blocker.code()`.

**La garde d'écriture, nommée** — `journal_entries::modification_guard(executor, company_id, id) ->
Result<Option<ReversalBlockerHit>, DbError>` : le **premier** motif de `reversal_blockers` **hors `AccountArchived`**
(sept motifs possibles : `IsAReversal`, `AlreadyReversed`, les quatre `OwnedBy*`, `MatchedBankTransaction`). Elle
ne lève rien : la conversion en erreur est une seconde fonction, `modification_refusal(hit: ReversalBlockerHit) ->
DbError` (`AlreadyReversed` → `EntryIsReversed`, tout autre → `EntryNotModifiable { … }`), partagée par `update` et
`delete_in_tx`. Le motif d'**écran** est une autre fonction, qui l'appelle (D8).

**Sérialisation — le verrou d'abord, et aucune lecture ordinaire avant lui.** Le dépôt tourne en `REPEATABLE READ`
(`pool.rs:17`) : la vue de lecture InnoDB naît à la **première lecture ordinaire** de la transaction, et toutes les
lectures ordinaires suivantes — `reversal_blockers`, `reversed_by`, la borne de verrou, les comptes — lisent **cette
vue**, pas l'état courant. Une lecture ordinaire posée **avant** le `FOR UPDATE` de l'écriture fige donc une vue
antérieure à l'attente du verrou : une contre-passation (ou une pièce) commitée pendant cette attente reste
**invisible** à la garde, et le `PUT` réécrit une écriture déjà contre-passée — exactement le motif « contre-passation
faussée » que la 24-4b fermait. *(Défaut de la première rédaction de cette fiche, relevé en validation P1 : elle
plaçait la lecture des projets et de la borne avant le verrou.)*

D'où la règle, celle de `opening_complement.rs:23-27` (« tous les verrous d'abord […] les lectures ordinaires ensuite
— l'instantané REPEATABLE READ s'ouvre là, après les verrous ») et celle que suivent déjà `delete_in_tx` (`:1005`) et
`reverse_in_tx_inner` (`:1518`) : **le premier acte de la transaction est le `SELECT … FOR UPDATE` de l'écriture**.
Ensuite, tout ce qui peut changer la garde passe par la ligne verrouillée — une contre-passation verrouille l'origine
d'abord (`reverse_in_tx_inner`), une pièce qui s'attache prend le verrou **partagé** de clé étrangère sur l'écriture —
et attend donc notre commit : une vue ouverte **après** le verrou ne peut plus manquer aucun des deux. *(À citer dans
le doc-comment de `update` et de `delete_in_tx` ; ne jamais lire la garde sur le pool hors transaction dans le chemin
d'écriture ; testé à deux connexions, AC 8.)*

### D3 — Le garde-fou d'inventaire : un ensemble clos, pas une énumération

⛔ **Une nouvelle table qui référence `journal_entries` passerait sous la garde sans que rien ne rougisse** — c'est
exactement la classe d'un test muet. Règle du `CLAUDE.md` § *Inventorier les sites NON RÉSOLUS* : on inventorie
l'ensemble clos, on n'énumère pas les formes connues.

Test d'intégration `#[sqlx::test(migrations = "./test-schema")]` — **monté sur le squash**, et c'est suffisant :
`crates/kesh-db/tests/test_schema_guard.rs` prouve déjà, sur `information_schema` complet, que le squash égale le
schéma de `MIGRATOR`. Le monter sur `MIGRATOR` obligerait à l'inscrire dans `ALLOWED_REAL_MIGRATOR_FILES`
(`test_schema_guard.rs:38`) et à rejouer 75 migrations pour un gain nul. Fichier : `crates/kesh-db/tests/
journal_entries_modification.rs` (neuf, le même que le test de concurrence de l'AC 8). Toutes les requêtes portent
`TABLE_SCHEMA = DATABASE()` (les bases éphémères voisines vivent sur le même serveur).

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
3. ⚠️ **Une colonne sans clé étrangère échappe au point 1** : `information_schema.COLUMNS` dont le nom correspond à
   `%journal_entry%` ou `%entry_id%`, **hors** `journal_entries` elle-même et hors les colonnes du point 1, doit égaler
   une liste déclarée avec sa justification. Relevé au sol le 2026-10-08 (`awk` sur le squash) : **une seule**,
   `company_invoice_settings.journal_entry_description_template` — un gabarit de libellé, pas une référence ;
   `audit_log.entity_id` n'y tombe pas (nom sans motif, référence logique non gelante). Sans ce contrôle, l'ensemble
   ne serait « clos » que pour les clés étrangères.

Le message d'échec dit quoi faire : « trier la référence dans `reversal_blockers` (gel) ou la déclarer ici avec sa
justification ».

⚠️ **Il rougira à coup sûr avec la 15-1a (lettrage)**, qui ajoute `journal_entry_lines.lettering_id`
(`15-1a-socle-lettrage.md:67`). C'est voulu : une ligne lettrée ne doit pas être réécrite, et la seconde des deux
stories à merger devra trancher (vraisemblablement un motif `LETTERED` qui gèle l'écriture). **L'orchestrateur doit
le signaler à la spécification de la 15-1a.**

### D4 — `PUT /api/v1/journal-entries/{id}` : le contrat

**Corps** — `UpdateJournalEntryRequest` (`routes/journal_entries.rs`, **neuf** : la struct a disparu avec la 24-4b,
`grep -rn UpdateJournalEntryRequest crates frontend/src` ne rend rien ; seul `CreateJournalEntryRequest` existe,
`:80`) :

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateJournalEntryRequest {
    pub entry_date: NaiveDate,
    pub journal: CoreJournal,
    pub description: String,
    pub lines: Vec<CreateJournalEntryLineRequest>, // la ligne du POST, réutilisée telle quelle
    pub version: i32,
}
```

soit en JSON `{ entryDate, journal, description, lines: [{ accountId, debit, credit, projectId? }], version }`.
**Réponse** : `200` + `JournalEntryResponse` (forme du `POST`). Côté TypeScript, `UpdateJournalEntryRequest =
CreateJournalEntryRequest & { version: number }` (`journal-entries.types.ts`).

**Le handler** (`routes/journal_entries.rs`, `update_journal_entry` `:596`, réécrit) :

1. extracteur `Json<UpdateJournalEntryRequest>` rétabli — ⚠️ le doc-comment actuel (`:585-595`) justifiait son
   absence par le refus inconditionnel ; il tombe ;
2. ⛔ **DRY** : la préparation de `create_journal_entry` (`:476-545` — trim et longueur du libellé, borne de lignes,
   parse des montants, `accounting::validate`, construction du `NewJournalEntry`) est **extraite** dans une fonction
   commune aux deux handlers, jamais recopiée :

   ```rust
   /// Refus de FORME uniquement : ne lit pas la base.
   fn prepare_new_journal_entry(
       company_id: i64,
       entry_date: NaiveDate,
       journal: CoreJournal,
       description: &str,
       lines: &[CreateJournalEntryLineRequest],
   ) -> Result<NewJournalEntry, AppError>
   ```

   `company_id` est celui de `get_company_for(&current_user, …)` (la société de l'appelant, jamais une donnée du
   corps). ⚠️ **Effet de bord sur le `POST`, assumé** (C-15-8-16) : son pré-contrôle `find_covering_date` (`:510-525`) se place
   **après** `prepare_new_journal_entry`, si bien qu'un corps à la fois déséquilibré **et** sans exercice rend
   désormais `ENTRY_UNBALANCED` avant `NO_FISCAL_YEAR` (aujourd'hui l'inverse). Greper les tests du `POST` qui
   cumulent les deux causes (`grep -rn "NO_FISCAL_YEAR\|FISCAL_YEAR_CLOSED" crates/kesh-api/tests`) et dire au Dev
   Agent Record qu'aucun ne les cumule, ou lequel change ;
3. **pas** de pré-contrôle `find_covering_date` pour le `PUT` : l'exercice est celui **de l'écriture**, connu
   seulement sous le verrou (C-15-8-3) ;
4. appel de `journal_entries::update(…, current_user.user_id, current_user.api_key_id, …)`, erreurs propagées par
   `AppError::from`.

⚠️ Les refus de **forme** (400 du corps, de `accounting::validate`) précèdent le 404, comme au `POST` : ils ne
dépendent d'aucune donnée de la base, ils ne révèlent donc pas l'existence d'une ressource d'une autre société.

**Le repository** — `journal_entries::update(pool, company_id, id, version, user_id, actor_api_key_id: Option<i64>,
updated: NewJournalEntry) -> Result<JournalEntryWithLines, DbError>`. ⛔ **Écrite contre le code ACTUEL**, avec
l'ancien corps pour **modèle** (`git show d2910022:crates/kesh-db/src/repositories/journal_entries.rs`, `:870-1208`,
dont `is_no_op_change` `:891`) — **jamais** recopiée de `d2910022` : neuf commits ont touché ce fichier depuis le gel
(`git log --oneline 08e20353..HEAD -- crates/kesh-db/src/repositories/journal_entries.rs`), et l'ancien corps appelle
`validate_lines_accounts_in_tx` avec l'exemption D-A1 que cette story retire. Audit : `NewAuditLogEntry::for_actor
(user_id, actor_api_key_id, "journal_entry.updated", …)` (`entities/audit_log.rs`, patron d'`invoices.rs:2521`) —
**jamais** `::user`.

Ordre — ⛔ **le verrou de l'écriture d'abord, aucune lecture ordinaire avant lui** (D2, Sérialisation) ; puis les
autres verrous ; puis les refus dans l'ordre de précédence :

| étape | geste | refus |
|---|---|---|
| 1 | **premier acte de la transaction** : `SELECT je.version, je.entry_date, je.fiscal_year_id FROM journal_entries je WHERE je.id = ? AND je.company_id = ? FOR UPDATE` — l'écriture **seule**, sans la jointure `fiscal_years` (cf. ordre des verrous ci-dessous) | `NotFound` → 404 |
| 1-bis | projets : lecture des projets déjà présents sur l'écriture (*grandfathering* de l'ancien `update`, conservé — C-15-8-4 ; c'est la **première lecture ordinaire**, elle ouvre la vue **après** le verrou), puis `projects::validate_taggable_in_tx` sur les seuls projets **nouveaux** (sentinelle `companies` puis `FOR UPDATE` des projets) | — ⚠️ le verrou se prend ici, mais le **refus** (404 / 409 projet) est **gardé et rendu à l'étape 6**, avec les comptes : rendu ici, il parlerait avant l'exercice clos et le gel, contre l'AC 7. Une erreur de base (`DbError` hors `NotFound`/`IllegalStateTransition`) se propage aussitôt |
| 1-ter | `SELECT status, start_date, end_date FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE` | — |
| 1-quater | lecture **non verrouillante** de `books_locked_through`, **après** les verrous (patron de `delete_in_tx`, `:1059-1068`) | — |
| 2 | exercice clos | `FiscalYearClosed` → **400** |
| 3 | `modification_guard` (D2), **dans la transaction** (`&mut **tx`) → `modification_refusal` | `AlreadyReversed` → `EntryIsReversed` (409 `ENTRY_IS_REVERSED`) ; autres → `EntryNotModifiable` (409 `<code du motif>`) |
| 4 | `version` | `OptimisticLockConflict` → 409 `OPTIMISTIC_LOCK_CONFLICT` |
| 5 | nouvelle date ∈ `[fy.start_date, fy.end_date]` | `DateOutsideFiscalYear` → 400 |
| 6 | refus projet gardé à l'étape 1-bis, s'il y en a un ; puis `validate_lines_accounts_in_tx(…, enforce_postable = true)` — ⛔ **sans** l'exemption D-A1 de l'ancien `update` (C-15-8-4) ; le paramètre `exempt_ids` disparaît (Dev Notes) | projet → 404 / 409 ; `InactiveOrInvalidAccounts` → 400 |
| 7 | verrou de période : **ancienne** date, puis **nouvelle** date, seuil **inclusif** | `PeriodLocked { attempted }` → 400 |
| 8 | instantané « avant » ; **court-circuit no-op** (`is_no_op_change`) : rollback, rend l'état, ni audit ni `version` | — |
| 9 | `DELETE` lignes, `UPDATE` en-tête (`entry_date`, `journal`, `description`, `version = version + 1`, `updated_at`), `INSERT` lignes (`line_order` séquentiel) | — |
| 10 | recontrôle `SUM(debit) = SUM(credit)` | `Invariant` → 500 |
| 11 | instantané « après » ; audit `journal_entry.updated` par `for_actor`, `details = { before, after }` (`entry_snapshot_json`, `:906`) ; commit | — |

⚠️ **L'ordre des verrous** (C-15-8-10) : **écriture → [sentinelle `companies` → projets] → exercice**. L'écriture se
verrouille **seule** à l'étape 1 parce que la verrouiller avec son exercice (jointure, comme `delete_in_tx`) puis
prendre les projets inverserait l'ordre de la création (`companies → projects → fiscal_years`, `create_in_tx_inner`
`:236-247`) : deux saisies taguées sur le même exercice se bloqueraient en croix. Écriture puis exercice est l'ordre
de `delete_in_tx` et de la contre-passation (classe de cycles déjà nommée par la 25-7, D6). Écriture puis sentinelle
ou projets : aucun chemin ne prend la sentinelle ou un projet **puis** le verrou d'une écriture **existante** —
vérifié au sol (`grep -rn "acquire_company_sentinel_lock" crates/kesh-db/src crates/kesh-api/src` : projets, TVA,
relances, comptes bancaires, `create_opening_entry` `:581`, `validate_taggable_in_tx` ; aucun ne verrouille ensuite
une écriture existante). À **revérifier** par le développeur et à écrire dans le doc-comment ; un cycle résiduel
serait détecté par InnoDB (erreur 1213), jamais une attente infinie.

⛔ **Ni `entry_number`, ni `fiscal_year_id`, ni `id`, ni `created_at`, ni `reverses_entry_id` ne sont jamais
écrits.** Le numéro ne change pas (C-15-8-3) ; ⚠️ la date peut, dans l'exercice : l'ordre des numéros peut alors
cesser de suivre l'ordre des dates — c'est déjà le cas du complément de 25-7, et le manuel le dit (§ Numérotation).

⚠️ **Pourquoi le no-op vient APRÈS toutes les gardes** (héritage KF-004) : un `PUT` identique sur une écriture gelée
doit rendre le refus, pas un 200 trompeur. **Corollaire tranché** (C-15-8-11) : un `PUT` identique sur une écriture
dont un compte a été archivé ou rendu non imputable rend **400 `INACTIVE_OR_INVALID_ACCOUNTS`**, pas 200 — l'AC 3
l'exclut de son « `PUT` identique → 200 », l'AC 4 le teste (`update_no_op_with_inactive_account_returns_inactive_error`,
rétabli).

⚠️ **La précédence est figée et testée** (AC 7) : un exercice clos parle avant tout 409 ; une écriture contre-passée
répond `ENTRY_IS_REVERSED`, jamais un code de pièce ; le verrou de version vient **après** le gel (le motif réel prime
sur « rechargez ») ; le verrou de période parle **en dernier** des refus de date, comme à la création (`:306-326`).

### D5 — `DELETE /api/v1/journal-entries/{id}` : le même cadre

`delete_in_tx` (`:1005`) garde ses deux appelants. Le paramètre **`enforce_immutability` est renommé
`enforce_ownership`** et change de sens ; un paramètre **`actor_api_key_id: Option<i64>`** s'ajoute, et l'audit
`journal_entry.deleted` passe de `NewAuditLogEntry::user` (`:1114`) à `NewAuditLogEntry::for_actor` — sans quoi
une suppression par clé d'API serait attribuée au créateur de la clé (C-15-8-8 : « la trace porte la clé »).
Signatures : `delete_by_id(pool, company_id, id, user_id, actor_api_key_id)` et `delete_in_tx(tx, company_id, id,
user_id, actor_api_key_id, enforce_ownership)` ; la route passe `current_user.api_key_id`.

⚠️ `delete_in_tx` respecte **déjà** la règle de sérialisation de D2 sur la route : `delete_by_id` ouvre la
transaction et le premier acte de `delete_in_tx` est son `FOR UPDATE` (étape 2) ; la borne de verrou se lit après
(`:1059-1068`). Le doc-comment le dit désormais ; le test de concurrence de l'AC 8 a son pendant `DELETE`.

| appelant | valeur | effet |
|---|---|---|
| `delete_by_id` (la route) | `true` | étape 3-ter : `modification_guard` → `modification_refusal` (D2) |
| `invoices::unvalidate` (`invoices.rs:1654`) | `false`, `actor_api_key_id = None` | la facture supprime **sa** propre écriture — la garde `OwnedByInvoice` n'a pas de sens ici. ⚠️ `unvalidate` ne reçoit aujourd'hui aucun `actor_api_key_id` (`invoices.rs:1486-1492`) : **hors périmètre**, signalé à l'orchestrateur (même défaut sur `create` et `reverse`, `routes/journal_entries.rs:460`, `:566`) |

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
| **supprimer** l'ouverture, seule écriture de la société | la société redevient vierge → **la génération est de nouveau proposée** (statut `READY`) — exactement la procédure que la 24-4b avait fermée (24-4b D7). ⚠️ La nouvelle ouverture reçoit le **numéro 2**, pas 1 : le compteur de la 25-2-c ne réattribue jamais un numéro (`un_numero_libere_n_est_jamais_reattribue`, `journal_entries.rs:2087`). **Accepté** (C-15-8-12) : le trou est expliqué par l'instantané `journal_entry.deleted` ; le manuel (§ soldes de départ) le dit, l'AC 13 l'asserte |
| supprimer l'ouverture quand d'autres écritures existent | génération toujours refusée ; ses comptes sans autre mouvement deviennent complétables, et le complément est daté du premier jour du premier exercice s'il est ouvert et hors période verrouillée (25-7, arbitrage 2) |

Le **complément** de 25-7 (`opening_complement.rs`) est lui aussi une OD qu'aucune table ne référence : modifiable et
supprimable sous D1, testé (AC 13).

⚠️ **Limite assumée, à dire au manuel** : faute de marqueur, rien n'empêche d'ajouter à l'écriture d'ouverture une
ligne de **résultat** (la génération le refuse, `D4` de 14-4) — elle est devenue une écriture manuelle comme une autre.

**Concurrence avec le complément** : la modification insère des lignes, donc prend le verrou **partagé** de clé
étrangère sur chaque compte visé — elle sérialise avec le complément, qui tient ces comptes en **exclusif** (25-7,
étape 2). Elle verrouille l'écriture, puis (si de nouveaux projets sont tagués) la sentinelle et les projets, **puis**
son exercice (D4, C-15-8-10) — l'écriture avant l'exercice, comme `delete_in_tx` et la contre-passation : elle
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
- `modificationBlockedBy: string | null` — un **code d'écran**, l'une de ces neuf valeurs, dans **cet** ordre de
  précédence : `FISCAL_YEAR_CLOSED`, `IS_A_REVERSAL`, `ALREADY_REVERSED`, `OWNED_BY_INVOICE`, `OWNED_BY_CREDIT_NOTE`,
  `OWNED_BY_SUPPLIER_INVOICE`, `OWNED_BY_SETTLEMENT`, `MATCHED_BANK_TRANSACTION`, `PERIOD_LOCKED` — c'est l'ordre de
  D4 restreint aux refus qui ne dépendent pas du corps (étapes 2, 3 et 7, ancienne date), et, à l'étape 3, l'ordre
  de `reversal_blockers` ;
- `modificationBlockedLabel: string | null` — numéro de pièce (`document_label`), ou la borne du verrou
  (`AAAA-MM-JJ`) pour `PERIOD_LOCKED`, `null` sinon.

**Deux fonctions, nommées, une seule logique** (C-15-8-13) :

| fonction | rôle | appelée par | rend |
|---|---|---|---|
| `modification_guard(executor, company_id, id)` | **garde d'écriture** — D2 seule : premier motif de `reversal_blockers` hors `AccountArchived` | `update` (étape 3), `delete_in_tx` (étape 3-ter), `modification_blocker` | `Result<Option<ReversalBlockerHit>, DbError>` |
| `modification_blocker(executor, company_id, id)` | **motif d'écran** — exercice clos, puis `modification_guard`, puis verrou de période sur la date **présente** | `get_journal_entry` (sur le pool, comme `reversal_blocker` `:435`) | `Result<Option<ModificationBlocker>, DbError>` |

`ModificationBlocker` (`crates/kesh-db/src/errors.rs`, à côté de `ReversalBlocker`) :

```rust
pub enum ModificationBlocker {
    FiscalYearClosed,
    Owned { blocker: ReversalBlocker, document_id: Option<i64>, document_label: Option<String> },
    PeriodLocked { locked_through: NaiveDate },
}
impl ModificationBlocker {
    pub fn code(&self) -> &'static str { /* FISCAL_YEAR_CLOSED | blocker.code() | PERIOD_LOCKED */ }
    pub fn label(&self) -> Option<String> { /* document_label | locked_through | None */ }
}
```

`Owned` ne porte jamais `AccountArchived` (filtré par `modification_guard`). Le motif d'écran ne voit que l'état
**présent** : la nouvelle date d'un `PUT`, le corps et la version restent contrôlés à l'écriture seule.

**Table de correspondance — code d'écran ↔ refus d'écriture** (assertion de l'AC 14, écrite **en dur** dans le
test, pas recalculée) :

| `modificationBlockedBy` | `PUT` et `DELETE` rendent |
|---|---|
| `FISCAL_YEAR_CLOSED` | 400 `FISCAL_YEAR_CLOSED` |
| `IS_A_REVERSAL` | 409 `IS_A_REVERSAL` |
| `ALREADY_REVERSED` | 409 **`ENTRY_IS_REVERSED`** — ⚠️ **le seul écart de nom**, voulu : l'écran réutilise le vocabulaire de `ReversalBlocker` (et sa clé `journal-entries-reverse-blocked-already-reversed`), l'écriture garde le code d'erreur existant de la 24-4a |
| `OWNED_BY_INVOICE`, `OWNED_BY_CREDIT_NOTE`, `OWNED_BY_SUPPLIER_INVOICE`, `OWNED_BY_SETTLEMENT`, `MATCHED_BANK_TRANSACTION` | 409 sous le **même** code |
| `PERIOD_LOCKED` | 400 `PERIOD_LOCKED` (ancienne date) |
| `null` (modifiable) | `PUT` identique avec la bonne version → 200, ou 400 `INACTIVE_OR_INVALID_ACCOUNTS` si un compte est archivé (C-15-8-11 — un compte archivé ne rend **pas** l'écriture non modifiable) |

**Fiche (`frontend/src/routes/(app)/journal-entries/[id]/+page.svelte`)** :

- boutons **« Modifier »** (`data-testid="edit-entry"`) et **« Supprimer »** (`data-testid="delete-entry"`) à côté de
  « Contre-passer » — ⛔ **absents, pas grisés**, quand `modifiable` est faux (convention de la 24-4a), le motif traduit
  affiché à leur place (`data-testid="modification-blocked-reason"`) ; ⚠️ si le motif de contre-passation et celui de
  modification sont **le même code**, ne l'afficher **qu'une fois** ;
- ⛔ **rôle Consultation** (C-15-8-14) : les trois boutons — « Modifier », « Supprimer » **et** « Contre-passer »
  (aujourd'hui affiché à ce rôle, `[id]/+page.svelte:236-243`, et qui rend 403 au clic) — sont **absents**, sans motif
  affiché ; le lien « Historique » aussi (le journal d'audit est refusé à Consultation, 403, `lib.rs:707-723`). Rôle lu
  dans `authState` (`$lib/app/stores/auth.svelte.ts`). Le 403 du serveur reste le refus qui fait autorité ;
- motifs traduits par un `switch` **exhaustif** typé (`_exhaustif: never`, et chaîne vide au `default` — cf. la leçon
  écrite dans `blockedLabel`, `[id]/+page.svelte`) ; type `ModificationBlocker` (union des neuf codes) dans
  `journal-entries.types.ts` ; ⛔ **réutiliser** les clés `journal-entries-reverse-blocked-*` existantes pour les sept
  codes communs, n'ajouter que `journal-entries-modify-blocked-fiscal-year-closed` et
  `journal-entries-modify-blocked-period-locked` (`{ $date }`) ;
- **« Modifier »** remplace la vue par `JournalEntryForm` en **mode édition**, pré-rempli ; enregistrer → toast
  `journal-entry-saved` (clé existante), la fiche se recharge (même `id`, même numéro) ; annuler → retour à la vue ;
- **« Supprimer »** ouvre une confirmation (dialogue `role="dialog"`, patron de la confirmation de contre-passation de
  la même page) qui dit que le numéro **ne sera pas réattribué** et que la suppression est **inscrite au journal
  d'audit** ; confirmé → toast `journal-entry-deleted`, retour à `/journal-entries` ;
- **historique** : mention « Modifiée » (`journal-entry-modified`) quand `version > 1` (`data-testid="entry-modified"`),
  et lien « Historique » (`journal-entry-history`, `data-testid="entry-history-link"`) vers
  `/audit-log?entityType=journal_entry&entityId={id}` — l'écran du journal d'audit lit déjà ces filtres depuis l'URL
  (`audit-log/+page.svelte:120-125`).

**Formulaire (`JournalEntryForm.svelte`)** — le mode édition est **rétabli par inversion du commit du gel**, pas par
copie de l'état `d2910022` (C-15-8-15) :

```sh
git show 08e20353 -- <fichier> | git apply -R --3way   # 08e20353 = feat(24-4b), le gel (squash de la PR #422)
```

| fichier | inversion | après |
|---|---|---|
| `src/lib/features/journal-entries/form-helpers.ts` | s'applique net (`git apply -R --check`, vérifié le 2026-10-08) | — |
| `src/lib/features/journal-entries/form-helpers.test.ts` (7 tests) | net — le fichier renaît | — |
| `src/lib/features/journal-entries/journal-entries.api.ts` | net — `updateJournalEntry`, `deleteJournalEntry` reviennent | le commentaire de fin de fichier tombe avec |
| `src/lib/features/journal-entries/journal-entries.types.ts` | net — `UpdateJournalEntryRequest` revient | y ajouter les trois champs du détail et `ModificationBlocker` |
| `src/lib/features/journal-entries/JournalEntryForm.svelte` | ⚠️ **conflit** à résoudre à la main (`--3way`) : la 24-4c (`54ae4a70`) a ajouté depuis la prop `booksLockedThrough` et le `min` de la date (`minEntryDate`) | **garder** les deux ajouts de la 24-4c ; reprendre `initialEntry`, `version`, la branche `updateJournalEntry` |
| `src/lib/shared/e2e-selecteurs-traduits.test.ts` | ⚠️ **pas d'inversion aveugle** : les 2 lignes retirées (`journal-entries.spec.ts :: Annuler`, `:: Supprimer`) inscrivaient les sélecteurs traduits des specs de **liste** | y inscrire les sélecteurs traduits que les **nouvelles** specs (T7-bis) emploient réellement, ni plus ni moins |
| `src/routes/(app)/journal-entries/+page.svelte` | ⛔ **NE PAS inverser** : le mode édition de `d2910022` vivait dans la **liste** (✎, 🗑, modale de conflit), que C-15-8-7 écarte | lire l'ancien état comme **modèle** du chargement des props et du dialogue de suppression |
| `tests/e2e/journal-entries.spec.ts`, `src/lib/shared/i18n-keys.test.ts` | ⛔ **NE PAS inverser** : parcours de liste ; `i18n-keys.test.ts` a bougé cinq fois depuis (25-4-d1 à 25-7) | réécrits (T7-bis, T7) |

Mode édition, **spécifié** (ce qui n'existait pas dans `d2910022`) :

- **dates** : `min` = le plus tardif de `fy.startDate` et du lendemain de `booksLockedThrough` ; `max` = `fy.endDate`,
  où `fy` est l'**exercice de l'écriture** (`entry.fiscalYearId`, cherché dans `listFiscalYears()`,
  `fiscal-years.api.ts:13`, chargé au clic sur « Modifier »). Prop neuve `entryFiscalYear?: { startDate: string;
  endDate: string } | null`, nulle en création (où `min` reste celui de la 24-4c). **Confort de saisie seulement** : le
  refus qui fait autorité reste le 400 du serveur (`DATE_OUTSIDE_FISCAL_YEAR`, `PERIOD_LOCKED`) ;
- **`FISCAL_YEAR_CLOSED` en édition** : ⛔ **ne pas** passer par `notifyMissingFiscalYearOrFallback`
  (`shared/utils/notify.ts:98-123`), dont le message — « L'exercice qui couvre cette date est clôturé. Vérifiez la date
  saisie » avec un bouton vers les paramètres — est faux ici : c'est l'exercice **de l'écriture** qui a été clôturé
  entre-temps. Toast `journal-entries-modify-blocked-fiscal-year-closed` (la clé du motif d'écran, réutilisée), puis la
  fiche se recharge (elle rend alors le motif à la place des boutons) ;
- **409 de course** (`ENTRY_IS_REVERSED`, `IS_A_REVERSAL`, `OWNED_BY_*`, `MATCHED_BANK_TRANSACTION`) : toast du
  message serveur (déjà traduit), puis la fiche se recharge — même geste que ci-dessus ; une branche `case` nommée par
  code, pas le `default` du `switch` (`JournalEntryForm.svelte:191`) ;
- **`OPTIMISTIC_LOCK_CONFLICT`** : toast `journal-entry-edit-conflict` (« Cette écriture a été modifiée entre-temps :
  la fiche a été rechargée. »), puis rechargement (pas de modale : la fiche se recharge d'elle-même) ;
- **`PERIOD_LOCKED`, `DATE_OUTSIDE_FISCAL_YEAR`, `ENTRY_UNBALANCED`, `INACTIVE_OR_INVALID_ACCOUNTS`** : le formulaire
  reste ouvert, toast du message serveur — l'utilisateur corrige sa saisie (les quatre premiers `case` existent déjà,
  `:176-179`) ;
- ⚠️ une ligne pré-remplie sur un compte **archivé ou non imputable** n'est pas sélectionnable dans
  `AccountAutocomplete` : l'afficher avec son numéro et son nom (la fiche charge `fetchAccounts(true)`) et un
  avertissement `journal-entry-line-account-unusable` (« Compte archivé ou non imputable — à remplacer »,
  `data-testid="line-account-unusable"`) — le refus faisant autorité reste le 400 du serveur ;
- les props de la fiche (`booksLockedThrough`, `recoverableAccountId`, comptes **actifs** pour la sélection, projets,
  `entryFiscalYear`) se chargent au clic sur « Modifier », comme la page de liste les charge (`+page.svelte:95-161`).

**Clés i18n neuves de l'écran** — quatre locales, nommées ici pour qu'aucune ne s'invente au développement :
`journal-entry-edit`, `journal-entry-delete`, `journal-entry-delete-confirm-{title,message,cancel,delete}`,
`journal-entry-deleted` (noms repris de `d2910022`, message de confirmation **réécrit**), `journal-entry-modified`,
`journal-entry-history`, `journal-entry-edit-conflict`, `journal-entry-line-account-unusable`,
`journal-entries-modify-blocked-fiscal-year-closed`, `journal-entries-modify-blocked-period-locked`. Les quatre clés
de conflit de `d2910022` (`journal-entry-conflict-{title,message,reload,reloaded}`) **ne reviennent pas** (plus de
modale).

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
La trace d'audit porte la clé (`for_actor`, D4 et D5 ; AC 3 et 9).
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
   `journal_entry`, id de l'écriture) dont `details.before` égale l'état antérieur et `details.after` l'état final,
   **lignes comprises** (compte, débit, crédit, ordre, projet). **Par clé d'API** : le même `PUT` porté par une clé
   `read-write` écrit `actor_type = 'ApiKey'` et `actor_api_key_id` = l'id de la clé (test dédié ; idem pour la
   suppression, AC 9). Un `PUT` identique à l'état présent, sur une écriture qui passe **toutes** les gardes → 200,
   **aucune** entrée, `version` inchangée — ⚠️ un `PUT` identique sur une écriture portant un compte archivé ou non
   imputable n'est **pas** un no-op : il rend le 400 de l'AC 4 (C-15-8-11).
4. **Mêmes contrôles qu'à la saisie** : déséquilibre → 400 `ENTRY_UNBALANCED` ; compte archivé, non imputable ou d'une
   autre société — **y compris sur une ligne inchangée** → 400 `INACTIVE_OR_INVALID_ACCOUNTS` ; nouvelle date hors de
   l'exercice **de l'écriture**, même si un autre exercice ouvert la couvre → 400 `DATE_OUTSIDE_FISCAL_YEAR` ; et rien
   n'a changé en base. ⚠️ Le *grandfathering* de postabilité de l'ancien `update` est **inversé**, pas rétabli :
   `test_update_grandfathers_non_postable_by_account` (`d2910022` `:3762`) devient
   `test_update_refuses_a_line_on_an_account_made_non_postable` — édition du seul libellé, puis ajout d'une ligne sur le
   même compte, chacun → 400. Les projets archivés déjà présents restent tolérés (C-15-8-4).
   Un `PUT` **identique** sur une écriture dont un compte a été archivé → 400 `INACTIVE_OR_INVALID_ACCOUNTS`
   (`update_no_op_with_inactive_account_returns_inactive_error`, rétabli).
5. **Période verrouillée et exercice clos** : ancienne date ≤ borne → 400 `PERIOD_LOCKED` ; ancienne date libre mais
   **nouvelle** date ≤ borne → 400 `PERIOD_LOCKED`. Seuil inclusif testé (borne = date). **Exercice clos seul**
   (aucune autre cause) : `PUT` et `DELETE` → 400 `FISCAL_YEAR_CLOSED`, chacun par un test qui ne monte que cette
   cause (`update_no_op_in_closed_fy_returns_fiscal_year_closed` rétabli pour le `PUT` au repository).
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
   **Sérialisation (D2)** — test à deux connexions, `crates/kesh-db/tests/journal_entries_modification.rs`,
   `update_waits_for_a_concurrent_reversal_then_refuses` : la connexion B ouvre une transaction et contre-passe
   l'écriture E par `reverse_in_tx` **sans commiter** ; la tâche A lance `update(E)` ; le test attend de voir A
   bloquée sur son verrou (`test_fixtures::attendre_une_requete_en_cours`, `test_fixtures.rs:560`, motifs tirés du
   texte du `SELECT` de l'étape 1, p. ex. `["je.version", "FOR UPDATE"]`) ; B commite ; A doit rendre
   `EntryIsReversed` et E est inchangée. ⛔ **Mutation à tuer** (à exécuter une fois, déclarée au Dev Agent Record) :
   placer une lecture ordinaire (la borne `books_locked_through`) **avant** le `FOR UPDATE` de l'étape 1 fait passer A
   — le test doit rougir. Pendant `DELETE` : `delete_waits_for_a_concurrent_reversal_then_refuses`, même montage.
9. **Supprimer une écriture manuelle** → **204** ; l'écriture et ses lignes ont disparu ; une entrée
   `journal_entry.deleted` porte l'instantané complet et l'acteur (utilisateur, ou clé d'API : `actor_type =
   'ApiKey'`, `actor_api_key_id`) ; la **création suivante** du même exercice ne reprend **pas** le numéro (compteur
   25-2-c).
10. **Supprimer une contre-passation** → 409 `IS_A_REVERSAL` (D5 : la base ne l'aurait pas refusé).
11. **Dévalidation inchangée** : `invoices::unvalidate` (`enforce_ownership = false`) supprime toujours l'écriture de
    sa facture — ses tests existants restent verts, sans réécriture.
12. **IDOR et rôles** : `PUT`/`DELETE` sur un `id` d'une autre société ou inexistant → **404**, jamais 409 ;
    Consultation → **403** avant tout autre contrôle ; un corps malformé → 400/422 de l'extracteur, sans lecture de la
    base.
13. **Soldes de départ** (D6) : après **modification** de l'ouverture, `GET /opening-balances/status` rend toujours
    `ALREADY_HAS_ENTRIES` ; après modification qui **retire** la ligne d'un compte, ce compte figure dans
    `completableAccounts` ; après **suppression** de l'ouverture seule écriture de la société, le statut redevient
    `READY` et une nouvelle génération réussit, sous le **numéro 2** (asserté — C-15-8-12). Une écriture de
    **complément** (25-7) se modifie et se supprime comme l'ouverture (200 / 204).
14. **Détail** : `GET /journal-entries/{id}` rend `modifiable` / `modificationBlockedBy` / `modificationBlockedLabel`.
    Pour **chacun** des neuf codes d'écran (D8), un montage qui ne porte que cette cause : le `GET` rend ce code, et
    `PUT` (corps identique, bonne version) **et** `DELETE` rendent le refus que la **table de correspondance** de D8
    associe à ce code — table écrite en dur dans le test (`ALREADY_REVERSED` ↔ `ENTRY_IS_REVERSED` est le seul écart
    de nom). Un compte archivé ne rend **pas** l'écriture non modifiable (`modifiable = true`, et le `PUT` identique
    rend le 400 de l'AC 4).
15. **Garde-fou d'inventaire** (D3) : le test existe, passe, et ⛔ **sa mutation est tuée** — ajouter en test une
    colonne ou une clé étrangère factice vers `journal_entries` le fait rougir (à exécuter une fois, déclarer au Dev
    Agent Record).
16. **Écran** : depuis la fiche, « Modifier » ouvre le formulaire pré-rempli ; enregistrer met à jour la fiche (même
    numéro) ; « Supprimer » demande confirmation puis ramène à la liste ; les deux boutons sont **absents** avec le motif
    quand l'écriture n'est pas modifiable ; « Modifiée » et « Historique » apparaissent après une modification ; le lien
    ouvre le journal d'audit filtré sur l'écriture ; au rôle **Consultation**, « Modifier », « Supprimer »,
    « Contre-passer » et « Historique » sont absents (C-15-8-14). Couvert par **Playwright** (le seul test qui voit la
    valeur traverser la frontière HTTP) — specs nommées en T7-bis.
17. **Retraits** : `DbError::EntryIsPosted`, le code `ENTRY_IS_POSTED`, la clé `journal-entries-blocked-posted` (quatre
    catalogues) n'existent plus ; ⛔ greper **le dépôt entier** —
    `grep -rn "ENTRY_IS_POSTED\|EntryIsPosted\|enforce_immutability\|blocked-posted" . --exclude-dir={node_modules,target,.git,.svelte-kit}`
    — et trier à la main (leçon de la 24-4b). Sortie attendue : **uniquement** des story files historiques sous
    `_bmad-output/implementation-artifacts/` (`24-4b-*`, `24-4a-*`, …, qui **ne se réécrivent pas** : ce sont des
    archives) et la section `## [0.13.0]` du `CHANGELOG.md` qui dit `ENTRY_IS_POSTED` retiré. Toute autre ligne est un
    résidu.
18. **Audit lisible** : `journal_entry.updated` revient dans `ACTIONS` (`audit_labels.rs:130-132`) avec sa clé
    `audit-log-action-journal-entry-updated` dans les **quatre** locales (la garde `tests/audit_label_registry.rs`
    l'exige) ; `audit_route_registry.rs:82` passe le `PUT` à `Traced`.
19. **Documentation** : manuel utilisateur et administrateur, `docs/api-external.md`,
    `docs/optimistic-locking-patterns.md`, `CHANGELOG.md` et `README.md` disent le nouveau cadre (cf. Dev Notes) ; PDF
    régénérés et **contrôlés aplatis** : pour chacun des deux manuels,
    `pdftotext f.pdf - | tr '\n' ' ' | tr -s ' ' | grep -ciE "ne se modifie plus|ni modifiable ni supprimable|depuis la version qui introduit le gel|modifications et suppressions antérieures au gel"`
    rend **0**, hors passages volontairement conservés et nommés au Dev Agent Record ; la table des matières porte le
    nouveau titre de la section renommée.
20. **Export** : après une modification, l'export ZIP de souveraineté porte l'état **présent** de l'écriture
    (`journal_entries.csv` : `version` incrémentée, `updated_at` à jour — `csv_tables.rs:316-342`) et sa trace
    (`audit_log.csv` contient l'entrée `journal_entry.updated` avec `before`/`after`). Test au niveau de l'export, sans
    changement de format (D9).

## Invariants testables

- **I1 — L'identité ne bouge pas.** Pour tout `PUT` réussi : `id`, `entry_number`, `fiscal_year_id`, `created_at`,
  `reverses_entry_id` identiques avant et après.
- **I2 — Rien ne change sans trace.** Le nombre d'entrées `journal_entry.updated` d'une écriture égale `version − 1`
  pour une écriture créée par Kesh ≥ 0.13 (no-op exclus des deux côtés), **hors restauration** : l'import d'un
  `.keshbackup` **fusionne** `audit_log` au lieu de le remplacer (`backup.rs:438-455`, Story 25-1a) — après la
  restauration d'une archive plus ancienne, le journal garde des mises à jour que la `version` restaurée a oubliées.
- **I3 — Aucune écriture d'une pièce ne bouge.** Après tout `PUT`/`DELETE` refusé, lignes et en-tête identiques
  (comptes, montants, ordre, projets, `version`).
- **I4 — La correction reste possible.** Une écriture modifiable reste contre-passable (si aucun compte archivé) : les
  deux voies coexistent, la modification ne retire pas la contre-passation.

## Tâches

- [ ] **T0 — Inventaire au sol** (AC 17, 19) : exécuter les trois `grep` des Dev Notes (« Les sites qui supposent
      aujourd'hui l'immuabilité ») **avant** toute écriture de code ; sortie triée (site → tableau de la fiche, ou
      site neuf) au Dev Agent Record. Un site absent des tableaux s'y ajoute, il ne s'ignore pas.
- [ ] **T1 — Erreurs** (AC 6, 17)
  - [ ] `DbError::EntryNotModifiable { blocker, document_id, document_label }` + repli `code()` `"ENTRY_NOT_MODIFIABLE"` ;
        retrait d'`EntryIsPosted` ; `ModificationBlocker` (`code()`, `label()`, D8)
  - [ ] `kesh-api/src/errors.rs` : mappage partagé avec `EntryNotReversable` (fonction extraite) ; retrait de la branche
        `EntryIsPosted` (`:3042`) **et** du commentaire de la 24-4b resté au-dessus du verrou de période (`:2986-2992`)
  - [ ] message d'`ENTRY_IS_REVERSED` élargi (« … ne peut plus être modifiée ni supprimée »), clé
        `journal-entries-delete-blocked-reversed` **conservée** (consommée par `errors.rs:2982`), texte revu dans les
        quatre locales
- [ ] **T2 — `update`** (AC 1–8) : écrite contre le code actuel, l'ancien corps pour modèle (D4) ; verrou de l'écriture
      **premier acte**, ordre des verrous écriture → [sentinelle → projets] → exercice ; `actor_api_key_id` +
      `for_actor` ; `modification_guard`, `modification_refusal`, `modification_blocker` (D2, D8) ; `exempt_ids`
      retiré de `validate_lines_accounts_in_tx`
  - [ ] tests unitaires de `mod tests` retirés par la 24-4b (`git show 08e20353 -- crates/kesh-db/src/repositories/
        journal_entries.rs`), traités **un par un** (tableau « Tests unitaires de l'ancien `update` », Dev Notes) ;
        helper `three_accounts` rétabli ; `mk_project`, `line`, `tagged_line` existent encore (`:3439-3469`) —
        **ne pas les dupliquer**
  - [ ] test à deux connexions (AC 8) et sa mutation tuée
- [ ] **T3 — `delete_in_tx`** (AC 9–11) : `enforce_immutability` → `enforce_ownership`, étape 3-ter = garde D2 ;
      `actor_api_key_id` sur `delete_by_id` et `delete_in_tx`, audit par `for_actor` ; `invoices::unvalidate` passe
      `None` ; doc-comments de `delete_by_id` (`:924-949`) et `delete_in_tx` (`:967-1004`) **réécrits** (⛔ la 24-4b a
      payé un MEDIUM pour un doc-comment resté à l'octet près) ; commentaires d'`invoices.rs:1442` et `:1650` ; en-tête de
      module (`:1-40`, « corriger par contre-passation plutôt que par suppression ») revu ; test de concurrence `DELETE`
      (AC 8)
- [ ] **T4 — Route et détail** (AC 1, 3, 12, 14, 20) : `UpdateJournalEntryRequest`, `prepare_new_journal_entry` extraite
      du `POST` (D4) ; handler `PUT` ; `api_key_id` passé au `PUT` et au `DELETE` ; trois champs du détail ;
      doc-comment du `DELETE` (`:609-610`, « asymétrie volontaire avec UPDATE ») revu ; tests de clé d'API (AC 3, 9),
      table de correspondance (AC 14), export (AC 20)
- [ ] **T5 — Garde-fou d'inventaire** (AC 15, D3) : `crates/kesh-db/tests/journal_entries_modification.rs`, monté sur
      le squash (pas d'inscription dans `test_schema_guard.rs`) ; trois contrôles (clés étrangères, colonnes de
      `journal_entry_lines`, colonnes à nom d'écriture sans clé étrangère) ; mutation tuée
- [ ] **T6 — Audit** (AC 18) : `ACTIONS`, clé i18n ×4, `audit_route_registry.rs:82`
- [ ] **T7 — Écran** (AC 14, 16) : fiche (boutons, motifs, rôle Consultation, « Modifiée », « Historique ») ;
      formulaire en mode édition **par inversion du gel** (D8, tableau des fichiers) ; `form-helpers.test.ts` renaît ;
      api, types ; clés i18n ×4 (liste fermée de D8) ; replis Svelte de `settings/opening-balances/+page.svelte:233-234`
      et `:387-388` ; `i18n-keys.test.ts` (`ATTENDU.sitesTotal`) mis à jour **avec sa ventilation** dans le
      doc-comment ; `e2e-selecteurs-traduits.test.ts` selon les sélecteurs réellement employés
  - [ ] **Vitest** : exhaustivité du `switch` des motifs de modification (un test par code, sur le patron de celui
        de `blockedLabel`) ; bornes `min`/`max` de date en édition (exercice de l'écriture, borne de verrou) ; branches
        d'erreur du mode édition (`FISCAL_YEAR_CLOSED` sans `notifyMissingFiscalYearOrFallback`, 409 de course,
        `OPTIMISTIC_LOCK_CONFLICT`) ; `fromJournalEntryResponse` sur une ligne à compte archivé
- [ ] **T7-bis — Playwright** (AC 16) — `tests/e2e/journal-entries.spec.ts`, bloc « le gel (Story 24-4b) » renommé ;
      les quatre specs de liste retirées par la 24-4b sont **remplacées** (pas rétablies : la liste n'offre plus ces
      gestes) par des parcours **depuis la fiche** :
  - [ ] « édition nominale — modification du libellé » → `modifier depuis la fiche : le libellé change, le numéro reste`
  - [ ] « suppression avec confirmation » → `supprimer depuis la fiche : confirmation, retour à la liste, l'écriture a disparu`
  - [ ] « annulation suppression » → `annuler la suppression : la fiche reste, l'écriture aussi`
  - [ ] « conflit 409 affiche la modale de reload » → `conflit de version : toast et fiche rechargée` (version
        bougée par un `PUT` d'API entre l'ouverture du formulaire et l'enregistrement)
  - [ ] neufs : `écriture de facture : ni Modifier ni Supprimer, le motif est affiché` ; `après modification :
        « Modifiée » et « Historique », qui ouvre le journal d'audit filtré` ; `rôle Consultation : aucun bouton`
  - [ ] les deux tests de liste de la 24-4b (`la liste n'offre plus ni modification ni suppression`, `la ligne renvoie
        vers la fiche, d'où part la contre-passation`) **restent** ; JSDoc reformulé
- [ ] **T8 — Tests qui changent de sens** (liste ci-dessous) : réécrits, **pas** supprimés en bloc ; chaque test retiré
      nommé au Dev Agent Record avec son remplaçant
- [ ] **T9 — Documentation** (AC 19) — cf. Dev Notes ; `make fr` dans `docs/manual/`, PDF commités, contrôle aplati de
      l'AC 19, table des matières vérifiée
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
- **Audit par clé d'API de la création, de la contre-passation et de la dévalidation** : non — défaut préexistant
  (`routes/journal_entries.rs:460`, `:566` ; `invoices::unvalidate` sans `actor_api_key_id`, `invoices.rs:1486-1492`),
  signalé à l'orchestrateur pour une issue ; cette story ne le corrige que sur les deux verbes qu'elle rouvre.

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
| `kesh-api/tests/period_lock_e2e.rs:3-10` (doc de module) | « la 24-4b a supprimé `journal_entries::update` et refuse le `DELETE`, donc plus rien n'est réécrivable » | reformuler : `update` est rétablie, et le verrou de période la garde sur l'ancienne **et** la nouvelle date (D4 étape 7) |
| `kesh-report/src/balance_sheet.rs:31-34` | « L'invariant était aussi tenu par `journal_entries::update`, supprimée … son `entry_date` ne peut plus sortir de ses bornes » | reformuler : `update` le tient de nouveau, par sa garde de date (D4 étape 5) |
| `kesh-db/src/repositories/accounts.rs:544-545` | « Le modèle était `journal_entries::update`, supprimée par la Story 24-4b » | reformuler (le modèle existe de nouveau) |
| `journal_entries.rs` `mod tests` — **tous** les commentaires qui citent le gel ou la 24-4b, pas seulement `:2120`, `:2396-2444`, `:2509` | gel | relevés par le deuxième `grep` de T0, traités un par un |

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

**Tests unitaires de l'ancien `update`** — retirés de `journal_entries.rs` `mod tests` par la 24-4b (`git show
08e20353 -- crates/kesh-db/src/repositories/journal_entries.rs`, relevé au sol), avec le helper `is_no_op_change`
(production, rétabli par T2) :

| test (`d2910022`) | décision |
|---|---|
| `update_no_op_returns_unchanged_entity_no_lines_churn` (`:2922`) | **rétabli**, plus l'assertion « aucune entrée `journal_entry.updated` » (AC 3) |
| `update_no_op_in_closed_fy_returns_fiscal_year_closed` (`:3008`) | **rétabli** (AC 5, exercice clos seul) |
| `update_no_op_with_inactive_account_returns_inactive_error` (`:3102`) | **rétabli** (AC 4, C-15-8-11) |
| `update_partial_change_bumps_version` (`:3187`) | **rétabli** (AC 1) |
| `test_update_project_only_change_is_not_noop` (`:3459`) | **rétabli** |
| `test_update_grandfathers_preexisting_archived_project` (`:3516`) | **rétabli** (C-15-8-4 garde le *grandfathering* des projets) |
| `test_update_moves_archived_tag_between_lines` (`:3604`) | **rétabli** |
| `test_update_grandfathers_non_postable_by_account` (`:3762`) | ⛔ **inversé** → `test_update_refuses_a_line_on_an_account_made_non_postable` (AC 4) : ses deux cas (édition du libellé, ligne ajoutée sur le compte) rendent 400 |

Helpers : `three_accounts` **rétabli** (disparu) ; `mk_project`, `line`, `tagged_line` existent encore
(`:3439-3469`, déplacés par la 24-4b pour `test_create_line_projects_mixed`) — **ne pas les dupliquer**. Les appels
`update(…)` des tests rétablis prennent la nouvelle signature (`actor_api_key_id = None`).

**Frontend**

| site | à faire |
|---|---|
| `src/lib/features/journal-entries/JournalEntryForm.svelte:55-59` (« CRÉATION SEULE ») | mode édition rétabli (D8) |
| `src/lib/features/journal-entries/form-helpers.ts:1-13` | fonctions rétablies, doc revu ; `form-helpers.test.ts` rétabli |
| `src/lib/features/journal-entries/journal-entries.api.ts:56-62` | `updateJournalEntry`, `deleteJournalEntry` |
| `src/lib/features/journal-entries/journal-entries.types.ts` | `UpdateJournalEntryRequest`, champs du détail, `ModificationBlocker` |
| `src/routes/(app)/journal-entries/+page.svelte:34` (commentaire « plus de mode 'edit' ») | reformuler : l'édition vit sur la fiche ; ⛔ **ne pas inverser** le gel sur ce fichier (D8) |
| `src/routes/(app)/journal-entries/[id]/+page.svelte` | D8 |
| `src/routes/(app)/settings/opening-balances/+page.svelte:233-234` (repli de `opening-balances-complete-confirm`, « Elle ne se modifie plus ensuite ») et `:387-388` (repli de `opening-balances-locked-already-has-entries`) | replis Svelte alignés sur le nouveau texte des clés — ⚠️ un `grep` sur la clé seule ne rend pas la ligne du repli suivante |
| `src/lib/shared/i18n-keys.test.ts:165-182` | `sitesTotal` + ventilation |
| `tests/e2e/journal-entries.spec.ts:274-…` (bloc « le gel (Story 24-4b) ») | les deux tests de liste **restent vrais** (la liste n'a toujours ni ✎ ni 🗑) — renommer le bloc, reformuler le JSDoc ; parcours AC 16 : specs nommées en **T7-bis** |
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
| `:380` *keshnote* § postabilité | « **Une écriture déjà enregistrée ne se modifie plus du tout** : depuis la version qui introduit le gel, elle se corrige par contre-passation » | ⛔ faux — dire qu'elle se modifie tant que l'exercice est ouvert, et que la modification applique la **même** garde de postabilité, **y compris aux lignes non touchées** (C-15-8-4) |
| `:472-476` § « Cycle de vie » | « immédiatement comptabilisée » | reste vrai ; ajouter qu'elle reste **modifiable** jusqu'à la clôture (cadre D1) |
| `:478-494` § « Une écriture enregistrée ne se modifie plus » | gel inconditionnel | ⛔ **section réécrite et renommée** (ex. « Modifier ou supprimer une écriture ») — le titre est aussi l'entrée **7.4 de la table des matières** du PDF, à contrôler après `make fr` ; la section n'a pas de `\label`, aucun `\ref` ne la vise (vérifié : les renvois s'écrivent en toutes lettres, « voir « Corriger une écriture » », `:380`) : le cadre D1, la trace d'audit avant/après, le lien « Historique », la date qui reste dans l'exercice, la suppression qui laisse un trou ; ⚠️ le *keshtip* « Pourquoi il n'y a pas de brouillon » reste vrai, mais « ce que vous validez part directement aux livres » appelle « … et reste corrigeable tant que l'exercice est ouvert » |
| `:506-510` § verrou de période | « aucune écriture déjà passée dans la période ne pourra plus en disparaître » | ajouter « ni être modifiée » ; et dire que **verrouiller est le geste qui fige un trimestre déclaré** (D9) |
| `:532-570` § contre-passation, *keshnote* `:554` | « La contre-passation est **imposée** : la modification et la suppression d'une écriture sont refusées » | ⛔ faux — la contre-passation devient **le** chemin pour une écriture d'exercice clos, de période verrouillée ou de pièce ; « en usage courant, une seule voie fait encore disparaître une écriture » devient faux |
| `:572-585` § Numérotation | « Une écriture ne se supprime plus » ; « si la dernière écriture disparaît — par la dévalidation de sa facture » | la suppression d'une écriture manuelle creuse aussi un trou ; le numéro n'est jamais repris |
| `:631` § clôture | « La modification et la suppression, elles, sont refusées en permanence — clôture ou non » | ⛔ inversé : la clôture est **ce qui** les ferme |
| `:642` § réouverture | « L'exercice rouvert redevient modifiable » | redevient **vrai** pour les écritures manuelles (cadre D1) — vérifier la formulation |
| `:667` *keshnote* soldes de départ | « L'écriture d'ouverture, comme toutes les autres, ne se modifie ni ne se supprime ensuite » | ⛔ faux — c'est le cas déclencheur : dire comment corriger une inversion (fiche → Modifier), et que la supprimer quand elle est seule rouvre la génération (D6) — la nouvelle ouverture prenant alors le **numéro 2** (C-15-8-12) |
| `:681` *keshnote* compléter | « On le corrige … en contre-passant l'écriture fautive, ou par une écriture de correction » | ajouter **la modification** en premier chemin, et la ligne retirée qui redevient complétable |
| `:2115-2140` FAQ « revenir en arrière » | « Écriture : ni modification ni suppression, jamais » | ⛔ faux ; la puce « exercice clôturé » reste vraie pour la contre-passation |

Et `docs/manual/fr/admin-manual.tex` : `:1796` (clés `read-write`), `:1834-1838`, quatre passages nommés :
`:1834` (verrou de période : « aucune écriture de la période ne peut plus en être **supprimée** » → « ni modifiée ni
supprimée ») ; `:1835` (« n'est plus ni modifiable ni supprimable… L'immutabilité n'est donc plus seulement tracée, elle est
imposée » → faux ; la conformité 958f repose désormais sur la **trace avant/après**, la clôture et le verrou de
période) ; `:1837` (« En usage courant, une seule voie fait encore disparaître une écriture […] Ce n'est pas une
brèche dans l'immutabilité » → la suppression d'une écriture manuelle est une seconde voie, tracée ; l'immutabilité
n'est plus le cadre) ; `:1838` (« conserve aussi les modifications et suppressions **antérieures au gel** » → et
postérieures, sous `journal_entry.updated` / `journal_entry.deleted`), `:2004` (Olico art. 3 : « les corrections
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
| corps de `update`, no-op, *grandfathering* projets — **modèle**, pas copie (D4) | `git show d2910022:crates/kesh-db/src/repositories/journal_entries.rs` `:870-1208` |
| tests unitaires de l'ancien `update` | `git show 08e20353 -- crates/kesh-db/src/repositories/journal_entries.rs` (lignes `-`) |
| mode édition du formulaire, helpers, tests, api, types | `git show 08e20353 -- <fichier> \| git apply -R --3way` (D8, tableau des fichiers) — **jamais** une copie de l'état `d2910022` |
| test d'entrelacement à deux connexions | `test_fixtures::attendre_une_requete_en_cours` (`test_fixtures.rs:560`) ; exemples `opening_complement_repository.rs:717`, `reconciliation_cancel.rs:148` |
| audit par acteur (utilisateur ou clé) | `NewAuditLogEntry::for_actor` (`entities/audit_log.rs`), patron `invoices.rs:2521` |
| inventaire des propriétaires | `reversal_blockers` (`journal_entries.rs:1265`) |
| mappage d'erreur à motif | `EntryNotReversable` (`kesh-api/src/errors.rs:2698`) |
| instantané d'audit | `entry_snapshot_json` (`:906`) |
| verrou de période, seuil inclusif | `create_in_tx_inner` `:250-326`, `delete_in_tx` `:1059-1089` |
| validation des comptes | `validate_lines_accounts_in_tx` (`:84`) |
| montage de chaque pièce | `every_document_owned_entry_is_refused` (`journal_entry_reversal_e2e.rs:485`) |
| filtres d'URL du journal d'audit | `audit-log/+page.svelte:120-125` (`parseQueryFromUrl`) |

### Pièges

- ⛔ **Le `FOR UPDATE` de l'écriture est le PREMIER acte de la transaction** (D2) : aucune lecture ordinaire avant
  lui — ni projets, ni borne, ni rien. Une lecture ordinaire ouvre la vue `REPEATABLE READ` ; posée avant le verrou,
  elle rend la garde aveugle à ce qui a été commité pendant l'attente. Le test à deux connexions de l'AC 8 le tient.
- ⛔ **La garde D2 se lit DANS la transaction**, sous ce verrou — `reversal_blockers` est générique sur `Executor` :
  lui passer `&mut **tx`.
- ⚠️ `attendre_une_requete_en_cours` est **couplé au texte** de la requête : changer la forme du `SELECT … FOR UPDATE`
  de l'étape 1, c'est changer les motifs du test (sa doc le dit).
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
| 2026-10-08 | **Validation P1** (deux lentilles Sonnet, contexte frais, prompt versionné `15-8-validate-prompt-p1.md` ; rapports `target/gate-logs/15-8-p1-{R,F}.md`). **R** : 0 CRITICAL, 1 HIGH, 4 MEDIUM, 6 LOW ; **F** : 0 CRITICAL, 1 HIGH, 3 MEDIUM, 4 LOW ; R1 = F1 (même défaut), R7 = F7, R3 ≈ F4 + F8 — **tout appliqué**. **HIGH (R1/F1)** : la fiche plaçait deux lectures ordinaires (projets, borne de verrou) avant le `FOR UPDATE` de l'écriture ; en `REPEATABLE READ` la garde lisait alors une vue antérieure à l'attente du verrou, et un `PUT` pouvait réécrire une écriture contre-passée pendant cette attente — le motif même que la 24-4b fermait. Remédié : `FOR UPDATE` de l'écriture premier acte (D2, D4), ordre des verrous écriture → [sentinelle → projets] → exercice (C-15-8-10), refus projet différé à l'étape 6, test à deux connexions avec mutation à tuer (AC 8). **MEDIUM** : audit par clé d'API sur `update` et `delete_in_tx` (`for_actor`, AC 3 et 9 — F2) ; formulaire rétabli par **inversion du commit du gel** `08e20353` sur cinq fichiers, la page de liste, la spec E2E et `i18n-keys.test.ts` exclus, mode édition spécifié (dates bornées à l'exercice de l'écriture, `FISCAL_YEAR_CLOSED` hors `notifyMissingFiscalYearOrFallback`, 409 de course) — F3, C-15-8-15 ; `modification_guard` / `modification_blocker` nommées, `ModificationBlocker`, table de correspondance écran ↔ écriture qui rend l'AC 14 testable (R5, C-15-8-13) ; inventaires : huit tests unitaires de l'ancien `update` (sept rétablis, un **inversé**), quatre specs Playwright **remplacées** par des parcours depuis la fiche plus trois neuves, AC 3 / AC 4 tranché (C-15-8-11), sites périmés ajoutés (`user-manual.tex:380`, table des matières, `admin-manual.tex:1834-1838`, `period_lock_e2e.rs:3-10`, `balance_sheet.rs:31-34`, `accounts.rs:544-545`), grep de l'AC 17 sur le dépôt entier (R2, R3, F4) ; T0, T7-bis (Playwright) et Vitest ajoutés (R4). **LOW** : repli `code()` (R6), « sept motifs sur sept colonnes » (R7/F7), clés i18n neuves nommées et replis Svelte localisés (R8), AC 5 exercice clos seul, AC 20 export, complément 25-7 et numéro 2 asserté à l'AC 13, I2 borné hors restauration (R9, R10, C-15-8-12), rôle Consultation sans boutons (R10, C-15-8-14), `UpdateJournalEntryRequest` et `prepare_new_journal_entry` définis (R11, C-15-8-16), garde-fou D3 monté sur le squash avec un troisième contrôle par nom de colonne (F5, F6), T0 d'inventaire au sol (F8). **Propagation** : symptômes grepés dans la fiche (`huit`, numéros d'étape de D4, `d2910022`, `NewAuditLogEntry::user`, ordre `IS_A_REVERSAL`/`ALREADY_REVERSED`, `test_schema_guard`, lignes `:1835-1838`). **Recompte** (fiche, après remédiation) : 20 AC, 12 tâches (T0 à T10 et T7-bis), 4 invariants, 10 décisions, choix C-15-8-1 à 16. **Signalé à l'orchestrateur** : audit par clé absent de `create`, `reverse` et `invoices::unvalidate` (préexistant, hors périmètre) ; collision attendue de D3 avec la 15-1a. |
