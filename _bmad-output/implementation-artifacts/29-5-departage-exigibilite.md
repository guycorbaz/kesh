# Story 29.5 : À score égal, la facture échue la première passe en tête (art. 87 CO)

## Status

**ready-for-dev** — fiche créée le 2026-10-14 en autonomie (`bmad-create-story`, Opus 5.5), sur `fcd2ae75` (`main`
après le kickoff de l'Epic 29), branche `story/29-5`, worktree `/home/gcorbaz/devel/kesh-29-5`. Story **ajoutée à
l'Epic 29** par l'orchestrateur le 2026-10-14. Validation P1 faite (Sonnet, lentilles R et F : 0/0/4/4 et
0/0/2/5) et remédiée le 2026-10-14 ; validation P2 faite (Opus, lentilles R et F : 0/0/1/10 et 0/0/1/7) et
remédiée le même jour ; **passe ciblée P3 à lancer** (`29-5-validate-prompt-p3-ciblee.md`, Haiku, une lentille sur
le diff de la remédiation P2). Choix consignés au registre propre à la story, `epic-29-choix-autonomes-29-5.md`,
**C-29-5-1 à C-29-5-12** (à verser par union au registre de l'epic). 9 AC. PR : **`closes #668`** — dans le titre ou
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
plus ancienne » et l'imputation proportionnelle à échéances identiques. **Validation P1** : les deux lentilles ont
confronté les quatre points au texte de l'art. 87 que leur a remis l'orchestrateur — **conformes** ; ni l'une ni
l'autre n'a pu lire Fedlex, et les art. 75 et 86 **n'ont pas été confrontés** à la source. La règle retenue
ci-dessous ne dépend que des alinéas 1 et 2.

⚠️ **« Exigible » et « échue »** : l'al. 1 parle de dette **exigible**, les al. 2 et 3 de dettes **échues**. Pour une
facture à échéance, les deux coïncident à `E`. Pour une facture **sans** échéance, la fiche tient sa date `F` pour la
date où elle est exigible **et** échue : c'est une **convention** (C-29-5-12), écrite comme telle au doc-comment, non
une lecture du texte. Conséquence : une facture sans échéance datée au plus tard du jour du paiement compte
toujours pour **échue**, et l'al. 3 (« aucune n'est échue ») ne la vise jamais.

### Les prémisses, établies au code (`fcd2ae75`)

| Prémisse (issue #668 et sa rectification ; consigne de l'orchestrateur) | Au code | Verdict |
|---|---|---|
| `propose_matches` départage deux factures de même score par `invoice_id` croissant | `crates/kesh-reconciliation/src/matching.rs:97-105` : `b.score.total.partial_cmp(&a.score.total)…then_with(\|\| a.invoice_id.cmp(&b.invoice_id))` | **Vraie** |
| Une facture désignée par le paiement (référence, numéro) prime déjà par le score | `reference_score` (`matching.rs:121-147`) : 1 si la référence et le numéro se contiennent, 0,5 sur un préfixe commun ≥ 4 ; poids 0,40, **contre 0,50 au montant** (`:80` : `0.50 * amount_score + 0.40 * reference_score + 0.10 * contact_score`), montant **binaire** au centime (`amount_score`, `:109-119`). Une facture désignée a un score **strictement supérieur** à une facture du même client non désignée **si leurs scores de montant sont égaux** (toutes deux au montant exact, ou aucune). Une facture désignée dont le reste dû diffère de quelques centimes (dans la tolérance de 0,05 de la requête) vaut 0,40 ou 0,50 ; une facture non désignée du même client au montant exact vaut 0,60 et **passe devant** ; d'un autre client (nom sans rapport avec la contrepartie, aucune demi-référence), elle vaut 0,50 et passe devant la facture désignée à 0,40, mais **égale** celle à 0,50, l'ordre d'imputation tranchant alors (validation P2, R1 et F1) | **Vraie sous condition de montant** — non touchée (C-29-5-6) ; la condition est écrite au manuel et au CHANGELOG (AC7, validation P1, R2) |
| Le départage se fait aussi par l'`ORDER BY` / `LIMIT 50` de `find_unpaid_invoices_for_window` | `crates/kesh-db/src/repositories/reconciliation.rs:258-259` : `ORDER BY i.date ASC, i.id ASC LIMIT 50`. Il ne départage pas le score — `propose_matches` retrie tout — mais il **choisit quelles** candidates sont scorées : au-delà de 50, les plus récentes **par date** sont écartées, fût-ce celle que l'art. 87 désigne | **Vraie, au sens de la troncature** — et **aggravée par la 29-2** : sa fenêtre ancrée fait entrer, à 30 jours net, les factures jusqu'à `F ≥ D − 60`, et des factures anciennes **non échues** (échéance lointaine) qui, plus anciennes par date, passent devant une facture échue dans l'`ORDER BY` et peuvent la pousser hors des 50. La 29-2 laisse ordre et troncature inchangés et renvoie ce point ici (C-29-2-7, C-29-2-13, § *Zones partagées* de sa fiche, état `e17368f8`). Traité par AC3 : l'ordre SQL devient celui de l'art. 87 (C-29-5-4) |
| Les données de l'exigibilité sont disponibles au départage | `Invoice` porte `date` (`NaiveDate`) et `due_date` (`Option<NaiveDate>`, nullable — `migrations/20260416000001_invoices.sql:22`) ; le triplet candidat passé à `propose_matches` (`routes/reconciliation.rs:732-746`) contient l'`Invoice` complète | **Vraie** : aucune donnée à ajouter au triplet |
| « premières poursuites (en Kesh : rappels ?) » | Kesh **n'enregistre aucune poursuite** : `grep -rniE poursuite crates frontend/src -l` rend quatre fichiers, tous du **texte** des rappels — le libellé du dernier niveau, « Dernier rappel avant poursuite » (`crates/kesh-db/src/entities/email_template_defaults.rs:96`, commentaire `:68`), son test (`crates/kesh-api/tests/email_templates_e2e.rs:601`) et deux commentaires du badge du dernier rappel (« poursuite à envisager », `frontend/src/lib/features/reminders/reminders.types.ts:21`, `ReminderTerminalBadge.svelte:5`) ; aucune table, aucune colonne. Un **rappel** est une interpellation (mise en demeure, art. 102 CO), **pas** une poursuite (réquisition de poursuite, LP) | **Corrigée** : un rappel ne vaut pas poursuite ; le critère n'a pas de donnée en Kesh (C-29-5-3) |
| Attendu d'origine de l'issue : « la récente passe en tête », départage par la proximité de la date d'ancrage | Contredit par l'art. 87 (rectification de Guy) ; la date d'ancrage de la 29-2 (`max(date, échéance, dernier rappel)`) mêle les rappels, qui ne changent ni l'exigibilité ni l'ordre légal | **Écartée** (C-29-5-2) |

### La règle retenue (C-29-5-2)

Pour une facture de date `F` et d'échéance `E` (nullable), on pose sa **date d'exigibilité**

> `X = max(F, E)` — `E` absente valant `F`

(une facture sans échéance est exigible dès sa date — c'est, en substance, l'art. 75 CO pour une obligation sans
terme ; texte non relu à la source, d'où l'absence de guillemets — et tenue pour échue à cette date, convention
C-29-5-12 ; le `max` couvre une échéance antérieure à la date, données d'avant #245 : `X = F`). **À score égal**, les
candidates se rangent par

> `X` croissante, puis `F` croissante, puis `invoice_id` croissant.

**Pourquoi cela suffit à « d'abord les échues »** : pour un paiement de date `D`, une facture est échue si `X ≤ D` et
non échue si `X > D` ; toute facture échue a donc une `X` **plus petite** que toute facture non échue. Trier par `X`
place les échues en tête **sans lire `D`**, puis, parmi elles, celle **échue la première** (al. 1). Parmi les non
échues — l'al. 3 renvoie aux garanties, que Kesh ignore —, la plus proche de son échéance passe d'abord : choix par
défaut écrit (C-29-5-5). Une échéance lointaine, ou mal saisie (2099), passe **derrière** à score égal : l'angle
mort C-29-2-11 de la 29-2 est fermé **à score égal** seulement ; la candidature sans limite de temps d'une facture
non échue ou à échéance lointaine, elle, **reste** (C-29-2-11 : seule candidate, ou mieux notée, elle est toujours
proposée) (validation P2, R7).

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
| 06-01, — | 05-20, 05-25 | 06-01 / 05-25 | **B** (l'absence d'échéance donne ici la **plus grande** `X` : une comparaison d'`Option` — `None` en tête — mettrait A devant, M3) | A |

### Ce que la règle ne fait pas

- **Elle ne touche pas au score** : une facture désignée par la référence **et** au montant exact garde la tête,
  échue ou non, devant toute facture que la référence ne désigne pas aussi (art. 86 : la déclaration du débiteur
  prime l'imputation légale ; `reference_score` est un contenant bidirectionnel, `matching.rs:132`, si bien qu'une
  référence peut désigner deux factures — validation P2, R3). Le montant pesant plus que la référence (0,50 contre
  0,40, `matching.rs:80`), une facture **du même client** au montant exact passe devant une facture à quelques
  centimes près — **même désignée par le numéro** —, quelle que soit leur exigibilité ; une facture d'un **autre**
  client au montant exact **peut** passer devant elle : leurs scores peuvent être égaux (0,40 + 0,10 contre 0,50,
  contact au seul client désigné, aucune demi-référence), et c'est alors l'ordre d'imputation qui tranche
  (C-29-5-6 ; validation P2, R1 et F1).
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

Dans `crates/kesh-db/src/repositories/reconciliation.rs` (C-29-5-8) — les **fonctions** Rust à côté de
`payment_date` et d'`InvoiceDateWindow` ; la **macro** `invoice_exigibility_date_sql!` définie **avant**
`invoice_payment_anchor_sql!` de la 29-2 (une `macro_rules!` n'est visible qu'après sa définition dans le texte : dans
la forme commitée de la 29-2, `8f483388`, la macro d'ancre est définie à `:163` et développée à `:180` et `:186`,
avant `payment_date` `:208` — définie près de `payment_date`, la macro d'exigibilité y serait introuvable ; validation
P2, R4 et F2) :

- `pub fn invoice_exigibility_date(invoice: &Invoice) -> NaiveDate` : `max(invoice.date,
  invoice.due_date.unwrap_or(invoice.date))`. ⛔ Jamais une comparaison d'`Option<NaiveDate>` : `None < Some(_)` en
  Rust placerait une facture **sans** échéance avant toutes les autres (mutation M3).
- `pub fn imputation_order(a: &Invoice, b: &Invoice) -> std::cmp::Ordering` : `(X, F, id)` croissants, dans cet ordre.
- L'expression SQL est écrite **une fois**, par un **`macro_rules!`** qui rend un **littéral** —
  `macro_rules! invoice_exigibility_date_sql { () => { "CAST(GREATEST(i.date, COALESCE(i.due_date, i.date)) AS DATE)" }; }`
  —, et exposée en `const` pour les requêtes composées par `format!` :
  `pub const INVOICE_EXIGIBILITY_DATE_SQL: &str = invoice_exigibility_date_sql!();`. Macro et non `const` seule :
  `concat!` n'accepte que des littéraux, et l'ancre de la 29-2 doit s'y composer (point suivant ; C-29-5-11).
  Précédents exacts du dépôt : `columns!` (`repositories/accounts.rs:20-34`, « un `macro_rules!` plutôt qu'un `const`
  pour rester utilisable dans `concat!` ») et `invoice_columns!` / `INVOICE_COLUMNS` (`repositories/invoices.rs:50-65`).
  Prérequis alias `i`, comme les autres expressions du module. ⛔ `COALESCE` obligatoire : en MariaDB, `GREATEST`
  rend `NULL` dès qu'un argument l'est, et un `NULL` trié en `ASC` passe **en tête** (mutation M9).
- Doc-comments `///` : l'art. 87 (en substance, avec la réserve de la § *Le critère légal*), l'art. 75 pour la
  facture sans échéance **et la convention « exigible = échue » qu'elle impose** (C-29-5-12), l'équivalence « échues
  d'abord ⇔ `X` croissante » et sa condition, les deux écritures et le test qui les tient d'accord (AC4), ce que la
  règle ne fait pas (score — dont la condition de montant de la désignation : une facture au montant exact du même
  client passe devant une facture désignée à quelques centimes près, celle d'un autre client **peut** y passer (score
  supérieur ou égal ; à égalité, l'ordre d'imputation tranche — validation P2, R1) —, clients différents, al. 2, al. 3, poursuites).
- **Si la 29-2 est mergée** (attendu) : son ancre est écrite par un **`macro_rules!` d'ancre** sur les mêmes
  précédents — forme **commitée** dans la 29-2 (`8f483388`, non mergée à la validation P2), en deux termes :
  `concat!("CAST(GREATEST(", "GREATEST(i.date, COALESCE(i.due_date, i.date)), ", "COALESCE(DATE(rmd.last_sent_at), i.date)", ") AS DATE)")`,
  le **premier terme** étant réservé par son doc-comment à « une expression d'exigibilité commune » (C-29-2-15). La
  29-5 **insère** la macro d'exigibilité dans cette macro d'ancre, **à la place du premier terme (C-29-2-15)** —
  `concat!("CAST(GREATEST(", invoice_exigibility_date_sql!(), ", ", "COALESCE(DATE(rmd.last_sent_at), i.date)", ") AS DATE)")`,
  forme qu'admet `concat!` (un appel de macro qui rend un littéral, ou un `concat!` de littéraux, s'y développe :
  précédents `line_ttc_sql!`, `repositories/invoices.rs:186-197`, appelé dans le `concat!` de
  `INVOICE_TTC_SUBQUERY_SQL`, `:214-216`, et la 29-2 elle-même, qui appelle `invoice_payment_anchor_sql!()` dans le
  `concat!` de sa requête scalaire, `:184-188`) —, et la constante d'ancre de la 29-2 reste définie depuis sa macro.
  `GREATEST(i.date, COALESCE(i.due_date, i.date))` ne s'écrit donc qu'une fois (règle DRY du `CLAUDE.md`).
  Équivalence : substitution du seul premier terme par le même terme enveloppé d'un `CAST(… AS DATE)`, l'identité sur
  une `DATE` (`i.date` est `NOT NULL`, le terme n'est jamais `NULL`) ; **aucun test de la 29-2 ne change** — ni
  assertion, ni nom, ni montage —, et le Dev Agent Record le constate au gate (C-29-5-9). ⚠️ **Si la forme mergée diffère** (constante `&str` sans macro, autre nom, autre ordre
  des arguments), la relecture se fait **à T0** : le dev compose sur ce qui est mergé, sans changer la nature de
  l'ancre d'une façon qui toucherait les tests de la 29-2, et l'écrit au Dev Agent Record ; à défaut de composition
  possible, deux écritures tenues d'accord par le test de parité de l'AC4 étendu à l'ancre, déclarées comme écart à
  ce point.

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
ceux que la 29-2 aura posés compris. Doc-comment de la fonction (point 3, `:208-213`, « Résultat trié par `date ASC,
id ASC` (C-28-7-10) ») et en-tête du module (`:10-15`, « par `(date, id)` croissants ») réécrits : l'ordre est celui
de l'imputation, **pour que la troncature n'écarte jamais, parmi des candidates de même score, celle que
`propose_matches` placerait en tête** — propriété qui tient parce que les deux ordres coïncident (les candidates
gardées forment un préfixe de l'ordre d'imputation dans chaque classe de score). La troncature reste aveugle au
**score** : une facture désignée par sa référence peut encore être écartée au-delà de 50 candidates (C-28-7-10,
inchangé en nature, écrit au doc-comment). ⚠️ **Ce qui change, c'est la victime, et le doc-comment le dit** (validation
P1, F3) : avant, au-delà de 50, étaient écartées les factures les plus **récentes par date** ; désormais, celles dont
l'**exigibilité est la plus lointaine** — par exemple une facture désignée par sa référence, à 60 jours net, derrière
cinquante factures de même reste dû plus tôt exigibles. Même fragilité, autre population : ce n'est pas un gain net
sur l'angle mort C-28-7-10, seulement l'alignement de la troncature sur le départage.

**AC4 — Tests `kesh-db` (rouges d'abord).**

`crates/kesh-db/src/repositories/reconciliation.rs`, `mod tests` (unitaires, pas de base) :
`invoice_exigibility_date` sur `E` nulle (`X = F`), `E > F` (`X = E`), `E = F`, `E < F` (`X = F`) ;
`imputation_order` sur les **six** lignes du tableau d'exemples (entrées et ordre attendu **écrits en dur** ; la
sixième — facture sans échéance de plus grande `X` — est celle qui tue M3 au niveau de la fonction), plus `X`, `F`
égales et `id` différents (tue M13).

`crates/kesh-db/tests/reconciliation_repository.rs` — un `due_date` à poser sur les factures de test
(`UPDATE invoices SET due_date = ? WHERE id = ?` après `insert_test_invoice`, ou paramètre ajouté sans changer les
appels existants — au choix du dev ; **si la 29-2 a déjà ajouté une aide, la réutiliser**) :

- `find_unpaid_invoices_orders_by_exigibility_before_limit` : transaction à `D` = 2026-06-15 ; **51** factures de
  même reste dû, toutes dans la fenêtre de dates, dont l'ordre d'exigibilité **diverge** de `(date, id)` :
  - la facture **la plus récente échue** (`F = D`, `E = D`, `X = D`) porte la plus petite `X` ;
  - une facture **sans échéance**, **obligatoire** (validation P1, F2 et R8), datée `F = D + 1` (admise par la
    fenêtre, #548) : `X = D + 1`, donc **deuxième** sous l'ordre correct. Sous M9, où `GREATEST(i.date, i.due_date)`
    vaut `NULL` pour elle, deux régimes (validation P2, R5 et F3) : **sans** la 29-2, elle passe **première** (un
    `NULL` trié en `ASC` passe en tête) ; **avec** l'ancre composée (AC1), la mutation passe aussi dans l'ancre, qui
    vaut `NULL`, et `NULL >= ?` est faux dans le `WHERE` : elle est **exclue** des candidates (liste rendue : la
    facture `F = D`, puis les quarante-neuf). Rouge dans les deux cas ; le Dev Agent Record écrit le mécanisme
    **constaté**. Placée à `F ≤ D`, elle serait en tête sous l'ordre correct et sous M9 sans la 29-2, et ne
    distinguerait pas M9 dans ce régime ;
  - les quarante-neuf autres, plus anciennes (`F` de `D − 30` à `D − 1`, répétitions admises), ont une échéance
    lointaine (`E` = `D + 40` et au-delà, **distinctes**).

  Insertion dans un ordre où `id ASC` diverge aussi. Assertions : la liste rendue vaut, **écrite en dur** par une
  expression indépendante de la production (pas d'appel à `imputation_order` pour construire l'attendu — leçon de
  l'Epic 23, *Inventorier les sites non résolus*), la facture `F = D` **en tête**, la facture sans échéance
  **deuxième**, puis les 48 suivantes par `E` croissante ; la 51ᵉ (`E` la plus lointaine) est écartée. Montage :
  `assert_ne!` entre l'ordre attendu et `(date, id)` — sinon le test passerait à vide sous l'ancien `ORDER BY`
  (mutation M8).
- `find_unpaid_invoices_orders_same_exigibility_by_invoice_date_before_limit` (validation P1, F2 ; tue **M12**) :
  la coupure du `LIMIT 50` traverse un **groupe de même `X` et de `F` différentes**. Transaction à `D` = 2026-06-15 ;
  **51** factures de même reste dû : quarante-neuf de remplissage **sans échéance**, `F = D − 25` (`X = D − 25`),
  puis un groupe de deux factures de même échéance `E = D + 10` (`X = D + 10`) — G1 à `F = D − 20`, G2 à `F = D − 10`,
  **G2 insérée avant G1** (plus petit `id`). Ordre correct `(X, F, id)` : les quarante-neuf, puis G1 ; **G2 écartée**.
  Sous M12 (`ORDER BY X, i.id`, sans `i.date`) : G2 gardée, G1 écartée. Assertions **écrites en dur** : la liste
  rendue (les quarante-neuf dans leur ordre d'`id`, puis G1) et l'absence de G2. Montage : `assert_ne!` entre
  l'attendu et l'ordre `(X, id)` calculé à la main sur les mêmes entrées. Les dates sont dans la fenêtre avec ou sans
  la 29-2 (`F ≥ D − 30`, ancre `≥ F`).
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
date et une échéance (`make_invoice` pose aujourd'hui `date` 2026-05-10 et `due_date: None`, `:227-228`) ; la
transaction est celle de `make_tx`, **telle quelle** : `propose_matches` ne lit pas la date du paiement (le tri par
`X` place les échues d'abord sans elle, § *La règle retenue*), si bien que sa `booking_date` est sans effet sur le
verdict — les dates du tableau se posent telles quelles sur les factures. Les factures des cas 1 à 7 ont **même
montant, même contact, aucune référence** (score égal : 0,60), et sont passées à `propose_matches` dans l'ordre des
**`invoice_id` croissants, la facture que la règle doit écarter portant le plus petit `id` et venant en premier** —
sans quoi l'ancien départage passerait par coïncidence —, **sauf le cas 7**, qui entre ses ids dans l'ordre
**décroissant** pour tuer M13 (validation P2, R10 et F4). Le cas 8 a ses propres scores (ci-dessous) :

1. `ties_put_the_due_invoice_before_a_not_yet_due_older_one` — le cas de l'issue (ligne 1 du tableau) ;
2. `ties_put_the_first_fallen_due_before_an_older_invoice` (ligne 2 : la plus récente, échue la première) ;
3. `ties_treat_a_missing_due_date_as_due_on_the_invoice_date` (ligne 3) ;
4. `ties_on_the_same_due_date_keep_the_oldest_invoice` (ligne 4) ;
5. `ties_ignore_a_due_date_before_the_invoice_date` (ligne 5) ;
6. `ties_put_an_earlier_due_date_before_a_later_invoice_without_due_date` (ligne 6 : A sans échéance, de plus grande
   `X`, créée d'abord ; B en tête — tue M3, que les lignes 3 et 5 laissent survivre parce que la facture sans
   échéance y est aussi celle de plus petite `X`) ;
7. `ties_on_same_dates_fall_back_to_invoice_id` (même `F`, même `E`, ids 102 puis 101 en entrée → 101 en tête ; tue
   M13 : `sort_by` étant stable, une clé sans `id` garderait l'ordre d'entrée, 102 en tête) ;
8. `score_still_beats_exigibility` : une facture **désignée** (référence = numéro) **au montant exact** (score 1,0)
   **non échue** passe devant une facture échue non désignée (0,60) ; une facture au **montant exact** non échue
   (0,60) devant une facture échue à 3 centimes près (0,10, contact seul) ; et une facture **désignée** par le numéro
   mais à 3 centimes près (0,50), échue, **derrière** une facture non désignée au montant exact (0,60), non échue — la
   condition de montant de la désignation, que le manuel écrit (AC7). Toutes les factures du cas 8 sont du **même
   client**, dont le nom est la contrepartie de la transaction (contact 0,10 chacune). ⚠️ Les scores annoncés
   supposent que les factures **non désignées** n'ont **aucun préfixe commun de 4 caractères** avec la référence de la
   transaction (`reference_score`, `matching.rs:137-144` ; `normalize` ne retire pas les tirets) : leur
   `invoice_number` vaut `None` (ou un numéro sans tel préfixe, par exemple `"OTHER"`, comme
   `propose_matches_returns_sorted_desc`) — avec un numéro de même série (`"INV-2026-002"` contre `"INV-2026-001"`),
   la demi-référence donnerait 0,20 de plus (0,80 au lieu de 0,60), l'ordre asserté restant juste mais une assertion
   de `score.total` ne l'étant plus (validation P2, R2 et F4). Le deuxième sous-cas se joue sur une transaction
   **sans** référence ;
9. `propose_matches_returns_sorted_desc` (`:377`) et les autres tests existants : verts **sans modification
   d'assertion**. Ce test n'a **pas** des scores distincts (validation P1, R5, F6) : 101 et 103 valent 0,50 chacune
   et sont départagées par l'identifiant ; il reste vert parce que `make_invoice` leur donne même date et aucune
   échéance (`X`, `F` égales, l'`id` décide). Sa docstring (`:375`, « tie-break invoice_id ASC ») et son commentaire
   (`:386`, « tie-break id ASC ») sont **réécrits** : à score égal, l'ordre d'imputation, qui se réduit ici à l'`id`.

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
  applique la règle d'imputation de la loi (art.~87~CO), dans la mesure où il la connaît~: d'abord une facture
  **échue**, et parmi elles celle **échue la première**~; une facture sans échéance --- ou dont l'échéance précède la
  date --- est tenue pour due dès sa date. À échéance égale, la plus ancienne. Si aucune n'est échue, Kesh propose
  celle dont l'échéance est la plus proche~; la loi désigne alors la dette la moins garantie, que Kesh ne connaît
  pas. » Puis les limites, brèves : un paiement qui porte le **numéro** d'une facture **et son montant exact** la
  désigne et passe devant toute facture qu'il ne désigne pas aussi (score) --- le montant pesant plus que la
  référence, une autre facture au montant exact **peut** en revanche passer devant une facture désignée dont le
  montant diffère, fût-ce d'un centime, et y passe toujours si elle est du même client ; Kesh ne
  connaît pas les poursuites et ne répartit pas un paiement entre deux factures échues le même jour --- pour imputer
  autrement, enregistrez le règlement depuis la fiche de la facture voulue (`\S\ref{sec:reglement-client}`), puis
  rejetez la transaction (*Rejeter*) ; entre deux clients que rien ne distingue, cet ordre n'est qu'un choix par
  défaut~: vérifiez la proposition. *(Formulation de la limite du score : validation P1, R2 — le score est la somme
  pondérée de `matching.rs:80`, la désignation ne prime donc qu'à montant égal ; l'al. 3, l'ordre entre non échues et
  le cas `E < F` : R7, F7. Validation P2, R1 et F1 : « peut passer devant » et non « passe devant » — une facture
  d'un autre client au montant exact (0,50) peut égaler une facture désignée à quelques centimes près dont le nom
  du client correspond (0,40 + 0,10), et l'ordre d'imputation tranche alors ; du même client, elle vaut au moins
  0,10 de plus. Validation P2, R3 : « toute facture qu'il ne désigne pas aussi », une référence pouvant contenir deux
  numéros, ou un numéro contenu dans un autre.)*
- `:2048` (« la facture la mieux notée s'il y en a une ») : ajouter « --- à score égal, celle qu'impute la règle
  ci-dessus » (ou un renvoi équivalent).
- **Inventaire clos** des phrases du manuel qui décrivent un ordre entre factures candidates : `grep -nE "mieux
  notée|score égal|la plus ancienne|plus petit|identifiant croissant" docs/manual/fr/user-manual.tex` au rebase ; au
  sol (`fcd2ae75`) il rend `:2048` (à compléter, ci-dessus), `:2109` (« le plus petit nombre l'emporte » : priorité
  des **règles**, hors sujet) et `:2116` (ordre des **règles**, hors sujet) — trois lignes, dont une seule touchée.
  À relire après la 29-3, qui ajoute des phrases au même §.

PDF régénéré (`make fr` dans `docs/manual/`, sous le verrou de gate) et commité ; contrôle **aplati et ligatures
normalisées** :
`pdftotext docs/manual/fr/user-manual.pdf - | tr '\n' ' ' | tr -s ' ' | sed 's/ﬀ/ff/g;s/ﬁ/fi/g;s/ﬂ/fl/g;s/ﬃ/ffi/g'`,
puis `grep -c "échue la première"` → ≥ 1, `grep -c "art. 87"` (ou la graphie retenue, espace insécable rendue par
`pdftotext`) → ≥ 1. Gardes textuelles : `cargo nextest run -E 'binary(textes_coherents)'` vert.

`docs/api-external.md` § *Accepter des propositions de rapprochement* (`:547`) : la 29-3 y écrit une phrase sur
l'ordre des `candidates` (fiche 29-3 AC8) qui porte **le rang** — factures à indice, puis la règle d'affectation (son
`score.total = 1.0` conventionnel, non comparable), `candidates[0]` affichée et acceptée — et, dans le rang des
factures, « par score décroissant puis selon le départage de `propose_matches` (identifiant croissant au sol ; à
relire si la 29-5, #668, est mergée au rebase) ». ⛔ **Ne remplacer que les deux parenthèses que la 29-5 rend
fausses** (validation P1, R4 et R6) ; **le reste de la phrase de la 29-3 est conservé** mot pour mot :

- la parenthèse du départage « (identifiant croissant au sol ; à relire si la 29-5, #668, est mergée au rebase) »
  devient : « (à score égal, par **date d'exigibilité** croissante — la plus tardive de la date de la facture
  (`invoiceDate`) et de son échéance, l'échéance, absente de la réponse, valant la date à défaut —, puis date de
  facture, puis identifiant ; avant la v0.13.4 : identifiant seul) » (validation P2, R8 : `dueDate` n'est pas un
  champ de la réponse — la candidate porte `invoiceDate`, `routes/reconciliation.rs:140`, et aucune échéance ; en
  police de code, un nom de champ absent tromperait l'intégrateur) ;
- la parenthèse « *(depuis la v0.13.4 ; l'ordre lui-même est celui des versions précédentes)* » devient fausse — la
  29-5 change l'ordre à score égal dans la même version — et devient : « *(depuis la v0.13.4 ; l'ordre des
  candidates est celui des versions précédentes, sauf entre factures de même score, et, au-delà de 50 factures
  ouvertes du même montant, celles qui sont écartées, #668)* » (validation P2, R11 et F7 : la troncature change aussi
  l'ensemble, AC3).

Si la 29-3 n'a rien écrit à cet endroit au rebase, ajouter une phrase propre après « Les propositions viennent de
`GET /api/v1/reconciliation/proposals` » : par score décroissant, puis la parenthèse du départage ci-dessus.
⚠️ **Contrat** : la forme JSON ne change pas ; l'ordre des candidates à score égal change, et, au-delà de 50
factures ouvertes du même montant, l'**ensemble** des candidates aussi (celles qu'écarte la troncature, AC3) — une
intégration qui prenait `candidates[0]` peut recevoir une autre facture qu'avant (validation P2, R11 et F7).

`CHANGELOG.md`, section `## [0.13.4] — Non publié` (créée par la 29-2 ou la 29-3 ; sinon la créer au-dessus de
`## [0.13.3] — 2026-10-14`, forme exacte de `scripts/prepare-release.sh`), `### Corrigé` : « **À score égal, la
réconciliation propose la facture échue la première ([#668](https://github.com/guycorbaz/kesh/issues/668)).** Entre
deux factures que rien ne distingue dans le paiement — même client, même montant, aucune référence —, Kesh proposait
la plus anciennement créée, même pas encore échue, devant une facture échue. Il propose désormais celle sur laquelle
la loi impute le paiement (art. 87 CO) : une facture échue avant une facture qui ne l'est pas, et, parmi les échues,
celle échue la première ; une facture sans échéance est due dès sa date. Un paiement qui porte le numéro d'une
facture et son montant exact la désigne toujours. API : la forme de `GET /api/v1/reconciliation/proposals` ne change
pas, l'ordre des candidates à score égal, si — et, au-delà de 50 factures ouvertes du même montant, celles qui sont
écartées. »

⛔ **L'entrée de la 29-3 dans la même section devient fausse** (sa fiche la dit `Changed` ; la rubrique du dépôt est `### Modifié`, `CHANGELOG.md:110`) (validation P1, R4) : elle dit de l'ordre des
`candidates` qu'il « est désormais écrit et documenté (`docs/api-external.md`) — **il ne change pas** ». Au rebase,
ces derniers mots deviennent « — il ne change qu'entre factures de même score, et dans le choix de celles qu'écarte
la limite de 50 (voir l'entrée de #668, sous `Corrigé`) » ; le reste de
l'entrée de la 29-3 est conservé. Une entrée d'une version **publiée** ne se réécrit pas ; celle-ci est dans
`[0.13.4] — Non publié`, et c'est le même tag qui porte les deux stories.

**Aucun texte de l'écran ne change** : ni clé i18n, ni capture (C-29-5-7). La phrase d'aide que la 29-3 ajoute
au-dessus de la liste (« la facture la mieux notée s'il y en a une ») reste vraie.

**AC8 — Mutations (chacune tuée par un test nommé au Dev Agent Record).**

| # | Mutation | Tuée par |
|---|---|---|
| M1 | `propose_matches` : rétablir `a.invoice_id.cmp(&b.invoice_id)` comme départage | AC5 cas 1, 2, 5 ; AC6 |
| M2 | `invoice_exigibility_date` rend `invoice.date` (échéance ignorée) | AC5 cas 1, 2 ; AC4 unitaires ; AC6 |
| M3 | comparer `invoice.due_date` (`Option`) au lieu de `X` (`None` passe en tête) | AC5 cas 6 ; AC4 unitaire `imputation_order`, ligne 6 du tableau. ⚠️ **Pas** AC5 cas 3 ni 5 : la facture sans échéance y a aussi la plus petite `X`, et M3 la met en tête comme l'ordre correct (validation P1, R1) |
| M4 | `due_date.unwrap_or(date)` sans `max` avec `date` | AC5 cas 5 ; AC4 unitaire `E < F` ; parité AC4 |
| M5 | `X` décroissante | AC5 cas 1, 2 ; AC4 |
| M6 | `imputation_order` sans la composante `F` (`(X, id)`) | AC5 cas 4 (entrée : la plus récente avec le plus petit id) |
| M7 | ordre d'imputation **avant** le score | AC5 cas 8 |
| M8 | `ORDER BY` rétabli à `i.date ASC, i.id ASC` | AC4 `…_by_exigibility_before_limit` (le test `…_same_exigibility_…` ne la tue **pas** : ses `F` croissent avec `X`, l'ancien ordre y garde aussi G1) |
| M9 | SQL : `GREATEST(i.date, i.due_date)` sans `COALESCE` | AC4 parité (`NULL` au décodage `NaiveDate`) **et** `…_by_exigibility_before_limit`, par sa facture sans échéance **obligatoire** à `F = D + 1` (deuxième sous l'ordre correct ; sous M9, **première** sans la 29-2, **exclue** par le `WHERE` avec l'ancre composée — validation P1, F2 et R8 ; validation P2, R5 et F3). Avec l'ancre composée rougissent aussi les tests à factures sans échéance (`:261`, `:314`, cas sans échéance de la 29-2) : mécanisme constaté au Dev Agent Record |
| M10 | SQL : `COALESCE(i.due_date, i.date)` sans `GREATEST` | AC4 parité (cas `E < F`) |
| M11 | composition de l'ancre 29-2 (AC1, dernier point) : retirer le terme des rappels | tests de la 29-2 (cas d, f de son AC5) — **à constater**, pas à écrire |
| M12 | SQL : `ORDER BY {X} ASC, i.id ASC` (sans le second critère `i.date`) | AC4 `…_same_exigibility_by_invoice_date_before_limit` (G2 gardée au lieu de G1) — validation P1, F2 : aucun autre test ne l'exerce, `X = F` partout ailleurs |
| M13 | `imputation_order` sans la composante `id` (`(X, F)`) | AC5 cas 7 (`sort_by` stable : l'ordre d'entrée 102, 101 resterait) ; AC4 unitaire `X`, `F` égales — validation P1, F6 |
| M14 | SQL : `ORDER BY {X} DESC, i.date ASC, i.id ASC` (M5 côté SQL) | AC4 `…_by_exigibility_before_limit` (les quarante-neuf par `E` décroissante, puis la facture sans échéance ; la facture `F = D` écartée) — validation P2, F5 |
| M15 | SQL : `ORDER BY {X} ASC, i.date ASC` (sans `i.id`) | **à exécuter et à déclarer** : seul `…_same_exigibility_…` a des égalités `(X, F)` (ses quarante-neuf de remplissage), et leur ordre de sortie suivra vraisemblablement l'ordre de parcours, donc l'`id` — mutation **probablement équivalente**, sans garantie (un tri sous `LIMIT` ne fixe pas l'ordre des égalités). Le Dev Agent Record la déclare **tuée** ou **équivalente**, avec la sortie du run (validation P2, F5) |

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
      partagées* (forme mergée de l'ancre 29-2 — macro d'ancre attendue, AC1 —, phrase d'ordre de la 29-3 dans
      `api-external.md` et sa parenthèse « l'ordre lui-même est celui des versions précédentes », entrée de la 29-3
      à la section `[0.13.4]` du CHANGELOG et ses mots « il ne change pas », phrases du manuel). **Relire
      explicitement les `Vec` ordonnés des tests de la 29-2** (fiche 29-2 AC5 et M15, validation P1 R3) : leur
      ordre attendu est celui de la requête, que la 29-5 change de `(i.date, i.id)` en `(X, F, id)` — voir
      § *Inventaire (c)*.
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
| Propositions de lettrage (`kesh-core/src/lettering/proposals.rs`, Story 15-1b) | paires débit/crédit de montants égaux, classées par écart de dates croissant ; Kesh n'écrit rien | hors classe : les lignes de pièce — donc de facture — sont exclues des candidates (R5, `repositories/letterings/open_items.rs:531-532`) ; classement sans rapport avec l'imputation d'un paiement sur une facture (validation P2, F6) |
| Factures fournisseur et lots pain.001 | paiements **sortants** | hors classe : Kesh y est le débiteur |
| Troncature `LIMIT 50` sur le **score** (une facture désignée écartée au-delà de 50 candidates) | C-28-7-10 | **non résolu, assumé** (inchangé ; AC3 l'écrit au doc-comment) |
| Poursuites (al. 1), garanties (al. 3), imputation proportionnelle (al. 2) | aucune donnée en Kesh / une seule proposition | **angles morts écrits** (C-29-5-3, C-29-5-5) |
| Clients différents à score égal | art. 87 sans objet | **ordre par défaut écrit** (C-29-5-7) |

### Inventaire (c) — les tests existants dont le sens pourrait changer

```
$ grep -lE "propose_matches|find_unpaid_invoices_for_window|reconciliation/proposals" crates/*/tests/*.rs crates/*/src/*.rs
```

Au sol : `kesh-reconciliation/src/matching.rs` (tests à une facture ou à scores distincts, **sauf**
`propose_matches_returns_sorted_desc`, `:377` : 101 et 103 y valent 0,50 chacune et sont départagées par
l'identifiant — vert sans modification d'assertion parce que `make_invoice` leur donne même date et aucune échéance,
mais sa docstring `:375` et son commentaire `:386` disent le départage ancien et sont réécrits, AC5 cas 9) ; `kesh-db/tests/reconciliation_repository.rs` (factures **sans**
échéance : `X = F`, ordre inchangé ; les tests `:513`, `:566`, `:842`, `:862`, `:945` assertent une présence ou un
montant, pas un ordre) ; `kesh-db/tests/invoice_write_off.rs:785-810` (`any`, pas d'ordre) ;
`kesh-api/tests/reconciliation_e2e.rs`, `reconciliation_rules_e2e.rs`, `bank_accounts_closed_e2e.rs` (aucun ne dépend
de l'ordre de deux factures à score égal — fiche 29-3, inventaire clos des lecteurs de l'ordre ; **à revérifier** au
gate : un rouge y signerait le contraire).

**Les tests de la 29-2** (validation P1, R3 — la fiche le disait à tort « ensembles ») : la parité HTTP (AC6 de sa
fiche, `proposed_couples_287`) asserte un **ensemble** (`BTreeSet`), insensible à l'ordre ; mais l'AC5 de sa fiche
asserte la liste de `find_unpaid_invoices_for_window` comme un **`Vec` ordonné**, « l'ordre de la requête,
`(i.date, i.id)` », répétitions comprises (M15 de la 29-2). Montants distincts par cas : une liste n'y porte qu'**une**
facture (ou ses doublons sous M15), et son ordre ne change pas ; seul le cas **inter-factures** (deux factures, un
rappel sur l'une) peut porter deux factures distinctes dans une même liste, et leur ordre change si leurs `X`
s'ordonnent autrement que leurs `(date, id)`. Au rebase (T0) : `grep -n "assert_eq!(ids\|vec!\[" crates/kesh-db/tests/reconciliation_repository.rs`,
et pour chaque liste à plusieurs factures distinctes, recalculer à la main l'ordre `(X, F, id)` ; une assertion dont
l'ordre change est **réécrite et déclarée** au Dev Agent Record comme modification d'assertion d'un test de la 29-2
(seule exception admise à « aucun test de la 29-2 ne change » de l'AC1, qui vise la composition de l'ancre). Toute
phrase de la 29-2 qui dit l'ordre « de la requête, `(i.date, i.id)` » — commentaire, doc-comment de test — devient
« l'ordre de la requête, `(X, F, id)` (Story 29-5) ».

E2E : la base `kesh_e2e` partagée garde des factures **avec échéance** (contacts « 30 jours net ») ; les specs de
réconciliation tirent des montants uniques (`reconciliation-amount-due.spec.ts`, `Date.now()`) : une facture d'une
autre spec ne devient pas une candidate à score égal. Le Dev Agent Record déclare tout rouge d'une spec de
réconciliation, fichier par fichier.

### Propagation — le symptôme à greper après chaque patch

La **valeur** du départage ancien et ses formulations :

```sh
grep -rnE "invoice_id ASC|invoice_id croissant|identifiant croissant|plus petit .?invoice_id|départag|tie-break|then_with\(\|\| a\.invoice_id|i\.date ASC, i\.id ASC|date ASC, id ASC|plus récentes|\(date, id\)|\(i\.date, i\.id\)|l'ordre de la requête|C-29-2-11|C-29-3-12|#668" \
  crates frontend/src docs/manual/fr/*.tex docs/api-external.md CHANGELOG.md
```

(`date ASC, id ASC` et `plus récentes` ajoutés à la validation P2, R6 : le doc-comment du point 3,
`repositories/reconciliation.rs:210-212`, écrit l'ordre **sans** alias et dit « ce sont toujours les plus récentes
qui sont écartées » — faux après l'AC3 ; le test existant `…_orders_by_date_then_id_before_limit` le dit aussi, au
sol, dans son doc-comment (`tests/reconciliation_repository.rs:304-305`, coupé en fin de ligne : le motif ne le rend
que par `\(date, id\)`) et son message d'assertion (`:378`, « les deux plus récentes sont écartées », qui reste vrai
pour ses factures sans échéance). Ni l'un ni l'autre n'était rendu par le motif `i\.date ASC, i\.id ASC`.)

Et les deux phrases de la 29-3 que la 29-5 rend fausses (validation P1, R4), plus la condition de montant de la
désignation (R2) :

```sh
grep -nE "il ne change pas|celui des versions précédentes|seul l'ordre" CHANGELOG.md docs/api-external.md
grep -rnE "désigne (et prime|toujours)|prime (déjà )?par le score|passe (en revanche )?devant|devant toute autre" docs/manual/fr/user-manual.tex CHANGELOG.md docs/api-external.md crates/kesh-db/src/repositories/reconciliation.rs crates/kesh-reconciliation/src
```

(`passe (en revanche )?devant` et `devant toute autre` ajoutés à la validation P2, R1, F1 et R3 : une facture
d'un **autre** client au montant exact **peut** passer devant une facture désignée à quelques centimes près, elle n'y
passe pas toujours ; du **même** client, si. Toute phrase qui l'affirme sans condition de client se récrit en
« peut passer devant » ou se borne au même client.)

Occurrences attendues à trier : doc-comments de la 29-3 (`order_candidates`, cas 8 de son AC2 : « c'est
`propose_matches` qui a fixé `invoice_id ASC` » → à réécrire), angle mort C-29-3-12 cité au doc-comment de
`order_candidates` (« départage à score égal, cf. fiche 29-3 » ou #668 → **résolu par la 29-5**), doc-comments de la
29-2 sur C-29-2-11 (→ « fermé **à score égal** par la 29-5 ; la candidature sans limite de temps reste », jamais
« résolu » — validation P2, R7), docstring et commentaire « tie-break » de `propose_matches_returns_sorted_desc` (`matching.rs:375`,
`:386`), phrases de la 29-2 sur l'ordre « de la requête, `(i.date, i.id)` » (§ *Inventaire (c)*) ; les entrées
publiées du CHANGELOG **ne se réécrivent pas** — l'entrée de la 29-3 sous `[0.13.4] — Non publié`, si (AC7).

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
| `kesh-db/tests/reconciliation_repository.rs` | aide de rappel, `due_date`, cas a-k — liste assertée en **`Vec` ordonné** `(i.date, i.id)` (son AC5, M15) | aucune | trois tests neufs, deux doc-comments ; **relecture de l'ordre des `Vec` de la 29-2** (§ *Inventaire (c)*, T0) |
| `kesh-api/tests/reconciliation_e2e.rs` | parité étendue, cas #529 | trois tests, `create_rule_ok` | un test neuf |
| `user-manual.tex` § *Algorithme de matching* | `:2029`, `:2031`, `:2902`, `:1305` | `:2041`, `:2049` | après `:2041`, et `:2048` — **regreper les valeurs**, pas les numéros |
| `docs/api-external.md` `:547` | phrase des raisons `payment_date_*` | phrase de l'ordre des `candidates`, dont « l'ordre lui-même est celui des versions précédentes » | remplace **les deux parenthèses** que la 29-5 rend fausses ; le reste de la phrase de la 29-3 conservé (AC7) |
| `CHANGELOG.md` `[0.13.4]` | `Corrigé` (#529) | `Changed` / `Modifié` (#527), dont « il ne change pas » | `Corrigé` (#668) ; « il ne change pas » de la 29-3 corrigé (AC7) — une seule section |

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
- Précédents `macro_rules!` composables par `concat!` (AC1, C-29-5-11) : `crates/kesh-db/src/repositories/accounts.rs:20-34`,
  `crates/kesh-db/src/repositories/invoices.rs:50-65`, `:186-216`.
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
- 2026-10-14 — **Validation P1** (Sonnet, lentilles R et F ; rapports `kesh-gate-logs/29-5-validate-p1-{R,F}.md`) :
  R **0 CRITICAL / 0 HIGH / 4 MEDIUM / 4 LOW**, F **0 / 0 / 2 / 5** ; art. 87 conforme au texte remis par
  l'orchestrateur (les deux lentilles), art. 75 et 86 non confrontés. **Remédiée** (Opus 5.5), tous les findings
  traités, aucun refusé :
  - **F1** (MEDIUM) : l'expression d'exigibilité s'écrit par `macro_rules!` rendant un littéral, insérée dans la
    macro d'ancre de la 29-2 par `concat!` ; tests de la 29-2 inchangés ; relecture à T0 si la forme mergée diffère
    (AC1, C-29-5-9 amendé, C-29-5-11 neuf) ;
  - **F2** (MEDIUM) et **R8** : test neuf `…_same_exigibility_by_invoice_date_before_limit` (coupure du `LIMIT 50`
    à travers un groupe de même `X`) et mutation **M12** ; facture sans échéance **obligatoire**, à `F = D + 1`, dans
    `…_by_exigibility_before_limit` (tue M9 hors parité) ;
  - **R1** (MEDIUM) : sixième ligne d'exemple (facture sans échéance de plus grande `X`), AC5 cas 6 neuf, colonne
    « Tuée par » de M3 corrigée — les cas 3 et 5 ne la tuaient pas ;
  - **R2** (MEDIUM) : la désignation par le numéro ne prime qu'au **montant égal** (`matching.rs:80`) — prémisse,
    § *Ce que la règle ne fait pas*, texte du manuel, CHANGELOG, AC5 cas 8 (troisième sous-cas) ;
  - **R3** (MEDIUM) : les tests de la 29-2 assertent un `Vec` **ordonné** — inventaire (c), zones partagées et T0
    corrigés, relecture explicite au rebase ;
  - **R4** (MEDIUM) et **R6** : les deux phrases de la 29-3 rendues fausses (CHANGELOG « il ne change pas »,
    `api-external.md` « l'ordre lui-même est celui des versions précédentes ») ajoutées à l'AC7, à T0 et au grep de
    propagation ; seules les parenthèses sont remplacées, le reste de la phrase de la 29-3 conservé ;
  - **F3** : la population écartée par la troncature change (exigibilité la plus lointaine au lieu de date la plus
    récente), écrit à l'AC3 ;
  - **F4** : prémisse « poursuite » récrite sur les quatre fichiers réels du grep, chemin `kesh-db` ;
  - **F5** : art. 75 sans guillemets ; convention « exigible = échue » pour une facture sans terme (C-29-5-12) ;
  - **F6 / R5** : docstring et commentaire de `propose_matches_returns_sorted_desc` réécrits (AC5 cas 9) ; consigne
    sur `D` allégée (`propose_matches` ne lit pas la date du paiement) ; mutation **M13** (`imputation_order` sans
    `id`) ;
  - **F7 / R7** : le manuel prescrit dit l'al. 3, l'ordre entre non échues et le cas `E < F` ; inventaire clos du
    manuel recompté (`:2048`, `:2109`, `:2116`) ;
  - **R8** : doc-comment du point 3 relevé à `:208-213`.

  Après remédiation : 9 AC, 7 tâches (T0-T6), **13 mutations (M1-M13)**, **6 lignes d'exemple**, **9 cas à l'AC5**
  (8 neufs, le 9ᵉ les existants), **3 tests neufs** dans `reconciliation_repository.rs`, 4 modules, choix
  **C-29-5-1 à 12**. Symptômes grepés après chaque correction (§ *Propagation*, élargi : `tie-break`,
  `l'ordre de la requête`, `il ne change pas`, `celui des versions précédentes`, `désigne et prime`).
- 2026-10-14 — **Validation P2** (Opus 5.5, lentilles R et F ; rapports `kesh-gate-logs/29-5-validate-p2-{R,F}.md`) :
  R **0 CRITICAL / 0 HIGH / 1 MEDIUM / 10 LOW**, F **0 / 0 / 1 / 7** ; le MEDIUM est **convergent** (R1 = F1) et vient
  de la remédiation P1 (R2). Bilan des findings P1 : tous fermés, sauf R2 rouvert sous une autre forme (le MEDIUM).
  **Remédiée** (Opus 5.5), tous les findings traités, aucun refusé :
  - **R1 = F1** (MEDIUM) : « une autre facture au montant exact passe devant une facture désignée dont le montant
    diffère » est faux pour une facture d'un **autre** client (0,40 + 0,10 contre 0,50 : égalité, l'ordre
    d'imputation tranche). Écrit « **peut** passer devant », et « toujours si elle est du même client » : manuel
    prescrit (AC7), § *Ce que la règle ne fait pas*, prémisse, doc-comment prescrit (AC1), C-29-5-6 amendé ; motif
    `passe (en revanche )?devant` ajouté au grep de § *Propagation* ;
  - **R3** : « passe devant toute autre » → « devant toute facture qu'il ne désigne pas aussi » (`reference_score`
    est un contenant bidirectionnel, `matching.rs:132` — le rapport R citait `:135`, relevé à `:132`) ; motif
    `devant toute autre` ajouté au grep ;
  - **R4 / F2** : la macro d'exigibilité est définie **avant** `invoice_payment_anchor_sql!` (portée textuelle des
    `macro_rules!` ; forme commitée de la 29-2, `8f483388` : macro d'ancre `:163`, développée `:180` et `:186`,
    `payment_date` `:208`), les fonctions à côté de `payment_date` ; insertion « à la place du **premier terme**
    (C-29-2-15) », forme commitée citée ; contexte de C-29-5-9 rectifié ; C-29-5-8 et C-29-5-11 amendés ;
  - **R5 / F3** : M9 décrite dans ses deux régimes — sans la 29-2, la facture sans échéance passe première ; avec
    l'ancre composée, elle est **exclue** par le `WHERE` (`NULL >= ?`) — rouge dans les deux cas, mécanisme constaté
    au Dev Agent Record (AC4, table des mutations) ;
  - **R2 / F4 / R10** : AC5 cas 8 — factures non désignées sans préfixe commun de 4 caractères avec la référence
    (`invoice_number: None` ou `"OTHER"`), toutes du même client ; règle d'entrée « sauf le cas 7 » ;
  - **F5** : mutations **M14** (`{X} DESC` en SQL, tuée par `…_by_exigibility_before_limit`) et **M15** (`i.id`
    retiré de l'`ORDER BY`, probablement équivalente, à exécuter et à déclarer avec la sortie du run) ;
  - **F6** : propositions de lettrage (`kesh-core/src/lettering/proposals.rs`) ajoutées à l'inventaire (b), hors
    classe (lignes de pièce exclues, R5) ;
  - **F7 / R11** : la troncature change aussi l'**ensemble** des candidates au-delà de 50 factures du même montant —
    parenthèse d'API, note de contrat, CHANGELOG et correction de l'entrée de la 29-3 le disent ; motif
    `seul l'ordre` ajouté au grep ;
  - **R6** : motifs `date ASC, id ASC` et `plus récentes` ajoutés au grep (doc-comment du point 3,
    `repositories/reconciliation.rs:210-212`) ; le texte « plus récentes » que R attribuait à un test de la 29-2 est
    **déjà au sol**, dans le test existant de la 28-7 (`tests/reconciliation_repository.rs:304-305`, `:378`) ;
  - **R7** : l'angle mort C-29-2-11 est fermé **à score égal** seulement ; la candidature sans limite de temps reste
    (§ *La règle retenue*, tri du grep) ;
  - **R8** : parenthèse d'API sans `dueDate` (absent de la réponse ; la candidate porte `invoiceDate`,
    `routes/reconciliation.rs:140`) ;
  - **R9 / F8** (commentaire public de #668) : rectifié par l'orchestrateur ; rien dans la fiche.

  Après remédiation : 9 AC, 7 tâches (T0-T6), **15 mutations (M1-M15)**, 6 lignes d'exemple, 9 cas à l'AC5 (8 neufs,
  le 9ᵉ les existants), 3 tests neufs dans `reconciliation_repository.rs`, 4 modules, choix C-29-5-1 à 12 (6, 8, 9 et
  11 amendés). Recompté depuis la fiche (`grep -cE '^\| M[0-9]+ \|'` → 15). Symptômes grepés sur la fiche, le
  registre, l'epic et `sprint-status.yaml` (`passe (en revanche )?devant`, `devant toute autre`, `deux premiers
  arguments`, `à côté de payment_date`, `dueDate`, `seul l'ordre`, `première sous M9`, `M1-M13`). Suite : **passe
  ciblée P3** (Haiku, une lentille sur le diff de cette remédiation, `29-5-validate-prompt-p3-ciblee.md`).
