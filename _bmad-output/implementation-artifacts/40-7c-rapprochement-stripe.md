# Story 40-7c : rapprochement Stripe — charges rattachées aux ventes des boutiques, frais et versements écrits, écarts listés

Status: drafted

*Fiche rédigée le 2026-11-20, branche `epic/40-stripe`, numéros de ligne relevés sur `origin/main` au commit `fd04099f`.
Epic 40, **phase 1 KISS** (décision de Beat du 2026-11-20). **Dépend de** : 40-7b (transactions Stripe importées,
réglages) ; **38-11c** (rapprochement PayPal : rattachement à la vente par identifiant, frais écrits, liste des écarts,
et — à confirmer — rattachement **en devise**, C-40-K3) ; Epic 37 (lignes en devise, cours `afc`). **Suivie par** :
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
| `FR-40-7c-3` | Un versement dont le total concorde s'écrit : compte de versement contre compte de transit, montant net du versement, en devise d'origine ; il se rattache à sa transaction Stripe. | AC-4 |
| `FR-40-7c-4` | La réception du versement sur le compte de versement est **proposée** (même devise, même montant, fenêtre de dates), et se rattache par le geste de l'utilisateur à la **même** écriture — comme l'autre côté d'un virement entre deux comptes. | AC-5 |
| `FR-40-7c-5` | Remboursements, `payout_reversal` et toute autre catégorie restent « à examiner ». | AC-6 |
| `FR-40-7c-6` | Écarts listés : charge sans vente, vente sans charge, montant ou devise différents, identifiant ambigu, versement dont le total ne concorde pas, versement sans réception, réglage manquant, cours manquant. | AC-7 |
| `FR-40-7c-7` | Rattachement à une écriture existante **en devise** : une seule règle (celle de v017-3a et 34-11b), généralisée. | AC-1 |
| `NFR-40-7c-1` | Rejouable sans effet de bord : seul ce qui est en attente est traité ; un second passage n'écrit rien de plus. | AC-8 |
| `NFR-40-7c-2` | *Pattern batch* : une erreur sur une transaction ne fait pas échouer le geste ; elle devient un écart. | AC-8 |

## Critères d'acceptation

