# Story 15.1a2-i : Le lettrage des pièces clientes — une facture soldée est lettrée, et seulement elle

## Status

ready-for-dev *(découpée de la 15-1a2 le 2026-10-09 à la remédiation de sa validation P1 — C-15-1a2-1 ;
validation P2 remédiée le 2026-10-09 — **refus** plutôt qu'abstention au délettrage, C-15-1a2-10 ;
**validation P3 à mener avant tout développement**)*

## Story

**As a** indépendant, PME ou fiduciaire qui encaisse ses factures dans Kesh,
**I want** que les lignes d'une facture soldée — la créance, ses règlements, son solde, son avoir —
soient lettrées entre elles sans que j'aie à le refaire, et délettrées si un règlement est annulé,
**so that** le grand livre dise ce que disent la fiche facture, la balance âgée et les relances :
une facture est soldée **si et seulement si** son reste dû est nul — dans les limites, nommées, des
périodes closes (P7) : une pièce historique entièrement close n'est pas lettrée après coup, et un
lettrage figé par une période close ne se défait pas sans qu'un administrateur la rouvre.

Première des deux sous-fiches de la **15-1a2** (index : `15-1a2-lettrage-des-pieces.md`). **Suppose la
15-1a-i (#587) et la 15-1a-ii (#593) mergées** — base `056997b0`. Ordre : 15-1a-i → 15-1a-ii →
**15-1a2-i** → 15-1a2-ii → 15-1b-0 → 15-1b → 15-1c. ⛔ La **15-1a2-ii** (fournisseurs et rattrapage) suppose
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
son `mod tests` (`#[cfg(test)]`, `:4662` et `:4699`), et **huit** fichiers en écrivent en SQL brut hors
production — sept fichiers de tests et le `mod tests` d'`invoices.rs` (validation P2, R-5 : le `mod tests`
d'`invoice_settlements.rs` n'en porte aucun, sa seule occurrence `:415` est la production ; Dev Notes). La
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
    /// Rien n'est écrit : la règle des périodes interdit de LETTRER (P7 point 1) —
    /// rendu par la synchronisation seule, jamais par la dissolution …
    AbstainedClosedPeriods,
    /// … ou le compte n'est pas lettrable (P3 point 3).
    AccountNotLetterable,
}
```

⛔ **La dissolution ne s'abstient jamais** (C-15-1a2-10, validation P2) : un groupe `document` sans
ligne en période ouverte ne se dissout pas, et le geste qui l'exigerait est **refusé en amont** par le
rang 2 bis de la file commune (P7 point 2) — la dissolution n'est donc appelée que sur un groupe qui a
une ligne en période ouverte, et elle n'évalue pas elle-même la règle des périodes (une seule garde par
motif, patron de `cancel_settlement_in_tx`).

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
5. `E = {k}` sinon — **défensif, inatteignable par un geste** (voir ci-dessous) : si aucune ligne **du
   groupe `k`** n'est en période ouverte (P7) → `AbstainedClosedPeriods`, **rien n'est écrit** ; sinon
   `dissolve_group` (mode `System`) puis, si `T`, création.
6. `E = ∅` et `T` → création (mode `System`, origine `document`) ; `E = ∅` sans `T` → `Unchanged`, ou
   `AbstainedClosedPeriods` / `AccountNotLetterable` si c'est la seule raison qui l'empêche.

**`dissolve_invoice_document_group_in_tx`** (appelée **avant** une contre-passation, P4, **après** les
refus du geste, dont le rang 2 bis) : étapes 1 et 2 ; `E = ∅` → `Unchanged` (**no-op**, finding F-11 :
l'annulation d'un règlement **partiel** n'a aucun groupe à défaire, et `dissolve_group_in_tx` rendrait
`NotFound`) ; sinon dissolution (mode `System`) → `Dissolved`. Elle ne rend jamais
`AbstainedClosedPeriods`.

⚠️ **L'exercice tenu doit couvrir une ligne du groupe** (`check_held_fiscal_year`, sinon
`DbError::Invariant`). Les sites de P4 le garantissent : la ligne de règlement (création, dissolution)
ou d'avoir (création) est dans l'exercice tenu. **Un groupe existant différent de `T` sur un chemin de
création (étape 5) n'est produit par aucun geste** : un groupe `document` n'existe que sur une facture
soldée, qui n'accepte ni règlement, ni solde, ni avoir ; et l'état que la validation P2 avait montré
atteignable — facture **due** dont la ligne de vente est restée lettrée après l'annulation d'un
règlement de période verrouillée, puis déverrouillage et nouveau règlement en N+1 → `Invariant` (R-1,
M-2) — **n'existe plus** : cette annulation est refusée (P7 point 2). L'`Invariant` qui sortirait de
l'étape 5 signale donc un défaut. Vérifié contre le motif « invariant dans le temps » : ni
`unlock_books`, ni `fiscal_years::reopen`, ni la restauration d'une sauvegarde (qui rejoue M1 de la
15-1a2-ii, même règle) ne produisent un groupe `document` sur une facture non soldée.

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
| `cancel_settlement_in_tx` client (`invoice_settlements_write.rs:781`) — **couvre** l'annulation d'un **solde** (`write_off`, une ligne `invoice_settlements` comme une autre) et le dé-rapprochement (`reconciliation_cancel.rs`) | `dissolve_invoice_document_group_in_tx` | **après** les gardes de l'étape (3) — qui refusent désormais aussi le **rang 2 bis** (P7 point 2) —, **avant** `reverse_owned_in_tx` (étape (4), `:852`) | l'exercice de l'écriture de règlement, verrouillé à l'étape (2-bis) (`:827` — sa valeur, aujourd'hui jetée, est **gardée**) | `Actor { user_id, api_key_id: None }` |

**Le dé-rapprochement refuse le rang 2 bis dans SA famille** (C-15-1a2-12). `reconciliation_cancel::cancel_in_tx`
refuse à son étape (4) le rang 2 (`FiscalYearClosed`) en `ReconciliationNotCancellable`, **avant** de défaire
le lien, puis appelle `cancel_settlement_in_tx`. Le rang 2 bis s'ajoute à ce même filtre (`if let
Some((FiscalYearClosed | <rang 2 bis>, _, _))`) : sans lui, le refus remonterait de `cancel_settlement_in_tx`
en `SettlementNotCancellable`, avec le texte « ce règlement » au lieu de « ce rapprochement ». Le
dé-rapprochement d'une écriture **propre** (`ReconciliationKind::Entry`) ne rencontre jamais ce rang : une
écriture qui n'appartient à aucune pièce ne porte pas de lettrage `document`.

⛔ **Pourquoi pas dans `create_in_tx`** (finding R1 = F-5, C-15-1a2-6). `invoice_settlements::create_in_tx`
ne reçoit ni acteur ni exercice, et surtout `accept_one_invoice` l'appelle **avant** son contrôle de
version (g) : une synchronisation placée là prendrait les verrous des lignes **avant** la ligne
`invoices` — l'ordre inverse de `settle_invoice` (facture `FOR UPDATE`, puis lignes), d'où un cycle
(finding F-3) — et une modification concurrente sortirait en erreur de lettrage au lieu du refus
`RECONCILIATION_INVOICE_NOT_ELIGIBLE` / `race_during_update`. Placée **après** (g), elle suit la ligne
`invoices` déjà verrouillée par l'`UPDATE`. ⚠️ Ce n'est **pas** l'ordre des autres gestes (validation P2,
L-2) : `accept_one_invoice` insère l'écriture et la ligne `invoice_settlements` (`:1806`, `:1820`)
**avant** de prendre la facture (`:1930` ; `:1913-1914` « Pas de `FOR UPDATE` sur la facture en amont »),
quand `settle_invoice`, `write_off_invoice` et `cancel_settlement_in_tx` tiennent la facture d'abord.
Le cycle *accept ‖ règlement ou solde de la même facture* — la découverte `FOR UPDATE` de ceux-ci lit
`invoice_settlements` de la facture et attend la ligne non commitée du rapprochement, qui attend la
facture — est **nommé** (Dev Notes) et défendu par le rejeu. Les trois appels sont **explicites**, et
c'est le test lexical d'AC8 qui ferme l'inventaire.

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
| `DbError::Invariant`, `DbError::LetteringConcurrentChange`, et tout refus de lettrage que la synchronisation exclut par construction (`LetteringLineAlreadyLettered`, `LetteringUnbalanced`, `LetteringAccountNotLetterable`, `LetteringAccountsDiffer`, `LetteringTooFewLines`, `NotFound`) | `INTERNAL_ERROR`, précédé d'un `tracing::error!` | défaut structurel — même traitement que l'étape (e) de la même fonction |
| toute autre (`Sqlx`…) | `DATABASE_ERROR` (`details.message`) | comme les autres étapes |

⛔ **`LETTERING_CONCURRENT_CHANGE` n'est pas un code per-proposal** (validation P2, M-4 ; C-15-1a2-14). Il
naît d'un seul site : l'`UPDATE` final de la primitive, dont le compte de lignes trouvées
(`CLIENT_FOUND_ROWS`) passe par `kesh_core::lettering::check_rows_affected` (`letterings.rs:739`, `:850`).
Or cet `UPDATE` vise les lignes **tenues `FOR UPDATE` par l'acte 1 de la même transaction** : aucune
transaction concurrente ne peut les modifier entre les deux, et le compte est égal par construction. Le
code est donc **inatteignable** sur ce chemin ; s'il sortait, ce serait un défaut — `INTERNAL_ERROR`, déjà
libellé à l'écran (`failed-proposal-label.ts:152`). Conséquence : `failed-proposal-label.ts`, son décompte
de **28** codes et son test restent inchangés, et `api-external.md` § rapprochement aussi. Il en va de même
pour les routes de règlement, de solde, d'annulation et d'avoir (validation P2, L-3) : elles ne peuvent
pas rendre ce 409 ; rien n'est ajouté à leurs tableaux de refus.

Un interblocage levé **dans** la synchronisation est absorbé en `FailedProposal`, puis le
`ROLLBACK TO SAVEPOINT` échoue (1305) et `accept_batch` rejoue tout le lot (`retry_with`,
`reconciliation.rs` ≈ `:977-1003`) : mécanisme existant, inchangé.

### P5 — Les dissolutions ne butent pas sur un exercice clos

La dissolution ① de P4 est appelée **après** le verrou de l'étape (2-bis) et les refus des rangs 1, 2 et
**2 bis** (`settlement_cancel_blocker` ; le rang 2 lit l'exercice de l'écriture de règlement,
`settlement_cancellation.rs:85-100` ; le rang 2 bis, P7 point 2) : l'exercice tenu est **ouvert**, il
porte la ligne de règlement du groupe — condition du mode `System` —, et le groupe a une ligne en période
ouverte. Les refus des rangs 3 à 5 viennent **après**, dans le socle
(`reverse_owned_in_tx`) : s'ils parlent, ils annulent la dissolution avec la transaction (finding F-12 :
la phrase « laisserait… rien » est retirée). Ce que l'ordre protège, c'est **la cause du refus rendu** :
une dissolution appelée avant les gardes rendrait un `Invariant` sur un exercice clos au lieu du refus
`FiscalYearClosed`.

Facture de l'exercice N **clos**, règlement et annulation en N+1 : le groupe a une ligne en N+1, tenue et
ouverte → la dissolution passe (AC4).

### P7 — Les périodes closes : lettrer s'abstient, délettrer est refusé (C-15-1a2-2, révisée par C-15-1a2-10)

**Règle.** « En période ouverte » : exercice `Open`, aucun exercice postérieur `Closed`, date strictement
postérieure à `companies.books_locked_through` — la règle du socle (15-1a-i R7, C113,
`LETTERING_ALL_LINES_IN_CLOSED_PERIODS`), que le mode `System` n'évalue pas. Elle joue des deux côtés,
mais **pas de la même façon** :

- **lettrer** une pièce dont aucune ligne n'est en période ouverte : la synchronisation **s'abstient**
  (`AbstainedClosedPeriods`, rien n'est écrit) — un geste qui crée ne doit jamais échouer à cause du
  lettrage (P3 point 3) ;
- **délettrer** un groupe `document` dont aucune ligne n'est en période ouverte : **le geste qui
  l'exigerait est refusé** — rang **2 bis** de la file commune des annulations (point 2).

**Évaluation — UNE seule factorisation, publique** (C-15-1a2-18 ; validation P2, R-6, L-1 ; reprise par la
15-1b) : dans `letterings.rs`,

```rust
/// L'état des exercices et la borne, lus SANS verrou — la règle des périodes hors du mode `Manual`.
pub struct OpenPeriodRule { /* privés : exercices (open, later_closed) par id, locked_through */ }
pub async fn open_period_rule(conn: &mut MySqlConnection, company_id: i64, fiscal_year_ids: &[i64])
    -> Result<OpenPeriodRule, DbError>;
