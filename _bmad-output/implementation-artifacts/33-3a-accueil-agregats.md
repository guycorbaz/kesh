# Story 33-3a : les chiffres des graphiques de l'accueil — revenus et charges par mois, postes de charges, sur l'exercice ouvert

Status: ready-for-dev

*Fiche rédigée le 2026-11-06 en autonomie (Opus 5.5), worktree `/home/gcorbaz/devel/kesh-e33`, branche `story/33-0-kickoff`,
numéros de ligne de `4ef19901`. Epic 33, release **0.17.5**, **sans migration**. Registre : `e33-choix-autonomes-e332.md`, choix
`C-e332-k`. Issue **#918** (`refs` : elle se ferme avec la 33-3b). Modules : `kesh-report`, `kesh-api`. Précède la 33-3b.*

## Story

En tant qu'utilisateur de Kesh, je veux une lecture qui rende, pour l'exercice ouvert à la date du jour, les revenus et les
dépenses de chaque mois et les plus gros postes de charges, tirés du journal, sans fausse dépense ni faux revenu, et dont le
total égale le compte de résultat.

## Ce que le code dit (4ef19901)

1. **Aucune écriture de clôture n'existe — prémisse de l'epic à corriger.** `fiscal_years::close`
   (`crates/kesh-db/src/repositories/fiscal_years.rs:1115`) est un verrou. Le manuel le dit : « Kesh n'en passe aucune »
   (`docs/manual/fr/user-manual.tex:434`), « La clôture est un simple verrou : elle ne génère aucune écriture » (`:854`). Aucune
   colonne ne marque une telle écriture : `JournalEntry` porte `journal`, `description`, `reverses_entry_id`
   (`crates/kesh-db/src/entities/journal_entry.rs:126-147`), et le journal `OD` mélange clôture et régularisation (`:42`). **Il n'y a
   donc rien à écarter ni aucun moyen de le reconnaître.** Le seul résidu : les comptes 9000/9100/9200, typés `Expense`, rendus non
   imputables par `20260909000001_closing_accounts_not_postable.sql` (la migration dit que les montants déjà passés restent en
   charges, `:40-43`), que le Plan comptable peut rouvrir (`:32-33`). Voir C-e332-8.
2. **La contre-passation est un fait de la base.** `journal_entries.reverses_entry_id`, `UNIQUE`
   (`20260828000002_journal_entries_reversal.sql:44-48`). L'inverse se dérive par `WHERE reverses_entry_id = ?`
   (`journal_entry.rs:140-142`). **Pas de chaîne** : une contre-passation ne se contre-passe pas (`ReversalBlocker::IsAReversal`) et
   une écriture ne se contre-passe qu'une fois (`AlreadyReversed`) (`crates/kesh-db/src/errors.rs:57-62`,
   `reversal_blockers`, `journal_entries.rs:2405`). **Miroir exact, figé** : chaque ligne D ↔ C sur le même compte
   (`reverse_in_tx_inner`, `reversal_lines`), et « une fois contre-passées, l'écriture et sa contre-passation ne se modifient ni ne se
   suppriment plus » (`user-manual.tex:745-746`). Une paire vaut donc **zéro, compte par compte**.
3. **Une paire peut franchir l'exercice, et c'est même l'usage d'une de ses deux routes.**
   `ReversalDating::Today` date à l'exercice ouvert du jour (`journal_entries.rs`, étape (4)) ; `NextFiscalYearStart`
   (`reverse_at_next_fiscal_year_start`, `:3093`, Story 29-1b) date au premier jour de l'exercice suivant : c'est la **transitoire**,
   qui n'est pas une correction. Or Guy a validé d'écarter les « contre-passations **de corrections** » (#918, commentaire du
   2026-11-05). Rien, dans l'écriture, ne distingue les deux, sinon l'exercice.
4. **L'avoir n'est pas une contre-passation au sens de la base.** Il a sa propre écriture, sans `reverses_entry_id`
   (`git grep -n reverses_entry_id -- crates/kesh-db/src/repositories/credit_notes.rs` vide ; le motif `OwnedByInvoice`,
   `errors.rs:63-66`, renvoie à lui). Il peut être partiel. Un mois peut donc porter des **revenus négatifs**. L'annulation d'une
   facture fournisseur (`supplier_invoices::cancel`), le dé-rapprochement (`reconciliation_cancel.rs:602`) et l'annulation d'un
   règlement (`invoice_settlements_write.rs:937`), eux, passent par le socle et posent `reverses_entry_id`.
