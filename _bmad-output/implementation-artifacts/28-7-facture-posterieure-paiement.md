# Story 28.7 : Une facture postérieure au paiement n'est plus proposée

## Status

**ready-for-dev** — fiche créée le 2026-10-13 en autonomie (`bmad-create-story`, Opus 5.5), sur `5aad8b35`
(kickoff de l'Epic 28), branche `story/28-7`, worktree `/home/gcorbaz/devel/kesh-28-7`. Validation P1 faite
(Sonnet, lentilles R et F : 0 CRITICAL, 0 HIGH, 5 MEDIUM soit 4 distincts, 12 LOW soit 10 distincts — remédiés,
cf. Change Log) ;
validation P2 à lancer (`28-7-validate-prompt-p2.md`, Opus, lentilles R et F). Choix consignés au registre
`epic-28-choix-autonomes.md`, **C-28-7-1 à C-28-7-10**, dont **C-28-7-4 retiré** en P1 (neuf choix en vigueur).
11 AC. PR : **`closes #548`**.

⚠️ **Sérialisée derrière la 28-5 et la 28-6**, qui touchent aussi `crates/kesh-api/src/routes/reconciliation.rs`
(fichier d'epic, § *Ordre*). Le développement **commence par un rebase** sur `origin/main` une fois la 28-6 mergée,
et **relit** les numéros de ligne de cette fiche (ils sont relevés sur `5aad8b35`).

## Story

**As a** utilisateur qui rapproche ses relevés (Comptable ou Admin),
**I want** que la réconciliation ne me propose que des factures que l'acceptation admettra,
**so that** une proposition cochée et acceptée ne revienne pas en échec avec « Le paiement est daté de plus d'un
jour avant la facture ».

Issue : [#548](https://github.com/guycorbaz/kesh/issues/548) (P3, relevée par la revue de code de la 15-5c,
finding E1). Epic : `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § *28-7*. Cible : **v0.13.3**.

### Le défaut, au sol (`5aad8b35`)

Deux définitions de la fenêtre de dates, qui ne coïncident pas :

| | Date de référence | Fenêtre de la date de facture `F` | Site |
|---|---|---|---|
| **Proposition** | `booking_date` (`B`) | `B − 30 ≤ F ≤ B + 30` | `crates/kesh-db/src/repositories/reconciliation.rs:135` (SQL), appelée par `crates/kesh-api/src/routes/reconciliation.rs:649-656` avec `tx.booking_date` et `WINDOW_DAYS` (`:64`) |
| **Acceptation** | `value_date`, à défaut `booking_date` (`D`) | `D ≥ F − 1` **et** `D ≤ F + 30`, soit `D − 30 ≤ F ≤ D + 1` | `crates/kesh-api/src/routes/reconciliation.rs:1656-1682` (`accept_one_invoice`, étape 6) |

Une facture datée de **2 à 30 jours après** la transaction est proposée, puis refusée en
`RECONCILIATION_INVOICE_NOT_ELIGIBLE` / `payment_date_before_invoice_date`. Les deux définitions diffèrent aussi
par leur **date de référence** : quand la date de valeur précède la date de comptabilisation, une facture datée du
jour de comptabilisation est proposée et refusée ; quand elle la suit, une facture ancienne de `B − 30` est
proposée et refusée (`payment_date_outside_window`).

### La règle retenue (C-28-7-1)

**L'acceptation fait foi ; la proposition s'y aligne.** Une facture peut être réglée par une transaction si, `D`
étant la date de valeur de la transaction (à défaut sa date de comptabilisation) et `F` la date de la facture :

> `D − 30 jours ≤ F ≤ D + 1 jour`

c'est-à-dire : le paiement n'est **pas antérieur de plus d'un jour** à la facture, et pas **postérieur de plus de
30 jours**. La règle d'acceptation est **inchangée** (mêmes bornes, mêmes raisons, mêmes `details`) ; seule la
proposition change. Pourquoi ce sens et pas l'inverse :

- **Élargir l'acceptation** (admettre un paiement antérieur à sa facture) changerait ce que Kesh **écrit** au
  journal — un règlement daté avant la créance —, avec un risque P1 que #548 (P3) ne justifie pas, romprait
  l'accord avec le règlement depuis la fiche de la facture (`settle_invoice` et `write_off_invoice`,
  `invoice_settlements_write.rs`), qui refusent la même chose, et se heurterait à la base elle-même : la CHECK
  `chk_invoices_paid_at_after_date` (`DATE(paid_at) >= date - INTERVAL 1 DAY`, migration
  `20260419000001_invoice_paid_at.sql:32-33`) porte la même borne et ne se desserre que par une migration.
- **Un acompte** (paiement reçu avant l'émission de la facture) n'est pas le règlement de cette facture : Kesh n'a
  pas de modèle d'acompte, et le manuel le dit déjà (`user-manual.tex:1921`, « un acompte, par exemple ---, il ne
  règle pas cette facture »). Il se rapproche à la main (*Affecter manuellement*) — hors périmètre.
- **Une facture antidatée** (datée avant son émission réelle) est *plus ancienne* que le paiement : elle reste
  dans la fenêtre tant qu'elle a moins de 30 jours. Une facture **postdatée** au-delà du lendemain du paiement
  n'est plus proposée ; c'est la conséquence voulue, et le remède est de corriger sa date (dévalider, modifier,
  revalider) — même refus qu'aujourd'hui à l'acceptation et au règlement manuel.
- **La tolérance d'un jour** est conservée : `settled_on`/`value_date` est une date bancaire, `invoice.date` une
  date métier locale ; l'écart de fuseau suffit à faire paraître un paiement « la veille »
  (`invoice_settlements_write.rs:108-114`).

### Effet sur #529 (paiement reçu plus de 30 jours après la facture) — non aggravé (C-28-7-6)

#529 (P2) **n'est pas absorbé**. La correction ne l'aggrave pas, et cela se démontre :

- ancienne proposition ∩ acceptation = `[B−30, B+30] ∩ [D−30, D+1]` ⊆ `[D−30, D+1]` = nouvelle proposition (pour
  le critère de date) : **aucune facture acceptable n'est retirée des propositions** ;
- ce qui sort des propositions est exactement ce que l'acceptation refusait — dont, quand `D > B`, les factures
  de `[B−30, D−30[` (paiement tardif de plus de 30 jours selon la date de valeur) : elles ne pouvaient pas
  aboutir ;
- quand `D < B`, la borne basse recule à `D − 30` : des factures acceptables, jusqu'ici absentes, sont proposées.

La borne de 30 jours après la facture (le cœur de #529) reste exactement où elle est. Le test de parité de l'AC4
porte un cas à `F = D − 30` (proposée et acceptée) et à `F = D − 31` (ni l'une ni l'autre), qui fige ce point.

## Critères d'acceptation

**AC1 — Une seule définition de la fenêtre (C-28-7-2, C-28-7-3).**

Dans `crates/kesh-db/src/repositories/reconciliation.rs`, une fonction pure (aucun I/O) et ses constantes, seule
définition de la fenêtre dans le dépôt :

- `pub const INVOICE_WINDOW_DAYS_AFTER_INVOICE: i64 = 30;` — un paiement au plus 30 jours après la facture ;
- `pub const PAYMENT_BEFORE_INVOICE_TOLERANCE_DAYS: i64 = 1;` — au plus un jour avant ;
- `pub fn payment_date(tx: &BankTransaction) -> NaiveDate` — `tx.value_date.unwrap_or(tx.booking_date)` ;
- `pub struct InvoiceDateWindow { pub earliest_invoice_date: NaiveDate, pub latest_invoice_date: NaiveDate }`
  (`Debug, Clone, Copy, PartialEq, Eq`), avec :
  - `pub fn for_payment_date(d: NaiveDate) -> Self` — `earliest = d − 30`, `latest = d + 1` ;
  - `pub fn for_transaction(tx: &BankTransaction) -> Self` — `for_payment_date(payment_date(tx))` ;
  - `pub fn check(&self, invoice_date: NaiveDate) -> Result<(), InvoiceDateRefusal>` —
    `invoice_date > latest` → `PaymentBeforeInvoice` ; `invoice_date < earliest` → `PaymentTooLate` ; sinon `Ok`.
- `pub enum InvoiceDateRefusal { PaymentBeforeInvoice, PaymentTooLate }` (`Debug, Clone, Copy, PartialEq, Eq`).

Arithmétique : l'arithmétique de dates ordinaire de chrono (`d - Days::new(30)`, `d + Days::new(1)`, ou
`Duration::days` comme aujourd'hui), **sans aucun ramènement de borne** — `check` compare donc exactement ce que
compare l'acceptation actuelle, à toute date. Une date lue d'une colonne `DATE` ne peut pas faire déborder
`NaiveDate` (chrono 0.4.45, `Cargo.lock`). *(Le ramènement à la plage `DATE` prescrit à la création, C-28-7-4,
est retiré en validation P1 : non nécessaire à #548, non mesuré, et il écartait `check` de l'acceptation aux
extrémités.)* Doc-comment de module et de fonction : la règle, le pourquoi de chaque borne (C-28-7-1), la mention
que la proposition **et** l'acceptation la lisent, et l'inventaire des sites non résolus (AC9).

**AC2 — La proposition lit la fenêtre (C-28-7-2).**

`find_unpaid_invoices_for_window` prend une `InvoiceDateWindow` à la place de `tx_date` et `window_days` :

```rust
pub async fn find_unpaid_invoices_for_window<'e, E>(
    executor: E,
    company_id: i64,
    window: InvoiceDateWindow,
    tx_amount: Decimal,
    amount_tolerance: Decimal,
) -> Result<Vec<UnpaidInvoiceCandidate>, DbError>
```

Le SQL devient `AND i.date BETWEEN ? AND ?`, lié à `window.earliest_invoice_date` puis
`window.latest_invoice_date` ; `DATE_SUB` / `DATE_ADD` disparaissent de la requête. Les filtres, le `HAVING` et
les jointures du reste dû sont **inchangés**. Le **`LIMIT 50` est inchangé, désormais déterministe** : la requête
gagne `ORDER BY i.date ASC, i.id ASC` entre le `HAVING` et le `LIMIT` (C-28-7-10) — la plus ancienne facture
d'abord, ce qui est l'ordre de fait que livre l'index `idx_invoices_company_validated_unpaid_date (company_id,
status, paid_at, date)` puis la clé primaire, et le départage `invoice_id ASC` du tri aval
(`kesh-reconciliation/src/matching.rs:104-110`). Au-delà de 50 candidates, ce sont donc les plus récentes qui
sont écartées, et toujours les mêmes. Le doc-comment (point 3 des filtres, `:91`) est réécrit et dit l'ordre. `get_proposals` (`routes/reconciliation.rs:649`) passe
`InvoiceDateWindow::for_transaction(&tx)` ; la constante `WINDOW_DAYS` (`:62-64`) **disparaît** de
`routes/reconciliation.rs`. Aucun appelant ne peut plus choisir sa propre fenêtre.

**AC3 — L'acceptation lit la même fenêtre, sans changer de comportement.**

Dans `accept_one_invoice`, les deux `if` de dates (`:1660-1682`) sont remplacés par
`InvoiceDateWindow::for_transaction(&bank_transaction).check(invoice.date)` :

- `PaymentBeforeInvoice` → `RECONCILIATION_INVOICE_NOT_ELIGIBLE`, `details = { "reason": "payment_date_before_invoice_date" }` ;
- `PaymentTooLate` → `RECONCILIATION_INVOICE_NOT_ELIGIBLE`, `details = { "reason": "payment_date_outside_window", "window_days": INVOICE_WINDOW_DAYS_AFTER_INVOICE }`.

Codes, raisons, clés et valeurs de `details` **identiques à l'existant** (contrat d'API inchangé — les tests
`post_accept_rejects_payment_date_before_invoice_date` et `post_accept_rejects_payment_date_outside_window`,
`reconciliation_e2e.rs:2397` et `:2725`, passent **sans modification**). La liaison locale `paid_at_candidate`
(`:1656-1658`) **reste**, réduite à `let paid_at_candidate = payment_date(&bank_transaction);` — même valeur. Elle
a huit sites au sol (`grep -n paid_at_candidate` : définition `:1656`, lectures `:1660`, `:1673`, `:1740`
`paid_at_dt`, `:1911` et `:1917` exercice ouvert couvrant la date, `:1961` `entry_date`, `:1994` `settled_on`) ;
les lectures `:1660` et `:1673` disparaissent avec les deux `if`, les cinq autres sont **inchangées**. Les deux
seuls emplois de `Duration` (`:1660`, `:1673`) disparaissant aussi, l'import `use chrono::{Duration, NaiveDate};`
(`:30`) devient `use chrono::NaiveDate;` — sans quoi `clippy -D warnings` rougit (`unused import`), sauf si
`Duration` est encore employé au rebase (le revérifier au `grep -nw Duration`).
L'ordre des contrôles de l'étape 6 est conservé (statut, payée, écriture, puis dates).

**AC4 — Parité proposition ⇔ acceptation, aux bords (test d'intégration HTTP).**

Nouveau test `#[sqlx::test(migrations = "../kesh-db/test-schema")]` dans `crates/kesh-api/tests/reconciliation_e2e.rs`,
`proposal_and_acceptance_share_the_invoice_date_window` : pour chaque décalage `k` de
`{ −31, −30, −1, 0, +1, +2, +30 }`, une transaction créditrice au jour `D` (`value_date = booking_date = D`) et une
facture validée datée `D + k`, d'un **montant propre à `k`**, distant des autres de **plus de 0.05 CHF** (la
tolérance : aucune facture n'est candidate d'une autre transaction) et d'une **référence propre à `k`**, reconnue.
`D` est **en plein exercice 2026** — par exemple `D = 2026-06-15`, de sorte que `D − 31` et `D + 30` restent en
2026 : `seed_validated_invoice` n'ouvre que l'exercice 2026 (`insert_fake_fiscal_year`,
`reconciliation_e2e.rs:279`), et l'acceptation exige un exercice ouvert couvrant la date du paiement
(`find_open_covering_date`, `routes/reconciliation.rs:1911`). Puis :

1. `GET /api/v1/reconciliation/proposals` → l'ensemble `P` des couples (transaction, facture) proposés ;
2. `POST /api/v1/reconciliation/accept` avec **les sept couples** → l'ensemble `A` des acceptés ;
3. assertions : `P == A` (même ensemble de couples), et `P` contient exactement les décalages
   `{ −30, −1, 0, +1 }` ; les décalages `+2` et `+30` sont refusés en `payment_date_before_invoice_date`,
   `−31` en `payment_date_outside_window`.

Le cas `+2` est **le cas de #548** (proposée puis refusée avant la story) ; `+30` est le pire cas de l'ancienne
fenêtre ; `−30`/`−31` figent la borne de #529 (AC « non aggravé »).

**AC5 — La date de référence est la date de valeur (test d'intégration HTTP).**

Nouveau test `proposals_use_the_value_date_like_the_acceptance` dans le même fichier. Une transaction et une
facture par cas, montants distincts de plus de 0.05 CHF, références distinctes ; `B` = date de comptabilisation,
`D` = date de valeur ; `B` en plein exercice 2026 (ex. `B = 2026-06-15`, de sorte que `B − 35` et `B + 6` restent
en 2026, même raison qu'à l'AC4) :

| Cas | `booking_date` | `value_date` | Facture datée | Fenêtre `[D−30, D+1]` | Attendu (après) | Avant la story |
|---|---|---|---|---|---|---|
| (a) | `B` | `B − 5` | `B` | `[B−35, B−4]` | **non proposée** ; acceptation forcée refusée `payment_date_before_invoice_date` | proposée, puis refusée |
| (b) | `B` | `B − 5` | `B − 34` | `[B−35, B−4]` | **proposée et acceptée** | non proposée (`B−34 < B−30`) bien qu'acceptable |
| (c) | `B` | `B + 5` | `B − 28` | `[B−25, B+6]` | **non proposée** ; acceptation forcée refusée `payment_date_outside_window` | proposée, puis refusée |
| (d) | `B` | absente | `B + 1` | `[B−30, B+1]` | proposée et acceptée | proposée et acceptée |
| (e) | `B` | absente | `B + 2` | `[B−30, B+1]` | **non proposée** | proposée, puis refusée |

« Acceptation forcée » : `POST /accept` du couple, bien qu'absent des propositions (le serveur recalcule tout).
Le cas (b) est celui qui montre qu'aucune facture acceptable n'est perdue — il en **gagne** une. Il asserte en
outre la **date du règlement écrit** : `invoice_settlements.settled_on == B − 5` (la date de valeur, distincte de
la date de comptabilisation) — sans quoi rien ne fixe que le règlement est daté par `payment_date` (les
assertions existantes sur `paid_at` n'en vérifient que la présence, `.is_some()` / `.is_none()`).

**AC6 — Tests unitaires de la fenêtre (purs, sans base).**

Module `#[cfg(test)] mod tests` de `repositories/reconciliation.rs` (il n'en a pas : à créer) :

- `for_payment_date` : `earliest = D − 30`, `latest = D + 1` ;
- `check` : `D + 1` → `Ok` ; `D + 2` → `PaymentBeforeInvoice` ; `D` → `Ok` ; `D − 1` → `Ok` ; `D − 30` → `Ok` ;
  `D − 31` → `PaymentTooLate` ;
- `payment_date` / `for_transaction` : `value_date` prioritaire ; `booking_date` à défaut.

Le test de dépôt `find_unpaid_invoices_filters_by_30_day_window` (`crates/kesh-db/tests/reconciliation_repository.rs:258`)
est **réécrit** sur la nouvelle signature avec des factures à `D − 31`, `D − 30`, `D + 1`, `D + 2` : seules
`D − 30` et `D + 1` reviennent. Les dix autres appels de tests (`reconciliation_repository.rs` : 370, 385, 462,
516, 793, 814, 898, 916 ; `invoice_write_off.rs` : 785, 802) sont **adaptés** à la signature
(`InvoiceDateWindow::for_payment_date(<leur date>)`) sans changer ce qu'ils prouvent — chacun pose une facture
datée **au plus** au jour de la transaction (vérifié au `sed -n`, § *Inventaire (c)*) : leur résultat ne change pas.

Nouveau test de dépôt `find_unpaid_invoices_orders_by_date_then_id_before_limit`
(`crates/kesh-db/tests/reconciliation_repository.rs`, C-28-7-10) : **52** factures validées de **même reste dû**,
datées sur `[D − 30, D + 1]` (plusieurs par jour), **insérées dans l'ordre des dates décroissantes** pour que
`id ASC` et `date ASC` divergent. Assertions : exactement 50 candidates ; ce sont les 50 premières de `(date, id)`
croissants, **dans cet ordre** ; les deux dernières de `(date, id)` n'y figurent pas.

**AC7 — Le libellé de refus ne parle plus d'une facture « proposée » (C-28-7-5).**

La clé `reconciliation-failed-payment-before-invoice` reste (la raison reste atteignable : facture redatée entre
l'affichage et l'acceptation, intégration par clé d'API) ; son texte perd la proposition finale, qui décrivait #548 :

| Locale | Fichier : ligne | Texte nouveau |
|---|---|---|
| fr-CH | `crates/kesh-i18n/locales/fr-CH/messages.ftl:1233` | `Le paiement est daté de plus d’un jour avant la facture : il ne peut pas la régler.` |
| de-CH | `…/de-CH/messages.ftl:1170` | `Die Zahlung liegt mehr als einen Tag vor dem Rechnungsdatum: Sie kann die Rechnung nicht begleichen.` |
| en-CH | `…/en-CH/messages.ftl:1170` | `The payment is dated more than one day before the invoice: it cannot settle this invoice.` |
| it-CH | `…/it-CH/messages.ftl:1170` | `Il pagamento è anteriore di più di un giorno alla data della fattura: non può saldarla.` |

Le repli de `frontend/src/lib/features/reconciliation/failed-proposal-label.ts:224` prend le texte fr-CH à
l'identique, apostrophe typographique `’` comprise — la garde **G13**
(`frontend/src/lib/shared/i18n-repli-divergent-actif.test.ts`) exige l'égalité du repli à site unique et de la
valeur fr-CH du catalogue, et rougirait sur un seul caractère d'écart. Le doc-comment du même fichier (`:30-41`, « la seule qu'un utilisateur rencontre sans avoir rien
fait de travers : la proposition retient […] (#548) ») et le commentaire du test
(`failed-proposal-label.test.ts:170-171`) sont réécrits : depuis la 28-7, la proposition et l'acceptation
partagent la fenêtre ; la raison ne se rencontre plus que par une facture redatée ou un appel d'API. L'assertion
`toContain('plus d’un jour avant la facture')` (`:177`) reste vraie. Aucune clé ajoutée ni retirée : les
compteurs de `i18n-keys.test.ts` (`sitesTotal`, etc.) sont **inchangés** — à vérifier par `grep -c "i18nMsg("`
aux deux bornes, non à supposer.

**AC8 — Le manuel décrit la fenêtre commune (PDF compris).**

`docs/manual/fr/user-manual.tex` (seul manuel rédigé : `docs/manual/{de,en,it}/` ne portent qu'un `README.md`) :

- **`:1919`** (§ *Algorithme de matching*) : « si la transaction tombe dans les 30 jours qui précèdent ou suivent la
  date de la facture » devient la règle de l'AC1 — le paiement (date de valeur, à défaut de comptabilisation) au
  plus **30 jours après** la date de la facture et au plus **un jour avant**. La phrase sur le paiement reçu plus de
  30 jours après (#529) reste.
- **`:1921`** (paragraphe `\label{sec:facture-posterieure}`, « Une facture proposée peut être refusée à
  l'acceptation ») : réécrit — la proposition applique la même fenêtre que l'acceptation ; une facture datée de
  plus d'un jour après le paiement **n'est plus proposée** (depuis la v0.13.3) ; le refus ne se voit plus que si
  la facture a été redatée entre-temps ; le règlement depuis la fiche applique la même tolérance d'un jour avant
  la facture (et aucune borne de 30 jours après, ce qui reste le contournement de #529) ; l'acompte comme
  aujourd'hui. **Le `\label` est conservé** (référencé à `:2787`) ; le titre du paragraphe change (ex. « Une
  facture datée après le paiement n'est pas proposée. »).
- **`:2787`** (FAQ « aucun match automatique ») : « À l'inverse, une facture datée de plus d'un jour après le
  paiement peut être proposée, mais elle sera refusée » devient « une facture datée de plus d'un jour après le
  paiement n'est pas proposée (\S\ref{sec:facture-posterieure}) ».
- `:1299` (rappels, « 30 jours de la date de la facture ») reste vrai : **non modifié**.

PDF régénéré (`make fr` dans `docs/manual/`, sous le verrou de gate) et commité ; contrôle **aplati** :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "peut être proposée[^.]*"` ne rend
plus rien, et la nouvelle phrase s'y lit. Gardes textuelles : `cargo nextest run -E 'binary(textes_coherents)'`
vert.

**AC9 — Inventaire clos des sites, et sites non résolus écrits (C-28-7-7).**

Le doc-comment de `InvoiceDateWindow` nomme ses **deux** lecteurs (proposition, acceptation) et les **trois**
sites qui portent la même borne basse **sans** la lire, avec la raison (Dev Notes § *Inventaire (b)*), **par leur
nom et non par un numéro de ligne** (un numéro de ligne pourrit au premier commit voisin) :

- `settle_invoice` et `write_off_invoice` (`crates/kesh-db/src/repositories/invoice_settlements_write.rs`, le
  règlement et le solde par escompte, perte, frais ou arrondi) — même tolérance d'un jour
  (`settled_on < invoice_date − 1`), sans borne haute, sur une date **saisie** et non une transaction. Ils ne
  changent pas (C-28-7-7) ;
- la CHECK `chk_invoices_paid_at_after_date` (migration `20260419000001_invoice_paid_at.sql`, copie dans
  `test-schema/0001_schema_squash.sql`) — `DATE(paid_at) >= date - INTERVAL 1 DAY` : la base double la tolérance
  d'un jour, qui **ne se desserre pas sans migration** (donc sans bump `min_required`, P2/P3).

Aucun autre site du dépôt ne choisit ni ne valide une facture pour une transaction (Dev Notes § *Inventaire (a)*,
greps copiés).

**AC10 — Mutations (chacune fait rougir au moins un test nommé).**

| # | Mutation | Test qui doit rougir |
|---|---|---|
| M1 | `latest = d + 2` (`for_payment_date`) | AC6 `check`, AC4 (décalage `+2` proposé et accepté) |
| M2 | `latest = d` | AC6, AC4 (`+1` ni proposé ni accepté) |
| M3 | `earliest = d − 29` | AC6, AC4 (`−30`) |
| M4 | `earliest = d − 31` | AC6, AC4 (`−31`) |
| M5 | `payment_date` rend `tx.booking_date` | AC6 (`value_date` prioritaire), AC5 (a), (b), (c) |
| M6 | `get_proposals` passe `InvoiceDateWindow::for_payment_date(tx.booking_date)` | AC5 (a), (b), (c) |
| M7 | `check` : `>` remplacé par `>=` sur `latest` | AC6 (`D + 1` → `Ok`), AC4 |
| M8 | les deux branches de refus échangées dans `accept_one_invoice` | `post_accept_rejects_payment_date_before_invoice_date`, `…_outside_window`, AC4 |
| M9 | SQL : `BETWEEN ? AND ?` lié dans l'ordre inverse (`latest`, `earliest`) | AC4, `find_unpaid_invoices_filters_by_30_day_window` |
| M10 | `ORDER BY i.date ASC, i.id ASC` retiré | `find_unpaid_invoices_orders_by_date_then_id_before_limit` — **peut survivre** : sans `ORDER BY`, le plan actuel livre probablement le même ordre (index puis clé primaire) ; c'est précisément ce que l'`ORDER BY` *garantit* au lieu de le laisser au plan. Survivante, elle se déclare **équivalente au plan observé**, avec l'ordre relevé |
| M11 | `accept_one_invoice` lit `InvoiceDateWindow::for_payment_date(bank_transaction.booking_date)` | AC5 (a), (b), (c) (acceptations forcées) |
| M12 | `let paid_at_candidate = bank_transaction.booking_date;` (le règlement écrit est daté par la comptabilisation) | AC5 (b) (`settled_on == B − 5`) |
| M13 | `ORDER BY i.date DESC, i.id DESC` | `find_unpaid_invoices_orders_by_date_then_id_before_limit` |
| M14 | `ORDER BY i.id ASC` (sans la date) | `find_unpaid_invoices_orders_by_date_then_id_before_limit` (insertion en dates décroissantes : `id ASC` = dates décroissantes) |

Protocole : `docs/agents-orchestration.md` § 7 (motif vérifié unique, `git checkout -- <f>` **puis `touch`**),
campagne sous le verrou de gate. Résultat consigné au Dev Agent Record : tuée / survivante / équivalente, avec le
nom du test rouge.

**AC11 — Gates.**

`kesh-db` est touché (repository et tests) : **gate complet même en cours de boucle** (`CLAUDE.md` § *Pendant une
boucle de revue*, exception `kesh-db`). Au dernier commit de code : base remise à zéro **de ses propres bases**
(`kesh_287`, `kesh_e2e_287` — jamais de `docker restart`), `scripts/test-fast.sh` complet, Vitest complet,
`npm run check`, `npm run lint-i18n-ownership`, `npm run build`, E2E Playwright complet (port 3040), tous sous
`flock <kesh-gate-logs>/.gate.lock`. E2E : le rouge se juge contre `docs/testing.md` § *Les échecs attendus*.
Aucune migration : pas de bump `min_required`, pas de ligne `docs/migrations-idempotence-audit.md`, pas
d'entrée `POST_RESTORE_BACKFILLS`.

## Tasks / Subtasks

- [ ] **T0 — Rebase et relevé** (prérequis)
  - [ ] Rebaser `story/28-7` sur `origin/main` après le merge de la 28-6 ; relire les numéros de ligne de la fiche
        (`grep -nF` des motifs cités) et noter tout écart au Dev Agent Record.
  - [ ] Créer `kesh_287` et `kesh_e2e_287` (migrations du worktree, seed) ; `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-28-7/target`.
- [ ] **T1 — La fenêtre** (AC1, AC6)
  - [ ] Constantes, `payment_date`, `InvoiceDateWindow`, `InvoiceDateRefusal` (arithmétique sans ramènement de borne) ; doc-comments (règle, pourquoi, lecteurs, trois sites non résolus nommés par leur nom).
  - [ ] `mod tests` unitaire.
- [ ] **T2 — La proposition** (AC2)
  - [ ] Nouvelle signature et SQL `BETWEEN ? AND ?` ; `ORDER BY i.date ASC, i.id ASC` avant `LIMIT 50` (C-28-7-10) ; doc-comment du filtre 3 (ordre compris) et doc de module (`:5-10`).
  - [ ] `get_proposals` : `InvoiceDateWindow::for_transaction(&tx)` ; retrait de `WINDOW_DAYS`.
- [ ] **T3 — L'acceptation** (AC3)
  - [ ] `check` dans `accept_one_invoice`, raisons et `details` identiques ; `let paid_at_candidate = payment_date(&bank_transaction);` (ses cinq lectures restantes inchangées) ; `Duration` retiré de l'import `:30` s'il n'a plus d'emploi (`grep -nw Duration`, `clippy -D warnings`).
  - [ ] Commentaires P3-M2 / « Lower bound (existant) » (`:1659`, `:1667-1672`) réécrits ; doc de `AcceptProposalBody::Split` (`:465`) **non touché** (date d'écriture, hors fenêtre).
- [ ] **T4 — Tests d'intégration** (AC4, AC5, AC6 fin)
  - [ ] `proposal_and_acceptance_share_the_invoice_date_window`, `proposals_use_the_value_date_like_the_acceptance` (dont `settled_on` au cas (b)).
  - [ ] Réécriture de `find_unpaid_invoices_filters_by_30_day_window` ; adaptation des dix appels de tests ;
        `find_unpaid_invoices_orders_by_date_then_id_before_limit` (AC6, C-28-7-10).
  - [ ] Grep des tests de propositions dont une facture serait datée **après** `D + 1` ou **avant** `D − 30`
        (proposition attendue qui disparaîtrait) — `reconciliation_e2e.rs`, `reconciliation_rules_e2e.rs`,
        `rejeu_interblocage_e2e.rs`, `frontend/tests/e2e/reconciliation-amount-due.spec.ts` : en lister le résultat
        au Dev Agent Record, même vide.
- [ ] **T5 — Libellé** (AC7) : quatre `.ftl`, repli (G13), doc-comment et commentaire de test ; commentaire de
      `frontend/tests/e2e/reconciliation-amount-due.spec.ts:78` (« dans la fenêtre de 30 jours autour du relevé »
      → « au plus 30 jours avant le relevé », § *Inventaire (c)*) ; recompte `i18nMsg(`.
- [ ] **T6 — Manuel** (AC8) : trois sites, `make fr`, contrôle aplati du PDF, `textes_coherents` ; grep de
      propagation sur **tout** `docs/manual/fr/*.tex` — `marketing-brochure.tex:167` (« sur une fenêtre de 30
      jours ») reste exact (la fenêtre garde 30 jours côté facture ancienne) : trié au Dev Agent Record, non modifié.
- [ ] **T7 — CHANGELOG** : sous `## [0.13.3] — Non publié` (à créer s'il n'existe pas encore au rebase, au-dessus
      de `## [0.13.2]` ; sinon y ajouter), section `### Corrigé` :
      « **La réconciliation ne propose plus une facture qu'elle refuserait ensuite ([#548](https://github.com/guycorbaz/kesh/issues/548)).**
      Une facture datée de plus d'un jour **après** le paiement était proposée, puis l'acceptation la refusait
      (« Le paiement est daté de plus d'un jour avant la facture »). La proposition applique désormais la même
      fenêtre que l'acceptation, sur la même date — la date de valeur de la transaction, à défaut sa date de
      comptabilisation : la facture est datée au plus un jour après le paiement, et au plus 30 jours avant. Une
      facture datée de 30 jours avant à un jour après le paiement est donc proposée même quand la date de valeur
      précède la date de comptabilisation ; à l'inverse, quand la date de valeur suit la date de comptabilisation,
      une facture datée de plus de 30 jours avant la date de valeur n'est plus proposée — l'acceptation la
      refusait déjà. Au-delà de 50 factures candidates, ce sont désormais toujours les plus anciennes qui sont
      examinées, les 50 plus anciennes. Un paiement reçu plus de 30 jours après la facture n'est toujours pas
      proposé ([#529](https://github.com/guycorbaz/kesh/issues/529)). L'API de réconciliation est inchangée. »
- [ ] **T8 — Mutations** (AC10) et **gates** (AC11) ; Dev Agent Record (chiffres recomptés, périmètre déclaré).

## Dev Notes

### Inventaire (a) — les sites qui choisissent ou valident une facture pour une transaction, au sol

Greps exécutés sur `5aad8b35`, sorties copiées.

```
$ grep -rn 'find_unpaid_invoices_for_window(' crates frontend/src --include=*.rs
crates/kesh-api/src/routes/reconciliation.rs:649:        let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
crates/kesh-db/tests/invoice_write_off.rs:785:    let before = reconciliation::find_unpaid_invoices_for_window(
crates/kesh-db/tests/invoice_write_off.rs:802:    let after = reconciliation::find_unpaid_invoices_for_window(
crates/kesh-db/tests/reconciliation_repository.rs:306:    let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
crates/kesh-db/tests/reconciliation_repository.rs:330:/// `find_unpaid_invoices_for_window(company_A)` ne doit pas retourner
crates/kesh-db/tests/reconciliation_repository.rs:370:    let candidates_a = reconciliation_repo::find_unpaid_invoices_for_window(
crates/kesh-db/tests/reconciliation_repository.rs:385:    let candidates_b = reconciliation_repo::find_unpaid_invoices_for_window(
crates/kesh-db/tests/reconciliation_repository.rs:462:    let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
crates/kesh-db/tests/reconciliation_repository.rs:516:    let candidates = reconciliation_repo::find_unpaid_invoices_for_window(
crates/kesh-db/tests/reconciliation_repository.rs:793:    let found_ttc = reconciliation_repo::find_unpaid_invoices_for_window(
crates/kesh-db/tests/reconciliation_repository.rs:814:    let found_ht = reconciliation_repo::find_unpaid_invoices_for_window(
crates/kesh-db/tests/reconciliation_repository.rs:898:    let found_due = reconciliation_repo::find_unpaid_invoices_for_window(
crates/kesh-db/tests/reconciliation_repository.rs:916:    let found_ttc = reconciliation_repo::find_unpaid_invoices_for_window(

$ grep -rn 'propose_matches\|accept_one_invoice\|find_invoice_by_id_for_company' crates --include=*.rs | grep -v '/tests/'
  (production) routes/reconciliation.rs:741 (propose_matches, après le filtre de dates — ne lit aucune date),
  :1388 (seul appel d'accept_one_invoice), :1598 (chargement de la facture dans accept_one_invoice),
  :1707 (re-score) ; kesh-reconciliation/src/matching.rs:62 (propose_matches : score montant/référence/contact,
  aucune date — `grep -n 'date' crates/kesh-reconciliation/src/matching.rs` ne rend que des fixtures de test).

$ grep -rn "i.date BETWEEN\|invoices.*date BETWEEN\|INTERVAL ? DAY" crates --include=*.rs
crates/kesh-report/src/vat_report.rs:125:           AND i.date BETWEEN ? AND ?",
crates/kesh-report/src/vat_report.rs:266:           AND i.date BETWEEN ? AND ? \
crates/kesh-db/src/repositories/reconciliation.rs:135:           AND i.date BETWEEN DATE_SUB(?, INTERVAL ? DAY) AND DATE_ADD(?, INTERVAL ? DAY) \

$ grep -rn 'WINDOW_DAYS' crates --include=*.rs
crates/kesh-api/src/routes/reconciliation.rs:62:/// Fenêtre temporelle ± `WINDOW_DAYS` autour de `tx.booking_date` pour
crates/kesh-api/src/routes/reconciliation.rs:64:const WINDOW_DAYS: i64 = 30;
crates/kesh-api/src/routes/reconciliation.rs:654:            WINDOW_DAYS,
crates/kesh-api/src/routes/reconciliation.rs:1667:    // Upper bound (P3-M2 Pass 3) — paiement <= invoice.date + WINDOW_DAYS.
crates/kesh-api/src/routes/reconciliation.rs:1673:    if paid_at_candidate > invoice.date + Duration::days(WINDOW_DAYS) {
crates/kesh-api/src/routes/reconciliation.rs:1679:                "window_days": WINDOW_DAYS,

$ grep -rln 'bank_transaction' crates/kesh-api/src/routes/ crates/kesh-db/src/repositories/ | sort
  bank_accounts.rs, bank_imports.rs, reconciliation.rs (routes) ; bank_accounts.rs, bank_imports.rs,
  bank_transactions.rs, invoices.rs, journal_entries.rs, mod.rs, reconciliation_cancel.rs, reconciliation.rs,
  settlement_cancellation.rs (repositories)
$ grep -n -i 'invoice' crates/kesh-api/src/routes/bank_imports.rs crates/kesh-db/src/repositories/bank_transactions.rs crates/kesh-db/src/repositories/bank_imports.rs
crates/kesh-api/src/routes/bank_imports.rs:1417:/// (patron `imported_supplier_invoices::complete_import`) : un lot
```

Les fichiers qui citent `bank_transaction` hors `reconciliation.rs` **lisent ou annulent** un rapprochement
(`reconciliation_cancel.rs`, `settlement_cancellation.rs`, `invoices.rs` — dévalidation), aucun ne **choisit**
une facture pour une transaction. `vat_report.rs` filtre des factures par période, sans transaction.
**Ensemble clos : deux sites** — la proposition (`find_unpaid_invoices_for_window`, appelée par `get_proposals`)
et l'acceptation (`accept_one_invoice`, étape 6). Le frontend ne filtre aucune date (`grep -rn
'WINDOW_DAYS\|window_days\|windowDays' frontend/src` ne rend que `failed-proposal-label.test.ts:214`, une
fixture).

### Inventaire (b) — les sites non résolus (bornes voisines qui ne liront pas la fenêtre)

```
$ grep -rn 'days(1)\|before_invoice\|BEFORE_INVOICE' crates/*/src --include=*.rs
  (…) crates/kesh-api/src/routes/reconciliation.rs:1660 (site de la story)
crates/kesh-db/src/repositories/invoice_settlements_write.rs:115:    if settled_on < invoice_date - chrono::Duration::days(1) {
crates/kesh-db/src/repositories/invoice_settlements_write.rs:489:    if settled_on < invoice_date - chrono::Duration::days(1) {

$ grep -rniE "paid_at_after_date|INTERVAL 1 DAY" crates/kesh-db/migrations      (ajouté en validation P1, F3)
  seule 20260419000001_invoice_paid_at.sql (ADD CONSTRAINT IF NOT EXISTS chk_invoices_paid_at_after_date,
  CHECK (paid_at IS NULL OR DATE(paid_at) >= date - INTERVAL 1 DAY), :32-33)
```

| Site | Ce qu'il borne | Sort |
|---|---|---|
| `invoice_settlements_write.rs:115` (`settle_invoice`, `:67`) | date de règlement **saisie** ≥ facture − 1 | **non résolu, assumé** (C-28-7-7) : pas de transaction, pas de borne haute ; même tolérance, écrite au doc-comment de la fenêtre |
| `invoice_settlements_write.rs:489` (`write_off_invoice`, `:429` — le solde par escompte, perte, frais ou arrondi, Story 25-4-d2a) | idem | idem |
| CHECK `chk_invoices_paid_at_after_date` (`migrations/20260419000001_invoice_paid_at.sql:32-33` ; `test-schema/0001_schema_squash.sql:755`) | `DATE(paid_at) >= date - INTERVAL 1 DAY` | **non résolu, assumé** : borne de schéma, la seule qui ne se desserre pas sans migration (P2/P3) ; nommée au doc-comment de la fenêtre. C'est elle qui rend l'élargissement de l'acceptation coûteux (C-28-7-1) |
| `routes/reconciliation.rs:2470`, `:2879`, `:3747`, `:4221` (`value_date.or(…).unwrap_or(booking_date)`) | **date d'écriture** d'un éclatement, d'un rapprochement manuel, d'une règle | hors classe : n'apparie aucune facture |

### Inventaire (c) — les tests existants dont le sens pourrait changer

- `kesh-db` : `reconciliation_repository.rs:258` (fenêtre ± 30, réécrit, AC6) ; les autres appels posent des
  factures au jour de la transaction ou avant (`:345`, `:410`, `:491` : `day` pour les deux ;
  `:765-766`, `:843-844` : facture au 2026-04-30, transaction au 2026-05-15) — **à revérifier au rebase**.
- `kesh-api` : `post_accept_rejects_payment_date_before_invoice_date` (`:2397`, transaction le 2026-04-15, facture
  le 2026-05-01) et `…_outside_window` (`:2725`, `window_days == 30` à `:2785`) — inchangés, garde du contrat ;
  81 `#[sqlx::test]` au total dans le fichier (`grep -c`), dont les `get_proposals_*` (`:614`, `:734`, `:849`,
  `:899`, `:2067`, `:3305`, `:4444`) — T4 en vérifie les dates.
- Frontend : `failed-proposal-label.test.ts:170-178` (commentaire réécrit, assertion inchangée) ;
  `frontend/tests/e2e/reconciliation-amount-due.spec.ts:78` (« Date de facture dans la fenêtre de 30 jours autour
  du relevé ») — facture au 2026-05-10, relevé au 2026-05-15 : toujours dans la fenêtre ; **le commentaire est
  réécrit** (« au plus 30 jours avant le relevé »), symptôme de la même formulation.

### Propagation — le symptôme à greper après chaque patch

La **valeur** `30` et la **formulation** « autour », « ± », « précèdent ou suivent », « peut être proposée » :

```sh
grep -rnE "±\s?30|\b30 (jours|days|Tage|giorni)\b|WINDOW_DAYS|window_days|autour de .*booking|peut être proposée|même si elle a été proposée|even though the invoice was proposed|auch wenn diese vorgeschlagen|anche se è stata proposta" \
  crates frontend/src frontend/tests docs/manual/fr/*.tex docs/api-external.md CHANGELOG.md
```

Les occurrences légitimes (rappels `:1299`, conditions « 30 jours net », jetons de session, `window_days` du
contrat d'API) se trient à la main. `docs/api-external.md` ne cite ni la fenêtre ni `window_days` (grep vide au
`5aad8b35`) : rien à y changer, le contrat est inchangé.

### Ce qui doit être préservé

- Le contrat de `POST /api/v1/reconciliation/accept` : codes, `details.reason`, `details.window_days` (= 30).
- `get_proposals` : filtres de devise et de signe (`tx.amount <= 0` → aucune candidate) **avant** la requête,
  ordre des passes, `LIMIT 50`, `has_more`, règles en repli.
- La date du règlement écrit (`paid_at_dt`, `:1740`) : toujours `value_date`, à défaut `booking_date`.
- La 28-6 (si mergée avant) ajoute une garde de signe à l'entrée d'`accept_one_invoice` : la conserver, placée
  avant l'étape 6.

### Exclusions écrites (périmètre minimal)

- **#529** (paiement tardif de plus de 30 jours, P2) : non absorbé ; démontré non aggravé (§ *Effet sur #529*).
- **Acompte** : pas de modèle ; aucun changement (C-28-7-1).
- **Règlement depuis la fiche** (`invoice_settlements_write.rs`) : non modifié (C-28-7-7).
- **Priorité au montant exact avant le `LIMIT 50`** : l'`ORDER BY i.date, i.id` intégré en validation P1
  (C-28-7-10, AC2) rend la troncature déterministe, mais des candidates « à 5 centimes près » plus anciennes
  peuvent encore évincer une candidate au montant exact. Trier d'abord sur l'écart de montant est une autre
  décision (ordre sur `amount_due`), hors #548 — non prise ici.
- **Captures d'écran** (B2 de l'Epic 27) : la story ne change aucun contrôle ni aucun texte de l'écran de
  propositions (seul le libellé d'un refus devenu rare change) — pas de capture (C-28-7-8).
- **Pas de spec Playwright neuve** (C-28-7-9) : la frontière HTTP est exercée par les deux tests `spawn_app` de
  l'AC4/AC5 (vraie requête HTTP sur le binaire) ; la suite E2E complète tourne au dernier commit de code.

### Modules touchés (règle de découpage : au plus cinq)

1. `kesh-db` — `repositories/reconciliation.rs` (+ deux fichiers de tests) ;
2. `kesh-api` — `routes/reconciliation.rs` (+ `tests/reconciliation_e2e.rs`) ;
3. `kesh-i18n` — quatre `.ftl`, une clé ;
4. frontend `features/reconciliation` — un repli, un doc-comment, un commentaire de test ; une spec E2E (commentaire) ;
5. documentation — manuel FR (+ PDF), CHANGELOG.

Cinq, dont deux de texte seulement : pas de découpage.

### Pièges connus du dépôt, applicables ici

- **Gate complet en boucle** (`kesh-db`), et base remise à zéro avant chaque gate complet (KF-039).
- **Mutation restaurée, binaire périmé** : `git checkout -- f && touch f`.
- **`pdftotext` coupe les lignes** : aplatir avant de greper.
- **Le `\label{sec:facture-posterieure}` est référencé** (`:2787`) : le garder, sinon `??` dans le PDF.
- **Ne jamais réécrire l'entrée d'une passe passée** (consigne 15).
- **Données réelles** : fixtures synthétiques seulement ; aucun relevé de `~/kesh-test/` dans un test, un commit
  ou un rapport versionné.

### References

- Issue #548 ; #529 (voisine, non absorbée).
- `_bmad-output/planning-artifacts/epic-28-recette-v0-13-2.md` § *28-7*, § *Ordre*.
- `_bmad-output/implementation-artifacts/epic-15-choix-autonomes.md` C-15-5c-3 (libellé dédié, moteur inchangé).
- `crates/kesh-db/src/repositories/reconciliation.rs:86-160` ; `crates/kesh-api/src/routes/reconciliation.rs:62-64`,
  `:600-660`, `:1549-1740` ; `crates/kesh-db/src/repositories/invoice_settlements_write.rs:108-117`, `:488-491`.
- `docs/manual/fr/user-manual.tex:1917-1921`, `:2787` ; `docs/agents-orchestration.md` ; `CLAUDE.md`
  § *Review Iteration Rule*, § *Test Locally First*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- **2026-10-13 — fiche créée** (`bmad-create-story`, Opus 5.5, autonomie), sur `5aad8b35`. 11 AC, T0-T8,
  10 mutations, choix C-28-7-1 à 9. Validation P1 à lancer.
- **2026-10-13 — Validation P1** (`28-7-validate-prompt-p1.md`, Sonnet 5.5, lentilles R et F en contexte frais ;
  rapports hors dépôt `kesh-gate-logs/28-7-validate-p1-{R,F}.md`). **F : 0 C / 0 H / 3 M / 3 L ; R : 0 C / 0 H /
  2 M / 9 L** — soit 5 MEDIUM dont **4 distincts** (F1 = R2) et 12 LOW dont **10 distincts** (F4 = R4, F6 = R8).
  Les deux lentilles confirment l'algèbre `D − 30 ≤ F ≤ D + 1` (bornes incluses, exactement l'acceptation), la
  démonstration « #529 non aggravé », les tableaux de l'AC4 et de l'AC5 ligne à ligne, l'ensemble clos à deux
  sites qui choisissent une facture, et les recomptes de la création. Remédiation (remédiateur Opus 5.5, décisions
  de l'orchestrateur) :
  - **F1 = R2 (MEDIUM) — clamp retiré** : C-28-7-4 marqué *retiré* au registre (texte d'origine conservé) ; AC1
    réécrit (arithmétique ordinaire, sans ramènement de borne), les deux assertions de bornes de la plage `DATE`
    de l'AC6 retirées, ancienne M10 retirée, T1 corrigé.
  - **F2 (MEDIUM) — `ORDER BY i.date ASC, i.id ASC` intégré** : nouveau choix **C-28-7-10** ; AC2 réécrit (« `LIMIT
    50` inchangé, désormais déterministe ») ; nouveau test de dépôt
    `find_unpaid_invoices_orders_by_date_then_id_before_limit` à l'AC6 (52 factures de même reste dû, insérées
    en dates décroissantes) ; mutations **M10** (`ORDER BY` retiré — déclarée possiblement équivalente au plan),
    **M13** (`DESC`), **M14** (`id` seul) ; T2 et T4 ; l'exclusion « `LIMIT 50` sans `ORDER BY` » remplacée par
    l'exclusion restante (priorité au montant exact) ; CHANGELOG (T7) d'une phrase.
  - **F3 (MEDIUM)** : la CHECK `chk_invoices_paid_at_after_date` (migration `20260419000001_invoice_paid_at.sql:32-33`,
    squash `:755`) ajoutée à l'inventaire (b) (grep copié) et à l'AC9, troisième détentrice de la borne basse ;
    citée par C-28-7-1 (fiche et registre).
  - **R1 (MEDIUM)** : `invoice_settlements_write.rs:489` est `write_off_invoice` (le solde), non « le règlement du
    reste » ; AC9, inventaire (b) et C-28-7-7 nomment `settle_invoice` et `write_off_invoice` par leur fonction,
    et l'AC9 exige le nom, non la ligne, au doc-comment livré.
  - **LOW** : F4 = R4 (AC3 et T3 : `let paid_at_candidate = payment_date(&bank_transaction);` conservée, ses huit
    sites au sol nommés, import `Duration` retiré s'il n'a plus d'emploi) ; F5 (**M11** acceptation sur
    `booking_date`, **M12** règlement daté par `booking_date`, et l'AC5 (b) asserte `settled_on == B − 5`) ;
    F6 = R8 (`D` et `B` en plein exercice 2026, ex. 2026-06-15 ; montants distants de plus de 0.05 CHF ;
    références distinctes) ; R3 (G13 nommée à l'AC7) ; R5 (commentaire de
    `reconciliation-amount-due.spec.ts:78` porté à T5) ; R6 (`marketing-brochure.tex:167` trié à T6, grep de
    propagation étendu à `docs/manual/fr/*.tex`) ; R7 (AC8 : « la même tolérance d'un jour avant la facture ») ;
    R9 (en-CH : « it cannot settle this invoice ») ; R10 (CHANGELOG : l'autre sens du changement de date de
    référence) ; R11 (`:1656-1683` → `:1656-1682` / `:1660-1682` ; `:543` retiré de l'inventaire (c), c'est
    `find_pending_transactions_excludes_auto_match_rejected`). En sus, T4 nomme les trois autres consommateurs de
    propositions relevés par R (`reconciliation_rules_e2e.rs`, `rejeu_interblocage_e2e.rs`, la spec Playwright).
  - Symptômes grepés dans la fiche et le registre après patch (`clamp|ramen|9999|1000-01|checked_|plage`,
    `ORDER BY|LIMIT 50`, `:489|:115|règlement du reste|1683|:543|it cannot`) : seuls restent les sites voulus et
    l'entrée de création ci-dessus, non réécrite.
  - **Recompte après remédiation** : 11 AC (AC1-AC11) ; 9 tâches (T0-T8) ; **14 mutations** (M1-M14 ; l'ancienne
    M10 retirée, le numéro réattribué) ; **10 choix numérotés, 9 en vigueur** (C-28-7-4 retiré, C-28-7-10 ajouté) ;
    modules inchangés, **cinq** (l'`ORDER BY` et son test restent dans `kesh-db`). Tests neufs prescrits :
    deux d'intégration HTTP (AC4, AC5), un module unitaire (AC6), un test de dépôt (C-28-7-10).
  - Validation P2 à lancer : `28-7-validate-prompt-p2.md`, **Opus**, lentilles R et F, axe premier sur les
    régressions de cette remédiation.