impl OpenPeriodRule {
    pub fn line_in_open_period(&self, fiscal_year_id: i64, entry_date: NaiveDate) -> bool;
}
/// `open_period_rule` puis « au moins une ligne en période ouverte ».
pub async fn lines_in_open_period(conn: &mut MySqlConnection, company_id: i64,
    lines: &[(i64, NaiveDate)]) -> Result<bool, DbError>;
```

`open_period_rule` lit le statut des exercices nommés, `fiscal_years::find_later_closed` pour chaque
exercice ouvert, et la borne. `line_in_open_period` et le `any_line_in_open_period` du mode `Manual`
(`letterings.rs:494`, dont l'état des exercices est lu sous verrou) appellent **le même** prédicat par
ligne, privé — factorisé, jamais recopié. Elle **rend un `Result`** (elle lit la base) et ses types sont
**publics** : la 15-1b l'emploie telle quelle pour `inOpenPeriod` et le filtre de ses propositions, sans
seconde factorisation. Tolérance : l'exercice de l'écriture examinée est verrouillé par le geste ; la
borne se lit ordinairement, avec la même tolérance qu'à la création d'écriture (`lock_books` pris entre
la lecture et le `COMMIT` : voir point 2).

**Ce que la règle atteint réellement**, et ce que voit l'utilisateur :

1. **Création — abstention.** Un geste vivant ajoute toujours une ligne **en période ouverte** (une
   écriture ne se crée ni sur un exercice clos, ni sous la borne : `PeriodLocked`,
   `journal_entries.rs:402`). La création ne s'abstient donc **jamais** sur un geste ; elle s'abstient
   sur les pièces **historiques** entièrement closes — le rattrapage de la 15-1a2-ii et la
   synchronisation appelée sur elles (AC6 de la 15-1a2-ii, AC14 (c) ici). **Ce que voit
   l'utilisateur** : la facture soldée reste **ouverte** au grand livre (la 15-1b la liste,
   `documentState = nothingDue`). ⛔ Elle n'est **pas** lettrable à la main : ses lignes sont des lignes
   de pièce (15-1a R5, `LETTERING_LINE_OWNED_BY_DOCUMENT`). Elle le reste — jusqu'à ce qu'un
   administrateur rouvre la période : la synchronisation ne repasse pas d'elle-même, mais un rejeu de M1
   à l'import d'une sauvegarde la lettrerait alors (15-1a2-ii, P6).
2. **Dissolution — refus, rang 2 bis.** L'annulation d'un règlement, d'un solde (`write_off`) ou d'un
   rapprochement de facture dont le groupe `document` n'a **aucune** ligne en période ouverte — facture
   et règlement du premier trimestre, verrou au 31.03, annulation en avril ; ou groupe entièrement dans
   un exercice clos, ou suivi d'un exercice clos — est **refusée**. Aujourd'hui le socle l'accepte : le
   rang 2 ne lit que `fy.status == "Closed"` de l'écriture de règlement (`settlement_cancellation.rs:85-100`)
   et aucun rang ne compare la **date** du règlement à la borne (la contre-passation, datée du jour,
   franchit le verrou). Le rang neuf :
   - **variante** `SettlementCancelBlocker::DocumentLetteringInClosedPeriods` (`kesh-db/src/errors.rs`),
     placée **entre** `FiscalYearClosed` (rang 2) et `MatchedBankTransaction` (rang 3) — d'où « 2 bis »
     (C-15-1a2-12). Après le rang 2 : un exercice clos se nomme par son propre motif, de même remède.
     Avant le rang 3 : annoncer « annulez d'abord le rapprochement » serait vain, le dé-rapprochement
     étant refusé par ce même rang ;
   - **évalué dans la file commune** `settlement_entry_cancel_blocker`, sur l'écriture examinée : « une
     ligne de cette écriture appartient à un groupe d'origine `document` dont aucune ligne n'est en
     période ouverte » — par une fonction publique de `letterings.rs`,
     `document_group_frozen_by_periods(conn, company_id, entry_id) -> Result<Option<i64>, DbError>` (la
     clé du premier tel groupe), qui s'appuie sur `open_period_rule`. ⛔ Une seule évaluation, deux
     lecteurs : le **prédicteur** (l'écran masque le bouton et dit le motif, `cancelBlockedBy`) et le
     **geste** (`cancel_settlement_in_tx` étape (3) l'ajoute aux rangs qu'il refuse ;
     `reconciliation_cancel::cancel_in_tx` étape (4) aussi, dans sa famille — P4). La file étant
     commune, les deux gestes fournisseurs en héritent : la 15-1a2-ii n'a qu'à ajouter le rang aux
     refus de `supplier_invoices::cancel_settlement_in_tx` et `cancel_in_tx` ;
   - **code** : `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, réemployé (C-15-1a2-11) — `SettlementCancelBlocker::code`
     pose que « tous ces codes réemploient ceux d'états du monde déjà nommés » ; « toutes les lignes du
     groupe sont en période close » est l'état que ce code nomme déjà. Statut **409**
     (`SettlementNotCancellable`, `ReconciliationNotCancellable`, `SupplierInvoiceNotCancellable` le
     rendent tous en `CONFLICT`) ;
   - **remède écrit**, comme le rang 2 : un **administrateur** déverrouille la période
     (`POST /companies/current/books-lock/release`, `companies::unlock_books`, motif obligatoire) ou
     **rouvre** les exercices clôturés concernés, du plus récent au plus ancien (`fiscal_years::reopen`) ;
     l'annulation passe alors, avec dissolution et paires `reversal` (AC4) ;
   - **textes** : une clé par famille, quatre locales — `settlement-cancel-blocked-lettering-closed`
     (règlement et solde, client **et** fournisseur : la clé de la queue est partagée),
     `reconciliation-cancel-blocked-lettering-closed`, `supplier-invoices-cancel-blocked-lettering-closed` —
     et leurs replis Rust (`kesh-api/src/errors.rs`), **mot pour mot** le FTL fr-CH ; l'écran (AC17).
   **Ce que voit l'utilisateur** : au 31.03, rien ne bouge — ni la vue des postes ouverts, ni la pièce ;
   la fiche facture n'offre pas l'annulation et dit pourquoi, et qui peut la débloquer. **Il n'existe
   plus de groupe gardé** : aucune facture due n'a de ligne de vente lettrée, AC5 et AC9 n'ont plus
   d'exception de période, et l'`Invariant` que la validation P2 avait atteint après un déverrouillage
   (R-1, M-2) n'a plus d'état qui le produise (P3).
   ⚠️ **Tolérance résiduelle, nommée** : la règle se lit sans verrou sur la borne et sur les exercices
   autres que celui de l'écriture examinée. Un `lock_books` (ou une clôture d'un autre exercice du groupe)
   validé entre la lecture du rang 2 bis et le `COMMIT` laisse passer une dissolution dont toutes les
   lignes sont, au `COMMIT`, en période close — la même tolérance qu'une écriture créée pendant la pose
   du verrou, que le socle assume déjà. C'est le **seul** chemin qui dissout un groupe hors période ; il
   n'en reste aucun par un geste séquentiel (vérifié : `dissolve_group_in_tx` en mode `Manual` évalue R7 ;
   les dissolutions `System` sont celles de P3, gardées par ce rang ou défensives ; la contre-passation et
   le rattrapage ne dissolvent rien).

