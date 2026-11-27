# Story 53-6 : Annuler un rapprochement contre-passe dans l'exercice de l'écriture, à sa date (#1674)

Status: ready-for-dev

*Fiche rédigée le 2026-11-27 (Opus 5.5), worktree `kesh-53-6`, branche `story/53-6-annulation-rapprochement-exercice` partie de
`epic/53-ecritures-justes` à `03bb9ac6d`, par le workflow `bmad-create-story`, en autonomie. Epic 53 « Écritures justes », cinquième story
(53-1a, 53-1b, 53-2, 53-3, **53-6**, solde 53-5) ; release **v0.21.0** (vendredi 2026-12-04). Priorité **P1** (#1674, CLO-2 : une
contre-passation tombe dans un autre exercice que son écriture, deux exercices sont faux). **Aucune migration.** Correctif décidé par le
Project Lead le 2026-11-26 (texte de l'issue, § « Correctif décidé ») ; les questions qu'il laisse ouvertes sont au § *Questions de produit*,
**pour décision de Guy**. ⚠️ **La fiche est écrite contre `origin/main` à `56d47b2cc`** (v0.20.0 en publication), non contre la branche
d'epic : voir § *Écarts entre `main` et la branche d'epic* — la branche de la story reprend `main` avant tout développement (T0).*

## Story

En tant que comptable qui défait un rapprochement fait par erreur — pour rattacher la transaction à la bonne pièce —, je veux que Kesh
contre-passe l'écriture **dans son propre exercice, à sa propre date**, et refuse clairement quand cet exercice est clôturé ou la période
verrouillée, afin qu'annuler puis refaire un rapprochement ne fausse jamais deux exercices ; et je veux pouvoir **retrouver** les
contre-passations que les versions précédentes ont datées dans un autre exercice, pour les corriger moi-même par une écriture.

## Étape 0 — prémisses relues au code (`origin/main`, `56d47b2cc`)

**P1 — la route lit la date du jour et la passe au geste (confirmée).** `crates/kesh-api/src/routes/reconciliation.rs:6270`
`let aujourd_hui = state.horloge.aujourd_hui();`, passée à `cancel_reconciliation_once` (`:6278`) puis à
`reconciliation_cancel::cancel_in_tx` (`:6307-6314`), sous `retry_app_on_deadlock("reconciliation::cancel", …)` (`:6271`) et
`with_account_lock`. La vue `GET /api/v1/reconciliation/transactions/{id}` passe aussi `state.horloge.aujourd_hui()` à `get_view`
(`:6149`), et la liste des écritures candidates (`GET …/entry-candidates`) à `entry_cancel_outlook` (`:6974`, `:6981`).

**P2 — les cinq bras du geste contre-passent tous « au jour » (confirmée ; l'issue en nomme quatre).**
`crates/kesh-db/src/repositories/reconciliation_cancel.rs` :
- écriture rapprochée seule : `:930-934` `journal_entries::reverse_in_tx(tx, company_id, entry_id, acteur, aujourd_hui)` ;
- règlement client : `:899-905` `invoice_settlements_write::cancel_settlement_in_tx(…, aujourd_hui)`, qui appelle
  `journal_entries::reverse_owned_in_tx(…, ReversalAuthority::ClientSettlement { settlement_id }, aujourd_hui)`
  (`invoice_settlements_write.rs:1128-1136`) ;
- règlement fournisseur : `:918` `supplier_invoices::cancel_settlement_in_tx(tx, company_id, id, acteur, aujourd_hui)`
  (`supplier_invoices.rs:3466`), qui appelle `reverse_owned_in_tx` (`:3558`) ;
- règlement groupé **client** : `:726-734` → `grouped_settlements::cancel_in_tx` (`grouped_settlements.rs:1753`), contre-passation
  `:1846-1848` `reverse_in_tx(…, aujourd_hui)` ;
- règlement groupé **fournisseur** : `grouped_settlements::supplier_cancel_in_tx` (`:1953`), contre-passation `:2030-2032`
  `reverse_in_tx(…, aujourd_hui)`.
Le paiement par carte Wise groupé (Story 39-11c, étape « 5 bis », `reconciliation_cancel.rs:844-881`, **présent sur `main` seulement**)
passe par le bras « écriture seule » : une contre-passation, plusieurs transactions remises en attente.

**P3 — la contre-passation n'a que deux datations, et celle du jour ignore l'exercice de l'origine (confirmée).**
`journal_entries.rs:2942-2953` (`main`) : `enum ReversalDating { Today(NaiveDate), NextFiscalYearStart }`, **privé** au module.
`reverse_in_tx` (`:2834`, `:2847`) et `reverse_owned_in_tx` (`:2916`, `:2930`) imposent `ReversalDating::Today(aujourd_hui)`.
`reverse_in_tx_inner` (`:3098`), étape (4), `:3249-3260` : `ReversalDating::Today(today) => find_open_covering_date(tx, company_id, today)`
→ date de l'inverse = `today`, exercice = l'exercice ouvert du jour. L'étape (1) (`:3106-3120`) verrouille déjà l'origine **et son
exercice** (`journal_entries je JOIN fiscal_years fy … FOR UPDATE`, qui lit `je.entry_number, fy.name, fy.end_date`) : la date de l'origine
n'y est pas lue, mais la même requête peut la lire (`je.entry_date`) **sans verrou de plus** — patron exact de la Story 29-1b (AC11), qui a
ajouté `fy.end_date` à cette requête pour `NextFiscalYearStart`.

**P4 — le défaut est plus large que « exercice rouvert » (établi au code, à dire au manuel).** Avec `Today`, toute annulation faite
**après la fin** de l'exercice de l'origine tombe dans un autre exercice, que celui-ci ait été clôturé puis rouvert **ou simplement pas encore
clôturé** — le cas le plus courant : en janvier, on défait le rapprochement d'un paiement de décembre avant la clôture. L'issue décrit le
premier cas ; le correctif couvre les deux, et l'outil de détection (D7) aussi. Une annulation faite **dans** l'exercice de l'origine, à une
date ultérieure, tombe dans le bon exercice mais dans une autre période (TVA trimestrielle) : elle ne fausse pas les comptes annuels ; voir Q4.

**P5 — un exercice d'origine clos est déjà refusé, rouvert ne l'est pas (confirmée).** Rang 2 de la queue commune,
`settlement_cancellation.rs:111-120` (`fy.status == "Closed"` → `SettlementCancelBlocker::FiscalYearClosed`) ; refusé par le geste à
`reconciliation_cancel.rs:817-825` (`DbError::ReconciliationNotCancellable { blocker }`), **après** le verrou de l'écriture et de son
exercice (étape 3, `:756-768`) ; filet du socle `journal_entries.rs:554` (`create_in_tx_inner`, `status == "Closed"` →
`DbError::FiscalYearClosed`). Un exercice rouvert a `status = 'Open'` : il passe. Message existant aux quatre locales :
`reconciliation-cancel-blocked-fiscal-year-closed` (`crates/kesh-i18n/locales/fr-CH/messages.ftl:957`, et `de-CH`, `en-CH`, `it-CH`).
Il reste juste après la story (« rouvrir les exercices clôturés jusqu'à celui-ci ») : une fois l'exercice rouvert, l'annulation se date dans
cet exercice.

**P6 — la réouverture est LIFO, la clôture dans l'ordre (confirmée) : un exercice d'origine ouvert n'a jamais de postérieur clos.**
`fiscal_years.rs` : `reopen` refuse si un exercice de date de début postérieure est clos (`FY_REOPEN_LIFO_BLOCKED_KEY`, `:246`) ; `close`
refuse si un antérieur est ouvert (`EARLIER_FISCAL_YEAR_OPEN`). Une contre-passation datée dans l'exercice de l'origine, ouvert, ne rencontre
donc `LATER_FISCAL_YEAR_CLOSED` (`journal_entries.rs:578-584`) que dans l'**état hérité** (restauration, version antérieure ; manuel
d'administration, `admin-manual.tex:1673`) — refus du socle, déjà au manuel (`user-manual.tex:1121`, #568).

**P7 — le verrou de période devient un motif courant (établi au code ; c'est la conséquence principale du correctif).**
`create_in_tx_inner`, étape 1-bis, `journal_entries.rs:599-625` : `new.entry_date <= books_locked_through` → `DbError::PeriodLocked`
(seuil **inclusif**). Le commentaire `:605-615` le dit : la contre-passation du jour traverse cette garde **parce que** sa date est celle du jour
et que la borne est strictement antérieure (`companies::lock_books`, `through >= aujourd_hui` refusé) ; « la contre-passation au début de
l'exercice suivant … est alors refusée ici, en `PeriodLocked`, comme toute écriture ». Datée à l'origine, une annulation de rapprochement dont
l'écriture est **sous la borne** sera refusée. Aujourd'hui, la queue commune ne l'évalue pas : « ⚠️ Limite assumée : le verrou de période
**du jour** (24-4c) n'est pas évalué ici — il se contrôle au clic » (`settlement_cancellation.rs:50-52`). Ce qui était un angle mort
improbable devient un refus fréquent (une période TVA déclarée est typiquement verrouillée : `admin-manual.tex:3559`) : l'écran doit
l'annoncer (D5). ⚠️ Le **re-rapprochement** de la transaction écrit une écriture datée de la transaction, donc sous la même borne : il serait
refusé lui aussi. Kesh ne permet donc pas de « refaire » un rapprochement **qui crée une écriture** dans une période verrouillée sans la déverrouiller.
⚠️ **Établi à la validation P1 (M4), et plus étroit que la fiche ne le disait** : `routes/reconciliation.rs:3044` (`entry_date: paid_at_candidate`)
et `invoice_settlements_write.rs:370`, `:751` (`entry_date: settled_on`) datent l'écriture d'une acceptation de facture à la date de la transaction ;
mais le **rattachement à une écriture existante** (`reconciliation_link.rs:588-679`, `days_apart` : « trie, ne filtre pas ») **ne crée aucune
écriture** et garde celle de l'écriture rattachée (manuel, `user-manual.tex:369-370`) : il n'est donc **pas** refusé par la borne. La phrase
« le re-rapprochement serait refusé de toute façon » vaut pour l'acceptation (facture, règle, rapprochement manuel, éclatement — à relire à T0
pour règle, manuel et éclatement, seule l'acceptation de facture l'est ici), non pour le rattachement.

**P8 — Kesh ne tient aucun décompte de TVA déposé (vérifié).** `git grep -i "submitted\|declared\|vat_returns\|vat_declarations"` sur
`crates/kesh-db/src/repositories/` et `routes/vat.rs` : rien ; migrations TVA : taux et comptes seulement. Le décompte se calcule des
lignes par date. Une contre-passation datée dans une période **déclarée mais non verrouillée** change donc ce décompte sans signal : seul le
verrou de période protège une période déclarée (texte voisin déjà au catalogue : `supplier-invoices-unvalidate-confirm-body-cancelled`,
`messages.ftl:3340`, « si l'une a déjà été déclarée, faites d'abord verrouiller la période »).

**P9 — une contre-passation ne se contre-passe pas, une écriture contre-passée non plus (confirmée) : la correction des livres touchés se
saisit à la main.** `ReversalBlocker::IsAReversal` et `AlreadyReversed` (`errors.rs:272`, variantes 1 et 2) ; `reversed_by` dérivé de
l'`UNIQUE` de `reverses_entry_id` (`journal_entries.rs:2784`). Ni l'origine E ni son inverse R ne peuvent être contre-passées de nouveau
par Kesh : la correction (D8) passe par deux écritures OD saisies.

**P10 — la piste d'audit distingue l'annulation de rapprochement des autres contre-passations (confirmée).** `reconciliation_cancel.rs:
974-1006` : `reconciliation.cancelled`, entité `bank_transaction`, détails `kind`, `matchedEntryId`, `reversalJournalEntryId` (nul pour un
retrait seul), `unlinkOnly`, `invoiceId`, `settlementId`, `supplierInvoiceId` ; pour un paiement Wise groupé, une entrée par transaction
remise en attente, **même** `reversalJournalEntryId`, `groupedWith` (`:1008-1028`). Une contre-passation de `/journal-entries/{id}/reverse`
ou d'une annulation de règlement depuis la fiche n'a pas cette entrée. C'est le critère de la requête de lecture que le Project Lead a lancée
sur sa base (dossier hors dépôt ; **aucun résultat n'en est recopié ici**) et celui de l'outil de détection (D7).

**P11 — l'API documente « datée du jour » (confirmée).** `docs/api-external.md:992` (§ *Annuler un rapprochement bancaire*, `:988`) :
« contre-passe l'écriture du rapprochement (**datée du jour**) » ; `:677` et `:788` pour les annulations depuis la fiche (hors périmètre,
Q3). Manuel : `user-manual.tex:4082-4083` (« une écriture inverse, datée du jour »), `:4144` (« aucun exercice ouvert ne couvre la date du
jour : créez-le »), `:863-866` (encadré `sec:date-du-jour` : « la contre-passation et les annulations qui contre-passent (règlement, facture
fournisseur, rapprochement) »). Écran : `reconciliation-cancel-confirm-entry`, `-confirm-invoice`, `-confirm-supplier-invoice`
(`messages.ftl:964-966`), `-confirm-group`, `-confirm-group-supplier` (`:1850-1851`), « Une écriture inverse datée d'aujourd'hui sera
passée », et `reconciliation-cancel-blocked-no-fiscal-year` (`:961`), aux quatre locales ; fallback en dur
`crates/kesh-api/src/errors.rs:6958`.

### Écarts entre `main` et la branche d'epic (`03bb9ac6d`)

Base commune `fbfa64a27`. `main` a reçu depuis les Epics 39 (Wise) et 46 (rapports) et la PR #1615 ; la branche d'epic, les 53-1a, 53-1b et
53-3. Sur les fichiers de la story :

| Fichier | `main` (`56d47b2cc`) | branche d'epic |
|---|---|---|
| `reconciliation_cancel.rs` | **+115 lignes** : étapes « 5 bis » et « 7 bis » (paiement Wise groupé, `PROVIDER_GROUP_SIBLING`, `provider_group_siblings`) | sans elles |
| `settlement_cancellation.rs` | rang 3 exempte les lignes sœurs Wise (`:145-160`) | sans |
| `supplier_invoices.rs` | `DuplicatePolicy` réexporté de `duplicate_policy` | défini sur place (53-1a l'a déplacé — à réconcilier à la reprise) |
| `journal_entries.rs` | `reverse_in_tx` `:2834`, `ReversalDating` `:2942`, `reverse_in_tx_inner` `:3098`, étape (4) `:3252` | mêmes fonctions à `:3093`, `:3170`, `:3357`, `:3511` ; `create_in_tx_inner` y prend un paramètre de plus (53-1a, D2 : `None` à l'appel de la contre-passation, `:3580`) |
| `reconciliation.rs` (route), `grouped_settlements.rs`, `invoice_settlements_write.rs` | identiques | identiques |

Conséquence : **T0** reprend `main` dans la branche de la story (ou attend que la branche d'epic l'ait reprise après la publication de la
v0.20.0, CLAUDE.md § *Branches* — décision de l'orchestrateur), puis relit les numéros de ligne de cette fiche. Le bras Wise (AC-7) n'existe
que là.

## Inventaire clos des contre-passations qui passent par le dé-rapprochement

Commande : `git grep -n "reverse_in_tx(\|reverse_owned_in_tx(\|cancel_settlement_in_tx(\|grouped_settlements::cancel_in_tx(" origin/main --
crates`, hors tests et doc-comments, puis lecture de chaque appelant.

| # | Site | Appelé par | Datation après la story |
|---|---|---|---|
| R1 | `reconciliation_cancel::cancel_in_tx`, bras `Entry` (`:930`) | dé-rapprochement | **origine** (D1) |
| R2 | idem, bras `InvoiceSettlement` → `invoice_settlements_write::cancel_settlement_in_tx` (`:899`) | dé-rapprochement | **origine** |
| R3 | idem, bras `SupplierSettlement` → `supplier_invoices::cancel_settlement_in_tx` (`:918`) | dé-rapprochement | **origine** |
| R4 | `grouped_settlements::cancel_in_tx` (`:1846`) | dé-rapprochement d'un groupe client | **origine** |
| R5 | `grouped_settlements::supplier_cancel_in_tx` (`:2030`) | dé-rapprochement d'un groupe fournisseur | **origine** |
| R6 | bras `Entry` + « 5 bis » (Wise groupé) | dé-rapprochement | **origine** (= R1) |
| R7 | `invoice_settlements_write::cancel_settlement` (`:1253`) | `POST /invoices/{id}/settlements/{sid}/cancel` (fiche) | **jour, inchangé** (Q3) |
| R8 | `supplier_invoices::cancel_settlement` (`:3663`) | `POST /supplier-invoices/{id}/settlement/cancel` (fiche) | **jour, inchangé** (Q3) |
| R9 | `supplier_invoices` `:3243` (achat, `SupplierPurchase`) | `POST /supplier-invoices/{id}/cancel` | **jour, inchangé** (Q3) |
| R10 | `journal_entries::reverse` (`:3432`) | `POST /journal-entries/{id}/reverse` | **jour, inchangé** — sujet de la contre-passation sans date choisie (Epic 38), distinct |
| R11 | `reverse_at_next_fiscal_year_start*` | `/reverse-at-next-fiscal-year-start` | inchangé |

Sites **non résolus** par la story : R7 à R9 — même classe de défaut (une annulation faite après la fin de l'exercice de l'écriture tombe
dans un autre exercice), **hors périmètre de #1674** ; chacun doit avoir son issue (texte au § *Issues à ouvrir*), Q3. Le paramètre de
datation que la story ajoute (D2) les rendra corrigibles par un seul argument. R10, R11 : non concernés (datation choisie par le geste).

## Questions de produit — pour décision de Guy

Le Project Lead a tranché le 2026-11-26 le cœur du correctif (origine ouverte → date de l'origine ; origine close → refus). Restent :

- **Q1 — Confirmer la date de l'origine même quand l'annulation se fait dans le même exercice.** La décision du 2026-11-26 dit « datée dans
  cet exercice, à la date de l'écriture d'origine », et ses tests nomment « exercice ouvert de l'année courante » : la règle vaut donc aussi
  pour une annulation faite en novembre d'un rapprochement de mars, qui serait aujourd'hui datée de novembre. *Variante écartée* : date du
  jour quand le jour est dans l'exercice de l'origine, date de l'origine sinon. **Recommandation : garder la date de l'origine partout.**
  Raisons : (1) annuler un rapprochement dit que l'écriture n'aurait jamais dû exister ; l'écriture et son inverse à la même date s'annulent
  dans **chaque** état à **chaque** date — solde du compte bancaire au jour, décompte de TVA de la période, grand livre ; (2) le
  re-rapprochement qui suit passe la bonne écriture à la date de la transaction, donc dans la période de l'erreur : dater l'inverse du jour
  laisse cette période comptée deux fois (l'erreur et la bonne écriture) et la période courante avec l'inverse seul ; (3) une période déjà
  déclarée doit être verrouillée (P8), et le verrou fait alors refuser (Q2). Conséquence à accepter : dans une période déclarée **non**
  verrouillée, le décompte change ; le dialogue de confirmation le dit (AC-10). Autre conséquence, à dire (validation P1, L8) : l'inverse
  datée de l'origine reçoit le **prochain numéro** du compteur de l'exercice (`next_number_for`, `journal_entries.rs:670-671`), donc un numéro
  postérieur à des écritures de date postérieure : l'ordre des numéros ne suit plus celui des dates (déjà vrai d'une saisie manuelle antidatée ;
  le manuel du § *Annuler un rapprochement* le dit en une phrase).
- **Q2 — Période verrouillée : refuser, ou dater au premier jour libre ?** L'écriture d'origine est sous la borne du verrou (P7). (a)
  **Refus** `PERIOD_LOCKED`, annoncé d'avance par l'écran, rien n'est écrit ; un administrateur recule la borne (motif obligatoire), on
  annule, on refait le rapprochement, il repose la borne. (b) Dater au lendemain de la borne, s'il est dans l'exercice de l'origine
  (sinon refus). (c) **Garder la date du jour quand l'origine est sous la borne** (inverse dans la période courante, comme aujourd'hui) — c'est le
  principe que le manuel énonce pour toute écriture d'une période verrouillée : « Une écriture d'une période verrouillée reste corrigeable
  par contre-passation. La contre-passation est datée du jour, donc après le verrou : elle passe. […] Le verrou arrête ce qui entre dans le
  passé, jamais ce qui corrige depuis le présent. C'est cette distinction qui empêche le verrou de rendre vos livres incorrigibles »
  (`user-manual.tex:826-836`, encadré « Ce que le verrou n'empêche pas, et c'est voulu »). Elle rétablit l'asymétrie que (a) supprime, mais
  refait le défaut de #1674 dans une période verrouillée (l'inverse tombe dans un autre exercice si l'origine est d'un exercice antérieur) :
  à écarter si le Project Lead tient #1674 pour premier, à retenir si c'est le principe du manuel. **Recommandation : (a), refus,** en
  disant à Guy qu'elle **contredit ce principe du manuel pour le seul dé-rapprochement** (l'AC-15 corrige l'encadré en conséquence : la
  contre-passation depuis la fiche d'une écriture **non rapprochée** reste datée du jour et passe ; l'écriture rapprochée est refusée par
  la contre-passation directe, `MATCHED_BANK_TRANSACTION`). Raisons : le re-rapprochement **par acceptation** serait refusé de toute façon
  sous la même borne (P7, corrigé) — avec (b) on aurait une inverse hors de la période de l'erreur **et** une transaction qu'on ne peut pas
  accepter à sa date ; et c'est déjà la règle de la contre-passation au début de l'exercice suivant (`journal_entries.rs:612-615`). Contre-
  argument à peser : le rattachement à une écriture existante, lui, reste possible sous la borne. ⚠️ **Effet** : une
  annulation qui passe aujourd'hui (datée du jour, hors période verrouillée) sera refusée après la story. Annoncé ⚠️ aux notes de version.
- **Q3 — Les autres annulations qui datent du jour (R7 à R9) : même défaut, hors de cette story ?** Annuler depuis la fiche le règlement
  d'une facture client, le paiement d'une facture fournisseur, ou annuler une facture fournisseur, après la fin de l'exercice de l'écriture,
  date l'inverse dans un autre exercice que l'origine. **Établi au code à la validation P1 (M6), ligne lue, les trois affirmations sont vraies** :
  R7 `routes/invoices.rs:1619` `let aujourd_hui = state.horloge.aujourd_hui();` → `cancel_settlement(…, aujourd_hui)` (`:1624-1631`) →
  `invoice_settlements_write.rs:1128-1136` `reverse_owned_in_tx(…, ClientSettlement { settlement_id }, aujourd_hui)` ; R8
  `routes/supplier_invoices.rs:1064-1069` → `supplier_invoices.rs:3558` ; R9 `routes/supplier_invoices.rs:1025-1032` → `supplier_invoices.rs:3243`
  (`SupplierPurchase`) — tous trois aboutissent à `ReversalDating::Today` (`journal_entries.rs:2930`) et à `find_open_covering_date(today)`
  (`:3254-3257`). **L'effet sur les comptes n'est pas le même partout**, ce qui commande la priorité des issues : R9 contre-passe l'écriture d'**achat**
  (charges, TVA récupérable, créanciers) : la charge et l'impôt préalable restent dans N, l'inverse crédite l'exercice courant — résultat et
  décompte de TVA faux dans **deux** exercices (P1). R7 et R8 contre-passent une écriture de **règlement** : banque et débiteurs (ou créanciers)
  seulement, des comptes de bilan dont le solde se cumule — les soldes cumulés sont justes, mais le bilan de N montre un paiement que l'annulation
  défait en N+1, et **le résultat est faux dès que l'écriture porte un écart** (escompte, perte, correction de TVA : le « solde » d'une facture
  s'annule par la même route, `invoices-write-off-cancel-confirm`, TVA comprise) ; sans écart, le résultat et la TVA restent justes, mais le
  **solde de clôture** de la banque et des tiers dans N est faux (priorité : voir la recommandation, validation P2). Aucune de ces trois annulations n'est atteinte par le dé-rapprochement. Les inclure ajoute les domaines `frontend/…/invoices` et `…/supplier-invoices` (textes « datée
  d'aujourd'hui ») et les routes des factures : la story passerait de quatre à six modules (découpage obligatoire) et l'epic a déjà six
  stories. **Recommandation : trois issues, une par geste, toutes trois P1**, ouvertes par l'orchestrateur dès la décision ; la story pose le
  paramètre de datation qui les rend triviales. ⚠️ **Validation P2 (M2)** : la proposition de la P1 (« R7 P1 s'il y a écart, sinon P2 ; R8 P2 sauf
  écart ») est écartée, pour deux raisons. (1) Une priorité se pose **par issue**, donc par geste, non par cas : le geste R7 annule aussi un
  **solde** (escompte, perte, TVA reprise), il peut donc fausser le résultat et la TVA — l'issue est P1. (2) « Bilan seul » n'est pas « sans
  effet sur les comptes » : CLAUDE.md § *Priorités des défauts* range en P1 « solde, compte ou total faux ». Exemple pour R8 comme pour R7 :
  paiement de 100.– de décembre N imputé à la mauvaise facture, annulé en janvier N+1 (inverse datée du jour, dans N+1) puis refait à sa date
  (dans N) — la banque de N porte +200.– au lieu de +100.–, celle de N+1 −100.– ; le cumul est juste, **le solde de clôture de N est faux**.
  Conséquence à dire à Guy : trois P1 de plus, que la règle « tout P1 passe en priorité dans la release suivante » place dans la release qui
  suit la v0.21.0 (ou dans celle-ci, si Guy élargit la story — découpage obligatoire, ci-dessus). *(Une seule exception à discuter : si Guy juge que
  l'annulation depuis la fiche doit rester datée du jour — correction de l'exercice courant d'une erreur d'un exercice clos, ce qui se
  défend —, les issues deviennent sans objet ; la story ne préjuge rien.)*
- **Q4 — Outil de détection des livres déjà touchés : quelle forme, quelle portée ?** (a) Une route de lecture et un **panneau** sur la page
  des imports bancaires, visible seulement s'il y a un cas, qui liste chaque écriture et son inverse avec leurs exercices et renvoie à la
  procédure du manuel (D7) ; (b) seulement la procédure et la requête de lecture au manuel d'administration ; (c) rien de plus que la
  requête déjà remise au Project Lead. Portée : les seules contre-passations d'annulation de rapprochement **dans un autre exercice** que
  leur origine, ou aussi celles du même exercice mais d'une autre période (P4) ? **Recommandation : (a), portée « autre exercice »
  seulement.** Raisons : Guy n'est pas le seul utilisateur (contributeurs, futures installations) et une requête SQL n'est pas un outil
  d'utilisateur ; un décalage de période dans le même exercice ne fausse pas les comptes annuels, et la concordance annuelle de la TVA
  l'absorbe ; l'élargir listerait comme « à corriger » des annulations que rien n'oblige à corriger. *(Si (b) est retenu : AC-14 et AC-15
  se réduisent au manuel, la story perd le panneau et sa route, et reste à trois modules.)* ⚠️ **À trancher aussi (validation P1, M8)** : rien
  ne dit quand un cas **quitte** le panneau — la correction du manuel (D8) pose deux écritures OD que ni l'audit ni le journal ne relient au cas ;
  le panneau listerait le même cas à jamais. Options : (a1) un **acquittement** par cas, tracé au journal d'audit (action neuve, entité
  `bank_transaction`, route `POST` mutante à inscrire à `audit_route_registry.rs`, aucune migration) et exclu par la requête ; (a2) aucun
  acquittement, le panneau est **une aide à la correction** et le dit (« tant que l'inverse reste dans un autre exercice que l'écriture ») ;
  (a3) le cas disparaît quand l'origine et l'inverse sont l'un et l'autre **contre-passés ou lettrés avec une écriture de correction**, ce que
  Kesh ne sait pas lire. ⚠️ **Validation P2 (M3)** : sous (a2), le cas **ne quitte jamais** le panneau — la correction D8 ajoute deux écritures OD
  et ne déplace pas l'inverse R, qui reste dans l'exercice M (P9 : ni E ni R ne se contre-passent). La formule proposée par la P1 (« tant que
  l'inverse reste dans un autre exercice ») laissait croire à une sortie qui n'arrive pas : elle est retirée. Le texte du panneau sous (a2) dit
  au contraire : « Kesh ne peut pas savoir si ce cas a été corrigé : il reste listé après correction. » Coût à peser : un panneau **permanent**
  sur la page des imports pour toute installation touchée (celle de Guy l'est), que l'on finit par ne plus lire. **Recommandation : (a2) pour
  cette story, en le disant ainsi**, et l'acquittement (a1) en issue — il ajoute une route mutante et un geste d'écriture à une story de
  lecture ; si Guy juge le panneau permanent inacceptable, (a1) entre dans la story (un module de plus : non, même domaine ; une route de plus
  à `audit_route_registry.rs`), ou Q4 (b).
- **Q5 — Le changement de date au regard des versions de l'API.** `docs/api-external.md:992` documente « datée du jour ». La forme de la
  réponse ne change pas ; la vue gagne deux champs (ajout) et `cancelBlockedBy` une valeur (`PERIOD_LOCKED`, ajout) ; la **date écrite**
  change, et des annulations jusqu'ici acceptées sont refusées (Q2). **Recommandation : pas de `/api/v2`** — c'est la correction d'un
  défaut qui écrivait faux, annoncée ⚠️ en tête des notes de version, comme le Project Lead l'a demandé. ⚠️ **Deux précisions (validation P1,
  L4)** : (1) `docs/api-external.md:992` **documente** « datée du jour » : le contrat écrit change, il n'est pas rétabli — le cas « correction d'un
  comportement contraire au contrat documenté » de § *Versions de l'API* ne s'applique qu'une fois le texte du contrat dit faux par le Project
  Lead (décision du 2026-11-26), à écrire comme tel ; (2) la fiche disait « la release monte `b` de toute façon (migration de la 53-2) » : la 53-2
  est `backlog` (« migration probable »), donc **rien n'assure** cette migration ; c'est la **modification du contrat** (date écrite, refus
  neuf) qui porte `b` par § *Numérotation des versions*. Décision à écrire au Change Log.
- **Q6 — Exercice d'origine clos et comptes déposés : la correction (D8) se fait-elle quand même ?** Pour un cas déjà dans les livres dont
  l'exercice d'origine a été **clôturé et ses comptes approuvés**, rouvrir pour corriger change des comptes déposés. **Recommandation :
  le manuel dit de ne pas rouvrir** — l'inverse tombée dans l'exercice suivant y vaut alors correction d'un exercice antérieur, à laisser,
  éventuellement commentée par une écriture de texte ; il ne propose la correction que pour un exercice d'origine **ouvert** (jamais clôturé,
  ou rouvert) ou clôturé sans comptes déposés. Le choix reste à l'utilisateur et à sa fiduciaire ; Kesh ne corrige rien d'office.

## Décisions

- **D1 — La datation du dé-rapprochement.** Exercice de l'écriture d'origine **ouvert** (rouvert compris) : la contre-passation est datée
  de la **date de l'écriture d'origine**, dans **son** exercice. Exercice **clos** : refus, motif `FISCAL_YEAR_CLOSED` existant (rang 2),
  inchangé. **Période verrouillée** (date de l'origine ≤ borne) : refus `PERIOD_LOCKED` (Q2 a). Vaut pour les cinq bras R1 à R6. Le
  retrait seul d'un côté de virement (`unlink_only`) n'écrit rien et ne change pas ; le rang `PeriodLocked` ne s'évalue donc pas pour lui
  (AC-8). **Devises** (validation P1, L5) : l'inverse recopie montant, cours et provenance de l'origine (`journal_entries.rs:3216-3234`) et se
  juge désormais dans l'exercice **de l'origine** : `controler_lignes` rang 2 (`journal_entry_fx.rs:90`, compte en devise dans un exercice
  mono-devise) ne peut plus refuser ce que l'origine a déjà passé ; une **réévaluation de fin d'exercice déjà passée** dans l'exercice de
  l'origine ne se recalcule pas — l'inverse change le solde en devise sur lequel elle portait : le manuel (§ *Annuler un rapprochement*) dit de
  la rejouer après l'annulation (aucune garde ne l'y oblige, aucun refus nouveau). *Écartées* : variante de Q1, variante (b) de Q2.
- **D2 — Une datation, nommée, dans le socle — jamais une seconde contre-passation.** `ReversalDating` gagne une troisième valeur,
  `OriginDate`, et devient **`pub`** (validation P1, M2 : `pub(crate)` ne suffit pas — `kesh-api/src/services/factures.rs:585` appelle
  `settlement_cancel_blocker`, les tests d'intégration de `kesh-db` (`crates/kesh-db/tests/`, crates distincts) et de `kesh-api` appellent
  `reverse_in_tx`, `cancel_settlement_in_tx` ou leurs blocages, et une `pub fn` ne peut porter un type `pub(crate)` dans sa signature —
  `private_interfaces`, rouge sous `-D warnings`). ⚠️ **Validation P2 (L1)** : exposer l'enum tel quel rend `NextFiscalYearStart` passable à
  `reverse_owned_in_tx`, **avec une autorité** — une combinaison que le socle n'a jamais connue (« aucune annulation de pièce ne date au début
  de l'exercice suivant », doc de `reverse_at_next_fiscal_year_start_in_tx`, `journal_entries.rs:2964-2966`) ; la phrase « sans conséquence,
  faute d'autorité » que portait la fiche était fausse pour cette entrée. Deux remèdes, au choix du développeur, écrit au Dev Agent Record :
  (i) **un type public étroit** pour les deux entrées, `pub enum CancellationDating { Today(NaiveDate), OriginDate }`, converti en
  `ReversalDating` (qui reste privé) — rien n'est exposé de plus ; (ii) `ReversalDating` `pub`, et `reverse_owned_in_tx` refuse
  `NextFiscalYearStart` par `DbError::Invariant` (test unitaire). (i) est préféré. Les **trois** entrées publiques restent
  (`reverse_in_tx`, `reverse_owned_in_tx`, `reverse_at_next_fiscal_year_start_in_tx`) ; aucune n'est ajoutée. Étape (1) de `reverse_in_tx_inner` : la requête
  verrouillante existante lit **aussi** `je.entry_date` **et `je.fiscal_year_id`** (celui-ci pour la confirmation `fy.id == origin_fy_id` ;
  aucune ligne de plus, patron 29-1b AC11 — la requête actuelle ne lit ni l'un ni l'autre, `journal_entries.rs:3110`). Étape (4), bras `OriginDate` : `fy = find_open_covering_date(tx, company_id, origin_date)` — lecture **verrouillante**
  qui **confirme** l'exercice ouvert —, `None` → `DbError::FiscalYearClosed` (l'origine est dans un exercice clos : le rang 2 l'a déjà
  refusé ; filet), et `fy.id != origin_fy_id` → `DbError::Invariant` (les exercices ne se chevauchent pas : patron de `NextFiscalYearStart`,
  `:3280-3287`). Date de l'inverse : `origin_date`. Le reste — `create_in_tx_inner` et ses refus (dont `PeriodLocked`), lettrage `reversal`
  sous `Mode::System { held_open_fiscal_year_id: fy.id }`, relecture des lignes — est **commun**. Le libellé redevient « Contre-passation
  écriture n° N » sans suffixe d'exercice (même exercice : la règle `:3293-3297` le donne sans changement). Audit `journal_entry.reversed` :
  `{"reversalJournalEntryId": …, "dating": "originDate"}` (le chemin `Today` garde son JSON octet pour octet, T-b2 de la 29-1b).
  Signatures : la paire d'entrées publiques des annulations (`reverse_in_tx`, `reverse_owned_in_tx`) prend la datation en
  paramètre (`Today(aujourd_hui)` ou `OriginDate`) au lieu de `aujourd_hui` ; `invoice_settlements_write::cancel_settlement_in_tx`,
  `supplier_invoices::cancel_settlement_in_tx`, leurs `*_cancel_blocker*` et `settlement_entry_cancel_blocker` prennent la même datation ;
  les gestes depuis la fiche (R7, R8) passent `Today(aujourd_hui)`, le dé-rapprochement `OriginDate`. *Écartée 1* : une fonction
  `reverse_at_origin_date_in_tx` à côté des deux autres — le socle a été unifié (25-3-zero, 29-1b) précisément pour ne pas avoir de second
  chemin. *Écartée 2* : la route calcule la date et la passe en `Today(date_origine)` — `Today` irait alors chercher l'exercice par la date
  sans savoir qu'elle doit être celle de l'origine ; la confirmation `fy.id == origin_fy_id` disparaîtrait.
  ⚠️ **Limite à écrire d'avance (validation P1, M5)** : sur le chemin du dé-rapprochement, l'exercice de l'origine est déjà verrouillé par
  l'étape (3) de `cancel_in_tx` (et (2-bis) des gestes de règlement) **avant** toute lecture non verrouillante ; une clôture concurrente
  attend ce verrou, et l'instantané ne se fige qu'après lui. Une lecture **ordinaire** de l'étape (4) verrait donc le même état que la
  lecture verrouillante : la mutation M7 ne se tue vraisemblablement par aucun entrelacement jouable. La confirmation reste (patron de la
  29-1b, défense de tout appelant futur qui n'aurait pas verrouillé avant) ; M7 est écrite **angle mort assumé** dès la fiche, sauf si le
  développement établit un entrelacement qui la tue.
- **D3 — Verrous : l'ordre ne change pas, l'ensemble diminue.** Le dé-rapprochement tient déjà, dans l'ordre, la transaction bancaire, la
  ligne de règlement et la facture, l'écriture **et son exercice** (étape 3, `reconciliation_cancel.rs:756-768`) ; `reverse_in_tx_inner`
  re-verrouille l'origine et son exercice (même ligne, déjà tenue), puis le parcours de `find_open_covering_date` part du premier exercice
  de la société et s'arrête à **celui de l'origine** au lieu de celui du jour : il verrouille **moins** de lignes quand l'annulation est faite
  plus tard (le cas du défaut), jamais d'autre table. C'est l'ordre « exercice de l'origine, puis celui de la contre-passation » que la
  décision du 2026-11-26 rappelle des 53-1 et 53-3 — ici le même exercice. Les cycles connus (clôture de N, création d'un exercice antérieur
  ou postérieur ; doc-comment de `reverse_in_tx_inner`, `:3002-3032`) restent de la même classe et se résolvent par le rejeu déjà en place
  (`retry_app_on_deadlock`). ⚠️ À établir au développement, non à supposer : la mesure (AC-12) des verrous pris, avant et après, pour une
  origine en N et une horloge en N+1.
- **D4 — La vue annonce la date et l'exercice de la contre-passation.** `get_view` / `GET /api/v1/reconciliation/transactions/{id}` gagne
  `reversalDate` (la date de l'origine, `null` pour un retrait seul ou une transaction non rapprochée) et `reversalFiscalYearName` ; ajout
  de champs, contrat additif. Le dialogue de confirmation dit « Une écriture inverse datée du { $date } (date de l'écriture du rapprochement)
  sera passée dans l'exercice { $fiscalYear } » à la place de « datée d'aujourd'hui », dans les cinq messages de confirmation
  (`-confirm-entry`, `-confirm-invoice`, `-confirm-supplier-invoice`, `-confirm-group`, `-confirm-group-supplier`), aux quatre locales, et
  ajoute, quand la date est antérieure au premier jour du mois courant, une phrase sur la TVA : « Les totaux de cette période changent, TVA
  comprise : si elle a déjà été déclarée, faites d'abord verrouiller la période. » *(Le seuil « mois courant » est une proposition : un seuil
  plus juste demanderait de connaître la périodicité de TVA, que Kesh ne tient pas ; à défaut, la phrase se montre toujours. Au développeur
  de choisir l'une des deux et de l'écrire au Dev Agent Record.)* ⚠️ **Validation P2 (L4)** : le seuil « mois courant » exige le jour, que D6
  retire de la vue (`get_view` ne lit plus l'horloge). S'il est retenu, il se calcule à l'écran par l'utilitaire du jour de Zurich
  (`frontend/src/lib/shared/utils/local-date.ts`), inscrit à l'inventaire de `local-date-guard.test.ts` — jamais par un `new Date()` nu —, et
  l'horloge de test du serveur n'y est pas vue (un E2E sous `KESH_TEST_MODE` ne peut pas le figer) ; sinon, la phrase se montre toujours.
- **D5 — Le verrou de période devient un rang de la queue commune, pour la datation `OriginDate` seulement.** Nouvelle variante
  `SettlementCancelBlocker::PeriodLocked` (code `PERIOD_LOCKED`, celui de `DbError::PeriodLocked`, « même état du monde », patron du rang
  2 bis) ; `SettlementCancelHit` porte la borne en étiquette (`Some(locked_through.to_string())`, patron du compte archivé au rang 4).
  **Place** : celle de l'ordre réel du socle — après `AccountArchived` (étape 3 du socle) et à la place du rang 5 (`NoOpenFiscalYearToday`),
  qui devient **inatteignable** pour `OriginDate` (l'exercice de l'origine, ouvert au rang 2, couvre sa date) et reste celui de `Today`. Son code
  est **`FISCAL_YEAR_INVALID`** (`kesh-db/src/errors.rs:800` ; la fiche l'écrivait `NO_OPEN_FISCAL_YEAR_TODAY`, qui n'existe nulle part :
  `git grep -n NO_OPEN_FISCAL_YEAR_TODAY origin/main` rend vide, validation P1, M1).
  Lecture **non verrouillante** de `companies.books_locked_through` (même tolérance que l'étape 0-bis du socle, `journal_entries.rs:517-
  536`). **Qui refuse au clic** : le socle (`create_in_tx_inner`, 400 `PERIOD_LOCKED`), « une seule garde par motif » (`errors.rs:672-675`) ;
  le geste ne le refuse pas lui-même. Écran : clé `reconciliation-cancel-blocked-period-locked`, **sans variable** (« L'écriture de ce
  rapprochement est dans une période verrouillée : pour pouvoir l'annuler, un administrateur doit faire reculer le verrou avant sa date. »),
  quatre locales, la borne **ajoutée par l'écran** comme le numéro du compte archivé (`(${label})`, patron d'`ACCOUNT_ARCHIVED`,
  `reconciliation-cancel.ts:76-82`), formatée en date suisse. ⚠️ **Validation P2 (L2)** : la fiche donnait une clé à deux variables
  (`$date`, `$lockedThrough`) ; or `reconciliation_cancel_blocked_text` (`kesh-api/src/errors.rs:6916`) rend des couples `(&'static str,
  &'static str)` sans argument et « mot pour mot le FTL fr-CH » — une clé à variables ne s'y rend pas. **Le motif devient le neuvième**
  (`MOTIFS` et `ReconciliationCancelCode`, `reconciliation-cancel.ts:21-30`, `reconciliation.types.ts:284` ; « huit motifs » → « neuf ») ;
  **au clic**, le refus du socle (`400 PERIOD_LOCKED`, `details.lockedThrough` et `details.attempted`, `kesh-api/src/errors.rs:6477-6500`)
  se traduit alors par `reconciliationCancelErrorMessage` en ce même motif, la borne lue dans `details.lockedThrough` (validation P2, M1) ; `reconciliation-cancel-blocked-no-fiscal-year` (`messages.ftl:961`), son fallback
  `errors.rs:6958` et son entrée `FISCAL_YEAR_INVALID` de `frontend/src/lib/features/reconciliation/reconciliation-cancel.ts` (« huit motifs »,
  type exhaustif par `never`) deviennent **inatteignables** pour le dé-rapprochement et **ne se retirent pas** (validation P1, M1) : la variante
  `NoOpenFiscalYearToday` reste dans l'enum (gestes de la fiche, R7 à R9), donc les trois `match` exhaustifs de `kesh-api/src/errors.rs`
  (`:6243`, `:6956`, `:7005`) gardent leur bras ; le patron est celui du bras `SupplierInvoiceNotPaid` de `reconciliation_cancel_blocked_text`
  (« Inatteignable … le `match` reste exhaustif », clé et repli conservés). On y ajoute, dans **chacun des trois** `match`, le bras
  `PeriodLocked` (clé du dé-rapprochement pour `reconciliation_cancel_blocked_text`, repli de même ; délégation à cette fonction depuis les
  deux autres, comme les « têtes d'autres gestes »). **Commentaires** (validation P2, L7) : comme `DbError`, ces trois `match` ne voient que les
  rangs que le **geste** refuse lui-même (`kesh-db/src/errors.rs`, doc de `SettlementCancelBlocker` : rangs 1, 1 bis, 2, 2 bis) — les bras
  `PeriodLocked` et `NoOpenFiscalYearToday` y sont donc **inatteignables** dans les trois familles parce que le **socle** refuse ces rangs au
  clic avec sa propre erreur (`400 PERIOD_LOCKED`, `400 FISCAL_YEAR_INVALID`), non parce que `Today` ne produirait pas `PeriodLocked` ; le
  commentaire le dit ainsi. Un grep du symptôme (quatre locales, `i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts`) confirme qu'aucune clé
  n'est orpheline et que les replis Rust et TS de la clé neuve sont identiques.
- **D6 — Le dé-rapprochement ne lit plus la date du jour.** `cancel_in_tx`, **`cancel`** (l'enveloppe du pool, `reconciliation_cancel.rs:1104`), **`cancel_blocker`** (`:292`), `get_view`,
  `entry_cancel_outlook`, `group_cancel_blocker`, `supplier_group_cancel_blocker`,
  `grouped_settlements::{cancel_in_tx, supplier_cancel_in_tx}` perdent leur paramètre `aujourd_hui` ; la
  route `post_cancel_reconciliation` (`:6268-6270`), la vue (`:6149`) et les candidates (`:6974`) cessent de lire l'horloge pour ce geste.
  L'inventaire des lectures de l'horloge `crates/kesh-api/tests/date_du_jour_garde.rs` se met à jour en conséquence (lire ce qu'il compte
  avant de l'éditer). *Écartée* : garder `aujourd_hui` « au cas où » — un paramètre inutilisé masque la règle.
- **D7 — Détection en lecture des livres déjà touchés (Q4 a).** Dépôt : `reconciliation_cancel::cross_fiscal_year_cancellations(pool,
  company_id)` → `Vec<CrossFiscalYearCancellation>`, une ligne par **contre-passation** (dédoublonnée sur `reversalJournalEntryId` : un
  paiement Wise groupé écrit une entrée d'audit par transaction, P10), pour les entrées `reconciliation.cancelled` dont
  `reversalJournalEntryId` n'est pas nul et dont l'inverse et l'origine (`reverses_entry_id`) sont dans deux exercices différents ; champs :
  transaction bancaire, instant de l'annulation, genre, origine (id, numéro, date, exercice id/nom/statut), inverse (idem), total des
  débits de l'origine. **Portée par société lue sur l'écriture, non sur l'audit** (validation P1, M7) : `audit_log.company_id` est nullable (« NULL = société
  indéterminable, état permanent », `crates/kesh-db/test-schema/0001_schema_squash.sql:192`) — filtrer dessus **omettrait en silence** des cas ;
  la requête joint `journal_entries` (origine, `company_id = ?`) et ne lit l'audit que pour la nature du geste (`details_json`, colonne de
  `audit_log`, non `details`). Index de `audit_log` : `(entity_type, entity_id)`, `(user_id, created_at)`, `(company_id, created_at)` — aucun
  sur `action` : le plan filtre `entity_type = 'bank_transaction'` puis `action` ; à mesurer par `EXPLAIN` comme dit plus bas.
  **Lecture seule** : une transaction de lecture, aucun verrou, aucune écriture (pas même d'audit). Route
  `GET /api/v1/reconciliation/cancellations/cross-fiscal-year` (Comptable+ comme la vue ; clés `read` admises), tri par instant. Écran : un
  panneau dans `frontend/src/lib/features/reconciliation/` (`CrossFiscalYearCancellations.svelte`), monté sur la page des imports
  (`routes/(app)/bank-import/+page.svelte`), **absent** quand la liste est vide, qui dit pour chaque cas « L'écriture n° { $n } du { $date }
  (exercice { $fy }) a été contre-passée le { $date2 } dans l'exercice { $fy2 } » avec deux liens *Voir l'écriture* et un renvoi au
  manuel (§ D8). ⚠️ Avant d'écrire la requête : `EXPLAIN` sur `audit_log` filtré par `action` et `entity_type` (index existants ? la table
  grossit sans fin) — si le plan parcourt toute la table, le dire au Dev Agent Record et ouvrir une issue de performance plutôt que d'ajouter
  un index (ce serait une migration). ⚠️ `JSON_VALUE` existe sous MariaDB 10.11 (base de CI) ; vérifier que les entrées d'audit antérieures
  portent toutes `reversalJournalEntryId` (clé posée depuis la 25-3-b, à confirmer au code de cette story).
- **D8 — La correction se fait par l'utilisateur, par deux écritures, jamais d'office.** Procédure au manuel (nouvel alinéa du § *Annuler un
  rapprochement*, « Une contre-passation tombée dans un autre exercice ») : pour une origine E (exercice N, date dE) dont l'inverse R est dans
  l'exercice M : si N est **ouvert** (jamais clôturé, ou rouvert — Q6), saisir dans N, à la date dE, une écriture OD aux lignes de E
  inversées (« Correction : contre-passation de l'écriture n° … dans son exercice ») ; puis dans M, à la date de R, une écriture OD aux
  lignes de E (« Correction : annule la contre-passation n° … passée dans le mauvais exercice »). N porte alors E et son inverse, M porte R et
  son inverse ; aucun total ne change ailleurs. Kesh ne peut pas les générer : E est déjà contre-passée, R est une contre-passation (P9). ⚠️
  **À vérifier et à dire au manuel** (T6) : les deux écritures de correction touchent le compte du grand livre de la banque — apparaissent-
  elles parmi les écritures candidates d'un rattachement (`GET …/entry-candidates`) ou comme « déjà comptabilisée » ? Si oui, le manuel dit
  de ne pas les rattacher (elles ne portent aucun mouvement bancaire) ; si le risque de faux rattachement est réel, ouvrir une issue (CLAUDE.md
  § *Qualité du code* : « le faux rattachement est muet »). Lettrage : sur un compte lettrable (débiteurs, créanciers), les lignes des deux
  écritures de correction se soldent entre elles ; le manuel dit de les lettrer ensemble (`POST /letterings`, ou l'écran du lettrage). Si N
  est clos et ses comptes déposés : ne rien corriger (Q6).

## Critères d'acceptation

Tests d'intégration en `kesh-db` (`crates/kesh-db/tests/reconciliation_cancel.rs`, motif des tests existants) et de bout en bout en
`kesh-api` (`reconciliation_e2e.rs`, `reconciliation_group_e2e.rs`, `wise_reconciliation_e2e.rs`, `supplier_settlement_cancel_e2e.rs`),
horloge de test posée (`state.horloge`) pour que **le jour diffère toujours de la date de l'origine** — sans quoi un test de la date de
l'origine passe à vide sous `Today`.

- **AC-1 — Écriture seule, exercice de l'origine ouvert, annulation dans le même exercice** : origine du 14.03 en N, horloge au 20.11 de N ;
  l'inverse est datée du 14.03, exercice N, libellé « Contre-passation écriture n° … » sans suffixe ; **la date de la transaction bancaire diffère
  de celle de l'écriture** (rattachement à une écriture existante d'une autre date : sans cela, une date lue sur la transaction passerait,
  M10) ; `journal_entry.reversed` porte
  `"dating": "originDate"` ; `reconciliation.cancelled` garde sa forme.
- **AC-2 — Écriture seule, origine dans un exercice antérieur encore ouvert (jamais clôturé), horloge en N+1** : inverse datée de la date de
  l'origine, dans N ; **aucune** ligne n'est écrite dans N+1 (compte des `journal_entries` de N+1 avant et après).
- **AC-3 — Écriture seule, exercice de l'origine clôturé puis rouvert, horloge en N+1** : comme AC-2 ; puis N se clôture de nouveau sans
  obstacle nouveau.
- **AC-4 — Exercice de l'origine clos** : `POST …/cancel` refusé, `ReconciliationNotCancellable { FiscalYearClosed }`, message
  `reconciliation-cancel-blocked-fiscal-year-closed` aux quatre locales (contrôlé), rien n'est écrit, le lien reste ; la vue l'annonçait
  (`cancelBlockedBy: "FISCAL_YEAR_CLOSED"`).
- **AC-5 — Règlement client (R2)** : AC-1 à AC-4 ; en plus, le règlement est retiré, la facture redevient à régler, le lettrage `reversal`
  (lignes libres) est dans N.
- **AC-6 — Règlement fournisseur (R3)** : AC-1 à AC-4 ; la facture redevient `open`.
- **AC-7 — Règlements groupés client (R4) et fournisseur (R5), paiement Wise groupé (R6, sur `main`)** : AC-1 à AC-4 pour chacun ; pour le
  groupe, toutes les factures redeviennent à régler ; pour Wise, toutes les transactions du paiement reviennent en attente, une seule inverse.
- **AC-8 — Retrait seul d'un côté de virement** : inchangé (aucune inverse, `reversalJournalEntryId: null`), y compris quand l'exercice de
  l'écriture est clos ou la période verrouillée.
- **AC-9 — Période verrouillée** (date de l'origine ≤ `books_locked_through`, seuil inclusif testé au jour de la borne **et** au lendemain) :
  la vue rend `cancelBlockedBy: "PERIOD_LOCKED"` et la borne ; l'écran masque *Confirmer* et affiche `reconciliation-cancel-blocked-period-
  locked` ; au clic forcé par l'API, `400 PERIOD_LOCKED`, rien n'est écrit, le lien reste. Pour R1, R2, R4 au moins ; R3, R5 par le même
  rang (test de la queue commune). **Les candidates** (`GET …/entry-candidates`, `entry_cancel_outlook`) rendent le même motif pour une écriture
  sous la borne : le dialogue de rattachement l'annonce comme obstacle (case à cocher, `LinkSettlementModal.svelte:160`, `:401-415`) — test API et
  test unitaire du composant (validation P1, L9). *(Sous réserve de Q2.)*
- **AC-10 — La vue et le dialogue disent la date** : `reversalDate`, `reversalFiscalYearName` dans `GET …/transactions/{id}` (nuls pour un
  retrait seul) ; les cinq messages de confirmation, aux quatre locales, ne disent plus « aujourd'hui » et nomment la date et l'exercice ;
  la phrase TVA suit D4.
- **AC-11 — Les annulations depuis la fiche ne changent pas (R7, R8)** : annuler un règlement client depuis la fiche, et un paiement
  fournisseur depuis la fiche, origine en N ouvert, horloge en N+1 : l'inverse reste datée du jour, dans N+1, rang 5
  `FISCAL_YEAR_INVALID` inchangé (sans exercice ouvert au jour : refus ; avec N+1 ouvert : inverse du jour dans N+1). *(Test de non-régression qui fige la portée ; il changera avec les issues de Q3.)*
- **AC-12 — Verrous et concurrence** : (a) l'exercice de l'inverse est **confirmé** par une lecture verrouillante (mutation M7) et un
  exercice différent de celui de l'origine rend `Invariant` (test unitaire du socle, état forgé) ; (b) entrelacement `close(N)` concurrent
  d'une annulation dont l'origine est en N (il établit l'ordre des verrous, **non** la confirmation de l'étape (4) : M7 est angle mort assumé,
  voir D2), patron du test d'entrelacement du dé-rapprochement (`reconciliation_cancel.rs` doc, étape 2) :
  ou la clôture attend et l'annulation passe dans N ouvert, ou la clôture passe d'abord et l'annulation est refusée `FISCAL_YEAR_CLOSED` —
  jamais une inverse dans un exercice clos ; un interblocage est rejoué par la route ; (c) mesure des verrous (D3) écrite au Dev Agent Record.
- **AC-13 — Règle de datation, tests unitaires** : le verdict « date et exercice de l'inverse » selon la datation (`Today`, `OriginDate`,
  `NextFiscalYearStart`), l'état de l'exercice de l'origine et la borne, isolé dans une fonction pure de `kesh-db` testée sans base
  (tableau de cas : ouvert/rouvert/clos × borne absente/avant/égale/après), et la précédence de la queue commune pour `OriginDate`
  (`AccountArchived` avant `PeriodLocked`, `FiscalYearClosed` avant tout), paire par paire comme le veut `errors.rs:667`.
- **AC-14 — Détection (D7)** : la route rend un cas par contre-passation tombée dans un autre exercice (jeu : un cas `entry`, un cas
  `invoice_settlement`, un paiement Wise groupé de deux transactions → **un** cas, une annulation dans le même exercice → aucun, un retrait
  seul → aucun, une contre-passation par `/reverse` dans un autre exercice → aucun) ; elle n'écrit rien (compte de `journal_entries`,
  `audit_log` avant et après) ; elle est scopée à la société ; le panneau est absent sans cas. Les cas s'obtiennent en écrivant l'état ancien
  par SQL direct dans le test (aucun chemin de production ne le produit plus). *(Sous réserve de Q4.)*
- **AC-15 — Manuel et API** (`.tex` et `docs/api-external.md` ; les PDF ne se commitent pas dans `story/…`, CLAUDE.md § *Fusions et fichiers
  communs* — ils se régénèrent à la release) : `user-manual.tex` — § *Annuler un rapprochement* (`:4064-4145` : la puce « datée du jour »,
  la puce « aucun exercice ouvert ne couvre la date du jour » remplacée par la période verrouillée, l'alinéa D8), encadré
  `sec:date-du-jour` (`:863-866`, le rapprochement sort de la liste), `:3967` et `:3878` relus (exercice clôturé : toujours vrai) ; **sites que la fiche ne nommait pas (validation P1, M9)** :
  `user-manual.tex:384-386` (l'« écart fictif » après annulation : l'inverse n'est plus « datée du jour de l'annulation » — la cause décrite
  disparaît, la phrase se réécrit ou se retire), `:826-836` (encadré « Ce que le verrou n'empêche pas » : « La contre-passation est datée du
  jour, donc après le verrou : elle passe » devient faux **pour l'annulation d'un rapprochement** — Q2), `:3971` (« la fenêtre dit ce qu'il en
  serait aujourd'hui » : la période verrouillée devient un obstacle **annoncé**, non plus seulement possible), l'alinéa « Quand l'annulation n'est
  pas possible » (`:4144`, la puce « aucun exercice ouvert ne couvre la date du jour » **sort** de la liste du dé-rapprochement) ;
  `admin-manual.tex:3559` (le verrou de période est aussi le remède du refus d'annuler un **rapprochement** dont l'écriture est sous la borne) ;
  `docs/api-external.md:1005` et `:1007` (lignes de la table des refus du dé-rapprochement : « Aucun exercice ouvert ne couvre la date du jour —
  `FISCAL_YEAR_INVALID` » sort, « Date du jour dans une période verrouillée — `PERIOD_LOCKED` » devient « date de l'écriture ») et `:1065` (champs
  des candidates) ; `docs/
  api-external.md:992` (date de l'origine, refus `PERIOD_LOCKED`, champs `reversalDate` et `reversalFiscalYearName`, route de détection) et
  `:717` relu. **Sites ajoutés à la validation P2 (L6)** : `api-external.md:990` (vue `GET …/transactions/{id}` : c'est là, non à `:992`,
  que vont `reversalDate`, `reversalFiscalYearName`, la valeur `PERIOD_LOCKED` de `cancelBlockedBy` et la borne dans `cancelBlockedLabel`,
  aujourd'hui « le numéro du compte archivé ») et `:1006` (« L'exercice **du jour** est suivi d'un exercice clôturé — `LATER_FISCAL_YEAR_CLOSED` » :
  l'exercice **de l'écriture**, état hérité seulement, P6) ; `user-manual.tex` § *Un côté d'un virement* (`:4106-4118`, phrase `:4113`, « un exercice
  clôturé ou un lettrage figé ne l'empêchent pas » : ni une période verrouillée, AC-8) ; à l'écran,
  `LinkSettlementModal.svelte:394-399` et la clé `reconciliation-link-entry-cancel-today` (`messages.ftl:1776` et les trois autres locales :
  « une période verrouillée … pourraient l'empêcher, dès aujourd'hui sans que Kesh le voie » — Kesh la voit désormais, la période verrouillée
  sort de cette liste ; texte et `LinkSettlementModal.test.ts:483`). Le même dialogue d'annulation est monté sur la fiche d'une facture
  client (`frontend/src/routes/(app)/invoices/[id]/+page.svelte:1537`) : aucun code n'y change, mais la recette visuelle (AC-16) prend le
  dialogue **depuis cette fiche aussi** (L8). Recherche du symptôme dans les `.tex` **et** dans les PDF aplatis (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' '
  | tr -s ' ' | grep -o -i '.\{100\}datée du jour.\{100\}'`, puis `date du jour`, `aujourd'hui`) : chaque phrase qui parle du
  dé-rapprochement est corrigée, les autres (R7 à R11) restent.
- **AC-16 — E2E Playwright** (`frontend/tests/e2e/reconciliation-cancel.spec.ts`) : (1) annuler le rapprochement d'une écriture d'un
  exercice antérieur ouvert — le dialogue dit la date de l'origine ; après confirmation, *Voir l'écriture* de l'inverse montre cette date et
  cet exercice ; (2) écriture sous la borne du verrou — le motif s'affiche, pas de bouton de confirmation ; (3) le panneau de détection
  (état ancien posé par l'endpoint de test ou par SQL) liste le cas et mène aux deux écritures. Recette visuelle (CLAUDE.md) : captures du
  dialogue (date d'origine, phrase TVA), du motif « période verrouillée » et du panneau, sur une base de démonstration, dans
  `kesh-gate-logs/`.
- **AC-17 — Gates et mutations** : gate complet (`scripts/test-fast.sh`, puis `scripts/gate-ci.sh`), frontend (`check`,
  `lint-i18n-ownership`, `test:unit`, `build`), E2E complet au dernier commit de code ; mutations M1 à M11 tuées, chacune par le test annoncé.
- **AC-18 — Aucune migration** : `git diff --stat <base>..HEAD -- crates/kesh-db/migrations/` vide ; le schéma de test
  (`crates/kesh-db/test-schema/`) inchangé.

## Mutations (à jouer au développement, une à une, fichier touché après restauration — mémoire « mutation restaurée, binaire périmé »)

| # | Mutation | Test qui la tue |
|---|---|---|
| M1 | bras `Entry` du dé-rapprochement : `OriginDate` → `Today` | AC-2 (et AC-1, horloge ≠ origine) |
| M2 | bras règlement client : `OriginDate` → `Today` | AC-5 |
| M3 | bras règlement fournisseur : `OriginDate` → `Today` | AC-6 |
| M4 | groupe client : `OriginDate` → `Today` | AC-7 (client) |
| M5 | groupe fournisseur : `OriginDate` → `Today` | AC-7 (fournisseur) |
| M6 | rang `PeriodLocked` retiré de la queue commune | AC-9 (vue **et** candidates `entry-candidates`, qui lisent la même queue) — le clic reste refusé par le socle : la mutation ne doit tuer **que** ces tests de lecture, jamais celui du clic (validation P2, L5) |
| M7 | confirmation de l'étape (4) remplacée par une lecture ordinaire | **angle mort assumé d'avance** (D2 : le verrou de l'étape (3) précède toute lecture) ; à tuer seulement si un entrelacement le permet |
| M8 | gestes de la fiche (R7) passés en `OriginDate` | AC-11 |
| M9 | détection : dédoublonnage retiré | AC-14 (Wise groupé → deux cas) |
| M10 | date de l'inverse lue sur la **transaction bancaire** au lieu de l'écriture (bras `Entry`, rattachement d'une autre date) | AC-1 (date de la transaction ≠ date de l'écriture) |
| M11 | seuil du rang `PeriodLocked` de la queue : `<=` → `<` | AC-9 au **jour de la borne** (la vue doit refuser ; le socle, lui, garde `<=`) |

## Tâches

- [ ] **T0 — Reprise et relecture** : reprendre `origin/main` dans la branche de la story (demander d'abord à l'orchestrateur : reprise
  directe, ou attente de la reprise par la branche d'epic) ; résoudre `supplier_invoices.rs` (`DuplicatePolicy`) en gardant les deux
  intentions ; relire au code chaque prémisse P1 à P11 et chaque numéro de ligne de la fiche, corriger la fiche s'il le faut ; confirmer P7
  (date de l'écriture de chaque chemin d'acceptation) et la présence de `reversalJournalEntryId` dans toutes les entrées d'audit (D7).
- [ ] **T1 — Tests rouges d'abord** (AC-1 à AC-7, AC-9, AC-11) ; commit.
- [ ] **T2 — Socle** (D2) : `ReversalDating::OriginDate`, lecture de `je.entry_date` à l'étape (1), étape (4), audit `dating` ; fonction pure
  du verdict + ses tests (AC-13) ; signatures de `reverse_in_tx`/`reverse_owned_in_tx` et de leurs appelants (R1 à R10) ; commit.
- [ ] **T3 — Queue commune et gestes** (D5, D6) : datation passée à `settlement_entry_cancel_blocker`, `settlement_cancel_blocker*`,
  `supplier_settlement_cancel_blocker*`, `group_cancel_blocker`, `supplier_group_cancel_blocker` ; rang `PeriodLocked` ; retrait de
  `aujourd_hui` du dé-rapprochement ; `date_du_jour_garde.rs` ; commit.
- [ ] **T4 — Vue et API** (D4) : `reversalDate`, `reversalFiscalYearName` ; mapping de `PERIOD_LOCKED` dans `crates/kesh-api/src/errors.rs`
  (fallbacks, quatre locales) ; commit.
- [ ] **T5 — Détection** (D7, sous réserve de Q4) : dépôt, route (`crates/kesh-api/src/lib.rs`, à côté de `:1095-1129`), tests AC-14 ;
  commit.
- [ ] **T6 — Écran** : `CancelReconciliationDialog.svelte`, `reconciliation-cancel.ts`, `reconciliation.types.ts`, `reconciliation.api.ts`,
  `LinkSettlementModal.svelte` (relire ce qu'il affiche des motifs), panneau de détection et son montage ; tests unitaires
  (`CancelReconciliationDialog.test.ts`) ; i18n aux quatre locales, `sitesTotal` de `i18n-keys.test.ts` recompté ; vérification D8 (candidates
  de rattachement) ; commit.
- [ ] **T7 — Manuel et API** (AC-15) ; commit.
- [ ] **T8 — E2E** (AC-16), recette visuelle, gates, mutations M1 à M11 (AC-17) ; Dev Agent Record ; commit et push de la seule branche
  `story/…` à chaque étape.

## Dev Notes

### Fichiers touchés (prévus)

- `crates/kesh-db/src/repositories/journal_entries.rs` — `ReversalDating`, `reverse_in_tx`, `reverse_owned_in_tx`, `reverse_in_tx_inner`
  (étapes 1 et 4, audit) ; doc-comments qui disent « datée du jour » pour le dé-rapprochement.
- `crates/kesh-db/src/repositories/reconciliation_cancel.rs` — `cancel_in_tx`, `get_view`, `entry_cancel_outlook`, `entry_cancel_chain`,
  `cancel_blocker`, `ReconciliationView` (+ date et exercice), `ReconciliationCancellation` (doc « datée du jour », `:616`), requête de
  détection.
- `crates/kesh-db/src/repositories/settlement_cancellation.rs` — datation en paramètre, rang `PeriodLocked`, doc de la « limite assumée »
  (`:50-52`) réécrite.
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`, `supplier_invoices.rs`, `grouped_settlements.rs` — datation passée.
- `crates/kesh-db/src/errors.rs` — `SettlementCancelBlocker::PeriodLocked` (place dans l'enum = précédence, doc `:667`), `code()`.
- `crates/kesh-api/src/routes/reconciliation.rs` — route du geste, vue, candidates, route de détection ; `crates/kesh-api/src/lib.rs` —
  enregistrement de la route ; `crates/kesh-api/src/errors.rs` — mapping, fallbacks ; `crates/kesh-api/src/services/factures.rs:585`
  (appelle `settlement_cancel_blocker` : passe `Today`).
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`.
- `frontend/src/lib/features/reconciliation/` (+ montage sur `frontend/src/routes/(app)/bank-import/+page.svelte`).
- Tests : `crates/kesh-db/tests/reconciliation_cancel.rs`, `crates/kesh-api/tests/{reconciliation_e2e,reconciliation_group_e2e,
  wise_reconciliation_e2e,supplier_settlement_cancel_e2e,date_du_jour_garde}.rs`, `frontend/tests/e2e/reconciliation-cancel.spec.ts`. Les
  tests existants qui assertent une inverse **datée du jour** sur le dé-rapprochement (`git grep -n "entry_date\|entryDate"` dans ces
  fichiers : 9 sites dans `reconciliation_e2e.rs`, 2 dans `reconciliation_group_e2e.rs`, 3 dans `reconciliation_cancel.rs` à `main`) se
  relisent un par un : ceux qui portent sur le dé-rapprochement changent d'attendu, ceux des gestes de la fiche ne changent pas.
  **Tests que la fiche ne nommait pas (validation P1, M3)**, qui appellent le dé-rapprochement, ses blocages ou les signatures changées
  (`git grep -n "reconciliation/transactions/{}/cancel\|reverse_in_tx(\|cancel_settlement_in_tx(\|cancel_in_tx(\|settlement_cancel_blocker(\|entry_cancel_outlook" -- 'crates/*/tests/*'`) :
  `kesh-api/tests/jour_suisse_nuit_e2e.rs` — **deux tests changent d'attendu** : `ac3_annulation_d_un_rapprochement_la_nuit` (`:699-727`,
  l'inverse n'est plus datée du 1er janvier mais de l'origine, `assert_contre_passee_le_1er_janvier`) et `ac7_les_services_jugent_au_jour_suisse`
  (`:876-940`, la candidate `entry-candidates` ne rend plus `FISCAL_YEAR_INVALID` pour le dé-rapprochement : plus de rang 5) ; ses tests de R7
  à R9 (`:635-695`) ne changent pas — `kesh-api/tests/bank_accounts_closed_e2e.rs:1709` (appelle `cancel_in_tx`), `bank_account_closed_inventory.rs`
  (18 sites, inventaire lexical), `audit_acteur_garde.rs`, `reconciliation_link_e2e.rs`, `reconciliation_link_entry_e2e.rs`,
  `paypal_reconciliation_e2e.rs`, `stripe_reconciliation_e2e.rs`, `rejeu_interblocage_e2e.rs` ; `kesh-db/tests/grouped_settlements.rs`,
  `grouped_settlements_supplier.rs`, `supplier_invoices_repository.rs` (12), `invoice_settlement.rs` (7), `invoice_write_off.rs` (4),
  `letterings_lexical.rs` (11), `lettering_documents.rs`, `journal_entries_modification.rs`, `filet_bilan_clos.rs` (signatures :
  `ReversalDating::Today(…)`). Doc-comments et types à relire pour « datée du jour » : `reconciliation_cancel.rs:616`,
  `reconciliation.types.ts:345`, `reconciliation.api.ts:141`.
  ⚠️ **Validation P2 (M1, L3) — l'inventaire ci-dessus n'est pas sorti de sa commande, et il omettait le frontend.** (a) Le motif littéral
  `transactions/{}/cancel` ne trouve pas les `format!("…/transactions/{t}/cancel")` des tests, et les nombres entre parenthèses ne sont pas ceux
  qu'elle rend (à `56d47b2cc`, la commande rend : `bank_account_closed_inventory.rs` 15, `supplier_invoices_repository.rs` 5,
  `invoice_settlement.rs` 5, `letterings_lexical.rs` 2, et `invoice_reminders_repository.rs` 4, `migrations_upgrade_path.rs` 2, absents de la
  liste). Commandes **qui font foi**, périmètre `origin/main` à `56d47b2cc`, à rejouer à T0 sur la branche reprise :
  `git grep -c "reconciliation/transactions/{[a-z_0-9.]*}/cancel" -- 'crates/*/tests/*'` (12 fichiers, dont trois que la liste omettait :
  `bank_statements_manual_e2e.rs`, `project_reassignment_e2e.rs`, `reconciliation_rules_e2e.rs`) ;
  `git grep -c "reconciliation_cancel::cancel\|reconciliation_cancel::get_view\|reconciliation_cancel::entry_cancel_outlook\|cancel_blocker(" -- 'crates/*/tests/*'`
  (12 fichiers, dont `supplier_invoices_draft.rs`, omis) ; et la commande de la P1 ci-dessus **avec** ses nombres. Le compilateur trouve les
  signatures ; la relecture ne porte que sur les **attendus**. (b) **Frontend** : `reconciliation-cancel.test.ts:97-105` et
  `CancelReconciliationDialog.test.ts:101-117` prennent `PERIOD_LOCKED` comme l'exemple du « code **hors** motifs » (`expect(isReconciliationCancelCode('PERIOD_LOCKED')).toBe(false)`,
  « refusé AU CLIC par un code hors motifs ⇒ le message du serveur ») : ils changent de sens avec D5 — un autre code hors motifs
  (`LATER_FISCAL_YEAR_CLOSED`) les reprend, et deux tests neufs fixent `PERIOD_LOCKED` en motif, à la lecture et au clic (borne de
  `details.lockedThrough`). `reconciliation-cancel.test.ts:69-91` (« un texte par motif ») gagne la neuvième ligne. `CancelReconciliationDialog.test.ts:207`
  (retrait seul : `not.toContain("Une écriture inverse datée d'aujourd'hui sera passée")`, qui **passerait à vide** une fois le texte changé —
  à réécrire sur le texte neuf) et `LinkSettlementModal.test.ts:483` (texte `reconciliation-link-entry-cancel-today`, AC-15) changent d'attendu.
- Documentation : `docs/manual/fr/user-manual.tex`, `docs/api-external.md`. Pas de CHANGELOG (assemblé à l'intégration), pas de PDF.

### Ce qui doit être préservé

- L'ordre des étapes du dé-rapprochement (doc de `cancel_in_tx`, `reconciliation_cancel.rs:637-699`) : verrous avant toute lecture
  ordinaire, lien défait **avant** la contre-passation, revérification verrouillante du compte clos en **dernière** acquisition, audit.
- Le retrait seul (`unlink_only`), le rang 3 exempté (`unlinking`), les lignes sœurs Wise, `expectUnlinkOnly` / `409
  RECONCILIATION_CANCEL_CHANGED`.
- Le JSON de `journal_entry.reversed` du chemin `Today` (T-b2 de la 29-1b), et celui de `NextFiscalYearStart`.
- Les gestes R7 à R11 (AC-11).

### Recoupements

- **53-2** (`invoice_settlements_write.rs`, `credit_notes.rs`, migration) : fichier commun `invoice_settlements_write.rs`, fonctions
  distinctes (la 53-2 touche la correction de TVA de l'escompte et de la perte, la 53-6 la signature de `cancel_settlement_in_tx`) ; la
  seconde fusionnée reprend la première.
- **53-1a, 53-1b, 53-3** (`journal_entries.rs`, fusionnées dans la branche d'epic) : `create_in_tx_inner` a un paramètre de plus (53-1a),
  passé `None` par la contre-passation ; aucune autre fonction commune.
- **Epic 39** (Wise, sur `main`) : étapes « 5 bis » et « 7 bis » de `cancel_in_tx` — T0.
- **Epic 38** (contributeur, contre-passation sans date choisie, `POST /journal-entries/{id}/reverse`) : recoupe le socle (`ReversalDating`)
  si elle ajoute une datation choisie ; à contrôler à l'intégration (revue du code d'intégration), sans attendre.

### Règle de splitting préventif

Modules dont le code de production change (CLAUDE.md : un crate ou un domaine `frontend/src/lib/features/<domaine>`) : `kesh-db`,
`kesh-api`, `kesh-i18n`, `frontend/…/reconciliation` — **quatre** ; la page `routes/(app)/bank-import/+page.svelte` ne monte qu'un composant
du domaine (si la validation la compte comme module, cinq : toujours sous la borne). **Pas de découpage.** Q3 étendue aux gestes de la fiche
ferait six modules (`…/invoices`, `…/supplier-invoices`) : découpage obligatoire, d'où la recommandation de Q3. Q4 (b) ramènerait à trois.
Le signal de recyclage se jugera en validation.

**Signal de recyclage — déclaré au Project Lead (validation P2).** La P2 rend 3 MEDIUM, dont **trois sur trois** naissent d'un correctif de la
P1 ou en reprennent la classe sous une autre forme : M1 (l'inventaire des tests, remède de M3, reste incomplet — frontend), M2 (les priorités
de Q3, remède de M6, contredisent CLAUDE.md), M3 (la sortie du panneau, remède de M8, promet une sortie qui n'arrive pas). Sévérité égale
(MEDIUM) : le critère de CLAUDE.md § *Règle de splitting préventif* est rempli. Nuance à peser : les trois portent sur le **texte** de la fiche
(questions de produit, inventaire), non sur la conception du correctif (D1 à D3, inchangés et confirmés par la P1 et la P2). Découpage
possible, s'il est décidé : **53-6a** — datation `OriginDate`, rang `PeriodLocked`, vue, écran, manuel (AC-1 à AC-13, AC-15, AC-16 (1)(2)) —
et **53-6b** — détection et panneau (D7, D8, AC-14, AC-16 (3), Q4), qui dépend des réponses de Guy à Q4 et ne bloque pas le correctif du P1.
Décision au Project Lead.

### Migration

**Aucune.** Rien ne change au schéma ; la détection lit `audit_log`, `journal_entries`, `fiscal_years`. *Migration breaking policy* sans
objet (P1-P7 non déclenchés) ; aucun backfill : les livres déjà touchés ne se réparent pas d'office (CLAUDE.md § *Ce qu'impose la
production*).

### Notes de version (à reprendre dans le CHANGELOG à l'intégration)

> ⚠️ **Annuler un rapprochement bancaire contre-passe désormais l'écriture à sa propre date, dans son propre exercice** (#1674). Jusqu'ici,
> l'écriture inverse était datée du jour : une annulation faite après la fin de l'exercice de l'écriture — exercice rouvert, ou pas encore
> clôturé — la plaçait dans l'exercice courant, et deux exercices étaient faux. Conséquences : une annulation dont l'écriture est dans une
> **période verrouillée** est désormais refusée (l'écran le dit d'avance ; un administrateur recule le verrou) ; la contre-passation change
> les totaux de la période de l'écriture, TVA comprise. Les annulations faites avec les versions précédentes ne sont pas corrigées : la page
> des imports bancaires liste celles qui sont tombées dans un autre exercice, et le manuel dit comment les corriger par une écriture.
> **API** : `POST /api/v1/reconciliation/transactions/{id}/cancel` date l'inverse à la date de l'écriture ; refus nouveau `400
> PERIOD_LOCKED` ; `GET …/transactions/{id}` gagne `reversalDate`, `reversalFiscalYearName` et la valeur `PERIOD_LOCKED` de
> `cancelBlockedBy` ; route nouvelle `GET /api/v1/reconciliation/cancellations/cross-fiscal-year`. Aucune migration.

### Issues à ouvrir (par l'orchestrateur, après Q3 ; une par problème, ≥ 5 s d'écart)

1. « Annuler depuis sa fiche le règlement d'une facture client contre-passe à la date du jour : après la fin de l'exercice du règlement,
   l'écriture et son inverse tombent dans deux exercices » — R7, `invoice_settlements_write.rs:1253`, `POST /invoices/{id}/settlements/
   {settlementId}/cancel` (`api-external.md:677`).
2. Même texte pour le paiement d'une facture fournisseur annulé depuis sa fiche — R8, `supplier_invoices.rs:3663`, `api-external.md:788`.
3. Même texte pour l'annulation d'une facture fournisseur (contre-passation de l'écriture d'achat) — R9, `supplier_invoices.rs:3243`,
   `api-external.md:802`.

### Références

- Issue #1674 (texte et « Correctif décidé », Project Lead, 2026-11-26).
- `CLAUDE.md` : *Ce qu'impose la production*, *Priorités des défauts*, *Migration breaking policy*, *Règle de splitting préventif*,
  *Versions de l'API*, *Recette visuelle*.
- Stories : 25-3-zero (socle unique), 25-3-a-1 (queue commune), 25-3-b (dé-rapprochement), 29-1b (`ReversalDating`, `NextFiscalYearStart`,
  patron de la confirmation), 24-4c (verrou de période), 35-5a0 (date du jour lue une fois), 39-11c (Wise groupé), 43-1a/43-2 (groupes),
  53-1a et 53-3 (ordre des verrous dans l'epic).
- `docs/testing.md` (Playwright local), `docs/agents-orchestration.md` (consignes des agents).

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-11-27 — Fiche rédigée (Opus 5.5) contre `origin/main` `56d47b2cc`, statut `ready-for-dev` ; Q1 à Q6 ouvertes pour Guy ; validation
  P1 à lancer (Sonnet, passe complète : prompt `53-6-annulation-rapprochement-exercice-validation-prompt-p1.md`).
- 2026-11-27 — **Validation P1 (Sonnet 5.5, passe complète) : 0 CRITICAL, 0 HIGH, 9 MEDIUM, 9 LOW.** Les trois points majeurs du prompt tiennent
  au code : D2 (la date de l'origine se lit dans la requête verrouillante de l'étape (1), `journal_entries.rs:3110`, sans verrou de plus ; la
  confirmation `find_open_covering_date` rend bien l'exercice de l'origine, `fiscal_years.rs:1008-1019`) ; D5 et P7 (`create_in_tx_inner`
  refuse `PeriodLocked` en 1-bis, `:610-625`, après `FiscalYearClosed` et `LATER_FISCAL_YEAR_CLOSED`, avant la validation des comptes ; la queue
  ne l'évalue pas, `settlement_cancellation.rs:50-52` ; l'ordre `AccountArchived` puis `PeriodLocked` est celui du socle, étape (3) de
  `reverse_in_tx_inner` puis création) ; R1 à R11 (aucune autre contre-passation n'est atteinte par `POST …/cancel` ; sonde positive du
  symptôme `ReversalDating::Today` / `aujourd_hui` passé à un `reverse`/`cancel` : le site de la 53-6 sort, avec R7 à R10, rien d'autre). Q3 :
  les trois affirmations sont vraies, effets distincts (voir Q3). Remédiés dans la fiche : M1 code `FISCAL_YEAR_INVALID` (non
  `NO_OPEN_FISCAL_YEAR_TODAY`, inexistant) et bras exhaustifs conservés ; M2 `ReversalDating` `pub` ; M3 inventaire des tests (dont deux de
  `jour_suisse_nuit_e2e.rs`) ; M4 P7 corrigée (le rattachement ne crée pas d'écriture) et Q2 complétée par l'option (c) et le principe du
  manuel `:826-836` ; M5 M7 angle mort d'avance ; M6 Q3 (effet et priorité par geste) ; M7 portée de D7 lue sur l'écriture
  (`audit_log.company_id` nullable) ; M8 sortie d'un cas du panneau (Q4) ; M9 sites de manuel et d'API non nommés ; L1 à L9 : colonnes lues à
  l'étape (1), bras des trois `match` d'erreurs, mutations M10 et M11, Q5 (b dépend d'une migration non acquise), devises et réévaluation,
  noms de D6, candidates (AC-9), numérotation (Q1). Questions de produit : **toujours ouvertes pour Guy** (Q1 à Q6, Q4 enrichie). Passe
  suivante : P2, Opus (prompt `53-6-annulation-rapprochement-exercice-validation-prompt-p2.md`) ; rapport P1 :
  `kesh-gate-logs/e53/53-6-validation-p1.md`.
- 2026-11-27 — **Validation P2 (Opus 5.5, passe complète) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 8 LOW.** Recontrôlé au code et tenu : M1 de la P1
  (`FISCAL_YEAR_INVALID`, `kesh-db/src/errors.rs:800` ; trois `match` exhaustifs, `kesh-api/src/errors.rs:6205`, `:6919`, `:6977` — aucun autre
  `match` sur l'enum hors `code()` et la queue), M5 (l'exercice de l'origine est verrouillé avant toute lecture : M7 angle mort, confirmé),
  M7 (index de `audit_log` lus au schéma de test, `company_id` nullable), les deux tests de `jour_suisse_nuit_e2e.rs` (transaction du 20.12.2025,
  origine ≠ 1er janvier), les sites de manuel et d'API de l'AC-15 (`.tex` et PDF aplatis). Remédiés dans la fiche : **M1** frontend du motif
  `PERIOD_LOCKED` (neuvième motif, chemin du clic, deux tests qui le fixent « hors motifs ») et inventaire des tests recompté depuis ses
  commandes ; **M2** priorités de Q3 (trois P1 : une priorité par geste, et « bilan seul » fausse un solde de clôture) ; **M3** Q4 (a2) : le cas ne
  quitte jamais le panneau, la formule de sortie est retirée ; **L1** exposition de `ReversalDating` (autorité + `NextFiscalYearStart`, type
  étroit préféré) ; **L2** clé sans variable, borne ajoutée par l'écran ; **L3** décomptes de l'inventaire ; **L4** jour du seuil de D4 ; **L5**
  M6 tue aussi le test des candidates ; **L6** `api-external.md:990`, `:1006`, manuel `:4113`, `LinkSettlementModal.svelte:394-399` ; **L7**
  raison des commentaires « inatteignable » ; **L8** dialogue monté sur la fiche de facture (recette visuelle). **Signal de recyclage : oui**
  (§ *Règle de splitting préventif*), déclaré au Project Lead avec un découpage possible (53-6a / 53-6b). Q1 à Q6 **toujours ouvertes pour
  Guy**. Passe suivante : P3, Sonnet (prompt `53-6-annulation-rapprochement-exercice-validation-prompt-p3.md`) ; rapport P2 :
  `kesh-gate-logs/e53/53-6-validation-p2.md`.
