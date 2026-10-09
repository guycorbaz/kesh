# Story 15.1b-0 : La propriété des lignes, par lot — une seule source, prouvée contre l'ancienne

## Status

ready-for-dev *(extraite de la 15-1b le 2026-10-09 à la remédiation de sa validation P2 — signal D5 levé
par un HIGH né d'une remédiation, C-15-1b-9 ; validation P1 remédiée le 2026-10-09 — la lecture de la
route en une transaction, la forme et la mesure du lot fixées, l'oracle durci, les prédicats de
`DocumentKind` publiés ; validation P2 remédiée le 2026-10-09 — `DocumentRef` typé en `DocumentKind`
(C-15-1b-0-3), parité prouvée aussi **par lot**, mesure par `Com_stmt_execute`, garde de la transaction de la
route (C-15-1b-0-4), tests de `first_document_owner` dans `letterings.rs` ; **validation P3 à mener avant tout
développement**)*.

Story **patron** au sens de la § « Règle de splitting préventif » (« dégager d'abord un story-zero qui
pose le pattern ») : elle pose la lecture **par lot** de la propriété des écritures et réécrit dessus les
deux lecteurs du socle ; la **15-1b** (vue des postes ouverts, propositions) la consomme. **Ordre** :
15-1a-i → 15-1a-ii → 15-1a2-0 → 15-1a2-i → 15-1a2-ii → **15-1b-0** → 15-1b → 15-1c. Elle ne dépend **pas** de la
15-1a2 (aucune marque `document` n'y est lue) : elle pourrait la précéder ; l'ordre de l'epic la place
juste avant la 15-1b, qui la suppose mergée. ⛔ Pas de tag entre les merges (C124).

## Story

**As a** développeur de Kesh,
**I want** une fonction qui dise, pour un **lot** d'écritures, quelle pièce possède chacune — facture,
avoir, facture fournisseur, règlement, transaction bancaire —, et que les deux lecteurs actuels de cette
propriété (`reversal_blockers`, `first_document_owner`) la lisent par elle,
**so that** la vue des postes ouverts (15-1b) affiche la pièce de chaque ligne et filtre les lignes de
pièce **sans N+1 et sans seconde liste** des motifs de propriété — et que la refonte soit **prouvée**
contre le comportement d'avant, pas contre elle-même.

## Pourquoi cette story existe

La remédiation P1 de la 15-1b (C-15-1b-2) avait mis dans la vue la refonte d'une fonction du socle. La
validation P2 l'a montrée **non implémentable sans deviner** (15-1b R-4 = F-5 : un `E: Executor` pris par
valeur ne permet pas les deux requêtes que la fiche prescrivait ; appelants non inventoriés, dont un sur
le **pool** ; docstring « ⛔ Une seule requête » contredite) et sa garde **verte par construction** (F-4 :
le test 8 comparait `reversal_blockers` à `document_owners` après que l'un eut été réécrit sur l'autre —
exactement l'AC7 de l'Epic 23 que le `CLAUDE.md` décrit sous « Inventorier les sites NON RÉSOLUS »). Le
signal de découpage D5 était levé (HIGH né d'une remédiation, P1 0 HIGH → P2 1 HIGH). Décision de
l'orchestrateur : **extraire** cette refonte — un repository du socle, gate complet à chaque passe — dans
une story propre, la 15-1b gardant la vue et les propositions.

## Le modèle réel — relevé sur `056997b0`

*(Lignes citées sur `056997b0` ; elles bougeront — **citer et re-greper par le nom de fonction**.)*

**`journal_entries::reversal_blockers<'e, E: Executor>(executor: E, company_id, id) -> Result<Vec<ReversalBlockerHit>, DbError>`**
(`journal_entries.rs:2090`) : **une** requête, onze colonnes par sous-requêtes corrélées
(`sqlx::query_as(` `:2118`, sous-requêtes `:2119-2141`), consommée par `.fetch_optional(executor)`
(`:2144`). Ordre des motifs = précédence
(`IsAReversal`, `AlreadyReversed`, `OwnedByInvoice`, `OwnedByCreditNote`, `OwnedBySupplierInvoice`,
`OwnedBySettlement`, `MatchedBankTransaction`, `AccountArchived`). Chaque sous-requête de propriété est un
`LIMIT 1` **sans `ORDER BY`** : déterministe tant qu'une seule pièce de chaque type vise l'écriture. Pour
les quatre pièces, c'est un **invariant applicatif, non une contrainte** (validation P1, R6 = F-6) : seul
`invoice_settlements.journal_entry_id` est `UNIQUE` (`uq_invoice_settlements_entry`, squash
`0001_schema_squash.sql:690`) ; `invoices.journal_entry_id`, `credit_notes.journal_entry_id` et les deux
colonnes de `supplier_invoices` n'ont qu'un `KEY` et une FK — un `INSERT` brut peut faire viser une même
écriture par deux factures, ou par une facture **et** un avoir **et** une facture fournisseur. Rien ne
l'interdit non plus pour `bank_transactions.matched_entry_id`
(deux transactions peuvent pointer la même écriture : c'est l'« exemption étroite » de la file des
annulations, `settlement_cancellation.rs` doc-comment du rang 3). Le doc-comment de `reversal_blocker`
(`:2051-2053`) pose « ⛔ **Une seule requête** : les sept causes se calculent par sous-requêtes corrélées,
jamais par sept allers-retours ». ⚠️ **Défaut de l'ancienne requête, nommé** (validation P1, F-8) : pour la
facture fournisseur, `id` et `supplier_invoice_number` sont lus par **deux** sous-requêtes `LIMIT 1`
indépendantes (`:2127-2132`), qui peuvent désigner deux lignes différentes si deux factures visent
l'écriture ; la nouvelle lit l'identifiant et le numéro **de la même ligne** (D1).

**`letterings::first_document_owner`** (`letterings.rs:511`, privée) : boucle sur les écritures
distinctes du groupe et appelle `reversal_blockers` **une écriture à la fois** (`:522`), retenant
`OwnedByInvoice | OwnedByCreditNote | OwnedBySupplierInvoice | OwnedBySettlement` (R5).
`MatchedBankTransaction` n'en est pas. **Deux appelants** (validation P2, F2-5 ; `grep -n "first_document_owner"
letterings.rs` → `:511`, `:711`, `:818`) : `create_group_in_tx` (`:711`, refus 5 du lettrage manuel, lignes lues
par l'acte 1 `:191-197`) et `dissolve_group_in_tx` (`:818`, refus 2 de la dissolution — paire `reversal` dont une
ligne est celle d'une pièce, C106 —, lignes lues par `LOCK_LINES_BY_KEY_SQL`, `:204`). Les deux lectures sont
bornées par société.

**Appelants de `reversal_blockers` / `reversal_blocker`** — inventaire **fermé**, relevé par
`grep -rn "reversal_blockers\|reversal_blocker(" crates --include=*.rs`, **et leurs appelants transitifs** par
`modification_guard` (validation P1, R1 = F-1) :

| site | exécuteur passé | contexte |
|---|---|---|
| `kesh-api/src/routes/journal_entries.rs:472` (`reversal_blocker`) | `&state.pool` | lecture pour l'écran — **hors transaction** |
| `kesh-api/src/routes/journal_entries.rs:474` (`modification_blocker`, **transitif**) | `&state.pool` | lecture pour l'écran ; `modification_blocker` (`journal_entries.rs:1203`) fait `pool.acquire()` (`:1208`, **pas** de `begin`) et enchaîne **six** lectures en autocommit, dont `modification_guard` (`:1233`) — six instantanés, sept après la refonte |
| `kesh-db/src/repositories/journal_entries.rs:1068` (`modification_guard`) | `&mut *conn` | appelée par `update` (`:1529`) et `delete_in_tx` (`:1877`) **sous le verrou** de l'écriture, et par `modification_blocker` (`:1233`) **sur une connexion nue** (C-15-8-24 : « une connexion, pas un `Executor` générique ») |
| `journal_entries.rs:2072` (`reversal_blocker`, enveloppe) | `executor` | délègue |
| `journal_entries.rs:2438` (`reverse_owned_in_tx`) | `&mut **tx` | dans la transaction du geste |
| `settlement_cancellation.rs:107` (file commune des annulations) | `&mut *conn` | prédicteurs (en transaction : `routes/invoices.rs:1440`, `supplier_invoices.rs:1414`, `reconciliation_cancel.rs:220`) et gestes |
| `letterings.rs:522` (`first_document_owner`) | `&mut **tx` | acte 1 du lettrage `Manual` (`create_group_in_tx`, `:711`) **et** de la dissolution (`dissolve_group_in_tx`, `:818`) |
| `kesh-db/tests/letterings.rs:588` | `&mut *conn` | test — **déjà** sur une connexion : compile sans changement |
| `kesh-db/tests/invoice_settlement.rs:579`, `:584` | `&pool` | test |
| `kesh-db/tests/supplier_invoices_repository.rs:298`, `:724`, `:1300` | `&pool` | test |
| `kesh-db/tests/supplier_invoices_repository.rs:746` (`modification_guard`, **transitif**) | `&mut conn` | test sur connexion nue — inchangé |

S'y ajoutent deux **mentions** sans appel, à relire : `kesh-db/tests/journal_entries_modification.rs:104`
(la constante `QUOI_FAIRE` renvoie à `reversal_blockers` pour trier une référence) et
`kesh-api/tests/journal_entry_reversal_e2e.rs:1061` (commentaire). **Six** sites de production directs (dont
l'enveloppe) **plus un** transitif de route (`modification_blocker`) ; **six** appels de test directs dans
**cinq** fonctions et **trois** fichiers — dont **cinq** à adapter (`letterings.rs:588` passe déjà une
connexion ; validation P1, R7) — plus **un** transitif inchangé (`supplier_invoices_repository.rs:746`) ;
**deux** mentions dans deux autres fichiers — **cinq** fichiers de tests en tout.

## Décisions

### D1 — `document_owners`, par lot, une connexion

```rust
/// Le type d'une pièce propriétaire, dans l'ordre de précédence de `reversal_blockers` (rangs 3 à 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DocumentKind { Invoice, CreditNote, SupplierInvoice, Settlement, BankTransaction }

impl DocumentKind {
    /// Le motif de contre-passation que porte ce type — la table de correspondance, écrite UNE fois,
    /// employée par `reversal_blockers` (pour rendre `ReversalBlockerHit`) et par `first_document_owner`
    /// (pour `DbError::LetteringLineOwnedByDocument`).
    pub fn reversal_blocker(self) -> ReversalBlocker;
    /// R5 : ce type fait-il d'une ligne une **ligne de pièce**, non lettrable à la main ? Vrai pour
    /// Invoice, CreditNote, SupplierInvoice, Settlement ; **faux pour BankTransaction**. Employé par
    /// `first_document_owner` et par la 15-1b (`manuallyLetterable`, filtre des propositions) — jamais
    /// une seconde liste.
    pub fn blocks_manual_lettering(self) -> bool;
    /// La valeur sérialisée : `invoice`, `creditNote`, `supplierInvoice`, `settlement`, `bankTransaction` —
    /// celle de `document.type` de la vue (15-1b) et de `documentType` d'audit : `DocumentRef.document_type`
    /// (15-1a2-i) est **typé** `DocumentKind` par cette story et sérialisé par cette méthode (D1 bis).
    pub fn as_str(self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentOwner {
    pub kind: DocumentKind,
    pub id: i64,
    /// Numéro de la pièce quand elle en a un (facture, avoir, facture fournisseur), sinon `None` — lu sur
    /// la MÊME ligne que `id`.
    pub number: Option<String>,
    /// Pour `Settlement` : la facture réglée (`invoice_settlements.invoice_id`) et son numéro ; `None` sinon.
    pub invoice_id: Option<i64>,
    pub invoice_number: Option<String>,
}

pub async fn document_owners(
    conn: &mut MySqlConnection, company_id: i64, entry_ids: &[i64],
) -> Result<BTreeMap<i64, Vec<DocumentOwner>>, DbError>;
```

Dans `crates/kesh-db/src/repositories/journal_entries.rs` (le module que la 15-1b appelle).

- **Forme fixée** (validation P1, R2 = F-4) : **une** instruction par tranche de **500** écritures — un
  `UNION ALL` de **cinq** blocs, un par type, chacun borné par `… IN (…) AND je.company_id = ?` et rendant
  `(entry_id, kind, id, number, invoice_id, invoice_number)` ; dans chaque bloc, le **plus petit `id`** par
  écriture est choisi par une table dérivée `GROUP BY entry_id` (`MIN(id)`) **jointe en retour** sur la table
  de la pièce, si bien que `id` et `number` sortent **de la même ligne** (F-8) ; le bloc fournisseur porte les
  deux colonnes (`purchase_journal_entry_id`, `settlement_journal_entry_id`) dans sa dérivée ; le bloc
  règlement joint `invoices` pour le numéro de la facture réglée. **Le bloc fournisseur** (validation P2,
  F2-2) : sa dérivée est l'`UNION ALL` des **deux** colonnes — `SELECT purchase_journal_entry_id AS entry_id, id
  FROM supplier_invoices WHERE purchase_journal_entry_id IN (…) UNION ALL SELECT settlement_journal_entry_id, id
  FROM supplier_invoices WHERE settlement_journal_entry_id IN (…)` —, puis `GROUP BY entry_id` (`MIN(id)`), puis
  la jointure de retour **sur `id`** ; ⛔ jamais un `CASE` qui choisit **une** colonne par ligne : quand l'achat
  **et** le règlement d'une même facture sont dans le lot, il perdrait l'un des deux (le défaut muet du
  « faux rattachement » : une ligne de règlement fournisseur rendue libre). Liste vide → `Ok(BTreeMap::new())`
  **sans requête**. Doublons dans `entry_ids` : **dédupliqués et triés** (`BTreeSet`) avant découpage — les
  tranches sont alors déterministes (validation P2, R2-5).
- `Vec<DocumentOwner>` **dans l'ordre de précédence** (Invoice, CreditNote, SupplierInvoice, Settlement,
  BankTransaction — l'ordre de `DocumentKind`). Une écriture sans propriétaire est **absente** de la table —
  jamais présente avec un vecteur vide (tranché ; validation P1, R8 = F-9 ; écrit en doc-comment, asserté).
  Une écriture d'une autre société, ou inexistante, est absente elle aussi.
- **Au plus un propriétaire par type et par écriture, le plus petit `id`** — là où la requête d'aujourd'hui
  prend un `LIMIT 1` arbitraire. Écart **nommé** : sur des données **saines** (une pièce par type, l'invariant
  applicatif du modèle réel), rien ne change ; sur des données qui violent cet invariant — deux factures, ou
  deux transactions bancaires, visant une écriture —, le résultat devient déterministe ; et l'identifiant et
  le numéro de la facture fournisseur viennent de la même ligne (F-8).
- ⛔ **`&mut MySqlConnection`, pas un `Executor` générique** (C-15-1b-10) — patron C-15-8-24 de
  `modification_guard` : la fonction enchaîne plusieurs requêtes, ce qu'un exécuteur consommé par valeur
  interdit. Une transaction se passe déréférencée (`&mut **tx`), une connexion se reprête (`&mut *conn`).
- **Les trois méthodes de `DocumentKind`** (validation P1, R4 = F-5 ; C-15-1b-0-2) sont la **source unique** de
  la correspondance type → motif, de la liste R5 et des chaînes sérialisées ; un test à **table fermée** (les
  cinq variants, valeurs écrites à la main) les fige.

### D1 bis — `DocumentRef` typé : plus de seconde source des chaînes d'audit (validation P2, R2-1 = F2-1 ; C-15-1b-0-3)

La 15-1a2-i (mergée **avant**) définit `DocumentRef { document_type: &'static str, … }` et y écrit `"invoice"` ;
la 15-1a2-ii, `"supplierInvoice"`. Ces stories ne pouvaient pas appeler `DocumentKind::as_str`, qui n'existait
pas encore : le littéral était forcé chez elles, et elles le disent (15-1a2-i P3, doc-comment de `DocumentRef`).
**Cette story le résorbe**, faute de quoi « source unique » serait faux dès son merge :
- `DocumentRef.document_type` devient **`DocumentKind`** (`letterings.rs`) ; `audit_details` le sérialise par
  `as_str()` ; les sites de construction passent `DocumentKind::Invoice` / `DocumentKind::SupplierInvoice`. Le
  compilateur ferme alors l'inventaire : aucun littéral ne peut plus y entrer. *(Écartée : remplacer le
  littéral par `DocumentKind::Invoice.as_str()` en gardant `&'static str` — rien n'empêcherait un littéral neuf.)*
- **Le test qui tient les deux d'accord** : les tests d'audit de la 15-1a2-i et de la 15-1a2-ii
  (`audit_details_carry_the_invoice`, `supplier_audit_details_carry_the_invoice`) assertent **en dur**
  `documentType = "invoice"` et `"supplierInvoice"` ; `document_kind_table_is_closed` asserte en dur les mêmes
  valeurs de `as_str()`. Après le typage, l'audit **est** `as_str()` : les deux tables écrites à la main, toutes
  deux vertes **sans modification de leurs assertions**, prouvent que la valeur publiée n'a pas bougé. Un écart
  entre les deux rougit l'une ou l'autre.
- **Relevé au T0** : `git grep -nE '"(invoice|supplierInvoice)"' -- crates/kesh-db/src` — chaque site trié au Dev
  Agent Record (les sites de `DocumentRef` sont rebranchés ; les autres, s'il en est — types d'entité d'audit,
  par exemple —, parlent d'autre chose), compteur des sites rebranchés écrit.

### D2 — `reversal_blockers` réécrit dessus, signature `&mut MySqlConnection`

- `reversal_blockers(conn: &mut MySqlConnection, company_id, id)` : **deux** requêtes — la sienne, réduite
  aux rangs 1, 2 et 8 (`reverses_entry_id`, `reversed_by`, `archived_account_number`), et
  `document_owners(conn, company_id, &[id])` pour les rangs 3 à 7 —, puis la même liste, dans le même
  ordre, avec les mêmes identifiants et étiquettes (numéro de pièce ; `None` pour règlement et transaction).
  `NotFound` inchangé (écriture absente ou d'une autre société).
- `reversal_blocker` (enveloppe) suit la même signature.
- **Le doc-comment « ⛔ Une seule requête » est réécrit** : deux requêtes, **un instantané** dès que
  l'appelant tient une transaction (`REPEATABLE READ` : l'instantané se fige à la première lecture et vaut
  pour la seconde). ⚠️ Cette propriété repose sur le **niveau d'isolation par défaut** d'InnoDB, que Kesh ne
  configure pas (`grep -rn "ISOLATION" crates --include=*.rs` hors tests : aucune sortie ; 15-1b P3, F L-8) —
  écrit tel quel dans le doc-comment.
- **La route** (`routes/journal_entries.rs:472-474`) ouvre **une** transaction de **lecture**
  (`state.pool.begin()`, lectures, puis `rollback`) et y lit les **trois** champs de ce que l'écran peut faire
  — `reversal_blocker(&mut *tx, …)`, `reversed_by(&mut *tx, …)` (exécuteur générique, accepte la connexion) et
  `modification_blocker(&mut tx, …)` — sans quoi les requêtes liraient plusieurs instantanés et un motif
  pourrait s'afficher contre un état qu'aucun instant n'a connu (validation P1, R1 = F-1 ; C-15-1b-0-1).
- **`modification_blocker` prend `tx: &mut Transaction<'_, MySql>`** au lieu de `&MySqlPool` (son seul appelant
  est la route ; son doc-comment « sur une connexion acquise du pool (six lectures enchaînées) » est réécrit :
  « dans la transaction de lecture de l'appelant, un instantané »). **Une transaction, non une connexion**
  (validation P2, F2-6 ; C-15-1b-0-4) : avec `&mut MySqlConnection`, `modification_blocker(&mut
  *state.pool.acquire().await?, …)` compilerait et rouvrirait les instantanés multiples que C-15-1b-0-1 ferme ;
  le type `Transaction` fait du compilateur la garde de ce lecteur-là. Les deux autres lectures de la route
  (`reversal_blocker`, `reversed_by`) gardent leurs signatures (d'autres appelants) : un `acquire()` y resterait
  possible — d'où la **garde lexicale** de la route (AC4, T3), qui refuse tout `acquire()` et tout `&state.pool`
  passé à ces trois lecteurs dans le corps de `get_journal_entry`. `find_by_id` (`:741`, `&MySqlPool`, ses
  propres requêtes) **reste hors** de cette transaction — écart **assumé**, préexistant : l'en-tête de
  l'écriture n'est pas un motif, et le rattacher changerait une signature de plus sans rien fermer de ce que
  la refonte ouvre.
- Tous les autres appelants de production tiennent déjà une transaction (tableau ci-dessus). Les tests sur
  `&pool` passent une connexion acquise (`&mut *pool.acquire().await?`) — plusieurs instantanés y sont
  indifférents (aucun écrivain concurrent dans ces tests).

### D3 — `first_document_owner` en un lot

Un seul appel `document_owners(&mut **tx, company_id, &entrées_distinctes)` au lieu de la boucle ; la
première ligne (dans l'ordre des lignes — `ORDER BY jel.id` des deux lectures d'acte 1 —, comme aujourd'hui)
dont l'écriture a un propriétaire `o` tel que
`o.kind.blocks_manual_lettering()` rend `LetteringLineOwnedByDocument { blocker: o.kind.reversal_blocker(),
document_id, document_label }` avec **les mêmes** valeurs qu'aujourd'hui — aucune liste de quatre motifs
écrite dans le corps. Sous les verrous de l'acte 1 : inchangé (aucun verrou pris). Le doc-comment
(`letterings.rs:507-510`, « rangs 3 à 6 — réutilisés, jamais une seconde liste ») renvoie aux méthodes de
`DocumentKind`.

⚠️ **Changement de comportement nommé** (validation P1, F-10) : l'ancienne boucle faisait remonter
`NotFound` de `reversal_blockers` pour une écriture absente ou d'une autre société ; `document_owners` la
rend absente, et `first_document_owner` rend `None` pour elle. **Inatteignable** pour ses **deux** appelants
(validation P2, F2-5) : les lignes viennent de l'acte 1, lecture bornée par société — `LOCK_LINES_BY_ID_SQL`
(`letterings.rs:193-197`) à la création, `LOCK_LINES_BY_KEY_SQL` (`:202-206`) à la dissolution.

### D4 — La parité se prouve contre un ORACLE INDÉPENDANT (C-15-1b-10)

**Fixture**, montée en **SQL direct** (patron de l'en-tête de `kesh-db/tests/letterings.rs` ; validation P1,
R10) — aucun dépôt, pour pouvoir poser des cumuls que les gestes interdisent (le schéma, lui, les permet :
modèle réel). Elle exerce les **huit** rangs, seuls **et cumulés** : règlement encaissé par rapprochement
(`OwnedBySettlement` **et** `MatchedBankTransaction`) ; facture fournisseur achat **et** règlement ;
contre-passation et contre-passée ; compte archivé ; et (validation P1, R3 = F-2) **une écriture qui porte
les cinq types à la fois** — une facture, un avoir, une facture fournisseur, une ligne `invoice_settlements`
et une transaction bancaire visant la même écriture — pour que toute permutation de deux types soit
observable.

Deux oracles, aucun ne sortant de `document_owners` :

1. **L'ancien code, gelé dans le binaire de test** — module `crates/kesh-db/tests/document_owners/reversal_blockers_frozen.rs`,
   inclus par `#[path]` dans `document_owners.rs` et **dans lui seul** (validation P1, R5 : pas sous
   `tests/common/`, dont `mod.rs` sert cinq binaires de backfill sans `#[allow(dead_code)]` — un module qu'ils
   n'appellent pas y ferait rougir `clippy -D warnings` ; un fichier d'un sous-dossier de `tests/` sans
   `main.rs` n'est pas une cible cargo) : la requête de `reversal_blockers` **copiée telle qu'au commit
   `056997b0`** (SQL et décodage, ordre des rangs), avec un en-tête qui dit d'où elle vient, pourquoi elle ne
   doit **pas** suivre le code de production, et le commit de référence. Pour **chaque** écriture de la
   fixture, `reversal_blockers` (nouveau) == oracle gelé — motif, identifiant, étiquette, ordre — **y compris**
   le cas **`NotFound`** (écriture inexistante, écriture d'une autre société : les deux rendent `NotFound`).
   **Et par lot** (validation P2, F2-2 — le chemin neuf, celui que la story existe pour poser, n'était comparé
   à rien d'ancien) : **un seul** appel `document_owners(&mut conn, c, &toutes_les_écritures_de_la_fixture)`, et
   pour **chaque** écriture, sa projection (type → motif par `DocumentKind::reversal_blocker()`, `id`, étiquette,
   ordre) égale les **rangs 3 à 7** de l'oracle gelé pour cette écriture ; l'absence de la table égale l'absence
   de tout motif de propriété. La fixture place l'**achat et le règlement d'une même facture fournisseur** dans ce
   lot — le cas qu'un bloc fournisseur à une colonne perdrait.
   ⚠️ La fixture ne contient **pas** deux pièces **du même type** sur une écriture (le `LIMIT 1` sans
   `ORDER BY` de l'oracle n'y serait pas déterministe) ; ce cas — deux transactions, deux factures — a son
   test propre, à valeurs écrites.
2. **Des valeurs écrites à la main** : pour une écriture de chaque type **et** pour l'écriture aux cinq types,
   le `Vec<DocumentOwner>` attendu (type, id, numéro, facture du règlement), **dans l'ordre**, est écrit en dur
   dans le test — ce qui garde l'oracle gelé contre un défaut qu'il partagerait avec le nouveau code — ; le cas
   « deux transactions » (et « deux factures ») rend la plus petite ; la table des trois méthodes de
   `DocumentKind` est écrite en dur (cinq lignes).
3. **`first_document_owner`** (validation P1, F-3) : un test par type de R5 (facture, avoir, facture
   fournisseur, règlement) sur `create_group_in_tx` en mode `Manual`, avec `blocker`, `document_id` et
   `document_label` écrits en dur ; deux écritures possédées dans le groupe → la **première ligne** gagne. **Dans
   `crates/kesh-db/tests/letterings.rs`**, à côté de `rank_5_line_owned_by_a_document` (`:526`), sur ses aides
   (`monde`, `ligne`, `faire_facture`, `lettrer`… ; validation P2, R2-2 : `first_document_owner` est privée, on
   ne l'atteint que par les primitives, après les rangs 1 à 4 bis, et ce montage d'environ trois cents lignes est
   privé à ce binaire — un binaire neuf l'aurait recopié). Les pièces que ces aides ne posent pas (avoir,
   facture fournisseur, règlement) le sont par des aides **locales** neuves, en SQL direct, sur le patron de
   `faire_facture` (`:255`). ⚠️ `POST` est retiré : la route n'est pas atteignable depuis `kesh-db/tests`. Déjà
   couverts, **inchangés** et cités : l'exclusion de `BankTransaction` (`a_bank_matched_entry_is_not_a_document`,
   `:549` — une ligne seulement rapprochée ne refuse pas ; la version P1 prévoyait un test neuf qui le
   doublait, retiré) ; le **second appelant**, la dissolution (`dissolution_of_a_reversal_pair_with_a_document_line_is_refused`,
   `:790-805`, qui asserte `document_id` seul — non le motif ; validation P2, R2-8, F2-5).
4. **La portée par société** : `owners_are_scoped_by_company` entre dans les tests de D4 (elle tue la
   mutation (c), F-11).

### D5 — La mutation est ÉPROUVÉE, pas supposée

Au développement, **six** mutations, chacune appliquée puis annulée (fichier **touché** après restauration :
cargo garde sinon le binaire muté) : (a) inverser deux rangs (CreditNote avant Invoice) — rougit sur
l'écriture aux cinq types (valeurs écrites **et** oracle gelé) ; (b) retirer la jointure de la facture d'un
règlement ; (c) retirer le filtre `company_id` de `document_owners` — rougit `owners_are_scoped_by_company` ;
(d) retirer un type de `blocks_manual_lettering`, ou y ajouter `BankTransaction` — rougit la table fermée de
`DocumentKind` et un test de `first_document_owner` (ou `a_bank_matched_entry_is_not_a_document`) ; (e)
(validation P2, F2-2) faire la jointure de retour sur l'**écriture** au lieu de `id`, ou réduire le bloc
fournisseur à **une** colonne (`CASE`) — rougit la comparaison **par lot** de `owners_match_the_frozen_reversal_blockers` ;
(f) (validation P2, F2-6) lire `reversal_blocker` de la route sur un `state.pool.acquire()` au lieu de la
transaction — rougit `get_journal_entry_reads_in_one_transaction`. Chacune doit faire **rougir** au moins un test de D4 ; le
Dev Agent Record nomme, pour chacune, le test qui a rougi. Une mutation qui reste verte est un finding.

## Critères d'acceptation

**AC1** — `document_owners` existe avec la signature, les types et les trois méthodes de `DocumentKind` de
D1 ; **mesure du lot** (validation P1, R2 = F-4 ; protocole fixé en validation P2, R2-4 = F2-3, C-15-1b-0-4) :
1 200 écritures posées par un `INSERT` direct multi-valeurs — dont des écritures **possédées** aux rangs **1,
501 et 1001** de l'ordre trié (D1 : dédupliquées et triées), donc une dans chaque tranche, pour que le test
prouve que toutes les tranches sont lues et que chaque écriture y reçoit **son** propriétaire — sont lues en
**trois** instructions. **Mesure** : delta de `SHOW SESSION STATUS LIKE 'Com_stmt_execute'` lu **avant et
après** l'appel **sur la même connexion** — compteur de session (les autres connexions n'y entrent pas),
**une** unité par exécution d'instruction préparée, quel que soit le nombre de préparations : `document_owners`
passe par `sqlx::query(…)` avec arguments, donc par `COM_STMT_PREPARE` + `COM_STMT_EXECUTE`
(`sqlx-mysql-0.8.6/src/connection/executor.rs:120-154`), et les trois tranches font **deux** textes SQL (500
marqueurs, puis 200) — un compteur de préparations ou `Com_select` mêleraient les deux. ⚠️ La **lecture** du
compteur doit passer par le protocole TEXTE — `sqlx::raw_sql("SHOW SESSION STATUS LIKE 'Com_stmt_execute'")` —,
**jamais** par `sqlx::query(…)`, qui passe en protocole préparé (`sqlx-core/src/query.rs:661`,
`sqlx-mysql/src/connection/executor.rs:119-121`) et incrémenterait le compteur qu'il lit (validation P3 ciblée,
P3C-1). **Étalonnage** au T0, sur la même connexion, en deux mesures : (i) deux lectures successives sans rien
entre elles → delta **0** (la lecture ne se compte pas elle-même) ; (ii) un `sqlx::query("SELECT ?").bind(1)`
entre deux lectures → delta **1**. Si (i) ne vaut pas 0, son delta `c` est un **surcoût additif** de la lecture :
l'attendu devient `3 + c`, jamais un multiple ; si (ii) ne vaut pas 1, le compteur ne mesure pas des exécutions
et le test n'est pas une preuve — **finding**, pas d'ajustement. Les deux mesures sont écrites au Dev Agent Record ;
un delta instable d'un run à l'autre est un **finding**.
Une liste **vide** donne un delta de **0** ; une liste **avec doublons** donne le même résultat et le même
delta que sans eux (validation P2, R2-7) ; une écriture d'une autre société n'a aucun propriétaire ; une écriture
sans propriétaire est **absente** de la table (asserté dans `owners_match_handwritten_expectations`).

**AC2** — `reversal_blockers` et `reversal_blocker` prennent `&mut MySqlConnection` ; leurs résultats sont
**identiques** à l'oracle gelé sur toute la fixture de D4 (1), `NotFound` compris ; **et un appel par lot** de
`document_owners` sur toute la fixture, achat et règlement d'une même facture fournisseur compris, égale les
rangs 3 à 7 de l'oracle pour chaque écriture (D4 (1), validation P2, F2-2) ; leur doc-comment ne dit plus « une
seule requête » et dit l'instantané, et sa dépendance au niveau d'isolation par défaut.

**AC3** — `first_document_owner` fait **un** appel à `document_owners` par groupe (plus de boucle sur
`reversal_blockers` — contrôlé **à la relecture** du corps, déclaré au Dev Agent Record ; validation P2, R2-7) et
décide par `DocumentKind::blocks_manual_lettering` / `reversal_blocker`, pour ses **deux** appelants (création,
dissolution) ; les tests de R5 existants — `kesh-db/tests/letterings.rs:533` (`OwnedByInvoice`, création),
`:802` (dissolution, second appelant : asserte `document_id` seul — validation P2, R2-8),
`supplier_invoices_repository.rs:2568` (`OwnedBySupplierInvoice`) et `a_bank_matched_entry_is_not_a_document`
(`:549`, exclusion de `BankTransaction`) — restent verts **sans modification de leurs assertions**, et les tests
neufs de D4 (3), **dans `letterings.rs`**, couvrent les quatre types de R5 avec leurs valeurs écrites — dont les
deux qu'aucun test n'exerçait (`OwnedByCreditNote`, `OwnedBySettlement`) — et l'ordre des lignes.
*(Les sites `supplier_invoices_repository.rs:298`, `:724`, `:1300` sont des appels directs de
`reversal_blocker(s)`, non des tests de R5 — AC4 ; validation P1, F-3.)*

**AC4** — **Inventaire fermé des appelants** : les six sites de production directs, le transitif
`modification_blocker` et les cinq appels de test à adapter du tableau sont adaptés ; la route lit
`reversal_blocker`, `reversed_by` et `modification_blocker` dans **une** transaction de lecture (D2) ;
`modification_blocker` prend `&mut Transaction<'_, MySql>`. Contrôles : **le workspace compile** (`cargo build
--workspace --all-targets`) — un `&pool` ne se déréférence pas en `&mut MySqlConnection`, c'est le compilateur
qui ferme l'inventaire, non un `grep` (validation P1, R9 = F-11) ; **et la garde de la transaction** (validation
P2, F2-6 ; C-15-1b-0-4) — le compilateur ne voit pas une connexion acquise à côté de la transaction : le test
`get_journal_entry_reads_in_one_transaction` (`kesh-api/src/routes/journal_entries.rs`, `mod tests`) lit le
source du module (`include_str!`), isole le corps de `get_journal_entry` et asserte **un** `pool.begin()`,
**aucun** `acquire()`, et qu'aucun des trois lecteurs ne reçoit `&state.pool` ; éprouvé par la mutation (f).
Écart **assumé et écrit** : `find_by_id` reste sur `&state.pool` (D2), le test le sait.

**AC5** — **Oracle indépendant** (D4) et **mutations éprouvées** (D5, **six**), consignées au Dev Agent Record.

**AC6** — **Aucun changement visible** : aucune route, aucun code d'erreur, aucun texte, aucune clé i18n ;
`CHANGELOG.md`, `api-external.md` et les manuels **inchangés** (refonte interne, sans effet de contrat).
Contrôle **manuel**, consigné au Dev Agent Record (validation P1, R9) : `git diff --stat` sur `docs/`,
`CHANGELOG.md`, `crates/kesh-i18n/`, `frontend/` ne rend rien.

**AC7** — **`DocumentRef` typé** (D1 bis ; validation P2, R2-1 = F2-1 ; C-15-1b-0-3) : `DocumentRef.document_type`
est un `DocumentKind`, sérialisé par `as_str()` dans `audit_details` ; plus aucun littéral `"invoice"` /
`"supplierInvoice"` ne construit un `DocumentRef` (re-grep du T0, sites triés, compteur au Dev Agent Record) ;
`audit_details_carry_the_invoice` (15-1a2-i), `supplier_audit_details_carry_the_invoice` (15-1a2-ii) et
`document_kind_table_is_closed` restent verts **sans modification de leurs assertions** — les deux tables écrites
à la main tiennent ensemble la valeur publiée.

## Tasks

- [ ] **T0** — Re-greper l'inventaire des appelants **et transitifs** (`modification_guard`,
      `modification_blocker`) sur la base réelle du développement (le tableau est relevé sur `056997b0` ; la
      15-1a2 aura pu en ajouter, par exemple dans la file commune) ; tout site neuf entre au tableau et à AC4.
      Re-greper les appelants de `first_document_owner` (`grep -n "first_document_owner" crates/kesh-db/src` ;
      deux sur `056997b0`, `:711` et `:818` ; la 15-1a2-i retouche la dissolution, `dissolve_group_inner`) et
      les littéraux de `DocumentRef` (`git grep -nE '"(invoice|supplierInvoice)"' -- crates/kesh-db/src`, D1
      bis). Étalonner `Com_stmt_execute` (AC1), lecture par `sqlx::raw_sql` — deux mesures, surcoût additif.
- [ ] **T1** (D1, AC1) — `DocumentKind` et ses trois méthodes, `DocumentOwner`, `document_owners` (forme de
      D1), doc-comments.
- [ ] **T2** (D1 bis, D2, D3, AC2, AC3, AC7) — `reversal_blockers` / `reversal_blocker` réécrits ;
      `first_document_owner` en un lot, sur les méthodes de `DocumentKind` ; `DocumentRef.document_type` typé
      `DocumentKind`, sites de construction rebranchés (D1 bis). Doc-comments qui vieillissent avec la refonte
      (validation P1, F-7 ; P2, R2-3 = F2-4), greppés **par la valeur** (`git grep -nE "Une seule requête|exécuteur
      par|LIMIT 1. \*\*sans|rangs 3 à 6|hors transaction|connexion du pool" -- crates/kesh-db/src`) et réécrits :
      `Lecture::Conseil` (`journal_entries.rs:1129`, « sur une connexion du pool, hors transaction » → « dans la
      transaction de lecture de la route » ; la suite, « une lecture verrouillante y attendrait », reste vraie),
      `reversal_blocker` (`journal_entries.rs:2051-2053`), `reversal_blockers`
      (`:2078-2089` : y écrire les deux requêtes et l'instantané), `modification_guard` (`:1025-1029`, « [`reversal_blockers`] prend son exécuteur
      par valeur » ; « [`modification_blocker`] passe la connexion qu'il a acquise »), `modification_blocker`
      (`:1199-1200`), `settlement_cancellation.rs:64-71` (« l'identifiant que rend `reversal_blockers` sort d'un
      `LIMIT 1` **sans `ORDER BY`** » → le plus petit `id` ; l'argument de la requête dédiée tient toujours),
      `letterings.rs:507-510`. Chaque site du grep est trié au Dev Agent Record (sur `056997b0`, « Une seule
      requête » rend aussi `email_templates.rs:137`, `accounts.rs:445`, `invoices.rs:563`, qui parlent d'autre
      chose).
- [ ] **T3** (AC4) — Les appelants : route en **une** transaction de lecture pour trois lectures,
      `modification_blocker` sur `&mut Transaction<'_, MySql>`, garde lexicale de la route
      (`get_journal_entry_reads_in_one_transaction`), tests sur connexion acquise. Mentions : la constante
      `QUOI_FAIRE` de `journal_entries_modification.rs:104` (« trier la référence dans `reversal_blockers` (gel) ou
      la déclarer ici ») est **réécrite** — un type de pièce propriétaire neuf s'ajoute désormais à
      `DocumentKind` / `document_owners` (rangs 3 à 7), non plus à la requête de `reversal_blockers`, réduite
      aux rangs 1, 2 et 8 : « trier la référence dans `DocumentKind` / `document_owners` (propriété) ou dans
      `reversal_blockers` (rangs 1, 2, 8), ou la déclarer ici » (validation P2, R2-6) ;
      `journal_entry_reversal_e2e.rs:1061` (commentaire) relu.
- [ ] **T4** (D4, D5, AC5) — Module gelé, fixture SQL directe (dont l'écriture aux cinq types, et l'achat et le
      règlement d'une même facture fournisseur), comparaison par écriture **et par lot**, valeurs écrites, tests
      de `first_document_owner` dans `letterings.rs`, **six** mutations et leur procès-verbal.

**Tests prévus** (11 neufs) :
- `crates/kesh-db/tests/document_owners.rs` (neuf, `test-schema`) — 5 : `owners_match_the_frozen_reversal_blockers`
  (AC2, D4 (1) : par écriture, `NotFound` compris, **et par lot** sur toute la fixture), `owners_match_handwritten_expectations`
  (D4 (2), dont l'écriture aux cinq types, deux transactions et deux factures sur une écriture, l'absence d'une
  écriture sans propriétaire), `document_kind_table_is_closed` (D1, D4 (2) : les trois méthodes, cinq variants
  écrits à la main), `owners_are_read_by_batches_of_500` (AC1 : `Com_stmt_execute`, liste vide, doublons),
  `owners_are_scoped_by_company` (AC1, D4 (4)). Plus le module `document_owners/reversal_blockers_frozen.rs`
  (oracle, pas un test) ;
- `crates/kesh-db/tests/letterings.rs` — 5 (validation P2, R2-2) : `first_document_owner_names_an_invoice`,
  `…_a_credit_note`, `…_a_supplier_invoice`, `…_a_settlement` (D4 (3) : quatre tests, un par type),
  `first_document_owner_takes_the_first_line` (D4 (3)) ;
- `crates/kesh-api/src/routes/journal_entries.rs` (`mod tests`) — 1 : `get_journal_entry_reads_in_one_transaction`
  (AC4, garde lexicale ; validation P2, F2-6).

Tests existants **adaptés sans changement d'assertion** : `invoice_settlement.rs:579`, `:584`,
`supplier_invoices_repository.rs:298`, `:724`, `:1300` ; `letterings.rs:588` **déjà conforme**. **Cités, inchangés** :
`a_bank_matched_entry_is_not_a_document` (`letterings.rs:549`, exclusion de `BankTransaction` — la version P1
prévoyait `first_document_owner_ignores_a_bank_transaction_only`, qui le doublait : retiré),
`dissolution_of_a_reversal_pair_with_a_document_line_is_refused` (`:790`, second appelant), et, pour AC7, les
deux tests d'audit de la 15-1a2-i et de la 15-1a2-ii.

*(Recompte depuis cette liste : 5 + 5 + 1 = **11 fonctions de test neuves** — le même total qu'en P1, par
coïncidence : un test retiré (doublon), un ajouté (garde de la route) ; **5 appels de test adaptés**, dans
**4 fonctions** — `invoice_settlement.rs:579` et `:584` sont dans la même,
`annuler_l_unique_reglement_remet_la_facture_a_regler` — et **2 fichiers** ; `letterings.rs:588` compile sans
changement (validation P1, R7). ⚠️ À recompter au T0 depuis la source : le nombre de **fonctions** de test
touchées n'est pas celui des **appels**.)*

## Dev Notes

- **Gate `kesh-db` complet, même en cours de boucle** (repository du socle — § « Exception `kesh-db` » du
  `CLAUDE.md`) ; base remise à zéro avant ; E2E complet au dernier commit de code (la route de
  contre-passation est exercée par l'écran).
- **Aucune migration** : P5–P8 sans objet.
- **Modules** — aux deux grains (C-15-1a2-21) : crates `kesh-db`, `kesh-api` = **2** ; modules métier
  `kesh-db/repositories/journal_entries` (lot, `reversal_blockers`, `modification_blocker`, doc-comments),
  `…/letterings` (`first_document_owner`, doc-comment), `…/settlement_cancellation` (doc-comment seul),
  `kesh-api/routes/journal_entries` (une route, sa garde lexicale) = **4**, sous le seuil. *(Validation P2 :
  `DocumentRef` vit dans `…/letterings`, déjà compté ; la garde de la route est un test du module déjà compté —
  aucun module neuf.)* `modification_guard` et
  `reverse_owned_in_tx` passent déjà une connexion ou une transaction déréférencée, et restent tels quels.
- **Ce que la story ne fait pas** : elle n'expose rien à l'API ; la vue et les propositions sont la 15-1b.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Validation P2 — 2026-10-09 (Opus 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, seul remédiateur des fiches de la suite du lettrage, en autonomie)

**Rapports** : `kesh-gate-logs/15-1b-0-validate-p2-R.md` (**0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW**) et `…-F.md`
(**0 CRITICAL, 0 HIGH, 2 MEDIUM, 4 LOW**). Recoupements : R2-1 = F2-1, R2-3 = F2-4, R2-4 = F2-3 ; R2-8 et F2-5
(le second appelant de `first_document_owner`) se recoupent sans s'égaler, comptés à part → **3 MEDIUM
distincts** (R2-1 = F2-1, R2-2, F2-2) et **8 LOW distincts** (R2-3 = F2-4, R2-4 = F2-3, R2-5 à R2-8, F2-5, F2-6).
**Trend** : P1 **0 HIGH / 4 MEDIUM distincts** → P2 **0 HIGH / 3 MEDIUM distincts**.

⚠️ **Signal D5 levé, déclaré, non découpé.** R2-1 = F2-1 et R2-2 sont **nés de la remédiation P1** (lignes
ajoutées par `76e7893a` : le doc-comment de `as_str` qui revendique `DocumentRef`, les six tests de D4 (3) placés
dans un binaire neuf) ; F2-2 l'est **en partie** (la forme du lot, fixée en P1 ; le trou de l'oracle, d'origine).
Ce sont des **recyclages** au sens de l'amendement D5. **Pourquoi pas de découpage** (constat écrit) : les trois
sont **locaux** — une chaîne entre deux fiches que la 15-1b-0, seule à pouvoir l'unifier, reprend ; un
emplacement de tests ; une comparaison qui manquait au banc d'oracle —, sans dispersion : la fiche reste à **4**
modules et **2** crates, n'en gagne aucun (`DocumentRef` vit dans `…/letterings`, déjà compté). Découper
séparerait la refonte de son oracle, qui en est la preuve. Si la P3 trouve un défaut né de **cette**
remédiation, une passe ciblée sur ce commit suffit à le voir (§ « La passe ciblée »).

| finding | sévérité | verdict | où |
|---|---|---|---|
| R2-1 = F2-1 — `as_str` « source unique » alors que `DocumentRef` (15-1a2-i, -ii, mergées avant) écrit les mêmes chaînes en littéraux ; personne ne les rebranche | MEDIUM | **corrigé** : D1 bis — `DocumentRef.document_type` **typé** `DocumentKind` (le compilateur ferme l'inventaire), re-grep au T0, tests d'audit des deux fiches et table fermée de `DocumentKind` tenant ensemble la valeur ; AC7 (C-15-1b-0-3) ; la 15-1a2-i le dit dans son doc-comment | D1, D1 bis, AC7, T0, T2 |
| R2-2 — six tests de `first_document_owner` dans un binaire neuf sans le montage | MEDIUM | **corrigé** : dans `kesh-db/tests/letterings.rs`, sur ses aides, à côté de `rank_5_line_owned_by_a_document` ; `POST` retiré ; doublon de `a_bank_matched_entry_is_not_a_document` retiré (vérifié `letterings.rs:526`, `:549`) | D4 (3), AC3, tests |
| F2-2 — la parité ne passe que par des lots d'une écriture ; bloc fournisseur sous-spécifié | MEDIUM | **corrigé** : comparaison **par lot** sur toute la fixture (rangs 3 à 7 de l'oracle), achat **et** règlement d'une même facture fournisseur dans le lot, dérivée fournisseur en `UNION ALL` des deux colonnes jointe en retour sur `id`, mutation (e) | D1, D4 (1), D5, AC2 |
| R2-3 = F2-4 — doc-comment `Lecture::Conseil` (`:1129`, « hors transaction ») absent de T2 | LOW | **corrigé** (site nommé, jetons `hors transaction|connexion du pool` au grep) | T2 |
| R2-4 = F2-3 — étalonnage de `Com_select` sans protocole ni repli | LOW | **corrigé** : mesure par `Com_stmt_execute` (une unité par exécution préparée, préparations exclues ; sqlx relu, `executor.rs:120-154`), étalonnage par `SELECT ?` lié à sa première exécution, repli `3 × k` écrit, instabilité = finding (C-15-1b-0-4) | AC1, T0 |
| R2-5 — tranches non déterministes | LOW | **corrigé** (`BTreeSet` ; possédées aux rangs 1, 501, 1001) | D1, AC1 |
| R2-6 — `QUOI_FAIRE` à réécrire, non à relire | LOW | **corrigé** (texte cible écrit) | T3 |
| R2-7 — clauses d'AC sans test nommé | LOW | **corrigé** (liste vide et doublons dans `owners_are_read_by_batches_of_500`, absence dans `owners_match_handwritten_expectations`, « un appel » contrôlé à la relecture, déclaré) | AC1, AC3, tests |
| R2-8 — la dissolution couverte par l'existant seul, citation inexacte | LOW | **corrigé** (écrit ; `:802` asserte `document_id` seul) | D4 (3), AC3 |
| F2-5 — second appelant de `first_document_owner` non nommé | LOW | **corrigé** (`:711`, `:818` au modèle et au tableau, D3 cite `:193-197` et `:202-206`, T0 re-grepe) | modèle réel, D3, T0 |
| F2-6 — la transaction de la route gardée par aucun test | LOW | **corrigé** : `modification_blocker(&mut Transaction<'_, MySql>)` (garde du compilateur) **et** garde lexicale `get_journal_entry_reads_in_one_transaction`, éprouvée par la mutation (f) (C-15-1b-0-4) | D2, AC4, D5, T3, tests |

**Reçu de la validation P4 de la 15-1a2-i** (F4-5, porté ici par décision de l'orchestrateur) : c'est le même
défaut que R2-1 = F2-1, vu de l'autre fiche — traité par D1 bis. **Propagation** (valeurs grepées sur les fiches
15-1b-0, 15-1b, 15-1a2-i, -ii, l'index, le registre) : `Com_select`, `document_owners.rs` (six tests),
`ignores_a_bank_transaction_only`, `quatre mutations`, `modification_blocker(&mut MySqlConnection`,
`DocumentKind::as_str` — la 15-1b emploie `as_str` pour `document.type` (inchangé) et ne cite pas
`modification_blocker` : **non modifiée** (validation close). Résidus : Change Logs, prompts versionnés, registre
annoté. **Recompte** (depuis ce fichier) : **7 critères** (AC1–AC7 ; AC7 neuf), **5 tâches** (T0–T4), **11 tests
neufs** (5 + 5 + 1) ; 5 appels de test adaptés dans 4 fonctions ; **six** mutations. Modules : **2** crates, **4**
modules métier. Choix : **C-15-1b-0-3, C-15-1b-0-4** ; C-15-1b-0-1 et C-15-1b-0-2 annotés. Prochaine passe :
**P3** — **ciblée** possible sur ce commit (une lentille) : la remédiation ne change aucune règle métier ; elle
type un champ, déplace des tests, ajoute une comparaison et une garde. Si l'orchestrateur juge que le typage de
`DocumentRef` (deux fiches mergées avant) élargit le périmètre, complète (Sonnet).

### Validation P1 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, seul remédiateur des fiches de la suite du lettrage, en autonomie)

**Rapports** : `kesh-gate-logs/15-1b-0-validate-p1-R.md` (**0 CRITICAL, 0 HIGH, 4 MEDIUM, 6 LOW**) et `…-F.md`
(**0 CRITICAL, 0 HIGH, 5 MEDIUM, 6 LOW**). Recoupements : R1 = F-1, R2 = F-4, R3 = F-2 + F-3, R4 = F-5 ; R6 =
F-6, R8 = F-9, R9 = F-11 → **4 MEDIUM distincts**, **9 LOW distincts**. Première passe : pas de trend. Chaque
finding relu au code (`056997b0`).

| finding | sévérité | verdict | où |
|---|---|---|---|
| R1 = F-1 — `modification_blocker` lit sur une connexion nue (`journal_entries.rs:1203-1208`, `pool.acquire()`), appelée par la même route ; `supplier_invoices_repository.rs:746` absent | MEDIUM | **corrigé** : `modification_blocker(&mut MySqlConnection)`, la route lit trois champs dans **une** transaction de lecture ; `find_by_id` hors, écart assumé (C-15-1b-0-1) ; inventaire avec transitifs | modèle réel, D2, AC4, T3 |
| R2 = F-4 — « trois requêtes » sans compteur ni forme | MEDIUM | **corrigé** : un `UNION ALL` par tranche, delta de `Com_select` sur la même connexion (étalonné au T0), amorçage par `INSERT` direct avec des écritures possédées dans chaque tranche (vérifié : aucun compteur dans `crates/`) | D1, AC1, T0 |
| R3 = F-2 + F-3 — mutation (a) non observable ; `first_document_owner` exercé pour 2 types sur 4 ; `NotFound` absent de l'oracle | MEDIUM | **corrigé** : écriture aux **cinq** types en SQL direct (vérifié : seul `invoice_settlements` est `UNIQUE`, squash `:690`), `NotFound` comparé, six tests de `first_document_owner`, mutation (d) | D4, D5, AC3, tests |
| R4 = F-5 — liste R5 privée ; correspondance type → motif écrite deux fois | MEDIUM | **corrigé** : `DocumentKind::reversal_blocker()`, `blocks_manual_lettering()`, `as_str()` publics, table fermée testée ; la 15-1b les consomme (C-15-1b-0-2) | D1, D3 |
| R5 — `tests/common/` et `dead_code` | LOW | **corrigé** (`tests/document_owners/…` par `#[path]`) | D4 |
| R6 = F-6 — « garanti par les contraintes » faux | LOW | **corrigé** (invariant applicatif ; écart nommé sur données saines) | modèle réel, D1 |
| R7 — citations décalées ; `letterings.rs:588` déjà conforme | LOW | **corrigé** (`:2118`, `:2144` ; 5 appels adaptés, 4 fonctions, 2 fichiers) | modèle réel, tests |
| R8 = F-9 — absente ou vecteur vide ; dérivations | LOW | **corrigé** (absente ; `Debug, Clone, PartialEq, Eq` ; doublons dédupliqués ; module `journal_entries`) | D1 |
| R9 = F-11 — contrôles par `grep` | LOW | **corrigé** (AC4 : le workspace compile ; AC6 : contrôle manuel nommé) | AC4, AC6 |
| R10 — fixture non guidée | LOW | **corrigé** (SQL direct) | D4 |
| F-7 — doc-comments qui vieillissent | LOW | **corrigé** (six sites nommés, grep de la valeur, sites hors lot triés) | T2 |
| F-8 — l'ancienne requête fournisseur lit id et numéro sur deux lignes | LOW | **corrigé** (défaut nommé, la nouvelle lit la même ligne) | modèle réel, D1 |
| F-10 — `NotFound` de l'ancienne boucle | LOW | **corrigé** (changement nommé, inatteignable) | D3 |

**Propagation** : « tests/common/reversal_blockers_frozen », « six appels de test adaptés », « 4 tests neufs »,
« trois mutations », `modification_blocker(pool` grepés sur les fiches 15-1b-0, 15-1b, l'index et le registre.
**Recompte** (depuis ce fichier) : **6 critères** (AC1–AC6), **5 tâches** (T0–T4), **11 tests neufs** ; 5 appels
de test adaptés. Choix : **C-15-1b-0-1, C-15-1b-0-2**. Prochaine passe : **P2, complète, Opus** — la remédiation
change une signature (`modification_blocker`), la forme de la requête et la surface publique de `DocumentKind`.

### Création — 2026-10-09 (Opus 5.5, remédiation de la validation P2 de la 15-1b, en autonomie)

Story **extraite** de la 15-1b (C-15-1b-9) : la refonte du socle que la remédiation P1 de la 15-1b y
avait placée (ancienne T2, ancien test 8, C-15-1b-2), réécrite pour corriger ce que la validation P2 y
a trouvé — signature inapplicable et appelants non inventoriés (R-4 = F-5, MEDIUM), garde de parité verte
par construction (F-4, MEDIUM). **6 critères** (AC1–AC6), **5 tâches** (T0–T4), **4 tests neufs** —
recomptés depuis ce fichier. Choix consignés : C-15-1b-9, C-15-1b-10 (registre). Prochaine passe :
validation **P1** (Sonnet, contexte frais, complète).

- 2026-10-09 — **Validation P3 ciblée** (Sonnet, prompt `56394f14` ; rapport `/home/gcorbaz/devel/kesh-gate-logs/15-1b-0-validate-p3-ciblee.md`) :
  1 MEDIUM, 4 LOW. **P3C-1** (MEDIUM, né de la P2) : lue par `sqlx::query`, la lecture de `Com_stmt_execute` passe en
  protocole préparé et s'incrémente elle-même (delta 4 au lieu de 3), et le repli `3 × k` était faux (surcoût
  ADDITIF) → lecture par `sqlx::raw_sql` (protocole texte), étalonnage en deux mesures (lecture seule → 0 ;
  `SELECT ?` → 1), repli additif `3 + c`, et une étalonnage (ii) ≠ 1 est un finding. Remédiation faite par
  l'orchestrateur, fiche seule. LOW laissés au T0 du développement, écrits ici : P3C-2 (D2 dit la signature de
  `reversal_blocker` gardée alors qu'AC2 la fait passer à `&mut MySqlConnection` : AC2 fait foi), P3C-3 (garde
  lexicale de la route : délimitation du corps, commentaires et `== 1` à écrire sur le patron de `admin.rs:911`),
  P3C-4 (la branche « jointure sur l'écriture » de la mutation (e) peut rester verte sur la fixture : seule la
  branche `CASE` est exigée rouge), P3C-5 (relevé T0 des littéraux : partir de `git grep DocumentRef crates`, le
  compilateur fermant l'inventaire). **Validation close** : la passe ciblée de fin de boucle ne laisse aucun
  correctif de production. Trend : P1 4 MEDIUM → P2 2 MEDIUM (D5 déclaré, non découpé) → P3 ciblée 1 MEDIUM corrigé.