**Historique de la décision — écrit tel qu'il s'est déroulé** (C-15-1a2-2, révisée deux fois) :
1. *Validation P1* : **abstention partout** — lettrer comme délettrer. Elle produisait le « groupe gardé » :
   une facture due à ligne de vente lettrée, des paires ouvertes à jamais, et un `Invariant` (500) au
   premier règlement après un déverrouillage (validation P2 : R-1, R-2, M-1 à M-3 ; 15-1a2-ii R-1, R-6,
   F2-1 HIGH ; 15-1b R-1/F-1 HIGH).
2. *Révision 1, écartée* : **délettrer toujours** (dissoudre sans évaluer la règle, le mode `System` le
   permet). Elle contredit C113 — la règle des périodes fait entrer le verrou dans « période ouverte »
   pour que la vue « au 31.03 » d'un trimestre verrouillé ne soit plus réécrite — et le manuel livré,
   `user-manual.tex:577-579` : « un groupe dont toutes les lignes sont dans la période verrouillée (ou
   dans des exercices clôturés) ne se lettre ni ne se délettre ».
3. *Révision 2, retenue* : **refus** du geste qui exigerait de délettrer (C-15-1a2-10). L'objection écrite en
   P1 contre ce choix — « un refus neuf sur un geste existant » — pèse peu : la v0.13.0 n'est pas taguée,
   aucune comptabilité réelle n'est encore tenue dans Kesh (`CLAUDE.md`), et le refus a la forme exacte du
   rang 2 existant (même file, même remède administratif).

