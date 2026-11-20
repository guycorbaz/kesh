# Story 40-7c : rapprochement Stripe — charges rattachées aux ventes des boutiques, frais et versements écrits, écarts listés

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, numéros de ligne relevés sur `origin/main` au commit `fd04099f`.
Epic 40, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 40-7b (transactions Stripe importées,
réglages) ; **38-11c** (rapprochement PayPal, relu : le **rattachement en devise** — conditions 1 et 3 de la règle —,
`mark_reconciled_in_tx`, l'écriture en devise de la proposition `fee`, `with_account_locks` de 38-11b ; C-40-K3 tenu
pour ces deux points) ; 39-11c (la réception côté Wise) ; Epic 37 (lignes en devise, cours mensuel retenu). **Suivie par** :
40-7d (écrans).*

## Story

En tant que **Comptable**, je veux qu'un geste rapproche les transactions Stripe importées : chaque charge **rattachée à
la vente** que la boutique a déjà passée (même identifiant, même brut, même devise), chaque frais **écrit** au compte de
frais de la devise, chaque versement **écrit** vers le compte bancaire de versement — et la réception de ce versement sur
Wise ou à la banque **proposée** au rattachement, jamais comptée deux fois —, et que tout ce qui ne concorde pas soit
**listé**, afin de tenir mes ventes en ligne sans ressaisie et de voir mes erreurs.

## Exigences

| Exigence | Texte | AC |
|---|---|---|
| `FR-40-7c-1` | Une charge se rattache à **l'écriture de vente** existante dont le libellé porte son identifiant, au compte de transit de sa devise, au même brut ; aucune écriture n'est créée pour elle. | AC-2 |
| `FR-40-7c-2` | Un frais (de charge, de remboursement, ou frais seul) s'écrit : compte de frais de la devise contre compte de transit, en devise d'origine, et se rattache à sa transaction. | AC-3 |
| `FR-40-7c-3` | Un versement ordinaire s'écrit : compte de versement contre compte de transit, montant net du versement, en devise d'origine ; il se rattache à sa transaction Stripe. Un versement qui porte un `payout_reversal`, ou de net négatif, reste « à examiner ». | AC-4 |
| `FR-40-7c-4` | La réception du versement sur le compte de versement est **proposée** (même devise, même montant, fenêtre de dates), et se rattache par le geste de l'utilisateur à la **même** écriture — comme l'autre côté d'un virement entre deux comptes de **même devise**. | AC-1, AC-5 |
| `FR-40-7c-5` | Remboursements, `payout_reversal` et toute autre catégorie restent « à examiner ». | AC-6 |
| `FR-40-7c-6` | Écarts listés : charge sans vente, vente sans charge, montant ou devise différents, identifiant ambigu, versement dont le total ne concorde pas, versement sans réception, réglage manquant, cours manquant. | AC-7 |
| `FR-40-7c-7` | Rattachement à une écriture existante **en devise** : une seule règle (celle de v017-3a et 34-11b), généralisée par 38-11c ; 40-7c y ajoute l'autre côté d'un virement **de même devise**. | AC-1 |
| `NFR-40-7c-1` | Rejouable sans effet de bord : seul ce qui est en attente est traité ; un second passage n'écrit rien de plus. | AC-8 |
| `NFR-40-7c-2` | *Pattern batch* : une erreur sur une transaction ne fait pas échouer le geste ; elle devient un écart. | AC-8 |

## Critères d'acceptation