1. **AC-1 — Rattachement en devise (C-40-K3).** *Si 38-11c l'a fait, 40-7c ne fait que le réutiliser et le tester sur
   Stripe ; sinon 40-7c le porte, dans les mêmes fonctions.* La règle du rattachement à une écriture déjà passée
   (`crates/kesh-db/src/repositories/reconciliation_link.rs`, en-tête `:1-60`, règle `:500-520`) est **généralisée, non
   doublée** :
   - condition 1, `transaction_can_be_linked_to_entry` (`:535-537`, aujourd'hui `!t.amount.is_zero() && t.currency ==
     "CHF"`) : admet aussi une transaction dont la devise **égale celle du compte du plan** du compte bancaire ;
   - `M` (`ENTRY_MOVEMENTS`, `:800-804`, `SUM(debit) − SUM(credit)`) : pour un compte du plan en devise, le mouvement se
     lit en devise — somme signée de `amount_currency` (`crates/kesh-db/src/entities/journal_entry.rs:176`), débit
     positif — ; en CHF, inchangé ;
   - l'autre côté d'un virement (`transfer_other_side`, `:773` ; `other_side_predicate`, `:698` ; `linkable_side`,
     `:739`) compare des montants opposés **dans la devise de chaque transaction** — les deux côtés d'un versement Stripe
     EUR → Wise EUR sont en EUR ;
   - les lecteurs de la même règle suivent sans seconde définition : `find_entry_candidates`, `link_entry_in_tx`
     (`:1156`), `find_matched_elsewhere`, `count_already_booked_for_page` (`:959`) et le miroir `canLink` de l'écran ; le
     test d'égalité de la 34-4 (`s34_4_ac8_la_lecture_par_page_egale_les_fonctions_du_dialogue`) couvre un compte en
     devise.
   Les autres gestes de la réconciliation restent en CHF (refus `RECONCILIATION_CURRENCY_MISMATCH`,
   `crates/kesh-api/src/routes/reconciliation.rs:4387-4396`, inchangé) : limite écrite au manuel par 40-7d.
2. **AC-2 — Charge → vente.** Pour chaque transaction `pending` d'un compte bancaire Stripe, `provider_type = 'charge'`,
   `amount > 0` (le brut) : identifiant = `reference` (40-7b AC-5). Écritures candidates : de la société, avec une ligne
   sur le compte du plan du compte bancaire (le transit de la devise), non contre-passées, non rapprochées, dont la
   **description** contient l'identifiant comme mot entier (`ch_…`/`py_…` : borné par début, fin ou un caractère hors
   `[A-Za-z0-9_]`). Lecture par **lot** (une requête par compte et par geste, jetons extraits en Rust, comme la lecture
   par page de la 34-4) — jamais une requête `LIKE` par transaction. Décision :
   - une seule candidate et son mouvement `M` (AC-1) **égale** le brut → rattachement par `link_entry_in_tx` (la règle
     généralisée), aucune écriture ;
   - aucune écriture ne porte l'identifiant → écart `CHARGE_WITHOUT_SALE` ;
   - l'identifiant est trouvé, mais sur une écriture sans ligne au transit de cette devise → `CURRENCY_MISMATCH`
     (`details: { journalEntryId, expected }`), ou avec un mouvement différent → `AMOUNT_MISMATCH` (`details:
     { journalEntryId, expected, found }`) ;
   - plusieurs candidates → `AMBIGUOUS_SALE` (`details: { journalEntryIds }`) ; rien n'est tranché.
   La recherche des écritures par identifiant dans le libellé est **une** fonction partagée avec PayPal (`TXN <id>`,
   38-11c) : si 38-11c l'a écrite, 40-7c l'appelle avec son motif de jeton ; sinon elle naît ici, paramétrée.
3. **AC-3 — Frais écrits.** Transactions `pending` dont `transaction_id` finit par `:FEE`, et transactions brutes de
   catégorie `fee` (frais seuls : facturation, frais de versement en autre devise…) : avec un compte de frais réglé
   (40-7b) → écriture au journal (le journal des écritures de réconciliation, celui de `post_manual`,
   `reconciliation.rs:5116`), datée de `booking_date`, libellé « Frais Stripe · <balance_transaction_id> · <details> »,
   deux lignes — compte de frais au débit et transit au crédit pour un montant négatif, l'inverse pour un frais rendu
   (montant positif) —, montant en **devise d'origine** (`amount_currency` sur la ligne d'un compte en devise, cours
   `afc` du mois retenu par la société, `find_company_rate_in_tx`,
   `crates/kesh-db/src/repositories/exchange_rates.rs:365`, par le contrôle central de 37-3a,
   `repositories/journal_entry_fx.rs`) ; créée par `journal_entries::create_in_tx`
   (`crates/kesh-db/src/repositories/journal_entries.rs:381`) et rattachée dans la même transaction de base, sous le
   verrou du compte. Le constructeur d'écriture est celui de la réconciliation manuelle,
   `build_journal_entry_for_counterparty` (`crates/kesh-reconciliation/src/manual.rs:68`), **étendu** à une ligne en
   devise (paramètre facultatif ; appelants existants inchangés) — ou celui que 38-11c a déjà étendu. Sans compte de
   frais → `FEE_ACCOUNT_MISSING` ; cours absent → `RATE_UNAVAILABLE` (le code de 37-3a) ; exercice clos ou période
   verrouillée → le code existant (`PERIOD_LOCKED`…) ; chacun un écart, la transaction reste en attente. **Aucun code
   TVA** (phase 1).
4. **AC-4 — Versement écrit.** Transaction `pending`, `provider_type = 'payout'` : (a) sans compte de versement →
   `PAYOUT_ACCOUNT_MISSING` ; (b) **contrôle du total** : la somme des autres transactions du même compte bancaire
   portant `end_to_end_id = payout_id` doit valoir `−amount` ; sinon `PAYOUT_TOTAL_MISMATCH` (`details: { payoutId,
   expected, found }`), rien d'écrit (cas d'un rapport filtré, ou de lignes manquantes) ; (c) sinon écriture datée de
   `booking_date` (la date du versement du rapport), libellé « Versement Stripe <payout_id> », compte du plan du compte
   de versement au débit et transit au crédit, montant net en devise d'origine (même devise par construction, 40-7b
   AC-2), rattachée à la transaction Stripe (**premier côté**).
5. **AC-5 — Réception du versement : proposée, jamais comptée deux fois.** Après l'AC-4, sur le compte de versement :
   transactions `pending`, même devise, montant **égal** au net, `booking_date` dans `[date du versement − 2 jours, date
   + 10 jours]` (bornes en constantes nommées). La candidate n'est **pas** rattachée d'office (règle « un appariement
   automatique propose ») : elle est rendue à l'écart `RECEIPT_TO_CONFIRM` (`details: { payoutEntryId,
   bankTransactionId, bankAccountId }`) ; l'utilisateur la confirme par le geste existant *Rattacher à une écriture
   existante* (`POST /api/v1/reconciliation/transactions/{id}/link-entry`, `lib.rs:1046`), qui l'accepte comme **autre
   côté** du virement (AC-1, condition 4 de 34-11b) ; aucune autre écriture n'est créée. Aucune candidate →
   `PAYOUT_WITHOUT_RECEIPT` (le relevé Wise ou bancaire n'est peut-être pas encore importé). Plusieurs → `RECEIPT_TO_CONFIRM`
   avec la liste. *Contrat avec l'Epic 39 (C-40-K5) : l'import Wise n'écrit aucune écriture pour « Argent reçu de
   Stripe » et laisse la transaction en attente.*
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
8. **AC-8 — Le geste.** `POST /api/v1/stripe-matching` (`{ account }`, Comptable et plus, clés `read-write`, registre
   des routes auditées) : traite, compte bancaire par compte bancaire (verrou de chacun, par identifiant croissant),
   charges, frais, versements, propositions, dans cet ordre ; une transaction de base **par transaction Stripe**
   traitée (une erreur n'annule pas les précédentes ; *Pattern batch* : `200` et écart, jamais un 4xx/5xx pour une
   transaction ; 401/403/400/500 global seulement). Réponse : `{ linkedSales, feesWritten, payoutsWritten,
   receiptsProposed, ecarts }` (`ecarts` = la liste de l'AC-7). Rejouable : un second appel n'écrit rien (test). L'import
   (40-7b) appelle ce geste après son commit si le corps porte `match=true` (défaut à l'écran, 40-7d). Trace d'audit
   par écriture créée (celle de `journal_entries::create_in_tx`) et par rattachement (`reconciliation.entry_linked`),
   acteur = utilisateur ou clé d'API.

## Tâches

- [ ] T0 — Relire 38-11c au code fusionné : rattachement en devise, recherche par identifiant, constructeur d'écriture
  en devise ; décider « réutiliser » ou « porter ici », au Change Log.
- [ ] T1 — Règle du rattachement généralisée en devise et ses lecteurs (AC-1).
- [ ] T2 — Fonctions pures de classement et de décision (kesh-reconciliation), recherche par lot (AC-2, AC-6).
- [ ] T3 — Frais et versements écrits, contrôle du total (AC-3, AC-4).
- [ ] T4 — Propositions de réception, liste des écarts, routes (AC-5, AC-7, AC-8) ; i18n des codes d'écart.

## Stratégie de tests

- **Unitaires** (pures) : décision charge → vente (unique, aucune, ambiguë, montant, devise) ; extraction des jetons
  (bornes de mot, `ch_` dans un mot plus long refusé) ; sens des lignes de frais (prélevé, rendu) ; contrôle du total.
- **Dépôt** (`#[sqlx::test]`) : rattachement en devise (EUR) accepté, CHF inchangé, autre côté d'un virement EUR ;
  égalité de la lecture par page (34-4) sur un compte en devise.
- **HTTP** (`crates/kesh-api/tests/stripe_matching_e2e.rs`), fixture de 40-7a + écritures de vente fictives créées par
  l'API (libellé « … TXN ch_TEST… », débit transit / crédit ventes, en devise) et un compte Wise EUR fictif portant une
  réception : charges rattachées, frais et versements écrits (montants, devises, CHF au cours du mois), réception
  proposée puis confirmée par `link-entry` (deux transactions sur une écriture), chaque écart de l'AC-7 obtenu au moins
  une fois, second passage sans écriture, clé `read` → 403.
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
- « Sans conversion automatique » est lu ainsi : aucune conversion d'une devise vers une autre ; la valeur CHF d'une
  ligne en devise suit la règle de l'Epic 37 (cours `afc` du mois). **Point à confirmer avec Beat** (fichier d'epic,
  Q-40-K2).
- Date de l'écriture du versement : la date du versement du rapport (`automatic_payout_effective_at`), que Stripe donne
  comme la date d'arrivée attendue en banque ; **non confirmé** au mot près.
- Remplace les anciennes 40-2 (classement des types) et 40-3 (versements et rattachement), `superseded`.

## Change Log

- 2026-11-20 — fiche rédigée (phase 1 KISS), non validée.
