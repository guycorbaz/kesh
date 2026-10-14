# Story 29.6 : La réconciliation ne propose plus une facture qu'un trop-perçu de quelques centimes fait refuser

## Status

**ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `fcd2ae75` (kickoff de
l'Epic 29), branche `story/29-6`, worktree `/home/gcorbaz/devel/kesh-29-6`. Story **ajoutée à l'Epic 29** par
l'orchestrateur le 2026-10-14 ; sortie dans la **v0.13.4**. Choix consignés au registre propre à la story,
`epic-29-choix-autonomes-29-6.md`, **C-29-6-1 à C-29-6-19** (à verser par union au registre de l'epic). 10 AC. PR :
**`closes #665`** — dans le titre ou le corps de la PR **seulement**, jamais dans un message de commit, même en prose
(B4 de la rétrospective 28). Validation P1 faite et remédiée (Sonnet, 0/0/1/9) ; **le périmètre s'étend à l'écran**
(AC10, décision de l'orchestrateur sur le MEDIUM R1). Validation P2 faite et remédiée (Opus, 0/0/4/16) ; passe ciblée
P3 à lancer (`29-6-validate-prompt-p3-ciblee.md`, Haiku, une lentille sur le diff de la remédiation P2).

⚠️ **Angle mort assumé, renvoyé à [#670]** *(validation P2, R2)* : une transaction **rejetée reste `pending`**, compte
comme non rapprochée et **empêche la clôture du compte bancaire** ; aucun geste ne la rapproche d'un règlement déjà
saisi depuis la fiche. Le rejet que cette fiche rend possible (AC10) **retire** la transaction de la liste et empêche
qu'elle soit passée une seconde fois ; il ne la **règle** pas. Les textes prescrits (AC7) le disent ; la 29-6 ne ferme
pas #670 (l'intégration de #670 à l'epic revient à l'auteur ; la fiche n'en dépend pas, C-29-6-17).

⚠️ **Dernière de la série 29-2 → 29-3 → 29-5 → 29-6** : elle touche la même requête que la 29-2
(`find_unpaid_invoices_for_window`) et des textes que la 29-3 écrit sur ce défaut même (§ *Zones partagées*). Le
développement **commence par un rebase** sur `origin/main` une fois les trois autres mergées, et **relit** tous les
numéros de ligne de cette fiche, relevés sur `fcd2ae75`.

## Story

**As a** utilisateur qui rapproche ses relevés (Comptable ou Admin),
**I want** que la réconciliation ne me propose jamais une facture dont l'acceptation sera refusée parce que le paiement
la dépasse de quelques centimes,
**so that** ce que l'écran propose soit ce qu'il accepte — comme pour les dates depuis la v0.13.3 (#548) —, et que je
sache, pour un client qui a payé un peu trop, qu'il faut passer par la fiche de la facture — puis rejeter, depuis
l'écran, la transaction restée sans proposition, pour qu'elle ne soit pas passée une seconde fois (AC10 ; elle reste
en attente, [#670]).

Issue : [#665](https://github.com/guycorbaz/kesh/issues/665) (P2, bug ; relevée pendant la fiche de la 29-3). Epic :
`_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-6*. Cible : **v0.13.4**.

### Les prémisses, établies au code (`fcd2ae75`) — exactes, une précision (C-29-6-1)

| Prémisse (#665) | Au code | Verdict |
|---|---|---|
| La proposition retient une facture dont le reste dû est à ± 5 centimes du montant (`AMOUNT_TOLERANCE_HUNDREDTHS = 5`) | `crates/kesh-api/src/routes/reconciliation.rs:72` ; `Decimal::new(AMOUNT_TOLERANCE_HUNDREDTHS, 2)` `:637`, passé à `find_unpaid_invoices_for_window` `:656-662` ; `HAVING amount_due BETWEEN ? - ? AND ? + ?` (`crates/kesh-db/src/repositories/reconciliation.rs:257`, binds `:267-270`) | **Exacte** |
| L'acceptation classe un paiement supérieur au reste dû en `Overpayment`, sauf s'il égale l'arrondi au centime | `classify_payment` (`crates/kesh-db/src/repositories/invoice_settlements.rs:175-184`), appelée par `accept_one_invoice` étape (c) (`routes/reconciliation.rs:1915-1944` — commentaire dès `:1915`, `match` `:1930-1931` ; validation P2, F7) → `RECONCILIATION_OVERPAYMENT` | **Exacte** |
| Une transaction qui dépasse la facture de 1 à 5 centimes est proposée, puis refusée | Une facture n'est candidate que si son score total est > 0 (`kesh-reconciliation/src/matching.rs:81`) ; le score de montant est nul hors égalité au centime (`:109-119`) : la facture proposée porte donc une référence ou un nom de client reconnu, passe la garde de score (étape 7bis, `routes/reconciliation.rs:1808`) et tombe sur le trop-perçu. Exemple : 10.02 sur un reste de 10.0050 — **candidate par le calcul** (reste dans `[9.97, 10.07]`, référence reconnue), et **refusée** par le test existant `two_centimes_over_a_half_centime_invoice_is_an_overpayment` (`crates/kesh-api/tests/reconciliation_e2e.rs:5489`), qui n'appelle que `post_accept_one` : il établit le refus, pas la proposition *(validation P1, R6)* | **Exacte** |
| « Aucun effet comptable : rien ne s'écrit » | Le refus précède toute écriture (`FailedProposal`, pattern batch) | **Exacte** |

**La précision** : la tolérance est **symétrique**, ses deux côtés ne se valent pas.

- **La transaction dépasse le reste dû** (le cas de #665) : refusée en trop-perçu, sauf d'un **demi-centime** au plus
  (`SettlesWithRounding` : le paiement égale l'arrondi au centime du reste brut). C'est le défaut.
- **Le reste dû dépasse la transaction** de 1 à 5 centimes : classée `Ordinary`, **acceptée** en règlement partiel ; la
  facture reste ouverte pour la différence. Ni refus ni écriture fausse : **pas le défaut de #665**, et inchangé
  (C-29-6-4).

Et le doc-comment du dépôt (`repositories/reconciliation.rs:214-219`) affirme que « la tolérance couvre l'écart à son
arrondi au centime (au plus 0.005) » : c'est **faux depuis son écriture** (`eecf40cb`, Story 25-4-c ; la tolérance de
0,05 date de `22fd6d4a`, Story 8-4) — elle vaut dix fois l'écart d'arrondi. Réécrit par l'AC2.

### Les deux voies de #665, et leurs conséquences comptables établies au code (C-29-6-2)

**Voie 1 — la proposition écarte le trop-perçu.** Rien ne s'écrit de nouveau. La borne basse du reste dû devient la
plus petite valeur que l'acceptation admet ; la borne haute (paiement partiel) reste à 5 centimes. Le client qui a payé
3 centimes de trop n'est plus proposé ; il se traite comme aujourd'hui depuis la fiche de la facture — où le règlement
manuel le **refuse aussi** comme trop-perçu (`settle_invoice`, `invoice_settlements_write.rs:223-226`) : on enregistre
le montant dû, et l'écart reste sur le compte bancaire à justifier (remboursement au client, ou écriture manuelle).
C'est le traitement que le manuel décrit déjà pour les frais de rappel payés (`user-manual.tex:1308`).

⚠️ **Le geste qui suit — rejeter la transaction — est impraticable au sol** *(validation P1, R1, vérifié par
l'orchestrateur)* : la ligne d'une transaction sans candidate n'a pas de case à cocher
(`ReconciliationProposals.svelte:318-330`, manuel `:2050`), et `onReject` n'agit que sur les lignes cochées
(`:237-254`). Après la voie 1, le client qui a payé trop reste sur une ligne « Aucune correspondance » qu'aucun geste
de l'écran n'écarte — la route `POST /reconciliation/reject` l'accepterait, l'écran ne l'atteint pas. Le même défaut
vaut pour le contournement de la 29-2 (C-29-2-8, reliquat d'une facture non proposée). **La 29-6 étend donc son
périmètre** : une ligne sans proposition se coche ; *Accepter* ne la traite pas — elle se rejette, ou s'impute comme
aujourd'hui par *Affecter manuellement* ou *Éclater* (AC10, C-29-6-10 ; reformulé à la validation P2, R1/F2 : « pour
*Rejeter* seulement » excluait à tort ces deux boutons, présents sur la ligne). Rejetée, la transaction **reste en
attente** et empêche la clôture du compte bancaire : angle mort assumé, [#670] (C-29-6-17).

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
`crates/kesh-core/src/types/money.rs:66-71`) : pour `R ≥ 0`, `arrondi(R) = P` ⇔ `R ∈ [P − 0,005 ; P + 0,005[`. Sur
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

- `const HALF_CENTIME: Decimal = Decimal::from_parts(5, 0, 0, false, 3);` — la valeur est `0.005`. **Cette forme
  seule** : `Decimal::from_parts` est une `const fn` (rust_decimal 1.42.1), précédent dans le même crate
  `const ROUNDING_WRITE_OFF_LIMIT: Decimal = Decimal::from_parts(5, 0, 0, false, 2);`
  (`invoice_settlements_write.rs:406`) ; `dec!` n'est pas disponible en production (`rust_decimal_macros` est sous
  `[dev-dependencies]`, `crates/kesh-db/Cargo.toml:38-42`) et `Decimal::new` n'est pas `const` *(validation P1, F1)* ;
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
`accept_one_invoice` étape (c) (`:1915-1922`, validation P2, F7) reçoit une ligne : la proposition n'offre plus ce que ce garde refuse
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
- Le libellé de `RECONCILIATION_OVERPAYMENT` (`frontend/src/lib/features/reconciliation/failed-proposal-label.ts:244-248`)
  reste juste pour l'appel direct : non modifié. L'écran change **seulement** par l'AC10 (une case sur les lignes sans
  proposition) ; ni l'API de rejet ni celle de l'annulation du rejet ne changent *(validation P1 — remplace la rédaction
  d'origine « aucun écran, aucune clé i18n, aucune capture », C-29-6-6 annoté)*.

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

Un second test neuf, `find_unpaid_invoices_overpayments_take_no_place_in_the_limit`, garde l'invariant « le filtre est
dans la requête, avant `ORDER BY … LIMIT 50` » (C-29-6-3, C-29-6-13) *(validation P1, R4)* : une transaction de
**100.00** à `D` ; **50** factures à **99.99** (trop-perçu), puis une facture à **100.00**, toutes à la date `D`,
insérées dans cet ordre (la valide a le plus grand `id`, donc la dernière place de `ORDER BY i.date, i.id`) ; tolérance
`dec!(0.05)`. Attendu, écrit en dur : **une seule** facture rendue, celle à 100.00. Cinquante suffisent et sont le
nombre juste : avec un filtre posé après la requête, les cinquante places du `LIMIT` sont prises par les trop-perçus et
la valide n'est jamais lue (mutation M8). Rouge au sol (les cinquante trop-perçus sont rendus, la valide non), vert
après. Le nombre `50` se lit au `LIMIT` du SQL au moment du développement ; s'il a changé, le test suit le `LIMIT`
(constante ou commentaire qui le nomme), sans quoi il cesse de garder ce qu'il prétend.

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
  réconciliation — cochez sa ligne « Aucune correspondance » et *Rejeter* (geste rendu possible par l'AC10) —, pour
  qu'elle ne soit pas passée une seconde fois, et traitez l'écart (remboursement, ou écriture manuelle). ⛔ Le texte
  **ne dit pas** que le rejet règle la transaction ou la sort de l'attente *(validation P2, R2)* : rejetée, elle
  **reste en attente** dans le détail de son import, compte comme non rapprochée et **empêche la clôture du compte
  bancaire** (renvoi `\S\ref{sec:clore-compte-bancaire}`, qui le dit déjà, `:1690-1694`) — angle mort [#670], sans
  numéro d'issue au manuel (C-29-6-17). La phrase d'origine « — sans quoi la transaction reste en attente » est
  **abandonnée** : elle promettait l'inverse. ⚠️ Le même paragraphe est réécrit par la 29-2 (dates) : au rebase, ne
  toucher que la condition de montant et la phrase du trop-perçu.
- **Liste des lignes, `:2050`** (« sans candidate, la ligne indique «~Aucune correspondance~» et n'a pas de case à
  cocher ») devient : la ligne indique «~Aucune correspondance~» et porte une case, comme les autres ; \emph{Accepter}
  ne la traite pas — tant qu'une telle ligne est cochée, il reste grisé et la page le dit (AC10) ; elle se rejette, ou
  s'impute par \emph{Affecter manuellement} ou \emph{Éclater} *(validation P2, R1/F2 : la rédaction « sa case ne sert
  qu'à \emph{Rejeter} » excluait ces deux boutons, présents sur toute ligne en attente, `.svelte:371-397`)*.
- **Paragraphe \emph{Rejeter}, `:2057`** *(validation P2, R3/F3 — la P1 le disait « vrai, relu »)* : « \emph{Rejeter}
  écarte la proposition » devient : \emph{Rejeter} écarte **la transaction** de la réconciliation — qu'elle ait une
  proposition ou non — **sans la passer en comptabilité** ; la suite du paragraphe (marquée comme revue, statut
  «~pending~», *Annuler le rejet*) reste, et reçoit une incise : une transaction rejetée compte comme non rapprochée et
  empêche la clôture de son compte bancaire (`\S\ref{sec:clore-compte-bancaire}`).
- La légende de la capture (`:2025`, « la case à cocher ») reste vraie, relue. La phrase « Chaque ligne cochée est
  traitée pour elle-même » (`:2062`) reste vraie : *Accepter* n'envoie jamais de ligne sans proposition. Le paragraphe
  de la clôture du compte (`:1690-1694`) reste vrai, relu : il dit déjà qu'une transaction rejetée empêche la clôture.
- **FAQ, `:2901`** : « Aucune facture validée et non payée n'a un \textbf{reste dû} égal au montant de la transaction
  \textbf{à 5 centimes près} » devient : aucune facture n'a un reste dû **égal au montant, ou supérieur de 5 centimes
  au plus** — un client qui a payé trop, même de quelques centimes, n'est pas proposé.
- **FAQ, « Que faire ensuite », `:2910`** *(validation P2, R7 ; R1)* : la phrase « La transaction bancaire reste ensuite
  dans la liste de la réconciliation~: Kesh ne la relie pas au règlement saisi depuis la fiche. » devient : Kesh ne relie
  pas la transaction au règlement saisi depuis la fiche ; **rejetez-la** (cochez sa ligne, puis \emph{Rejeter}) pour
  qu'elle ne soit pas passée une seconde fois — rejetée, elle reste en attente dans le détail de son import et empêche
  la clôture du compte bancaire (`\S\ref{sec:clore-compte-bancaire}`). La consigne voisine « N'utilisez ni
  \emph{Affecter manuellement} ni \emph{Éclater} » (même puce) reste — c'est pourquoi l'indication de l'écran ne les
  nomme pas (C-29-6-16). ⚠️ La 29-2 (C-29-2-8) prescrit le même rejet : au rebase, relire si elle a déjà touché cette
  puce, et ne garder qu'une rédaction.
- **Les phrases que la 29-3 écrit sur ce défaut** (fiche 29-3, AC7, ligne `:2049` au sol : « une facture que le
  paiement dépasse de quelques centimes est proposée, mais son acceptation est refusée en trop-perçu --- enregistrez
  alors le règlement depuis la fiche de la facture (issue \#665) ») : **retirées** au rebase — elles deviennent
  fausses. Commande de l'inventaire : `grep -n '665\|trop-perçu\|dépasse' docs/manual/fr/user-manual.tex`.
- **Non modifiés, relus** : `:1308` (frais de rappel : « ne verra pas son versement proposé […] puisqu'il dépasse ce
  qui reste à payer » — désormais vrai **à tout montant**, auparavant faux pour 1 à 5 centimes de frais) ; `:2330`
  (« Pas de tolérance de montant » du lettrage, inchangé) ; `:971` (arrondi à 5 centimes des factures émises, autre
  sujet).

PDF régénéré (`make fr` dans `docs/manual/`, sous le verrou de gate) et commité ; contrôle **aplati et normalisé**
(ligatures, comme à la 29-3) :

```sh
pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' \
  | sed "s/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g;s/’/'/g" > /tmp/claude-1000/29-6-manuel.txt
```

(`s/’/'/g` : le `.tex` écrit `n'a`, le PDF rend `n’a` — sans cette normalisation, une portion à apostrophe rend 0 sans
aucune césure ; validation P2, R6.)

- **contrôles positifs** *(validation P1, R2)* : `grep -c` d'une **portion courte** (trois à cinq mots, sans césure
  probable) de **chacune** des phrases neuves — la condition de montant de `:2029`, le contournement de `:2029`, la FAQ
  `:2901`, la ligne «~Aucune correspondance~» de `:2050`, le paragraphe \emph{Rejeter} de `:2057` et la FAQ `:2910`
  *(ces deux derniers : validation P2)* — rend **≥ 1** chacune ; les portions retenues et leurs comptes vont au Dev
  Agent Record ;
- **contrôles négatifs**, qui ne valent qu'adossés aux positifs (un zéro peut venir d'une césure) :
  `grep -o "à 5 centimes près"` ne rend plus rien, `grep -o "pas de case à cocher"` non plus, ni
  `grep -o "écarte la proposition"`, ni `grep -o "sans quoi la transaction reste en attente"`, ni
  `grep -o "ne sert qu'à"` *(validation P2)* ; et `grep -o "#665"` ne rend plus que ce que le texte final assume (rien,
  si le renvoi est retiré), `grep -o "670"` rien (C-29-6-17).

Gardes textuelles : `cargo nextest run -E 'binary(textes_coherents)'` vert.

`docs/api-external.md` § *Accepter des propositions de rapprochement* (`:551`, « un montant supérieur au reste dû est
refusé (`RECONCILIATION_OVERPAYMENT`, jamais écrit) ») : une phrase ajoutée — `GET /reconciliation/proposals` ne
propose plus une facture dont le reste dû est inférieur **de plus d'un demi-centime** au montant de la transaction
*(depuis la v0.13.4 ; validation P1, R8 : « hors l'écart d'arrondi au centime » se lisait « jusqu'à 0,01 »)* ; le refus ne se rencontre plus que par un couple choisi hors des propositions, ou une
facture modifiée entre la lecture et l'acceptation. Contrat inchangé.

`CHANGELOG.md` : sous `## [0.13.4] — Non publié` (créée par la 29-2 ; sinon à créer au-dessus de `## [0.13.3] —
2026-10-14`), section `### Corrigé` : « **La réconciliation ne propose plus une facture qu'un paiement dépasse de
quelques centimes ([#665](https://github.com/guycorbaz/kesh/issues/665)).** Une facture était proposée dès que son
reste dû valait le montant reçu à 5 centimes près, dans les deux sens ; mais un paiement qui la dépasse est un
trop-perçu, et son acceptation était refusée. Seuls sont désormais proposés le reste dû exact (à l'arrondi au centime
près) et un paiement inférieur de 5 centimes au plus, accepté comme règlement partiel. Un client qui a payé trop
s'enregistre depuis la fiche de la facture ; rejetez ensuite la transaction, pour qu'elle ne soit pas passée une
seconde fois : une ligne « Aucune correspondance » se coche désormais — *Accepter* ne la traite pas et reste grisé tant
qu'elle est cochée. Rejetée, la transaction reste en attente et empêche la clôture du compte bancaire
([#670](https://github.com/guycorbaz/kesh/issues/670)). API : mêmes codes. » *(Validation P2, R1/R2 : la rédaction
P1 « pour *Rejeter* seulement » excluait *Affecter manuellement* et *Éclater*, et le rejet n'était pas dit laisser la
transaction en attente.)*

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
| M8 | le filtre posé **après** la requête, **dans `find_unpaid_invoices_for_window`, après le `fetch_all`** (`repositories/reconciliation.rs:271` au sol ; posé dans `get_proposals`, M8 serait indiscernable de M3 au dépôt — validation P2, F6) : `HAVING` rendu à l'ancienne borne (`BETWEEN ? - ? AND ? + ?`), puis, en Rust, les lignes rendues filtrées par `classify_payment(tx_amount, amount_due) != Overpayment` | AC5, second test (`find_unpaid_invoices_overpayments_take_no_place_in_the_limit`) — le seul : AC5 premier test, AC6 et AC4 restent verts, c'est ce qui fait la mutation *(validation P1, R4)* |
| M9 | la case rendue de nouveau conditionnelle (`{#if p.candidates.length > 0}` autour de l'`<input>`) | AC10, Vitest (case présente sur la ligne sans proposition) ; AC10, E2E |
| M10 | *Accepter* non grisé quand une ligne sans proposition est cochée (condition retirée du `disabled`) | AC10, Vitest — **l'assertion sur l'attribut** (`(btn as HTMLButtonElement).disabled === true`) **seule** ; l'assertion « `acceptProposals` jamais appelé » ne la tue pas, la garde de tête d'`onAccept` arrêtant le clic *(validation P2, F1)* |
| M11 | l'indication « ne peut pas être acceptée » (`reconciliation-unmatched-not-acceptable-hint`) rendue sans condition | AC10, Vitest (indication absente quand seules des lignes avec proposition sont cochées) |
| M12 | la garde de tête d'`onAccept` retirée (`selectionHasUnmatched` ôté du `return` initial) | AC10, Vitest — `fireEvent.click` sur *Accepter* grisé : ni `reconciliation-error`, ni appel d'`acceptProposals`, la table reste *(validation P2, F1)* |
| M13 | *Rejeter* grisé quand une ligne sans proposition est cochée (`selectionHasUnmatched` ajouté à son `disabled`) | AC10, Vitest — l'assertion `disabled === false` sur *Rejeter* ; l'assertion « `rejectProposals(17, [2])` appelé » ne la tue pas seule, `fireEvent.click` livrant l'événement à un bouton désactivé *(validation P2, F1)* |

Restauration : `git checkout -- <fichier> && touch <fichier>` (`docs/agents-orchestration.md` § 7). M4 et M7 coïncident
dans leur effet ; elles diffèrent par le site muté et se déclarent toutes deux.

⚠️ **`fireEvent.click` atteint le gestionnaire d'un bouton désactivé, dans le harnais du dépôt** *(validation P2, F1 —
la P1 déclarait la garde d'`onAccept` « inatteignable », prescription fausse au code)* : jsdom ne bloque que la
méthode `click()` d'un contrôle désactivé, pas `dispatchEvent` ; Svelte 5 délègue `onclick` et appelle le gestionnaire
quand la cible est le bouton lui-même, désactivé ou non (`node_modules/svelte/src/internal/client/dom/elements/events.js`,
condition `event.target === current_target`). Le dépôt le documente déjà
(`frontend/src/lib/features/bank-import/DeleteBankImportAction.test.ts:191-194`). Conséquences : un état grisé se prouve
par **l'attribut** `disabled`, jamais par l'absence d'appel après `fireEvent.click` ; et la garde de tête d'`onAccept`
est **éprouvable** — sans elle, le clic tombe sur le `throw` « No candidate for txId=… » (`.svelte:205-207`), `errorMsg`
est posé et la branche `{:else if errorMsg}` (`:270`) remplace la table par `reconciliation-error` : c'est ce que tue
M12. Seul le `throw` « No candidate », que la garde précède, reste **non éprouvé** (inatteignable tant que la garde
est là) : il se déclare tel au Dev Agent Record.

**AC9 — Gates.**

`kesh-db` touché : **gate complet même en boucle de revue** (`CLAUDE.md` § *Pendant une boucle de revue*), base
`kesh_296` remise à zéro avant chaque gate complet (KF-039), `scripts/test-fast.sh` sous `flock` ; frontend (`npm run
check`, `lint-i18n-ownership`, `test:unit`, `build`) — le composant, ses tests et une clé i18n changent (AC10) ; E2E
complet au **dernier commit de code** (base `kesh_e2e_296`, port **3047**, recette de `docs/testing.md` à la lettre),
le spec neuf de l'AC10 compris, chaque rouge jugé fichier par fichier contre `docs/testing.md` § *Les échecs
attendus* ; captures clair et sombre de l'AC10 relues à l'œil et décrites au Dev Agent Record. Le Dev Agent Record ne
déclare que ce qui a tourné, chiffres recomptés.

**AC10 — Une transaction sans proposition se rejette depuis l'écran (C-29-6-10, C-29-6-11, C-29-6-16 à C-29-6-19).**

*Ajouté à la validation P1 (R1), sur décision de l'orchestrateur : sans lui, le geste « régler depuis la fiche, puis
rejeter la transaction » que prescrivent la 29-6 (AC7) et la 29-2 (C-29-2-8) est impraticable.*

**Établi au code (`fcd2ae75`), et qui ne change pas** :

- **Quelles lignes sont « sans proposition »** *(validation P2, R1/F3 — la P1 n'écrivait que le trop-perçu)* : toute
  transaction dont `candidates` est vide, c'est-à-dire, au sol, (i) les **débits** et les transactions **non CHF**,
  pour lesquels `get_proposals` ne cherche aucune facture (`routes/reconciliation.rs:640-652` : `tx.currency != "CHF"`
  et `tx.amount <= Decimal::ZERO` → `Vec::new()`), **sans règle** qui les vise — paiements fournisseurs, loyers,
  frais bancaires ; (ii) les crédits CHF sans facture candidate **ni** règle — dont le client qui a payé trop (#665) et
  le reliquat d'une facture non proposée (29-2, C-29-2-8). **La case leur est ouverte à toutes**, et c'est voulu : le
  rejet en masse d'un débit n'a rien de faux (réversible par *Annuler le rejet*), et *Affecter manuellement* et
  *Éclater*, présents sur toute ligne en attente (`.svelte:371-397`), restent le geste du manuel pour une charge
  (`user-manual.tex:2911`). D'où l'indication, qui dit ce qui est vrai de la **sélection** (*Accepter* ne la traite
  pas) et ne pousse vers aucun geste (point 4, C-29-6-16). Une ligne à **règle** seule a `candidates.length > 0` : elle
  compte comme « avec proposition », acceptable (F5).
- `POST /api/v1/reconciliation/reject` ne lit **aucune** candidate : après le refus d'un compte archivé (`404`) et
  le pré-vol du **compte clos** (`routes/reconciliation.rs:3256-3266`, chaque transaction en `failed[]`
  `BANK_ACCOUNT_CLOSED`, `200` — validation P2, R11), le pré-vol classe chaque transaction par
  `classify_batch_transaction` (`routes/reconciliation.rs:851-901` — absente, rapprochée, déjà rejetée, sinon
  éligible), puis `reject_batch` (`:3431-3550`) pose `auto_match_rejected_at` par `UPDATE … WHERE matched_entry_id IS
  NULL AND auto_match_rejected_at IS NULL`. Chaque refus va dans `failed[]` en `200` (Pattern batch), aucun ne dépend
  d'une proposition. Le test existant `post_reject_marks_transactions_as_manually_reviewed`
  (`crates/kesh-api/tests/reconciliation_e2e.rs:1560`) rejette deux transactions **sans aucune facture** : la route
  admet déjà ce que l'écran ne lui envoie pas. **Aucun changement d'API, aucun test backend neuf.**
- **Réversibilité** : *Annuler le rejet* (Story 28-5b, `UnrejectTransactionAction`, détail de l'import,
  `routes/(app)/bank-import/[id]/+page.svelte:160` ; route `post_unreject_transaction`, doc `:4759-4782`) remet
  `auto_match_rejected_at` à `NULL` sans lire de candidate : une transaction sans proposition rejetée par erreur revient
  dans la liste. Le rappel sous la liste (`reconciliation-rejected-hint`, `.svelte:452-460`) reste vrai.
- **Ce que le rejet ne fait pas — angle mort assumé, [#670]** *(validation P2, R2)* : une transaction rejetée **reste
  `pending`** (`auto_match_rejected_at` posé, `status` inchangé ; manuel `:2057`), compte comme non rapprochée et
  **empêche la clôture du compte bancaire** (`crates/kesh-db/src/repositories/bank_accounts.rs:1025`, `status =
  'pending'` ; manuel `:1690-1694`). Pour le trop-perçu ou le reliquat réglé depuis la fiche, **aucun geste ne la
  rapproche ensuite** : le règlement de la fiche ne s'y relie pas, *Affecter manuellement* et *Éclater* n'offrent que
  les classes 5, 6 et 7, et l'import ne se supprime que s'il n'a aucune transaction rapprochée. Le blocage est donc
  permanent, et la transaction tombera dans le contrôle « transactions bancaires en attente » que la 29-1 prévoit
  avant le bouclement. **Non résolu par la 29-6**, renvoyé à #670 (bug P2) ; aucun texte prescrit ne dit que le rejet
  règle la transaction (AC7) — C-29-6-17.
- **États de la sélection, relevés au code** *(validation P2, F5 ; aucun ne fait accepter une ligne sans
  proposition)* : (a) `load()` remet `selected` à vide (`.svelte:109`) — après tout lot, affectation ou éclatement
  réussis, l'indication disparaît ; `proposals` ne change qu'au `load()`, si bien qu'une transaction ne peut pas perdre
  sa proposition en restant cochée ; (b) sur erreur de `rejectProposals` (`catch`, `:249`), pas de rechargement : la
  sélection et l'indication restent — juste ; (c) pendant `busy`, les cases ne sont pas désactivées (`:320-326`) mais
  les items d'`onAccept` sont calculés avant le premier `await` (`:203-226`) ; (d) une ligne à règle seule est « avec
  proposition ». Le chemin *Accepter* → ligne sans proposition est fermé trois fois : `disabled`, garde de tête,
  `throw`.

**Ce qui change**, dans `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte` :

1. **La case** (`:318-330`) : rendue sur **toutes** les lignes, la condition `{#if p.candidates.length > 0}` retirée.
2. **Accepter grisé** dès qu'une ligne cochée n'a pas de proposition : un `$derived`
   `selectionHasUnmatched = proposals.some((p) => selected.has(p.bankTransactionId) && p.candidates.length === 0)`,
   ajouté au `disabled` du bouton (`:284`) ; le compteur « Accepter (n) » reste `selected.size`. *Rejeter* (`:293`)
   inchangé : il agit sur toutes les lignes cochées.
3. **Garde défensive** en tête d'`onAccept` (`:192`) : `if (selected.size === 0 || selectionHasUnmatched) return;` ; le
   commentaire `:199-202` (« Une tx sans candidate ne peut pas être sélectionnée ») est réécrit : elle peut l'être
   (pour *Rejeter*, ou avant *Affecter manuellement* / *Éclater*) ; *Accepter* est grisé et gardé, le `throw` reste en
   dernier rempart.
4. **Une indication**, rendue **seulement si** `selectionHasUnmatched`, **dans la rangée des boutons** — dernier
   enfant du `<div class="mb-4 flex gap-2">` (`:280`), après *Rejeter* —, et non entre la rangée et la table
   *(validation P2, F4 : un paragraphe qui apparaît au cochage au-dessus de la table la descend d'une ligne sous le
   pointeur, et le clic suivant peut cocher la ligne voisine, puis *Rejeter* la mauvaise transaction)*. Le `div`
   reçoit `flex-wrap items-center` ; l'élément : `<p class="text-sm text-text-muted"
   data-testid="reconciliation-unmatched-not-acceptable-hint">`. À largeur étroite, l'indication peut passer à la
   ligne et décaler la table : accepté et écrit (C-29-6-18), à vérifier sur les captures. Clé
   **`reconciliation-labels-unmatched-not-acceptable-hint`** *(renommée à la validation P2, R1/F2 : « reject-only »
   reproduisait l'erreur du texte)*, repli = valeur fr-CH mot pour mot :
   - **fr-CH** : `Une transaction sans correspondance ne peut pas être acceptée : décochez-la pour accepter les autres.`
   - **de-CH** : `Eine Transaktion ohne Übereinstimmung kann nicht akzeptiert werden: Entfernen Sie die Markierung, um die übrigen zu akzeptieren.`
   - **it-CH** : `Una transazione senza corrispondenza non può essere accettata: tolga la selezione per accettare le altre.`
   - **en-CH** : `A transaction with no match cannot be accepted: untick it to accept the others.`

   *(Validation P2, R1/F2 : la rédaction P1, « ne peut qu’être rejetée » — de « kann nur abgelehnt werden », it « può
   solo essere rifiutata », en « can only be rejected » —, était fausse à l'écran : la même ligne porte *Affecter
   manuellement* et *Éclater*, et pour un débit sans règle elle poussait vers un rejet sans écriture au lieu de
   l'imputation. Le texte retenu dit la seule vérité de la sélection — *Accepter* ne la traite pas — et **ne nomme
   aucun autre geste** : pour un paiement de client, la FAQ (`:2910`) interdit précisément *Affecter manuellement* et
   *Éclater*, si bien qu'une indication qui les proposerait pousserait le cas même de #665 vers un geste faux,
   C-29-6-16.)*

   Termes alignés sur l'écran (`reconciliation-labels-no-candidate`, `-actions-accept`, `-actions-reject`, fr `:1212`,
   `:1252-1253`, de/it/en `:1149`, `:1189-1190`) ; vouvoiement de l'it-CH comme `reconciliation-labels-rejected-hint`
   (« il suo ruolo ») ; apostrophe typographique `’` en fr-CH comme les clés voisines. Ponctuation du deux-points :
   celle du catalogue fr-CH, relue au voisinage. Gardes : `npm run lint-i18n-ownership`, `i18n-libelle-en-dur.test.ts`,
   parité des quatre catalogues, et `frontend/src/lib/shared/i18n-keys.test.ts`, dont `sitesTotal` (`:634`, 2107 au sol)
   prend **+1** — **recompté à la source au rebase** (la 29-3 en ajoute un au même fichier), avec son commentaire daté.
5. **L'en-tête du composant** (`:6-12`, « sans checkbox (tx non-sélectionnable) ») est réécrit — **dans sa version
   29-3** : la 29-3 touche `:8`, contenu dans `:6-12` (zones **chevauchantes**, non voisines ; validation P2, F9).
6. **Ordre des indications** *(validation P2, R9/F4)* : dans la branche « liste non vide », de haut en bas — la rangée
   des boutons, qui porte en dernier l'indication de la 29-6 ; puis le paragraphe de priorité de la 29-3
   (`reconciliation-priority-hint`, fiche 29-3 `:286-290`, entre la rangée et la table, rendu dès qu'une ligne a une
   candidate, donc **sans** dépendre du cochage) ; puis la table. Les deux ne se contredisent pas : liste toute
   « Aucune correspondance », seule celle de la 29-6 peut paraître.

**Hors du composant** — le commentaire du spec `frontend/tests/e2e/reconciliation-deleted-import.spec.ts:12-14`
*(validation P2, R4)* — « La **règle d'affectation** qui rend les deux transactions sélectionnables […] pour ne rendre
sélectionnable aucune transaction d'un autre spec » — devient faux : toute ligne se coche. Réécrit : la règle n'est
plus nécessaire à la sélection depuis la 29-6 ; elle est **gardée** (le spec reste identique dans ce qu'il éprouve —
un rejet en lot dont une transaction a disparu) ; le ciblage par `data-tx-id` (`:153-154`) suffit à ne cocher aucune
transaction d'un autre spec. Aucune ligne de code du spec ne change (C-29-6-19).

**Tests Vitest** (`ReconciliationProposals.test.ts`, rouges d'abord) :

- le test `:179` *shows neutral state for tx without candidates* asserte aujourd'hui **zéro** case : il est **réécrit**
  (une case, `data-tx-id` de la ligne) — c'est le seul test du dépôt qui change de sens (inventaire (c)) ;
- ligne sans proposition cochée, seule → *Rejeter* est **actif** (`(btn as HTMLButtonElement).disabled === false`,
  qui tue M13) et appelle `rejectProposals(17, [2])` ; *Accepter* est grisé (`disabled === true`, qui tue M10) ;
  l'indication est rendue ;
- même état, `fireEvent.click` sur *Accepter* grisé → **aucun** `reconciliation-error`, `acceptProposals` non appelé,
  la table (`tx-checkbox`) toujours présente — tue M12 (garde de tête retirée) ;
- une ligne avec proposition et une sans, toutes deux cochées → *Accepter* grisé (attribut), indication rendue ;
  décocher la ligne sans proposition → *Accepter* actif (attribut), indication absente, et *Accepter* envoie la seule
  ligne avec proposition ;
- seules des lignes avec proposition cochées → indication **absente** ;
- repli de l'indication égal au catalogue fr-CH (patron `:661-678`).

La fabrique sans proposition (`makeProposalWithoutCandidate`, `:108-123`) est **créditrice** ; aucun test débiteur
n'est prescrit : le composant ne lit pas le signe — seule `candidates.length` décide —, si bien qu'un débit sans règle
s'y comporte à l'identique (validation P2, F3, relevé non bloquant).

⚠️ Un état **grisé ou actif se prouve par l'attribut `disabled`**, jamais par l'absence ou la présence d'un appel après
`fireEvent.click`, qui livre l'événement à un bouton désactivé (AC8, encadré ; précédent
`DeleteBankImportAction.test.ts:191-194`) *(validation P2, F1)*.

**E2E Playwright** — `frontend/tests/e2e/reconciliation-reject-unmatched.spec.ts`, neuf, sur le patron de
`reconciliation-unreject.spec.ts` (relevé **unique** à l'exécution : références, montant, contrepartie réécrits ;
import supprimé en `finally`) : importer une transaction qu'aucune facture ni règle ne rapproche (contrepartie et
références uniques de préfixe propre au spec ; montant tiré à l'exécution **hors de la plage** des factures que
montent les autres specs de la base partagée — par exemple au-dessus de 90 000 CHF —, plage relevée au rebase par un
grep des montants des specs ; même réserve que le patron sur la fenêtre des 100 transactions les plus récentes) ; sur *Réconciliation*, sa ligne indique « Aucune correspondance » **et
porte une case** ; la cocher → *Accepter* grisé, indication visible ; *Rejeter* → « 1 opération(s) réussie(s). », la
ligne quitte la liste ; au détail de l'import, « Rejetée le » ; *Annuler le rejet* → la ligne revient dans la liste.
Une fois par schéma (`page.emulateMedia`, `light` / `dark`), deux captures — la ligne cochée avec l'indication, le
bilan après le rejet — dans `KESH_E2E_CAPTURES_DIR`, sinon la sortie du test, hors dépôt, sans `toHaveScreenshot`. C'est
le seul test qui voit l'identifiant d'une ligne sans proposition traverser la frontière HTTP jusqu'à `…/reject`.
**Sélecteurs** *(validation P2, R5)* : la garde `frontend/src/lib/shared/e2e-selecteurs-traduits.test.ts` (allowlist
décroissante, KF-043 #326) rougit sur tout `getByText('…')` ou `getByRole(…, { name: '…' })` neuf figé sur un libellé
français ; le spec cible donc par `getByTestId` (`tx-checkbox` + `data-tx-id`, `reconciliation-accept-btn`,
`reconciliation-reject-btn`, `reconciliation-unmatched-not-acceptable-hint`, `reconciliation-success`, et au détail
de l'import `detail-tx-row` + `data-tx-id`, `detail-tx-rejected-on`, `detail-tx-unreject`) et vérifie les textes
(« Aucune correspondance », « Rejetée le ») par `toContainText` sur un élément ciblé par `data-testid`, comme le patron
(`reconciliation-unreject.spec.ts:146-150`). L'allowlist ne prend **aucune** entrée.

## Tasks / Subtasks

- [ ] **T0 — Rebase** sur `origin/main` (29-2, 29-3, 29-5 mergées) ; relire les numéros de ligne ; regreper
      `#665` dans `crates/`, `docs/`, `CHANGELOG.md` (§ *Zones partagées*).
- [ ] **T1 — Tests rouges** (AC4, AC5 — ses deux tests —, AC6, AC10 Vitest et E2E) : constater le rouge (compilation
      pour AC4, assertion pour les autres).
- [ ] **T2 — `kesh-db`** (AC1, AC2) : `HALF_CENTIME`, `least_due_without_overpayment`, `HAVING`, renommage du
      paramètre, doc-comments.
- [ ] **T3 — `kesh-api`** (AC2, AC3) : doc-comments de la constante et de l'étape (c), variable locale.
- [ ] **T3bis — Écran et i18n** (AC10) : case sur toutes les lignes, *Accepter* grisé et gardé, indication, clé dans
      les quatre catalogues, `sitesTotal` recompté, en-tête du composant ; commentaire de
      `reconciliation-deleted-import.spec.ts:12-14` ; spec E2E et captures clair et sombre.
- [ ] **T4 — Manuel FR + PDF, `api-external.md`, CHANGELOG** (AC7), dont le retrait des angles morts #665 de la 29-3,
      la ligne «~Aucune correspondance~» (`:2050`), le paragraphe \emph{Rejeter} (`:2057`) et la FAQ (`:2910`).
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

**Le rejet, établi à la validation P1** : un seul écran envoie `POST /reconciliation/reject`
(`ReconciliationProposals.svelte`, `onReject`, `:237-254`, via `rejectProposals`) ; il n'envoie que les lignes cochées,
et seule une ligne avec proposition se coche au sol (`:318-330`). La route n'impose rien de tel (AC10). Commande de
relecture : `grep -rn "rejectProposals\|reconciliation/reject" frontend/src --include=*.ts --include=*.svelte | grep -v
'\.test\.ts'`.

### Inventaire (b) — les refus d'acceptation d'une facture proposée qui restent (sites non résolus)

| Refus | Pourquoi une facture proposée peut le rencontrer | Sort |
|---|---|---|
| `RECONCILIATION_OVERPAYMENT` | appel direct, ou facture modifiée (règlement, avoir) entre la proposition et l'acceptation | **non résolu, assumé** : course, même statut que la facture redatée de la 28-7 |
| `ROUNDING_ACCOUNT_NOT_CONFIGURED` / `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` (`role: rounding`) | reste à quatre décimales soldé au centime (cas d et f d'AC6) sans compte d'arrondi utilisable | **non résolu, hors classe** : refus de **configuration**, déjà documenté (`user-manual.tex:2056`), relevé par la 29-3 (C-29-3-7). Une facture n'a de reste à quatre décimales qu'à TVA arrondie sur des lignes à quatre décimales ; le compte d'arrondi n'est exigé à la validation que **si le réglage d'arrondi à 5 centimes du total est actif** (`user-manual.tex:971`) — réglage décochable, absent des factures antérieures : un reste à quatre décimales peut donc exister sans compte d'arrondi désigné, et rencontrer ce refus *(validation P1, R3 : la rédaction d'origine laissait croire que la validation l'excluait)* |
| `SETTLEMENT_COUNTERPARTY_IS_CLAIM_ACCOUNT` (`role: counterparty`), `BANK_ACCOUNT_NOT_CONFIGURED`, `FISCAL_YEAR_INVALID` | configuration du compte bancaire ou exercice clos | **hors classe**, documentés |
| `RECONCILIATION_INVOICE_NOT_ELIGIBLE` (dates) | fermé par la 28-7 (et la 29-2 garde une seule borne) | hors classe |
| `RECONCILIATION_SCORE_TOO_LOW` | score recalculé nul : impossible pour une facture proposée (même calcul) hors course | hors classe |

**Et ce qui reste après le contournement** *(validation P2, R2)* — hors de la classe « refus d'acceptation », mais
conséquence directe du geste que la fiche prescrit :

| Site non résolu | Pourquoi | Sort |
|---|---|---|
| La transaction **rejetée** après un règlement saisi depuis la fiche (trop-perçu de #665 ; reliquat de la 29-2, C-29-2-8) | rejetée, elle reste `pending` (`bank_accounts.rs:1025`) et empêche la clôture du compte bancaire ; aucun geste ne la rapproche d'une écriture déjà passée | **non résolu, assumé, renvoyé à [#670]** (bug P2) ; écrit au manuel (`:2029`, `:2057`, `:2910`) et au CHANGELOG ; à signaler à la 29-1 (contrôle « transactions en attente » avant le bouclement) — C-29-6-17 |

### Inventaire (c) — les tests existants dont le sens pourrait changer

```
$ grep -rln "find_unpaid_invoices_for_window\|reconciliation/proposals" crates/*/tests/*.rs frontend/tests/e2e/*.ts
$ grep -rln "reconciliation/proposals" crates/*/tests/*.rs        # relevé de la validation P1 (R5)
crates/kesh-api/tests/bank_accounts_closed_e2e.rs
crates/kesh-api/tests/reconciliation_e2e.rs
crates/kesh-api/tests/reconciliation_rules_e2e.rs
$ grep -rn "tx-checkbox\|Aucune correspondance" frontend/src frontend/tests --include=*.ts   # pour l'AC10
```

à relever au sol et au rebase ; pour chaque fichier, chercher une facture dont le reste dû est **inférieur** au montant
de la transaction de 0,0051 à 0,05 et dont la proposition est attendue. Relevé du 2026-10-14, complété à la validation
P1 *(R5 : la commande rendait deux fichiers que le relevé ne nommait pas)* :

- `bank_accounts_closed_e2e.rs:637` : compte clos, aucune facture — sens inchangé.
- `reconciliation_rules_e2e.rs:942, :995, :1068, :1128` : montants 150.00 / -150.00, aucune facture ou règle seule —
  sens inchangé.
- `reconciliation_e2e.rs`, les lectures `GET /proposals` (`:687, :801, :881, :945, :1927, :2101, :2806, :3614, :3960,
  :7904`) : factures 100.00 / 250.00 / 108.10 au montant exact ou sans facture, 600 sur 1000 réglée en partie,
  `proposed_couples_287` (écarts de 10 CHF) — aucune n'attend la proposition d'une facture au reste inférieur au
  paiement : sens inchangé.
- **AC10** : un seul test change de sens, `ReconciliationProposals.test.ts:179` (zéro case attendue sur une ligne sans
  proposition) — réécrit par l'AC10. Les autres usages de `tx-checkbox` (`ReconciliationProposals.test.ts:225-699`,
  `reconciliation-deleted-import.spec.ts:153-154`) cochent des lignes **avec** proposition : inchangés. Les tests qui
  montent une ligne sans proposition (`:135`, `:246`, `:601`) ne comptent pas les cases : inchangés. **Le commentaire**
  de `reconciliation-deleted-import.spec.ts:12-14` (la règle « qui rend les deux transactions sélectionnables »)
  devient faux : réécrit, code du spec inchangé (AC10, C-29-6-19 ; validation P2, R4 — le second grep de § *Propagation*
  ne cherchait pas `sélectionnable` seul).

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
| `frontend/…/ReconciliationProposals.svelte` (AC10) | aucune (C-29-2-6) | en-tête `:8`, indication de priorité **avant la table** dans la branche « liste non vide », rendue si une ligne a une candidate | ? | case `:318-330`, `disabled` d'*Accepter* `:284`, garde et commentaire d'`onAccept` `:192-202`, indication **dans la rangée des boutons** (`:280`, C-29-6-18), en-tête `:6-12` — **même fichier que la 29-3** : en-tête **chevauchant** (la 29-6 réécrit `:6-12`, qui contient le `:8` de la 29-3 : réécrire la version 29-3), le reste voisin ; rebase à la main. **Ordre fixé** : rangée des boutons (indication 29-6 en dernier), paragraphe de priorité 29-3, table (AC10, point 6 ; validation P2, R9/F4/F9) |
| `ReconciliationProposals.test.ts`, `i18n-keys.test.ts`, `messages.ftl` ×4 | — | fabriques, tests, une clé, `sitesTotal` +1 | ? | test `:179` réécrit, tests neufs, une clé, `sitesTotal` +1 — **recompter** `sitesTotal` à la source après le rebase sur la 29-3 |
| Contournement « régler depuis la fiche, puis rejeter » | C-29-2-8 (reliquat d'une facture non proposée) : **garde son texte** et compte sur l'AC10 de la 29-6, mergée avant le tag v0.13.4 | — | — | le rend praticable (AC10) ; le manuel de la 29-2 n'est pas réécrit par la 29-6, **sauf** si la 29-2 a écrit à la FAQ `:2910` ou au `:2029` que le rejet sort la transaction de l'attente : au rebase, la réserve de [#670] s'y applique (C-29-6-17 ; validation P2, R2) |
| `docs/api-external.md` `:549-551` | phrase des dates | phrase `:549` | ? | phrase du trop-perçu |

La 29-5 n'a pas de fiche à la date de celle-ci : son périmètre (#668) est le départage à score égal, dans
`kesh-reconciliation` ; aucune zone de code commune attendue. À vérifier au rebase.

### Propagation — le symptôme à greper après chaque patch

La **valeur** et les formulations de la tolérance symétrique :

```sh
grep -rnE "à 5 centimes près|± ?0[.,]05|± ?5 centimes|amount_tolerance|AMOUNT_TOLERANCE|au plus 0\.005|#665|\b665\b|trop-perçu|dépasse de quelques centimes" \
  crates frontend/src docs/manual/fr/*.tex docs/api-external.md CHANGELOG.md
```

Et, pour l'AC10, le symptôme « une ligne sans proposition ne se sélectionne pas » sous toutes ses graphies :

```sh
grep -rnE "case à cocher|sélectionnable|ne peut pas être sélectionnée|sans checkbox|disabled checkbox|candidates\.length > 0|tx-checkbox" \
  frontend/src frontend/tests docs/manual/fr/*.tex docs/*.md website
```

(`sélectionnable` seul, et non plus `non-sélectionnable` : validation P2, R4 — la forme longue manquait le commentaire
de `reconciliation-deleted-import.spec.ts:12-14`.)

Et le symptôme de la validation P2 — l'indication ou le texte qui fait du rejet le seul geste, ou qui dit que le rejet
règle la transaction (R1/F2, R2) :

```sh
grep -rnE "ne peut qu|qu’être rejet|qu'être rejet|reject-only|kann nur abgelehnt|può solo essere|can only be rejected|ne sert qu|pour \*?Rejeter\*? seulement|écarte la proposition|sans quoi la transaction reste en attente|\b670\b" \
  frontend/src frontend/tests crates/kesh-i18n docs/manual/fr/*.tex docs/api-external.md CHANGELOG.md \
  _bmad-output/implementation-artifacts/29-6-*.md _bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-6.md
```

Les occurrences de la fiche et du registre qui **citent** la rédaction d'origine pour la corriger, et les relevés
datés (entrées de passe, rapports), sont légitimes.

Les occurrences légitimes (le CHANGELOG des versions publiées, **qui ne se réécrit pas** ; le trop-perçu du règlement
manuel ; l'arrondi à 5 centimes des factures émises) se trient à la main.

### Ce qui doit être préservé

- Le contrat de `POST /api/v1/reconciliation/accept` et de `GET /proposals` (forme, codes, ordre des refus).
- Le côté du paiement partiel : tolérance de 5 centimes, acceptation en `Ordinary`.
- L'absence de `ROUND` en SQL ; une seule définition de l'arrondi (`amount_due_to_centime`).
- Le filtre avant `ORDER BY … LIMIT 50` — gardé par le second test de l'AC5 et la mutation M8.
- *Accepter* n'envoie **jamais** une ligne sans proposition ; *Rejeter* envoie toutes les lignes cochées et n'est
  jamais grisé par une ligne sans proposition (AC10, M13).
- *Affecter manuellement* et *Éclater* sur toute ligne en attente, avec ou sans proposition (`.svelte:371-397`) ; aucun
  texte neuf ne les dit indisponibles (validation P2, R1/F2).

### Exclusions écrites (périmètre minimal)

- **Absorber un petit trop-perçu** (voie 2 de #665) : non fait (C-29-6-2) ; évolution possible sur demande de Guy, pas
  d'issue ouverte d'office (ce n'est pas un défaut : le refus est juste, c'est la proposition qui ne l'était pas).
- **Le côté du paiement partiel** (reste supérieur au paiement de 1 à 5 centimes) : inchangé (C-29-6-4). Observation,
  non un défaut : le reliquat de 0,05 exactement ne se solde pas en nature « arrondi » (`write_off_invoice` exige
  moins de 5 centimes, `api-external.md:505`) mais en escompte ou frais bancaires — chemins existants.
- **Migration, index** : aucun. **Aucun changement d'API** : ni `…/reject`, ni `…/unreject`, ni `…/accept`.
- **Accepter une ligne sans proposition** (l'ouvrir sur l'affectation manuelle, par exemple) : non fait — *Affecter
  manuellement* et *Éclater* existent déjà sur chaque ligne (`.svelte:371-397`).
- **Sortir de l'attente une transaction rejetée** (la rapprocher d'un règlement déjà saisi, ou un statut « écartée »
  non compté par la clôture du compte) : non fait — [#670], bug P2 ouvert par l'orchestrateur à la validation P2 ;
  ni mécanique ni de ce périmètre (C-29-6-17).

### Modules touchés (règle de découpage : au plus cinq)

Recompté à la validation P1, après l'extension de l'AC10 (C-29-6-12). **Granularité déclarée à la validation P2**
*(R8/F8 : la liste mêlait le crate et le module métier)* : **le crate Rust et le paquet npm** — les deux premières
unités que nomme la règle (« crates Rust, packages npm ») —, la documentation comptée à part par prudence bien
qu'elle ne soit pas un module au sens de la règle :

1. `kesh-db` (crate) — `repositories/invoice_settlements.rs`, `repositories/reconciliation.rs` (+ `tests/reconciliation_repository.rs`) ;
2. `kesh-api` (crate) — `routes/reconciliation.rs` (commentaires) (+ `tests/reconciliation_e2e.rs`) ;
3. `frontend` (paquet npm) — `src/lib/features/reconciliation/ReconciliationProposals.svelte` (+ `.test.ts`),
   `src/lib/shared/i18n-keys.test.ts` (compteur seul), `tests/e2e/` (spec neuf, commentaire d'un spec voisin) ;
4. `kesh-i18n` (crate) — les quatre `messages.ftl` (une clé) ;
5. documentation — manuel FR (+ PDF), `docs/api-external.md`, `CHANGELOG.md`.

À la granularité du **module métier** (celle des exemples de la règle, `kesh-api/routes/invoices`), on compterait six
ou sept unités (les deux dépôts de `kesh-db`, `frontend/…/reconciliation`, `frontend/…/shared`, `frontend/tests/e2e`…) ;
si une passe tient cette granularité pour la bonne, la fiche écrit alors une **dérogation** sur l'argument de
C-29-6-12 (aucune moitié ne se livre vraie seule), qui ne dépend pas du compte.

**Cinq à la granularité déclarée : la limite, non dépassée** (« plus de 5 » découpe) — pas de découpage. Le découpage en 29-6a (écran) / 29-6b
(proposition) a été pesé et écarté (C-29-6-12) : les deux moitiés sont indépendantes au code, mais le texte du manuel
et du CHANGELOG de l'une prescrit le geste de l'autre, si bien qu'aucune ne se livre vraie seule ; et l'AC10 tient en
un composant, sans API. ⚠️ Toute extension de plus fait dépasser la règle : elle découpe, ou s'écrit en dérogation.

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
  `crates/kesh-api/src/routes/reconciliation.rs:69-72`, `:586-662`, `:1808-1817`, `:1915-1985` ;
  `crates/kesh-db/src/repositories/invoice_settlements_write.rs:215-245` ; `crates/kesh-core/src/types/money.rs:66-71`.
- `docs/manual/fr/user-manual.tex:971`, `:1308`, `:2025`, `:2029`, `:2049`, `:2050`, `:2057`, `:2062`, `:2330`,
  `:2901`, `:2910-2911`, `:1690-1694` ; `docs/api-external.md:505`, `:551`.
- AC10 : `frontend/src/lib/features/reconciliation/ReconciliationProposals.svelte:6-12`, `:192-254`, `:280-330`,
  `:452-460` ; `ReconciliationProposals.test.ts:108-123`, `:179-197`, `:661-678` ;
  `frontend/tests/e2e/reconciliation-unreject.spec.ts` (patron) ; `crates/kesh-api/src/routes/reconciliation.rs:851-901`,
  `:3205-3550`, `:4759-4782` ; `crates/kesh-api/tests/reconciliation_e2e.rs:1560` ; `frontend/src/lib/shared/i18n-keys.test.ts:634`.
- Validation P2 : `crates/kesh-api/src/routes/reconciliation.rs:640-652`, `:3256-3266` ;
  `crates/kesh-db/src/repositories/bank_accounts.rs:1025` ; `frontend/src/lib/features/bank-import/DeleteBankImportAction.test.ts:191-194` ;
  `frontend/src/lib/shared/e2e-selecteurs-traduits.test.ts` ; `frontend/tests/e2e/reconciliation-deleted-import.spec.ts:12-14` ;
  issue [#670].
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
- 2026-10-14 — **Validation P1** (Sonnet 5.5, lentilles R et F, lecture seule ; rapports
  `kesh-gate-logs/29-6-validate-p1-{R,F}.md`) : **R 0 CRITICAL · 0 HIGH · 1 MEDIUM · 8 LOW ; F 0 · 0 · 0 · 1 LOW** —
  au total 0/0/1/9. Remédiée (Opus 5.5), **dix findings traités, aucun refusé** :
  - **R1 MEDIUM** — le rejet prescrit après le règlement depuis la fiche était impraticable : une ligne « Aucune
    correspondance » n'a pas de case. **Décision de l'orchestrateur**, vérifiée par lui au code
    (`ReconciliationProposals.svelte:318-330`, `onReject`) : **le périmètre s'étend à l'écran**. Établi au code avant
    d'écrire : la route de rejet ne lit aucune candidate (refus en `failed[]`, Pattern batch), un test existant rejette
    déjà des transactions sans facture, *Annuler le rejet* (28-5b) non plus ne lit de candidate — aucune API ne change.
    AC10 neuf (case sur toute ligne, *Accepter* grisé et gardé, indication, une clé en quatre locales, `sitesTotal` +1,
    tests Vitest dont le `:179` réécrit, spec E2E neuf et captures clair et sombre) ; AC3, AC7 (`:2029`, `:2050`,
    CHANGELOG), AC8 (M9 à M11), AC9, T1, T3bis, T4, inventaires (a) et (c), zones partagées (29-3 sur le même
    composant ; la 29-2 garde son texte C-29-2-8 et renvoie à l'AC10), propagation (second grep), exclusions, modules
    mis à jour. Choix C-29-6-10, 11, 12, 15 ; C-29-6-6 et C-29-6-7 annotés.
  - **R2** contrôles PDF positifs (portions courtes, ligatures normalisées) avant les négatifs ; **R3** inventaire (b)
    borné au réglage d'arrondi actif ; **R4** second test de dépôt (50 trop-perçus puis une valide) et mutation M8,
    C-29-6-13 ; **R5** relevé de l'inventaire (c) inscrit (trois fichiers de `crates/*/tests`) ; **R6** prémisse 3 :
    le test cité établit le refus, la proposition se montre par le calcul — et « faux depuis l'origine » devient
    « faux depuis son écriture » (`eecf40cb` ; tolérance `22fd6d4a`), ce qui corrige aussi le renvoi « réécrit par
    l'AC1 » en AC2 ; l'entrée de création ci-dessus garde sa formulation, relevé daté ; **R7** v0.13.4 = 29-2 à 29-6
    (fichier d'epic § *Releases*, commentaire d'`epic-29`), C-29-6-14 ; **R8** phrase d'`api-external.md` : « de plus
    d'un demi-centime » ; **R9** `money.rs:66-71` ; **F1** `HALF_CENTIME` par `Decimal::from_parts` seule, précédent
    `invoice_settlements_write.rs:406`.
  - Symptômes grepés après correction : `aucun écran|aucune clé|aucune capture|non-sélectionnable|n'a pas de case|depuis
    l'origine|29-2, 29-3, 29-4 et 29-6|29-2..4|money.rs:61-69` sur la fiche, le registre, le fichier d'epic et
    `sprint-status.yaml` ; restent les relevés datés (entrée de création, rapports P1) et les citations du texte
    d'origine que la fiche réécrit.
  - Recomptes (fiche) : **10 AC**, **8 tâches** (T0 à T6 et T3bis), **11 mutations** (M1 à M11), **15 choix**,
    **5 modules** (limite, non dépassée, C-29-6-12).
  - Prompt de la validation P2 : `29-6-validate-prompt-p2.md` (Opus, lentilles R et F, axe obligatoire : relire cette
    remédiation, qui étend le périmètre à l'écran).
- 2026-10-14 — **Validation P2** (Opus 5.5, lentilles R et F, lecture seule ; rapports
  `kesh-gate-logs/29-6-validate-p2-{R,F}.md`) : **R 0 CRITICAL · 0 HIGH · 2 MEDIUM · 9 LOW ; F 0 · 0 · 2 MEDIUM ·
  7 LOW** — au total 0/0/4/16, **vingt findings**, dont des convergences (R1 = F2 ; R8 = F8 ; R9 rejoint F4 sur
  l'ordre des indications ; R3 rejoint F3 sur `:2057`). Trois des quatre MEDIUM viennent de la **remédiation P1**
  (AC10) ; R2 de la conception d'origine, gardée par elle. Remédiée (Opus 5.5) : **vingt traités, aucun refusé** —
  une seule correction proposée écartée en faveur d'une autre (R1 (b), version longue de l'indication, C-29-6-16).
  - **R1 = F2 MEDIUM** — l'indication « ne peut qu'être rejetée » (et ses trois traductions) était fausse : la ligne
    porte *Affecter manuellement* et *Éclater*, et les lignes sans proposition sont aussi les débits et le non-CHF sans
    règle (`routes/reconciliation.rs:640-652`). Indication « ne peut pas être acceptée » dans les quatre locales, clé
    `reconciliation-labels-unmatched-not-acceptable-hint`, `data-testid` `reconciliation-unmatched-not-acceptable-hint`
    (M11 renommée) ; périmètre réel écrit à l'AC10 (*Établi au code*) ; textes prescrits du manuel (`:2050`, `:2057`,
    FAQ `:2910`), du CHANGELOG et du fichier d'epic (annoté) corrigés ; C-29-6-10 et C-29-6-11 annotés, C-29-6-16.
  - **R2 MEDIUM** — une transaction rejetée reste `pending` et bloque la clôture du compte bancaire ; aucun geste ne la
    rapproche d'un règlement saisi depuis la fiche. **[#670]** (bug P2, ouverte par l'orchestrateur) : écrit comme
    **angle mort assumé** (statut, AC10, inventaire (b) — tableau neuf —, exclusions, zones partagées pour la 29-2) ;
    les textes prescrits (`:2029`, `:2057`, `:2910`, CHANGELOG) ne disent plus que le rejet règle la transaction ;
    contrôles PDF négatifs ajoutés. C-29-6-17. La décision de l'auteur sur l'intégration de #670 à l'epic reste
    attendue ; la fiche n'en dépend pas.
  - **F1 MEDIUM** — `fireEvent.click` atteint le gestionnaire d'un bouton désactivé (jsdom, délégation Svelte 5 ;
    précédent `DeleteBankImportAction.test.ts:191-194`) : M10 n'est tuée que par l'attribut `disabled` ; la garde de
    tête d'`onAccept` est **éprouvable** — test neuf (`fireEvent.click` sur *Accepter* grisé : ni erreur, ni appel, la
    table reste) et **M12** ; le test de *Rejeter* asserte `disabled === false` — **M13**. Le paragraphe « aucun test
    ne la tue » est remplacé par l'encadré de l'AC8 ; seul le `throw` reste non éprouvé.
  - **LOW** : **R3/F3** paragraphe *Rejeter* `:2057` inventorié (« écarte la transaction », sans écriture), aucun
    test débiteur requis (le composant ne lit pas le signe) ; **R4** commentaire de
    `reconciliation-deleted-import.spec.ts:12-14` inventorié, `sélectionnable` seul au second grep, C-29-6-19 ;
    **R5** garde `e2e-selecteurs-traduits.test.ts` nommée, sélecteurs `getByTestId` + `toContainText` ; **R6**
    `s/’/'/g` à la normalisation du PDF ; **R7** FAQ `:2910` inventoriée ; **R8 = F8** granularité des modules
    déclarée (crate / paquet npm : cinq ; au module métier, six ou sept, dérogation à écrire si cette granularité est
    tenue) ; **R9/F4** indication dans la rangée des boutons, ordre des indications 29-3/29-6 fixé (AC10 points 4 et
    6), C-29-6-18 ; **R10** commentaire du kickoff d'`epic-29` rétabli (« 29-2..4 ») puis annoté, non réécrit ;
    **R11** pré-vol du compte clos (`:3256-3266`) cité ; **F5** états de la sélection écrits ; **F6** site de M8
    (`find_unpaid_invoices_for_window`, après `fetch_all`) ; **F7** `:1915-1944` / `:1915-1922` ; **F9** en-tête du
    composant chevauchant, réécrit dans sa version 29-3.
  - Symptômes grepés après correction (troisième grep de § *Propagation*, et `sous les boutons|inatteignable|aucun
    test ne la tue|1919|11 mutations|C-29-6-1 à 15`) sur la fiche, le registre, le fichier d'epic et
    `sprint-status.yaml` : restent les citations de la rédaction d'origine que la fiche corrige, les entrées datées
    (Change Log P1, titre de C-29-6-10, texte retenu de C-29-6-11, annotés) et la ligne P1 de `sprint-status.yaml`.
  - Recomptes (fiche) : **10 AC**, **8 tâches** (T0 à T6 et T3bis), **13 mutations** (M1 à M13), **19 choix**
    (C-29-6-1 à C-29-6-19), **5 modules** à la granularité déclarée.
  - Prompt de la passe ciblée P3 : `29-6-validate-prompt-p3-ciblee.md` (Haiku, une lentille, le seul diff de cette
    remédiation).

[#670]: https://github.com/guycorbaz/kesh/issues/670
- 2026-10-14 — **Validation P3, passe ciblée** (Haiku 5.5, une lentille sur le diff `2410c8a2..e20a418e` ; rapport
  `kesh-gate-logs/29-6-validate-p3-ciblee.md`) : 0/0/0/4. **Validation close.** Les quatre LOW sont à appliquer à
  l'étape 0 du développement : P3-1 (`reconciliation.rs:3257-3267` et `:4910` au lieu de `:3256-3266`) ; P3-2
  (« le blocage est donc permanent » → « sans geste juste pour un règlement déjà saisi », et nommer le chemin *Annuler le
  rejet* puis *Affecter manuellement*, qui lève le blocage au prix d'une écriture fausse) ; P3-3 (#670 n'attend plus de
  décision : elle est la **Story 29-8**, décision de Guy du 2026-10-14 — annotation datée, entrée P2 non réécrite) ;
  P3-4 (Change Log P2 : dix-neuf findings traités, un — F3 — déclaré non bloquant ; annotation datée). Trend : P1
  0/0/1/9 → P2 0/0/4/16 → P3 ciblée 0/0/0/4.