1. **AC-1 — Rattachement en devise : ce que 38-11c fait, et le seul ajout de 40-7c.** 38-11c AC-1 généralise la
   règle de `crates/kesh-db/src/repositories/reconciliation_link.rs` (en-tête `:480-520`) : condition 1,
   `transaction_can_be_linked_to_entry` (`:535-537`, aujourd'hui `!t.amount.is_zero() && t.currency == "CHF"`) admet une
   transaction dont la devise est celle du compte du plan **actuel** de son compte bancaire ; condition 3, un mouvement
   `M` lu **en devise** (seconde constante à côté d'`ENTRY_MOVEMENTS`, `:800-804`, par `entry_sql`) ; 40-7c **ne les
   refait pas** : elle les **utilise** et les teste sur Stripe (un compte en devise). **Ajout propre à 40-7c — la
   condition 4**, l'autre côté d'un virement (`other_side_predicate` `:698`, `linkable_side` `:739`,
   `transfer_other_side` `:773`) : elle compare `bt.amount = −(M)` **sans comparer les devises** ; avec des comptes en
   devise, 100 EUR et −100 CHF s'apparieraient. Le prédicat gagne `AND bt.currency = <devise de t>` (un paramètre de
   plus, **un seul texte** pour `linkable_side` et `transfer_other_side`), de sorte que les deux côtés d'un versement
   Stripe EUR → Wise EUR sont en EUR et qu'aucune autre paire ne passe ; en CHF, comportement inchangé. Les lecteurs de la
   même règle (`find_entry_candidates`, `link_entry_in_tx` `:1156`, `find_matched_elsewhere`,
   `count_already_booked_for_page` `:959`, `entry_cancel_outlook`, le miroir `canLink` de l'écran) sont **relus au code
   fusionné** (grep de `bt.amount = -(` et de `t.currency`) : tout site qui ne passe pas par `other_side_predicate` est
   corrigé ici ; le test d'égalité de la 34-4 (`s34_4_ac8_la_lecture_par_page_egale_les_fonctions_du_dialogue`) couvre un
   compte en devise. Les autres gestes de la réconciliation restent en CHF (refus `RECONCILIATION_CURRENCY_MISMATCH`,
   `crates/kesh-api/src/routes/reconciliation.rs:4387-4396`, inchangé) : limite écrite au manuel par 40-7d.
2. **AC-2 — Charge → vente.** Pour chaque transaction `pending` d'un compte bancaire Stripe, `provider_type = 'charge'`,
   `amount > 0` (le brut) : identifiant = `reference` (40-7b AC-5). Écritures candidates : de la société, avec une ligne
   sur le compte du plan du compte bancaire (le transit de la devise), non contre-passées, non rapprochées, dont la
   **description** contient l'identifiant comme **mot entier, casse comprise** (les identifiants Stripe sont sensibles à la
   casse ; `ch_…`/`py_…` : borné par début, fin ou un caractère hors `[A-Za-z0-9_]`). Lecture par **lot** (une requête par
   compte et par geste, jetons extraits en Rust, comme la lecture par page de la 34-4) — jamais une requête `LIKE` par
   transaction. **La lecture des écritures de vente est celle de 38-11c** (T4 : écritures bornées à la période et aux
   comptes du plan des comptes bancaires du prestataire, mouvement en devise, état de rattachement), **paramétrée**
   par la fonction d'extraction des jetons (`txn_ids_in` pour PayPal, `charge_tokens_in` pour Stripe, fonctions pures) :
   une seule requête. Décision :
   - une seule candidate et son mouvement `M` (AC-1) **égale** le brut → rattachement par `link_entry_in_tx` (la règle
     généralisée), aucune écriture ;
   - aucune écriture ne porte l'identifiant → écart `CHARGE_WITHOUT_SALE` ;
   - l'identifiant est trouvé, mais sur une écriture sans ligne au transit de cette devise → `CURRENCY_MISMATCH`
     (`details: { journalEntryId, expected }`), ou avec un mouvement différent → `AMOUNT_MISMATCH` (`details:
     { journalEntryId, expected, found }`), ou déjà rapprochée d'une autre transaction → `AMOUNT_MISMATCH` aussi, avec
     `details.alreadyLinked: true` ;
   - plusieurs candidates → `AMBIGUOUS_SALE` (`details: { journalEntryIds }`) ; rien n'est tranché.
   ⚠️ **Ce rattachement se fait d'office, à la différence de 38-11c** (où `link_sale` est une proposition à accepter) :
   la règle « un appariement automatique propose, il ne crée jamais » (`CLAUDE.md`) vise le rattachement **douteux** ;
   ici l'identifiant, le brut et la devise concordent exactement avec **une** seule écriture, rien n'est créé, et
   l'annulation du rattachement existe (`reconciliation_cancel`). Tout ce qui n'est pas exactement cela devient un
   écart. Décision de la phase 1 (« l'import Stripe rattache chaque charge à la vente », Beat, 2026-11-19/20) ; si Beat
   préfère l'acceptation par l'utilisateur comme pour PayPal, c'est une variante de l'AC-8 (la liste devient une revue).
3. **AC-3 — Frais écrits.** Transactions `pending` dont `transaction_id` finit par `:FEE`, et transactions brutes de
   catégorie `fee` (frais seuls : facturation, frais de versement en autre devise…) : avec un compte de frais réglé
   (40-7b) → écriture **par la fonction qu'écrit 38-11c AC-8 pour la proposition `fee`** (T0 : son nom et son lieu ;
   elle n'est pas recopiée ; si 38-11c l'a laissée dans la route PayPal, elle passe à `kesh-reconciliation` et la route
   PayPal l'appelle) : journal Banque, par `journal_entries::create_in_tx`
   (`crates/kesh-db/src/repositories/journal_entries.rs:381`), datée de `booking_date`, libellé « Frais Stripe ·
   <balance_transaction_id> · <details> » (tronqué UTF-8 à la borne existante), deux lignes — compte de frais au débit et
   transit au crédit pour un montant négatif, l'inverse pour un frais rendu (montant positif) —, montant en **devise
   d'origine** (`amount_currency` sur la ligne d'un compte en devise ; cours **mensuel retenu par la société** pour la
   devise et le mois de la date, `find_company_rate_in_tx`, `crates/kesh-db/src/repositories/exchange_rates.rs:365`,
   `RateKind::Month`, CHF = `round(montant × cours, 2)` **identique des deux côtés** ; compte de frais en CHF : sa ligne
   porte ce CHF ; contrôle central de 37-3a, `repositories/journal_entry_fx.rs`), rattachée dans la **même** transaction
   de base par `mark_reconciled_in_tx` (38-11c), sous le verrou du compte. ⚠️ Ce n'est **pas**
   `build_journal_entry_for_counterparty` (`crates/kesh-reconciliation/src/manual.rs:68`) : il pose `tx.amount` comme
   montant CHF et `foreign: None`, et la réconciliation manuelle est CHF. Sans compte de frais → `FEE_ACCOUNT_MISSING` ;
   cours absent → `RATE_UNAVAILABLE` (le code de 37-3a) ; exercice clos ou période verrouillée → le code existant
   (`PERIOD_LOCKED`…) ; chacun un écart, la transaction reste en attente. **Aucun code TVA** (phase 1).
4. **AC-4 — Versement écrit.** Transaction `pending`, `provider_type = 'payout'` : (a) **un versement dont l'une des lignes
   est un `payout_reversal`** (autre transaction du même `end_to_end_id` de ce type), ou dont le montant est ≥ 0 (Σ net ≤ 0),
   n'est **pas écrit** : écart `TO_REVIEW` (`details: { category: "payout", reason: "reversal" | "not_negative" }`) — le
   rapport de 2024 compte deux versements qui ne portent que des retours de versement, et le sens de leur écriture n'est
   pas établi ; (b) sans compte de versement réglé, ou le compte de versement archivé depuis → `PAYOUT_ACCOUNT_MISSING` ;
   (c) **contrôle de cohérence** : la somme des autres transactions du même compte bancaire portant `end_to_end_id =
   payout_id` doit valoir `−amount` ; sinon `PAYOUT_TOTAL_MISMATCH` (`details: { payoutId, expected, found }`), rien
   d'écrit — le cas d'un versement importé en deux fichiers incomplets (le total fixé au premier fichier, des lignes
   ajoutées par le second) ; ⚠️ il **ne voit pas** un rapport filtré importé d'un seul fichier (le total vient des mêmes
   lignes) : celui-là se voit à la réception (AC-5) ; (d) sinon écriture datée de `booking_date` (la date du
   versement du rapport), libellé « Versement Stripe <payout_id> », compte du plan du compte de versement au débit et
   transit au crédit, montant net en devise d'origine (même devise par construction, 40-7b AC-2 : **aucune conversion en
   phase 1** — un compte de versement d'une autre devise est refusé aux réglages ; si une phase ultérieure l'admet, la
   règle commune des conversions s'applique : un côté CHF, une écriture ; deux devises étrangères, deux écritures au même
   CHF par le compte de conversion), construite par la **même fonction** que les frais (deux lignes de même devise, même
   cours, même CHF), rattachée à la transaction Stripe (**premier côté**) par `mark_reconciled_in_tx`.
