# Story 40-7c : rapprochement Stripe — revue et acceptation : charges rattachées aux ventes des boutiques, frais et versements écrits, écarts listés

Status: review

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, numéros de ligne relevés sur `origin/main` au commit `fd04099f`.
Epic 40, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 40-7b (transactions Stripe importées,
paramètres, compte de versement) ; **38-11c** (revue et acceptation PayPal : rattachement en devise — conditions 1 et 3
de la règle —, `build_review`, moteur d'acceptation, proposition générique `link_entry` exécutée par `link_entry_in_tx`, écriture de
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
   introuvable (code de 40-7b AC-4), 404 `FISCAL_YEAR_NOT_FOUND`. **Rattachement en devise** : celui de 38-11c1 AC-1
   (conditions 1 et 3), **utilisé, non refait**, testé ici sur un compte Stripe EUR. **La condition 4 (autre côté d'un
   virement) n'est pas touchée par cette story** : la revue Stripe ne rattache aucune réception (AC-5) ; le prédicat
   de l'autre côté (`other_side_predicate`, `reconciliation_link.rs:698`) s'écrit en **un texte unique à deux
   branches**, **attribué à la 38-11c** par 39-11c AC-3 (relevé de la P2 de l'Epic 39) — ⚠️ **porté par 38-11c1 AC-1b**, au même texte (phase 1 de l'Epic 38 validée P3, 2026-11-20), que cette story **réutilise** sans l'écrire ; texte, pour mémoire : `(bt.bank_account_id <> ? AND
   ((bt.currency = ? AND bt.amount = -(M)) OR (bt.currency <> ? AND SIGN(bt.amount) = -SIGN(M))) AND exclusions)` —
   même devise : montant opposé exact (virement, versement Stripe) ; devise différente : signe opposé (les deux côtés
   d'une conversion) —, la devise de `t` en **paramètre commun** (devise du compte du plan, comme `t.bank_account_id`).
   ⛔ **Aucun `AND bt.currency = …` isolé** : il contredirait la branche de conversion. 40-7c ne l'écrit pas ; elle en
   teste le cas Stripe (réception Wise EUR rattachée à l'écriture d'un versement EUR ; une transaction CHF de même
   nombre refusée, puisqu'elle n'est ni de même devise ni de signe opposé) ; si, à T0, ni 38-11c1 ni 39-11c ne l'ont
   écrit (Epic 38 pas encore dans `main`), 40-7c écrit **ce même texte à deux branches**, tests des deux branches compris. Les autres gestes de la réconciliation restent en CHF (refus `RECONCILIATION_CURRENCY_MISMATCH`,
   `crates/kesh-api/src/routes/reconciliation.rs:4387-4396`, inchangé) : limite écrite au manuel par 40-7d.
2. **AC-2 — Charge → vente : une proposition `link_entry`.** Classement (fonction pure neuve,
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
   **`link_entry { bankTransactionId, journalEntryId }`** ; aucune → écart `receipt_without_sale` ; une, mais sur un autre
   compte (devise différente), de mouvement différent ou déjà rattachée → écart `amount_or_currency_mismatch` ;
   plusieurs → « à examiner » `ambiguous_sale` ; écriture de vente non rattachée dont aucun jeton n'est l'identifiant
   d'une transaction (pending ou rapprochée) de ces comptes bancaires → écart `sale_without_receipt` — **les codes de
   38-11c**, aucun code jumeau (l'écran les libelle « charge sans vente », « vente sans charge » pour Stripe, 40-7d).
   ⚠️ **Rien n'est rattaché d'office** (`CLAUDE.md`, « Un appariement automatique propose, il ne crée jamais » ;
   décision de l'orchestrateur du 2026-11-20, validation P2) : le rattachement n'a lieu qu'à l'acceptation (AC-6),
   comme le `link_entry` de 38-11c2 ; « Tout cocher » puis « Accepter la sélection » (40-7d) en fait un geste.
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
   - (c) **contrôle de cohérence** : la somme des transactions **autres que celle du versement** (bruts et `:FEE` ; la
     transaction du versement porte elle aussi `end_to_end_id = payout_id`, 40-7b AC-5 — la compter ferait toujours
     valoir la somme zéro, donc toujours un écart) ne vaut pas `−amount` → écart
     `payout_total_mismatch` (`details: { payoutId, expected, found }`), pas de proposition — le cas d'un versement
     importé en deux fichiers incomplets (total fixé au premier, lignes ajoutées par le second) ; ⚠️ il **ne voit pas**
     un rapport filtré importé d'un seul fichier (le total vient des mêmes lignes) : celui-là se voit à la réception
     (AC-5) ;
   - (d) sinon proposition **`payout { bankTransactionId }`** — la proposition de la **revue** porte en plus, pour
     l'affichage (40-7d), `payoutBankAccountId` et le libellé du compte de versement, que le corps de l'acceptation ne
     reprend pas. À l'acceptation : le compte de versement se lit depuis les paramètres du compte Stripe **avant les
     verrous** avec les comptes des propositions, et **son compte bancaire entre dans l'ensemble verrouillé**
     (`with_account_locks`, identifiants croissants) ; **relu sous verrou** (le client ne le désigne pas ; changé
     depuis la lecture ou la revue, ou archivé → `failed[]` `PROVIDER_PROPOSAL_STALE` (38-11c2) ;
     clos → `BankAccountClosed` en `failed[]`, par `ensure_open_in_tx`, comme `link_entry_in_tx`) ; contrôles de la
     contrepartie : ceux de la proposition `fee` **adaptés** — compte du plan du compte de versement actif, imputable,
     de la devise du transit, de type actif (le contrôle « compte de charge » du compte de frais ne s'y applique pas),
     relevé à T0 ; écriture datée de `booking_date` (la
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
   l'exercice rapproché dont l'écriture n'est pointée par **aucune** transaction d'un autre compte bancaire que Stripe
   (la réception, quel que soit le compte de versement réglé depuis) — relevé pas encore
   importé, réception pas encore confirmée, **ou** rapport filtré dont le montant n'est pas le bon. *Contrat avec
   l'Epic 39 (C-40-K5)* : l'import Wise n'écrit rien pour « Argent reçu de Stripe » et laisse la transaction en
   attente ; aucune des deux revues n'écrit pour la réception.
   « À examiner » `refund` et `other_category` (`details: { category }`) pour les transactions `Other` ; leur part de
   frais suit l'AC-3. L'utilisateur passe l'écriture à la main (en devise, 37-3b) puis la rattache (en devise, AC-1).
6. **AC-6 — Acceptation.** `POST /api/v1/stripe/accept`, corps `{ account, proposals: [ {type, …} ] }` (`link_entry`,
   `fee`, `payout`), par **le moteur d'acceptation de 38-11c AC-7** : comptes bancaires lus avant les verrous,
   `with_account_locks` (38-11b), une transaction de base, un point de sauvegarde par proposition, chaque proposition
   **rejugée** sous verrou (transaction encore pending, même genre, même écriture, mêmes paramètres ; sinon `failed[]`
   périmé), *Pattern batch* (`CLAUDE.md`) : 400 global seulement pour un corps mal formé, vide, de plus de
   `MAX_IN_CLAUSE_IDS` (`repositories/reconciliation.rs:1121`) propositions ou portant deux fois la même transaction ;
   tout le reste → `failed[]` (`FailedProposal`, `routes/reconciliation.rs:576`), HTTP 200. `link_entry` →
   `reconciliation_link::link_entry_in_tx` (`:1156`), ses refus (`LinkRefusal`) sous leurs codes. **Registre des routes
   auditées** (`crates/kesh-api/tests/audit_route_registry.rs:440`) : `("post", "stripe::accept_proposals", Traced,
   Rejouee)` (nom du module au choix de la reprise, aligné sur celui de PayPal et de Wise ; **jamais `post_accept`** : l’exemption `RETRY_WITH_AUTORISE` ne regarde que le nom de la fonction, angle mort (d) de `audit_route_registry.rs:376-378`, et une `fn post_accept` appelant `retry_with` ne ferait pas rougir le test), handler sous
   **`crate::retry::retry_app_on_deadlock`** (`crates/kesh-api/src/retry.rs:40`), **jamais** `retry_with` (réservé à
   `reconciliation::post_accept`, `RETRY_WITH_AUTORISE`, `:1303`, volet (c bis)) ; ⚠️ **piège du patron par
   proposition** (38-11c AC-7) : un interblocage levé dans une proposition n'est **jamais** rangé en `failed[]` — il
   remonte en `AppError` et l'enveloppe rejoue toute l'opération (test sur le modèle de `rejeu_interblocage_e2e.rs`) ;
   compte de `the_registry_partition_is_what_the_story_declares` (`:1082`) relevé de un, phrase et somme du manuel
   d'administration (`.tex`, `le_manuel_dit_le_compte_du_registre`, `:1194`), **recomptés** depuis la source après la
   fusion des Epics 38 et 39. Le `GET` n'écrit rien (hors du balayage du registre). Trace par proposition acceptée (la
   trace générique `provider.proposal_accepted` de 38-11c2), sous l'acteur
   (`NewAuditLogEntry::for_actor`), inscrite à `audit_acteur_garde.rs` ; traces des écritures (`journal_entries::
   create_in_tx`) et des rattachements comme en 38-11c. **L'import (40-7b) n'appelle pas la revue** : l'écran importe,
   puis recharge la revue (40-7d) ; un script d'API fait de même. Un second appel de la revue après acceptation ne
   propose plus ce qui a été accepté (transactions `reconciled`) ; une proposition acceptée deux fois → `failed[]`
   périmé, rien d'écrit (test). Annulation : le geste existant (`reconciliation_cancel`) défait un rattachement ou
   contre-passe l'écriture d'un frais ou d'un versement (genre `Entry`, comme 38-11c) **tant que la réception n'est pas
   rattachée** ; une fois rattachée, annuler le côté Stripe ne retire que son lien (34-11b) et la transaction redevient
   proposable en `payout` : l'utilisateur la rattache de nouveau à l'écriture du versement (`link-entry`) au lieu
   d'accepter la proposition, qui écrirait un second versement (limite écrite au manuel, 40-7d ; comportement à
   confirmer à T0).

## Amendement du 21.11

Écrit avant le développement, sur les décisions du responsable de l'epic du 21.11 (c, d, f), la demande de libellé
des notes de l'epic, et ce que T0 a relevé au code. Là où il contredit un AC plus haut, il prévaut.

1. **Date (décision f ; AC-3 et AC-4 disaient déjà `booking_date`).** Le moteur commun datait la revue (période,
   tri, résumé) et les écritures par `value_date`, à défaut `booking_date` (`payment_date`,
   `provider_review.rs`, et `pending_in_period`). Pour Stripe, `value_date` est `available_on` (40-7b) : les frais
   d'une vente du 30 décembre tombaient en janvier, dans l'exercice suivant. Le moteur date désormais **toute**
   transaction de prestataire par `booking_date` — le jour de la transaction dans le rapport, dans le fuseau de
   l'export (le rapport du tableau de bord est en heure de Zurich) ; sans effet pour PayPal et Wise, dont les imports
   laissent `value_date` vide (`paypal_imports.rs:309` ; `wise_imports.rs:219` sur `epic/39-wise`). Hors périmètre :
   un rapport exporté avec les colonnes `*_utc` porte des jours UTC (40-7b ne convertit pas).
2. **Libellé des écritures (notes de l'epic, format du tableur comptable).** Les écritures de frais et de versement
   prennent le **libellé de la transaction** écrit par l'import (40-7b : « Frais Stripe · <Encaissement|catégorie>
   Stripe de <brut> <DEV> -<frais>=<net> <DEV> BRUT - <txn_…> (<created>- <description>) - <po_…> », « Versement
   Stripe <po_…> <net> <DEV> »), borné aux 200 caractères du moteur en gardant la fin `) - <po_…>` (seule la
   description cède) ; libellé vide → « Stripe <compte> · <identifiant> ». Remplace les libellés de l'AC-3 et de
   l'AC-4 (celui du versement en est le début). Une charge n'a pas d'écriture (rattachement).
3. **Une proposition qu'on n'accepte pas n'est pas perdue (décision c).** Sa transaction reste `pending`, la revue
   suivante la propose encore ; toute ligne (proposée ou à examiner) se traite à la main par l'API : écriture (en
   devise, 37-3b) par `POST /journal-entries`, puis `POST /reconciliation/transactions/{id}/link-entry` (rattachement
   en devise de la 38-11c1) — testé (remboursement CHF, retour de versement USD).
4. **Contrepartie d'une règle en CHF ou dans la devise de la transaction (décision d)** : portée par le moteur
   (38-11c2) ; sans objet pour Stripe, qui ne propose aucune règle (AC-5 : toute autre catégorie à examiner).
5. **Proposition `payout` au corps de l'acceptation** : `payoutBankAccountId` y est **facultatif** ; s'il est donné
   (la proposition de la revue renvoyée telle quelle) et n'est plus le compte de versement réglé relu sous les
   verrous → `PROVIDER_PROPOSAL_STALE` (`provider_settings_changed`), comme `feeAccountId` pour un frais. Le client ne
   désigne toujours pas la destination : c'est un contrôle, non un choix.
6. **Compte de versement clos ou archivé** : relu sous son verrou nommé (`bank_accounts::find_by_id_in_tx`) ; archivé
   → périmé (`payout_account_archived`), clos → `BANK_ACCOUNT_CLOSED` en `failed[]`. Non par `ensure_open_in_tx` :
   aucune ligne n'est rattachée au compte de versement (seule l'écriture touche son compte du plan) ; l'inventaire de
   la fenêtre de clôture (`bank_account_closed_inventory.rs`) ne le concerne pas. À la revue, un compte de versement
   archivé, clos ou sans compte du plan vaut « aucun » (`payout_account_missing`) ; son type et sa devise se
   contrôlent à l'acceptation (`CounterpartyRule::AssetInCurrency`).
7. **Moteur commun (T0)** : il était déjà paramétré par le prestataire (38-11c2 : `ProviderKind::Specific`,
   `Review<P>`, `AcceptBody<P>`, `EngineProposal`, `extra_lock_accounts`, `CounterpartyRule::AssetInCurrency`) ; il
   gagne `AcceptContext::locked_accounts` (les comptes verrouillés du lot, que la proposition `payout` exige du compte
   de versement) et la borne commune `ENTRY_DESCRIPTION_MAX_CHARS`. `other_side_predicate` est le texte à deux
   branches de la 38-11c1, inchangé.
8. **La 40-7b n'était pas dans `epic/40-stripe`** à la reprise (close sur `story/40-7b`, `a53f6a24`) : fusionnée dans
   la branche de cette story, raccords avec la 38-11c2 (liste des actions de trace d'import, registre des routes,
   phrase du manuel d'administration).

## Tâches

- [x] T0 — Relire 38-11c et 39-11c au code fusionné : moteur paramétré par le prestataire, lecture des écritures de
  vente, écriture `fee`, `mark_reconciled_in_tx`, code périmé, `other_side_predicate` (deux branches de 39-11c ?) ;
  relever ce qui manque et le dire au Change Log ; contrôles de contrepartie de `fee` (AC-4 d) et inventaire
  `bank_account_closed_inventory` de la fonction d'écriture du versement.
- [x] T1 — `stripe.rs` : `classify`, `stripe_ids_in`, contrôles du versement (AC-2, AC-4, AC-5) ; tests purs.
- [x] T2 — Revue : lectures (écritures de vente paramétrées, lignes d'un versement, versements sans réception), route
  `GET` (AC-1 à AC-5).
- [x] T3 — Acceptation : proposition `payout` au moteur, route `POST`, registre, phrase du manuel d'administration (AC-6) ;
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
  ni la condition 4 de la règle, ni la proposition de réception, ni de moteur propre (revue et acceptation de 38-11c2, génériques) ; il reste le classement, trois lectures, deux gestionnaires minces et la proposition
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
- 2026-11-20 — validation P3 ciblée (Claude Sonnet 5.5, contexte frais ; une lentille : les patches P2 contre
  `origin/main` et les fiches 38-11c et 39-11c à HEAD) : 0 CRITICAL, 0 HIGH, 3 MEDIUM, 3 LOW, remédiés (rapport
  `kesh-gate-logs/epic-39-40-kiss/validation/40-7-P3-ciblee.md`). MEDIUM : le contrôle du total d'un versement
  (AC-4 c) comptait la transaction du versement, qui porte elle aussi `end_to_end_id = payout_id` : somme toujours nulle,
  écart toujours levé → exclue ; le compte de versement n'était ni dans l'ensemble verrouillé ni contrôlé ouvert à
  l'écriture (clos entre la revue et l'acceptation) → lu avant les verrous, verrouillé, relu, `ensure_open_in_tx` ; la
  proposition `payout` ne portait pas le compte de versement que 40-7d affiche. LOW : `other_side_predicate` dit « porté
  par 38-11c » alors que la fiche 38-11c ne le reprend pas (repli 39-11c, puis 40-7c) ; `payout_without_receipt` lié au
  compte de versement *actuel* (faux écart après un changement de réglage) → toute transaction d'un autre compte
  bancaire que Stripe ; annulation du côté Stripe d'un versement dont la réception est rattachée (ne retire que le lien,
  second versement proposé) → limite écrite. Contrôles de contrepartie de `fee` à relever à T0 pour le compte de
  versement (actif, non de charge).
- 2026-11-20 — validation P4 ciblée (Claude Haiku 5.5, contexte frais ; une lentille : le seul commit `bb40b420`, contre `origin/main` et les fiches sœurs) : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 0 LOW, remédiés (rapport `kesh-gate-logs/epic-39-40-kiss/validation/40-7-P4-ciblee.md`). MEDIUM : l'exemption `RETRY_WITH_AUTORISE` désigne le handler par son seul nom (angle mort (d) de `audit_route_registry.rs:376-378`) ; le handler `stripe::post_accept` est renommé `stripe::accept_proposals` dans le registre (AC-6), avec la consigne de ne jamais nommer un handler de cette fiche `post_accept`. Les autres points de la P3 ciblée sont relus sans écart.
- 2026-11-20 — alignement sur la phase 1 de l'Epic 38 (Claude Sonnet 5.5, contexte frais ; fiches 38-11c1, 38-11c2, 38-11d et rapport de validation P3 de `28623871` lus) : `link_sale` → `link_entry` (AC-2, AC-6 et renvois) ; AC-1 : le prédicat de l'autre côté est porté par 38-11c1 AC-1b au même texte, la mention « 38-11c ne le reprend pas » est retirée, repli conservé seulement si l'Epic 38 arrive après ; rattachement en devise = 38-11c1 AC-1 ; code périmé `PROVIDER_PROPOSAL_STALE`, trace `provider.proposal_accepted` ; moteur de 38-11c2 générique. Rapport `kesh-gate-logs/epic-39-40-kiss/validation/alignement-38-P3.md`.
- 2026-11-21 — amendement du 21.11 (section ci-dessus), écrit avant le développement : date `booking_date` (décision
  f), libellé repris de la transaction, décision (c), `payoutBankAccountId` facultatif au corps, compte de versement
  clos ou archivé, moteur commun (`locked_accounts`, borne commune du libellé), fusion de la 40-7b.
- 2026-11-21 — développement (`bmad-dev-story`), statut `review` ; voir le Dev Agent Record.

## Dev Agent Record

### Agent Model Used

Claude Opus 5.5 (contexte unique, worktree `kesh-wt40c`, branche `story/40-7c` partie de `epic/40-stripe` `d897c4c8`).

### T0 — relevé au code (ligne lue citée)

- **La 40-7b n'était pas dans `epic/40-stripe`** (`git merge-base --is-ancestor a53f6a24 d897c4c8` → faux), contrairement
  à la prémisse de la reprise : fusionnée dans la branche de la story (`c9409181`). Conflits résolus : `sprint-status.yaml`
  (les deux statuts `done`), `provider_imports.rs` (`AUDIT_ACTIONS` en tranche, la 38-11c2, plus `stripe_import.created`,
  la 40-7b), `audit_route_registry.rs` (160 → 161 routes, 152 → 153 tracées, 164 → 165 au rejeu, registre + routes de
  test 164 → 165), manuel d'administration (« 153 des 161 », « 153 + 4 + 4 = 161 »). `migrations.sha384`, audit
  d'idempotence (97 migrations, `yes` 13) et squash fusionnés sans conflit. fmt + clippy du tout verts après la fusion
  (`clippy-merge.log`).
- **Moteur de la 38-11c2 déjà paramétré par le prestataire** (`provider_review.rs:91-169` : `ProviderKind::Specific`,
  `ProviderSpec::linkable/ruleable` ; `:427` `ReviewProposal<P>` ; `provider_accept.rs:486` `EngineProposal`, `:639`
  `prepare_accept(…, extra_lock_accounts)`, `:1044` `CounterpartyRule::AssetInCurrency` écrit « versement Stripe,
  40-7c AC-4 d ») : aucune fiche `40-7c0`, aucun test PayPal modifié en assertion (une ligne ajoutée à la
  construction d'`AcceptContext`, `paypal_reconciliation_e2e.rs:1897`, `locked_accounts: &[]`).
- `payment_date` (`provider_review.rs:554` avant la story) et `pending_in_period` (`repositories/provider_review.rs:99`) dataient par
  `COALESCE(value_date, booking_date)` ; l'import Stripe pose `value_date = available_on`
  (`stripe_imports.rs:227`, `value_date: row.available_on`) → amendement 1. PayPal et Wise : `value_date: None`
  (`paypal_imports.rs:309`, `epic/39-wise:crates/kesh-api/src/routes/wise_imports.rs:219`).
- `other_side_predicate` (`reconciliation_link.rs:766-779`) est le texte à deux branches de la 38-11c1 : non touché,
  testé (réception EUR rattachée, CHF du même nombre refusée).
- `bank_account_closed_inventory.rs` : la proposition `payout` n'écrit aucune ligne des sept tables hors
  `mark_reconciled_in_tx` (déjà au registre) et n'appelle pas `ensure_open_in_tx` (amendement 6) : inventaire inchangé,
  vert.

### Completion Notes

- `kesh-reconciliation/src/stripe.rs` (neuf) : `classify`, `stripe_ids_in`, `match_key` (`reference`), `entry_label` +
  `bounded_label`, `SPEC`, `payout_lines`, `judge_payout` (seule définition, revue et rejugement), `complete_review`
  (genres propres : versement, remboursement, autre catégorie ; versements sans réception), types
  `StripeReviewProposal` / `PayoutProposal` / `PayoutDestination`.
- `kesh-api/src/routes/stripe.rs` (neuf) : `review` (une transaction de lecture : revue commune puis lignes des
  versements, comptes de versement, versements sans réception), `accept_proposals` (comptes de versement lus avant les
  verrous et passés en `extra_lock_accounts`, `retry_app_on_deadlock("stripe::accept")`), `StripeProposal` (non
  étiqueté : `link_entry`, `fee`, `payout` ; autres genres → 400), `accept_payout`. Routes dans `lib.rs` ; registre
  `("post", "stripe::accept_proposals", Traced, Rejouee)`.
- `kesh-db` : `provider_review::transactions_of_groups`, `provider_review::reconciled_without_other_side`,
  `bank_accounts::find_by_ids` (lectures seules) ; `pending_in_period` sur `booking_date`.
- Moteur commun : `payment_date` = `booking_date` ; `ENTRY_DESCRIPTION_MAX_CHARS` ; `AcceptContext::locked_accounts`.
- Constantes partagées avec l'import : `stripe_imports::PAYOUT_PROVIDER_TYPE` et `CHARGE_CATEGORY` repris de
  `kesh_reconciliation::stripe` ; test d'égalité de `PROVIDER` et `FEE_SUFFIX`.
- i18n : sept clés `stripe-review-*` / `stripe-gap-*` dans les quatre locales (servies à l'écran par la 40-7d) ; de-CH
  « aufgelöst » (la garde `la_cloture_d_exercice_ne_parle_pas_comme_un_panneau` réserve « geschlossen »).
- Documentation : `docs/api-external.md` (paragraphe « Revue et acceptation Stripe ») ; manuel d'administration :
  « 154 des 162 routes », « 154 + 4 + 4 = 162 ». Le manuel utilisateur et les limites restent à la 40-7d (fiche). Aucun
  PDF régénéré (règle des branches d'epic).
- Comportement à confirmer (AC-6) : **confirmé** — l'annulation du côté Stripe d'un versement non encore reçu
  contre-passe son écriture et le rend de nouveau proposable (`annulation_d_un_versement_avant_reception`). Le cas
  « réception déjà rattachée » n'est pas testé ici (comportement de la 34-11b, inchangé).

### Tests ajoutés (recomptés, périmètre `c9409181..HEAD`)

28 : 13 `#[test]` dans `kesh-reconciliation/src/stripe.rs`, 3 dans `kesh-api/src/routes/stripe.rs`, 12
`#[sqlx::test]` dans `kesh-api/tests/stripe_reconciliation_e2e.rs` (`grep -c`). Le test de volume : revue de 600
transactions et 600 écritures de vente en **127 ms** (`volume.log`, `--nocapture`).

