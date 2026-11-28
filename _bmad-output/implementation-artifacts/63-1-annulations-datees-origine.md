# Story 63-1 : Les annulations depuis la fiche contre-passent dans l'exercice de l'écriture, à sa date (#1752, #1753, #1754)

Status: in-progress  <!-- développement commencé le 2026-11-28 ;  validation terminée le 2026-11-27 : P1 (Sonnet) 0/0/2/6, P2 (Opus) 0/0/1/6, P3 (Sonnet, complète) 0/0/0/5 ; aucune passe de plus -->

*Fiche rédigée le 2026-11-27 (Opus 5.5, `bmad-create-story`), worktree `kesh-63-1`, branche `story/63-1-annulations-datees-origine`, partie
de `story/53-6-annulation-rapprochement-exercice` à `6b71d4c21` (la 53-6a en développement). Epic 63 « Annulations datées de l'origine »
(`_bmad-output/planning-artifacts/epic-63-annulations-datees-origine.md`), **story de correction de la release v0.21.0** (décision du Project
Lead du 2026-11-27), seule story de l'epic. Trois issues **P1** : #1752, #1753, #1754. **Aucune migration.** ⚠️ **Dépend de la 53-6a**
(#1674) : la story réemploie le mécanisme que celle-ci pose (`CancellationDating::OriginDate`, rang `PeriodLocked` de la queue commune). Les
numéros de ligne de cette fiche sont lus à `6b71d4c21` ; **T0 les relit** sur la branche reprise d'`epic/53-ecritures-justes` une fois la
53-6a fusionnée.*

## Story

En tant que comptable qui annule depuis sa fiche un règlement client, un solde, un paiement fournisseur ou une facture fournisseur saisis
par erreur, je veux que Kesh contre-passe l'écriture **dans son propre exercice, à sa propre date**, et refuse clairement — avant le clic —
quand cet exercice est clôturé ou la période verrouillée, afin qu'annuler une pièce après la fin de son exercice ne fausse jamais deux
exercices, ni le résultat et la TVA.

## Les trois défauts (issues, lues le 2026-11-27, lecture seule)

| Issue | Geste | Route → geste (à `6b71d4c21`) | Écriture contre-passée | Ce qui est faux aujourd'hui |
|---|---|---|---|---|
| **#1752** | annuler un **règlement** client, ou un **solde** (escompte, frais, perte, arrondi : `settlement_type = 'write_off'`), depuis la fiche de la facture | `routes/invoices.rs:1611` `cancel_invoice_settlement_handler` → `invoice_settlements_write::cancel_settlement` (`:1249`, `Today(aujourd_hui)` `:1267`) → `cancel_settlement_in_tx` (`:1024`) → `reverse_owned_in_tx(…, ClientSettlement, dating)` (`:1134`) | règlement : banque (ou compte interne) / débiteurs, avec l'écart éventuel ; solde : nature / débiteurs, **et la correction de TVA due** pour l'escompte et la perte | bilan de clôture (banque, débiteurs) de N et N+1 ; pour un solde, **résultat et TVA** de deux périodes |
| **#1753** | annuler le **paiement** d'une facture fournisseur, depuis sa fiche | `routes/supplier_invoices.rs:1056` `cancel_supplier_invoice_settlement` → `supplier_invoices::cancel_settlement` (`:3628`, `Today` `:3642`) → `cancel_settlement_in_tx` (`:3439`) → `reverse_owned_in_tx(…, SupplierSettlement, dating)` (`:3531`) | créanciers / banque (ou compte interne) | bilan de clôture (banque, créanciers) de N et N+1 |
| **#1754** | annuler une **facture fournisseur** validée ou payée | `routes/supplier_invoices.rs:1017` `cancel_supplier_invoice` → `supplier_invoices::cancel` (`:3300`) → `cancel_in_tx` (`:3120`, `Today` `:3221`) → `reverse_owned_in_tx(…, SupplierPurchase, …)` (`:3211`) | charges, **impôt préalable**, créanciers | **résultat et décompte de TVA** de deux exercices — le plus grave des trois |

Le défaut commun (P4 de la 53-6a) : datée du jour, toute annulation faite **après la fin** de l'exercice de l'écriture tombe dans un autre
exercice, que celui-ci ait été clôturé puis rouvert ou **simplement pas encore clôturé** (en janvier, avant la clôture). Une annulation faite
**dans** l'exercice, à une date ultérieure, tombe dans une autre période (TVA trimestrielle). Textes source des issues :
`/home/gcorbaz/devel/kesh-gate-logs/e53/53-6-issues-q3/R7.md`, `R8.md`, `R9.md`.

## Étape 0 — prémisses relues au code (`6b71d4c21`)

**P1 — le mécanisme existe et il suffit d'un argument (confirmée).** La 53-6a a posé `pub enum CancellationDating { Today(NaiveDate),
OriginDate }` (`journal_entries.rs:3192`), converti en `ReversalDating` privé (`:3205`) ; `reverse_in_tx` (`:3095`) et `reverse_owned_in_tx`
(`:3169`) la prennent ; la règle pure `reversal_date_verdict` (`:3278`) et son entrée de crate `cancellation_date_verdict` jugent date,
exercice clos et borne. Les trois gestes de la fiche passent déjà par `reverse_owned_in_tx` : le changement de datation est **un argument par
geste**, sans fonction nouvelle du socle (D2 de la 53-6a : « jamais une seconde contre-passation »).

**P2 — la queue commune annonce déjà `PeriodLocked` sous `OriginDate` (confirmée).** `settlement_cancellation.rs:102`
`settlement_entry_cancel_blocker(…, dating)` : rang 2 (exercice de l'écriture **examinée**), 2 bis (lettrage figé), 3, 4, puis
`OriginDate` → rang `PeriodLocked` (borne en étiquette, seuil inclusif, `:200-226`), `Today` → rang 5 `NoOpenFiscalYearToday` (`:228-237`).
Les trois têtes passent la datation : `invoice_settlements_write::settlement_cancel_blocker(_unlinking)` (`:876`, `:889`),
`supplier_invoices::supplier_settlement_cancel_blocker(_unlinking)` (`:3330`, `:3356`) ; **sauf** `supplier_invoice_cancel_blocker`
(`supplier_invoices.rs:3020`), qui prend `aujourd_hui` et passe `Today` en dur (`:3063`).

**P3 — les lecteurs des motifs passent `Today` (confirmée) : ils doivent passer la même datation que le geste.**
- fiche client : `kesh-api/src/services/factures.rs:569` `reglements`, `:585-591` (`Today(lecture.aujourd_hui())`) — sert
  `GET /api/v1/invoices/{id}/settlements` (`cancellable`, `cancelBlockedBy`, `cancelBlockedLabel`) et le MCP ;
- fiche fournisseur : `supplier_invoices::get_settlement_view_in_tx` (`:3718`, `Today` `:3739`, puis `supplier_invoice_cancel_blocker(…,
  aujourd_hui)` `:3743`), appelée par `get_settlement_view` (`:3690`) et `commit_with_view` (`:3785`, réponse de **chacun des neuf gestes**
  qui écrivent une facture fournisseur, `docs/api-external.md:801`) ; `kesh-api/src/services/factures_fournisseur.rs:206`.
⛔ « Une seule fonction pour lire et pour écrire » (doc de `settlement_cancel_blocker`, `invoice_settlements_write.rs:867-870`) : un lecteur
resté sous `Today` afficherait un bouton que le clic refuse (`PERIOD_LOCKED`), ou un refus (`FISCAL_YEAR_INVALID`) que le clic ne ferait plus.

**P4 — la date du jour n'a plus d'autre usage sur ces chemins (confirmée à la validation P1 pour les routes et les enveloppes ; T0 la relit).** ⚠️ Retirer
`aujourd_hui` de `commit_with_view` le retire **aussi** de **toutes** ses enveloppes, qui ne le font que suivre : `supplier_invoices::validate`
(`supplier_invoices.rs:1478`), `unvalidate` (`:1498`), `unvalidate_with_policy` (`:1521`), `pay` (`:2463`), en plus de `cancel` et `cancel_settlement` ;
côté routes, `update_supplier_invoice` et son `_once` (`routes/supplier_invoices.rs:799`, `:826`), `validate` (`:920`), `unvalidate` (`:956`), `pay` (`:992`),
`cancel` (`:1025`), `cancel_settlement` (`:1064`) et les trois routes de `imported_supplier_invoices.rs` avec leurs `_once` (`:139`/`:173`, `:599`/`:625`,
`:710`/`:734`) : partout la lecture `state.horloge.aujourd_hui()` ne sert qu'à ce transit et tombe (D3). Dans `get_settlement_view_in_tx`,
`aujourd_hui` ne sert qu'aux deux bloqueurs (`:3734-3744`) ; dans `commit_with_view`, qu'à `get_settlement_view_in_tx` (`:3800`) ; dans
`supplier_invoices::cancel` et `cancel_settlement`, qu'à la datation et à `commit_with_view` ; dans `services::factures::reglements`, qu'au
bloqueur (`:582-591`). La route client garde **une** lecture de l'horloge : `InvoiceResponse::from_parts(…, aujourd_hui)` (`invoices.rs:1637`,
l'échéance de la réponse). Les routes `imported_supplier_invoices.rs` (`:139`, `:599`, `:710`) lisent l'horloge et la passent à
`commit_with_view` (`:251`, `:671`, `:762`) : T0 lit si elles s'en servent ailleurs.

**P5 — un exercice d'origine clos est déjà refusé, avant tout (confirmée) ; Q6 ne change rien.** Rang 2 de la queue, refusé par chacun des
trois gestes (listes fermées : `invoice_settlements_write.rs:1103-1113`, `supplier_invoices.rs:3495-3504`, `:3177-3188`), en `409
FISCAL_YEAR_CLOSED`, après le verrou de l'écriture **et de son exercice** (`invoice_settlements_write.rs:1060-1087`, `supplier_invoices.rs:3466-3478`, `:3148-3158`). Un exercice rouvert est
`Open` : l'annulation s'y datera.

**P6 — au clic, `PeriodLocked` est refusé par le socle, non par le geste (confirmée).** Les trois listes fermées ci-dessus n'admettent pas la
variante : le socle (`create_in_tx_inner`, étape 1-bis) rend `400 PERIOD_LOCKED` (`details.lockedThrough`, `details.attempted`,
`kesh-api/src/errors.rs:6494`). C'est le contrat déjà fixé par la 53-6a pour R2 et R3 (D5 : « une seule garde par motif »). Aucun geste
groupé n'est en cause ici (le refus `409` sans `details` des groupes ne concerne que le dé-rapprochement).

**P7 — le lettrage ne change pas de règle, il change d'exercice tenu (confirmée).** Chaque geste dissout le groupe `document` de la pièce
**avant** la contre-passation, en tenant l'exercice de l'écriture verrouillé (`invoice_settlements_write.rs:1123` ;
`supplier_invoices.rs:3518` ; `:3198`) ; la contre-passation lettre ensuite l'origine avec son miroir sous `Mode::System {
held_open_fiscal_year_id: fy.id }`, `fy` étant, sous `OriginDate`, **le même exercice** (D2 de la 53-6a). Facture fournisseur payée
(#1754) : le groupe {achat, règlement} est dissous ; l'achat se lettre avec son inverse ; le règlement **détaché** reste ouvert, à sa date,
inchangé (`supplier_invoices.rs:3085-3089`, `:3192-3196`).

**P8 — le décompte de TVA lit les pièces, non le grand livre (établi au manuel, à confirmer au code à T0).** `user-manual.tex:4736-4752`
(« Le contrôle porte sur les écritures des factures et des soldes retenus par le décompte ») : annuler un escompte ou une perte retire le solde
du décompte de **sa** période (la ligne `invoice_settlements` est supprimée, `invoice_settlements_write.rs:1145-1151`) et, aujourd'hui, sa
contre-passation datée du jour « n'est reprise nulle part ». Datée du solde, l'inverse tombe **dans la même période** : grand livre et
décompte disent alors la même chose, période par période. La période du solde change toujours si elle a été déclarée (P8 de la 53-6a : Kesh
ne tient aucun décompte déposé) : seul le verrou la protège — d'où la phrase TVA des confirmations (D5).

**P9 — le repassage en brouillon d'une facture fournisseur annulée supprime l'achat et sa contre-passation (confirmée ; inchangé).**
`supplier_invoices.rs:1560-1626` (doc d'`unvalidate_in_tx`) : refus de l'exercice clos, de la période verrouillée, etc., **sur l'une et
l'autre écriture**. Datées du même jour, elles tombent sous les mêmes gardes ; le manuel (`user-manual.tex:2489-2494`, « nette à zéro qu'au
cumul … deux périodes ») ne vaut plus que pour les annulations faites avant la v0.21.0.

**P10 — ce qui reste daté du jour après la story (inventaire clos).** `git grep -n "reverse_in_tx(\|reverse_owned_in_tx(\|reverse_at_next_fiscal_year_start_in_tx(" -- 'crates/*/src/*'`
à `6b71d4c21`, hors doc-comments et `#[cfg(test)]` :

