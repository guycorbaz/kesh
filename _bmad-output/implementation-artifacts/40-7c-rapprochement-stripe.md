# Story 40-7c : rapprochement Stripe — revue et acceptation : charges rattachées aux ventes des boutiques, frais et versements écrits, écarts listés

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, numéros de ligne relevés sur `origin/main` au commit `fd04099f`.
Epic 40, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 40-7b (transactions Stripe importées,
paramètres, compte de versement) ; **38-11c** (revue et acceptation PayPal : rattachement en devise — conditions 1 et 3
de la règle —, `build_review`, moteur d'acceptation, proposition `link_sale` exécutée par `link_entry_in_tx`, écriture de
la proposition `fee`, `mark_reconciled_in_tx`, `with_account_locks` de 38-11b, enveloppe de rejeu) ; **39-11c** (le
même moteur **paramétré par le prestataire**, et la réception d'un versement Stripe proposée côté Wise, 39-11c AC-4) ;
Epic 37 (lignes en devise, cours mensuel retenu). **Suivie par** : 40-7d (écrans). Relue à la validation P2 contre les
fiches 38-11c (`kesh-wt38`, validée P2) et 39-11c (`kesh-wt39`).*

## Story

En tant que **Comptable**, je veux que Kesh **propose**, pour les transactions Stripe importées d'un exercice, le
rattachement de chaque charge à la vente que la boutique a déjà passée (même identifiant, même brut, même devise),
l'écriture de chaque frais au compte de frais et de chaque versement vers son compte bancaire de versement, que
j'accepte en lot — et que tout ce qui ne concorde pas soit **listé** —, afin de tenir mes ventes en ligne sans
ressaisie et de voir mes erreurs.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-7c-1` | Une charge est **proposée** au rattachement à **l'écriture de vente** existante dont le libellé porte son identifiant, au compte de transit de sa devise, au même brut ; l'utilisateur l'accepte (en lot possible) ; aucune écriture n'est créée pour elle. | AC-2, AC-6 |
| `FR-40-7c-2` | Un frais (de charge, de remboursement, ou frais seul) est proposé à l'écriture : compte de frais contre compte de transit, en devise d'origine ; accepté, il s'écrit et se rattache à sa transaction. | AC-3, AC-6 |
| `FR-40-7c-3` | Un versement ordinaire est proposé à l'écriture : compte du plan du compte de versement contre compte de transit, montant net, en devise d'origine ; accepté, il s'écrit et se rattache à sa transaction Stripe. Un versement qui porte un `payout_reversal`, de net non négatif, ou dont le total ne concorde pas, n'est pas proposé. | AC-4, AC-6 |
| `FR-40-7c-4` | La réception du versement (Wise ou banque) se rattache à l'écriture du versement par les gestes existants — proposée côté Wise par 39-11c, signalée « déjà comptabilisée » à la réconciliation d'un compte CHF — ; la revue Stripe ne la propose pas et liste le versement tant que sa réception manque. | AC-5 |
| `FR-40-7c-5` | Remboursements, `payout_reversal` et toute autre catégorie restent « à examiner ». | AC-4, AC-5 |
| `FR-40-7c-6` | Écarts listés : charge sans vente, vente sans charge, montant ou devise différents ; à examiner : identifiant ambigu, versement à retour ou de net non négatif, compte de frais ou de versement manquant ; écarts de versement : total qui ne concorde pas, réception manquante. | AC-2 à AC-5 |
| `NFR-40-7c-1` | Le moteur de revue et d'acceptation est **celui de 38-11c**, paramétré par le prestataire comme pour Wise (39-11c) : aucune seconde logique de lot, de verrou, de rejugement ni d'écriture. | AC-1, AC-6 |
| `NFR-40-7c-2` | Kesh propose, rien n'est écrit à la lecture ; *Pattern batch* à l'acceptation. Un second passage ne propose plus ce qui a été accepté. | AC-1, AC-6 |

## Critères d'acceptation

1. **AC-1 — Où vivent la revue et l'acceptation.** Comme Wise (39-11c AC-1), Stripe suit **le chemin de 38-11c** :
   `GET /api/v1/stripe/review?account=<libellé>&fiscalYearId=<id>` (Comptable et plus, clés d'API admises en lecture,
   lecture seule, une transaction de lecture) et `POST /api/v1/stripe/accept` (Comptable et plus, clés `read-write`),
   **gestionnaires minces** sur le moteur de 38-11c paramétré par le prestataire (classement et revue propres à Stripe,
   AC-2 à AC-5 ; acceptation et écritures communes) ; si, à la reprise, le moteur est encore propre à PayPal, c'est
   39-11c qui le généralise (G10) — 40-7c ne le généralise qu'à défaut, tests PayPal et Wise inchangés en assertions
   (T0, au Change Log). Réponses de la forme de 38-11c (`{ account, fiscalYearId, proposals, gaps, toReview, counts }` ;
   `{ accepted, failed }`), montants en chaînes décimales, champs en camelCase. La revue porte sur les transactions
   **pending** des comptes bancaires `provider = 'stripe'` du libellé, datées dans l'exercice ; refus 404 compte
   introuvable (code de 40-7b AC-4), 404 `FISCAL_YEAR_NOT_FOUND`. **Rattachement en devise** : celui de 38-11c AC-1
   (conditions 1 et 3), **utilisé, non refait**, testé ici sur un compte Stripe EUR. **La condition 4 (autre côté d'un
   virement) n'est pas touchée par cette story** : la revue Stripe ne rattache aucune réception (AC-5) ; le prédicat
   de l'autre côté (`other_side_predicate`, `reconciliation_link.rs:698`) s'écrit en **un texte unique à deux
   branches**, **porté par la 38-11c** (relevé de la P2 de l'Epic 39, 39-11c AC-3) : `(bt.bank_account_id <> ? AND
   ((bt.currency = ? AND bt.amount = -(M)) OR (bt.currency <> ? AND SIGN(bt.amount) = -SIGN(M))) AND exclusions)` —
   même devise : montant opposé exact (virement, versement Stripe) ; devise différente : signe opposé (les deux côtés
   d'une conversion) —, la devise de `t` en **paramètre commun** (devise du compte du plan, comme `t.bank_account_id`).
   ⛔ **Aucun `AND bt.currency = …` isolé** : il contredirait la branche de conversion. 40-7c ne l'écrit pas ; elle en
   teste le cas Stripe (réception Wise EUR rattachée à l'écriture d'un versement EUR ; une transaction CHF de même
   nombre refusée, puisqu'elle n'est ni de même devise ni de signe opposé) ; si, à T0, ni 38-11c ni 39-11c ne l'ont
   écrit, 40-7c écrit **ce même texte à deux branches**, tests des deux branches compris. Les autres gestes de la réconciliation restent en CHF (refus `RECONCILIATION_CURRENCY_MISMATCH`,
   `crates/kesh-api/src/routes/reconciliation.rs:4387-4396`, inchangé) : limite écrite au manuel par 40-7d.
2. **AC-2 — Charge → vente : une proposition `link_sale`.** Classement (fonction pure neuve,
   `crates/kesh-reconciliation/src/stripe.rs`, `classify(tx) -> StripeKind`, sur `provider_type`, le signe et le
   suffixe `:FEE` de `transaction_id`, constante de 38-11b) : `Fee` (suffixe `:FEE`, quel que soit le signe ; ou
   catégorie `fee`) ; `Charge` (catégorie `charge`, montant > 0) ; `Payout` (`payout`) ; `Other` (le reste :
   `refund`, `payout_reversal`, toute autre catégorie). Identifiant d'une charge = `reference` (40-7b AC-5 :
   `source_id`, à défaut `charge_id`). Écritures de vente : **la lecture de 38-11c** (AC-5 de 38-11c : écritures de
   l'exercice, non contre-passées, avec une ligne au **débit** d'un compte du plan des comptes bancaires du prestataire
   — et de tout compte bancaire qui partage ces comptes du plan —, mouvement en devise par compte, état de
   rattachement), **paramétrée** par la fonction pure d'extraction des jetons — `txn_ids_in` pour PayPal,
   `stripe_ids_in` pour Stripe : jetons `ch_…` et `py_…`, `[A-Za-z0-9_]` en mot entier (borné par début, fin ou un
   caractère hors de cet ensemble), **casse comprise** (les identifiants Stripe y sont sensibles) — et par la clé de la
   transaction (`transaction_id` pour PayPal, `reference` pour Stripe) ; une requête, la recherche en Rust, jamais un
   `LIKE` par transaction. La décision est **celle de `build_review`** (38-11c AC-5, encaissement) : exactement une
   écriture, sur le compte du plan **du même compte bancaire**, mouvement en devise = brut, non rattachée → proposition
   **`link_sale { bankTransactionId, journalEntryId }`** ; aucune → écart `receipt_without_sale` ; une, mais sur un autre
   compte (devise différente), de mouvement différent ou déjà rattachée → écart `amount_or_currency_mismatch` ;
   plusieurs → « à examiner » `ambiguous_sale` ; écriture de vente non rattachée dont aucun jeton n'est l'identifiant
   d'une transaction (pending ou rapprochée) de ces comptes bancaires → écart `sale_without_receipt` — **les codes de
   38-11c**, aucun code jumeau (l'écran les libelle « charge sans vente », « vente sans charge » pour Stripe, 40-7d).
   ⚠️ **Rien n'est rattaché d'office** (`CLAUDE.md`, « Un appariement automatique propose, il ne crée jamais » ;
   décision de l'orchestrateur du 2026-11-20, validation P2) : le rattachement n'a lieu qu'à l'acceptation (AC-6),
   comme le `link_sale` de 38-11c ; « Tout cocher » puis « Accepter la sélection » (40-7d) en fait un geste.
3. **AC-3 — Frais : une proposition `fee`.** Pour une transaction `Fee` : compte de frais réglé (40-7b) → proposition
   **`fee { bankTransactionId, feeAccountId }`** de 38-11c ; sinon « à examiner » `fee_account_missing` (code de 38-11c).
   À l'acceptation, **l'écriture de la proposition `fee` de 38-11c AC-8**, inchangée : journal Banque,
   `journal_entries::create_in_tx` (`crates/kesh-db/src/repositories/journal_entries.rs:381`), datée de `booking_date`,
   sens selon le signe (frais prélevé : débit compte de frais, crédit transit ; frais rendu : l'inverse), ligne du
   transit en devise au cours mensuel retenu (`find_company_rate_in_tx`,
   `crates/kesh-db/src/repositories/exchange_rates.rs:365`, `RateKind::Month`), contrepartie au même CHF (et, compte de
   frais de la même devise, au même montant en devise et au même cours), rattachement par `mark_reconciled_in_tx`.
   Libellé : celui de 38-11c avec le prestataire (« Frais Stripe <devise> · <balance_transaction_id> »), tronqué UTF-8
   à la borne existante. ⚠️ Ce n'est **pas** `build_journal_entry_for_counterparty`
   (`crates/kesh-reconciliation/src/manual.rs:68`, CHF seul, `foreign: None`). Cours absent, exercice clos, période
   verrouillée : `failed[]` sous les codes de 38-11c (`RATE_UNAVAILABLE`, …). **Aucun code TVA**.
4. **AC-4 — Versement : une proposition `payout`.** Pour une transaction `Payout` (fonction pure, sur les transactions
   du même compte bancaire qui portent `end_to_end_id = payout_id`, **pending ou rapprochées, quelle que soit leur
   date** — les lignes d'un versement peuvent précéder le début de l'exercice ; lecture par lot, une requête par
   compte) :
   - (a) une de ces transactions est un `payout_reversal`, ou le montant du versement est ≥ 0 (Σ net ≤ 0) → « à
     examiner » `payout_reversal` ou `payout_not_negative` (le rapport de 2024 compte deux versements qui ne portent que
     des retours de versement ; le sens de leur écriture n'est pas établi) ;
   - (b) aucun compte de versement réglé, ou le compte de versement archivé ou clos depuis → « à examiner »
     `payout_account_missing` ;
   - (c) **contrôle de cohérence** : la somme de ces transactions (bruts et `:FEE`) ne vaut pas `−amount` → écart
     `payout_total_mismatch` (`details: { payoutId, expected, found }`), pas de proposition — le cas d'un versement
     importé en deux fichiers incomplets (total fixé au premier, lignes ajoutées par le second) ; ⚠️ il **ne voit pas**
     un rapport filtré importé d'un seul fichier (le total vient des mêmes lignes) : celui-là se voit à la réception
     (AC-5) ;
   - (d) sinon proposition **`payout { bankTransactionId }`**. À l'acceptation : le compte de versement est **relu sous
     verrou** depuis les paramètres du compte Stripe (le client ne le désigne pas ; changé, archivé ou clos depuis la
     revue → `failed[]` `PROVIDER_PROPOSAL_STALE`, ou le code périmé de 38-11c), écriture datée de `booking_date` (la
     date du versement du rapport), libellé « Versement Stripe <payout_id> », compte du plan du compte de versement au
     débit, transit au crédit, montant net en devise d'origine — **même devise par construction** (40-7b AC-2), deux
     lignes au même montant en devise, au même cours, au même CHF : la forme de l'écriture `fee` de 38-11c pour une
     contrepartie de même devise, par la **même fonction d'écriture** (la proposition `withdrawal` de 38-11c reste
     propre à PayPal et CHF : elle prend une contrepartie choisie en CHF) —, rattachement de la transaction Stripe
     (**premier côté**) par `mark_reconciled_in_tx`. Aucune conversion en phase 1 (si une phase ultérieure admet un
     compte de versement d'une autre devise, la règle commune des conversions de 38-11c s'applique).
5. **AC-5 — Réception du versement : les gestes existants, jamais comptée deux fois.** La revue Stripe **ne propose
   aucune réception** (elle vit sur un autre compte bancaire, et la proposer ici dupliquerait celle de 39-11c) : côté
   Wise, 39-11c AC-4 propose `link_entry` de « Argent reçu de Stripe » vers l'écriture du versement (déjà rapprochée de
   la transaction Stripe, `find_matched_elsewhere`, ± 5 jours) ; côté banque CHF, la réconciliation existante signale la
   ligne CAMT « déjà comptabilisée » (34-4) et la rattache par `POST /api/v1/reconciliation/transactions/{id}/link-entry`
   (`lib.rs:1046`) comme **autre côté** du virement (34-11b) ; aucune autre écriture. La revue Stripe liste en écart
   **`payout_without_receipt`** (`details: { payoutId, journalEntryId, payoutBankAccountId }`) tout versement de
   l'exercice rapproché dont l'écriture ne porte **aucune** transaction du compte de versement — relevé pas encore
   importé, réception pas encore confirmée, **ou** rapport filtré dont le montant n'est pas le bon. *Contrat avec
   l'Epic 39 (C-40-K5)* : l'import Wise n'écrit rien pour « Argent reçu de Stripe » et laisse la transaction en
   attente ; aucune des deux revues n'écrit pour la réception.
   « À examiner » `refund` et `other_category` (`details: { category }`) pour les transactions `Other` ; leur part de
   frais suit l'AC-3. L'utilisateur passe l'écriture à la main (en devise, 37-3b) puis la rattache (en devise, AC-1).
6. **AC-6 — Acceptation.** `POST /api/v1/stripe/accept`, corps `{ account, proposals: [ {type, …} ] }` (`link_sale`,
   `fee`, `payout`), par **le moteur d'acceptation de 38-11c AC-7** : comptes bancaires lus avant les verrous,
   `with_account_locks` (38-11b), une transaction de base, un point de sauvegarde par proposition, chaque proposition
   **rejugée** sous verrou (transaction encore pending, même genre, même écriture, mêmes paramètres ; sinon `failed[]`
   périmé), *Pattern batch* (`CLAUDE.md`) : 400 global seulement pour un corps mal formé, vide, de plus de
   `MAX_IN_CLAUSE_IDS` (`repositories/reconciliation.rs:1121`) propositions ou portant deux fois la même transaction ;
   tout le reste → `failed[]` (`FailedProposal`, `routes/reconciliation.rs:576`), HTTP 200. `link_sale` →
   `reconciliation_link::link_entry_in_tx` (`:1156`), ses refus (`LinkRefusal`) sous leurs codes. **Registre des routes
   auditées** (`crates/kesh-api/tests/audit_route_registry.rs:440`) : `("post", "stripe::post_accept", Traced,
   Rejouee)` (nom du module au choix de la reprise, aligné sur celui de PayPal et de Wise), handler sous
   **`crate::retry::retry_app_on_deadlock`** (`crates/kesh-api/src/retry.rs:40`), **jamais** `retry_with` (réservé à
   `reconciliation::post_accept`, `RETRY_WITH_AUTORISE`, `:1303`, volet (c bis)) ; ⚠️ **piège du patron par
   proposition** (38-11c AC-7) : un interblocage levé dans une proposition n'est **jamais** rangé en `failed[]` — il
   remonte en `AppError` et l'enveloppe rejoue toute l'opération (test sur le modèle de `rejeu_interblocage_e2e.rs`) ;
   compte de `the_registry_partition_is_what_the_story_declares` (`:1082`) relevé de un, phrase et somme du manuel
   d'administration (`.tex`, `le_manuel_dit_le_compte_du_registre`, `:1194`), **recomptés** depuis la source après la
   fusion des Epics 38 et 39. Le `GET` n'écrit rien (hors du balayage du registre). Trace par proposition acceptée (la
   trace de 38-11c, généralisée par 39-11c, ou `stripe.proposal_accepted`), sous l'acteur
   (`NewAuditLogEntry::for_actor`), inscrite à `audit_acteur_garde.rs` ; traces des écritures (`journal_entries::
   create_in_tx`) et des rattachements comme en 38-11c. **L'import (40-7b) n'appelle pas la revue** : l'écran importe,
   puis recharge la revue (40-7d) ; un script d'API fait de même. Un second appel de la revue après acceptation ne
   propose plus ce qui a été accepté (transactions `reconciled`) ; une proposition acceptée deux fois → `failed[]`
   périmé, rien d'écrit (test). Annulation : le geste existant (`reconciliation_cancel`) défait un rattachement ou
   contre-passe l'écriture d'un frais ou d'un versement (genre `Entry`, comme 38-11c).

## Tâches

- [ ] T0 — Relire 38-11c et 39-11c au code fusionné : moteur paramétré par le prestataire, lecture des écritures de
  vente, écriture `fee`, `mark_reconciled_in_tx`, code périmé, `other_side_predicate` (deux branches de 39-11c ?) ;
  relever ce qui manque et le dire au Change Log.
- [ ] T1 — `stripe.rs` : `classify`, `stripe_ids_in`, contrôles du versement (AC-2, AC-4, AC-5) ; tests purs.
- [ ] T2 — Revue : lectures (écritures de vente paramétrées, lignes d'un versement, versements sans réception), route
  `GET` (AC-1 à AC-5).
- [ ] T3 — Acceptation : proposition `payout` au moteur, route `POST`, registre, phrase du manuel d'administration (AC-6) ;
  i18n des motifs et écarts propres à Stripe.

## Stratégie de tests

- **Purs** (`stripe.rs`) : chaque genre (charge, frais `:FEE`, frais seul `fee`, frais rendu, versement, remboursement,
  `payout_reversal`, catégorie inconnue) ; `stripe_ids_in` (bornes de mot, `ch_` dans un mot plus long refusé, casse) ;
  versement : ordinaire, à retour, de net positif, total faux, sans compte de versement ; revue sur des jeux construits
  à la main (vente concordante, sans vente, autre devise, montant différent, déjà rattachée, deux ventes, vente sans
  charge).
- **HTTP** (`crates/kesh-api/tests/stripe_reconciliation_e2e.rs`), fixture de 40-7a importée par 40-7b (dates de 2020,
  exercice 2020, multi-devises activé, cours mensuels de 2020 posés par `PUT /exchange-rates/rates`), écritures de vente
  fictives créées par l'API (libellé « … TXN ch_TEST… », débit transit / crédit ventes, en devise) et un compte `wise`
  EUR fictif créé par l'API dont la réception est insérée par le dépôt : revue → listes attendues (chaque écart et motif
  au moins une fois) ; **rien d'écrit à la lecture** ; acceptation du tout → charges rattachées, frais et versements
  écrits (montants, devises, CHF au cours du mois) ; versement importé en deux fichiers → `payout_total_mismatch` ;
  `payout_without_receipt` puis réception rattachée par `link-entry` → écart disparu, deux transactions sur l'écriture ;
  réception CHF du même nombre refusée sur un versement EUR (AC-1) ; vente rattachée entre la revue et l'acceptation →
  périmé ; compte de versement changé entre la revue et l'acceptation → périmé ; second passage sans proposition ;
  cours absent → `RATE_UNAVAILABLE` dans `failed[]`, le reste accepté ; interblocage rejoué ; clé `read` → 403 au `POST`.
  Volume : 600 transactions et 600 écritures de vente, temps relevé au Change Log (le rapport d'un exercice de 2024 en
  compte 336).
- Gate **complet** si un dépôt de `kesh-db` change (exception `kesh-db`), ciblé sinon entre les passes.

## Modules touchés (4)

`kesh-reconciliation` (`stripe.rs`), `kesh-api` (routes `stripe` review et accept, gestionnaires minces), `kesh-db`
(lectures paramétrées, s'il en manque une), `kesh-i18n` (motifs et écarts propres à Stripe). Plus une phrase du manuel
d'administration (`.tex`).

## Note de version

*Ajouté* — Rapprochement Stripe : Kesh propose de rattacher chaque charge à la vente de la boutique (même identifiant,
même brut, même devise) et d'écrire les frais et les versements en devise d'origine ; vous acceptez en lot ; les écarts
et les lignes à examiner sont listés, dont les versements dont la réception sur Wise ou à la banque n'est pas encore
rattachée.

## Notes de développement

- Schéma comptable (2024, notes sans donnée personnelle) : vente boutique D transit / C ventes au brut ; frais D frais /
  C transit ; versement D banque de versement / C transit au net. Le transit d'un versement se solde : brut − frais −
  net = 0. Les livres 2024 passaient deux écritures par charge (frais puis net) ; la phase 1 passe une écriture de frais
  par transaction et **une** de versement par versement (décision de Beat du 2026-11-19/20 : « le versement, net du
  payout »).
- « Sans conversion automatique » (**décidé, 2026-11-20**) : aucune conversion d'une devise vers une autre ; les montants
  restent en devise d'origine et la valeur CHF d'une ligne suit le cours mensuel retenu par la société, comme toute
  écriture en devise de l'Epic 37. Le transit d'un versement peut donc garder un petit écart en CHF (vente passée au cours
  du jour de la boutique, frais et versement au cours du mois) : la réévaluation de fin d'exercice (37-5) le traite.
- **Taille (validation P2)** : le découpage 40-7c1 / 40-7c2 écrit en P1 n'est **pas appliqué** — la fiche ne porte plus
  ni la condition 4 de la règle, ni la proposition de réception, ni de moteur propre (revue et acceptation de 38-11c,
  généralisées par 39-11c) ; il reste le classement, trois lectures, deux gestionnaires minces et la proposition
  `payout`. **Si, à T0, le moteur de 38-11c n'est pas paramétré par le prestataire** (39-11c non développée), la
  généralisation est le morceau à détacher, en fiche `40-7c0`, avant cette fiche.
- Date de l'écriture du versement : la date du versement du rapport (`automatic_payout_effective_at`), que Stripe donne
  comme la date d'arrivée attendue en banque ; **non confirmé** au mot près. Une vente et sa charge de part et d'autre du
  1er janvier paraissent en écarts (limite de 38-11c, manuel par 40-7d).
- Remplace les anciennes 40-2 (classement des types) et 40-3 (versements et rattachement), `superseded`.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
- 2026-11-20 — validation P1 (Claude Sonnet 5.5, contexte frais) : 0 CRITICAL, 3 HIGH, 6 MEDIUM, 2 LOW, remédiés
  (rapport `kesh-gate-logs/epic-39-40-kiss/validation/40-7c-rapprochement-stripe-P1.md`). HIGH : le contrôle du total d'un
  versement (AC-4) était tautologique pour un rapport filtré (total calculé sur les mêmes lignes), annoncé à tort comme
  tel par 40-7b/40-7c → dit pour ce qu'il attrape (versement en deux fichiers) ; versement qui ne porte qu'un
  `payout_reversal` (2 sur 100 en 2024) écrit avec un montant de sens incertain → `TO_REVIEW` ; route d'écriture au
  journal sans statut `Rejouee` ni enveloppe de rejeu au registre. MEDIUM : AC-1 refaisait ce que 38-11c AC-1 fait déjà,
  et manquait la devise dans l'autre côté d'un virement (`other_side_predicate` compare les montants sans les devises :
  100 EUR et −100 CHF s'apparient) ; `build_journal_entry_for_counterparty` n'écrit qu'en CHF (`foreign: None`,
  `tx.amount` posé en CHF) → fonction d'écriture de la proposition `fee` de 38-11c ; lecture des écritures de vente et
  extraction des jetons dédoublonnées avec 38-11c ; fenêtre de réception alignée sur 39-11c (± 5 jours, non −2/+10) et
  double proposition (Wise et Stripe) dite sans double compte ; `match=true` de l'import retiré (deux routes) ;
  rattachement d'office des charges, à la différence de 38-11c, justifié et signalé à Beat ; verrous des comptes de
  versement et `with_account_locks`. LOW : casse des identifiants Stripe ; point de coupure `40-7c2` écrit.
- 2026-11-20 — validation P2 (Claude Opus 5.5, contexte frais) : 0 CRITICAL, 1 HIGH, 4 MEDIUM, 1 LOW, remédiés, et la
  **décision de l'orchestrateur** appliquée (rapport `kesh-gate-logs/epic-39-40-kiss/validation/40-7c-rapprochement-stripe-P2.md`).
  Décision : une charge n'est plus rattachée d'office à la vente — c'est une proposition `link_sale` acceptée par
  l'utilisateur, en lot possible, comme en 38-11c (`CLAUDE.md`, « un appariement automatique propose, il ne crée
  jamais ») ; la justification de l'écart est retirée. Conséquence, pour la simplicité et sans doublon : le geste
  `POST /api/v1/stripe-matching` et sa liste `GET … ?from=&to=` sont remplacés par la revue et l'acceptation de 38-11c
  (`GET /api/v1/stripe/review`, `POST /api/v1/stripe/accept`, gestionnaires minces sur le moteur paramétré par le
  prestataire, comme Wise en 39-11c) ; frais et versements deviennent des propositions `fee` et `payout` ; codes
  d'écart de 38-11c repris (aucun code jumeau). HIGH : l'AC-1 ajoutait `AND bt.currency = <devise de t>` à
  `other_side_predicate`, que 39-11c AC-3 réécrit en deux branches (même devise et montant opposé, ou devise
  différente et signe opposé) : écrit à la lettre après 39-11c, il supprimait l'autre côté des conversions à une
  écriture ; la condition 4 sort de cette fiche (testée, écrite seulement à défaut, dans la forme à deux branches).
  MEDIUM : l'enveloppe de rejeu était dite « comme `post_accept` », qui seule garde `retry_with`, et le patron par
  proposition range un interblocage en `failed[]` (38-11c P2) → `retry_app_on_deadlock`, interblocage jamais en
  `failed[]` ; la proposition de réception doublait celle de 39-11c AC-4 (seconde recherche de candidate, verrous des
  comptes de versement) → retirée, écart `payout_without_receipt` ; le versement s'écrivait vers un compte en devise que
  la proposition `withdrawal` de 38-11c n'admet pas (contrepartie CHF choisie par le client) → proposition `payout`,
  compte de versement relu sous verrou depuis les paramètres, écriture à deux lignes de même devise de la forme `fee` ;
  le contrôle du total d'un versement ne bornait pas ses lignes (période, état) → pending ou rapprochées, toutes dates.
  LOW : compte de versement clos traité comme archivé. Découpage 40-7c1 / 40-7c2 non appliqué (fiche réduite, raison en
  Notes de développement).
- 2026-11-20 — complément P2 (Claude Opus 5.5), sur relevé de la P2 de l'Epic 39 (`kesh-wt39` `02ed58c6`) : AC-1 cite
  le texte unique à deux branches d'`other_side_predicate`, porté par la 38-11c, devise de `t` en paramètre commun, et
  interdit un `AND bt.currency = …` isolé ; remédié dans la fiche, rapport P2 complété.