### Sondes par mutation (`mut.sh`, `mut2.sh` ; journaux `mut-*.log`)

Douze mutations, chacune **rouge par un échec de test** (aucune erreur de compilation, `test result: FAILED`
contrôlé) : `date-valeur` (`payment_date` rendu à `value_date`), `frais-seul`, `prefixe-seul`, `total-ignore`,
`retour-ignore`, `libelle-fixe`, `fin-perdue`, `destination-revue-ignoree`, `archive-ignore`,
`destination-non-verrouillee` (comptes de versement non passés au pré-vol), `reception-ignoree` (`NOT EXISTS` de
`reconciled_without_other_side` neutralisé), `type-regle-admis`. **Angle mort déclaré** : la période de la revue
(`pending_in_period` sur `booking_date`) n'a pas de sonde propre (l'exercice de test couvre 2020-2030).

### Gates (au dernier commit de code `4d08eb00`)

- Gate ciblé avec base (`cible-base-2.log`) : fmt, clippy workspace, `kesh-reconciliation` 87, `kesh-api --lib` 780
  (5 ignorés), `audit_label_registry` 12, `audit_route_registry` 14, `bank_account_closed_inventory` 11,
  `paypal_reconciliation_e2e` 28, `stripe_import_e2e` 10, `stripe_reconciliation_e2e` 12, `textes_coherents` 30,
  `kesh-i18n` 42 — verts.
