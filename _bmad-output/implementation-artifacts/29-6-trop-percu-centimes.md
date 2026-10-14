# Story 29.6 : La réconciliation ne propose plus une facture qu'un trop-perçu de quelques centimes fait refuser

## Status

**ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `fcd2ae75` (kickoff de
l'Epic 29), branche `story/29-6`, worktree `/home/gcorbaz/devel/kesh-29-6`. Story **ajoutée à l'Epic 29** par
l'orchestrateur le 2026-10-14 ; sortie dans la **v0.13.4**. Choix consignés au registre propre à la story,
`epic-29-choix-autonomes-29-6.md`, **C-29-6-1 à C-29-6-9** (à verser par union au registre de l'epic). 9 AC. PR :
**`closes #665`** — dans le titre ou le corps de la PR **seulement**, jamais dans un message de commit, même en prose
(B4 de la rétrospective 28). Validation P1 à lancer (`29-6-validate-prompt-p1.md`, Sonnet, lentilles R et F).

⚠️ **Dernière de la série 29-2 → 29-3 → 29-5 → 29-6** : elle touche la même requête que la 29-2
(`find_unpaid_invoices_for_window`) et des textes que la 29-3 écrit sur ce défaut même (§ *Zones partagées*). Le
développement **commence par un rebase** sur `origin/main` une fois les trois autres mergées, et **relit** tous les
numéros de ligne de cette fiche, relevés sur `fcd2ae75`.

## Story

**As a** utilisateur qui rapproche ses relevés (Comptable ou Admin),
**I want** que la réconciliation ne me propose jamais une facture dont l'acceptation sera refusée parce que le paiement
la dépasse de quelques centimes,
**so that** ce que l'écran propose soit ce qu'il accepte — comme pour les dates depuis la v0.13.3 (#548) —, et que je
sache, pour un client qui a payé un peu trop, qu'il faut passer par la fiche de la facture.

Issue : [#665](https://github.com/guycorbaz/kesh/issues/665) (P2, bug ; relevée pendant la fiche de la 29-3). Epic :
`_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-6*. Cible : **v0.13.4**.

### Les prémisses, établies au code (`fcd2ae75`) — exactes, une précision (C-29-6-1)

| Prémisse (#665) | Au code | Verdict |
|---|---|---|
| La proposition retient une facture dont le reste dû est à ± 5 centimes du montant (`AMOUNT_TOLERANCE_HUNDREDTHS = 5`) | `crates/kesh-api/src/routes/reconciliation.rs:72` ; `Decimal::new(AMOUNT_TOLERANCE_HUNDREDTHS, 2)` `:637`, passé à `find_unpaid_invoices_for_window` `:656-662` ; `HAVING amount_due BETWEEN ? - ? AND ? + ?` (`crates/kesh-db/src/repositories/reconciliation.rs:257`, binds `:267-270`) | **Exacte** |
| L'acceptation classe un paiement supérieur au reste dû en `Overpayment`, sauf s'il égale l'arrondi au centime | `classify_payment` (`crates/kesh-db/src/repositories/invoice_settlements.rs:175-184`), appelée par `accept_one_invoice` étape (c) (`routes/reconciliation.rs:1919-1944`) → `RECONCILIATION_OVERPAYMENT` | **Exacte** |
| Une transaction qui dépasse la facture de 1 à 5 centimes est proposée, puis refusée | Une facture n'est candidate que si son score total est > 0 (`kesh-reconciliation/src/matching.rs:81`) ; le score de montant est nul hors égalité au centime (`:109-119`) : la facture proposée porte donc une référence ou un nom de client reconnu, passe la garde de score (étape 7bis, `routes/reconciliation.rs:1808`) et tombe sur le trop-perçu. Le test existant `two_centimes_over_a_half_centime_invoice_is_an_overpayment` (`crates/kesh-api/tests/reconciliation_e2e.rs:5489`) en est un exemple : 10.02 sur un reste de 10.0050, candidate (reste dans `[9.97, 10.07]`) puis refusée | **Exacte** |
| « Aucun effet comptable : rien ne s'écrit » | Le refus précède toute écriture (`FailedProposal`, pattern batch) | **Exacte** |

**La précision** : la tolérance est **symétrique**, ses deux côtés ne se valent pas.

- **La transaction dépasse le reste dû** (le cas de #665) : refusée en trop-perçu, sauf d'un **demi-centime** au plus
  (`SettlesWithRounding` : le paiement égale l'arrondi au centime du reste brut). C'est le défaut.
- **Le reste dû dépasse la transaction** de 1 à 5 centimes : classée `Ordinary`, **acceptée** en règlement partiel ; la
  facture reste ouverte pour la différence. Ni refus ni écriture fausse : **pas le défaut de #665**, et inchangé
  (C-29-6-4).

Et le doc-comment du dépôt (`repositories/reconciliation.rs:214-219`) affirme que « la tolérance couvre l'écart à son
arrondi au centime (au plus 0.005) » : c'est **faux** depuis l'origine — la tolérance vaut 0,05, dix fois l'écart
d'arrondi. Réécrit par l'AC1.

### Les deux voies de #665, et leurs conséquences comptables établies au code (C-29-6-2)

**Voie 1 — la proposition écarte le trop-perçu.** Rien ne s'écrit de nouveau. La borne basse du reste dû devient la
plus petite valeur que l'acceptation admet ; la borne haute (paiement partiel) reste à 5 centimes. Le client qui a payé
3 centimes de trop n'est plus proposé ; il se traite comme aujourd'hui depuis la fiche de la facture — où le règlement
manuel le **refuse aussi** comme trop-perçu (`settle_invoice`, `invoice_settlements_write.rs:223-226`) : on enregistre
le montant dû, et l'écart reste sur le compte bancaire à justifier (remboursement au client, ou écriture manuelle).
C'est le traitement que le manuel décrit déjà pour les frais de rappel payés (`user-manual.tex:1308`).

**Voie 2 — l'acceptation accepte l'écart toléré sur le compte d'arrondi**, comme `SettlesWithRounding` : écriture à
trois lignes, `D banque (payé) / C créance (reste brut) / C compte d'arrondi (écart)` (`settlement_journal_lines`,
`invoice_settlements.rs:198-…`), créance soldée à zéro, `paid_at` posé, facture lettrée. Établi au code, elle :

1. **ne ferme pas le défaut sans compte d'arrondi** : `rounding_account_for_write` (`routes/reconciliation.rs:1946-1985`)
   refuse en `ROUNDING_ACCOUNT_NOT_CONFIGURED` dès qu'il y a un écart ; une société sans compte de différences
   d'arrondi désigné verrait toujours proposer ce qui est refusé — or **une écriture de produit d'arrondi sans compte
   d'arrondi configuré est exclue** (consigne de l'orchestrateur), et la proposition devrait alors lire les réglages ;
2. **dénature le compte d'arrondi** : il reçoit aujourd'hui des écarts d'**un demi-centime au plus** (arrondi au
   centime, `classify_payment`) et l'arrondi à 5 centimes des factures émises (`user-manual.tex:971`) ; un trop-perçu
   de 5 centimes est un paiement du client en trop — une **dette envers lui** jusqu'à remboursement ou abandon —, pas
   une différence d'arrondi. Le passer en produit est une **décision comptable** qui revient à Guy, pas à une story de
   correctif ;
3. **change deux chemins ou en crée deux règles** : `classify_payment` est partagée avec le règlement manuel
   (`settle_invoice`, `invoice_settlements_write.rs:223`). L'élargir change aussi la fiche de la facture ; ne pas
   l'élargir fait deux règles du trop-perçu, ce que #476 (Story 25-4-c3-b) avait précisément fermé ;
4. **contredit deux promesses écrites** : « le trop-perçu est refusé, jamais écrit » (`classify_payment`,
   `accept_one_invoice` étape (c), `api-external.md:551`) et « Pas de tolérance de montant. Un écart de quelques
   centimes ne s'absorbe pas » (`user-manual.tex:2330`).

**Retenu : la voie 1** — le périmètre minimal sûr. La voie 2 reste possible plus tard comme **évolution** (absorber
les petits trop-perçus, sur un compte à désigner), si Guy la demande ; elle n'est pas un correctif.

### La règle retenue (C-29-6-3)

**Une seule définition du trop-perçu, deux lecteurs.** Pour une transaction de montant `P > 0` au centime (colonne
`bank_transactions.amount DECIMAL(18,2)`, `migrations/20260504000001_bank_imports.sql:62`) et une facture de reste dû
brut `R` (quatre décimales), `classify_payment(P, R)` rend `Overpayment` **si et seulement si** `R < P − 0,005`.

*Démonstration* (à recopier au doc-comment) : `Overpayment` ⇔ `P > R` et non (`P = arrondi(R)` et `arrondi(R) ≠ R`).
L'arrondi est `MidpointAwayFromZero` au centime (`kesh_core::types::Money::round_to_centimes`,
`crates/kesh-core/src/types/money.rs:61-69`) : pour `R ≥ 0`, `arrondi(R) = P` ⇔ `R ∈ [P − 0,005 ; P + 0,005[`. Sur
`R < P`, l'égalité `arrondi(R) = P` vaut donc exactement pour `R ∈ [P − 0,005 ; P[`, et `arrondi(R) ≠ R` y est vrai
(`R` n'est pas au centime `P`). Reste `Overpayment` ⇔ `R < P − 0,005`. Pour `R < 0` (impossible ici : `R ≥ P − 0,05 >
−0,05` et `P ≥ 0,01`, mais la fonction est publique), `R < P − 0,005` aussi, et `classify_payment` rend `Overpayment` :
l'équivalence tient.

D'où :

- `invoice_settlements::least_due_without_overpayment(paid) -> Decimal` = `paid − 0,005`, **à côté de
  `classify_payment`**, son doc-comment portant la démonstration et la précondition (`paid > 0`, au centime). Une
  **constante nommée** pour le demi-centime (`HALF_CENTIME`), privée au module. ⛔ **Pas de `ROUND` en SQL** : la
  valeur est calculée en Rust et **liée** (`.bind`), la règle de l'arrondi garde une seule définition.
- La proposition filtre `HAVING amount_due BETWEEN ? AND ? + ?` — borne basse `least_due_without_overpayment(tx)`,
  borne haute `tx + tolérance` (inchangée). Le filtre reste **dans la requête**, avant `ORDER BY … LIMIT 50` : un filtre
  posé après coup, en Rust, laisserait des trop-perçus occuper des places du `LIMIT 50` et ferait disparaître une
  facture valable au-delà (C-29-6-3).
- L'acceptation **ne change pas** : `classify_payment` reste le juge, et un appel direct de l'API (couple forgé, facture
  modifiée entre la proposition et l'acceptation) rend toujours `RECONCILIATION_OVERPAYMENT`.

Le lien entre les deux lecteurs est tenu par un **test exhaustif** (AC4) : sur une grille de restes au dix-millième, de
part et d'autre de plusieurs montants, `R ≥ least_due_without_overpayment(P)` ⇔ `classify_payment(P, R) ≠ Overpayment`.
Un changement de l'une sans l'autre rougit.

Exemples, transaction de 100.00 (référence reconnue, compte d'arrondi désigné) :

| reste dû `R` | aujourd'hui | après | acceptation (inchangée) |
|---|---|---|---|
| 99.95 | proposée | **non proposée** | refusée `RECONCILIATION_OVERPAYMENT` |
| 99.99 | proposée | **non proposée** | refusée `RECONCILIATION_OVERPAYMENT` |
| 99.9949 | proposée | **non proposée** | refusée `RECONCILIATION_OVERPAYMENT` |
| 99.9950 | proposée | proposée | acceptée, écart de 0.0050 au crédit du compte d'arrondi |
| 99.9999 | proposée | proposée | acceptée, écart de 0.0001 au crédit du compte d'arrondi |
| 100.00 | proposée | proposée | acceptée, deux lignes |
| 100.0049 | proposée | proposée | acceptée, écart de 0.0049 au débit du compte d'arrondi |
| 100.01 | proposée | proposée | acceptée en partiel, reste 0.01 |
| 100.05 | proposée | proposée | acceptée en partiel, reste 0.05 |
| 100.06 | non proposée | non proposée | acceptée en partiel par un appel direct (inchangé : la proposition est plus étroite que l'acceptation du côté du paiement partiel) |

## Critères d'acceptation

**AC1 — La borne du trop-perçu a une définition, à côté de `classify_payment`.**

Dans `crates/kesh-db/src/repositories/invoice_settlements.rs`, sous `classify_payment` (`:175-184`) :

- `const HALF_CENTIME: Decimal = Decimal::from_parts(5, 0, 0, false, 3);` (ou `dec!`-équivalent en `const`, au choix du
  dev — la valeur est `0.005`) ;
- `pub fn least_due_without_overpayment(paid: Decimal) -> Decimal { paid - HALF_CENTIME }`, doc-comment : ce qu'elle
  rend, **la démonstration** de § *La règle retenue*, la précondition (`paid > 0`, au centime ; la colonne
  `DECIMAL(18,2)` la garantit ; hors précondition, l'équivalence n'est pas promise), et **qui la lit**
  (`find_unpaid_invoices_for_window`) ;
- le doc-comment de `classify_payment` reçoit une ligne : « Réciproque : [`least_due_without_overpayment`] — la
  proposition de la réconciliation ne charge que les factures que cette fonction ne classe pas en trop-perçu (#665). »

**AC2 — La proposition ne charge plus une facture que le paiement dépasse.**

`find_unpaid_invoices_for_window` (`crates/kesh-db/src/repositories/reconciliation.rs:234-276`) :

- le paramètre `amount_tolerance` est **renommé** `underpayment_tolerance` (position et type inchangés : aucun site
  d'appel ne change, inventaire (c)) ;
- `HAVING amount_due BETWEEN ? - ? AND ? + ?` devient `HAVING amount_due BETWEEN ? AND ? + ?`, liés dans l'ordre
  `invoice_settlements::least_due_without_overpayment(tx_amount)`, `tx_amount`, `underpayment_tolerance` ;
- le filtre 4 du doc-comment (`:214-219`) est réécrit : borne basse = le plus petit reste que l'acceptation n'estime pas
  trop-perçu (renvoi à `least_due_without_overpayment`, #665) ; borne haute = `tx + underpayment_tolerance`, un
  paiement **partiel** de quelques centimes, accepté en `Ordinary` ; la phrase fausse « la tolérance couvre l'écart à
  son arrondi au centime (au plus 0.005) » disparaît ; le « ⛔ Pas de `ROUND` ici » reste, et dit que la borne basse
  est calculée en Rust et liée ;
- l'en-tête du module (`:12-13`, « tolérance amount ± `amount_tolerance` ») est réécrit de même ;
- `WHERE`, `ORDER BY i.date ASC, i.id ASC`, `LIMIT 50` **inchangés**.

Dans `crates/kesh-api/src/routes/reconciliation.rs` : le doc-comment d'`AMOUNT_TOLERANCE_HUNDREDTHS` (`:69-71`, « ±
5 centimes ») dit qu'elle borne le seul **paiement partiel** (le reste dû au-dessus de la transaction) et renvoie à
`least_due_without_overpayment` pour l'autre côté. Le **nom** de la constante est conservé (cité par la fiche 29-3) ;
la variable locale `amount_tolerance` (`:637`) peut être renommée `underpayment_tolerance`. Le commentaire de
`accept_one_invoice` étape (c) (`:1919-1927`) reçoit une ligne : la proposition n'offre plus ce que ce garde refuse
(#665) ; il ne se rencontre plus que par un appel direct ou une facture modifiée entre les deux.

**AC3 — Rien d'autre ne change.**

- Contrat de `POST /api/v1/reconciliation/accept` : codes, `details`, ordre des refus — **inchangés**.
  `RECONCILIATION_OVERPAYMENT` reste émis (appel direct, course).
- `GET /api/v1/reconciliation/proposals` : forme JSON inchangée ; seules disparaissent des candidates que l'acceptation
  refusait.
- `classify_payment`, `settle_invoice`, `settlement_journal_lines`, `kesh-reconciliation` : **non modifiés** (hors
  doc-comment d'AC1).
- **Verrous et naissance de la vue (B5)** : `get_proposals` lit sur `&state.pool`, **hors transaction**
  (`routes/reconciliation.rs:620-662`) ; l'acceptation n'est pas touchée. Aucun verrou ajouté, aucune lecture déplacée
  dans une transaction. Le Dev Agent Record le vérifie (`git diff` de `accept_one_invoice` : commentaires seulement).
- Aucun écran, aucune clé i18n, aucune capture : le libellé de `RECONCILIATION_OVERPAYMENT`
  (`frontend/src/lib/features/reconciliation/failed-proposal-label.ts:244-248`) reste juste pour l'appel direct.

**AC4 — Test unitaire : les deux lecteurs disent la même chose (rouge d'abord).**

Dans `invoice_settlements.rs`, `mod tests` (`:824`), un test `least_due_without_overpayment_is_the_exact_overpayment_bound` :
pour chaque `paid` de `[0.01, 0.05, 1.00, 10.01, 99.99, 100.00, 123.45, 999999.99]`, et chaque `R` de `paid − 0.0700` à
`paid + 0.0700` **par pas de 0.0001** (1401 valeurs), asserter
`(R >= least_due_without_overpayment(paid)) == (classify_payment(paid, R) != PaymentAgainstDue::Overpayment)`, le
message nommant `paid` et `R`. Plus trois assertions écrites en dur, indépendantes de la production :
`least_due_without_overpayment(dec!(100.00)) == dec!(99.995)`, et pour `paid = 100.00` : `R = 99.9950` admis,
`R = 99.9949` trop-perçu. Le test **existe avant** la fonction (compilation rouge), puis vert.

**AC5 — Test du dépôt (`crates/kesh-db/tests/reconciliation_repository.rs`, rouge d'abord).**

Un test neuf `find_unpaid_invoices_excludes_an_invoice_the_payment_exceeds`, sur le patron de
`find_unpaid_invoices_filters_amount_within_tolerance` (`:535-582`) : une transaction de **100.00** à `D`, une facture
par reste — `99.95`, `99.99`, `99.9949`, `99.9950`, `99.9999`, `100.00`, `100.0049`, `100.05`, `100.06` (`insert_test_invoice`,
`total_amount` et `line_total` en `DECIMAL(19,4)`, `migrations/20260416000001_invoices.sql:24`, `:49` : les quatre
décimales passent) —, tolérance `dec!(0.05)`. Attendu, **écrit en dur** : les cinq factures `99.9950`, `99.9999`,
`100.00`, `100.0049`, `100.05`, dans l'ordre `(date, id)`. Rouge au sol (les trois premières y sont), vert après.

Le test existant `find_unpaid_invoices_filters_amount_within_tolerance` (facture à 100.04 candidate, 100.06 non) reste
vert **sans modification** : il n'éprouve que le côté du paiement partiel.

**AC6 — Test HTTP : ce qui est proposé, l'acceptation l'admet (`crates/kesh-api/tests/reconciliation_e2e.rs`).**

Un test neuf `proposal_never_offers_an_invoice_the_payment_exceeds`, sur le patron de
`proposal_and_acceptance_share_the_invoice_date_window` (`:2888-2958`) : `setup_company` (`:570`),
`designate_rounding_account` (`:5301`), une facture par cas (`seed_validated_invoice`, `:384`, numéro propre au cas,
même date que la transaction, en plein exercice 2026), une transaction par cas (`make_new_tx`, `:482`, référence = le
numéro de la facture, pour que le score soit > 0 hors montant exact ; `seed_bank_transactions`, `:512`). Montants : une
base propre au cas (`200.00 + 10 × i`, écarts de 10 CHF, bien au-delà de la tolérance : aucune facture n'est candidate
d'une autre transaction), la transaction à la base, le reste dû à `base + δ` :

| cas | δ (reste − paiement) | proposée | acceptée | code du refus |
|---|---|---|---|---|
| a | −0.05 | non | non | `RECONCILIATION_OVERPAYMENT` |
| b | −0.01 | non | non | `RECONCILIATION_OVERPAYMENT` |
| c | −0.0051 | non | non | `RECONCILIATION_OVERPAYMENT` |
| d | −0.0050 | **oui** | oui (3 lignes, écart au crédit de l'arrondi) | — |
| e | 0 | oui | oui | — |
| f | +0.0049 | oui | oui (3 lignes, écart au débit de l'arrondi) | — |
| g | +0.05 | oui | oui (partiel) | — |
| h | +0.06 | non | oui (partiel, appel direct) | — |

Assertions : l'ensemble proposé (helper `proposed_couples_287`, `:2799`) est **exactement** `{d, e, f, g}` ; tous les
couples sont ensuite acceptés en un lot par un **helper neuf** `accept_couples_296` (le `accept_couples_287` de la 28-7
asserte que tout refus est `RECONCILIATION_INVOICE_NOT_ELIGIBLE` et ne se réutilise pas tel quel ; ne pas le modifier,
la 29-2 l'emploie) qui rend les couples acceptés et le `errorCode` par transaction refusée ; attendu : acceptés
`{d, e, f, g, h}`, refusés `{a, b, c}` en `RECONCILIATION_OVERPAYMENT`. Et l'invariant de #665, écrit tel quel :
`proposed ⊆ accepted`. Rouge au sol (a, b, c proposés), vert après.

Restent verts **sans modification** : `a_centime_payment_settles_a_half_centime_invoice_with_a_rounding_line`
(`:5396`, 10.01 sur 10.0050 — **la borne exacte**, proposée et acceptée), `two_centimes_over_a_half_centime_invoice_is_an_overpayment`
(`:5489`, appel direct, toujours refusé), `accept_refuses_an_overpayment_without_writing_anything` (`:3680`, appel
direct), `the_counterparty_refusal_comes_before_the_reconciliation_overpayment` (`:6032`).

**AC7 — Manuel, API externe et CHANGELOG (PDF compris).**

`docs/manual/fr/user-manual.tex` (seul manuel rédigé ; `docs/manual/{de,en,it}/` ne portent qu'un `README.md`) :

- **§ *Algorithme de matching*, `:2029`** : « si \textbf{ce qui reste à payer} […] égale le montant de la transaction à
  5 centimes près » devient : si le reste dû **n'est pas inférieur** au montant de la transaction — un paiement qui le
  dépasse serait un trop-perçu, refusé — **et ne le dépasse pas de plus de 5 centimes** (un paiement un peu inférieur
  est accepté comme règlement partiel ; la facture reste ouverte pour la différence, que l'on solde depuis sa fiche,
  `\S\ref{sec:reglement-client}`) ; l'écart d'un demi-centime au plus entre un reste à quatre décimales et le paiement
  au centime est absorbé sur le compte de différences d'arrondi (formulation au choix du dev, sans chiffre plus fort
  que le code). Une phrase de plus : un client qui a payé **quelques centimes de trop** n'est pas proposé ;
  enregistrez le règlement du reste dû depuis la fiche de la facture, puis **rejetez** la transaction à la
  réconciliation (*Rejeter*), et traitez l'écart (remboursement, ou écriture manuelle) — sans quoi la transaction
  reste en attente. ⚠️ Le même paragraphe est réécrit par la 29-2 (dates) : au rebase, ne toucher que la condition de
  montant.
- **FAQ, `:2901`** : « Aucune facture validée et non payée n'a un \textbf{reste dû} égal au montant de la transaction
  \textbf{à 5 centimes près} » devient : aucune facture n'a un reste dû **égal au montant, ou supérieur de 5 centimes
  au plus** — un client qui a payé trop, même de quelques centimes, n'est pas proposé.
- **Les phrases que la 29-3 écrit sur ce défaut** (fiche 29-3, AC7, ligne `:2049` au sol : « une facture que le
  paiement dépasse de quelques centimes est proposée, mais son acceptation est refusée en trop-perçu --- enregistrez
  alors le règlement depuis la fiche de la facture (issue \#665) ») : **retirées** au rebase — elles deviennent
  fausses. Commande de l'inventaire : `grep -n '665\|trop-perçu\|dépasse' docs/manual/fr/user-manual.tex`.
- **Non modifiés, relus** : `:1308` (frais de rappel : « ne verra pas son versement proposé […] puisqu'il dépasse ce
  qui reste à payer » — désormais vrai **à tout montant**, auparavant faux pour 1 à 5 centimes de frais) ; `:2330`
  (« Pas de tolérance de montant » du lettrage, inchangé) ; `:971` (arrondi à 5 centimes des factures émises, autre
  sujet).

PDF régénéré (`make fr` dans `docs/manual/`, sous le verrou de gate) et commité ; contrôle **aplati** :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "à 5 centimes près"` ne rend plus
rien, et `grep -o "#665"` ne rend plus que ce que le texte final assume (rien, si le renvoi est retiré). Gardes
textuelles : `cargo nextest run -E 'binary(textes_coherents)'` vert.

`docs/api-external.md` § *Accepter des propositions de rapprochement* (`:551`, « un montant supérieur au reste dû est
refusé (`RECONCILIATION_OVERPAYMENT`, jamais écrit) ») : une phrase ajoutée — `GET /reconciliation/proposals` ne
propose plus une facture dont le reste dû est inférieur au montant de la transaction (hors l'écart d'arrondi au
centime) *(depuis la v0.13.4)* ; le refus ne se rencontre plus que par un couple choisi hors des propositions, ou une
facture modifiée entre la lecture et l'acceptation. Contrat inchangé.

`CHANGELOG.md` : sous `## [0.13.4] — Non publié` (créée par la 29-2 ; sinon à créer au-dessus de `## [0.13.3] —
2026-10-14`), section `### Corrigé` : « **La réconciliation ne propose plus une facture qu'un paiement dépasse de
quelques centimes ([#665](https://github.com/guycorbaz/kesh/issues/665)).** Une facture était proposée dès que son
reste dû valait le montant reçu à 5 centimes près, dans les deux sens ; mais un paiement qui la dépasse est un
trop-perçu, et son acceptation était refusée. Seuls sont désormais proposés le reste dû exact (à l'arrondi au centime
près) et un paiement inférieur de 5 centimes au plus, accepté comme règlement partiel. Un client qui a payé trop
s'enregistre depuis la fiche de la facture ; rejetez ensuite la transaction. API : mêmes codes. »

**AC8 — Mutations (chacune tuée par un test nommé au Dev Agent Record).**

| # | Mutation | Tuée par |
|---|---|---|
| M1 | `HALF_CENTIME` → `0.004` | AC4 (grille) ; AC5 (`99.9950`) ; AC6 d ; `a_centime_payment_settles_a_half_centime_invoice_with_a_rounding_line` |
| M2 | `HALF_CENTIME` → `0.006` | AC4 (grille) ; AC6 c ; AC5 (`99.9949`) |
| M3 | `HAVING` rendu à `BETWEEN ? - ? AND ? + ?` (l'ancienne borne, binds d'origine) | AC5 ; AC6 a, b, c |
| M4 | borne basse liée à `tx_amount` (sans demi-centime) | AC5 (`99.9950`, `99.9999`) ; AC6 d ; test `:5396` |
| M5 | borne haute liée à `tx_amount` (sans tolérance) | AC5 (`100.0049`, `100.05`) ; AC6 f, g ; `find_unpaid_invoices_filters_amount_within_tolerance` |
| M6 | `classify_payment` : `paid > raw_due` → `paid >= raw_due` (l'autre lecteur seul) | AC4 (grille : la réciproque cesse de tenir) — preuve que le test lie les deux |
| M7 | `least_due_without_overpayment` rend `paid - HALF_CENTIME` **arrondi au centime** (`paid` : la borne perd le demi-centime) | AC4 ; AC5 ; AC6 d |

Restauration : `git checkout -- <fichier> && touch <fichier>` (`docs/agents-orchestration.md` § 7). M4 et M7 coïncident
dans leur effet ; elles diffèrent par le site muté et se déclarent toutes deux.

**AC9 — Gates.**

`kesh-db` touché : **gate complet même en boucle de revue** (`CLAUDE.md` § *Pendant une boucle de revue*), base
`kesh_296` remise à zéro avant chaque gate complet (KF-039), `scripts/test-fast.sh` sous `flock` ; frontend (`npm run
check`, `lint-i18n-ownership`, `test:unit`, `build`) — aucun fichier frontend ne change, le gate le confirme ; E2E
complet au **dernier commit de code** (base `kesh_e2e_296`, port **3047**, recette de `docs/testing.md` à la lettre),
chaque rouge jugé fichier par fichier contre `docs/testing.md` § *Les échecs attendus*. Le Dev Agent Record ne déclare
que ce qui a tourné, chiffres recomptés.

## Tasks / Subtasks

- [ ] **T0 — Rebase** sur `origin/main` (29-2, 29-3, 29-5 mergées) ; relire les numéros de ligne ; regreper
      `#665` dans `crates/`, `docs/`, `CHANGELOG.md` (§ *Zones partagées*).
- [ ] **T1 — Tests rouges** (AC4, AC5, AC6) : constater le rouge (compilation pour AC4, assertion pour AC5 et AC6).
- [ ] **T2 — `kesh-db`** (AC1, AC2) : `HALF_CENTIME`, `least_due_without_overpayment`, `HAVING`, renommage du
      paramètre, doc-comments.
- [ ] **T3 — `kesh-api`** (AC2, AC3) : doc-comments de la constante et de l'étape (c), variable locale.
- [ ] **T4 — Manuel FR + PDF, `api-external.md`, CHANGELOG** (AC7), dont le retrait des angles morts #665 de la 29-3.
- [ ] **T5 — Propagation** : le grep de § *Propagation*, trié à la main, sortie résumée au Dev Agent Record.
- [ ] **T6 — Mutations** (AC8) et **gates** (AC9) ; Dev Agent Record.

## Dev Notes

### Inventaire (a) — les sites qui proposent ou acceptent une facture pour une transaction, au sol (`fcd2ae75`)

```
$ grep -rn "find_unpaid_invoices_for_window\|propose_matches(" crates --include=*.rs | grep -v '/tests/' | grep -v 'matching.rs'
crates/kesh-api/src/routes/reconciliation.rs:656:        let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
crates/kesh-api/src/routes/reconciliation.rs:747:            let match_proposals = propose_matches(&tx, &candidates_with_contacts);
crates/kesh-api/src/routes/reconciliation.rs:1790:    let proposals_score = propose_matches(&bank_transaction, &candidates_for_score);
crates/kesh-db/src/repositories/reconciliation.rs:234:pub async fn find_unpaid_invoices_for_window<'e, E>(
$ grep -rn "classify_payment\|PaymentAgainstDue::" crates --include=*.rs | grep -v "src/repositories/invoice_settlements.rs"
crates/kesh-api/src/routes/reconciliation.rs:1931:        match invoice_settlements::classify_payment(bank_transaction.amount, due_before) {
crates/kesh-db/src/repositories/invoice_settlements_write.rs:223:        match invoice_settlements::classify_payment(amount, due_before) {
```

**Ensemble clos** : un seul producteur de candidates factures (`find_unpaid_invoices_for_window`, appelé par
`get_proposals` seul), un seul juge à l'acceptation (`classify_payment`, partagé avec le règlement manuel, qui ne
propose rien). Le score (`kesh-reconciliation`) ne filtre que `total > 0` et ne lit pas le trop-perçu. Le frontend ne
filtre aucun montant.

### Inventaire (b) — les refus d'acceptation d'une facture proposée qui restent (sites non résolus)

| Refus | Pourquoi une facture proposée peut le rencontrer | Sort |
|---|---|---|
| `RECONCILIATION_OVERPAYMENT` | appel direct, ou facture modifiée (règlement, avoir) entre la proposition et l'acceptation | **non résolu, assumé** : course, même statut que la facture redatée de la 28-7 |
| `ROUNDING_ACCOUNT_NOT_CONFIGURED` / `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` (`role: rounding`) | reste à quatre décimales soldé au centime (cas d et f d'AC6) sans compte d'arrondi utilisable | **non résolu, hors classe** : refus de **configuration**, déjà documenté (`user-manual.tex:2056`), relevé par la 29-3 (C-29-3-7). Une facture n'a de reste à quatre décimales qu'à TVA arrondie sur des lignes à quatre décimales, et sa validation exige déjà un compte d'arrondi quand le total n'est pas rond (`user-manual.tex:971`) |
| `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` (`role: counterparty`), `BANK_ACCOUNT_NOT_CONFIGURED`, `FISCAL_YEAR_INVALID` | configuration du compte bancaire ou exercice clos | **hors classe**, documentés |
| `RECONCILIATION_INVOICE_NOT_ELIGIBLE` (dates) | fermé par la 28-7 (et la 29-2 garde une seule borne) | hors classe |
| `RECONCILIATION_SCORE_TOO_LOW` | score recalculé nul : impossible pour une facture proposée (même calcul) hors course | hors classe |

### Inventaire (c) — les tests existants dont le sens pourrait changer

```
$ grep -rln "find_unpaid_invoices_for_window\|reconciliation/proposals" crates/*/tests/*.rs frontend/tests/e2e/*.ts
```

à relever au sol et au rebase ; pour chaque fichier, chercher une facture dont le reste dû est **inférieur** au montant
de la transaction de 0,0051 à 0,05 et dont la proposition est attendue. Relevé du 2026-10-14 :

- `reconciliation_repository.rs` : tous les appels passent `dec!(0.05)` ou `dec!(0)` ; les factures sont au montant
  exact ou **au-dessus** (100.04, 100.06) — sens inchangé.
- `invoice_write_off.rs:785-808` : tolérance `dec!(0)`, transaction 2.00 sur un reste de 2.00 — borne basse 1.995,
  reste 2.00 : sens inchangé.
- `reconciliation_e2e.rs` : `:5396` (10.01 sur 10.0050, **la borne exacte**, proposée) — doit rester vert ; tout autre
  test qui attend une proposition sur un paiement **supérieur** au reste serait rouge et signerait un cas que la fiche
  ignore — à déclarer au Dev Agent Record.
- Fiches voisines : la 29-3 (AC3, AC6) monte **99.99 sur 100.00** et **972,89 sur 972,90** — paiement **inférieur**,
  toujours proposé ; la 29-2 tire des montants à 10 CHF d'écart — sens inchangé.
- Frontend E2E (`reconciliation-amount-due.spec.ts` et voisins) : montants exacts ou uniques à l'exécution ; à relire
  au rebase, à juger au gate.

### Zones partagées avec les 29-2, 29-3, 29-5 (fiches non mergées, lues en lecture seule)

| Fichier | 29-2 (#529) | 29-3 (#527) | 29-5 (#668) | 29-6 |
|---|---|---|---|---|
| `kesh-db/src/repositories/reconciliation.rs`, `find_unpaid_invoices_for_window` | `WHERE` (ancre), jointure `rmd`, `InvoiceDateWindow` | aucune | fiche non écrite au 2026-10-14 (départage, `kesh-reconciliation`) | **`HAVING` et ses binds, paramètre renommé, doc-comment du filtre 4** — lignes voisines du `WHERE` de la 29-2 : rebase à la main |
| `kesh-api/src/routes/reconciliation.rs` | commentaire de la passe 2 (`:653-655`), étape 6 d'`accept_one_invoice` | `candidate_rank`, `order_candidates`, commentaire « AC #114 » | — | doc-comment d'`AMOUNT_TOLERANCE_HUNDREDTHS` (`:69-71`), variable `:637`, commentaire de l'étape (c) — zones distinctes. ⚠️ Le doc-comment de `candidate_rank` (29-3, AC1) cite l'angle mort #665 (C-29-3-7) : **à réécrire** au rebase — une facture que le paiement dépasse n'est plus candidate |
| `kesh-db/src/repositories/invoice_settlements.rs` | — | — | — | `HALF_CENTIME`, `least_due_without_overpayment`, test |
| `tests/reconciliation_e2e.rs` | test de parité des dates (réutilise `proposed_couples_287` / `accept_couples_287`) | trois tests, `create_rule_ok` | ? | un test, `accept_couples_296` |
| `user-manual.tex` `:2029` | réécrit la condition de **dates** | — | ? | réécrit la condition de **montant** du même paragraphe |
| `user-manual.tex` `:2049` | — | ajoute la raison de la priorité **et l'angle mort #665** | — | **retire** l'angle mort #665 |
| `CHANGELOG.md` `[0.13.4]` | crée la section | ajoute | ajoute | ajoute |
| `docs/api-external.md` `:549-551` | phrase des dates | phrase `:549` | ? | phrase du trop-perçu |

La 29-5 n'a pas de fiche à la date de celle-ci : son périmètre (#668) est le départage à score égal, dans
`kesh-reconciliation` ; aucune zone de code commune attendue. À vérifier au rebase.

### Propagation — le symptôme à greper après chaque patch

La **valeur** et les formulations de la tolérance symétrique :

```sh
grep -rnE "à 5 centimes près|± ?0[.,]05|± ?5 centimes|amount_tolerance|AMOUNT_TOLERANCE|au plus 0\.005|#665|\b665\b|trop-perçu|dépasse de quelques centimes" \
  crates frontend/src docs/manual/fr/*.tex docs/api-external.md CHANGELOG.md
```

Les occurrences légitimes (le CHANGELOG des versions publiées, **qui ne se réécrit pas** ; le trop-perçu du règlement
manuel ; l'arrondi à 5 centimes des factures émises) se trient à la main.

### Ce qui doit être préservé

- Le contrat de `POST /api/v1/reconciliation/accept` et de `GET /proposals` (forme, codes, ordre des refus).
- Le côté du paiement partiel : tolérance de 5 centimes, acceptation en `Ordinary`.
- L'absence de `ROUND` en SQL ; une seule définition de l'arrondi (`amount_due_to_centime`).
- Le filtre avant `ORDER BY … LIMIT 50`.

### Exclusions écrites (périmètre minimal)

- **Absorber un petit trop-perçu** (voie 2 de #665) : non fait (C-29-6-2) ; évolution possible sur demande de Guy, pas
  d'issue ouverte d'office (ce n'est pas un défaut : le refus est juste, c'est la proposition qui ne l'était pas).
- **Le côté du paiement partiel** (reste supérieur au paiement de 1 à 5 centimes) : inchangé (C-29-6-4). Observation,
  non un défaut : le reliquat de 0,05 exactement ne se solde pas en nature « arrondi » (`write_off_invoice` exige
  moins de 5 centimes, `api-external.md:505`) mais en escompte ou frais bancaires — chemins existants.
- **Migration, index, frontend, i18n, captures** : aucun.

### Modules touchés (règle de découpage : au plus cinq)

1. `kesh-db` — `repositories/invoice_settlements.rs`, `repositories/reconciliation.rs` (+ `tests/reconciliation_repository.rs`) ;
2. `kesh-api` — `routes/reconciliation.rs` (commentaires) (+ `tests/reconciliation_e2e.rs`) ;
3. documentation — manuel FR (+ PDF), `docs/api-external.md`, `CHANGELOG.md`.

Trois : pas de découpage.

### Pièges connus du dépôt, applicables ici

- **Gate complet en boucle** (`kesh-db`), base remise à zéro avant chaque gate complet (KF-039).
- **Mutation restaurée, binaire périmé** : `git checkout -- f && touch f`.
- **`pdftotext` coupe les lignes** : aplatir avant de greper.
- **Greper la valeur** (`0.05`, `5 centimes`, `665`), pas la phrase.
- **Ne jamais réécrire l'entrée d'une passe passée** ; **recompter** les décomptes du Dev Agent Record.
- **Aucun « closes #665 » dans un message de commit** : `refs #665` partout ; le mot-clé va dans la PR (B4).
- **Données réelles** : fixtures synthétiques ; aucune ligne du relevé de Guy.

### References

- Issue #665 ; #548 (Story 28-7, une seule borne pour les dates) ; #476 (Story 25-4-c3-b, `classify_payment` à double
  borne) ; #527 (29-3, C-29-3-7) ; #529 (29-2) ; #668 (29-5) ; #401 (frais de rappel).
- `crates/kesh-db/src/repositories/invoice_settlements.rs:128-184`, `:198-…`, `:824-865` ;
  `crates/kesh-db/src/repositories/reconciliation.rs:1-20`, `:201-276` ;
  `crates/kesh-api/src/routes/reconciliation.rs:69-72`, `:586-662`, `:1808-1817`, `:1919-1985` ;
  `crates/kesh-db/src/repositories/invoice_settlements_write.rs:215-245` ; `crates/kesh-core/src/types/money.rs:61-69`.
- `docs/manual/fr/user-manual.tex:971`, `:1308`, `:2029`, `:2049`, `:2330`, `:2901` ; `docs/api-external.md:505`, `:551`.
- `docs/agents-orchestration.md` ; `CLAUDE.md` § *Review Iteration Rule*, § *Pattern batch*, § *Test Locally First*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-14 — Fiche créée (`bmad-create-story`, Opus 5.5, autonomie) sur `fcd2ae75`. Prémisses de #665 établies au
  code (exactes ; précision : la tolérance est symétrique et seul le côté « le paiement dépasse » est refusé ; un
  doc-comment du dépôt faux depuis l'origine). Voie 1 retenue, voie 2 écartée sur ses conséquences comptables établies
  au code. Choix C-29-6-1 à 9.
