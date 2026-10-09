# Story 15.1a2-0 : Le lettrage se fige avec la période — les annulations qui le dissoudraient sont refusées

## Status

review *(développement livré le 2026-10-09, Opus 5.5 ; revue de code à mener)* — antérieurement ready-for-dev *(créée le 2026-10-09 à la remédiation de la validation P3 de la 15-1a2-i — finding F-1,
décision de l'orchestrateur, C-15-1a2-19 ; validation P1 remédiée le 2026-10-09 — la **documentation
publique** du refus portée à la 15-1a2-i (C-15-1a2-24), textes écrits dans les quatre locales et conformes
aux gardes G8, G8-bis et G9 (C-15-1a2-26), précédence éprouvée contre tous les rangs voisins (C-15-1a2-27) ;
**validation P2 remédiée le 2026-10-09 — VALIDATION CLOSE** : 0 MEDIUM restant, les sept LOW appliqués,
antécédent de « celui-ci » levé et constante sœur de G8 (C-15-1a2-30) ; prochaine étape : `bmad-dev-story`)*.

⛔ **Story DORMANTE : elle n'écrit que ce qu'exige le code** (C-15-1a2-24) — la règle des périodes, la
variante et son rang, la file, les quatre filtres, les prédicteurs, les textes i18n (quatre locales, conformes
aux gardes), l'écran, les doc-comments internes. **Tout ce qui dit le refus à l'utilisateur ou à
l'intégrateur** — manuel utilisateur (listes de motifs, § du verrou de période, son encadré et sa note sur la
contre-passation, exceptions de la contre-passation), manuel d'administration, `api-external.md`,
`CHANGELOG.md` — est l'**AC18 de la 15-1a2-i**, qui livre les premiers groupes `document` et rend le refus
atteignable. Écrire ici « le lettrage de la facture est figé » dans un manuel dont le glossaire dit encore
« Kesh ne lettre pas encore de lui-même une facture soldée » (`user-manual.tex:2424-2425`) le ferait se contredire entre les deux merges (finding F-2 de la P1).

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
  textes, écran. La **documentation**, elle, attend le comportement qu'elle décrit : 15-1a2-i, AC18.
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
`:3763`, `supplier_invoice_cancel_blocked_text` `:3814`), trois listes TypeScript
(`SettlementCancelTailCode` / `settlementCancelTailMessage` de `lib/shared/utils/settlement-cancel-blocked.ts`,
terminée par `const exhaustive: never = code` `:59` ; `features/reconciliation/reconciliation-cancel.ts` et
son type `reconciliation.types.ts:126-132` ; `features/supplier-invoices/invoice-cancel.ts`). Statut : les
trois erreurs rendent `409 CONFLICT` avec `blocker.code()` (`errors.rs:3255-3330`).

**Les gardes que ces textes rencontrent** (validation P1, R-1, R-2 — la version P1 de cette fiche n'en nommait
aucune) :
- **G8** (`kesh-i18n/src/loader.rs`, `les_prescriptions_de_reouverture_disent_l_ordre`, ≈ `:1042-1103`) : toute
  clé dont la valeur fr-CH contient `[Rr]ouvr|[Rr]éouv` (hors dix exemptions fermées) porte, **dans les quatre
  locales**, le marqueur d'ordre — « en commençant par le plus récent », « beginnend mit dem neuesten »,
  « cominciando dal più recente », « starting with the most recent » ; sa liste nominative `CLES_569` (six
  clés) asserte que des clés nommées restent au domaine (anti-test-muet) ;
- **G8-bis** (même fichier, `les_prescriptions_de_reouverture_sont_bornees`, ≈ `:1106-1190`) : la même clé porte,
  dans chaque locale, une **borne** — « les exercices postérieurs clôturés » / « die späteren abgeschlossenen
  Geschäftsjahre » / « gli esercizi successivi chiusi » / « the later closed fiscal years », **ou** « jusqu'à
  celui-ci » / « bis zu diesem Geschäftsjahr » / « fino a questo esercizio » / « down to this one » ;
- **G9** (`kesh-api/tests/textes_coherents.rs`, `les_replis_rust_suivent_le_catalogue`, `TABLE` ≈ `:667-690`) :
  le seul mécanisme du dépôt qui compare un repli Rust au FTL fr-CH, **sur une table fermée** — une clé n'y est
  comparée que si on l'y inscrit, avec son nombre de sites. Les trois clés `*-cancel-blocked-fiscal-year-closed`
  y sont ; les clés neuves n'y entrent pas d'elles-mêmes.

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
    /// Exercice inconnu de la règle (non nommé à `open_period_rule`) → `DbError::Invariant` :
    /// l'appelant nomme les exercices des lignes qu'il interroge ; un exercice absent est un défaut
    /// de l'appelant, jamais une réponse (C-15-1a2-25).
    pub fn line_in_open_period(&self, fiscal_year_id: i64, entry_date: NaiveDate)
        -> Result<bool, DbError>;
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