- Backend complet `scripts/test-fast.sh --ci` (`gate-backend.log`) : **6493 exécutés, 6493 passés, 15 ignorés** ;
  `cargo publish --dry-run -p kesh-import` vert.
- Frontend (`front-*.log`) : `npm run check` 0 erreur (29 avertissements, préexistants), `lint-i18n-ownership` vert,
  Vitest **4829** (280 fichiers), `npm run build` vert. Aucun fichier frontend modifié.
- `scripts/gate-ci.sh` (copie `git archive` de `4d08eb00`, `gate-ci.log`) : lancé avant la nouvelle règle des gates
  du 22.11 ; **en cours à la remise** — au moins 97 binaires `test result: ok`, aucun `FAILED` relevé à cet instant ;
  non déclaré vert. Selon la nouvelle règle, gate-ci et E2E complet se jouent à la fin de l'epic.
- **E2E partiel (1 spec)** (`e2e.log`) : `smoke.spec.ts`, 2 passés. Retenue : (c) le smoke ; (a) aucune spec ajoutée ou
  modifiée ; (b) aucune spec ne touche les routes de la story (`grep -rli "stripe|paypal/review|paypal/accept"
  frontend/tests/e2e/` vide) ni aucun écran (aucun fichier frontend modifié). E2E complet et gate-ci à la fin de
  l'epic.

### File List

- `crates/kesh-reconciliation/src/stripe.rs` (neuf), `lib.rs`, `provider_review.rs`
- `crates/kesh-api/src/routes/stripe.rs` (neuf), `routes/mod.rs`, `lib.rs`, `routes/provider_accept.rs`,
  `routes/stripe_imports.rs`
- `crates/kesh-api/tests/stripe_reconciliation_e2e.rs` (neuf), `tests/audit_route_registry.rs`,
  `tests/paypal_reconciliation_e2e.rs`
- `crates/kesh-db/src/repositories/provider_review.rs`, `repositories/bank_accounts.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `docs/api-external.md`, `docs/manual/fr/admin-manual.tex`
- `_bmad-output/implementation-artifacts/40-7c-rapprochement-stripe.md`, `sprint-status.yaml`
- (fusion de la 40-7b, `c9409181` : ses fichiers, voir sa fiche)
