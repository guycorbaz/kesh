# Story 15.1a2-i : Le lettrage des pièces clientes — une facture soldée est lettrée, et seulement elle

## Status

ready-for-dev *(découpée de la 15-1a2 le 2026-10-09 à la remédiation de sa validation P1 — C-15-1a2-1 ;
**validation P2 (Opus, contexte frais) à mener avant tout développement**)*

## Story

**As a** indépendant, PME ou fiduciaire qui encaisse ses factures dans Kesh,
**I want** que les lignes d'une facture soldée — la créance, ses règlements, son solde, son avoir —
soient lettrées entre elles sans que j'aie à le refaire, et délettrées si un règlement est annulé,
**so that** le grand livre dise ce que disent la fiche facture, la balance âgée et les relances :
une facture est soldée **si et seulement si** son reste dû est nul — dans les limites, nommées, des
périodes closes (P7).

Première des deux sous-fiches de la **15-1a2** (index : `15-1a2-lettrage-des-pieces.md`). **Suppose la
15-1a-i (#587) et la 15-1a-ii (#593) mergées** — base `056997b0`. Ordre : 15-1a-i → 15-1a-ii →
**15-1a2-i** → 15-1a2-ii → 15-1b → 15-1c. ⛔ La **15-1a2-ii** (fournisseurs et rattrapage) suppose
**celle-ci** mergée : elle réutilise sa synchronisation, son évaluation des périodes et son extension
d'audit, et son test d'accord (AC6) compare le rattrapage à la synchronisation livrée ici.

**Numérotation conservée** de la 15-1a2 (P1, P3, P4, P5 ; AC1–AC5, AC8–AC10, AC12) : les renvois des
fiches sœurs (« 15-1a2 P2 », « AC5 de la 15-1a2 », « 15-1a2 P5 ») restent justes. Les numéros neufs
commencent à **P7** et **AC13**. Les éléments partagés avec la 15-1a2-ii (AC8, AC9, AC10, AC12) portent
ici leur **part i**.

## Pourquoi cette story existe

`invoice_settlements` (Story 24-2) est **déjà** un lettrage : chaque ligne y rattache une écriture de
règlement à **une** facture, et c'est l'utilisateur qui l'a décidé — en saisissant le règlement sur la
facture, ou en acceptant une proposition de rapprochement. La migration qui a créé la table le dit :
*« Cette table est aussi le SUBSTRAT DU LETTRAGE »* (`20260827000001_invoice_settlements.sql:15-18`).

Faire relettrer à la main ce que l'utilisateur a déjà apparié serait une double saisie ; laisser le
grand livre muet sur ce qui est soldé ferait **mentir la vue des postes ouverts** (15-1b) sur le cas le
plus fréquent. Ce n'est **pas** un appariement automatique au sens du `CLAUDE.md` : Kesh ne rapproche
aucune donnée entrante, il **recopie au grand livre un rattachement que l'utilisateur a déclaré** (C93).

## Le modèle réel — relevé sur `056997b0`

*(Lignes citées sur `056997b0` ; elles bougeront — **citer et re-greper par le nom de fonction**.)*

**Compte de créance `A`** d'une facture : celui de **la première ligne au débit de son écriture de
vente** — lecture de `invoice_settlements::sale_receivable_account` (`invoice_settlements.rs:540`),
déjà faite par le règlement, le rapprochement et — depuis la **15-6a** (`39b52628`, closes #473) —
par l'avoir (`credit_notes.rs`, étape « (3 bis) Les comptes de la VENTE »). Ce n'est **pas** le réglage
du moment.

| pièce | lignes sur `A` | source |
|---|---|---|
| écriture de vente | **un** débit = TTC arrondi (l'arrondi à 5 centimes est porté par cette ligne) | `generate_invoice_journal_lines(_rounded)`, `invoices.rs` |
| chaque règlement (`bank_transfer`, `internal_account`) | **un** crédit = montant **réglé** (brut ; l'écart d'arrondi éventuel va sur une 3ᵉ ligne, hors `A`) | `invoice_settlements::settlement_journal_lines` |
| solde du reste (`write_off`) | **un** crédit = reste exact | `invoice_settlements::write_off_journal_lines` |
| avoir (total, unique, refusé s'il existe un règlement) | **un** crédit = TTC + arrondi de la facture, **sur `A`** (15-6a) | `credit_notes::generate_credit_note_journal_lines` |

**#474 est fermée** (15-6b `f8b2accd`, 15-6c `803f3e15`) : un règlement ne peut plus viser `A` lui-même.

⚠️ **Données héritées d'avant la 15-6a** : un avoir émis sous la v0.12.x après un changement du réglage
de créance a crédité **un autre compte** que `A`. Son groupe ne se forme pas (P1 ne regarde que `A`) —
c'est la vérité du grand livre ; c'est l'exception (a) d'AC5.

⚠️ **Facture à TTC nul** (finding R12) : elle ne se valide pas — son écriture de vente serait de total
nul, refusée (`CoreError::EntryZeroTotal`, `kesh-core/src/accounting/balance.rs:196`). Elle n'entre
donc jamais dans P1 (« dont l'écriture de vente existe »).

**Écrivains de `invoice_settlements`** — inventaire **fermé**, relevé par
`grep -rn "INTO invoice_settlements\|DELETE FROM invoice_settlements" crates/ --include=*.rs` hors
tests : **un** `INSERT` (`invoice_settlements::create_in_tx`, `invoice_settlements.rs:397`), appelé par
`settle_invoice` (`invoice_settlements_write.rs:66`, appel `:282`), `write_off_invoice` (`:411`, appel
`:619`) et `accept_one_invoice` (`kesh-api/src/routes/reconciliation.rs:1383`, appel `:1820`) ; **un**
`DELETE` (`cancel_settlement_in_tx`, `invoice_settlements_write.rs:781`, `DELETE` `:862`), aussi appelé
par le dé-rapprochement (`reconciliation_cancel.rs`). Hors production : `invoices.rs` en porte deux dans
son `mod tests` (`#[cfg(test)]`), et neuf fichiers de tests en écrivent en SQL brut (F-10, Dev Notes). La
FK `ON DELETE CASCADE` depuis `invoices` ne joue que sur un brouillon (`invoices::delete` refuse une
facture validée).

**Écrivain de `credit_notes`** — **un** `INSERT` (`create_credit_note`, `credit_notes.rs:327`, `INSERT`
`:725`). Aucune route n'annule un avoir.

## Décisions

### P1 — Le groupe `document` d'une facture client

Pour une facture `I` (statut `validated`, ou `cancelled` par avoir) dont l'écriture de vente existe :

- l'**ancre** de `I` est sa ligne de vente sur `A` ;
- `C(I)` = l'ancre, plus les lignes **sur `A`** des écritures de **tous** ses règlements en vigueur
  (lignes de `invoice_settlements`, soldes compris) et de l'écriture de son **avoir** émis
  (`credit_notes.status = 'issued'`).

**Le groupe existe si et seulement si** : `|C(I)| ≥ 2` **et** `Σ(débit − crédit) = 0` sur `C(I)` **et**
`A` est lettrable (`letterings::is_letterable_account`, P3 point 3) **et** au moins une ligne de `C(I)`
est **en période ouverte** (P7). Il porte l'origine `document`.

⚠️ **Le critère est la somme au grand livre, pas le reste dû** — le reste dû est **dérivé** des pièces
(`INVOICE_AMOUNT_DUE_DERIVED_SQL`, `invoice_settlements.rs:120`) et peut dire 0 alors que les lignes ne
s'annulent pas sur `A` (avoir hérité sur un autre compte). Lettrer sur le reste dû violerait la règle du
groupe (somme nulle, 15-1a R3). AC5 vérifie que les deux **s'accordent**, et nomme les cas où ils
divergent.

⚠️ **Données héritées `paid_at` sans règlement** (antérieures à la 24-2) : `|C(I)| = 1` → **pas de
groupe** ; la créance reste ouverte au grand livre, ce qui est **vrai** (aucune écriture d'encaissement
n'existe). Le reste dû dérivé vaut le TTC : les deux côtés s'accordent.

⚠️ **Facture créditée ET réglée** (Reçu point 6 — **tranché**, C-15-1a2-7) : état **hérité** seulement
(l'avoir est refusé sur une facture réglée, `invoice_settlements_write.rs` étape 3,
`SettlementCancelBlocker::InvoiceCredited`). `Σ C(I) ≠ 0` → **aucun groupe** ; la créance, l'avoir et le
règlement restent ouverts au compte débiteurs, **non lettrables à la main** (R5). C'est ce que disent
déjà, depuis la 15-1a-i, le manuel, `api-external.md` et les clés `*-cancel-blocked-credited` (« le
règlement **reste ouvert au compte débiteurs** ») : **aucun de ces sites n'est réécrit**.

### P3 — La synchronisation d'une facture client, idempotente

Dans `crates/kesh-db/src/repositories/letterings.rs` :

```rust
pub async fn sync_invoice_in_tx(
    tx: &mut Transaction<'_, MySql>, company_id: i64, invoice_id: i64,
    held_open_fiscal_year_id: i64, actor: Actor,
) -> Result<SyncOutcome, DbError>;

pub async fn dissolve_invoice_document_group_in_tx(
    tx: &mut Transaction<'_, MySql>, company_id: i64, invoice_id: i64,
    held_open_fiscal_year_id: i64, actor: Actor,
) -> Result<SyncOutcome, DbError>;

pub enum SyncOutcome {
    Unchanged,
    Created { key: i64 },
    Dissolved { key: i64 },
    Recreated { dissolved: i64, created: i64 },
    /// Rien n'est écrit : la règle des périodes l'interdit (P7) …
    AbstainedClosedPeriods,
    /// … ou le compte n'est pas lettrable (P3 point 3).
    AccountNotLetterable,
}
```

`held_open_fiscal_year_id` est l'exercice **ouvert** que l'appelant tient `FOR UPDATE` (mode
`System`, 15-1a-i R7 point 3) ; P4 le nomme pour chaque site. `actor` est l'auteur du geste (P4,
AC10).

**`sync_invoice_in_tx`** :

1. **Découverte, par une lecture VERROUILLANTE** (`… FOR UPDATE`) des lignes sur `A` de l'écriture de
   vente, des écritures des règlements en vigueur et de l'écriture de l'avoir émis — avec leur
   `lettering_key`, `lettering_origin`, exercice et date (finding F-3 : sous `REPEATABLE READ`, une
   lecture ordinaire lirait l'instantané). Rend `C(I)` et l'ancre.
2. **Le groupe existant** (finding R13, défini) : `E` = l'ensemble des `lettering_key` **distincts et non
   nuls** des lignes de `C(I)`. Une ligne de `C(I)` lettrée d'origine `manual` ou `reversal` →
   `DbError::Invariant` (état impossible : une ligne de pièce n'est pas lettrable à la main, 15-1a R5 ;
   une ligne de règlement n'est contre-passée qu'après le retrait de sa ligne `invoice_settlements`),
   **jamais un écrasement**. `|E| > 1` → `DbError::Invariant`.
3. **La cible** `T` : `C(I)` si elle qualifie (P1), sinon aucune. ⛔ **Compte non lettrable** (findings
   R5 = F-2, C-15-1a2-3) : si `A` n'est pas lettrable (`is_letterable_account`, appelé **avant** la
   primitive), pas de cible et **aucune erreur** — `SyncOutcome::AccountNotLetterable` —, comme la
   contre-passation saute un compte non lettrable (`journal_entries.rs:2578`,
   `if !is_letterable_account(…) { continue; }`). **Un règlement, un solde ou un avoir n'échoue JAMAIS à
   cause du lettrage** : la primitive ne doit pas rendre `LetteringAccountNotLetterable` sur ce chemin.
4. `E = {k}` et les lignes de `k` sont exactement `T` → `Unchanged`.
5. `E = {k}` sinon : si aucune ligne **du groupe `k`** n'est en période ouverte (P7) →
   `AbstainedClosedPeriods`, **rien n'est écrit** (ni dissolution, ni création) ; sinon
   `dissolve_group` (mode `System`) puis, si `T`, création.
6. `E = ∅` et `T` → création (mode `System`, origine `document`) ; `E = ∅` sans `T` → `Unchanged`, ou
   `AbstainedClosedPeriods` / `AccountNotLetterable` si c'est la seule raison qui l'empêche.

**`dissolve_invoice_document_group_in_tx`** (appelée **avant** une contre-passation, P4) : étapes 1 et 2 ;
`E = ∅` → `Unchanged` (**no-op**, finding F-11 : l'annulation d'un règlement **partiel** n'a aucun groupe
à défaire, et `dissolve_group_in_tx` rendrait `NotFound`) ; aucune ligne du groupe en période ouverte →
`AbstainedClosedPeriods` ; sinon dissolution (mode `System`).

⚠️ **L'exercice tenu doit couvrir une ligne du groupe** (`check_held_fiscal_year`, sinon
`DbError::Invariant`). Les sites de P4 le garantissent : la ligne de règlement (création, dissolution)
ou d'avoir (création) est dans l'exercice tenu. Un groupe existant qui ne contient **aucune** ligne de
l'exercice tenu et qu'il faudrait défaire sur un chemin de création n'est produit par aucun geste (une
facture soldée n'accepte ni règlement, ni solde, ni avoir) : l'`Invariant` qui en sortirait signale un
défaut.

**L'audit** (findings R6 = F-5, C-15-1a2-5). Les deux primitives de la 15-1a-i gardent leur signature
publique ; leur corps passe dans deux fonctions **privées** du même module (`create_group_inner`,
`dissolve_group_inner`) qui prennent en plus `document: Option<&DocumentRef>` ;
`create_group_in_tx` / `dissolve_group_in_tx` les appellent avec `None` (routes, contre-passation et
tests **inchangés**), la synchronisation avec `Some`. `audit_details` ajoute alors `documentType`
(`"invoice"`), `documentId` et `documentNumber` (`invoice_number`) — et rien quand `None`, si bien que
les `details` des routes et de la contre-passation ne changent pas. ⛔ Les deux appels
`check_rows_affected` exigés « dans chaque primitive » par `letterings_lexical.rs`
(`each_primitive_checks_the_rows_its_update_found`) vivent désormais dans les fonctions `*_inner` : le
test est réaligné sur elles (T4), jamais affaibli.

### P4 — Où la synchronisation est appelée — inventaire FERMÉ

| geste (`056997b0`) | appel | place | exercice tenu (`FOR UPDATE`) | acteur |
|---|---|---|---|---|
| `settle_invoice` (`invoice_settlements_write.rs:66`) | `sync_invoice_in_tx` | après l'`UPDATE invoices SET paid_at …` (`:314`) | `fy` de `find_open_covering_date(settled_on)` (`:243`) — l'exercice de l'écriture de règlement | `Actor { user_id, api_key_id: None }` |
| `write_off_invoice` (`:411`) | `sync_invoice_in_tx` | après l'`UPDATE invoices` (`:639`) | `fy` (`:588`) | idem |
| `accept_one_invoice` (`routes/reconciliation.rs:1383`) | `sync_invoice_in_tx` | **après** l'étape (g) — l'`UPDATE invoices … AND version = ?` et son contrôle `rows_affected` —, avant l'audit « Step 9 » | `fiscal_year` de l'étape (d) (`:1744`) | `Actor { user_id, api_key_id: actor_api_key_id }` |
| `create_credit_note` (`credit_notes.rs:327`) | `sync_invoice_in_tx` | **après** l'étape (11), la bascule `UPDATE invoices SET status = 'cancelled'` (`:772`), avant la relecture (12) | `fy` de la date de l'avoir (`:553`) | `Actor { user_id, api_key_id: None }` |
| `cancel_settlement_in_tx` client (`invoice_settlements_write.rs:781`) — **couvre** le dé-rapprochement (`reconciliation_cancel.rs`) | `dissolve_invoice_document_group_in_tx` | **après** les gardes de l'étape (3), **avant** `reverse_owned_in_tx` (étape (4), `:852`) | l'exercice de l'écriture de règlement, verrouillé à l'étape (2-bis) (`:827` — sa valeur, aujourd'hui jetée, est **gardée**) | `Actor { user_id, api_key_id: None }` |

⛔ **Pourquoi pas dans `create_in_tx`** (finding R1 = F-5, C-15-1a2-6). `invoice_settlements::create_in_tx`
ne reçoit ni acteur ni exercice, et surtout `accept_one_invoice` l'appelle **avant** son contrôle de
version (g) : une synchronisation placée là prendrait les verrous des lignes **avant** la ligne
`invoices` — l'ordre inverse de `settle_invoice` (facture `FOR UPDATE`, puis lignes), d'où un cycle
(finding F-3) — et une modification concurrente sortirait en erreur de lettrage au lieu du refus
`RECONCILIATION_INVOICE_NOT_ELIGIBLE` / `race_during_update`. Placée **après** (g), elle suit la ligne
`invoices` déjà verrouillée par l'`UPDATE` : même ordre que les autres gestes. Les trois appels sont
donc **explicites**, et c'est le test lexical d'AC8 qui ferme l'inventaire.

⛔ **Pas de synchronisation après le `DELETE`** de l'annulation (C-15-1a2-9). Après le retrait d'un
règlement de montant `> 0`, `Σ C(I)` vaut le reste dû, `> 0` : `C(I)` ne qualifie jamais. La
dissolution ① suffit ; les autres règlements partiels redeviennent ouverts avec elle.

⛔ **La contre-passation lettre ensuite ce qui est libre** (15-1a-ii R6) : règlement annulé ↔ son miroir.
La créance **redevient ouverte**.

**`accept_one_invoice` — une erreur de synchronisation est per-proposal** (findings R1, F-3 ;
`CLAUDE.md` § « Pattern batch »). Toute `Err` de `sync_invoice_in_tx` devient un `FailedProposal`
(jamais un `AppError` global) :

| erreur | `error_code` | motif |
|---|---|---|
| `DbError::LetteringConcurrentChange` | `LETTERING_CONCURRENT_CHANGE` | refus métier « réessayez » du socle (15-1a-i R7 point 4) |
| `DbError::Invariant`, et tout refus de lettrage que la synchronisation exclut par construction (`LetteringLineAlreadyLettered`, `LetteringUnbalanced`, `LetteringAccountNotLetterable`, `LetteringAccountsDiffer`, `LetteringTooFewLines`, `NotFound`) | `INTERNAL_ERROR`, précédé d'un `tracing::error!` | défaut structurel — même traitement que l'étape (e) de la même fonction |
| toute autre (`Sqlx`…) | `DATABASE_ERROR` (`details.message`) | comme les autres étapes |

Un interblocage levé **dans** la synchronisation est absorbé en `FailedProposal`, puis le
`ROLLBACK TO SAVEPOINT` échoue (1305) et `accept_batch` rejoue tout le lot (`retry_with`,
`reconciliation.rs` ≈ `:977-1003`) : mécanisme existant, inchangé.

### P5 — Les dissolutions ne butent pas sur un exercice clos

La dissolution ① de P4 est appelée **après** le verrou de l'étape (2-bis) et les refus des rangs 1 et 2
(`settlement_cancel_blocker` ; le rang 2 lit l'exercice de l'écriture de règlement,
`settlement_cancellation.rs:95`) : l'exercice tenu est **ouvert**, et il porte la ligne de règlement
du groupe — condition du mode `System`. Les refus des rangs 3 à 5 viennent **après**, dans le socle
(`reverse_owned_in_tx`) : s'ils parlent, ils annulent la dissolution avec la transaction (finding F-12 :
la phrase « laisserait… rien » est retirée). Ce que l'ordre protège, c'est **la cause du refus rendu** :
une dissolution appelée avant les gardes rendrait un `Invariant` sur un exercice clos au lieu du refus
`FiscalYearClosed`.

Facture de l'exercice N **clos**, règlement et annulation en N+1 : le groupe a une ligne en N+1, tenue et
ouverte → la dissolution passe (AC4).

### P7 — Les périodes closes : la synchronisation s'abstient (C-15-1a2-2)

**Règle.** La synchronisation **ne lettre pas** un groupe dont aucune ligne n'est en période ouverte, et
**ne délettre pas** un groupe dont aucune ligne n'est en période ouverte — la règle du socle (15-1a-i R7,
`LETTERING_ALL_LINES_IN_CLOSED_PERIODS`) que le mode `System` n'évalue pas, la synchronisation l'évalue
**elle-même**, sans verrou. « En période ouverte » : exercice `Open`, aucun exercice postérieur `Closed`,
date strictement postérieure à `companies.books_locked_through`.

**Évaluation** : une fonction du module, `lines_in_open_period(conn, company_id, &lines) -> bool`, qui
lit sans verrou le statut des exercices des lignes, `fiscal_years::find_later_closed` pour chaque
exercice ouvert et la borne, puis appelle **le même** prédicat que le mode `Manual`
(`any_line_in_open_period`, `letterings.rs` — factorisé, jamais recopié). Tolérance : l'exercice tenu
est verrouillé par l'appelant ; la borne se lit ordinairement, avec la même tolérance qu'à la création
d'écriture.

**Ce que la règle atteint réellement**, et ce que voit l'utilisateur :

1. **Création** — un geste vivant ajoute toujours une ligne **en période ouverte** (une écriture ne se
   crée ni sur un exercice clos, ni sous la borne : `PeriodLocked`, `journal_entries.rs:402`). La
   création ne s'abstient donc **jamais** sur un geste ; elle s'abstient sur les pièces **historiques**
   entièrement closes — le rattrapage de la 15-1a2-ii et la synchronisation appelée sur elles (AC6 de la
   15-1a2-ii, AC14 (c) ici). **Ce que voit l'utilisateur** : la facture soldée reste **ouverte** au grand
   livre (la 15-1b la liste, `documentState = nothingDue`). ⛔ Elle n'est **pas** lettrable à la main :
   ses lignes sont des lignes de pièce (15-1a R5, `LETTERING_LINE_OWNED_BY_DOCUMENT`). Elle le reste.
2. **Dissolution** — l'annulation d'un règlement dont **toutes** les lignes du groupe sont sous la borne
   `books_locked_through` (facture et règlement du premier trimestre, verrou au 31.03, annulation en
   avril : le socle l'accepte, l'exercice étant ouvert — `settlement_cancellation.rs`, « le verrou de
   période du jour n'est pas évalué ici »). Le groupe **est gardé** ; la contre-passation laisse la ligne
   de règlement dans son groupe (15-1a-ii R6, « une ligne déjà lettrée garde son groupe ») et le
   **miroir**, daté du jour, reste **ouvert**. **Ce que voit l'utilisateur** : au 31.03, la vue des
   postes ouverts est inchangée (c'est l'objet du verrou) ; aujourd'hui, la créance et le règlement
   annulé sont lettrés ensemble, et le **miroir ouvert porte exactement le montant redevenu dû** — la
   somme des postes ouverts égale le reste dû. Mais la facture **due** a une ligne de vente **lettrée** :
   c'est l'exception nommée d'AC5 et d'AC9.
3. **Après 2, un nouveau règlement** de la même facture : le groupe gardé n'a toujours aucune ligne en
   période ouverte → abstention entière (étape 5 de P3). Le nouveau règlement (ligne de pièce) et le
   miroir restent **ouverts** l'un en face de l'autre, de somme nulle, **ni l'un ni l'autre lettrable à
   la main** (le règlement est une ligne de pièce, R5). Coût assumé et nommé (AC14 (b)).

⚠️ **Le coût du 2 et du 3 est écrit, non réparé.** L'alternative — dissoudre quand même (le mode
`System` le permet) — réécrirait la liste des postes ouverts « au 31.03 » d'un trimestre verrouillé ;
celle de refuser en amont l'annulation d'un règlement de période verrouillée serait un refus neuf sur
un geste existant. Décision de l'orchestrateur (C-15-1a2-2) ; à rouvrir si la recette le demande.

## Pour la 15-1b — ce que cette fiche change à sa D1

*(Section d'information ; la fiche 15-1b n'est **pas** modifiée ici — une remédiation en cours la tient.)*

La D1 de la 15-1b (« sans objet : la 15-1a2 garantit *lettrée `document`* ⇔ *reste dû nul* ») tient
**pour toute facture dont une ligne est en période ouverte**, hors les exceptions nommées d'AC5. Elle ne
tient pas dans deux cas, que la 15-1b doit pouvoir montrer :

- **facture soldée, non lettrée** — pièce historique entièrement close (P7 point 1) ou compte de
  créance non lettrable (AC13) : lignes ouvertes, `documentState = nothingDue` ;
- **facture due, ligne de vente lettrée** — groupe gardé après l'annulation d'un règlement de période
  verrouillée (P7 point 2) : le miroir de l'annulation est la ligne ouverte qui porte le reste dû ;
  après un nouveau règlement (point 3), deux lignes ouvertes de somme nulle.

La balance âgée, les relances et le rapprochement lisent le reste dû : ils ne sont pas touchés. La
fonction `lines_in_open_period` (P7) est celle que la 15-1b peut réutiliser pour son prédicat partagé
(C-15-1b-4).

## Critères d'acceptation

**AC1** — Une facture client entièrement réglée (un, puis **trois** règlements partiels, dont un
`internal_account`) est lettrée `document` : créance + lignes de règlement sur `A`, une seule clé.
Un règlement partiel seul ne lettre **rien**.

**AC2** — Solde du reste (`write_off`, chacune des quatre natures) qui éteint la facture → lettrée,
la ligne de solde comprise. Arrondi à 5 centimes (`round_to_5_centimes`) et règlement
`SettlesWithRounding` → lettrée, la ligne d'écart d'arrondi **exclue** (elle n'est pas sur `A`).

**AC3** — Avoir total sur une facture validée → facture et avoir lettrés `document` (l'avoir crédite
`A`, 15-6a). Avoir **hérité** crédité sur un autre compte (fabriqué en SQL brut, état d'avant la 15-6a)
puis synchronisation → aucun groupe, aucune erreur.

**AC4** — Annulation d'un règlement d'une facture soldée → le groupe `document` est **dissous**, le
règlement et son miroir sont lettrés `reversal`, la créance est **ouverte** (et les autres règlements
partiels aussi). Même résultat par le **dé-rapprochement** d'une facture encaissée par rapprochement.
Annulation d'un règlement **partiel** (aucun groupe) → réussit, aucune dissolution, aucune entrée
`lettering.removed` (F-11). Facture de l'exercice N **clos**, règlement et annulation en N+1 →
**réussit** (P5).

**AC5** — **Accord grand livre ↔ pièce**, sur la **fixture partagée** de T5 (paiement total, partiels,
solde, avoir, annulation, rapprochement, données héritées `paid_at` sans règlement, facture créditée et
réglée héritée) : pour chaque facture validée ou créditée **dont une ligne de `C(I)` est en période
ouverte**, *lettrée `document`* ⇔ *reste dû dérivé nul* — avec **deux** exceptions nommées, et elles
seules : (a) l'avoir **hérité** crédité sur un autre compte que `A` (d'avant la 15-6a) ; (b) le compte
de créance **non lettrable** (AC13). Les pièces sans ligne en période ouverte sont **hors** de ce
critère et vérifiées par AC14. L'héritage `paid_at` sans règlement et la facture créditée et réglée ne
sont **pas** des exceptions (les deux côtés s'accordent : non lettrées, reste dû non nul). Toute autre
divergence fait rougir le test **en nommant la facture**.

**AC8 (part i)** — **Inventaire fermé, tenu par un test lexical** (dans `letterings_lexical.rs`, qui
réutilise `decouper`, `bloc_apres` et `blocs_de_test` — finding R11) :
(a) tout `INSERT INTO invoice_settlements` ou `DELETE FROM invoice_settlements` du code de production
(hors blocs `#[cfg(test)]`) est dans `invoice_settlements.rs` / `invoice_settlements_write.rs` ;
(b) toute fonction de production qui appelle `invoice_settlements::create_in_tx(` appelle **après lui,
dans son propre corps** (borné par `bloc_apres` sur la copie masquée), `sync_invoice_in_tx(` ;
(c) le seul `INSERT INTO credit_notes` de production est dans `create_credit_note`, dont le corps
appelle `sync_invoice_in_tx(` **après** lui ;
(d) le corps de `invoice_settlements_write::cancel_settlement_in_tx` appelle
`dissolve_invoice_document_group_in_tx(` **avant** `reverse_owned_in_tx(`.
Un site neuf fait rougir le test **en le nommant** (fichier, fonction). Le détecteur est éprouvé sur un
source synthétique (patron `the_detector_sees_writes_and_only_writes`).

**AC9 (part i)** — Ajout au test d'invariant de la 15-1a (`lettering_invariants`,
`crates/kesh-db/tests/letterings.rs`) : une ligne d'origine `document` est sur l'écriture de vente, d'un
règlement **en vigueur** ou d'un avoir émis d'une facture client ; une ligne d'une de ces écritures n'est
ni `manual` ni `reversal`. **Exceptions nommées** : la ligne d'un règlement **annulé** (plus possédée :
`reversal` avec son miroir, ou — P7 point 2 — encore dans le groupe `document` gardé, dont **aucune**
ligne n'est alors en période ouverte ; le test vérifie cette condition au lieu d'exempter en bloc).

**AC10 (part i)** — Audit : chaque création/dissolution `document` d'une facture client produit
`lettering.created` / `lettering.removed` (15-1a-i AC10, `fiscalYearId`/`fiscalYearName` par ligne)
dont les `details` portent **en plus** `documentType = "invoice"`, `documentId`, `documentNumber`.
Acteur : l'auteur du geste. ⚠️ **Écart nommé** (comme le lettrage `reversal`, 15-1a-i R3) : les gestes
de dépôt (`settle_invoice`, `write_off_invoice`, `create_credit_note`, `cancel_settlement_in_tx`) ne
reçoivent que l'utilisateur — `api_key_id: None` ; seul `accept_one_invoice` porte la clé. Les `details`
des routes `/letterings` et de la contre-passation sont **inchangés** (aucune clé `document*`).

**AC12 (part i)** — Documentation, **par la valeur** (findings R8, R9 = F-7) :
- `CHANGELOG.md` (`[0.13.0]`, entrée du lettrage, aujourd'hui `:15`) : retirer « Kesh ne lettre pas
  encore de lui-même une facture soldée par ses règlements » ; écrire « une facture client soldée — par
  ses règlements, son solde ou son avoir — est lettrée d'office ; l'annulation d'un règlement la
  délettre ».
- `docs/api-external.md` : `:223` et `:291` — sortir `document` de la réserve (« posé par Kesh sur les
  lignes d'une facture client soldée ; il suit ses règlements et son avoir ») ; `:324` — retirer
  l'annotation « *(aucun groupe `document` n'existe encore)* » et réécrire le libellé du refus (voir le
  message ci-dessous).
- `docs/manual/fr/user-manual.tex` : `:765` (« les encaissements des factures ne se lettrent pas encore »
  → « les encaissements des factures se lettrent d'eux-mêmes avec leur facture ») ; § « Enregistrer et
  annuler un règlement » (`sec:reglement-client`, `:1141`) — un paragraphe *Lettrage* : facture soldée
  lettrée, annulation qui délettre, cas du règlement de période verrouillée (P7 points 2-3), créance
  non lettrable ; § « Avoirs et notes de crédit » (`:1262`, `:1267`) — « l'écriture d'origine reste
  intacte » et « sa propre écriture reste intacte » complétés « hors la marque de lettrage, qui la
  rattache à l'avoir » (Reçu point 20) ; glossaire, entrée *Lettrage* (`:2420-2426`, **coupée sur deux
  lignes** : « Kesh ne lettre / pas encore de lui-même ») — « Kesh lettre de lui-même une facture client
  soldée avec ses règlements, son solde ou son avoir ».
- **Message `LETTERING_IS_DOCUMENT`**, faux pour un groupe facture + avoir (aucun règlement à annuler,
  aucun avoir annulable — R9, C-15-1a2-8) : réécrit **neutre** dans les quatre locales
  (`crates/kesh-i18n/locales/*/messages.ftl:54`) et son repli Rust (`kesh-api/src/errors.rs`, branche
  `DbError::LetteringIsDocument`) : « Ce lettrage est celui d'une pièce : il suit ses règlements et son
  avoir, il ne se défait pas à la main. » (de/it/en traduits).
- **PDF** : `make fr` dans `docs/manual/`, les trois PDF commités ; contrôle **aplati** :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oE "(ne se lettrent pas encore|Kesh ne lettre pas encore)[^.]*\."`
  ne rend **plus rien** (deux lignes aujourd'hui, relevées en validation P1).
- Contrôle de propagation : `git grep -nE "ne se lettrent pas|ne lettre pas encore|réservée? aux lettrages|aucune route ne la rend|n'existe encore|annulez le règlement plutôt" -- CHANGELOG.md docs crates/kesh-i18n crates/kesh-api/src website README.md`
  ne rend plus que les sites **fournisseurs** laissés à la 15-1a2-ii, nommés au Change Log.

**AC13** — **Compte de créance non lettrable** (findings R5 = F-2) : le compte `A` d'une facture validée
est retypé en charge (`Expense`) ou rattaché à un compte bancaire, puis la facture est réglée en entier
→ le règlement **réussit**, aucun groupe, aucune erreur, aucune entrée d'audit de lettrage
(`SyncOutcome::AccountNotLetterable`). Réciproquement, un groupe `document` posé **avant** le retypage
survit (C104) et se **dissout** à l'annulation d'un règlement (la dissolution n'exige pas la
lettrabilité).

**AC14** — **Périodes closes** (P7) : (a) facture et règlement complet datés sous
`books_locked_through`, exercice ouvert ; annulation du règlement → **réussit**, le groupe `document`
est **gardé**, le miroir est **ouvert**, aucune paire `reversal`, aucune entrée `lettering.removed` ;
(b) puis nouveau règlement complet → réussit, **rien** n'est lettré (règlement et miroir ouverts) ;
(c) facture et règlements historiques entièrement sous la borne, **sans** lettrage (marques effacées en
SQL brut) → `sync_invoice_in_tx` rend `AbstainedClosedPeriods` et n'écrit rien.

**AC15** — **Le rapprochement** (`accept_one_invoice`) : (a) une proposition qui solde la facture la
lettre `document`, via l'API (`POST /reconciliation/accept`) ; (b) une modification concurrente de la
facture entre l'instantané et l'étape (g) rend `RECONCILIATION_INVOICE_NOT_ELIGIBLE` /
`race_during_update` **sans** qu'aucune marque ne soit posée ; (c) entrelacement *accept ‖ annulation
d'un règlement de la même facture* (patron `rejeu_interblocage_e2e.rs`) : les deux finissent (succès,
refus per-proposal ou rejeu), et l'état final satisfait AC5 ; (d) une erreur de synchronisation devient
un `FailedProposal` selon la table de P4 — test du mappage sur une fonction pure extraite.

## Tasks

- [ ] **T0** — Relevés au sol sur la base du gate : (a) `EXPLAIN` de la requête de découverte de P3 (accès
      par clé primaire / `idx` des écritures, pas de balayage de `journal_entry_lines`) ; (b) re-greper les
      ancres de P4 par le nom (`grep -nF "pub async fn settle_invoice"`, etc.) sur la base réelle du
      développement ; (c) vérifier que les routes appelantes restent `Rejouee` (`audit_route_registry.rs`) :
      `POST /invoices/{id}/settlements`, `…/settlements/{settlementId}/cancel`, `…/write-off`,
      `POST /credit-notes`, `POST /reconciliation/accept`, `POST /reconciliation/transactions/{id}/cancel`.
- [ ] **T1** (P3, P7) — `letterings.rs` : `SyncOutcome`, `DocumentRef`, `lines_in_open_period`
      (prédicat factorisé), `create_group_inner` / `dissolve_group_inner` (+ `document`), extension de
      `audit_details`, `sync_invoice_in_tx`, `dissolve_invoice_document_group_in_tx` ; doc-comments
      (verrous, abstention, compte non lettrable) ; en-tête du module : les exceptions nommées au lettrage
      gagnent la synchronisation (elle **appelle** la primitive, n'écrit pas la marque).
      Doc-comment de `SettlementCancelBlocker::InvoiceCredited` (`kesh-db/src/errors.rs` ≈ `:350`, « son
      traitement est la 15-1a2 ») réécrit selon C-15-1a2-7 : le règlement reste ouvert, la 15-1a2 l'a tranché.
- [ ] **T2** (P4) — Les cinq appels du tableau, chacun à la place indiquée, avec l'exercice tenu et
      l'acteur ; `cancel_settlement_in_tx` client garde l'`id` d'exercice de l'étape (2-bis).
- [ ] **T3** (P4, AC15) — `accept_one_invoice` : appel après (g), mappage per-proposal (fonction pure
      `lettering_error_to_failed_proposal`, testée).
- [ ] **T4** (AC8 part i) — Tests lexicaux dans `letterings_lexical.rs` ; réalignement de
      `each_primitive_checks_the_rows_its_update_found` sur `*_inner`.
- [ ] **T5** — Tests (liste ci-dessous) ; la **fixture partagée** d'AC5 vit dans
      `crates/kesh-db/tests/common/` (module `lettering_documents`), réutilisée par AC6 de la 15-1a2-ii.
- [ ] **T6** (AC12) — CHANGELOG, `api-external.md`, manuel FR + `make fr` + PDF aplati, message
      `LETTERING_IS_DOCUMENT` (quatre `.ftl` + repli Rust), grep de propagation.

**Tests prévus** (22 neufs, 1 étendu) :
- `crates/kesh-db/tests/lettering_documents.rs` (neuf, `test-schema`) — 15 :
  `full_settlement_letters_sale_and_settlements` (AC1), `three_partials_with_internal_account_letter_on_the_last` (AC1),
  `partial_settlement_letters_nothing` (AC1), `write_off_each_kind_letters` (AC2),
  `rounding_line_is_left_out` (AC2), `credit_note_letters_invoice_and_note` (AC3),
  `legacy_credit_note_on_other_account_forms_no_group` (AC3), `cancel_settlement_dissolves_and_pairs` (AC4),
  `unreconcile_dissolves_and_pairs` (AC4), `cancel_partial_without_group_is_a_noop` (AC4),
  `closed_year_n_settled_and_cancelled_in_n1` (AC4, P5), `ledger_agrees_with_amount_due` (AC5),
  `audit_details_carry_the_invoice` (AC10), `receivable_not_letterable_is_skipped` (AC13, deux volets),
  `locked_period_cancel_keeps_the_group` (AC14 a, b, c) ;
- `crates/kesh-db/tests/letterings.rs` — `lettering_invariants` **étendu** (AC9 part i ; pas un test neuf) ;
- `crates/kesh-db/tests/letterings_lexical.rs` — 3 neufs : `invoice_settlement_writers_stay_in_their_module_and_sync`
  (AC8 a, b), `credit_note_insert_is_followed_by_sync_and_cancel_dissolves_first` (AC8 c, d),
  `the_function_body_detector_sees_calls_and_order` (synthétique) ;
- `crates/kesh-api/tests/reconciliation_e2e.rs` — 2 neufs : `accept_letters_a_fully_settled_invoice` (AC15 a),
  `accept_race_refuses_before_any_lettering` (AC15 b) ;
- `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` — 1 neuf : `accept_and_settlement_cancel_interleave` (AC15 c) ;
- `crates/kesh-api/src/routes/reconciliation.rs` (`mod tests`) — 1 neuf :
  `lettering_errors_map_to_failed_proposals` (AC15 d).

*(Recompte depuis cette liste : 15 + 3 + 2 + 1 + 1 = **22 fonctions de test neuves**, plus **1 test
étendu**.)*

## Dev Notes

- **Gate `kesh-db` complet** (repositories touchés : ciblage interdit, `CLAUDE.md`) ; base remise à zéro
  avant, **sans** redémarrer le conteneur partagé (consignes de l'Epic 15 : `kesh_<clé>`, migrations du
  worktree, seed). E2E complet au dernier commit de code.
- **Aucune migration** dans cette sous-fiche : P8 sans objet ; le rattrapage est à la 15-1a2-ii.
- **Verrous et interblocages** (finding R14, analyse réécrite). La synchronisation s'exécute **après** les
  verrous du geste : la facture (`FOR UPDATE`, ou l'`UPDATE` de (g) pour le rapprochement), l'écriture et
  son exercice. Elle prend ensuite, par sa découverte puis par l'acte 1 de la primitive, les **lignes**
  et **en-têtes** du groupe. Cycles **connus et nommés** par le socle (`letterings.rs` § « Interblocages
  résiduels » ; `journal_entries.rs` ≈ `:2363-2370`, cycle lignes ↔ écriture de la contre-passation) :
  en-tête puis lignes contre lignes puis en-tête ; et le **trou d'`idx_jel_lettering`** pendant la
  dissolution du groupe de plus petite clé, mesuré en T0 de la 15-1a-i (toute insertion de ligne,
  toutes sociétés). **Aucune absence de cycle n'est affirmée** : l'ordre réduit la fréquence, le rejeu
  est la défense — toutes les routes de T0 (c) sont rejouées (`Rejouee`, et `retry_with` pour
  `accept_batch`).
- **Tests existants touchés** (finding F-10, relevés sur `056997b0`, non exécutés) :
  - neuf fichiers écrivent `INSERT INTO invoice_settlements` en SQL brut (`admin_full_import_e2e.rs`,
    `reconciliation_e2e.rs`, `exports_global_e2e.rs`, `journal_entry_reversal_e2e.rs`,
    `reconciliation_repository.rs`, `invoice_write_off.rs`, `invoice_settlement.rs`, plus les `mod tests`
    de `invoice_settlements.rs` et `invoices.rs`) : ils fabriquent des factures « soldées » **sans**
    groupe — cohérent (le SQL brut contourne le geste) ; aucun n'asserte l'absence de marque. À relire
    si l'un rougit ;
  - `reconciliation_cancel.rs` (`reconciliation_cancel_letters_the_reversal_pair`, rapprochement **hors
    facture**) : inchangé ;
  - `journal_entry_reversal_e2e.rs`, `letterings.rs`, `letterings_e2e.rs` : `details` d'audit des routes
    et de la contre-passation **inchangés** (aucune clé `document*` quand `None`) ;
  - `invoice_settlement.rs` (annulations) : le résultat final de R6 ne change pas ; la séquence devient
    dissolution → paire.
- **Dépendances** : 15-1a-i, 15-1a-ii (mergées). 15-6a/b/c/d **mergées** (`39b52628`, `f8b2accd`,
  `803f3e15`, `1ae3963e`) : les avertissements #473/#474 de la fiche d'origine sont retirés.
- **Règle de découpage** : modules **de premier niveau** touchés, comptés par domaine : (1) le lettrage
  (`kesh-db` `letterings.rs` + son message `LETTERING_IS_DOCUMENT` — quatre `.ftl` et un repli Rust),
  (2) les règlements clients (`invoice_settlements_write.rs`), (3) les avoirs (`credit_notes.rs`),
  (4) le rapprochement (`kesh-api` `routes/reconciliation.rs`), (5) la documentation. **Cinq**, au seuil.
  Par fichier de production : `letterings.rs`, `invoice_settlements_write.rs`, `credit_notes.rs`,
  `routes/reconciliation.rs`, `errors.rs` (une chaîne), quatre `.ftl`. `invoice_settlements.rs`,
  `reconciliation_cancel.rs`, `routes/letterings.rs` et `journal_entries.rs` ne changent pas.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Validation P1 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, en autonomie)

Fiche **née** de cette remédiation : découpage de la 15-1a2 (finding F-9, sept modules pour un seuil de
cinq ; décision de l'orchestrateur, C-15-1a2-1). Findings de la passe (deux rapports,
`kesh-gate-logs/15-1a2-validate-p1-{R,F}.md`) : **R** 3 HIGH, 7 MEDIUM, 4 LOW ; **F** 0 CRITICAL, 2 HIGH,
7 MEDIUM, 3 LOW — doublons réunis (R2 = F-6, R3 = F-1, R5 = F-2, R7 = F-4, R8 = F-8, R9 = F-7, R10 = F-13,
R1 ≈ F-5). Bilan complet par finding dans l'index. Portés **ici** : R1/F-5 (signatures, exercice tenu,
acteur, audit — P3, P4), R2/F-6 (périodes — P7, AC5, AC14, « Pour la 15-1b »), R4 (Reçus intégrés —
points 2, 6, 9, 11, 15, 17, 18, 20), R5/F-2 (compte non lettrable — P3, AC13), R6 (couture d'audit — P3,
AC10), R7/F-4 (avoir après la bascule — P4), R8/F-8 (faits périmés : 15-6a–d, ancres sur `056997b0`),
R9/F-7 part i (manuel, PDF, glossaire coupé, `:765`, avoirs « intacte », message `LETTERING_IS_DOCUMENT`
— AC12), R11 part i (lexical — AC8), R12 (TTC nul : réfuté — inatteignable), R13 (groupe existant —
P3), R14 (verrous — Dev Notes), F-3 (cycle d'`accept_one_invoice`, mappage — P4, AC15), F-10 part i
(tests existants — Dev Notes), F-11 (dissolution sans groupe — P3, AC4), F-12 (P5 réécrite). Choix
consignés : C-15-1a2-1 à C-15-1a2-9 (registre). **12 critères** (AC1–AC5, AC8–AC10, AC12–AC15), **7
tâches** (T0–T6), **22 tests neufs + 1 étendu** — recomptés depuis ce fichier. Prochaine passe : P2,
Opus, complète.