**Exercice inconnu → `Invariant`, non `false`** (validation P1, F-11 ; C-15-1a2-25). Un `false` silencieux
serait *fail-closed* pour le rang 2 bis (refus de trop) mais **muet** pour la 15-1b, qui filtrerait une ligne
des propositions sans que rien le signale. Le `Result` coûte un `?` aux appelants, qui nomment tous à
`open_period_rule` les exercices des lignes qu'ils interrogent. Le prédicat **par ligne** partagé prend l'état
**trouvé** d'un exercice (`&FiscalYearState`) ; la recherche reste à chaque appelant : le mode `Manual` garde
la sienne telle quelle (`exercices.get(..).is_some_and(..)`, `letterings.rs:499-504` — son état est lu sur les
exercices des lignes mêmes, sous verrou, le cas ne s'y produit pas ; AC1 : ses tests inchangés),
`OpenPeriodRule` rend `Invariant`.

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

  **Doc-comments qui énumèrent les rangs ou les motifs — un ensemble clos de SITES NOMMÉS** (validation P1,
  F-5, R-6 ; **P2, R2-3 = F2-1, R2-7** : l'inventaire de la P1 était une *forme* de grep, `rangs? [0-9]+( (à|et)
  [0-9]+)?`, et elle ratait la majuscule, le tiret « 1-2 » et les énumérations en prose — les trois commentaires
  posés au-dessus des filtres mêmes que D3 modifie. Sites relevés par `git grep -niE "rangs? [0-9]+( ?(à|et|-)
  ?[0-9]+)?|refusés? ici|arrivent ici|Refusés par son geste"` sur `056997b0`, **contrôle, non définition** :
  la liste ci-dessous est la définition, chaque site est relu et, s'il devient faux avec le 2 bis, réécrit,
  puis trié au Dev Agent Record) :
  - la file : `settlement_cancellation.rs:28` (« Les rangs 2 à 5 » → « 2 à 5, dont 2 bis ») ;
  - **les commentaires des quatre filtres de D3** : `invoice_settlements_write.rs:838` (« Rangs 1-2 : refusés
    ici. Rangs 3-5 : laissés au socle » : le 2 bis y est aussi refusé), `supplier_invoices.rs:1061` (« Têtes et
    exercice clos : refusés ici », filtre de la facture), `supplier_invoices.rs:1267` (« Rang 1 et « exercice
    clos » : refusés ici », filtre du paiement) — `reconciliation_cancel.rs:322` (« (4) Les motifs — forme
    EXEMPTÉE ») n'énumère rien et reste juste, trié ;
  - les énoncés de rang des gestes : `invoice_settlements_write.rs:687-688` (« rangs 2 à 5 »), `:772-773` (« ne
    refuse lui-même que les rangs 1 et 2 … les rangs 3 à 5 ») ; `reconciliation_cancel.rs:263-265` (« les rangs 0
    et 2 … les rangs 3 à 5 ») ; `supplier_invoices.rs:954` (« rangs 2 à 5 ») ;
  - `kesh-db/src/errors.rs` : `:318` (« les rangs 2 à 5 »), `:329-330`, **`:340`** (« Refusés par son geste
    ([`DbError::SupplierInvoiceNotCancellable`]) : les deux têtes et l'exercice clos » → + le 2 bis),
    `:401-404` (la doc de `SettlementCancelBlocker::code()` énumère entre parenthèses les codes réemployés :
    y ajouter `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, de `DbError::LetteringAllLinesInClosedPeriods` — le
    code du 2 bis y est réemployé, la parenthèse le tait), `:868-870` ;
  - `kesh-api/src/errors.rs` : **`:3257-3258`** (« Seuls les rangs que le GESTE refuse lui-même arrivent ici
    (facture créditée, exercice clos) » → + le 2 bis), `:3308` (« rang 0 … rang 2 »), **`:3316-3318`** (« refusée
    par le geste lui-même (déjà annulée, exercice de l'achat clos, lot en cours) » → + le 2 bis) ;
  - `frontend/src/lib/shared/utils/settlement-cancel-blocked.ts:4-6` : « Ces quatre motifs tiennent à
    l'**écriture** de règlement (son exercice, son rapprochement, ses comptes, l'exercice du jour) » — le compte
    (AC7 d, « quatre » → « cinq ») **et** la parenthèse, qui doit nommer le lettrage figé par la période.
  Les sites qui ne parlent que d'un rang existant sans borner la queue (`kesh-api/src/errors.rs:2966`, rang 2
  d'une autre file ; `settlement_cancellation.rs:24-25`, `:43`, `:59`, `:64-66`) sont triés au Dev Agent Record,
  non réécrits. ⚠️ Le contrôle par grep rend des sites que la liste n'a pas nommés : chacun est **ajouté à la
  liste**, ou trié « sans objet » avec sa raison — un site ni nommé ni trié est le défaut que cette liste
  existe pour fermer.
- **Code** : `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, réemployé (C-15-1a2-11) — `SettlementCancelBlocker::code`
  pose que « tous ces codes réemploient ceux d'états du monde déjà nommés » (`errors.rs:401`) ; « toutes
  les lignes du groupe sont en période close » est l'état que ce code nomme déjà. Statut **409**.
- **Remède écrit, précis** (findings F-6 et L-6 de la P3 de la 15-1a2-i ; C-15-1a2-20). Le groupe redevient
  dissoluble dès qu'**une** de ses lignes est en période ouverte. Tout se lit sur **la ligne la plus récente
  du lettrage** (validation P1, F-7, R-11) : c'est elle qui se libère la première, et elle décide des deux
  causes — si son exercice est clôturé ou suivi d'un exercice clôturé, ceux des lignes plus anciennes le sont
  aussi. ⚠️ Ce n'est **pas** toujours le dernier règlement : un règlement peut être daté la veille de sa
  facture (`invoice_settlements_write.rs:114`), et un avoir hérité, après le dernier règlement. Deux causes,
  qui peuvent se cumuler :
  1. **la borne du verrou**, si cette date est sous la borne : un administrateur la fait reculer **avant
     elle**, par `POST /companies/current/books-lock/release` (`companies::unlock_books`, motif obligatoire ;
     nouvelle borne antérieure, ou aucune). Un recul qui laisse la borne **à ou après** cette date ne lève
     rien — vrai sans réserve, puisque c'est la plus récente : toutes les lignes restent alors sous la borne ;
  2. **la clôture**, si l'exercice de cette date est clôturé ou suivi d'un exercice clôturé : un
     administrateur **rouvre** les exercices clôturés, du plus récent au plus ancien (`fiscal_years::reopen`),
     jusqu'à celui de cette date.
  Quand les deux s'appliquent, il faut **les deux** : les textes énoncent les deux conditions, chacune avec
  son remède, reliées par « et » — jamais « ou » seul (C-15-1a2-26). Le refus ne porte **aucune date** dans
  `details` (C-15-1a2-20 : variante sans champ, comme le rang 2 ; la date se lit sur la pièce, et le texte dit
  laquelle chercher — « en général celle du dernier règlement »).
- **Textes** : une clé par famille, quatre locales — `settlement-cancel-blocked-lettering-closed`
  (règlement et solde, client **et** fournisseur : la clé de la queue est partagée),
  `reconciliation-cancel-blocked-lettering-closed`, `supplier-invoices-cancel-blocked-lettering-closed` — et
  leurs replis Rust (`kesh-api/src/errors.rs`, un bras dans chacune des trois tables), **mot pour mot** le
  FTL fr-CH, **comparés par G9** (les trois clés inscrites à sa `TABLE`, un site chacune — validation P1, R-2).
  **Les douze textes sont fixés ici** (validation P1, R-1, F-10 ; C-15-1a2-26) — chacun porte le marqueur
  d'ordre de G8 **et** la borne « jusqu'à celui-ci » de G8-bis. **L'antécédent de « celui-ci » est le dernier
  terme de la condition** (validation P2, R2-6 = F2-4 : dans « clôturé ou suivi d'un exercice clôturé », le nom
  le plus proche était « un exercice clôturé », le **successeur** — lu ainsi, quand l'exercice de la date est
  lui-même clos et suivi d'un clos, la réouverture s'arrêtait un exercice trop tôt et le refus revenait) : les
  deux causes sont donc écrites dans l'ordre « suivi d'un exercice clôturé, **ou clôturé lui-même** … jusqu'à
  celui-ci », qui laisse « son exercice » (celui de la date la plus récente) comme seul antécédent possible ;
  la borne littérale qu'exige G8-bis et le marqueur d'ordre de G8 sont gardés (C-15-1a2-30) ; aucun ne dit « son avoir » (un groupe fournisseur
  n'en a pas) ; la queue dit « sa facture », vrai pour le client comme pour le fournisseur :

  | clé | fr-CH |
  |---|---|
  | `settlement-cancel-blocked-lettering-closed` | Ce règlement est lettré avec sa facture, et toutes les lignes de ce lettrage sont dans une période close : il est figé. Pour pouvoir l'annuler, prenez la date la plus récente du lettrage (en général celle du dernier règlement) : si elle est sous le verrou de période, un administrateur doit faire reculer le verrou avant elle ; et si son exercice est suivi d'un exercice clôturé, ou clôturé lui-même, il doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent. |
  | `reconciliation-cancel-blocked-lettering-closed` | Ce rapprochement est lettré avec sa facture, et toutes les lignes de ce lettrage sont dans une période close : il est figé. Pour pouvoir l'annuler, prenez la date la plus récente du lettrage (en général celle du dernier règlement) : si elle est sous le verrou de période, un administrateur doit faire reculer le verrou avant elle ; et si son exercice est suivi d'un exercice clôturé, ou clôturé lui-même, il doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent. |
  | `supplier-invoices-cancel-blocked-lettering-closed` | Cette facture est lettrée avec son paiement, et toutes les lignes de ce lettrage sont dans une période close : il est figé. Pour pouvoir l'annuler, prenez la date la plus récente du lettrage (en général celle du paiement) : si elle est sous le verrou de période, un administrateur doit faire reculer le verrou avant elle ; et si son exercice est suivi d'un exercice clôturé, ou clôturé lui-même, il doit rouvrir les exercices clôturés jusqu'à celui-ci, en commençant par le plus récent. |

  | clé | de-CH |
  |---|---|
  | `settlement-cancel-blocked-lettering-closed` | Diese Zahlung ist mit ihrer Rechnung ausgeglichen, und alle Zeilen dieses Ausgleichs liegen in einer abgeschlossenen Periode: Er ist fixiert. Damit sie storniert werden kann, nehmen Sie das jüngste Datum des Ausgleichs (meist das der letzten Zahlung): Liegt es in der Periodensperre, muss ein Administrator die Sperre vor dieses Datum zurücksetzen; und folgt seinem Geschäftsjahr ein abgeschlossenes oder ist es selbst abgeschlossen, muss er die abgeschlossenen Geschäftsjahre bis zu diesem Geschäftsjahr wieder öffnen, beginnend mit dem neuesten. |
  | `reconciliation-cancel-blocked-lettering-closed` | Dieser Abgleich ist mit seiner Rechnung ausgeglichen, und alle Zeilen dieses Ausgleichs liegen in einer abgeschlossenen Periode: Er ist fixiert. Damit der Abgleich aufgehoben werden kann, nehmen Sie das jüngste Datum des Ausgleichs (meist das der letzten Zahlung): Liegt es in der Periodensperre, muss ein Administrator die Sperre vor dieses Datum zurücksetzen; und folgt seinem Geschäftsjahr ein abgeschlossenes oder ist es selbst abgeschlossen, muss er die abgeschlossenen Geschäftsjahre bis zu diesem Geschäftsjahr wieder öffnen, beginnend mit dem neuesten. |
  | `supplier-invoices-cancel-blocked-lettering-closed` | Diese Rechnung ist mit ihrer Zahlung ausgeglichen, und alle Zeilen dieses Ausgleichs liegen in einer abgeschlossenen Periode: Er ist fixiert. Damit sie storniert werden kann, nehmen Sie das jüngste Datum des Ausgleichs (meist das der Zahlung): Liegt es in der Periodensperre, muss ein Administrator die Sperre vor dieses Datum zurücksetzen; und folgt seinem Geschäftsjahr ein abgeschlossenes oder ist es selbst abgeschlossen, muss er die abgeschlossenen Geschäftsjahre bis zu diesem Geschäftsjahr wieder öffnen, beginnend mit dem neuesten. |

  | clé | it-CH |
  |---|---|
  | `settlement-cancel-blocked-lettering-closed` | Questo pagamento è abbinato alla sua fattura, e tutte le righe di questo abbinamento sono in un periodo chiuso: è bloccato. Per poterlo annullare, considerate la data più recente dell'abbinamento (in genere quella dell'ultimo pagamento): se cade nel blocco di periodo, un amministratore deve riportare il blocco prima di essa; e se il suo esercizio è seguito da un esercizio chiuso, o è chiuso esso stesso, deve riaprire gli esercizi chiusi fino a questo esercizio, cominciando dal più recente. |
  | `reconciliation-cancel-blocked-lettering-closed` | Questa riconciliazione è abbinata alla sua fattura, e tutte le righe di questo abbinamento sono in un periodo chiuso: è bloccata. Per poterla annullare, considerate la data più recente dell'abbinamento (in genere quella dell'ultimo pagamento): se cade nel blocco di periodo, un amministratore deve riportare il blocco prima di essa; e se il suo esercizio è seguito da un esercizio chiuso, o è chiuso esso stesso, deve riaprire gli esercizi chiusi fino a questo esercizio, cominciando dal più recente. |
  | `supplier-invoices-cancel-blocked-lettering-closed` | Questa fattura è abbinata al suo pagamento, e tutte le righe di questo abbinamento sono in un periodo chiuso: è bloccata. Per poterla annullare, considerate la data più recente dell'abbinamento (in genere quella del pagamento): se cade nel blocco di periodo, un amministratore deve riportare il blocco prima di essa; e se il suo esercizio è seguito da un esercizio chiuso, o è chiuso esso stesso, deve riaprire gli esercizi chiusi fino a questo esercizio, cominciando dal più recente. |

  | clé | en-CH |
  |---|---|
  | `settlement-cancel-blocked-lettering-closed` | This settlement is matched with its invoice, and all lines of this matching group are in a closed period: it is fixed. Before it can be cancelled, take the most recent date of the matching group (usually that of the last settlement): if it falls under the period lock, an administrator must move the lock back before it; and if its fiscal year is followed by a closed fiscal year, or is closed itself, the administrator must reopen the closed fiscal years down to this one, starting with the most recent. |
  | `reconciliation-cancel-blocked-lettering-closed` | This reconciliation is matched with its invoice, and all lines of this matching group are in a closed period: it is fixed. Before it can be cancelled, take the most recent date of the matching group (usually that of the last settlement): if it falls under the period lock, an administrator must move the lock back before it; and if its fiscal year is followed by a closed fiscal year, or is closed itself, the administrator must reopen the closed fiscal years down to this one, starting with the most recent. |
  | `supplier-invoices-cancel-blocked-lettering-closed` | This invoice is matched with its payment, and all lines of this matching group are in a closed period: it is fixed. Before it can be cancelled, take the most recent date of the matching group (usually that of the payment): if it falls under the period lock, an administrator must move the lock back before it; and if its fiscal year is followed by a closed fiscal year, or is closed itself, the administrator must reopen the closed fiscal years down to this one, starting with the most recent. |

  Vocabulaire repris des catalogues : *lettrage* / *Ausgleich* / *abbinamento* / *matching group*
  (`error-lettering-*`), *verrou de période* / *Periodensperre* / *blocco di periodo* / *period lock*
  (`settings-books-lock-title`). ⚠️ Les gardes G8 et G8-bis prennent le **verbe fr-CH** comme critère de domaine
  (`rouvrir` y est) : les trois clés y entrent d'elles-mêmes ; elles sont en outre **nommées** dans une liste
  anti-muet de G8 (la constante sœur `CLES_2_BIS`, AC7 a), pour qu'une réécriture qui perdrait le verbe ne les sorte pas du contrôle en
  silence (AC7 a). Le développeur peut retoucher la forme de/it/en ; il ne retire ni le marqueur, ni la borne,
  ni l'antécédent, et il recopie le fr-CH tel quel dans les replis Rust et Svelte.

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
écriture **propre** (`ReconciliationKind::Entry`) ne rencontre ce rang pour **aucune écriture créée par le
rapprochement** : elle n'appartient à aucune pièce et ne porte pas de lettrage `document`. ⚠️ **Seule exception,
nommée** (validation P1, R-12) : l'état hérité d'avant la 24-2 que décrit `reconciliation_cancel.rs:26-31` — un
lien qui pointe l'écriture de **vente**, classé `Entry` et aujourd'hui refusé plus loin par le socle
(`OWNED_BY_INVOICE`). Si cette vente est dans un groupe `document` figé, le rang 2 bis parle **avant** le socle :
le dé-rapprochement reste refusé, rien n'est écrit, seul le motif change — et il est rendu dans **sa** famille,
le filtre de l'étape (4) ne dépendant pas du `kind`.

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

**Le montage, dans cet ordre** (validation P1, F-9 ; C-15-1a2-27) :
1. **les écritures d'abord**, par les gestes réels (facture validée, règlement, solde, paiement) — une
   écriture ne se crée plus sous une borne posée (`PeriodLocked`) : verrouiller d'abord rendrait le montage
   impossible, ou le ferait passer par du SQL qui contourne ce qu'on veut éprouver. ⚠️ **Toutes les lignes du
   groupe sont datées STRICTEMENT AVANT aujourd'hui** (validation P2, R2-5 = F2-3) : `lock_books` **et**
   `unlock_books` refusent toute borne `>= aujourd'hui` (`BOOKS_LOCK_BOUND_NOT_PAST`, `companies.rs:369` et
   `:455-458`) — un règlement daté du jour rendrait le verrou **et** le déverrouillage d'AC3 (a) impossibles.
   Calendrier de `monter` (`invoice_settlement.rs`) : `D = aujourd'hui − 60 j`, facture `D − 20`, règlement
   `D − 10` ; les montages API (`invoice_echeancier_e2e.rs`, `reconciliation_e2e.rs`,
   `supplier_settlement_cancel_e2e.rs`) passent un `settledOn` passé de même ;
2. **le groupe** par l'`UPDATE` brut, suivi d'une **assertion de montage** : nombre de lignes marquées égal au
   nombre attendu (`rows_affected`), somme `débit − crédit` nulle sur la clé, clé = plus petite ligne — sans
   elle, un `UPDATE` qui ne trouverait rien laisserait les assertions négatives (AC2 b) vertes à vide ;
3. **le verrou ensuite**, par `companies::lock_books` (jamais par `UPDATE` brut : les dates sont passées, le
   chemin réel les accepte, et une borne `>= aujourd'hui` est un état que l'application interdit — invariant
   I2), à une date **au moins égale à la plus récente** des lignes du groupe ; puis, s'il y a lieu, la
   clôture ;
4. **règlement et solde sur deux factures distinctes** : sur une même facture soldée par un règlement **et** un
   solde, l'annulation du règlement rend `INVOICE_WRITTEN_OFF` (rang 1 bis, qui précède le 2 bis) — c'est la
   précédence 1 bis × 2 bis (AC2 c), pas le refus d'AC3.

**Après la 15-1a2-i** : au 31.03 verrouillé, la fiche facture n'offre pas l'annulation d'un règlement dont
le lettrage est figé, et dit pourquoi et qui peut la débloquer ; la vue des postes ouverts « au 31.03 » ne
bouge plus. ⚠️ **Tolérance résiduelle, nommée** : la règle se lit sans verrou sur la borne et sur les
exercices autres que celui de l'écriture examinée (tenu par le geste). Un `lock_books` (ou la clôture d'un
autre exercice du groupe **ou d'un exercice postérieur** à celui de la ligne la plus récente, que
`find_later_closed` lit sans verrou) validé entre la lecture du rang 2 bis et le `COMMIT` laisse passer l'annulation
— la tolérance qu'a déjà une écriture créée pendant la pose du verrou. Aucun verrou neuf.