## Pour la 15-1b — ce que cette fiche change à sa D1

*(Section d'information. **Intégrée à la fiche 15-1b par la même remédiation** — validation P2 des trois
fiches, menée par un seul remédiateur pour qu'elles cessent de se contredire ; la D1, la définition de la
stabilité, AC12 et « Pour la 15-1c » de la 15-1b sont réécrits en conséquence.)*

La D1 de la 15-1b (« sans objet : la 15-1a2 garantit *lettrée `document`* ⇔ *reste dû nul* ») tient
**pour toute facture dont une ligne de `C(I)` est en période ouverte**, hors les deux exceptions nommées
d'AC5 (avoir hérité sur un autre compte ; compte de créance non lettrable). Hors de ce périmètre, **un
seul** cas atteignable : la **facture soldée non lettrée** d'une pièce historique entièrement close
(P7 point 1) — lignes ouvertes, `documentState = nothingDue`, `manuallyLetterable = false` (R5). Le cas
du compte de créance non lettrable (AC13) **n'est pas montrable par la vue** : elle répond 409
`LETTERING_ACCOUNT_NOT_LETTERABLE` sur un tel compte (15-1b AC7, C-15-1b-1) ; il se lit au grand livre.
Le cas « facture due, ligne de vente lettrée » que la version P1 annonçait **n'existe plus** (refus, P7
point 2).

**Stabilité « au X »** (pour la définition de la 15-1b) : pour un `X` ≤ `books_locked_through`, ou situé
dans un exercice clos ou suivi d'un exercice clos, aucune ligne datée ≤ X n'entre ni ne sort d'un groupe
entièrement daté ≤ X — lettrage et délettrage manuels exigent une ligne en période ouverte (R7), la
synchronisation s'abstient de lettrer (point 1), le geste qui délettrerait est refusé (point 2), et tout
groupe créé par un geste contient une ligne en période ouverte, donc datée après X. La liste « au X » ne change donc que si un
**administrateur** déverrouille (`unlock_books`) ou rouvre un exercice (`fiscal_years::reopen`), ou
restaure une sauvegarde — sous la tolérance nommée au point 2.

La balance âgée, les relances et le rapprochement lisent le reste dû : ils ne sont pas touchés. Le
prédicat des périodes est `open_period_rule` / `OpenPeriodRule::line_in_open_period` /
`lines_in_open_period` (P7) : la 15-1b l'emploie tel quel, sans seconde factorisation (C-15-1a2-18,
C-15-1b-4 révisée).

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
règlement et son miroir sont lettrés `reversal` (et, pour un règlement `internal_account` sur un compte
lettrable, sa ligne de contrepartie avec la sienne — R6), la créance est **ouverte** (et les autres
règlements partiels aussi). Même résultat par l'annulation du **solde** (`write_off`) qui éteignait la
facture (validation P2, L-5) et par le **dé-rapprochement** d'une facture encaissée par rapprochement.
Annulation d'un règlement **partiel** (aucun groupe) → réussit, aucune dissolution, aucune entrée
`lettering.removed` (F-11). Facture de l'exercice N **clos**, règlement et annulation en N+1 →
**réussit** (P5 : le groupe a une ligne en N+1, en période ouverte).