5. **AC-5 — Réception du versement : proposée, jamais comptée deux fois.** Après l'AC-4, sur le compte de versement :
   transactions `pending`, même devise, montant **égal** au net, `booking_date` dans `[date du versement − 5 jours,
   date + 5 jours]` (la fenêtre de 39-11c AC-5, en **une** constante nommée partagée avec elle si 39-11c en a une).
   La candidate n'est **pas** rattachée d'office (règle « un appariement automatique propose ») : elle est rendue à
   l'écart `RECEIPT_TO_CONFIRM` (`details: { payoutEntryId, bankTransactionId, bankAccountId }`) ; l'utilisateur la
   confirme par le geste existant *Rattacher à une écriture existante* (`POST
   /api/v1/reconciliation/transactions/{id}/link-entry`, `lib.rs:1046`), qui l'accepte comme **autre côté** du virement
   (AC-1, condition 4 de 34-11b) ; aucune autre écriture n'est créée. Aucune candidate → `PAYOUT_WITHOUT_RECEIPT` (le
   relevé Wise ou bancaire n'est pas encore importé, **ou** le rapport était filtré et le montant n'est pas le bon).
   Plusieurs → `RECEIPT_TO_CONFIRM` avec la liste. *Contrat avec l'Epic 39 (C-40-K5) : l'import Wise n'écrit aucune
   écriture pour « Argent reçu de Stripe » et laisse la transaction en attente ; **39-11c propose, de son côté, le même
   rattachement** (`stripe_payout`, même devise, même montant, ± 5 jours) : les deux écrans mènent au même lien, le
   premier confirmé fait disparaître l'autre proposition (la transaction n'est plus en attente, l'écriture a ses deux
   côtés) — aucune des deux n'écrit.*