### D5 — Le message `LETTERING_IS_DOCUMENT` neutre, et un doc-comment (C-15-1a2-8, C-15-1a2-7)

*(Déplacés de la 15-1a2-i pour que les textes et `kesh-db/src/errors.rs` ne soient touchés que par une
story ; C-15-1a2-19.)*

- Le message de `LETTERING_IS_DOCUMENT` dit aujourd'hui « annulez le règlement plutôt », faux pour un groupe
  facture + avoir (aucun règlement à annuler, aucun avoir annulable). Réécrit **neutre** dans les quatre
  locales (`crates/kesh-i18n/locales/*/messages.ftl:54`) et son repli Rust (`kesh-api/src/errors.rs`, branche
  `DbError::LetteringIsDocument`, `:3057-3058`) — texte **fixé** (validation P1, F-10 : « son avoir » était faux
  pour le groupe d'une facture fournisseur, qui n'en a pas) :
  fr-CH « Ce lettrage est celui d'une pièce : il suit la pièce et ses règlements, et ne se défait pas à la
  main. » ; de-CH « Dieser Ausgleich ist der eines Belegs: Er folgt dem Beleg und seinen Zahlungen und lässt
  sich nicht von Hand aufheben. » ; it-CH « Questo abbinamento è quello di un documento: segue il documento e i
  suoi pagamenti, e non si annulla a mano. » ; en-CH « This matching group belongs to a document: it follows the
  document and its settlements, and cannot be undone by hand. » Vrai avant la 15-1a2-i (aucun groupe
  `document` : la route ne le rend pas) comme après, pour le client (« la pièce » couvre l'avoir) comme pour le
  fournisseur. Le repli Rust entre à la `TABLE` de G9 (un site), comme les trois clés neuves.
- Doc-comment de `SettlementCancelBlocker::InvoiceCredited` (`kesh-db/src/errors.rs` ≈ `:350`, « son
  traitement est la 15-1a2 ») réécrit selon C-15-1a2-7 : le règlement reste ouvert au compte débiteurs, la
  15-1a2 l'a tranché.

## Critères d'acceptation

**AC1** — **La règle des périodes** (D1) : `open_period_rule` / `line_in_open_period` / `lines_in_open_period`
existent avec les signatures de D1 ; sur un exercice ouvert sans successeur clos, une ligne **datée du jour
de la borne** n'est pas en période ouverte, une ligne **du lendemain** l'est ; une ligne d'un exercice
clôturé, ou d'un exercice suivi d'un exercice clôturé, ne l'est pas ; sans borne, toute ligne d'un exercice
ouvert sans successeur clos l'est ; `line_in_open_period` sur un exercice **non nommé** à `open_period_rule`
rend `DbError::Invariant` (C-15-1a2-25). Les tests existants du mode `Manual` (`letterings.rs`, R7) restent
verts **sans modification de leurs assertions** (le prédicat par ligne est partagé).

**AC2** — **Le rang 2 bis dans la file commune** (D2), sur un groupe `document` posé à la main selon le
montage de D4 (écritures, groupe et assertion de montage, verrou) :
(a) facture soldée par un règlement complet, groupe posé, verrou posé **ensuite** à la date du règlement ou
après, exercice ouvert → `settlement_entry_cancel_blocker` sur l'écriture de règlement rend
`DocumentLetteringInClosedPeriods` ;
(b) même montage **sans** groupe → pas ce rang ; groupe dont **une** ligne est après la borne → pas ce rang ;
(c) **précédence, contre TOUS les rangs voisins, lecture ET écriture** (validation P1, R-4 = F-3 ;
C-15-1a2-27) — par les bancs qui la prouvent déjà pour les autres rangs, non par un test local :
- **client** : la matrice `RANGS` de `kesh-db/tests/invoice_settlement.rs`
  (`la_precedence_de_l_annulation_lecture_et_ecriture`) passe de **cinq** à **six** rangs — `InvoiceCredited`,
  `FiscalYearClosed`, **`DocumentLetteringInClosedPeriods`**, `MatchedBankTransaction`, `AccountArchived`,
  `NoOpenFiscalYearToday` —, chaque rang seul puis chaque paire, la lecture annonçant le plus fort et
  l'écriture refusant pour ce même motif : **6 seuls + 14 paires = 20 cas**. La quinzième paire,
  `InvoiceCredited` × 2 bis, **n'existe dans aucune donnée** : une facture créditée **et** réglée a
  `Σ C(I) ≠ 0`, aucun chemin n'y pose de groupe (15-1a2-i P1, C-15-1a2-7 ; le rattrapage de la 15-1a2-ii suit
  la même règle) — elle est **exclue et nommée** dans le test, non montée en forçant un groupe déséquilibré.
  `monter` gagne le motif 2 bis (la facture y est alors **soldée** : règlement du montant entier ou second
  règlement du reste, choix écrit au Dev Agent Record — le groupe doit être à somme nulle) ; le bras de
  `ecriture_attendue` du 2 bis rend `SettlementNotCancellable { blocker }`, comme `FiscalYearClosed` ;
- **1 bis × 2 bis** (rang absent de `RANGS`) : dans `kesh-db/tests/invoice_write_off.rs`, une facture soldée par
  un règlement **et** un solde, groupe figé : l'annulation du **règlement** rend `WriteOffExists` (lecture et
  écriture), celle du **solde** rend le 2 bis ;
- **fournisseur, paiement** : `tail_motives_through_the_supplier_path`
  (`kesh-db/tests/supplier_invoices_repository.rs`) gagne quatre cas — `[2 bis]` → 2 bis ;
  `[FiscalYearClosed, 2 bis]` → `FiscalYearClosed` ; `[2 bis, AccountArchived]` → 2 bis ;
  `[2 bis, NoOpenFiscalYearToday]` → 2 bis — et son `match` d'écriture le bras
  `SettlementNotCancellable { blocker: DocumentLetteringInClosedPeriods }` ; la tête `SupplierInvoiceNotPaid` ×
  2 bis n'existe pas (une facture non payée n'a pas de règlement, donc pas de groupe) — nommé ;
- **fournisseur, facture** : `invoice_cancel_motives_and_their_precedence` gagne quatre cas sur une facture
  **payée** — `[2 bis]` → 2 bis ; `[FiscalYearClosed, 2 bis]` → `FiscalYearClosed` ; `[2 bis, AccountArchived]`
  → 2 bis ; `[2 bis, NoOpenFiscalYearToday]` → 2 bis — et le bras `SupplierInvoiceNotCancellable { blocker }`
  pour le 2 bis. Le rang 6 (`SupplierInvoiceInPaymentBatch`) × 2 bis est **inatteignable** (validation P2, R2-2 = F2-2) :
  il suppose une facture **payée** dans un lot `generated`, et `supplier_invoices.rs:1224-1228` l'écrit — « une
  facture `paid` ne peut pas être dans un lot `generated` (`create_batch` exige `open`, `pay` et `cancel`
  refusent une facture en lot `generated`, et `confirm_batch` règle et confirme dans la même transaction) ».
  Il est **nommé au test avec cette raison**, comme `InvoiceCredited` × 2 bis et la tête `SupplierInvoiceNotPaid`
  × 2 bis, et **non monté en forçant** : `engager_dans_un_lot` forge un lot en SQL sur n'importe quelle facture
  (`supplier_invoices_repository.rs:1744-1780`), si bien qu'un critère « montable » serait toujours vrai et
  ferait monter un état que le code déclare impossible.

