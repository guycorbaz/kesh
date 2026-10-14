# Story 29.5 : À score égal, la facture échue la première passe en tête (art. 87 CO)

## Status

**ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `fcd2ae75` (`main`
après le kickoff de l'Epic 29), branche `story/29-5`, worktree `/home/gcorbaz/devel/kesh-29-5`. Story **ajoutée à
l'Epic 29** par l'orchestrateur le 2026-10-14. Validation P1 à lancer (`29-5-validate-prompt-p1.md`, Sonnet,
lentilles R et F). Choix consignés au registre propre à la story, `epic-29-choix-autonomes-29-5.md`,
**C-29-5-1 à C-29-5-10** (à verser par union au registre de l'epic). 9 AC. PR : **`closes #668`** — dans le titre ou
le corps de la PR **seulement**, jamais dans un message de commit (B4 de la rétrospective 28).

⚠️ **Sérialisée derrière la 29-2 (#529) et la 29-3 (#527)**, non mergées à la date de la fiche, qui touchent les
mêmes fichiers (§ *Zones partagées*). La 29-2 la suppose **mergée avant le tag v0.13.4** (C-29-2-13) : sans elle, la
fenêtre élargie de la 29-2 sortirait avec un départage et une troncature qui l'aggravent. Le développement **commence par un rebase** sur `origin/main` une fois les deux
mergées, et **relit** tous les numéros de ligne de cette fiche (relevés sur `fcd2ae75`). Indépendante de la 29-4
(#651, chemins d'écriture des comptes bancaires) : aucune zone commune.

## Story

**As a** utilisateur qui rapproche ses relevés (Comptable ou Admin),
**I want** que, entre deux factures que rien ne distingue dans le paiement, la réconciliation propose celle sur
laquelle la loi impute le paiement — la facture **échue la première** —,
**so that** « Accepter » ne solde pas une facture pas encore due, ou plus récente, à la place d'une facture échue du
même client : le lettrage et les rappels viseraient la mauvaise pièce, et le client recevrait un rappel pour une
facture qu'il a, selon la loi, payée.

Issue : [#668](https://github.com/guycorbaz/kesh/issues/668) (P2, bug) et son **commentaire de rectification** du
2026-10-14 (Guy). Epic : `_bmad-output/planning-artifacts/epic-29-preparer-cloture.md` § *29-5*. Cible : **v0.13.4**,
avec la 29-2 qui aggrave le défaut (C-29-5-1).

### Le critère légal

**Art. 87 CO** (imputation légale, RS 220), en substance : faute de déclaration du débiteur (art. 86 al. 1) et
d'imputation portée sur la quittance (art. 86 al. 2), le paiement s'impute

1. sur la dette **exigible** ;
2. entre plusieurs dettes exigibles, sur celle qui a fait l'objet des **premières poursuites** ; à défaut de
   poursuites, sur la dette **échue la première** (al. 1) ;
3. entre dettes échues **en même temps**, proportionnellement (al. 2) ;
4. si **aucune** n'est échue, sur celle qui présente le moins de **garanties** pour le créancier (al. 3).

⚠️ Le texte est cité **en substance** : Fedlex (`fedlex.admin.ch`, RS 220) exige un navigateur à JavaScript et n'a pas
pu être relu à la source pendant la fiche ; un résumé secondaire (juriup.ch) confirme la règle « dette exigible la
plus ancienne » et l'imputation proportionnelle à échéances identiques. **La validation P1 confronte la phrase au
texte officiel** si elle le peut (axe nommé du prompt). La règle retenue ci-dessous ne dépend que des alinéas 1 et 2
tels que les rapportent l'issue et ce résumé.

### Les prémisses, établies au code (`fcd2ae75`)

| Prémisse (issue #668 et sa rectification ; consigne de l'orchestrateur) | Au code | Verdict |
|---|---|---|
| `propose_matches` départage deux factures de même score par `invoice_id` croissant | `crates/kesh-reconciliation/src/matching.rs:97-105` : `b.score.total.partial_cmp(&a.score.total)…then_with(\|\| a.invoice_id.cmp(&b.invoice_id))` | **Vraie** |
| Une facture désignée par le paiement (référence, numéro) prime déjà par le score | `reference_score` (`matching.rs:121-147`) : 1 si la référence et le numéro se contiennent, 0,5 sur un préfixe commun ≥ 4 ; poids 0,40 (`:80`). Une facture désignée a un score **strictement supérieur** à une facture du même client non désignée, au même montant | **Vraie** — non touchée (C-29-5-6) |
| Le départage se fait aussi par l'`ORDER BY` / `LIMIT 50` de `find_unpaid_invoices_for_window` | `crates/kesh-db/src/repositories/reconciliation.rs:258-259` : `ORDER BY i.date ASC, i.id ASC LIMIT 50`. Il ne départage pas le score — `propose_matches` retrie tout — mais il **choisit quelles** candidates sont scorées : au-delà de 50, les plus récentes **par date** sont écartées, fût-ce celle que l'art. 87 désigne | **Vraie, au sens de la troncature** — et **aggravée par la 29-2** : sa fenêtre ancrée fait entrer, à 30 jours net, les factures jusqu'à `F ≥ D − 60`, et des factures anciennes **non échues** (échéance lointaine) qui, plus anciennes par date, passent devant une facture échue dans l'`ORDER BY` et peuvent la pousser hors des 50. La 29-2 laisse ordre et troncature inchangés et renvoie ce point ici (C-29-2-7, C-29-2-13, § *Zones partagées* de sa fiche, état `e17368f8`). Traité par AC3 : l'ordre SQL devient celui de l'art. 87 (C-29-5-4) |
| Les données de l'exigibilité sont disponibles au départage | `Invoice` porte `date` (`NaiveDate`) et `due_date` (`Option<NaiveDate>`, nullable — `migrations/20260416000001_invoices.sql:22`) ; le triplet candidat passé à `propose_matches` (`routes/reconciliation.rs:732-746`) contient l'`Invoice` complète | **Vraie** : aucune donnée à ajouter au triplet |
| « premières poursuites (en Kesh : rappels ?) » | Kesh **n'enregistre aucune poursuite** : `grep -rni poursuite crates frontend/src` ne rend que le libellé du dernier niveau de rappel, « Dernier rappel avant poursuite » (`entities/email_template_defaults.rs:96`). Un **rappel** est une interpellation (mise en demeure, art. 102 CO), **pas** une poursuite (réquisition de poursuite, LP) | **Corrigée** : un rappel ne vaut pas poursuite ; le critère n'a pas de donnée en Kesh (C-29-5-3) |
| Attendu d'origine de l'issue : « la récente passe en tête », départage par la proximité de la date d'ancrage | Contredit par l'art. 87 (rectification de Guy) ; la date d'ancrage de la 29-2 (`max(date, échéance, dernier rappel)`) mêle les rappels, qui ne changent ni l'exigibilité ni l'ordre légal | **Écartée** (C-29-5-2) |

### La règle retenue (C-29-5-2)

Pour une facture de date `F` et d'échéance `E` (nullable), on pose sa **date d'exigibilité**

> `X = max(F, E)` — `E` absente valant `F`

(une facture sans échéance est exigible dès sa date : art. 75 CO, « à défaut de terme, l'obligation est exigible
immédiatement » ; le `max` couvre une échéance antérieure à la date, données d'avant #245). **À score égal**, les
candidates se rangent par

> `X` croissante, puis `F` croissante, puis `invoice_id` croissant.

**Pourquoi cela suffit à « d'abord les échues »** : pour un paiement de date `D`, une facture est échue si `X ≤ D` et
non échue si `X > D` ; toute facture échue a donc une `X` **plus petite** que toute facture non échue. Trier par `X`
place les échues en tête **sans lire `D`**, puis, parmi elles, celle **échue la première** (al. 1). Parmi les non
échues — l'al. 3 renvoie aux garanties, que Kesh ignore —, la plus proche de son échéance passe d'abord : choix par
défaut écrit (C-29-5-5). Une échéance lointaine, ou mal saisie (2099), passe **derrière** : c'est l'angle mort
C-29-2-11 de la 29-2, fermé.

⚠️ **Conséquence à écrire au doc-comment** : l'équivalence « échues d'abord ⇔ `X` croissante » ne vaut que tant que
l'ordre des non échues est lui aussi `X` croissante. Qui ordonnera un jour les non échues autrement (garanties,
al. 3) devra réintroduire la date du paiement dans la clé.

Exemples, `D` = 2026-06-15, deux factures du même client, même reste dû, sans référence (score égal) :

| Facture A (`F`, `E`) | Facture B (`F`, `E`) | `X` A / B | En tête | Avant la story (`invoice_id`, A créée d'abord) |
|---|---|---|---|---|
| 05-26, 07-25 (non échue) | 06-10, 06-15 (échue le jour même) | 07-25 / 06-15 | **B** | A — **le cas de l'issue** |
| 05-20, 06-14 | 05-25, 06-05 | 06-14 / 06-05 | **B** (échue la première, quoique plus récente) | A |
| 05-20, — | 05-25, 06-20 | 05-20 / 06-20 | **A** (sans échéance : exigible dès sa date) | A |
| 05-20, 06-10 | 05-25, 06-10 | 06-10 / 06-10 | **A** (même échéance : la plus ancienne, al. 2 — C-29-5-5) | A |
| 05-20, 05-10 (échéance antérieure) | 05-15, — | 05-20 / 05-15 | **B** (`max` garde `F`) | A |

### Ce que la règle ne fait pas

- **Elle ne touche pas au score** : une facture désignée par la référence garde la tête, échue ou non (art. 86 : la
  déclaration du débiteur prime l'imputation légale). Une facture au montant exact passe devant une facture à
  quelques centimes près, quelle que soit leur exigibilité (C-29-5-6).
- **Elle ne vaut, en droit, qu'entre dettes d'un même débiteur.** À score égal, deux factures de **clients
  différents** que rien ne distingue (montant seul, aucun nom ne correspond) se rangent aussi par `X` : ce n'est plus
  l'art. 87, c'est un ordre par défaut — la proposition reste une proposition (`CLAUDE.md` § *Un appariement
  automatique propose, il ne crée jamais*). Écrit au manuel et au doc-comment (C-29-5-7).
- **Elle n'impute pas proportionnellement** (al. 2) : Kesh propose une seule facture ; entre deux échéances
  identiques, la plus ancienne. L'utilisateur qui veut répartir enregistre les règlements depuis les fiches
  (C-29-5-5).
- **Elle ne lit pas les poursuites** (aucune donnée) ni les **rappels** (qui n'en sont pas) (C-29-5-3).
- **Elle ne change pas l'acceptation** : `accept_one_invoice` rescore **une** facture (`routes/reconciliation.rs:1779-
  1790`) ; l'ordre y est sans objet.

## Critères d'acceptation

**AC1 — La date d'exigibilité a deux écritures, dans un même module, tenues d'accord par un test.**

Dans `crates/kesh-db/src/repositories/reconciliation.rs`, à côté de `payment_date` et d'`InvoiceDateWindow` (C-29-5-8) :

- `pub fn invoice_exigibility_date(invoice: &Invoice) -> NaiveDate` : `max(invoice.date,
  invoice.due_date.unwrap_or(invoice.date))`. ⛔ Jamais une comparaison d'`Option<NaiveDate>` : `None < Some(_)` en
  Rust placerait une facture **sans** échéance avant toutes les autres (mutation M3).
- `pub fn imputation_order(a: &Invoice, b: &Invoice) -> std::cmp::Ordering` : `(X, F, id)` croissants, dans cet ordre.
- `pub const INVOICE_EXIGIBILITY_DATE_SQL: &str` : `CAST(GREATEST(i.date, COALESCE(i.due_date, i.date)) AS DATE)` —
  prérequis alias `i`, comme les autres expressions du module. ⛔ `COALESCE` obligatoire : en MariaDB, `GREATEST`
  rend `NULL` dès qu'un argument l'est, et un `NULL` trié en `ASC` passe **en tête** (mutation M9).
- Doc-comments `///` : l'art. 87 (en substance, avec la réserve de la § *Le critère légal*), l'art. 75 pour la
  facture sans échéance, l'équivalence « échues d'abord ⇔ `X` croissante » et sa condition, les deux écritures et le
  test qui les tient d'accord (AC4), ce que la règle ne fait pas (score, clients différents, al. 2, poursuites).
- **Si la 29-2 est mergée** (attendu) : son expression d'ancrage (`INVOICE_PAYMENT_ANCHOR_SQL`, fiche 29-2 AC1) est
  **récrite en composant** celle-ci — `CAST(GREATEST({INVOICE_EXIGIBILITY_DATE_SQL}, COALESCE(DATE(rmd.last_sent_at),
  i.date)) AS DATE)` —, pour que `GREATEST(i.date, COALESCE(i.due_date, i.date))` ne s'écrive qu'une fois (règle DRY
  du `CLAUDE.md`). Équivalence algébrique (`GREATEST` est associatif) ; les tests de la 29-2 restent verts **sans
  modification d'assertion**, et le Dev Agent Record le constate (C-29-5-9). Si la forme mergée de la 29-2 diffère de
  sa fiche, le dev compose sur ce qui est mergé et l'écrit.

**AC2 — `propose_matches` départage par l'ordre d'imputation.**

`crates/kesh-reconciliation/src/matching.rs:97-105` : le `then_with(|| a.invoice_id.cmp(&b.invoice_id))` devient le
départage par `kesh_db::repositories::reconciliation::imputation_order` des **factures** des deux propositions. Le
vecteur `out` ne portant que `invoice_id`, le dev garde la facture à côté de la proposition le temps du tri (paire,
index, ou référence — au choix, sans clone superflu), puis rend `Vec<MatchProposal>` : **signature et forme JSON
inchangées**. Le score reste la **première** clé (`score.total` décroissant), `imputation_order` la seconde ;
`invoice_id` reste dans la clé (troisième composante d'`imputation_order`), donc l'ordre reste **total et
déterministe** (« utile aux tests E2E + audit reproductibilité », commentaire actuel `:95-96`, à réécrire). Le
doc-comment du module (`:1-25`) et de la fonction (`:55-61`, « triées par `score.total DESC` ») disent le départage.
Le smoke de performance `propose_matches_handles_1000_x_500_under_200ms` (`:415`) reste vert.

**AC3 — La troncature du `LIMIT 50` suit le même ordre.**

`find_unpaid_invoices_for_window` (`kesh-db/src/repositories/reconciliation.rs:234-276`) : `ORDER BY i.date ASC, i.id
ASC` devient `ORDER BY {INVOICE_EXIGIBILITY_DATE_SQL} ASC, i.date ASC, i.id ASC`, **avant** le `LIMIT 50` inchangé
(C-29-5-4). Aucun paramètre lié de plus (`D` n'entre pas dans la clé). Filtres, jointures, `HAVING` : inchangés —
ceux que la 29-2 aura posés compris. Doc-comment de la fonction (point 3, `:211-216`, « Résultat trié par `date ASC,
id ASC` (C-28-7-10) ») et en-tête du module (`:10-15`, « par `(date, id)` croissants ») réécrits : l'ordre est celui
de l'imputation, **pour que la troncature n'écarte jamais, parmi des candidates de même score, celle que
`propose_matches` placerait en tête** — propriété qui tient parce que les deux ordres coïncident (les candidates
gardées forment un préfixe de l'ordre d'imputation dans chaque classe de score). La troncature reste aveugle au
**score** : une facture désignée par sa référence peut encore être écartée au-delà de 50 candidates (C-28-7-10,
inchangé, écrit au doc-comment).

**AC4 — Tests `kesh-db` (rouges d'abord).**

`crates/kesh-db/src/repositories/reconciliation.rs`, `mod tests` (unitaires, pas de base) :
`invoice_exigibility_date` sur `E` nulle (`X = F`), `E > F` (`X = E`), `E = F`, `E < F` (`X = F`) ;
`imputation_order` sur les cinq lignes du tableau d'exemples (entrées et ordre attendu **écrits en dur**), plus
`X`, `F` égales et `id` différents.

`crates/kesh-db/tests/reconciliation_repository.rs` — un `due_date` à poser sur les factures de test
(`UPDATE invoices SET due_date = ? WHERE id = ?` après `insert_test_invoice`, ou paramètre ajouté sans changer les
appels existants — au choix du dev ; **si la 29-2 a déjà ajouté une aide, la réutiliser**) :

- `find_unpaid_invoices_orders_by_exigibility_before_limit` : transaction à `D` = 2026-06-15 ; **51** factures de
  même reste dû, toutes dans la fenêtre de dates, dont l'ordre d'exigibilité **diverge** de `(date, id)` : la facture
  **la plus récente par date** (`F = D`) porte la plus petite `X` (`E` = `F`, exigible) quand les cinquante autres,
  plus anciennes (`F` de `D − 30` à `D − 1`), ont une échéance lointaine (`E` = `D + 40` et au-delà, distinctes) ;
  insertion dans un ordre où `id ASC` diverge aussi. Assertions : la liste rendue vaut, **écrite en dur** par une
  expression indépendante de la production (pas d'appel à `imputation_order` pour construire l'attendu — leçon de
  l'Epic 23, *Inventorier les sites non résolus*), la facture `F = D` **en tête**, puis les 49 suivantes par `E`
  croissante ; la 51ᵉ (`E` la plus lointaine) est écartée. Montage : `assert_ne!` entre l'ordre attendu et `(date,
  id)` — sinon le test passerait à vide sous l'ancien `ORDER BY` (mutation M8).
- `exigibility_date_sql_and_rust_agree` (**parité**, C-29-5-8) : six factures — `E` nulle, `E > F`, `E = F`, `E < F`,
  `E` lointaine (2099-12-31), `F` au premier jour de l'exercice —, `SELECT i.id, {INVOICE_EXIGIBILITY_DATE_SQL} FROM
  invoices i WHERE i.company_id = ?` (décodée en `NaiveDate` : constate aussi que le `CAST` rend un type que sqlx
  décode), comparée pour chacune à `invoice_exigibility_date` de la facture relue (`invoices::find_by_id` ou
  équivalent du dépôt) **et** à une valeur attendue écrite en dur.
- **Existants** : `find_unpaid_invoices_filters_by_30_day_window` (`:261`) et
  `find_unpaid_invoices_orders_by_date_then_id_before_limit` (`:314`) restent verts **sans modification
  d'assertion** — leurs factures n'ont pas d'échéance, `X = F`, l'ordre d'exigibilité s'y réduit à `(date, id)`.
  Leur doc-comment (« par `(date, id)` croissants ») est complété d'une phrase qui le dit ; le nom du second reste
  (C-29-5-10).

**AC5 — Tests `kesh-reconciliation` (rouges d'abord).**

`crates/kesh-reconciliation/src/matching.rs`, `mod tests` : `make_invoice` reçoit (ou un constructeur voisin) une
date et une échéance ; transaction `make_tx` (`booking_date` 2026-05-15 — **adapter** les dates du tableau, ou poser
`D` = 2026-06-15 par un `tx.booking_date` explicite). Les factures de chaque cas ont **même montant, même contact,
aucune référence** (score égal : 0,60), et sont passées à `propose_matches` dans l'ordre des **`invoice_id`
croissants avec la facture que la règle doit écarter en premier** — sans quoi l'ancien départage passerait par
coïncidence :

1. `ties_put_the_due_invoice_before_a_not_yet_due_older_one` — le cas de l'issue (ligne 1 du tableau) ;
2. `ties_put_the_first_fallen_due_before_an_older_invoice` (ligne 2 : la plus récente, échue la première) ;
3. `ties_treat_a_missing_due_date_as_due_on_the_invoice_date` (ligne 3) ;
4. `ties_on_the_same_due_date_keep_the_oldest_invoice` (ligne 4) ;
5. `ties_ignore_a_due_date_before_the_invoice_date` (ligne 5) ;
6. `ties_on_same_dates_fall_back_to_invoice_id` (même `F`, même `E`, ids 102 puis 101 en entrée → 101 en tête) ;
7. `score_still_beats_exigibility` : une facture **désignée** (référence = numéro, score 1,0) **non échue** passe
   devant une facture échue non désignée (0,60) ; et une facture au **montant exact** non échue (0,60) devant une
   facture échue à 3 centimes près (0,10, contact seul) ;
8. `propose_matches_returns_sorted_desc` (`:377`) et les autres tests existants : verts sans modification.

**AC6 — Test HTTP : le cas de l'issue bout à bout (`crates/kesh-api/tests/reconciliation_e2e.rs`).**

`get_proposals_puts_the_due_invoice_first_at_equal_score` (`#[sqlx::test(migrations = "../kesh-db/test-schema")]`) :
deux factures du **même** client, même reste dû, aucune référence dans la transaction, créées dans cet ordre —
A (`F = D − 20`, `E = D + 40`, non échue), puis B (`F = D − 5`, `E = D`, échue le jour du paiement) ; échéance posée
par `UPDATE invoices SET due_date` (`seed_validated_invoice`, `:384`, n'en a pas). Dates choisies pour être dans la
fenêtre **avec ou sans** la 29-2 (`F ≥ D − 30`). `GET /api/v1/reconciliation/proposals` : `candidates[0]` est B
(`candidateType == "invoice"`, `invoiceId`), `candidates[1]` est A. Puis `POST /accept` de `candidates[0]` : B est
réglée, A reste due. **Rouge au sol** (A a le plus petit `invoice_id`). `D` en plein exercice 2026
(`insert_fake_fiscal_year`).

**AC7 — Le manuel, l'API externe et le CHANGELOG disent la règle d'imputation (PDF compris).**

`docs/manual/fr/user-manual.tex` (seul manuel rédigé ; `docs/manual/{de,en,it}/` ne portent qu'un `README.md`),
§ *Algorithme de matching* :

- **après** « Une candidate dont le score vaut zéro n'est pas proposée. » (`:2041`) — ou en paragraphe propre juste
  avant, au choix rédactionnel —, la règle dite pour l'utilisateur : « Quand plusieurs factures obtiennent le même
  score --- typiquement deux factures du même client et du même montant, sans référence dans le paiement ---, Kesh
  propose celle sur laquelle la loi impute le paiement (art.~87~CO)~: d'abord une facture **échue**, et parmi elles
  celle **échue la première**~; une facture sans échéance est due dès sa date. À échéance égale, la plus ancienne. »
  Puis les limites, brèves : un paiement qui porte le **numéro** d'une facture la désigne et prime (score) ; Kesh ne
  connaît pas les poursuites et ne répartit pas un paiement entre deux factures échues le même jour --- pour imputer
  autrement, enregistrez le règlement depuis la fiche de la facture voulue (`\S\ref{sec:reglement-client}`), puis
  rejetez la transaction (*Rejeter*) ; entre deux clients que rien ne distingue, cet ordre n'est qu'un choix par
  défaut~: vérifiez la proposition.
- `:2048` (« la facture la mieux notée s'il y en a une ») : ajouter « --- à score égal, celle qu'impute la règle
  ci-dessus » (ou un renvoi équivalent).
- **Inventaire clos** des phrases du manuel qui décrivent un ordre entre factures candidates : `grep -nE "mieux
  notée|score égal|la plus ancienne|plus petit|identifiant croissant" docs/manual/fr/user-manual.tex` au rebase ; au
  sol (`fcd2ae75`) il ne rend, hors sujet, que `:2048` et `:2116` (ordre des **règles**, non touché). À relire après
  la 29-3, qui ajoute des phrases au même §.

PDF régénéré (`make fr` dans `docs/manual/`, sous le verrou de gate) et commité ; contrôle **aplati et ligatures
normalisées** :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g'`,
puis `grep -c "échue la première"` → ≥ 1, `grep -c "art. 87"` (ou la graphie retenue, espace insécable rendue par
`pdftotext`) → ≥ 1. Gardes textuelles : `cargo nextest run -E 'binary(textes_coherents)'` vert.

`docs/api-external.md` § *Accepter des propositions de rapprochement* (`:547`) : la 29-3 y écrit l'ordre des
`candidates` (« par score décroissant puis identifiant croissant », fiche 29-3 AC8). Cette phrase devient : par score
décroissant, puis, à score égal, par **date d'exigibilité** croissante (`max(date, dueDate)`, l'échéance absente
valant la date), puis date de facture, puis identifiant *(depuis la v0.13.4 ; avant : identifiant seul)*. Si la 29-3
n'a rien écrit à cet endroit au rebase, ajouter la phrase après « Les propositions viennent de `GET
/api/v1/reconciliation/proposals` ». ⚠️ **Contrat** : la forme JSON ne change pas ; seul l'ordre des candidates à score
égal change — une intégration qui prenait `candidates[0]` peut recevoir une autre facture qu'avant.

`CHANGELOG.md`, section `## [0.13.4] — Non publié` (créée par la 29-2 ou la 29-3 ; sinon la créer au-dessus de
`## [0.13.3] — 2026-10-14`, forme exacte de `scripts/prepare-release.sh`), `### Corrigé` : « **À score égal, la
réconciliation propose la facture échue la première ([#668](https://github.com/guycorbaz/kesh/issues/668)).** Entre
deux factures que rien ne distingue dans le paiement — même client, même montant, aucune référence —, Kesh proposait
la plus anciennement créée, même pas encore échue, devant une facture échue. Il propose désormais celle sur laquelle
la loi impute le paiement (art. 87 CO) : une facture échue avant une facture qui ne l'est pas, et, parmi les échues,
celle échue la première ; une facture sans échéance est due dès sa date. Un paiement qui porte le numéro d'une
facture la désigne toujours. API : la forme de `GET /api/v1/reconciliation/proposals` ne change pas, l'ordre des
candidates à score égal, si. »

**Aucun texte de l'écran ne change** : ni clé i18n, ni capture (C-29-5-7). La phrase d'aide que la 29-3 ajoute
au-dessus de la liste (« la facture la mieux notée s'il y en a une ») reste vraie.

**AC8 — Mutations (chacune tuée par un test nommé au Dev Agent Record).**

| # | Mutation | Tuée par |
|---|---|---|
| M1 | `propose_matches` : rétablir `a.invoice_id.cmp(&b.invoice_id)` comme départage | AC5 cas 1, 2, 5 ; AC6 |
| M2 | `invoice_exigibility_date` rend `invoice.date` (échéance ignorée) | AC5 cas 1, 2 ; AC4 unitaires ; AC6 |
| M3 | comparer `invoice.due_date` (`Option`) au lieu de `X` (`None` passe en tête) | AC5 cas 3 ; AC4 unitaires |
| M4 | `due_date.unwrap_or(date)` sans `max` avec `date` | AC5 cas 5 ; AC4 unitaire `E < F` ; parité AC4 |
| M5 | `X` décroissante | AC5 cas 1, 2 ; AC4 |
| M6 | `imputation_order` sans la composante `F` (`(X, id)`) | AC5 cas 4 (entrée : la plus récente avec le plus petit id) |
| M7 | ordre d'imputation **avant** le score | AC5 cas 7 |
| M8 | `ORDER BY` rétabli à `i.date ASC, i.id ASC` | AC4 `…_by_exigibility_before_limit` |
| M9 | SQL : `GREATEST(i.date, i.due_date)` sans `COALESCE` | AC4 parité (`NULL` au décodage) et `…_before_limit` si une facture sans échéance y figure — le dev en place une |
| M10 | SQL : `COALESCE(i.due_date, i.date)` sans `GREATEST` | AC4 parité (cas `E < F`) |
| M11 | composition de l'ancre 29-2 (AC1, dernier point) : retirer le terme des rappels | tests de la 29-2 (cas d, f de son AC5) — **à constater**, pas à écrire |

Restauration de chaque mutation : `git checkout -- <fichier> && touch <fichier>` (`docs/agents-orchestration.md`
§ 7). Une mutation équivalente se déclare avec sa raison et la sortie du run.

**AC9 — Gates.**

`kesh-db` touché : **gate complet même en boucle de revue** (`CLAUDE.md` § *Pendant une boucle de revue*), base remise
à zéro avant chaque gate complet (KF-039), `scripts/test-fast.sh` sous `flock` ; frontend (`npm run check`,
`lint-i18n-ownership`, `test:unit`, `build`) — aucun fichier frontend n'est censé changer, le gate le confirme ; E2E
complet au **dernier commit de code** (base `kesh_e2e_295`, port **3046**, recette de `docs/testing.md` à la lettre),
rouges jugés fichier par fichier contre `docs/testing.md` § *Les échecs attendus*. Base de travail `kesh_295`,
`CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-29-5/target`. Le Dev Agent Record ne déclare que ce qui a tourné, chiffres
recomptés.

## Tasks / Subtasks

- [ ] **T0 — Rebase** sur `origin/main` (29-2 et 29-3 mergées) ; relire les lignes citées ; regreper au § *Zones
      partagées* (forme mergée de l'ancre 29-2, phrase d'ordre de la 29-3 dans `api-external.md`, section
      `[0.13.4]` du CHANGELOG, phrases du manuel).
- [ ] **T1 — Tests rouges** (AC4, AC5, AC6) : constater le rouge (compilation ou assertion) avant T2.
- [ ] **T2 — `kesh-db`** (AC1, AC3) : fonction, ordre, constante SQL, `ORDER BY`, composition de l'ancre, doc-comments.
- [ ] **T3 — `kesh-reconciliation`** (AC2).
- [ ] **T4 — Manuel FR + PDF, `api-external.md`, CHANGELOG** (AC7).
- [ ] **T5 — Propagation** : le grep de § *Propagation*, trié à la main, sortie résumée au Dev Agent Record.
- [ ] **T6 — Mutations** (AC8) et **gates** (AC9) ; Dev Agent Record.

## Dev Notes

### Inventaire (a) — les sites qui ordonnent des factures candidates pour un paiement (`fcd2ae75`)

```
$ grep -rnE "invoice_id\.cmp|ORDER BY i\.date|propose_matches\(" crates/*/src --include=*.rs
crates/kesh-db/src/repositories/reconciliation.rs:258:         ORDER BY i.date ASC, i.id ASC \
crates/kesh-api/src/routes/reconciliation.rs:747:            let match_proposals = propose_matches(&tx, &candidates_with_contacts);
crates/kesh-api/src/routes/reconciliation.rs:1790:    let proposals_score = propose_matches(&bank_transaction, &candidates_for_score);
(+ la définition, kesh-reconciliation/src/matching.rs:62, et son tri :99-105)
```

**Ensemble clos : deux sites d'ordre** — le tri de `propose_matches` (AC2) et l'`ORDER BY` de la proposition
(AC3). `:1790` (acceptation) passe **une** facture : ordre sans objet. La 29-3 ajoute `order_candidates`
(`routes/reconciliation.rs`, fiche 29-3 AC1) : tri **stable** par rang (facture à signal / règle / facture sans
signal), qui **conserve** l'ordre de `propose_matches` à l'intérieur d'un rang — l'ordre d'imputation le traverse donc
intact ; non touché ici. Le frontend n'ordonne rien (il affiche `candidates[0]`, fiche 29-3 § *Sites qui lisent
l'ordre*).

### Inventaire (b) — les sites non résolus (choix d'une facture pour un paiement, hors de cette règle)

| Site | Ce qu'il fait | Sort |
|---|---|---|
| Règlement depuis la fiche (`settle_invoice`, `invoice_settlements_write.rs`) | l'utilisateur **désigne** la facture | hors classe : imputation par déclaration (art. 86) — c'est le recours écrit au manuel |
| Lettrage manuel (`repositories/letterings.rs`) | l'utilisateur choisit les lignes | hors classe |
| Factures fournisseur et lots pain.001 | paiements **sortants** | hors classe : Kesh y est le débiteur |
| Troncature `LIMIT 50` sur le **score** (une facture désignée écartée au-delà de 50 candidates) | C-28-7-10 | **non résolu, assumé** (inchangé ; AC3 l'écrit au doc-comment) |
| Poursuites (al. 1), garanties (al. 3), imputation proportionnelle (al. 2) | aucune donnée en Kesh / une seule proposition | **angles morts écrits** (C-29-5-3, C-29-5-5) |
| Clients différents à score égal | art. 87 sans objet | **ordre par défaut écrit** (C-29-5-7) |

### Inventaire (c) — les tests existants dont le sens pourrait changer

```
$ grep -lE "propose_matches|find_unpaid_invoices_for_window|reconciliation/proposals" crates/*/tests/*.rs crates/*/src/*.rs
```

Au sol : `kesh-reconciliation/src/matching.rs` (tests à une facture ou à scores distincts — `propose_matches_
returns_sorted_desc` : scores distincts, inchangé) ; `kesh-db/tests/reconciliation_repository.rs` (factures **sans**
échéance : `X = F`, ordre inchangé ; les tests `:513`, `:566`, `:842`, `:862`, `:945` asserent une présence ou un
montant, pas un ordre) ; `kesh-db/tests/invoice_write_off.rs:785-810` (`any`, pas d'ordre) ;
`kesh-api/tests/reconciliation_e2e.rs`, `reconciliation_rules_e2e.rs`, `bank_accounts_closed_e2e.rs` (aucun ne dépend
de l'ordre de deux factures à score égal — fiche 29-3, inventaire clos des lecteurs de l'ordre ; **à revérifier** au
gate : un rouge y signerait le contraire). Les tests de la 29-2 qui posent des échéances (cas a-k, parité) asserent
des **ensembles**, non des ordres — à constater au rebase (`grep -n "assert_eq!(ids\|vec!\[" …`).

E2E : la base `kesh_e2e` partagée garde des factures **avec échéance** (contacts « 30 jours net ») ; les specs de
réconciliation tirent des montants uniques (`reconciliation-amount-due.spec.ts`, `Date.now()`) : une facture d'une
autre spec ne devient pas une candidate à score égal. Le Dev Agent Record déclare tout rouge d'une spec de
réconciliation, fichier par fichier.

### Propagation — le symptôme à greper après chaque patch

La **valeur** du départage ancien et ses formulations :

```sh
grep -rnE "invoice_id ASC|invoice_id croissant|identifiant croissant|plus petit .?invoice_id|départag|then_with\(\|\| a\.invoice_id|i\.date ASC, i\.id ASC|\(date, id\)|C-29-2-11|C-29-3-12|#668" \
  crates frontend/src docs/manual/fr/*.tex docs/api-external.md CHANGELOG.md
```

Occurrences attendues à trier : doc-comments de la 29-3 (`order_candidates`, cas 8 de son AC2 : « c'est
`propose_matches` qui a fixé `invoice_id ASC` » → à réécrire), angle mort C-29-3-12 cité au doc-comment de
`order_candidates` (« départage à score égal, cf. fiche 29-3 » ou #668 → **résolu par la 29-5**), doc-comments de la
29-2 sur C-29-2-11 ; les entrées publiées du CHANGELOG **ne se réécrivent pas**.

### Ce qui doit être préservé

- `propose_matches` : signature, `MatchProposal` / `MatchScore` (forme JSON), filtre `total > 0.0`, score en première
  clé.
- `find_unpaid_invoices_for_window` : signature, filtres, `HAVING`, `LIMIT 50`, et la fenêtre de la 29-2.
- `accept_one_invoice` : aucune ligne modifiée.
- `order_candidates` de la 29-3 : non touché ; sa stabilité est ce qui laisse passer l'ordre d'imputation.
- Aucune migration, aucun index (filesort sur un ensemble que `paid_at IS NULL` et le reste dû réduisent à quelques
  factures ouvertes de même montant).

### Zones partagées avec la 29-2 (#529) et la 29-3 (#527), non mergées

| fichier | 29-2 | 29-3 | 29-5 |
|---|---|---|---|
| `kesh-db/repositories/reconciliation.rs` | `InvoiceDateWindow` (champ renommé `earliest_anchor_date`, `check(F, A)`), jointure `rmd`, `INVOICE_PAYMENT_ANCHOR_SQL`, `WHERE` de `find_unpaid_invoices_for_window`, `invoice_payment_anchor_date` | aucune | `invoice_exigibility_date`, `imputation_order`, `INVOICE_EXIGIBILITY_DATE_SQL`, **`ORDER BY`** (la 29-2 le laisse inchangé, C-29-2-7), ancre récrite par composition (AC1) |
| `kesh-reconciliation/src/matching.rs` | aucune (C-29-2-7) | aucune (C-29-3-12) | le tri `:97-105`, doc-comments, tests |
| `kesh-api/routes/reconciliation.rs` | étape 6 d'`accept_one_invoice`, commentaire de la passe 2 | `candidate_rank`, `order_candidates`, passe 4 | **aucune ligne de code** ; doc-comment d'`order_candidates` si la propagation l'atteint |
| `kesh-db/tests/reconciliation_repository.rs` | aide de rappel, `due_date`, cas a-k | aucune | deux tests neufs, deux doc-comments |
| `kesh-api/tests/reconciliation_e2e.rs` | parité étendue, cas #529 | trois tests, `create_rule_ok` | un test neuf |
| `user-manual.tex` § *Algorithme de matching* | `:2029`, `:2031`, `:2902`, `:1305` | `:2041`, `:2049` | après `:2041`, et `:2048` — **regreper les valeurs**, pas les numéros |
| `docs/api-external.md` `:547` | phrase des raisons `payment_date_*` | phrase de l'ordre des `candidates` | **réécrit** la phrase de la 29-3 |
| `CHANGELOG.md` `[0.13.4]` | `Corrigé` (#529) | `Changed` (#527) | `Corrigé` (#668) — une seule section |

### Modules touchés (règle de découpage : au plus cinq)

1. `kesh-db` — `repositories/reconciliation.rs` (+ `tests/reconciliation_repository.rs`) ;
2. `kesh-reconciliation` — `matching.rs` ;
3. `kesh-api` — `tests/reconciliation_e2e.rs` seulement ;
4. documentation — manuel FR (+ PDF), `docs/api-external.md`, `CHANGELOG.md`.

Quatre, une règle métier unique : pas de découpage.

### Pièges connus du dépôt, applicables ici

- **`GREATEST` et `NULL`** en MariaDB ; **`Option` et l'ordre** en Rust (`None` en tête) — M3, M9.
- **Un test d'ordre qui passe par coïncidence de montage** : insérer les factures de sorte que `invoice_id ASC` et
  `(date, id)` donnent un **autre** ordre que l'ordre d'imputation, et l'asserter au montage (`assert_ne!`).
- **L'attendu ne se calcule pas avec la fonction testée** (AC4 : deux côtés sortis du même extracteur = vert par
  construction, Epic 23).
- **Gate complet en boucle** (`kesh-db`), base remise à zéro avant chaque gate complet (KF-039).
- **Mutation restaurée, binaire périmé** : `git checkout -- f && touch f`.
- **`pdftotext` coupe les lignes et ligature** : aplatir et normaliser avant de greper.
- **Aucun « closes #668 » dans un message de commit**, même en prose : `refs #668` ; le mot-clé va dans la PR.
- **Données réelles** : fixtures synthétiques seulement.

### References

- Issue #668 et son commentaire du 2026-10-14 ; #529 (29-2, fiche `kesh-29-2/…/29-2-paiement-tardif.md`, C-29-2-7,
  C-29-2-11) ; #527 (29-3, fiche `kesh-29-3/…/29-3-meilleure-proposition.md`, AC1, AC8, C-29-3-12) ; #548 (28-7,
  C-28-7-10).
- `crates/kesh-reconciliation/src/matching.rs:1-107`, `:370-440` ;
  `crates/kesh-db/src/repositories/reconciliation.rs:1-40`, `:200-276` ;
  `crates/kesh-api/src/routes/reconciliation.rs:650-760`, `:1765-1800`.
- `docs/manual/fr/user-manual.tex:2027-2052` ; `docs/api-external.md:547-567`.
- Art. 75, 86, 87, 102 CO (RS 220) ; `CLAUDE.md` § *Un appariement automatique propose, il ne crée jamais*,
  § *Inventorier les sites NON RÉSOLUS*, § *Test Locally First*.

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

- 2026-10-14 — Fiche créée (`bmad-create-story`, Opus 5.5, autonomie) sur `fcd2ae75`. Prémisses de #668, de sa
  rectification et de la consigne établies au code : départage `invoice_id ASC` et primauté de la désignation
  **confirmés** ; « premières poursuites = rappels » **corrigé** (Kesh n'enregistre pas de poursuite ; un rappel n'en
  est pas une) ; attendu d'origine (« la récente », proximité de l'ancre) **écarté** par l'art. 87. Constat : trier
  par la date d'exigibilité suffit à placer les échues d'abord, sans la date du paiement. Choix C-29-5-1 à 10. 9 AC,
  7 tâches (T0-T6), 11 mutations (M1-M11), 5 lignes d'exemple, 7 cas à l'AC5, 4 modules.