5. **Le précédent d'exclusion.** `reconciliation_link.rs:596-597` écarte déjà les deux côtés :
   `e.reverses_entry_id IS NULL AND NOT EXISTS (SELECT 1 FROM journal_entries r WHERE r.reverses_entry_id = e.id)`. Même forme ici,
   bornée à l'exercice (C-e332-9).
6. **Le sens des montants** : `Revenue` = crédit − débit, `Expense` = débit − crédit (`crates/kesh-report/src/income_statement.rs:1-6`,
   `fetch_section`, `:63-67`). Le compte de résultat sélectionne sur `a.account_type`, société sur `accounts` **et** sur
   `journal_entries`, exercice sur `je.fiscal_year_id`, sans filtre `active` (`:74-85`). `journal_entry_lines` n'a pas de `company_id`
   (`routes/reports.rs`, doc-comment du grand livre).
7. **Une seule devise — le « multi-devises » n'a pas lieu.** `journal_entry_lines` n'a pas de colonne de devise
   (`20260412000001_journal_entries.sql:42-43`, `DECIMAL(19,4)`) ; `git grep -n currency -- crates/kesh-db/src/entities/journal_entry.rs
   crates/kesh-report/src/income_statement.rs` est vide. Aucune conversion, aucun champ `currency` dans la réponse.
8. **Les droits des rapports** : `authenticated_routes`, « tous rôles authentifiés — lecture seule »
   (`crates/kesh-api/src/lib.rs:1232`) ; `AFFICHAGE_LIVRES` = `Role::Consultation`, portée MCP `BooksRead`
   (`services/rapports.rs:132-135`). **Mais chaque rapport écrit une trace** : `Rapport::afficher` → `report.generated`
   (`services/rapports.rs:1-14`, `:560-572`). Le motif prévoit la lecture sans trace : « Une lecture sans trace (le plan comptable) rend
   directement » (`services/mod.rs`, R5). Les trois tuiles de l'accueil ne tracent pas. Voir C-e332-11.
9. **Le motif des services** (`services/mod.rs`) : R1 (pas d'`AppState` ni d'Axum), R2 (`acces.exiger` en première instruction), R3
   (une `Exigence` nommée), R4 (`Lecture` seulement), R7 (test de refus, contrôle positif, exécution sur une `Lecture` en lecture
   seule), R8 (garde lexicale de `tests/services_motif.rs`, que T3 laisse entière).
10. **L'exercice du jour.** `fiscal_years::find_covering_date` (`fiscal_years.rs:676-696`) le lit sans verrou, quel que soit son statut.
    ⛔ **Ne pas employer `find_open_covering_date`** : elle prend un `FOR UPDATE` dans une transaction (`fiscal_years.rs:753-758`, et le doc-comment du module, `:100-125`).
    La date du jour en écriture : `Utc::now().date_naive()` (`reverse_in_tx_inner`).
11. **Un exercice n'a pas forcément douze mois** : `chk_fiscal_years_dates CHECK (end_date > start_date)` seulement
    (`20260404000001_initial_schema.sql:51`). « Douze mois » est le cas ordinaire, non une borne.
12. **Le décimal** : `rust_decimal`, `serde-str` (`crates/kesh-report/Cargo.toml:15`) : les montants sortent en chaînes exactes, que
    le frontend relit par `big.js` (`frontend/src/lib/features/homepage/homepage.ts:3-5`).
13. **Le registre des routes mutantes ne s'applique pas** : il ne porte que les routes mutantes (`tests/audit_route_registry.rs:1-3`).

## Décisions