**AC5** — **Accord grand livre ↔ pièce**, sur la **fixture partagée** de T5 (paiement total, partiels,
solde, avoir, annulation, rapprochement, données héritées `paid_at` sans règlement, facture créditée et
réglée héritée, **pièce soldée puis passée sous la borne** — validation P2, R-2) : pour chaque facture
validée ou créditée **dont une ligne de `C(I)` est en période ouverte**, *lettrée `document`* ⇔ *reste dû
dérivé nul*, où **« lettrée `document` » signifie : les lignes de `C(I)` forment exactement un groupe, et
il est d'origine `document`** (prédicat testé, écrit dans le test) — avec **deux** exceptions nommées, et
elles seules : (a) l'avoir **hérité** crédité sur un autre compte que `A` (d'avant la 15-6a) ; (b) le
compte de créance **non lettrable** (AC13). Le refus du rang 2 bis (P7 point 2) ne laisse aucune troisième
exception : aucun geste ne produit une facture due dont une ligne de `C(I)` est lettrée. Les pièces sans
ligne en période ouverte sont **hors** de ce critère et vérifiées par AC14. L'héritage `paid_at` sans
règlement et la facture créditée et réglée ne sont **pas** des exceptions (les deux côtés s'accordent :
non lettrées, reste dû non nul). Toute autre divergence fait rougir le test **en nommant la facture**.

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
ni `manual` ni `reversal`. La ligne d'un règlement **annulé** n'est plus possédée (sa ligne
`invoice_settlements` est retirée) et n'est **jamais** `document` : `reversal` avec son miroir, ou libre si
`A` n'est plus lettrable (R6 saute un compte non lettrable, C104). ⛔ **Aucune exception de période** : le
« groupe gardé » de la version P1 n'existe plus (P7 point 2) — l'invariant ne décrit donc que des états
que les gestes produisent, à toute date, déverrouillage ou réouverture compris.

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
- `docs/api-external.md`, tableaux des refus de `POST /invoices/{id}/settlements/{settlementId}/cancel`
  (≈ `:377`) et de `POST /reconciliation/transactions/{id}/cancel` (≈ `:461`) : une ligne « Règlement
  (rapprochement) lettré avec sa facture dans une période close — période verrouillée, exercice clôturé
  ou suivi d'un exercice clôturé ; un administrateur déverrouille ou rouvre » | `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`
  | `409`, placée **après** `FISCAL_YEAR_CLOSED` et **avant** `MATCHED_BANK_TRANSACTION` (l'ordre des rangs).
  Les tableaux fournisseurs sont à la 15-1a2-ii.
- `docs/manual/fr/user-manual.tex` : `:765` (« les encaissements des factures ne se lettrent pas encore »
  → « les encaissements des factures se lettrent d'eux-mêmes avec leur facture ») ; § « Enregistrer et
  annuler un règlement » (`sec:reglement-client`, `:1141`) — un paragraphe *Lettrage* : facture soldée
  lettrée, annulation (et annulation du solde) qui délettre, **refus** de l'annulation quand le lettrage
  est figé par une période close et son remède (P7 point 2), pièce historique close restée ouverte (P7
  point 1), créance non lettrable ; § du verrou de période (`sec:verrou-periode`, `:562`, après
  `:577-579`) — une phrase : « l'annulation d'un règlement ou d'un rapprochement dont le lettrage s'est
  figé avec la période est refusée ; un administrateur déverrouille la période pour la permettre » ; le §
  du dé-rapprochement (re-greper « rapprochement » dans le chapitre des imports bancaires) — même refus,
  même remède ; § « Avoirs et notes de crédit » (`:1262`, `:1267`) — « l'écriture d'origine reste
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
- `CHANGELOG.md`, même entrée : « l'annulation d'un règlement, d'un solde ou d'un rapprochement dont le
  lettrage est figé par une période close est refusée (`409 LETTERING_ALL_LINES_IN_CLOSED_PERIODS`) ; un
  administrateur déverrouille la période ou rouvre l'exercice ».
- Contrôle de propagation, **dans les quatre langues** (validation P2, R-9) : `git grep -nE "ne se lettrent pas|ne lettre pas encore|réservée? aux lettrages|aucune route ne la rend|n'existe encore|annulez le règlement plutôt|Stornieren Sie die Zahlung, statt|annullate il pagamento invece|cancel the settlement rather than" -- CHANGELOG.md docs crates/kesh-i18n crates/kesh-api/src website README.md`
  ne rend plus que les sites **fournisseurs** laissés à la 15-1a2-ii, nommés au Change Log (les trois
  jetons de/it/en relevés en validation P2 au `messages.ftl:54` de leur locale disparaissent avec la
  réécriture du message).

**AC13** — **Compte de créance non lettrable** (findings R5 = F-2) : le compte `A` d'une facture validée
est **rattaché à un compte bancaire** (voie de la fixture : un compte d'actif y est admis) — ou retypé
en charge (`Expense`), ce qui exige de lui retirer d'abord tout rôle (`check_role_account_type`,
`accounts.rs:72-97`) et de confirmer le retypage d'un compte mouvementé (`confirm_retype`, Story 25-2-a ;
validation P2, R-7) —, puis la facture est réglée en entier
→ le règlement **réussit**, aucun groupe, aucune erreur, aucune entrée d'audit de lettrage
(`SyncOutcome::AccountNotLetterable`). Réciproquement, un groupe `document` posé **avant** le retypage
survit (C104) et se **dissout** à l'annulation d'un règlement (la dissolution n'exige pas la
lettrabilité).

**AC14** — **Périodes closes** (P7) : (a) facture et règlement complet datés sous
`books_locked_through`, exercice ouvert, groupe `document` posé avant le verrou ; l'annulation du
règlement est **refusée** — `409 LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, clé
`settlement-cancel-blocked-lettering-closed` —, **rien** n'est écrit (aucune écriture inverse, la ligne
`invoice_settlements` reste, le groupe est intact, aucune entrée d'audit) ; même refus pour l'annulation
d'un **solde** et pour le **dé-rapprochement** d'une facture encaissée par rapprochement, ce dernier dans
**sa** famille (`ReconciliationNotCancellable`, clé `reconciliation-cancel-blocked-lettering-closed`) ;
(b) puis un administrateur **déverrouille** (`POST /companies/current/books-lock/release`, motif) →
l'annulation **réussit** : dissolution, règlement ↔ miroir `reversal`, créance ouverte — et un nouveau
règlement complet daté en N+1 **réussit** et lettre (aucun `Invariant` : l'état de R-1/M-2 n'existe pas) ;
(c) facture et règlements historiques entièrement sous la borne, **sans** lettrage (marques effacées en
SQL brut) → `sync_invoice_in_tx` rend `AbstainedClosedPeriods` et n'écrit rien ; (d) état hérité « exercice
du groupe ouvert, suivi d'un exercice clos » (fabriqué en SQL brut, comme les tests de la 15-12b) → même
refus qu'en (a), le texte prescrivant de rouvrir les exercices postérieurs.

**AC17** — **Le refus du rang 2 bis, du prédicteur à l'écran** (C-15-1a2-10 à 12) : (a) **une seule
évaluation** — `settlement_entry_cancel_blocker` rend `DocumentLetteringInClosedPeriods` au rang 2 bis :
sur une écriture de règlement à la fois lettrée dans un groupe clos **et** rapprochée d'une transaction,
le motif rendu est le 2 bis (pas `MATCHED_BANK_TRANSACTION`) ; sur une écriture d'exercice clos, c'est
`FISCAL_YEAR_CLOSED` ; (b) la vue de la facture porte `cancelBlockedBy = "LETTERING_ALL_LINES_IN_CLOSED_PERIODS"`
sur ce règlement, et l'écran masque l'annulation en affichant le texte du motif (fiche facture,
`settlement-cancel-blocked.ts` ; dialogue de dé-rapprochement, `reconciliation-cancel.ts`) ; (c) les
trois clés existent dans les **quatre** locales, chaque repli Rust (`kesh-api/src/errors.rs`) et Svelte dit
**mot pour mot** le FTL fr-CH, et chaque texte nomme le remède (déverrouiller, ou rouvrir les exercices
clôturés du plus récent au plus ancien) ; (d) la clé fournisseur
`supplier-invoices-cancel-blocked-lettering-closed` et son cas dans `invoice-cancel.ts` sont posés ici,
**dormants** jusqu'à la 15-1a2-ii (aucun groupe `document` fournisseur n'existe avant elle).

**AC15** — **Le rapprochement** (`accept_one_invoice`) : (a) une proposition qui solde la facture la
lettre `document`, via l'API (`POST /reconciliation/accept`) ; (b) une modification concurrente de la
facture entre l'instantané et l'étape (g) rend `RECONCILIATION_INVOICE_NOT_ELIGIBLE` /
`race_during_update` **sans** qu'aucune marque ne soit posée ; (c) entrelacement *accept ‖ annulation
d'un règlement de la même facture* (patron `rejeu_interblocage_e2e.rs`) : les deux finissent (succès,
refus per-proposal ou rejeu), et l'état final satisfait AC5 ; (d) une erreur de synchronisation devient
un `FailedProposal` selon la table de P4 — test du mappage sur une fonction pure extraite, qui couvre
`LetteringConcurrentChange` → `INTERNAL_ERROR` (C-15-1a2-14 : le code n'entre pas dans `failed[]`, et le
décompte de 28 codes de `failed-proposal-label.ts` ne bouge pas).

## Tasks

- [ ] **T0** — Relevés au sol sur la base du gate : (a) `EXPLAIN` de la requête de découverte de P3 (accès
      par clé primaire / `idx` des écritures, pas de balayage de `journal_entry_lines`) ; (b) re-greper les
      ancres de P4 par le nom (`grep -nF "pub async fn settle_invoice"`, etc.) sur la base réelle du
      développement ; (c) vérifier que les routes appelantes restent `Rejouee` (`audit_route_registry.rs`) :
      `POST /invoices/{id}/settlements`, `…/settlements/{settlementId}/cancel`, `…/write-off`,
      `POST /credit-notes`, `POST /reconciliation/accept`, `POST /reconciliation/transactions/{id}/cancel`.
- [ ] **T1** (P3, P7) — `letterings.rs` : `SyncOutcome`, `DocumentRef`, `OpenPeriodRule`,
      `open_period_rule`, `lines_in_open_period` (prédicat par ligne factorisé avec
      `any_line_in_open_period`, types publics), `document_group_frozen_by_periods`, `create_group_inner` / `dissolve_group_inner` (+ `document`), extension de
      `audit_details`, `sync_invoice_in_tx`, `dissolve_invoice_document_group_in_tx` ; doc-comments
      (verrous, abstention, compte non lettrable) ; en-tête du module : les exceptions nommées au lettrage
      gagnent la synchronisation (elle **appelle** la primitive, n'écrit pas la marque).
      Doc-comment de `SettlementCancelBlocker::InvoiceCredited` (`kesh-db/src/errors.rs` ≈ `:350`, « son
      traitement est la 15-1a2 ») réécrit selon C-15-1a2-7 : le règlement reste ouvert, la 15-1a2 l'a tranché.
- [ ] **T2** (P4) — Les cinq appels du tableau, chacun à la place indiquée, avec l'exercice tenu et
      l'acteur ; `cancel_settlement_in_tx` client garde l'`id` d'exercice de l'étape (2-bis).
- [ ] **T2-bis** (P7 point 2, AC14, AC17) — Le refus du rang 2 bis : variante
      `SettlementCancelBlocker::DocumentLetteringInClosedPeriods` (doc-comment : remède, place, code
      réemployé) et son `code()` ; évaluation dans `settlement_entry_cancel_blocker` entre les rangs 2 et
      3 (doc-comment du module : « Les rangs 2 à 5 » → « 2 à 5, dont 2 bis ») ; refus dans
      `cancel_settlement_in_tx` étape (3) et dans `reconciliation_cancel::cancel_in_tx` étape (4) ;
      `kesh-api/src/errors.rs` : un bras dans chacune des trois tables de textes
      (`SettlementNotCancellable`, `reconciliation_cancel_blocked_text`, `supplier_invoice_cancel_blocked_text`),
      replis mot pour mot ; trois clés × quatre locales ; frontend : `SettlementCancelTailCode` et
      `settlementCancelTailMessage` (`lib/shared/utils/settlement-cancel-blocked.ts`), la liste et le
      texte de `features/reconciliation/reconciliation-cancel.ts` (et `reconciliation.types.ts:132`), le
      cas de `features/supplier-invoices/invoice-cancel.ts` (dormant), la liste de
      `lib/shared/i18n-repli-divergent-actif.test.ts:217-219` si elle doit nommer les clés de la famille ;
      `lint-i18n-ownership` vert.
- [ ] **T3** (P4, AC15) — `accept_one_invoice` : appel après (g), mappage per-proposal (fonction pure
      `lettering_error_to_failed_proposal`, testée).
- [ ] **T4** (AC8 part i) — Tests lexicaux dans `letterings_lexical.rs` ; réalignement de
      `each_primitive_checks_the_rows_its_update_found` sur `*_inner`.
- [ ] **T5** — Tests (liste ci-dessous) ; la **fixture partagée** d'AC5 vit dans
      `crates/kesh-db/tests/common/` (module `lettering_documents`), réutilisée par AC6 de la 15-1a2-ii.
- [ ] **T6** (AC12) — CHANGELOG, `api-external.md` (dont les deux tableaux de refus), manuel FR (dont
      le refus, `sec:verrou-periode`, le dé-rapprochement) + `make fr` + PDF aplati, message
      `LETTERING_IS_DOCUMENT` (quatre `.ftl` + repli Rust), grep de propagation dans les quatre langues.

**Tests prévus** (30 neufs, 3 étendus) :
- `crates/kesh-db/tests/lettering_documents.rs` (neuf, `test-schema`) — 19 :
  `full_settlement_letters_sale_and_settlements` (AC1), `three_partials_with_internal_account_letter_on_the_last` (AC1),
  `partial_settlement_letters_nothing` (AC1), `write_off_each_kind_letters` (AC2),
  `rounding_line_is_left_out` (AC2), `credit_note_letters_invoice_and_note` (AC3),
  `legacy_credit_note_on_other_account_forms_no_group` (AC3), `cancel_settlement_dissolves_and_pairs` (AC4),
  `cancel_write_off_dissolves_and_pairs` (AC4, L-5), `unreconcile_dissolves_and_pairs` (AC4),
  `cancel_partial_without_group_is_a_noop` (AC4), `closed_year_n_settled_and_cancelled_in_n1` (AC4, P5),
  `ledger_agrees_with_amount_due` (AC5), `audit_details_carry_the_invoice` (AC10),
  `receivable_not_letterable_is_skipped` (AC13, deux volets),
  `locked_period_cancel_is_refused_until_unlocked` (AC14 a — règlement, solde —, b),
  `historical_closed_history_abstains` (AC14 c), `later_closed_year_cancel_is_refused` (AC14 d),
  `closed_lettering_rank_precedes_bank_match` (AC17 a) ;
- `crates/kesh-db/tests/letterings.rs` — `lettering_invariants` **étendu** (AC9 part i ; pas un test neuf) ;
- `crates/kesh-db/tests/letterings_lexical.rs` — 3 neufs : `invoice_settlement_writers_stay_in_their_module_and_sync`
  (AC8 a, b), `credit_note_insert_is_followed_by_sync_and_cancel_dissolves_first` (AC8 c, d),
  `the_function_body_detector_sees_calls_and_order` (synthétique) ;
- `crates/kesh-api/tests/reconciliation_e2e.rs` — 3 neufs : `accept_letters_a_fully_settled_invoice` (AC15 a),
  `accept_race_refuses_before_any_lettering` (AC15 b),
  `unreconcile_of_a_closed_lettering_is_refused_in_its_family` (AC14 a, par l'API : code, clé, rien d'écrit) ;
- `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` — 1 neuf : `accept_and_settlement_cancel_interleave` (AC15 c) ;
- `crates/kesh-api/tests/invoice_echeancier_e2e.rs` — 1 neuf : `settlement_cancel_blocked_by_closed_lettering`
  (AC17 b : `cancelBlockedBy` de la vue, puis `POST …/cancel` → 409, même code) ;
- `crates/kesh-api/src/routes/reconciliation.rs` (`mod tests`) — 1 neuf :
  `lettering_errors_map_to_failed_proposals` (AC15 d) ;
- `crates/kesh-api/src/errors.rs` (`mod tests`) — 1 neuf : `closed_lettering_texts_follow_their_family`
  (AC17 c : trois familles, trois clés, replis non vides, remède nommé) ;
- `frontend/src/lib/features/supplier-invoices/invoice-cancel.test.ts` (neuf, Vitest) — 1 :
  le motif dormant a son texte (AC17 d) ;
- **étendus** (Vitest) : `lib/shared/utils/settlement-cancel-blocked.test.ts` et
  `features/reconciliation/reconciliation-cancel.test.ts` — le code neuf a son texte, mot pour mot le FTL
  (AC17 b, c).

*(Recompte depuis cette liste : 19 + 3 + 3 + 1 + 1 + 1 + 1 + 1 = **30 fonctions de test neuves** (29 Rust,
1 Vitest), plus **3 tests étendus** (`lettering_invariants`, deux fichiers Vitest).)*

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
  `accept_batch`). Cycle **neuf**, nommé (validation P2, L-2) : *accept ‖ règlement ou solde de la même
  facture* — `accept_one_invoice` insère sa ligne `invoice_settlements` avant de prendre la facture,
  `settle_invoice` / `write_off_invoice` tiennent la facture puis lisent `invoice_settlements` par la
  découverte `FOR UPDATE` ; AC15 (c) ne couvre que *accept ‖ annulation* — le rejeu couvre les deux.
  Le rang 2 bis n'ajoute **aucun** verrou : il lit, sans verrou, les lignes du groupe, les exercices et
  la borne, après les verrous que le geste prend déjà.
- **Codes jamais rendus par ces routes** (validation P2, L-3, réfuté sur le fond) : `409
  LETTERING_CONCURRENT_CHANGE` et un `Invariant` de synchronisation ne sont pas des refus de ces gestes —
  le premier est inatteignable par construction (C-15-1a2-14), le second est un défaut ; ni l'un ni
  l'autre n'entre aux tableaux de refus d'`api-external.md`.
- **Tests existants touchés** (finding F-10, relevés sur `056997b0`, non exécutés) :
  - **huit** fichiers écrivent `INSERT INTO invoice_settlements` en SQL brut hors production
    (`admin_full_import_e2e.rs`, `reconciliation_e2e.rs`, `exports_global_e2e.rs`,
    `journal_entry_reversal_e2e.rs`, `reconciliation_repository.rs`, `invoice_write_off.rs`,
    `invoice_settlement.rs`, plus le `mod tests` d'`invoices.rs` — relevé par
    `grep -rlE "INSERT INTO invoice_settlements" --include=*.rs crates/`, qui rend neuf fichiers dont
    `invoice_settlements.rs`, où la seule occurrence `:415` est la production ; validation P2, R-5) : ils fabriquent des factures « soldées » **sans**
    groupe — cohérent (le SQL brut contourne le geste) ; aucun n'asserte l'absence de marque. À relire
    si l'un rougit ;
  - `reconciliation_cancel.rs` (`reconciliation_cancel_letters_the_reversal_pair`, rapprochement **hors
    facture**) : inchangé ;
  - `journal_entry_reversal_e2e.rs`, `letterings.rs`, `letterings_e2e.rs` : `details` d'audit des routes
    et de la contre-passation **inchangés** (aucune clé `document*` quand `None`) ;
  - `invoice_settlement.rs` (annulations) : le résultat final de R6 ne change pas ; la séquence devient
    dissolution → paire ;
  - les tests de la file commune (`settlement_cancellation`, prédicteurs client et fournisseur, textes
    de `kesh-api/src/errors.rs`, `invoice_echeancier_e2e.rs`, `supplier_settlement_cancel_e2e.rs`,
    `reconciliation_e2e.rs`) : un `match` exhaustif sur `SettlementCancelBlocker` gagne un bras ; aucun
    rang existant ne change de place ; le test des textes de la queue côté frontend
    (`settlement-cancel-blocked.test.ts`) gagne un code. ⚠️ Le commentaire d'`invoices.rs:1667-1673`
    (« la marque de lettrage … inatteignable ici », dévalidation) **reste vrai** : sans groupe gardé,
    aucune facture sans règlement n'a de ligne de vente lettrée (validation P2, M-2 — dépendait du
    groupe gardé).
- **Dépendances** : 15-1a-i, 15-1a-ii (mergées). 15-6a/b/c/d **mergées** (`39b52628`, `f8b2accd`,
  `803f3e15`, `1ae3963e`) : les avertissements #473/#474 de la fiche d'origine sont retirés.
- **Règle de découpage** : modules **de premier niveau** touchés, comptés par domaine : (1) le lettrage
  (`kesh-db` `letterings.rs` + son message `LETTERING_IS_DOCUMENT` — quatre `.ftl` et un repli Rust),
  (2) les règlements clients **et leur file d'annulation** (`invoice_settlements_write.rs`,
  `settlement_cancellation.rs`, la variante de `SettlementCancelBlocker` dans `kesh-db/src/errors.rs`),
  (3) les avoirs (`credit_notes.rs`), (4) le rapprochement (`kesh-api` `routes/reconciliation.rs`,
  `kesh-db` `reconciliation_cancel.rs`), (5) **l'écran et ses textes** — trois clés × quatre locales, trois
  bras de `kesh-api/src/errors.rs`, `settlement-cancel-blocked.ts`, `reconciliation-cancel.ts`,
  `invoice-cancel.ts` —, (6) la documentation. **Six**, un de plus que le seuil : voir la dérogation
  ci-dessous. Par fichier de production : `letterings.rs`, `invoice_settlements_write.rs`,
  `settlement_cancellation.rs`, `reconciliation_cancel.rs`, `kesh-db/src/errors.rs`, `credit_notes.rs`,
  `routes/reconciliation.rs`, `kesh-api/src/errors.rs`, quatre `.ftl`, trois modules TypeScript (et
  `reconciliation.types.ts`). `invoice_settlements.rs`, `routes/letterings.rs` et `journal_entries.rs` ne
  changent pas.

### Dérogation règle de splitting

*(C-15-1a2-13 ; décision de l'orchestrateur, validation P2.)* Le refus du rang 2 bis (P7 point 2) fait
passer la fiche de **cinq** à **six** modules : l'écran et ses textes s'ajoutent. **Dérogation**, non
découpage, pour trois raisons :

1. **Un refus nommé de plus, sans logique neuve hors du blocker existant.** Le rang 2 bis s'insère dans
   `SettlementCancelBlocker` et dans la file `settlement_entry_cancel_blocker`, qui ont **déjà** leurs
   trois familles de textes, leurs trois listes TypeScript et leurs tests ; chaque site gagne un bras d'un
   `match` exhaustif — le compilateur et `never` les énumèrent. Aucune règle métier neuve n'y vit :
   l'évaluation est celle de P7 (`document_group_frozen_by_periods`, module du lettrage).
2. **Le découper le séparerait de ce qu'il garde.** Sans le refus, la dissolution de P4 rencontre un
   groupe clos et retombe dans l'alternative écartée (abstention ou dissolution) ; une sous-story « refus »
   mergée après la synchronisation laisserait entre les deux merges l'état que la validation P2 a montré
   défaillant (R-1, M-2). Le refus et la dissolution vont ensemble.
3. **Le motif du seuil — la tenue dans un seul modèle mental adversarial — ne joue pas** sur ce module :
   la revue de l'écran est mécanique (trois textes, trois cas de `switch`), file par file.

**Risque accepté** : une revue plus large. **Contre-mesure** : la passe de revue de code nomme l'axe
« les trois familles de textes et leurs replis, mot pour mot », et le gate frontend complet
(`npm run check`, `lint-i18n-ownership`, `test:unit`, `build`) tourne au dernier commit de code. Aucun
autre débordement constaté (les gestes fournisseurs restent à la 15-1a2-ii, qui garde ses cinq modules).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Validation P2 — 2026-10-09 (Opus 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, seul remédiateur des trois fiches, en autonomie)

**Rapports** : `kesh-gate-logs/15-1a2-i-validate-p2-R.md` (**0 CRITICAL, 0 HIGH, 2 MEDIUM, 7 LOW**) et
`…-F.md` (**0 CRITICAL, 0 HIGH, 4 MEDIUM, 5 LOW**). Doublons : R-1 ≈ M-2, R-2 ≈ M-3, R-6 = L-1 → **4 MEDIUM
distincts**, tous **nés de la remédiation P1** (trois de C-15-1a2-2, un de la table per-proposal de P4).
**Trend** (la passe 1 portait sur la fiche mère 15-1a2) : P1 **3 HIGH / 7 MEDIUM** (R) et **2 HIGH / 7
MEDIUM** (F) → P2 **0 HIGH / 4 MEDIUM distincts**. Signal D5 : **non levé** (sévérité en baisse) ; le motif
« la sévérité se déplace vers ce qu'on vient d'écrire » est en revanche **entier** — d'où la décision de
l'orchestrateur de traiter la cause (C-15-1a2-2) plutôt que ses symptômes. Chaque finding relu au code
(`grep -nF` / `sed -n` sur `056997b0`).

| finding | sévérité | verdict | où |
|---|---|---|---|
| R-1 ≈ M-2 — après un déverrouillage, le groupe gardé fait sortir `Invariant` (500) d'un règlement, d'un avoir ou d'une annulation | MEDIUM | **corrigé à la racine** : le groupe gardé n'existe plus (refus, C-15-1a2-10) ; réserve de P3 réécrite ; AC14 (b) asserte un règlement réussi après déverrouillage | P3, P7 point 2, AC14 |
| R-2 ≈ M-3 — AC5 « deux exceptions et elles seules » faux sur un état atteignable ; « lettrée `document` » non défini | MEDIUM | **corrigé** : prédicat écrit (« les lignes de `C(I)` forment exactement un groupe `document` ») ; l'état divergent n'existe plus ; fixture enrichie d'une pièce passée sous la borne | AC5 |
| M-1 — le groupe gardé produit des états définitifs que rien ne répare | MEDIUM | **corrigé** : la voie que la lentille proposait est retenue — refus au rang 2 bis, remède administratif (C-15-1a2-10 à 12) | P7, AC14, AC17, T2-bis |
| M-4 — `LETTERING_CONCURRENT_CHANGE` per-proposal sans libellé, test du compte muet | MEDIUM | **corrigé** : inatteignable par construction (lignes tenues par l'acte 1) → `INTERNAL_ERROR` ; aucun module frontend (C-15-1a2-14) | P4, AC15 (d) |
| R-3 — mauvaise phrase du socle citée | LOW | **corrigé** (rang 2, `:85-100`) | P5, P7 |
| R-4 — « aucune paire `reversal` » faux pour un `internal_account` | LOW | **corrigé** (AC4 nomme la contrepartie ; AC14 ne porte plus sur les paires) | AC4 |
| R-5 — « neuf fichiers » | LOW | **corrigé** : **huit** (F le confirmait à tort en comptant la production d'`invoice_settlements.rs`) | modèle, Dev Notes |
| R-6 = L-1 — `-> bool` qui lit la base ; frontière avec la 15-1b | LOW | **corrigé** : `Result`, types publics, **une** factorisation reprise par la 15-1b (C-15-1a2-18) | P7, T1 |
| R-7 — retypage refusé si `A` porte un rôle | LOW | **corrigé** (voie bancaire privilégiée ; rôle retiré, `confirm_retype`) | AC13 |
| R-8 — l'avoir après un groupe gardé | LOW | **sans objet** (état disparu) | — |
| R-9 — grep de propagation en français seulement | LOW | **corrigé** (jetons de/it/en) | AC12 |
| L-2 — « même ordre que les autres gestes » inexact ; cycle accept ‖ règlement | LOW | **corrigé** (phrase réécrite, cycle nommé) | P4, Dev Notes |
| L-3 — 409 / 500 non documentés sur les routes de règlement | LOW | **réfuté sur le fond** : inatteignables (C-15-1a2-14) ; écrit aux Dev Notes | Dev Notes |
| L-4 — « une remédiation en cours la tient » périmé | LOW | **corrigé** : « Pour la 15-1b » réécrite **et intégrée** à la 15-1b dans la même passe | Pour la 15-1b |
| L-5 — annulation d'un solde non testée | LOW | **corrigé** (`cancel_write_off_dissolves_and_pairs`) | AC4, tests |

**Décisions de l'orchestrateur appliquées** : (1) **refus** au délettrage — C-15-1a2-2 révisée **deux
fois**, historique écrit en P7 et au registre (abstention partout → délettrer toujours, écartée : contredit
C113 et `user-manual.tex:577-579` → refus) ; (2) **dérogation** à la règle de splitting (six modules,
C-15-1a2-13 ; aucun autre débordement constaté) ; (3) le prédicat des périodes, **une** factorisation
(C-15-1a2-18) ; (4) `LETTERING_CONCURRENT_CHANGE` → `INTERNAL_ERROR` (C-15-1a2-14). ⚠️ **Heurt signalé** :
la consigne disait « nouveau code d'erreur » ; le type pose que ses codes réemploient ceux d'états du
monde déjà nommés — le refus est neuf, son code réemploie `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`
(C-15-1a2-11). **Propagation** (valeurs grepées sur les trois fiches, l'index et le registre) : « gardé »,
`AbstainedClosedPeriods` côté dissolution, « points 2-3 », « une remédiation en cours », « neuf fichiers »,
`-> bool`. Choix consignés : **C-15-1a2-10 à 14, 18** ; C-15-1a2-2 annotée. **Recompte** (depuis ce
fichier) : **13 critères** (AC1–AC5, AC8–AC10, AC12–AC15, AC17 neuf), **8 tâches** (T0, T1, T2, T2-bis,
T3–T6), **30 tests neufs** (29 Rust, 1 Vitest) **+ 3 étendus**. Prochaine passe : **P3, Sonnet**, complète
(la remédiation change une règle métier et touche plusieurs modules).

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