**AC3** — **Règlement et solde clients** : (a) **règlement** — facture A soldée par un règlement, groupe et
verrou selon D4 → `invoice_settlements_write::cancel_settlement` rend `SettlementNotCancellable { blocker:
DocumentLetteringInClosedPeriods }` ; **rien** n'est écrit (aucune écriture inverse, la ligne
`invoice_settlements` reste, les marques sont intactes, aucune entrée d'audit) ; `settlement_cancel_blocker`
(la lecture de la vue) rend ce motif ; un administrateur **déverrouille** (`companies::unlock_books`, motif) avec
une nouvelle borne **à** la date la plus récente du groupe → le motif demeure (déverrouillage insuffisant) ;
puis **avant** elle → la lecture ne rend plus ce rang ; (b) **solde** — facture B, **distincte** (D4 point 4),
soldée par un solde (`write_off`) seul, même montage → l'annulation du solde est refusée pour le même motif,
rien n'est écrit. Par l'API (`invoice_echeancier_e2e.rs`) : `cancelBlockedBy =
"LETTERING_ALL_LINES_IN_CLOSED_PERIODS"` dans la vue, puis `POST /invoices/{id}/settlements/{settlementId}/cancel`
→ `409`, même code, texte de la clé `settlement-cancel-blocked-lettering-closed`. *(L'annulation elle-même
après déverrouillage — dissolution, paires `reversal` — est l'AC14 (b) de la 15-1a2-i : sans la
synchronisation, rien ne dissout le groupe posé à la main.)*

**AC4** — **Dé-rapprochement** : facture encaissée par un règlement rapproché d'une transaction (montage de
`invoice_settlement.rs` : `settle_cash` puis `match_to_bank`), groupe et verrou selon D4 → au dépôt,
`reconciliation_cancel::cancel` rend `ReconciliationNotCancellable { blocker: DocumentLetteringInClosedPeriods }`
et le lien `matched_entry_id` est **intact** ; par l'API, `POST /reconciliation/transactions/{id}/cancel` rend
`409 LETTERING_ALL_LINES_IN_CLOSED_PERIODS` dans **sa** famille (clé `reconciliation-cancel-blocked-lettering-closed`)
— le code **et** la clé assertés, ce qui attrape un motif codé en dur (L-2) — ; `GET
/reconciliation/transactions/{id}` porte le motif.

**AC5** — **Fournisseurs** : achat et paiement par les gestes, groupe sur la ligne d'achat et celle du paiement
(compte fournisseurs), verrou ensuite (D4) → l'annulation du **paiement** est refusée
(`SettlementNotCancellable`, clé `settlement-cancel-blocked-lettering-closed` par l'API), l'annulation de la
**facture** aussi (`SupplierInvoiceNotCancellable`, clé `supplier-invoices-cancel-blocked-lettering-closed`) ;
rien n'est écrit, la facture reste `paid` ; les deux prédicteurs (`settlementCancelBlockedBy`,
`cancelBlockedBy`) le disent.

**AC6** — **État hérité « exercice du groupe ouvert, suivi d'un exercice clos »** (fabriqué en SQL brut,
comme les tests de la 15-12b) → même refus qu'en AC3 (a), sans verrou de période : c'est la seconde cause
seule.

