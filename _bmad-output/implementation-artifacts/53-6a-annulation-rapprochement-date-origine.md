# Story 53-6a : Annuler un rapprochement contre-passe dans l'exercice de l'écriture, à sa date (#1674)

Status: review  <!-- développement terminé le 2026-11-27 (Opus 5.5) ; gate-ci non obtenu (environnement d'idefix), E2E complet à la clôture ; validée : P1 (Opus, 0/1/1/10), P2 (Sonnet, 0/0/1/6) et P3 (Opus, 0/0/1/4) remédiées le 2026-11-27 ; P4 ciblée (Haiku, 0/0/0/1) sans finding au-dessus de LOW ; signal de recyclage déclaré au Project Lead -->

*Fiche **née du découpage de la 53-6 le 2026-11-27** (décision du Project Lead, signal de recyclage de la validation P2, CLAUDE.md § *Règle de splitting préventif*) : la 53-6a porte le **correctif** (cœur de la P1) ; la détection des livres déjà touchés est la **53-6b** (`53-6b-detection-contre-passations-autre-exercice.md`, dépend de celle-ci). Rédigée à l'origine le 2026-11-27 (Opus 5.5), worktree `kesh-53-6`, branche `story/53-6-annulation-rapprochement-exercice`, par `bmad-create-story` ; validations P1 (Sonnet) et P2 (Opus) faites sur la fiche d'origine, dont elle reprend les remèdes. Epic 53 « Écritures justes », release **v0.21.0** (vendredi 2026-12-04). Priorité **P1** (#1674, CLO-2). **Aucune migration.** Les questions Q1 à Q6 de la fiche d'origine sont **tranchées par le Project Lead le 2026-11-27** (§ *Décisions du Project Lead*). ⚠️ **La fiche est écrite contre `origin/main` à `56d47b2cc`**, non contre la branche d'epic : voir § *Écarts entre `main` et la branche d'epic* — la branche reprend `main` avant tout développement (T0). **Recontrôle de la validation P1 de la 53-6a** : `main` est depuis à `0e99cd2db` (tag v0.20.0) ; entre les deux, **aucun fichier de code de la story ne change** (`git diff --stat 56d47b2cc 0e99cd2db` sur les fichiers de code de la story, tests compris : vide) ; seuls `docs/api-external.md` (lignes citées inchangées), `docs/manual/fr/user-manual.tex` (une ligne, hors des sites cités) et `docs/manual/fr/admin-manual.tex` (−3 lignes avant `:3559`, qui devient **`:3556`**) bougent.*

## Story

En tant que comptable qui défait un rapprochement fait par erreur — pour rattacher la transaction à la bonne pièce —, je veux que Kesh contre-passe l'écriture **dans son propre exercice, à sa propre date**, et refuse clairement quand cet exercice est clôturé ou la période verrouillée, afin qu'annuler puis refaire un rapprochement ne fausse jamais deux exercices.


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
premier cas ; le correctif couvre les deux ; leur détection est la 53-6b. Une annulation faite **dans** l'exercice de l'origine, à une
date ultérieure, tombe dans le bon exercice mais dans une autre période (TVA trimestrielle) : elle ne fausse pas les comptes annuels ; Q4 (décision du 2026-11-27) la laisse hors de la détection de la 53-6b.

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
improbable devient un refus fréquent (une période TVA déclarée est typiquement verrouillée : `admin-manual.tex:3556` à `0e99cd2db`) : l'écran doit
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
| `kesh-db/src/errors.rs`, `kesh-api/src/errors.rs`, les quatre `messages.ftl`, `user-manual.tex`, `api-external.md` *(lignes ajoutées à la validation P1 de la 53-6a)* | modifiés des deux côtés depuis `fbfa64a27` (`main` : +147, +352/−3, ≈ +223 chacun, +337/−25, et la table de l'API ; branche : +24 — `JournalEntryProbableDuplicate` de la 53-1a, sans rapport avec `SettlementCancelBlocker` —, +39, …, +26) | **aucune décision ne change** ; tous les numéros de ligne de ces fichiers cités par la fiche bougent à la reprise : T0 les relit |

Conséquence : **T0** reprend `main` dans la branche de la story (ou attend que la branche d'epic l'ait reprise après la publication de la
v0.20.0 — publiée depuis, tag à `0e99cd2db` ; au 2026-11-27, `origin/epic/53-ecritures-justes` ne contient pas encore `main` —, CLAUDE.md § *Branches* — décision de l'orchestrateur), puis relit les numéros de ligne de cette fiche. Le bras Wise (AC-7) n'existe
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

## Décisions du Project Lead du 2026-11-27 (questions Q1 à Q6 de la fiche d'origine, tranchées)

- **Q1 — La contre-passation prend la date de l'écriture d'origine**, dans son exercice, **même à l'intérieur d'un même exercice** (annuler en novembre le rapprochement d'une écriture de mars date l'inverse de mars).
- **Q2 — En période verrouillée, refus** (`PERIOD_LOCKED`), **annoncé dans la vue avant le clic**. ⚠️ **Aux notes de version** : une annulation acceptée aujourd'hui peut être refusée après la story. Cette règle contredit, pour le seul dé-rapprochement, le principe du manuel « le verrou arrête ce qui entre dans le passé, jamais ce qui corrige depuis le présent » (`user-manual.tex:826-836`) : l'AC-14 corrige l'encadré (la contre-passation depuis la fiche d'une écriture **non rapprochée** reste datée du jour et passe ; l'écriture rapprochée est refusée par la contre-passation directe, `MATCHED_BANK_TRANSACTION`).
- **Q3 — Les trois autres annulations datées du jour vont en trois issues P1 séparées, hors de cette story** : règlement client annulé depuis sa fiche (R7), paiement fournisseur annulé depuis sa fiche (R8), annulation d'une facture fournisseur (R9). Elles sont ouvertes par l'orchestrateur (textes : `kesh-gate-logs/e53/53-6-issues-q3/`). La story pose le paramètre de datation qui les rend corrigibles par un seul argument ; l'AC-11 fige leur état actuel.
- **Q4 — La détection des livres déjà touchés se fait dans la 53-6b**, non ici. La 53-6a ne produit ni route de lecture, ni panneau, ni procédure de correction.
- **Q5 — Pas d'`/api/v2`.** C'est la correction d'un défaut qui écrivait faux, annoncée ⚠️ en tête des notes de version ; `docs/api-external.md:992` documente « datée du jour » : le contrat écrit change, il est dit faux par la décision du Project Lead du 2026-11-26 et réécrit comme tel. La modification du contrat (date écrite, refus neuf) porte `b` par § *Numérotation des versions*, sans dépendre d'une migration de la 53-2 (non acquise). Décision à écrire au Change Log.
- **Q6 — On ne rouvre pas un exercice clos** : refus explicite, `FISCAL_YEAR_CLOSED` (message `reconciliation-cancel-blocked-fiscal-year-closed`, inchangé). Un exercice **rouvert** par l'utilisateur est ouvert : l'annulation s'y date.

Le reste de ce qui accompagnait ces questions dans la fiche d'origine (variantes écartées, arguments) est dans `git log -p` de la fiche et la rétrospective ; seule la décision fait foi ici.


## Décisions

- **D1 — La datation du dé-rapprochement.** Exercice de l'écriture d'origine **ouvert** (rouvert compris) : la contre-passation est datée
  de la **date de l'écriture d'origine**, dans **son** exercice. Exercice **clos** : refus, motif `FISCAL_YEAR_CLOSED` existant (rang 2),
  inchangé. **Période verrouillée** (date de l'origine ≤ borne) : refus `PERIOD_LOCKED` (Q2 a). Vaut pour les cinq bras R1 à R6. Le
  retrait seul d'un côté de virement (`unlink_only`) n'écrit rien et ne change pas ; le rang `PeriodLocked` ne s'évalue donc pas pour lui
  (AC-8). **Devises** (validation P1, L5) : l'inverse recopie montant, cours et provenance de l'origine (`journal_entries.rs:3220-3247`) et se
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
  `{"reversalJournalEntryId": …, "dating": "originDate"}` (le chemin `Today` garde son JSON octet pour octet, T-b2 de la 29-1b : `kesh-api/tests/journal_entry_reversal_e2e.rs:3703`).
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
  sera passée dans l'exercice « { $fiscalYear } » » *(guillemets ajoutés à la revue P1, L3 : le nom par défaut d'un exercice est « Exercice AAAA », « dans l'exercice Exercice 2026 » se lisait mal)* à la place de « datée d'aujourd'hui », dans les cinq messages de confirmation
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
  536`) et de la **date de l'écriture** (validation P2, L2 : la requête du rang 2 de `settlement_entry_cancel_blocker` ne lit que `fy.status` ;
  elle lit aussi `je.entry_date`, même ligne, sans verrou de plus).
  ⚠️ **Validation P2 (M1) — le rang 2 bis précède `PeriodLocked` : pour un règlement lettré, le motif visible est le lettrage figé.** Une facture réglée
  **en entier** porte un groupe de lettrage de pièce (`origin = 'document'`, créé au règlement, dissous par le geste : `dissolve_invoice_document_group_in_tx`,
  `dissolve_supplier_invoice_document_group_in_tx`) ; quand la borne couvre la date du règlement, **toutes** les lignes du groupe sont sous la borne (la facture
  précède son paiement) et le rang 2 bis, `409 LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, répond **avant** `PeriodLocked` — c'est déjà le cas aujourd'hui, et le test
  existant `group_unreconciliation_refuses_a_lettering_frozen_by_the_period` (`kesh-db/tests/grouped_settlements.rs`, `groupe_regle`, borne au 30.06 pour un
  règlement du 15.06) le fixe. `PERIOD_LOCKED` ne s'atteint donc, pour R2 à R5, que sans groupe de lettrage figé ; ⚠️ **validation P3 (M1) — les états diffèrent
  par bras** (le texte de la P2 donnait le règlement partiel pour les quatre) :
  - **R2** (règlement client unitaire) : règlement **partiel** (aucun groupe : la somme sur la créance n'est pas nulle, `sync_document_in_tx`, `letterings.rs`) ;
    règlement **antérieur** d'une facture soldée par un règlement ultérieur postérieur à la borne ; facture datée **après** le paiement (ci-dessous) ; état hérité.
  - **R3** (paiement fournisseur unitaire) : **jamais partiel** — une facture fournisseur n'a qu'une écriture de règlement (`discover_supplier_invoice_document`,
    `("paid", Some(reglement))`) ; facture datée après le paiement ; état hérité.
  - **R4, R5** (groupes) : **jamais partiels** — le groupe solde chaque facture (R4 : `grouped_settlements.rs`, étape (9), `:820`, `Invariant` « la facture … garde un reste dû » ; R5 : le contrôle `payable_amount != invoice.total_amount`, `:2438`, et la mise à `paid` d'un seul tenant, `:2574`, à `0e99cd2db` — correction du LOW L1 de la P4 ciblée, au développement) ;
    facture datée après le paiement ; état hérité.
  - **L'état de production commun aux quatre** : la facture datée **après** le paiement, dans la tolérance de la fenêtre — un jour pour le client
    (`PAYMENT_BEFORE_INVOICE_TOLERANCE_DAYS = 1`, `kesh-db/src/repositories/reconciliation.rs:288`, contrôlé au groupe `grouped_settlements.rs:653`), sept pour le
    fournisseur (`SUPPLIER_PAYMENT_BEFORE_INVOICE_TOLERANCE_DAYS = 7`, `:493`, `grouped_settlements.rs:2417`) — et la borne posée **entre** la date du règlement
    (comprise) et celle de la facture (exclue) : la ligne de la facture est en période ouverte, le groupe n'est pas figé (`line_open_in_period`, `letterings.rs:609`,
    `entry_date > borne`), l'écriture de règlement est sous la borne.
  - **État hérité** : facture soldée sans groupe `document` (données d'avant la 15-1a2-i que le rattrapage de la 15-1a2-ii n'a pas lettrées, aucune de leurs lignes
    n'étant alors en période ouverte) — montable en SQL de test seulement (`UPDATE journal_entry_lines SET lettering_key = NULL, lettering_origin = NULL …`).
  Pour R1 et R6 (écriture rapprochée seule, aucune pièce), il s'atteint d'ordinaire. **Le texte du lettrage figé reste juste après la story pour un règlement unique**
  (reculer le verrou avant « la date la plus récente du lettrage », celle du règlement, libère le groupe **et** sort l'écriture de la borne) ; pour le règlement
  **antérieur** d'une facture soldée en plusieurs fois, reculer le verrou avant le dernier règlement libère le groupe, puis `PERIOD_LOCKED` s'annonce pour l'écriture
  plus ancienne (validation P3, L4) — deux motifs successifs, chacun vrai : les clés `reconciliation-cancel-blocked-lettering-closed` et `-group-lettering-closed` **ne
  changent pas** ; le manuel le dit (AC-14). L'ordre des rangs ne change pas (le définitif d'abord, l'ordre réel du socle) : un développeur dont le test
  d'AC-9 rend le lettrage ne **déplace pas** le rang, il monte son état autrement (AC-9). **Qui refuse au clic** : pour les bras R1, R2, R3 et R6, le socle (`create_in_tx_inner`, 400 `PERIOD_LOCKED`), « une seule garde par motif » (`errors.rs:672-675`) ;
  ces gestes ne le refusent pas eux-mêmes — leur `match` de refus est une **liste fermée** (`reconciliation_cancel.rs:817-825` : `FiscalYearClosed | DocumentLetteringInClosedPeriods` ;
  `invoice_settlements_write.rs:1097-1101` ; `supplier_invoices.rs:3523-3525`), où la variante neuve n'entre pas.
  ⚠️ **Validation P1 de la 53-6a (H1) — les deux bras groupés (R4, R5) refusent eux-mêmes TOUT motif de la queue.**
  `grouped_settlements.rs:1812-1816` (`cancel_in_tx`) et `:1988-1992` (`supplier_cancel_in_tx`) : `if let Some(((blocker, _, _), _)) =
  group_cancel_blocker(…).await? { return Err(DbError::ReconciliationNotCancellable { blocker }); }` — aucun filtre (doc `:1735-1739` : « tous
  les refus, avant toute écriture … compte archivé, pas d'exercice ouvert au jour »). Pour un groupe dont l'écriture est sous la borne, le
  rang `PeriodLocked` est donc refusé **par le geste**, en **`409 PERIOD_LOCKED`** (`kesh-api/src/errors.rs:6270-6273`, famille
  `ReconciliationNotCancellable`, texte `reconciliation-cancel-blocked-period-locked`), **sans `details`** — non en `400` du socle. C'est
  déjà le cas aujourd'hui des rangs 4 et 5 pour un groupe (`409 ACCOUNT_ARCHIVED`, `409 FISCAL_YEAR_INVALID`) : le doc de
  `SettlementCancelBlocker` (`kesh-db/src/errors.rs:672-675`, « les rangs 3 à 5 sont refusés par le socle ») est faux pour les groupes depuis
  la 43-1a. **Décision de la fiche** (la moins étendue ; changer le filtre des groupes changerait aussi les rangs 4 et 5, hors périmètre) :
  les gestes groupés **gardent** leur refus sans filtre ; le contrat du clic est `409 PERIOD_LOCKED` pour R4 et R5, `400 PERIOD_LOCKED` pour
  les autres bras ; l'écran ne l'atteint qu'en course (la vue masque *Confirmer*), et `reconciliationCancelErrorMessage` rend alors le texte
  du motif **sans** la borne (aucun `details.lockedThrough` sous `409`) — dit au Dev Agent Record. Le doc de `SettlementCancelBlocker`
  (`:672-675`) et celui de `grouped_settlements::cancel_in_tx` se réécrivent en conséquence (les groupes refusent eux-mêmes tous les rangs).
  Si le développeur préfère filtrer les groupes comme l'unitaire, c'est un changement de contrat des rangs 4 et 5 : il le remonte à
  l'orchestrateur avant de l'écrire. Écran : clé `reconciliation-cancel-blocked-period-locked`, **sans variable** (« L'écriture de ce
  rapprochement est dans une période verrouillée : pour pouvoir l'annuler, un administrateur doit faire reculer le verrou avant sa date. »),
  quatre locales, la borne **ajoutée par l'écran** comme le numéro du compte archivé (`(${label})`, patron d'`ACCOUNT_ARCHIVED`,
  `reconciliation-cancel.ts:76-82`), formatée en date suisse *(annotation du 2026-11-27, revue de code P1, L1 : le `(${label})` accolé après « avant sa date. » se lisait comme la date de l'écriture ; l'écran ajoute désormais la borne en phrase à part, clé `reconciliation-cancel-blocked-period-locked-through` à une variable `$date`, « La période est verrouillée jusqu'au … inclus. » ; la clé du motif reste sans variable pour le serveur)*. ⚠️ **Validation P2 (L2)** : la fiche donnait une clé à deux variables
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
  `PeriodLocked` et `NoOpenFiscalYearToday` y sont donc **inatteignables** dans les familles `SettlementNotCancellable` et
  `SupplierInvoiceNotCancellable` pour **deux** raisons que le commentaire dit (validation P2, L1 : la formule « non parce que `Today` ne produirait pas `PeriodLocked` » contredisait D5, qui ne
  produit ce rang que sous `OriginDate`) : sous `OriginDate` (R2, R3, par le dé-rapprochement), leur `match` de refus est une **liste fermée** et le **socle** refuse
  le rang au clic avec sa propre erreur (`400 PERIOD_LOCKED`) ; sous `Today` (R7 à R9), la queue ne produit pas `PeriodLocked` du tout ; le
  commentaire le dit ainsi. ⚠️ **Validation P1 de la 53-6a (H1)** : dans la famille **`ReconciliationNotCancellable`**
  (`reconciliation_cancel_blocked_text`), le bras `PeriodLocked` est **atteignable** — par les gestes groupés R4 et R5, ci-dessus —, comme
  l'est aujourd'hui `NoOpenFiscalYearToday` pour un groupe sous `Today` (inatteignable sous `OriginDate`) ; son commentaire le dit, et un test
  API le fixe (AC-9). Un grep du symptôme (quatre locales, `i18n-keys.test.ts`, `i18n-un-repli-par-cle.test.ts`) confirme qu'aucune clé
  n'est orpheline et que les replis Rust et TS de la clé neuve sont identiques.
- **D6 — Le dé-rapprochement ne lit plus la date du jour.** `cancel_in_tx`, **`cancel`** (l'enveloppe du pool, `reconciliation_cancel.rs:1104`), **`cancel_blocker`** (`:292`), `get_view`,
  `entry_cancel_outlook`, `group_cancel_blocker`, `supplier_group_cancel_blocker`,
  `grouped_settlements::{cancel_in_tx, supplier_cancel_in_tx}` perdent leur paramètre `aujourd_hui` ; la
  route `post_cancel_reconciliation` (`:6268-6270`), la vue (`:6149`) et les candidates (`:6974`) cessent de lire l'horloge pour ce geste.
  L'inventaire des lectures de l'horloge `crates/kesh-api/tests/date_du_jour_garde.rs` se met à jour en conséquence (lire ce qu'il compte
  avant de l'éditer). *Écartée* : garder `aujourd_hui` « au cas où » — un paramètre inutilisé masque la règle. *(Fin de phrase perdue au
  découpage du 2026-11-27, rétablie depuis `c692c3406` à la validation P1 de la 53-6a.)*

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
  locked` ; au clic forcé par l'API, rien n'est écrit, le lien reste, et le refus est **`400 PERIOD_LOCKED`** (`details.lockedThrough`,
  `details.attempted`) pour R1 et R2, **`409 PERIOD_LOCKED`** sans `details` pour le groupe client R4 (refus du geste groupé, D5, H1 de la
  validation P1 de la 53-6a), texte `reconciliation-cancel-blocked-period-locked`. Pour R1, R2, R4 au moins ; R3 et R5 : la queue commune **et** le geste
  (test `kesh-db` : `cancel_settlement_in_tx` fournisseur rend `DbError::PeriodLocked` du socle — 400 —, `supplier_cancel_in_tx` rend
  `ReconciliationNotCancellable { PeriodLocked }` — 409 —, la queue seule ne distinguant pas les deux statuts ; validation P2, L3).
  ⚠️ **Fixture (validation P2, M1 ; validation P3, M1 ; D5)** : pour R2 à R5, le montage ordinaire (facture réglée en entier, borne sur la date du règlement) rend le **rang 2 bis**
  `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, non `PERIOD_LOCKED`. Le montage **préféré**, par les chemins réels et commun aux quatre bras : la facture datée **après** le
  paiement dans la tolérance (client : lendemain ; fournisseur : jusqu'à sept jours), la borne posée sur la date de l'écriture de règlement (jour de la borne) puis la
  veille (lendemain de la borne : l'annulation passe) — le groupe `document` existe et n'est pas figé. R2 peut aussi monter un règlement **partiel** ; R3, R4 et R5
  **ne le peuvent pas** (D5). Le retrait du groupe en SQL (état hérité) reste un repli, écrit au test. **Précédence** : sous `OriginDate`, deux tests existants
  deviennent les tests de précédence sans changer d'attendu — `reconciliation_e2e.rs` `unreconcile_of_a_closed_lettering_is_refused_in_its_family` (R2, `:7519`,
  verrou à la date de l'encaissement) et `grouped_settlements.rs` `group_unreconciliation_refuses_a_lettering_frozen_by_the_period` (R4, `:1823`, borne au 30.06
  pour un règlement du 15.06) : l'écriture y est sous la borne, et ils rougiraient si l'on plaçait le rang avant le 2 bis ; ils restent verts. Aucun ne couvre le
  chemin **fournisseur** du dé-rapprochement (`supplier_settlement_cancel_e2e.rs:806` `cancel_blocked_by_closed_lettering` passe par la fiche, R8 et R9 ; `grouped_settlements_supplier.rs`
  n'a aucun test du lettrage figé) : **un** test neuf fixe la précédence pour R3 ou R5. R1 (écriture rapprochée seule) n'a pas ce détour. **Les candidates** (`GET …/entry-candidates`, `entry_cancel_outlook`) rendent le même motif pour une écriture
  sous la borne : le dialogue de rattachement l'annonce comme obstacle (case à cocher, `LinkSettlementModal.svelte:160`, `:401-415`) — test API et
  test unitaire du composant (validation P1, L9).
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
  (tableau de cas : ouvert/clos × borne absente/avant/égale/après — « rouvert » est un exercice `Open` comme un autre, qu'une fonction pure ne distingue pas : il se teste en base, AC-3 ; validation P1 de la 53-6a, L), et la précédence de la queue commune pour `OriginDate`
  (`AccountArchived` avant `PeriodLocked`, `FiscalYearClosed` avant tout), paire par paire comme le veut `errors.rs:667`. ⚠️ **Validation P3 (L3)** : cette
  précédence-là ne se teste pas sans base — la queue lit la base. Elle étend le harnais existant `kesh-db/tests/invoice_settlement.rs`
  `la_precedence_de_l_annulation_lecture_et_ecriture` (`RANGS`, `monter`, `ecriture_attendue`, `RANGS` `:1213`, test `:1419-1492` à `0e99cd2db`), aujourd'hui joué sous `Today` par le
  geste de la fiche (`cancel_settlement`, R7) et qui le reste : une seconde passe, sous `OriginDate`, lit `settlement_cancel_blocker` et écrit par
  `cancel_settlement_in_tx` avec cette datation, `PeriodLocked` placé entre `AccountArchived` et `NoOpenFiscalYearToday` (inatteignable sous `OriginDate`, hors des
  cas). Montage du rang seul : l'état de base du harnais (règlement **partiel** de 40.00 sur 100.00, daté `D − 10`) plus `lock_books` à `D − 10` ; la paire
  2 bis × `PeriodLocked` est celle que le montage du 2 bis produit déjà (verrou à `D − 10`). L'attendu d'écriture du rang : `DbError::PeriodLocked` du socle sous
  `OriginDate` ; sous `Today`, jamais produit (bras `false`, comme les motifs d'autres gestes).
- **AC-14 — Manuel et API** (`.tex` et `docs/api-external.md` ; les PDF ne se commitent pas dans `story/…`, CLAUDE.md § *Fusions et fichiers
  communs* — ils se régénèrent à la release) : `user-manual.tex` — § *Annuler un rapprochement* (`:4064-4145` : la puce « datée du jour »,
  la puce « aucun exercice ouvert ne couvre la date du jour » remplacée par la période verrouillée ; l'alinéa de correction des livres déjà touchés **n'est pas écrit ici** : il relève de la 53-6b), encadré
  `sec:date-du-jour` (`:863-866`, le rapprochement sort de la liste), `:3967` et `:3878` relus (exercice clôturé : toujours vrai) ; **sites que la fiche ne nommait pas (validation P1, M9)** :
  `user-manual.tex:384-386` (l'« écart fictif » après annulation : l'inverse n'est plus « datée du jour de l'annulation » — la cause décrite
  disparaît, la phrase se réécrit ou se retire), `:826-836` (encadré « Ce que le verrou n'empêche pas » : « La contre-passation est datée du
  jour, donc après le verrou : elle passe » devient faux **pour l'annulation d'un rapprochement** — Q2), `:3971` (« la fenêtre dit ce qu'il en
  serait aujourd'hui » : la période verrouillée devient un obstacle **annoncé**, non plus seulement possible), l'alinéa « Quand l'annulation n'est
  pas possible » (`:4144`, la puce « aucun exercice ouvert ne couvre la date du jour » **sort** de la liste du dé-rapprochement) ;
  `admin-manual.tex:3556` (`:3559` à `56d47b2cc`) (le verrou de période est aussi le remède du refus d'annuler un **rapprochement** dont l'écriture est sous la borne) ;
  `docs/api-external.md:1005` et `:1007` (lignes de la table des refus du dé-rapprochement : « Aucun exercice ouvert ne couvre la date du jour —
  `FISCAL_YEAR_INVALID` » sort, « Date du jour dans une période verrouillée — `PERIOD_LOCKED` » devient « date de l'écriture ») et `:1065` (champs
  des candidates) ; `docs/
  api-external.md:992` (date de l'origine, refus `PERIOD_LOCKED`, champs `reversalDate` et `reversalFiscalYearName`) et
  `:717` relu. **Sites ajoutés à la validation P2 (L6)** : `api-external.md:990` (vue `GET …/transactions/{id}` : c'est là, non à `:992`,
  que vont `reversalDate`, `reversalFiscalYearName`, la valeur `PERIOD_LOCKED` de `cancelBlockedBy` et la borne dans `cancelBlockedLabel`,
  aujourd'hui « le numéro du compte archivé ») et `:1006` (« L'exercice **du jour** est suivi d'un exercice clôturé — `LATER_FISCAL_YEAR_CLOSED` » :
  l'exercice **de l'écriture**, état hérité seulement, P6) ; `user-manual.tex` § *Un côté d'un virement* (`:4106-4118`, phrase `:4113`, « un exercice
  clôturé ou un lettrage figé ne l'empêchent pas » : ni une période verrouillée, AC-8) ; à l'écran,
  `LinkSettlementModal.svelte:394-399` et la clé `reconciliation-link-entry-cancel-today` (`messages.ftl:1776` et les trois autres locales :
  « une période verrouillée … pourraient l'empêcher, dès aujourd'hui sans que Kesh le voie » — Kesh la voit désormais, la période verrouillée
  sort de cette liste ; texte et `LinkSettlementModal.test.ts:483`). Le même dialogue d'annulation est monté sur la fiche d'une facture
  client (`frontend/src/routes/(app)/invoices/[id]/+page.svelte:1537`) : aucun code n'y change, mais la recette visuelle (AC-15) prend le
  dialogue **depuis cette fiche aussi** (L8). **Sites ajoutés à la validation P1 de la 53-6a (M1)** : dans § *Annuler un rapprochement*,
  les **deux phrases que D1 et D4 prescrivent** et qu'aucune liste ne nommait — la réévaluation de fin d'exercice déjà passée dans l'exercice
  de l'origine est à rejouer après l'annulation (D1), et la contre-passation change les totaux de la période de l'écriture, TVA comprise
  (D4 ; si la période est déclarée, la faire verrouiller d'abord) ; `user-manual.tex:4012` (§ *Défaire un règlement groupé*, « Quand ce
  rapprochement ne peut pas s'annuler --- exercice clôturé, lettrage figé, facture créditée ou soldée » : la période verrouillée s'y ajoute) ;
  `:1119` relu (« Une contre-passation, datée du jour, n'est refusée que si l'exercice du jour est lui-même suivi d'un exercice clôturé » —
  vrai de `/reverse`, plus du dé-rapprochement, dont l'exercice est celui de l'écriture : préciser ou laisser, au Dev Agent Record) ; dans
  la table des refus de `api-external.md:1007`, le statut de `PERIOD_LOCKED` est `400`, **`409` pour un règlement groupé** (D5, H1).
  **Sites ajoutés à la validation P2 (L4, L5)** : `user-manual.tex:812-819` (§ *Verrou de période*, « L'annulation d'un règlement, d'un solde, d'un rapprochement … dont le
  lettrage s'est figé avec la période est donc refusée » : vrai, mais lu seul il dit que le lettrage est la **seule** cause ; l'annulation d'un rapprochement est aussi
  refusée quand son écriture est sous la borne, lettrage ou non — une phrase de plus, ou une précision) ; `docs/api-external.md:1635` (table générale des codes d'erreur :
  `PERIOD_LOCKED` y est `400` pour toute écriture ; ajouter `409`, sans `details`, pour le dé-rapprochement d'un règlement groupé).
  **Site ajouté à la validation P3 (L4)** : dans la liste « Quand l'annulation n'est pas possible » du § *Annuler un rapprochement*, la puce du lettrage figé
  (`user-manual.tex:4135-4141` à `0e99cd2db`, « Pour pouvoir l'annuler, prenez la date la plus récente du lettrage (en général celle du dernier règlement) ») gagne
  une phrase : si l'écriture de ce rapprochement est plus ancienne que le dernier règlement et elle-même sous le verrou, l'annulation est ensuite refusée pour la
  période verrouillée (puce voisine) — le verrou recule avant **sa** date (D5). La puce homonyme du § des règlements (`:1950-1956`, R7) ne change pas.
  Recherche du symptôme dans les `.tex` **et** dans les PDF aplatis (`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' '
  | tr -s ' ' | grep -o -i '.\{100\}datée du jour.\{100\}'`, puis `date du jour`, `aujourd'hui`) : chaque phrase qui parle du
  dé-rapprochement est corrigée, les autres (R7 à R11) restent.
- **AC-15 — E2E Playwright** (`frontend/tests/e2e/reconciliation-cancel.spec.ts`) : (1) annuler le rapprochement d'une écriture d'un
  exercice antérieur ouvert — le dialogue dit la date de l'origine ; après confirmation, *Voir l'écriture* de l'inverse montre cette date et
  cet exercice ; (2) écriture sous la borne du verrou — le motif s'affiche, pas de bouton de confirmation . Recette visuelle (CLAUDE.md) : captures du
  dialogue (date d'origine, phrase TVA), et du motif « période verrouillée », sur une base de démonstration, dans
  `kesh-gate-logs/`.
- **AC-16 — Gates et mutations** : gate complet (`scripts/test-fast.sh`, puis `scripts/gate-ci.sh`), frontend (`check`, `lint-i18n-ownership`, `test:unit`, `build`), E2E complet au dernier commit de code ; mutations M1 à M8, M10 et M11 tuées (M7 angle mort assumé), chacune par le test annoncé.
- **AC-17 — Aucune migration** : `git diff --stat <base>..HEAD -- crates/kesh-db/migrations/` vide ; le schéma de test (`crates/kesh-db/test-schema/`) inchangé.


## Mutations (à jouer au développement, une à une, fichier touché après restauration — mémoire « mutation restaurée, binaire périmé »)

| # | Mutation | Test qui la tue |
|---|---|---|
| M1 | bras `Entry` du dé-rapprochement : `OriginDate` → `Today` | AC-2 (et AC-1, horloge ≠ origine) |
| M2 | bras règlement client : `OriginDate` → `Today` | AC-5 |
| M3 | bras règlement fournisseur : `OriginDate` → `Today` | AC-6 |
| M4 | groupe client : `OriginDate` → `Today` | AC-7 (client) |
| M5 | groupe fournisseur : `OriginDate` → `Today` | AC-7 (fournisseur) |
| M6 | rang `PeriodLocked` retiré de la queue commune | AC-9 (vue **et** candidates `entry-candidates`, qui lisent la même queue) — pour R1 et R2, le clic reste refusé par le socle : la mutation ne doit tuer **que** ces tests de lecture, jamais celui du clic (validation P2, L5) ; pour le groupe R4, elle tue **aussi** le test du clic (`409` → `400` du socle, D5, H1 de la validation P1 de la 53-6a), attendu — sur l'état monté pour AC-9 (règlement sans lettrage figé, validation P2, M1) |
| M7 | confirmation de l'étape (4) remplacée par une lecture ordinaire | **angle mort assumé d'avance** (D2 : le verrou de l'étape (3) précède toute lecture) ; à tuer seulement si un entrelacement le permet |
| M8 | gestes de la fiche (R7) passés en `OriginDate` | AC-11 |
| M10 | date de l'inverse lue sur la **transaction bancaire** au lieu de l'écriture (bras `Entry`, rattachement d'une autre date) | AC-1 (date de la transaction ≠ date de l'écriture) |
| M11 | seuil du rang `PeriodLocked` de la queue : `<=` → `<` | AC-9 au **jour de la borne** (la vue doit refuser ; le socle, lui, garde `<=`) |

## Tâches

- [x] **T0 — Reprise et relecture** : reprendre `origin/main` dans la branche de la story (demander d'abord à l'orchestrateur : reprise directe, ou attente de la reprise par la branche d'epic) ; résoudre `supplier_invoices.rs` (`DuplicatePolicy`) en gardant les deux intentions ; relire au code chaque prémisse P1 à P8 et P11 et chaque numéro de ligne de la fiche, corriger la fiche s'il le faut ; confirmer P7 (date de l'écriture de chaque chemin d'acceptation).
- [x] **T1 — Tests rouges d'abord** (AC-1 à AC-7, AC-9 ; AC-11, test de non-régression, est **vert** dès l'écriture et le reste) ; commit.
- [x] **T2 — Socle** (D2) : `ReversalDating::OriginDate`, lecture de `je.entry_date` à l'étape (1), étape (4), audit `dating` ; fonction pure
  du verdict + ses tests (AC-13) ; signatures de `reverse_in_tx`/`reverse_owned_in_tx` et de leurs appelants (R1 à R10) ; commit.
- [x] **T3 — Queue commune et gestes** (D5, D6) : datation passée à `settlement_entry_cancel_blocker`, `settlement_cancel_blocker*`,
  `supplier_settlement_cancel_blocker*`, `group_cancel_blocker`, `supplier_group_cancel_blocker` ; rang `PeriodLocked` ; retrait de
  `aujourd_hui` du dé-rapprochement ; `date_du_jour_garde.rs` ; commit.
- [x] **T4 — Vue et API** (D4) : `reversalDate`, `reversalFiscalYearName` ; mapping de `PERIOD_LOCKED` dans `crates/kesh-api/src/errors.rs`
  (fallbacks, quatre locales) ; commit.
- [x] **T5 — Écran** : `CancelReconciliationDialog.svelte`, `reconciliation-cancel.ts`, `reconciliation.types.ts`, `reconciliation.api.ts`, `LinkSettlementModal.svelte` (relire ce qu'il affiche des motifs) ; tests unitaires (`CancelReconciliationDialog.test.ts`) ; i18n aux quatre locales, `sitesTotal` de `i18n-keys.test.ts` recompté ; commit.
- [x] **T6 — Manuel et API** (AC-14) ; commit.
- [x] **T7 — E2E** (AC-15), recette visuelle, gates, mutations (AC-16) ; Dev Agent Record ; commit et push de la seule branche `story/…` à chaque étape.


## Dev Notes

### Fichiers touchés (prévus)

- `crates/kesh-db/src/repositories/journal_entries.rs` — `ReversalDating`, `reverse_in_tx`, `reverse_owned_in_tx`, `reverse_in_tx_inner`
  (étapes 1 et 4, audit) ; doc-comments qui disent « datée du jour » pour le dé-rapprochement.
- `crates/kesh-db/src/repositories/reconciliation_cancel.rs` — `cancel_in_tx`, `get_view`, `entry_cancel_outlook`, `entry_cancel_chain`,
  `cancel_blocker`, `ReconciliationView` (+ date et exercice), `ReconciliationCancellation` (doc « datée du jour », `:616`).
- `crates/kesh-db/src/repositories/settlement_cancellation.rs` — datation en paramètre, rang `PeriodLocked`, doc de la « limite assumée »
  (`:50-52`) réécrite.
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`, `supplier_invoices.rs`, `grouped_settlements.rs` — datation passée.
- `crates/kesh-db/src/errors.rs` — `SettlementCancelBlocker::PeriodLocked` (place dans l'enum = précédence, doc `:667`), `code()`.
- `crates/kesh-api/src/routes/reconciliation.rs` — route du geste, vue, candidates, `crates/kesh-api/src/errors.rs` — mapping, fallbacks ; `crates/kesh-api/src/services/factures.rs:585`
  (appelle `settlement_cancel_blocker` : passe `Today`).
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`.
- `frontend/src/lib/features/reconciliation/` .
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
  `reconciliation.types.ts:345`, `reconciliation.api.ts:141` ; **ajoutés à la validation P1 de la 53-6a (L)** : `reconciliation_cancel.rs` doc d'`entry_cancel_outlook` (`:361-363`, « angles morts … verrou de période du jour ») ; `grouped_settlements.rs:1723` (`GroupedCancellation`, « datée du jour ») et `:1735-1739` (doc de `cancel_in_tx`, liste des refus) ; `kesh-db/src/errors.rs:672-675` (« les rangs 3 à 5 sont refusés par le socle », faux pour les groupes, D5) ; `reconciliation-cancel.ts:7` et `:163-167` (« huit motifs », `PERIOD_LOCKED` donné comme code hors motifs).
  **Tests dont le refus du rang 5 disparaît pour le dé-rapprochement (validation P2, L6)** : `kesh-db/tests/grouped_settlements.rs` —
  `group_unreconciliation_refuses_without_an_open_year_today` (`NoOpenFiscalYearToday` d'un groupe par une horloge hors exercice) n'a plus d'objet (D6 : plus d'horloge ;
  l'exercice de l'origine, ouvert, couvre sa date) : il se **remplace** par le test `PeriodLocked` d'AC-9, et l'aide `refus_du_derapprochement` (`aujourd_hui` en paramètre)
  perd ce paramètre ; les `match` exhaustifs sur `SettlementCancelBlocker` des tests (`invoice_settlement.rs`, autour de `:1401`, `supplier_invoices_repository.rs`, autour de
  `:1829` et `:2156`) gagnent le bras `PeriodLocked` (le compilateur les trouve ; l'attendu à choisir : `DbError::PeriodLocked` du socle pour R2 et R3).
  **Attendus à relire, hors des comptes ci-dessus (validation P1 de la 53-6a, L)** : `kesh-db/tests/grouped_settlements.rs:2896-2908` et `grouped_settlements_supplier.rs:2220-2227` figent la suite d'audit du dé-rapprochement groupé, dont l'`entryDate` de l'inverse et le JSON `journal_entry.reversed` sans `dating` : ils changent d'attendu (date de l'origine, `"dating": "originDate"`) — ils ne sont pas des sites de signature seulement.
  ⚠️ **Validation P2 (M1, L3) — l'inventaire ci-dessus n'est pas sorti de sa commande, et il omettait le frontend.** (a) Le motif littéral
  `transactions/{}/cancel` ne trouve pas les `format!("…/transactions/{t}/cancel")` des tests, et les nombres entre parenthèses ne sont pas ceux
  qu'elle rend (à `56d47b2cc`, la commande rend : `bank_account_closed_inventory.rs` 15, `supplier_invoices_repository.rs` 5,
  `invoice_settlement.rs` 5, `letterings_lexical.rs` 2, et `invoice_reminders_repository.rs` 4, `migrations_upgrade_path.rs` 2, absents de la
  liste). *(Annotation de la validation P3, 2026-11-27, L1 : ces nombres étaient faux à leur date, comme deux de ceux de la liste de la P1 ci-dessus (18 et 12). Recomptés par `git grep -c` de la commande de la P1, identiques à
  `56d47b2cc` et à `0e99cd2db` : `bank_account_closed_inventory.rs` 23, `bank_accounts_closed_e2e.rs` 4, `jour_suisse_nuit_e2e.rs` 1, `filet_bilan_clos.rs` 2,
  `grouped_settlements.rs` 1, `grouped_settlements_supplier.rs` 2, `invoice_reminders_repository.rs` 4, `invoice_settlement.rs` 7, `invoice_write_off.rs` 4,
  `journal_entries_modification.rs` 2, `lettering_documents.rs` 1, `letterings_lexical.rs` 11, `migrations_upgrade_path.rs` 2, `supplier_invoices_repository.rs` 9 —
  14 fichiers, 73 lignes. Les deux autres commandes rendent bien 12 fichiers chacune.)* Commandes **qui font foi**, périmètre `origin/main` à `56d47b2cc`, à rejouer à T0 sur la branche reprise :
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
  à réécrire sur le texte neuf) et `LinkSettlementModal.test.ts:483` (texte `reconciliation-link-entry-cancel-today`, AC-14) changent d'attendu.
- Documentation : `docs/manual/fr/user-manual.tex`, `docs/api-external.md`. Pas de CHANGELOG (assemblé à l'intégration), pas de PDF.

### Ce qui doit être préservé

- L'ordre des étapes du dé-rapprochement (doc de `cancel_in_tx`, `reconciliation_cancel.rs:637-699`) : verrous avant toute lecture
  ordinaire, lien défait **avant** la contre-passation, revérification verrouillante du compte clos en **dernière** acquisition, audit.
- Le retrait seul (`unlink_only`), le rang 3 exempté (`unlinking`), les lignes sœurs Wise, `expectUnlinkOnly` / `409
  RECONCILIATION_CANCEL_CHANGED`.
- Le JSON de `journal_entry.reversed` du chemin `Today` (T-b2 de la 29-1b), et celui de `NextFiscalYearStart`.
- Les gestes R7 à R11 (AC-11).

- **Le périmètre de la 53-6a** : ni route de détection, ni panneau, ni procédure de correction (53-6b).
 
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

Modules dont le code de production change : `kesh-db`, `kesh-api`, `kesh-i18n`, `frontend/…/reconciliation` — **quatre**, sous la borne de cinq. Le découpage de la 53-6 en 53-6a et 53-6b (décision du Project Lead du 2026-11-27, signal de recyclage de la validation P2 : trois MEDIUM sur trois nés d'un remède de la P1, tous sur le texte de la fiche) retire à cette story la route de détection et le panneau ; Q3 (gestes de la fiche) en est restée hors, en trois issues.

### Migration

**Aucune.** Rien ne change au schéma. *Migration breaking policy* sans objet (P1-P7 non déclenchés) ; aucun backfill : les livres déjà touchés ne se réparent pas d'office (CLAUDE.md § *Ce qu'impose la production*) ; leur détection est la 53-6b.

### Notes de version (à reprendre dans le CHANGELOG à l'intégration)

> ⚠️ **Annuler un rapprochement bancaire contre-passe désormais l'écriture à sa propre date, dans son propre exercice** (#1674). Jusqu'ici, l'écriture inverse était datée du jour : une annulation faite après la fin de l'exercice de l'écriture — exercice rouvert, ou pas encore clôturé — la plaçait dans l'exercice courant, et deux exercices étaient faux. Conséquences : **une annulation acceptée jusqu'ici peut être refusée** — celle dont l'écriture est dans une **période verrouillée** (l'écran le dit d'avance ; un administrateur recule le verrou) ; celle d'un exercice clôturé l'était déjà ; la contre-passation change les totaux de la période de l'écriture, TVA comprise. Les annulations faites avec les versions précédentes ne sont pas corrigées par cette release.
> **API** : `POST /api/v1/reconciliation/transactions/{id}/cancel` date l'inverse à la date de l'écriture ; refus nouveau `400 PERIOD_LOCKED` ; `409 PERIOD_LOCKED` pour un règlement groupé ; le refus `FISCAL_YEAR_INVALID` (« aucun exercice ouvert ne couvre la date du jour ») ne se produit plus pour ce geste ; `GET …/transactions/{id}` gagne `reversalDate`, `reversalFiscalYearName` et la valeur `PERIOD_LOCKED` de `cancelBlockedBy` (borne dans `cancelBlockedLabel`), et `GET …/transactions/{id}/entry-candidates` la même valeur ; le détail d'audit `journal_entry.reversed` de ce geste gagne `"dating": "originDate"`. Aucune migration.

### Issues à ouvrir (Q3, par l'orchestrateur ; une par problème, ≥ 5 s d'écart, toutes trois P1)

Corps et titres : `kesh-gate-logs/e53/53-6-issues-q3/` (`R7.md`, `R8.md`, `R9.md`, `titres.txt`).

### Références

- Issue #1674 (texte et « Correctif décidé », Project Lead, 2026-11-26) ; décisions du 2026-11-27 (Q1 à Q6).
- Fiche d'origine, désormais page de renvoi : `53-6-annulation-rapprochement-exercice.md` ; rapports : `kesh-gate-logs/e53/53-6-validation-p1.md`, `…-p2.md`.
- `CLAUDE.md` : *Ce qu'impose la production*, *Priorités des défauts*, *Migration breaking policy*, *Règle de splitting préventif*, *Versions de l'API*, *Recette visuelle*.
- Stories : 25-3-zero (socle unique), 25-3-a-1 (queue commune), 25-3-b (dé-rapprochement), 29-1b (`ReversalDating`, `NextFiscalYearStart`,
  patron de la confirmation), 24-4c (verrou de période), 35-5a0 (date du jour lue une fois), 39-11c (Wise groupé), 43-1a/43-2 (groupes),
  53-1a et 53-3 (ordre des verrous dans l'epic).
- `docs/testing.md` (Playwright local), `docs/agents-orchestration.md` (consignes des agents).

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`claude-opus-5-5`), `bmad-dev-story`, 2026-11-27, worktree `kesh-53-6`.

### Completion Notes List

**T0 — reprise et relecture.** `origin/main` (`0e99cd2db`, v0.20.0) repris par fusion (`95c577893`) : conflits README (ligne publiée de la v0.20.0, ligne v0.21.0 prévue gardées), `sprint-status.yaml` (lignes `last_updated` des deux côtés), compteurs i18n recomptés par la garde (`sitesTotal` 3973, `CANDIDATES_ATTENDUES` 169) ; `supplier_invoices.rs` (`DuplicatePolicy`) fusionné sans conflit (réexport de `duplicate_policy`). #1674 ouverte (`gh issue view 1674 --json state` → `OPEN`, lecture seule). Lignes relues sur la branche reprise (`git show 95c577893:<fichier> | grep -nF`) : route `aujourd_hui` `:6270` (inchangé), bras `Entry` `reconciliation_cancel.rs:932`, refus fermé `:818`, « limite assumée » `settlement_cancellation.rs:50`, `ReversalDating` `journal_entries.rs:3201` (la fiche disait `:2942` à `main`, `:3170` à la branche d'epic — 53-1b depuis), bras `Today` `:3513`, garde `PeriodLocked` du socle `:684`, refus groupés `grouped_settlements.rs:1813`, `:1989`, `reconciliation_cancel_blocked_text` `kesh-api/src/errors.rs:6922`, FTL `:964`, doc `SettlementCancelBlocker` `kesh-db/src/errors.rs:681`, API `:999`, manuel `:4090` (§ *Annuler un rapprochement*). Toutes les prémisses P1 à P8 et P11 tiennent ; seuls les numéros ont bougé. P7 confirmée : l'éclatement date son écriture de la transaction (`kesh-reconciliation/src/split.rs:72`), le rapprochement manuel reçoit la date du gestionnaire, l'acceptation de facture `routes/reconciliation.rs:3044`. LOW L1 de la P4 corrigé dans D5 (R5 : `:2438`, `:2574`).

**D2 — remède (i) retenu** : `pub enum CancellationDating { Today(NaiveDate), OriginDate }`, converti en `ReversalDating` (privé, troisième valeur `OriginDate`) ; `reverse_in_tx` et `reverse_owned_in_tx` la prennent ; `reverse_at_next_fiscal_year_start_in_tx` inchangée ; aucune entrée ajoutée. Étape (1) : la requête verrouillante lit aussi `je.entry_date`, `je.fiscal_year_id`, `fy.status` (même ligne). Étape (4) `OriginDate` : `reversal_date_verdict` (fonction pure, AC-13, 24 cas) puis `find_open_covering_date` sur la date de l'origine (`None` → `FiscalYearClosed`, autre exercice → `Invariant`) ; audit `"dating": "originDate"`.

**D4 — phrase TVA montrée toujours** (hors retrait seul) : Kesh ne tient pas la périodicité de TVA, et le seuil « mois courant » exigerait le jour, que la vue ne lit plus (D6). Clé `reconciliation-cancel-confirm-vat`.

**D5** — rang `SettlementCancelBlocker::PeriodLocked` (code `PERIOD_LOCKED`, borne en étiquette) entre `AccountArchived` et `NoOpenFiscalYearToday`, évalué sous `OriginDate` seulement, par la même règle pure que le socle (`cancellation_date_verdict`, seuil inclusif). Les gestes groupés gardent leur refus sans filtre : `409 PERIOD_LOCKED` sans `details` (testé, `reconciliation_group_e2e::s536a_un_groupe_sous_le_verrou_est_refuse_en_409`) ; au clic d'un geste unitaire, `400 PERIOD_LOCKED` du socle (`details.lockedThrough`, `attempted`). Au clic d'un 409 groupé, l'écran rend le motif sans borne (aucun `details`).

**D6** — `cancel_in_tx`, `cancel`, `cancel_blocker`, `get_view`, `entry_cancel_outlook`, `entry_cancel_chain`, `group_cancel_blocker`, `supplier_group_cancel_blocker`, `grouped_settlements::{cancel_in_tx, supplier_cancel_in_tx}` perdent `aujourd_hui` ; la route du geste, la vue et les candidates ne lisent plus l'horloge. `date_du_jour_garde.rs` **non édité** : il inventorie les dates tirées **hors** de l'horloge (`FORMES`, l. 136 : `.date_naive(`, `CURDATE(`…), non les lectures de `state.horloge` ; les lectures retirées n'y figuraient pas (binaire vert, `t4-kesh-api-cible-1.log`).

**Gestes depuis la fiche (R7 à R9)** : `CancellationDating::Today(aujourd_hui)`, inchangés (Q3) ; AC-11 fixé par les tests existants `jour_suisse_nuit_e2e::ac3_annulation_d_un_reglement_client_la_nuit` (R7), `ac3_annulation_d_un_reglement_fournisseur_la_nuit` (R8), `ac3_annulation_d_une_facture_fournisseur_la_nuit` (R9) et `ac3_sans_exercice_au_jour_suisse_l_annulation_est_refusee` (rang 5), verts.

**Tests par critère** :
*(Tableau réécrit à la remédiation de la revue P1, M2 : chaque affirmation renvoie à un test qui existe, par bras — R1 écriture seule, R2 règlement client, R3 règlement fournisseur, R4 groupe client, R5 groupe fournisseur, R6 Wise groupé. Les tests marqués « neuf » sont de cette remédiation.)*
- AC-1 (même exercice, jour ≠ date de l'écriture) : **R1** `jour_suisse_nuit_e2e::s536a_ac1_une_ecriture_du_meme_exercice_est_contre_passee_a_sa_date` (neuf : horloge au 22.12.2025, écriture du 15.12, transaction du 20.12) et l'E2E Playwright « même exercice » ; **R3** `reconciliation_cancel::unreconciling_a_supplier_settlement_undoes_it` (attendu changé : 15.06.2026) ; **R4** `grouped_settlements::s536a_group_unreconciliation_refuses_an_entry_under_the_period_lock` (15.06, au lendemain de la borne) ; **R5** `grouped_settlements_supplier::s536a_a_supplier_group_under_the_period_lock_is_refused` (idem). **R2 et R6 : pas de test « même exercice » dédié** (limite déclarée) — leur datation est tuée par les tests d'exercice antérieur ci-dessous (M-R2, M-R1).
- AC-1 à AC-2 combinés, **R1** : `jour_suisse_nuit_e2e::s536a_ac1_l_annulation_est_datee_de_l_ecriture_dans_son_exercice` (horloge au 1.1.2026, écriture du 15.12.2025 rattachée à une transaction du 20.12 : date de l'écriture, exercice 2025, rien dans 2026, libellé, `dating`).
- AC-2 (exercice antérieur ouvert, jamais clôturé) : **R1** `reconciliation_cancel::s536a_ac2_…` et le test précédent ; **R3** `reconciliation_cancel::s536a_ac6_a_supplier_settlement_reverses_in_its_open_year` ; **R4** `grouped_settlements::s536a_group_unreconciliation_reverses_in_the_entry_year` ; **R5** `grouped_settlements_supplier::s536a_ac7_a_supplier_group_of_an_open_earlier_year_reverses_in_that_year` (neuf) ; **R2** `jour_suisse_nuit_e2e::s536a_ac5_un_reglement_client_rattache_se_contre_passe_a_sa_date` (au lendemain de la borne) et `s536a_ac5_un_reglement_qui_solde_la_facture_la_rend_a_regler` (neuf) ; **R6** `wise_reconciliation_e2e::s536a_ac7_un_paiement_wise_groupe_se_contre_passe_dans_son_exercice` (neuf).
- AC-3 (exercice clôturé puis rouvert, reclôturé) : **R1** `reconciliation_cancel::s536a_ac3_un_exercice_rouvert_recoit_l_inverse_et_se_recloture` et `jour_suisse_nuit_e2e::s536a_ac3_une_ecriture_d_un_exercice_rouvert_est_contre_passee_dans_celui_ci` (neuf) ; **R2** `jour_suisse_nuit_e2e::s536a_ac3_un_reglement_client_d_un_exercice_rouvert_se_contre_passe_dans_celui_ci` (neuf) ; **R3** `reconciliation_cancel::s536a_ac6_a_supplier_settlement_reverses_in_a_reopened_year` (neuf) ; **R4** `grouped_settlements::s536a_group_unreconciliation_reverses_in_a_reopened_year` (neuf) ; **R5** `grouped_settlements_supplier::s536a_ac7_a_supplier_group_of_a_reopened_year_reverses_in_that_year` (neuf) ; **R6** le même test Wise (phases « clos » puis « rouvert », le statut posé en SQL : la clôture réelle exigerait la réévaluation des soldes en devises du montage).
- AC-4 (exercice de l'écriture clos : refus, rien d'écrit, le lien reste) : **R1** `jour_suisse_nuit_e2e::s536a_ac4_un_exercice_clos_refuse_l_annulation` (409, texte fr) ; **R2** `jour_suisse_nuit_e2e::s536a_ac4_un_reglement_client_d_un_exercice_clos_refuse_l_annulation` (neuf) ; **R3** `reconciliation_cancel::a_closed_year_refuses_the_supplier_unreconciliation` ; **R4** `grouped_settlements::group_unreconciliation_refuses_a_closed_year` ; **R5** `grouped_settlements_supplier::s536a_ac7_a_supplier_group_of_a_closed_year_is_refused` (neuf) ; **R6** phase « clos » du test Wise. `reconciliation_cancel::a_concurrent_close_waits_for_the_unreconciliation` est la course d'une clôture contre une écriture propre (R1), non un cas d'AC-4 pour R2 à R5 *(la version précédente de cette ligne l'y rangeait à tort)*. Ces tests exercent le rang 2, **commun** à tous les bras (`settlement_entry_cancel_blocker`) : ils ne dépendent pas de la datation du bras, et aucune mutation de datation ne les rougit. Le message aux quatre locales : clé existante, parité des locales contrôlée par les gardes i18n.
- AC-5 : `jour_suisse_nuit_e2e::s536a_ac5_un_reglement_client_rattache_se_contre_passe_a_sa_date` (règlement partiel rattaché, verrou — `cancelBlockedLabel`, `details.attempted`, rien d'écrit, le lien et le règlement restent —, puis datée du règlement, règlement retiré) et `s536a_ac5_un_reglement_qui_solde_la_facture_la_rend_a_regler` (neuf : la facture soldée redevient à régler, `paid_at` revenu à `NULL` ; le lettrage `reversal` du règlement et de son miroir, une seule clé, dans 2025).
- AC-6 : `reconciliation_cancel::s536a_ac6_a_supplier_settlement_reverses_in_its_open_year` (2019 ouvert), `s536a_ac6_a_supplier_settlement_reverses_in_a_reopened_year` (neuf) et le test fournisseur existant.
- AC-7 : R4 `grouped_settlements::s536a_group_unreconciliation_reverses_in_the_entry_year` (2019) et `…_in_a_reopened_year` (neuf) ; R5 : les trois tests neufs `grouped_settlements_supplier::s536a_ac7_*` (exercice antérieur ouvert, rouvert, clos), `s536a_a_supplier_group_under_the_period_lock_is_refused` (datée de l'écriture) et le relevé de caractérisation (attendu changé) ; R6 (Wise groupé) : le test dédié `wise_reconciliation_e2e::s536a_ac7_un_paiement_wise_groupe_se_contre_passe_dans_son_exercice` (neuf ; le geste passe par le bras R1).
- AC-8 : retrait seul inchangé (pas de rang pour `unlink_only`, `entry_cancel_outlook` retourne avant la queue) ; tests existants du retrait seul verts ; manuel et API disent « ni une période verrouillée ».
- AC-9 : R1 vue, clic 400, rien d'écrit, **le lien reste (assertion directe sur `matched_entry_id`)** et lendemain de la borne (`s536a_ac9_une_ecriture_sous_le_verrou_est_refusee`), candidates (`s536a_ac9_une_candidate_sous_le_verrou_l_annonce`), R2 (`s536a_ac5_un_reglement_client_rattache_se_contre_passe_a_sa_date` : `cancelBlockedLabel`, `details.lockedThrough` et `attempted`, rien d'écrit, le lien et le règlement restent), R3 (`reconciliation_cancel::s536a_a_supplier_settlement_under_the_period_lock_is_refused_by_the_core`, montage de repli dit : la fiche l'admet ; le chemin réel — facture datée après le paiement, tolérance de sept jours — est joué pour R5), R4 (kesh-db et 409 API), R5 (`ReconciliationNotCancellable { PeriodLocked }`) ; précédence : tests existants (R2, R4) verts, test neuf fournisseur `s536a_a_frozen_supplier_lettering_is_named_before_the_period_lock` ; écran : `CancelReconciliationDialog.test.ts`, `LinkSettlementModal.test.ts`.
- AC-10 : vue (`reversalDate`, `reversalFiscalYearName`) et cinq confirmations (Vitest, cinq genres) ; **nuls pour un retrait seul** (`reconciliation_link_entry_e2e::s34_11b_ac4_annuler_un_cote_puis_l_autre`, ajout de la remédiation : `null` au côté défait seul, une date et un exercice au dernier côté) ; dialogue d'une vue sans date ni exercice (serveur antérieur) : un tiret, non une phrase tronquée (`CancelReconciliationDialog.test.ts`).
- AC-12 : (a) `reconciliation_cancel::s536a_ac12_un_autre_exercice_que_celui_de_l_origine_est_un_invariant` (état forgé) ; (b) `a_concurrent_close_waits_for_the_unreconciliation` (clôture concurrente : l'annulation attend et refuse `FISCAL_YEAR_CLOSED`, jamais d'inverse dans un exercice clos) ; (c) mesure : `s536a_ac12_la_contre_passation_datee_de_l_origine_ne_tient_pas_l_exercice_du_jour` (sonde `FOR UPDATE NOWAIT`, origine en 2019, jour dans l'exercice suivant) — sous `OriginDate`, 2019 tenu, l'exercice du jour libre ; sous `Today`, les deux tenus. **Portée exacte de la sonde** (revue P1, L7) : elle appelle `reverse_in_tx` seul, non le geste entier `cancel_in_tx`, et ne regarde que deux lignes de `fiscal_years` ; elle établit que l'exercice du jour est tenu sous `Today` et libre sous `OriginDate` (la thèse de D3), non que « 2019 tenu » vienne de l'étape (4) (l'étape (1) le tient déjà), ni l'absence de verrou ailleurs ou d'intervalle. Écrite au doc-comment du test.
- AC-13 : `journal_entries::verdict_de_datation::la_table_des_cas` (24 cas) ; précédence en base : `invoice_settlement::s536a_la_precedence_sous_la_date_de_l_origine` (seconde passe du harnais, 20 cas) ; `s536a_le_socle_refuse_une_origine_d_un_exercice_clos` (filet).
- AC-14 : manuel (`user-manual.tex`, `admin-manual.tex`) et `api-external.md` ; recherche du symptôme dans le PDF aplati (`pdftotext user-manual.pdf`, v0.20.0) : le seul « datée du jour » du dé-rapprochement (§ *Annuler un rapprochement*) est corrigé au `.tex` ; les autres (R7 à R11, fiche fournisseur) restent. `:1119` (exercices en désordre) précisé. Les manuels DE/IT/EN n'existent pas au dépôt (`docs/manual/{de,en,it}` absents). PDF non régénérés (release).
- AC-15 : `frontend/tests/e2e/reconciliation-cancel.spec.ts` — « même exercice » (dialogue daté, exercice nommé entre guillemets, phrase TVA complète), « période verrouillée » (borne au 30.06 distincte de la date de l'écriture du 15.05 : « La période est verrouillée jusqu'au 30.06.2026 inclus. », sans *Confirmer*) et, **revue P1 M1, le cas du défaut**, `53-6a — une écriture d'un exercice antérieur encore ouvert` : société sans exercice (`with-company-no-fy`), exercices 2025 et 2026 ouverts, relevé de fin 2025 (transaction du 15.12), rapprochement manuel daté du **12.12.2025** (la date de l'écriture diffère de celle de la transaction) ; le dialogue dit « datée du 12.12.2025 » et « dans l'exercice « Exercice 2025 » » ; après confirmation, l'inverse s'ouvre **par clics** (*Voir l'écriture* de la ligne, puis le renvoi « Contre-passée par l'écriture n° … » de l'écriture d'origine — le lien de la ligne disparaît avec le rapprochement), montre le 12.12.2025, et l'API dit son exercice (2025, jamais l'exercice du jour).
- AC-17 : `git diff --stat 0e99cd2db..HEAD -- crates/kesh-db/migrations/ crates/kesh-db/test-schema/` vide.

**Hors périmètre, à signaler** : aucune issue neuve demandée (les trois issues Q3 sont ouvertes par l'orchestrateur, textes `kesh-gate-logs/e53/53-6-issues-q3/`). *(Remarque cosmétique d'origine — la borne ajoutée après le point final du motif, « … avant sa date. (15.05.2026) » — corrigée à la revue P1, L1 : le motif ajoute désormais une phrase à part, « La période est verrouillée jusqu'au … inclus. ».)*

**Mutations (AC-16)** — campagne `kesh-gate-logs/e53/s536a/mutations/` (`resume.txt`, un journal par mutation), à `7ebaa0107`, restauration par copie du fichier sauvegardé puis `touch`, `git diff` vide après chacune :

| # | Résultat | Tuée par |
|---|---|---|
| M1 | tuée | `jour_suisse_nuit_e2e::s536a_ac1`, `s536a_ac9_une_ecriture…`, `reconciliation_cancel::s536a_ac2`, `s536a_ac3` |
| M2 | tuée | `jour_suisse_nuit_e2e::s536a_ac5` |
| M3 | tuée | `reconciliation_cancel::s536a_ac6`, `s536a_a_supplier_settlement_under_the_period_lock…` |
| M4 | tuée | `grouped_settlements::s536a_group_unreconciliation_refuses_an_entry_under_the_period_lock`, `s536a_group_unreconciliation_reverses_in_the_entry_year` |
| M5 | tuée | `grouped_settlements_supplier::s536a_a_supplier_group_under_the_period_lock_is_refused` |
| M6 | tuée | huit tests de lecture (vue et candidates, kesh-db et kesh-api), et le 409 groupé |
| M7 | **survit — mutant équivalent** (revue P1, L6) : l'étape (1) du socle verrouille, pour tout appelant, l'écriture d'origine et son exercice (`… JOIN fiscal_years … FOR UPDATE`) ; quand `find_open_covering_date` relit cette ligne, la transaction la tient déjà, et une lecture ordinaire verrait le même état — pour le dé-rapprochement **comme pour un appelant futur** (le motif « défense d'un appelant futur » écrit ici et au commentaire était inexact, corrigé). Le contrôle `fy.id != origin_fy_id` garde sa valeur (AC-12 a, état forgé) ; ce n'est pas un angle mort de test |
| M8 | tuée | `jour_suisse_nuit_e2e::ac3_annulation_d_un_reglement_client_la_nuit`, `ac3_sans_exercice_au_jour_suisse_l_annulation_est_refusee` (AC-11) |
| M10 | tuée | `jour_suisse_nuit_e2e::s536a_ac1` (transaction du 20.12, écriture du 15.12), et deux autres |
| M11 | tuée | neuf tests, dont la table pure, `s536a_ac9_*`, le 409 groupé et les tests de groupes |

**Mutations de la remédiation (revue P1, M2)** — un seul script tenant les verrous de la station, `$GATE_LOGS/e53/s536a/remediation/gate-rust.sh` (journaux `mutations/M-R*.log`, `rust-codes.txt`), à `9a12c5b0f` ; **une mutation de la datation par bras** : `CancellationDating::OriginDate` → `CancellationDating::Today(chrono::Utc::now().date_naive())` à la ligne du bras, puis restauration par copie du fichier sauvegardé, `touch`, `git diff` vide (contrôlé après chacune). Chaque test neuf ne traverse qu'**un** bras ; il se prouve rouge sous la mutation de **ce** bras, sur la date ou l'exercice de l'inverse (`left: (2026-11-27, …)`, la date du jour, `right: (date de l'écriture, exercice de l'écriture)`) :

| Mutation | Bras | Tests neufs rouges (sain : verts) |
|---|---|---|
| M-R1 (`reconciliation_cancel.rs:978`, bras `Entry`) | R1, et R6 qui passe par lui | `jour_suisse_nuit_e2e::s536a_ac1_une_ecriture_du_meme_exercice_est_contre_passee_a_sa_date`, `s536a_ac3_une_ecriture_d_un_exercice_rouvert_est_contre_passee_dans_celui_ci`, `wise_reconciliation_e2e::s536a_ac7_un_paiement_wise_groupe_se_contre_passe_dans_son_exercice` (3 sur 3 ; à `9a12c5b0f` le test Wise rougissait par `409 MULTI_CURRENCY_DISABLED` (exercice « suivant » du décor mono-devise), non par la date : **corrigé au lot de la revue P2**, voir la ligne suivante) |
| M-R1, rejouée au lot de la revue P2 (P2-L1, P2-L5), `$GATE_LOGS/e53/s536a/remediation-p2/` | R1 et R6 | l'exercice suivant du test Wise est créé `multi_currency = TRUE` : `wise_reconciliation_e2e::s536a_ac7_un_paiement_wise_groupe_se_contre_passe_dans_son_exercice` rougit désormais **sur la date** (`left: [(2026-11-27, 2)]`, `right: [(2020-03-10, 1)]`, `wise_reconciliation_e2e.rs:1098`) ; test neuf `s536a_ac1_un_paiement_wise_groupe_du_meme_exercice_se_contre_passe_a_sa_date` (AC-1 même exercice, R6) rouge aussi sur la date (`left: [(2026-11-27, 1)]`, `right: [(2020-03-10, 1)]`, `:1147`) |
| M-R2, rejouée au même lot (P2-L5) | R2 | test neuf `jour_suisse_nuit_e2e::s536a_ac1_un_reglement_client_du_meme_exercice_se_contre_passe_a_sa_date` (AC-1 même exercice, R2 : règlement du 20.12.2025, jour le 22.12.2025) rouge sur la date (`left: (2026-11-27, 2)`, `right: (2025-12-20, 1)`, `:1808` au journal, pré-fmt ; `:1806` dans le fichier committé) |
| M-R2 (`:942`, règlement client) | R2 | `s536a_ac3_un_reglement_client_d_un_exercice_rouvert_se_contre_passe_dans_celui_ci`, `s536a_ac5_un_reglement_qui_solde_la_facture_la_rend_a_regler` (2 sur 2) |
| M-R3 (`:959`, règlement fournisseur) | R3 | `reconciliation_cancel::s536a_ac6_a_supplier_settlement_reverses_in_a_reopened_year` (1 sur 1) |
| M-R4 (`grouped_settlements.rs:1849`, groupe client) | R4 | `grouped_settlements::s536a_group_unreconciliation_reverses_in_a_reopened_year` (1 sur 1) |
| M-R5 (`grouped_settlements.rs:2038`, groupe fournisseur) | R5 | `grouped_settlements_supplier::s536a_ac7_a_supplier_group_of_an_open_earlier_year_reverses_in_that_year` et `…_of_a_reopened_year_reverses_in_that_year` (2 sur 2) |

**Non rouges sous une mutation de datation, par nature** : les tests d'AC-4 (`s536a_ac4_un_reglement_client_d_un_exercice_clos_refuse_l_annulation`, `s536a_ac7_a_supplier_group_of_a_closed_year_is_refused`, la phase « clos » du test Wise) exercent le rang 2, commun à tous les bras (`settlement_entry_cancel_blocker`) : ils refusent avant toute datation. Ils attestent le refus et l'état intact (rien d'écrit, le lien reste), non la date.

**Preuve du rouge de l'E2E (M1)** — `kesh-api-mutant-R1` (binaire construit avec M-R1, copié hors de `target/`, source restaurée puis binaire sain reconstruit : `e2e-mutant.sh`, `e2e-mutant-codes.txt`) : la spec `53-6a — une écriture d'un exercice antérieur encore ouvert` rougit (`e2e-cancel-mutant.log`) à l'assertion de la fiche de l'inverse, `toContainText('12.12.2025')` (`reconciliation-cancel.spec.ts:413`), l'inverse étant datée du jour ; verte contre le binaire sain (`e2e-cancel-1.log`, 4/4).

**Gates** :
- Ciblés, station (base `kesh_e536a`) : `t2-kesh-db-cible-1.log` et `-2.log` (kesh-db), `t4-kesh-api-cible-1.log` (18 binaires, 1440/1442, les deux attendus de `jour_suisse_nuit_e2e` changés ensuite), `-2.log`, `-3.log` ; `t4-fmt.log`, `t4-clippy.log` (`-D warnings`) ; Vitest `t5-vitest-2.log`, complet `t7-front-vitest-complet.log` (325 fichiers, 5572 tests) ; `npm run check` (0 erreur), `lint-i18n-ownership`, `npm run build` (`t7-front-*.log`) — tous dans `kesh-gate-logs/e53/s536a/`.
- Complets, idefix, à `6b71d4c21` : **test-fast vert** (7053 tests, `kesh-gate-logs/idefix/20261127-151916-…-test-fast/`) ; **front vert** (`…/20261127-182433-…-front/`) ; **gate-ci** : premier passage rouge à la **compilation** (`journal_entry_duplicate_e2e.rs:17`, `CreateJournalEntryBody` introuvable alors qu'il existe à cette tête, `journal_entries.rs:150`, et que `clippy --all-targets` et `test-fast` l'ont compilé : artefact périmé présumé côté idefix, non réparé), relancé (`…/20261127-205720-…-gate-ci/`) : rouge **plus tôt**, à la compilation de la bibliothèque `kesh-api`, qui ne trouve ni `kesh_db::repositories::duplicate_policy` (53-1a, présent sur `main`), ni `journal_entries::CancellationDating`, ni `ReconciliationView::reversal_date` — le `kesh-db` que compile `gate-ci` sur idefix n'est pas celui de la tête `6b71d4c21`, alors que `test-fast`, sur la même machine et la même tête, l'a compilé et passé. **Échec d'environnement d'idefix, décrit, non réparé : gate-ci non obtenu** ; à rejouer par l'orchestrateur (idefix réparé, ou `scripts/gate-ci.sh kesh_ci_e536a` sur la station).
- E2E complet : non lancé (à la clôture). La spec `reconciliation-cancel.spec.ts` seule a tourné pour les captures : 3/3 (`t7-captures-playwright.log`).
- **Remédiation de la revue P1** (`$GATE_LOGS/e53/s536a/remediation/`, base `kesh_e536`, verrous de la station) : `cargo fmt --check` vert ; `cargo clippy --workspace --all-targets -D warnings` vert (`rust-clippy.log`) ; nextest des sept binaires touchés (`reconciliation_cancel`, `grouped_settlements`, `grouped_settlements_supplier`, `filet_bilan_clos`, `jour_suisse_nuit_e2e`, `wise_reconciliation_e2e`, `reconciliation_link_entry_e2e`) : **227 passés, 2 ignorés** (`rust-nextest.log`) ; Vitest (25 fichiers, 636 tests : `features/reconciliation` et les deux gardes i18n, `front-vitest-2.log`), `npm run check`, `lint-i18n-ownership`, `npm run build` verts (`front-*.log`) ; E2E Playwright joué seul contre le binaire sain : `reconciliation-cancel.spec.ts` 4/4, `reconciliation-link-entry.spec.ts` 2/2, `reconciliation-link.spec.ts` (clair) 1/1 (`e2e-cancel-1.log`, `e2e-link-1.log`, `e2e-link-2.log`) ; rouge de l'E2E contre le binaire muté (`e2e-cancel-mutant.log`). **Le `test-fast --ci` de la station (`gates-station/test-fast.log`, étiqueté « tête `d8a6fd639` ») a compilé et joué l'arbre de travail de cette remédiation, non la tête étiquetée** : 7066 tests, vert (`test-fast 0`), tests neufs compris. Le `gate-ci` de la station, lancé ensuite sur le même arbre, n'avait pas rendu son verdict à la clôture de la remédiation. ⚠️ **Revue P2** : ces deux gates ne valent pour **aucun commit** (arbre en cours de modification, étiquette fausse) et ne comptent pas comme preuve ; les gates de clôture (`test-fast`, `gate-ci`, E2E complet) se rejouent sur la tête finale (AC-16).
- **Reste ouvert à la remédiation** : (1) **L10** — le montage de repli de R3 pour AC-9 est gardé, avec sa raison (la fiche l'admet ; le chemin réel est joué pour R5) ; (2) et (3) du même endroit — le test Wise rouge par `MULTI_CURRENCY_DISABLED`, AC-1 « même exercice » sans test pour R2 et R6 — **corrigés au lot de la revue P2** (P2-L1, P2-L5 : exercice suivant multidevise, deux tests neufs, rouges sur la date sous M-R1 et M-R2).
- **Gates du lot de la revue P2** (`$GATE_LOGS/e53/s536a/remediation-p2/`, base `kesh_e536`, verrous de la station, tête du commit du lot, tests seuls) : `cargo fmt --check` vert (`fmt2.log`, après un `cargo fmt` qui a remis en forme la signature du test R2) ; `cargo clippy --workspace --all-targets -- -D warnings` vert (`clippy2.log`) ; nextest `binary(wise_reconciliation_e2e) | binary(jour_suisse_nuit_e2e)` : **69 passés, 0 échec** (`nextest-final.log`, `codes-final.txt`). Mutations M-R1 et M-R2 : rouges sur la date, `git diff` vide après chacune (`codes.txt`). Gate complet, gate-ci et E2E complet : à la clôture (AC-16).

**Recette visuelle** *(reprise à la remédiation de la revue P1, L11 ; thème clair seulement, animations coupées — `animations: 'disabled'` —, base jetable de la station `kesh_e2e_e536r`, hors dépôt)* — `$GATE_LOGS/e53/s536a/captures/` :
- `53-6a-confirmation-datee.png` — confirmation datée du 15.05.2026, « dans l'exercice « Exercice CI 2020-2030 » », phrase TVA complète ;
- `53-6a-motif-periode-verrouillee.png` — le motif, la borne en phrase à part (« La période est verrouillée jusqu'au 30.06.2026 inclus. »), sans *Confirmer* ;
- `53-6a-inverse-datee.png` — fiche de l'inverse (même exercice), datée du 15.05.2026 ;
- `53-6a-exercice-anterieur-confirmation.png` et `53-6a-exercice-anterieur-inverse.png` (neuves) — le cas du défaut : écriture du 12.12.2025, exercice 2025, annulation d'un exercice 2026 ouvert ; la fiche de l'inverse, datée du 12.12.2025 ;
- `53-6a-dialogue-depuis-la-fiche-facture.png` (neuve) — le même dialogue ouvert depuis la fiche d'une facture client (règlement du 16.05.2026) ;
- `53-6a-rattachement-candidate-sous-la-borne.png` (neuve) — la modale de rattachement avec une candidate (14.05.2026) sous la borne (31.05.2026) : l'obstacle, la phrase de la borne, la case à cocher, *Rattacher* grisé ;
- `v017-3a-dialogue-debit-choisi.png` — la modale de rattachement, état « Aujourd'hui, Kesh ne voit aucun obstacle » (texte touché par la story).
Prises sur une base `with-company` (ou `with-company-no-fy` pour l'exercice antérieur) jetable, non sur la société de démonstration (`kesh-seed`) : les états se montent par l'API, la démonstration ne porte ni verrou ni exercice antérieur ouvert.

### File List

- `crates/kesh-db/src/errors.rs`
- `crates/kesh-db/src/repositories/journal_entries.rs`
- `crates/kesh-db/src/repositories/settlement_cancellation.rs`
- `crates/kesh-db/src/repositories/reconciliation_cancel.rs`
- `crates/kesh-db/src/repositories/invoice_settlements_write.rs`
- `crates/kesh-db/src/repositories/supplier_invoices.rs`
- `crates/kesh-db/src/repositories/grouped_settlements.rs`
- `crates/kesh-db/tests/{reconciliation_cancel,grouped_settlements,grouped_settlements_supplier,invoice_settlement,invoice_write_off,supplier_invoices_repository,supplier_invoices_draft,filet_bilan_clos,lettering_documents,journal_entries_modification}.rs`
- `crates/kesh-api/src/errors.rs`, `crates/kesh-api/src/routes/reconciliation.rs`, `crates/kesh-api/src/services/factures.rs`
- `crates/kesh-api/tests/{jour_suisse_nuit_e2e,reconciliation_group_e2e,bank_accounts_closed_e2e,wise_reconciliation_e2e,reconciliation_link_entry_e2e}.rs` *(les deux derniers : remédiation de la revue P1)*
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `frontend/src/lib/features/reconciliation/{CancelReconciliationDialog.svelte,CancelReconciliationDialog.test.ts,LinkSettlementModal.svelte,LinkSettlementModal.test.ts,reconciliation-cancel.ts,reconciliation-cancel.test.ts,reconciliation.api.ts,reconciliation.types.ts}`
- `frontend/src/lib/shared/{i18n-keys.test.ts,i18n-libelle-en-dur.test.ts}`
- `frontend/tests/e2e/{reconciliation-cancel,reconciliation-link,reconciliation-link-entry}.spec.ts` *(les deux derniers : remédiation de la revue P1, captures)*
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/admin-manual.tex`, `docs/api-external.md`
- `README.md`, `_bmad-output/implementation-artifacts/sprint-status.yaml` (fusion de `main`, statut)

## Change Log

- 2026-11-27 — **Fiche née du découpage de la 53-6** (décision du Project Lead, signal de recyclage de la validation P2) : reprend de la fiche d'origine la datation, les cinq chemins, les refus, la vue, l'écran, les tests, les mutations (sans M9), le manuel et le guide de l'API ; AC renumérotés (l'ancienne AC-15 devient AC-14, AC-16 → AC-15, AC-17 → AC-16, AC-18 → AC-17 ; l'ancienne AC-14, détection, passe à la 53-6b). Q1 à Q6 tranchées le 2026-11-27 (Q5 : pas d'`/api/v2`, `b` porté par la modification du contrat). Validations P1 et P2 de la fiche d'origine tiennent pour ce périmètre ; **validation de la 53-6a à lancer** (P1, Opus : prompt `53-6a-annulation-rapprochement-date-origine-validation-prompt-p1.md`).
- 2026-11-27 — **Validation P1 de la 53-6a** (Opus 5.5, passe complète, contexte frais ; rapport `kesh-gate-logs/e53/53-6a-validation-p1.md`) : **0 CRITICAL / 1 HIGH / 1 MEDIUM / 10 LOW**. Code de référence recontrôlé à `origin/main` `0e99cd2db` (v0.20.0) : aucun fichier de code de la story n'a bougé depuis `56d47b2cc` ; `admin-manual.tex:3559` → `:3556`. **H1** — les gestes groupés (R4, R5) refusent eux-mêmes tout motif de la queue (`grouped_settlements.rs:1812-1816`, `:1988-1992`) : le rang `PeriodLocked` y sort en `409` sans `details`, non en `400` du socle ; D5 (qui refuse, commentaires des trois `match`), AC-9 et M6 corrigés, doc de `SettlementCancelBlocker` (`errors.rs:672-675`) à réécrire. **M1** — AC-14 ne nommait ni les deux phrases du manuel que D1 et D4 prescrivent (réévaluation à rejouer, totaux et TVA de la période de l'écriture), ni `user-manual.tex:4012` ; ajoutés, avec `:1119` relu et le statut `409` à la table de l'API. LOW corrigés dans la fiche : tableau des écarts (errors.rs, catalogues, manuel, guide de l'API), fin de D6 perdue au découpage rétablie, alinéa de correction (53-6b) dit hors de la story, AC-13 « rouvert » indiscernable d'une fonction pure, T1 (AC-11 vert d'emblée), citation `:3216-3234` → `:3220-3247`, fichier du test T-b2, doc-comments à relire (`entry_cancel_outlook`, `GroupedCancellation`, `reconciliation-cancel.ts`), attendus d'audit des tests groupés, notes de version (409 groupé, `FISCAL_YEAR_INVALID` qui disparaît, candidates, `dating`). **Recyclage : non** (H1 est un défaut d'origine neuf, présent dès la Q2 de la fiche d'origine ; la remédiation P2 L7 en avait étendu la prémisse fausse aux trois `match`). Une passe P2 (Sonnet) est requise (H1 et M1 corrigés hors production) : prompt `53-6a-annulation-rapprochement-date-origine-validation-prompt-p2.md`.
- 2026-11-27 — **Validation P2 de la 53-6a** (Sonnet 5.5, passe complète, contexte frais ; rapport `kesh-gate-logs/e53/53-6a-validation-p2.md`) : **0 CRITICAL / 0 HIGH / 1 MEDIUM / 6 LOW**. Remède H1 de la P1 recontrôlé au code, ligne copiée : `grouped_settlements.rs:1812-1816` et `:1988-1992` refusent sans filtre, `errors.rs:6270-6273` rend `409` sans `details` (`build_response`), les trois `match` de `errors.rs` sont les seuls sites exhaustifs hors tests ; statut, code, texte et absence de `details` exacts ; D5, AC-9, M6 et notes de version cohérents. Manuel : `.tex` et PDF aplatis de `0e99cd2db` (`user-manual.pdf` v0.20.0, 11 occurrences de « datée du jour », toutes classées), aucun autre site que ceux d'AC-14 hors les deux ajoutés ci-dessous. **M1** — le rang 2 bis (lettrage de pièce figé) précède `PeriodLocked` : pour un règlement **entier** (facture et règlement sous la borne), la queue rend `LETTERING_ALL_LINES_IN_CLOSED_PERIODS`, non `PERIOD_LOCKED` (test existant `group_unreconciliation_refuses_a_lettering_frozen_by_the_period`, `groupe_regle`, borne au 30.06) ; AC-9 (R2 à R5) et M6 ne se montaient pas avec le montage ordinaire ; D5, AC-9 et M6 corrigés (état à monter, test de précédence). LOW corrigés : commentaire des `match` de `SettlementNotCancellable`/`SupplierInvoiceNotCancellable` (« non parce que `Today` ne produirait pas… » contredisait D5 ; deux raisons) ; lecture de `je.entry_date` à la queue ; tests de R3 et R5 au geste, non à la seule queue (400 / 409) ; `user-manual.tex:812-819` (§ Verrou de période) ; `api-external.md:1635` (table générale, `PERIOD_LOCKED` `400` seulement) ; tests qui perdent leur objet (`group_unreconciliation_refuses_without_an_open_year_today`, `match` exhaustifs de `invoice_settlement.rs` et `supplier_invoices_repository.rs`). **Recyclage : non** — M1 préexiste au remède (la clause R2 et R4 d'AC-9 date de la fiche d'origine) et sa sévérité est inférieure à celle de la P1 (HIGH) ; seule sa clause R4 `409` naît du remède de la P1. Une passe **P3 (Opus, complète)** est requise (M1 corrigé hors production) : prompt `53-6a-annulation-rapprochement-date-origine-validation-prompt-p3.md`.
- 2026-11-27 — **Validation P3 de la 53-6a** (Opus 5.5, passe complète, contexte frais ; rapport `kesh-gate-logs/e53/53-6a-validation-p3.md`) : **0 CRITICAL / 0 HIGH / 1 MEDIUM / 4 LOW**. Remède M1 de la P2 recontrôlé au code (`origin/main` `0e99cd2db`) : le rang 2 bis précède bien le rang `PeriodLocked` (`settlement_cancellation.rs:121-133`, puis rangs 3 et 4) ; un groupe `document` naît sur une facture soldée (`sync_document_in_tx` : somme nulle, compte lettrable, une ligne en période ouverte) et se dissout au geste (`dissolve_invoice_document_group_in_tx`, `dissolve_supplier_invoice_document_group_in_tx`) ; « c'est déjà le cas aujourd'hui » est vrai (`grouped_settlements.rs:1823`, `reconciliation_e2e.rs:7519`). **M1** — les états qui atteignent `PERIOD_LOCKED` étaient faux pour trois bras sur quatre : R3 n'a qu'un paiement, les groupes R4 et R5 soldent chaque facture (`Invariant` « garde un reste dû ») — aucun n'admet le règlement partiel que D5 et AC-9 prescrivaient —, et l'état de production commun aux quatre (facture datée après le paiement dans la tolérance de la fenêtre, un jour client, sept fournisseur, `reconciliation.rs:288`, `:493`) manquait ; D5 et AC-9 corrigés (états par bras, montage préféré par les chemins réels, SQL en repli ; précédence : deux tests existants nommés, un test neuf pour le chemin fournisseur). LOW corrigés : décomptes de la P2 faux à leur date (annotation datée : 23, 9, 7, 11 — 14 fichiers, 73 lignes) ; AC-13, précédence de la queue sous `OriginDate` en base, par le harnais `invoice_settlement.rs` `la_precedence_de_l_annulation_lecture_et_ecriture` ; texte du lettrage figé (clés inchangées, deux motifs successifs pour un règlement antérieur) et puce du manuel `user-manual.tex:4135-4141` ajoutée à AC-14. **Recyclage : oui (signal)** — M1 est de même sévérité que la P2 et naît de son correctif, sur le même sujet (états à monter pour AC-9) ; déclaré au Project Lead, à qui revient la suite (la story est déjà le correctif minimal ; le défaut est dans le texte des montages, non dans la règle). **Tendance** : P1 0/1/1/10 (Opus) → P2 0/0/1/6 (Sonnet) → P3 0/0/1/4 (Opus). Une passe **P4 ciblée (Haiku)** sur le seul correctif de la P3 est requise (M1 corrigé hors production, sévérité venue du seul correctif précédent) : prompt `53-6a-annulation-rapprochement-date-origine-validation-prompt-p4.md`.
- 2026-11-27 — **Validation P4 ciblée de la 53-6a** (Haiku 5.5, passe ciblée, contexte frais, lentille unique sur le seul commit `e866dde57` ; rapport `kesh-gate-logs/e53/53-6a-validation-p4-ciblee.md`) : **0 CRITICAL / 0 HIGH / 0 MEDIUM / 1 LOW**. Remède de la P3 recontrôlé au code (`origin/main` `0e99cd2db`) : les sites cités existent aux lignes dites (tolérances `reconciliation.rs:288`, `:493` ; contrôle au groupe `grouped_settlements.rs:653`, `:2417` ; `letterings.rs:609` ; `routes/reconciliation.rs:2754`, `:3642` ; tests `reconciliation_e2e.rs:7519`, `tests/grouped_settlements.rs:1823`, `supplier_settlement_cancel_e2e.rs:806`, `tests/invoice_settlement.rs:1213` et `:1419-1492` ; manuel `user-manual.tex:4135-4141` et `:1950-1956`) ; R3 n'a qu'une écriture de règlement (`letterings.rs:1968`) ; le décompte rejoué donne 14 fichiers, 73 lignes, chiffres identiques. **LOW** — pour R5, la fiche cite l'étape (9) de `grouped_settlements.rs` (« garde un reste dû », `:820`), qui est le groupe **client** ; l'équivalent fournisseur est le contrôle `payable_amount != invoice.total_amount` (`:2438`) et la mise à `paid` d'un seul tenant (`:2574`). L'affirmation « R5 jamais partiel » tient ; sa citation est à corriger au développement (étape 0). **Tendance** : P1 0/1/1/10 (Opus) → P2 0/0/1/6 (Sonnet) → P3 0/0/1/4 (Opus) → P4 ciblée 0/0/0/1 (Haiku). **Recyclage** : signal déclaré à la P3, non reconduit en P4. **Statut** : `ready-for-dev`, sans finding au-dessus de LOW.
- 2026-11-27 — **Développement** (`bmad-dev-story`, Opus 5.5) : reprise de `main` (`0e99cd2db`) ; socle `CancellationDating` (remède (i)), `ReversalDating::OriginDate`, règle pure `reversal_date_verdict` ; rang `PeriodLocked` de la queue sous `OriginDate` ; dé-rapprochement sans date du jour (D6) ; vue `reversalDate`, `reversalFiscalYearName` ; motif `PERIOD_LOCKED` à l'écran (neuf motifs), confirmations datées aux quatre locales, phrase TVA toujours montée ; manuel et guide de l'API ; LOW L1 de la P4 corrigé (D5). Mutations : neuf tuées, M7 survit (angle mort assumé). Gates : test-fast et front verts sur idefix, gate-ci non obtenu (environnement), E2E complet à la clôture. Statut `review`.
- 2026-11-27 — **Revue de code P1 de la 53-6a** (Opus 5.5, passe complète, contexte frais ; rapport `$GATE_LOGS/e53/s536a/53-6a-revue-p1.md`, tête `d8a6fd639`) : **0 CRITICAL / 0 HIGH / 2 MEDIUM / 13 LOW**. **Remédiation** (Opus 5.5, commits `f28faee8f`, `9a12c5b0f` et celui-ci ; `refs #1674`). **M1** (l'E2E d'AC-15 (1) ne jouait pas le cas du défaut) : scénario neuf sur un exercice antérieur encore ouvert, date de l'écriture ≠ date de la transaction, inverse ouverte par clics, exercice vérifié ; rouge prouvé contre un binaire muté. **M2** (couverture par bras, Dev Agent Record faux) : tests neufs (R5 ×3, R6, AC-3 pour R1 à R4, AC-4 pour R2, AC-1 pour R1, AC-5 facture à régler et lettrage `reversal`), « Tests par critère » réécrit par bras, une mutation de datation par bras (M-R1 à M-R5, toutes rouges). **LOW** : L1 (la borne en phrase à part, clé neuve, `sitesTotal` 3975 → 3976, manuel), L2 (phrase TVA complète), L3 (nom de l'exercice entre guillemets), L4 (tiret si la vue est sans date), L5 (angle mort #568 réécrit pour `OriginDate`, test jumeau), L6 (M7 : mutant équivalent, commentaire et fiche corrigés), L7 (portée exacte de la sonde, écrite), L8 (champs nuls d'un retrait seul testés), L9 (attendus d'AC-9 pour R1 et R2), L11 (captures : animations coupées, deux neuves, trois reprises), L12 (rang 5 pour R8), L13 (tolérance du verrou lu sans verrou, doc du rang). **L10 non corrigé, avec sa raison** (montage de repli de R3 admis par la fiche ; chemin réel joué pour R5). Reclassements : aucun. Prochaine passe : **P2, Sonnet, complète** (prompt `53-6a-annulation-rapprochement-date-origine-review-prompt-p2.md`, qui liste les commits de la remédiation). **Condition de clôture** (AC-16) inchangée : `gate-ci` de la station et E2E complet attendus avant `done`.
- 2026-11-27 — **Revue de code P2 de la 53-6a** (Sonnet 5.5, passe complète, contexte frais ; rapport `$GATE_LOGS/e53/s536a/53-6a-revue-p2.md`, tête `267974aa5`) : **0 CRITICAL / 0 HIGH / 0 MEDIUM / 5 LOW**. Tendance : P1 Opus 0/0/2/13 → P2 Sonnet 0/0/0/5. Remèdes de la P1 tous tenus sauf L10 (non corrigé, raison admise) et L11 (partiel : base jetable). LOW : test Wise R6 rouge sous M-R1 par `MULTI_CURRENCY_DISABLED` (régression attrapée, durcissement facultatif) ; gates de la station étiquetés `d8a6fd639` sans valeur de preuve (texte corrigé) ; captures hors base de démonstration ; tiret de repli du dialogue ; AC-1 sans test dédié pour R2 et R6. Rejoué : 139 tests Rust verts, PDF du manuel régénérés et lus. Reclassements : aucun. Aucune passe de plus requise ; la clôture (`done`) attend les gates de clôture sur la tête finale (AC-16).
- 2026-11-27 — **Lot des LOW de tests de la revue P2 de la 53-6a** (Opus 5.5 ; `refs #1674` ; tests seuls, aucun code de production). **P2-L1** : l'exercice « suivant » du test Wise (R6) est créé `multi_currency = TRUE` ; rejoué sous M-R1, le test rougit sur la date (`left: [(2026-11-27, 2)]`, `right: [(2020-03-10, 1)]`) et non plus sur `409 MULTI_CURRENCY_DISABLED`. **P2-L5** : deux tests « AC-1, même exercice », `wise_reconciliation_e2e::s536a_ac1_un_paiement_wise_groupe_du_meme_exercice_se_contre_passe_a_sa_date` (R6, M-R1) et `jour_suisse_nuit_e2e::s536a_ac1_un_reglement_client_du_meme_exercice_se_contre_passe_a_sa_date` (R2, M-R2), verts sains, rouges sur la date sous leur mutation (`OriginDate` → `Today(...)`, restauration par copie, `touch`, `git diff` vide contrôlé ; script `$GATE_LOGS/e53/s536a/remediation-p2/lot.sh`, journaux dans le même dossier). Gates : voir le Dev Agent Record du lot (fmt, clippy, nextest des deux binaires). Reclassements : aucun. Prochaine passe : **P3, Haiku, ciblée**, une lentille, sur le seul commit de ce lot (prompt `53-6a-annulation-rapprochement-date-origine-review-prompt-p3.md`).
- 2026-11-27 — **Revue de code P3 de la 53-6a, passe ciblée** (Haiku 5.5, contexte frais, une lentille sur le seul commit `3fdad08cf` ; rapport `$GATE_LOGS/e53/s536a/53-6a-revue-p3.md`, tête `3fdad08cf`) : **0 CRITICAL / 0 HIGH / 0 MEDIUM / 2 LOW**. Les trois tests lus contre le code et les journaux : sous M-R1 et M-R2, le rouge porte sur l'assertion de date (`wise_reconciliation_e2e.rs:1098` et `:1147` ; `jour_suisse_nuit_e2e.rs:1808` au journal), aucun `409` ; au code sain, les trois passent (`nextest-sain.log`). **LOW corrigé** : le numéro de ligne de l'assertion R2 est celui du journal pré-fmt, le fichier committé la porte à `:1806`. **LOW non corrigé** (test, non texte) : écriture et transaction ont la même date dans les deux décors (10.03.2020 et 20.12.2025), donc les tests n'établissent pas que la date vient de l'écriture plutôt que de la transaction. Tendance : P1 Opus 0/0/2/13 → P2 Sonnet 0/0/0/5 → P3 Haiku ciblée 0/0/0/2. Reclassements : aucun. Boucle de revue close : aucune passe de plus requise ; la clôture (`done`) attend les gates de clôture sur la tête finale (AC-16).
