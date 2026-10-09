# Story 15.1a2-0 : Le lettrage se fige avec la période — les annulations qui le dissoudraient sont refusées

## Status

ready-for-dev *(créée le 2026-10-09 à la remédiation de la validation P3 de la 15-1a2-i — finding F-1,
décision de l'orchestrateur, C-15-1a2-19 ; **validation P1 à mener avant tout développement**)*.

Première des trois sous-fiches de la **15-1a2** (index : `15-1a2-lettrage-des-pieces.md`). Base `056997b0`
(15-1a-i #587 et 15-1a-ii #593 mergées). **Ordre** : 15-1a-i → 15-1a-ii → **15-1a2-0** → 15-1a2-i →
15-1a2-ii → 15-1b-0 → 15-1b → 15-1c. ⛔ Pas de tag entre les merges (C124).

## Story

**As a** indépendant, PME ou fiduciaire qui a verrouillé une période ou clôturé un exercice,
**I want** que l'annulation d'un règlement, d'un solde, d'un rapprochement, d'un paiement fournisseur ou
d'une facture fournisseur payée soit **refusée** quand elle devrait défaire un lettrage de pièce entièrement
figé par la période — avec la raison, et ce qu'un administrateur doit faire pour la permettre,
**so that** ce qui est clos le reste : la liste des postes ouverts « au 31.03 » d'un trimestre verrouillé
ne bouge plus, et l'écran dit qui peut débloquer, et comment.

## Pourquoi cette story existe, et pourquoi AVANT la 15-1a2-i

La validation P2 de la 15-1a2-i a remplacé l'abstention au délettrage par un **refus** (C-15-1a2-10) :
le rang **2 bis** de la file commune des annulations. Le refus a fait passer la fiche de cinq à six
domaines, et une dérogation à la règle de splitting l'a gardé dans la 15-1a2-i (C-15-1a2-13). La
validation P3 (lentille F, F-1) a montré que la dérogation reposait sur un argument incomplet : elle
n'examinait qu'**un** ordre de merge — « une sous-story *refus* mergée **après** la synchronisation
laisserait entre les deux merges l'état défaillant » — et pas l'ordre inverse, **le refus d'abord**.

Or le refus d'abord ne laisse aucun état défaillant :

- **Il est dormant.** Tant que la 15-1a2-i n'a pas livré la synchronisation, aucun groupe d'origine
  `document` n'existe en base (la 15-1a-i a posé la marque, ses deux primitives ne créent l'origine
  `document` que pour un appelant qui la demande, et aucun appelant de production ne la demande sur
  `056997b0` : `grep -rn "Origin::Document" crates --include=*.rs` hors tests ne rend que
  `letterings.rs:812`, le refus du délettrage manuel, et `:906`, dans son `mod tests`). Le rang 2 bis ne s'applique donc à aucune donnée réelle : aucun geste existant n'est
  refusé.
- **Il se teste en isolation** : un groupe `document` **posé à la main** en SQL brut — la même voie que
  les tests de la 15-12b et que l'AC14 (c) de la 15-1a2-i — exerce toute la chaîne : prédicteur, gestes,
  textes, écran.
- **Il précède ce qu'il garde.** Quand la 15-1a2-i livrera la dissolution, le refus sera déjà en place :
  aucun intervalle entre deux merges ne voit une dissolution rencontrer un groupe clos.

Décision de l'orchestrateur (C-15-1a2-19) : **sortir le refus** dans cette story préalable. La dérogation
C-15-1a2-13 est **retirée** (registre annoté).

## Le modèle réel — relevé sur `056997b0`

*(Lignes citées sur `056997b0` ; elles bougeront — **citer et re-greper par le nom de fonction**.)*

**La file commune des annulations** : `settlement_cancellation::settlement_entry_cancel_blocker(conn,
company_id, entry_id, unlinking)` (`settlement_cancellation.rs:74`) évalue, sur l'écriture **examinée**
(règlement, rapprochement, achat), les rangs 2 à 5 de `SettlementCancelBlocker`
(`kesh-db/src/errors.rs:343`) : rang 2 `FiscalYearClosed` (l'exercice de l'écriture examinée,
`fy.status == "Closed"`, `:85-100`), rang 3 `MatchedBankTransaction`, puis les rangs de compte et
d'exercice du jour. ⚠️ **Aucun rang ne compare la date** de l'écriture examinée à
`companies.books_locked_through` (`grep -n "books_locked" settlement_cancellation.rs` : aucune sortie) :
la contre-passation, datée du jour, franchit le verrou.

**Ses lecteurs** — deux par geste, une évaluation :

| geste | prédicteur (l'écran) | refus du geste (filtre des rangs) |
|---|---|---|
| annulation d'un règlement ou d'un solde client | `cancelBlockedBy` de `GET /invoices/{id}/settlements` | `invoice_settlements_write::cancel_settlement_in_tx` étape (3) : `blocker @ (InvoiceCredited \| WriteOffExists \| FiscalYearClosed)` (`:841-847`) |
| dé-rapprochement | `cancelBlockedBy` de `GET /reconciliation/transactions/{id}` | `reconciliation_cancel::cancel_in_tx` étape (4) : `if let Some((FiscalYearClosed, _, _))`, **motif codé en dur** dans l'erreur (`:322-327`), **avant** de défaire le lien (étape 5, `:330`) |
| annulation d'un paiement fournisseur | `settlementCancelBlockedBy` (`supplier_settlement_cancel_blocker`, `supplier_invoices.rs:1176`) | `supplier_invoices::cancel_settlement_in_tx` : `blocker @ (SupplierInvoiceNotPaid \| FiscalYearClosed)` (`:1270-1271`) |
| annulation d'une facture fournisseur | `cancelBlockedBy` (`supplier_invoice_cancel_blocker`, `:966`, sur l'écriture d'**achat**) | `supplier_invoices::cancel_in_tx` : `blocker @ (SupplierInvoiceCancelled \| FiscalYearClosed \| …)` (`:1065-1067`) |

Les prédicteurs appellent la file commune et rendent `h.0.code()` sans `match` exhaustif
(`routes/supplier_invoices.rs:180-184`) : ils **héritent** de toute variante neuve. Les quatre filtres de
refus sont des **listes fermées** : un rang neuf n'y entre que si on l'y ajoute.

**Les textes du refus** — trois familles, trois tables `match` exhaustives sur `SettlementCancelBlocker`
dans `kesh-api/src/errors.rs` (`SettlementNotCancellable` `:3262`, `reconciliation_cancel_blocked_text`
`:3764`, `supplier_invoice_cancel_blocked_text` `:3815`), trois listes TypeScript
(`SettlementCancelTailCode` / `settlementCancelTailMessage` de `lib/shared/utils/settlement-cancel-blocked.ts`,
terminée par `const exhaustive: never = code` `:68-71` ; `features/reconciliation/reconciliation-cancel.ts` et
son type `reconciliation.types.ts:126-132` ; `features/supplier-invoices/invoice-cancel.ts`). Statut : les
trois erreurs rendent `409 CONFLICT` avec `blocker.code()` (`errors.rs:3255-3330`).

**La règle des périodes du socle** : `letterings::any_line_in_open_period` (`letterings.rs:494`, privée, mode
`Manual`, état des exercices lu sous verrou) — exercice `Open`, aucun exercice postérieur `Closed`
(`fiscal_years::find_later_closed(conn, company_id, start_date)`, `fiscal_years.rs:890`, qui prend la
**date de début** de l'exercice), date **strictement** postérieure à `books_locked_through` — et son code
`LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (15-1a-i R7, C113).

## Décisions

### D1 — La règle des périodes, une seule factorisation, publique (C-15-1a2-18)

Dans `crates/kesh-db/src/repositories/letterings.rs` :

```rust
/// L'état des exercices et la borne, lus SANS verrou — la règle des périodes hors du mode `Manual`.
pub struct OpenPeriodRule { /* privés : par exercice (open, later_closed), et locked_through */ }

/// Lit le statut des exercices NOMMÉS, leur date de début (pour `find_later_closed`), et la borne.
pub async fn open_period_rule(conn: &mut MySqlConnection, company_id: i64, fiscal_year_ids: &[i64])
    -> Result<OpenPeriodRule, DbError>;

impl OpenPeriodRule {
    /// La ligne `(fiscal_year_id, entry_date)` est-elle « en période ouverte » ?
    /// Exercice inconnu de la règle (non nommé à `open_period_rule`) → `false`.
    pub fn line_in_open_period(&self, fiscal_year_id: i64, entry_date: NaiveDate) -> bool;
}

/// `open_period_rule` sur les exercices des lignes, puis « au moins une ligne en période ouverte ».
/// `lines` : couples `(fiscal_year_id, entry_date)` — l'identifiant d'**exercice**, pas d'écriture.
pub async fn lines_in_open_period(conn: &mut MySqlConnection, company_id: i64,
    lines: &[(i64, NaiveDate)]) -> Result<bool, DbError>;
```

`open_period_rule` lit, pour chaque exercice nommé, `status` **et** `start_date` (l'argument de
`find_later_closed`, `fiscal_years.rs:890` — validation P3 de la 15-1a2-i, L-9 b), appelle
`find_later_closed` pour chaque exercice ouvert, et lit la borne. `line_in_open_period` et
`any_line_in_open_period` (mode `Manual`, état lu sous verrou) appellent **le même** prédicat par ligne,
privé — `open && !later_closed && locked_through.map_or(true, |b| entry_date > b)` — factorisé, jamais
recopié. **Borne stricte** : le jour de la borne est clos, le lendemain ouvert (AC1, finding F-5 de la P3).
Types publics : la 15-1a2-i (synchronisation) et la 15-1b (`inOpenPeriod`, filtre des propositions)
l'emploient telle quelle.

### D2 — Le rang 2 bis : `DocumentLetteringInClosedPeriods`

- **Variante** `SettlementCancelBlocker::DocumentLetteringInClosedPeriods` (`kesh-db/src/errors.rs`), sans
  champ, placée **entre** `FiscalYearClosed` (rang 2) et `MatchedBankTransaction` (rang 3) — d'où « 2 bis »
  (C-15-1a2-12). Après le rang 2 : un exercice clos se nomme par son propre motif, de même remède. Avant le
  rang 3 : annoncer « annulez d'abord le rapprochement » serait vain, le dé-rapprochement étant refusé par
  ce même rang.
- **Évaluation**, dans `settlement_entry_cancel_blocker`, entre les rangs 2 et 3, sur l'écriture
  **examinée** : « une ligne de cette écriture appartient à un groupe d'origine `document` dont aucune ligne
  n'est en période ouverte », par une fonction publique de `letterings.rs` :

  ```rust
  /// La clé du premier (plus petite clé) groupe d'origine `document` qui contient une ligne de
  /// l'écriture `entry_id` et dont AUCUNE ligne n'est en période ouverte (D1) ; `None` sinon.
  /// Lecture sans verrou des lignes du groupe, des exercices et de la borne.
  pub async fn document_group_frozen_by_periods(conn: &mut MySqlConnection, company_id: i64,
      entry_id: i64) -> Result<Option<i64>, DbError>;
  ```

  Doc-comment du module `settlement_cancellation` : « Les rangs 2 à 5 » → « 2 à 5, dont 2 bis ».
- **Code** : `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, réemployé (C-15-1a2-11) — `SettlementCancelBlocker::code`
  pose que « tous ces codes réemploient ceux d'états du monde déjà nommés » (`errors.rs:401`) ; « toutes
  les lignes du groupe sont en période close » est l'état que ce code nomme déjà. Statut **409**.
- **Remède écrit, précis** (findings F-6 et L-6 de la P3 de la 15-1a2-i ; C-15-1a2-20). Le groupe redevient
  dissoluble dès qu'**une** de ses lignes est en période ouverte. Deux causes, qui peuvent se cumuler :
  1. **la borne du verrou** : un administrateur la fait reculer **avant la date de la ligne la plus récente du
     lettrage** — en pratique, avant la date du dernier règlement (ou paiement) de la pièce —, par
     `POST /companies/current/books-lock/release` (`companies::unlock_books`, motif obligatoire ; nouvelle
     borne antérieure, ou aucune). Un recul qui laisse la borne **à ou après** cette date ne lève rien ;
  2. **la clôture** : l'exercice des lignes est clôturé, ou suivi d'un exercice clôturé — un administrateur
     **rouvre** les exercices clôturés, du plus récent au plus ancien (`fiscal_years::reopen`), jusqu'à
     celui de la ligne la plus récente.
  Quand les deux s'appliquent, il faut **les deux** : le texte dit « selon le cas … ; si les deux
  s'appliquent, les deux », jamais « ou » seul. Le refus ne porte **aucune date** dans `details` (C-15-1a2-20 :
  variante sans champ, comme le rang 2 ; la date de la ligne la plus récente se lit sur la pièce — les dates
  de ses règlements — et le texte dit laquelle chercher).
- **Textes** : une clé par famille, quatre locales — `settlement-cancel-blocked-lettering-closed`
  (règlement et solde, client **et** fournisseur : la clé de la queue est partagée),
  `reconciliation-cancel-blocked-lettering-closed`, `supplier-invoices-cancel-blocked-lettering-closed` — et
  leurs replis Rust (`kesh-api/src/errors.rs`, un bras dans chacune des trois tables), **mot pour mot** le
  FTL fr-CH. Proposition fr-CH pour la queue : « Le lettrage de cette facture est figé par une période
  close. Un administrateur doit, selon le cas, faire reculer le verrou de période avant la date du dernier
  règlement, ou rouvrir les exercices clôturés en commençant par le plus récent — si les deux
  s'appliquent, les deux. »

### D3 — Les quatre gestes refusent le rang 2 bis, chacun dans sa famille

| geste | changement | erreur |
|---|---|---|
| `invoice_settlements_write::cancel_settlement_in_tx` étape (3) | le motif lié `blocker @ (InvoiceCredited \| WriteOffExists \| FiscalYearClosed \| DocumentLetteringInClosedPeriods)` | `SettlementNotCancellable { blocker }` |
| `reconciliation_cancel::cancel_in_tx` étape (4) | le motif **lié** `Some((blocker @ (FiscalYearClosed \| DocumentLetteringInClosedPeriods), _, _))`, et `blocker` — non plus `FiscalYearClosed` en dur — dans l'erreur (patron de `invoice_settlements_write.rs:841-847` ; finding L-2 de la P3) ; toujours **avant** de défaire le lien | `ReconciliationNotCancellable { blocker }` |
| `supplier_invoices::cancel_settlement_in_tx` | `blocker @ (SupplierInvoiceNotPaid \| FiscalYearClosed \| DocumentLetteringInClosedPeriods)` | `SettlementNotCancellable { blocker }` (clé partagée de la queue) |
| `supplier_invoices::cancel_in_tx` | `blocker @ (SupplierInvoiceCancelled \| FiscalYearClosed \| DocumentLetteringInClosedPeriods \| …)` | `SupplierInvoiceNotCancellable { blocker }` |

Sans le rang dans le filtre du dé-rapprochement, le refus remonterait de `cancel_settlement_in_tx` en
`SettlementNotCancellable` (« ce règlement » au lieu de « ce rapprochement »), après que le lien a été
défait — annulé par le rollback de l'appelant, mais sous le mauvais texte. Le dé-rapprochement d'une
écriture **propre** (`ReconciliationKind::Entry`) ne rencontre jamais ce rang : une écriture qui
n'appartient à aucune pièce ne porte pas de lettrage `document`.

⛔ **Les gestes fournisseurs sont ici**, et non à la 15-1a2-ii (C-15-1a2-19) : la file étant commune, les
prédicteurs fournisseurs héritent du rang dès cette story ; laisser les deux gestes sans le refuser
ferait annoncer un motif que le clic ne refuse pas (bouton masqué, API acceptant). Une seule story pose
le refus partout.

### D4 — Dormant, et ce que voit l'utilisateur

**Avant la 15-1a2-i** : aucun groupe `document` n'existe ; aucun geste n'est refusé ; rien ne change à
l'écran. Les tests posent le groupe à la main (`UPDATE journal_entry_lines SET lettering_key = …,
lettering_origin = 'document'` sur les lignes d'une facture soldée et de ses règlements, ou d'un achat et
de son paiement — somme nulle, même compte, clé = plus petite ligne : la forme que produira la
synchronisation).

**Après la 15-1a2-i** : au 31.03 verrouillé, la fiche facture n'offre pas l'annulation d'un règlement dont
le lettrage est figé, et dit pourquoi et qui peut la débloquer ; la vue des postes ouverts « au 31.03 » ne
bouge plus. ⚠️ **Tolérance résiduelle, nommée** : la règle se lit sans verrou sur la borne et sur les
exercices autres que celui de l'écriture examinée (tenu par le geste). Un `lock_books` (ou la clôture d'un
autre exercice du groupe) validé entre la lecture du rang 2 bis et le `COMMIT` laisse passer l'annulation
— la tolérance qu'a déjà une écriture créée pendant la pose du verrou. Aucun verrou neuf.

### D5 — Le message `LETTERING_IS_DOCUMENT` neutre, et un doc-comment (C-15-1a2-8, C-15-1a2-7)

*(Déplacés de la 15-1a2-i pour que les textes et `kesh-db/src/errors.rs` ne soient touchés que par une
story ; C-15-1a2-19.)*

- Le message de `LETTERING_IS_DOCUMENT` dit aujourd'hui « annulez le règlement plutôt », faux pour un groupe
  facture + avoir (aucun règlement à annuler, aucun avoir annulable). Réécrit **neutre** dans les quatre
  locales (`crates/kesh-i18n/locales/*/messages.ftl:54`) et son repli Rust (`kesh-api/src/errors.rs`, branche
  `DbError::LetteringIsDocument`, `:3057-3058`) : « Ce lettrage est celui d'une pièce : il suit ses
  règlements et son avoir, il ne se défait pas à la main. » (de/it/en traduits). Vrai avant la 15-1a2-i
  (aucun groupe `document` : la route ne le rend pas) comme après.
- Doc-comment de `SettlementCancelBlocker::InvoiceCredited` (`kesh-db/src/errors.rs` ≈ `:350`, « son
  traitement est la 15-1a2 ») réécrit selon C-15-1a2-7 : le règlement reste ouvert au compte débiteurs, la
  15-1a2 l'a tranché.

## Critères d'acceptation

**AC1** — **La règle des périodes** (D1) : `open_period_rule` / `line_in_open_period` / `lines_in_open_period`
existent avec les signatures de D1 ; sur un exercice ouvert sans successeur clos, une ligne **datée du jour
de la borne** n'est pas en période ouverte, une ligne **du lendemain** l'est ; une ligne d'un exercice
clôturé, ou d'un exercice suivi d'un exercice clôturé, ne l'est pas ; sans borne, toute ligne d'un exercice
ouvert sans successeur clos l'est. Les tests existants du mode `Manual` (`letterings.rs`, R7) restent verts
**sans modification de leurs assertions** (le prédicat par ligne est partagé).

**AC2** — **Le rang 2 bis dans la file commune** (D2), sur un groupe `document` posé à la main (D4) :
(a) facture et règlement complet datés **sous** `books_locked_through`, exercice ouvert →
`settlement_entry_cancel_blocker` sur l'écriture de règlement rend `DocumentLetteringInClosedPeriods` ;
(b) même montage **sans** groupe → pas ce rang ; groupe dont **une** ligne est après la borne → pas ce rang ;
(c) **précédence** : sur une écriture d'exercice clôturé, le motif est `FISCAL_YEAR_CLOSED` ; sur une
écriture à la fois lettrée dans un groupe figé **et** rapprochée d'une transaction, le motif est le 2 bis,
pas `MATCHED_BANK_TRANSACTION`.

**AC3** — **Règlement et solde clients** : sur le montage d'AC2 (a), `POST
/invoices/{id}/settlements/{settlementId}/cancel` d'un règlement, puis d'un **solde** (`write_off`) →
`409 LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, texte de la clé `settlement-cancel-blocked-lettering-closed` ;
**rien** n'est écrit (aucune écriture inverse, la ligne `invoice_settlements` reste, les marques sont
intactes, aucune entrée d'audit) ; la vue de la facture porte `cancelBlockedBy =
"LETTERING_ALL_LINES_IN_CLOSED_PERIODS"` sur ce règlement. Puis un administrateur **déverrouille**
(`POST /companies/current/books-lock/release`, motif, borne **avant** la date du règlement) → le
prédicteur ne rend plus ce rang (`cancellable = true`). *(L'annulation elle-même après déverrouillage —
dissolution, paires `reversal` — est l'AC14 (b) de la 15-1a2-i : sans la synchronisation, rien ne dissout
le groupe posé à la main.)* Un déverrouillage dont la nouvelle borne reste **à ou après** la date du
règlement ne lève rien (D2, remède précis).

**AC4** — **Dé-rapprochement** : facture encaissée par rapprochement, groupe posé à la main sous la borne
→ `POST /reconciliation/transactions/{id}/cancel` rend `409 LETTERING_ALL_LINES_IN_CLOSED_PERIODS` dans **sa**
famille (`ReconciliationNotCancellable`, clé `reconciliation-cancel-blocked-lettering-closed`) — le code
**et** la clé assertés, ce qui attrape un motif codé en dur (L-2) — ; le lien `matched_entry_id` est
**intact** ; `GET /reconciliation/transactions/{id}` porte le motif.

**AC5** — **Fournisseurs** : achat et paiement datés sous la borne, groupe posé à la main sur la ligne
d'achat et celle du paiement (compte fournisseurs) → l'annulation du **paiement** est refusée (`409`, clé
`settlement-cancel-blocked-lettering-closed`), l'annulation de la **facture** aussi (clé
`supplier-invoices-cancel-blocked-lettering-closed`) ; rien n'est écrit, la facture reste `paid` ; les deux
prédicteurs (`settlementCancelBlockedBy`, `cancelBlockedBy`) le disent.

**AC6** — **État hérité « exercice du groupe ouvert, suivi d'un exercice clos »** (fabriqué en SQL brut,
comme les tests de la 15-12b) → même refus qu'en AC3, le texte prescrivant de rouvrir les exercices
postérieurs.

**AC7** — **Les textes et l'écran** : (a) les trois clés existent dans les **quatre** locales ; chaque repli
Rust (trois bras de `kesh-api/src/errors.rs`) et chaque repli Svelte dit **mot pour mot** le FTL fr-CH ;
chaque texte nomme le remède **précis** de D2 (borne avant la date du dernier règlement ; réouverture du plus
récent au plus ancien ; les deux si les deux) ; (b) l'écran masque l'annulation et affiche le texte du motif
— fiche facture (`settlement-cancel-blocked.ts`), dialogue de dé-rapprochement (`reconciliation-cancel.ts`),
fiche fournisseur (`invoice-cancel.ts`, et la queue partagée pour le paiement) ; (c) les gardes frontend à
valeur épinglée sont **recomptées**, pas contournées (finding F-3 de la P3) : `sitesTotal` de
`lib/shared/i18n-keys.test.ts` (`:526`, `1920` sur `056997b0`) par la procédure écrite dans ce fichier
(`grep -o "i18nMsg("` aux deux bornes), et la liste `REPLIS_A_SITE_UNIQUE` de
`lib/shared/i18n-repli-divergent-actif.test.ts` (`:216`) gagne les **trois** clés neuves — **impératif** :
c'est ce test seul qui fait tenir « chaque repli Svelte dit mot pour mot le FTL » pour une clé à site
unique ; son commentaire « Chacune des dix clés » (`:209`) est recompté ; (d) le décompte « sept motifs » écrit en dur
devient « huit » partout où il compte les motifs du dé-rapprochement (finding L-8) — `reconciliation.types.ts:126`,
`reconciliation-cancel.ts:7` et `:28`, `reconciliation-cancel.test.ts:43` — par la **valeur** (`git grep -nE
"sept motifs|les sept|Les sept" frontend/src` trié : `blocker-messages.ts:9`, qui compte les motifs de pièce de la
contre-passation, **ne change pas**) ; `lint-i18n-ownership` vert.

**AC8** — **Documentation, par la valeur** :
- `docs/api-external.md` — (a) tableaux de refus de `POST /invoices/{id}/settlements/{settlementId}/cancel`
  (ligne `FISCAL_YEAR_CLOSED` à `:379`) et de `POST /reconciliation/transactions/{id}/cancel` (`:462`) : une ligne « Règlement
  (rapprochement) lettré avec sa facture dans une période close — un administrateur fait reculer le verrou
  avant la date du dernier règlement, ou rouvre les exercices clôturés ; les deux si les deux » |
  `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` | `409`, placée **après** `FISCAL_YEAR_CLOSED` et **avant**
  `MATCHED_BANK_TRANSACTION` ; (b) les deux listes **en prose** des refus fournisseurs — « Refus de
  l'annulation : … » (annulation du paiement, `:426`) et « Refus, dans l'ordre de précédence : … »
  (annulation de la facture, `:434`) — reçoivent `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` (`409`) **après**
  `FISCAL_YEAR_CLOSED` et **avant** `ACCOUNT_ARCHIVED` (findings R3-6 = F3-5 de la P3 de la 15-1a2-ii : ce sont
  des phrases, non des tableaux, et `MATCHED_BANK_TRANSACTION` n'y figure pas — ancrer par le texte, non par
  la ligne ; l'absence du rang 3 dans la liste du paiement fournisseur est un écart **préexistant**, hors de
  cette story, signalé à l'orchestrateur) ; (c) **la table de référence des codes** (§ 10 « Gestion des
  erreurs », ≈ `:566`), qui range aujourd'hui `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` sous les seules routes
  `/letterings` (findings M-2 = F-4 de la P3) : la cause s'étend — « et refus de l'annulation d'un règlement,
  d'un solde, d'un rapprochement, d'un paiement ou d'une facture fournisseur dont le lettrage est figé par une
  période close (§ des annulations) » ; le texte rendu diffère selon la route (celui du lettrage, celui de la
  famille d'annulation) — à écrire une fois dans cette ligne.
- `docs/manual/fr/user-manual.tex` — (a) les **deux listes exhaustives** de motifs (finding M-1 de la P3) :
  `:1198-1215` (§ `sec:reglement-client`, « le bouton est remplacé par la raison ») — un item « **le lettrage de
  la facture est figé par une période close** » **entre** « l'exercice du règlement est clôturé » et « le
  règlement est rapproché », avec le remède précis ; `:1770-1783` (§ `sec:annuler-rapprochement`) — même item
  **après** « l'exercice de l'écriture du rapprochement est clôturé » ; (b) les deux phrases-listes
  fournisseurs (`:1455-1459`, annulation du paiement ; `:1478-1484`, annulation de la facture) — le même
  motif, après l'exercice clôturé ; (c) la liste des exceptions de la contre-passation (`:2337-2342`, « ne
  s'annulent pas ») — le motif nommé, renvoi au § du verrou ; (d) § du verrou de période (`sec:verrou-periode`,
  `:562`, après `:577-579`) — une phrase : « l'annulation d'un règlement, d'un rapprochement ou d'un paiement
  fournisseur dont le lettrage s'est figé avec la période est refusée ; un administrateur fait reculer le
  verrou avant la date du dernier règlement pour la permettre ».
- `docs/manual/fr/admin-manual.tex:2101` (le verrou de période, OLICo Art. 9) — une phrase : le déverrouillage
  est aussi le remède du refus d'annuler un règlement dont le lettrage est figé, et la borne doit passer avant
  la date du dernier règlement (finding F-4 point 3 de la P3).
- `CHANGELOG.md` (`[0.13.0]`) : « l'annulation d'un règlement, d'un solde, d'un rapprochement, d'un paiement
  ou d'une facture fournisseur dont le lettrage est figé par une période close est refusée (`409
  LETTERING_ALL_LINES_IN_CLOSED_PERIODS`) ; un administrateur fait reculer le verrou ou rouvre l'exercice ».
- **PDF** : `make fr` dans `docs/manual/`, les trois PDF commités ; contrôle **aplati** (`pdftotext … | tr '\n' '
  ' | tr -s ' '`) : l'item figure dans **chacune** des quatre listes de motifs du manuel utilisateur, et la
  phrase dans le manuel d'administration.
- **Propagation par la valeur** (CLAUDE.md « Greper la VALEUR ») : `git grep -nF
  "LETTERING_ALL_LINES_IN_CLOSED_PERIODS" -- docs crates/kesh-api/src crates/kesh-db/src frontend/src
  CHANGELOG.md` — chaque site trié au Change Log (sur `056997b0` : `api-external.md:314`, `:326`, `:566`, et
  les deux sites de `kesh-api/src/errors.rs`).

**AC9** — **Message `LETTERING_IS_DOCUMENT` et doc-comment** (D5) : les quatre `.ftl` et le repli Rust disent
le texte neutre ; `git grep -nE "annulez le règlement plutôt|Stornieren Sie die Zahlung, statt|annullate il
pagamento invece|cancel the settlement rather than" -- crates docs` ne rend plus rien ; le test
`errors.rs:4283`, qui n'asserte que la clé (`texte(cle)`), reste vert sans modification.

## Tasks

- [ ] **T0** — Relevés au sol sur la base réelle du développement : re-greper les ancres par le nom
      (`settlement_entry_cancel_blocker`, les quatre filtres de D3, les trois tables de textes) ; vérifier
      que les routes des quatre gestes restent `Rejouee` (`audit_route_registry.rs`) ; relever `sitesTotal`
      et la liste `REPLIS_A_SITE_UNIQUE` aux deux bornes.
- [ ] **T1** (D1, AC1) — `OpenPeriodRule`, `open_period_rule`, `line_in_open_period`, `lines_in_open_period`,
      prédicat par ligne factorisé avec `any_line_in_open_period` ; doc-comments (lecture sans verrou,
      tolérance, borne stricte).
- [ ] **T2** (D2, AC2, AC6) — Variante `DocumentLetteringInClosedPeriods` (doc-comment : place, code
      réemployé, remède précis), son `code()` ; `document_group_frozen_by_periods` ; évaluation entre les rangs
      2 et 3 de `settlement_entry_cancel_blocker` ; doc-comment du module.
- [ ] **T3** (D3, AC3–AC5) — Les quatre filtres de refus, motifs **liés** (`blocker @`), dé-rapprochement
      avant le lien défait.
- [ ] **T4** (D2, AC7) — `kesh-api/src/errors.rs` : un bras dans chacune des trois tables, replis mot pour mot ;
      trois clés × quatre locales ; frontend : `SettlementCancelTailCode` et `settlementCancelTailMessage`,
      liste et texte de `reconciliation-cancel.ts` et son type, cas de `invoice-cancel.ts` ; « sept » → « huit »
      (AC7 d) ; gardes `sitesTotal` et `REPLIS_A_SITE_UNIQUE` recomptées ; `lint-i18n-ownership` vert.
- [ ] **T5** (D5, AC9) — Message `LETTERING_IS_DOCUMENT` (quatre `.ftl` + repli Rust) ; doc-comment
      d'`InvoiceCredited`.
- [ ] **T6** — Tests (liste ci-dessous).
- [ ] **T7** (AC8) — `api-external.md` (deux tableaux, deux listes en prose, table des codes), manuel
      utilisateur (quatre listes, verrou), manuel d'administration, CHANGELOG, `make fr`, PDF aplati, grep de la
      valeur.

**Tests prévus** (12 neufs, 4 modifiés) :
- `crates/kesh-db/tests/lettering_closed_period_refusal.rs` (neuf, `test-schema`, groupes posés en SQL brut) — 7 :
  `open_period_rule_reads_the_bound_strictly` (AC1), `rank_2_bis_sees_a_frozen_document_group` (AC2 a, b),
  `rank_2_bis_precedence` (AC2 c), `client_settlement_and_write_off_cancels_are_refused` (AC3, dépôt : rien
  d'écrit, prédicteur levé après déverrouillage, déverrouillage insuffisant sans effet),
  `unreconcile_refuses_in_its_family_before_unlinking` (AC4, dépôt : variante **et** lien intact),
  `supplier_cancels_are_refused` (AC5), `later_closed_year_group_is_refused` (AC6) ;
- `crates/kesh-api/src/errors.rs` (`mod tests`) — 1 : `closed_lettering_texts_follow_their_family` (AC7 a :
  trois familles, trois clés, replis non vides, remède nommé) ;
- `crates/kesh-api/tests/invoice_echeancier_e2e.rs` — 1 : `settlement_cancel_blocked_by_closed_lettering`
  (AC3 par l'API : `cancelBlockedBy` de la vue, puis `POST …/cancel` → 409, même code) ;
- `crates/kesh-api/tests/reconciliation_e2e.rs` — 1 : `unreconcile_of_a_closed_lettering_is_refused_in_its_family`
  (AC4 par l'API : code, clé, rien d'écrit) ;
- `crates/kesh-api/tests/supplier_settlement_cancel_e2e.rs` — 1 : `cancel_blocked_by_closed_lettering` (AC5 par
  l'API : les deux prédicteurs, puis 409 et clé) ;
- `frontend/src/lib/features/supplier-invoices/invoice-cancel.test.ts` (neuf, Vitest) — 1 : le motif a son
  texte, mot pour mot le FTL (AC7 b) ;
- **modifiés** (Vitest) : `lib/shared/utils/settlement-cancel-blocked.test.ts` et
  `features/reconciliation/reconciliation-cancel.test.ts` (le code neuf a son texte, « huit »),
  `lib/shared/i18n-keys.test.ts` (`sitesTotal` recompté), `lib/shared/i18n-repli-divergent-actif.test.ts`
  (`REPLIS_A_SITE_UNIQUE` + trois clés).

*(Recompte depuis cette liste : 7 + 1 + 1 + 1 + 1 + 1 = **12 fonctions de test neuves** (11 Rust, 1 Vitest),
**4 fichiers de test modifiés** (Vitest).)*

## Dev Notes

- **Gate `kesh-db` complet** (repositories touchés : ciblage interdit, `CLAUDE.md`) ; base remise à zéro avant,
  **sans** redémarrer le conteneur partagé (consignes de l'Epic 15). **Gate frontend complet** (`npm run
  check`, `lint-i18n-ownership`, `test:unit`, `build`) et **E2E complet** au dernier commit de code (D7).
- **Aucune migration** : P5–P8 sans objet ; aucun bump.
- **Verrous** : aucun neuf — le rang 2 bis lit, sans verrou, les lignes du groupe, les exercices et la borne,
  après les verrous que chaque geste prend déjà (tolérance de D4).
- **Tests existants touchés** (relevés sur `056997b0`, non exécutés) : les tests de la file commune
  (`settlement_cancellation`, prédicteurs client et fournisseur, textes de `kesh-api/src/errors.rs`,
  `invoice_echeancier_e2e.rs`, `supplier_settlement_cancel_e2e.rs`, `reconciliation_e2e.rs`) — un `match`
  exhaustif gagne un bras, aucun rang existant ne change de place ; `filet_bilan_clos.rs::predicteur_muet_sous_un_exercice_futur_clos`
  (écriture manuelle, aucun groupe `document`) : inchangé ; `errors.rs:4283` (`LETTERING_IS_DOCUMENT`, clé
  seule) : inchangé.
- **Règle de découpage** — compté aux **deux** grains, comme la 15-1a-i et la 15-1a-ii (C-15-1a2-21) :
  - au grain « **crates Rust, packages npm** » : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` = **4** —
    seuil (« plus de 5 ») non franchi ;
  - au grain des **modules métier de premier niveau** (patron des exemples de la règle,
    `kesh-api/routes/invoices`) : `kesh-db/repositories/letterings` (règle, gel), `…/settlement_cancellation`
    (rang), `kesh-db/errors` (variante, doc-comment), `…/invoice_settlements_write`, `…/reconciliation_cancel`,
    `…/supplier_invoices` (un bras de filtre chacun), `kesh-api/errors` (trois bras de texte, un repli),
    `kesh-i18n`, `frontend/shared/utils`, `frontend/features/reconciliation`, `frontend/features/supplier-invoices`
    — **onze**, dont **sept mécaniques** (un bras d'un `match` ou d'un motif fermé, que le compilateur ou
    `never` énumèrent). La documentation n'est pas un module au sens de la règle (ni crate, ni package, ni
    module métier) : elle est déclarée à part.
  Le seuil est franchi au second grain ; **signal déclaré** à l'orchestrateur. Un découpage plus fin
  séparerait la variante de ses textes, ce que le compilateur interdit (trois `match` exhaustifs dans
  `kesh-api/src/errors.rs`) ; il resterait un découpage serveur / écran, possible (l'écran tolère un code
  inconnu tant qu'aucun groupe n'existe), non retenu ici sans arbitrage.

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

### Création — 2026-10-09 (Opus 5.5, remédiation de la validation P3 des fiches 15-1a2-i et 15-1a2-ii, en autonomie)

Story **extraite** de la 15-1a2-i (finding F-1 MEDIUM de sa validation P3, lentille F : la dérogation
C-15-1a2-13 n'examinait qu'un ordre de merge ; décision de l'orchestrateur, **C-15-1a2-19** ; dérogation
**retirée**). Y sont venus : la règle des périodes (ancienne P7 « Évaluation », C-15-1a2-18), le rang 2 bis
(ancienne P7 point 2, T2-bis, AC17, AC14 (d)), les refus des deux gestes **fournisseurs** (ancienne T2 de la
15-1a2-ii, C-15-1a2-19), le message `LETTERING_IS_DOCUMENT` et le doc-comment d'`InvoiceCredited` (anciens
AC12 et T1 de la 15-1a2-i). Y sont corrigés, au passage, les findings de la P3 qui portaient sur ces
éléments : **M-1** (listes exhaustives du manuel), **M-2 = F-4** (table des codes `:566`, grep par la
valeur), **F-3** (gardes `sitesTotal`, `REPLIS_A_SITE_UNIQUE`), **F-5** (borne stricte testée), **F-6** et
**L-6** (remède précis, « les deux si les deux » ; C-15-1a2-20), **L-2** (motif lié du dé-rapprochement),
**L-8** (« sept » → « huit », par la valeur), **L-9 a, b** (signatures de D1), **F-4 point 3** (manuel
d'administration), et, de la 15-1a2-ii, **R3-6 = F3-5** (listes en prose fournisseurs) et le LOW non numéroté
de F (`user-manual.tex:2337`). **9 critères** (AC1–AC9), **8 tâches** (T0–T7), **12 tests neufs** (11 Rust, 1
Vitest) **+ 4 fichiers modifiés** — recomptés depuis ce fichier. Choix consignés : **C-15-1a2-19, 20, 21**.
Prochaine passe : validation **P1**, complète (Sonnet, contexte frais).

## Dérogation règle de splitting

Au grain fin, la fiche dépasse cinq modules ; au grain des crates et paquets — celui que la règle a toujours appliqué dans cet epic —, elle est sous le seuil. Le dépassement ne vient que de la propagation mécanique de textes (catalogues ×4, manuels et PDF, `api-external.md`, CHANGELOG, libellés), qui ne porte aucune règle. Décision de l'orchestrateur : pas de découpage (registre **C-15-1a2-23**, alternatives et réversibilité). Accepted risk : une passe de revue doit relire la propagation des textes comme un axe à part entière.
