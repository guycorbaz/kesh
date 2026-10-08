# Story 15.8a : Modifier une écriture tant que son exercice est ouvert

Status: ready-for-dev

<!-- Issue de la story 15-8, DÉCOUPÉE le 2026-10-08 après la validation P2 (choix C-15-8-17 de
     `epic-15-choix-autonomes.md`). Elle porte la MODIFICATION ; la suppression et l'historique visible
     sur la fiche sont dans `15-8b-supprimer-une-ecriture.md`, qui dépend de celle-ci. Choix applicables :
     C-15-8-1, C-15-8-3 à C-15-8-8, C-15-8-10, C-15-8-11 (révisé par C-15-8-18), C-15-8-13 à C-15-8-16,
     C-15-8-17 à C-15-8-21, et ceux de la validation P3 : C-15-8-22 à C-15-8-29 (C-15-8-23 corrige C-15-8-19 ;
     C-15-8-24 corrige C-15-8-5, C-15-8-10, C-15-8-12 et C-15-8-13). Historique des passes P1 et P2 : Change Log de l'index
     `15-8-modifier-une-ecriture.md` ; P3 : Change Log de cette fiche. -->

**Issue : [#532]** (`refs #532` — c'est la 15-8b qui la ferme) — CR du Project Lead, 2026-10-08, constat de recette de
la v0.12.1. ⚠️ **URGENTE.** **Révise la Story 24-4b** (`24-4b-gel-ecriture.md`, gel inconditionnel, #380).

**Dépendances** — ✅ **la 15-5a est mergée** (`b11a074a`, PR #535) et **la branche est rebasée dessus** (2026-10-08,
validation P3). Vérifié au code après le rebase : `validate_lines_accounts_in_tx(tx, company_id, account_ids,
enforce_postable)` n'a plus de paramètre `exempt_ids` ; `DbError::AccountsNotPostable` rend **`400
ACCOUNT_NOT_POSTABLE`** avec `details.rejected[{accountId, accountNumber}]` ; `400 INACTIVE_OR_INVALID_ACCOUNTS` reste
au compte **inconnu, d'une autre société ou archivé** (qui prime sur un compte non imputable d'une autre ligne) ;
`case 'ACCOUNT_NOT_POSTABLE'` est au formulaire de saisie ; `## [0.13.0] — Non publié` existe au `CHANGELOG.md` ;
`ACCOUNT_NOT_POSTABLE` figure à `docs/api-external.md:415`. ⚠️ **Les numéros de ligne de cette fiche sont ceux d'avant
le rebase**, sauf mention contraire ; la **table des décalages mesurés** est en T0 — s'y reporter, et revérifier.
**La 15-8b dépend de cette story.**

## Story

**As a** personne qui tient les livres,
**I want** pouvoir corriger une écriture saisie à la main — l'écriture d'ouverture comprise — tant que son exercice
n'est pas clôturé et que sa période n'est pas verrouillée,
**so that** une inversion de débit et de crédit se corrige là où elle est, sans trois écritures pour une, et sans que
la correction cesse d'être apparente.

## Le déclencheur, et ce qui change

Guy a saisi ses soldes de départ (25-7) avec débit et crédit inversés. Depuis la 24-4b, toute écriture est gelée dès
son insertion : `PUT` et `DELETE /api/v1/journal-entries/{id}` rendent **409 `ENTRY_IS_POSTED`**, et la seule
correction est la contre-passation datée du jour — trois écritures (l'ouverture fausse, son inverse, une OD juste), et
les comptes deviennent « mouvementés », donc **non complétables** (25-7).

> « La majorité des logiciels comptables permettent ces modifications tant que l'exercice comptable n'est pas
> bouclé : je veux cette fonctionnalité dans Kesh. » — Guy, #532

**Cette story rouvre la modification** (`PUT`). La suppression (`DELETE`), rouverte dans **le même cadre**
(C-15-8-2), est la 15-8b : jusqu'à son merge, le `DELETE` garde son refus `409 ENTRY_IS_POSTED`.

## Ce qui ne change pas — les motifs de la 24-4b qui restent valables (normatif, #532)

| motif de la 24-4b | ce qu'il devient ici |
|---|---|
| **La correction doit être apparente** (art. 958f CO, Olico art. 3) | chaque modification écrit `journal_entry.updated` avec l'instantané **avant et après, lignes comprises**, l'utilisateur **ou la clé d'API** (`NewAuditLogEntry::for_actor`, D4) et l'horodatage |
| **Les écritures d'une pièce restent gelées** — leur montant est lu ailleurs (`invoice_settlements.amount`, reste dû) | refus pour **toute** écriture référencée par une table métier — l'inventaire est celui de `reversal_blockers` (D2), fermé par un garde-fou (D3) — **et** pour le paiement détaché d'une facture fournisseur annulée (C-15-8-20) |
| **Contre-passée et contre-passation restent gelées** | `ENTRY_IS_REVERSED` (contre-passée) et `IS_A_REVERSAL` (contre-passation) |
| **Période verrouillée et exercice clôturé intouchables** | `FISCAL_YEAR_CLOSED` (400) ; **`LATER_FISCAL_YEAR_CLOSED`** (400) dès qu'un exercice **postérieur** est clos — le bilan est cumulatif (C-15-8-22) ; `PERIOD_LOCKED` (400) sur l'**ancienne ET la nouvelle** date |
| **Mêmes contrôles qu'à la saisie** | équilibre (`kesh_core::accounting::validate`), comptes **actifs** et **imputables** sans exemption (C-15-8-4) — avec **les deux refus de la saisie** (15-5a), nouvelle date dans l'exercice **de l'écriture** (C-15-8-3) |

## Décisions

### D1 — Le cadre : ce qui se modifie, ce qui ne se modifie pas

Une écriture est **modifiable** par la route si, et seulement si, toutes ces conditions tiennent (la 15-8b applique
**le même** cadre à la suppression) :

1. son exercice est **ouvert** ;
1-bis. ⛔ **aucun exercice postérieur au sien n'est clos** (C-15-8-22, finding F1 de la validation P3 de la 15-8b) ;
2. elle n'est **pas contre-passée** et n'est **pas une contre-passation** ;
3. **aucune pièce** ne la référence (facture, avoir, facture fournisseur — achat ou règlement —, règlement client —
   solde compris —, transaction bancaire rapprochée) ;
4. elle n'est **pas le paiement détaché d'une facture fournisseur annulée** (C-15-8-20 — voir plus bas) ;
5. sa date **et sa nouvelle date** sont **postérieures** à `books_locked_through`.

⇒ En pratique : les **écritures saisies à la main**, l'**écriture d'ouverture** (25-7) et les **compléments de soldes
de départ** (25-7) — qu'aucune table ne référence, cf. D6.

⛔ **Pourquoi la condition 1-bis** (C-15-8-22). Le bilan est **cumulatif** : les soldes des comptes de bilan sont
« cumulatifs depuis l'origine — Σ de **toutes** les écritures `entry_date ≤ date d'arrêté`, tous exercices confondus »
(`kesh-report/src/balance_sheet.rs:9-11`). Modifier une écriture de l'exercice N change donc le bilan de **tout**
exercice postérieur. Or l'état « N ouvert, N+1 clos » est **atteignable** : `fiscal_years::close`
(`fiscal_years.rs:739`) ne contrôle aucun exercice antérieur — seule la **réouverture** a une garde d'ordre (LIFO,
`find_later_closed_in_tx`, `fiscal_years.rs:639`, appelée par `reopen` `:877`). Sans cette condition, la modification
réécrirait le bilan d'un exercice que l'application tient pour clos. Refus : **400 `LATER_FISCAL_YEAR_CLOSED`**, qui
nomme l'exercice postérieur clos (D4, étape 2-bis). Le chemin de correction reste la contre-passation (datée du jour,
dans l'exercice courant), ou la réouverture de l'exercice postérieur par un administrateur.

⚠️ **Le même trou existe ailleurs, et n'est pas fermé ici** : une **création** dans N (`POST /journal-entries`), une
dévalidation de facture de N, un règlement daté dans N changent eux aussi le bilan cumulé d'un N+1 clos. Défaut
préexistant, **hors périmètre** de cette story urgente — signalé à l'orchestrateur pour une issue (C-15-8-22).

⚠️ **Un compte archivé ou non imputable ne gèle PAS l'écriture** (contrairement à la contre-passation, où
`ACCOUNT_ARCHIVED` masque le bouton) : on peut remplacer le compte. Mais l'enregistrement refuse tant qu'une ligne vise
un compte archivé (`INACTIVE_OR_INVALID_ACCOUNTS`) ou non imputable (`ACCOUNT_NOT_POSTABLE`, qui le nomme) — **y
compris une ligne qu'on n'a pas touchée** (C-15-8-4).

⚠️ **Le paiement détaché** (C-15-8-20, finding F3 de la validation P2). `supplier_invoices::cancel_in_tx`
(`supplier_invoices.rs:905-912`) annule une facture fournisseur **payée** en contre-passant l'achat **sans**
contre-passer le règlement, puis remet `settlement_journal_entry_id` à `NULL` (arbitrage du 2026-09-26 : l'argent est
sorti, le paiement reste au grand livre comme « paiement sans facture »). Plus aucune colonne ne référence alors cette
écriture : sous les conditions 1 à 3, elle deviendrait modifiable, alors qu'elle représente une sortie de banque réelle.
**Il n'existe aucun marqueur structurel de l'origine d'une écriture** — vérifié au sol : `journal_entries` n'a que
`journal` (`Achats|Ventes|Banque|Caisse|OD`, partagé avec la saisie manuelle) et `reverses_entry_id` ; l'audit
`journal_entry.created` est écrit par `create_in_tx_inner` pour **tous** les flux (`journal_entries.rs:471-481`). La
**seule** trace en base est l'audit du geste : `supplier_invoice.cancelled`, entité `supplier_invoice`,
`details_json.settlementJournalEntryId` (`supplier_invoices.rs:931-935`). La garde la lit (D2) : c'est la solution la
moins invasive (ni migration, ni changement du geste d'annulation), et la **contre-passation reste offerte** — le
manuel la promet (`user-manual.tex:1335-1337`), elle n'est donc **pas** ajoutée à `reversal_blockers`.

⚠️ **Trois réserves, écrites** (C-15-8-25, finding F7 de la validation P3) — la dette structurelle est **#541** (« la
colonne qui garde le lien d'un paiement détaché ») ; cette lecture d'audit en est le palliatif, à retirer avec elle :

1. **Elle révise un arbitrage, en lettre.** Le garde-fou de la 25-3-c dit de **ne pas** rechercher ce lien dans l'audit
   (`frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte:409-411` ; fiche 25-3-c, arbitrage Q1 du 2026-09-26 :
   « écriture de règlement libre, comme si aucune facture n'y était liée »). Le paiement détaché cesse d'être « libre » :
   il devient **gelé à la modification (et, 15-8b, à la suppression), contre-passable**. C'est une révision de cet
   arbitrage, nommée comme telle — le commentaire de `+page.svelte:409-411` se reformule (T6), il ne se contredit pas en
   silence.
2. **Faux positif possible après restauration.** L'import d'un `.keshbackup` **fusionne** `audit_log` au lieu de le
   remplacer (`backup.rs:438-455`) ; les autres tables sont vidées puis rechargées. Une ligne d'audit
   `supplier_invoice.cancelled` d'une **autre histoire** peut alors se joindre, par `entity_id`, à une facture
   `cancelled` restaurée qui porte le même `id`, et désigner par `settlementJournalEntryId` une écriture manuelle sans
   rapport — gelée à tort (409 sans issue par la modification ; la contre-passation reste offerte). Improbable (il faut
   la coïncidence de deux `id`), **limite assumée**, dite au manuel administrateur (§ restauration) en une phrase.
   *(Le cas bénin — une archive plus ancienne de la même installation — est neutralisé par `si.status = 'cancelled'` :
   la facture restaurée y est `paid` et reprend sa colonne.)*
3. **Coût de lecture.** `JSON_VALUE(...)` ne s'indexe pas : la requête balaie les factures **annulées** de la société
   et leurs lignes d'audit (index `idx_audit_log_entity`), à **chaque** `GET /journal-entries/{id}` (motif d'écran) et
   chaque `PUT`. Acceptable au volume actuel (les annulations de factures payées sont rares) ; à dire au doc-comment, et
   c'est un argument de plus pour #541.

La requête est **bornée par société des deux côtés** : `si.company_id = ?` **et** `al.company_id = ?` — le filtre
strict du journal d'audit (`audit_log.rs:204`, « sans `OR company_id IS NULL` »). Une trace sans société
(`company_id` NULL : utilisateur sans société au moment du geste, cas que la 25-1c-zero dit « indéterminable ») n'est
**pas** lue — faux négatif résiduel, nommé : l'écriture redeviendrait modifiable. Les annulations de factures
fournisseur payées datent de la 25-3-c, postérieure à la colonne (`20260915000001_audit_log_company_id.sql`), et
l'écrivain renseigne `company_id` depuis l'utilisateur (`audit_log.rs:91-92`).

### D2 — Où vit la garde : au repository, et l'inventaire n'est PAS réécrit

⛔ **Un garde-fou posé à la route ne protège que la route** (24-4b D2) : les refus vivent dans
`crates/kesh-db/src/repositories/journal_entries.rs`.

⛔ **L'inventaire des propriétaires existe déjà, une seule fois** : `reversal_blockers` (`journal_entries.rs:1265`),
une requête à sous-requêtes corrélées qui rend **huit** motifs : **sept motifs de propriété ou d'état**, lus sur les
**sept** colonnes de clé étrangère vers `journal_entries` autres que `journal_entry_lines.entry_id` (D3 ; la huitième
colonne est l'enfant, non gelant), **plus** le compte archivé. La garde de modification le réutilise, filtré, et y
ajoute **un** contrôle qui ne vaut que pour elle :

| motif | modification |
|---|---|
| `IsAReversal` | refus `IS_A_REVERSAL` (409) |
| `AlreadyReversed` | refus `ENTRY_IS_REVERSED` (409, **existant** — contrat de la 24-4a, testé) |
| `OwnedByInvoice`, `OwnedByCreditNote`, `OwnedBySupplierInvoice`, `OwnedBySettlement`, `MatchedBankTransaction` | refus sous le **code du motif** (409), `details.documentId` / `details.documentNumber` |
| `AccountArchived` | ⛔ **ignoré** — ce n'est pas un gel (D1) |
| paiement détaché (hors `reversal_blockers`, C-15-8-20) | refus **`DETACHED_SUPPLIER_SETTLEMENT`** (409), `details.documentId` = id de la facture fournisseur annulée, `details.documentNumber` = son numéro |

**Les types** (`crates/kesh-db/src/errors.rs`, à côté de `ReversalBlocker`) :

```rust
/// Pourquoi une écriture ne se modifie (ni, 15-8b, ne se supprime) pas — hors exercices clos (le sien, un postérieur) et période.
pub enum ModificationGuard {
    /// Un motif de `reversal_blockers`, jamais `AccountArchived`.
    Owned { blocker: ReversalBlocker, document_id: Option<i64>, document_label: Option<String> },
    /// Le paiement d'une facture fournisseur annulée, détaché par `cancel_in_tx` (C-15-8-20).
    DetachedSupplierSettlement { supplier_invoice_id: i64, supplier_invoice_number: Option<String> },
}
impl ModificationGuard {
    pub fn code(&self) -> &'static str { /* blocker.code() | "DETACHED_SUPPLIER_SETTLEMENT" */ }
}
```

Nouveau `DbError::EntryNotModifiable(ModificationGuard)`, **jamais** construit avec `Owned { blocker: AlreadyReversed }`
(celui-là devient `EntryIsReversed`). Mappé côté `kesh-api` : pour `Owned`, **exactement** comme `EntryNotReversable`
(`errors.rs:2698`) — même code (celui du motif), même corps `details`, **mêmes clés i18n**, dont les messages nomment
déjà le chemin de la pièce ; ⛔ **DRY** : extraire le corps du mappage dans une fonction partagée par les deux
variantes, ne pas le recopier. Pour `DetachedSupplierSettlement` : 409, code `DETACHED_SUPPLIER_SETTLEMENT`, clé
`journal-entries-modify-blocked-detached-settlement`, numéro de la facture suffixé comme pour les autres pièces. Code
machine : `DbError::code()` (`errors.rs:759`) rend pour `EntryNotReversable` le **repli générique**
`"ENTRY_NOT_REVERSABLE"` (commentaire `:757-758`) ; la nouvelle variante prend de même un repli
`"ENTRY_NOT_MODIFIABLE"`, et le code **exposé** vient du mappage `kesh-api`, qui rend `guard.code()`.

**La garde d'écriture, nommée** — `journal_entries::modification_guard(conn: &mut sqlx::MySqlConnection, company_id,
id) -> Result<Option<ModificationGuard>, DbError>` (C-15-8-24, finding R3-1 de la validation P3) : le **premier** motif
de `reversal_blockers` **hors `AccountArchived`**, sinon le paiement détaché. ⛔ **Une connexion, pas un `Executor`
générique** : son corps enchaîne **deux** lectures (`reversal_blockers`, puis la trace), et `reversal_blockers` prend
son exécuteur **par valeur** (`journal_entries.rs:1265`) — un `E: Executor` consommé par le premier appel ne sert pas au
second. Avec `&mut MySqlConnection`, chaque appel reprête `&mut *conn`. `update` (et `delete_in_tx`, 15-8b) passent
`&mut **tx` ; `modification_blocker` passe la connexion qu'elle a acquise (D8). La trace se lit par :

```sql
SELECT si.id, si.supplier_invoice_number FROM supplier_invoices si
JOIN audit_log al ON al.entity_type = 'supplier_invoice' AND al.entity_id = si.id
 AND al.action = 'supplier_invoice.cancelled' AND al.company_id = ?
WHERE si.company_id = ? AND si.status = 'cancelled'
  AND JSON_VALUE(al.details_json, '$.settlementJournalEntryId') = ?
LIMIT 1
```

(index `idx_audit_log_entity` ; bornée par société **des deux côtés** — réserves et faux négatif résiduel en D1,
C-15-8-25 ; le filtre `si.status = 'cancelled'` neutralise la restauration d'une archive plus ancienne de la même
installation, qui ramène la facture à `paid` et lui rend sa colonne. ⚠️ Comparaison `JSON_VALUE(...) = ?` avec un
entier lié : `JSON_VALUE` rend une **chaîne** ; vérifier au premier test (AC 6) que MariaDB 10.11 compare bien
numériquement, ou lier l'id en chaîne — non exécuté en validation. Le numéro `supplier_invoices.supplier_invoice_number`
est **nullable**, d'où l'`Option` — même lecture que `reversal_blockers`.) Elle ne lève rien : la conversion en erreur est une seconde fonction,
`modification_refusal(guard: ModificationGuard) -> DbError` (`Owned { AlreadyReversed }` → `EntryIsReversed`, tout
autre → `EntryNotModifiable(guard)`), partagée par `update` et — 15-8b — `delete_in_tx`. Le motif d'**écran** est une
autre fonction, qui l'appelle (D8).

**Sérialisation — le verrou d'abord, et aucune lecture ordinaire avant les verrous.** Le dépôt tourne en
`REPEATABLE READ` (`pool.rs:17`) : la vue de lecture InnoDB naît à la **première lecture ordinaire** de la
transaction, et toutes les lectures ordinaires suivantes — `reversal_blockers`, `reversed_by`, la trace du paiement
détaché, la borne de verrou, les comptes — lisent **cette vue**, pas l'état courant. Une lecture ordinaire posée
**avant** un verrou fige donc une vue antérieure à l'attente de ce verrou : une contre-passation (ou une pièce)
commitée pendant cette attente reste **invisible** à la garde, et le `PUT` réécrit une écriture déjà contre-passée —
exactement le motif « contre-passation faussée » que la 24-4b fermait. *(Défaut de la première rédaction, relevé en
validation P1.)*

D'où la règle, celle que suivent déjà `delete_in_tx` (`:1005`) et `reverse_in_tx_inner` (`:1518`) : **le premier acte
de la transaction est le `SELECT … FOR UPDATE` de l'écriture**, et **aucune lecture ordinaire ne le précède**.

⚠️ **La vue s'ouvre ensuite à l'étape 1-bis, avant les verrous des projets et des exercices — et c'est voulu**
(C-15-8-23, findings R3-2 et F1 de la validation P3, arbitrage de l'orchestrateur). La lecture des projets déjà présents
sur l'écriture est une **lecture ordinaire**, faite **après** l'étape 1 : l'écriture étant tenue en exclusif, **ses
lignes sont stables** (seul `update` les réécrit, et il passe par ce verrou), la lecture est donc exacte. La première
rédaction la faisait en `LOCK IN SHARE MODE` pour retarder l'ouverture de la vue ; ce verrou partagé portait, sous
`REPEATABLE READ`, un **verrou d'intervalle** sur `journal_entry_lines` (index non uniques sur le préfixe `entry_id`,
`test-schema/0001_schema_squash.sql:796-797`) jusqu'à l'enregistrement suivant — jusqu'au *supremum* quand l'écriture
est la plus récente —, **avant** le verrou de l'exercice : une création du même exercice (qui tient l'exercice et
insère ses lignes dans cet intervalle) et le `PUT` formaient un **quatrième cycle**, non hérité, dont la victime
pouvait être la création — non rejouée, donc un 500. La phrase « coût nul » était fausse ; elle est retirée avec le
verrou.

**Ce que coûte une vue ouverte avant les verrous des exercices — et pourquoi la garde reste juste** :

- **tout ce qui change la garde passe par la ligne verrouillée de l'écriture**, donc attend notre commit : une
  contre-passation verrouille l'origine d'abord (`reverse_in_tx_inner`) ; une pièce qui s'attache (facture, règlement,
  rapprochement) prend le verrou **partagé** de clé étrangère sur l'écriture ;
- **le paiement détaché** n'y passe pas — `cancel_in_tx` verrouille l'écriture d'**achat**, pas celle du règlement —,
  mais il est sûr par **l'atomicité du commit** : la colonne `settlement_journal_entry_id` passe à `NULL` et la ligne
  d'audit s'écrit **dans la même transaction**, et la garde lit les deux **dans la même vue** — elle voit soit la
  colonne (`OwnedBySupplierInvoice`), soit la trace (`DetachedSupplierSettlement`), jamais ni l'une ni l'autre. *(La
  première rédaction invoquait un « verrou partagé de clé étrangère tenu jusqu'au commit » de l'annulation : il
  n'existe pas — mettre une clé étrangère à `NULL` ne verrouille pas le parent ; findings R3-4, F3.)* Le second site
  qui vide cette colonne, `cancel_settlement_in_tx` (`supplier_invoices.rs:1120`), **contre-passe d'abord** le
  règlement : l'écriture y devient `AlreadyReversed` → `ENTRY_IS_REVERSED` ;
- **l'exercice et les exercices postérieurs** se lisent par des lectures **verrouillantes** (étape 1-ter), qui lisent
  l'état **courant**, pas la vue : une clôture commitée pendant l'attente est vue ;
- ⚠️ **ce qui n'est PAS vu** : la borne `books_locked_through`, lue **ordinairement** (étape 1-quater) dans une vue
  ouverte avant l'attente des exercices — une pose de borne commitée pendant cette attente n'est pas vue. **Même
  tolérance qu'à la création**, qui lit la borne sans verrou et le dit (`create_in_tx_inner`, `journal_entries.rs:260-278`
  après rebase : « le seul écart possible est qu'une écriture commitée à l'instant même de la pose passe ») ; et, comme
  avant, un archivage de compte commité pendant ce temps (verrou des comptes hors périmètre, choix C21 de la 15-5a ;
  l'`INSERT` des lignes prend le verrou partagé de clé étrangère sur chaque compte, sans relire `active`).

*(À citer dans le doc-comment de `update` ; ne jamais lire la garde sur le pool hors transaction dans le chemin
d'écriture ; testé à deux connexions, AC 8.)*

### D3 — Le garde-fou d'inventaire : un ensemble clos, pas une énumération

⛔ **Une nouvelle table qui référence `journal_entries` passerait sous la garde sans que rien ne rougisse** — c'est
exactement la classe d'un test muet. Règle du `CLAUDE.md` § *Inventorier les sites NON RÉSOLUS* : on inventorie
l'ensemble clos, on n'énumère pas les formes connues.

Test d'intégration `#[sqlx::test(migrations = "./test-schema")]` — **monté sur le squash**, et c'est suffisant :
`crates/kesh-db/tests/test_schema_guard.rs` prouve déjà, sur `information_schema` complet, que le squash égale le
schéma de `MIGRATOR`. Le monter sur `MIGRATOR` obligerait à l'inscrire dans `ALLOWED_REAL_MIGRATOR_FILES`
(`test_schema_guard.rs:38`) et à rejouer 75 migrations pour un gain nul. Fichier : `crates/kesh-db/tests/
journal_entries_modification.rs` (neuf, le même que les tests de concurrence des AC 8 et 9). Toutes les requêtes portent
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

   ⚠️ **Et `REFERENCED_TABLE_NAME = 'journal_entry_lines'` doit être VIDE** (finding F7 de la P2) : la modification
   supprime et réinsère les lignes, dont les `id` changent. Une future table qui référencerait une **ligne** (lettrage
   par ligne, ventilation) ferait échouer le `PUT` en 1451 (`RESTRICT`) ou effacerait en silence (`CASCADE`). Vérifié :
   `grep -n 'REFERENCES \`journal_entry_lines' crates/kesh-db/test-schema/0001_schema_squash.sql` ne rend rien.
2. les **colonnes** de `journal_entry_lines` doivent égaler `{id, entry_id, account_id, line_order, debit, credit,
   project_id}` — car la modification **supprime et réinsère** les lignes : toute colonne neuve y serait perdue en
   silence.
3. ⚠️ **Une colonne sans clé étrangère échappe au point 1** : `information_schema.COLUMNS` dont le nom correspond à
   `%journal_entry%` ou `%entry_id%`, **hors** `journal_entries` elle-même et hors les colonnes du point 1, doit égaler
   une liste déclarée avec sa justification. Relevé au sol le 2026-10-08 (`awk` sur le squash) : **une seule**,
   `company_invoice_settings.journal_entry_description_template` — un gabarit de libellé, pas une référence ;
   `audit_log.entity_id` n'y tombe pas (nom sans motif, référence logique non gelante).

⚠️ **Angle mort assumé, nommé** : la trace du paiement détaché (D1, C-15-8-20) vit dans `audit_log.details_json`, pas
dans une colonne — aucun des trois contrôles ne la voit. Elle est tenue par son propre test (AC 6). Un futur geste qui
**rattacherait** ce paiement à une facture (le manuel l'annonce, `user-manual.tex:1333-1334`) devra revoir la garde.

Le message d'échec dit quoi faire : « trier la référence dans `reversal_blockers` (gel) ou la déclarer ici avec sa
justification ».

⚠️ **Il rougira à coup sûr avec la 15-1a (lettrage)**, qui ajoute `journal_entry_lines.lettering_id`
(`15-1a-socle-lettrage.md:67`) — par le point 2, et par le point 1 bis si le lettrage pose une clé vers une ligne. C'est
voulu : une ligne lettrée ne doit pas être réécrite, et la seconde des deux stories à merger devra trancher
(vraisemblablement un motif `LETTERED` qui gèle l'écriture). **L'orchestrateur doit le signaler à la spécification de
la 15-1a.**

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

1. extracteur `Json<UpdateJournalEntryRequest>` rétabli — ⚠️ le doc-comment actuel (`:583-595`) justifiait son
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
   corps). ⚠️ **Effet de bord sur le `POST`, assumé** (C-15-8-16) : son pré-contrôle `find_covering_date` (`:510-525`)
   se place **après** `prepare_new_journal_entry`, si bien qu'un corps à la fois déséquilibré **et** sans exercice rend
   désormais `ENTRY_UNBALANCED` avant `NO_FISCAL_YEAR` (aujourd'hui l'inverse). Greper les tests du `POST` qui cumulent
   les deux causes (`grep -rn "NO_FISCAL_YEAR\|FISCAL_YEAR_CLOSED" crates/kesh-api/tests`) et dire au Dev Agent Record
   qu'aucun ne les cumule, ou lequel change ;
3. **pas** de pré-contrôle `find_covering_date` pour le `PUT` : l'exercice est celui **de l'écriture**, connu
   seulement sous le verrou (C-15-8-3) ;
4. ⛔ **rejeu sur interblocage** (C-15-8-19) : l'appel au repository est enveloppé dans
   `kesh_db::retry::retry_with(DEFAULT_MAX_DEADLOCK_ATTEMPTS, |err: &DbError| is_deadlock_error(err), || …)`, sur le
   patron de `complete_opening_balances` (`routes/opening_balances.rs:581-610`). Le rejeu est sûr : la transaction est
   rejouée **entière** (le repository ouvre et ferme la sienne), l'interblocage l'a annulée sans rien écrire, et le
   contrôle de `version` refuserait de toute façon un second passage. La préparation (point 2) reste **hors** de la
   fermeture : elle ne lit pas la base ;
5. appel de `journal_entries::update(…, current_user.user_id, current_user.api_key_id, …)`, erreurs propagées par
   `AppError::from`.

⚠️ Les refus de **forme** (400 du corps, de `accounting::validate`) précèdent le 404, comme au `POST` : ils ne
dépendent d'aucune donnée de la base, ils ne révèlent donc pas l'existence d'une ressource d'une autre société.

**Le repository** — `journal_entries::update(pool, company_id, id, version, user_id, actor_api_key_id: Option<i64>,
updated: NewJournalEntry) -> Result<JournalEntryWithLines, DbError>`. ⛔ **Écrite contre le code ACTUEL** (après le
rebase sur la 15-5a), avec l'ancien corps pour **modèle** (`git show d2910022:crates/kesh-db/src/repositories/
journal_entries.rs`, `:870-1208`, dont `is_no_op_change` `:891`) — **jamais** recopiée de `d2910022` : neuf commits ont
touché ce fichier depuis le gel avant même la 15-5a (`git log --oneline 08e20353..HEAD -- crates/kesh-db/src/
repositories/journal_entries.rs`), et l'ancien corps appelle `validate_lines_accounts_in_tx` avec l'exemption D-A1,
dont la 15-5a a retiré le paramètre. Audit : `NewAuditLogEntry::for_actor(user_id, actor_api_key_id,
"journal_entry.updated", …)` (`entities/audit_log.rs:194`, patron de l'appel `kesh-db/src/repositories/invoices.rs:2559` — le dépôt, pas la route) — **jamais**
`::user`.

Ordre — ⛔ **le verrou de l'écriture d'abord, aucune lecture ordinaire avant lui** (D2, Sérialisation) ; puis les
refus dans l'ordre de précédence :

| étape | geste | refus |
|---|---|---|
| 1 | **premier acte de la transaction** : `SELECT je.version, je.entry_date, je.fiscal_year_id FROM journal_entries je WHERE je.id = ? AND je.company_id = ? FOR UPDATE` — l'écriture **seule**, sans la jointure `fiscal_years` (cf. ordre des verrous ci-dessous) | `NotFound` → 404 |
| 1-bis | projets : lecture **ordinaire** des projets déjà présents sur l'écriture — `SELECT DISTINCT project_id FROM journal_entry_lines WHERE entry_id = ? AND project_id IS NOT NULL` (*grandfathering* de l'ancien `update`, conservé — C-15-8-4 ; ⚠️ **la vue s'ouvre ici**, après le verrou de l'écriture qui rend ces lignes stables — **pas** de `LOCK IN SHARE MODE`, C-15-8-23) — puis `projects::validate_taggable_in_tx` sur les seuls projets **nouveaux** (sentinelle `companies` puis `FOR UPDATE` des projets) | — ⚠️ le verrou se prend ici, mais le **refus** (404 / 409 projet) est **gardé et rendu à l'étape 6**, avec les comptes : rendu ici, il parlerait avant l'exercice clos et le gel, contre l'AC 7. Une erreur de base (`DbError` hors `NotFound`/`IllegalStateTransition`) se propage aussitôt — l'interblocage compris, que le handler rejoue |
| 1-ter | `SELECT status, start_date, end_date FROM fiscal_years WHERE id = ? AND company_id = ? FOR UPDATE` ; **puis** les exercices postérieurs clos, `fiscal_years::find_later_closed_in_tx(tx, company_id, fy.start_date)` (`fiscal_years.rs:639`, `FOR UPDATE` sur l'intervalle `start_date > ?` — réutilisée telle quelle, C-15-8-22) — **derniers verrous**, lus à l'état **courant** | — |
| 1-quater | lecture ordinaire de `books_locked_through` (patron de `delete_in_tx`, requête `:1075-1081`), dans la vue ouverte à 1-bis (tolérance de D2) | — |
| 2 | exercice clos | `FiscalYearClosed` → **400** |
| 2-bis | exercice postérieur clos (C-15-8-22) | `LaterFiscalYearClosed { fiscal_year_id, fiscal_year_name }` → **400 `LATER_FISCAL_YEAR_CLOSED`**, `details.fiscalYearId` / `details.fiscalYearName` (le **plus proche** postérieur clos, celui que rend `find_later_closed_in_tx`) |
| 3 | `modification_guard` (D2), **dans la transaction** (`&mut **tx`, une `&mut MySqlConnection`) → `modification_refusal` | `Owned { AlreadyReversed }` → `EntryIsReversed` (409 `ENTRY_IS_REVERSED`) ; autres → `EntryNotModifiable` (409 `<code du motif>`, dont `DETACHED_SUPPLIER_SETTLEMENT`) |
| 4 | `version` | `OptimisticLockConflict` → 409 `OPTIMISTIC_LOCK_CONFLICT` |
| 5 | nouvelle date ∈ `[fy.start_date, fy.end_date]` | `DateOutsideFiscalYear` → 400 |
| 6 | refus projet gardé à l'étape 1-bis, s'il y en a un ; puis `validate_lines_accounts_in_tx(…, enforce_postable = true)` — la fonction **telle que la 15-5a l'a laissée** (sans `exempt_ids` : aucune exemption, C-15-8-4) | projet → 404 / 409 ; compte inconnu, d'une autre société ou archivé → `InactiveOrInvalidAccounts` (400 `INACTIVE_OR_INVALID_ACCOUNTS`) ; sinon compte non imputable → `AccountsNotPostable` (400 `ACCOUNT_NOT_POSTABLE`, `details.rejected`) — l'ordre est celui de la 15-5a |
| 7 | verrou de période : **ancienne** date, puis **nouvelle** date, seuil **inclusif** | `PeriodLocked { attempted }` → 400 |
| 8 | instantané « avant » ; **court-circuit no-op** (`is_no_op_change`) : rollback, rend l'état, ni audit ni `version` | — |
| 9 | `DELETE` lignes, `UPDATE` en-tête (`entry_date`, `journal`, `description`, `version = version + 1`, `updated_at`), `INSERT` lignes (`line_order` séquentiel) | — |
| 10 | recontrôle `SUM(debit) = SUM(credit)` | `Invariant` → 500 |
| 11 | instantané « après » ; audit `journal_entry.updated` par `for_actor`, `details = { before, after }` (`entry_snapshot_json`, `:906`) ; commit | — |

**Le refus de l'exercice postérieur clos** (C-15-8-22) — nouvelle variante `DbError::LaterFiscalYearClosed {
fiscal_year_id: i64, fiscal_year_name: String }` (`crates/kesh-db/src/errors.rs`, à côté de `FiscalYearClosed`),
`code()` → `"LATER_FISCAL_YEAR_CLOSED"` ; mappée côté `kesh-api` en **400** (même statut que `FISCAL_YEAR_CLOSED` :
c'est l'état d'un exercice, pas un conflit sur l'écriture), `details = { fiscalYearId, fiscalYearName }`, message par
la clé `journal-entries-modify-blocked-later-fiscal-year-closed` (`{ $name }`, quatre locales ; lue **aussi** par
l'écran, D8) — FR : « L'exercice postérieur { $name } est clôturé, et son bilan reprend cette écriture : elle reste
figée tant qu'il l'est. Un administrateur peut rouvrir cet exercice ; sinon, corrigez par une contre-passation. » (⚠️
tournure choisie pour ne heurter aucun motif du contrôle de l'AC 17, qui lit aussi le `.ftl`.)
*(La même clé sert au `DELETE` de la 15-8b ; tant que la 15-8b n'est pas mergée, le `DELETE` refuse de toute façon.)*
Repli Rust aligné, `#[error]` en français. ⚠️ Ne **pas** passer par `FY_REOPEN_LIFO_BLOCKED_KEY` (`Invariant`, 500) :
c'est un refus métier nommé, pas un invariant.

⚠️ **L'ordre des verrous** (C-15-8-10, complété par C-15-8-22) : **écriture → [sentinelle `companies` → projets] →
exercice → exercices postérieurs (par `start_date` croissant, l'ordre de `reopen`) → (comptes, en partagé, par les clés
étrangères de l'`INSERT` ; intervalle des lignes de l'écriture, au `DELETE` des lignes de l'étape 9)**. L'écriture se verrouille **seule** à l'étape 1 parce que la
verrouiller avec son exercice (jointure, comme `delete_in_tx`) puis prendre les projets inverserait l'ordre de la
création (`companies → projects → fiscal_years`, `create_in_tx_inner` `:236-247`) : deux saisies taguées sur le même
exercice se bloqueraient en croix. Écriture puis sentinelle ou projets : aucun chemin ne prend la sentinelle ou un
projet **puis** le verrou d'une écriture **existante** — vérifié au sol par
`grep -rnE "acquire_company_sentinel_lock|FROM companies WHERE id = \? (FOR UPDATE|LOCK IN SHARE MODE)" crates/*/src`
(sentinelles par la fonction — projets, TVA, relances, comptes bancaires — **et** en SQL littéral —
`create_opening_entry` `journal_entries.rs:581`, `lock_books` et son inverse `companies.rs:350`, `:438`, le complément
`opening_complement.rs:502` en `LOCK IN SHARE MODE`), **complété par** `grep -rn validate_taggable_in_tx crates/*/src`
— ses neuf appelants prennent la sentinelle **en interne** et le premier motif ne les rend pas (finding R3-10) :
`reconciliation.rs:2411`, `:3136`, `reconciliation_rules.rs:187`, `:274`, `supplier_invoices.rs:283`, `invoices.rs:700`,
`:1113`, `journal_entries.rs:247` (numéros d'avant le rebase). Aucun ne verrouille ensuite une écriture existante
(créations et brouillons). À **revérifier** par le développeur et à écrire dans le doc-comment.

⛔ **Mais l'ordre du `PUT` n'est PAS sans cycle**, et le dépôt n'a pas d'ordre de verrouillage unique
(`opening_complement.rs:26-35`). Trois cycles **hérités** — ils existent déjà entre la création et ces mêmes chemins —
où le `PUT` entre (findings F2, R2-3 de la P2), **nommés** ici et dans le doc-comment. ⚠️ Depuis C-15-8-22, chacun
vaut pour l'exercice de l'écriture **et pour chaque exercice postérieur** que l'étape 1-ter verrouille (la colonne
« exercice » se lit « un exercice que le `PUT` tient ») : une création, un règlement ou une contre-passation **dans un
exercice postérieur** referme le même cycle. *(Le quatrième cycle relevé en P3 — intervalle des lignes ↔ exercice —
n'existe plus : la lecture des projets ne verrouille rien, C-15-8-23 ; l'intervalle des lignes ne se prend qu'au
`DELETE` de l'étape 9, **après** tous les exercices, comme dans l'ancien `update`.)*

| cycle | le `PUT` tient… et attend… | l'autre chemin tient… et attend… |
|---|---|---|
| **exercice ↔ compte** | l'exercice (1-ter), puis le verrou **partagé** de clé étrangère sur un compte à l'`INSERT` des lignes (étape 9) | un compte `FOR UPDATE`, puis l'exercice : le règlement client (`invoice_settlements_write.rs:157` puis `create_in_tx` `:214`), le complément 25-7 (`opening_complement.rs:532` puis `:548`) — et, à leur merge, les comptes de réglage verrouillés avant l'exercice de la 15-5d |
| **exercice ↔ `companies`** | la sentinelle `companies` (1-bis, projets nouveaux), puis l'exercice | l'exercice (`FOR UPDATE` d'une création, d'une contre-passation, d'un règlement), puis le verrou **partagé** de clé étrangère `company_id` sur `companies` à l'`INSERT INTO journal_entries` |
| **projet ↔ exercice** | un projet nouveau `FOR UPDATE` (1-bis), puis l'exercice | une contre-passation d'une écriture du même exercice dont une ligne porte ce projet : l'origine **et** son exercice (`reverse_in_tx_inner`, jointure `:1526-1530`), puis le verrou **partagé** de clé étrangère sur le projet à l'insertion des lignes copiées — lues `:1610-1630`, insérées par l'appel à `create_in_tx_inner` (`:1656` ; l'`INSERT INTO journal_entry_lines` est dans `create_in_tx_inner`, `:417`) (finding R3-7) |

**Leur issue** : InnoDB casse un tel cycle par l'erreur **1213** (interblocage), que le handler **rejoue**
(`retry_with`, point 4). ⚠️ La doctrine écrite du dépôt est plus prudente (`retry.rs:7-10`,
`MULTI-TENANT-SCOPING-PATTERNS.md` Pattern 5 : « MariaDB ne détecte pas les deadlocks cross-table avant
`innodb_lock_wait_timeout` ») : un cycle non détecté se termine par l'expiration du délai (50 s, erreur **1205**,
**non** rejouée) et rend un **500**. *La phrase de la première rédaction — « un cycle résiduel serait détecté par
InnoDB, jamais une attente infinie » — était fausse sur ce point ; elle est retirée.* Le `PUT` est inscrit à la **liste
des exceptions** de Pattern 5 (`docs/MULTI-TENANT-SCOPING-PATTERNS.md`, tableau « Deny list », aujourd'hui vide,
`:336`), avec l'ordre `journal_entries → [companies → projects] → fiscal_years (l'exercice, puis les postérieurs) →
(accounts, partagé)`, sa raison (le verrou de l'écriture doit précéder toute lecture, D2) et sa mitigation
(`retry_with`). ⚠️ **Deux phrases du même paragraphe deviennent fausses** et se réécrivent avec le tableau (finding
R3-9) : `:324` (« all current write endpoints follow the documented order ») et `:330` (« Deny list of divergent
endpoints — none currently »).

⚠️ **Deux coûts, assumés et dits au doc-comment** :

- **Le verrou des exercices postérieurs** (étape 1-ter) porte sur **tout** l'intervalle `start_date > ?` : le filtre
  `status = 'Closed'` n'est pas indexé, et `find_later_closed_in_tx` s'appuie justement sur ce verrou d'intervalle pour
  se sérialiser avec une clôture concurrente (son doc-comment, « Hypothèse de verrouillage »). Un `PUT` sur une
  écriture d'un exercice ancien sérialise donc, le temps de sa transaction, les créations de **tous** les exercices
  postérieurs. Bref (une transaction de quelques requêtes), et rare (on corrige surtout l'exercice courant).
- **Un `PUT` voué au refus prend quand même la sentinelle** (finding F8) : le verrou des projets nouveaux (1-bis)
  précède tous les refus, si bien qu'un `PUT` sur une écriture gelée dont le corps ajoute un projet bloque les créations
  de la société jusqu'à son rollback. Seul un client d'API le provoque (l'écran n'offre pas « Modifier » sur une
  écriture gelée).

⛔ **Ni `entry_number`, ni `fiscal_year_id`, ni `id`, ni `created_at`, ni `reverses_entry_id` ne sont jamais
écrits.** Le numéro ne change pas (C-15-8-3) ; ⚠️ la date peut, dans l'exercice : l'ordre des numéros peut alors
cesser de suivre l'ordre des dates — c'est déjà le cas du complément de 25-7, et le manuel le dit (§ Numérotation).

⚠️ **Pourquoi le no-op vient APRÈS toutes les gardes** (héritage KF-004) : un `PUT` identique sur une écriture gelée
doit rendre le refus, pas un 200 trompeur. **Corollaire tranché** (C-15-8-11, révisé par C-15-8-18) : un `PUT`
identique sur une écriture dont un compte a été **archivé** rend **400 `INACTIVE_OR_INVALID_ACCOUNTS`** ; dont un
compte a été rendu **non imputable**, **400 `ACCOUNT_NOT_POSTABLE`** — jamais 200. L'AC 3 les exclut de son « `PUT`
identique → 200 », l'AC 4 les teste.

⚠️ **La précédence est figée et testée** (AC 7) : un exercice clos parle avant tout 409, puis un exercice
postérieur clos (C-15-8-22) ; une écriture contre-passée
répond `ENTRY_IS_REVERSED`, jamais un code de pièce ; le verrou de version vient **après** le gel (le motif réel prime
sur « rechargez ») ; le verrou de période parle **en dernier** des refus de date, comme à la création (`:306-326`).

### D5 — `DELETE` : inchangé dans cette story

Le `DELETE` garde son refus `409 ENTRY_IS_POSTED` (`delete_in_tx` étape 3-ter, `enforce_immutability`) jusqu'à la
15-8b, qui le fait passer sous la garde D2. ⚠️ **Ce que cette story ne fait donc PAS** : retirer
`DbError::EntryIsPosted`, la clé `journal-entries-blocked-posted`, la branche `kesh-api/src/errors.rs:3042-3050`, ni
toucher `delete_by_id` / `delete_in_tx`. Elle **réécrit** en revanche, dans le commentaire de la 24-4b resté au-dessus
du verrou de période (`kesh-api/src/errors.rs:2986-2996`), ce qui concerne le `PUT`, et elle dit dans le doc-comment
de la variante `EntryIsPosted` (`kesh-db/src/errors.rs:626-627`) qu'elle n'a plus qu'un émetteur, le `DELETE`, retiré
par la 15-8b.

⚠️ **Mais le MESSAGE de ce refus devient faux, et se réécrit ici** (C-15-8-27, finding F4 de la validation P3). La clé
`journal-entries-blocked-posted` dit aujourd'hui « Une écriture comptabilisée **ne se modifie plus**. Pour la corriger,
contre-passez-la » (fr `messages.ftl:356`, de/en/it équivalents) ; après cette story, le `DELETE` d'une écriture
modifiable répondrait qu'elle ne se modifie plus et conseillerait la contre-passation au lieu de la modification —
visible des clients d'API jusqu'au merge de la 15-8b. **Réécrits, dans les quatre locales** : la clé — FR « Une
écriture comptabilisée ne se supprime pas. Pour la corriger, modifiez-la tant que son exercice est ouvert, ou
contre-passez-la. » —, son repli Rust (`kesh-api/src/errors.rs:3045-3048`) et le `#[error]` de la variante
(`kesh-db/src/errors.rs:637` : « Écriture comptabilisée : suppression refusée »). La 15-8b retire le tout.

### D6 — L'écriture d'ouverture et le mode « Compléter » (25-7)

Vérifié au sol : **rien ne marque l'écriture d'ouverture.** C'est une OD datée du premier jour du premier exercice
(`opening_balances.rs`, `create_opening_entry` `journal_entries.rs:562`), qu'aucune table ne référence. Elle tombe donc
sous D1 comme toute écriture manuelle — **aucun traitement particulier n'est à écrire**, et c'est le résultat voulu.

Conséquences sur 25-7, toutes **sans code neuf**, à tester (AC 11) et à dire au manuel :

| geste | effet sur l'écran « Soldes de départ » |
|---|---|
| **modifier** l'ouverture (débit/crédit inversés) | aucun : la société garde ses écritures, l'écran reste en mode « compléter ». Le contrôle « aucune écriture » de la génération (`count_by_company`, `:495`) ne regarde que le **nombre** d'écritures. |
| modifier l'ouverture en **retirant** la ligne d'un compte | ce compte redevient « jamais mouvementé » → **complétable** (`opening_complement.rs`, même règle qu'après une dévalidation, manuel `:681`) |

*(La suppression de l'ouverture — et la génération rouverte, sous le numéro 2 — est la 15-8b.)*

Le **complément** de 25-7 (`opening_complement.rs`) est lui aussi une OD qu'aucune table ne référence : modifiable sous
D1, testé (AC 11).

⚠️ **Limite assumée, à dire au manuel** : faute de marqueur, rien n'empêche d'ajouter à l'écriture d'ouverture une
ligne de **résultat** (la génération le refuse, `D4` de 14-4) — elle est devenue une écriture manuelle comme une autre.

**Concurrence avec le complément** : la modification insère des lignes, donc prend le verrou **partagé** de clé
étrangère sur chaque compte visé — elle sérialise avec le complément, qui tient ces comptes en **exclusif** (25-7,
étape 2). C'est le cycle **exercice ↔ compte** de D4 : rejoué par `retry_with` des deux côtés (le complément l'est déjà,
`routes/opening_balances.rs:591`).

### D7 — Concurrence : le verrou optimiste `version` existant

`journal_entries.version` (`INT NOT NULL DEFAULT 1`) existe, n'est bougée par **aucun** autre chemin (vérifié :
`grep -rn "UPDATE journal_entries" crates` rend quatre lignes, **aucune** sur `version` — le `reverses_entry_id = NULL`
de `delete_all_by_company` (`journal_entries.rs:1204`) et trois `UPDATE` de montage dans des tests,
`period_lock_e2e.rs:409`, `reconciliation_e2e.rs:3816`, `reconciliation_cancel.rs:71` ; finding R3-7) et figure
déjà dans `JournalEntryResponse`. Le `PUT` l'exige ; périmée → `409 OPTIMISTIC_LOCK_CONFLICT`.

⚠️ **Corollaire utile** : `version > 1` ⇔ l'écriture a été modifiée (par Kesh ≥ 0.13, ou avant le gel). La 15-8b
l'affiche sur la fiche.

`docs/optimistic-locking-patterns.md` : la ligne 8 (`:49`) se **rétablit** comme **Variant B** (`update` verrouille
l'écriture par `SELECT … FOR UPDATE`, étape 1, puis contrôle `version`, étape 4 — pas de race), en le disant. ⚠️ La
puce « Variant A protégée » (`:75`) ne redevient **pas** vraie telle quelle : la retirer, ou la déplacer sous
Variant B (finding R2-17).

### D8 — L'écran : « Modifier » depuis la fiche

Choix C-15-8-7. **La liste ne change pas** (son lien vers la fiche, 24-4b D4, reste le seul geste de ligne).
*(« Supprimer », la mention « Modifiée » et le lien « Historique » sont la 15-8b.)*

**Détail (`GET /journal-entries/{id}`)** — `JournalEntryDetailResponse` (`routes/journal_entries.rs:129`) gagne trois
champs, sur le patron de `reversable` / `reversalBlockedBy` / `reversalBlockedLabel` :

- `modifiable: bool` ;
- `modificationBlockedBy: string | null` — un **code d'écran**, l'une de ces **onze** valeurs, dans **cet** ordre de
  précédence : `FISCAL_YEAR_CLOSED`, `LATER_FISCAL_YEAR_CLOSED` (C-15-8-22), `IS_A_REVERSAL`, `ALREADY_REVERSED`,
  `OWNED_BY_INVOICE`, `OWNED_BY_CREDIT_NOTE`, `OWNED_BY_SUPPLIER_INVOICE`, `OWNED_BY_SETTLEMENT`,
  `MATCHED_BANK_TRANSACTION`, `DETACHED_SUPPLIER_SETTLEMENT`, `PERIOD_LOCKED` — c'est l'ordre de D4 restreint aux refus
  qui ne dépendent pas du corps (étapes 2, 2-bis, 3 et 7, ancienne date), et, à l'étape 3, l'ordre de
  `modification_guard` ;
- `modificationBlockedLabel: string | null` — numéro de pièce (`document_label`, ou numéro de la facture fournisseur
  annulée), **nom de l'exercice postérieur clos** pour `LATER_FISCAL_YEAR_CLOSED`, ou la borne du verrou
  (`AAAA-MM-JJ`) pour `PERIOD_LOCKED`, `null` sinon.

**Deux fonctions, nommées, une seule logique** (C-15-8-13) :

| fonction | rôle | appelée par | rend |
|---|---|---|---|
| `modification_guard(conn: &mut MySqlConnection, company_id, id)` | **garde d'écriture** — D2 seule : premier motif de `reversal_blockers` hors `AccountArchived`, puis le paiement détaché | `update` (étape 3, `&mut **tx`), `modification_blocker` (sa connexion) ; `delete_in_tx` à la 15-8b | `Result<Option<ModificationGuard>, DbError>` |
| `modification_blocker(pool: &MySqlPool, company_id, id)` | **motif d'écran** — exercice clos, puis exercice postérieur clos, puis `modification_guard`, puis verrou de période sur la date **présente** | `get_journal_entry` (sur le pool, comme `reversal_blocker` `:435`) | `Result<Option<ModificationBlocker>, DbError>` |

⚠️ **Signatures** (C-15-8-24, finding R3-1 de la validation P3 — la correction de R2-15 avait déplacé le défaut sur
`modification_guard` au lieu de le corriger) : `modification_blocker` prend `&MySqlPool` et fait **`let mut conn =
pool.acquire().await?`** au début ; elle enchaîne **cinq** lectures sur `&mut *conn` — l'exercice de l'écriture,
l'exercice postérieur clos, `modification_guard` (qui en fait deux), la borne. Un `Executor` pris par valeur ne sert
qu'une fois ; une connexion se reprête. Pour l'exercice postérieur clos, une lecture **non verrouillante** :
`fiscal_years::find_later_closed(conn: &mut MySqlConnection, company_id, start_date)`, **sœur** de
`find_later_closed_in_tx` — ⛔ **DRY** : le texte du `SELECT` est une constante privée partagée, la version `_in_tx` y
ajoute `FOR UPDATE` ; pas de second `SELECT` écrit à la main.

`ModificationBlocker` (`crates/kesh-db/src/errors.rs`) :

```rust
pub enum ModificationBlocker {
    FiscalYearClosed,
    LaterFiscalYearClosed { fiscal_year_name: String },
    Guard(ModificationGuard),
    PeriodLocked { locked_through: NaiveDate },
}
impl ModificationBlocker {
    pub fn code(&self) -> &'static str { /* FISCAL_YEAR_CLOSED | LATER_FISCAL_YEAR_CLOSED | guard.code() | PERIOD_LOCKED */ }
    pub fn label(&self) -> Option<String> { /* None | fiscal_year_name | document_label ou numéro de facture | locked_through */ }
}
```

Le motif d'écran ne voit que l'état **présent** : la nouvelle date d'un `PUT`, le corps et la version restent
contrôlés à l'écriture seule.

**Table de correspondance — code d'écran ↔ refus d'écriture** (assertion de l'AC 12, écrite **en dur** dans le test,
pas recalculée ; la 15-8b y ajoute la colonne `DELETE`) :

| `modificationBlockedBy` | `PUT` rend |
|---|---|
| `FISCAL_YEAR_CLOSED` | 400 `FISCAL_YEAR_CLOSED` |
| `LATER_FISCAL_YEAR_CLOSED` | 400 `LATER_FISCAL_YEAR_CLOSED` |
| `IS_A_REVERSAL` | 409 `IS_A_REVERSAL` |
| `ALREADY_REVERSED` | 409 **`ENTRY_IS_REVERSED`** — ⚠️ **le seul écart de nom**, voulu : l'écran réutilise le vocabulaire de `ReversalBlocker` (et sa clé `journal-entries-reverse-blocked-already-reversed`), l'écriture garde le code d'erreur existant de la 24-4a |
| `OWNED_BY_INVOICE`, `OWNED_BY_CREDIT_NOTE`, `OWNED_BY_SUPPLIER_INVOICE`, `OWNED_BY_SETTLEMENT`, `MATCHED_BANK_TRANSACTION`, `DETACHED_SUPPLIER_SETTLEMENT` | 409 sous le **même** code |
| `PERIOD_LOCKED` | 400 `PERIOD_LOCKED` (ancienne date) |
| `null` (modifiable) | `PUT` identique avec la bonne version → 200 ; ou 400 `INACTIVE_OR_INVALID_ACCOUNTS` si un compte est archivé ; ou 400 `ACCOUNT_NOT_POSTABLE` s'il est devenu non imputable (C-15-8-18 — ni l'un ni l'autre ne rend l'écriture non modifiable) |

**Fiche (`frontend/src/routes/(app)/journal-entries/[id]/+page.svelte`)** :

- bouton **« Modifier »** (`data-testid="edit-entry"`) à côté de « Contre-passer » — ⛔ **absent, pas grisé**, quand
  `modifiable` est faux (convention de la 24-4a), le motif traduit affiché à sa place
  (`data-testid="modification-blocked-reason"`) ; ⚠️ si le motif de contre-passation et celui de modification sont
  **le même code**, ne l'afficher **qu'une fois** ;
- ⛔ **rôle Consultation** (C-15-8-14) : « Modifier » **et** « Contre-passer » (aujourd'hui affiché à ce rôle,
  `[id]/+page.svelte:236-243`, et qui rend 403 au clic) sont **absents**, sans motif affiché. Rôle lu dans `authState`
  (`$lib/app/stores/auth.svelte.ts`). Le 403 du serveur reste le refus qui fait autorité ;
- motifs traduits par un `switch` **exhaustif** typé (`_exhaustif: never`, et chaîne vide au `default` — cf. la leçon
  écrite dans `blockedLabel`, `[id]/+page.svelte`) ; type `ModificationBlocker` (union des onze codes) dans
  `journal-entries.types.ts` ; ⛔ **réutiliser** les clés `journal-entries-reverse-blocked-*` existantes pour les sept
  codes communs, n'ajouter que `journal-entries-modify-blocked-fiscal-year-closed`,
  `journal-entries-modify-blocked-later-fiscal-year-closed` (`{ $name }`, aussi lue par le serveur, D4),
  `journal-entries-modify-blocked-period-locked` (`{ $date }`) et `journal-entries-modify-blocked-detached-settlement`
  (aussi lue par le serveur, D2) ;
- **« Modifier »** remplace la vue par `JournalEntryForm` en **mode édition**, pré-rempli ; enregistrer → toast
  `journal-entry-saved` (clé existante), la fiche se recharge (même `id`, même numéro) ; annuler → retour à la vue.

**Formulaire (`JournalEntryForm.svelte`)** — le mode édition est **rétabli par inversion du commit du gel**, pas par
copie de l'état `d2910022` (C-15-8-15) :

```sh
git show 08e20353 -- <fichier> | git apply -R --3way   # 08e20353 = feat(24-4b), le gel (squash de la PR #422)
```

| fichier | inversion | après |
|---|---|---|
| `src/lib/features/journal-entries/form-helpers.ts` | s'applique net (`git apply -R --check`, vérifié le 2026-10-08) | — |
| `src/lib/features/journal-entries/form-helpers.test.ts` (7 tests) | net — le fichier renaît | — |
| `src/lib/features/journal-entries/journal-entries.api.ts` | net — `updateJournalEntry` et `deleteJournalEntry` reviennent | ⚠️ `deleteJournalEntry` revient avec l'inversion : **le garder** (la 15-8b l'emploie), sans appelant dans cette story ; le commentaire de fin de fichier tombe avec |
| `src/lib/features/journal-entries/journal-entries.types.ts` | net — `UpdateJournalEntryRequest` revient | y ajouter les trois champs du détail, `ModificationBlocker` |
| `src/lib/features/journal-entries/JournalEntryForm.svelte` | ⚠️ **conflits** à résoudre à la main (`--3way`) : la 24-4c (`54ae4a70`) a ajouté la prop `booksLockedThrough` et le `min` de la date (`minEntryDate`) ; la 15-5a a ajouté `case 'ACCOUNT_NOT_POSTABLE'` | **garder** les ajouts de la 24-4c et de la 15-5a ; **reprendre** `initialEntry`, `isEdit`, `version`, le pré-remplissage par `fromJournalEntryResponse`, la branche `updateJournalEntry`, et `{#if !isEdit}` autour de `VatPurchaseAssistant` (C-15-8-21 : l'assistant compose une écriture d'achat neuve, il reste réservé à la création) ; ⛔ **ÉCARTER** du bloc inversé **la modale de conflit** (C-15-8-21) : `let showConflictDialog`, le `case 'OPTIMISTIC_LOCK_CONFLICT'` qui l'ouvre, la prop `onConflictReload`, `handleConflictReload`, le bloc `<!-- Modale de conflit de version 409 (story 3.3) -->` et ses quatre usages de clés (`journal-entry-conflict-{title,message,reload}`, troisième site de `journal-entry-form-cancel`) — D8 veut un toast puis un rechargement, voir ci-dessous |
| `src/lib/shared/e2e-selecteurs-traduits.test.ts` | ⚠️ **pas d'inversion aveugle** : les 2 lignes retirées (`journal-entries.spec.ts :: Annuler`, `:: Supprimer`) inscrivaient les sélecteurs traduits des specs de **liste** | y inscrire les sélecteurs traduits que les **nouvelles** specs (T7) emploient réellement, ni plus ni moins |
| `src/routes/(app)/journal-entries/+page.svelte` | ⛔ **NE PAS inverser** : le mode édition de `d2910022` vivait dans la **liste** (✎, 🗑, modale de conflit), que C-15-8-7 écarte | lire l'ancien état comme **modèle** du chargement des props |
| `tests/e2e/journal-entries.spec.ts`, `src/lib/shared/i18n-keys.test.ts` | ⛔ **NE PAS inverser** : parcours de liste ; `i18n-keys.test.ts` a bougé cinq fois depuis (25-4-d1 à 25-7) | réécrits (T7, T6) |

⚠️ `frontend/scripts/lint-i18n-ownership.js:87-89` inscrit encore à `KNOWN_VIOLATIONS` les trois entrées
`JournalEntryForm.svelte:journal-entry-conflict-*`, périmées depuis la 24-4b et qui ne reviennent pas : **les retirer**
(finding F4).

Mode édition, **spécifié** (ce qui n'existait pas dans `d2910022`) :

- **rechargement** : une prop neuve `onStale?: () => void`, appelée **après** le toast par les trois branches
  ci-dessous qui exigent de relire la fiche (`FISCAL_YEAR_CLOSED` et `LATER_FISCAL_YEAR_CLOSED`, 409 de course,
  `OPTIMISTIC_LOCK_CONFLICT`) ; la fiche
  la câble sur son rechargement. Elle **remplace** `onConflictReload`, qui ne revient pas ;
- **dates** : `min` = le plus tardif de `fy.startDate` et du lendemain de `booksLockedThrough` ; `max` = `fy.endDate`,
  où `fy` est l'**exercice de l'écriture** (`entry.fiscalYearId`, cherché dans `listFiscalYears()`,
  `fiscal-years.api.ts:13`, chargé au clic sur « Modifier »). Prop neuve `entryFiscalYear?: { startDate: string;
  endDate: string } | null`, nulle en création (où `min` reste celui de la 24-4c). **Confort de saisie seulement** : le
  refus qui fait autorité reste le 400 du serveur (`DATE_OUTSIDE_FISCAL_YEAR`, `PERIOD_LOCKED`) ;
- **`FISCAL_YEAR_CLOSED` en édition** : ⛔ **ne pas** passer par `notifyMissingFiscalYearOrFallback`
  (`shared/utils/notify.ts:98-123`), dont le message — « L'exercice qui couvre cette date est clôturé. Vérifiez la date
  saisie » avec un bouton vers les paramètres — est faux ici : c'est l'exercice **de l'écriture** qui a été clôturé
  entre-temps. Toast `journal-entries-modify-blocked-fiscal-year-closed` (la clé du motif d'écran, réutilisée), puis
  `onStale` (la fiche rend alors le motif à la place du bouton) ;
- **`LATER_FISCAL_YEAR_CLOSED` en édition** (C-15-8-22) : même traitement — toast du message serveur (déjà traduit, il
  nomme l'exercice), puis `onStale` ; une branche `case` nommée ;
- **409 de course** (`ENTRY_IS_REVERSED`, `IS_A_REVERSAL`, `OWNED_BY_*`, `MATCHED_BANK_TRANSACTION`,
  `DETACHED_SUPPLIER_SETTLEMENT`) : toast du message serveur (déjà traduit), puis `onStale` ; une branche `case` nommée
  par code, pas le `default` du `switch` (`JournalEntryForm.svelte:191`) ;
- **`OPTIMISTIC_LOCK_CONFLICT`** : toast `journal-entries-edit-conflict` (« Cette écriture a été modifiée entre-temps :
  la fiche a été rechargée. »), puis `onStale` — **pas de modale** ;
- **`PERIOD_LOCKED`, `DATE_OUTSIDE_FISCAL_YEAR`, `ENTRY_UNBALANCED`, `INACTIVE_OR_INVALID_ACCOUNTS`,
  `ACCOUNT_NOT_POSTABLE`, `VALIDATION_ERROR`** : le formulaire reste ouvert, toast du message serveur — l'utilisateur
  corrige sa saisie. Les `case` existants sont `ENTRY_UNBALANCED`, `DATE_OUTSIDE_FISCAL_YEAR`,
  `INACTIVE_OR_INVALID_ACCOUNTS`, `VALIDATION_ERROR` (`JournalEntryForm.svelte:176-179`) et, après la 15-5a,
  `ACCOUNT_NOT_POSTABLE` ; **`PERIOD_LOCKED` y est ajouté** — il tombe aujourd'hui au `default`, au même effet, mais
  le code est nommé (findings R2-7, F9) ;
- ⚠️ une ligne pré-remplie sur un compte **archivé ou non imputable** n'est pas sélectionnable dans
  `AccountAutocomplete` : l'afficher avec son numéro et son nom (la fiche charge `fetchAccounts(true)`) et un
  avertissement `journal-entries-line-account-unusable` (« Compte archivé ou non imputable — à remplacer »,
  `data-testid="line-account-unusable"`) — le refus faisant autorité reste le 400 du serveur ;
- les props de la fiche (`booksLockedThrough`, `recoverableAccountId`, comptes **actifs** pour la sélection, projets,
  `entryFiscalYear`) se chargent au clic sur « Modifier », comme la page de liste les charge (`+page.svelte:95-161`).

**Clés i18n neuves de cette story** — quatre locales, nommées ici pour qu'aucune ne s'invente au développement :
`journal-entry-edit` (bouton de la fiche — une **route**, hors du contrôle de `lint-i18n-ownership`) ;
`journal-entries-edit-conflict` et `journal-entries-line-account-unusable` (employées par `JournalEntryForm.svelte`,
sous `features/journal-entries/` : ⛔ le préfixe **`journal-entries-`** est celui qu'exige `keyBelongsToFeature`,
`lint-i18n-ownership.js:178-187` — un nom en `journal-entry-` rougirait au gate ou agrandirait la dette #30, finding
F8) ; `journal-entries-modify-blocked-fiscal-year-closed`, `journal-entries-modify-blocked-later-fiscal-year-closed`,
`journal-entries-modify-blocked-period-locked`, `journal-entries-modify-blocked-detached-settlement`. Les quatre clés de conflit de `d2910022`
(`journal-entry-conflict-{title,message,reload,reloaded}`) **ne reviennent pas** (plus de modale).

### D9 — Rapports, export complet, `.keshbackup` : aucune incohérence introduite — vérifié

- **Rapports** : calculés à la volée depuis les lignes. Modifier une écriture d'une période **non verrouillée**
  change un rapport déjà tiré : **c'est le sens même de la CR**, et le verrou de période (24-4c) est l'outil qui fige
  une période déclarée. Le manuel doit le dire en toutes lettres (§ verrou et § modification).
- **Export ZIP de souveraineté** : `journal_entries.csv` porte `version` et `updated_at` (`csv_tables.rs:301-346`) ;
  `audit_log.csv` porte les instantanés avant/après (`exports/global.rs:61`, `:151`). Une modification donne donc
  l'état présent **et** sa trace. Aucun changement de format.
- **`.keshbackup`** : `audit_log` y est (`backup.rs:64`) ; aucune migration, aucune colonne neuve → inventaire de tables
  inchangé, compatibilité des sauvegardes intacte. **P1–P8 muets** (aucun fichier de migration). ⛔ P8 reste un piège
  de grep — voir « À NE PAS toucher » (deux migrations publiées citent `journal_entries::update`).

### D10 — Les clés API

Le `PUT` reste dans `comptable_routes` (`lib.rs:366-368`), donc ouvert aux clés `read-write` (C-15-8-8). La trace
d'audit porte la clé (`for_actor`, D4 ; AC 3).
À écrire dans `docs/api-external.md` : § 7 (tableau `:212`, ajouter `PUT /journal-entries/{id}`), une sous-section
« Modifier une écriture » (corps, réponse, **tableau des refus dans l'ordre de précédence de D4**, avec son propre
décompte de lignes s'il reprend le gabarit du paragraphe `:247`) ; et ajouter `PUT /journal-entries/{id}` à la liste
des routes qui rendent `ACCOUNT_NOT_POSTABLE` (ligne posée par la 15-5a, tableau des codes d'erreur, `:415` après
rebase) ; et deux lignes neuves au même tableau : `LATER_FISCAL_YEAR_CLOSED` (400, C-15-8-22) et
`DETACHED_SUPPLIER_SETTLEMENT` (409, C-15-8-20). ⚠️
`ENTRY_IS_POSTED` n'a **jamais** figuré dans `docs/api-external.md` (`grep -c ENTRY_IS_POSTED docs/api-external.md` →
`0`) : le changement de contrat se dit au `CHANGELOG.md` seulement (finding R2-13). Manuel administrateur : liste
« ce qu'une clé `read-write` peut faire » (`admin-manual.tex:1796`).

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
   `read-write` écrit `actor_type = 'ApiKey'` et `actor_api_key_id` = l'id de la clé (test dédié). Un `PUT` identique à
   l'état présent, sur une écriture qui passe **toutes** les gardes → 200, **aucune** entrée, `version` inchangée — ⚠️
   un `PUT` identique sur une écriture portant un compte archivé ou non imputable n'est **pas** un no-op : il rend le
   400 de l'AC 4 (C-15-8-18).
4. **Mêmes contrôles qu'à la saisie** : déséquilibre → 400 `ENTRY_UNBALANCED` ; compte **inconnu, d'une autre société
   ou archivé** — **y compris sur une ligne inchangée** → 400 `INACTIVE_OR_INVALID_ACCOUNTS` ; compte **non imputable**
   — **y compris sur une ligne inchangée** → 400 `ACCOUNT_NOT_POSTABLE` dont `details.rejected` nomme le compte ; un
   compte archivé sur une ligne **et** un non imputable sur une autre → `INACTIVE_OR_INVALID_ACCOUNTS` (ordre de la
   15-5a) ; nouvelle date hors de l'exercice **de l'écriture**, même si un autre exercice ouvert la couvre → 400
   `DATE_OUTSIDE_FISCAL_YEAR` ; et rien n'a changé en base. ⚠️ Le *grandfathering* de postabilité de l'ancien `update`
   est **inversé**, pas rétabli : `test_update_grandfathers_non_postable_by_account` (`d2910022` `:3762`) devient
   `test_update_refuses_a_line_on_an_account_made_non_postable` — édition du seul libellé, puis ajout d'une ligne sur le
   même compte, chacun → `DbError::AccountsNotPostable` nommant le compte. Les projets archivés déjà présents restent
   tolérés (C-15-8-4). Un `PUT` **identique** sur une écriture dont un compte a été archivé → 400
   `INACTIVE_OR_INVALID_ACCOUNTS` (`update_no_op_with_inactive_account_returns_inactive_error`, rétabli) ; dont un compte
   a été rendu non imputable → 400 `ACCOUNT_NOT_POSTABLE` (`update_no_op_with_non_postable_account_returns_not_postable`,
   neuf).
5. **Période verrouillée et exercice clos** : ancienne date ≤ borne → 400 `PERIOD_LOCKED` ; ancienne date libre mais
   **nouvelle** date ≤ borne → 400 `PERIOD_LOCKED`. Seuil inclusif testé (borne = date). **Exercice clos seul**
   (aucune autre cause) : `PUT` → 400 `FISCAL_YEAR_CLOSED`, par un test qui ne monte que cette cause
   (`update_no_op_in_closed_fy_returns_fiscal_year_closed` rétabli au repository). **Exercice postérieur clos**
   (C-15-8-22) : écriture de N, N ouvert, N+1 **clos** (clôturé directement, ce que `fiscal_years::close` autorise) →
   400 `LATER_FISCAL_YEAR_CLOSED`, `details.fiscalYearName` = le nom de N+1, rien n'a changé ; avec N+1 et N+2 clos,
   c'est N+1 (le plus proche) qui est nommé ; après réouverture de N+1 (et de N+2), le même `PUT` → 200. Un exercice
   **antérieur** clos ne gêne pas (N−1 clos, N ouvert → 200).
6. **Écritures gelées** : pour **chacune** des pièces — facture, avoir, facture fournisseur (achat **et** règlement),
   règlement client, **solde** (`write_off`), transaction bancaire rapprochée — `PUT` rend **409 sous le code du
   motif**, avec `details.documentId` ; une écriture contre-passée → **409 `ENTRY_IS_REVERSED`** ; une
   contre-passation → **409 `IS_A_REVERSAL`**. ⚠️ Réutiliser le montage de `every_document_owned_entry_is_refused`
   (`journal_entry_reversal_e2e.rs:485`), ne pas en réinventer un — **et l'étendre** : son doc annonce « les six
   chemins » (`:478`) et il n'a **pas** de cas `write_off` ; lui ajouter un solde (`invoice_settlements_write.rs`, chemin
   du solde `:497-524`) et mettre son doc à jour (finding R2-11). **Paiement détaché** (C-15-8-20) : une facture
   fournisseur **payée** puis **annulée** — son écriture de règlement, que plus aucune colonne ne référence, rend au
   `PUT` **409 `DETACHED_SUPPLIER_SETTLEMENT`**, `details.documentId` = l'id de la facture ; et elle reste
   **contre-passable** (`POST …/reverse` → 201), comme le manuel le promet.
7. **Précédence** — chaque paire testée en montant **les deux** causes : exercice clos **et** contre-passée → 400 ;
   exercice clos **et** exercice postérieur clos → `FISCAL_YEAR_CLOSED` ; exercice postérieur clos **et** contre-passée
   → `LATER_FISCAL_YEAR_CLOSED` ; exercice postérieur clos **et** pièce → `LATER_FISCAL_YEAR_CLOSED` ;
   contre-passée **et** version périmée → `ENTRY_IS_REVERSED` ; pièce **et** version périmée → code de la pièce ;
   pièce **et** période verrouillée → code de la pièce ; version périmée **et** déséquilibre… ⚠️ le déséquilibre est un
   400 de **forme**, rendu par le handler avant la base : ce cas se teste comme « 400 avant 404 » (AC 10).
8. **Concurrence** : `PUT` avec une `version` périmée → 409 `OPTIMISTIC_LOCK_CONFLICT`, rien n'a changé.
   **Sérialisation (D2)** — test à deux connexions, `crates/kesh-db/tests/journal_entries_modification.rs`,
   `update_waits_for_a_concurrent_reversal_then_refuses` : la connexion B ouvre une transaction et contre-passe
   l'écriture E par `reverse_in_tx` **sans commiter** ; la tâche A lance `update(E)` ; le test attend de voir A
   bloquée sur son verrou (`test_fixtures::attendre_une_requete_en_cours`, `test_fixtures.rs:560`, motifs tirés du
   texte du `SELECT` de l'étape 1, p. ex. `["je.version", "FOR UPDATE"]`) ; B commite ; A doit rendre
   `EntryIsReversed` et E est inchangée. ⛔ **Mutation à tuer** (à exécuter une fois, déclarée au Dev Agent Record) :
   placer une lecture ordinaire (la borne `books_locked_through`) **avant** le `FOR UPDATE` de l'étape 1 fait passer
   A (au `PUT`, rien ne s'y oppose ensuite : aucune clé étrangère ne protège la réécriture des lignes — contrairement
   au `DELETE`, qui bute sur `RESTRICT`, 15-8b AC 5) — le test doit rougir. *(La seconde mutation de la rédaction P2 — la lecture des projets en `LOCK IN SHARE MODE` —
   tombe avec ce verrou, C-15-8-23 : la borne lue dans la vue ouverte à 1-bis peut être périmée, tolérance assumée
   comme à la création, D2.)* **Exercice postérieur clos, sous concurrence** (C-15-8-22) —
   `update_waits_for_a_concurrent_close_of_a_later_year_then_refuses` : B ouvre une transaction et passe N+1 à
   `Closed` (`UPDATE fiscal_years SET status = 'Closed' WHERE id = ?`) **sans commiter** ; A lance `update(E)`, E dans
   N ; le test attend A bloquée (motifs du `SELECT` de `find_later_closed_in_tx`, p. ex. `["start_date > ?",
   "FOR UPDATE"]`) ; B commite ; A rend `LaterFiscalYearClosed` nommant N+1 et E est inchangée. ⛔ **Mutation à
   tuer**, même déclaration : lire les exercices postérieurs par une lecture **ordinaire** (`find_later_closed` au lieu
   de `find_later_closed_in_tx`) — A ne bloque plus, lit la vue d'avant le commit de B, et le test rougit.
9. **Rejeu sur interblocage** (C-15-8-19) : le handler `PUT` est enveloppé dans `retry_with` sur `is_deadlock_error`
   (revue de code : `grep -nF "retry_with" crates/kesh-api/src/routes/journal_entries.rs`). **Le cycle est réel, pas
   décoratif** — test à deux connexions `update_and_a_reversal_of_the_same_year_can_deadlock` : A lance `update(E1)`
   avec un projet **nouveau** P ; B tient l'exercice de E1 (`SELECT … FROM fiscal_years … FOR UPDATE`) ; le test attend
   A bloquée sur l'exercice (motifs du `SELECT` de l'étape 1-ter) ; B demande P en partagé
   (`SELECT id FROM projects WHERE id = ? LOCK IN SHARE MODE`) ; l'une des deux reçoit une erreur pour laquelle
   `is_deadlock_error` est vrai. *(Le rejeu lui-même est le helper existant, testé dans `retry.rs`.)* Pattern 5 porte
   la ligne du `PUT` (`docs/MULTI-TENANT-SCOPING-PATTERNS.md`).
10. **IDOR et rôles** : `PUT` sur un `id` d'une autre société ou inexistant → **404**, jamais 409 ; Consultation →
    **403** avant tout autre contrôle ; un corps malformé → 400/422 de l'extracteur, sans lecture de la base.
11. **Soldes de départ** (D6) : après **modification** de l'ouverture, `GET /opening-balances/status` rend toujours
    `ALREADY_HAS_ENTRIES` ; après modification qui **retire** la ligne d'un compte, ce compte figure dans
    `completableAccounts`. Une écriture de **complément** (25-7) se modifie comme l'ouverture (200).
12. **Détail** : `GET /journal-entries/{id}` rend `modifiable` / `modificationBlockedBy` / `modificationBlockedLabel`.
    Pour **chacun** des onze codes d'écran (D8), un montage qui ne porte que cette cause : le `GET` rend ce code, et le
    `PUT` (corps identique, bonne version) rend le refus que la **table de correspondance** de D8 associe à ce code —
    table écrite en dur dans le test (`ALREADY_REVERSED` ↔ `ENTRY_IS_REVERSED` est le seul écart de nom). Un compte
    archivé ou non imputable ne rend **pas** l'écriture non modifiable (`modifiable = true`, et le `PUT` identique rend
    le 400 de l'AC 4).
13. **Garde-fou d'inventaire** (D3) : le test existe, passe, et ⛔ **ses mutations sont tuées** — ajouter en test une
    colonne ou une clé étrangère factice vers `journal_entries`, puis une clé étrangère factice vers
    `journal_entry_lines`, le fait rougir chaque fois (à exécuter une fois, déclarer au Dev Agent Record).
14. **Écran** : depuis la fiche, « Modifier » ouvre le formulaire pré-rempli ; enregistrer met à jour la fiche (même
    numéro) ; le bouton est **absent** avec le motif quand l'écriture n'est pas modifiable ; un conflit de version rend
    un toast et recharge la fiche, **sans modale** ; au rôle **Consultation**, « Modifier » et « Contre-passer » sont
    absents (C-15-8-14). Couvert par **Playwright** (le seul test qui voit la valeur traverser la frontière HTTP) —
    specs nommées en T7.
15. **Ce qui reste du gel** : le `PUT` ne rend plus jamais `ENTRY_IS_POSTED` ; `DbError::EntryIsPosted` n'a plus
    **qu'un** émetteur, `delete_in_tx` étape 3-ter, et son doc-comment le dit, en renvoyant à la 15-8b. ⛔ **Sortie
    attendue, liste fermée** (findings R3-5, F5 — numéros **après rebase**, relevés le 2026-10-08) de
    `grep -rn "EntryIsPosted" crates/*/src` : `kesh-db/src/errors.rs:753` (variante) et `:890` (bras de `code()`) ;
    `kesh-api/src/errors.rs:3042` (mappage) ; `kesh-db/src/repositories/journal_entries.rs:1066` (l'émetteur) ; les
    doc-comments de `delete_by_id` / `delete_in_tx` (`:941`, `:951`, `:991`) ; le test
    `le_gel_parle_avant_le_verrou_de_periode` — son doc `:2516`, son assertion `:2522-2523` (réécrit par la 15-8b).
    **Dix** lignes, à numéros près (onze aujourd'hui, `grep -c` exécuté le 2026-10-08) ;
    l'émetteur du `PUT` (`kesh-api/src/routes/journal_entries.rs:605`) a **disparu**. Toute autre ligne est un résidu.
16. **Audit lisible** : `journal_entry.updated` revient dans `ACTIONS` (`audit_labels.rs:130-132`) avec sa clé
    `audit-log-action-journal-entry-updated` dans les **quatre** locales (la garde `tests/audit_label_registry.rs`
    l'exige) ; `audit_route_registry.rs:82` passe le `PUT` à `Traced`.
17. **Documentation** : manuel utilisateur et administrateur, `docs/api-external.md`,
    `docs/optimistic-locking-patterns.md`, `docs/MULTI-TENANT-SCOPING-PATTERNS.md` et `CHANGELOG.md` disent le nouveau
    cadre **de la modification** (cf. Dev Notes) ; PDF régénérés. *(`README.md:29` est édité par la **15-8b seule**,
    C-15-8-26.)* ⛔ **Contrôle** (C-15-8-27, findings R3-6, F6, F9, F4) — motif :

    ```sh
    M="ne se modifie plus|ne se modifie ni|ni modifiable ni supprimable|ni modifiée ni supprimée|depuis la version qui introduit le gel|antérieures au gel|contre-passation est imposée|est définitive et se corrige|refusées en permanence|ni modification ni suppression|elle est imposée|aucune mise à jour ne peut plus|seul chemin|passent par (la )?contre-passation|seule voie de correction"
    for f in docs/manual/fr/user-manual.pdf docs/manual/fr/admin-manual.pdf; do
      pdftotext "$f" - | tr '\n' ' ' | tr -s ' ' | grep -oiE "$M"; done        # FAIT FOI
    grep -niE "$M" crates/kesh-i18n/locales/fr-CH/messages.ftl                   # le catalogue aussi
    ```

    rend **rien**, hors passages volontairement conservés et nommés au Dev Agent Record. **Le contrôle fait foi sur les
    PDF aplatis**, pas sur le `.tex` : le balisage TeX y coupe les tournures (`La contre-passation est
    \textbf{imposée}`, `user-manual.tex:554` ; `est \textbf{définitive} et se corrige`, `admin-manual.tex:1969`), si
    bien qu'un `grep` sur le `.tex` passe à tort ; et `grep -o` compte les **occurrences**, là où `grep -c` sur une
    ligne aplatie ne rend que 0 ou 1. **Ligne de base mesurée** sur les PDF commités, après rebase (2026-10-08) :
    `user-manual.pdf` **11** occurrences, `admin-manual.pdf` **5**, `fr-CH/messages.ftl` **2** lignes
    (`journal-entries-blocked-posted`, réécrite par D5 ; `opening-balances-complete-confirm`, réécrite par le tableau
    i18n). Les trois autres locales ne se grepent pas en français : les clés du tableau i18n (Dev Notes) y sont **relues
    à la main**, et dites au Dev Agent Record. **Revus à la main** (le motif ne les prend pas) : `user-manual.tex:681`
    (« on le corrige … en contre-passant ») et `admin-manual.tex:2004` s'il est reformulé hors motif. ⚠️ **La tournure
    prescrite pour le paiement détaché** (`user-manual.tex:1331-1337`, FAQ `:2140`) est choisie pour ne heurter aucun
    motif, ni celui-ci ni celui de la 15-8b : « La fiche de ce paiement n'offre pas « Modifier » : elle affiche le
    motif, et la contre-passation reste disponible. » La table des matières porte le nouveau titre de la section
    renommée. ⚠️ Le motif est **large à dessein** (règle « Inventorier les sites NON RÉSOLUS ») ; les tournures de la
    **suppression** (« ne se supprime plus », « une seule voie fait encore disparaître ») restent vraies jusqu'à la 15-8b
    et ne sont **pas** dans ce motif — c'est la 15-8b qui les ferme.
18. **Export** : après une modification, l'export ZIP de souveraineté porte l'état **présent** de l'écriture
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
- **I3 — Aucune écriture d'une pièce ne bouge.** Après tout `PUT` refusé, lignes et en-tête identiques (comptes,
  montants, ordre, projets, `version`).
- **I4 — La correction reste possible.** Une écriture modifiable reste contre-passable (si aucun compte archivé) : les
  deux voies coexistent, la modification ne retire pas la contre-passation.

## Tâches

- [ ] **T0 — Rebase et inventaire au sol** (AC 15, 17)
  - [x] rebaser la branche sur `main` après le merge de la 15-5a — **fait** le 2026-10-08 (validation P3), sur
        `b11a074a` (finding R3-8)
  - [ ] revérifier **tous** les numéros de ligne cités par cette fiche, relevés **avant** le rebase. **Décalages
        mesurés** au rebase (correspondance ligne à ligne `c98f4017` → `HEAD`, `difflib`, 2026-10-08) — ⚠️ à
        **ajouter** aux numéros de la fiche, sauf mention « après rebase » :

        | fichier | décalage |
        |---|---|
        | `kesh-db/src/repositories/journal_entries.rs` | `0` jusqu'à `:65` ; **`+10`** à partir de `:82` (`validate_lines_accounts_in_tx` `:84` → `:94`, `delete_in_tx` `:1005` → `:1015`, `reversal_blockers` `:1265` → `:1275`, `reverse_in_tx_inner` `:1518` → `:1528`, `mod tests` `:3439` → `:3449`) |
        | `kesh-db/src/errors.rs` | **`+115`** pour les variantes citées (`:623` → `:738`, `:626-627` → `:741-742`, `:637-638` → `:752-753`, `:647` → `:762`) ; **`+122`** pour `code()` (`:757-759` → `:879-881`, `:768` → `:890`) |
        | `kesh-db/src/repositories/supplier_invoices.rs` | **`+31`** (`cancel_in_tx` `:851` → `:882`, détachement `:905-912` → `:936-943`, audit `:931-935` → `:962-966`, `:1120` → `:1151`) |
        | `kesh-db/src/repositories/invoice_settlements_write.rs` | `:157` → `:158` (ligne modifiée par la 15-5a) ; **`+10`** ensuite (`:214` → `:224`, `:497-524` → `:507-534`) |
        | `kesh-api/src/routes/opening_balances.rs` | **`+3`** (`:581-610` → `:584-613`, `:591` → `:594`) |
        | `kesh-i18n/locales/fr-CH/messages.ftl` | **`+4`** (`:355-356` → `:359-360`, `:929` → `:933`, `:962` → `:966`, `:979` → `:983`) |
        | `JournalEntryForm.svelte` | `0` jusqu'à `:176` ; `+2` ensuite (`:179` → `:181`, `:191` → `:193`) |
        | `kesh-api/src/errors.rs`, `kesh-api/src/routes/journal_entries.rs`, `kesh-api/tests/journal_entry_reversal_e2e.rs`, manuels | **inchangés** aux lignes citées |
  - [ ] exécuter les greps des Dev Notes (« Les sites qui supposent aujourd'hui l'immuabilité ») **et** les deux
        greps des sentinelles de D4 (dont `grep -rn validate_taggable_in_tx crates/*/src`, finding R3-10) **avant** toute
        écriture de code ; relever leur **sortie complète** au Dev Agent Record, triée ligne par ligne (site → tableau
        de la fiche, site conservé avec sa raison, ou site neuf) — pas seulement l'écart aux tableaux. Un site absent
        des tableaux s'y ajoute, il ne s'ignore pas.
- [ ] **T1 — Erreurs** (AC 6, 15)
  - [ ] `ModificationGuard` (`code()`), `DbError::EntryNotModifiable(ModificationGuard)` + repli `code()`
        `"ENTRY_NOT_MODIFIABLE"` ; `ModificationBlocker` (`code()`, `label()`, D8 — dont `LaterFiscalYearClosed`) ;
        `DbError::LaterFiscalYearClosed { fiscal_year_id, fiscal_year_name }`, `code()` `"LATER_FISCAL_YEAR_CLOSED"`
        (D4, C-15-8-22) ; doc de `EntryIsPosted` : un seul émetteur restant (D5)
  - [ ] `kesh-api/src/errors.rs` : mappage partagé avec `EntryNotReversable` (fonction extraite) ; branche
        `DetachedSupplierSettlement` ; branche `LaterFiscalYearClosed` (400, `details.fiscalYearId` /
        `fiscalYearName`, clé `journal-entries-modify-blocked-later-fiscal-year-closed` ×4) ; commentaire de la 24-4b
        au-dessus du verrou de période (`:2986-2996`) réécrit pour le `PUT`
  - [ ] message d'`ENTRY_IS_POSTED` réécrit (« … ne se supprime pas. Pour la corriger, modifiez-la tant que son
        exercice est ouvert, ou contre-passez-la ») : clé `journal-entries-blocked-posted` ×4, repli Rust
        `kesh-api/src/errors.rs:3045-3048`, `#[error]` `kesh-db/src/errors.rs:637` (D5, C-15-8-27, finding F4)
  - [ ] message d'`ENTRY_IS_REVERSED` élargi (« … ne peut plus être modifiée ni supprimée ») **partout où il
        s'écrit** : clé `journal-entries-delete-blocked-reversed` (quatre locales, consommée par `errors.rs:2982`),
        repli Rust `kesh-api/src/errors.rs:2983`, `#[error]` de `DbError::EntryIsReversed` (`kesh-db/src/errors.rs:623`)
        (finding R2-10)
- [ ] **T2 — `update`** (AC 1–9) : écrite contre le code actuel rebasé, l'ancien corps pour modèle (D4) ; verrou de
      l'écriture **premier acte**, lecture **ordinaire** des projets existants **après** lui (C-15-8-23), ordre des
      verrous écriture → [sentinelle → projets] → exercice → exercices postérieurs (`find_later_closed_in_tx`,
      C-15-8-22) ; `actor_api_key_id` + `for_actor` ; `modification_guard(conn: &mut MySqlConnection, …)` (dont la
      trace du paiement détaché, bornée par société des deux côtés), `modification_refusal`, `modification_blocker`
      (pool → `pool.acquire()`, C-15-8-24) ; `fiscal_years::find_later_closed` (lecture ordinaire, `SELECT` partagé
      avec `find_later_closed_in_tx` par une constante) ; `validate_lines_accounts_in_tx` appelée telle que la 15-5a
      l'a laissée ; doc-comment : règle de sérialisation (dont la vue ouverte à 1-bis et la borne tolérée périmée),
      atomicité du détachement, ordre des verrous, **les trois cycles nommés, étendus aux exercices postérieurs**, les
      deux coûts (verrou d'intervalle des exercices postérieurs, sentinelle d'un `PUT` voué au refus) et le coût de la
      lecture d'audit (D1, D2, D4)
  - [ ] tests unitaires de `mod tests` retirés par la 24-4b (`git show 08e20353 -- crates/kesh-db/src/repositories/
        journal_entries.rs`), traités **un par un** (tableau « Tests unitaires de l'ancien `update` », Dev Notes) ;
        helper `three_accounts` rétabli ; `mk_project`, `line`, `tagged_line` existent encore (`:3439-3469`) —
        **ne pas les dupliquer**
  - [ ] tests à deux connexions (AC 8 — dont la clôture concurrente d'un exercice postérieur —, AC 9) et leurs mutations
        tuées
- [ ] **T3 — Route et détail** (AC 1, 3, 9, 10, 12, 18) : `UpdateJournalEntryRequest`, `prepare_new_journal_entry`
      extraite du `POST` (D4) ; handler `PUT` enveloppé dans `retry_with` ; `api_key_id` passé au `PUT` ; trois champs du
      détail ; tests de clé d'API (AC 3), table de correspondance (AC 12), export (AC 18) ;
      `every_document_owned_entry_is_refused` étendu au solde `write_off` et au paiement détaché (AC 6)
- [ ] **T4 — Garde-fou d'inventaire** (AC 13, D3) : `crates/kesh-db/tests/journal_entries_modification.rs`, monté sur
      le squash (pas d'inscription dans `test_schema_guard.rs`) ; contrôles des clés étrangères vers `journal_entries`
      **et** vers `journal_entry_lines`, des colonnes de `journal_entry_lines`, des colonnes à nom d'écriture sans clé
      étrangère ; mutations tuées
- [ ] **T5 — Audit** (AC 16) : `ACTIONS`, clé i18n ×4, `audit_route_registry.rs:82`
- [ ] **T6 — Écran** (AC 12, 14) : fiche (« Modifier », motifs, rôle Consultation pour « Modifier » et
      « Contre-passer ») ; formulaire en mode édition **par inversion du gel**, **modale de conflit écartée**, prop
      `onStale` (D8, tableau des fichiers) ; `form-helpers.test.ts` renaît ; api, types ; clés i18n ×4 (liste fermée de
      D8) ; `KNOWN_VIOLATIONS` : retrait des trois entrées `journal-entry-conflict-*` (`lint-i18n-ownership.js:87-89`) ;
      replis Svelte de `settings/opening-balances/+page.svelte:233-234` et `:387-388` ; `i18n-keys.test.ts`
      (`ATTENDU.sitesTotal`) mis à jour **avec sa ventilation** dans le doc-comment — la ventilation **ne compte pas**
      les sites de la modale écartée (dont le troisième `journal-entry-form-cancel`) ; `e2e-selecteurs-traduits.test.ts`
      selon les sélecteurs réellement employés ; commentaire de la 25-3-c
      `routes/(app)/supplier-invoices/[id]/+page.svelte:409-411` (« Ne pas le rechercher dans l'audit ») **reformulé** :
      la fiche de la facture ne le recherche toujours pas pour l'afficher, mais la garde d'écriture le lit pour geler
      le paiement (D1, réserve 1, C-15-8-25)
  - [ ] **Vitest** : exhaustivité du `switch` des motifs de modification (un test par code, onze codes, sur le patron de
        celui de `blockedLabel`) ; bornes `min`/`max` de date en édition (exercice de l'écriture, borne de verrou) ;
        branches d'erreur du mode édition (`FISCAL_YEAR_CLOSED` sans `notifyMissingFiscalYearOrFallback`, 409 de course
        dont `DETACHED_SUPPLIER_SETTLEMENT`, `OPTIMISTIC_LOCK_CONFLICT` → toast puis `onStale`, **aucune** modale) ;
        `fromJournalEntryResponse` sur une ligne à compte archivé
- [ ] **T7 — Playwright** (AC 14) — `tests/e2e/journal-entries.spec.ts`, bloc « le gel (Story 24-4b) » renommé ; les
      specs de liste retirées par la 24-4b qui portaient la **modification** sont **remplacées** (pas rétablies : la
      liste n'offre plus ce geste) par des parcours **depuis la fiche** :
  - [ ] « édition nominale — modification du libellé » → `modifier depuis la fiche : le libellé change, le numéro reste`
  - [ ] « conflit 409 affiche la modale de reload » → `conflit de version : toast et fiche rechargée` (version bougée par
        un `PUT` d'API entre l'ouverture du formulaire et l'enregistrement)
  - [ ] neufs : `écriture de facture : pas de Modifier, le motif est affiché` ; `rôle Consultation : ni Modifier ni
        Contre-passer`
  - [ ] les deux tests de liste de la 24-4b (`la liste n'offre plus ni modification ni suppression`, `la ligne renvoie
        vers la fiche, d'où part la contre-passation`) **restent** ; JSDoc reformulé
- [ ] **T8 — Tests qui changent de sens** (liste ci-dessous) : réécrits, **pas** supprimés en bloc ; chaque test retiré
      nommé au Dev Agent Record avec son remplaçant
- [ ] **T9 — Documentation** (AC 17) — cf. Dev Notes ; `make fr` dans `docs/manual/`, PDF commités, contrôle de l'AC 17
      **sur les PDF aplatis** (il fait foi) et sur `fr-CH/messages.ftl`, relecture à la main des trois autres locales,
      table des matières vérifiée ; `README.md` **non touché** (15-8b, C-15-8-26)
- [ ] **T10 — Gates** (⛔ complets — exception `kesh-db` : ciblage interdit dès qu'un repository est touché)
  - [ ] base remise à zéro (KF-039), `scripts/test-fast.sh` sous `mem-guard`
  - [ ] `npm run check` · `lint-i18n-ownership` · `test:unit` · `build`
  - [ ] suite Playwright **complète au dernier commit de code**, jugée fichier par fichier contre `docs/testing.md`
        § « Les échecs attendus » (D7 de la rétro Epic 25 : « rejouée au push » interdit)

## Hors périmètre

- **La suppression** (`DELETE`), la mention « Modifiée », le lien « Historique », le retrait d'`ENTRY_IS_POSTED` :
  **15-8b**.
- **Statut brouillon** : non (24-4b D1 tient ; la CR rend la modification possible sans état intermédiaire).
- **Écran de versions** avec différentiel : non (C-15-8-6) ; le journal d'audit porte l'information.
- **Marqueur d'écriture d'ouverture**, et plus généralement **marqueur structurel de l'origine d'une écriture** : non
  (D1, D6) ; le paiement détaché est gelé par la trace d'audit (C-15-8-20) — la dette est **#541** : une colonne qui
  garde le lien d'un paiement détaché (ou une table des « paiements à rattacher ») remplacerait cette lecture d'audit et
  rendrait le lien visible au garde-fou D3.
- **L'exercice postérieur clos pour les autres chemins d'écriture** (création, dévalidation, règlement, contre-passation
  datée dans un exercice antérieur à un exercice clos) : défaut préexistant, non fermé ici — **signalé à l'orchestrateur
  pour une issue** (C-15-8-22).
- **Modifier une écriture de pièce** : non, par construction (#532 point 2) — elle se corrige par le chemin de sa pièce.
- **Déplacer une écriture d'exercice** : non (C-15-8-3).
- **Audit par clé d'API de la création, de la contre-passation et de la dévalidation** : non — défaut préexistant
  (`routes/journal_entries.rs:460`, `:566` ; `invoices::unvalidate` sans `actor_api_key_id`, `invoices.rs:1486-1492`),
  signalé à l'orchestrateur pour une issue.
- **Verrou des comptes à la validation des lignes** : hors périmètre (choix C21 de la 15-5a).

## Dev Notes

### Règle de découpage — déclenchée, découpée (C-15-8-17)

Recompté à la granularité de la règle (« modules métier de premier niveau ») en P2 (finding F6) : la story d'origine
touchait `kesh-db/repositories/{journal_entries, invoices, accounts}`, `kesh-db/errors`,
`kesh-api/routes/journal_entries`, `kesh-api/{errors, audit_labels}`, `kesh-report`, `kesh-i18n`,
`frontend/features/journal-entries`, `routes/journal-entries/[id]`, `routes/settings/opening-balances` — bien plus de
cinq. Le signal a été déclaré ; l'orchestrateur a découpé selon la ligne pré-déclarée par C-15-8-9 : la modification
ici (15-8a), la suppression en 15-8b. Cette fiche reste au-dessus de cinq modules (une partie des touches ne porte que
sur des commentaires) : c'est le **noyau** du geste demandé, qui ne se coupe plus sans livrer un `PUT` sans écran.

### Dérogation règle de splitting (C-15-8-28)

**Constat** (finding F2 de la validation P3) : cette fiche touche encore **plus de cinq** modules de premier niveau
(`kesh-db` — repositories, errors, tests —, `kesh-api` — routes, errors, `audit_labels`, tests —, `kesh-i18n`,
`kesh-report` en commentaires, frontend `features/journal-entries`, `routes/journal-entries/[id]`,
`routes/settings/opening-balances`, plus docs et manuels). Le critère « scope cross-cutting » de la règle de splitting
préventif est **franchi**, et l'exception prévue par le `CLAUDE.md` (cycle Cargo, merge intermédiaire non testable)
**ne s'applique pas** telle quelle : un `PUT` sans écran **est** testable. La règle n'est donc pas contournée en
silence : elle est **dérogée**, par écrit.

**Justification** :

1. **Story URGENTE, demandée par le Project Lead** (#532, constat de recette de la v0.12.1 : ses soldes de départ ont
   débit et crédit inversés, et la seule correction aujourd'hui en coûte trois écritures).
2. **Elle est déjà issue d'un découpage** (15-8 → 15-8a / 15-8b, C-15-8-17) : la ligne de coupe pré-déclarée a été
   suivie ; ce qui reste ici est un seul geste — modifier une écriture — de la base à l'écran.
3. **Un `PUT` sans écran ne rend aucun service à l'utilisateur qui l'attend** : Guy corrige son écriture d'ouverture
   depuis la fiche, pas par `curl`. Une coupe backend / écran livrerait d'abord une API inutilisable par lui, et
   retarderait d'autant la correction demandée.

**Risque accepté** : une revue adversariale sur un périmètre large voit moins bien les régressions de remédiation
(motif mesuré aux Epics 22 et 23). Il est contenu par la passe **ciblée** de fin de boucle (CLAUDE.md § *La passe
ciblée*) et par le fait que la moitié des modules ne portent que des commentaires et des textes.

**Repli écrit, si une passe RECYCLE** (défaut qui revient sous une autre forme, ou qui naît du correctif précédent —
amendement D5 de la règle) : couper alors en **15-8a-1** — l'API (D1 à D7, D9, D10 ; AC 1 à 13, 15, 16, 18 ; T0 à T5,
T8 et la partie API de T9 : `docs/api-external.md`, Pattern 5, `optimistic-locking-patterns.md`, `CHANGELOG.md`) — et
**15-8a-2** — l'écran et le manuel (D8 ; AC 14, 17 ; T6, T7, la partie manuel de T9). 15-8a-2 dépend de 15-8a-1 ; la
15-8b dépend des deux. Décision **consignée au registre** (C-15-8-28) ; le signal reste déclaré au Project Lead à
chaque passe qui le lèverait.

### Les sites qui supposent aujourd'hui l'immuabilité — inventaire au sol (2026-10-08)

⛔ **Recontrôler par grep sur le dépôt ENTIER avant de déclarer la liste close** — la 24-4b a raté quatre fois ce geste
(périmètre `frontend/` seul, arrêt au premier résultat, `crates/` seul, report incomplet). Commandes (larges à dessein ;
elles rendent ~180 lignes, dont une majorité sans rapport — gel des PDF de facture, adresses, relances — à trier) :

```sh
grep -rn "ENTRY_IS_POSTED\|EntryIsPosted\|enforce_immutability\|blocked-posted" . --exclude-dir={node_modules,target,.git,.svelte-kit}
grep -rnEi "définitiv|imposée|immuab|immutab|ne se (modifie|supprime|réécri)|ni modifi|\bgel(ée|é|er)?\b|gelée?s?\b|24-4b" crates/*/src crates/*/tests crates/kesh-db/migrations frontend/src frontend/tests docs/*.md README.md
grep -nEi "définitiv|imposée|immuab|immutab|ne se (modifie|supprime|réécri)|ni modifi|ni supprim|ni modification|seule voie|seul chemin|refusées en permanence|antérieures au gel|\bgel\b|gelée?s?\b" docs/manual/fr/*.tex
```

Les sites ci-dessous sont ceux de la **modification** ; ceux qui ne parlent que de la suppression sont dans la 15-8b.

**Code (`crates/`)**

| site | ce qu'il dit / fait | à faire |
|---|---|---|
| `kesh-db/src/errors.rs:623` (`#[error]` d'`EntryIsReversed`), `:626-627` (doc d'`EntryIsPosted`), `:637` (`#[error]` d'`EntryIsPosted`) | « suppression refusée » ; « ne se réécrit ni ne se supprime » ; « modification et suppression refusées » | message élargi (T1) ; doc : seul émetteur restant le `DELETE`, retiré par la 15-8b ; `#[error]` d'`EntryIsPosted` → « suppression refusée » (D5, C-15-8-27) |
| `kesh-db/src/repositories/journal_entries.rs:1-40` (module) | gel ; « Immutabilité post-clôture » (`:36`, reste vrai) | réécrire ce qui concerne la modification (« corriger par contre-passation plutôt que par suppression » : à la 15-8b) |
| `journal_entries.rs:65-82` (doc de `validate_lines_accounts_in_tx`, **réécrit par la 15-5a**) | « seul chemin d'écriture d'une ligne d'écriture depuis que la modification n'existe plus (Story 24-4b) » | ⛔ faux après cette story : appelée aussi par `update` |
| `journal_entries.rs:292` (« symétrique à `update` (:671) ») | renvoi à une fonction absente | renvoi rétabli vers `update` |
| `journal_entries.rs` `mod tests` — **tous** les commentaires qui citent le gel ou la 24-4b sur la modification (relevés par le deuxième `grep` de T0, dont `:2146`, « ce que le gel de l'Epic 24 interdit ») | gel | traités un par un ; ceux du `DELETE` (`:2120`, `:2396-2444`, `:2465`, `:2505-2509`) sont à la 15-8b |
| `kesh-db/src/repositories/journal_entry_number_sequences.rs:17` | « ce que le gel de l'Epic 24 interdit précisément » | ré-attribuer : c'est C-15-8-3 (le numéro ne change jamais) qui l'interdit désormais |
| `kesh-db/src/repositories/reconciliation_cancel.rs:5-8` | « le gel de la 24-4b en interdit la modification » | reformuler : une écriture **rapprochée** reste non modifiable, par la garde `MATCHED_BANK_TRANSACTION` (D2) — le constat reste vrai, son motif change |
| `kesh-db/src/repositories/accounts.rs:544-545` | « Le modèle était `journal_entries::update`, supprimée par la Story 24-4b » | reformuler (le modèle existe de nouveau) |
| `kesh-api/src/errors.rs:2986-2996` | commentaire du gel au-dessus du verrou de période | réécrit pour le `PUT` (D5) |
| `kesh-api/src/errors.rs:1146` | repli Rust de `error-opening-complement-account-moved` (« par une contre-passation ou une écriture de correction ») | aligné sur le nouveau texte de la clé (« en modifiant l'écriture ») |
| `kesh-api/src/routes/journal_entries.rs:583-606` | handler `PUT` refusant et son doc-comment | réécrire |
| `kesh-api/src/audit_labels.rs:130-132`, `:192` | `journal_entry.updated` absent d'`ACTIONS`, « qu'aucun site n'écrit plus » | rétablir, reformuler |
| `kesh-api/tests/audit_route_registry.rs:82` | `NoMatter("… 409 ENTRY_IS_POSTED")` | `Traced` |
| `kesh-api/tests/admin_full_import_e2e.rs:1386-1388` | « ne se réécrit plus du tout (24-4b) » | reformuler : le repointage reste nécessaire (une écriture **de facture** ne se modifie pas) |
| `kesh-api/tests/period_lock_e2e.rs:3-10` (doc de module) | « la 24-4b a supprimé `journal_entries::update` et refuse le `DELETE`, donc plus rien n'est réécrivable » | reformuler : `update` est rétablie, et le verrou de période la garde sur l'ancienne **et** la nouvelle date (D4 étape 7) ; le `DELETE` : 15-8b |
| `kesh-api/tests/period_lock_e2e.rs:396`, `:431` | « l'ordre 24-4a → 24-4b » ; « geler sans laisser corriger » | **conservés** : historiques, et toujours vrais pour une écriture de période verrouillée |
| `kesh-api/tests/journal_entry_reversal_e2e.rs:6`, `:1095` (en-tête du bloc), `:1311`, `:1378-1383`, `:1435`, `:1439` | doc du gel | reformuler avec les tests qu'ils coiffent (tableau suivant) — ceux qui ne parlent que du `DELETE` : 15-8b |
| `kesh-report/src/balance_sheet.rs:31-34` | « L'invariant était aussi tenu par `journal_entries::update`, supprimée … » | reformuler : `update` le tient de nouveau, par sa garde de date (D4 étape 5) |
| `kesh-report/src/trial_balance.rs:42-45` | « qu'aucune mise à jour ne peut plus défaire (Story 24-4b) » | jumeau du précédent : reformuler de même |

**Sites relevés et conservés** (le grep les rend ; ils restent vrais) : `kesh-core/src/accounting/mod.rs:1`, `:9` et
`kesh-db/src/entities/fiscal_year.rs:15`, `:22` (immutabilité **post-clôture**) ; `kesh-db/src/backup.rs:43` (« une
écriture **référencée** ne se supprime pas ») ; `journal_entries.rs:213` (projet archivé sur une contre-passation) ;
`kesh-db/tests/invoice_settlement.rs:510` (écriture d'un règlement : gelée par sa pièce) ;
`kesh-db/tests/fiscal_years_repository.rs:905-908` (exercice clos) ; `kesh-report/src/balance_sheet.rs:277` (exercice
clos) ; les mentions du gel des **PDF de facture** (`invoice_frozen_pdf*`, `issued_invoice_pdf.rs`,
`invoice_email.rs`, `invoices/[id]`) et de **facture validée** (`invoice_delete_e2e.rs`, `invoices.spec.ts:359`).

**Tests de la 24-4b qui changent de sens** — `crates/kesh-api/tests/journal_entry_reversal_e2e.rs` (part du `PUT`) :

| test | aujourd'hui | devient |
|---|---|---|
| `putting_a_posted_entry_is_refused_and_changes_nothing` (`:1148`) | PUT → 409 | AC 1 + AC 3 (modification réussie, audit) |
| `putting_with_an_empty_or_broken_body_is_refused_the_same_way` (`:1209`) | corps ignoré → 409 | AC 10 (corps malformé → 400/422) |
| `put_and_delete_never_leak_the_existence_of_a_foreign_entry` (`:1254`) | 404 | **reste** — le `PUT` porte désormais un corps **valide** ; le `DELETE` inchangé |
| `a_reversed_entry_answers_reversed_not_posted` (`:1317`, message `:1331`) | `ENTRY_IS_REVERSED` ≠ `ENTRY_IS_POSTED` | reste **tel quel** ici (son sens tient tant que le `DELETE` a le gel) ; ⚠️ **la 15-8b le réécrit** — il cesse de distinguer quoi que ce soit au retrait du gel |
| `a_closed_fiscal_year_answers_before_both_conflicts` (`:1341`) | 400 avant 409 | reste (AC 7) ; ⚠️ **la 15-8b le réécrit** (« les deux 409 » n'en font plus qu'un au retrait du gel) |
| `consultation_can_neither_rewrite_nor_delete` (`:1365`) | 403 | reste (AC 10) |
| `the_opening_entry_is_frozen_but_still_correctable` (`:1386`) | ouverture gelée | ⛔ **inversé pour le `PUT`** : AC 2 + AC 11 ; la moitié `DELETE` reste « refusée » jusqu'à la 15-8b, qui l'inverse à son tour |
| `an_entry_of_a_closed_year_stays_correctable` (`:1445`) | contre-passable en exercice clos | reste ; ajouter `PUT` → 400 |
| `every_document_owned_entry_is_refused` (`:485`) | six chemins de pièce | **étendu** : solde `write_off`, paiement détaché (AC 6) ; sert de montage au `PUT` |

`deleting_a_posted_entry_is_refused` (`:1231`) et `deleting_a_reversed_entry_is_refused_but_bulk_delete_still_works`
(`:727`) **ne bougent pas** ici : 15-8b.

**Tests unitaires de l'ancien `update`** — retirés de `journal_entries.rs` `mod tests` par la 24-4b (`git show
08e20353 -- crates/kesh-db/src/repositories/journal_entries.rs`, relevé au sol), avec le helper `is_no_op_change`
(production, rétabli par T2) :

| test (`d2910022`) | décision |
|---|---|
| `update_no_op_returns_unchanged_entity_no_lines_churn` (`:2922`) | **rétabli**, plus l'assertion « aucune entrée `journal_entry.updated` » (AC 3) |
| `update_no_op_in_closed_fy_returns_fiscal_year_closed` (`:3008`) | **rétabli** (AC 5, exercice clos seul) |
| `update_no_op_with_inactive_account_returns_inactive_error` (`:3102`) | **rétabli** (AC 4 — compte **archivé**) ; plus un jumeau neuf `update_no_op_with_non_postable_account_returns_not_postable` (C-15-8-18) |
| `update_partial_change_bumps_version` (`:3187`) | **rétabli** (AC 1) |
| `test_update_project_only_change_is_not_noop` (`:3459`) | **rétabli** |
| `test_update_grandfathers_preexisting_archived_project` (`:3516`) | **rétabli** (C-15-8-4 garde le *grandfathering* des projets) |
| `test_update_moves_archived_tag_between_lines` (`:3604`) | **rétabli** |
| `test_update_grandfathers_non_postable_by_account` (`:3762`) | ⛔ **inversé** → `test_update_refuses_a_line_on_an_account_made_non_postable` (AC 4) : ses deux cas (édition du libellé, ligne ajoutée sur le compte) rendent `AccountsNotPostable` nommant le compte |

Helpers : `three_accounts` **rétabli** (disparu) ; `mk_project`, `line`, `tagged_line` existent encore
(`:3439-3469`, déplacés par la 24-4b pour `test_create_line_projects_mixed`) — **ne pas les dupliquer** ; la 15-5a a
ajouté `account_number` et `set_active` dans le même `mod tests` — **les réutiliser**. Les appels `update(…)` des tests
rétablis prennent la nouvelle signature (`actor_api_key_id = None`).

**Frontend**

| site | à faire |
|---|---|
| `src/lib/features/journal-entries/JournalEntryForm.svelte:60-63` (« CRÉATION SEULE ») | mode édition rétabli (D8) — le commentaire tombe avec l'inversion |
| `src/lib/features/journal-entries/form-helpers.ts:1-13` | fonctions rétablies, doc revu ; `form-helpers.test.ts` rétabli |
| `src/lib/features/journal-entries/journal-entries.api.ts:56-62` | `updateJournalEntry` (et `deleteJournalEntry`, gardé pour la 15-8b) |
| `src/lib/features/journal-entries/journal-entries.types.ts` | `UpdateJournalEntryRequest`, champs du détail, `ModificationBlocker` |
| `src/routes/(app)/journal-entries/+page.svelte:34-35`, `:268`, `:554-555` (commentaires du gel) | reformuler : l'édition vit sur la fiche ; ⛔ **ne pas inverser** le gel sur ce fichier (D8) |
| `src/routes/(app)/journal-entries/[id]/+page.svelte` | D8 |
| `src/routes/(app)/settings/opening-balances/+page.svelte:233-234` (repli de `opening-balances-complete-confirm`, « Elle ne se modifie plus ensuite ») et `:387-388` (repli de `opening-balances-locked-already-has-entries`) | replis Svelte alignés sur le nouveau texte des clés — ⚠️ un `grep` sur la clé seule ne rend pas la ligne du repli suivante |
| `src/lib/shared/i18n-keys.test.ts:163-182` | `sitesTotal` + ventilation (sans les sites de la modale écartée) |
| `frontend/scripts/lint-i18n-ownership.js:87-89` | retirer les trois entrées `journal-entry-conflict-*` |
| `tests/e2e/journal-entries.spec.ts:274-…` (bloc « le gel (Story 24-4b) ») | les deux tests de liste **restent vrais** — renommer le bloc, reformuler le JSDoc ; parcours AC 14 : specs nommées en **T7** |
| `tests/e2e/fiscal-years.spec.ts:108-118` (JSDoc) | `update` existe de nouveau et lève `FISCAL_YEAR_CLOSED` (400) — ses neuf tests ne passent toujours que par la création |

**i18n** (`crates/kesh-i18n/locales/{fr,de,en,it}-CH/messages.ftl`) — en **quatre** locales :

| clé | à faire |
|---|---|
| `journal-entries-delete-blocked-reversed` (fr `:355`) | texte : « modifiée ni supprimée » (T1) |
| `journal-entries-blocked-posted` (fr `:356`) | texte réécrit : « ne se supprime pas … modifiez-la tant que son exercice est ouvert, ou contre-passez-la » (D5, C-15-8-27) — la clé reste, la 15-8b la retire |
| `opening-balances-locked-already-has-entries` (fr `:929`) | ajouter la modification de l'écriture d'ouverture comme chemin de correction |
| `opening-balances-complete-confirm` (fr `:962`) | « Elle ne se modifie plus ensuite » devient faux |
| `error-opening-complement-account-moved` (fr `:979`) | ajouter « en modifiant l'écriture » ; repli Rust `errors.rs:1146` aligné |
| `audit-log-action-journal-entry-updated` | **ajouter** |
| clés neuves de l'écran (D8) | `journal-entry-edit` (nom repris de `d2910022`, lisible dans `git show d2910022:crates/kesh-i18n/locales/fr-CH/messages.ftl`), `journal-entries-edit-conflict`, `journal-entries-line-account-unusable`, `journal-entries-modify-blocked-{fiscal-year-closed,later-fiscal-year-closed,period-locked,detached-settlement}` |

*(`journal-entries-blocked-posted` reste jusqu'à la 15-8b : le `DELETE` l'émet encore — avec son texte réécrit.)*

⚠️ **Une clé lue par le backend ne se retire pas sans greper `crates/`** : `t(key, default)` rend **la clé brute** si
le bundle est chargé et la clé absente (`kesh-i18n/src/loader.rs`, `format_unknown_key_returns_key`) — le piège S1-C1
de la 24-4b. `i18n-keys.test.ts` ne scanne que le frontend.

**Manuel utilisateur** (`docs/manual/fr/user-manual.tex`) — ⛔ aucun gate ne le lit ; passages relevés. ⚠️ Plusieurs
mêlent modification et suppression : cette story les réécrit pour dire **vrai entre les deux merges** (la modification
est ouverte, la suppression encore refusée) ; la 15-8b complète.

| ligne | ce qu'il dit | à faire |
|---|---|---|
| `:380` *keshnote* § postabilité | « **Une écriture déjà enregistrée ne se modifie plus du tout** : depuis la version qui introduit le gel, elle se corrige par contre-passation » | ⛔ faux — dire qu'elle se modifie tant que l'exercice est ouvert, et que la modification applique la **même** garde de postabilité, **y compris aux lignes non touchées** (C-15-8-4), avec le refus qui **nomme** le compte (15-5a) |
| `:472-476` § « Cycle de vie » | « immédiatement comptabilisée » | reste vrai ; ajouter qu'elle reste **modifiable** jusqu'à la clôture (cadre D1) |
| `:478-494` § « Une écriture enregistrée ne se modifie plus » | gel inconditionnel | ⛔ **section réécrite et renommée** (« Modifier une écriture » ; la 15-8b la renommera « Modifier ou supprimer une écriture ») — le titre est aussi l'entrée **7.4 de la table des matières** du PDF, à contrôler après `make fr` ; la section n'a pas de `\label`, aucun `\ref` ne la vise (les renvois s'écrivent en toutes lettres, `:380`) : le cadre D1, la trace d'audit avant/après, la date qui reste dans l'exercice, le paiement détaché qui reste gelé ; ⚠️ le *keshtip* « Pourquoi il n'y a pas de brouillon » reste vrai, mais « ce que vous validez part directement aux livres » appelle « … et reste corrigeable tant que l'exercice est ouvert » |
| `:506-510` § verrou de période | « aucune écriture déjà passée dans la période ne pourra plus en disparaître » | ajouter « ni être modifiée » ; et dire que **verrouiller est le geste qui fige un trimestre déclaré** (D9) |
| `:532-570` § contre-passation, *keshnote* `:554-555` | « La contre-passation est **imposée** : la modification et la suppression d'une écriture sont refusées » | ⛔ faux pour la modification — la contre-passation devient **le** chemin pour une écriture d'exercice clos, de période verrouillée ou de pièce ; la phrase sur la suppression reste, jusqu'à la 15-8b |
| `:631` § clôture | « La modification et la suppression, elles, sont refusées en permanence — clôture ou non » | ⛔ inversé pour la modification : la clôture est **ce qui** la ferme — celle de l'exercice de l'écriture **ou d'un exercice postérieur** (le bilan est cumulatif ; clore N+1 fige donc aussi N, C-15-8-22) ; la suppression reste refusée jusqu'à la 15-8b |
| `:642` § réouverture | « L'exercice rouvert redevient modifiable » | redevient **vrai** pour les écritures manuelles (cadre D1) — vérifier la formulation |
| `:667` *keshnote* soldes de départ | « L'écriture d'ouverture, comme toutes les autres, ne se modifie ni ne se supprime ensuite » | ⛔ faux — c'est le cas déclencheur : dire comment corriger une inversion (fiche → Modifier) ; la suppression : 15-8b |
| `:681` *keshnote* compléter | « On le corrige … en contre-passant l'écriture fautive, ou par une écriture de correction » | ajouter **la modification** en premier chemin, et la ligne retirée qui redevient complétable |
| `:1331-1337` § annulation d'une facture fournisseur payée | le paiement détaché ; « sa fiche d'écriture offre la contre-passation » | ajouter qu'il reste figé (C-15-8-20), par la **tournure prescrite** de l'AC 17 (« La fiche de ce paiement n'offre pas « Modifier » : elle affiche le motif, et la contre-passation reste disponible. ») |
| `:2115-2140` FAQ « revenir en arrière » | « Écriture : ni modification ni suppression, jamais » (`:2120`) ; paiement détaché (`:2140`) | ⛔ faux pour la modification ; la puce « exercice clôturé » reste vraie pour la contre-passation ; `:2140` : le paiement détaché ne se modifie pas |

**Manuel administrateur** (`docs/manual/fr/admin-manual.tex`) — **six** passages relevés, et **un** ajout :

| ligne | ce qu'il dit | à faire |
|---|---|---|
| `:1796` | ce qu'une clé `read-write` peut faire depuis la v0.12.1 | ajouter : **modifier une écriture** manuelle (le `PUT`), sous les mêmes gardes, tracée avec la clé |
| `:1834` | verrou de période : « aucune écriture de la période ne peut plus en être **supprimée** » | « ni modifiée ni supprimée » |
| `:1835` | « n'est plus ni modifiable ni supprimable… L'immutabilité n'est donc plus seulement tracée, elle est imposée » | ⛔ faux — la conformité 958f repose désormais sur la **trace avant/après**, la clôture et le verrou de période ; la suppression reste refusée jusqu'à la 15-8b |
| `:1838` | « conserve aussi les modifications et suppressions **antérieures au gel** » | et les modifications **postérieures**, sous `journal_entry.updated` |
| `:1969` | « une écriture enregistrée est **définitive** et se corrige par contre-passation » (art. 957a, al. 1) | ⛔ faux — modifiable tant que l'exercice est ouvert, chaque modification tracée avant/après ; définitive à la clôture ou sous le verrou de période |
| `:2004` | Olico art. 3 : « les corrections passent par contre-passation » | par contre-passation **ou** par modification tracée |
| § restauration (après `:1838`, la fusion d'`audit_log`) | — | **une phrase** : la garde du paiement détaché lit le journal d'audit ; la fusion d'une archive d'une **autre histoire** peut, par coïncidence d'identifiants, figer une écriture manuelle (contre-passation toujours offerte) — limite assumée, #541 (D1, réserve 2, C-15-8-25) |

`:1837` (« une seule voie fait encore disparaître une écriture ») reste vrai jusqu'à la 15-8b. Brochure : rien (`:393`
parle des avoirs). DE/EN/IT : `README.md` seuls, rien à propager.

**Autres documents**

- `docs/api-external.md` : D10.
- `docs/optimistic-locking-patterns.md:49`, `:75` (D7).
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md` : Pattern 5, tableau « Deny list » — ligne du `PUT` (D4).
- `CHANGELOG.md` : la section `## [0.13.0] — Non publié` **existe** (créée par la 15-5a) — ⛔ **y ajouter**, ne pas
  dupliquer l'en-tête ; `### Modifié` : *Une écriture se modifie tant que son exercice est ouvert (#532)* — cadre, trace
  avant/après, ce qui reste gelé (dont le paiement détaché et toute écriture d'un exercice suivi d'un exercice clos —
  code neuf `LATER_FISCAL_YEAR_CLOSED`), l'API (`PUT` rétabli, corps + `version`, codes de refus, le `PUT` ne rend plus
  `ENTRY_IS_POSTED`, il rend `ACCOUNT_NOT_POSTABLE` comme la saisie — à dire, c'est un changement de contrat pour une
  intégration ; le message d'`ENTRY_IS_POSTED`, encore rendu par le `DELETE`, change aussi).
- `README.md:29` (« écritures validées ») : ⛔ **pas ici** — la ligne est éditée par la **15-8b seule**, dans sa forme
  finale « modifiables et supprimables tant que l'exercice est ouvert » (C-15-8-26 : deux fiches qui éditent la même
  ligne se heurtent au merge ; entre les deux merges, la ligne **tait** la modification sans rien affirmer de faux).
  `README.md:219` (ligne de la v0.12.0 : « une écriture enregistrée n'est plus ni modifiable ni supprimable ») est
  **historique** — l'état d'une version publiée — et reste tel quel ; à nommer au Dev Agent Record (finding F6).

**À NE PAS toucher** (P8 : modifier une migration appliquée empêche le démarrage) :
`crates/kesh-db/migrations/20260729000001_invoice_lines_revenue_account_backfill.sql:54` et
`crates/kesh-db/migrations/20260830000001_companies_books_lock.sql:13` (« La 24-4b a supprimé
`journal_entries::update` et refuse le `DELETE` ») — toutes deux publiées ; le grep de T0 les rend, elles **restent
fausses à jamais**, et c'est voulu ; ni `20260827000001_invoice_settlements.sql:37-38` (vrai, et publié). Les story
files historiques de `_bmad-output/` (`3-3`, `24-4a`, `24-4b`, …) ; `docs/known-failures.md` et
`docs/change_request.md` (archivés).

### Ce que la story réutilise — ne pas réinventer

| besoin | existant |
|---|---|
| corps de `update`, no-op, *grandfathering* projets — **modèle**, pas copie (D4) | `git show d2910022:crates/kesh-db/src/repositories/journal_entries.rs` `:870-1208` |
| tests unitaires de l'ancien `update` | `git show 08e20353 -- crates/kesh-db/src/repositories/journal_entries.rs` (lignes `-`) |
| mode édition du formulaire, helpers, tests, api, types | `git show 08e20353 -- <fichier> \| git apply -R --3way` (D8, tableau des fichiers) — **jamais** une copie de l'état `d2910022` |
| test d'entrelacement à deux connexions | `test_fixtures::attendre_une_requete_en_cours` (`test_fixtures.rs:560`) ; exemples `opening_complement_repository.rs:717`, `reconciliation_cancel.rs:148` |
| rejeu sur interblocage | `kesh_db::retry::retry_with`, patron `routes/opening_balances.rs:581-610` |
| audit par acteur (utilisateur ou clé) | `NewAuditLogEntry::for_actor` (`entities/audit_log.rs:194`), appel modèle `kesh-db/src/repositories/invoices.rs:2559` |
| inventaire des propriétaires | `reversal_blockers` (`journal_entries.rs:1265`) |
| exercice postérieur clos, verrouillé | `fiscal_years::find_later_closed_in_tx` (`fiscal_years.rs:639`, garde LIFO de `reopen`) |
| mappage d'erreur à motif | `EntryNotReversable` (`kesh-api/src/errors.rs:2698`) |
| instantané d'audit | `entry_snapshot_json` (`:906`) |
| verrou de période, seuil inclusif | `create_in_tx_inner` `:250-326`, `delete_in_tx` `:1069-1089` |
| validation des comptes, deux refus | `validate_lines_accounts_in_tx` (`:84`, telle que la 15-5a l'a laissée) |
| montage de chaque pièce | `every_document_owned_entry_is_refused` (`journal_entry_reversal_e2e.rs:485`) |
| annulation d'une facture fournisseur payée | `supplier_invoices::cancel_in_tx` (`supplier_invoices.rs:851`) |

### Pièges

- ⛔ **Le `FOR UPDATE` de l'écriture est le PREMIER acte de la transaction** (D2) : aucune lecture ordinaire avant
  lui. Une lecture ordinaire ouvre la vue `REPEATABLE READ` ; posée avant ce verrou, elle rend la garde aveugle à ce qui
  a été commité pendant l'attente. La vue s'ouvre ensuite à 1-bis (projets existants, lecture ordinaire, C-15-8-23) :
  ⛔ **ne pas** y remettre de `LOCK IN SHARE MODE` (verrou d'intervalle sur les lignes avant l'exercice : un quatrième
  cycle, D2) ; l'exercice et les exercices postérieurs restent lus par des lectures **verrouillantes** (état courant).
  Les tests à deux connexions de l'AC 8 le tiennent.
- ⛔ `FOR SHARE` est une **erreur de syntaxe** sur MariaDB 10.11 : le verrou partagé s'écrit `LOCK IN SHARE MODE`
  (`opening_complement.rs:38-39`).
- ⛔ **La garde D2 se lit DANS la transaction**, sous ce verrou — `modification_guard` prend une `&mut MySqlConnection`
  (C-15-8-24) : lui passer `&mut **tx`, et reprêter `&mut *conn` à `reversal_blockers` puis à la lecture de la trace.
- ⚠️ `attendre_une_requete_en_cours` est **couplé au texte** de la requête : changer la forme du `SELECT … FOR UPDATE`
  de l'étape 1 ou 1-ter, c'est changer les motifs des tests (sa doc le dit).
- ⚠️ `reversal_blockers` rend `NotFound` sur une écriture absente : dans la transaction, elle vient **après** le
  `FOR UPDATE` qui a déjà établi l'existence — ne pas transformer ce cas en 500.
- ⚠️ **`AccountArchived` est le DERNIER motif de `reversal_blockers`** : le filtrer ne change pas le premier motif
  restant. Mais ne pas réordonner la liste — sa précédence est celle de la contre-passation (24-4a D6).
- ⚠️ **Le `INSERT` des lignes doit omettre toute colonne absente des schémas anciens** — plusieurs tests montent un
  schéma antérieur puis exercent le vrai chemin (commentaire `:358-366`). Les colonnes de `LINE_COLUMNS` existent
  toutes depuis 19-2 : sans objet aujourd'hui, à garder en tête si la 15-1a passe d'abord.
- ⚠️ **Le rejeu ne doit rien laisser hors de la transaction** : la fermeture de `retry_with` ne fait que l'appel au
  repository ; aucun effet (toast, journal, compteur) avant son retour.
- ⚠️ **Base de gate piégée** (KF-039) : la remettre à zéro **avant** chaque gate complet, inconditionnellement.
- ⚠️ **Haiku et les diffs multi-commits** : en revue, fournir le diff aplati `main..HEAD`.

### Références

- Issue **#532** ; Story **24-4b** (gel) — D1, D2, D4, D5, D7 ; Story **24-4a** (contre-passation) — D3, D6 ;
  Story **24-4c** (verrou de période) ; Story **25-2-c** (#381, compteur) ; Story **25-3-c** (#454, annulation d'une
  facture fournisseur payée, règlement détaché) ; Story **25-7** (#445, soldes de départ, verrous du complément) ;
  Story **15-5a** (`ACCOUNT_NOT_POSTABLE`, `exempt_ids` retiré) ; Story **15-1a** (lettrage, `lettering_id`) ;
  Story **15-8b** (la suite).
- `CLAUDE.md` §§ *Review Iteration Rule* (exception `kesh-db`), *Inventorier les sites NON RÉSOLUS*, *Propagation
  post-patch*, *Recompter ses propres comptes rendus*, *Le prompt d'une passe doit NOMMER le manuel*, *Règle de
  splitting préventif*, *Migration breaking policy* (P8).
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md` Pattern 5 ; `crates/kesh-db/src/retry.rs`.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

### Change Log

| date | ce qui s'est passé |
|---|---|
| 2026-10-08 | **Créée par découpage** de la 15-8 après la validation P2 (choix C-15-8-17) ; spec, P1 et P2 au Change Log de l'index `15-8-modifier-une-ecriture.md`. Remédiations de la P2 appliquées ici : dépendance à la 15-5a et deux refus de compte séparés (F1, C-15-8-18) ; rejeu sur interblocage, trois cycles nommés, Pattern 5, projets existants lus en `LOCK IN SHARE MODE` (F2, R2-2, R2-3, C-15-8-19) ; paiement détaché d'une facture fournisseur annulée gelé par la trace d'audit (F3, C-15-8-20) ; modale de conflit écartée de l'inversion, prop `onStale`, clés `journal-entries-*` (R2-1, F4, F8, C-15-8-21) ; inventaire des sites élargi et contrôle de l'AC 17 élargi au `.tex` (R2-4, F5) ; LOW R2-5 à R2-17, F7 à F12 sauf ceux de la suppression (15-8b). **Recompte** (cette fiche) : 18 AC, 11 tâches (T0 à T10), 4 invariants, 10 décisions (D1 à D10), dix codes d'écran. |
| 2026-10-08 | **Validation P3** (deux lentilles **Sonnet**, contexte frais, lecture seule — rotation D6 : Sonnet P1, Opus P2, Sonnet P3 ; prompt versionné `15-8a-validate-prompt-p3.md` ; rapports `target/gate-logs/15-8a-p3-{R,F}.md`). **R** : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 9 LOW ; **F** : 0 CRITICAL, 0 HIGH, 2 MEDIUM, 7 LOW. Doublons : R3-2 = F1, R3-4 = F3, R3-5 = F5, R3-6 ≈ F6 — soit **3 MEDIUM et 13 LOW distincts** ; s'y ajoute le MEDIUM F1 de la 15-8b, qui vaut pour cette fiche. **Branche rebasée** sur `origin/main` (`b11a074a`, 15-5a) avant la remédiation : conflits du registre et de `sprint-status.yaml` résolus par union ; faits « après 15-5a » revérifiés au code (en-tête, T0 et sa **table des décalages mesurés**). **Tout appliqué**, sur décisions de l'orchestrateur : **exercice postérieur clos** — condition 1-bis du cadre, refus 400 `LATER_FISCAL_YEAR_CLOSED`, étape 2-bis, verrou `find_later_closed_in_tx` après l'exercice, onze codes d'écran, tests de refus, de précédence et de concurrence avec mutation, manuel (15-8b F1 — C-15-8-22) ; **projets existants lus ordinairement après le verrou de l'écriture**, `LOCK IN SHARE MODE` et « coût nul » retirés, la vue s'ouvre à 1-bis, la borne tolérée périmée comme à la création, seconde mutation de l'AC 8 retirée, cycles restants nommés et étendus aux exercices postérieurs (R3-2, F1 — C-15-8-23, révise C-15-8-19) ; **`modification_guard(conn: &mut MySqlConnection, …)`**, `pool.acquire()` à l'écran, `find_later_closed` non verrouillante au `SELECT` partagé (R3-1 — C-15-8-24, qui corrige aussi les renvois de C-15-8-5, 10, 12, 13) ; **dérogation écrite** à la règle de splitting, repli 15-8a-1 / 15-8a-2 (F2 — C-15-8-28) ; **paiement détaché** : trois réserves écrites, #541, requête bornée par société des deux côtés (F7 — C-15-8-25) ; **contrôles de manuel** sur les PDF aplatis, motifs complétés, `.ftl` contrôlé, lignes de base mesurées, tournure prescrite du paiement détaché ; message d'`ENTRY_IS_POSTED` réécrit dès cette story (R3-6, F6, F9, F4 — C-15-8-27). **LOW** appliqués tels que proposés : atomicité du détachement au lieu d'un verrou inexistant, `cancel_settlement_in_tx` (R3-4, F3) ; AC 15 en liste fermée (R3-5, F5) ; numéros de ligne (`lint-i18n-ownership.js:87-89`, e2e `:478`, `kesh-db/…/invoices.rs:2559`, `errors.rs:623`, insertion des lignes copiées par `create_in_tx_inner`, constat de D7 — R3-7) ; T0 rebasé (R3-8) ; trois sites de Pattern 5 (R3-9) ; grep `validate_taggable_in_tx` (R3-10) ; « cinq lectures » (R3-11) ; `README.md:29` laissé à la 15-8b, `README.md:219` historique (F6 — C-15-8-26) ; sentinelle d'un `PUT` voué au refus au doc-comment (F8) ; Dev Agent Record à ne pas déclarer « net » pour le formulaire (F9, déjà conforme). **Signal D5** : sévérité MEDIUM en P2 et P3 (« égale ») ; défauts **distincts** — mais R3-1 et R3-2/F1 **naissent de la remédiation P2** (R2-15, C-15-8-19) : recyclage déclaré ; arbitrage de l'orchestrateur : **pas de découpage**, dérogation écrite, repli prêt (C-15-8-28). **Propagation** : symptômes grepés dans les trois fiches 15-8 et le registre (`dix`, `LOCK IN SHARE MODE`, `coût nul`, `(executor`, `modifiables tant que`, `ReversalBlockerHit`, `fait passer A`, `86-88`, `:493`, « après le dernier verrou ») — résidus seulement aux sites qui citent l'erreur corrigée. **Recompte** (cette fiche, `grep`) : 18 AC, 11 tâches (T0 à T10), 4 invariants, 10 décisions (D1 à D10), **onze** codes d'écran ; registre : C-15-8-1 à 29 (`grep -c '^## C-15-8-'` → 29). **Signalé à l'orchestrateur** : issue pour l'exercice postérieur clos sur la création, la dévalidation et le règlement (C-15-8-22). |