| # | Site | Geste | Datation après la story |
|---|---|---|---|
| R1–R6 | `reconciliation_cancel.rs:973` (`reverse_in_tx`) ; `:936`, `:954` (appels de `cancel_settlement_in_tx`, que la commande ci-dessus ne rend pas : ils atteignent `reverse_owned_in_tx` par R7 et R8) ; `grouped_settlements.rs:1844`, `:2033` | dé-rapprochement (53-6a) | origine (inchangé) |
| **R7** | `invoice_settlements_write.rs:1134` par `cancel_settlement` (`:1249`) | règlement **et** solde depuis la fiche (#1752) | **origine** |
| **R8** | `supplier_invoices.rs:3531` par `cancel_settlement` (`:3628`) | paiement fournisseur depuis la fiche (#1753) | **origine** |
| **R9** | `supplier_invoices.rs:3211` (`cancel_in_tx`) | annulation de facture fournisseur (#1754) | **origine** |
| R10 | `journal_entries.rs:3994` (`reverse`) | `POST /journal-entries/{id}/reverse` | **jour, inchangé** — la correction d'une écriture **non possédée** depuis le présent (principe du manuel, `user-manual.tex:854-875`) ; sujet de l'Epic 38 (datation choisie) |
| R11 | `journal_entries.rs:4027` | `/reverse-at-next-fiscal-year-start` | inchangé |

Hors inventaire, vérifiés : les **avoirs** (`credit_notes.rs`) ne contre-passent pas par le socle — ils passent leur propre écriture, datée de
l'avoir (sujet de la 53-2, #525) ; l'annulation d'un **lot de paiement** ne passe aucune écriture (à relire à T0) ; le **repassage en
brouillon** supprime, il ne contre-passe pas (P9). Après la story, la **seule construction** de `CancellationDating::Today(…)` en production est R10 : une garde
lexicale le fige (AC-10). ⚠️ « Construction » : le jeton figure aussi, **sans construire**, à la définition de l'énumération (`journal_entries.rs:3192`),
à la conversion `From<CancellationDating> for ReversalDating` (`:3208`, `CancellationDating::Today(today) => Self::Today(today)`) et au bras du `match`
de la queue (`settlement_cancellation.rs:228`) : ce sont des motifs, conservés (D7).

## Décisions du Project Lead appliquées (Q1 à Q6 de la 53-6a, du 2026-11-27)

La décision de lancer la 63-1 (Project Lead, 2026-11-27) applique aux trois gestes les décisions prises pour le dé-rapprochement :

- **Q1** — la contre-passation prend la **date de l'écriture d'origine**, dans son exercice, **même à l'intérieur d'un exercice**.
- **Q2** — **période verrouillée : refus** (`PERIOD_LOCKED`), **annoncé à l'écran avant le clic** ; ⚠️ **aux notes de version** : une
  annulation acceptée aujourd'hui peut être refusée après la story.
- **Q3** — sans objet ici (c'est elle qui a ouvert #1752, #1753, #1754).
- **Q4** — la **détection** des livres déjà touchés n'est pas dans cette story (voir Q-B ci-dessous pour son étendue).
- **Q5** — **pas d'`/api/v2`** : correction d'un défaut qui écrivait faux, annoncée ⚠️ en tête des notes de version ; le contrat écrit
  (« datée du jour », `docs/api-external.md:688`, `:799`, `:813`) est réécrit ; `b` est déjà porté par la v0.21.0 (53-6a).
- **Q6** — **on ne rouvre pas un exercice clos** : refus `FISCAL_YEAR_CLOSED`, inchangé (P5). Un exercice rouvert est ouvert : l'annulation s'y date.

## Décisions de la fiche

- **D1 — Les trois gestes passent `CancellationDating::OriginDate`.** `invoice_settlements_write::cancel_settlement` (`:1267`),
  `supplier_invoices::cancel_settlement` (`:3642`), `supplier_invoices::cancel_in_tx` (`:3221`). Rien ne change dans `reverse_in_tx_inner`, ni
  dans la règle pure, ni dans `create_in_tx_inner`. Le règlement soldé **avec écart** (trois lignes) se contre-passe entier à sa date (la
  contre-passation recopie les lignes de l'origine, `journal_entries.rs` étape (5)). L'audit `journal_entry.reversed` de ces trois gestes porte
  `"dating": "originDate"` (D2 de la 53-6a) ; ceux des gestes (`invoice.settlement_cancelled`, `supplier_invoice.settlement_cancelled`,
  `supplier_invoice.cancelled`) gardent leur forme. *Écartée* : passer à la route une date calculée (`Today(date_origine)`) — écartée par la
  53-6a (D2, « Écartée 2 ») pour la même raison : la confirmation `fy.id == origin_fy_id` disparaîtrait.
- **D2 — Lire comme on écrit.** Chaque lecteur des motifs (P3) passe `OriginDate` : `services::factures::reglements`,
  `get_settlement_view_in_tx` (les **deux** bloqueurs) ; `supplier_invoice_cancel_blocker` perd `aujourd_hui` et passe `OriginDate` à la
  queue (une seule datation possible pour l'annulation de facture ; *écartée* : lui donner un paramètre `dating` que personne ne ferait
  varier). Conséquences : le rang `PeriodLocked` est **annoncé** par `GET /invoices/{id}/settlements` et par la vue fournisseur
  (`settlementCancelBlockedBy`, `cancelBlockedBy` = `PERIOD_LOCKED`, borne `AAAA-MM-JJ` dans `…BlockedLabel`) ; le rang 5
  (`FISCAL_YEAR_INVALID`) n'y est plus produit.
- **D3 — La date du jour sort de ces chemins** (patron D6 de la 53-6a : « un paramètre inutilisé masque la règle »). Perdent leur paramètre
  `aujourd_hui` : `invoice_settlements_write::cancel_settlement`, `supplier_invoices::{cancel, cancel_in_tx, cancel_settlement,
  supplier_invoice_cancel_blocker, get_settlement_view, get_settlement_view_in_tx, commit_with_view, validate, unvalidate,
unvalidate_with_policy, pay}` (les quatre derniers ne font que passer le paramètre à `commit_with_view`, P4) — **230 sites d'appel dans 35 fichiers de
`crates/*/tests/`** (`git grep -o "supplier_invoices::\(validate\|unvalidate\|unvalidate_with_policy\|pay\|cancel\|cancel_in_tx\|cancel_settlement\|get_settlement_view\|get_settlement_view_in_tx\|commit_with_view\)(\|invoice_settlements_write::cancel_settlement(\|supplier_invoice_cancel_blocker(" -- '*/tests/*' | wc -l`,
à `6b71d4c21`, dont 97 pour `validate|unvalidate|unvalidate_with_policy|pay` et 4 pour `cancel_in_tx` — `bank_accounts_closed_e2e.rs:1734`, `migrations_upgrade_path.rs:1558`, `:1577`,
`supplier_invoices_repository.rs:2259`, que la commande de la P1 omettait (226 / 34) ; aucun import groupé ni alias de ces fonctions dans les tests ; à recompter à T0) changent de signature : un changement mécanique, trouvé par le compilateur — mais **le
changement de datation, lui, change des attendus** (AC-11, « Autres tests qui changent d'attendu ») ; `services::factures::reglements`
  cesse de lire `lecture.aujourd_hui()` ; `services::factures_fournisseur.rs:206` de même. Routes : `cancel_invoice_settlement_handler` garde
  sa lecture de l'horloge pour `InvoiceResponse::from_parts` seulement (commentaire `:1618-1619` réécrit : le bloqueur et la contre-passation
  ne la partagent plus) ; `cancel_supplier_invoice` et `cancel_supplier_invoice_settlement` ne la lisent plus ; tous les autres appelants de
  `commit_with_view` et de ses enveloppes (liste de P4 : routes `supplier_invoices.rs` `update`, `validate`, `unvalidate`, `pay` et leurs
  `_once`, `imported_supplier_invoices.rs:251`, `:671`, `:762` et leurs `_once`) ne la passent plus et ne la lisent plus (P4 : elle ne sert qu'à ce transit). `crates/kesh-api/tests/date_du_jour_garde.rs` se relit (la 53-6a a établi qu'il
  inventorie les dates tirées **hors** de l'horloge, non les lectures de `state.horloge` : à confirmer, non à supposer) ; la garde du périmètre
  « lecture seule » des services (`services/mod.rs:159-187`, `kesh-api/tests/services_rollout.rs`) se relit : la vue lit désormais
  `companies.books_locked_through` (rang `PeriodLocked`) au lieu de `fiscal_years` (rang 5).
- **D4 — Refus, et qui refuse.** Inchangés : rangs 1 et 1 bis (têtes), 2 (`409 FISCAL_YEAR_CLOSED`, Q6), 2 bis (`409
  LETTERING_ALL_LINES_IN_CLOSED_PERIODS`), 3, 4, lot de paiement (rang 6 de l'annulation de facture). Nouveau : **`PeriodLocked`**, annoncé par
  la vue (D2), refusé **au clic par le socle** en **`400 PERIOD_LOCKED`** (`details.lockedThrough`, `details.attempted`), les listes fermées des
  trois gestes ne changeant pas (P6). Disparu : `400 FISCAL_YEAR_INVALID` (« aucun exercice ouvert ne couvre la date du jour ») pour ces trois
  routes. `LATER_FISCAL_YEAR_CLOSED` ne se rencontre plus que dans l'**état hérité** (P6 de la 53-6a : un exercice ouvert n'a jamais de
  postérieur clos). ⚠️ **Précédence du lettrage** (D5 de la 53-6a, M1 de sa P2) : pour une pièce **soldée**, le rang 2 bis précède
  `PeriodLocked` ; l'état qui atteint `PERIOD_LOCKED` dépend du geste :
  - **#1752 règlement** : règlement **partiel** (aucun groupe) — montage le plus simple ; règlement antérieur d'une facture soldée par un
    règlement postérieur à la borne ;
  - **#1752 solde** : un solde **solde** la facture (groupe `document`) et il est en général le dernier geste : une borne qui couvre sa date
    couvre d'ordinaire toutes les lignes du groupe → **rang 2 bis**, non `PERIOD_LOCKED`. `PERIOD_LOCKED` ne s'atteint que dans l'état hérité
    (groupe absent, SQL de test) ; le test d'AC-3 du solde porte sur la **date**, le test de `PERIOD_LOCKED` sur le règlement partiel ;
  - **#1753** : le paiement solde toujours la facture (groupe {achat, paiement}) ; `PERIOD_LOCKED` s'atteint quand la borne couvre le paiement
    mais pas l'achat (paiement **antérieur** à la date de la facture : `pay` l'accepte — lu à la validation P1, `pay_in_tx_as` n'exige qu'un exercice ouvert qui couvre `payment_date`, `supplier_invoices.rs:2686`, et aucune comparaison à `invoice_date` ; montage : valider la facture au `d2`, payer au `d1 < d2`, **puis** poser le verrou jusqu'à `d1` — le poser avant refuserait le paiement lui-même), sinon par l'état hérité (SQL) —
    patron du montage de repli de la 53-6a (R3) ;
  - **#1754** : facture **validée, non payée** (aucun groupe) sous la borne → `PERIOD_LOCKED` d'ordinaire ; facture payée, achat sous la
    borne, paiement après → groupe non figé → `PERIOD_LOCKED`. C'est **le cas courant** : annuler une facture fournisseur d'un trimestre TVA
    verrouillé devient impossible sans faire reculer le verrou (Q-A).
- **D5 — Les textes.**
  - **Motif neuf, à l'écran et au serveur, sans variable** (la borne ajoutée par l'écran en date suisse, patron d'`ACCOUNT_ARCHIVED` et du motif
    `PERIOD_LOCKED` de `reconciliation-cancel.ts`) :
    `settlement-cancel-blocked-period-locked` = « Ce règlement est dans une période verrouillée : pour pouvoir l'annuler, un administrateur
    doit faire reculer le verrou avant sa date. » (queue partagée client/fournisseur, `frontend/src/lib/shared/utils/settlement-cancel-blocked.ts`,
    union `SettlementCancelTailCode`) ;
    `supplier-invoices-cancel-blocked-period-locked` = « L'écriture d'achat de cette facture est dans une période verrouillée : pour pouvoir
    l'annuler, un administrateur doit faire reculer le verrou avant sa date. » (`features/supplier-invoices/invoice-cancel.ts`, union
    `SupplierInvoiceCancelCode`). Quatre locales. Au serveur (`kesh-api/src/errors.rs`), le bras `PeriodLocked` de la famille
    `SettlementNotCancellable` (`:6261`, aujourd'hui délégué au texte du dé-rapprochement) et celui de `supplier_invoice_cancel_blocked_text`
    (`:7051`) prennent ces clés (« les clés sont celles de l'écran », un repli identique au FTL fr-CH) ; ils restent **inatteignables** dans
    ces familles (listes fermées, P6) — le commentaire le dit pour cette seule raison (la raison « sous `Today` la queue ne le produit pas »
    tombe). Le refus du clic (`400 PERIOD_LOCKED` du socle) garde son texte (`journal-entries-period-locked`, borne et date) : T5 vérifie ce que
    les deux fiches en affichent.
  - **Confirmations réécrites** (quatre clés, quatre locales ; « datée d'aujourd'hui » sort) **sans variable** — la date de la pièce est
    affichée à côté du bouton sur chacune des deux fiches :
    `invoices-settlement-cancel-confirm` = « Annuler ce règlement ? Une écriture inverse, datée du règlement, sera passée au grand livre dans
    son exercice, et le montant redeviendra dû. Les totaux de cette période changent : si elle a déjà été déclarée, faites d'abord verrouiller
    la période. » ;
    `invoices-write-off-cancel-confirm` = « Annuler ce solde ? Une écriture inverse, datée du solde, sera passée au grand livre dans son
    exercice, TVA comprise, et le montant redeviendra dû. Les totaux de cette période changent, TVA comprise : si elle a déjà été déclarée,
    faites d'abord verrouiller la période. » ;
    `supplier-invoices-settlement-cancel-confirm` = « Annuler ce règlement ? Une écriture inverse, datée du paiement, sera passée au grand
    livre dans son exercice, et la facture redeviendra validée, à payer. Les totaux de cette période changent : si elle a déjà été déclarée,
    faites d'abord verrouiller la période. » ;
    `supplier-invoices-cancel-confirm` = « Annuler cette facture ? Une écriture inverse de l'achat, datée comme lui, sera passée au grand
    livre dans son exercice. Les totaux de cette période changent, TVA comprise : si elle a déjà été déclarée, faites d'abord verrouiller la
    période. »
    *(Textes proposés : le développeur peut les ajuster, sans rouvrir la règle ; DE, IT, EN traduits sur le même sens.)*
    *Écartée* : la date et le nom de l'exercice en variables (forme de la 53-6a, D4) — il faudrait ajouter `reversalDate` et
    `reversalFiscalYearName` à trois réponses (`GET /invoices/{id}/settlements`, vue fournisseur ×2) pour une information que la fiche montre
    déjà (date du règlement, du paiement, de la facture), et la date du paiement fournisseur n'est pas exposée telle quelle (`paidAt` est un
    horodatage, l'écriture est datée de `payment_date`, `supplier_invoices.rs:2728`).
  - **Inatteignables, conservés** : `settlement-cancel-blocked-no-fiscal-year`, `supplier-invoices-cancel-blocked-no-fiscal-year` (et leurs
    bras `FISCAL_YEAR_INVALID` des unions TS), comme la 53-6a a conservé `reconciliation-cancel-blocked-no-fiscal-year` : la variante reste
    (D7). Leur commentaire dit qu'elles ne sont plus produites par la fiche.
- **D6 — Ce qui distingue les trois gestes, tranché.**
  - **#1752, règlement** (lu à la validation P1 : `entry_date: settled_on` aux deux créations) : inverse datée du règlement (`settled_on`, qui est la date de son écriture par construction :
    `invoice_settlements_write.rs:370`, `:751`) ; ligne retirée, `paid_at` reprojeté, lettrage dissous puis `reversal` — inchangés.
  - **#1752, solde** (même route, `settlement_type = 'write_off'`) : inverse datée du solde ; la correction de TVA de l'escompte ou de la
    perte est contre-passée **dans la période du solde**, où le décompte (fondé sur les pièces, P8) ne retient plus le solde : grand livre et
    décompte concordent désormais dans cette période. Le rang 1 bis (« annulez d'abord le solde ») ne change pas : annuler le solde puis le
    règlement les contre-passe chacun à **sa** date. ⚠️ **Recoupement 53-2** (#1198, la correction de TVA de l'escompte et de la perte prend le
    compte de la vente) : la contre-passation recopie les lignes de l'origine, quel que soit le compte choisi à la création ; fonctions
    distinctes du même fichier.
  - **#1753** : inverse datée du paiement (`payment_date`, `supplier_invoices.rs:2728`) ; la facture redevient `open` ; le lot de paiement
    confirmé qui l'a réglée n'est pas modifié ; l'avertissement du lot (`supplier-invoices-settlement-cancel-batch-warning`) reste.
  - **#1754** : inverse de l'achat datée de l'écriture d'achat (`entry_date: invoice_date` à la validation, `supplier_invoices.rs:2308-2315`, **seule** création d'une écriture d'achat : lu à la validation P1,
    `:1831` n'est pas une création mais l'indice d'un « paiement détaché » (`Hint { entry_date: Some(entry_date) }`), et `:2720` crée le règlement) ;
    charge et impôt préalable reviennent à zéro **dans la période de l'achat** ; la facture passe `cancelled` (inchangé : elle n'est **pas**
    remise à `open`, l'annulation reste le geste « la facture n'était pas due », et le repassage en brouillon reste la voie d'une faute de
    saisie, P9) ; payée, son règlement reste détaché, à sa date, ouvert au lettrage (inchangé, P7).
- **D7 — Le rang 5 et `Today` dans la queue deviennent inatteignables en production ; ils restent.** Après la story, aucun lecteur ni geste
  d'annulation ne passe `CancellationDating::Today` à la queue ni à `reverse_owned_in_tx` ; seul `reverse` (R10) passe `Today` à
  `reverse_in_tx`. La variante `NoOpenFiscalYearToday`, le bras `Today` de `settlement_entry_cancel_blocker`, leurs clés et replis restent
  (exhaustivité des `match`, tests du harnais de précédence) ; leurs doc-comments disent « inatteignable en production depuis la 63-1 ».
  *Écartée* : les retirer ici — le retrait touche aussi `frontend/src/lib/features/reconciliation/` (union `FISCAL_YEAR_INVALID` de
  `reconciliation-cancel.ts`), soit un sixième module (§ *Règle de splitting préventif*). Il fait l'objet de l'issue `technical-debt` **#1759** (Q-C),
  texte au § *Issues ouvertes*.
- **D8 — Aucune migration, aucun backfill** : les annulations passées ne se réparent pas d'office (CLAUDE.md § *Ce qu'impose la production*) ;
  leur détection est l'objet de Q-B.

## Critères d'acceptation

Tests d'intégration `kesh-db` (`crates/kesh-db/tests/invoice_settlement.rs`, `invoice_write_off.rs`, `supplier_invoices_repository.rs`) et
de bout en bout `kesh-api` (`jour_suisse_nuit_e2e.rs`, `supplier_settlement_cancel_e2e.rs`, `invoice_settlement_response_e2e.rs`,
`supplier_invoice_response_e2e.rs`), **horloge de test posée pour que le jour diffère toujours de la date de l'origine** (sans quoi un test de
la date de l'origine passe à vide sous `Today`) ; le montage de `jour_suisse_nuit_e2e.rs` (horloge au 2026-01-01, origine en 2025) est le
patron.

- **AC-1 — Règlement client (#1752), origine dans un exercice antérieur encore ouvert, horloge en N+1** : `POST /invoices/{id}/settlements/{sid}/cancel`
  → `200` ; l'inverse est datée du règlement, dans N ; **aucune** écriture n'est créée dans N+1 (compte avant/après) ; la ligne est retirée,
  la facture redevient à régler ; le lettrage `reversal` est dans N ; `journal_entry.reversed` porte `"dating": "originDate"`.
- **AC-2 — Même geste, même exercice** : origine en mars, horloge en novembre du même exercice → inverse datée de mars (Q1).
- **AC-3 — Solde (#1752)** : un escompte (avec correction de TVA) daté de N, horloge en N+1, annulé → l'inverse, **ligne de TVA comprise**, est
  datée du solde, dans N ; le décompte de TVA de la période du solde (`kesh-report`, motif de `kesh-report/tests/vat_report_write_off.rs`) ne
  retient plus le solde **et** le grand livre de cette période nette à zéro les comptes du solde ; rien dans N+1.
- **AC-4 — Paiement fournisseur (#1753)** : comme AC-1 ; la facture redevient `open`, ses colonnes de règlement à `null`.
- **AC-5 — Facture fournisseur (#1754)**, validée **et** payée (deux cas) : inverse de l'achat datée de l'écriture d'achat, dans N ; charge et
  impôt préalable nets à zéro dans la période de l'achat ; facture `cancelled` ; payée : le règlement reste au grand livre, à sa date,
  détaché (`settlementJournalEntryId` à `null`, audit `supplier_invoice.cancelled` le nomme).
- **AC-6 — Exercice clôturé puis rouvert** (au moins un des trois gestes, #1754 de préférence) : l'annulation se date dans N rouvert ; N se
  reclôture ensuite sans obstacle nouveau.
- **AC-7 — Exercice de l'origine clos** (chacun des trois gestes) : la vue l'annonçait (`FISCAL_YEAR_CLOSED`) ; le clic est refusé
  `409 FISCAL_YEAR_CLOSED` ; rien n'est écrit (tests existants à relire : ils restent verts, Q6).
- **AC-8 — Période verrouillée** (date de l'origine ≤ `books_locked_through`, seuil inclusif testé au **jour de la borne** et au **lendemain**) :
  (a) `GET /invoices/{id}/settlements` rend `cancellable: false`, `cancelBlockedBy: "PERIOD_LOCKED"`, `cancelBlockedLabel` = la borne
  (`AAAA-MM-JJ`), pour un règlement **partiel** ; (b) la vue fournisseur rend `settlementCancelBlockedBy: "PERIOD_LOCKED"` (montage de D4 ou
  repli SQL, écrit au test) et `cancelBlockedBy: "PERIOD_LOCKED"` pour une facture validée non payée dont l'achat est sous la borne ; (c) au
  clic forcé par l'API, chacun des trois gestes rend **`400 PERIOD_LOCKED`** avec `details.lockedThrough` et `details.attempted`, rien n'est
  écrit ; (d) au lendemain de la borne, l'annulation passe ; (e) **précédence** : une facture client soldée dont toutes les lignes du groupe
  sont sous la borne rend toujours le rang 2 bis (`LETTERING_ALL_LINES_IN_CLOSED_PERIODS`), non `PERIOD_LOCKED` — test existant, relevé à la
  validation P2 : `kesh-db/tests/lettering_documents.rs::locked_period_cancel_is_refused_until_unlocked` (`:814`, règlement et solde, rang 2 bis puis
  déverrouillage à `le − 1` ; le règlement, daté `le`, reste au-dessus de la nouvelle borne : inchangé sous `OriginDate`).
- **AC-9 — Le rang 5 ne s'annonce plus depuis la fiche** : sans exercice ouvert au jour mais avec l'exercice de l'origine ouvert, la liste des
  règlements ne rend plus `FISCAL_YEAR_INVALID` et l'annulation **passe** (l'ancien test
  `jour_suisse_nuit_e2e::ac3_sans_exercice_au_jour_suisse_l_annulation_est_refusee`, `:749`, change d'attendu : il devient le test de cet AC).
- **AC-10 — Inventaire clos, figé** : une garde lexicale (`kesh-db/tests/`, motif des inventaires existants : `kesh-api/tests/bank_account_closed_inventory.rs`
  ou `kesh-db/tests/opening_types_sous_verrou_lexical.rs`) établit que **`CancellationDating::Today(` en position de construction** (expression, non
  motif) n'apparaît, dans `crates/*/src/`, hors `#[cfg(test)]` et doc-comments, **qu'à** `journal_entries::reverse`
  (R10) ; un site neuf la rougit. **Forme prescrite (validation P2)** : ne pas distinguer construction et motif par la syntaxe (un bras
  `X => CancellationDating::Today(j),` construit, et porte un `=>`) ; tenir l'**inventaire clos de toutes les occurrences** du jeton dans le texte
  masqué de `tests/lexique/mod.rs` (`decouper`, `blocs_de_test`, `fonctions`, `fonction_englobante`), chacune rattachée à sa fonction englobante,
  en égalité stricte avec `{journal_entries.rs::reverse : 1 (construction, R10) ; journal_entries.rs::from : 1 (motif, conversion `:3208`) ;
  settlement_cancellation.rs::settlement_entry_cancel_blocker : 1 (motif, bras `:228`)}` — la définition `:3192` ne porte pas le jeton (`Today(NaiveDate)`).
  Contournements fermés par la même garde : aucune occurrence de `CancellationDating as`, `CancellationDating::{`, `CancellationDating::*` ni d'un
  `use …CancellationDating::Today` en production (un alias, sur le patron du `ReversalDating as D` du module de test `:3324`, échapperait sinon au
  jeton) ; **angle mort écrit** : un `#[cfg(test)] mod x;` sans accolades fait masquer par `blocs_de_test` le code de production jusqu'à la fin du
  premier bloc `{ … }` qui suit (`bloc_apres` prend la première `{`) — trois sites à `6b71d4c21` : `kesh-api/src/pre_upgrade_backup.rs:432` et
  `kesh-core/src/exchange_rates/parse.rs:736` (fin de fichier, rien de masqué), `kesh-report/src/lib.rs:26` (masque `:26-43`, des `pub mod`/`pub use`,
  sans jeton) ; défaut de l'outil commun, hors story (issue **#1760**, § *Issues ouvertes*). ⚠️ **Retrait des tests par bloc, non par troncature à la première ligne `#[cfg(test)]`** : `journal_entries.rs` en
  porte une dès `:3322` (module `verdict_de_datation`), avant `reverse` (`:3994`) : une troncature ne verrait pas R10 et la garde passerait à vide. Sonde positive
  écrite au test : R10 **doit** être trouvé (le test échoue s'il ne trouve aucune construction), plus des échantillons synthétiques rougissants —
  un argument sur plusieurs lignes, un bras `=> CancellationDating::Today(j)`, un alias `use … as D; D::Today(j)` (mémoire « pathspec git muet »).
- **AC-11 — Le test de non-régression AC-11 de la 53-6a S'INVERSE.** La 53-6a figeait « les annulations depuis la fiche restent datées du
  jour » par `jour_suisse_nuit_e2e::ac3_annulation_d_un_reglement_client_la_nuit` (`:635`), `ac3_annulation_d_un_reglement_fournisseur_la_nuit`
  (`:668`), `ac3_annulation_d_une_facture_fournisseur_la_nuit` (`:653`) et `ac3_sans_exercice_au_jour_suisse_l_annulation_est_refusee`
  (`:749`). Ces quatre tests **changent d'attendu** : l'inverse est datée de l'origine (2025), non du 1er janvier 2026
  (`assert_contre_passee_le_1er_janvier` ne s'y applique plus) ; le dernier devient AC-9. ⚠️ Ils tuaient la mutation M7 de la 35-5a0 (jour
  UTC au lieu du jour suisse) pour ces gestes : ces gestes ne lisant plus le jour, seul `ac3_contre_passation_la_nuit_du_nouvel_an` (R10,
  `:616`) la tue encore — le dire au Dev Agent Record. `ac3_contre_passation_la_nuit_du_nouvel_an` et le dé-rapprochement (`:703`) ne changent pas.
  **Autres tests qui changent d'attendu** (relevés à la validation P2, à `6b71d4c21` ; liste **non close** — voir la dernière puce) :
  - `jour_suisse_nuit_e2e::ac7_les_services_jugent_au_jour_suisse` (`:893`) : `GET /invoices/{id}/settlements` y rend `cancelBlockedBy:
    "FISCAL_YEAR_INVALID"` (`:941`, rang 5 sous `Today`) ; sous `OriginDate`, l'exercice 2025 du règlement est ouvert : `null`, et
    `cancellable: true` (`:940`). Le doc-comment (`:878-881`) se réécrit, comme la 53-6a l'a fait pour la candidate (`:883-886`). Ce test tuait la
    forme « services » de la mutation M7 de la 35-5a0 pour ce service : `services::factures::reglements` ne lisant plus le jour, le Dev Agent
    Record dit quel test tient encore le jour suisse des services, ou qu'aucun ne le tient plus pour ce service.
  - `kesh-db/tests/lettering_documents.rs::locked_period_supplier_cancels_are_refused_until_unlocked` (`:1530`) : après le déverrouillage à
    `le − 1` (`:1651`), `supplier_invoices::cancel(s2)` (`:1675`, `.expect("facture annulée après déverrouillage")`) contre-passe un achat daté
    `jours_avant(100)`, **sous** la nouvelle borne : sous `OriginDate`, `PERIOD_LOCKED` — c'est exactement le cas D4 #1754 « achat sous la borne,
    paiement après, groupe non figé ». Le test garde son intention (le rang 2 bis cède au déverrouillage) en déverrouillant **avant l'achat**
    pour la facture, et gagne une assertion `PeriodLocked` entre les deux (preuve `kesh-db` d'AC-8 c pour #1754). L'annulation du paiement
    `s1` (`:1661`, daté `le`) reste acceptée.
  - **Les trois harnais de précédence « lecture ET clic »** qui lisent sous `Today` et cliquent par le geste **public** : après la story, la
    lecture et le clic ne disent plus la même chose (« une seule fonction pour lire et pour écrire » n'est plus tenue par le test) —
    `kesh-db/tests/invoice_settlement.rs::precedence(_, origine = false)` (`:1475` ; lecture `Today` `:1505`, clic `cancel_settlement` `:1555`,
    commentaire « Sous `Today`, le geste de la fiche (R7, inchangé) » devenu faux) ; `supplier_invoices_repository.rs::tail_motives_through_the_supplier_path`
    (`:1762` ; lecture `Today` `:1793`, clic `supplier_invoices::cancel_settlement` `:1806`, cas `NoOpenFiscalYearToday` `:1767-1768`, `:1782`) ;
    `invoice_cancel_motives_and_their_precedence` (`:2076` ; `supplier_invoice_cancel_blocker` perd sa date, cas `NoOpenFiscalYearToday`
    `:2095`, `:2102`, attendu `FiscalYearInvalid` `:2156`). **Prescription** (D7 : la queue `Today` reste testée) : pour les deux familles de
    règlements, la branche `Today` clique par la forme `_in_tx` sous `Today` (patron de la branche `origine = true`, `:1540-1552`) ; la
    branche du geste public est jouée sous `OriginDate` (lecture comprise). Pour l'annulation de facture, qui n'a plus de datation en
    paramètre, les lignes `NoOpenFiscalYearToday` deviennent « lecture `None`, clic accepté » (AC-9 pour #1754) et une ligne `PeriodLocked`
    s'ajoute (facture validée non payée sous la borne) ; le rang 5 de cette famille devient inatteignable (D7), ce que le doc-comment dit.
  - `kesh-db/tests/open_items.rs::as_of_under_the_lock_is_stable_until_an_admin_unlocks` (`:871`, relevé à la validation P3) : la facture est datée
    `jours_avant(100)`, le règlement `jours_avant(90)`, `X = jours_avant(85)` (`:887`) ; après `unlock_books(… None …)` (`:923`),
    `invoice_settlements_write::cancel_settlement` (`:933`) contre-passe **à la date du règlement** (−90), non au jour. Le groupe `reversal`
    {règlement, miroir} est lettré à `MAX(entry_date)` = −90 ≤ `X` : il est **acquis au X** (`open_items.rs:71-77` de `letterings/`). Les attendus de
    `:955-962` changent : `item(&apres, l_regl)` ne trouve plus la ligne (elle n'est plus listée), `OpenReason::LetteredAfterAsOf` (`:948`) ne s'y
    applique plus, `apres.total == avant.total + 2` (`:952`) devient `+ 1` (seule la créance de la vente, `Unlettered`, réapparaît) ; l'invariant
    `invariant(&pool, &m, &apres)` tient toujours (le miroir daté −90 entre dans la balance au X). Le doc-comment (`:865-869`, « la créance et la ligne
    de règlement réapparaissent ») se réécrit ; la démonstration « lettrée après `X` » ne peut plus se faire par ce geste (l'inverse n'est jamais
    postérieure à son origine) : elle reste portée par les autres tests du fichier (`:444`, `:487`, `:630`, `:861-862`).
  - **Montages relus à la validation P3 sans changement d'attendu attendu** (la date de l'inverse y passe du jour à l'origine, sans que rien ne la
    compare ; à confirmer par le vert, non à croire) : `tests/support/lettering_documents.rs::seed_lettering_documents` (annulation de `apres`
    = −150, verrou posé ensuite à −300), `lettering_documents_backfill.rs::rattrapage` (annulations à −200/−140, borne −250 ensuite),
    `admin_full_import_e2e.rs::lettering_pieces` (annulations avant la borne du 2026-03-31), `letterings_e2e.rs::forecast_equals_delete_over_http`
    (annulations avant le verrou `jours_avant(1)`), `letterings.rs::group_detail_of_reversal_pairs` (borne = le jour, couvre l'inverse datée −100 ;
    commentaire `:2392` « datée du jour » à réécrire), `supplier_invoices_internal_number.rs::cancelled_then_draft_keeps_the_number` (`:340`,
    « la contre-passation se date du jour : A doit le couvrir » devient faux ; `UPDATE fiscal_years SET end_date = '2099-12-31'` n'est plus
    nécessaire), `invoice_settlement.rs` `:764` (« contre-passation datée du jour ») et `:1652-1654` (le garde « exercice du jour » ne
    s'applique plus : le socle ne tient plus l'exercice du jour ; le test garde sa preuve, le rang 2 bis étant refusé avant le socle), `:2531`.
    Les commentaires de test qui disent « datée du jour » pour l'un des trois gestes se corrigent au même patch (propagation du symptôme).
  - **Méthode de clôture, à T1/T2** : la liste ci-dessus vient de la lecture (`FISCAL_YEAR_INVALID`/`NoOpenFiscalYearToday`, verrou puis geste
    public dans une même fonction de test) ; un test dont le montage pose verrou ou exercices dans un auxiliaire lui échappe. Après T2, le
    développeur lance `kesh-db` et `kesh-api` entiers, **inventorie chaque rouge** au Dev Agent Record (fichier, test, attendu ancien → nouveau,
    raison : datation de l'origine, ou défaut) et n'en change aucun attendu sans cette ligne.
- **AC-12 — Concurrence** : `close(N)` concurrent d'une annulation de facture fournisseur (#1754) dont l'achat est en N — ou la clôture
  attend et l'annulation passe dans N, ou elle passe d'abord et l'annulation est refusée `FISCAL_YEAR_CLOSED` ; jamais d'inverse dans un
  exercice clos (patron `a_concurrent_close_waits_for_the_unreconciliation`, 53-6a AC-12 b ; les gestes de la fiche tiennent déjà l'écriture et
  son exercice avant toute lecture ordinaire, P5). Ordre des verrous : celui de la 53-6a (D3, « l'ensemble diminue ») — le développeur le
  constate par la sonde `FOR UPDATE NOWAIT` de la 53-6a (`s536a_ac12_la_contre_passation_datee_de_l_origine_ne_tient_pas_l_exercice_du_jour`)
  rejouée sur **un** geste de la fiche, au Dev Agent Record. La seconde branche (la clôture passe d'abord) a déjà son test, sans `sleep` :
  `kesh-db/tests/supplier_invoices_repository.rs::a_concurrent_close_waits_for_the_invoice_cancellation` (`:2209`, attente par
  `test_fixtures::attendre_une_requete_en_cours` sur `fiscal_years … FOR UPDATE`, refus `FiscalYearClosed`) — il ne change que de signature et
  reste vert ; la première branche (l'annulation tient l'exercice, la clôture attend) s'écrit sur son patron. Course avec `lock_books` : la borne
  est lue **sans verrou** par `create_in_tx_inner` (`journal_entries.rs:605-616`, choix écrit : « ferme le passé et non l'instant présent ») —
  sous `OriginDate`, une annulation peut donc être validée à l'instant où une borne qui couvre sa date est posée, comme toute écriture saisie
  dans le passé ; sous `Today`, l'écart n'existait pas (borne < jour). Angle mort assumé, hérité de la 53-6a, à dire au Dev Agent Record.
- **AC-13 — Écran** (Vitest) : `settlement-cancel-blocked.test.ts`, `invoices/settlement-cancel.test.ts`, `supplier-invoices/settlement-cancel.test.ts`,
  `supplier-invoices/invoice-cancel.test.ts` — un texte par motif, `PERIOD_LOCKED` compris, borne ajoutée en date suisse ; `InvoiceSettlements.test.ts`
  — le motif remplace le bouton ; les quatre confirmations ne contiennent plus « aujourd'hui » (assertion **sur le texte neuf**, non
  `not.toContain` de l'ancien, qui passerait à vide — leçon de la 53-6a, M1 de sa P2) ; `sitesTotal` de `i18n-keys.test.ts` et les gardes
  i18n recomptés depuis la source.
- **AC-14 — Manuel et API** (`.tex` et `docs/api-external.md` ; les PDF ne se commitent pas, ils se régénèrent à la release) :
  `user-manual.tex` — encadré « Ce que le verrou n'empêche pas » (`:854-875` : « La contre-passation est datée du jour, donc après le verrou :
  elle passe » ne vaut plus que pour `/reverse` d'une écriture non possédée et les transitoires ; les **trois** gestes rejoignent l'exception
  du rapprochement) ; encadré `sec:date-du-jour` (`:895-899`, « règlement, facture fournisseur » sortent de la liste « datée du jour ») ;
  § *Enregistrer et annuler un règlement* (`:1964-1966` le solde, `:1968-1973` le règlement, la puce « aucun exercice ouvert ne couvre la date
  du jour » `:1998-1999` **remplacée** par la période verrouillée) ; § règlement fournisseur (`:2618-2621`, et sa liste « Quand l'annulation
  n'est pas possible » `:2630` et suivantes) ; § *Annuler une facture fournisseur* (`:2694-2698`, liste des motifs `:2741-2748` : « aucun
  exercice ouvert ne couvre la date du jour » remplacé par la période verrouillée) ; encadré « La paire achat et contre-passation n'est nette
  à zéro qu'au cumul » (`:2489-2494` : vrai pour une annulation **d'avant** la v0.21.0 seulement) ; § décompte de TVA, « Le solde annulé »
  (`:4747-4752` : réécrit selon P8 — la reprise a lieu dans la période du solde) ; `:4741-4745` (l'avoir) **relu, inchangé** (hors story, 53-2) ;
  `:1152` (« Une contre-passation, datée du jour, n'est refusée que si l'exercice du jour est lui-même suivi d'un exercice clôturé » ; la 53-6a y a ajouté le dé-rapprochement : les trois gestes s'y ajoutent). `admin-manual.tex:3556` — le verrou de
  période est aussi le remède du refus d'annuler un règlement, un solde, un paiement ou une facture fournisseur dont l'écriture est sous la
  borne. `docs/api-external.md` — `:686` (`GET …/settlements` : valeur `PERIOD_LOCKED` de `cancelBlockedBy`, borne dans `cancelBlockedLabel` ; l'« angle mort assumé » parle de l'exercice **du jour** : c'est désormais l'exercice de l'écriture, état hérité seulement) ; `:688` (« datée du jour, dans l'exercice ouvert qui le couvre » → date et exercice de l'écriture) et sa table (`:699` « Aucun exercice ouvert ne couvre la date du jour » **sort** ; `:700` « L'exercice du jour est suivi… » → l'exercice de l'écriture, état hérité ; `:701` « Date du jour dans une période verrouillée » → date de l'écriture) ; `:799` (« datée du jour ») ; `:801` (valeurs des champs de la vue) ; la liste des refus `:809` (`FISCAL_YEAR_INVALID` sort ; `LATER_FISCAL_YEAR_CLOSED` : exercice de l'écriture) ; `:813` (« datée du jour ») ; `:815` (valeurs) ; la liste des refus `:817` ; l'avertissement `:819` (« la contre-passation, elle, serait datée d'un exercice ouvert » devient faux : elle est datée de l'exercice de l'achat) ; la table générale des codes (`PERIOD_LOCKED`, `FISCAL_YEAR_INVALID`) relue. **Recherche du symptôme** dans les `.tex` **et** dans les PDF aplatis
  (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o -i '.\{100\}datée du jour.\{100\}'`, puis `date du jour`,
  `aujourd'hui`, `du jour de l'annulation`) : chaque phrase qui parle des trois gestes est corrigée ; celles de R10, R11, des avoirs et du
  dé-rapprochement restent. Les manuels DE, IT, EN n'existent pas au dépôt (constat de la 53-6a, à reconfirmer).
- **AC-15 — E2E Playwright** : (1) `frontend/tests/e2e/invoice-write-off.spec.ts` (ou une spec voisine) — annuler un règlement d'une facture
  datée dans un exercice antérieur ouvert : après confirmation, *Voir l'écriture* de l'inverse montre la date du règlement ; (2) une facture
  fournisseur validée dont l'achat est sous la borne : le motif « période verrouillée » remplace le bouton *Annuler la facture*.
  **Recette visuelle** (CLAUDE.md) : captures des quatre confirmations et des deux motifs neufs, fiche client et fiche fournisseur, sur une base
  de démonstration, dans `kesh-gate-logs/`.
- **AC-16 — Gates et mutations** : gate complet (`scripts/test-fast.sh`, puis `scripts/gate-ci.sh`), frontend (`check`,
  `lint-i18n-ownership`, `test:unit`, `build`), E2E complet au dernier commit de code ; mutations M1 à M8 tuées, chacune par le test annoncé.
- **AC-17 — Aucune migration** : `git diff --stat <base>..HEAD -- crates/kesh-db/migrations/ crates/kesh-db/test-schema/` vide.

## Mutations (une à une, fichier touché après restauration — mémoire « mutation restaurée, binaire périmé »)

⚠️ **Forme des mutations M1 à M6 après D3** : les gestes et lecteurs de ces mutations n'ont plus de paramètre de date ; « `OriginDate` → `Today` » se
fait donc en réintroduisant une date, `Today(chrono::Utc::now().date_naive())` (le jour réel du poste, qui diffère de l'origine dans tous les
montages d'AC-1 à AC-9, passée en `jours_avant` ou posée en 2025 sous l'horloge d'`kesh-api`), et non en rendant un paramètre qui n'existe plus.
Après chaque mutation, toucher le fichier à la restauration.

| # | Mutation | Test qui la tue |
|---|---|---|
| M1 | R7 : `OriginDate` → `Today` dans `invoice_settlements_write::cancel_settlement` | AC-1 (et AC-3 pour le solde) |
| M2 | R8 : `OriginDate` → `Today` dans `supplier_invoices::cancel_settlement` | AC-4 |
| M3 | R9 : `OriginDate` → `Today` dans `cancel_in_tx` | AC-5 |
| M4 | lecteur client (`services::factures::reglements`) resté sous `Today` | AC-8 (a) et AC-9 |
| M5 | vue fournisseur, bloqueur du **règlement**, resté sous `Today` | AC-8 (b), règlement |
| M6 | `supplier_invoice_cancel_blocker` resté sous `Today` | AC-8 (b), facture |
| M7 | garde lexicale : un `CancellationDating::Today` ajouté dans un geste d'annulation | AC-10 |
| M8 | écran : bras `PERIOD_LOCKED` retiré de `settlementCancelTailMessage` (ou de `supplierInvoiceCancelMessage`) — le `never` le refuserait à la compilation ; la mutation est donc le **texte** (clé échangée) | AC-13 |

## Tâches

- [x] **T0 — Reprise et relecture** : la 53-6a fusionnée dans `epic/53-ecritures-justes`, repartir de cette branche (rebase ou nouvelle
  branche `story/63-1-…` depuis elle — demander à l'orchestrateur) ; relire chaque prémisse P1 à P10 et chaque numéro de ligne de la fiche,
  corriger la fiche s'il le faut (P4 fichier par fichier ; P8 au code de `kesh-report` ; D6 la date de l'écriture d'achat `:1831` ; D4 si `pay`
  accepte un paiement antérieur à la facture ; P10 l'annulation d'un lot) ; `gh issue view 1752|1753|1754 --json state` (lecture seule,
  espacées).
- [ ] **T1 — Tests rouges d'abord** (AC-1 à AC-5, AC-8, AC-9, AC-10 ; AC-11 : changer les attendus des quatre tests `ac3_*`) ; commit, push.
- [ ] **T2 — Gestes et lecteurs** (D1, D2, D3) : les trois datations, les trois lecteurs, le retrait d'`aujourd_hui` de la chaîne de vue
  fournisseur, de ses enveloppes en `kesh-db` (`validate`, `unvalidate`, `unvalidate_with_policy`, `pay`, `cancel`, `cancel_in_tx`, `cancel_settlement`,
  `get_settlement_view`) et de **tous** leurs appelants (routes, quatre `_once`, 230 sites de tests, P4/D3) ; doc-comments « datée du jour » (`SettlementCancellation`, `CancelledSettlement` `:1233`,
  `SupplierSettlementCancellation` `:3413`, `SupplierSettlementCancelled` `:3620`, doc de `cancel_settlement_in_tx` `:3417-3438`,
  `supplier_invoice_cancel_blocker`, routes `invoices.rs:1601-1610`, `supplier_invoices.rs:1007-1016`, `:1046-1055`) ; commit, push.
- [ ] **T3 — Textes serveur** (D5, D7) : `kesh-api/src/errors.rs` (deux bras `PeriodLocked`, commentaires des rangs 5) ; quatre locales
  (deux motifs neufs, quatre confirmations) ; garde lexicale d'AC-10 ; commit, push.
- [ ] **T4 — Écran** : `settlement-cancel-blocked.ts`, `invoice-cancel.ts` (unions, textes, borne) ; **les quatre replis inline des deux pages**
  (`invoices/[id]/+page.svelte:1867` et `:1871`, `supplier-invoices/[id]/+page.svelte:319` et `:366`, qui disent « datée d'aujourd'hui » et
  doivent dire mot pour mot le FTL fr-CH réécrit — garde `i18n-un-repli-par-cle.test.ts`) ; grep de la **valeur** `datée d'aujourd'hui` /
  `d'aujourd'hui` sur `frontend/src` et `kesh-i18n/locales` (quatre locales : DE « heutigen Datum », IT « oggi », EN « today »), et relecture
  — sans changement forcé — de `supplier-invoices-unvalidate-confirm-body-cancelled` (« la période de l'achat et de celle de l'annulation » reste
  vrai pour une annulation d'avant la v0.21.0, `+page.svelte:1342-1346`, commentaire `:14`) ; `invoices/[id]/+page.svelte` et
  `supplier-invoices/[id]/+page.svelte` (lecture de ce qu'ils affichent d'un `400 PERIOD_LOCKED` au clic) ; tests unitaires ; `sitesTotal` ;
  commit, push.
- [ ] **T5 — Manuel et API** (AC-14) ; commit, push.
- [ ] **T6 — E2E** (AC-15), recette visuelle, gates, mutations (AC-16) ; Dev Agent Record ; commit, push de la seule branche `story/…`.

## Dev Notes

### Fichiers touchés (prévus)

- `crates/kesh-db/src/repositories/invoice_settlements_write.rs` — `cancel_settlement` (datation, paramètre), doc-comments.
- `crates/kesh-db/src/repositories/supplier_invoices.rs` — `cancel`, `cancel_in_tx`, `cancel_settlement`, `supplier_invoice_cancel_blocker`,
  `get_settlement_view(_in_tx)`, `commit_with_view`, doc-comments.
- `crates/kesh-db/src/repositories/settlement_cancellation.rs` — doc du bras `Today` et du rang 5 (D7) ; aucun code.
- `crates/kesh-db/src/errors.rs` — doc de `NoOpenFiscalYearToday` (D7).
- `crates/kesh-api/src/routes/invoices.rs`, `routes/supplier_invoices.rs`, `routes/imported_supplier_invoices.rs` (appels de
  `commit_with_view`), `services/factures.rs`, `services/factures_fournisseur.rs`, `services/mod.rs` (doc du périmètre), `errors.rs`.
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`.
- `frontend/src/lib/shared/utils/settlement-cancel-blocked.ts` (+ test), `frontend/src/lib/features/invoices/` (`settlement-cancel.ts`,
  `InvoiceSettlements.svelte` et leurs tests), `frontend/src/lib/features/supplier-invoices/` (`invoice-cancel.ts`, `settlement-cancel.ts` et
  leurs tests), `frontend/src/routes/(app)/invoices/[id]/+page.svelte`, `frontend/src/routes/(app)/supplier-invoices/[id]/+page.svelte`,
  `frontend/src/lib/shared/i18n-keys.test.ts`.
- Tests : `crates/kesh-api/tests/jour_suisse_nuit_e2e.rs` (AC-9, AC-11), `supplier_settlement_cancel_e2e.rs`,
  `invoice_settlement_response_e2e.rs`, `supplier_invoice_response_e2e.rs` ; `crates/kesh-db/tests/invoice_settlement.rs`,
  `invoice_write_off.rs`, `supplier_invoices_repository.rs`, une garde lexicale (AC-10) ; `crates/kesh-report/tests/vat_report_write_off.rs`
  (AC-3, à relire) ; E2E `frontend/tests/e2e/`. Le compilateur trouve les signatures changées (`git grep -n "commit_with_view(\|get_settlement_view\|supplier_invoice_cancel_blocker(\|supplier_invoices::cancel(\|cancel_settlement(" -- 'crates/*/tests/*'`) ;
  la relecture porte sur les **attendus** : tout test qui affirme une inverse **datée du jour** sur l'un des trois gestes change d'attendu
  (`git grep -n "entry_date\|entryDate\|aujourd_hui_test" ` dans les fichiers ci-dessus, et
  `git grep -ln "settlements/{[a-z_0-9.]*}/cancel\|settlement/cancel\|supplier-invoices/{[a-z_0-9.]*}/cancel" -- 'crates/*/tests/*'` — 15 à 20
  fichiers à `6b71d4c21`, dont `letterings_e2e.rs`, `homepage_summary_e2e.rs`, `invoice_echeancier_e2e.rs`, `project_reassignment_e2e.rs`,
  `supplier_invoice_draft_e2e.rs`, `supplier_invoice_duplicate_e2e.rs`, `admin_full_import_e2e.rs`, `rejeu_interblocage_e2e.rs` ; à recompter à
  T0 et à écrire au Dev Agent Record avec son périmètre). Les tests `kesh-db` qui passent `CancellationDating::Today(aujourd_hui_test())` à
  `cancel_settlement_in_tx` ou aux bloqueurs **ne changent pas** quand ils **cliquent aussi** par la forme `_in_tx` : elle garde sa datation en
  paramètre. ⚠️ Ceux qui lisent sous `Today` **et** cliquent par le geste public changent (les trois harnais d'AC-11) ; et
  `supplier_invoices::cancel_in_tx` **perd** sa date (D3) : ses quatre appels de test se datent désormais de l'origine.
- Documentation : `docs/manual/fr/user-manual.tex`, `docs/manual/fr/admin-manual.tex`, `docs/api-external.md`. Pas de CHANGELOG (assemblé à
  l'intégration), pas de PDF.

### Ce qui doit être préservé

- L'ordre des étapes de chaque geste (verrous de la pièce, de l'écriture **et de son exercice** avant toute lecture ordinaire ; compte bancaire
  clos revérifié en **dernière** acquisition ; audit du geste en dernier).
- Les listes fermées des refus des trois gestes (P6) : `PeriodLocked` n'y entre pas.
- Le dé-rapprochement (R1 à R6, 53-6a), `/reverse` (R10, `Today`), `/reverse-at-next-fiscal-year-start` (R11), la dévalidation (P9), les avoirs.
- Le JSON de `journal_entry.reversed` du chemin `Today` (`/reverse`) octet pour octet (T-b2 de la 29-1b).
- Les formes `_in_tx` (`cancel_settlement_in_tx` client et fournisseur) gardent leur paramètre `dating` : le dé-rapprochement les appelle en
  `OriginDate`, les tests en `Today`.

### Règle de splitting préventif

Modules dont le code de production change : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend/src/lib/features/invoices`,
`frontend/src/lib/features/supplier-invoices` — **cinq**, à la borne (« plus de cinq » découpe) ; `frontend/src/lib/shared/utils/` et les deux
pages `routes/(app)/…/[id]` relèvent de ces domaines ou du commun, non d'un sixième domaine. Le retrait du rang 5 (D7) est sorti de la story
pour cette raison (il toucherait `features/reconciliation`). **Si la validation recycle** (une passe N+1 de sévérité égale ou supérieure dont
les défauts naissent du correctif précédent), découpage proposé, sans cycle : **63-1a** — #1752 (fiche client : `kesh-db`, `kesh-api`,
`kesh-i18n`, `features/invoices`, `lib/shared`) ; **63-1b** — #1753 et #1754 (fiche fournisseur : `kesh-db`, `kesh-api`, `kesh-i18n`,
`features/supplier-invoices`), qui dépend de 63-1a pour la clé partagée `settlement-cancel-blocked-period-locked`. La story unique est
préférée : les trois gestes partagent la queue, les textes et le manuel, et la borne de six de l'epic n'est pas en jeu (une seule story).

### Recoupements

- **53-6a** (dont elle dépend) : mêmes fichiers `invoice_settlements_write.rs`, `supplier_invoices.rs`, `settlement_cancellation.rs`,
  `kesh-api/src/errors.rs`, catalogues, `user-manual.tex` (encadré du verrou `:854-875`, `sec:date-du-jour`), `api-external.md` ; la 63-1 part
  de la 53-6a fusionnée, sans conflit attendu.
- **53-2** (#525, #1198, migration probable) : `invoice_settlements_write.rs` (correction de TVA du solde) et `credit_notes.rs` — fonctions
  distinctes ; la seconde fusionnée reprend la première ; la 63-1 ne change pas les lignes du solde, seulement la date de leur inverse.
- **53-6b** (détection, `backlog`) : **élargie** par la décision Q-B du 2026-11-27 (critère `reverses_entry_id`, tout geste d'annulation dont l'origine et l'inverse sont dans deux exercices) ; l'orchestrateur met sa fiche à jour, la 63-1 n'y touche pas.
- **Epic 38** (contributeur, contre-passation à date choisie de `/reverse`) : touche le socle (`CancellationDating`/`ReversalDating`) ; la 63-1
  n'y change rien ; à contrôler à la revue du code d'intégration.

### Migration

**Aucune.** *Migration breaking policy* sans objet.

### Notes de version (à reprendre dans le CHANGELOG à l'intégration)

> ⚠️ **Annuler un règlement, un solde, un paiement fournisseur ou une facture fournisseur contre-passe désormais l'écriture à sa propre date,
> dans son propre exercice** (#1752, #1753, #1754). Jusqu'ici, l'écriture inverse était datée du jour : une annulation faite après la fin de
> l'exercice de l'écriture — exercice rouvert, ou pas encore clôturé — la plaçait dans l'exercice courant, et deux exercices étaient faux :
> le bilan (banque, débiteurs, créanciers), et pour un solde ou une facture fournisseur, le résultat et la TVA. Conséquences : **une annulation
> acceptée jusqu'ici peut être refusée** — celle dont l'écriture est dans une **période verrouillée** (la fiche le dit d'avance ; un
> administrateur recule le verrou) ; c'est notamment le cas d'une facture fournisseur d'un trimestre TVA verrouillé. L'annulation change les
> totaux de la période de l'écriture, TVA comprise : si elle a été déclarée, faites-la verrouiller d'abord. Les annulations faites avec les
> versions précédentes ne sont pas corrigées par cette release.
> **API** : `POST /api/v1/invoices/{id}/settlements/{settlementId}/cancel`, `POST /api/v1/supplier-invoices/{id}/settlement/cancel` et
> `POST /api/v1/supplier-invoices/{id}/cancel` datent l'inverse à la date de l'écriture ; refus nouveau `400 PERIOD_LOCKED` (date de
> l'écriture sous la borne) ; le refus `FISCAL_YEAR_INVALID` (« aucun exercice ouvert ne couvre la date du jour ») ne se produit plus pour ces
> gestes ; `GET /invoices/{id}/settlements` (`cancelBlockedBy`), `GET /supplier-invoices/{id}` et les réponses des gestes qui écrivent une
> facture fournisseur (`settlementCancelBlockedBy`, `cancelBlockedBy`) gagnent la valeur `PERIOD_LOCKED`, borne dans le champ `…Label` ; le
> détail d'audit `journal_entry.reversed` de ces gestes gagne `"dating": "originDate"`. Aucune migration.

### Issues ouvertes (le 2026-11-27, par l'orchestrateur, une par problème)

- **#1759 — Q-C, acceptée le 2026-11-27** — `technical-debt`, `P3` : « La queue commune des annulations garde un rang « aucun exercice ouvert au
  jour » et une datation `Today` que plus aucune annulation n'emploie » — corps : D7 de cette fiche ; retrait de
  `SettlementCancelBlocker::NoOpenFiscalYearToday`, du bras `Today` de `settlement_entry_cancel_blocker` et du paramètre `dating` des formes
  qui ne reçoivent plus qu'`OriginDate`, des clés `*-blocked-no-fiscal-year` (trois familles, quatre locales) et des bras `FISCAL_YEAR_INVALID`
  des unions TS (`settlement-cancel-blocked.ts`, `invoice-cancel.ts`, `reconciliation-cancel.ts`) ; aucune donnée réelle.
- **#1760 — relevée à la validation P2** — `technical-debt`, `P3` (rattachement au Project Lead) : « `lexique::blocs_de_test` masque du code
  de production après un `#[cfg(test)] mod x;` sans accolades » — corps : `bloc_apres` prend la première `{` qui suit l'attribut ; à
  `6b71d4c21`, `kesh-report/src/lib.rs:26` masque `:26-43` (sans conséquence aujourd'hui), `pre_upgrade_backup.rs:432` et
  `exchange_rates/parse.rs:736` sont en fin de fichier ; toute garde lexicale de `kesh-db/tests/` (`letterings_lexical.rs`,
  `chf_line_lexical.rs`, `opening_types_sous_verrou_lexical.rs`, `kesh-api/tests/bank_account_closed_inventory.rs`, et celle d'AC-10) en hérite.

### Références

- Issues #1752, #1753, #1754 (P1, milestone v0.21.0) ; textes source `kesh-gate-logs/e53/53-6-issues-q3/` ; #1674 et la fiche 53-6a
  (`53-6a-annulation-rapprochement-date-origine.md` : D2, D5, D6, AC-11, inventaire R1 à R11, décisions Q1 à Q6).
- `CLAUDE.md` : *Ce qu'impose la production*, *Priorités des défauts*, *Migration breaking policy*, *Règle de splitting préventif*,
  *Versions de l'API*, *Recette visuelle*, *Pattern batch* (sans objet : aucun endpoint batch touché).
- Stories : 25-3-a-1 et 25-3-a-2 (annulation des règlements, queue commune), 25-3-c (annulation de facture fournisseur), 25-4-d2a (solde),
  15-1a2-0/i/ii (lettrage de pièce), 24-4c (verrou de période), 35-5a0 (date du jour lue une fois), 34-8 (`commit_with_view`), v0172
  (repassage en brouillon d'une annulée), 53-6a.
- `docs/testing.md` (Playwright local), `docs/agents-orchestration.md` (consignes des agents).

## Questions pour le Project Lead — tranchées le 2026-11-27

**Décisions du Project Lead du 2026-11-27**, toutes selon la recommandation de la fiche (ne se rouvrent pas) :

- **Q-A — tranchée** : le refus en période verrouillée s'applique **tel quel** aussi à l'annulation d'une facture fournisseur et d'un solde. Une demande de
  changement pourra proposer plus tard une annulation volontairement datée du jour (choisie et journalisée).
- **Q-B — tranchée** : la détection de la **53-6b** s'élargit à toute contre-passation d'annulation dont l'origine et l'inverse sont dans deux exercices,
  quel que soit le geste (critère `reverses_entry_id`). **La fiche de la 53-6b n'est pas touchée ici** : l'orchestrateur la met à jour.
- **Q-C — tranchée** : le retrait du rang « aucun exercice ouvert au jour » et de `Today` dans la queue va en issue `technical-debt` P3, **hors de la
  v0.21.0** ; l'orchestrateur l'a ouverte : **#1759** (texte au § *Issues ouvertes*).
- **Q-D — tranchée** : la dette de l'Epic 63 se verse à la story de solde de l'Epic 53 (**53-5**) ; l'orchestrateur l'écrit au fichier d'epic.

*Questions posées, avec la recommandation de la fiche entre parenthèses :*

- **Q-A — Q2 appliquée à l'annulation d'une facture fournisseur et d'un solde.** Pour le dé-rapprochement, le refus en période verrouillée
  touchait surtout un paiement récent ; ici, il touche **le cas courant** de l'annulation d'une facture fournisseur reçue d'un trimestre TVA
  déjà verrouillé : il faudra reculer le verrou (motif, audit), annuler, reverrouiller — et la période déclarée change. Alternative : laisser
  ces gestes datés du jour **quand** l'écriture est sous la borne (correction « depuis le présent »), ce qui refait deux exercices faux si la
  borne et la fin d'exercice se croisent. *(Recommandation : appliquer Q2 telle quelle, comme le lancement de la 63-1 le demande ; la
  correction d'une période déclarée reste l'affaire de la fiduciaire, et le manuel dit le geste. Si l'usage le rend pénible, une demande de
  changement séparée pourra proposer une annulation explicitement datée du jour, choisie et journalisée.)*
- **Q-B — La détection de la 53-6b doit-elle couvrir aussi les annulations depuis la fiche ?** La 53-6b ne lit que `reconciliation.cancelled`
  (P10 de sa fiche) ; les trois gestes ont laissé le même défaut dans les livres, avec leurs propres entrées d'audit
  (`invoice.settlement_cancelled`, `supplier_invoice.settlement_cancelled`, `supplier_invoice.cancelled`, toutes avec
  `reversalJournalEntryId`). *(Recommandation : élargir la 53-6b à toute contre-passation d'annulation dont l'origine et l'inverse sont dans
  deux exercices — critère `reverses_entry_id`, quel que soit le geste —, ou ouvrir une issue P1 sœur si l'on veut garder la 53-6b étroite.)*
- **Q-C — Retrait du rang 5 et de la datation `Today` de la queue** (D7). *(Recommandation : issue `technical-debt` P3 ouverte dès la fiche
  validée, traitée hors de la v0.21.0.)*
- **Q-D — Story de solde de l'Epic 63.** CLAUDE.md en veut une par epic ; l'Epic 63 n'a qu'une story de correction. *(Recommandation :
  pas de story de solde propre — la dette de revue de la 63-1 va à la 53-5, story de solde de l'Epic 53, qui sort dans la même release, dans
  la limite de son plafond, ou en issues ; à écrire au fichier d'epic.)*

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`), `bmad-dev-story`, worktree `kesh-63-1`, le 2026-11-28.

### T0 — reprise et relevé des prémisses (2026-11-28)

**Reprise.** `git merge origin/epic/53-ecritures-justes` (tête `843bcc27f`, 53-6a fusionnée) dans la branche à `6965ba802` : **aucun
conflit** — la branche de la story ne portait que `_bmad-output/` (fiche, prompts de validation, `sprint-status.yaml`, fichier d'epic) ;
aucun compteur commun n'a donc divergé (`sitesTotal` de `i18n-keys.test.ts` est celui de l'epic, recompté au T4). Commit de fusion seul
`a9b551d5a`, poussé.

**Prémisses relues sur la branche reprise** (numéros à `a9b551d5a` ; seuls changent ceux des fichiers que la 53-6a a touchés après
`6b71d4c21` : `settlement_cancellation.rs`, `journal_entries.rs` (±12 lignes, hors des sites cités), `jour_suisse_nuit_e2e.rs`,
`user-manual.tex`) :

- **P1 confirmée** : `CancellationDating` `journal_entries.rs:3192`, `From` `:3205`/`:3208`, `reverse_in_tx` `:3095`, `reverse_owned_in_tx`
  `:3169`, `reversal_date_verdict` `:3278`. Inchangés.
- **P2 confirmée, lignes décalées** : bras `OriginDate` de la queue `settlement_cancellation.rs:209-235` (non `:200-226`), bras `Today`
  `:237` (non `:228`). Têtes inchangées (`invoice_settlements_write.rs:876`, `:889` ; `supplier_invoices.rs:3330`, `:3356`) ;
  `supplier_invoice_cancel_blocker` `:3020`, `Today` en dur `:3063` : confirmé.
- **P3 confirmée** : `services/factures.rs:590` (`Today(aujourd_hui)`), `supplier_invoices.rs:3739` et `:3743`, `factures_fournisseur.rs:206`.
- **P4 confirmée fichier par fichier** : `grep -n aujourd_hui` sur `supplier_invoices.rs` hors tests — la date ne sert qu'au transit
  (`:1478/1482`, `:1498/1506`, `:1521/1527`, `:2463/2472`, `:3024/3063`, `:3125/3185/3221`, `:3305/3308/3309`, `:3633/3642/3645`,
  `:3694/3697`, `:3722/3739/3743`, `:3790/3800`). Routes : `supplier_invoices.rs:799/807/826/878`, `:920/928`, `:956/969`, `:992/1001`,
  `:1025/1032`, `:1064/1069` ; `imported_supplier_invoices.rs:139/147/173/251`, `:599/607/625/676`, `:710/717/734/767` — **aucun autre
  usage** dans ces fonctions (lu). `invoices.rs:1621` garde une lecture pour `from_parts` (`:1637`).
- **P5, P6, P7, P9 confirmées** (fichiers inchangés depuis `6b71d4c21`).
- **P8 confirmée au code** : `kesh-report/src/vat_report.rs:292` lit `invoice_settlements.write_off_vat` et `:342` joint
  `invoice_settlements s ON s.journal_entry_id = jel.entry_id` — le décompte part des pièces.
- **P10 confirmée** : `payment_batches::cancel_batch` (`:712`) ne crée ni ne contre-passe aucune écriture (aucun `create_in_tx`,
  `reverse_in_tx`, `reverse_owned_in_tx` dans le fichier).
- **D4 confirmée** (`pay` sans comparaison à `invoice_date`) ; **D6 confirmée** (`:2308-2315` seule création d'achat, `entry_date: invoice_date` ;
  `:1831` est un `Hint` ; `:2728` `entry_date: payment_date`).
- **D3, recompte** : `git grep -c` des signatures changées dans `*/tests/*` = **230 occurrences dans 35 fichiers** (confirmé), dont 183 passent
  `aujourd_hui_test()`.
- **AC-11, lignes décalées** (`jour_suisse_nuit_e2e.rs`, 53-6a) : `ac3_contre_passation_la_nuit_du_nouvel_an` `:622`, client `:641`,
  facture fournisseur `:659`, règlement fournisseur `:674`, rapprochement `:709`, `ac3_sans_exercice_au_jour_suisse_l_annulation_est_refusee`
  `:755`, `ac7_les_services_jugent_au_jour_suisse` `:940`. **Un test de plus que la fiche** :
  `ac3_sans_exercice_au_jour_suisse_l_annulation_d_un_paiement_fournisseur_est_refusee` (`:792`, ajouté par la revue P1 de la 53-6a, L12) —
  il change d'attendu comme le premier (AC-9 pour #1753). Tests `kesh-db` de l'AC-11 : numéros inchangés (`invoice_settlement.rs:1475`,
  `supplier_invoices_repository.rs:1762`, `:2076`, `:2209`, `lettering_documents.rs:814`, `:1530`, `open_items.rs:871`).
- **Issues** : #1752, #1753, #1754 `OPEN`, `bug`, `P1` (lecture seule, espacées).

### Completion Notes List

### File List

## Change Log

- 2026-11-27 — **Fiche créée** (Opus 5.5, `bmad-create-story`) : story de correction de la v0.21.0 (décision du Project Lead), Epic 63, trois
  P1 (#1752, #1753, #1754) ; réemploie le mécanisme de la 53-6a (`OriginDate`, rang `PeriodLocked`) et applique Q1 à Q6 ; tranche les
  différences entre gestes (D6), le retrait de la date du jour de la chaîne de vue fournisseur (D3), les textes sans variable (D5) et le maintien
  du rang 5 inatteignable (D7) ; l'AC-11 de la 53-6a s'inverse (AC-11). Cinq modules, sous la borne ; découpage de repli écrit. Questions Q-A à
  Q-D au Project Lead. Validation P1 (Sonnet) à lancer : `63-1-annulations-datees-origine-validation-prompt-p1.md`.
- 2026-11-27 — **Validation P1** (Sonnet, `bmad-create-story` validate ; rapport `kesh-gate-logs/e63/63-1-validation-p1.md`) : 0 CRITICAL, 0 HIGH, **2 MEDIUM
  corrigés** (V1 : D3/P4/T2 omettaient quatre enveloppes de `commit_with_view` — `validate`, `unvalidate`, `unvalidate_with_policy`, `pay` — et leurs routes,
  `_once` et 226 sites de tests *[annotation du 2026-11-27, P2 : 230 dans 35 fichiers, la commande omettant `cancel_in_tx`]* ; V2 : la garde d'AC-10 n'était
  pas constructible telle qu'écrite — jeton présent à la définition *[annotation du 2026-11-27, P2 : la définition `:3192` ne porte pas le jeton]*, à la conversion `:3208` et au
  `match` `:228`, et troncature à la première `#[cfg(test)]` (`journal_entries.rs:3322`) aveugle à R10), **4 LOW corrigés** (`:1831` n'est pas une création
  d'écriture d'achat ; `pay` accepte un paiement antérieur à la facture, montage précisé ; renvois R1–R6 ; chemin de `bank_account_closed_inventory.rs`).
  Q1 à Q6 de la 53-6a contrôlées appliquées. Décisions Q-A à Q-D du Project Lead portées (2026-11-27). Passe P2 (Opus) requise : MEDIUM corrigé.
- 2026-11-27 — **Validation P2** (Opus 5.5, `bmad-create-story` validate, contexte frais ; rapport `kesh-gate-logs/e63/63-1-validation-p2.md`) : 0 CRITICAL,
  0 HIGH, **1 MEDIUM corrigé**, **6 LOW corrigés**. Remède de la P1 contrôlé au code : les quatre enveloppes sont bien les seules (aucun autre appelant de
  production de `commit_with_view`, `get_settlement_view(_in_tx)`, `supplier_invoice_cancel_blocker` : MCP, `exports`, `inbox_import`, `kesh-seed` relus ;
  `supplier_invoice_response_e2e.rs`, `services_rollout.rs`, `date_du_jour_garde.rs`, `horloge_etat.rs` ne lisent aucune signature ni lecture
  d'horloge de ces routes). **M (V3, A5/A7)** : l'AC-11 ne nommait pas les tests qui changent d'attendu hors de `jour_suisse_nuit_e2e` —
  `ac7_les_services_jugent_au_jour_suisse` (`:941`), `lettering_documents::locked_period_supplier_cancels_are_refused_until_unlocked` (`:1675`,
  `PERIOD_LOCKED` après déverrouillage), les trois harnais de précédence qui lisent sous `Today` et cliquent par le geste public
  (`invoice_settlement.rs:1555`, `supplier_invoices_repository.rs:1806`, `:2102`) ; corrigé (AC-11, Dev Notes, méthode de clôture à T1/T2).
  **L** : recompte 226/34 → **230/35** (`cancel_in_tx` omis) ; AC-10 : inventaire clos par fonction englobante au lieu d'un tri syntaxique
  construction/motif, alias fermés, angle mort `#[cfg(test)] mod x;` écrit, définition `:3192` sans jeton ; T2 « sept enveloppes » remplacé par la
  liste ; AC-8 e nomme `locked_period_cancel_is_refused_until_unlocked` (`:814`) ; AC-12 nomme `a_concurrent_close_waits_for_the_invoice_cancellation`
  (`:2209`) et écrit la course avec `lock_books` ; issue `technical-debt` proposée (outil `lexique`). **Recyclage : non** — V3 est un défaut d'origine
  (présent dès la création, sur un axe que la P1 n'avait exercé qu'en partie, A7), distinct de V1 (signatures) et non né d'un correctif ; le seul
  défaut né du correctif de la P1 (le recompte) est LOW. Splitting : la cascade ne touche que des fichiers de test ; modules de production toujours
  cinq. Tendance : P1 0/0/2/6 → P2 0/0/1/6. Passe P3 requise (MEDIUM corrigé) : complète, Sonnet (`63-1-annulations-datees-origine-validation-prompt-p3.md`).
- 2026-11-27 — **Validation P3** (Sonnet 5.5, `bmad-create-story` validate, contexte frais, passe complète ; rapport `kesh-gate-logs/e63/63-1-validation-p3.md`) :
  0 CRITICAL, 0 HIGH, 0 MEDIUM, **5 LOW corrigés**. Remède V3 de la P2 contrôlé : les cinq tests de l'AC-11 changent bien d'attendu, la prescription des
  harnais (branche `Today` par la forme `_in_tx`, geste public sous `OriginDate`) garde la queue `Today` testée sans test vrai par construction ;
  recompte de D3 : 230 occurrences / 35 fichiers (confirmé). Montages dans les fonctions auxiliaires parcourus (`monter`, `facture_lettree`,
  `seed_lettering_documents`, `rattrapage`, `lettering_pieces`, helpers `monter`/`annuler`/`verrouiller` des fichiers de dépôt) : **un** test
  omis de l'AC-11, `open_items.rs::as_of_under_the_lock_is_stable_until_an_admin_unlocks` (LOW : la méthode de clôture le trouvait au premier rouge ;
  son attendu neuf est maintenant prescrit). Autres LOW : forme des mutations M1 à M6 (plus de paramètre de date après D3), quatre replis inline des
  deux pages Svelte non nommés à T4, commentaires de tests périmés (voir le rapport). Numéros d'issues portés (#1759, #1760).
  Axes A3, A6, A9, mutations M1 à M8 une à une, notes de version exercés. **Recyclage : non** (aucun défaut né d'un correctif ; tendance P1 0/0/2/6 →
  P2 0/0/1/6 → P3 0/0/0/5). Plus de passe requise : `ready-for-dev`.
