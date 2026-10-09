# Story 15.1a2-i : Le lettrage des pièces clientes — une facture soldée est lettrée, et seulement elle

## Status

review *(développée le 2026-10-09 — Opus 5.5, en autonomie ; revue de code à mener)* — ready-for-dev *(découpée de la 15-1a2 le 2026-10-09 à la remédiation de sa validation P1 — C-15-1a2-1 ;
validation P2 remédiée le 2026-10-09 — **refus** plutôt qu'abstention au délettrage, C-15-1a2-10 ;
validation P3 remédiée le 2026-10-09 — le refus **extrait** dans la story préalable **15-1a2-0**,
C-15-1a2-19, la dérogation C-15-1a2-13 retirée ; validation P4 remédiée le 2026-10-09 — fixture partagée sortie
du code de production (`tests/support/`, C-15-1a2-28), étape 3 **terminale** (C-15-1a2-29), documentation
publique du refus du rang 2 bis reçue de la 15-1a2-0 (**AC18**, C-15-1a2-24) ; **validation P5 à mener avant
tout développement** — faite : P5 ciblée close, cf. Change Log)*

## Story

**As a** indépendant, PME ou fiduciaire qui encaisse ses factures dans Kesh,
**I want** que les lignes d'une facture soldée — la créance, ses règlements, son solde, son avoir —
soient lettrées entre elles sans que j'aie à le refaire, et délettrées si un règlement est annulé,
**so that** le grand livre dise ce que disent la fiche facture, la balance âgée et les relances :
une facture est soldée **si et seulement si** son reste dû est nul — dans les limites, nommées, des
périodes closes (P7) : une pièce historique entièrement close n'est pas lettrée après coup, et un
lettrage figé par une période close ne se défait pas sans qu'un administrateur la rouvre.

