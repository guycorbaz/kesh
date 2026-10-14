# Story 29.2 : Un paiement reçu après l'échéance, ou après un rappel, est proposé

## Status

**ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `16eeed21`
(kickoff de l'Epic 29), branche `story/29-2`, worktree `/home/gcorbaz/devel/kesh-29-2`. Validation P1 à lancer
(`29-2-validate-prompt-p1.md`, Sonnet, lentilles R et F). Choix consignés au registre
`epic-29-choix-autonomes-29-2.md`, **C-29-2-1 à C-29-2-9**. 10 AC. PR : **`closes #529`** (dans le titre ou le corps
de la PR seulement — aucun mot-clé de fermeture dans un message de commit, même en prose : B4 de la rétrospective 28).

⚠️ **Sérialisée avant la 29-3** (#527, tri des candidates), qui touchera après elle les mêmes zones de
`crates/kesh-api/src/routes/reconciliation.rs` et de `crates/kesh-db/src/repositories/reconciliation.rs` (§ *Zones
partagées avec la 29-3*). Les numéros de ligne de cette fiche sont relevés sur `16eeed21`.

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
| « fenêtre de 30 jours **centrée** sur la date de la facture » (`get_proposals`) | Depuis la Story 28-7 (#548), **une seule** fenêtre, `InvoiceDateWindow` (`crates/kesh-db/src/repositories/reconciliation.rs:150-199`) : `D − 30 ≤ F ≤ D + 1`, `D` date de valeur (à défaut de comptabilisation), `F` date de facture. Lue par la proposition (`find_unpaid_invoices_for_window`, `:234-276`, appelée par `routes/reconciliation.rs:658-665`) **et** l'acceptation (`accept_one_invoice`, étape 6, `routes/reconciliation.rs:1738-1764`) | **Périmée** dans la forme ; **le défaut subsiste** : la borne basse est `F ≥ D − 30`, sur la date de la facture, jamais sur son échéance ni ses rappels |
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

Exemples, `D` = 2026-06-15 :

| Facture `F` | Échéance `E` | Dernier rappel `R` | `A` | Proposée et acceptée ? |
|---|---|---|---|---|
| 2026-05-01 | — | — | 05-01 | non (`A < D − 30` = 05-16) — **inchangé** |
| 2026-05-01 | 2026-05-31 (30 jours net) | — | 05-31 | **oui** (cas courant de #529) |
| 2026-03-01 | 2026-03-31 | 2026-06-01 | 06-01 | **oui** (payé après un rappel) |
| 2026-03-01 | 2026-03-31 | 2026-06-01, **annulé** | 03-31 | non |
| 2026-03-01 | 2026-03-31 | — | 03-31 | non — reliquat (C-29-2-9) |
| 2026-06-17 | 2026-07-17 | — | 07-17 | non (`F > D + 1`) — **inchangé**, #548 |

**Une seule borne, deux lecteurs** (exigence de l'orchestrateur, #548 non rouvert) : `A` a **une** définition, une
expression SQL (`INVOICE_PAYMENT_ANCHOR_SQL` sur la jointure `invoice_payment_anchor_joins()`), lue par la
proposition (forme jointe, dans le `WHERE`) **et** par l'acceptation (forme scalaire, même expression, même jointure,
`WHERE i.id = ?`) ; la comparaison à `D` reste `InvoiceDateWindow::check`, en Rust, pour l'acceptation, et le `WHERE`
de la proposition, tenus d'accord par le test de parité étendu (AC6). Le refus d'un débit (#482,
`RECONCILIATION_TRANSACTION_NOT_CREDIT`, étape 4bis) précède la lecture de la facture et n'est pas touché.

**Pourquoi pas supprimer la borne basse** (écartée, C-29-2-2) : à montant exact, toute facture impayée de même reste
dû devient candidate **à score égal** (0,5 pour le seul montant) ; `propose_matches` départage alors par
`invoice_id` croissant (`kesh-reconciliation/src/matching.rs:96-103`) — la **plus ancienne** gagne, et c'est la seule
affichée et acceptée par *Accepter (N)*. Une vieille facture oubliée d'un client capterait le virement d'un autre :
le **faux rattachement muet** que le `CLAUDE.md` (§ *Un appariement automatique propose, il ne crée jamais*) désigne
comme le danger. L'ancre sur l'échéance et les rappels borne ce risque à une facture **vivante** — échue depuis moins
de 30 jours, ou relancée depuis moins de 30 jours.

**Pourquoi pas le choix manuel d'une facture à l'écran** (voie 2 de #529, écartée, C-29-2-2) : elle exige que
l'acceptation admette ce que la proposition ne propose pas (deux bornes, ce que l'orchestrateur exclut) ou une
nouvelle voie d'acceptation sans borne, plus un dialogue neuf (liste des factures ouvertes, sélection, i18n quatre
locales, E2E, captures) : hors du périmètre minimal. Le reliquat qu'elle couvrirait (paiement plus de 30 jours après
l'échéance **et** sans rappel) est écrit au manuel avec son contournement complet (AC8) ; l'ouverture d'une demande
d'évolution est **demandée** à l'orchestrateur (C-29-2-9).

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
  `GREATEST(i.date, COALESCE(i.due_date, i.date), COALESCE(DATE(rmd.last_sent_at), i.date))`.
  ⛔ **Chaque argument porte son `COALESCE`** : en MariaDB, `GREATEST` rend `NULL` dès qu'un argument est `NULL`, et
  `NULL >= ?` est faux — une facture sans échéance (`due_date` nullable, `migrations/20260416000001_invoices.sql:22`)
  ou sans rappel **disparaîtrait** des candidates. Le `GREATEST` sur `i.date` couvre aussi une échéance antérieure à la
  date (données d'avant #245 : la garde `due_date >= date` de `routes/invoices.rs:765`, `:816` ne porte que sur une
  paire modifiée).
- Doc-comments `///` : ce que vaut `A`, pourquoi les rappels **annulés** n'y comptent pas (C-29-2-3 : l'annulation
  soft d'un Admin retire déjà la ligne du niveau courant, `entities/invoice_reminder.rs:5-7`), pourquoi `DATE(sent_at)`
  (C-29-2-3 : `sent_at` est un `DATETIME(6)` ; un écart de fuseau d'un jour au plus, sans effet mesurable sur une borne
  de 30 jours — angle mort écrit), et **qui le lit** (les deux lecteurs d'AC2).

**AC2 — Deux lecteurs, la même expression.**

- **Proposition** : `find_unpaid_invoices_for_window` (`:234-276`) joint `invoice_payment_anchor_joins()` à côté de
  `amount_due_derived_joins()`, et remplace `AND i.date BETWEEN ? AND ?` par
  `AND i.date <= ? AND {INVOICE_PAYMENT_ANCHOR_SQL} >= ?` (bind `window.latest_invoice_date` puis
  `window.earliest_anchor_date`). Filtres de statut, `HAVING` du reste dû, `ORDER BY i.date ASC, i.id ASC` et
  `LIMIT 50` **inchangés** (C-29-2-7). Le `WHERE` peut citer une colonne de table dérivée (ce n'est pas un alias de
  `SELECT`) ; le tri et la troncature restent ceux de la 28-7.
- **Acceptation** : une fonction neuve `invoice_payment_anchor_date(executor, company_id, invoice_id) ->
  Result<Option<NaiveDate>, DbError>` rend `SELECT {INVOICE_PAYMENT_ANCHOR_SQL} FROM invoices i {joins} WHERE
  i.company_id = ? AND i.id = ?` — **même constante, même jointure** : aucune seconde écriture de la règle, ni en SQL
  ni en Rust (C-29-2-4). Scopée société (KF-002) bien que la facture soit déjà chargée sous ce scope.

**AC3 — La fenêtre compare la date de facture ET la date d'ancrage.**

`InvoiceDateWindow` :

- le champ `earliest_invoice_date` est **renommé** `earliest_anchor_date` (`D − 30`) — l'ancien nom mentirait ;
  `latest_invoice_date` (`D + 1`) reste ;
- `check(&self, invoice_date, anchor_date)` : `PaymentBeforeInvoice` si `invoice_date > latest_invoice_date`
  (**testé d'abord**, comme aujourd'hui), puis `PaymentTooLate` si `anchor_date < earliest_anchor_date` ;
- doc-comments à réécrire : celui de la structure (`:113-149`, dont « Sites qui portent la même borne basse » : la
  borne basse **de la date de facture** `F ≥ D − 1`… n'est pas celle-ci — relire et corriger le vocabulaire), de
  `INVOICE_WINDOW_DAYS_AFTER_INVOICE` (`:90-95` : « c'est la borne de #529 … que la Story 28-7 laisse où elle est »
  devient « au plus 30 jours après la date d'ancrage »), d'`InvoiceDateRefusal::PaymentTooLate` (`:163-165`) et de
  l'en-tête du module (`:5-13`). Le nom de la constante reste (contrat `window_days`), son doc-comment dit « après la
  date d'ancrage ».

**AC4 — L'acceptation lit l'ancre sous le même verrou, sans rien déplacer.**

Dans `accept_one_invoice` (`routes/reconciliation.rs:1621`), étape 6 (`:1738-1764`) : après les trois gardes
d'éligibilité (`invoice_not_validated`, `invoice_already_paid`, `invoice_journal_entry_not_set`) et **avant**
`InvoiceDateWindow::check`, lire `invoice_payment_anchor_date(&mut **tx, company_id, invoice.id)` ; `Ok(None)` (facture
disparue entre les deux lectures) → `INVOICE_NOT_FOUND` avec `details.invoiceId` comme l'étape 5 ; `Err` →
`DATABASE_ERROR` avec `details.message`, comme les autres lectures de la fonction — **jamais** un `AppError` global
(§ *Pattern batch* du `CLAUDE.md`).

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
- Le commentaire de `:1748-1750` (« Paiement tardif au-delà de 30 jours : #529, rapprochement manuel ») est réécrit.
- La date du règlement écrit (`paid_at_dt`) reste `payment_date(&bank_transaction)` : **inchangée**.

**AC5 — Tests `kesh-db` (rouges d'abord).**

`crates/kesh-db/src/repositories/reconciliation.rs`, `mod tests` (unitaires) : les deux tests de bornes
(`:619-643`) récrits sur la nouvelle signature — `check(F, A)` admet `A = D − 30` et refuse `A = D − 31` ;
`F = D + 1` admis, `F = D + 2` refusé `PaymentBeforeInvoice` **même quand** `A` est dans la fenêtre (ordre des
refus) ; `F = D + 2` et `A = D − 31` simultanés (impossible si `A ≥ F`, mais `check` est public) → `PaymentBeforeInvoice`.

`crates/kesh-db/tests/reconciliation_repository.rs` : un helper `insert_test_reminder(pool, company_id, invoice_id,
sent_at, cancelled)` (colonnes `NOT NULL` : `level_number ≥ 1`, `fee_amount`, `sent_at`, `channel ∈ {email, manual}`,
`subject`, `body` — migration `20260715000001_invoice_reminders.sql:33-58`), et un `due_date` optionnel sur
`insert_test_invoice` (paramètre ajouté, ou `UPDATE invoices SET due_date = ?` après insertion — au choix du dev,
sans changer les appels existants : leur `due_date` reste `NULL`). Un test (ou un test par cas, au choix) sur une
transaction à `D` = 2026-06-15, montants **distincts par cas** (aucune facture candidate d'un autre cas), qui asserte
**à la fois** la liste rendue par `find_unpaid_invoices_for_window` **et** `invoice_payment_anchor_date` pour chaque
facture :

| cas | `F` | `E` | rappels | `A` attendu | candidate |
|---|---|---|---|---|---|
| a | D − 45 | `NULL` | — | D − 45 | non |
| b | D − 45 | D − 30 | — | D − 30 | **oui** (borne) |
| c | D − 45 | D − 31 | — | D − 31 | non (borne) |
| d | D − 90 | D − 60 | D − 10 | D − 10 | **oui** |
| e | D − 90 | D − 60 | D − 10 **annulé** | D − 60 | non |
| f | D − 90 | D − 60 | D − 50, puis D − 20 | D − 20 | **oui** (le **dernier** rappel, pas le premier) |
| g | D − 45 | D − 50 (échéance antérieure) | — | D − 45 | non (`GREATEST` garde `F`) |
| h | D + 2 | D + 32 | — | D + 32 | non (`F > D + 1`, #548) |
| i | D − 20 | `NULL` | — | D − 20 | **oui** (inchangé : `E` nul ne fait pas disparaître) |

Plus un cas **inter-factures** : le rappel d'une facture X ne déplace pas l'ancre d'une facture Y (jointure par
`invoice_id`). Les tests existants du fichier (`:256-313` « 30 day window », `:315+` ordre et `LIMIT 50`) restent
verts **sans modification d'assertion** — leurs factures n'ont ni échéance ni rappel ; seul le nom du champ renommé
change là où ils le lisent (aucun, à vérifier au `grep -n earliest_invoice_date crates`).

**AC6 — Tests `kesh-api` : parité étendue, et le paiement tardif aboutit.**

`crates/kesh-api/tests/reconciliation_e2e.rs` :

- **Parité** : un test neuf sur le patron de `proposal_and_acceptance_share_the_invoice_date_window` (`:2888-2958`)
  — mêmes helpers `proposed_couples_287` / `accept_couples_287` —, sur les cas a à h d'AC5 (échéance posée par
  `UPDATE invoices SET due_date`, rappels insérés en SQL brut ; `seed_validated_invoice` n'a pas de paramètre
  d'échéance). Assertions : `proposed == accepted`, l'ensemble attendu exact, et pour chaque refus sa raison
  (`payment_date_outside_window` pour a, c, e, g ; `payment_date_before_invoice_date` pour h). `D` en plein exercice
  2026 (`insert_fake_fiscal_year`) ; vérifier que `D − 90` y reste.
- **Le cas de #529 bout à bout** : transaction à `D`, facture à `D − 45` échue à `D − 15` ; `GET /proposals` la
  propose (`candidateType = invoice`, `invoiceId`), `POST /accept` l'accepte, et la facture est réglée
  (`paid_at` = date de valeur), comme `post_accept_reconciles_transaction_and_invoice` (`:973`) l'asserte déjà.
- **Inchangés, sans modification** : `post_accept_rejects_payment_date_outside_window` (`:2726-2789`, facture sans
  échéance : `A = F`, toujours refusée, `window_days == 30`), les deux tests de la 28-7 (`:2888`, `:2978`), et tout
  test qui lit les propositions (inventaire (c)).

**AC7 — L'inventaire des tests dont le sens peut changer est clos, et relu au gate.**

Ensemble clos des fichiers qui lisent les propositions ou la fenêtre (§ *Inventaire (c)*) : leurs factures n'ont pas
d'échéance, sauf `invoice_write_off.rs:81` (`due_date = D()`, égale à la date) — `A = F`, sens inchangé. Le Dev Agent
Record le **revérifie** par la commande de l'inventaire (c) et en copie la sortie. **E2E** : les specs partagent la
base `kesh_e2e` et y laissent des factures **avec échéance** (contacts « 30 jours net » de `invoices.spec.ts`,
`api-keys.spec.ts`, `inbox-import.spec.ts`, `invoices_echeancier.spec.ts`) : une facture d'une spec, échue récemment,
peut devenir candidate d'une transaction d'une spec de réconciliation **de même montant**. Les specs de
réconciliation tirent des montants uniques (`reconciliation-amount-due.spec.ts`, `Date.now()`), mais le Dev Agent
Record déclare, fichier par fichier contre `docs/testing.md` § *Les échecs attendus*, tout rouge d'une spec de
réconciliation au gate E2E.

**AC8 — Le manuel, l'API externe et le CHANGELOG disent la nouvelle règle (PDF compris).**

`docs/manual/fr/user-manual.tex` (seul manuel rédigé : `docs/manual/{de,en,it}/` ne portent qu'un `README.md`) :

- **`:1305`** (§ rappels, « Ce que réclame un rappel ») : « La réconciliation bancaire ne propose toutefois la facture
  que si le paiement arrive dans les 30 jours de la date de la facture --- ce qui, après un rappel, est rarement le
  cas~: enregistrez alors le règlement depuis la fiche » devient : la réconciliation propose la facture si le paiement
  arrive **au plus 30 jours après le dernier rappel** (ou après l'échéance, ou la date de la facture, la plus tardive
  des trois) ; un rappel **annulé** ne compte pas ; renvoi à `\S\ref{sec:facture-posterieure}` ou à l'algorithme.
- **`:2029`** (§ *Algorithme de matching*) : « tombe au plus 30 jours après la date de la facture --- sa date, pas son
  échéance --- » devient la règle d'AC1, dite pour l'utilisateur (« au plus 30 jours après la plus tardive de ces
  dates~: la date de la facture, son échéance, son dernier rappel ») ; la phrase « Un paiement reçu plus de 30 jours
  après la date de la facture --- une facture payée avec retard, ou après un rappel --- n'est en revanche pas
  proposé » devient le **reliquat** : plus de 30 jours après l'échéance **et** sans rappel depuis 30 jours ; le
  remède : enregistrer le règlement depuis la fiche, **puis rejeter la transaction** à la réconciliation
  (*Rejeter*), sans quoi elle reste en attente et pourrait être passée une seconde fois (C-29-2-8).
- **`:2031`** (`\label{sec:facture-posterieure}`, dernières phrases) : « Il n'a en revanche aucune limite de 30 jours
  après la facture~: c'est par lui qu'on enregistre un paiement reçu avec retard » devient « … un paiement que la
  réconciliation ne propose pas (reçu plus de 30 jours après l'échéance et le dernier rappel) », avec le même rejet de
  la transaction. Le `\label` est conservé (référencé à `:2902`).
- **`:2902`** (FAQ « aucun match automatique ») : « plus de 30 jours après la date de la facture --- de sa date, pas de
  son échéance~: une facture payée avec retard, ou après un rappel, n'est pas proposée » devient le reliquat.
- `:836`, `:914`, `:916`, `:946`, `:949` (« 30 jours net » des conditions de paiement) et `:95` (session) : hors
  sujet, **non modifiés**.

PDF régénéré (`make fr` dans `docs/manual/`, sous le verrou de gate) et commité ; contrôle **aplati** :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | grep -o "sa date, pas son échéance"` ne
rend plus rien, et `grep -o "dernier rappel[^.]*"` rend la nouvelle règle. Gardes textuelles :
`cargo nextest run -E 'binary(textes_coherents)'` vert.

`docs/api-external.md` § *Accepter des propositions de rapprochement* (`:547-567`) : une phrase *(depuis la v0.13.4)* —
une facture est acceptée si le paiement arrive au plus 30 jours après la plus tardive de sa date, de son échéance et
de son dernier rappel non annulé, et au plus un jour avant sa date ; au-delà, `RECONCILIATION_INVOICE_NOT_ELIGIBLE`
avec `payment_date_outside_window` (`window_days` = 30), en deçà `payment_date_before_invoice_date` ; une acceptation
autrefois refusée pour ce motif peut désormais aboutir (C-29-2-5).

`CHANGELOG.md` : `## [0.13.4] — Non publié` (à créer au-dessus de `## [0.13.3] — 2026-10-14` s'il n'existe pas encore
au rebase ; sinon y ajouter), section `### Corrigé` : « **Un paiement reçu après l'échéance, ou après un rappel, est
proposé à la réconciliation ([#529](https://github.com/guycorbaz/kesh/issues/529)).** Une facture n'était proposée
— et acceptée — que si le paiement arrivait au plus 30 jours après **sa date**, si bien qu'un client qui payait à
30 jours net avec un peu de retard, ou après un rappel, n'était jamais reconnu. Les 30 jours courent désormais de la
plus tardive des trois dates : la date de la facture, son échéance, son dernier rappel (un rappel annulé ne compte
pas). La proposition et l'acceptation appliquent toujours la même règle. Un paiement reçu plus de 30 jours après
l'échéance, sans rappel depuis, n'est toujours pas proposé : enregistrez-le depuis la fiche de la facture, puis
rejetez la transaction. API : mêmes codes et raisons qu'avant ; une acceptation refusée en
`payment_date_outside_window` peut désormais aboutir. »

Aucun texte de l'écran ne change : **ni clé i18n, ni capture** (C-29-2-6). Le repli de `failed-proposal-label.ts` pour
`payment_date_outside_window` est le libellé générique (« n'est pas éligible ») et ne cite aucune durée.

**AC9 — Mutations (chacune tuée par un test nommé au Dev Agent Record).**

| # | Mutation | Tuée par |
|---|---|---|
| M1 | retirer `COALESCE(i.due_date, i.date)` du `GREATEST` | AC5 b, d ; AC6 cas #529 |
| M2 | retirer le terme du rappel | AC5 d, f |
| M3 | retirer `WHERE cancelled_at IS NULL` | AC5 e |
| M4 | `MAX(sent_at)` → `MIN(sent_at)` | AC5 f |
| M5 | `COALESCE(i.due_date, i.date)` → `i.due_date` (sans repli) | AC5 a… i : i (`E` nul) disparaît |
| M6 | acceptation : passer `invoice.date` comme ancre au lieu de lire `invoice_payment_anchor_date` | AC6 parité (b, d acceptées) |
| M7 | proposition : `{ANCHOR} >= ?` → `{ANCHOR} > ?` | AC5 b |
| M8 | `check` : tester `PaymentTooLate` avant `PaymentBeforeInvoice` | test unitaire d'ordre (AC5) |
| M9 | jointure des rappels sur `rmd.invoice_id = i.contact_id` (ou sans `ON` correct) | cas inter-factures (AC5) |

Restauration de chaque mutation : `git checkout -- <fichier> && touch <fichier>` (`docs/agents-orchestration.md`
§ 7). Une mutation équivalente se déclare avec sa raison.

**AC10 — Gates.**

`kesh-db` touché : **gate complet même en boucle de revue** (`CLAUDE.md` § *Pendant une boucle de revue*), base remise
à zéro avant chaque gate complet (KF-039), `scripts/test-fast.sh` sous `flock` ; frontend (`npm run check`,
`lint-i18n-ownership`, `test:unit`, `build`) — aucun fichier frontend n'est censé changer, le gate le confirme ; E2E
complet au **dernier commit de code** (base `kesh_e2e_292`, port 3043, recette de `docs/testing.md` à la lettre).
Le Dev Agent Record ne déclare que ce qui a tourné, chiffres recomptés.

## Tasks / Subtasks

- [ ] **T1 — Tests rouges** (AC5, AC6) : helpers de rappel et d'échéance, cas a à i, inter-factures, parité HTTP, cas
      de #529 ; constater le rouge (compilation ou assertion) avant T2.
- [ ] **T2 — `kesh-db`** (AC1, AC2, AC3) : jointure et expression d'ancrage, `find_unpaid_invoices_for_window`,
      `invoice_payment_anchor_date`, renommage du champ, `check(F, A)`, doc-comments.
- [ ] **T3 — `kesh-api`** (AC4) : étape 6 d'`accept_one_invoice` ; commentaire de `get_proposals` (`:655-657`) qui
      décrit la fenêtre ; aucun autre site (inventaire (a)).
- [ ] **T4 — Revérifier l'inventaire (c)** (AC7) et copier la sortie au Dev Agent Record.
- [ ] **T5 — Manuel FR + PDF, `api-external.md`, CHANGELOG** (AC8).
- [ ] **T6 — Propagation** : le grep de § *Propagation*, trié à la main, sortie résumée au Dev Agent Record.
- [ ] **T7 — Mutations** (AC9) et **gates** (AC10) ; Dev Agent Record.

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
production `routes/reconciliation.rs:658`) et l'acceptation (`accept_one_invoice`, étape 6). Le score
(`kesh-reconciliation/src/matching.rs`) ne lit aucune date. Le frontend ne calcule aucune fenêtre.

### Inventaire (b) — les sites non résolus (bornes voisines qui ne liront pas l'ancre)

| Site | Ce qu'il borne | Sort |
|---|---|---|
| `invoice_settlements_write.rs:115`, `:489` (`settle_invoice`, `write_off_invoice`) | date de règlement saisie ≥ facture − 1, sans borne haute | **non résolu, assumé** (inchangé depuis C-28-7-7) : ne règle pas une transaction ; c'est le contournement du reliquat |
| CHECK `chk_invoices_paid_at_after_date` (`migrations/20260419000001_invoice_paid_at.sql:32-33`) | `DATE(paid_at) >= date − 1` | **non touché** : borne de la date de facture, que la story ne change pas — aucune migration |
| `SettleInvoiceDialog.svelte`, `WriteOffDialog.svelte` (`min={invoiceDate}`) | date saisie ≥ facture | **non touché**, hors classe |
| Éligibilité des rappels (`repositories/dunning_eligibility.rs`) | quelles factures relancer | **hors classe** : ne choisit aucune facture pour une transaction ; ne lit pas la fenêtre |
| `sent_at` des rappels **manuels** (`routes/dunning_reminders.rs:261`, saisi, `≤ now` UTC) et e-mail (`Utc::now`) | date du rappel | **angle mort écrit** (C-29-2-3) : `DATE(sent_at)` mêle une date saisie et une date UTC ; un jour d'écart au plus |

### Inventaire (c) — les tests existants dont le sens pourrait changer

```
$ for f in $(grep -ln 'reconciliation/proposals\|find_unpaid_invoices_for_window' crates/*/tests/*.rs); do echo "$f: proposals=$(grep -c 'reconciliation/proposals\|find_unpaid_invoices_for_window(' $f) due_date=$(grep -c 'due_date\|dueDate' $f) reminders=$(grep -c 'invoice_reminders' $f)"; done
crates/kesh-api/tests/bank_accounts_closed_e2e.rs: proposals=1 due_date=1 reminders=0
crates/kesh-api/tests/reconciliation_rules_e2e.rs: proposals=8 due_date=0 reminders=0
crates/kesh-db/tests/invoice_write_off.rs: proposals=2 due_date=1 reminders=0
crates/kesh-db/tests/reconciliation_repository.rs: proposals=11 due_date=0 reminders=0
crates/kesh-api/tests/reconciliation_e2e.rs: proposals=12 due_date=0 reminders=0
```

- `bank_accounts_closed_e2e.rs:451` : `due_date: None` ; `invoice_write_off.rs:81` : `due_date: Some(D())`, égale à
  la date de la facture → `A = F` dans les deux cas, sens inchangé.
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

### Zones partagées avec la 29-3 (#527, tri des candidates)

- `find_unpaid_invoices_for_window` (`repositories/reconciliation.rs:234-276`) : la 29-2 change le `WHERE` et les
  jointures ; la 29-3 touchera probablement l'`ORDER BY` / `LIMIT 50` adjacents (C-28-7-10, exclusion de la 28-7 :
  « priorité au montant exact avant le `LIMIT 50` »).
- `get_proposals`, passe 2 (`routes/reconciliation.rs:655-665`, appel de la fenêtre et son commentaire) et passe 4
  (`:744-870`, construction et ordre des candidates) : la 29-2 ne touche que le commentaire de la passe 2.
- `propose_matches` (`kesh-reconciliation/src/matching.rs:96-103`, départage `invoice_id ASC`) : non touché ici ;
  **la 29-2 rend ce départage plus coûteux** (C-29-2-7) — à score égal, la facture la plus ancienne, désormais
  possiblement échue et relancée, passe devant une facture récente. Critère à transmettre à la 29-3 : départager les
  égalités de score par la proximité de la date d'ancrage (ou de la date de facture) à la date de paiement.

### Exclusions écrites (périmètre minimal)

- **Choix manuel d'une facture ouverte à l'écran** (voie 2 de #529) : non fait (C-29-2-2) ; demande d'évolution à
  ouvrir (C-29-2-9).
- **Montant différent** (frais bancaires déduits, paiement du rappel frais compris — #401) : hors #529, inchangé.
- **Ordre des candidates, `LIMIT 50`, départage** : 29-3 (#527).
- **Migration, index** : aucun. Les jointures s'appuient sur `idx_invoice_reminders_company_invoice` /
  `idx_invoice_reminders_invoice` ; le filtre `i.date <= ?` garde une plage sur
  `idx_invoices_company_validated_unpaid_date`, sur un ensemble que `paid_at IS NULL` réduit aux factures ouvertes.
  Pas de mesure de performance exigée (factures ouvertes d'une PME : dizaines).
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

- Issue #529 ; #548 (Story 28-7, fiche `28-7-facture-posterieure-paiement.md`, C-28-7-1 à 16) ; #482 (28-6) ; #527
  (29-3) ; #401 (frais de rappel).
- `crates/kesh-db/src/repositories/reconciliation.rs:1-276`, `:590-668` ;
  `crates/kesh-api/src/routes/reconciliation.rs:586-876`, `:1621-1770` ;
  `crates/kesh-db/src/repositories/invoice_settlements.rs:42-113` (patron des tables dérivées) ;
  `crates/kesh-db/src/entities/invoice_reminder.rs` ; `migrations/20260715000001_invoice_reminders.sql`.
- `docs/manual/fr/user-manual.tex:1305`, `:2029`, `:2031`, `:2902` ; `docs/api-external.md:547-567`.
- `docs/agents-orchestration.md` ; `CLAUDE.md` § *Review Iteration Rule*, § *Pattern batch*, § *Un appariement
  automatique propose, il ne crée jamais*, § *Test Locally First*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-14 — Fiche créée (`bmad-create-story`, Opus 5.5, autonomie) sur `16eeed21`. Prémisses de #529 et de la
  consigne établies au code (deux périmées : la fenêtre « centrée », la promesse du manuel). Choix C-29-2-1 à 9.