6. **AC-6 — À examiner.** `refund` (brut négatif), `payout_reversal`, et toute catégorie autre que `charge`, `fee`,
   `payout` → écart `TO_REVIEW` (`details: { category }`) ; leur part de frais suit l'AC-3. L'utilisateur passe
   l'écriture à la main et la rattache par le geste de l'AC-5 (en devise, AC-1).
7. **AC-7 — Liste des écarts.** Calculée à la demande, **jamais stockée** (aucune table) : `GET
   /api/v1/stripe-matching?account=<libellé>&from=&to=` (Comptable et plus) rend les écarts des transactions en attente
   des comptes Stripe du compte, plus les **ventes sans charge** : écritures de la période avec une ligne au débit d'un
   transit Stripe, un jeton `ch_…`/`py_…` au libellé, non rapprochées → `SALE_WITHOUT_CHARGE` (`details:
   { journalEntryId, token, amount, currency }`). Période par défaut : de la première à la dernière date des
   transactions Stripe du compte. Chaque écart : `{ kind, bankTransactionId | null, journalEntryId | null,
   transactionId | null, date, amount, currency, details }`, trié par date.
8. **AC-8 — Le geste.** `POST /api/v1/stripe-matching` (`{ account }`, Comptable et plus, clés `read-write`) : traite,
   compte bancaire par compte bancaire, charges, frais, versements, propositions de réception, dans cet ordre. Verrous :
   `with_account_locks` (38-11b) sur les comptes Stripe **et leurs comptes de versement**, par identifiant croissant ;
   une transaction de base **par transaction Stripe** traitée (une erreur n'annule pas les précédentes). Le geste n'est
   pas un `accept_batch` (l'appelant ne choisit rien), mais en tient l'esprit (*Pattern batch*) : `200` et un écart par
   transaction en échec, jamais un 4xx/5xx pour une transaction ; 401/403/400/500 global seulement. Réponse :
   `{ linkedSales, feesWritten, payoutsWritten, receiptsProposed, ecarts }` (`ecarts` = la liste de l'AC-7). Rejouable :
   un second appel n'écrit rien (test). **Registre des routes auditées** (`audit_route_registry.rs:440`) : `("post",
   "stripe_matching::post_stripe_matching", Traced, Rejouee)` — la route **écrit au journal** — avec l'enveloppe
   `crate::retry::retry_app_on_deadlock` (opération `stripe_matching::run`, comme `post_accept` et `post_manual`) ; le
   rejeu est sûr parce que le geste est rejouable (NFR-40-7c-1) ; volet (c) du registre vert ; le compte de
   `the_registry_partition_is_what_the_story_declares` relevé de un, la phrase et la somme du manuel d'administration
   mises à jour (`.tex`), recomptées depuis la source. Une preuve dynamique du rejeu (entrelacement forcé) si elle se
   fait à coût raisonnable, sinon **« aucune preuve dynamique »** écrit au registre comme le font ses voisines. Le `GET`
   (AC-7) n'écrit rien (hors du balayage du registre). **L'import (40-7b) n'appelle pas ce geste** : l'écran appelle
   l'import, puis le rapprochement (40-7d) ; un script d'API fait de même. Trace d'audit par écriture créée (celle de
   `journal_entries::create_in_tx`) et par rattachement (`reconciliation.entry_linked`), acteur = utilisateur ou clé
   d'API (`NewAuditLogEntry::for_actor`).

## Tâches

- [ ] T0 — Relire 38-11c au code fusionné : rattachement en devise (conditions 1 et 3), lecture des écritures de vente,
  fonction d'écriture de la proposition `fee`, `mark_reconciled_in_tx`, `with_account_locks` ; relever ce qui manque
  (condition 4 de l'AC-1) et le dire au Change Log.
- [ ] T1 — Condition 4 de la règle (même devise) et relecture de ses lecteurs (AC-1).
- [ ] T2 — Fonctions pures de classement et de décision (kesh-reconciliation), recherche par lot (AC-2, AC-6).
- [ ] T3 — Frais et versements écrits, contrôle du total (AC-3, AC-4).
- [ ] T4 — Propositions de réception, liste des écarts, routes (AC-5, AC-7, AC-8) ; i18n des codes d'écart.

## Stratégie de tests

- **Unitaires** (pures) : décision charge → vente (unique, aucune, ambiguë, montant, devise) ; extraction des jetons
  (bornes de mot, `ch_` dans un mot plus long refusé) ; sens des lignes de frais (prélevé, rendu) ; contrôle du total.
- **Dépôt** (`#[sqlx::test]`) : rattachement en devise (EUR) accepté, CHF inchangé, autre côté d'un virement EUR ;
  égalité de la lecture par page (34-4) sur un compte en devise.
- **HTTP** (`crates/kesh-api/tests/stripe_matching_e2e.rs`), fixture de 40-7a (dates de 2020, exercice 2020 ouvert, multi-devises activé, cours
  mensuels de 2020 posés par `PUT /exchange-rates/rates`) + écritures de vente fictives créées par l'API (libellé
  « … TXN ch_TEST… », débit transit / crédit ventes, en devise) et un compte `wise` EUR fictif créé par l'API dont la
  réception est insérée par le dépôt (l'import Wise n'est pas requis ici) : charges rattachées, frais et versements écrits (montants, devises, CHF au cours du mois), réception
  proposée puis confirmée par `link-entry` (deux transactions sur une écriture), chaque écart de l'AC-7 obtenu au moins
  une fois, second passage sans écriture, clé `read` → 403.
- Un cas par ligne de l'AC-4 : versement ordinaire, avec `payout_reversal`, de net positif, importé en deux fichiers
  (`PAYOUT_TOTAL_MISMATCH`), sans compte de versement, sans réception, réception de montant faux (rapport filtré :
  `PAYOUT_WITHOUT_RECEIPT`) ; autre côté en devise (EUR/CHF de même montant non apparié). Volume : 600 transactions et
  600 écritures de vente, temps relevé au Change Log (le rapport d'un exercice de 2024 en compte 336).
- Gate **complet** (le patch touche un dépôt de `kesh-db`).

## Modules touchés (4)

`kesh-db` (règle de rattachement, recherche par jeton), `kesh-reconciliation` (classement, décisions, constructeur en
devise), `kesh-api` (routes `stripe_matching`, appel depuis l'import), `kesh-i18n` (codes d'écart).

## Note de version

*Ajouté* — Le rapprochement Stripe rattache chaque charge à la vente de la boutique (même identifiant, même brut, même
devise), écrit les frais et les versements en devise d'origine, propose la réception du versement sur Wise ou à la
banque, et liste les écarts. Une transaction en devise se rattache à une écriture existante comme une transaction en CHF.

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
- **Point de coupure prévu** si la fiche dépasse la nuit : AC-4 et AC-5 (versements et réceptions) passent à une fiche
  `40-7c2` ; AC-1 à AC-3 et AC-6 à AC-8 restent, les écarts de versement s'ajoutant à la seconde.
- Date de l'écriture du versement : la date du versement du rapport (`automatic_payout_effective_at`), que Stripe donne
  comme la date d'arrivée attendue en banque ; **non confirmé** au mot près.
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