Deuxième des trois sous-fiches de la **15-1a2** (index : `15-1a2-lettrage-des-pieces.md`). **Suppose la
15-1a-i (#587), la 15-1a-ii (#593) et la 15-1a2-0 mergées** — base `056997b0` plus la 15-1a2-0. Ordre :
15-1a-i → 15-1a-ii → 15-1a2-0 → **15-1a2-i** → 15-1a2-ii → 15-1b-0 → 15-1b → 15-1c. ⛔ La **15-1a2-0** (le
lettrage se fige avec la période) livre ce que cette fiche **emploie** : la règle des périodes
(`open_period_rule`, `lines_in_open_period` — sa D1) et le **refus** du rang 2 bis dans les quatre gestes
d'annulation, ses textes et son écran (sa D2, D3) ; elle est dormante tant que la présente story n'a pas posé
de groupe `document`. ⛔ La **15-1a2-ii** (fournisseurs et rattrapage) suppose **celle-ci** mergée : elle
réutilise sa synchronisation et son extension d'audit, et son test d'accord (AC6) compare le rattrapage à
la synchronisation livrée ici. ⛔ **La documentation publique du refus** (manuels, `api-external.md`,
CHANGELOG) est **ici** (AC18), non à la 15-1a2-0 (C-15-1a2-24, décision de l'orchestrateur) : c'est cette story
qui pose les premiers groupes `document` et rend le refus atteignable ; écrite plus tôt, elle aurait contredit
le manuel qui dit encore « Kesh ne lettre pas encore de lui-même ». Elle couvre les **quatre** gestes, les deux
gestes fournisseurs compris, dont le code refuse le rang dès la 15-1a2-0 et dont les groupes naissent à la
15-1a2-ii, mergée juste après — sans tag entre les deux (C124), la documentation publiée est juste.

**Numérotation conservée** de la 15-1a2 (P1, P3, P4, P5 ; AC1–AC5, AC8–AC10, AC12) : les renvois des
fiches sœurs (« 15-1a2 P2 », « AC5 de la 15-1a2 », « 15-1a2 P5 ») restent justes. Les numéros neufs
commencent à **P7** et **AC13** ; **AC18** (documentation du refus, validation P4) est le dernier. Les éléments partagés avec la 15-1a2-ii (AC8, AC9, AC10, AC12) portent
ici leur **part i**. **AC17** (le refus, du prédicteur à l'écran) est parti à la 15-1a2-0 (AC2 à AC7) ; son
numéro n'est pas réattribué.

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
facture validée). ⚠️ **Hors de l'inventaire, à dessein** (validation P3, L-5) : la **restauration** d'une
sauvegarde écrit la même table par **nom dynamique** (`kesh-db/src/backup.rs:34-44`, `TABLES_TO_TRUNCATE`),
que le littéral ne voit pas. Elle rétablit lignes **et** marques ensemble ; le cas d'une sauvegarde
antérieure à la 15-1a2 est le rejeu M1 de la 15-1a2-ii (P6). L'inventaire « fermé » porte sur les écrivains
**par un geste**.

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
rang 2 bis de la file commune (livré par la **15-1a2-0**, D2-D3 ; P7 point 2 ici) — la dissolution n'est
donc appelée que sur un groupe qui a une ligne en période ouverte, et elle n'évalue pas elle-même la règle
des périodes (une seule garde par motif, patron de `cancel_settlement_in_tx`).

**`DocumentRef`** — défini **ici, une fois** (validation P3, F-7 = L-1 ; la 15-1a2-ii le consomme tel quel ;
C-15-1a2-22) :

```rust
/// La pièce dont une synchronisation pose ou défait le groupe `document` — pour l'audit.
pub struct DocumentRef {
    /// `"invoice"` ici ; `"supplierInvoice"` à la 15-1a2-ii — les valeurs mêmes de `document.type`
    /// de la vue des postes ouverts (15-1b). ⚠️ Littéral **provisoire** : la 15-1b-0, mergée après,
    /// type ce champ en `DocumentKind` et le sérialise par `DocumentKind::as_str` (sa T2, C-15-1b-0-3).
    pub document_type: &'static str,
    pub id: i64,
    /// Le numéro de la pièce ; `None` quand elle n'en a pas (facture fournisseur sans numéro, 15-1a2-ii).
    pub number: Option<String>,
}
```

`audit_details` émet les trois clés `documentType`, `documentId`, `documentNumber` quand `document` est
`Some` — **`documentNumber` présent et `null`** quand `number` est `None` (la clé n'est jamais omise) — et
**aucune** quand `document` est `None`.

**Précédence des issues** (validation P3, F-7 ; réécrite en validation P4, M-2 = F4-3) : la lettrabilité est
jugée à l'étape 3, qui est **terminale** — compte non lettrable → `AccountNotLetterable`, rien n'est écrit,
**quel que soit le groupe existant** (C-15-1a2-29). Les étapes 4 à 6, et donc la règle des périodes, ne
s'évaluent que pour un compte lettrable : `AccountNotLetterable` l'emporte sur `AbstainedClosedPeriods`, et
aucun groupe existant n'est jamais défait par la synchronisation sur ce motif.

`held_open_fiscal_year_id` est l'exercice **ouvert** que l'appelant tient `FOR UPDATE` (mode
`System`, 15-1a-i R7 point 3) ; P4 le nomme pour chaque site. `actor` est l'auteur du geste (P4,
AC10).

**`sync_invoice_in_tx`** :

1. **Découverte, par une lecture VERROUILLANTE** (`… FOR UPDATE`) des lignes sur `A` de l'écriture de
   vente, des écritures des règlements en vigueur et de l'écriture de l'avoir émis — avec leur
   `lettering_key`, `lettering_origin`, exercice et date (finding F-3 : sous `REPEATABLE READ`, une
   lecture ordinaire lirait l'instantané). Rend `C(I)` et l'ancre. Le compte `A` se lit par
   `sale_receivable_account` ; une écriture de vente **sans** ligne au débit (`None`) → `DbError::Invariant`,
   comme le font déjà `settle_invoice` et `write_off_invoice` sur le même cas (`invoice_settlements_write.rs:123-125`, `:491-493` ; validation P3, L-9 d).
2. **Le groupe existant** (finding R13, défini) : `E` = l'ensemble des `lettering_key` **distincts et non
   nuls** des lignes de `C(I)`. Une ligne de `C(I)` lettrée d'origine `manual` ou `reversal` →
   `DbError::Invariant` (état impossible : une ligne de pièce n'est pas lettrable à la main, 15-1a R5 ;
   une ligne de règlement n'est lettrée `reversal` que dans la transaction qui retire sa ligne
   `invoice_settlements` — la contre-passation, `invoice_settlements_write.rs:852`, précède le `DELETE`, `:862`,
   mais après le `COMMIT` la ligne n'est plus dans `C(I)` ; validation P4, L-2),
   **jamais un écrasement**. `|E| > 1` → `DbError::Invariant`.
3. **La cible** `T` : `C(I)` si elle qualifie (P1), sinon aucune. ⛔ **Compte non lettrable** (findings
   R5 = F-2, C-15-1a2-3) : si `A` n'est pas lettrable (`is_letterable_account`, appelé **avant** la
   primitive), **retour immédiat** `SyncOutcome::AccountNotLetterable`, **aucune erreur**, **rien n'est écrit,
   quel que soit `E`** — étape **terminale** (validation P4, M-2 = F4-3 ; C-15-1a2-29) : un groupe `document`
   posé avant que `A` ne devienne non lettrable **survit** (C104, AC13) ; seule la dissolution de P4, qui n'exige
   pas la lettrabilité, le défait. Comme la contre-passation saute un compte non lettrable
   (`journal_entries.rs:2578`, `if !is_letterable_account(…) { continue; }`). **Un règlement, un solde ou un
   avoir n'échoue JAMAIS à cause du lettrage** : la primitive ne doit pas rendre
   `LetteringAccountNotLetterable` sur ce chemin.
4. `E = {k}` et les lignes de `k` sont exactement `T` → `Unchanged`.
5. `E = {k}` sinon — **défensif, inatteignable par un geste** (voir ci-dessous) : si aucune ligne **du
   groupe `k`** n'est en période ouverte (P7) → `AbstainedClosedPeriods`, **rien n'est écrit** ; sinon
   `dissolve_group` (mode `System`) puis, si `T`, création.
6. `E = ∅` et `T` → création (mode `System`, origine `document`) ; `E = ∅` sans `T` → `Unchanged`, ou
   `AbstainedClosedPeriods` si la règle des périodes est la seule raison qui l'empêche (le compte non
   lettrable est sorti à l'étape 3).

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
| `accept_one_invoice` (`routes/reconciliation.rs:1383`) | `sync_invoice_in_tx` | **après** l'étape (g) — l'`UPDATE invoices … AND version = ?` et son contrôle `rows_affected` —, avant l'audit « Step 9 » | `fiscal_year` de l'étape (d) (`find_open_covering_date`, `:1745`) | `Actor { user_id, api_key_id: actor_api_key_id }` |
| `create_credit_note` (`credit_notes.rs:327`) | `sync_invoice_in_tx` | **après** l'étape (11), la bascule `UPDATE invoices SET status = 'cancelled'` (`:772`), avant la relecture (12) | `fy` de la date de l'avoir (`:553`) | `Actor { user_id, api_key_id: None }` |
| `cancel_settlement_in_tx` client (`invoice_settlements_write.rs:781`) — **couvre** l'annulation d'un **solde** (`write_off`, une ligne `invoice_settlements` comme une autre) et le dé-rapprochement (`reconciliation_cancel.rs`) | `dissolve_invoice_document_group_in_tx` | **après** les gardes de l'étape (3) — qui refusent aussi le **rang 2 bis** depuis la 15-1a2-0 (sa D3) —, **avant** `reverse_owned_in_tx` (étape (4), `:852`) | l'exercice de l'écriture de règlement, verrouillé à l'étape (2-bis) (`:827` — sa valeur, aujourd'hui jetée, est **gardée**) | `Actor { user_id, api_key_id: None }` |

**Le dé-rapprochement refuse le rang 2 bis dans SA famille**, **avant** de défaire le lien — livré par la
15-1a2-0 (D3, motif **lié** `blocker @ (FiscalYearClosed | DocumentLetteringInClosedPeriods)`) ; il appelle
ensuite `cancel_settlement_in_tx`, qui porte la dissolution de cette fiche.

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
`settlement_cancellation.rs:85-100` ; le rang 2 bis, livré par la 15-1a2-0) : l'exercice tenu est **ouvert**, il
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
  l'exigerait est refusé** — rang **2 bis** de la file commune des annulations (point 2), livré par la
  **15-1a2-0**.

**Évaluation** : la factorisation **publique** de la **15-1a2-0** (sa D1 ; C-15-1a2-18) —
`open_period_rule(conn, company_id, &fiscal_year_ids) -> Result<OpenPeriodRule, DbError>`,
`OpenPeriodRule::line_in_open_period(fiscal_year_id, entry_date)`, `lines_in_open_period(conn, company_id,
&[(fiscal_year_id, entry_date)]) -> Result<bool, DbError>`, le même prédicat par ligne que le mode `Manual`.
Cette fiche l'**emploie** (étapes 5 et 6 de P3 ; validation P5 ciblée, P5-3) ; elle n'en écrit pas de seconde.

**Ce que la règle atteint réellement**, et ce que voit l'utilisateur :

1. **Création — abstention.** Un geste vivant ajoute toujours une ligne **en période ouverte** (une
   écriture ne se crée ni sur un exercice clos, ni sous la borne : `PeriodLocked`,
   `journal_entries.rs:405`). La création ne s'abstient donc **jamais** sur un geste ; elle s'abstient
   sur les pièces **historiques** entièrement closes — le rattrapage de la 15-1a2-ii et la
   synchronisation appelée sur elles (AC6 de la 15-1a2-ii, AC14 (c) ici). **Ce que voit
   l'utilisateur** : la facture soldée reste **ouverte** au grand livre (la 15-1b la liste,
   `documentState = nothingDue`). ⛔ Elle n'est **pas** lettrable à la main : ses lignes sont des lignes
   de pièce (15-1a R5, `LETTERING_LINE_OWNED_BY_DOCUMENT`). Elle le reste — jusqu'à ce qu'un
   administrateur rouvre la période : la synchronisation ne repasse pas d'elle-même, mais un rejeu de M1
   à l'import d'une sauvegarde la lettrerait alors (15-1a2-ii, P6).
2. **Dissolution — refus, rang 2 bis, livré par la 15-1a2-0** (ses D2 à D4 : variante
   `SettlementCancelBlocker::DocumentLetteringInClosedPeriods`, évaluation `document_group_frozen_by_periods`
   dans la file commune, refus des quatre gestes d'annulation — règlement et solde clients, dé-rapprochement,
   paiement et facture fournisseurs —, code réemployé `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, textes, écran,
   remède précis). Dormant jusqu'à cette story-ci, qui pose les premiers groupes `document` : l'annulation
   d'un règlement dont le groupe n'a **aucune** ligne en période ouverte — facture et règlement du premier
   trimestre, verrou au 31.03, annulation en avril — est refusée, **avant** la dissolution de P4. Ce que la
   présente fiche y ajoute : l'**intégration** (AC14 a, b — un groupe posé par le geste, puis figé, puis
   libéré par le déverrouillage) et la **documentation publique** du refus (AC18, C-15-1a2-24). **Ce que voit l'utilisateur** : au 31.03, rien ne bouge — ni la vue des
   postes ouverts, ni la pièce. **Il n'existe plus de groupe gardé** : aucune facture due n'a de ligne de
   vente lettrée, AC5 et AC9 n'ont plus d'exception de période, et l'`Invariant` que la validation P2 avait
   atteint après un déverrouillage (R-1, M-2) n'a plus d'état qui le produise (P3). ⚠️ **Tolérance
   résiduelle** (15-1a2-0 D4) : un `lock_books` validé entre la lecture du rang 2 bis et le `COMMIT` laisse
   passer une dissolution dont toutes les lignes sont, au `COMMIT`, en période close. C'est le **seul**
   chemin qui dissout un groupe hors période ; il n'en reste aucun par un geste séquentiel (vérifié :
   `dissolve_group_in_tx` en mode `Manual` évalue R7 ; les dissolutions `System` sont celles de P3, gardées
   par ce rang ou défensives ; la contre-passation et le rattrapage ne dissolvent rien).

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
`lines_in_open_period` (15-1a2-0 D1) : la 15-1b l'emploie tel quel, sans seconde factorisation (C-15-1a2-18,
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

**AC12 (part i)** — Documentation **du lettrage des pièces clientes**, **par la valeur** (findings R8, R9 = F-7 ;
la documentation **du refus** est l'AC18) :
- `CHANGELOG.md` (`[0.13.0]`, entrée du lettrage, aujourd'hui `:15`) : retirer « Kesh ne lettre pas
  encore de lui-même une facture soldée par ses règlements » ; écrire « une facture client soldée — par
  ses règlements, son solde ou son avoir — est lettrée d'office ; l'annulation d'un règlement la
  délettre ». *(Le refus de l'annulation sous une période close est l'entrée d'AC18.)*
- `docs/api-external.md` : `:223` et `:291` — sortir `document` de la réserve (« posé par Kesh sur les
  lignes d'une facture client soldée ; il suit ses règlements et son avoir ») ; **`:324` — la ligne entière
  est réécrite, cette story en est le seul propriétaire** (validation P4, L-4 = F4-2 ; la 15-1a2-0 ne touche
  pas `docs/`, C-15-1a2-24) : le libellé « annuler le règlement, pas délettrer » est faux pour un groupe
  facture + avoir — il devient « Groupe d'origine `document` — il suit sa pièce et ses règlements, et ne se
  délettre pas à la main », sur le message réécrit par la 15-1a2-0 (D5) —, et l'annotation « *(aucun groupe
  `document` n'existe encore)* » est retirée ; `:299` — « Les deux gestes sont **tracés** au journal d'audit
  (`lettering.created`, `lettering.removed`) … et la clé qui les a faits » : ajouter que ces événements sont
  aussi émis par les règlements, soldes, avoirs, rapprochements et annulations d'une facture client, avec
  `documentType`, `documentId`, `documentNumber` en plus, et **sans** la clé hors du rapprochement (écart
  d'AC10 ; validation P3, L-4).
- `docs/manual/fr/user-manual.tex` : `:765` (« les encaissements des factures ne se lettrent pas encore »
  → « les encaissements des factures se lettrent d'eux-mêmes avec leur facture ») ; § « Enregistrer et
  annuler un règlement » (`sec:reglement-client`, `:1141`) — un paragraphe *Lettrage* : facture soldée
  lettrée, annulation (et annulation du solde) qui délettre, pièce historique close restée ouverte (P7
  point 1), créance non lettrable ; le refus sous une période close est dans la liste des motifs du même §
  (AC18) — le paragraphe y renvoie, sans le redire ; § « Avoirs et notes de crédit » (`:1262`, `:1267`) —
  « l'écriture d'origine reste intacte » et « sa propre écriture reste intacte » complétés « hors la marque
  de lettrage, qui la rattache à l'avoir » (Reçu point 20) ; glossaire, entrée *Lettrage* (`:2420-2426`,
  **coupée sur deux lignes** : « Kesh ne lettre / pas encore de lui-même ») — « Kesh lettre de lui-même une
  facture client soldée avec ses règlements, son solde ou son avoir ; un lettrage de pièce figé par une
  période close ne se défait pas : l'annulation qui le défairait est refusée (§ du verrou de période) ».
- **PDF** : `make fr` dans `docs/manual/`, les trois PDF commités ; contrôle **aplati** :
  `pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -oE "(ne se lettrent pas encore|Kesh ne lettre pas encore)[^.]*\."`
  ne rend **plus rien** (deux lignes aujourd'hui, relevées en validation P1).
- Contrôle de propagation, **en français** (les jetons de/it/en du message `LETTERING_IS_DOCUMENT` sont
  contrôlés par la 15-1a2-0, AC9, qui le réécrit ; validation P4, L-3) : `git grep -nE "ne se lettrent
  pas|ne lettre pas encore|réservée? aux lettrages|aucune route ne la rend|aucun groupe .document. n.existe
  encore|pas délettrer" -- CHANGELOG.md docs crates/kesh-i18n crates/kesh-api/src website README.md` ne rend
  **plus rien**. Sur `056997b0` il rend **cinq** sites, tous réécrits ici : `CHANGELOG.md:15`,
  `api-external.md:223`, `:291`, `:324`, `user-manual.tex:765` ; **aucun** site fournisseur. *(Le jeton large
  « n'existe encore » de la version P3 rendait aussi `routes/admin.rs:262` et `routes/email_templates.rs:81`,
  qui parlent d'autre chose : resserré. Le glossaire, coupé sur deux lignes, échappe à tout `grep` : le
  contrôle PDF le couvre.)*

**AC13** — **Compte de créance non lettrable** (findings R5 = F-2) : le compte `A` d'une facture validée
est **rattaché à un compte bancaire** (voie de la fixture : un compte d'actif y est admis) — ou retypé
en charge (`Expense`), ce qui exige de lui retirer d'abord tout rôle (`check_role_account_type`,
`accounts.rs:72-97`) et de confirmer le retypage d'un compte mouvementé (`confirm_retype`, Story 25-2-a ;
validation P2, R-7) —, puis la facture est réglée en entier
→ le règlement **réussit**, aucun groupe, aucune erreur, aucune entrée d'audit de lettrage
(`SyncOutcome::AccountNotLetterable`). Réciproquement, un groupe `document` posé **avant** que `A` ne devienne
non lettrable **survit** (C104) : (a) la synchronisation appelée sur lui rend `AccountNotLetterable`, marques
**inchangées**, **aucune** entrée d'audit — l'étape 3 est terminale (validation P4, M-2 = F4-3 ; C-15-1a2-29) ;
(b) il se **dissout** à l'annulation d'un règlement (la dissolution n'exige pas la lettrabilité).

**AC14** — **Périodes closes, intégrées au geste** (P7 ; le refus lui-même est éprouvé par la 15-1a2-0 sur
des groupes posés à la main — ici, sur des groupes posés **par la synchronisation**). Énoncé **au niveau du
dépôt**, où vit son test (validation P4, L-1 ; la correspondance HTTP du motif est l'AC3 de la 15-1a2-0) :
(a) facture et règlement complet, groupe `document` posé **par le règlement**, verrou posé **ensuite** à la date
du règlement ou après (`companies::lock_books`), exercice ouvert ; `cancel_settlement` rend
`DbError::SettlementNotCancellable { blocker: DocumentLetteringInClosedPeriods }` **avant** la dissolution :
**rien** n'est écrit (aucune écriture inverse, la ligne `invoice_settlements` reste, le groupe est intact,
aucune entrée d'audit) ; même refus pour l'annulation d'un **solde**, sur une autre facture ; (b) puis un
administrateur **déverrouille** (`companies::unlock_books`, motif, nouvelle borne **avant** la date la plus
récente du groupe) → l'annulation **réussit** : dissolution, règlement ↔ miroir `reversal`, créance ouverte — et
un nouveau règlement complet daté **après la nouvelle borne** **réussit** et lettre (aucun `Invariant` : l'état
de R-1/M-2 n'existe pas) ; (c) facture et règlements historiques entièrement sous la borne, **sans** lettrage
(marques effacées en SQL brut) → `sync_invoice_in_tx` rend `AbstainedClosedPeriods` et n'écrit rien.
*(L'ancien volet (d), état hérité « exercice suivi d'un exercice clos », est l'AC6 de la 15-1a2-0.)*

**AC17** — *déplacé à la 15-1a2-0* (AC2 à AC7 : précédence, écran, textes, clé fournisseur). Numéro non
réattribué.

**AC15** — **Le rapprochement** (`accept_one_invoice`) : (a) une proposition qui solde la facture la
lettre `document`, via l'API (`POST /reconciliation/accept`), **sous une clé d'API** : l'entrée
`lettering.created` porte l'`api_key_id` de la clé (l'acteur d'AC10 — seul geste qui la porte ; validation P4,
L-5) ; (b) une modification concurrente de la
facture entre l'instantané et l'étape (g) rend `RECONCILIATION_INVOICE_NOT_ELIGIBLE` /
`race_during_update` **sans** qu'aucune marque ne soit posée ; (c) entrelacement *accept ‖ annulation
d'un règlement de la même facture* (patron `rejeu_interblocage_e2e.rs`) : les deux finissent (succès,
refus per-proposal ou rejeu), et l'état final satisfait AC5 ; (d) une erreur de synchronisation devient
un `FailedProposal` selon la table de P4 — test du mappage sur une fonction pure extraite,
`fn lettering_error_to_failed_proposal(bank_transaction_id: i64, err: DbError) -> FailedProposal`
(validation P3, L-9 c), placée à côté de `claim_account_failed_proposal` (`routes/reconciliation.rs:1322`),
qui joue le même rôle (F-8), et qui couvre `LetteringConcurrentChange` → `INTERNAL_ERROR` (C-15-1a2-14 : le
code n'entre pas dans `failed[]`, et le décompte de 28 codes de `failed-proposal-label.ts` ne bouge pas).

**AC18** — **Documentation publique du refus du rang 2 bis** (reçue de la 15-1a2-0, ex-AC8 et T7 — décision de
l'orchestrateur, C-15-1a2-24 ; findings F-1, F-2 de la validation P1 de la 15-1a2-0). Le refus est **livré**
par la 15-1a2-0 (code, textes, écran) ; il ne devient **atteignable** qu'ici. Par la valeur :
- `docs/api-external.md` — (a) tableaux de refus de `POST /invoices/{id}/settlements/{settlementId}/cancel`
  (ligne `FISCAL_YEAR_CLOSED` à `:379`) et de `POST /reconciliation/transactions/{id}/cancel` (`:462`) : une
  ligne « Règlement (rapprochement) lettré avec sa facture dans une période close — un administrateur fait
  reculer le verrou avant la date la plus récente du lettrage et/ou rouvre les exercices clôturés jusqu'à
  celui de cette date, selon la cause » | `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` | `409`, placée **après**
  `FISCAL_YEAR_CLOSED` et **avant** `MATCHED_BANK_TRANSACTION` ; (b) les deux listes **en prose** des refus
  fournisseurs — « Refus de l'annulation : … » (annulation du paiement, `:426`) et « Refus, dans l'ordre de
  précédence : … » (annulation de la facture, `:434`) — reçoivent `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`
  (`409`) **après** `FISCAL_YEAR_CLOSED` et **avant** `ACCOUNT_ARCHIVED` (findings R3-6 = F3-5 de la P3 de la
  15-1a2-ii : ce sont des phrases, non des tableaux — ancrer par le texte ; l'absence du rang 3 dans la liste
  du paiement fournisseur est un écart **préexistant**, issue #595) ; (c) **la table de référence des codes**
  (§ 10 « Gestion des erreurs », ≈ `:566`), qui range aujourd'hui le code sous les seules routes `/letterings`
  (findings M-2 = F-4 de la P3) : la cause s'étend — « et refus de l'annulation d'un règlement, d'un solde,
  d'un rapprochement, d'un paiement ou d'une facture fournisseur dont le lettrage est figé par une période
  close (§ des annulations) » ; le texte rendu diffère selon la route (celui du lettrage, celui de la famille
  d'annulation) — écrit une fois dans cette ligne. *(La ligne `:324`, message `LETTERING_IS_DOCUMENT`, est à
  AC12.)*
- `docs/manual/fr/user-manual.tex` — (a) les **deux listes exhaustives** de motifs : `:1198-1215`
  (§ `sec:reglement-client`, « le bouton est remplacé par la raison ») — un item « **le lettrage de la facture
  est figé par une période close** » **entre** « l'exercice du règlement est clôturé » et « le règlement est
  rapproché », avec le remède — **le texte de la clé fr-CH de la famille** (D2 de la 15-1a2-0), qui nomme
  les deux causes, verrou et exercice clôturé ou suivi d'un exercice clôturé (validation P5 ciblée, P5-1, P5-2) ; `:1770-1783` (§ `sec:annuler-rapprochement`) — même item **après**
  « l'exercice de l'écriture du rapprochement est clôturé » ; (b) les deux phrases-listes fournisseurs
  (`:1455-1459`, annulation du paiement ; `:1478-1484`, annulation de la facture) — le même motif, après
  l'exercice clôturé ; (c) la liste des exceptions de la contre-passation (`:2337-2342`, « ne s'annulent pas »)
  — le motif nommé, renvoi au § du verrou ; (d) § du verrou de période (`sec:verrou-periode`, `:562`) — une
  phrase après `:577-579` : « l'annulation d'un règlement, d'un solde, d'un rapprochement, d'un paiement ou
  d'une facture fournisseur dont le lettrage s'est figé avec la période est refusée ; un administrateur fait
  reculer le verrou avant la date la plus récente du lettrage — en général celle du dernier règlement — pour la
  permettre, et rouvre aussi les exercices clôturés si l'exercice de cette date l'est **ou s'il est suivi d'un exercice
  clôturé** — jusqu'à celui-ci, en commençant par le plus récent » (validation P5 ciblée, P5-1 : sans la seconde cause,
  couverte par la D2 de la 15-1a2-0 et ses douze textes, un lecteur qui recule le verrou resterait refusé) ; (e) **l'encadré
  `:588-594`** (« Ce que le verrou n'empêche pas, et c'est voulu » — « Une écriture d'une période verrouillée
  reste corrigeable par contre-passation ») et **la note `:626-631`** (« La contre-passation est *le* chemin de
  correction d'une écriture désormais figée ») : chacun reçoit l'exception — l'annulation d'un règlement (ou
  d'un paiement) dont le lettrage s'est figé avec la période est refusée, et le verrou doit reculer pour la
  permettre ; sans elle, le manuel promet une correction que Kesh refuse (validation P1 de la 15-1a2-0, F-1) ;
  (f) glossaire : AC12.
- `docs/manual/fr/admin-manual.tex:2101` (le verrou de période, OLICo Art. 9) — une phrase : le déverrouillage
  est aussi le remède du refus d'annuler un règlement dont le lettrage est figé, et la borne doit passer avant
  la date la plus récente du lettrage ; si l'exercice de cette date est clôturé ou suivi d'un exercice clôturé,
  l'administrateur les rouvre aussi, du plus récent jusqu'à celui-ci (finding F-4 point 3 de la P3 ; P5-1).
- `CHANGELOG.md` (`[0.13.0]`) : « l'annulation d'un règlement, d'un solde, d'un rapprochement, d'un paiement
  ou d'une facture fournisseur dont le lettrage est figé par une période close est refusée (`409
  LETTERING_ALL_LINES_IN_CLOSED_PERIODS`) ; un administrateur fait reculer le verrou avant la date la plus
  récente du lettrage et/ou rouvre les exercices clôturés, selon la cause ». *(« et/ou » : la remarque de la
  validation P4, lentille F, sur la version « ou » de la 15-1a2-0, qui contredisait sa D2. La 15-1a2-ii
  n'ajoute **pas** de seconde entrée du refus, validation P4 de la 15-1a2-ii, F4-2.)*
- **PDF** : `make fr` dans `docs/manual/`, les trois PDF commités ; contrôle **aplati** (`pdftotext … | tr '\n' '
  ' | tr -s ' '`) : l'item figure dans **chacune** des quatre listes de motifs du manuel utilisateur, dans la
  liste des exceptions (`:2337`), dans le § du verrou, dans l'encadré et dans la note ; la phrase figure dans le
  manuel d'administration.
- **Propagation par la valeur** : `git grep -nF "LETTERING_ALL_LINES_IN_CLOSED_PERIODS" -- docs CHANGELOG.md` —
  sur `056997b0`, **trois** sites (`api-external.md:314`, `:326`, `:566`), plus ceux qu'ajoute cet AC ; chaque
  site trié au Change Log. *(Les trois sites du code sont l'AC8 de la 15-1a2-0.)* Et le jeton du remède :
  `git grep -nE "dernier règlement|ou rouvre l.exercice" -- docs CHANGELOG.md` trié — aucun site ne doit
  prescrire « ou » seul.

## Tasks

- [x] **T0** — Relevés au sol sur la base du gate : (a) `EXPLAIN` de la requête de découverte de P3 (accès
      par clé primaire / `idx` des écritures, pas de balayage de `journal_entry_lines`) ; (b) re-greper les
      ancres de P4 par le nom (`grep -nF "pub async fn settle_invoice"`, etc.) sur la base réelle du
      développement ; (c) vérifier que les routes appelantes restent `Rejouee` (`audit_route_registry.rs`) :
      `POST /invoices/{id}/settlements`, `…/settlements/{settlementId}/cancel`, `…/write-off`,
      `POST /credit-notes`, `POST /reconciliation/accept`, `POST /reconciliation/transactions/{id}/cancel`.
- [x] **T1** (P3) — `letterings.rs` : `SyncOutcome`, `DocumentRef` (défini en P3, champs et sérialisation
      écrits), `create_group_inner` / `dissolve_group_inner` (+ `document`), extension de
      `audit_details`, `sync_invoice_in_tx`, `dissolve_invoice_document_group_in_tx` — sur la règle des
      périodes de la 15-1a2-0 (`open_period_rule`, `lines_in_open_period`), sans seconde factorisation ;
      doc-comments (verrous, abstention, compte non lettrable, précédence des issues) ; en-tête du module :
      les exceptions nommées au lettrage gagnent la synchronisation (elle **appelle** la primitive, n'écrit
      pas la marque). **Les énoncés de R3 qui nomment les deux primitives** (validation P4, F4-6 ; grep du **nom**
      `create_group_in_tx` dans les doc-comments) : `letterings.rs:10-11` (« UNE seule fonction écrit la marque
      — [`create_group_in_tx`] ») et `entities/journal_entry.rs:163-164` (« écrite par `create_group_in_tx` …
      et par elles seules ») deviennent « la marque s'écrit dans `create_group_inner` / `dissolve_group_inner`,
      atteints par les deux primitives et par la synchronisation seules » ; `letterings_lexical.rs:4-5` et son
      message d'échec `:291` suivent (T4) ; `kesh-core/src/lettering.rs:14` (« les fonctions de refus que la
      primitive appelle à son rang ») **reste vrai** — la primitive les appelle par son corps — : trié, non
      réécrit. Le décompte « exactement deux écritures de la marque » (`letterings_lexical.rs:299-303`) tient.
- [x] **T2** (P4) — Les cinq appels du tableau, chacun à la place indiquée, avec l'exercice tenu et
      l'acteur ; `cancel_settlement_in_tx` client garde l'`id` d'exercice de l'étape (2-bis).
- [x] **T3** (P4, AC15) — `accept_one_invoice` : appel après (g), mappage per-proposal (fonction pure
      `lettering_error_to_failed_proposal`, signature d'AC15 (d), à côté de `claim_account_failed_proposal`,
      testée).
- [x] **T4** (AC8 part i) — Tests lexicaux dans `letterings_lexical.rs` ; réalignement de
      `each_primitive_checks_the_rows_its_update_found` sur `*_inner`.
- [x] **T5** — Tests (liste ci-dessous) ; la **fixture partagée** d'AC5 vit **hors de `src/`**, dans
      **`crates/kesh-db/tests/support/lettering_documents.rs`**, fonction `seed_lettering_documents` et prédicat
      d'AC5 (validation P4, M-1 = F4-1 ; C-15-1a2-28). Elle est incluse par `#[path = "support/lettering_documents.rs"]
      mod lettering_support;` dans `lettering_documents.rs`, dans le binaire de rattrapage de la 15-1a2-ii (son
      AC6) et, pour AC15 (c), dans `kesh-api/tests/rejeu_interblocage_e2e.rs` par
      `#[path = "../../kesh-db/tests/support/lettering_documents.rs"]`. Un fichier d'un sous-dossier de `tests/`
      sans `main.rs` n'est pas une cible cargo ; chaque binaire n'en emploie qu'une partie : le module porte
      `#![allow(dead_code)]` en tête, **avec sa raison en commentaire** (fichier d'appui partagé par trois
      binaires, chacun sur un sous-ensemble) — non dans `tests/common/`, dont `mod.rs` sert cinq binaires de
      backfill sans `allow` (validation P3, F-2).
      ⛔ **Pas dans `kesh_db::test_fixtures`** (le choix de la P3, défait en P4) : ce module est compilé **en
      permanence** pour l'endpoint `_test/seed` (`test_fixtures.rs:11-15`, `lib.rs:16`) — c'est du **code de
      production** pour les deux détecteurs lexicaux, `no_production_code_writes_the_lettering_mark_outside_the_primitive`
      (`letterings_lexical.rs`, balaie `crates/*/src` hors `#[cfg(test)]`) et l'AC8 (a)–(c) d'ici. Les états
      hérités y exigeraient des littéraux que ces deux gardes refusent.
      **Fabrication des états hérités — tranché : SQL brut**, dans le fichier d'appui (C-15-1a2-28). Les
      détecteurs ne lisent que `crates/*/src` : `tests/support/` est **hors de leur balayage**, par
      construction (`letterings_lexical.rs:6-11`, et AC8 « du code de production ») — ils ne s'en émeuvent
      pas, et c'est juste : rien de ce qui s'y écrit n'est atteignable par l'application. Écartée,
      `dissolve_group_in_tx` en mode `System` : elle exige un exercice tenu `FOR UPDATE` et **écrit une entrée
      d'audit** `lettering.removed`, qu'aucune donnée réelle héritée ne porte — elle fausserait AC10 et le
      volet « aucune entrée d'audit » d'AC13 et d'AC6 (e) de la 15-1a2-ii. Recettes, chacune suivie d'une
      **assertion de montage** (lignes trouvées par l'`UPDATE`, marques relues) :
      - **facture créditée et réglée** : gabarit « détacher, créditer, rattacher » d'`invoice_settlement.rs:985-1005`
        — mais `create_credit_note` **lettre** désormais pendant le détachement (`C(I)` = {vente, avoir},
        somme nulle) : la recette **efface ensuite** ces marques par `UPDATE journal_entry_lines SET
        lettering_key = NULL, lettering_origin = NULL WHERE lettering_key = ?`, puis rattache le règlement ;
      - **avoir hérité sur un autre compte** (exception (a) d'AC5, AC3) : avoir émis par le geste (qui le
        lettre), marques effacées comme ci-dessus, **puis** `UPDATE journal_entry_lines SET account_id = …` de
        sa ligne de créance ;
      - **compte de créance non lettrable** (exception (b) d'AC5, AC13) : voie « compte bancaire » — lier `A` est
        refusé par `refuse_if_ledger_is_claim_account` tant que `A` est **désigné** (`bank_accounts.rs:741-760`) :
        lien posé en SQL brut, ou un autre compte désigné d'abord ;
      - **pièce passée sous la borne** : geste en période ouverte, puis verrou (une écriture ne se crée pas sous
        la borne) ;
      - **rapprochement** (validation P4, F4-4) : `accept_one_invoice` vit dans `kesh-api`, inatteignable de
        `kesh-db/tests` : `settle_invoice` (qui lettre), puis `UPDATE bank_transactions SET matched_entry_id =
        <écriture de règlement>, status = 'reconciled'` — le dé-rapprochement retrouve le règlement par
        `invoice_settlements.journal_entry_id = matched_entry_id` (`reconciliation_cancel.rs:76-79`). ⚠️ Une ligne
        `invoice_settlements` posée en SQL brut, comme le font `reconciliation_e2e.rs` et d'autres, ne lettrerait
        rien et ferait rougir AC5 : à ne pas imiter ici. Le chemin de synchronisation d'`accept_one_invoice`
        lui-même est couvert côté `kesh-api` (AC15 a, c).
      La fixture contient **les deux exceptions d'AC5** (validation P4, L-6) — sans elles, le filtre d'exceptions
      du test d'accord ne serait exercé par aucune facture, et une exception trop large resterait verte.
- [x] **T6** (AC12, AC18) — CHANGELOG, `api-external.md` (`:223`, `:291`, `:299`, `:324` ; et, pour AC18, `:379`,
      `:426`, `:434`, `:462`, `:566`), manuels FR (utilisateur : AC12 et les sites d'AC18, encadré `:588-594` et
      note `:626-631` compris ; administration `:2101`) + `make fr` + PDF aplati, greps de propagation
      (AC12, AC18).

**Tests prévus** (27 neufs, 1 étendu) :
- `crates/kesh-db/tests/lettering_documents.rs` (neuf, `test-schema`) — 20 :
  `full_settlement_letters_sale_and_settlements` (AC1), `three_partials_with_internal_account_letter_on_the_last` (AC1),
  `partial_settlement_letters_nothing` (AC1), `write_off_each_kind_letters` (AC2),
  `rounding_line_is_left_out` (AC2), `credit_note_letters_invoice_and_note` (AC3),
  `legacy_credit_note_on_other_account_forms_no_group` (AC3), `cancel_settlement_dissolves_and_pairs` (AC4),
  `cancel_write_off_dissolves_and_pairs` (AC4, L-5), `unreconcile_dissolves_and_pairs` (AC4),
  `cancel_partial_without_group_is_a_noop` (AC4), `closed_year_n_settled_and_cancelled_in_n1` (AC4, P5),
  `ledger_agrees_with_amount_due` (AC5), `audit_details_carry_the_invoice` (AC10 — dont `documentNumber`
  présent ; **les deux événements** — `lettering.created` par le règlement, `lettering.removed` par
  l'annulation ; validation P4, L-5), `receivable_not_letterable_is_skipped` (AC13, **trois** volets : règlement
  sur `A` non lettrable ; synchronisation appelée sur un groupe survivant → `AccountNotLetterable`, marques
  inchangées, aucune entrée d'audit — étape 3 terminale, validation P4, M-2 ; dissolution du survivant à
  l'annulation — et la précédence `AccountNotLetterable` sur `AbstainedClosedPeriods`),
  `locked_period_cancel_is_refused_until_unlocked`
  (AC14 a — règlement, solde —, b), `historical_closed_history_abstains` (AC14 c),
  **`sync_is_idempotent`** (P3 : un second appel sur une facture lettrée rend `Unchanged`, n'écrit rien, ne
  produit aucune entrée d'audit — validation P3, F-5), **`sync_never_overwrites_a_foreign_mark`** (P3
  étape 2, fabriqué en SQL brut : ligne de vente marquée `manual`, puis `reversal`, puis deux clés distinctes
  sur `C(I)` → `DbError::Invariant` à chaque fois, **marques inchangées** — F-5, L-3),
  **`sync_step_5_recreates_or_abstains`** (P3 étape 5, défensive, fabriquée en SQL brut : groupe `document`
  d'une autre cible avec une ligne ouverte → `Recreated` ; le même entièrement sous la borne →
  `AbstainedClosedPeriods`, rien d'écrit — L-3 : ces branches ne sont atteintes par aucun geste, un test les
  empêche de devenir muettes ; ⚠️ montage : l'exercice tenu doit couvrir une ligne **du groupe `k`** et une de
  `T`, faute de quoi `check_held_fiscal_year` rend `Invariant` au lieu de `Recreated` — validation P4,
  F4-7) ;
- `crates/kesh-db/tests/letterings.rs` — `lettering_invariants` **étendu** (AC9 part i ; pas un test neuf) ;
- `crates/kesh-db/tests/letterings_lexical.rs` — 3 neufs : `invoice_settlement_writers_stay_in_their_module_and_sync`
  (AC8 a, b), `credit_note_insert_is_followed_by_sync_and_cancel_dissolves_first` (AC8 c, d),
  `the_function_body_detector_sees_calls_and_order` (synthétique) ;
- `crates/kesh-api/tests/reconciliation_e2e.rs` — 2 neufs : `accept_letters_a_fully_settled_invoice` (AC15 a, sous
  une clé d'API : `api_key_id` de l'entrée `lettering.created` asserté — validation P4, L-5),
  `accept_race_refuses_before_any_lettering` (AC15 b) ;
- `crates/kesh-api/tests/rejeu_interblocage_e2e.rs` — 1 neuf : `accept_and_settlement_cancel_interleave` (AC15 c) ;
- `crates/kesh-api/src/routes/reconciliation.rs` (`mod tests`) — 1 neuf :
  `lettering_errors_map_to_failed_proposals` (AC15 d).

*(Recompte depuis cette liste : 20 + 3 + 2 + 1 + 1 = **27 fonctions de test neuves** (toutes Rust), plus **1
test étendu** (`lettering_invariants`). Partis à la 15-1a2-0, **sous les noms qu'elle leur donne** (validation P1
de la 15-1a2-0, R-7) : `later_closed_year_group_is_refused` (ex-`later_closed_year_cancel_is_refused`), la
précédence — devenue la matrice `RANGS` à six rangs de `invoice_settlement.rs`
(`la_precedence_de_l_annulation_lecture_et_ecriture`, ex-`closed_lettering_rank_precedes_bank_match`) —,
`unreconcile_of_a_closed_lettering_is_refused_in_its_family`,
`settlement_cancel_blocked_by_closed_lettering`, `closed_lettering_texts_follow_their_family`, le test Vitest
d'`invoice-cancel.ts` et les deux fichiers Vitest étendus.)*

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
  Le rang 2 bis (15-1a2-0) n'ajoute **aucun** verrou.
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
  - les tests de la file commune et des textes : relèvent de la 15-1a2-0, qui ajoute le rang ;
    ⚠️ le commentaire d'`invoices.rs:1667-1673` (« la marque de lettrage … inatteignable ici »,
    dévalidation) **reste vrai** : sans groupe gardé, aucune facture sans règlement n'a de ligne de vente
    lettrée (validation P2, M-2 — dépendait du groupe gardé).
- **Dépendances** : 15-1a-i, 15-1a-ii (mergées), **15-1a2-0** (la règle des périodes et le refus du rang
  2 bis, dormant jusqu'ici). 15-6a/b/c/d **mergées** (`39b52628`, `f8b2accd`,
  `803f3e15`, `1ae3963e`) : les avertissements #473/#474 de la fiche d'origine sont retirés.
- **Règle de découpage** — comptée aux **deux** grains (validation P3, L-7 = F-1 : la version P2 comptait
  « par domaine », grain que la règle n'emploie pas ; C-15-1a2-21) :
  - au grain « **crates Rust, packages npm** » : `kesh-db`, `kesh-api` = **2** ;
  - au grain des **modules métier de premier niveau** (patron `kesh-api/routes/invoices`) :
    `kesh-db/repositories/letterings` (synchronisation, audit), `kesh-db/repositories/invoice_settlements_write`
    (trois appels), `kesh-db/repositories/credit_notes` (un appel), `kesh-db/entities/journal_entry`
    (doc-comment R3 seul, F4-6), `kesh-api/routes/reconciliation` (un appel, mappage) = **5** — au seuil
    (« plus de 5 »), non franchi. *(Validation P4 : `kesh-db/test_fixtures` **sort** du compte — la fixture
    passe sous `tests/support/`, M-1 — et `entities/journal_entry` y **entre** pour un doc-comment, F4-6 ;
    `kesh-core/lettering` reste vrai, non touché.)*
  La documentation (CHANGELOG, `api-external.md`, deux manuels — AC12 et AC18) n'est pas un module au sens de
  la règle ; elle est déclarée à part : AC18 ajoute de la **documentation**, aucun module. Ne changent pas :
  `invoice_settlements.rs`, `routes/letterings.rs`, `journal_entries.rs`, `kesh-db/src/errors.rs`,
  `kesh-api/src/errors.rs`, `kesh-i18n`, `frontend/` (tous à la 15-1a2-0, ou intouchés).

### Dérogation règle de splitting — retirée

*(C-15-1a2-13, retirée à la validation P3 — finding F-1, décision de l'orchestrateur, C-15-1a2-19.)* La
dérogation de la P2 gardait ici le refus du rang 2 bis au motif qu'une sous-story « refus » mergée **après**
la synchronisation laisserait un état défaillant entre deux merges. L'ordre inverse — **le refus d'abord,
dormant** — n'avait pas été examiné ; il ne laisse aucun état défaillant (aucun groupe `document` n'existe
avant cette story) et se teste en SQL brut. Le refus est devenu la **15-1a2-0** ; cette fiche revient à cinq
modules au grain le plus fin (ci-dessus) et n'a plus de dérogation.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`), en autonomie, worktree `kesh-15-1a2-i`, base `46b08cde` (tête de la 15-1a2-0).

### Completion Notes List

**T0 — relevés au sol** (base du gate `kesh_1a2i`). (a) `EXPLAIN` de la découverte : sur table peu peuplée,
l'optimiseur prenait `idx_jel_account` pour les lignes (le `FOR UPDATE` aurait verrouillé toutes les lignes du
compte de créance), `idx_journal_entries_company_date` pour l'en-tête et `idx_credit_notes_company_status` pour
l'avoir → index **forcés** (`idx_jel_entry`, `STRAIGHT_JOIN … FORCE INDEX (PRIMARY)`, `uq_credit_notes_invoice`,
`idx_invoice_settlements_company_invoice`), plans relus : `range idx_jel_entry` puis `eq_ref PRIMARY`
(C-15-1a2-i-2). ⚠️ Observation **hors périmètre** : l'acte 1 de la primitive (`LOCK_LINES_BY_ID_SQL`, 15-1a-i)
présente la même dégénérescence de plan sur base vide (`ref idx_journal_entries_company_date`) ; non touché,
signalé. (b) ancres de P4 re-grepées par le nom (`settle_invoice`, `write_off_invoice`, `cancel_settlement_in_tx`,
`create_credit_note`, `accept_one_invoice`, `claim_account_failed_proposal`) : présentes, conformes. (c) les six
routes appelantes sont `Rejouee` (`audit_route_registry.rs` : `credit_notes::create_credit_note`,
`invoices::settle_invoice_handler`, `write_off_invoice_handler`, `cancel_invoice_settlement_handler`,
`reconciliation::post_accept`, `post_cancel_reconciliation`).

**T1** — `letterings.rs` : `SyncOutcome`, `DocumentRef`, corps privés `create_group_inner` / `dissolve_group_inner`
(les primitives publiques y délèguent avec `None`), `audit_details(group, document)` (trois clés, `documentNumber`
présent et `null` sans numéro), `sync_invoice_in_tx` (étapes 1 à 6, étape 3 terminale), 
`dissolve_invoice_document_group_in_tx`, sur `lines_in_open_period` de la 15-1a2-0 (aucune seconde
factorisation). Énoncés R3 réécrits : en-tête du module, `entities/journal_entry.rs` (doc de `lettering_key`),
`letterings_lexical.rs` (en-tête, message d'échec). Triés **vrais, non réécrits** (ils parlent de la primitive
vue par les routes) : `kesh-core/src/lettering.rs:14`, `kesh-db/src/errors.rs:1129`,
`kesh-db/tests/letterings.rs:2`, `kesh-api/tests/audit_route_registry.rs:248`, `journal_entries.rs:2364`.
Brouillon → `Unchanged` (C-15-1a2-i-3). « Exactement deux écritures de la marque » : tient (`letterings_lexical`).

**T2** — cinq appels : `settle_invoice` et `write_off_invoice` après l'`UPDATE invoices` (exercice `fy`),
`create_credit_note` après la bascule (11) (exercice de l'avoir), `cancel_settlement_in_tx` après les refus de
l'étape (3), avant `reverse_owned_in_tx` (l'`id` d'exercice de l'étape (2-bis) est gardé). **T3** — `accept_one_invoice`
après (g), `lettering_error_to_failed_proposal` à côté de `claim_account_failed_proposal`. **T4** — trois tests
lexicaux neufs, `each_primitive_checks_the_rows_its_update_found` réaligné sur `*_inner`. **T5** — fixture
`tests/support/lettering_documents.rs` (C-15-1a2-28 ; incluse par `lettering_documents.rs`, `letterings.rs` (AC9)
et `rejeu_interblocage_e2e.rs`) ; `support/document_group.rs` de la 15-1a2-0 trouve désormais le groupe posé par
le geste (C-15-1a2-i-1). **T6** — CHANGELOG (deux entrées, AC12 et AC18), `api-external.md`, manuels FR, README
(feuille de route, C-15-1a2-i-4), `make fr`.

**Reçu B-2 de la 15-1a2-0** (texte `reconciliation-cancel-blocked-lettering-closed` pour un lien `Entry` hérité
qui pointe la vente) : le texte reste vrai sur le fond — la vente est dans le groupe de sa facture —, mais son
remède ne lève que le rang 2 bis ; le socle refuse ensuite par `OWNED_BY_INVOICE`. L'état est déclaré **inexistant**
par l'arbitrage Q1 (`reconciliation_cancel.rs:26-31` : Kesh n'est pas en production, aucun chemin) : trié, texte
**non réécrit**.

**Tests** (périmètre `46b08cde` → commit de développement ; `git diff 46b08cde -- crates | grep -cE
'^\+\s*#\[(sqlx::test|tokio::test|test)'` = **28**) : `lettering_documents.rs` **21** (les 20 prévus +
`only_the_anchor_of_the_sale_is_in_the_group`, ajouté après la survie de la mutation M12), `letterings_lexical.rs` 3,
`reconciliation_e2e.rs` 2, `rejeu_interblocage_e2e.rs` 1, `routes/reconciliation.rs` 1 ; **1 étendu**
(`lettering_invariants`, AC9 : `violations_des_pieces` et deux contrôles négatifs). AC15 (c) porte un **témoin**
(rapprochement seul, accepté et lettré) pour ne pas être vert à vide.

**Mutations** (exécutées une à une, restaurées, fichier retouché ; filtre : binaires `lettering_documents`,
`letterings_lexical`, `letterings`, ou les tests `kesh-api` visés) — **12 tuées sur 12** : M1 sans synchronisation au
règlement (16 rouges), M2 au solde (5), M3 sans dissolution à l'annulation (8), M4 sans synchronisation à l'avoir (4),
M5 étape 3 non terminale (1 : `receivable_not_letterable_is_skipped`), M6 somme ignorée (11), M7 périodes ignorées
(2), M8 audit sans pièce (1), M9 `LETTERING_CONCURRENT_CHANGE` mappé hors `INTERNAL_ERROR` (1), M10 sans
synchronisation au rapprochement (2), M11 marque étrangère acceptée (1), M12 ancre élargie à toute ligne de vente
(survivante au premier passage → test ajouté → 1). ⚠️ Les mutations « appel neutralisé par `if false` » (M3, M4)
ne font pas rougir les détecteurs lexicaux (le texte de l'appel reste) : c'est voulu, ils gardent l'inventaire,
les tests de dépôt gardent le comportement.

**Gates au commit de développement** : `scripts/test-fast.sh --no-lint` (avant les tests neufs) **3233 passés, 4
ignorés** ; binaires ciblés verts (`lettering_documents` 21/21, `letterings` 33/33, `letterings_lexical` 6/6,
`rejeu_interblocage_e2e` 11/11, trois tests `kesh-api` ciblés). **Gate complet, frontend et E2E : au dernier commit
de code, après le rebase sur `main`.** Frontend non touché (`git diff --stat 46b08cde -- frontend` vide).

**Contrôles de documentation.** PDF aplati : `(ne se lettrent pas encore|Kesh ne lettre pas encore)` → 0 ; l'item du
motif dans les quatre listes du manuel utilisateur, la liste des exceptions, le § du verrou (deux causes),
l'encadré, la note et le glossaire, la phrase du manuel d'administration — chacun présent. Grep d'AC12 : un seul
résidu, `CHANGELOG.md:15` « ne se lettrent pas à la main » (énoncé vrai du lettrage manuel, trié légitime). Grep
de la valeur `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (`docs`, `CHANGELOG.md`) : 3 sites d'origine
(`api-external.md` lettrage `POST`, `DELETE`, table § 10 — cette dernière complétée) + **6** neufs (CHANGELOG,
`api-external.md:293`, 2 tableaux et 2 listes en prose des annulations) = **9** *(corrigé en revue P1, A-3 : « 5 neufs »
oubliait `:293`)* — chacun le refus du 2 bis ou celui du lettrage manuel. Remède :
aucun site ne prescrit « ou » seul.

**Choix consignés** : C-15-1a2-i-1 à C-15-1a2-i-4.

### File List

- `crates/kesh-db/src/repositories/letterings.rs` — synchronisation, `DocumentRef`, `SyncOutcome`, corps `*_inner`, audit
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs` — trois appels (règlement, solde, annulation)
- `crates/kesh-db/src/repositories/credit_notes.rs` — un appel (avoir)
- `crates/kesh-db/src/entities/journal_entry.rs` — doc-comment R3
- `crates/kesh-api/src/routes/reconciliation.rs` — appel, `lettering_error_to_failed_proposal`, test
- `crates/kesh-db/tests/lettering_documents.rs` — **neuf**
- `crates/kesh-db/tests/support/lettering_documents.rs` — **neuf**
- `crates/kesh-db/tests/support/document_group.rs`, `crates/kesh-db/tests/letterings.rs`, `crates/kesh-db/tests/letterings_lexical.rs`
- `crates/kesh-api/tests/reconciliation_e2e.rs`, `crates/kesh-api/tests/rejeu_interblocage_e2e.rs`
- `CHANGELOG.md`, `README.md`, `docs/api-external.md`
- `docs/manual/fr/user-manual.tex`, `admin-manual.tex` et les trois PDF
- `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`, `sprint-status.yaml`

## Change Log

### Revue de code P1 — 2026-10-09 (Sonnet 5.5 ×3, lentilles B, E, A ; remédiation Opus 5.5)

**Prompt** : `15-1a2-i-review-prompt-p1.md`. **Rapports** : `kesh-gate-logs/15-1a2-i-review-p1-{B,E,A}.md` — B **0 C / 0 H /
0 M / 4 L**, E **0 / 0 / 0 / 3 L**, A **0 / 0 / 1 M / 6 L** ; recoupement B-2 = E-1 → **1 MEDIUM, 12 LOW distincts**.
Affirmations vérifiées par `grep -nF` (A-1 : `user-manual.tex:1246` « jusqu'à ce qu'un administrateur rouvre la
période » ; `sync_invoice_in_tx(` appelé aux quatre seuls sites de geste ; A-2 : `letterings.rs:878`, `:1028`).
Axes non exercés des trois lentilles : toute exécution — reprise par l'orchestrateur (gate complet, ci-dessous ;
mutations au Dev Agent Record).

| finding | sév. | verdict |
|---|---|---|
| A-1 — paragraphe *Lettrage* du manuel : « deux cas » (il y en a plus) et « jusqu'à ce qu'un administrateur rouvre la période » (rien ne relettre à la réouverture) | MEDIUM | **corrigé** : critère du grand livre, cas listés (période close, créance non lettrable, deux états hérités), « rouvrir la période ne la lettre pas d'elle-même », « au moment du geste qui la solde » ; `make fr`, PDF aplati contrôlé. Les factures soldées **avant** la mise à jour relèvent du rattrapage de la 15-1a2-ii, qui le documente (C124 : aucun tag entre les deux) |
| A-2 — « la seule fonction qui écrit / efface la marque » sur les primitives publiques | LOW | **corrigé** (doc-comments renvoient aux corps `*_inner`) |
| A-3 — décompte des sites de la valeur (5 neufs → 6, total 9) | LOW | **corrigé** (Dev Agent Record) |
| A-4 — `documentNumber: null` non exercé | LOW | **corrigé** : test unitaire `audit_details_carry_the_document_or_nothing` (`letterings.rs`, `mod tests`) |
| A-5 — volets (c), (d) d'AC8 non éprouvés sur source synthétique | LOW | **corrigé** : `appelle_avant` extraite, (c) et (d) éprouvés dans `the_function_body_detector_sees_calls_and_order` |
| A-6 — `UPDATE` du test d'avoir hérité sans assertion de montage | LOW | **corrigé** (`rows_affected == 1`) |
| A-7 — Status : « validation P5 à mener » | LOW | **corrigé** |
| B-1 — cycles *rapprochement ‖ rapprochement* et *‖ avoir* non nommés | LOW | **corrigé** (doc du module) |
| B-2 = E-1 — verrous d'intervalle des lectures `FOR UPDATE` de `invoice_settlements` / `credit_notes` | LOW | **nommé** au module (attente d'une facture voisine, au pire interblocage rejoué) ; non mesuré |
| B-3 — refus du mode `Manual` mappés `DATABASE_ERROR` | LOW | **corrigé** : `INTERNAL_ERROR` (inatteignables en `System`), test étendu |
| B-4 — un règlement sans ligne sur `A` ferait sortir `Invariant` de la dissolution | LOW | **réfuté** : `create_in_tx` des trois écrivains crédite toujours `A` (`settlement_journal_lines`, `write_off_journal_lines`) ; un règlement hérité sans ligne sur `A` ne laisse pas la facture soldée sur `A` (reste dû négatif, trop-perçu refusé) — aucun groupe ne le contient |
| E-2 — lectures ordinaires (étapes 4, 5, lettrabilité, périodes) sur l'instantané d'`accept_batch` | LOW | **accepté** : le groupe `k` d'une facture ne change que sous le verrou de la facture, tenu par l'`UPDATE` de (g) avant la synchronisation ; pire cas une abstention ou une recréation superflue, jamais un échec du geste |
| E-3 — bords sans test dédié (arrondi négatif réel, règlement daté la veille, restauration, règlements concurrents, `ENTRY_LETTERED` sur une ligne `document`) | LOW | **accepté** : le filtre de l'ancre est tenu par `only_the_anchor_of_the_sale_is_in_the_group` ; la garde `ENTRY_LETTERED` est indépendante de l'origine (15-1a-ii) ; la restauration et son rejeu sont la 15-1a2-ii |

**Gate au commit de remédiation** (base `kesh_1a2i` remise à zéro, sans redémarrer MariaDB) : `scripts/test-fast.sh`
**3259 passés, 4 ignorés** (+1 : le test unitaire d'A-4). Signal D5 : sans objet (sévérité en baisse). La remédiation
touche du code de production (`letterings.rs` : doc-comments et `mod tests` ; `routes/reconciliation.rs` : quatre bras
du mappage) et deux modules : passe **P2 complète, Opus**.

### Développement — 2026-10-09 (Opus 5.5, `bmad-dev-story`, en autonomie)

T0–T6 livrées : synchronisation idempotente des factures clientes (règlement, solde, avoir, rapprochement),
dissolution à l'annulation, audit avec la pièce, inventaire fermé tenu par trois tests lexicaux, documentation
publique du lettrage et du refus du rang 2 bis (AC12, AC18). 28 tests neufs + 1 étendu, 12 mutations tuées.
Choix C-15-1a2-i-1 à 4. Détail au Dev Agent Record. Prochaine étape : rebase sur `main` (15-1a2-0 fusionnée,
`a06e1927`), puis `bmad-code-review` P1.

### Validation P4 — 2026-10-09 (Opus 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, seul remédiateur des fiches de la suite du lettrage, en autonomie)

**Rapports** : `kesh-gate-logs/15-1a2-i-validate-p4-R.md` (**0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW**) et `…-F.md`
(**0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW**, plus une observation sur la 15-1a2-0). Recoupements : M-1 = F4-1 ; M-2 =
F4-3 (MEDIUM chez R, LOW chez F : compté MEDIUM) ; F4-2 = L-4 (MEDIUM chez F, LOW chez R : compté MEDIUM) →
**3 MEDIUM distincts** et **10 LOW distincts** (L-1, L-2, L-3, L-5, L-6 ; F4-4 à F4-8). **Trend** : P1 (fiche
mère) **3 HIGH / 7 MEDIUM** (R), **2 HIGH / 7 MEDIUM** (F) → P2 **0 HIGH / 4 MEDIUM distincts** → P3 **0 HIGH / 4
MEDIUM distincts** → P4 **0 HIGH / 3 MEDIUM distincts**.

⚠️ **Signal D5 levé, déclaré, non découpé.** Les trois MEDIUM sont **RECYCLÉS** — tous nés de la remédiation P3
(`76e7893a`), aucun de la conception d'origine : M-1 du correctif de F-2 (la fixture placée dans un module
compilé en production), M-2 de celui de F-7 / L-1 (une phrase de précédence ajoutée sans réécrire l'étape 3),
F4-2 du découpage (la ligne `:324` laissée entre les deux fiches). **Pourquoi pas de découpage** (constat écrit,
comme l'exige l'amendement D5) : ce sont des défauts **locaux** — un emplacement de fichier de test, une étape
de l'algorithme écrite sans le mot « terminale », un propriétaire de ligne de documentation —, sans dispersion :
la fiche **perd** un module (`test_fixtures`) et n'en gagne qu'un pour un doc-comment (5 au grain fin, 2 crates,
sous le seuil). Découper ne séparerait aucun de ces défauts de sa cause. Le prochain recyclage, s'il touche
encore une phrase ajoutée par la remédiation précédente, impose une passe **ciblée** sur ce commit plutôt
qu'une passe complète (§ « La passe ciblée »).

| finding | sévérité | verdict | où |
|---|---|---|---|
| M-1 = F4-1 — la fixture partagée dans `kesh_db::test_fixtures`, compilé en production, que balaient les deux détecteurs lexicaux ; l'état « créditée et réglée » s'y fabrique mal (l'avoir lettre pendant le détachement) | MEDIUM | **corrigé** : `crates/kesh-db/tests/support/lettering_documents.rs`, inclus par `#[path]` (deux binaires de `kesh-db`, un de `kesh-api`), `#![allow(dead_code)]` justifié ; états hérités en **SQL brut**, `dissolve_group_in_tx(System)` écartée (audit parasite) ; recettes écrites, marques effacées après l'avoir ; vérifié : `test_fixtures.rs:11-15`, `lib.rs:16`, `letterings_lexical.rs:6-11` (C-15-1a2-28) | T5, AC5, Dev Notes |
| M-2 = F4-3 — l'étape 3 ne dit pas si elle termine ; la précédence contredit l'étape 5 ; un survivant de C104 serait dissous | MEDIUM | **corrigé** : étape 3 **terminale**, rien n'est écrit quel que soit `E` ; précédence réécrite ; étape 6 alignée ; AC13 (a) et troisième volet de `receivable_not_letterable_is_skipped` ; AC6 (e) de la 15-1a2-ii aligné (C-15-1a2-29) | P3, AC13, tests, 15-1a2-ii AC6 |
| F4-2 = L-4 — `api-external.md:324` (« annuler le règlement, pas délettrer ») réécrit par aucune des deux fiches | MEDIUM | **corrigé** : propriétaire unique **ici** (AC12 : ligne réécrite sur le message de la 15-1a2-0 D5, annotation retirée) ; jeton `pas délettrer` ajouté au grep | AC12 |
| L-1 — AC14 énoncé en HTTP, testé au dépôt ; « N+1 » sans exercice N+1 | LOW | **corrigé** (variante `SettlementNotCancellable`, `companies::unlock_books` ; « après la nouvelle borne ») | AC14 |
| L-2 — l'étape 2 justifie par un ordre faux (la contre-passation précède le `DELETE`) | LOW | **corrigé** (vérifié `invoice_settlements_write.rs:852`, `:862`) | P3 étape 2 |
| L-3 — contrôle d'AC12 : faux positifs, « quatre langues » | LOW | **corrigé** (jeton resserré, cinq sites nommés sur la base — rejoué ici —, « en français ») | AC12 |
| L-5 — dissolution et clé d'API sans test nommé | LOW | **corrigé** (`audit_details_carry_the_invoice` sur les deux événements ; `api_key_id` sous clé d'API dans `accept_letters_a_fully_settled_invoice`) | AC15, tests |
| L-6 — la fixture d'AC5 sans ses deux exceptions | LOW | **corrigé** (recettes de T5) | T5 |
| F4-4 — rapprochement inatteignable de `kesh-db/tests` | LOW | **corrigé** (recette : `settle_invoice` puis `matched_entry_id` en SQL ; ligne `invoice_settlements` brute proscrite ; `accept_one_invoice` couvert côté `kesh-api`) | T5 |
| F4-5 — `DocumentRef.document_type` littéral contre `DocumentKind::as_str` | LOW | **porté à la 15-1b-0** (sa T2 type le champ en `DocumentKind`, C-15-1b-0-3) ; doc-comment d'ici le dit | P3, 15-1b-0 |
| F4-6 — énoncés R3 qui nomment `create_group_in_tx` | LOW | **corrigé** (trois sites réécrits, `kesh-core/src/lettering.rs:14` trié : reste vrai) | T1, Dev Notes |
| F4-7 — montage de `sync_step_5_recreates_or_abstains` | LOW | **corrigé** (exercice tenu couvrant `k` et `T`) | tests |
| F4-8 — README « Feuille de route » : E15 « 📋 Backlog » | LOW | **hors fiche** (préexistant, imputable à l'epic) — **signalé à l'orchestrateur** ; la 15-1a2-ii AC12 et la 15-1c T7 le vérifient déjà | — |

**Décision de l'orchestrateur appliquée** (C-15-1a2-24) : la **documentation publique** du refus du rang 2 bis
— ex-AC8 et T7 de la 15-1a2-0, plus l'encadré `user-manual.tex:588-594` et la note `:626-631` (F-1 de sa P1),
`:2337`, le glossaire, `admin-manual.tex:2101`, les listes et la table § 10 d'`api-external.md`, le CHANGELOG
(« et/ou », remarque de la lentille F sur la 15-1a2-0) — devient l'**AC18** d'ici ; T6 l'exécute.
**Propagation** (valeurs grepées sur les fiches 15-1a2-0, -i, -ii, 15-1b, 15-1b-0, l'index, le registre et le code
cité) : `kesh_db::test_fixtures`, `seed_lettering_documents`, `15-1a2-0, AC8`, `15-1a2-0 AC8`, `n'existe encore`,
« dans les quatre langues », `daté en N+1`, « Précédence des issues », `later_closed_year_cancel_is_refused`,
`closed_lettering_rank_precedes_bank_match` — résidus : Change Logs (historique) et prompts versionnés.
**Recompte** (depuis ce fichier) : **13 critères** actifs (AC1–AC5, AC8–AC10, AC12–AC15, **AC18** ; AC17 déplacé,
numéro non réattribué), **7 tâches** (T0–T6), **27 tests neufs** (tous Rust) **+ 1 étendu** — inchangés en
nombre (trois volets et deux assertions ajoutés à des tests existants de la liste). Modules : **2** crates,
**5** modules métier. Choix consignés : **C-15-1a2-24, 28, 29**. Prochaine passe : **P5** — **ciblée** possible
(une lentille, braquée sur ce commit : la remédiation est locale — un emplacement de fixture, une étape, une
ligne de documentation — plus un AC de documentation **déplacé** sans règle neuve) ; elle doit relire AC18
contre le code de la 15-1a2-0 et contre le PDF, et la recette des états hérités de T5.

### Validation P3 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, seul remédiateur des fiches de la suite du lettrage, en autonomie)

**Rapports** : `kesh-gate-logs/15-1a2-i-validate-p3-R.md` (**0 CRITICAL, 0 HIGH, 2 MEDIUM, 7 LOW**) et
`…-F.md` (**0 CRITICAL, 0 HIGH, 2 MEDIUM, 6 LOW**). Recoupements : M-2 ≈ F-4 point 1 (MEDIUM chez R, LOW chez
F : compté MEDIUM), M-1 ≈ F-4 point 2, L-1 = F-7, L-7 ≈ F-1 point 1, L-6 ≈ F-6 → **4 MEDIUM distincts** (M-1,
M-2, F-1, F-2) et **13 LOW distincts** — une ligne chacun ci-dessous (le rapport R annonce « 7 LOW » en tête
et en **détaille neuf**, L-1 à L-9 : recomptés depuis le corps du rapport, neuf). **Trend** (la P1 portait sur la fiche mère) : P1 **3 HIGH / 7 MEDIUM**
(R), **2 HIGH / 7 MEDIUM** (F) → P2 **0 HIGH / 4 MEDIUM distincts** → P3 **0 HIGH / 4 MEDIUM distincts**.
⚠️ **Signal D5 levé** (sévérité égale, MEDIUM → MEDIUM, **et** trois des quatre MEDIUM nés de la remédiation
P2 : M-1 et M-2 de la documentation du refus, F-1 de sa dérogation) — **suivi** : décision de l'orchestrateur,
le refus est **extrait** en **15-1a2-0** (C-15-1a2-19), la dérogation C-15-1a2-13 **retirée**. F-2 est d'origine
(P1). Chaque finding relu au code (`grep -nF` / `sed -n` sur `056997b0`).

| finding | sévérité | verdict | où |
|---|---|---|---|
| F-1 — la dérogation n'examine qu'un ordre de merge ; décompte par domaine | MEDIUM | **corrigé par découpage** : le refus d'abord, dormant → **15-1a2-0** ; modules comptés aux deux grains (2 / **5**) | Status, P4, P7, Dev Notes ; C-15-1a2-19, 21 |
| F-2 — fixture partagée dans `tests/common/` : `dead_code`, `clippy -D warnings` | MEDIUM | **corrigé** : `kesh_db::test_fixtures::seed_lettering_documents` (vérifié : `tests/common/mod.rs` sans `allow(dead_code)`, cinq binaires de backfill ; `pub mod test_fixtures`, `lib.rs:16`) ; remarque AC13 (`refuse_if_ledger_is_claim_account`) écrite | T5 |
| M-1 (≈ F-4 point 2) — les deux listes **exhaustives** de motifs du manuel non nommées | MEDIUM | **corrigé, porté à la 15-1a2-0** (AC8 : `:1198-1215`, `:1770-1783`, plus `:1455`, `:1478`, `:2337`) | 15-1a2-0 AC8 |
| M-2 ≈ F-4 point 1 — `api-external.md:566` range le code sous `/letterings` seul ; grep par formulation | MEDIUM | **corrigé, porté à la 15-1a2-0** (AC8 (c), grep de la **valeur** `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, sites triés) | 15-1a2-0 AC8 |
| L-1 = F-7 — `DocumentRef` jamais défini ; précédence des issues | LOW | **corrigé** : défini en P3 (`number: Option<String>`, clé `documentNumber` présente et nulle), précédence `AccountNotLetterable` (C-15-1a2-22) | P3, T1 |
| L-2 — motif codé en dur au dé-rapprochement | LOW | **corrigé, porté** (motif lié) | 15-1a2-0 D3 |
| L-3 (≈ F-5 point 2) — étapes 2 et 5 sans test | LOW | **corrigé** : `sync_never_overwrites_a_foreign_mark`, `sync_step_5_recreates_or_abstains` | Tests |
| L-4 — `api-external.md:299` (audit des lettrages) | LOW | **corrigé** | AC12 |
| L-5 — la restauration hors de l'« inventaire fermé » | LOW | **corrigé** (exclusion écrite, avec sa raison) | modèle réel |
| L-6 ≈ F-6 — remède « ou » ; jusqu'où reculer la borne | LOW | **corrigé, porté** : « avant la date du dernier règlement », « les deux si les deux » (C-15-1a2-20) | 15-1a2-0 D2, AC7, AC8 |
| L-7 — six « domaines » | LOW | **corrigé** (deux grains, C-15-1a2-21) | Dev Notes |
| L-8 — « sept motifs » en dur, `REPLIS_A_SITE_UNIQUE` | LOW | **corrigé, porté** (par la valeur ; `blocker-messages.ts:9` trié, inchangé) | 15-1a2-0 AC7 |
| L-9 — signatures | LOW | **corrigé** : (a), (b) à la 15-1a2-0 D1 (`fiscal_year_id` ; `start_date` lue) ; (c) AC15 (d) ; (d) P3 étape 1 (`Invariant`, comme `invoice_settlements_write.rs:123-125`) | P3, AC15, 15-1a2-0 |
| F-3 — gardes `sitesTotal`, `REPLIS_A_SITE_UNIQUE` | LOW | **corrigé, porté** (vérifié : `i18n-keys.test.ts:526` = 1920, `i18n-repli-divergent-actif.test.ts:216`) | 15-1a2-0 AC7 (c) |
| F-4 point 3 — `admin-manual.tex:2101` | LOW | **corrigé, porté** | 15-1a2-0 AC8 |
| F-5 — idempotence, `Invariant`, borne stricte | LOW | **corrigé** : `sync_is_idempotent`, `sync_never_overwrites_a_foreign_mark` ici ; borne (jour de la borne clos, lendemain ouvert) à la 15-1a2-0 AC1 | Tests ; 15-1a2-0 |
| F-8 — citations | LOW | **en partie réfuté** : `sec:reglement-client` est bien à `:1141` (label sur la ligne de la sous-section, `grep -n` le confirme) ; `PeriodLocked` corrigé `:402` → `:405`, `find_open_covering_date` `:1744` → `:1745` ; `reconciliation.types.ts` porté à la 15-1a2-0 ; `lettering_error_to_failed_proposal` placée à côté de `claim_account_failed_proposal` (`:1322`) | P4, P7, AC15 |

**Découpage effectif** : sont partis à la 15-1a2-0 la règle des périodes (ancienne « Évaluation » de P7), P7
point 2 (réduit ici à un renvoi et à ce que la synchronisation y ajoute), T2-bis, AC17, AC14 (d), le message
`LETTERING_IS_DOCUMENT` et le doc-comment d'`InvoiceCredited` (anciens AC12 et T1), et les tests
correspondants. **Propagation** (valeurs grepées sur les fiches, l'index, le registre et le code cité) :
« 15-1a2-i P7 point 2 », « 15-1a2-i AC17 », `T2-bis`, `C-15-1a2-13`, `tests/common`, `:1744`, `:402`,
`"supplier_invoice"` — résidus restants : Change Logs (historique), prompts versionnés. **Recompte** (depuis ce
fichier) : **12 critères** actifs (AC1–AC5, AC8–AC10, AC12–AC15 ; l'en-tête d'AC17 marque le déplacement),
**7 tâches** (T0–T6), **27 tests neufs** (tous Rust) **+ 1 étendu**. Choix consignés : **C-15-1a2-19 à 22** ;
C-15-1a2-8, 10, 12, 13 (retirée), 17, 18 annotés. Prochaine passe : **P4, complète, Opus** — la remédiation
change le périmètre (découpage) et touche plusieurs sections ; la 15-1a2-0 commence à sa **P1**.

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

- 2026-10-09 — **Validation P5 ciblée** (Sonnet, prompt `56394f14` ; rapport `/home/gcorbaz/devel/kesh-gate-logs/15-1a2-i-validate-p5-ciblee.md`) :
  1 MEDIUM, 5 LOW ; AC18 exact sur lignes, rangs et codes ; fixture constructible ; étape 3 terminale ; recomptes
  justes. **P5-1** (MEDIUM) : le texte prescrit pour le § du verrou (AC18 d) et `admin-manual.tex:2101` n'énonçait
  pas la seconde cause du refus (période suivie d'un exercice clôturé) → ajoutée, avec l'ordre de réouverture ; les
  deux listes de motifs (a) renvoient au texte de la famille D2 de la 15-1a2-0, qui nomme les deux causes (P5-2).
  P5-3 corrigé (étapes 5 et 6). LOW laissés au T0 : P5-4 (l'étape 2 peut rendre `Invariant` avant l'étape 3 — « quel
  que soit `E` » s'entend après l'étape 2), P5-5 (précédence sur `AbstainedClosedPeriods` d'un survivant entièrement
  sous la borne : à monter au test), P5-6 (référence historique dans la 15-1a2-ii, ligne de Change Log). Remédiation
  faite par l'orchestrateur, fiche seule. **Validation close** (passe ciblée de fin de boucle, aucun correctif de
  production). Trend : P1 (15-1a2) 3 HIGH / 7 MEDIUM → P2 4 MEDIUM → P3 4 MEDIUM (découpage 15-1a2-0) → P4 2+2 MEDIUM
  recyclés (D5 déclaré, non découpé) → P5 ciblée 1 MEDIUM de texte, corrigé.
