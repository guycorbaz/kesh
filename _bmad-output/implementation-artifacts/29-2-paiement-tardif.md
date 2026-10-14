# Story 29.2 : Un paiement reçu après l'échéance, ou après un rappel, est proposé

## Status

**review** — développement fait le 2026-10-14 (`bmad-dev-story`, Opus 5.5) : commits `8f483388` (tests et code) et
`d45279f7` (documentation, derniers doc-comments) ; gate backend 3611/3611, Vitest 1610, E2E 279 passés + 8 échecs
attendus ; 20 mutations jouées (17 tuées dont M14bis, ajoutée ; M12, M13 et M14 équivalentes) ; choix C-29-2-15 et 16,
annotation de C-29-2-8 ; revue de code P1 à lancer (`29-2-review-prompt-p1.md`, Sonnet, lentilles B/E/A). Cf. Dev
Agent Record.

*Statut antérieur :* **ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `16eeed21`
(kickoff de l'Epic 29), branche `story/29-2`, worktree `/home/gcorbaz/devel/kesh-29-2`. Validation P1 faite
(Sonnet, lentilles R et F : 1 HIGH, 3 MEDIUM et 8 LOW distincts après fusion des doublons, remédiés — cf. Change
Log) ; validation P2 faite (Opus, lentilles R et F : 2 MEDIUM et 12 LOW distincts après fusion, remédiés — cf.
Change Log) ; passe ciblée P3 à lancer (`29-2-validate-prompt-p3-ciblee.md`, Haiku). Choix consignés au registre
`epic-29-choix-autonomes-29-2.md`, **C-29-2-1 à C-29-2-14**. 10 AC. PR : **`closes #529`** (dans le titre ou le corps
de la PR seulement — aucun mot-clé de fermeture dans un message de commit, même en prose : B4 de la rétrospective 28).

⚠️ **Sérialisée avant la 29-3** (#527, tri des candidates), qui touchera après elle les mêmes zones de
`crates/kesh-api/src/routes/reconciliation.rs` et de `crates/kesh-db/src/repositories/reconciliation.rs` (§ *Zones
partagées*). Les numéros de ligne de cette fiche sont relevés sur `16eeed21`.

⛔ **La Story 29-5 (#668, départage à score égal) doit être mergée avant le tag v0.13.4** (décision de l'orchestrateur,
C-29-2-13) : la 29-2 élargit les candidates, la 29-5 corrige le départage qui les ordonne, et aucune release ne livre
l'une sans l'autre.

## Story

**As a** utilisateur qui rapproche ses relevés (Comptable ou Admin),
**I want** qu'un client qui paie après l'échéance de sa facture, ou après un rappel, voie son paiement proposé contre
cette facture,
**so that** je ne doive plus enregistrer à la main, depuis la fiche, un règlement que la réconciliation devrait
proposer — et ne risque plus de passer le paiement en produit par une règle d'affectation (produit compté deux fois).

Issue : [#529](https://github.com/guycorbaz/kesh/issues/529) (P2, bug). Epic :
`_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-2*. Cible : **v0.13.4**.

### Les prémisses, établies au code (`16eeed21`) — deux sont périmées (C-29-2-1)

| Prémisse (#529, 2026-10-08 ; consigne de l'orchestrateur) | Au code | Verdict |
|---|---|---|
| « fenêtre de 30 jours **centrée** sur la date de la facture » (`get_proposals`) | Depuis la Story 28-7 (#548), **une seule** fenêtre, `InvoiceDateWindow` (`crates/kesh-db/src/repositories/reconciliation.rs:150-199`) : `D − 30 ≤ F ≤ D + 1`, `D` date de valeur (à défaut de comptabilisation), `F` date de facture. Lue par la proposition (`find_unpaid_invoices_for_window`, `:234-274`, appelée par `routes/reconciliation.rs:656-662`) **et** l'acceptation (`accept_one_invoice`, étape 6, en-tête `routes/reconciliation.rs:1716`, fenêtre `:1740-1765`) | **Périmée** dans la forme ; **le défaut subsiste** : la borne basse est `F ≥ D − 30`, sur la date de la facture, jamais sur son échéance ni ses rappels |
| « l'écran n'offre aucun moyen de choisir la facture » (*Affecter manuellement*, *Éclater* : classes 5 à 7) | Exact : `ManualMatchModal.svelte:80-90` filtre les classes 5/6/7 ; la ligne n'affiche que `p.candidates[0]` (`ReconciliationProposals.svelte:337-368`) | **Vraie** — et non traitée ici (C-29-2-2) |
| « le manuel (vers :1126, rappels) **promet** que le rapprochement propose la facture » | Depuis la v0.13.3, `user-manual.tex:1305` dit l'inverse : « ne propose toutefois la facture que si le paiement arrive dans les 30 jours de la date de la facture --- ce qui, après un rappel, est rarement le cas ~: enregistrez alors le règlement depuis la fiche » ; de même `:2029` et `:2902` | **Périmée** : le manuel dit vrai, il décrit le trou. Il change parce que le trou se ferme (AC8) |
| Le contournement (règlement depuis la fiche) est complet | `settle_invoice`, choix `BankTransfer` (`invoice_settlements_write.rs:130-154`) débite le compte comptable de la banque : la transaction bancaire reste **en attente** ; l'accepter ensuite autrement (règle, affectation) passerait le montant **deux fois** | **Incomplète** : le manuel ne dit pas de **rejeter** la transaction ensuite. Ajout mécanique au paragraphe réécrit (AC8, C-29-2-8) |

### La règle retenue (C-29-2-2)

**La borne basse s'ancre sur la plus tardive des dates qui rendent le paiement exigible ou le réclament.** Pour une
facture de date `F`, d'échéance `E` (nullable) et dont le dernier rappel **non annulé** est daté `R` (nullable), on
pose la **date d'ancrage**

> `A = max(F, E, R)` — `E` et `R` absents valant `F`

et une transaction de date de paiement `D` peut régler la facture si

> `F ≤ D + 1 jour` **et** `A ≥ D − 30 jours`

c'est-à-dire : le paiement ne précède pas la facture de plus d'un jour (**inchangé**, #548), et il n'arrive pas plus
de 30 jours après **la plus tardive** de la date de la facture, de son échéance et de son dernier rappel. Une facture
sans échéance ni rappel garde exactement la fenêtre d'aujourd'hui (`A = F`).

L'effet ne se borne pas au paiement **en retard** : un paiement reçu **avant** l'échéance mais plus de 30 jours après
la date de la facture (conditions à 60 jours net, payées à l'heure) n'était pas proposé non plus, et l'est désormais
(`A = E`, dans le futur ou récente). Le cas le plus courant — paiement **avant** l'échéance, dans les 30 jours de la
facture — reste proposé ; la borne haute porte **toujours sur `F`**, jamais sur `A` (sans quoi toute facture non
échue disparaîtrait : mutation M10).

⚠️ **Ce que la règle ne règle pas : le montant.** Elle ne touche qu'aux **dates** ; la facture n'est proposée qu'au
**montant de son reste dû** (tolérance 0,05). Un client qui paie le montant du courrier de rappel, frais compris
(#401, `user-manual.tex:1307`), n'est toujours pas proposé ; et un trop-perçu de 1 à 5 centimes est proposé puis
refusé (#665, divergence de montant entre proposition et acceptation, ouverte, hors périmètre).

Exemples, `D` = 2026-06-15 :

| Facture `F` | Échéance `E` | Dernier rappel `R` | `A` | Proposée et acceptée ? |
|---|---|---|---|---|
| 2026-05-01 | — | — | 05-01 | non (`A < D − 30` = 05-16) — **inchangé** |
| 2026-05-01 | 2026-05-31 (30 jours net) | — | 05-31 | **oui** (cas courant de #529) |
| 2026-03-01 | 2026-03-31 | 2026-06-01 | 06-01 | **oui** (payé après un rappel) |
| 2026-03-01 | 2026-03-31 | 2026-06-01, **annulé** | 03-31 | non |
| 2026-03-01 | 2026-03-31 | — | 03-31 | non — reliquat (C-29-2-9) |
| 2026-06-17 | 2026-07-17 | — | 07-17 | non (`F > D + 1`) — **inchangé**, #548 |
| 2026-06-05 | 2026-07-05 (30 jours net) | — | 07-05 | **oui** (payé avant l'échéance) — **inchangé** |
| 2026-04-15 | 2026-06-14 (60 jours net) | — | 06-14 | **oui** (payé à l'heure, plus de 30 jours après la facture) |

**Une seule borne, deux lecteurs** (exigence de l'orchestrateur, #548 non rouvert) : `A` a **une** définition, une
expression SQL (`INVOICE_PAYMENT_ANCHOR_SQL` sur la jointure `invoice_payment_anchor_joins()`), lue par la
proposition (forme jointe, dans le `WHERE`) **et** par l'acceptation (forme scalaire, même expression, même jointure,
`WHERE i.id = ?`) ; la comparaison à `D` reste `InvoiceDateWindow::check`, en Rust, pour l'acceptation, et le `WHERE`
de la proposition, tenus d'accord par le test de parité étendu (AC6). Le refus d'un débit (#482,
`RECONCILIATION_TRANSACTION_NOT_CREDIT`, étape 4bis) précède la lecture de la facture et n'est pas touché.

**Pourquoi pas supprimer la borne basse** (écartée, C-29-2-2) : à montant exact, toute facture impayée de même reste
dû devient candidate **à score égal** (0,5 pour le seul montant) ; `propose_matches` départage alors par
`invoice_id` croissant (`kesh-reconciliation/src/matching.rs:97-106`, départage `:104`) — la **plus ancienne** gagne, et c'est la seule
affichée et acceptée par *Accepter (N)*. Une vieille facture oubliée d'un client capterait le virement d'un autre :
le **faux rattachement muet** que le `CLAUDE.md` (§ *Un appariement automatique propose, il ne crée jamais*) désigne
comme le danger. L'ancre sur l'échéance et les rappels **réduit** ce risque aux factures dont `A ≥ D − 30` : échues
depuis moins de 30 jours, relancées depuis moins de 30 jours, **ou pas encore échues**. Ce dernier ensemble **n'est
pas borné dans le temps** : une facture à échéance lointaine (`due_date` n'a pas de borne haute, la seule garde,
`routes/invoices.rs:765`, est `due_date < date`) — ou une coquille à 2099 — reste candidate pour tout paiement de même
reste dû de `F − 1` à `E + 30`, et, à score égal, passe devant une facture **échue** (départage `invoice_id ASC`, qui
ignore l'exigibilité). Le cas ordinaire s'aggrave aussi : à 30 jours net, les candidates passent de `F ≥ D − 30` à
`F ≥ D − 60` (`A = F + 30 ≥ D − 30`), si bien que deux mensualités impayées d'un même client, de même reste dû et sans
référence discriminante, deviennent candidates ensemble. C'est un **angle mort écrit** (C-29-2-11), propre à toute
prise en compte de l'échéance, **porté par [#668](https://github.com/guycorbaz/kesh/issues/668)** (`bug`, P2) et
traité par la **Story 29-5**, qui sort dans la même release v0.13.4 et doit être mergée avant son tag (C-29-2-13).
L'attendu de #668, rectifié au regard de l'art. 87 CO (paiement sans désignation : dette exigible d'abord, puis la plus
ancienne), est : à score égal, les factures échues d'abord, puis l'ancienneté. La 29-2 ne traite pas ce départage ; elle
ne le crée pas pour les factures sans échéance ni rappel (`A = F`, inchangé).

**Pourquoi pas le choix manuel d'une facture à l'écran** (voie 2 de #529, écartée, C-29-2-2) : elle exige que
l'acceptation admette ce que la proposition ne propose pas (deux bornes, ce que l'orchestrateur exclut) ou une
nouvelle voie d'acceptation sans borne, plus un dialogue neuf (liste des factures ouvertes, sélection, i18n quatre
locales, E2E, captures) : hors du périmètre minimal. Le reliquat qu'elle couvrirait (paiement plus de 30 jours après
l'échéance **et** sans rappel) est écrit au manuel avec son contournement complet (AC8) ; il est couvert par la
demande d'évolution [#666](https://github.com/guycorbaz/kesh/issues/666) (C-29-2-9).

## Critères d'acceptation

**AC1 — La date d'ancrage a une seule définition, en SQL.**

Dans `crates/kesh-db/src/repositories/reconciliation.rs` :

- `INVOICE_PAYMENT_ANCHOR_JOIN_SQL` (ou une fonction `invoice_payment_anchor_joins() -> &'static str`) : la table
  dérivée des rappels, **prérequis alias `i` sur `invoices`**, sur le patron de `INVOICE_SETTLED_DERIVED_JOIN_SQL`
  (`invoice_settlements.rs:42-43`) :
  `LEFT JOIN (SELECT invoice_id, MAX(sent_at) AS last_sent_at FROM invoice_reminders WHERE cancelled_at IS NULL GROUP BY invoice_id) rmd ON rmd.invoice_id = i.id`.
  Alias `rmd` : distinct de `lt`, `cnt`, `cl_ttc`, `st` des jointures du reste dû, avec lesquelles il cohabite dans
  la requête de proposition (vérifier par `grep -n ') rmd\| rmd ' crates/kesh-db/src` qu'il est libre).
- `INVOICE_PAYMENT_ANCHOR_SQL` :
  `CAST(GREATEST(i.date, COALESCE(i.due_date, i.date), COALESCE(DATE(rmd.last_sent_at), i.date)) AS DATE)`.
  Le `CAST(… AS DATE)` fait du type rendu une **garantie** plutôt qu'une constatation (C-29-2-10) : aucun
  `GREATEST` sur des dates n'existe au dépôt (`grep -rn GREATEST crates` : `backup.rs:708`, `:729`, sur des entiers),
  donc aucun précédent de décodage `NaiveDate` par sqlx sur ce type.
  ⛔ **Chaque argument porte son `COALESCE`** : en MariaDB, `GREATEST` rend `NULL` dès qu'un argument est `NULL`, et
  `NULL >= ?` est faux — une facture sans échéance (`due_date` nullable, `migrations/20260416000001_invoices.sql:22`)
  ou sans rappel **disparaîtrait** des candidates. Le `GREATEST` sur `i.date` couvre aussi une échéance antérieure à la
  date (données d'avant #245 : la garde `due_date >= date` de `routes/invoices.rs:765`, `:816` ne porte que sur une
  paire modifiée).
- Doc-comments `///` : ce que vaut `A`, pourquoi les rappels **annulés** n'y comptent pas (C-29-2-3 : l'annulation
  soft d'un Admin retire déjà la ligne du niveau courant, `entities/invoice_reminder.rs:5-7`), pourquoi `DATE(sent_at)`
  (C-29-2-3 : `sent_at` est un `DATETIME(6)` ; un écart de fuseau d'un jour au plus, sans effet mesurable sur une borne
  de 30 jours — angle mort écrit), et **qui le lit** (les deux lecteurs d'AC2).
- Le doc-comment de la jointure **cite le site jumeau** de la notion « dernier rappel non annulé » :
  `repositories/dunning_eligibility.rs:80` (`MAX(sent_at) AS last_reminder_at … WHERE company_id = ? AND
  cancelled_at IS NULL GROUP BY invoice_id`, lu `:128` par `lr.date()`), pour que qui change la notion d'annulation
  change les deux. Pas de factorisation (C-29-2-12) : les agrégats diffèrent (`MAX(level_number)` en plus là-bas) et
  la table dérivée de l'éligibilité prend un `company_id` lié, celle-ci aucun (patron des jointures du reste dû, sans
  paramètre ; la jointure par `invoice_id` porte le scope, la facture étant filtrée par `i.company_id = ?`).

**AC2 — Deux lecteurs, la même expression.**

- **Proposition** : `find_unpaid_invoices_for_window` (`:234-274`) joint `invoice_payment_anchor_joins()` à côté de
  `amount_due_derived_joins()`, et remplace `AND i.date BETWEEN ? AND ?` par
  `AND i.date <= ? AND {INVOICE_PAYMENT_ANCHOR_SQL} >= ?` (bind `window.latest_invoice_date` puis
  `window.earliest_anchor_date`). Filtres de statut, `HAVING` du reste dû, `ORDER BY i.date ASC, i.id ASC` et
  `LIMIT 50` **inchangés** (C-29-2-7). Le `WHERE` peut citer une colonne de table dérivée (ce n'est pas un alias de
  `SELECT`) ; le tri et la troncature restent ceux de la 28-7.
- **Acceptation** : une fonction neuve `invoice_payment_anchor_date(executor, company_id, invoice_id) ->
  Result<Option<NaiveDate>, DbError>` rend `SELECT {INVOICE_PAYMENT_ANCHOR_SQL} FROM invoices i {joins} WHERE
  i.company_id = ? AND i.id = ?` — **même constante, même jointure** : aucune seconde écriture de la règle, ni en SQL
  ni en Rust (C-29-2-4). Scopée société (KF-002) bien que la facture soit déjà chargée sous ce scope — et cette portée
  est **testée** (AC5, facture d'une autre société → `Ok(None)`) et **mutée** (M11) : une garde sans test se retire en
  silence au prochain refactor.
- ⛔ **La borne haute reste sur `i.date`**, des deux côtés : `i.date <= ?` dans le `WHERE`, `invoice_date` comparée à
  `latest_invoice_date` dans `check`. La porter sur l'ancre ferait disparaître toute facture non échue (mutation M10,
  cas j et k d'AC5).

**AC3 — La fenêtre compare la date de facture ET la date d'ancrage.**

`InvoiceDateWindow` :

- le champ `earliest_invoice_date` est **renommé** `earliest_anchor_date` (`D − 30`) — l'ancien nom mentirait ;
  `latest_invoice_date` (`D + 1`) reste ;
- `check(&self, invoice_date, anchor_date)` : `PaymentBeforeInvoice` si `invoice_date > latest_invoice_date`
  (**testé d'abord**, comme aujourd'hui), puis `PaymentTooLate` si `anchor_date < earliest_anchor_date` ;
- doc-comments à réécrire : celui de la structure (`:117-149`, dont « Sites qui portent la même borne basse » : la
  borne basse **de la date de facture** `F ≥ D − 1`… n'est pas celle-ci — relire et corriger le vocabulaire), de
  `INVOICE_WINDOW_DAYS_AFTER_INVOICE` (`:90-95` : « c'est la borne de #529 … que la Story 28-7 laisse où elle est »
  devient « au plus 30 jours après la date d'ancrage »), d'`InvoiceDateRefusal::PaymentTooLate` (`:163-165`) et de
  l'en-tête du module (`:5-13`). Le nom de la constante reste (contrat `window_days`), son doc-comment dit « après la
  date d'ancrage ».

**AC4 — L'acceptation lit l'ancre sous le même verrou, sans rien déplacer.**

Dans `accept_one_invoice` (`routes/reconciliation.rs:1621`), étape 6 (en-tête `:1716`, fenêtre `:1740-1765`) : après les trois gardes
d'éligibilité (`invoice_not_validated`, `invoice_already_paid`, `invoice_journal_entry_not_set`) et **avant**
`InvoiceDateWindow::check`, lire `invoice_payment_anchor_date(&mut **tx, company_id, invoice.id)` ; `Ok(None)` (facture
disparue entre les deux lectures) → `INVOICE_NOT_FOUND` avec `details.invoiceId` comme l'étape 5 ; `Err` →
`DATABASE_ERROR` avec `details.message`, comme les autres lectures de la fonction — **jamais** un `AppError` global
(§ *Pattern batch* du `CLAUDE.md`).

- ⚠️ **Angle mort assumé** : ces deux issues (`Ok(None)`, `Err`) ne sont **ni testées ni mutées** — la facture vient
  d'être lue dans la même transaction, et aucun crochet de panne n'existe pour faire échouer la seule seconde lecture.
  Elles se vérifient à la **revue de code** (lecture des deux bras, `FailedProposal` et jamais `?` vers un `AppError`),
  et le Dev Agent Record le déclare ainsi.
- **Contrat d'API inchangé** : mêmes codes, mêmes raisons (`payment_date_before_invoice_date`,
  `payment_date_outside_window`), `details.window_days` = 30. Seul le **sens** de `payment_date_outside_window`
  s'élargit (« plus de 30 jours après la date d'ancrage »), écrit à `docs/api-external.md` (AC8).
- **Verrous et naissance de la vue (B5 de la rétrospective 28)** : la lecture ajoutée est une `SELECT` ordinaire,
  **sans** `FOR UPDATE`, placée **après** la lecture ordinaire de la facture (étape 5,
  `find_invoice_by_id_for_company`, `reconciliation.rs:361-376`) et de son contact (étape 5bis) : elle ne prend aucun
  verrou et ne fait **pas** naître la vue de la transaction, déjà née à une lecture antérieure. Le Dev Agent Record le
  vérifie en citant, au `grep -n`, la première lecture ordinaire du chemin et la place de la lecture ajoutée.
- **Course admise** : un rappel annulé (ou envoyé) entre l'affichage des propositions et l'acceptation change `A` ;
  l'acceptation relit `A` dans sa transaction et tranche sur ce qu'elle lit — même statut que la facture redatée de
  la 28-7. Écrit au doc-comment, pas de test.
- Le commentaire de `:1740-1745` (dont la ligne `:1745`, « Paiement tardif au-delà de 30 jours : #529, rapprochement
  manuel ») est réécrit.
- La date du règlement écrit (`paid_at_dt`) reste `payment_date(&bank_transaction)` : **inchangée**.

**AC5 — Tests `kesh-db` (rouges d'abord).**

`crates/kesh-db/src/repositories/reconciliation.rs`, `mod tests` (unitaires) : les deux tests de bornes
(`:619-643`) récrits sur la nouvelle signature — `check(F, A)` admet `A = D − 30` et refuse `A = D − 31` ;
`F = D + 1` admis, `F = D + 2` refusé `PaymentBeforeInvoice` **même quand** `A` est dans la fenêtre (ordre des
refus) ; `F = D + 2` et `A = D − 31` simultanés (impossible si `A ≥ F`, mais `check` est public) → `PaymentBeforeInvoice` ;
**`check(D − 10, D + 20) == Ok(())`** (facture non échue : l'ancre au-delà de `D + 1` ne déclenche pas
`PaymentBeforeInvoice` — tue la forme acceptation de M10).

`crates/kesh-db/tests/reconciliation_repository.rs` : un helper `insert_test_reminder(pool, company_id, invoice_id,
sent_at, cancelled)` (colonnes `NOT NULL` : `level_number ≥ 1`, `fee_amount`, `sent_at`, `channel ∈ {email, manual}`,
`subject`, `body` — migration `20260715000001_invoice_reminders.sql:33-58`), et un `due_date` optionnel sur
`insert_test_invoice` (paramètre ajouté, ou `UPDATE invoices SET due_date = ?` après insertion — au choix du dev,
sans changer les appels existants : leur `due_date` reste `NULL`). Un test (ou un test par cas, au choix) sur une
transaction à `D` = 2026-06-15, montants **distincts par cas** (aucune facture candidate d'un autre cas), qui asserte
**à la fois** la liste rendue par `find_unpaid_invoices_for_window` **et** `invoice_payment_anchor_date` pour chaque
facture. La liste s'asserte comme un **`Vec` ordonné** — l'ordre de la requête, `(i.date, i.id)` —, **répétitions
comprises**, et non comme un ensemble : une table dérivée qui rendrait une ligne par rappel dédoublerait la facture f,
et un ensemble (comme le `BTreeSet` de `proposed_couples_287`) avalerait le doublon (M15). Le scalaire est asserté pour
**chaque** cas, candidat ou non : c'est la seule assertion qui voit l'ancre d'une facture refusée dans les deux formes
(cas g, M14) :

| cas | `F` | `E` | rappels | `A` attendu | candidate |
|---|---|---|---|---|---|
| a | D − 45 | `NULL` | — | D − 45 | non |
| b | D − 45 | D − 30 | — | D − 30 | **oui** (borne) |
| c | D − 45 | D − 31 | — | D − 31 | non (borne) |
| d | D − 90 | D − 60 | D − 10 | D − 10 | **oui** |
| e | D − 90 | D − 60 | D − 10 **annulé** | D − 60 | non |
| f | D − 90 | D − 60 | D − 50, puis D − 20 | D − 20 | **oui** (le **dernier** rappel, pas le premier) |
| g | D − 45 | D − 50 (échéance antérieure) | — | D − 45 | non (`GREATEST` garde `F` ; seul le scalaire le voit) |
| h | D + 2 | D + 32 | — | D + 32 | non (`F > D + 1`, #548) |
| i | D − 20 | `NULL` | — | D − 20 | **oui** (inchangé : `E` nul ne fait pas disparaître) |
| j | D − 10 | D + 20 | — | D + 20 | **oui** (payé avant l'échéance — le cas le plus courant ; inchangé) |
| k | D − 45 | D + 15 | — | D + 15 | **oui** (payé avant l'échéance, plus de 30 jours après la facture) |

Plus un cas **inter-factures** : le rappel d'une facture X ne déplace pas l'ancre d'une facture Y (jointure par
`invoice_id`). Plus un cas **inter-sociétés** : `invoice_payment_anchor_date(pool, autre_company_id, facture_d)` rend
`Ok(None)` (la facture du cas d existe, mais dans l'autre société), et `find_unpaid_invoices_for_window` sous l'autre société ne
la rend pas. Les tests existants du fichier (`:256-313` « 30 day window », `:315+` ordre et `LIMIT 50`) restent
verts **sans modification d'assertion** — leurs factures n'ont ni échéance ni rappel ; seul le nom du champ renommé
change là où ils le lisent (aucun, à vérifier au `grep -n earliest_invoice_date crates`).

**AC6 — Tests `kesh-api` : parité étendue, et le paiement tardif aboutit.**

`crates/kesh-api/tests/reconciliation_e2e.rs` :

- **Parité** : un test neuf sur le patron de `proposal_and_acceptance_share_the_invoice_date_window` (`:2888-2958`)
  — mêmes helpers `proposed_couples_287` / `accept_couples_287` —, sur les cas a à k d'AC5 (échéance posée par
  `UPDATE invoices SET due_date`, rappels insérés en SQL brut ; `seed_validated_invoice` n'a pas de paramètre
  d'échéance). Assertions : `proposed == accepted`, l'ensemble attendu exact, et pour chaque refus sa raison
  (`payment_date_outside_window` pour a, c, e, g ; `payment_date_before_invoice_date` pour h) ; b, d, f, i, j et k
  proposés **et** acceptés (j et k tuent les deux formes de M10). `D` en plein exercice
  2026 (`insert_fake_fiscal_year`) ; vérifier que `D − 90` y reste.
- **Le cas de #529 bout à bout** : transaction à `D`, facture à `D − 45` échue à `D − 15` ; `GET /proposals` la
  propose (`candidateType = invoice`, `invoiceId`), `POST /accept` l'accepte, et la facture est réglée : `paid_at`
  vaut **`D` à minuit** (`Some(D.and_hms_opt(0, 0, 0))`), assertion de **valeur** et non de présence. Patron :
  `proposals_use_the_value_date_like_the_acceptance` (`:2978`, assertions `settled_on` / `paid_at` / `entry_date`
  `:3051-3086`) ; `post_accept_reconciles_transaction_and_invoice` (`:973`) ne vérifie que `paid_at.is_some()`.
- **Assertions inchangées** : `post_accept_rejects_payment_date_outside_window` (`:2726-2789`, facture sans
  échéance : `A = F`, toujours refusée, `window_days == 30`), les deux tests de la 28-7 (`:2888`, `:2978`), et tout
  test qui lit les propositions (inventaire (c)). **Seule exception, un doc-comment** : celui de
  `proposal_and_acceptance_share_the_invoice_date_window` (`:2887`, « `−30` / `−31` figent la borne de #529 ») devient
  inexact — ce test fige désormais la borne d'une facture **sans échéance ni rappel** (`A = F`). Il est réécrit ; ses
  assertions ne bougent pas.

**AC7 — L'inventaire des tests dont le sens peut changer est clos, et relu au gate.**

Ensemble clos des fichiers qui lisent les propositions ou la fenêtre, **ou acceptent une facture** (§ *Inventaire
(c)*) : leurs factures n'ont pas d'échéance, ou une échéance égale à la date (`invoice_write_off.rs:81`,
`due_date = D()` ; `rejeu_interblocage_e2e.rs`, factures créées par l'API sans délai de contact, payées le jour même)
— `A = F`, sens inchangé. Le Dev Agent
Record le **revérifie** par la commande de l'inventaire (c) et en copie la sortie. **E2E** : les specs partagent la
base `kesh_e2e` et y laissent des factures **avec échéance** (contacts « 30 jours net » de `invoices.spec.ts`,
`api-keys.spec.ts`, `inbox-import.spec.ts`, `invoices_echeancier.spec.ts`) et, surtout, des factures **échues et
relancées** : `reminders.spec.ts`, `dunning-roundtrip.spec.ts` et `homepage-reminders.spec.ts` créent des factures
échues à `J − 25` (`overdueDate`, `helpers/api-fixtures.ts:18`) et leur envoient un rappel le jour `J` (`A = J`) ; la
base persistant d'un jour à l'autre, une telle facture, candidate jusqu'à `J + 5` avant la story, le reste jusqu'à
`J + 30` après. Une facture d'une de ces specs peut donc devenir candidate d'une transaction d'une spec de
réconciliation **de même montant**. Les specs de
réconciliation tirent des montants uniques (`reconciliation-amount-due.spec.ts`, `Date.now()`), mais le Dev Agent
Record déclare, fichier par fichier contre `docs/testing.md` § *Les échecs attendus*, tout rouge d'une spec de
réconciliation au gate E2E.

**AC8 — Le manuel, l'API externe et le CHANGELOG disent la nouvelle règle (PDF compris).**

`docs/manual/fr/user-manual.tex` (seul manuel rédigé : `docs/manual/{de,en,it}/` ne portent qu'un `README.md`) :

- **`:1305`** (§ rappels, « Ce que réclame un rappel ») : « La réconciliation bancaire ne propose toutefois la facture
  que si le paiement arrive dans les 30 jours de la date de la facture --- ce qui, après un rappel, est rarement le
  cas~: enregistrez alors le règlement depuis la fiche » devient : la réconciliation propose la facture si le paiement
  **du reste dû** arrive **au plus 30 jours après le dernier rappel** (ou après l'échéance, ou la date de la facture,
  la plus tardive des trois) ; un rappel **annulé** ne compte pas ; renvoi à `\S\ref{sec:facture-posterieure}` ou à
  l'algorithme. La condition de **montant** se dit dans la même phrase, ou par un renvoi explicite au `keshwarning`
  qui suit (`:1307` : le montant du courrier, frais compris, n'est pas proposé — #401) : un lecteur ne doit pas
  conclure que **tout** paiement après un rappel est proposé. Le `keshwarning` reste tel quel.
- **`:2029`** (§ *Algorithme de matching*) : « tombe au plus 30 jours après la date de la facture --- sa date, pas son
  échéance --- » devient la règle d'AC1, dite pour l'utilisateur (« au plus 30 jours après la plus tardive de ces
  dates~: la date de la facture, son échéance, son dernier rappel »). La borne voisine, « et \textbf{au plus un jour
  avant elle} », se réécrit « au plus un jour avant **la date de la facture** » : citée après « la plus tardive de ces
  dates », « elle » se lirait comme l'ancre, et le texte dirait qu'un paiement antérieur à l'échéance est refusé — la
  mutation M10 écrite au manuel. La phrase dit ensuite qu'un paiement reçu **avant**
  l'échéance est proposé lui aussi, même plus de 30 jours après la facture (conditions à 60 jours) ; la condition de
  montant (le reste dû, à la tolérance près) n'est pas reformulée là où l'algorithme la dit déjà ; la phrase « Un paiement reçu plus de 30 jours
  après la date de la facture --- une facture payée avec retard, ou après un rappel --- n'est en revanche pas
  proposé » devient le **reliquat** : plus de 30 jours après l'échéance **et** sans rappel depuis 30 jours ; le
  remède : enregistrer le règlement depuis la fiche, **puis rejeter la transaction** à la réconciliation
  (*Rejeter*), sans quoi elle reste en attente et pourrait être passée une seconde fois (C-29-2-8).
- **`:2031`** (`\label{sec:facture-posterieure}`) — deux phrases :
  - « une facture proposée n'est plus refusée pour sa date. Ce refus ne se voit plus que si la facture a été
    \textbf{redatée} » se **borne au refus « avant la facture »** (« une facture proposée n'est plus refusée parce
    qu'elle serait postérieure au paiement »), et une phrase dit l'autre refus de date que la course admise d'AC4 rend
    atteignable : si un **rappel est annulé** entre l'affichage des propositions et leur acceptation, l'ancre recule et
    la facture peut être refusée pour un paiement trop tardif (motif générique des *Échecs partiels*) — C-29-2-14 ;
  - dernières phrases : « Il n'a en revanche aucune limite de 30 jours après la facture~: c'est par lui qu'on
    enregistre un paiement reçu avec retard » devient « … un paiement que la réconciliation ne propose pas (reçu plus
    de 30 jours après l'échéance et le dernier rappel) », avec le même rejet de la transaction. Le `\label` est
    conservé (référencé à `:2902`).
- **`:2902`** (FAQ « aucun match automatique ») : « plus de 30 jours après la date de la facture --- de sa date, pas de
  son échéance~: une facture payée avec retard, ou après un rappel, n'est pas proposée » devient le reliquat.
- `:836`, `:914`, `:916`, `:946`, `:949` (« 30 jours net » des conditions de paiement) et `:95` (session) : hors
  sujet, **non modifiés**.
- `docs/manual/fr/marketing-brochure.tex:167` (« Réconciliation automatique avec scoring (… sur une fenêtre de 30
  jours …) ») : **non modifiée**, trié et déclaré au Dev Agent Record — la fenêtre reste de 30 jours, seule son ancre
  change, et une brochure ne détaille pas l'ancre.

PDF régénéré (`make fr` dans `docs/manual/`, sous le verrou de gate) et commité ; contrôle **aplati**, sur les
**six** formulations remplacées — chacune doit rendre `0` (au sol, sur `16eeed21` : `1`, `1`, `1`, `1`, `1`, `3`) :

```sh
pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' > /tmp/kesh-29-2-um.txt
for m in "sa date, pas son échéance" "pas de son échéance" "30 jours de la date de la facture" \
         "aucune limite de 30 jours après la facture" "n.est plus refusée pour sa date" \
         "30 jours après la date de la facture"; do
  printf '%s : ' "$m"; grep -o -- "$m" /tmp/kesh-29-2-um.txt | wc -l; done
```

(le `.` de « n.est » couvre l'apostrophe typographique `’` que rend `pdftotext`) ; et `grep -o "dernier rappel[^.]*"`
rend la nouvelle règle. Gardes textuelles :
`cargo nextest run -E 'binary(textes_coherents)'` vert.

`docs/api-external.md` § *Accepter des propositions de rapprochement* (`:547-567`) : ce § ne cite aujourd'hui aucune
raison `payment_date_*` (`grep -c payment_date docs/api-external.md` rend `0`). Les deux raisons sont **en service
depuis la Story 8-4** (`git log -S payment_date_outside_window` : `22fd6d4a`), et la v0.13.3 (#548) n'a fait
qu'aligner la proposition sur elles. La phrase à écrire documente donc les deux raisons comme **existantes**, sans
date d'introduction, et ne date que le changement : la date de facture est refusée en
`RECONCILIATION_INVOICE_NOT_ELIGIBLE` avec `payment_date_before_invoice_date` si le paiement la précède de plus d'un
jour, et avec `payment_date_outside_window` (`window_days` = 30) s'il arrive plus de 30 jours après la plus tardive
de sa date, de son échéance et de son dernier rappel non annulé ; *(depuis la v0.13.4)* **seule cette seconde
raison change de sens** — elle comptait les 30 jours depuis la seule date de la facture, si bien qu'une acceptation
autrefois refusée pour ce motif peut désormais aboutir (C-29-2-5). La phrase ne parle que de **dates** : les refus de
montant (`RECONCILIATION_OVERPAYMENT`, déjà décrit au même §) sont inchangés.

`CHANGELOG.md` : `## [0.13.4] — Non publié` (à créer au-dessus de `## [0.13.3] — 2026-10-14` s'il n'existe pas encore
au rebase ; sinon y ajouter), section `### Corrigé` : « **Un paiement reçu après l'échéance, ou après un rappel, est
proposé à la réconciliation ([#529](https://github.com/guycorbaz/kesh/issues/529)).** Une facture n'était proposée
— et acceptée — que si le paiement arrivait au plus 30 jours après **sa date**, si bien qu'un client qui payait à
30 jours net avec un peu de retard, ou après un rappel, n'était jamais reconnu — ni celui qui payait à l'heure, mais
plus de 30 jours après sa date, une facture à 60 jours. Les 30 jours courent désormais de la plus tardive des trois dates : la date de la facture, son
échéance, son dernier rappel (un rappel annulé ne compte pas). Seules les **dates** changent : la facture n'est
toujours proposée qu'au montant de son reste dû — un paiement qui inclut les frais de rappel ne l'est pas
([#401](https://github.com/guycorbaz/kesh/issues/401)). La proposition et l'acceptation appliquent toujours la même
règle de dates. Un paiement reçu plus de 30 jours après l'échéance et plus de 30 jours après le dernier rappel n'est toujours pas proposé :
enregistrez-le depuis la fiche de la facture, puis rejetez la transaction. API : mêmes codes et raisons qu'avant ;
une acceptation refusée en `payment_date_outside_window` peut désormais aboutir. »

Aucun texte de l'écran ne change : **ni clé i18n, ni capture** (C-29-2-6). Le repli de `failed-proposal-label.ts` pour
`payment_date_outside_window` est le libellé générique (« n'est pas éligible ») et ne cite aucune durée.

**AC9 — Mutations (chacune tuée par un test nommé au Dev Agent Record).**

| # | Mutation | Tuée par |
|---|---|---|
| M1 | retirer `COALESCE(i.due_date, i.date)` du `GREATEST` | AC5 b, k (liste), j (scalaire : `D − 10` au lieu de `D + 20`) ; AC6 cas #529 — pas d (le rappel `D − 10` y domine l'échéance) |
| M2 | retirer le terme du rappel | AC5 d, f |
| M3 | retirer `WHERE cancelled_at IS NULL` | AC5 e |
| M4 | `MAX(sent_at)` → `MIN(sent_at)` | AC5 f |
| M5 | `COALESCE(i.due_date, i.date)` → `i.due_date` (sans repli) | AC5 a… i : i (`E` nul) disparaît |
| M6 | acceptation : passer `invoice.date` comme ancre au lieu de lire `invoice_payment_anchor_date` | AC6 parité (b, d acceptées) |
| M7 | proposition : `{ANCHOR} >= ?` → `{ANCHOR} > ?` | AC5 b |
| M8 | `check` : tester `PaymentTooLate` avant `PaymentBeforeInvoice` | test unitaire d'ordre (AC5) |
| M9 | jointure des rappels sur `rmd.invoice_id = i.contact_id` (forme a), ou sans `ON` correct, jointure croisée (forme b) | forme a : AC5 d, f (la facture perd **son propre** rappel ; le cas inter-factures ne rougit que si les identifiants coïncident) ; forme b : cas inter-factures (AC5) |
| M10a | proposition : borne haute sur l'ancre — `AND {ANCHOR} <= ?` à la place de `AND i.date <= ?` | AC5 j, k |
| M10b | acceptation : `PaymentBeforeInvoice` testé sur `anchor_date` au lieu de `invoice_date` (dans `check`, ou `check(A, A)` à l'appel) | unitaire `check(D − 10, D + 20) == Ok(())` (AC5) ; AC6 parité j, k |
| M11 | scalaire : retirer `i.company_id = ? AND` (et son `.bind(company_id)`) d'`invoice_payment_anchor_date` | cas inter-sociétés (AC5) |
| M12 | retirer `DATE()` autour de `rmd.last_sent_at` | **équivalente des deux côtés, déclarée** (C-29-2-10) : le `CAST(… AS DATE)` extérieur, présent dans la constante que lisent les deux lecteurs, tronque le maximum, et la troncature est monotone (la date du maximum est le maximum des dates) |
| M13 | retirer le `CAST(… AS DATE)` | **équivalente si** MariaDB rend déjà un `DATE` (sinon tuée par l'assertion scalaire d'AC5, au décodage) : le Dev Agent Record dit laquelle des deux il a constatée. M12 et M13 appliquées **ensemble** : côté proposition, un `DATETIME` comparé à un `DATE` lié reste du bon côté de la borne (jour `D − 30` à toute heure `≥`, jour `D − 31` `<`) ; côté scalaire, le décodage `NaiveDate` d'un `DATETIME` en décide |
| M14 | retirer `i.date` du `GREATEST` (garder l'échéance et le rappel) | AC5 g, assertion **scalaire** d'`invoice_payment_anchor_date` (`D − 50` au lieu de `D − 45`) — g est refusé dans les deux formes, ni la liste ni la parité HTTP ne la voient |
| M15 | table dérivée sans agrégation par facture : `GROUP BY invoice_id` → `GROUP BY id` (une ligne par rappel) | AC5 f, liste assertée comme `Vec` ordonné avec ses répétitions (f y paraît deux fois) — un ensemble avalerait le doublon, et le scalaire (`fetch_optional`, première ligne d'un ordre non fixé) ne la tue pas de façon sûre |

Restauration de chaque mutation : `git checkout -- <fichier> && touch <fichier>` (`docs/agents-orchestration.md`
§ 7). Une mutation équivalente se déclare avec sa raison.

**AC10 — Gates.**

`kesh-db` touché : **gate complet même en boucle de revue** (`CLAUDE.md` § *Pendant une boucle de revue*), base remise
à zéro avant chaque gate complet (KF-039), `scripts/test-fast.sh` sous `flock` ; frontend (`npm run check`,
`lint-i18n-ownership`, `test:unit`, `build`) — aucun fichier frontend n'est censé changer, le gate le confirme ; E2E
complet au **dernier commit de code** (base `kesh_e2e_292`, port 3043, recette de `docs/testing.md` à la lettre).
Le Dev Agent Record ne déclare que ce qui a tourné, chiffres recomptés.

## Tasks / Subtasks

- [x] **T1 — Tests rouges** (AC5, AC6) : helpers de rappel et d'échéance, cas a à k, inter-factures, inter-sociétés,
      unitaires de `check` (dont `check(D − 10, D + 20)`), parité HTTP, cas de #529 ; constater le rouge (compilation
      ou assertion) avant T2.
- [x] **T2 — `kesh-db`** (AC1, AC2, AC3) : jointure et expression d'ancrage, `find_unpaid_invoices_for_window`,
      `invoice_payment_anchor_date`, renommage du champ, `check(F, A)`, doc-comments.
- [x] **T3 — `kesh-api`** (AC4) : étape 6 d'`accept_one_invoice` ; commentaire de `get_proposals` (`:653-655`) qui
      décrit la fenêtre ; aucun autre site (inventaire (a)).
- [x] **T4 — Revérifier l'inventaire (c)** (AC7) et copier la sortie au Dev Agent Record.
- [x] **T5 — Manuel FR + PDF, `api-external.md`, CHANGELOG** (AC8).
- [x] **T6 — Propagation** : le grep de § *Propagation*, trié à la main, sortie résumée au Dev Agent Record.
- [x] **T7 — Mutations** (AC9) et **gates** (AC10) ; Dev Agent Record.

## Dev Notes

### Inventaire (a) — les sites qui choisissent ou valident une facture pour une transaction, au sol (`16eeed21`)

```
$ grep -rn 'InvoiceDateWindow' crates --include=*.rs | grep -v '/tests/' | grep -v 'kesh-db/src/repositories/reconciliation.rs'
crates/kesh-api/src/routes/reconciliation.rs:41:    INVOICE_WINDOW_DAYS_AFTER_INVOICE, InvoiceDateRefusal, InvoiceDateWindow,
crates/kesh-api/src/routes/reconciliation.rs:659:            InvoiceDateWindow::for_transaction(&tx),
crates/kesh-api/src/routes/reconciliation.rs:1746:    match InvoiceDateWindow::for_transaction(&bank_transaction).check(invoice.date) {
$ grep -rn 'payment_date_outside_window\|INVOICE_WINDOW_DAYS_AFTER_INVOICE\|earliest_invoice_date' crates frontend/src --include=*.rs --include=*.ts --include=*.svelte | grep -v '/tests/'
  routes/reconciliation.rs:41, :1760, :1761 ; repositories/reconciliation.rs:91, :95, :152, :164, :172, :189, :194, :208, :265, :622 ;
  frontend failed-proposal-label.ts:55 (doc-comment, liste des raisons) et .test.ts:253 (fixture, window_days: 30)
```

**Ensemble clos : deux sites**, ceux de la 28-7 — la proposition (`find_unpaid_invoices_for_window`, seul appel de
production `routes/reconciliation.rs:656`) et l'acceptation (`accept_one_invoice`, étape 6). Le score
(`kesh-reconciliation/src/matching.rs`) ne lit aucune date. Le frontend ne calcule aucune fenêtre.

### Inventaire (b) — les sites non résolus (bornes voisines qui ne liront pas l'ancre)

| Site | Ce qu'il borne | Sort |
|---|---|---|
| `invoice_settlements_write.rs:115`, `:500` (`settle_invoice`, `write_off_invoice`) | date de règlement saisie ≥ facture − 1, sans borne haute | **non résolu, assumé** (inchangé depuis C-28-7-7) : ne règle pas une transaction ; c'est le contournement du reliquat |
| CHECK `chk_invoices_paid_at_after_date` (`migrations/20260419000001_invoice_paid_at.sql:32-33`) | `DATE(paid_at) >= date − 1` | **non touché** : borne de la date de facture, que la story ne change pas — aucune migration |
| `SettleInvoiceDialog.svelte`, `WriteOffDialog.svelte` (`min={invoiceDate}`) | date saisie ≥ facture | **non touché**, hors classe |
| Éligibilité des rappels (`repositories/dunning_eligibility.rs`) | quelles factures relancer | **hors classe** : ne choisit aucune facture pour une transaction ; ne lit pas la fenêtre. ⚠️ Mais **écrit la même notion** de « dernier rappel non annulé » (`:80`, `MAX(sent_at) … cancelled_at IS NULL`) : cité au doc-comment de la jointure neuve (AC1, C-29-2-12), non factorisé |
| `sent_at` des rappels **manuels** (`routes/dunning_reminders.rs:261`, saisi, `≤ now` UTC) et e-mail (`Utc::now`) | date du rappel | **angle mort écrit** (C-29-2-3) : `DATE(sent_at)` mêle une date saisie et une date UTC ; un jour d'écart au plus |

### Inventaire (c) — les tests existants dont le sens pourrait changer

```
$ for f in $(grep -lE 'reconciliation/proposals|reconciliation/accept|find_unpaid_invoices_for_window|accept_one_invoice' crates/*/tests/*.rs); do echo "$f: proposals=$(grep -cE 'reconciliation/proposals|find_unpaid_invoices_for_window\(' $f) accept=$(grep -cE 'reconciliation/accept|accept_one_invoice' $f) due_date=$(grep -cE 'due_date|dueDate' $f) reminders=$(grep -c 'invoice_reminders' $f)"; done
crates/kesh-api/tests/bank_accounts_closed_e2e.rs: proposals=1 accept=3 due_date=1 reminders=0
crates/kesh-api/tests/bank_account_closed_inventory.rs: proposals=0 accept=2 due_date=0 reminders=0
crates/kesh-api/tests/rejeu_interblocage_e2e.rs: proposals=0 accept=1 due_date=0 reminders=0
crates/kesh-db/tests/bank_import_delete.rs: proposals=0 accept=1 due_date=0 reminders=0
crates/kesh-db/tests/invoice_settlement.rs: proposals=0 accept=1 due_date=1 reminders=0
crates/kesh-api/tests/reconciliation_rules_e2e.rs: proposals=8 accept=5 due_date=0 reminders=0
crates/kesh-db/tests/reconciliation_repository.rs: proposals=11 accept=0 due_date=0 reminders=0
crates/kesh-db/tests/invoice_write_off.rs: proposals=2 accept=0 due_date=1 reminders=0
crates/kesh-api/tests/reconciliation_e2e.rs: proposals=12 accept=49 due_date=0 reminders=0
crates/kesh-db/tests/letterings_lexical.rs: proposals=0 accept=1 due_date=0 reminders=0
```

*(Commande élargie en validation P1, R4 : la première ne partait que de `proposals` et manquait les tests qui
n'appellent qu'`accept`. Relevé du 2026-10-14 sur `16eeed21`.)*

- `bank_accounts_closed_e2e.rs:451` : `due_date: None` ; `invoice_write_off.rs:81` : `due_date: Some(D())`, égale à
  la date de la facture → `A = F` dans les deux cas, sens inchangé.
- `rejeu_interblocage_e2e.rs:1517` : **seul** des nouveaux venus à appeler réellement `/reconciliation/accept` ;
  factures créées par `POST /api/v1/invoices` (`facture_validee`, date = `aujourd_hui()`, contact sans délai :
  `due_date` = date, `:289-290`), transaction au jour de la facture (`:1439`) → `A = F = D`, sens inchangé.
- `bank_account_closed_inventory.rs:112`, `:287`, `bank_import_delete.rs:26`, `invoice_settlement.rs:471`,
  `letterings_lexical.rs:404` : **mentions textuelles** d'`accept_one_invoice` (inventaires lexicaux, doc-comments),
  aucun appel ; la lecture ajoutée à l'étape 6 n'écrit rien et n'entre dans aucun de ces inventaires (à revérifier au
  gate, un rouge y signerait le contraire). `invoice_settlement.rs:61` : `due_date: Some(date)`, sans appel
  d'acceptation.
- Les helpers SQL bruts (`insert_invoice`, `reconciliation_e2e.rs:337` ; `insert_test_invoice`,
  `reconciliation_repository.rs:155`) n'écrivent pas `due_date` → `NULL` → `A = F`.
- `reconciliation_rules_e2e.rs` : une règle n'est proposée que sans candidate forte ; aucune échéance → aucune
  candidate neuve, sens inchangé. À revérifier au gate (un rouge y signerait une facture neuve candidate).
- Frontend : `failed-proposal-label.test.ts:253` (fixture) inchangé ;
  `frontend/tests/e2e/reconciliation-amount-due.spec.ts:78` (« au plus 30 jours avant le relevé : dans la fenêtre ») —
  toujours vrai (condition suffisante) ; non modifié. Pollution E2E par des factures avec échéance : AC7.

### Propagation — le symptôme à greper après chaque patch

La **valeur** `30` près de « facture », la formulation « sa date, pas son échéance », « plus de 30 jours après », et
les noms renommés :

```sh
grep -rnE "earliest_invoice_date|sa date, pas son échéance|pas de son échéance|plus de 30 jours après|30 jours de la date de la facture|\b30 (jours|days|Tage|giorni)\b|#529|window_days" \
  crates frontend/src frontend/tests docs/manual/fr/*.tex docs/api-external.md CHANGELOG.md
```

Les occurrences légitimes (« 30 jours net », session, `window_days` du contrat, historique du CHANGELOG des versions
publiées — **qui ne se réécrit pas**, l'entrée 0.13.3 citant #529 reste telle quelle) se trient à la main.

### Ce qui doit être préservé

- Le contrat de `POST /api/v1/reconciliation/accept` : codes, `details.reason`, `details.window_days` (= 30), ordre
  des refus de l'étape 4bis (#482) à l'étape 7bis (score nul).
- `get_proposals` : filtres de devise et de signe **avant** la requête, ordre des passes, `LIMIT 50`, `has_more`,
  règle en repli sous le seuil 0,5.
- La date du règlement écrit (`paid_at_dt`) : date de valeur, à défaut de comptabilisation.
- La borne haute `F ≤ D + 1` (#548) et ses deux raisons.

### Zones partagées avec la 29-3 (#527, tri des candidates) et la 29-5 (#668, départage à score égal)

- `find_unpaid_invoices_for_window` (`repositories/reconciliation.rs:234-274`) : la 29-2 change le `WHERE` et les
  jointures ; la 29-3 ou la 29-5 toucheront probablement l'`ORDER BY` / `LIMIT 50` adjacents (C-28-7-10, exclusion de
  la 28-7 : « priorité au montant exact avant le `LIMIT 50` »). ⚠️ **La troncature précède le départage** : l'ordre
  `ORDER BY i.date ASC, i.id ASC LIMIT 50` (`:258-259`) garde les 50 factures les plus **anciennes** de même reste dû
  (à 0,05 près) ; l'élargissement y avance des factures anciennes non échues, qui peuvent pousser hors des 50 une
  facture échue. Rare aux volumes d'une PME, mais un départage appliqué après une troncature qui a écarté la bonne
  facture ne la retrouve pas : la 29-5 doit l'examiner avec son départage (C-29-2-7).
- `get_proposals`, passe 2 (`routes/reconciliation.rs:653-662`, appel de la fenêtre et son commentaire) et passe 4
  (`:721-845`, construction et ordre des candidates) : la 29-2 ne touche que le commentaire de la passe 2.
- `propose_matches` (`kesh-reconciliation/src/matching.rs:97-106`, départage `invoice_id ASC` à `:104`) : non touché
  ici ; **la 29-2 rend ce départage plus coûteux** (C-29-2-7, C-29-2-11) — à score égal, la facture la plus ancienne
  passe en tête, sans égard à l'exigibilité, et elle peut désormais être **pas encore échue** : une facture ancienne à
  **échéance lointaine** (ou mal saisie, à 2099) reste candidate sans limite de temps et passe devant une facture
  échue. **Porté par [#668](https://github.com/guycorbaz/kesh/issues/668), Story 29-5**, même release v0.13.4, mergée
  avant le tag (C-29-2-13). Attendu de #668, rectifié au regard de l'art. 87 CO : la facture que le paiement désigne
  prime (déjà le cas par le score) ; sinon, à score égal, les factures **échues** d'abord, puis l'ancienneté
  (échéance, puis date) ; cas de test : deux factures de même reste dû, l'une ancienne à échéance lointaine (non
  échue), l'autre échue — l'échue en tête.

### Exclusions écrites (périmètre minimal)

- **Choix manuel d'une facture ouverte à l'écran** (voie 2 de #529) : non fait (C-29-2-2) ; reliquat couvert par
  [#666](https://github.com/guycorbaz/kesh/issues/666) (C-29-2-9).
- **Montant différent** (frais bancaires déduits, paiement du rappel frais compris — #401) : hors #529, inchangé.
- **Trop-perçu de 1 à 5 centimes proposé puis refusé** ([#665](https://github.com/guycorbaz/kesh/issues/665), voisine
  de #548 sur le **montant** : tolérance de 0,05 à la proposition contre `RECONCILIATION_OVERPAYMENT` à
  l'acceptation) : hors périmètre, ouverte, devenue la Story 29-6. La 29-2 n'aggrave pas sa règle ; en élargissant les candidates, elle n'en
  augmente que l'occurrence possible.
- **Ordre des candidates** (règle contre facture) : 29-3 (#527). **Départage à score égal et troncature `LIMIT 50`** :
  29-5 ([#668](https://github.com/guycorbaz/kesh/issues/668)), à merger avant le tag v0.13.4 (C-29-2-13).
- **Migration, index** : aucun. Les jointures s'appuient sur `idx_invoice_reminders_company_invoice` /
  `idx_invoice_reminders_invoice` ; le filtre `i.date <= ?` garde une plage sur
  `idx_invoices_company_validated_unpaid_date`, sur un ensemble que `paid_at IS NULL` réduit aux factures ouvertes.
  Pas de mesure de performance exigée (factures ouvertes d'une PME : dizaines). La table dérivée `rmd` lit **toute**
  `invoice_reminders`, sans prédicat poussé (même patron que `lt`, `cnt`, `cl_ttc`, `st`), une fois par
  transaction en attente (jusqu'à 100) : non bloquant aux volumes d'une PME (F5 de la validation P1, noté sans
  action).
- **Frontend, i18n, captures, spec Playwright neuve** : aucun texte ni contrôle de l'écran ne change (C-29-2-6) ; la
  frontière HTTP est exercée par les tests `spawn_app` d'AC6 ; la suite E2E complète tourne au dernier commit de
  code.

### Modules touchés (règle de découpage : au plus cinq)

1. `kesh-db` — `repositories/reconciliation.rs` (+ `tests/reconciliation_repository.rs`) ;
2. `kesh-api` — `routes/reconciliation.rs` (+ `tests/reconciliation_e2e.rs`) ;
3. documentation — manuel FR (+ PDF), `docs/api-external.md`, `CHANGELOG.md`.

Trois : pas de découpage.

### Pièges connus du dépôt, applicables ici

- **`GREATEST` et `NULL`** en MariaDB (AC1) — la mutation M5 le garde.
- **Gate complet en boucle** (`kesh-db`), base remise à zéro avant chaque gate complet (KF-039).
- **Mutation restaurée, binaire périmé** : `git checkout -- f && touch f`.
- **`pdftotext` coupe les lignes** : aplatir avant de greper.
- **Ne jamais réécrire l'entrée d'une passe passée** ; **recompter** les décomptes du Dev Agent Record.
- **Aucun « closes #529 » dans un message de commit** : `refs #529` partout ; le mot-clé va dans la PR (B4).
- **Données réelles** : fixtures synthétiques ; aucune ligne du relevé de Guy dans un test, un commit ou un rapport.

### References

- Issue #529 ; #666 (reliquat, choix manuel) ; #665 (trop-perçu, voisine, Story 29-6) ; #668 (départage à score égal,
  Story 29-5) ; #548 (Story 28-7, fiche `28-7-facture-posterieure-paiement.md`, C-28-7-1 à 16) ; #482 (28-6) ; #527
  (29-3) ; #401 (frais de rappel).
- `crates/kesh-db/src/repositories/reconciliation.rs:1-274`, `:590-668` ;
  `crates/kesh-api/src/routes/reconciliation.rs:586-845`, `:1621-1770` ;
  `crates/kesh-db/src/repositories/invoice_settlements.rs:42-113` (patron des tables dérivées) ;
  `crates/kesh-db/src/entities/invoice_reminder.rs` ; `migrations/20260715000001_invoice_reminders.sql`.
- `docs/manual/fr/user-manual.tex:1305`, `:2029`, `:2031`, `:2902` ; `docs/api-external.md:547-567`.
- `docs/agents-orchestration.md` ; `CLAUDE.md` § *Review Iteration Rule*, § *Pattern batch*, § *Un appariement
  automatique propose, il ne crée jamais*, § *Test Locally First*.

## Dev Agent Record

### Agent Model Used

Opus 5.5 (`bmad-dev-story`, autonomie), worktree `/home/gcorbaz/devel/kesh-29-2`, base `fcd2ae75` ; bases
`kesh_292` (gate) et `kesh_e2e_292` (E2E), port 3043, `CARGO_TARGET_DIR` du worktree.

### Debug Log References

Journaux (hors dépôt) : `/home/gcorbaz/devel/kesh-gate-logs/292-red-api.log`, `292-red-db.log` (rouges),
`292-mutations-campagne1.log` et `292-mutations-M14bis.log` (mutations), `292-gate-backend.log`,
`292-gate-frontend.log`, `292-make-fr.log`, `292-e2e.log` et `292-e2e-backend.log`.

### Completion Notes List

**Tests rouges d'abord (T1), constatés avant le code.**

- `kesh-api` (le test compile contre l'ancien code ; rouge à l'assertion), 4 tests filtrés, 2 rouges :
  ```
  a_payment_after_the_due_date_is_proposed_and_settles_the_invoice … panicked at reconciliation_e2e.rs:3304:5:
    {"bankTransactionId":1,"candidates":[],…}   left: None   right: Some("invoice")
  proposal_and_acceptance_share_the_anchor_of_due_date_and_reminders … panicked at reconciliation_e2e.rs:3229:5:
    cas proposés : b, d, f, i, j, k
      left: {(9, 9), (10, 10)}
     right: {(2, 2), (4, 4), (6, 6), (9, 9), (10, 10), (11, 11)}
  Summary 4 tests run: 2 passed, 2 failed
  ```
  (avant la story, seuls i et j sont proposés — et `proposed == accepted` passait déjà : la 28-7 tient la parité ;
  c'est l'ensemble qui change.)
- `kesh-db` : rouge à la compilation (fonction et champ absents, `check` à un argument) :
  ```
  error[E0425]: cannot find function `invoice_payment_anchor_date` in module `reconciliation_repo` (×4)
  error[E0609]: no field `earliest_anchor_date` on type `reconciliation::InvoiceDateWindow`
  error[E0061]: this method takes 1 argument but 2 arguments were supplied
  ```

**Implémentation (T2, T3).** Écart de forme à AC1, consigné **C-29-2-15** (consigne de l'orchestrateur) : l'ancre et
sa jointure sont des `macro_rules!` (`invoice_payment_anchor_sql!`, `invoice_payment_anchor_join_sql!`), dont dérivent
les `const` `INVOICE_PAYMENT_ANCHOR_SQL` et `INVOICE_PAYMENT_ANCHOR_JOIN_SQL` ; la requête scalaire est assemblée par
`concat!` sur les mêmes macros. L'ancre s'écrit en deux termes,
`CAST(GREATEST(GREATEST(i.date, COALESCE(i.due_date, i.date)), COALESCE(DATE(rmd.last_sent_at), i.date)) AS DATE)` —
équivalente au `GREATEST` à trois termes de la fiche —, le premier terme étant l'**exigibilité** que la 29-5 remplacera
par sa macro. Fonction de jointure de la fiche (`invoice_payment_anchor_joins()`) : non créée, la constante suffit (une
seule table dérivée). Proposition : `AND i.date <= ? AND {INVOICE_PAYMENT_ANCHOR_SQL} >= ?`, jointure placée après
`amount_due_derived_joins()` ; statut, `HAVING`, `ORDER BY i.date ASC, i.id ASC`, `LIMIT 50` inchangés. Champ renommé
`earliest_anchor_date` ; `check(invoice_date, anchor_date)`. Acceptation : lecture ajoutée entre les trois gardes
d'éligibilité et `check` ; `Ok(None)` → `INVOICE_NOT_FOUND` (`details.invoiceId`), `Err` → `DATABASE_ERROR`
(`details.message`), tous deux en `FailedProposal`, aucun `?`. **Angle mort assumé (AC4)** : ces deux bras ne sont ni
testés ni mutés ; ils se vérifient à la revue de code.

**Verrous et naissance de la vue (B5).** `invoice_payment_anchor_date` est un `SELECT` ordinaire sans `FOR UPDATE`
(`reconciliation.rs:325`, doc). Ordre dans `accept_one_invoice` (`grep -n`) :
```
1637:    let bank_transaction = match reconciliation_repo::find_pending_by_id_for_account(   ← lecture ordinaire (suffixe "")
1682:    let invoice = match reconciliation_repo::find_invoice_by_id_for_company(
1711:        contacts_repo::find_by_id_in_company(&mut **tx, invoice.contact_id, company_id)
1751:        match reconciliation_repo::invoice_payment_anchor_date(&mut **tx, company_id, invoice.id)
```
La première lecture ordinaire du chemin (`find_pending_by_id_for_account`, `bank_tx_by_id_for_account_sql("")`) fait
naître la vue ; la lecture ajoutée vient trois lectures après et ne prend aucun verrou.

**Inventaire (c) revérifié (T4)**, même commande que § *Inventaire (c)* :
```
crates/kesh-api/tests/bank_account_closed_inventory.rs: proposals=0 accept=2 due_date=0 reminders=0
crates/kesh-api/tests/bank_accounts_closed_e2e.rs: proposals=1 accept=3 due_date=1 reminders=0
crates/kesh-api/tests/rejeu_interblocage_e2e.rs: proposals=0 accept=1 due_date=0 reminders=0
crates/kesh-api/tests/reconciliation_e2e.rs: proposals=13 accept=49 due_date=9 reminders=1
crates/kesh-api/tests/reconciliation_rules_e2e.rs: proposals=8 accept=5 due_date=0 reminders=0
crates/kesh-db/tests/bank_import_delete.rs: proposals=0 accept=1 due_date=0 reminders=0
crates/kesh-db/tests/invoice_settlement.rs: proposals=0 accept=1 due_date=1 reminders=0
crates/kesh-db/tests/invoice_write_off.rs: proposals=2 accept=0 due_date=1 reminders=0
crates/kesh-db/tests/letterings_lexical.rs: proposals=0 accept=1 due_date=0 reminders=0
crates/kesh-db/tests/reconciliation_repository.rs: proposals=15 accept=0 due_date=7 reminders=2
```
Mêmes dix fichiers qu'au sol ; seuls changent les compteurs des deux fichiers qui portent les tests neufs. Aucun test
existant n'a vu son assertion modifiée ; seul le doc-comment de `proposal_and_acceptance_share_the_invoice_date_window`
(AC6) et celui de `find_unpaid_invoices_filters_by_30_day_window` (propagation) sont réécrits. Les inventaires
d'écrivains (`bank_account_closed_inventory.rs`, `letterings_lexical.rs`) sont verts au gate complet.
*(Annotation du 2026-10-14, revue de code P1, A3 : ce relevé date du commit `8f483388` — `due_date=7` y est exact.
Rejouée à `d45279f7` et à `729a1dc9`, la même commande rend `reconciliation_repository.rs: … due_date=8`, l'écart
venant du doc-comment ajouté par `d45279f7`, qui cite `the_anchor_is_the_latest_of_date_due_date_and_last_reminder` ;
les neuf autres lignes sont identiques. Le relevé n'est pas réécrit.)*

**Documentation (T5).** Manuel FR : `:1305`, `:2029`, `:2031` (deux phrases et la course, C-29-2-14), `:2902` ; un
`\label{sec:algorithme-matching}` neuf (**C-29-2-16**) porte les renvois. PDF régénéré (`make fr`, sous verrou, 3
passes stables) ; `admin-manual.pdf` et `marketing-brochure.pdf`, régénérés sans changement de source, ont été remis
à leur version commitée. Contrôle aplati :
```
sa date, pas son échéance : 0
pas de son échéance : 0
30 jours de la date de la facture : 0
aucune limite de 30 jours après la facture : 0
n.est plus refusée pour sa date : 0
30 jours après la date de la facture : 0
```
et `grep -o "dernier rappel[^.]*"` rend cinq passages neufs (§ rappels, algorithme ×2, `sec:facture-posterieure`,
FAQ). Brochure `:167` triée, **non modifiée** (la fenêtre reste de 30 jours). `docs/api-external.md` : paragraphe
**Dates** neuf. `CHANGELOG.md` : `## [0.13.4] — Non publié`, `### Corrigé`, texte prescrit d'AC8 tel quel. Aucun
fichier frontend, aucune clé i18n, aucune capture (C-29-2-6). Garde `textes_coherents` : verte au gate complet.

**Propagation (T6).** `grep -rnE` de § *Propagation* (sans les jetons `30 jours`/`window_days`, trop larges, triés à
part) : restent les mentions neuves de la story, la phrase du reliquat (« plus de 30 jours après l'échéance », juste),
et l'entrée 0.13.3 du CHANGELOG (`:74-75`, relevé daté d'une version publiée, **non réécrit**). Le doc-comment de
`find_unpaid_invoices_filters_by_30_day_window` (« paiement plus de 30 jours après la facture ») a été précisé
(factures sans échéance ni rappel). `failed-proposal-label.ts:55` ne cite que les noms de raison : inchangé.

**Mutations (T7, AC9)** — 20 jouées (les 19 lignes de la campagne, dont la combinaison M12 + M13 et la variante
M10b-appel, puis M14bis) : 17 tuées, 3 équivalentes ; script hors dépôt (scratchpad de la session), motif vérifié unique avant écriture,
restauration `git checkout -- <fichier> && touch <fichier>` après chacune ; filtre : `binary(reconciliation_repository)`,
unitaires `reconciliation::tests`, et cinq tests de `reconciliation_e2e` (27 tests).

| # | Résultat | Tuée par |
|---|---|---|
| M1 (exigibilité réduite à `i.date`) | tuée, 3 rouges | `the_anchor_is_the_latest_…` (kesh-db), parité 29-2, cas de #529 |
| M2 (terme du rappel retiré) | tuée, 4 rouges | `the_anchor_is_the_latest_…`, `a_reminder_moves_only_…`, `the_anchor_of_an_invoice_is_scoped_…`, parité 29-2 |
| M3 (`cancelled_at IS NULL` retiré) | tuée, 2 rouges | `the_anchor_is_the_latest_…` (e), parité 29-2 |
| M4 (`MAX` → `MIN`) | tuée, 2 rouges | `the_anchor_is_the_latest_…` (f), parité 29-2 |
| M5 (`COALESCE(i.due_date, i.date)` → `i.due_date`) | tuée, 13 rouges | dont `the_anchor_is_the_latest_…` et tous les tests de fenêtre sans échéance |
| M6 (acceptation : `check(F, F)`) | tuée, 2 rouges | parité 29-2, cas de #529 |
| M7 (`>=` → `>`) | tuée, 5 rouges | `the_anchor_is_the_latest_…` (b), `find_unpaid_invoices_filters_by_30_day_window`, `find_unpaid_invoices_orders_by_date_then_id_before_limit`, parités 28-7 et 29-2 |
| M8 (ordre des refus inversé) | tuée, 1 rouge | `check_refuses_payment_before_invoice_first` |
| M9a (`ON rmd.invoice_id = i.contact_id`) | tuée, 3 rouges | `the_anchor_is_the_latest_…`, `a_reminder_moves_only_…`, parité 29-2 |
| M9b (`ON 1 = 1`, jointure croisée) | tuée, 3 rouges | idem |
| M10a (borne haute sur l'ancre, proposition) | tuée, 2 rouges | `the_anchor_is_the_latest_…` (j, k), parité 29-2 |
| M10b (borne haute sur `anchor_date` dans `check`) | tuée, 4 rouges | les trois unitaires de `check`, parité 29-2 |
| M10b-appel (`check(A, A)` à l'appel) | tuée, 1 rouge | parité 29-2 |
| M11 (scalaire sans `company_id`, prédicat et `bind`) | tuée, 1 rouge | `the_anchor_of_an_invoice_is_scoped_to_its_company` |
| M12 (`DATE()` retiré) | **équivalente**, 27/27 verts | le `CAST(… AS DATE)` extérieur tronque le maximum (troncature monotone) — comme déclaré (C-29-2-10) |
| M13 (`CAST` retiré) | **équivalente constatée**, 27/27 verts | MariaDB rend déjà un `DATE` (`GREATEST` de `DATE` et de `DATE()`) |
| M12 + M13 | tuée, 8 rouges | décodage : `mismatched types; Rust type NaiveDate (as SQL type DATE) is not compatible with SQL type DATETIME` |
| M14 (`i.date` retiré du premier terme) | **équivalente, démontrée** — 27/27 verts | ⚠️ **écart à la fiche** : le repli `i.date` du terme du rappel (`COALESCE(DATE(rmd.last_sent_at), i.date)`) garde `F` dans le `GREATEST` ; la fiche (« tuée par g, scalaire `D − 50` ») se trompait, et la forme à trois termes de la fiche aurait la même équivalence. Remplacée par **M14bis** |
| M14bis (`i.date` retiré du premier terme **et** repli du rappel sur l'échéance) | tuée, 1 rouge | `the_anchor_is_the_latest_…`, assertion **scalaire** du cas g (seule à la voir, comme la fiche le prévoyait pour M14) |
| M15 (`GROUP BY id`) | tuée, 2 rouges | `the_anchor_is_the_latest_…` (f dédoublée dans le `Vec`), parité 29-2 |

État après la campagne : `git status --short crates` vide.

*(Annotation du 2026-10-14, revue de code P1, E1 = A2 (MEDIUM) et B2 (LOW) : la ligne **M14** ci-dessus est
**fausse** — M14 n'est pas équivalente. Le repli `i.date` du terme du rappel ne vaut `F` que **sans** rappel ; avec
une échéance `E < F` **et** un rappel non annulé `R < F`, l'ancre mutée vaut `max(E, R) < F`. Elle survivait faute
de ce jeu dans les tests, non par équivalence. Cas **l** ajouté à l'AC5 (`F = D − 25`, `E = D − 50`, rappel non
annulé à `D − 60`, ancre `D − 25`, candidate), posé par fixture SQL directe — l'API refuse une échéance antérieure
à la date, si bien que ce jeu n'est pas constructible par elle ; un rappel manuel antérieur à la facture, lui,
l'est. M14 rejouée sur `729a1dc9`, appliquée par Edit (motif unique vérifié), restaurée par `git checkout --` puis
`touch` : **tuée, 1 rouge** — `the_anchor_is_the_latest_…`, `cas l : ancre`, `left: Some(2026-04-26)` (`D − 50`),
`right: Some(2026-05-21)` (`D − 25`) ; filtre `binary(reconciliation_repository)` (16 tests) et trois tests de
`reconciliation_e2e` (`proposal_and_acceptance_share_the_invoice_date_window`, `…_share_the_anchor_of_due_date_and_reminders`,
`post_accept_rejects_payment_date_outside_window`) — 19 tests run: 18 passed, 1 failed ; journal
`292-mutations-M14-revue-p1.log`. Le cas de #529 (`a_payment_after_the_due_date_is_proposed_and_settles_the_invoice`)
n'était pas dans ce filtre. L'assertion
scalaire rougit la première ; la liste la verrait aussi (ancre mutée hors fenêtre), ce qui n'a pas été rejoué.
Bilan des mutations après la revue : 21 jouées, **18 tuées, 2 équivalentes** (M12, M13). La campagne d'origine
n'est pas réécrite.)*

*(Annotation du 2026-10-14, revue de code P1, A4 (LOW) : chaque cas de l'AC5 a un montant propre, si bien que la
liste assertée vaut `vec![id]` ou `vec![]` ; l'**ordre** `(date, id)` n'y est pas exercé — il l'est par
`find_unpaid_invoices_orders_by_date_then_id_before_limit`, inchangé. Ce qu'AC5 vise par le `Vec` (les répétitions,
M15) est tenu.)*

**Gates (AC10)**, au dernier commit de code (`d45279f7`, qui porte les derniers doc-comments), base `kesh_292`
recréée, migrée et seedée juste avant :

- `scripts/test-fast.sh` (fmt + clippy `-D warnings` + nextest), sous verrou : **3611 tests run: 3611 passed,
  6 skipped** (166,8 s). Recompte : 3604 au sol + 7 neufs (2 unitaires `check_refuses_payment_before_invoice_first`,
  `check_admits_an_invoice_not_yet_due` ; 3 `kesh-db` ; 2 `kesh-api`) = 3611.
- Frontend, sous verrou : `npm run check` 0 erreur (29 avertissements, 5 fichiers — préexistants) ;
  `lint-i18n-ownership` PASS ; `npm run test:unit` **146 fichiers, 1610 tests** verts ; `npm run build` vert.
- E2E complet, sous verrou, base `kesh_e2e_292` recréée, port 3043, recette de `docs/testing.md` : **279 passed,
  8 failed, 19 skipped** (11,1 min). Les 8, fichier par fichier contre § *Les échecs attendus* : les **7 KF-029**
  (`mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`, `onboarding.spec.ts:57`, `:77`, `:150`) et
  `sidebar-navigation.spec.ts:75` (KF-052 ou pollution — **non rejoué seul**, les deux causes sont dans la liste).
  Compte dans la fourchette 7 à 10. **Aucune spec de réconciliation, de rappels ni de factures en échec** (AC7).

### File List

- `crates/kesh-db/src/repositories/reconciliation.rs` — macros et constantes de l'ancre, `InvoiceDateWindow`
  (champ renommé, `check(F, A)`), `invoice_payment_anchor_date`, `find_unpaid_invoices_for_window`, doc-comments,
  unitaires.
- `crates/kesh-db/tests/reconciliation_repository.rs` — helpers `set_test_due_date`, `insert_test_reminder` ; trois
  tests neufs ; un doc-comment précisé.
- `crates/kesh-api/src/routes/reconciliation.rs` — étape 6 d'`accept_one_invoice` ; commentaire de `get_proposals`.
- `crates/kesh-api/tests/reconciliation_e2e.rs` — helpers `set_due_date_292`, `insert_reminder_292` ; deux tests
  neufs ; doc-comment du test de parité de la 28-7.
- `docs/manual/fr/user-manual.tex`, `docs/manual/fr/user-manual.pdf`, `docs/api-external.md`, `CHANGELOG.md`.
- `_bmad-output/implementation-artifacts/epic-29-choix-autonomes-29-2.md` (C-29-2-15, C-29-2-16, annotation de
  C-29-2-8), `29-2-review-prompt-p1.md`, cette fiche, `sprint-status.yaml`.

## Change Log

- 2026-10-14 — Fiche créée (`bmad-create-story`, Opus 5.5, autonomie) sur `16eeed21`. Prémisses de #529 et de la
  consigne établies au code (deux périmées : la fenêtre « centrée », la promesse du manuel). Choix C-29-2-1 à 9.
- 2026-10-14 — **Validation P1** (Sonnet 5.5, deux lentilles en contexte frais, lecture seule ; rapports
  `kesh-gate-logs/29-2-validate-p1-R.md` et `-F.md`, sur `45f157f7`). Bruts : R 0 CRITICAL / 0 HIGH / 3 MEDIUM /
  6 LOW, F 0 / 1 / 2 / 3. Après fusion des doublons (F1 = R1, F2 = R2, F6 = R6) : **1 HIGH, 3 MEDIUM, 8 LOW
  distincts**. Les deux lentilles ont déclaré leurs axes exercés et non exercés. Remédiation (Opus 5.5) :
  - **F1/R1 (HIGH)** — paiement **avant** l'échéance absent des tableaux, mutation « borne haute sur l'ancre »
    survivante : cas **j** (`F = D − 10`, `E = D + 20`) et **k** (`F = D − 45`, `E = D + 15`) ajoutés à AC5 et à la
    parité AC6 ; unitaire `check(D − 10, D + 20) == Ok(())` ; mutations **M10a** (proposition) et **M10b**
    (acceptation) ; deux lignes au tableau de § *La règle retenue* ; AC2 dit que la borne haute reste sur `i.date` ;
    manuel (`:2029`) et CHANGELOG prescrits disent que le paiement à l'heure d'une facture à 60 jours est proposé.
  - **F2/R2 (MEDIUM)** — portée société de la scalaire : cas inter-sociétés (`Ok(None)`) à AC5, mutation **M11**.
  - **F3 (MEDIUM)** — justification du faux rattachement fausse pour les factures non échues ou à échéance
    lointaine : phrase réécrite, angle mort écrit (**C-29-2-11**), C-29-2-7 complété, cas de test transmis à la 29-3
    (§ *Zones partagées*).
  - **R3 (MEDIUM)** — textes prescrits trop larges : manuel (`:1305`), CHANGELOG et `api-external.md` ne promettent
    que la **règle de dates**, au montant du reste dû (#401) ; « même règle » devient « même règle de dates » ; #665
    nommée au § de la règle et aux Exclusions.
  - **LOW** — F4 : `CAST(… AS DATE)` autour de l'ancre (**C-29-2-10**), M12 (`DATE()` retiré) déclarée équivalente,
    M13 (`CAST` retiré) équivalente ou tuée selon le constat ; R7 : site jumeau `dunning_eligibility.rs:80` cité à AC1
    et à l'inventaire (b), non factorisé (**C-29-2-12**) ; R8 : `Ok(None)` / `Err` d'AC4 écrits en angle mort assumé ;
    R9 : `api-external.md` documente les deux raisons comme existantes et ne date que le changement — ⚠️ la date
    proposée par R9 (« depuis la v0.13.3 ») est **réfutée** : `git log -S payment_date_outside_window` rend
    `22fd6d4a` (Story 8-4), la 28-7 n'a fait qu'y aligner la proposition ; R4 : inventaire (c) élargi aux tests qui
    n'appellent qu'`accept` (cinq fichiers de plus, dont un seul appel réel, `rejeu_interblocage_e2e.rs:1517`, sens
    inchangé ; sortie recopiée) ; R5 : numéros de ligne recomptés (`:1716`, `:1740-1765`, `:1745`, `:653-655`,
    `:656-662`, `invoice_settlements_write.rs:500`) et, au passage, la passe 4 de `get_proposals` (`:721-845`, et non
    `:744-870`) ; R6/F6 : #666 cité (C-29-2-9), #665 aux Exclusions. **F5** (coût de la table dérivée `rmd`) : noté aux
    Exclusions, sans action — c'est le constat de la lentille elle-même.
  - **Aucun refus.** Décomptes recomptés depuis la fiche : 10 AC, 7 tâches, 14 lignes de mutation (M1-M9, M10a,
    M10b, M11, M12, M13), 11 cas d'AC5 (a-k) plus inter-factures et inter-sociétés, 8 lignes d'exemple, 12 choix
    (C-29-2-1 à 12, dont 10 à 12 neufs, 7 et 9 complétés par annotation datée), 3 modules. Symptômes grepés sur le
    worktree (« vivante », « demande d'évolution », `a à h`, `M1-M9`, anciens numéros de ligne) : seule reste
    l'entrée de création ci-dessus, relevé daté qui ne se réécrit pas. Prochaine passe : **P2, Opus**, lentilles R et
    F (`29-2-validate-prompt-p2.md`), axe obligatoire : relire ce que cette remédiation a écrit.
  - *(Annotation du 2026-10-14, validation P2, R8/F6 : « Aucun refus » est inexact **à sa date** — l'entrée réfute
    deux lignes plus haut la date d'introduction proposée par R9. C'était un **refus partiel, fondé** :
    `git log -S payment_date_outside_window` rend `22fd6d4a` (Story 8-4, première version taguée `v0.1.0`), où les deux
    raisons et la borne de 30 jours sur la date de facture existent déjà. Aucun autre refus. L'entrée ci-dessus n'est
    pas réécrite.)*
- 2026-10-14 — **Validation P2** (Opus 5.5, deux lentilles en contexte frais, lecture seule ; rapports
  `kesh-gate-logs/29-2-validate-p2-R.md` et `-F.md`, sur `375f2fbf`). Bruts : R 0 CRITICAL / 0 HIGH / 1 MEDIUM /
  8 LOW, F 0 / 0 / 2 / 6. Fusions : F1 = R1, F6 = R8 ; F7 recouvre R3 (`:2031`) et R6 (`:2887`) et ajoute deux
  points (`:2029`, brochure) — compté comme un LOW distinct. **2 MEDIUM, 12 LOW distincts.** Les deux lentilles ont
  déclaré leurs axes exercés et non exercés ; F déclare un écart de protocole (fichier temporaire hors du dépôt,
  supprimé). Remédiation (Opus 5.5) :
  - **R1/F1 (MEDIUM)** — l'angle mort du faux rattachement (C-29-2-11) était « transmis à la 29-3 », qui le refuse
    (C-29-3-12) ; aucune issue citée. **Décision de l'orchestrateur** (C-29-2-13) : le départage à score égal est
    porté par **#668** (`bug`, P2), attendu rectifié au regard de l'art. 87 CO (dette exigible d'abord, puis la plus
    ancienne), devenue la **Story 29-5**, même release v0.13.4, mergée avant le tag. La 29-2 ne traite pas le
    départage ; la fiche (§ *La règle retenue*, Status, § *Zones partagées*, Exclusions, References) et le registre
    (C-29-2-7, C-29-2-11) citent #668 et la 29-5 là où ils renvoyaient à la 29-3 ; l'ancien critère transmis (« la
    récente en tête », proximité de date) est retiré, contraire à l'art. 87 CO. L'aggravation du cas ordinaire
    (`F ≥ D − 30` → `F ≥ D − 60` à 30 jours net, F1) est écrite au § *La règle retenue*.
  - **F2 (MEDIUM)** — CHANGELOG prescrit : « sans rappel depuis » → « et après le dernier rappel » (un rappel de plus de
    30 jours ne suffit pas) ; « ni celui qui payait à l'heure une facture à 60 jours » → « … à l'heure, mais plus de
    30 jours après sa date, … » (le paiement à l'heure dans les 30 jours était reconnu).
  - **LOW** — F3 : attributions de M1 (b, k, j au scalaire ; pas d) et de M9 (forme `contact_id` : d, f ; jointure
    croisée : inter-factures) corrigées ; F4 : mutations **M14** (`i.date` retiré du `GREATEST`, tuée par le seul
    scalaire du cas g) et **M15** (`GROUP BY id`, f dédoublée), AC5 asserte la liste comme `Vec` ordonné avec ses
    répétitions et le scalaire pour chaque cas ; F5 : la troncature `LIMIT 50` qui précède le départage est écrite
    (§ *Zones partagées*, C-29-2-7) et renvoyée à la 29-5 ; F7/R3 : `:2031` borné au refus « avant la facture », la
    course d'annulation d'un rappel dite (**C-29-2-14**) ; F7 : `:2029` « au plus un jour avant **la date de la
    facture** » ; F7/R6 : doc-comment `reconciliation_e2e.rs:2887` réécrit, assertions inchangées (AC6) ; F7 :
    brochure `:167` triée, non modifiée (AC8) ; R2 : `reminders.spec.ts`, `dunning-roundtrip.spec.ts`,
    `homepage-reminders.spec.ts` à AC7 ; R4 : contrôle aplati du PDF sur **six** formulations (relevé au sol :
    1, 1, 1, 1, 1, 3), apostrophe typographique couverte ; R5 : `paid_at` = `D` à minuit, patron `:2978`
    (`:3051-3086`) ; R7 : plages `:117-149`, `matching.rs:97-106` (départage `:104`), `:234-274` ; R8/F6 : annotation
    datée sous l'entrée P1, sans la réécrire ; R9 : raison de M12 réécrite (`CAST` dans la constante commune,
    troncature monotone), la comparaison `DATETIME`/`DATE` passée sous M13.
  - **Aucun refus.** Décomptes recomptés depuis la fiche : 10 AC, 7 tâches, 16 lignes de mutation (M1-M9, M10a,
    M10b, M11-M15), 11 cas d'AC5 (a-k) plus inter-factures et inter-sociétés, 8 lignes d'exemple, 14 choix
    (C-29-2-1 à 14, dont 13 et 14 neufs, 7 et 11 rectifiés par annotation datée), 3 modules. Symptômes grepés
    (valeurs : `29-3`, `#527`, `234-276`, `96-103`, `113-149`, `1-276`, `sans rappel depuis`, `à l'heure une`,
    `récente`, `proximité`, `transmis`, `Inchangés, sans modification`) sur la fiche et le registre : restent les
    renvois légitimes à la 29-3 (sérialisation, ordre règle contre facture, références), « sans rappel depuis
    30 jours » (forme juste du reliquat, `:2029` et C-29-2-9), l'entrée P1 ci-dessus (relevé daté, non réécrit) et les
    annotations datées du registre qui citent l'ancienne rédaction. Prochaine passe : **P3 ciblée, Haiku**, une lentille sur le seul diff de cette remédiation
    (`29-2-validate-prompt-p3-ciblee.md`).
- 2026-10-14 — **Validation P3, passe ciblée** (Haiku 5.5, une lentille sur le diff `e17368f8` ; rapport
  `kesh-gate-logs/29-2-validate-p3-ciblee.md`) : 0/0/2/3. Remédiée par l'orchestrateur, fiche seule :
  - **P3-1 (MEDIUM, né de la remédiation P2)** — réel : avec une échéance à D−40 et un rappel à D−5, l'ancre vaut D−5 et
    le paiement est proposé ; le texte prescrit du CHANGELOG disait le contraire. Corrigé en « plus de 30 jours après
    l'échéance et plus de 30 jours après le dernier rappel » (`grep -nF "après le dernier rappel"` : seule la ligne
    prescrite portait la forme fausse ; `:299` et `:313` étaient justes).
  - **P3-2 (MEDIUM)** — **réfuté comme défaut de la fiche** : les clés 29-5 et 29-6 et leurs paragraphes du fichier d'epic
    existent sur les branches `story/29-5` et `story/29-6` (`grep -n "29-5"
    ../kesh-29-5/_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` : l. 10, 107, 115) ; ils rejoignent
    celle-ci par **union au rebase**, comme toutes les clés de l'epic (consigne 7). La composition de la v0.13.4 se lit
    au fichier d'epic une fois réuni, non sur cette branche.
  - **P3-3 (LOW)** — le décompte brut « F 0/0/2/6 » de l'entrée P2 reprend le bilan du rapport F, qui annonce 6 LOW
    pour 5 titres de LOW ; relevé daté, non réécrit (B4). Les distincts « 2 MEDIUM, 12 LOW » restent justes.
  - **P3-4 (LOW)** — le titre de la story dit « après l'échéance, ou après un rappel » sans la borne des 30 jours :
    gardé comme titre (la règle est au § *La règle retenue*), le CHANGELOG prescrit porte la borne.
  - **P3-5 (LOW)** — le reliquat « plus de 30 jours après l'échéance » suppose E ≥ F, garanti par l'API
    (`routes/invoices.rs:765`, `:816`) ; le cas g de l'AC5 (E < F) est une fixture hors API, qui éprouve le `GREATEST`.
  **Validation close** : la remédiation de la passe ciblée ne touche que la fiche, aucune ligne de production
  (règle de la passe ciblée du `CLAUDE.md`). Trend : P1 1 HIGH / 3 MEDIUM → P2 0 / 3 → P3 ciblée 0 / 1 réel.
- 2026-10-14 — **Développement** (`bmad-dev-story`, Opus 5.5, autonomie) sur `fcd2ae75` : commits `8f483388` (tests
  rouges puis code, `kesh-db` et `kesh-api`) et `d45279f7` (manuel FR et PDF, `api-external.md`, CHANGELOG 0.13.4,
  registre, prompt de revue P1, doc-comments). Deux consignes de l'orchestrateur reçues en cours : ancre écrite par
  `macro_rules!` pour que la 29-5 y insère l'exigibilité (**C-29-2-15**) ; rejet d'une ligne sans proposition livré par
  la 29-6, mergée avant le tag (annotation de **C-29-2-8**, texte du manuel gardé). Label neuf
  `sec:algorithme-matching` (**C-29-2-16**). Écart à la fiche : **M14 est équivalente** (le repli `i.date` du terme du
  rappel garde `F`) — la fiche la disait tuée par le cas g ; remplacée par M14bis, tuée par l'assertion scalaire de g.
  Gates au dernier commit de code : backend 3611/3611 (6 ignorés), Vitest 1610/1610, check/lint/build verts, E2E 279
  passés, 8 échecs attendus (7 KF-029 + `sidebar-navigation.spec.ts:75`). Décomptes recomptés depuis les journaux :
  7 tests neufs, 20 mutations jouées (17 tuées, 3 équivalentes), 16 choix (C-29-2-1 à 16). Prochaine étape : revue de
  code P1, Sonnet, lentilles B/E/A (`29-2-review-prompt-p1.md`).