Décisions de Guy (#918, epic) : comptabilité ; exercice ouvert même vide ; par mois ; postes par compte ; cinq plus « Autres » ;
clôtures et contre-passations de corrections écartées. Le reste, au registre :

- **C-e332-8** : aucune exclusion au titre de la clôture, puisqu'il n'y en a pas ; un test de caractérisation le garde.
- **C-e332-9** : une paire s'écarte **des deux côtés, si et seulement si les deux écritures sont dans l'exercice lu**. Une paire à
  cheval sur deux exercices reste comptée, de part et d'autre.
- **C-e332-10** : d'où l'**invariant** : `totalRevenues` et `totalExpenses` égalent ceux du compte de résultat de l'exercice.
- **C-e332-11** : route sans trace d'audit et non servie au MCP.
- **C-e332-12** : l'avoir reste compté ; les montants mensuels sont signés.
- **C-e332-13** : au camembert, seuls les comptes de charges de solde **strictement positif**. Un compte **créditeur** est compté à part ; un compte **à solde nul** ne
  paraît nulle part, comme au compte de résultat (`HAVING balance != 0`, `income_statement.rs:83`).
- **C-e332-14** : autant de mois que l'exercice en couvre.
- **C-e332-15** : « aucun exercice ouvert » est un `200` explicite, et un exercice couvrant le jour mais clos en relève.

## Contrat

`GET /api/v1/reports/homepage-summary`, sans paramètre.

```jsonc
// État ordinaire
{
  "state": "open",
  "asOf": "2026-11-06",
  "fiscalYear": { "id": 3, "name": "Exercice 2026", "startDate": "2026-01-01", "endDate": "2026-12-31" },
  "months": [
    { "month": "2026-01", "from": "2026-01-01", "to": "2026-01-31", "revenues": "1000.0000", "expenses": "300.0000" }
    // … un par mois civil touché par l'exercice, y compris les mois à venir, à "0"
  ],
  "totalRevenues": "1000.0000",
  "totalExpenses": "300.0000",
  "expensePosts": [
    { "accountId": 41, "number": "6000", "name": "Loyer", "amount": "200.0000", "share": "66.7" }
    // au plus cinq, du plus gros au plus petit ; à égalité, par numéro de compte
  ],
  "otherExpenses": { "amount": "100.0000", "share": "33.3", "accountCount": 2 }, // null si au plus cinq comptes
  "expensePostsBase": "300.0000",       // somme des parts, base des pourcentages
  "creditBalanceExpenseAccounts": 0     // comptes de charges au solde créditeur, écartés du camembert
}
// Aucun exercice ouvert à la date du jour
{ "state": "noOpenFiscalYear", "asOf": "2026-11-06", "fiscalYear": null, "months": [], "totalRevenues": "0", "totalExpenses": "0",
  "expensePosts": [], "otherExpenses": null, "expensePostsBase": "0", "creditBalanceExpenseAccounts": 0 }
```

## Critères d'acceptation

### Exercice

- **AC1** — `state = "open"` et l'exercice couvrant `asOf` si son statut est `Open`. Cet exercice **vide** : `200`, tous les mois à
  `"0"`, `expensePosts` vide.
- **AC2** — `state = "noOpenFiscalYear"`, `200`, forme ci-dessus, dans **trois** cas : aucun exercice ; un exercice couvrant le jour
  mais clos, même si un autre est ouvert ; seul un exercice ne couvrant pas le jour est ouvert. Aucune bascule sur un autre exercice.
- **AC3** — `months` : un élément par mois civil touché, dans l'ordre. `from`/`to` bornés à l'exercice : un exercice du 15.03 au
  14.03 suivant rend **treize** mois, le premier du 15.03 au 31.03, le dernier du 01.03 au 14.03. Un exercice de 18 mois en rend 18.
  Mois de février d'une année bissextile juste.

### Montants

- **AC4** — Revenus = Σ(crédit − débit) des comptes `Revenue`, dépenses = Σ(débit − crédit) des comptes `Expense`, par mois de
  `entry_date`. Une ligne de bilan (`Asset`, `Liability`) n'y entre pas. Les montants sont **signés** : un mois qui ne porte qu'une
  ligne de charge au crédit rend `expenses` négatif, de même un revenu au débit (C-e332-12). Pas d'arrondi : la fraction à quatre
  décimales d'une ligne reste.
- **AC5** — **Invariant** (C-e332-10) : `totalRevenues` et `totalExpenses` égalent `totalRevenues` et `totalExpenses` de
  `GET /reports/income-statement?fiscalYearId=<id>`, comparés en `Decimal`, sur un montage qui mêle écritures ordinaires, une paire
  dans l'exercice, une paire à cheval, un avoir et une charge créditrice. Et `Σ months[].revenues = totalRevenues`, de même pour les
  dépenses.
- **AC6** — Société : les écritures et les comptes d'une autre société n'y entrent jamais.

### Contre-passations

- **AC7** — **Paire dans l'exercice, à deux mois différents** : origine charge de 500 au mois M, contre-passation au mois M+k. Le
  mois M et le mois M+k rendent **tous deux** ce qu'ils rendraient sans la paire, et ni le poste ni le total ne bougent. Écarter
  seulement l'un des côtés rend M à +500 ou M+k à −500 : c'est ce que le test sépare. Idem pour un revenu.
- **AC8** — **Paire à cheval** (`reverse_at_next_fiscal_year_start`, origine en N−1, contre-passation au premier jour de N, seul N
  ouvert à la date du jour) : la contre-passation **compte** dans le mois de N (−X), et le total égale le compte de résultat de N.
  Aucun autre montant n'est écarté. **Lue depuis N−1** (encore ouvert, `generate` appelé avec une date de N−1, la route lisant `Utc::now()`) : l'origine **compte**, à son mois (+X), et le total égale le compte de résultat de N−1.
- **AC9** — **Annulation d'une facture fournisseur dans le même exercice** (le socle) : l'achat et sa contre-passation disparaissent
  ensemble, sans mention particulière du type de pièce.
- **AC10** — Un **avoir** (écriture sans `reverses_entry_id`) reste compté, à son mois, en revenus négatifs.

### Clôture

- **AC11** — Caractérisation (C-e332-8) : clôturer l'exercice N−1 par `fiscal_years::close` n'ajoute aucune écriture
  (`COUNT(*)` de `journal_entries` égal), et la réponse pour N est identique octet pour octet avant et après. **Ce test tombera si
  une clôture écrit un jour** : c'est son rôle, et le message d'assertion renvoie à la décision de Guy de les écarter.

### Postes

- **AC12** — Sept comptes de charges de soldes distincts : cinq `expensePosts` décroissants, `otherExpenses` = somme des deux derniers,
  `accountCount = 2`.
- **AC13** — Bornes : cinq comptes → `otherExpenses: null` ; six → `accountCount = 1` ; égalité de soldes → ordre par numéro de compte,
  déterministe.
- **AC14** — La requête par compte porte `HAVING balance != 0`, comme `fetch_section` (`income_statement.rs:83`) : un compte de charges **à solde nul** (après écart
  des paires), par exemple des lignes qui se compensent sans former une paire, n'est dans aucun champ et n'est compté nulle part. Un compte de solde **négatif** n'est
  **ni** dans `expensePosts`, **ni** dans `otherExpenses`, et compte dans `creditBalanceExpenseAccounts`. `expensePostsBase` = somme de ce qui reste. **Si aucun compte n'a de
  solde positif**, `expensePosts` est vide et `expensePostsBase = "0"`, sans division par zéro. Un total des charges négatif ne suffit pas : un compte à +100 et un à −300
  rendent un poste de 100, à `share = "100.0"`, et `creditBalanceExpenseAccounts = 1`.
- **AC15** — `share` = `amount / expensePostsBase × 100`, **toujours une décimale écrite**, arrondi `MidpointAwayFromZero`. Exemples de test : 200 et 100
  → `66.7`, `33.3` ; 100 et 100 → `50.0`, `50.0` ; un seul poste → `100.0`. Pas de redressement de la somme à 100 : l'écran affiche ce qui est calculé.
- **AC16** — `name` est le nom du compte, `number` son numéro, sans filtre `active` : un compte archivé qui porte des écritures reste un
  poste, comme au compte de résultat.

### Droits

- **AC17** — Sans jeton : `401`. `Consultation`, `Comptable`, `Admin` : `200`. Une clé d'API en lecture : `200`.
- **AC18** — Le service s'ouvre par `acces.exiger(EXIGENCE_SYNTHESE_ACCUEIL)` : `role_api: Role::Consultation`, `portee_mcp: None` ;
  canal MCP, quelle que soit la portée : refusé avant toute lecture (R7). Exécution sur une `Lecture` en lecture seule (R7).
- **AC19** — Aucune écriture : `COUNT(*)` de `audit_log` inchangé après deux appels (C-e332-11).

## Tâches

### T0 — Étape 0

Relire les prémisses sur le dernier `HEAD`, lignes citées au Change Log. `git grep -n "reverses_entry_id"` : aucune nouvelle façon de
contre-passer ? `credit_notes.rs` n'en pose toujours pas ?

### T1 — Tests rouges (`crates/kesh-api/tests/homepage_summary_e2e.rs`, neuf ; `#[sqlx::test(migrations = "../kesh-db/test-schema")]`)

Montage repris de `reports_e2e.rs` (`spawn_app`, `forge_jwt`). L'exercice du montage se calcule depuis `Utc::now().date_naive()` : il
commence au **1er du mois, trois mois avant aujourd'hui**, pour qu'il y ait toujours deux mois écoulés et que le test ne rougisse pas en
janvier. `N−1` le précède immédiatement.

| Test | AC | Rouge parce que |
|---|---|---|
| `summary_of_an_empty_open_year` | 1 | route absente |
| `no_open_year_in_three_cases` | 2 | idem |
| `months_follow_the_fiscal_year_bounds` | 3 | idem |
| `signs_and_only_income_statement_accounts` | 4 | idem |
| `totals_equal_the_income_statement` | 5, 8, 10 | idem |
| `other_company_never_enters` | 6 | idem |
| `reversal_pair_in_the_year_leaves_both_months_untouched` | 7 | idem |
| `reversal_across_two_years_counts_on_both_sides` | 8 | idem |
| `cancelled_supplier_invoice_disappears` | 9 | idem |
| `credit_note_stays_counted` | 10 | idem |
| `closing_a_year_writes_no_entry_and_changes_nothing` | 11 | idem |
| `top_five_and_others`, `boundaries_five_six_and_ties` | 12, 13 | idem |
| `credit_balance_expense_accounts_are_left_out`, `zero_balance_account_is_counted_nowhere` | 14 | idem |
| `shares_are_rounded_once`, `archived_account_stays_a_post` | 15, 16 | idem |
| `any_role_reads_without_token_refused`, `api_key_reads` | 17 | idem |
| `reading_writes_nothing` | 19 | idem |

Dans `crates/kesh-report/src/homepage_summary.rs` (`#[cfg(test)]`, purs) : `calendar_months` (AC3), `rank_expense_posts` (AC12 à 15).

Dans `services/rapports.rs` ou `tests/services_motif.rs`, selon le voisinage : AC18.

### T2 — `crates/kesh-report/src/homepage_summary.rs` (neuf)

- `pub async fn generate(pool, company_id, today: NaiveDate) -> Result<HomepageSummary, ReportError>` ; `today` en paramètre pour les
  tests et pour R1.
- Fonctions pures `calendar_months(start, end) -> Vec<MonthBounds>`, `rank_expense_posts(rows) -> RankedPosts`, sans I/O.
  `rank_expense_posts` range un solde `> 0` en poste et **tout le reste** en créditeur : c'est la clause `HAVING balance != 0` seule qui écarte le zéro (sans quoi M18 survivrait, la clause étant redondante).
  `share` : `round_dp_with_strategy(1, MidpointAwayFromZero)` puis `rescale(1)`, car l'arrondi rend sa valeur telle quelle dès que l'échelle est ≤ 1 (`rust_decimal-1.43.0/src/decimal.rs:1502-1509`) : sans cela, `"50"` et non `"50.0"`.
- Constante partagée (DRY) par les deux requêtes :

  ```sql
  AND NOT EXISTS (SELECT 1 FROM journal_entries o
                   WHERE o.id = je.reverses_entry_id AND o.fiscal_year_id = je.fiscal_year_id)
  AND NOT EXISTS (SELECT 1 FROM journal_entries r
                   WHERE r.reverses_entry_id = je.id AND r.fiscal_year_id = je.fiscal_year_id)
  ```

- Requête mensuelle : `DATE_FORMAT(je.entry_date, '%Y-%m')`, `a.account_type`, somme selon le type, mêmes jointures et mêmes filtres que
  `fetch_section` ; requête par compte, pour `Expense`. Les mois sans ligne sont complétés à zéro côté Rust.
- `map_db_error` comme les voisins. Doc-comment du module : l'invariant, la paire, l'avoir, l'absence d'écriture de clôture, et le renvoi
  à la 33-3b.
- Réexport dans `lib.rs`.

### T3 — Le service et la route

- `services/rapports.rs` : `EXIGENCE_SYNTHESE_ACCUEIL`, `synthese_accueil(acces, lecture) -> Result<HomepageSummary, AppError>`, qui rend
  directement (R5), date du jour par `Utc::now().date_naive()`. Inscrire le service à l'inventaire de `services/mod.rs`.
- `routes/reports.rs` : **`homepage_summary`, sans le préfixe `get_`**, sur le modèle de `get_income_statement`, sans `Rapport`. La garde `ac10_vii_le_bon_usage_a_chaque_handler` (`tests/services_motif.rs:2687-2710`) compte neuf `get_…` qui appellent `.afficher(` : elle reste entière, **sans amendement**, et le nom dit que ce handler ne trace pas. `ac10_i_ii_handlers_sans_le_pool` reste vraie (`state.lecture()`, pas `state.pool`).
- `lib.rs` : `.route("/api/v1/reports/homepage-summary", get(...))`, dans le bloc de la Story 9-1 (`:1232`).

### T4 — Documents

- `docs/api-external.md` : un paragraphe sur la route et son contrat (mois, paires, avoir, parts), sur le modèle de `:259-261` ; aucune ligne au tableau des ressources, qui ne porte aucun rapport (`:209-226`).
- **Pas de manuel ni de CHANGELOG utilisateur dans cette story** : rien n'est visible avant la 33-3b, qui les porte. Si les deux stories
  ne partaient pas dans la même release, la ligne `docs/api-external.md` resterait vraie seule.
- `CHANGELOG.md`, `## [0.17.5] — Non publié` : la ligne de #918 s'écrit à la 33-3b ; ici, aucune.

## Mutations (`kesh-gate-logs/e333a/vmut.sh`, sous le verrou, restauration `git checkout --` puis `touch`)

| Id | Mutation | Tuée par |
|---|---|---|
| M1 | `NOT EXISTS` de l'origine retiré | `reversal_pair_in_the_year_...` (M+k à −500) |
| M2 | `NOT EXISTS` de la contre-passation retiré | idem (M à +500) |
| M3 | condition d'exercice retirée du premier | `reversal_across_two_years_...`, et `totals_equal_...` |
| M4 | condition d'exercice retirée du second | `reversal_across_two_years_...`, volet lu depuis N−1 (AC8) |
| M5 | sens de `Revenue` inversé | `signs_...` |
| M6 | sens de `Expense` inversé | `signs_...` |
| M7 | filtre `je.fiscal_year_id` retiré | `totals_equal_...` |
| M8 | filtre `company_id` de `je` retiré ; puis celui de `a` | `other_company_never_enters` |
| M9 | statut `Open` non contrôlé | `no_open_year_...` |
| M10 | `take(5)` → `take(6)` | `top_five_...` |
| M11 | tri croissant ; tri à égalité retiré | `top_five_...`, `boundaries_...` |
| M12 | filtre `> 0` retiré | `credit_balance_...` |
| M13 | base = total des charges | `credit_balance_...` (+100 et −300) |
| M14 | `Consultation` → `Comptable` | `any_role_...` |
| M15 | `portee_mcp: Some(BooksRead)` | AC18 |
| M16 | `from`/`to` non bornés | `months_follow_...` |
| M17 | exclure les avoirs par `journal = 'Ventes'` et montant négatif (mutation d'intention) | `credit_note_stays_counted` |
| M18 | `HAVING balance != 0` retiré de la requête par compte | `zero_balance_account_is_counted_nowhere` |

## Gates

Commun : `e33-commun.md`. Entre passes : `cargo fmt --all -- --check`, clippy du workspace,
`cargo nextest run -E 'binary(homepage_summary_e2e)'`. Au dernier commit de code : `scripts/test-fast.sh --ci`. Pas d'écran dans cette
story : l'E2E complet revient à la 33-3b, dernier commit de code de la paire ; le Dev Agent Record le dit tel quel et n'affirme pas un E2E
qui n'a pas tourné.

## Dev Notes

- **Pourquoi pas `income_statement::generate`** : il ne rend ni par mois, ni sans les paires, et sa trace n'a pas lieu d'être ici.
  `AccountBalance` (`balance_sheet`) ne se réemploie pas non plus : il exige `active` et `role`. Le réemploi, c'est le sens des montants,
  la forme des jointures, `map_db_error`, et l'invariant qui les tient égaux.
- **Limite assumée de la date** : `Utc::now().date_naive()`, comme la contre-passation. Dans la nuit du 31 décembre au 1er janvier, une à
  deux heures à Zurich, l'accueil lit encore l'exercice précédent. Issue demandée à l'orchestrateur (P4), avec les autres sites.
- **Ce qu'écarter les deux côtés coûte** : au grand livre ou à la liste des écritures, l'utilisateur retrouve les deux écritures, dont la
  somme est nulle ; le graphique ne les montre pas. Le manuel le dit (33-3b).
- **Ce qui reste compté et pourrait surprendre** : l'avoir, la contre-passation d'un exercice antérieur, et un montant que l'utilisateur
  aurait passé sur 9000 après avoir rouvert ce compte. Celui-là fausse aussi le compte de résultat et le bilan : le camembert le rend
  visible, ce qui vaut mieux qu'un chiffre poli (C-e332-8).
- **Performance** : deux requêtes groupées sur l'exercice, sur les index existants (`reverses_entry_id` unique, `fk`). Pas de mesure
  exigée ; à dire si une requête dépasse la seconde sur la base de démonstration.

## Recoupements

`routes/reports.rs` et `services/rapports.rs` : la **33-7** touche aussi ces deux fichiers et `mcp/outils/rapports.rs`, la **33-4** le premier et ce dernier ; `tests/services_motif.rs` : la
33-3a n'y amende aucune garde (T3) et peut y ajouter le test d'AC18, la 33-7 y touche aussi. Lignes distinctes, union à l'intégration. `tests/audit_route_registry.rs` : la route est en lecture, **aucune ligne à y ajouter** (prémisse 13).

## Change Log

- 2026-11-06 — Fiche rédigée en autonomie (Opus 5.5).
- 2026-11-06 — Remédiation de la validation P1 (rapport `kesh-gate-logs/e33-validate-p1-e332.md`, Opus 5.5, aucune compilation ni aucun test). R-7 (LOW) remédié, prémisse lue : `income_statement.rs:83`, `HAVING balance != 0` :
  la requête par compte garde la clause, le solde nul ne paraît nulle part, et le champ devient `creditBalanceExpenseAccounts`, le mot « non positif » n'ayant plus d'objet. **Défaut voisin corrigé, trouvé en remédiant** : AC14 disait qu'un total
  des charges négatif vidait `expensePosts`, ce qui est faux dès qu'un compte reste positif (+100 et −300). Recoupements ajoutés à partir de L-F4-1 et L-F7-2 du rapport `e33-validate-p1-e334.md`. C-e332-9 et C-e332-14, déjà lus par Guy,
  ne sont pas rouverts ; ils se disent au manuel (33-3b, T6). Décomptes de cette fiche, recomptés : 19 AC, 18 lignes de mutation.
- 2026-11-06 — Remédiation de la validation P2 (rapport `kesh-gate-logs/e33-validate-p2-e332.md`, Opus 5.5, aucune compilation ni aucun test, aucun commit). Quatre constats de cette fiche, tous LOW, tous remédiés, aucun réfuté : R-3, R-4, R-5, R-11. Prémisses relues : `tests/services_motif.rs:2073-2075`, `:2145-2157`, `:2687-2710`, et `routes/reports.rs` absent de `SITES_NON_RESOLUS` (R-3) ; `routes/reports.rs:245-254` (`state.lecture()`) ; `rust_decimal-1.43.0/src/decimal.rs:1050`, `:1502-1509` (R-5, l'échelle exacte d'une division n'étant pas jouée) ; `repositories/journal_entries.rs:3093-3110`, et `kesh-report` en dépendance de `kesh-api` (`Cargo.toml:14`, R-11).
  Le handler se nomme `homepage_summary` : **aucune garde n'est amendée**, et le « peut-être » de T3 et des recoupements disparaît. Décomptes de cette fiche, recomptés : 19 AC, 18 lignes de mutation, inchangés.
- 2026-11-06 — Remédiation de la validation P3 (rapport `kesh-gate-logs/e33-validate-p3-e332.md`, aucune compilation ni aucun test, aucun commit). **F-3 (LOW) remédié**, prémisse lue : `api-external.md:209-226`. **F-2 (LOW) réfuté** : à l'en-tête de T1, « neuf » veut dire *nouveau fichier*, comme à l'en-tête de T2, non un décompte ; rien à recompter (le tableau nomme vingt fonctions). Décomptes de cette fiche, recomptés : 19 AC, 18 lignes de mutation, inchangés.

## Dev Agent Record

*À remplir au développement.*