**AC7** — **Les textes et l'écran** : (a) les trois clés existent dans les **quatre** locales avec les textes de
D2 ; **G8 et G8-bis vertes** sur elles (marqueur d'ordre et borne « jusqu'à celui-ci » dans chaque locale), et
les trois clés **nommées** dans une **constante sœur dédiée** `CLES_2_BIS: [&str; 3]` de G8
(`les_prescriptions_de_reouverture_disent_l_ordre`), chaînée dans la boucle anti-muet existante
(`EXEMPTEES.iter().chain(CLES_569.iter()).chain(CLES_2_BIS.iter())`) — **`CLES_569` reste `[&str; 6]`**, son
commentaire « Les six clés de #569 » et le plancher `controlees >= 10` (« 6 clés de #569 + 4 ») restent vrais :
mêler trois clés d'une autre story à une liste documentée comme « les six clés de #569 » rendrait son
commentaire faux (validation P2, F2-6 ; C-15-1a2-30) ; **G9 verte** : les trois clés et `error-lettering-is-document`
inscrites à la `TABLE` de `les_replis_rust_suivent_le_catalogue` (`kesh-api/tests/textes_coherents.rs`), un site
chacune — c'est G9, non un test neuf, qui prouve « le repli Rust dit mot pour mot le FTL fr-CH » (validation P1,
R-2) ; (b) l'écran masque l'annulation et affiche le texte du motif — fiche facture
(`settlement-cancel-blocked.ts`), dialogue de dé-rapprochement (`reconciliation-cancel.ts`), fiche fournisseur
(`invoice-cancel.ts`, et la queue partagée pour le paiement) ; (c) les gardes frontend à valeur épinglée sont **recomptées**, pas contournées (finding F-3 de la P3 ; **P2,
R2-1** : la version P1 n'en nommait que deux et en omettait une qui rougit) — **trois gardes bougent** :
- `CLES_RELEVEES` de `lib/shared/i18n-un-repli-par-cle.test.ts` (`:125` et son `toBe(CLES_RELEVEES)` `:207`,
  `213` sur `056997b0`) : **213 → 214, +1 nommée** — `supplier-invoices-cancel-blocked-lettering-closed`, clé au
  préfixe `supplier-invoices-` portant un repli littéral dans `invoice-cancel.ts` (les deux autres clés neuves,
  `settlement-cancel-blocked-lettering-closed` et `reconciliation-cancel-blocked-lettering-closed`, n'ont aucun
  des quatre préfixes de la garde et n'y entrent pas). Le **commentaire** du fichier, qui exige qu'une hausse
  soit nommée (« Un +11 dont on ne sait pas dire lesquels se recompte, il ne s'entérine pas »), gagne son
  paragraphe `213 → 214, +1 nommée (Story 15-1a2-0, #518)` avec la clé, recomptée par la procédure du fichier ;
- `sitesTotal` de `lib/shared/i18n-keys.test.ts` (`:526`, `1920` sur `056997b0`) : **+3** attendu — un
  `i18nMsg(` neuf dans chacun de `settlement-cancel-blocked.ts` (4 → 5), `reconciliation-cancel.ts` (7 → 8) et
  `invoice-cancel.ts` (6 → 7) — soit `1923`, **recompté** par la procédure écrite dans ce fichier
  (`grep -o "i18nMsg("` aux deux bornes), non entériné ; `sitesNonResolus`, `relais`, `sitesGabarit` (clés
  littérales) ne bougent pas, `litterauxMin` et `clesDepuisTsMin` sont des planchers ;
- la liste `REPLIS_A_SITE_UNIQUE` de `lib/shared/i18n-repli-divergent-actif.test.ts` (`:216`) gagne les
  **trois** clés neuves — **impératif** : c'est ce test seul qui fait tenir « chaque repli Svelte dit mot pour
  mot le FTL » pour une clé à site unique ; son commentaire « Chacune des dix clés » (`:209`) devient « treize ».
  Relevé des autres gardes à valeur épinglée de `lib/shared/*.test.ts`
  (`grep -nE "toBe\([0-9]{2,}\)|const [A-Z_]+ = [0-9]{2,}"`, `056997b0`) : **seule `CANDIDATES_ATTENDUES = 47`**
  de `i18n-libelle-en-dur.test.ts` reste, et elle **ne bouge pas** (elle compte des déclarations à suffixe
  `Label`/`Message` : les trois ajouts sont des `case` dans des fonctions existantes). T0 refait ce grep ;
  `lint-i18n-ownership` vert ;
(d) **les décomptes de motifs écrits en dur**, par la **valeur** (validation P1, R-6) — `git grep -nE
"sept motifs|les sept|Les sept|septième|des six|six motifs|quatre motifs|Ces quatre" frontend/src` trié (sur `056997b0` : quinze sorties, dont des commentaires historiques de `i18n-keys.test.ts:259`, `:266`, `:470` et des « septième conflit » sans rapport, qui restent) : « sept » →
« huit » partout où il compte les motifs du dé-rapprochement (`reconciliation.types.ts:126`,
`reconciliation-cancel.ts:7` et `:28`, `reconciliation-cancel.test.ts:43`), « septième » → « neuvième »
(`reconciliation-cancel.ts:81`), « Un motif des six » (`:89`, déjà périmé) → « des huit »,
`settlement-cancel-blocked.ts:4` « Ces quatre motifs » → « cinq » ; `blocker-messages.ts:9`, qui compte les
motifs de pièce de la contre-passation, **ne change pas** ; les doc-comments Rust à numéros de rang sont ceux de
D2 ; (e) **les tests Vitest à listes figées** (validation P1, F-4) gagnent le code neuf, faute de quoi « chaque
code » cesserait d'être vrai sans rougir : `features/supplier-invoices/settlement-cancel.test.ts:22-31` (« la queue
est RÉUTILISÉE », quatre codes), `features/invoices/InvoiceSettlements.test.ts:77-83` (code → marqueur : le code
neuf avec un extrait **propre au texte neuf** — « toutes les lignes de ce lettrage » —, non « en commençant par
le plus récent », que `FISCAL_YEAR_CLOSED` porte déjà à la ligne 79 : une mutation renvoyant le code neuf vers
le texte de l'exercice clos passerait ; validation P2, R2-4 ; même extrait pour le tableau de
`reconciliation-cancel.test.ts:21-29`), `shared/utils/settlement-cancel-blocked.test.ts` (`textes.size`
`4` → `5`, `:29`, et la liste `:50-60`) ; `lint-i18n-ownership` vert.

**AC8** — *déplacé à la 15-1a2-i* (**AC18** : documentation publique du refus — `api-external.md`, manuels,
`CHANGELOG.md`, PDF ; C-15-1a2-24). Numéro non réattribué. Reste ici la **propagation de la valeur dans le
code** : `git grep -nF "LETTERING_ALL_LINES_IN_CLOSED_PERIODS" -- crates/kesh-api/src crates/kesh-db/src
frontend/src` — sur `056997b0`, **trois** sites (`kesh-db/src/errors.rs:1356`, le `code()` de
`DbError::LetteringAllLinesInClosedPeriods` ; `kesh-api/src/errors.rs:3005` et `:4248`), auxquels s'ajoutent
les sites neufs de cette story — chaque site trié au Dev Agent Record (validation P1, R-9 = F-8 : la version P1
en annonçait deux côté Rust). Le texte rendu sous ce code **diffère selon la route** (celui du lettrage manuel,
`error-lettering-all-lines-in-closed-periods` ; celui de la famille d'annulation) : doc-comment de la
variante.

**AC9** — **Message `LETTERING_IS_DOCUMENT` et doc-comment** (D5) : les quatre `.ftl` et le repli Rust disent
le texte neutre de D5 ; `git grep -nE "annulez le règlement plutôt|Stornieren Sie die Zahlung, statt|annullate il
pagamento invece|cancel the settlement rather than" -- crates` ne rend plus rien (les `docs/` n'en portent pas :
la ligne `api-external.md:324` dit autre chose, « annuler le règlement, pas délettrer », et appartient à la
15-1a2-i AC12) ; le test `errors.rs:4283`, qui n'asserte que la clé (`texte(cle)`), reste vert sans
modification ; G9 compare le repli (AC7 a).

## Tasks

- [x] **T0** — Relevés au sol sur la base réelle du développement : re-greper les ancres par le nom
      (`settlement_entry_cancel_blocker`, les quatre filtres de D3, les trois tables de textes, `RANGS`,
      `ecriture_attendue`, `tail_motives_through_the_supplier_path`, `invoice_cancel_motives_and_their_precedence`,
      la `TABLE` de G9, `CLES_569` de G8) ; vérifier que les routes des quatre gestes restent `Rejouee`
      (`audit_route_registry.rs`) ; relever `sitesTotal` et la liste `REPLIS_A_SITE_UNIQUE` aux deux bornes ;
      re-greper la phrase de `supplier_invoices.rs:1224-1228` (« ne peut pas être dans un lot `generated` »)
      qui fonde le rang 6 × 2 bis **inatteignable** (AC2 c — rien à trancher, la raison est écrite) ;
      grepper les gardes Vitest à valeur épinglée (`toBe(<nombre>)` sur un inventaire i18n) et vérifier que la
      liste de l'AC7 (c) est close.
- [x] **T1** (D1, AC1) — `OpenPeriodRule`, `open_period_rule`, `line_in_open_period` (`Result`, `Invariant` sur
      un exercice non nommé), `lines_in_open_period`, prédicat par ligne factorisé avec
      `any_line_in_open_period` ; doc-comments (lecture sans verrou, tolérance, borne stricte, exercice inconnu).
- [x] **T2** (D2, AC2, AC6) — Variante `DocumentLetteringInClosedPeriods` (doc-comment : place, code
      réemployé, remède sur la ligne la plus récente, texte qui diffère selon la route), son `code()` ;
      `document_group_frozen_by_periods` ; évaluation entre les rangs 2 et 3 de
      `settlement_entry_cancel_blocker` ; **les doc-comments qui énumèrent les rangs ou les motifs** : la liste
      de **sites nommés** de D2 (dont les commentaires des quatre filtres, `kesh-db/src/errors.rs:340` et
      `:401-404`, `kesh-api/src/errors.rs:3257-3258` et `:3316-3318`, `settlement-cancel-blocked.ts:4-6`), le
      grep n'étant qu'un contrôle ; chaque site trié au Dev Agent Record.
- [x] **T3** (D3, AC3–AC5) — Les quatre filtres de refus, motifs **liés** (`blocker @`), dé-rapprochement
      avant le lien défait.
- [x] **T4** (D2, AC7) — `kesh-api/src/errors.rs` : un bras dans chacune des trois tables, replis mot pour mot ;
      trois clés × quatre locales, **textes de D2** ; G8 (constante sœur `CLES_2_BIS` + trois clés, chaînée ; `CLES_569` inchangée), G9 (`TABLE` + quatre clés) ;
      frontend : `SettlementCancelTailCode` et `settlementCancelTailMessage`, liste et texte de
      `reconciliation-cancel.ts` et son type, cas de `invoice-cancel.ts` ; décomptes de motifs (AC7 d) ; tests
      Vitest à listes figées (AC7 e) ; **trois** gardes recomptées (AC7 c) : `CLES_RELEVEES` 213 → 214
      (clé `supplier-invoices-cancel-blocked-lettering-closed` nommée dans son commentaire), `sitesTotal`
      (+3, recompté), `REPLIS_A_SITE_UNIQUE` (+3, « treize ») ;
      `lint-i18n-ownership` vert.
- [x] **T5** (D5, AC9) — Message `LETTERING_IS_DOCUMENT` (quatre `.ftl` + repli Rust, textes de D5) ;
      doc-comment d'`InvoiceCredited`.
- [x] **T6** — Tests (liste ci-dessous) ; propagation de la valeur dans le code (AC8, trois sites sur la base).
- **T7** — *déplacée à la 15-1a2-i* (T6 de cette fiche-là, AC18 : `api-external.md`, manuels, CHANGELOG, PDF ;
  C-15-1a2-24). Numéro non réattribué.

**Tests prévus** (13 neufs ; 5 tests Rust et 7 fichiers Vitest modifiés) — chaque test **dans le binaire qui
porte déjà son montage** (validation P1, R-5 ; C-15-1a2-27 : `validated_invoice`, `settle_cash`,
`match_to_bank`, `monter` sont locaux à `invoice_settlement.rs`, les montages fournisseurs à
`supplier_invoices_repository.rs`, ceux du solde à `invoice_write_off.rs`, ceux de R7 à `letterings.rs` ; un
binaire neuf les aurait recopiés) :
- `crates/kesh-db/tests/letterings.rs` — 2 neufs : `open_period_rule_reads_the_bound_strictly` (AC1),
  `open_period_rule_refuses_an_unnamed_fiscal_year` (AC1, `Invariant`) ;
- `crates/kesh-db/tests/invoice_settlement.rs` — 4 neufs : `rank_2_bis_sees_a_frozen_document_group` (AC2 a, b),
  `settlement_cancel_is_refused_under_a_frozen_lettering` (AC3 a : rien d'écrit, lecture, déverrouillage
  insuffisant puis suffisant), `unreconcile_refuses_in_its_family_before_unlinking` (AC4, dépôt : variante
  **et** lien intact), `later_closed_year_group_is_refused` (AC6) ; **modifié** :
  `la_precedence_de_l_annulation_lecture_et_ecriture` (AC2 c, `RANGS` à six, 20 cas, paire 1 × 2 bis nommée
  exclue) — avec ses aides `monter` (motif 2 bis) et `ecriture_attendue` (bras neuf ; validation P1, R-3 =
  F-3 : ce `match` exhaustif cesse de compiler sans lui, et rend tout le binaire rouge) ;
- `crates/kesh-db/tests/invoice_write_off.rs` — 1 neuf : `write_off_cancel_and_rank_1_bis_under_a_frozen_lettering`
  (AC3 b : solde refusé, rien d'écrit ; AC2 c : 1 bis × 2 bis) ;
- `crates/kesh-db/tests/supplier_invoices_repository.rs` — 1 neuf : `supplier_cancels_are_refused_under_a_frozen_lettering`
  (AC5, dépôt : rien d'écrit, facture `paid`) ; **modifiés** : `tail_motives_through_the_supplier_path` (+4
  cas) et `invoice_cancel_motives_and_their_precedence` (+4 cas ; rang 6 × 2 bis inatteignable, nommé) — avec
  leurs aides `monter` et `monter_achat` (motif 2 bis) ;
- `crates/kesh-api/src/errors.rs` (`mod tests`) — 1 neuf : `closed_lettering_texts_follow_their_family` (AC7 a :
  chaque famille rend **sa** clé et le code `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` ; l'égalité mot pour mot
  repli ↔ FTL est la part de G9, non de ce test) ;
- `crates/kesh-api/tests/textes_coherents.rs` — **modifié** : `les_replis_rust_suivent_le_catalogue` (`TABLE` +
  quatre clés) ;
- `crates/kesh-i18n/src/loader.rs` (`mod tests`) — **modifié** : `les_prescriptions_de_reouverture_disent_l_ordre`
  (constante sœur `CLES_2_BIS`, trois clés, chaînée à la boucle anti-muet ; `CLES_569` inchangée) ; `les_prescriptions_de_reouverture_sont_bornees` (G8-bis) **inchangé**, il les
  contrôle par le domaine ;
- `crates/kesh-api/tests/invoice_echeancier_e2e.rs` — 1 neuf : `settlement_cancel_blocked_by_closed_lettering`
  (AC3 par l'API : `cancelBlockedBy` de la vue, puis `POST …/cancel` → 409, même code, clé) ;
- `crates/kesh-api/tests/reconciliation_e2e.rs` — 1 neuf : `unreconcile_of_a_closed_lettering_is_refused_in_its_family`
  (AC4 par l'API : code, clé, rien d'écrit) ;
- `crates/kesh-api/tests/supplier_settlement_cancel_e2e.rs` — 1 neuf : `cancel_blocked_by_closed_lettering` (AC5 par
  l'API : les deux prédicteurs, puis 409 et clé) ;
- `frontend/src/lib/features/supplier-invoices/invoice-cancel.test.ts` (neuf, Vitest) — 1 : le motif a son
  texte, mot pour mot le FTL (AC7 b) ;
- **modifiés** (Vitest, **sept** fichiers) : `lib/shared/utils/settlement-cancel-blocked.test.ts` (code neuf, taille
  `4` → `5`, liste `:50-60`) et `features/reconciliation/reconciliation-cancel.test.ts` (code neuf, « huit »),
  `features/supplier-invoices/settlement-cancel.test.ts` et `features/invoices/InvoiceSettlements.test.ts`
  (listes figées, AC7 e), `lib/shared/i18n-keys.test.ts` (`sitesTotal` recompté),
  `lib/shared/i18n-repli-divergent-actif.test.ts` (`REPLIS_A_SITE_UNIQUE` + trois clés) et
  `lib/shared/i18n-un-repli-par-cle.test.ts` (`CLES_RELEVEES` 213 → 214, clé nommée au commentaire).

*(Recompte depuis cette liste : 2 + 4 + 1 + 1 + 1 + 1 + 1 + 1 + 1 = **13 fonctions de test neuves** (12 Rust,
1 Vitest) ; **5 tests Rust modifiés** (`la_precedence_de_l_annulation_lecture_et_ecriture`,
`tail_motives_through_the_supplier_path`, `invoice_cancel_motives_and_their_precedence`,
`les_replis_rust_suivent_le_catalogue`, `les_prescriptions_de_reouverture_disent_l_ordre`) et **4 aides** de
montage (`monter` et `ecriture_attendue` d'`invoice_settlement.rs`, `monter` et `monter_achat` de
`supplier_invoices_repository.rs`) ; **7 fichiers Vitest modifiés** (les six de la P1 + `i18n-un-repli-par-cle.test.ts`, P2 R2-1 : le recompte
« 6 » de la P1 omettait la garde `CLES_RELEVEES`). Aucun fichier de test neuf côté Rust. À la
P1 : 12 neufs dont un fichier neuf de sept tests, 4 fichiers Vitest modifiés — décompte faux, il omettait le
`match` exhaustif d'`invoice_settlement.rs`, R-3 = F-3.)*

## Dev Notes

- **Gate `kesh-db` complet** (repositories touchés : ciblage interdit, `CLAUDE.md`) ; base remise à zéro avant,
  **sans** redémarrer le conteneur partagé (consignes de l'Epic 15). **Gate frontend complet** (`npm run
  check`, `lint-i18n-ownership`, `test:unit`, `build`) et **E2E complet** au dernier commit de code (D7).
- **Aucune migration** : P5–P8 sans objet ; aucun bump.
- **Aucune documentation publique** (C-15-1a2-24) : `docs/`, `CHANGELOG.md` et les PDF ne sont pas touchés par
  cette story — c'est l'AC18 de la 15-1a2-i. Contrôle : `git diff --stat` de la story sur `docs/` et
  `CHANGELOG.md` vide, consigné au Dev Agent Record.
- **Verrous** : aucun neuf — le rang 2 bis lit, sans verrou, les lignes du groupe, les exercices et la borne,
  après les verrous que chaque geste prend déjà (tolérance de D4).
- **Tests existants touchés** (relevés sur `056997b0` par `git grep -n "match motif\|match attendu\|=> false"
  crates/kesh-db/tests crates/kesh-api/tests`, non exécutés ; validation P1, R-3) :
  - **qui cessent de compiler** sans bras neuf : `invoice_settlement.rs::ecriture_attendue` (`match` exhaustif,
    sans `_`) — dans la liste des tests ci-dessus ;
  - **qui compilent mais ne voient pas le rang** : `supplier_invoices_repository.rs:1638` et `:1911` (`_ =>
    false`) — étendus (AC2 c) ;
  - **à listes figées** (Vitest) : AC7 (e) ;
  - **inchangés** : `filet_bilan_clos.rs::predicteur_muet_sous_un_exercice_futur_clos` (écriture manuelle, aucun
    groupe `document`) ; `errors.rs:4283` (`LETTERING_IS_DOCUMENT`, clé seule) ; les tests du mode `Manual` de
    `letterings.rs` (AC1).
- **Règle de découpage** — compté aux **deux** grains, comme la 15-1a-i et la 15-1a-ii (C-15-1a2-21) :
  - au grain « **crates Rust, packages npm** » : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend` = **4** —
    seuil (« plus de 5 ») non franchi ;
  - au grain des **modules métier de premier niveau** (patron des exemples de la règle,
    `kesh-api/routes/invoices`) : `kesh-db/repositories/letterings` (règle, gel), `…/settlement_cancellation`
    (rang), `kesh-db/errors` (variante, doc-comments), `…/invoice_settlements_write`, `…/reconciliation_cancel`,
    `…/supplier_invoices` (un bras de filtre chacun, doc-comments), `kesh-api/errors` (trois bras de texte, un
    repli), `kesh-i18n`, `frontend/shared/utils`, `frontend/features/reconciliation`,
    `frontend/features/supplier-invoices` — **onze**, dont **sept mécaniques** (un bras d'un `match` ou d'un
    motif fermé, que le compilateur ou `never` énumèrent). Les fichiers de tests ne sont pas des modules ; la
    documentation n'est plus touchée (C-15-1a2-24).
  Le seuil est franchi au second grain ; **signal déclaré** à l'orchestrateur, dérogation écrite en fin de
  fiche (C-15-1a2-23). La remédiation P1 n'ajoute **aucun** module : elle en retire la documentation et
  déplace des tests vers des binaires existants.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`), en autonomie, worktree `kesh-15-1a2-0`, base `f9b6b199`.

### Completion Notes List

**T0 — relevés au sol (sur `f9b6b199`, re-grepés par le nom).** `settlement_entry_cancel_blocker`
(`settlement_cancellation.rs`), les quatre filtres de D3, les trois tables de textes
(`SettlementNotCancellable`, `reconciliation_cancel_blocked_text`, `supplier_invoice_cancel_blocked_text`),
`RANGS` / `monter` / `ecriture_attendue` (`invoice_settlement.rs`), `tail_motives_through_the_supplier_path`
et `invoice_cancel_motives_and_their_precedence`, la `TABLE` de G9 et `CLES_569` de G8 : présents, conformes à
la fiche. Routes des quatre gestes `Rejouee` (`audit_route_registry.rs:270`, `:292`, `:293`, `:313`). Phrase
du lot `generated` : `supplier_invoices.rs:1227-1229` (« une facture `paid` ne peut pas / être dans un lot
`generated` »). Gardes Vitest à valeur épinglée (`grep -nE "toBe\([0-9]{2,}\)|const [A-Z_]+ = [0-9]{2,}"
lib/shared/*.test.ts`) : `CLES_RELEVEES` et `CANDIDATES_ATTENDUES` seules — la liste de l'AC7 (c) est close
(`sitesTotal` est un champ d'objet, `REPLIS_A_SITE_UNIQUE` une liste). Aucune ancre n'avait bougé.

**T1 — la règle des périodes (D1).** `OpenPeriodRule`, `open_period_rule`, `line_in_open_period`
(`Result`, `Invariant` sur un exercice non nommé — ou d'une autre société, la lecture filtrant par
`company_id`), `lines_in_open_period` (aucune ligne → `false`) ; le prédicat **par ligne**
`line_open_in_period(&FiscalYearState, borne, date)` est le seul texte de la règle, appelé par
`any_line_in_open_period` (mode `Manual`, inchangé dans son comportement : sa recherche `is_some_and` est
gardée) et par `OpenPeriodRule`. `books_locked_through` prend désormais une `&mut MySqlConnection` (les deux
appelants `Manual` passent leur transaction par coercition). Tests R7 de `letterings.rs` verts **sans
modification**.

**T2 — le rang 2 bis (D2).** Variante `DocumentLetteringInClosedPeriods` entre `FiscalYearClosed` et
`MatchedBankTransaction`, code réemployé `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` ;
`letterings::document_group_frozen_by_periods` (clés `document` de l'écriture par ordre croissant, puis
`lines_in_open_period` sur chaque groupe) ; évaluation entre les rangs 2 et 3 de la file.

*Sites nommés de D2 (la définition) — chacun relu, trié :*

| site (f9b6b199) | verdict |
|---|---|
| `settlement_cancellation.rs:28` « Les rangs 2 à 5 » | réécrit « dont le 2 bis » + § du rang 2 bis (tolérance, dormance) |
| `invoice_settlements_write.rs:838` « Rangs 1-2 : refusés ici » | réécrit « Rangs 1, 1 bis, 2 et 2 bis » |
| `supplier_invoices.rs:1061` « Têtes et exercice clos » | réécrit (+ lettrage figé) |
| `supplier_invoices.rs:1267` « Rang 1 et « exercice clos » » | réécrit (+ lettrage figé) |
| `reconciliation_cancel.rs:322` « (4) Les motifs — forme EXEMPTÉE » | n'énumère rien : gardé, complété (motif lié, avant le lien défait) |
| `invoice_settlements_write.rs:687-688` « rangs 2 à 5 » | réécrit « dont le 2 bis » |
| `invoice_settlements_write.rs:772-773` « rangs 1 et 2 » | réécrit « 1, 1 bis, 2 et 2 bis » (le 1 bis manquait déjà) |
| `reconciliation_cancel.rs:263-265` « rangs 0 et 2 » | réécrit « 0, 2 et 2 bis » |
| `supplier_invoices.rs:954` « rangs 2 à 5 » | réécrit « dont le 2 bis » |
| `kesh-db/src/errors.rs:318` « les rangs 2 à 5 » | réécrit « dont le 2 bis » |
| `kesh-db/src/errors.rs:329-330` « rangs 1 et 2 » | réécrit « 1, 1 bis, 2 et 2 bis » |
| `kesh-db/src/errors.rs:340` « les deux têtes et l'exercice clos » | réécrit (+ lettrage figé) |
| `kesh-db/src/errors.rs:401-404` codes réemployés | `LETTERING_ALL_LINES_IN_CLOSED_PERIODS` ajouté |
| `kesh-db/src/errors.rs:868-870` (`SettlementNotCancellable`) | réécrit (liste complète des rangs du geste) ; et `ReconciliationNotCancellable`, `SupplierInvoiceNotCancellable` de même |
| `kesh-api/src/errors.rs:3257-3258` « facture créditée, exercice clos » | réécrit (liste complète) |
| `kesh-api/src/errors.rs:3308` « rang 0 … rang 2 » | réécrit (+ rang 2 bis) |
| `kesh-api/src/errors.rs:3316-3318` | réécrit (+ lettrage figé) |
| `settlement-cancel-blocked.ts:4-6` « Ces quatre motifs » | « cinq », parenthèse nomme le lettrage figé |

*Sorties du grep de contrôle non nommées* (`git grep -niE "rangs? [0-9]+…"` sur `crates/kesh-db/src`,
`crates/kesh-api/src`, `frontend/src`) — triées **sans objet** : `kesh-api/src/errors.rs:2968` (rang 2 d'une
autre file, le lettrage manuel) ; `routes/reconciliation.rs:4189`, `:4191` et `reconciliation.types.ts:153`,
`:155` (rangs 3 et 4 nommés par leur champ, sans borne de queue) ; `kesh-db/src/errors.rs:346`, `:364`,
`:1130-1181` (rangs propres d'une tête ou du lettrage manuel) ; `invoice_settlements_write.rs:704-706`, `:732`,
`:743`, `:814`, `:823`, `:876` ; `reconciliation_cancel.rs:8`, `:131-135`, `:245-260`, `:274` ;
`settlement_cancellation.rs:24-25`, `:45`, `:55`, `:71-78` ; `supplier_invoices.rs:465`, `:957`, `:1170`,
`:1194`, `:1254` ; `letterings.rs` (rangs 3 à 6 de `reversal_blockers`) ; `reconciliation_rules.rs:181` —
chacun parle d'un rang existant sans borner la queue, ou d'une autre file.

**T3 — les quatre filtres (D3)**, motifs liés (`blocker @`) ; le dé-rapprochement ne code plus
`FiscalYearClosed` en dur et refuse **avant** de défaire le lien (étape 4).

**T4 — textes et écran.** Les douze textes de D2 **tels quels** dans les quatre `.ftl`, trois replis Rust (un
par table), mot pour mot le fr-CH ; G8 : constante sœur `CLES_2_BIS: [&str; 3]`, chaînée, `CLES_569` inchangée ;
G9 : quatre clés à la `TABLE`, un site chacune. Frontend : `SettlementCancelTailCode`, `MOTIFS` et type du
dé-rapprochement, `SupplierInvoiceCancelCode` + leurs `case` (repli = fr-CH). Décomptes en dur (AC7 d) :
« sept » → « huit » (`reconciliation.types.ts`, `reconciliation-cancel.ts` ×2, `reconciliation-cancel.test.ts`),
« septième » → « neuvième », « des six » → « des huit », « Ces quatre » → « Ces cinq » ;
`blocker-messages.ts:9` inchangé ; les autres sorties du grep (`admin-backup.api.ts:13`,
`duplicate-probe.test.ts:240`, `i18n-harvest.test.ts:160`, `i18n-keys.test.ts:259`, `:266`, `:470`,
`i18n-literal-reader.test.ts:71`) sont historiques ou sans rapport. Gardes recomptées (AC7 c) :
`CLES_RELEVEES` **213 → 214** (clé nommée au commentaire), `sitesTotal` **1920 → 1923** — recompté par
`grep -o "i18nMsg("` aux deux bornes : `settlement-cancel-blocked.ts` 4 → 5, `reconciliation-cancel.ts` 7 → 8,
`invoice-cancel.ts` 6 → 7 —, `REPLIS_A_SITE_UNIQUE` +3 (« treize »). `CANDIDATES_ATTENDUES = 47` inchangée.
`lint-i18n-ownership` vert, `npm run check` 0 erreur.

**T5 — `LETTERING_IS_DOCUMENT`** neutre dans les quatre `.ftl` et le repli Rust (textes de D5) ; G9 le
compare. `git grep -nE "annulez le règlement plutôt|Stornieren Sie die Zahlung, statt|annullate il pagamento
invece|cancel the settlement rather than" -- crates frontend/src` : **aucune sortie**. Doc-comment
d'`InvoiceCredited` réécrit (C-15-1a2-7) ; celui de `DbError::LetteringIsDocument` aussi, qui disait encore
« annuler le règlement, pas délettrer ».

**T6 — propagation de la valeur (AC8).** `git grep -nF "LETTERING_ALL_LINES_IN_CLOSED_PERIODS" --
crates/kesh-api/src crates/kesh-db/src frontend/src` : les **trois** sites d'origine (`kesh-db/src/errors.rs`
`code()` de `DbError::LetteringAllLinesInClosedPeriods` ; `kesh-api/src/errors.rs:3007`, réponse du lettrage
manuel ; `:4270`, son test) ; sites **neufs** : `kesh-db/src/errors.rs` — `SettlementCancelBlocker::code()`,
doc de la variante, doc de `code()`, doc de `DbError::LetteringAllLinesInClosedPeriods` (« texte différent
selon la route ») — ; `kesh-api/src/errors.rs` — le test `closed_lettering_texts_follow_their_family` — ;
frontend — `settlement-cancel-blocked.ts`, `reconciliation-cancel.ts`, `reconciliation.types.ts`,
`invoice-cancel.ts` et leurs tests. Chacun est le code du rang 2 bis ou celui du lettrage manuel : aucun résidu.

**Choix consignés** : **C-15-1a2-0-1** (l'aide de montage `tests/support/document_group.rs`, incluse par
`#[path]` dans six binaires — aucun binaire neuf) ; **C-15-1a2-0-2** (`monter` solde par un **second
règlement du reste**, 60.00 à `D − 10`, le règlement examiné restant celui de 40.00).

**Tests** (périmètre : `f9b6b199` → commit de développement) — **12 fonctions Rust neuves** (`letterings.rs` 2,
`invoice_settlement.rs` 4, `invoice_write_off.rs` 1, `supplier_invoices_repository.rs` 1, `kesh-api/src/errors.rs`
1, trois e2e `kesh-api` 1 chacun), recomptées par `git diff f9b6b199 -- crates | grep -cE '^\+\s*#\[(sqlx::test|tokio::test|test)'`
= 12 ; **5 tests Rust modifiés** et **4 aides** comme prévu ; **Vitest : 1 fichier neuf** (`invoice-cancel.test.ts`,
1 cas) et **2 cas neufs** dans des fichiers modifiés (`settlement-cancel-blocked.test.ts` : texte propre du 2 bis ;
`reconciliation-cancel.test.ts` : le motif reconnu et traduit dans sa famille) — au-delà des 13 prévus ; **7
fichiers Vitest modifiés**. **Mutations** (exécutées, restaurées, fichier touché) : retirer le 2 bis du filtre du
dé-rapprochement → `unreconcile_refuses_in_its_family_before_unlinking` rouge ; borne `>=` → AC1 et
`lettering_at_the_lock_boundary` rouges ; neutraliser l'évaluation du 2 bis dans la file → **12** tests rouges
(neuf des douze tests neufs — tous sauf les deux d'AC1 et celui des textes, qui ne passent pas par la file — , la matrice `RANGS` et les deux bancs fournisseurs : 9 + 1 + 2).

**Contrôle « aucune documentation publique »** : `git diff --stat f9b6b199 -- docs CHANGELOG.md` → vide.

**Gates au commit de développement** : gate ciblé (`fmt --check` vert, `clippy --workspace --all-targets -D
warnings` vert, nextest des binaires touchés : `invoice_settlement` 27/27, `letterings` 33/33,
`invoice_write_off` 23/23, `supplier_invoices_repository` 52/52, `kesh-i18n` 38/38, kesh-api ciblé 144/144 ;
Vitest ciblé 42 fichiers / 411 tests) — **gate complet au dernier commit de code**.

### File List

- `crates/kesh-db/src/repositories/letterings.rs` — D1 (`OpenPeriodRule`, prédicat partagé), `document_group_frozen_by_periods`
- `crates/kesh-db/src/repositories/settlement_cancellation.rs` — rang 2 bis
- `crates/kesh-db/src/errors.rs` — variante, `code()`, doc-comments
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`, `reconciliation_cancel.rs`, `supplier_invoices.rs` — filtres D3, doc-comments
- `crates/kesh-api/src/errors.rs` — trois bras, repli `LETTERING_IS_DOCUMENT`, test
- `crates/kesh-i18n/locales/{fr,de,it,en}-CH/messages.ftl` — trois clés, message neutre
- `crates/kesh-i18n/src/loader.rs` — `CLES_2_BIS`
- `crates/kesh-api/tests/textes_coherents.rs` — `TABLE` de G9
- `crates/kesh-db/tests/support/document_group.rs` — **neuf** (aide de montage, C-15-1a2-0-1)
- `crates/kesh-db/tests/{letterings,invoice_settlement,invoice_write_off,supplier_invoices_repository}.rs`
- `crates/kesh-api/tests/{invoice_echeancier_e2e,reconciliation_e2e,supplier_settlement_cancel_e2e}.rs`
- `frontend/src/lib/shared/utils/settlement-cancel-blocked.ts` (+ `.test.ts`)
- `frontend/src/lib/features/reconciliation/reconciliation-cancel.ts`, `reconciliation.types.ts` (+ `reconciliation-cancel.test.ts`)
- `frontend/src/lib/features/supplier-invoices/invoice-cancel.ts`, `invoice-cancel.test.ts` (**neuf**), `settlement-cancel.test.ts`
- `frontend/src/lib/features/invoices/InvoiceSettlements.test.ts`
- `frontend/src/lib/shared/{i18n-keys,i18n-repli-divergent-actif,i18n-un-repli-par-cle}.test.ts`
- `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md`, `sprint-status.yaml`

## Change Log

### Développement — 2026-10-09 (Opus 5.5, `bmad-dev-story`, en autonomie)

T0–T6 livrées (T7 déplacée à la 15-1a2-i). Rang 2 bis dans la file commune et les quatre gestes, règle des
périodes publique, douze textes et message neutre, écran, gardes recomptées ; dormant (aucun chemin de
production ne pose de groupe `document`). Choix **C-15-1a2-0-1**, **C-15-1a2-0-2**. Détail et décomptes au Dev
Agent Record. Prochaine étape : `bmad-code-review` P1.

### Validation P2 — 2026-10-09 (Opus 5.5 ×2, lentilles R et F ; remédiation Sonnet 5.5, en autonomie) — **VALIDATION CLOSE**

**Rapports** : `kesh-gate-logs/15-1a2-0-validate-p2-R.md` (**0 CRITICAL, 0 HIGH, 1 MEDIUM, 6 LOW**) et `…-F.md`
(**0 CRITICAL, 0 HIGH, 0 MEDIUM, 6 LOW**). Recoupements : R2-2 = F2-2, R2-3 = F2-1, R2-5 = F2-3, R2-6 = F2-4 (R2-7 est voisin de F2-1 :
`kesh-db/src/errors.rs:401-404`, que F ne nomme pas, est compté à part) → **1 MEDIUM** (R2-1) et **8 LOW
distincts** (6 + 6 − 4 recoupements : R2-2, R2-3, R2-4, R2-5, R2-6, R2-7, F2-5, F2-6), soit **9 findings**
(autant de lignes au tableau). **Trend** : P1 R 4 / F 3 MEDIUM → **P2 R 1 / F 0 MEDIUM** ; aucun CRITICAL/HIGH
aux deux passes. Le MEDIUM de la P2 **n'est pas né de la remédiation P1** : omission d'origine (la clé
fournisseur existait avant), **reconduite** par un recompte de P1 (« 6 fichiers Vitest »). Six des huit LOW
naissent de la remédiation P1 ou de ses textes (R2-2, R2-3, R2-4, R2-5 — F le dit né de F-9 —, R2-6, F2-6) ; R2-7
et F2-5 lui sont antérieurs. Signal D5 :
sévérité en baisse, défauts locaux, aucun module gagné — sans objet. Chaque finding relu au code
(`grep -nF` / `sed -n` sur `056997b0` ; `git diff --stat 056997b0 HEAD -- crates frontend docs` : vide).

| finding | sév. | verdict | où |
|---|---|---|---|
| R2-1 — la garde `CLES_RELEVEES = 213` (`i18n-un-repli-par-cle.test.ts:125`, `:207`) rougit avec la clé neuve `supplier-invoices-cancel-blocked-lettering-closed` ; absente de l'inventaire | MEDIUM | **corrigé** : 213 → 214, clé nommée au commentaire ; AC7 (c) à trois gardes, T4 et la liste des tests la nomment ; **7** fichiers Vitest ; autres gardes grepées (`sitesTotal` +3 attendu, `REPLIS_A_SITE_UNIQUE` +3, `CANDIDATES_ATTENDUES` inchangée) | AC7 (c), T4, tests |
| R2-2 = F2-2 — rang 6 × 2 bis « à trancher au T0 » | LOW | **corrigé** : **inatteignable** (`supplier_invoices.rs:1224-1228`), nommé au test, retiré du T0 | AC2 (c), T0, tests |
| R2-3 = F2-1 — le grep des commentaires à rangs rate les trois commentaires des filtres et les énumérations en prose | LOW | **corrigé** : liste de **sites nommés** (dont `invoice_settlements_write.rs:838`, `supplier_invoices.rs:1061`, `:1267`, `kesh-api/src/errors.rs:3257-3258`, `:3316-3318`, `kesh-db/src/errors.rs:340`, `settlement-cancel-blocked.ts:4-6`) ; le grep devient un contrôle | D2, T2 |
| R2-4 — extrait « en commençant par le plus récent » non discriminant (déjà porté par `FISCAL_YEAR_CLOSED`) | LOW | **corrigé** : extrait propre « toutes les lignes de ce lettrage » (`InvoiceSettlements.test.ts`, `reconciliation-cancel.test.ts`) | AC7 (e) |
| R2-5 = F2-3 — `lock_books` / `unlock_books` refusent le jour ; dates passées non prescrites | LOW | **corrigé** : écritures datées strictement avant aujourd'hui (calendrier de `monter`), repli SQL du verrou **retiré** | D4 (1, 3) |
| R2-6 = F2-4 — antécédent de « celui-ci » ambigu | LOW | **corrigé** : ordre des causes inversé dans les **douze** textes (C-15-1a2-30) | D2 |
| R2-7 — `SettlementCancelBlocker::code()` (`kesh-db/src/errors.rs:401-404`) énumère les codes réemployés | LOW | **corrigé** : site nommé à la liste de D2/T2 | D2 |
| F2-5 — la tolérance de D4 ne nomme pas la clôture d'un exercice **postérieur** | LOW | **corrigé** | D4 |
| F2-6 — trois clés neuves dans `CLES_569`, « Les six clés de #569 » | LOW | **corrigé** : constante sœur `CLES_2_BIS: [&str; 3]` chaînée (C-15-1a2-30) | AC7 (a), T4, tests |

**Les douze textes** relus contre `loader.rs` (G8 : `rouvr`/`réouv` au domaine fr-CH, `MARQUEURS` ; G8-bis :
`JUSQU_A`) et `textes_coherents.rs` (G9 : littéral exact, apostrophes droites, aucun `"`) après l'inversion :
marqueur d'ordre et borne présents dans chacun, verbe « rouvrir » conservé, aucune chaîne de `perime`.

**Propagation** (valeurs grepées sur `_bmad-output`, `docs`, `crates`, `frontend/src`) : « clôturé ou suivi d'un
exercice clôturé » (les douze textes réécrits ; les deux proses de D2 et AC1 le disent comme une description, non
comme un texte rendu ; la 15-1a2-i `:603`, `:622` le dit pour le CHANGELOG, sans « celui-ci » — hors du défaut),
`CLES_569` (fiche : trois sites rendus à la constante sœur), `213`, « 6 fichiers Vitest » / « six fichiers »,
« montable » / « à trancher » / « plus le rang 6 ». **Recompte** (depuis ce fichier) : **8 critères actifs**
(AC1–AC7, AC9), **7 tâches** (T0–T6), **13 tests neufs** (12 Rust, 1 Vitest), **5 tests Rust modifiés**,
**7 fichiers Vitest modifiés** (+1), 4 aides de montage ; modules **4** / **11** (inchangé). Choix consigné :
**C-15-1a2-30**. **Aucun code, aucun fichier hors fiche, registre et `sprint-status.yaml`.** La remédiation ne
touche aucune ligne de production : **validation close**, prochaine étape `bmad-dev-story`.

### Validation P1 — 2026-10-09 (Sonnet 5.5 ×2, lentilles R et F ; remédiation Opus 5.5, seul remédiateur des fiches de la suite du lettrage, en autonomie)

**Rapports** : `kesh-gate-logs/15-1a2-0-validate-p1-R.md` (**0 CRITICAL, 0 HIGH, 4 MEDIUM, 8 LOW**) et `…-F.md`
(**0 CRITICAL, 0 HIGH, 3 MEDIUM, 8 LOW**). Recoupements : R-3 = F-3 (le `match` exhaustif ; F-3 porte aussi la
moitié de R-4), R-9 = F-8, R-10 = F-6, R-8 ⊂ F-9 (a) → **6 MEDIUM distincts** (R-1, R-2, R-3 = F-3, R-4,
F-1, F-2) et **13 LOW distincts** (8 + 8 − 3 ; R-11 recoupe F-7 et F-10 sans les égaler, compté à part). Première passe de cette fiche : pas de trend (la fiche est
née de la remédiation P3 de la 15-1a2-i). Signal D5 sans objet en première passe. Chaque finding relu au code
(`grep -nF` / `sed -n` sur `056997b0`) ; un fait de rapport **réfuté en partie** (F-11 : « le mode `Manual`
traite ce cas en `Invariant` » — faux, `any_line_in_open_period` rend `false` par `is_some_and`,
`letterings.rs:499-504` ; la correction proposée est retenue, l'argument est réécrit).

| finding | sévérité | verdict | où |
|---|---|---|---|
| R-1 — le texte proposé rougit G8-bis (aucune borne) ; « jusqu'à celui-ci » sans antécédent | MEDIUM | **corrigé** : douze textes fixés (trois familles × quatre locales), marqueur d'ordre **et** borne « jusqu'à celui-ci » avec antécédent (« son exercice ») ; vérifié contre `loader.rs` `MARQUEURS`, `POSTERIEURS`, `JUSQU_A` ; clés nommées à `CLES_569` (C-15-1a2-26) | Modèle réel, D2, AC7 (a), T4 |
| R-2 — « mot pour mot » des replis Rust sans G9 | MEDIUM | **corrigé** : trois clés + `error-lettering-is-document` à la `TABLE` de G9 (vérifié `textes_coherents.rs:667-690`) ; le test neuf se limite aux clés et codes | D2, D5, AC7 (a), AC9, tests |
| R-3 = F-3 — `ecriture_attendue` (`match` exhaustif) cesse de compiler ; absent de l'inventaire | MEDIUM | **corrigé** : nommé, bras neuf ; « tests existants touchés » relevés par commande ; recompte | AC2 (c), tests, Dev Notes |
| R-4 (+ F-3) — précédence éprouvée contre les rangs 2 et 3 seulement ; matrice parallèle | MEDIUM | **corrigé** : `RANGS` à six (20 cas, paire 1 × 2 bis nommée exclue — inatteignable), 1 bis × 2 bis dans `invoice_write_off.rs`, +4 cas à chacun des deux bancs fournisseurs, tête `SupplierInvoiceNotPaid` et rang 6 nommés (C-15-1a2-27) | AC2 (c), tests, T0 |
| F-1 — l'encadré `user-manual.tex:588-594` et la note `:626-631` promettent la contre-passation que le refus interdit | MEDIUM | **porté à la 15-1a2-i** (AC18, avec le PDF aplati) — décision de l'orchestrateur | 15-1a2-i AC18 |
| F-2 — documenter un comportement dormant contre un manuel qui dit « Kesh ne lettre pas » | MEDIUM | **corrigé par décision de l'orchestrateur** : toute la documentation publique du refus passe à la 15-1a2-i (C-15-1a2-24) ; AC8 et T7 déplacés, numéros non réattribués | Status, AC8, T7, Dev Notes |
| R-5 — fixtures non spécifiées | LOW | **corrigé** : chaque test dans le binaire qui porte son montage, aucun fichier neuf (C-15-1a2-27) | tests |
| R-6 — décomptes « septième », « six », « quatre motifs », `errors.rs:318` | LOW | **corrigé** (grep de la valeur, quinze sorties triées) | AC7 (d), D2 |
| R-7 — noms de tests divergents avec la 15-1a2-i | LOW | **corrigé** dans la 15-1a2-i (renvoi aux noms d'ici) | 15-1a2-i tests |
| R-8 ⊂ F-9 — règlement et solde sur une même facture | LOW | **corrigé** : deux factures (D4 point 4) | D4, AC3 |
| R-9 = F-8 — trois sites Rust, non deux | LOW | **corrigé** (`kesh-db/src/errors.rs:1356` ajouté ; docs à la 15-1a2-i) | AC8 |
| R-10 = F-6 — `api-external.md:324` hors de tout grep | LOW | **porté** : propriétaire unique la 15-1a2-i (AC12) | AC9, 15-1a2-i AC12 |
| R-11 — « dernier règlement » n'est pas toujours la ligne la plus récente ; « paiement » côté fournisseur | LOW | **corrigé** : remède sur la date la plus récente du lettrage, « en général celle du dernier règlement / du paiement » | D2 |
| R-12 — lignes citées ; contrôle PDF ; dé-rapprochement `Entry` | LOW | **corrigé** (`:3763`, `:3814`, `:59` ; PDF à la 15-1a2-i ; exception héritée de `reconciliation_cancel.rs:26-31` nommée) | Modèle réel, D3 |
| F-4 — trois tests Vitest à listes figées | LOW | **corrigé** | AC7 (e), tests |
| F-5 — doc-comments à numéros de rang | LOW | **corrigé** (six fichiers, grep de la valeur) | D2, T2 |
| F-7 — « ne lève rien » inexact au bord | LOW | **corrigé** (ligne la plus récente, vraie sans réserve) | D2, AC3 |
| F-9 — montages : verrou après les écritures, assertion de montage | LOW | **corrigé** (D4, quatre points) | D4, AC2 |
| F-10 — textes des deux autres familles ; « son avoir » au fournisseur | LOW | **corrigé** (textes fixés ; message `LETTERING_IS_DOCUMENT` sans « son avoir ») | D2, D5 |
| F-11 — `line_in_open_period` sur exercice inconnu → `false` muet | LOW | **corrigé** : `Result`, `Invariant` (C-15-1a2-25) ; argument du rapport réfuté en partie (ci-dessus) | D1, AC1, tests |

**Remarque de la validation P4 de la 15-1a2-i (lentille F)** : le CHANGELOG prescrit « fait reculer le verrou
**ou** rouvre l'exercice » contredisait D2 — **corrigé** par le déplacement (15-1a2-i AC18 écrit « et/ou, selon
la cause ») et par les textes de D2, qui énoncent les deux conditions reliées par « et ».

**Propagation** (valeurs grepées sur les fiches 15-1a2-0, -i, -ii, 15-1b, 15-1b-0, l'index et le registre) :
`lettering_closed_period_refusal`, `rank_2_bis_precedence`, « 15-1a2-0 AC8 », « 15-1a2-0, AC8 »,
« avant la date du dernier règlement », « ou rouvre l'exercice », « son avoir » — résidus : Change Logs
(historique) et prompts versionnés. **Recompte** (depuis ce fichier) : **8 critères actifs** (AC1–AC7, AC9 ;
AC8 déplacé, numéro non réattribué), **7 tâches** (T0–T6 ; T7 déplacée), **13 tests neufs** (12 Rust, 1
Vitest), **5 tests Rust et 6 fichiers Vitest modifiés**, 4 aides de montage. Modules : **4** crates/paquets,
**11** modules métier (inchangé ; signal déjà déclaré, C-15-1a2-23). Choix consignés : **C-15-1a2-24 à 27**.
Prochaine passe : **P2, complète, Opus** — la remédiation fixe douze textes, change une signature
(`line_in_open_period`) et redistribue les tests ; une passe ciblée ne suffirait pas.

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

Au grain fin, la fiche dépasse cinq modules ; au grain des crates et paquets — celui que la règle a toujours appliqué dans cet epic —, elle est sous le seuil. Le dépassement ne vient que de la propagation mécanique de textes (catalogues ×4, libellés d'écran ; depuis la validation P1, manuels, PDF, `api-external.md` et CHANGELOG sont à la 15-1a2-i, C-15-1a2-24), qui ne porte aucune règle. Décision de l'orchestrateur : pas de découpage (registre **C-15-1a2-23**, alternatives et réversibilité). Accepted risk : une passe de revue doit relire la propagation des textes comme un axe à part entière.
