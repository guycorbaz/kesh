# Story 28.7 : Une facture postérieure au paiement n'est plus proposée

## Status

**done** — revue de code close à la P1 le 2026-10-13 (Sonnet, B, E, A : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 9 LOW, remédiés sans ligne de production ; cf. Change Log). — Développement fait le 2026-10-13 (`bmad-dev-story`, Opus 5.5) ; branche rebasée sur `origin/main`
(`5629acc2`, 28-6 fusionnée) ; dernier commit de code `7b10e555` ; gates complets verts au HEAD `0d94fdc7` ;
mutations 14 tuées sur 15, M10 équivalente au plan observé ; revue de code P1 à lancer
(`28-7-review-prompt-p1.md`, Sonnet, lentilles B, E, A). Choix C-28-7-14 à 16 ajoutés au développement.
Validation close par la passe ciblée P3 (Haiku : 1 MEDIUM, 7 LOW, remédiés à l'étape 0, cf. Change Log). — Fiche créée le 2026-10-13 en autonomie (`bmad-create-story`, Opus 5.5), sur `5aad8b35`
(kickoff de l'Epic 28), branche `story/28-7`, worktree `/home/gcorbaz/devel/kesh-28-7`. Validation P1 faite
(Sonnet, lentilles R et F : 0 CRITICAL, 0 HIGH, 5 MEDIUM soit 4 distincts, 12 LOW soit 10 distincts — remédiés,
cf. Change Log) ;
validation P2 faite (Opus, lentilles R et F : 0 CRITICAL, 0 HIGH, 2 MEDIUM distincts, 14 LOW soit 10 distincts —
remédiés, cf. Change Log) ; passe ciblée P3 à lancer (`28-7-validate-prompt-p3-ciblee.md`, Haiku, une lentille).
Choix consignés au registre `epic-28-choix-autonomes.md`, **C-28-7-1 à C-28-7-13**, dont **C-28-7-4 retiré** en
P1 (douze choix en vigueur).
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
extrémités.)* ⚠️ **Angle mort écrit (C-28-7-12, validation P2)** : ce qui ne déborde pas en Rust peut sortir de
la plage `DATE` **au SQL** — une transaction datée `9999-12-31`, que l'import CAMT accepte (aucune borne d'année,
`kesh-import/src/camt053/mod.rs`, `parse_date`), lierait `latest = +10000-01-01` ; le dépôt a déjà rencontré ce
mode d'échec (`routes/letterings.rs:507-511`). Le résultat n'est pas mesuré ; il est accepté, et ne se traite pas
ici. Doc-comment de module et de fonction : la règle, le pourquoi de chaque borne (C-28-7-1), la mention
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
d'abord, et le départage `invoice_id ASC` du tri aval (`kesh-reconciliation/src/matching.rs:104-110`). L'index
`idx_invoices_company_validated_unpaid_date (company_id, status, paid_at, date)` permet de servir cet ordre sans
tri **si** l'optimiseur le choisit ; ce choix n'est pas établi (aucun plan relevé, et la table porte aussi
`idx_invoices_company_date`) : c'est l'`ORDER BY` qui garantit l'ordre, quel que soit le plan. Aucun test ne lit
l'ordre du SQL (`propose_matches` retrie par `score DESC, invoice_id ASC`). *(Annotation du 2026-10-13, revue de
code P1, A-LOW-4 : la phrase précédente décrit l'état d'avant la remédiation P1 de la validation ; depuis, le test
de dépôt `find_unpaid_invoices_orders_by_date_then_id_before_limit` (AC6) lit l'ordre du SQL, et M13, M14 et M16
le prouvent. Aucun test HTTP ne le lit.)* Au-delà de 50 candidates, ce sont donc les plus récentes qui
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
`toContain('plus d’un jour avant la facture')` (`:177`) reste vraie, mais elle passe avec l'ancien texte comme
avec le nouveau : le même test gagne `expect(label).not.toContain('proposée')`, qui rougit tant que le texte
n'est pas raccourci (le repli suit le catalogue fr-CH par G13 ; pour de-CH, en-CH et it-CH, le grep de
propagation, § *Propagation*, qui porte déjà leurs motifs, sert de contrôle déclaré au Dev Agent Record). Aucune clé ajoutée ni retirée : les
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
  la facture a été redatée entre-temps ; le règlement saisi depuis la fiche de la facture — ou depuis la page des
  échéances (`routes/(app)/invoices/due-dates/+page.svelte`, qui monte le même `SettleInvoiceDialog`) — refuse lui
  aussi un règlement daté avant la facture — **l'écran** (`SettleInvoiceDialog.svelte`, `WriteOffDialog.svelte`) refuse
  toute date antérieure à la date de facture et n'en propose aucune au sélecteur (`min={invoiceDate}`) —, et n'a
  aucune borne de 30 jours après, ce qui reste le contournement de #529 ; l'acompte comme aujourd'hui. **Le
  manuel décrit ce que voit l'utilisateur, l'écran, et ne parle d'aucune tolérance d'un jour pour ce chemin** (le
  serveur en admet une, que l'écran ne laisse pas atteindre : C-28-7-11). **Le `\label` est conservé**
  (référencé à `:2787`) ; le titre du paragraphe change (ex. « Une facture datée de plus d'un jour après le
  paiement n'est pas proposée. ») : **le titre comme le corps disent « plus d'un jour après »** — une facture
  datée du lendemain du paiement reste proposée (AC4, décalage `+1` ; AC5 (d)).
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
  d'un jour, qui **ne se desserre que par une migration**.

Aucun autre site du dépôt ne choisit ni ne valide une facture pour une transaction (Dev Notes § *Inventaire (a)*,
greps copiés). Les deux dialogues de règlement du frontend (`SettleInvoiceDialog.svelte`, `WriteOffDialog.svelte`)
bornent eux aussi la date saisie, **sans tolérance** : ils figurent à l'inventaire (b), non au doc-comment Rust
(C-28-7-11).

**AC10 — Mutations (chacune fait rougir au moins un test nommé, sauf M10 : tuée ou équivalente selon le plan
observé).**

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
| M10 | `ORDER BY i.date ASC, i.id ASC` retiré | `find_unpaid_invoices_orders_by_date_then_id_before_limit` — **tuée ou équivalente selon le plan observé au dev** : sans `ORDER BY`, l'ordre livré dépend du plan (index, ou parcours en ordre de clé primaire — un parcours en clé primaire, sur 52 factures insérées en dates décroissantes, la tuerait ; un parcours d'index sur `date` pourrait la laisser survivre) ; le plan n'étant pas connu, l'issue n'est pas prédite. Survivante, elle se déclare **équivalente au plan observé**. Dans les deux cas, le résultat et l'ordre relevé sont consignés |
| M11 | `accept_one_invoice` lit `InvoiceDateWindow::for_payment_date(bank_transaction.booking_date)` | AC5 (a), (b), (c) (acceptations forcées) |
| M12 | `let paid_at_candidate = bank_transaction.booking_date;` (le règlement écrit est daté par la comptabilisation) | AC5 (b) (`settled_on == B − 5`) |
| M13 | `ORDER BY i.date DESC, i.id DESC` | `find_unpaid_invoices_orders_by_date_then_id_before_limit` |
| M14 | `ORDER BY i.id ASC` (sans la date) | `find_unpaid_invoices_orders_by_date_then_id_before_limit` (insertion en dates décroissantes : `id ASC` = dates décroissantes) |
| M15 | `check` : `<` remplacé par `<=` sur `earliest` (symétrique de M7) | AC6 (`D − 30` → `Ok`), AC4 (`−30` proposé par le SQL, refusé par `check` : `P ≠ A`) |

Numérotation stable : les numéros ne se réattribuent plus. ⚠️ « M10 » ne désigne pas la même mutation dans les
rapports P1 (hors dépôt : la mutation du ramènement de borne, retirée avec C-28-7-4) et dans cette fiche depuis
la remédiation P1 (l'`ORDER BY` retiré) — correspondance au Change Log, *Validation P2*.

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

- [x] **T0 — Rebase et relevé** (prérequis)
  - [x] Rebaser `story/28-7` sur `origin/main` après le merge de la 28-6 ; relire les numéros de ligne de la fiche
        (`grep -nF` des motifs cités) et noter tout écart au Dev Agent Record.
  - [x] Créer `kesh_287` et `kesh_e2e_287` (migrations du worktree, seed) ; `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-28-7/target`.
- [x] **T1 — La fenêtre** (AC1, AC6)
  - [x] Constantes, `payment_date`, `InvoiceDateWindow`, `InvoiceDateRefusal` (arithmétique sans ramènement de borne) ; doc-comments (règle, pourquoi, lecteurs, trois sites non résolus nommés par leur nom).
  - [x] `mod tests` unitaire.
- [x] **T2 — La proposition** (AC2)
  - [x] Nouvelle signature et SQL `BETWEEN ? AND ?` ; `ORDER BY i.date ASC, i.id ASC` avant `LIMIT 50` (C-28-7-10) ; doc-comment du filtre 3 (ordre compris) et doc de module (`:5-10`).
  - [x] `get_proposals` : `InvoiceDateWindow::for_transaction(&tx)` ; retrait de `WINDOW_DAYS`.
- [x] **T3 — L'acceptation** (AC3)
  - [x] `check` dans `accept_one_invoice`, raisons et `details` identiques ; `let paid_at_candidate = payment_date(&bank_transaction);` (ses cinq lectures restantes inchangées) ; `Duration` retiré de l'import `:30` s'il n'a plus d'emploi (`grep -nw Duration`, `clippy -D warnings`).
  - [x] Commentaires P3-M2 / « Lower bound (existant) » (`:1659`, `:1667-1672`) réécrits ; doc de `AcceptProposalBody::Split` (`:465`) **non touché** (date d'écriture, hors fenêtre).
- [x] **T4 — Tests d'intégration** (AC4, AC5, AC6 fin)
  - [x] `proposal_and_acceptance_share_the_invoice_date_window`, `proposals_use_the_value_date_like_the_acceptance` (dont `settled_on` au cas (b)).
  - [x] Réécriture de `find_unpaid_invoices_filters_by_30_day_window` ; adaptation des dix appels de tests ;
        `find_unpaid_invoices_orders_by_date_then_id_before_limit` (AC6, C-28-7-10).
  - [x] Grep des tests de propositions dont une facture serait datée **après** `D + 1` ou **avant** `D − 30`
        (proposition attendue qui disparaîtrait) — `reconciliation_e2e.rs` (lecteurs listés à l'inventaire (c)),
        `reconciliation_rules_e2e.rs`, `frontend/tests/e2e/reconciliation-amount-due.spec.ts` ; aucun lecteur de
        propositions hors de ces trois fichiers (`rejeu_interblocage_e2e.rs` n'appelle que `POST /accept` et
        `POST /manual`, jamais `reconciliation/proposals`) : en
        lister le résultat au Dev Agent Record, même vide.
- [x] **T5 — Libellé** (AC7) : quatre `.ftl`, repli (G13), doc-comment et commentaire de test, assertion
      `not.toContain('proposée')` ; commentaire de
      `frontend/tests/e2e/reconciliation-amount-due.spec.ts:78` (« dans la fenêtre de 30 jours autour du relevé »
      → « au plus 30 jours avant le relevé », § *Inventaire (c)*) ; recompte `i18nMsg(`.
- [x] **T6 — Manuel** (AC8) : trois sites, `make fr`, contrôle aplati du PDF, `textes_coherents` ; grep de
      propagation sur **tout** `docs/manual/fr/*.tex` — `marketing-brochure.tex:167` (« sur une fenêtre de 30
      jours ») reste exact (la fenêtre garde 30 jours côté facture ancienne) : trié au Dev Agent Record, non modifié.
- [x] **T7 — CHANGELOG** : sous `## [0.13.3] — Non publié` (à créer s'il n'existe pas encore au rebase, au-dessus
      de `## [0.13.2]` ; sinon y ajouter), section `### Corrigé` :
      « **La réconciliation ne propose plus une facture qu'elle refuserait ensuite ([#548](https://github.com/guycorbaz/kesh/issues/548)).**
      Une facture datée de plus d'un jour **après** le paiement était proposée, puis l'acceptation la refusait
      (« Le paiement est daté de plus d'un jour avant la facture »). La proposition applique désormais la même
      fenêtre que l'acceptation, sur la même date — la date de valeur de la transaction, à défaut sa date de
      comptabilisation : la facture est datée au plus un jour après le paiement, et au plus 30 jours avant. Une
      facture datée de 30 jours avant à un jour après le paiement est donc proposée même quand la date de valeur
      précède la date de comptabilisation ; à l'inverse, quand la date de valeur suit la date de comptabilisation,
      une facture datée de plus de 30 jours avant la date de valeur n'est plus proposée — l'acceptation la
      refusait déjà. Un paiement reçu plus de 30 jours après la facture n'est toujours pas
      proposé ([#529](https://github.com/guycorbaz/kesh/issues/529)). L'API de réconciliation est inchangée. »
      Le plafond de 50 candidates et leur ordre n'y figurent pas, ni au manuel, qui ne décrit pas ce plafond
      (C-28-7-13).
- [x] **T8 — Mutations** (AC10) et **gates** (AC11) ; Dev Agent Record (chiffres recomptés, périmètre déclaré).

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
crates/kesh-report/src/vat_report.rs:245:/// `i.date BETWEEN` couvre le même ensemble de factures que la TVA due dérivée
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
et l'acceptation (`accept_one_invoice`, étape 6). Le frontend ne filtre ni ne calcule aucune fenêtre
(`grep -rn 'WINDOW_DAYS\|window_days\|windowDays' frontend/src` ne rend que `failed-proposal-label.test.ts:214`,
une fixture) : il transmet à l'acceptation le candidat de tête que le serveur a proposé (sélection groupée,
`ReconciliationProposals.svelte`, `p.candidates[0]`), que l'acceptation revalide ; il borne en revanche la date de règlement **saisie** (inventaire (b), ajouté en validation P2).

### Inventaire (b) — les sites non résolus (bornes voisines qui ne liront pas la fenêtre)

```
$ grep -rn 'days(1)\|before_invoice\|BEFORE_INVOICE' crates/*/src --include=*.rs
  (…) crates/kesh-api/src/routes/reconciliation.rs:1660 (site de la story)
crates/kesh-db/src/repositories/invoice_settlements_write.rs:115:    if settled_on < invoice_date - chrono::Duration::days(1) {
crates/kesh-db/src/repositories/invoice_settlements_write.rs:489:    if settled_on < invoice_date - chrono::Duration::days(1) {

$ grep -rniE "paid_at_after_date|INTERVAL 1 DAY" crates/kesh-db/migrations      (ajouté en validation P1, F3)
  seule 20260419000001_invoice_paid_at.sql (ADD CONSTRAINT IF NOT EXISTS chk_invoices_paid_at_after_date,
  CHECK (paid_at IS NULL OR DATE(paid_at) >= date - INTERVAL 1 DAY), :32-33)

$ grep -nF 'settledOn < invoiceDate' frontend/src/lib/features/invoices/*.svelte      (ajouté en validation P2, F2)
frontend/src/lib/features/invoices/WriteOffDialog.svelte:105:		if (invoiceDate && settledOn < invoiceDate.slice(0, 10)) {
frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:155:		if (invoiceDate && settledOn < invoiceDate.slice(0, 10)) {
$ grep -nF 'min={invoiceDate}' frontend/src/lib/features/invoices/*.svelte
frontend/src/lib/features/invoices/WriteOffDialog.svelte:211:				min={invoiceDate}
frontend/src/lib/features/invoices/SettleInvoiceDialog.svelte:295:				<Input id="settle-date" type="date" bind:value={settledOn} min={invoiceDate} />
```

| Site | Ce qu'il borne | Sort |
|---|---|---|
| `invoice_settlements_write.rs:115` (`settle_invoice`, `:67`) | date de règlement **saisie** ≥ facture − 1 | **non résolu, assumé** (C-28-7-7) : pas de transaction, pas de borne haute ; même tolérance, écrite au doc-comment de la fenêtre |
| `invoice_settlements_write.rs:489` (`write_off_invoice`, `:429` — le solde par escompte, perte, frais ou arrondi, Story 25-4-d2a) | idem | idem |
| CHECK `chk_invoices_paid_at_after_date` (`migrations/20260419000001_invoice_paid_at.sql:32-33` ; `test-schema/0001_schema_squash.sql:755`) | `DATE(paid_at) >= date - INTERVAL 1 DAY` | **non résolu, assumé** : borne de schéma, la seule qui ne se desserre que par une migration ; nommée au doc-comment de la fenêtre. C'est elle qui rend l'élargissement de l'acceptation coûteux (C-28-7-1) |
| `SettleInvoiceDialog.svelte:155` et `:295` ; `WriteOffDialog.svelte:105` et `:211` (frontend, écrans de règlement et de solde depuis la fiche ; `SettleInvoiceDialog` est aussi monté par la page des échéances) | date de règlement **saisie** ≥ facture, **sans tolérance** ; le sélecteur n'offre aucune date antérieure (`min={invoiceDate}`) | **non résolu, assumé** (C-28-7-11) : écran plus strict que le serveur, hors #548 ; ne choisit aucune facture pour une transaction. C'est ce que le manuel décrit pour ce chemin (AC8) |
| `routes/reconciliation.rs:2470`, `:2879`, `:3747`, `:4221` (`value_date.or(…).unwrap_or(booking_date)`) | **date d'écriture** d'un éclatement, d'un rapprochement manuel, d'une règle | hors classe : n'apparie aucune facture |

### Inventaire (c) — les tests existants dont le sens pourrait changer

- `kesh-db` : `reconciliation_repository.rs:258` (fenêtre ± 30, réécrit, AC6) ; les autres appels posent des
  factures au jour de la transaction ou avant (`:345`, `:410`, `:491` : `day` pour les deux ;
  `:765-766`, `:843-844` : facture au 2026-04-30, transaction au 2026-05-15) — **à revérifier au rebase**.
- `kesh-api` : `post_accept_rejects_payment_date_before_invoice_date` (`:2397`, transaction le 2026-04-15, facture
  le 2026-05-01) et `…_outside_window` (`:2725`, `window_days == 30` à `:2785`) — inchangés, garde du contrat ;
  80 `#[sqlx::test]` au total dans le fichier (`grep -c '^#\[sqlx::test'` ; un `grep -c 'sqlx::test'` rend 81, il
  compte le doc-comment de la l. 19), dont les tests qui lisent les propositions (`:614`, `:734`, `:849`, `:899`,
  `:1919`, `:2067`, `:3305`, `:4444`, `:5095`, et le helper `assert_back_to_reconcile`, `:3616`) — T4 en vérifie
  les dates. `reconciliation_rules_e2e.rs` lit aussi les propositions (8 occurrences de `reconciliation/proposals`,
  `grep -c`) : T4 en vérifie les dates de la même façon.
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
- **Transaction datée aux extrémités de la plage `DATE`** (C-28-7-12) : atteignable par un CAMT daté
  `9999-12-31` (l'import accepte toute année) ; la borne `latest = +10000-01-01` liée au SQL a un comportement non
  mesuré (précédent `routes/letterings.rs:507-511`, où chrono lit ce que sqlx ou la base refusent) ; angle mort
  écrit et accepté, sans test ni ramènement (C-28-7-4 reste retiré).
- **Plafond de 50 candidates** (C-28-7-13) : son ordre devient déterministe, mais ni le CHANGELOG ni le manuel ne
  le décrivent — le manuel ne l'a jamais décrit, et l'écrire au seul CHANGELOG ouvrirait un écart entre supports.
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

Opus 5.5 (`bmad-dev-story`, autonomie).

### Debug Log References

**T0 — rebase et relevé (2026-10-13).** `story/28-7` rebasée sur `story/28-6` (`8ba4e168`, en revue, non
fusionnée) : conflits du registre (union, section 28-7 placée après la 28-6) et de `sprint-status.yaml` (union par
clé, `yaml.safe_load` vert). Bases `kesh_287` et `kesh_e2e_287` créées (80 migrations, seed). Numéros relevés au
`grep -n` contre le code rebasé (28-5a et 28-6 comprises) — écarts à la fiche, relevée sur `5aad8b35` :

| Cité par la fiche | Au rebase |
|---|---|
| `routes/reconciliation.rs:62-64` (`WINDOW_DAYS`) | `:66-68` |
| `:649-656` (appel de `find_unpaid_invoices_for_window`) | `:654-661` ; boucle `for tx` `:639` |
| `:741` (`propose_matches`) | `:746` |
| `:1656-1682` (dates d'`accept_one_invoice`) | `:1723-1749` ; `accept_one_invoice` `:1606`, garde de signe de la 28-6 `:1659`, **avant** l'étape 6 |
| `paid_at_candidate`, huit sites (`:1656`, `:1660`, `:1673`, `:1740`, `:1911`, `:1917`, `:1961`, `:1994`) | huit sites : `:1723`, `:1727`, `:1740`, `:1807`, `:1978`, `:1984`, `:2028`, `:2061` |
| `use chrono::{Duration, NaiveDate};` `:30` | `:34` ; `Duration` employé aux seules `:1727` et `:1740` |
| `repositories/reconciliation.rs:91`, `:135` | `:92` (filtre 3), `:136` (SQL) — fichier non touché par la 28-6 |
| `failed-proposal-label.ts:224` (repli) | `:226` ; doc-comment `:41` |
| `failed-proposal-label.test.ts:170-178` | `:172-179` |
| `user-manual.tex:2787` (FAQ) | `:2792` ; `:1919` et `:1921` inchangées |
| `messages.ftl` fr-CH `:1233`, autres `:1170` | inchangés |
| `reconciliation_repository.rs:258` et les dix appels | inchangés (`:306` pour l'appel du test de fenêtre) |
| CHANGELOG | `## [0.13.3] — Non publié` existe déjà (`:11`) |

*Après le second rebase (sur `origin/main` `5629acc2`, 28-6 fusionnée après sa revue)* : garde de signe de la 28-6
à `routes/reconciliation.rs:1661`, `accept_one_invoice` `:1608`, `check` de la fenêtre `:1733` — la garde reste
**avant** l'étape 6. `reconciliation_e2e.rs` compte 94 `#[sqlx::test]` à l'étape 0 (80 sur `5aad8b35` : la 28-5a
et la 28-6 en ont ajouté).

**T1 — rouge d'abord** (commit `0813d96f`, rebasé en `91e1494d` ; C-28-7-14). Sur le code non corrigé, journal
`kesh-gate-logs/28-7-t1-rouge.log` : `proposal_and_acceptance_share_the_invoice_date_window` **rouge** (proposés
`{−30, −1, 0, +1, +2, +30}`, acceptés `{−30, −1, 0, +1}` : les décalages `+2` et `+30` de #548) ;
`proposals_use_the_value_date_like_the_acceptance` **rouge** (proposés (a), (c), (d), (e) ; attendus (b), (d)) ;
`find_unpaid_invoices_orders_by_date_then_id_before_limit` **vert** dès l'ancien code — le plan livre déjà
`(date, id)` (C-28-7-15). T5 : `expect(label).not.toContain('proposée')` **rouge** avant la réécriture du repli
(« expected … not to contain 'proposée' »), vert après.

**T4 — lecteurs de propositions.** `reconciliation_e2e.rs` (binaire entier vert après la correction : aucune
proposition attendue n'a disparu), `reconciliation_rules_e2e.rs` et `rejeu_interblocage_e2e.rs` (58/58 verts, gate
ciblé) ; `reconciliation-amount-due.spec.ts` : facture au 2026-05-10, relevé au 2026-05-15, dans la fenêtre — la
spec passe à l'E2E complet. **Aucun test existant dont une facture sorte de `[D−30, D+1]`.**

**T8 — campagne de mutations** (`kesh-gate-logs/28-7-mutations-run.txt`, sorties `28-7-mutations/M*.log`, script
hors dépôt, sous le verrou ; filtre : binaires `reconciliation_e2e`, `reconciliation_repository`,
`invoice_write_off` et `repositories::reconciliation::tests`, 134 tests). Restauration `git checkout --` + `touch`
après chacune ; `git status` vide à la fin.

| # | Résultat | Tests rouges (extrait) |
|---|---|---|
| M1 | tuée | AC4, AC5, `check_admits…`, `the_window_of…`, `…filters_by_30_day_window` |
| M2 | tuée | idem M1 |
| M3 | tuée | AC4, `check_admits…`, `the_window_of…`, `…30_day_window`, `…orders_by_date…` |
| M4 | tuée | AC4, `check_admits…`, `the_window_of…`, `…30_day_window` |
| M5 | tuée | AC5, `the_payment_date_is_the_value_date_when_there_is_one` |
| M6 | tuée | AC5 |
| M7 | tuée | AC4, AC5, `check_admits…` |
| M8 | tuée | `post_accept_rejects_payment_date_before_invoice_date`, `…_outside_window`, AC4, AC5 |
| M9 | tuée | 14 tests, dont AC4, AC5, `…30_day_window`, `…orders_by_date…` |
| M10 | **survivante — équivalente au plan observé** (C-28-7-15) | aucun ; 134/134 verts |
| M11 | tuée | AC5 |
| M12 | tuée | AC5 (`settled_on`) |
| M13 | tuée | `…30_day_window`, `…orders_by_date…` |
| M14 | tuée | `…orders_by_date…` |
| M15 | tuée | AC4, `check_admits…` |

**14 tuées, 1 équivalente, 0 survivante non expliquée.**

*(Annotation du 2026-10-13, revue de code P1, E-LOW-2 : « équivalente au plan observé » est une inférence — aucun
`EXPLAIN` n'a été relevé ; seul l'**ordre** des résultats l'a été. Lire **« équivalente à l'ordre observé »** pour
M10, ici comme à C-28-7-15.)*

**Mutations de la revue de code P1** (`kesh-gate-logs/28-7-mutations-p1-run.txt`, sorties
`28-7-mutations-p1/M*.log`, sous le verrou, 135 tests ; le test d'ordre a désormais trois factures à `D + 1`, et la
coupure du `LIMIT 50` traverse ce groupe — E-LOW-1 ; l'AC5 lit aussi `paid_at` et `entry_date` — E-LOW-3) :

| # | Mutation | Résultat | Tests rouges |
|---|---|---|---|
| M10 | `ORDER BY` retiré (rejouée sur le nouveau montage) | **survivante — équivalente à l'ordre observé** | aucun ; 135/135 |
| M13 | `ORDER BY i.date DESC, i.id DESC` (rejouée) | tuée | `…30_day_window`, `…orders_by_date…` |
| M14 | `ORDER BY i.id ASC` (rejouée) | tuée | `…orders_by_date…` |
| M16 | `ORDER BY i.date ASC` sans `i.id` | **survivante — équivalente à l'ordre observé** | aucun ; 135/135 |
| M17 | `entry_date: bank_transaction.booking_date` (écriture d'encaissement) | tuée | AC5 |
| M18 | `paid_at_dt` calculé sur `bank_transaction.booking_date` | tuée | AC5 |

M16 survit pour la même raison que M10 : sur MariaDB 10.11 de dev, les factures de même date sortent en `id`
croissant sans départage explicite — le montage établit pourtant que la coupure tombe dans un groupe de même date
(assertion de montage), si bien que le test rougirait dès que cet ordre implicite changerait. L'`i.id` de l'`ORDER BY`
reste la seule garantie (C-28-7-10, C-28-7-15). Bilan de la story : **18 mutations distinctes (M1-M18), 16 tuées, 2
équivalentes à l'ordre observé (M10, M16)** ; M13 et M14, rejouées, ne comptent qu'une fois.

**Gates.** Au dernier commit de code, rebasé (`7b10e555`), mesurés au HEAD `0d94fdc7` (seuls des fichiers de
documentation après le code), sous le verrou, après remise à zéro de `kesh_287` et `kesh_e2e_287` (80 migrations,
seed) — `kesh-gate-logs/28-7-gates.txt` :
- `scripts/test-fast.sh` (fmt, clippy `-D warnings`, nextest) : **3560 passés / 3560**, 6 ignorés ;
- `npm run check` : 0 erreur (29 avertissements, préexistants, aucun dans un fichier touché) ;
  `npm run lint-i18n-ownership` : vert ; `npm run test:unit` : **1547 / 1547** (144 fichiers) ; `npm run build` : vert ;
- E2E complet (port 3040, `/health` : `smtpConfigured: true`) : **273 passés, 7 échecs, 19 ignorés**, 11,1 min. Les
  7 échecs sont exactement les sept de la **KF-029** (#97) de `docs/testing.md` § *Les échecs attendus* —
  `mode-expert.spec.ts:26` et `:41`, `onboarding-path-b.spec.ts:65` et `:92`, `onboarding.spec.ts:57`, `:77`, `:150`.
  Aucun échec hors liste ; aucun `Permission denied` dans le journal du backend.
- Ciblés en cours de route : `binary(textes_coherents)` 15/15 ; `kesh-i18n` 39/39 ; Vitest de
  `failed-proposal-label`, G13 et `i18n-keys` 79/79.

**Rebase sur `origin/main` `4687af69` (28-4a fusionnée), 2026-10-13.** Code rebasé sans conflit : dans
`accept_one_invoice`, la garde de signe de la 28-6 puis l'étape 6 (statut, payée, écriture, fenêtre
`InvoiceDateWindow`) restent dans cet ordre ; les gardes « compte clos » de la 28-4a sont des pré-vols de lot (R1
dans `post_accept`, avant `accept_one_*`) et ne touchent pas l'étape 6. Conflits documentaires seulement : registre
des choix (union, section 28-7 après la 28-4, séparateur en double retiré), registre de sprint (union par clé,
en-tête reconstruit depuis `main`, `yaml.safe_load` vert), CHANGELOG (union, une seule section 0.13.3). Manuel non
touché par la 28-4a : PDF inchangé. Aucune route ajoutée par la 28-7 : compteurs du registre des routes inchangés,
tenus par `audit_route_registry`. Gates au HEAD `1baa2e89`, sous le verrou, bases `kesh_287`/`kesh_e2e_287` remises à
zéro (81 migrations) — `kesh-gate-logs/28-7-r3-gates.txt` : `scripts/test-fast.sh` **3603/3603** (6 ignorés),
`npm run check`, `lint-i18n-ownership`, `build` verts, Vitest **1566/1566**, E2E complet (port 3040) **277 passés,
7 échecs, 19 ignorés** — les sept de la KF-029 (#97) : `mode-expert.spec.ts:26`, `:41`,
`onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`.

### Completion Notes List

- **Décomptes**, de `9f4e5237` (étape 0, rebasé) à `7b10e555` (dernier commit de code), recomptés à la source :
  **7 tests neufs** — 2 `#[sqlx::test]` dans `reconciliation_e2e.rs` (94 → 96), 1 dans
  `reconciliation_repository.rs` (12 → 13), 4 `#[test]` dans `repositories/reconciliation.rs` (0 → 4) ;
  `invoice_write_off.rs` inchangé (20) ; une assertion ajoutée à un test Vitest existant. **15 fichiers** hors
  `_bmad-output` de `9f4e5237` au HEAD (12 au dernier commit de code, avant manuel, PDF et CHANGELOG).
  `i18nMsg(` = **1962** aux deux bornes (aucune clé ajoutée ni retirée). *(Annotation du 2026-10-13, revue de code
  P1, B-LOW-2 = A-LOW-3 — commande et périmètre : nombre de **lignes** qui contiennent `i18nMsg(` dans les seuls
  fichiers `.ts` et `.svelte` de `frontend/src`, tests compris :
  `git grep -c 'i18nMsg(' <ref> -- 'frontend/src/*.ts' 'frontend/src/*.svelte' | awk -F: '{s+=$NF} END {print s}'`,
  qui rend 1962 à `9f4e5237` comme à `7b10e555`. Les 1970 des lentilles comptent tout `frontend/src` (autres
  extensions comprises), les 1986 les occurrences (`-o`) : l'invariant, l'égalité aux deux bornes, tient pour
  chacune.)*
- Propagation grepée (motif de la fiche, § *Propagation*) après le patch : restent les occurrences légitimes (« 30
  jours net », jetons de session, `window_days` du contrat et de ses tests, entrées historiques du CHANGELOG) ;
  `marketing-brochure.tex:167` (« sur une fenêtre de 30 jours ») trié, **non modifié** (la fenêtre garde 30 jours
  côté facture ancienne) ; `docs/`, `website/`, README : aucune occurrence. de-CH, en-CH, it-CH : texte de l'AC7
  contrôlé au `Read`.
- PDF contrôlé aplati et normalisé (`kesh-gate-logs/28-7-user-manual-flat.txt`) : « peut être proposée » absente,
  aucun `??`, le titre « Une facture datée de plus d'un jour après le paiement n'est pas proposée. » et le renvoi
  de la FAQ (§ 14.1.1) présents.
- Deux rebases : sur `story/28-6` (`8ba4e168`), puis, la 28-6 fusionnée, `--onto origin/main 8ba4e168`. Au second, le
  PDF du manuel (binaire) a été pris de `main` puis **régénéré** depuis le `.tex` fusionné (commit `0d94fdc7`).
- **Aucune migration** : pas de bump `min_required`, pas de ligne d'audit d'idempotence, pas d'entrée de rejeu.

### File List

- `crates/kesh-db/src/repositories/reconciliation.rs`
- `crates/kesh-db/tests/reconciliation_repository.rs`
- `crates/kesh-db/tests/invoice_write_off.rs`
- `crates/kesh-api/src/routes/reconciliation.rs`
- `crates/kesh-api/tests/reconciliation_e2e.rs`
- `crates/kesh-i18n/locales/{fr-CH,de-CH,en-CH,it-CH}/messages.ftl`
- `frontend/src/lib/features/reconciliation/failed-proposal-label.ts`
- `frontend/src/lib/features/reconciliation/failed-proposal-label.test.ts`
- `frontend/tests/e2e/reconciliation-amount-due.spec.ts`
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`
- `CHANGELOG.md`
- `_bmad-output/implementation-artifacts/28-7-facture-posterieure-paiement.md`, `epic-28-choix-autonomes.md`,
  `sprint-status.yaml`, `28-7-review-prompt-p1.md`

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
- **2026-10-13 — Validation P2** (`28-7-validate-prompt-p2.md`, Opus 5.5, lentilles R et F en contexte frais ;
  rapports hors dépôt `kesh-gate-logs/28-7-validate-p2-{R,F}.md`). **F : 0 C / 0 H / 1 M / 7 L ; R : 0 C / 0 H /
  1 M / 7 L** — soit **2 MEDIUM distincts** (F1, R1) et 14 LOW dont **10 distincts** (F4 = R2, F5 = R3 = R4,
  F8 = R6). Les deux lentilles confirment l'algèbre sans ramènement, la démonstration « #529 non aggravé », la
  syntaxe et la place de l'`ORDER BY`, la faisabilité du test à 52 factures, M1-M9 et M11-M14 tuables, et les
  recomptes de la P1. Origine : F1, F3, F4 = R2, F5 = R3 = R4, F8 = R6 viennent de la remédiation P1 ; R1, F2,
  F6, F7, R5, R7, R8 de la conception d'origine. Remédiation (remédiateur Opus 5.5) :
  - **F1 (MEDIUM)** : AC8, puce `:1921` — le règlement depuis la fiche est décrit tel que l'écran le montre
    (`SettleInvoiceDialog.svelte`, `WriteOffDialog.svelte` : aucune date antérieure à la facture,
    `min={invoiceDate}`), sans tolérance d'un jour ; nouveau choix **C-28-7-11**.
  - **R1 (MEDIUM)** : titre d'exemple de l'AC8 « Une facture datée de plus d'un jour après le paiement n'est pas
    proposée. », et l'exigence que le titre comme le corps disent « plus d'un jour après ».
  - **LOW** : F2 (les deux dialogues à l'inventaire (b), greps copiés, sort « non résolu, assumé » ; l'AC9 les
    renvoie à l'inventaire ; l'inventaire (a) ne dit plus que le frontend « ne filtre aucune date ») ; F3 (le fait
    écrit : l'import CAMT accepte toute année, précédent `letterings.rs:507-511` ; angle mort accepté, sans test
    ni ramènement, **C-28-7-12** ; la phrase fausse de C-28-7-4 barrée et annotée, non effacée) ; F4 = R2 (« ne se
    desserre que par une migration », à l'AC9 et à l'inventaire (b), sans mention de bump) ; F5 = R3 = R4 (AC2 et
    C-28-7-10 : l'ordre de l'index « non établi », l'`ORDER BY` le garantit ; parenthèse sur `candidates[0]`
    retirée ; M10 « tuée ou équivalente selon le plan observé au dev », en-tête de l'AC10 aligné) ; F6 (**M15**,
    `<` → `<=` sur `earliest`) ; F7 (80 `#[sqlx::test]`, commande `grep -c '^#\[sqlx::test'`) ; F8 = R6 (phrase du
    plafond de 50 retirée du CHANGELOG, manuel inchangé, **C-28-7-13**) ; R5 (AC7 et T5 :
    `expect(label).not.toContain('proposée')`) ; R7 (inventaire (c) : lecteurs des propositions `:614`, `:734`,
    `:849`, `:899`, `:1919`, `:2067`, `:3305`, `:4444`, `:5095` et le helper `:3616` ; T4 : `rejeu_interblocage_e2e.rs`
    retiré des lecteurs) ; R8 (`vat_report.rs:245` ajouté à la sortie copiée).
  - **Correspondance des numéros de mutation** (numérotation désormais stable) : rapports de création et P1 —
    **M10 = ramènement de borne** (retirée avec C-28-7-4) ; fiche depuis la remédiation P1 et rapports P2 —
    **M10 = `ORDER BY` retiré** ; M11-M14 ajoutées en P1, **M15** en P2.
  - Symptômes grepés dans la fiche, le registre et le fichier d'epic (`donc sans bump|sans migration|ordre de
    fait|candidates\[0\]|même tolérance d'un jour|aucun chemin du|\b81\b|les 50 plus anciennes|datée après le
    paiement|peut survivre|rejeu_interblocage|chacune fait rougir`) : restent la tolérance d'un jour **du serveur**
    (AC9, C-28-7-7, vraie), les sites voulus, et les entrées de passes passées, non réécrites.
  - **Recompte après remédiation** : 11 AC ; 9 tâches (T0-T8) ; **15 mutations** (M1-M15) ; **13 choix numérotés,
    12 en vigueur** (C-28-7-4 retiré ; C-28-7-11 à 13 ajoutés) ; modules inchangés, cinq (les dialogues ne sont
    pas touchés). Tests neufs prescrits inchangés (deux HTTP, un module unitaire, un de dépôt) ; une assertion
    ajoutée à un test existant (R5).
  - Passe ciblée P3 à lancer : `28-7-validate-prompt-p3-ciblee.md`, **Haiku**, une lentille, sur le diff aplati de
    cette remédiation.
- **2026-10-13 — Validation P3 ciblée** (`28-7-validate-prompt-p3-ciblee.md`, Haiku, une lentille, sur `git show
  bbace781` ; rapport hors dépôt `kesh-gate-logs/28-7-validate-p3-ciblee.md`). **0 C / 0 H / 1 M / 7 L.** Les neuf
  axes du prompt déclarés exercés, trois partiellement (rapport, § *Axes non exercés*). Aucun finding n'est une
  erreur de catégorie. Origine : P3-1 à P3-7 viennent de la remédiation P2, P3-8 de la conception d'origine. Le
  MEDIUM vise le registre, non une ligne de code ni une prescription : **la validation est close** (décision de
  l'orchestrateur), et les huit findings sont remédiés à l'étape 0 du développement :
  - **P3-1 (MEDIUM)** : C-28-7-12 disait « L'ancien SQL rendait `NULL` » comme un fait, que C-28-7-4 déclare non
    mesuré ; la prémisse est dite **supposée**, et le choix porte une annotation datée.
  - **LOW** : P3-2 (C-28-7-12 : une erreur ferait échouer toute la réponse de propositions du compte, le `?` de
    la boucle de `get_proposals`) ; P3-3 (inventaire (a) : le frontend ne calcule aucune fenêtre, il transmet le
    candidat de tête, `p.candidates[0]`, que l'acceptation revalide) ; P3-4 (C-28-7-12 : le précédent
    `letterings.rs` s'est résolu par une borne en 400) ; P3-5 (AC8 et inventaire (b) : `SettleInvoiceDialog` est
    aussi monté par la page des échéances) ; P3-6 (T4 : `rejeu_interblocage_e2e.rs` appelle `POST /accept` **et**
    `POST /manual`) ; P3-7 (M10 : la tension « aucune issue prédite » / « la tuerait » levée) ; P3-8 (inventaire
    (c) : `reconciliation_rules_e2e.rs`, 8 lectures de propositions).
  - Symptômes grepés (`ancien SQL rendait|ne choisit aucune facture|n'appelle que|aucune issue n'est
    prédite|depuis la fiche`) dans la fiche, le registre et le fichier d'epic : restent la citation de C-28-7-4
    (texte retiré, conservé), l'annotation neuve, les entrées des passes passées, et « depuis la fiche » là où il
    désigne le chemin serveur (`settle_invoice`, `write_off_invoice`), appelé par les deux écrans.
  - Trend de la validation : P1 5 MEDIUM (4 distincts) → P2 2 MEDIUM → P3 ciblée 1 MEDIUM (rédaction du
    registre) → close. Modèles : Sonnet, Opus, Haiku.
- **2026-10-13 — Développement** (`bmad-dev-story`, Opus 5.5). Étape 0 (`9f4e5237`) : rebase sur `story/28-6`,
  MEDIUM et LOW de la P3 remédiés, lignes recalées. Rouge d'abord (`91e1494d`), correction serveur (`ee36da36`),
  libellé (`7b10e555`, dernier commit de code), manuel, PDF et CHANGELOG (`b2cdf995`), PDF régénéré après le second
  rebase sur `origin/main` `5629acc2` (`0d94fdc7`). Mutations : 14 tuées, M10 équivalente au plan observé. Gates au
  HEAD `0d94fdc7` : nextest 3560/3560, Vitest 1547/1547, check, lint-i18n-ownership et build verts, E2E 273 passés
  + 7 KF-029. Choix C-28-7-14 à 16. Détail au Dev Agent Record. Statut **review** ; revue de code P1 à lancer :
  `28-7-review-prompt-p1.md` (Sonnet, lentilles B, E, A).
- **2026-10-13 — Revue de code P1** (`28-7-review-prompt-p1.md`, Sonnet, lentilles B, E et A en contexte frais ;
  rapports hors dépôt `kesh-gate-logs/28-7-review-p1-{B,E,A}.md`). **B : 0 C / 0 H / 0 M / 2 L ; E : 0 / 0 / 0 / 3 ;
  A : 0 / 0 / 0 / 4** — soit **0/0/0/9** bruts, 7 distincts (B-LOW-1 = A-LOW-1, B-LOW-2 = A-LOW-3). Aucun MEDIUM :
  **boucle close** (décision de l'orchestrateur). Origine : 5 développement, 1 conception d'origine (E-LOW-3), 1
  remédiation P1 de la validation (A-LOW-4). Remédiation **sans ligne de production** :
  - B-LOW-1 = A-LOW-1 : manuel, « ce qui est proposé est accepté » → « une facture proposée n'est plus refusée pour
    sa date » ; A-LOW-2 : la parenthèse « sa date, pas son échéance » rattachée à « la date de la facture », « au
    plus un jour avant elle ». PDF régénéré (`make fr` sous le verrou ; les PDF d'administration et de la brochure,
    au texte identique, remis à leur état commité), contrôlé aplati : formule absolue absente, aucun `??`.
  - E-LOW-1 : le test d'ordre a trois factures à `D + 1`, la coupure du `LIMIT 50` traverse ce groupe (assertion de
    montage) ; mutation **M16** (`ORDER BY i.date ASC` sans `i.id`) jouée : **survivante, équivalente à l'ordre
    observé**, comme M10 (Dev Agent Record, C-28-7-15 annoté).
  - E-LOW-2 : M10 et C-28-7-15 annotés « équivalente à l'ordre observé » (aucun `EXPLAIN` relevé).
  - E-LOW-3 : l'AC5 asserte `paid_at` et la date de l'écriture d'encaissement aux cas (b) et (d) ; **M17**
    (`entry_date` sur la date de comptabilisation) et **M18** (`paid_at` sur la date de comptabilisation) tuées.
  - B-LOW-2 = A-LOW-3 : commande et périmètre du décompte `i18nMsg(` déclarés (annotation datée).
  - A-LOW-4 : phrase de l'AC2 « Aucun test ne lit l'ordre du SQL » annotée.
  - Symptôme grepé (`ce qui est proposé est accepté`) sur `docs`, CHANGELOG, README, `website`, `_bmad-output`,
    `crates`, `frontend/src` : aucune autre occurrence.
  - Gates (`kesh-gate-logs/28-7-p1-gates.txt`), sous le verrou, après remise à zéro de `kesh_287` et `kesh_e2e_287` :
    `scripts/test-fast.sh` **3560/3560** (6 ignorés), Vitest **1547/1547** (144 fichiers). E2E non rejoué : ni le code
    de production ni les specs E2E n'ont bougé depuis `7b10e555` (E2E au HEAD `0d94fdc7` : 273 + 7 KF-029).
  - Recompte : `kesh-api/tests/reconciliation_e2e.rs` 96 `#[sqlx::test]`, `kesh-db/tests/reconciliation_repository.rs`
    13 — inchangés (assertions ajoutées à deux tests existants). Mutations : 18 distinctes, 16 tuées, 2 équivalentes à
    l'ordre observé.
  - Trend de la revue : P1 0 MEDIUM → close. Modèle : Sonnet.
- **2026-10-13 — Rebase après la revue P1.** `origin/main` a avancé pendant la finalisation (`5629acc2` →
  `d86832f2`, Story 28-5b) : branche rebasée ; conflits du registre de sprint (union par clé, en-tête reconstruit
  depuis celui de `main`) et du CHANGELOG (union : l'entrée #526 de la 28-5b et celle de #548 dans la même section
  `## [0.13.3]`) ; PDF du manuel pris de `main` puis régénéré depuis le `.tex` fusionné (28-5b et 28-7), contrôlé
  aplati (formule absolue et ancienne phrase absentes, aucun `??`). Gates complets **et** E2E au HEAD rebasé, sous le
  verrou, après remise à zéro des bases (`kesh-gate-logs/28-7-r2-gates.txt`) : `scripts/test-fast.sh` **3560/3560**
  (6 ignorés), Vitest **1566/1566**, check, lint-i18n-ownership et build verts, E2E **277 passés, 7 échecs — les sept
  de la KF-029** (#97, `docs/testing.md`), 19 ignorés. Aucune ligne de production modifiée depuis le dernier commit
  de code (`git diff --stat` vide sur `crates/*/src` et `frontend/src` hors tests).
