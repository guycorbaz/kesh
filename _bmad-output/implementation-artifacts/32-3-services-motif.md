# Story 32.3 : la couche de services — le motif, sur deux modules pilotes (rapports, plan comptable)

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-32-3`, branche `story/32-3` partie de la spike
`275e9b97`. Registre : `epic-32-choix-autonomes-323.md` (C-32-3-1 à C-32-3-30). Clé courte : `323`. Issue de l'epic :
#723 (CR « Serveur MCP intégré à Kesh ») ; issue du défaut conservé : #729 (export du grand livre tracé en consultation) ;
issue de l'écart antérieur du manuel : #730 (rôle Consultation et clés « sans écriture ») ; toutes trois en `refs`
seulement. Les numéros de ligne cités sont ceux de `275e9b97`.*

## Story

En tant que Guy, qui va brancher un agent sur ses livres réels par le serveur MCP de la 0.16.0, je veux que l'API et le
futur MCP lisent les livres **par les mêmes services**, qui portent eux-mêmes le contrôle d'accès et séparent la lecture
des données de l'écriture de la trace, afin que le MCP ne puisse ni contourner une règle d'accès écrite ailleurs, ni
perdre en silence la ligne d'audit d'un rapport lorsque le MCP lira par une connexion en lecture seule (32-6).

## Origine

- Décision du Project Lead (`planning-artifacts/epic-32-serveur-mcp.md` § *Décisions*) : « une couche de services
  partagée par l'API et le MCP, le contrôle d'accès **dans les services** ». Story-zéro du motif (règle de découpage
  préventif), pilotes **les rapports** (le cas difficile : chaque rapport écrit au journal d'audit) et **le plan
  comptable** (le cas simple) ; rollout mécanique en 32-4a/32-4b (C-32-0-4 : les **lectures** seulement).
- Prémisse 6 du kickoff, revérifiée au code : **des lectures écrivent**. Chaque handler de `routes/reports.rs` appelle
  l'une de **sept** fonctions d'audit qui ouvrent une transaction sur `state.pool` et insèrent dans `audit_log`, l'échec
  étant **avalé** par un `tracing::warn!` (`emit_report_audit`, l. 1407-1455, le `warn!` l. 1446-1454). Sur une
  connexion en lecture seule, l'`INSERT` échouerait en 1792 (spike, § 4.1) et la trace disparaîtrait **sans bruit**.
- Spike 32-2, § 4.4 et § 6 (32-3) : « le service reçoit `Lecture` et `Trace` **typées et distinctes** » ; la fiche
  inventorie l'ensemble clos des écritures sur les chemins de lecture des pilotes ; la mutation « la trace part sur
  `Lecture` » est tuée.
- ⚠️ **Q11 (forme de la lecture seule : mode de session, utilisateur MariaDB limité au `SELECT`, ou les deux) est en
  attente du Project Lead. Cette story n'en dépend pas** : le service reçoit une poignée `Lecture`. **La fabrication
  d'une `Lecture` en lecture seule** — mode de session, utilisateur `SELECT`, ou les deux, et le lieu où elle se
  fabrique — **est décidée par Guy (Q11) et réalisée par la 32-6** ; la 32-3 ne suppose rien d'autre qu'une poignée
  **distincte** de la `Trace` (C-32-3-22). En 32-3, `Lecture` et `Trace` enveloppent **le même** pool de production
  (`state.pool`) ; seul le test (AC9) fabrique une `Lecture` en mode de session `READ ONLY`, pour prouver la séparation
  — ce n'est **pas** la preuve de la lecture seule de production, qui appartient à la 32-6 (C-32-3-7).

## Ce que le code dit (relevé au code le 2026-10-24, `275e9b97`)

1. **`routes/reports.rs` porte 18 handlers publics** (`grep -c "^pub async fn" crates/kesh-api/src/routes/reports.rs`
   → 18) : 9 consultations JSON et 9 exports PDF/CSV, chacun appelant **exactement une** fonction d'audit (18 sites
   d'appel, relevés au tableau de l'AC6).
2. **Les sept fonctions d'audit** — `emit_ledger_audit` (l. 1372), `emit_report_audit` (l. 1407),
   `emit_report_export_audit` (l. 1463), `emit_project_report_audit` (l. 1514), `emit_project_report_export_audit`
   (l. 1548), `emit_aged_receivables_audit` (l. 599), `emit_aged_receivables_export_audit` (l. 626) — prennent toutes
   `pool: &MySqlPool`, ouvrent `pool.begin()`, appellent `audit_log::insert_in_tx` avec `NewAuditLogEntry::user(…)`,
   `commit`, et avalent l'erreur par `warn!`.
3. **Tous les générateurs de `kesh-report` prennent `&MySqlPool`** (`generate` de `balance_sheet`, `income_statement`,
   `trial_balance`, `journal_report`, `vat_report`, `aged_receivables`, `general_ledger` ; `generate_project_expenses`,
   `generate_project_return`, `resolve_scope`, `ReportPeriod::resolve`) et **aucun n'écrit** :
   `grep -rniE "for update|insert |update |delete from|\.begin\(\)" crates/kesh-report/src` ne rend qu'une ligne de
   commentaire (`trial_balance.rs:30`). **`kesh-report` n'a donc pas à changer** : `Lecture` expose son pool aux
   générateurs (C-32-3-2).
4. **La seule exception à l'ordre « générer puis tracer » est l'export** : le rendu PDF/CSV (`render_*`) se fait
   **avant** l'audit (p. ex. `export_balance_sheet`, rendu l. 487-490, audit l. 495) ; un rendu en échec ne laisse donc
   aucune ligne d'audit. L'ordre doit être tenu (C-32-3-3).
5. **Un écart de trace existant, conservé tel quel** : `export_general_ledger` (l. 362) trace par `emit_ledger_audit`,
   donc en `report.generated` et **sans format** — l'export du grand livre ne se distingue pas, dans le journal
   d'audit, d'une consultation à l'écran. C'est une **piste de contrôle incomplète** (P2 au barème) ; le corriger
   changerait le comportement, ce que cette story s'interdit : **#729** (`bug`, P2, ouverte), en `refs` (C-32-3-8).
6. **Limitation L2 conservée** : les traces de lecture sont écrites en `actor_type='user'` même pour une requête par
   clé d'API (commentaire l. 1422-1425). Inchangé ici ; la question de l'acteur d'une clé MCP revient à 32-5/32-6.
7. **Le plan comptable** (`routes/accounts.rs`) n'a qu'**une** lecture, `list_accounts` (l. 188-212) :
   `get_company_for` (`helpers.rs:66`, erreur `Internal` si la société manque), `accounts::list_by_company`
   (`&MySqlPool`), puis `letterings::letterable_account_ids` sur une connexion acquise du pool — trois lectures, aucune
   écriture. Les quatre handlers mutants (`create`, `update`, `archive`, `reactivate`) et l'aide `response()` (l. 179)
   **restent** dans le handler (C-32-0-4).
8. **RBAC actuel** (`lib.rs`) : toutes les routes de rapports et `GET /api/v1/accounts` sont dans
   `authenticated_routes` (tous rôles), **sauf** `GET /api/v1/reports/aged-receivables/export`, dans
   `comptable_routes` (l. 588-591). L'export du grand livre est tous rôles (l. 1074-1075). La comparaison de rôle est
   `current_user.role < min_role → AppError::Forbidden` (`middleware/rbac.rs:24-25`, ordre `Role::level`).
9. **Couverture actuelle de la trace** (périmètre : `crates/*/tests` à `275e9b97`, `grep -rnE
   "report\.(generated|exported)" crates/*/tests` → `reports_e2e.rs` seul) : **trois tests, tous sur le bilan en
   consultation** — un seul lit le **contenu** de la ligne, `report_generated_audit_emitted_on_success`
   (`tests/reports_e2e.rs:955`) ; deux la **comptent**, son pendant négatif (l. 1027) et
   `audit_log_scoped_to_company_via_user_join` (l. 1060). Aucun test ne lit la trace d'un export, du grand livre, de la
   balance âgée, d'un rapport de projet, ni des quatre autres rapports à période en consultation (`grep -n "audit_log"
   crates/kesh-api/tests/{reports_export_e2e,vat_report_e2e,project_reports_e2e}.rs` → rien). Une trace perdue serait
   donc **muette** aujourd'hui sur **dix-sept des dix-huit handlers** — sur **sept des huit chemins de trace** (forme ×
   usage : période, grand livre, balance âgée, projet), le seul couvert étant « période × affichage », et pour le seul
   bilan. L'AC9 (b) et (b′) les figent sur les dix-huit couples.
10. **`AppState` se construit aussi par littéral dans des tests** (commentaire de `new_for_tests`, `lib.rs:98-130` :
    « les tests d'envoi construisent AppState littéral ») : ajouter un **champ** à `AppState` modifierait des tests
    existants. Les poignées se fabriquent donc par **méthodes** sur `AppState`, sans champ neuf (C-32-3-6).

## Le motif (ce que la 32-4 répliquera)

```text
handler (routes/…)            service (services/…)                        poignées
  extraire la requête   ──►   1. acces.exiger(Exigence)        ◄── avant toute lecture
  (format d'export :          2. valider la demande (règles d'aujourd'hui, même ordre)
   validé ici, présentation)  3. lire par Lecture seulement    ──► Lecture  (lecture des livres)
                              4. rendre Rapport<T, U>
  rapport.afficher(&trace) / rapport.exporter(&trace, rendre)  ──► Trace    (journal d'audit)
  mettre en forme la réponse HTTP
```

- **`ContexteAcces`** : société, utilisateur, **canal** (`Api { role }` ou `Mcp { portees }`), en champs **privés**
  lus par accesseurs (C-32-3-29). Construit par `ContexteAcces::depuis_api(&CurrentUser)` pour l'API ;
  `ContexteAcces::mcp(company_id, user_id, portees)` existe pour les tests et la 32-6 ; aucun autre constructeur.
- **`Exigence`** : `{ role_api: Role, portee_mcp: Option<Portee> }` — `None` : jamais servi au MCP. `acces.exiger(e)`
  rend `Err(AppError::Forbidden)` si `Api { role }` et `role < e.role_api`, ou si `Mcp { portees }` et que
  `e.portee_mcp` est `None` ou absente de `portees`.
- **`Portee`** : `BooksRead`, `InvoicesRead` en 32-3 (les deux dont les pilotes ont besoin, C-32-0-7) ; la 32-5 ajoute
  les autres et leur persistance (C-32-3-4).
- **`Lecture`** et **`Trace`** : deux types distincts, sans conversion de l'un à l'autre, chacun un tuple à champ
  **privé** (`pub struct Lecture(MySqlPool);`), donc constructible par `new` seulement. Chacun expose son pool **à la
  seule couche de services** (`pub(in crate::services) fn pool(&self) -> &MySqlPool`) : celui de la `Lecture` passe aux
  dépôts, aux générateurs et à `helpers::company_for_ids`, celui de la `Trace` aux seules fonctions d'audit de
  `services/rapports.rs`. Hors de `services`, le compilateur refuse `lecture.pool()` (C-32-3-17) ; **dans** `services`,
  ce qui ouvrirait un autre chemin vers le pool — un accesseur d'un autre nom, un `impl Deref`/`AsRef`/`From`, une
  conversion — est attrapé par l'**inventaire clos** de l'AC10 (v), non par le compilateur (C-32-3-24). En production,
  fabriqués **uniquement** par `AppState::lecture()` et `AppState::trace()` ; en 32-3, tous deux sur `state.pool`.
- **`Rapport<T, U>`** : ce que rend un service de rapport. Le contenu `T` **ne s'obtient que** par
  `Rapport<T, Affichage>::afficher(self, &Trace) -> T` (trace de consultation) ou
  `Rapport<T, Export>::exporter(self, &Trace, rendre) -> Result<Vec<u8>, AppError>` (rendu, **puis** trace d'export).
  Un handler ne peut donc pas obtenir le rapport sans écrire la trace, ni tracer un export dont le rendu a échoué
  (C-32-3-3). Le `Rapport` porte le `user_id` de la trace (tiré du `ContexteAcces`) et les détails de la trace ; il ne
  porte **aucune** `Lecture`. Il expose **en lecture seule, sans donner le contenu**, de quoi nommer le fichier d'un
  export et le montrer à l'écran : la `ReportPeriod` **résolue par le service** (cinq rapports à période), les bornes
  `from`/`to` du grand livre, la date `as_of` de la balance âgée — **calculée une fois, dans le service**, et relue par
  le handler pour le nom de fichier, de sorte que contenu, trace et nom de fichier portent la même date même au passage
  de minuit UTC (C-32-3-23) —, le mode et les bornes d'un rapport de projet.

## Critères d'acceptation

### A — Le module et ses types

1. **AC1 — module `services`.** `crates/kesh-api/src/services/` neuf, déclaré `pub mod services;` dans `lib.rs` :
   `mod.rs` (doc-comment de module : **le motif écrit pour le rollout**, les règles R1 à R8 des Dev Notes, et ce qui
   reste à la 32-6 — la fabrication de `Lecture`), `acces.rs` (`ContexteAcces`, `Canal`, `Portee`, `Exigence`),
   `poignees.rs` (`Lecture`, `Trace`), `rapports.rs` (services des rapports, `Rapport<T, U>`, `Affichage`, `Export`,
   `FormatExport`, la trace des rapports, `identite_societe`), `plan_comptable.rs` (lecture du plan comptable).
   **Aucun type d'Axum** ni `AppState` dans le **code** de `services/` (C-32-3-14) — la règle porte sur les lignes de
   code, non sur la prose des doc-comments, qui nomment légitimement `AppState` (R1) : les lignes de
   `crates/kesh-api/src/services` qui contiennent `axum`, `AppState` ou `CurrentUser` **hors commentaires** (ligne dont
   le premier caractère non blanc ouvre `//`, doc-comments `///` et `//!` compris) sont **exactement deux**, toutes
   deux dans `acces.rs` : l'import `use crate::middleware::auth::CurrentUser;` et la signature de
   `ContexteAcces::depuis_api(u: &CurrentUser)`. Commande de contrôle : `grep -rnE "axum|AppState|CurrentUser"
   crates/kesh-api/src/services | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//'` → ces deux lignes ; la même règle est
   **exécutée** par le test de l'AC10 (vi) (C-32-3-20). Une ligne de code qui porte un commentaire **de fin de ligne**
   reste une ligne de code et compte (le jeton y est écrit en code ou à côté du code : rouge voulu, à ne pas « corriger »
   dans la garde). Un commentaire de bloc `/* … */` n'est **pas** retiré par la règle : il est lu comme du code et ne
   peut donc produire qu'un **faux rouge**, jamais masquer une ligne — d'où l'absence d'interdiction (C-32-3-30). Chaque
   élément public porte son doc-comment.
2. **AC2 — contrôle d'accès dans le service, en tête.** Chaque fonction de service publique commence par
   `acces.exiger(…)?` **avant toute lecture** — y compris avant la validation de la demande. Les exigences sont
   celles du tableau de l'AC6 ; pour le canal API, elles **répliquent** le RBAC actuel des routes (fait 8) : le
   comportement HTTP ne change pas, le middleware refusant toujours le premier.
3. **AC3 — poignées typées, fabriquées en un lieu.** `Lecture::new(pool)` et `Trace::new(pool)` publics (la 32-6 et les
   tests en ont besoin) ; `#[derive(Clone)]` sur les deux (un clone de pool, sans coût) ; champ de tuple **privé** sur
   les deux ; `Lecture::pool()` et `Trace::pool()` en `pub(in crate::services)`, seuls accesseurs de pool, seules
   méthodes avec `new` (l'inventaire de l'AC10 (v) les liste) — `Trace::pool()` n'étant appelé que par les fonctions
   d'audit de `services/rapports.rs` ; `AppState::lecture()` et `AppState::trace()` dans `lib.rs` (méthodes, **aucun champ ajouté** à
   `AppState`, fait 10), doc-comment **sans promesse sur Q11** : « la fabrication d'une `Lecture` en lecture seule
   (mode de session, utilisateur `SELECT`, ou les deux, et son lieu) est décidée par Guy (Q11) et réalisée par la
   32-6 ; en 32-3, les deux poignées enveloppent `pool` » (C-32-3-22). Aucune conversion `Lecture → Trace` ni
   `Trace → Lecture`, **où que ce soit** dans `src/`, `services/` compris : l'inventaire clos de l'AC10 (v) ne laisse
   `Lecture::new(`/`Trace::new(` qu'au câblage de `lib.rs` et aucun `impl` de trait sur les poignées (C-32-3-24).

### B — Les pilotes

4. **AC4 — neuf services de rapport.** `rapports::{bilan, compte_de_resultat, balance, journaux, tva, grand_livre,
   balance_agee, depenses_projet, rendement_projet}`, chacun générique sur l'usage (`Affichage` ou `Export(FormatExport)`),
   recevant `(&ContexteAcces, &Lecture, demande, usage)` et rendant `Result<Rapport<T, U>, AppError>`. La validation de
   la demande passe dans le service **dans l'ordre exact d'aujourd'hui** (`validate_fiscal_year_id` puis
   `ReportPeriod::resolve` ; `LedgerPeriod::new` puis `parse_ledger_account_ids` ; pour les projets, le corps de
   `resolve_project_report` l. 746-793 ; la date `as_of` de la balance âgée, `Utc::now().naive_utc().date()`, calculée
   dans le service **une fois**, et exposée par le `Rapport`, C-32-3-23). Les neuf services construisent leur `Rapport`
   par **un constructeur commun** (`Rapport::nouveau(contenu, details, user_id, usage)`, privé au module), et
   `afficher`/`exporter` écrivent la trace par **un aiguillage commun** (`tracer(&Trace, user_id, &DetailsTrace,
   usage)`, privé) qui appelle, selon (forme, usage), l'une des sept fonctions d'audit — c'est le point unique où la
   mutation M1 se pose (C-32-3-18). **Une aide de lecture en plus** : `rapports::identite_societe(&ContexteAcces,
   &Lecture) -> Result<(String, String), AppError>` (nom et `accounting_language` de la société du contexte, le `SELECT`
   de `load_pdf_context` l. 1229-1234 recopié avec son `map_db_error`), exigence `{ Consultation, None }` — `None` : le
   contexte PDF ne sert qu'aux exports, jamais au MCP ; `Consultation` : l'aide sert les **neuf** routes d'export, dont
   huit en Consultation et une en Comptable, et une aide partagée porte **le plus bas** des rôles de ses routes (règle
   R3), chaque route gardant le sien par son service et son middleware (C-32-3-27) ; la mise en forme (`PdfContext`,
   `map_language_to_bcp47`) reste au handler (C-32-3-17). Le grand livre garde ses deux régimes : affichage `offset` lu et `limit` à
   `Some(limit.unwrap_or(MAX_LEDGER_LIMIT))`, export `offset 0` et `limit None` (l. 295-301, 342-348). La validation
   du **format** d'export (`validate_format`, `validate_aged_export_format`) reste au handler, **en premier**, comme
   aujourd'hui. `ExportFormat` passe dans `services::rapports` sous le nom `FormatExport`, type **`pub`** (il paraît
   dans `Export(pub FormatExport)`), avec ses dérives d'aujourd'hui (`Debug, Clone, Copy, PartialEq, Eq` — `Debug` et
   `PartialEq` sont exigés par les `assert_eq!` du `mod tests`) et ses trois méthodes rendues **`pub`** (C-32-3-30), et
   `routes/reports.rs` l'importe sous son nom actuel (`use crate::services::rapports::FormatExport as ExportFormat;`),
   de sorte que le `mod tests` de `reports.rs` (l. 1587-1648) compile **sans modification** (C-32-3-11) ; ses deux
   méthodes de présentation (`content_type`, `extension`) le suivent, assumé (C-32-3-21). En export, `balance_agee`
   ne reçoit que `Export(FormatExport::Csv)` — le handler a rejeté tout autre format par `validate_aged_export_format`
   — et sa trace garde `"format": "csv"` **en littéral**, comme aujourd'hui (l. 636).
5. **AC5 — un service du plan comptable.** `plan_comptable::lister(&ContexteAcces, &Lecture, inclure_archives) ->
   Result<Vec<CompteDuPlan>, AppError>` (`CompteDuPlan { compte: Account, lettrable: bool }`), exigence
   `{ Consultation, Some(BooksRead) }`, pas de trace (aujourd'hui aucune). Il reprend `list_accounts` à l'identique :
   société lue d'abord (même erreur `Internal`, même message), puis `list_by_company`, puis
   `letterable_account_ids` sur une connexion acquise de la `Lecture`. Pour ne pas dupliquer le message, `helpers.rs`
   gagne `company_for_ids(pool, company_id, user_id)`, à laquelle `get_company_for` délègue (C-32-3-12). Le handler
   `list_accounts` construit `AccountResponse::new(compte, lettrable)` ; les handlers mutants ne changent pas.
6. **AC6 — handlers minces, comportement inchangé.** Chaque handler de lecture des pilotes appelle son service et ne
   fait plus que : extraire la requête, valider le format d'export, ouvrir le `span` d'export, charger le contexte PDF
   (`load_pdf_context(&ContexteAcces, &Lecture)`, privée au handler, qui lit par `rapports::identite_societe` puis met
   en forme — **au même rang qu'aujourd'hui**, après la génération : un échec n'y laisse aucune trace, le `Rapport`
   étant abandonné sans `exporter`), rendre, construire la réponse. Statuts, corps, en-têtes,
   ordre des erreurs, **action et `details_json` de chaque trace** : identiques. **Mesures du `span` d'export**
   (C-32-3-28) : `start` est pris à l'ouverture du `span`, comme aujourd'hui, et `byte_size` et `duration_ms`
   s'enregistrent **dans** la fermeture `rendre`, après le rendu — donc avant la trace, comme aujourd'hui
   (`reports.rs:487-493`) : `duration_ms` ne compte pas l'`INSERT` d'audit. Seul changement, assumé et sans effet sur
   l'API : la validation de la demande (`validate_fiscal_year_id`, aujourd'hui **avant** l'ouverture du `span`,
   l. 462, 1034) passe dans le service, appelé sous le `span` ; une erreur de validation naît donc dans le `span`, sans
   qu'aucun champ n'y soit enregistré (comme toute erreur postérieure aujourd'hui). Tableau des 18 + 1 chemins
   (ensemble clos, fait 1 et fait 7) :

   | Handler (l.) | Route | Service | Usage | Exigence API / MCP | Trace (fonction d'aujourd'hui, site) |
   |---|---|---|---|---|---|
   | `get_balance_sheet` (215) | `GET /reports/balance-sheet` | `bilan` | Affichage | Consultation / BooksRead | `emit_report_audit("balance-sheet")` (233) |
   | `get_income_statement` (248) | `…/income-statement` | `compte_de_resultat` | Affichage | Consultation / BooksRead | `emit_report_audit("income-statement")` (266) |
   | `get_trial_balance` (383) | `…/trial-balance` | `balance` | Affichage | Consultation / BooksRead | `emit_report_audit("trial-balance")` (401) |
   | `get_journal_report` (416) | `…/journals` | `journaux` | Affichage | Consultation / BooksRead | `emit_report_audit("journals", filtre)` (437) |
   | `get_vat_report` (1105) | `…/vat` | `tva` | Affichage | Consultation / BooksRead | `emit_report_audit("vat")` (1123) |
   | `get_general_ledger` (289) | `…/general-ledger` | `grand_livre` | Affichage | Consultation / BooksRead | `emit_ledger_audit` (306) |
   | `get_aged_receivables` (553) | `…/aged-receivables` | `balance_agee` | Affichage | Consultation / **InvoicesRead** | `emit_aged_receivables_audit` (559) |
   | `get_project_expenses` (796) | `…/project-expenses` | `depenses_projet` | Affichage | Consultation / BooksRead | `emit_project_report_audit("project-expenses")` (804) |
   | `get_project_return` (902) | `…/project-return` | `rendement_projet` | Affichage | Consultation / BooksRead | `emit_project_report_audit("project-return")` (910) |
   | `export_balance_sheet` (456) | `…/balance-sheet/export` | `bilan` | Export | Consultation / **jamais** | `emit_report_export_audit` (495) |
   | `export_income_statement` (654) | `…/income-statement/export` | `compte_de_resultat` | Export | Consultation / jamais | `emit_report_export_audit` (692) |
   | `export_trial_balance` (964) | `…/trial-balance/export` | `balance` | Export | Consultation / jamais | `emit_report_export_audit` (1002) |
   | `export_journal_report` (1028) | `…/journals/export` | `journaux` | Export | Consultation / jamais | `emit_report_export_audit` (1072) |
   | `export_vat_report` (1138) | `…/vat/export` | `tva` | Export | Consultation / jamais | `emit_report_export_audit` (1179) |
   | `export_general_ledger` (324) | `…/general-ledger/export` | `grand_livre` | Export | Consultation / jamais | `emit_ledger_audit` (362) — **`report.generated`, sans format** (fait 5, conservé) |
   | `export_aged_receivables` (565) | `…/aged-receivables/export` | `balance_agee` | Export | **Comptable** / jamais | `emit_aged_receivables_export_audit` (577) |
   | `export_project_expenses` (816) | `…/project-expenses/export` | `depenses_projet` | Export | Consultation / jamais | `emit_project_report_export_audit` (834) |
   | `export_project_return` (922) | `…/project-return/export` | `rendement_projet` | Export | Consultation / jamais | `emit_project_report_export_audit` (940) |
   | `list_accounts` (`accounts.rs:188`) | `GET /accounts` | `plan_comptable::lister` | — | Consultation / BooksRead | aucune |

   Les sept fonctions d'audit **quittent** `routes/reports.rs` pour `services/rapports.rs`, en **fonctions libres
   recevant `&Trace`** — et non en méthodes : un `impl Trace` hors de `poignees.rs` sortirait de l'inventaire clos de
   l'AC10 (v) (C-32-3-24) ; leurs corps — action, `entity_type`, `AUDIT_ENTITY_ID_NONE`, clés et valeurs du
   JSON, message et champs du `warn!` — sont **recopiés à l'identique** ; seule la source du pool change
   (`trace.pool().begin()` au lieu de `pool.begin()`, aucun type `sqlx` nommé). ⚠️ **L'action
   et le type d'entité restent des littéraux à chacun des sept sites `NewAuditLogEntry::user(`** : l'aiguillage
   `tracer` choisit **quelle fonction** appeler, jamais **quelle action** écrire. Une factorisation qui passerait
   l'action en variable ferait de `services/rapports.rs` un « site non résolu » pour
   `tests/audit_label_registry.rs` (`tout_site_dont_l_action_n_est_pas_litterale_est_inventorie`), que seule une
   entrée à `SITES_INDIRECTS` lèverait — modification d'un test existant, interdite par l'AC7 : **aucune entrée n'y
   est ajoutée** (C-32-3-19). Les doc-comments neufs de `services/` ne citent aucun compte sous la forme « NNNN Nom »
   (garde G4-ter de `tests/textes_coherents.rs`, qui lit toutes les sources `.rs`).

### C — Les tests

7. **AC7 — les tests existants restent verts, sans modification.** Aucun fichier de test existant n'est modifié,
   renommé ni supprimé ; le `mod tests` de `routes/reports.rs` non plus. Contrôle au dernier commit :
   `git diff --diff-filter=MDR --name-only $(git merge-base HEAD origin/main) HEAD -- crates/kesh-api/tests` → **vide**,
   et `diff <(git show $(git merge-base HEAD origin/main):crates/kesh-api/src/routes/reports.rs | sed -n
   '/^#\[cfg(test)\]/,$p') <(sed -n '/^#\[cfg(test)\]/,$p' crates/kesh-api/src/routes/reports.rs)` → **vide** (le
   bloc `#[cfg(test)] mod tests`, l. 1587 à la fin, identique octet pour octet). Fichiers qui doivent rester verts,
   relevé **large** du 2026-10-24 par `grep -rlE "/api/v1/reports|/api/v1/accounts" crates/kesh-api/tests` (**9**
   fichiers), **dont sept frappent une des 19 routes de l'AC6** (relu au site) : `accounts_e2e.rs` (`GET /accounts`),
   `opening_balances_e2e.rs` (`GET /reports/balance-sheet`, l. 357, 420), `open_items_e2e.rs` (`GET
   /accounts?includeArchived=true` et le champ `letterable`, l. 701 ; `GET /reports/general-ledger`, l. 581),
   `project_reports_e2e.rs`, `reports_e2e.rs`, `reports_export_e2e.rs`, `vat_report_e2e.rs` ; les deux autres ne
   frappent que des routes voisines — `idor_multi_tenant_e2e.rs` (`POST /accounts`, `PUT /accounts/{id}/archive`,
   l. 472, 491), `letterings_e2e.rs` (`/accounts/{id}/open-items`, l. 980). Attributs `sqlx::test` (`grep -c
   "sqlx::test"`) : `reports_e2e` 39, `opening_balances_e2e` 31, `reports_export_e2e` 21, `accounts_e2e` 19,
   `idor_multi_tenant_e2e` 14, `vat_report_e2e` 11, `letterings_e2e` 11, `project_reports_e2e` 8, `open_items_e2e` 7.
   Le développement relance ce `grep` au dernier commit et recopie la liste et les comptes **réellement exécutés** au
   Dev Agent Record. S'y ajoutent les **gardes de sources** qui balaient `crates/*/src` (ou `lib.rs`) et verront
   `services/` ou le test neuf — **huit**, relevé du 2026-10-24 par `grep -rlnE "include_str!|read_to_string|read_dir"
   crates/*/tests` (17 fichiers) puis lecture de chaque balayage, neuf écartés parce qu'ils lisent des fichiers de
   données ou un fichier nommé hors du périmètre (`admin_auto_backup_e2e`, `admin_full_export_e2e`,
   `admin_full_import_e2e`, `inbox_import_e2e`, `invoice_echeancier_e2e`, `onboarding_audit_e2e`,
   `reconciliation_e2e`, `supplier_settlement_cancel_e2e`, et `kesh-db/tests/lexique/mod.rs`, module de
   `letterings_lexical`) ; liste que le développement relance au dernier commit. Cinq que la story peut atteindre —
   `audit_label_registry.rs` (actions littérales, AC6), `textes_coherents.rs` (G4-ter),
   `admin_pat_denied_e2e.rs` (`the_admin_guards_have_no_alias_and_no_second_consumer_crate_wide`, l. 439, qui compte
   `require_admin_role`/`require_not_pat` dans tout `src/`), `audit_route_registry.rs` (lit `lib.rs`, que la story
   touche), `crates/kesh-db/tests/test_schema_guard.rs` (`every_sqlx_test_attribute_is_accounted_for`, balaie tout le
   workspace : chaque `#[sqlx::test(` de `services_motif.rs` tient **sur une ligne** et nomme
   `../kesh-db/test-schema`) — et trois hors d'atteinte du code prévu, lancées quand même : `configuration_transmise.rs` (lectures
   d'environnement, analyse `syn` de tout `crates/*/src` — un fichier de `services/` qui ne se parse pas y rougirait),
   `bank_account_closed_inventory.rs` (écritures SQL littérales sur sept tables bancaires) et
   `crates/kesh-db/tests/letterings_lexical.rs` (écritures littérales des marques de lettrage) : `services/` ne lit
   aucune variable d'environnement et n'écrit aucun SQL littéral hors `audit_log`.
8. **AC8 — un test de refus par service, et son contrôle positif.** Fichier neuf `crates/kesh-api/tests/services_motif.rs`
   (`#[sqlx::test(migrations = "../kesh-db/test-schema")]`, fixtures **synthétiques** :
   `kesh_db::test_fixtures::seed_accounting_company`, plus une facture ouverte et un projet posés en SQL pour la balance
   âgée et les rapports de projet). Pour **chacun des dix services** (neuf rapports, `lister`), une assertion nommée
   par cas — et pour l'aide `identite_societe` (AC4) : MCP avec toutes les portées → `Forbidden`, MCP avec toutes les
   portées sur une **société inexistante** → `Forbidden` (et non l'erreur de `fetch_one` : le refus précède la
   lecture), API `Consultation` → `Ok` (C-32-3-27) :
   - (a) canal **MCP sans la portée** exigée (portées vides, et — pour `balance_agee` — `{BooksRead}` seule ; pour les
     neuf autres — `{InvoicesRead}` seule) → `Err(AppError::Forbidden)` ;
   - (b) **contrôle positif** : canal MCP **avec** la portée exigée → `Ok` (sans quoi (a) pourrait rougir pour une autre
     raison) ;
   - (c) **le refus précède la validation et la lecture** : canal MCP sans portée, demande qui échouerait sans le
     contrôle → `Forbidden`, et non l'erreur attendue sans contrôle ; **deux cas par service** là où la demande porte
     les deux, une assertion nommée chacun (C-32-3-26) :
     - **(c-val), échec à la validation** — `fiscalYearId = 0` pour les cinq rapports à période (échec à
       `validate_fiscal_year_id`, `reports.rs:1334-1341`) ; période inversée (`from > to`) pour le grand livre (échec à
       `LedgerPeriod::new`) ; `projectId = 0` pour les deux rapports de projet (échec au contrôle `project_id <= 0`
       de `resolve_project_report`, l. 750-754) ;
     - **(c-lec), échec à la lecture** — `fiscalYearId` positif **inexistant** pour les cinq rapports à période
       (échec à `ReportPeriod::resolve`) ; `projectId` positif inexistant pour les deux rapports de projet (échec à
       `resolve_scope`) ; société inexistante pour `lister` (échec `Internal`) et pour `identite_societe` (échec de
       `fetch_one`) ; le grand livre ne lit, avant sa génération, aucune entité que nomme la demande : pas de cas
       (c-lec) pour lui, ce que le Dev Agent Record dit.
     Un `exiger` placé entre la validation et la lecture laisse (c-lec) vert et rougit (c-val) : c'est ce que tue
     M3′. **Sans objet pour `balance_agee`**, dont la demande ne porte rien qui puisse échouer : le contrôle y reste
     couvert par (a) et (b), et le Dev Agent Record le dit ;
   - (d) **exports jamais servis au MCP** : `Export` sur canal MCP **avec toutes les portées** → `Forbidden`, pour les
     neuf rapports ;
   - (e) **canal API** : `Export` de `balance_agee` avec le rôle `Consultation` → `Forbidden`, avec `Comptable` →
     `Ok` ; `Affichage` de chaque rapport et `lister` avec `Consultation` → `Ok`.
9. **AC9 — la trace s'écrit, et sur `Trace`.** Dans le même fichier, une `Lecture` fabriquée **en test** sur un pool
   dont chaque connexion passe `SET SESSION TRANSACTION READ ONLY` (`after_connect`, mêmes options de connexion que le
   pool du test ; aucun privilège requis, `docs/agents-orchestration.md` § 6.1) et une `Trace` sur le pool ordinaire
   du test :
   - (a) les **neuf** services de rapport, `lister` et `identite_societe` **réussissent** sur cette `Lecture` (en
     affichage et, pour les rapports, en export) — confirmation exécutée, **sur les branches que les fixtures
     exercent**, que leur chemin de lecture n'écrit rien (AC11). Fixtures : `seed_accounting_company` **retenu** — il
     pose déjà `company_invoice_settings` (TVA due sur le compte `2000`, impôt préalable sur le compte `1000`) et les
     taux de TVA (`8.10` compris) ; s'y ajoutent, en SQL ou par les dépôts, une facture **validée** et non réglée
     portant au moins une ligne au taux `8.10` (le rapport de TVA lit les `invoice_lines` des factures validées, taux
     par ligne ; la même facture rend la balance âgée non vide), une écriture débitant le compte d'impôt préalable
     `1000` (branche récupérable du rapport de TVA), et un projet portant une écriture imputée. S'inspirer de
     `seed_open_invoice_for` (`reports_e2e.rs:1562`) **sans** son `INSERT INTO company_invoice_settings` : la ligne
     existe déjà (clé `company_id`), le recopier heurterait la clé (C-32-3-30). Une branche de générateur que les
     fixtures n'atteignent pas est **nommée** au Dev Agent Record, non revendiquée ;
   - (b) **test tabulaire sur les dix-huit couples** (service, usage) — un par handler de l'AC6 —, une ligne de table
     par couple et une assertion nommée par ligne (le nom porte le service et l'usage) : après `afficher`/`exporter`,
     **exactement une** ligne neuve dans `audit_log` pour l'utilisateur, avec l'`action`, l'`entity_type` `"report"`,
     l'`entity_id` `0` et un `details_json` **égal, objet entier**, au littéral de la table — celui qu'écrit le code
     d'aujourd'hui (`routes/reports.rs:599-615, 626-645, 1372-1400, 1407-1435, 1463-1495, 1514-1538, 1548-1573`, et
     les valeurs passées aux 18 sites d'appel du tableau de l'AC6) (C-32-3-25). La table **est** la liste ci-dessous ;
     elle exerce un filtre de journal **non nul** (`journaux` en affichage), un filtre de comptes non nul (grand livre en
     affichage), le mode `cumulative` (rendement de projet, deux usages) et les deux formats. Notations : `fy` l'exercice
     de la demande ; `debut`/`fin` les bornes de la `ReportPeriod` **résolue** que le `Rapport` expose ; `du`/`au` les
     bornes de la demande du grand livre ; `compte` l'identifiant d'un compte seedé ; `pid` le projet ; `asof` la date
     qu'expose le `Rapport` ; dates en `"AAAA-MM-JJ"`.

     | # | Service × usage | `action` | `details_json` attendu |
     |---|---|---|---|
     | 1 | `bilan` × Affichage | `report.generated` | `{"report_type": "balance-sheet", "fiscal_year_id": fy, "period_start": debut, "period_end": fin, "journal_filter": null}` |
     | 2 | `bilan` × Export(Pdf) | `report.exported` | `{"report_type": "balance-sheet", "format": "pdf", "fiscal_year_id": fy, "period_start": debut, "period_end": fin, "journal_filter": null}` |
     | 3 | `compte_de_resultat` × Affichage | `report.generated` | `{"report_type": "income-statement", "fiscal_year_id": fy, "period_start": debut, "period_end": fin, "journal_filter": null}` |
     | 4 | `compte_de_resultat` × Export(Csv) | `report.exported` | `{"report_type": "income-statement", "format": "csv", "fiscal_year_id": fy, "period_start": debut, "period_end": fin, "journal_filter": null}` |
     | 5 | `balance` × Affichage | `report.generated` | `{"report_type": "trial-balance", "fiscal_year_id": fy, "period_start": debut, "period_end": fin, "journal_filter": null}` |
     | 6 | `balance` × Export(Pdf) | `report.exported` | `{"report_type": "trial-balance", "format": "pdf", "fiscal_year_id": fy, "period_start": debut, "period_end": fin, "journal_filter": null}` |
     | 7 | `journaux` × Affichage, filtre `Achats` | `report.generated` | `{"report_type": "journals", "fiscal_year_id": fy, "period_start": debut, "period_end": fin, "journal_filter": "Achats"}` |
     | 8 | `journaux` × Export(Csv), sans filtre | `report.exported` | `{"report_type": "journals", "format": "csv", "fiscal_year_id": fy, "period_start": debut, "period_end": fin, "journal_filter": null}` |
     | 9 | `tva` × Affichage | `report.generated` | `{"report_type": "vat", "fiscal_year_id": fy, "period_start": debut, "period_end": fin, "journal_filter": null}` |
     | 10 | `tva` × Export(Pdf) | `report.exported` | `{"report_type": "vat", "format": "pdf", "fiscal_year_id": fy, "period_start": debut, "period_end": fin, "journal_filter": null}` |
     | 11 | `grand_livre` × Affichage, filtre `[compte]` | `report.generated` | `{"report_type": "general-ledger", "period_start": du, "period_end": au, "account_ids": [compte]}` |
     | 12 | `grand_livre` × Export(Pdf), sans filtre | **`report.generated`** | `{"report_type": "general-ledger", "period_start": du, "period_end": au, "account_ids": null}` — **sans `format`**, comportement conservé (#729) : l'assertion se nomme `…_export_grand_livre_trace_en_consultation_729` et son commentaire renvoie à #729, que son correctif modifiera en connaissance de cause |
     | 13 | `balance_agee` × Affichage | `report.generated` | `{"report_type": "aged-receivables", "as_of": asof}` |
     | 14 | `balance_agee` × Export(Csv) | `report.exported` | `{"report_type": "aged-receivables", "format": "csv", "as_of": asof}` |
     | 15 | `depenses_projet` × Affichage, `fiscal_year` | `report.generated` | `{"report_type": "project-expenses", "project_id": pid, "mode": "fiscal_year"}` |
     | 16 | `depenses_projet` × Export(Pdf), `fiscal_year` | `report.exported` | `{"report_type": "project-expenses", "format": "pdf", "project_id": pid, "mode": "fiscal_year"}` |
     | 17 | `rendement_projet` × Affichage, `cumulative` | `report.generated` | `{"report_type": "project-return", "project_id": pid, "mode": "cumulative"}` |
     | 18 | `rendement_projet` × Export(Csv), `cumulative` | `report.exported` | `{"report_type": "project-return", "format": "csv", "project_id": pid, "mode": "cumulative"}` |

     Les clés `journal_filter` et `account_ids` sont **toujours présentes**, `null` sans filtre. Pour la balance âgée,
     `asof` est **celui qu'expose le `Rapport`**, lu avant `afficher`/`exporter` — le test ne recalcule pas la date, et
     ne peut donc pas franchir minuit UTC entre l'attendu et l'écrit ; une assertion de plus vérifie que cette date est
     celle du contenu (`AgedReceivables.as_of`) (C-32-3-23). La table est une **constante unique** du fichier de test,
     que (b′) relit ;
   - (b′) **la même table, par la route** : pour chacune des dix-huit lignes, la requête HTTP de la route du tableau de
     l'AC6 (routeur `kesh_api::build_router` sur `AppState::new_for_tests`, jeton JWT d'un utilisateur seedé, comme
     `reports_e2e.rs:54-80`, montage recopié dans `services_motif.rs`), puis **exactement une** ligne neuve dans
     `audit_log`, d'`action` et de `details_json` égaux à la ligne de la table — les demandes portent des bornes
     **explicites** dans l'exercice seedé, de sorte que `debut`/`fin` égalent la demande ; `asof` est accepté s'il égale
     la date UTC du jour lue **avant ou après** la requête (seul point où le test ne lit pas le `Rapport`). C'est le seul volet qui voie un **handler** passer par le mauvais usage (un export
     tracé `report.generated`, la forme de #729) ou le mauvais service (un `report_type` faux) (C-32-3-25) ;
   - (c) `exporter` avec un rendu qui échoue → `Err`, et **aucune** ligne d'audit (ordre du fait 4) ;
   - (d) un service qui échoue (période hors exercice) → aucune ligne (pendant de `reports_e2e.rs:1027`, ici au niveau
     du service).
   En (b), les bornes `debut`/`fin` sont lues sur le `Rapport` ; une demande sans bornes (résolues à celles de
   l'exercice) est employée pour au moins une ligne, afin que la résolution soit exercée.
10. **AC10 — garde lexicale de la frontière.** Dans `services_motif.rs`, un test qui lit les sources (comme
    `textes_coherents.rs` ou `audit_route_registry.rs`) et exige ce qui suit. **Tous les volets portent sur les lignes
    de code**, selon la définition de l'AC1 (est commentaire une ligne dont le premier caractère non blanc ouvre `//`,
    `///` et `//!` compris ; une ligne de code à commentaire de fin de ligne compte ; un `/* … */` est lu comme du code
    et ne peut produire qu'un faux rouge) :
    - (i) `crates/kesh-api/src/routes/reports.rs` ne contient plus ni `state.pool` ni `MySqlPool` ;
    - (ii) le corps de `list_accounts` (de `pub async fn list_accounts` au `pub async fn` suivant) ne contient pas
      `state.pool` ;
    - (iii)-(iv) *(remplacés en P2 par l'inventaire (v-d) ci-dessous, qui couvre aussi `services/`, C-32-3-24)* ;
    - (v) **inventaire clos des accès aux poignées** (C-32-3-24) — chacun de ces relevés est **égal**, comme ensemble de
      couples (fichier, ligne réduite de ses blancs de tête et de fin), à la liste écrite **ici** et recopiée en
      constante dans le test ; **toute ligne en plus rougit**, en nommant le fichier et la ligne ; la mise en forme
      exacte que rustfmt donne aux lignes listées est fixée au développement et recopiée au Dev Agent Record, sans
      qu'une ligne s'ajoute ni ne se retire :
      - **(v-a)** lignes de `crates/kesh-api/src/services/` (récursif) qui contiennent `MySql`, `Pool` ou `sqlx` (sous-
        chaînes, casse respectée : `MySqlPool`, `MySqlConnection`, `Pool<`, `PoolConnection`, `sqlx::` sont tous vus ;
        `lecture.pool()` ne l'est pas) — **huit** :
        1. `poignees.rs` : `use sqlx::MySqlPool;`
        2. `poignees.rs` : `pub struct Lecture(MySqlPool);`
        3. `poignees.rs` : `pub struct Trace(MySqlPool);`
        4. `poignees.rs` : `pub fn new(p: MySqlPool) -> Self {` (celui de `Lecture`)
        5. `poignees.rs` : `pub fn new(p: MySqlPool) -> Self {` (celui de `Trace`)
        6. `poignees.rs` : `pub(in crate::services) fn pool(&self) -> &MySqlPool {` (celui de `Lecture`)
        7. `poignees.rs` : `pub(in crate::services) fn pool(&self) -> &MySqlPool {` (celui de `Trace`)
        8. `rapports.rs` : `sqlx::query_as("SELECT name, accounting_language FROM companies WHERE id = ?")` (dans
           `identite_societe`, recopié de `load_pdf_context`, l. 1228)

        Les lignes 4-5 et 6-7 sont deux occurrences de la même chaîne : le relevé compare des **multiensembles**.
        Rougissent : `pub(crate)`, `pub(in crate)` ou `pub` sur `fn pool` (la ligne change) ; un accesseur d'un autre
        nom, où que ce soit dans `services/` (`pub fn brut(l: &Lecture) -> &MySqlPool`) ; un `type Target = MySqlPool;`
        ; une fonction d'audit qui garderait `pool: &MySqlPool` ;
      - **(v-b)** en-têtes de bloc `impl` de **tout** `crates/kesh-api/src/` (récursif) qui nomment `Lecture` ou
        `Trace` en mot entier — l'en-tête court du mot `impl` au premier `{`, **lignes jointes** (un `impl Deref\n for
        Lecture` replié par rustfmt est vu) — **deux** : `poignees.rs` : `impl Lecture {` et `impl Trace {`.
        Rougissent : `impl Deref for Lecture`, `impl AsRef<MySqlPool> for Trace`, `impl From<Lecture> for MySqlPool`,
        `impl Borrow…`, et un `impl Lecture` ou `impl Trace` inhérent posé dans un autre fichier (un `impl` inhérent
        peut vivre dans tout module du crate) ;
      - **(v-c)** lignes de `services/poignees.rs` qui contiennent `fn ` — **quatre** : les deux `new` et les deux
        `pool` de (v-a). Rougit : toute méthode ou fonction de plus dans `poignees.rs`, dont une conversion
        (`fn en_trace(&self) -> Trace`), qui seule pourrait y lire le champ privé ;
      - **(v-d)** lignes de **tout** `crates/kesh-api/src/` qui contiennent `Lecture::new(` ou `Trace::new(` —
        **deux**, au câblage de `lib.rs` : le corps de `AppState::lecture()` (`Lecture::new(self.pool.clone())`, chemin
        éventuel compris) et celui de `AppState::trace()` (`Trace::new(self.pool.clone())`). Remplace les anciens (iii)
        et (iv) et **couvre `services/`** : la conversion `Trace::new(lecture.pool().clone())`, que l'AC3 interdit et
        que M1 simule, y rougit. Ce volet décrit l'état de la 32-3, non un lieu imposé à la 32-6, qui l'amende en
        connaissance de cause si Q11 fait fabriquer la `Lecture` du MCP ailleurs (C-32-3-22) ;
      - **(v-e)** lignes de tout `crates/kesh-api/src/` qui contiennent `Lecture(` ou `Trace(` précédés d'une frontière
        de mot (`\b(Lecture|Trace)\(`) — **deux** : les définitions de (v-a) 2 et 3. Avec le champ privé, le
        compilateur refuse déjà ce constructeur hors de `poignees.rs`, que (v-c) ferme ; ce volet en fait un rouge
        nommé.

      **Ce que (v) ne voit pas, et qui reste à la revue** : une forme **non lexicale** — un alias de type défini hors
      de `services/` (`type P = sqlx::MySqlPool;` dans `helpers.rs`, puis `-> &crate::helpers::P` dans un service),
      ou du code engendré par une macro dont le texte ne nomme ni les poignées ni un type `sqlx`. La revue de code de
      la 32-3 et de chaque rollout lit donc les signatures publiques de `services/` ;
    - (vi) la règle de l'AC1 (lignes de code de `services/` portant `axum`, `AppState` ou `CurrentUser` : exactement
      les deux lignes attendues d'`acces.rs`) ;
    - (vii) **le bon usage à chaque handler** (C-32-3-25) : dans `routes/reports.rs`, le corps de chaque
      `pub async fn export_…` (de sa ligne d'ouverture à la première ligne égale à `}`, colonne 0) contient
      `.exporter(` et ne contient pas `.afficher(` ; celui de chaque `pub async fn get_…` contient `.afficher(` et ne
      contient pas `.exporter(` ; le test affirme **neuf** corps de chaque sorte (un chemin faux ou un renommage ne
      rend pas le volet vide).

    Chaque volet rend le fichier et la ligne fautifs. Contrôle positif écrit dans le test : chaque motif cherché est
    d'abord trouvé dans un texte témoin (dont, pour (v-b), un en-tête replié sur deux lignes), pour qu'une faute de
    frappe dans le motif ne rende pas la garde vide ; les balayages récursifs de `src/` affirment un nombre plancher
    de fichiers lus (comme `admin_pat_denied_e2e.rs`, l. 480-485), et celui de `services/` le nombre exact de ses
    cinq fichiers, pour qu'un chemin faux ne les rende pas vides.

### D — Inventaire, mutations, gates

11. **AC11 — inventaire clos des écritures sur les chemins de lecture des pilotes.** Ensemble clos, écrit au
    doc-comment de `services/mod.rs` et au Dev Agent Record : (1) les **sept** fonctions d'audit, **18** sites
    d'appel, toutes déplacées sur `Trace` (AC6) ; (2) `api_keys::touch_last_used` (`auth/api_key.rs:185`), écrite par
    le **middleware** sur `state.pool` pour une requête par PAT — **hors service**, inchangée ; une clé MCP aura sa
    propre validation, dont la 32-5 dira qu'elle écrit sa dernière utilisation sur `Trace` (spike § 4.4) ; (3) aucune
    autre : générateurs de `kesh-report`, `ReportPeriod::resolve`, `resolve_scope`, `identite_societe` (le `SELECT` de
    l'ancien `load_pdf_context`), `get_company_for`/`company_for_ids`, `list_by_company`, `letterable_account_ids` ne
    font que lire — **établi par la lecture statique** (fait 3 ; les dépôts appelés relus : `fiscal_years`, `projects`,
    `accounts`, `letterings`, `companies`, sans écriture ni verrou, et `invoice_settlements`, dont `kesh-report` ne prend
    que des fragments SQL et une analyse pure — `INVOICE_AMOUNT_DUE_DERIVED_SQL`, `amount_due_derived_joins`
    (`aged_receivables.rs:31`), `parse_write_off_vat` (`vat_report.rs:305`) —, aucun accès à la base) **et confirmé, sur les branches que les fixtures
    exercent, par l'AC9 (a)**, qui les exécute sur une connexion en lecture seule. La mise en forme du contexte PDF
    (`load_pdf_context` au handler) ne lit plus rien elle-même. Les `get_or_create_default` du § 4.4 de la spike ne
    sont sur aucun chemin de ces pilotes : `grep -rn "get_or_create" crates/kesh-api/src/routes/reports.rs
    crates/kesh-api/src/routes/accounts.rs crates/kesh-api/src/services crates/kesh-api/src/helpers.rs` → rien (rien
    aujourd'hui sur les fichiers existants ; à relancer au dernier commit, `services/` compris).
12. **AC12 — mutations qui doivent rougir** (une à la fois, restauration `git checkout -- <f> && touch <f>`, gate
    ciblé `cargo nextest run -E 'binary(services_motif)'`, plus le binaire nommé) ; chacune doit rougir **au moins une
    assertion nommée** :
    - **M1 — la trace part sur `Lecture`** (C-32-3-18). En production, le `Rapport` ne porte aucune `Lecture` : la
      mutation exige un **échafaudage nommé**, posé avec elle et retiré à la restauration (`git checkout -- <f>`) —
      un champ `lecture_m1: Lecture` ajouté à `Rapport`, rempli par le constructeur commun `Rapport::nouveau` à partir
      de la `Lecture` que reçoit le service (un paramètre de plus au constructeur, `lecture.clone()` aux neuf appels).
      **Mutation** : aux deux appels de l'aiguillage commun `tracer` (dans `afficher` et dans `exporter`), passer
      `&Trace::new(self.lecture_m1.pool().clone())` au lieu de la `Trace` reçue → **les dix-huit lignes de l'AC9 (b)
      rougissent** (l'`INSERT` échoue en 1792 sur la `Lecture` de test, avalé par le `warn!` : la ligne manque). L'AC10
      (v-d) rougit aussi (un `Trace::new(` hors du câblage) : attendu, c'est la conversion que l'AC3 interdit ; le Dev
      Agent Record consigne **les deux** rouges, celui de l'AC9 (b) étant la preuve à l'exécution. L'AC9 (b′) reste
      **vert** sous M1 — par la route, la `Lecture` est le pool ordinaire et l'`INSERT` réussit —, ce qui est consigné.
      **M1′** : même échafaudage, substitution au seul appel de `exporter`, gardée par
      `matches!(details, DetailsTrace::GrandLivre { .. })` → la seule ligne 12 de l'AC9 (b) (`…_729`) rougit, les
      dix-sept autres restant vertes. Le diff de l'échafaudage et de la mutation est recopié au Dev Agent Record.
      ⚠️ **Ce que M1 ne couvre pas, et pourquoi** : la mutation « `AppState::trace()` enveloppe la `Lecture` » n'est
      **pas exprimable** en 32-3 — `lib.rs` est hors de `crate::services` et ne peut pas lire `lecture.pool()`, si bien
      que la seule écriture possible de `trace()` est `Trace::new(self.pool.clone())`, c'est-à-dire le code lui-même.
      Elle le deviendra quand la 32-6 donnera à la `Lecture` un pool distinct ; la 32-6 la pose alors (C-32-3-30). Le
      lieu de fabrication est tenu par l'AC10 (v-d). **M1-témoin** (contrôle du test, non une mutation de production) :
      bâtir dans le test la `Trace` sur le pool en lecture seule → les dix-huit lignes de l'AC9 (b) rougissent ;
      consigné pour montrer que l'AC9 (b) voit une trace perdue.
    - **M2 — contrôle d'accès retiré du service** : supprimer l'`acces.exiger(…)?` de `rapports::bilan` → AC8 (a), (c),
      (d) de `bilan` rougissent ; **M2′** : le même retrait dans `plan_comptable::lister` → AC8 (a), (c-lec) de `lister` ;
      **M2″** : le même retrait dans `rapports::identite_societe` → ses deux assertions MCP de l'AC8 rougissent
      (C-32-3-27).
    - **M3 — contrôle d'accès déplacé après la lecture** (dans `rapports::journaux`) → AC8 (c-val) et (c-lec) de
      `journaux` rougissent, (a) restant vert : c'est ce qui prouve que (c) mesure l'ordre.
    - **M3′ — contrôle d'accès placé après la validation** (C-32-3-26) : dans `rapports::bilan`, `exiger` déplacé
      entre `validate_fiscal_year_id` et `ReportPeriod::resolve` → AC8 (c-val) de `bilan` rougit (`Validation` au lieu
      de `Forbidden`), (c-lec) de `bilan` restant vert : c'est ce qui prouve que (c-val) mesure le refus **avant la
      validation**, et non seulement avant la lecture.
    - **M4 — trace supprimée** : retirer l'`insert_in_tx` du chemin « balance âgée, export » → la ligne 14 de l'AC9 (b)
      et de l'AC9 (b′).
    - **M5 — mauvaise portée** : `balance_agee` exige `BooksRead` → AC8 (a) avec `{BooksRead}` seule rougit.
    - **M6 — export servi au MCP** : l'exigence d'export porte `Some(BooksRead)` → AC8 (d).
    - **M7 — rôle API perdu** : l'export de `balance_agee` exige `Consultation` → AC8 (e) rougit, alors que
      `aged_receivables_export_forbidden_for_consultation` (`reports_e2e.rs:1663`) reste **vert** (le middleware refuse
      d'abord) — à consigner : c'est la défense en profondeur que le test du service est seul à voir.
    - **M8 — handler revenu au pool** : dans `get_balance_sheet`, appeler `generate_balance_sheet(&state.pool, …)` →
      AC10 (i).
    - **M9 — trace avant rendu** : dans `exporter`, tracer avant `rendre` → AC9 (c).
    - **M10 — clé de détail changée** : renommer `fiscal_year_id` en `fiscalYearId` dans la trace de période →
      `report_generated_audit_emitted_on_success` (`reports_e2e.rs:955`) **et** les lignes 1 à 10 de l'AC9 (b) et (b′)
      rougissent.
    - **M11 — pool rendu atteignable des handlers** : `Lecture::pool()` en `pub(crate)` → AC10 (v-a) rougit (ligne 6
      changée ; le compilateur, lui, ne dit rien : l'élargissement compile).
    - **M11′ — `impl Deref for Lecture`** (`type Target = MySqlPool;`, posé dans `poignees.rs`) → AC10 (v-b) rougit (un
      en-tête d'`impl` de plus) et (v-a) aussi (la ligne `type Target`) (C-32-3-24).
    - **M11″ — accesseur d'un autre nom dans un autre fichier** : dans `services/plan_comptable.rs`,
      `pub fn brut(l: &Lecture) -> &MySqlPool { l.pool() }` → AC10 (v-a) rougit (une ligne de `plan_comptable.rs` en
      plus), alors que le compilateur l'accepte et que l'ancienne garde, limitée à `poignees.rs` et à trois chaînes, ne
      la voyait pas.
    - **M12 — `as_of` recalculé** : dans la **branche balance âgée de l'aiguillage `tracer`**, `Utc::now().naive_utc()
      .date()` passé à la fonction d'audit au lieu du `as_of` du `DetailsTrace` → mutation **équivalente hors minuit**,
      déclarée ; l'AC9 (b) la verrait seulement au passage de minuit UTC. Elle est consignée pour dire que la règle
      « calculée une fois » (C-32-3-23) tient par la **structure** — une seule variable `as_of`, calculée une fois dans
      le service, dont le contenu et le `DetailsTrace` reçoivent la valeur —, non par le test.
    - **M13 — export passé par `Affichage`** (C-32-3-25) : dans `export_trial_balance`, appeler `rapports::balance`
      avec `Affichage`, obtenir le contenu par `afficher(&trace)` et le rendre au handler → AC10 (vii) rougit (corps
      d'`export_` sans `.exporter(`) **et** la ligne 6 de l'AC9 (b′) rougit (`report.generated` au lieu de
      `report.exported`, sans `format`) ; l'AC9 (b), qui choisit l'usage lui-même, reste vert — consigné : c'est
      pourquoi (b′) existe.
    Une mutation qui ne rougit rien est **déclarée** au Dev Agent Record avec la raison (équivalente, ou test manquant
    ajouté puis mutation rejouée).
13. **AC13 — gates au dernier commit de code.** `kesh-db` n'est **pas** touché (aucun dépôt, aucune migration,
    `post_restore.rs` non plus) : en boucle de revue, gate ciblé (`fmt` et `clippy` sur le workspace, `nextest` sur
    `services_motif`, les neuf binaires de l'AC7 — `reports_e2e`, `reports_export_e2e`, `vat_report_e2e`,
    `project_reports_e2e`, `accounts_e2e`, `opening_balances_e2e`, `open_items_e2e`, `idor_multi_tenant_e2e`,
    `letterings_e2e` —, les huit gardes de sources de l'AC7 — `audit_label_registry`, `textes_coherents`,
    `admin_pat_denied_e2e`, `audit_route_registry`, `test_schema_guard`, `configuration_transmise`,
    `bank_account_closed_inventory`, `letterings_lexical` —, et la lib de `kesh-api`) ; au **dernier commit de code** : **gate complet** (`scripts/test-fast.sh` sous verrou, base
    remise à zéro d'abord) **et E2E Playwright complet** (recette de `docs/testing.md`, jugé fichier par fichier contre
    § *Les échecs attendus*), les quatre gates frontend inclus (aucun fichier frontend ne change, mais l'E2E les exige
    par la règle du dernier commit). Ne déclarer que ce qui a tourné, avec les chiffres.
14. **AC14 — documentation.** Aucun changement visible : ni manuel, ni `docs/api-external.md`, ni CHANGELOG, ni
    README (C-32-3-13) — le contrôle se fait quand même (une passe vérifie qu'aucun texte ne décrit ce qui bouge).
    Aucun écran ne change : la recette visuelle est sans objet. L'écart antérieur du manuel sur le rôle Consultation
    et les clés en lecture seule (« aucune mutation », « une lecture n'écrit rien ») n'est pas corrigé ici : **#730**
    (*Hors périmètre*). Les
    doc-comments de module de `routes/reports.rs` (l. 1-17 : « Pattern audit best-effort ») et de `routes/accounts.rs`
    renvoient à `services`. Le motif est écrit **une fois**, au doc-comment de `services/mod.rs`.
15. **AC15 — indépendance de Q11, écrite.** Le doc-comment de `poignees.rs` dit, en une phrase, que la fabrication
    d'une `Lecture` en lecture seule — mode de session, utilisateur `SELECT` ou les deux — **est la 32-6**, selon la
    décision de Guy sur Q11, et que rien en 32-3 n'en préjuge. `grep -rniE "transaction read only|tx_read_only"
    crates/kesh-api/src` → **vide** (vide aussi au relevé du 2026-10-24 ; seul le test de l'AC9 emploie le mode de
    session, dans `tests/`). *(Le motif `read_only` seul serait faux : il attrape `API_KEY_READ_ONLY`,
    `middleware/auth.rs:139`, `:207`, `errors.rs:189`.)*

## Tâches

- [ ] **T0** — Worktree `kesh-32-3`, branche `story/32-3` (déjà posés) ; bases `kesh_323`, `kesh_e2e_323` créées,
  migrées depuis les migrations du worktree et seedées ; port attribué par l'orchestrateur ;
  `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-32-3/target` (`docs/agents-orchestration.md` § 4).
- [ ] **T1 (AC1-AC3, AC15)** — `services/{mod,acces,poignees}.rs` ; `pub mod services;` ; `AppState::lecture()` et
  `trace()` ; doc-comments.
- [ ] **T2 (AC8, AC9, AC10 — rouge d'abord)** — `tests/services_motif.rs` : fixtures synthétiques, `Lecture` de test
  en lecture seule, assertions nommées ; constater qu'il ne compile pas ou rougit avant T3-T4.
- [ ] **T3 (AC4, AC6, AC11)** — `services/rapports.rs` : `FormatExport`, `Affichage`, `Export`, `Rapport<T, U>`, les
  neuf services, `identite_societe`, `Rapport::nouveau`, l'aiguillage `tracer`, la trace (les sept corps recopiés, actions
  littérales) ; `routes/reports.rs` : handlers minces, `load_pdf_context(&ContexteAcces, &Lecture)` (mise en forme
  seule), import `FormatExport as ExportFormat` ; gate ciblé.
- [ ] **T4 (AC5)** — `helpers.rs::company_for_ids` (et délégation de `get_company_for`) ; `services/plan_comptable.rs` ;
  `list_accounts` mince ; gate ciblé.
- [ ] **T5 (AC12)** — mutations M1 à M13 (M1′, M2′, M2″, M3′, M11′, M11″) et M1-témoin, chacune tuée ou déclarée
  (M1 en échafaudage, M12 équivalente hors minuit).
- [ ] **T6 (AC7, AC13, AC14)** — contrôles de l'AC7, gate complet et E2E complet au dernier commit de code ; Dev Agent
  Record : chiffres réellement exécutés, liste exacte des fichiers de l'AC7, inventaire de l'AC11.

## Dev Notes

### Les règles du motif (à recopier au doc-comment de `services/mod.rs`, pour la 32-4)

- **R1** — Un service ne reçoit ni `AppState`, ni type d'Axum : `(&ContexteAcces, &Lecture, demande[, usage])`, plus
  `&Trace` là où il trace. Le MCP l'appellera tel quel.
- **R2** — Première instruction : `acces.exiger(EXIGENCE)?` — avant la validation, avant toute lecture.
- **R3** — Une `Exigence` par opération, constante nommée à côté du service ; pour l'API, le rôle **égal** au RBAC de la
  route (le middleware reste ; le service ne l'assouplit jamais) — et, pour une aide servie par **plusieurs** routes de
  RBAC différents, **le plus bas** de leurs rôles, chaque route gardant le sien par son propre service et son
  middleware (`identite_societe`, C-32-3-27) ; pour le MCP, une portée, ou `None`.
- **R4** — Toute lecture passe par la `Lecture` ; **aucune écriture** sur un chemin de lecture : ce qui doit écrire
  (trace, dernière utilisation) passe par la `Trace`. Le pool d'une poignée ne sort pas de `services`
  (`pub(in crate::services)`, champ privé) ; aucun autre chemin vers lui n'est ouvert — l'inventaire clos de
  l'AC10 (v) le contrôle (C-32-3-24) ; ce qu'un handler doit lire pour sa présentation (le contexte PDF) passe par une
  fonction de service. L'action d'audit reste un **littéral** à chaque `NewAuditLogEntry::user(` (garde
  `audit_label_registry`).
- **R5** — Une trace qu'un handler pourrait oublier est une trace muette : le contenu ne se rend qu'à travers la méthode
  qui trace (`Rapport::afficher`/`exporter`). Une lecture sans trace (le plan comptable) rend directement.
- **R6** — La validation de la demande vit dans le service, **dans l'ordre d'aujourd'hui** ; la présentation (format
  d'export, rendu, en-têtes, nom de fichier) reste au handler.
- **R7** — Chaque service a son test de refus (portée absente, canal refusé, refus avant la validation **et** avant la
  lecture) **et** son contrôle positif ; un test qui l'exécute sur une `Lecture` en lecture seule ; et, s'il trace, sa
  ligne dans la table des traces, au service et par la route.
- **R8** — La garde lexicale (AC10) s'étend aux handlers migrés ; ses inventaires clos (v) s'amendent en connaissance
  de cause, ligne par ligne, jamais par un motif élargi.

### Sites

- `crates/kesh-api/src/routes/reports.rs` — 18 handlers (AC6) ; sept fonctions d'audit (l. 599, 626, 1372, 1407,
  1463, 1514, 1548) ; `load_pdf_context` (l. 1223) ; `resolve_project_report` (l. 746) ; `validate_fiscal_year_id`
  (l. 1334) ; `parse_ledger_account_ids` (l. 112) ; `ExportFormat` (l. 170-194), `validate_format` (l. 200) ;
  `mod tests` (l. 1587-1648, intouchable). Les `use kesh_report::{…}` (l. 32-44) se répartissent entre le handler
  (rendu) et le service (génération).
- `crates/kesh-api/src/routes/accounts.rs` — `list_accounts` (l. 188-212) seul ; `response()` (l. 179) et les
  mutations inchangés.
- `crates/kesh-api/src/helpers.rs` — `get_company_for` (l. 66-78).
- `crates/kesh-api/src/lib.rs` — `pub mod` (l. 9-26 ; ⚠️ `use tower_http::services::{ServeDir, ServeFile}` l. 35 ne
  heurte pas `crate::services`, chemins distincts) ; `impl AppState` (l. 98).
- `crates/kesh-api/src/middleware/rbac.rs:19-28` — la règle `role < min_role` (dans `check_role`, privée, qui prend une
  `Request`), que `ContexteAcces::exiger` reprend par la même comparaison sur `Role` : **une seule relation d'ordre**
  (`Ord` par `Role::level`, `kesh-db/src/entities/user.rs:42-60`), **deux sites de comparaison**, tenus d'accord par
  l'AC8 (e) et la mutation M7.
- `crates/kesh-api/tests/reports_e2e.rs` — `setup_full` (l. 302) et `seed_open_invoice_for` (l. 1562) montrent comment
  poser exercice, comptes, écritures, facture ouverte ; **ne pas les modifier** : s'en inspirer dans
  `services_motif.rs`, ou employer `kesh_db::test_fixtures::seed_accounting_company`.

### Esquisse (indicative ; les noms et les règles R1-R8 sont exigés, le détail est au développement)

```rust
// services/acces.rs
pub enum Portee { BooksRead, InvoicesRead }
pub enum Canal { Api { role: Role }, Mcp { portees: BTreeSet<Portee> } }
pub struct ContexteAcces { company_id: i64, user_id: i64, canal: Canal }   // champs privés (C-32-3-29)
pub struct Exigence { pub role_api: Role, pub portee_mcp: Option<Portee> }
impl ContexteAcces {
    pub fn depuis_api(u: &CurrentUser) -> Self;
    pub fn mcp(company_id: i64, user_id: i64, portees: BTreeSet<Portee>) -> Self;
    pub fn company_id(&self) -> i64;  pub fn user_id(&self) -> i64;  pub fn canal(&self) -> &Canal;
    pub fn exiger(&self, e: Exigence) -> Result<(), AppError>; // Err(AppError::Forbidden)
}
// services/poignees.rs — les huit lignes de l'AC10 (v-a) 1-7 et les quatre `fn` de (v-c), rien d'autre
#[derive(Clone)] pub struct Lecture(MySqlPool);                      // champ privé
impl Lecture { pub fn new(p: MySqlPool) -> Self; pub(in crate::services) fn pool(&self) -> &MySqlPool; }
#[derive(Clone)] pub struct Trace(MySqlPool);                        // champ privé
impl Trace   { pub fn new(p: MySqlPool) -> Self; pub(in crate::services) fn pool(&self) -> &MySqlPool; }
// (pool() de Trace : appelé par les seules fonctions d'audit, libres, de services/rapports.rs)
// services/rapports.rs
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum FormatExport { Pdf, Csv }  // as_str, content_type, extension : pub
pub struct Affichage;  pub struct Export(pub FormatExport);
pub struct Rapport<T, U> { contenu: T, details: DetailsTrace, user_id: i64, usage: U }  // champs privés, aucune Lecture
impl<T, U> Rapport<T, U> { fn nouveau(contenu: T, details: DetailsTrace, user_id: i64, usage: U) -> Self; }
// + accesseurs en lecture seule : période résolue, bornes, as_of, mode (C-32-3-23)
async fn tracer(t: &Trace, user_id: i64, d: &DetailsTrace, usage: UsageTrace); // aiguillage vers les sept fonctions
pub async fn identite_societe(a: &ContexteAcces, l: &Lecture) -> Result<(String, String), AppError>;
impl<T> Rapport<T, Affichage> { pub async fn afficher(self, t: &Trace) -> T; }
impl<T> Rapport<T, Export> {
    pub async fn exporter(self, t: &Trace, rendre: impl FnOnce(&T) -> Result<Vec<u8>, AppError>)
        -> Result<Vec<u8>, AppError>;   // rendre, puis tracer si Ok
}
pub async fn bilan<U: Usage>(a: &ContexteAcces, l: &Lecture, d: DemandePeriode, u: U)
    -> Result<Rapport<BalanceSheet, U>, AppError>;
```

`DetailsTrace` énumère les quatre formes (période avec type et filtre de journal ; grand livre avec bornes et comptes ;
balance âgée avec `as_of` ; projet avec type, projet et mode) ; `afficher` et `exporter` choisissent la fonction d'audit
d'aujourd'hui selon (forme, usage) — huit chemins, l'export du grand livre gardant `emit_ledger_audit`. Le refus
`Forbidden` vaut pour les deux canaux ; la 32-6 le traduira en « outil inconnu » côté MCP (aucun oracle, C-32-3-5).

### Hors périmètre, et ce qui reste non résolu (inventaire)

- **La fabrication d'une `Lecture` en lecture seule** — décidée par Guy (Q11), réalisée par la 32-6. En 32-3, le
  compilateur refuse `lecture.pool()` hors de `services` (`pub(in crate::services)`, champ privé). L'AC10 (v)
  **attrape** — par inventaire clos des lignes, non par énumération de formes — tout chemin **lexical** de plus vers
  le pool d'une poignée : une ligne de `services/` qui nomme un type `sqlx` hors des huit attendues (accesseur d'un
  autre nom, visibilité élargie), un `impl` sur une poignée hors des deux attendus (`Deref`, `AsRef`, `From`, `impl`
  inhérent ailleurs), une fonction de plus dans `poignees.rs`, un `Lecture::new(`/`Trace::new(` hors du câblage de
  `lib.rs`. **Reste à la revue** : une forme non lexicale — alias de type défini hors de `services/`, code engendré par
  macro (AC10, *Ce que (v) ne voit pas*). Reste **non observable** en 32-3 : un handler qui écrirait par un autre pool
  (`state.pool`) — fermé dans `routes/reports.rs` et `list_accounts` par l'AC10 (i)-(ii), **pas** dans les modules que
  la 32-4 n'a pas encore migrés ; et une `AppState::trace()` qui envelopperait le pool de la `Lecture` — **non
  exprimable** en 32-3 (`lib.rs` ne lit pas `lecture.pool()`), à poser par la 32-6 (AC12, M1). Au niveau du service,
  l'AC9.
- **L'export du grand livre tracé en consultation** (fait 5) — conservé ; **#729** (C-32-3-8).
- **Le manuel d'administration dit « sans écriture » ce qui écrit** — **#730** (`bug`, `documentation`, P3, ouverte par
  l'orchestrateur et complétée en P2). Deux sites : `admin-manual.tex:1464` (rôle Consultation : « aucune mutation, à
  une exception près », le premier téléchargement du PDF d'une facture) et `admin-manual.tex:2897` (clés d'API en
  lecture seule : « une lecture n'écrit rien, à une exception près »), le PDF disant la même chose. Écritures que ces
  phrases omettent : chaque consultation ou export de rapport (`report.generated`/`report.exported`, fait 1), l'export
  global (`exports.global`, `routes/exports.rs:131`, route tous rôles `lib.rs:1134`), et, pour une requête par clé,
  `api_keys::touch_last_used` (`auth/api_key.rs:185`, AC11 (2)). Écart **antérieur** à la story, qu'elle ne déplace
  pas (AC14) ; le manuel MCP de la 32-7 (clés en lecture seule) devra en tenir compte.
- **L2** (`actor_type='user'` via PAT, fait 6) — conservé ; l'acteur d'une clé MCP est à la 32-5/32-6.
- **Les autres modules de lecture** (écritures, postes ouverts, contacts, factures) — 32-4a/32-4b.
- **`touch_last_used`** — middleware, hors service (AC11 (2)).
- **Un éventuel champ ajouté à `AppState`** par la 32-6 (selon la forme que Q11 retiendra) toucherait les tests qui
  construisent `AppState` par littéral (fait 10) : à prévoir par la 32-6, signalé ici.

### Fichiers touchés (prévus)

`crates/kesh-api/src/services/{mod,acces,poignees,rapports,plan_comptable}.rs` (neufs) ;
`crates/kesh-api/src/lib.rs` (déclaration, deux méthodes) ; `crates/kesh-api/src/routes/reports.rs` ;
`crates/kesh-api/src/routes/accounts.rs` ; `crates/kesh-api/src/helpers.rs` ;
`crates/kesh-api/tests/services_motif.rs` (neuf). **Non touchés** : `kesh-report`, `kesh-db`, frontend, i18n, manuels.

**Découpage préventif** : modules de premier niveau touchés — `kesh-api/services` (neuf), `kesh-api/routes/reports`,
`kesh-api/routes/accounts`, `kesh-api/helpers` (une fonction extraite) — **4** ; `lib.rs` (câblage) et
`tests/services_motif.rs` (tests) en plus. Sous le seuil de 5 ; `kesh-report` n'a pas à changer (fait 3). Aucune
migration.

### Risques

- **Moyen — une trace perdue est muette.** Dix-sept des dix-huit handlers n'ont aujourd'hui aucun test qui lise leur
  trace (fait 9) : l'AC9 (b) fige les **dix-huit** couples (service, usage) au niveau du service, et l'AC9 (b′) les
  mêmes dix-huit par la route, avant le déplacement des sept fonctions. Écrire les tests (T2) **avant** de déplacer
  (T3).
- **Ordre des erreurs** : passer la validation dans le service ne doit pas changer quelle erreur sort quand plusieurs
  s'appliqueraient (format invalide **et** exercice nul : 400 sur le format d'abord, comme aujourd'hui). Les tests
  existants d'erreur (`reports_export_e2e.rs`) le voient en partie ; le développement relit l'ordre handler par handler.
- **Grand livre** : deux régimes de pagination (AC4) — une borne perdue rendrait un export tronqué, sans valeur
  probante (doc-comment l. 318-323).
- **Typestate** : si `Rapport<T, U>` devient lourd pour les rapports de projet (bornes de nom de fichier), garder la
  règle R5 — le contenu ne sort que par une méthode qui trace — quelle que soit la forme retenue, et l'écrire au
  registre.

### Références

`CLAUDE.md` § *Test Locally First*, § *Pendant une boucle de revue*, § *Règle de splitting préventif*, § *Inventorier
les sites NON RÉSOLUS*, § *Pattern batch* (non applicable : aucun lot) ; `docs/agents-orchestration.md` § 2-7 ;
`planning-artifacts/epic-32-serveur-mcp.md` § *32-3*, § *Ce que le code dit déjà* (prémisses 5, 6, 7), § *Points
d'attention* ; `planning-artifacts/epic-32-spike-mcp.md` § 4.3, § 4.4, § 5.3.1, § 6 ; registres
`epic-32-choix-autonomes.md` (C-32-0-4, C-32-0-7) et `-322.md` (C-32-2-16, C-32-2-24).

## Dev Agent Record

### Agent Model Used

### Completion Notes List

### File List

## Change Log

- **2026-10-24 — Rédaction** (Opus 5.5, agent de fiche, en autonomie) : fiche, registre `epic-32-choix-autonomes-323.md`
  (C-32-3-1 à 16), clé `32-3-services-motif: ready-for-dev`, prompt de validation P1
  (`323-validate-prompt-p1.md`, Sonnet, lentilles R et F). Issue demandée à l'orchestrateur : l'export du grand livre
  tracé en `report.generated` sans format (C-32-3-8).
- **2026-10-24 — Validation P1** (Sonnet, deux lentilles en contexte frais ; rapports `kesh-gate-logs/323-validate-p1-R.md`
  et `-F.md`) : **R 0/0/5/8, F 0/0/3/10** (CRITICAL/HIGH/MEDIUM/LOW). Recoupements : R1 = F1, R3 = F3, R12 ≈ F2 (R en
  LOW, F en MEDIUM : retenu MEDIUM), R2 ≈ F11, R4 ≈ F9, R6 = F4 (+ F5), R7 = F7, R8 ≈ F8, R10 = F10, R13 = F6 — soit
  **six défauts MEDIUM distincts** (AC1, M1, `Lecture::pool()`, `load_pdf_context`, gardes de sources, Q11). Remédiation
  (registre C-32-3-17 à C-32-3-23) :
  - **R1/F1** — la garde de l'AC1 porte sur les lignes de **code** hors commentaires, deux lignes attendues nommées,
    commande donnée, et exécutée par le test (AC10 (vi)) (C-32-3-20).
  - **R3/F3** — M1 par **échafaudage nommé** (`lecture_m1` dans le `Rapport`, posé par le constructeur commun) et
    substitution aux deux appels de l'aiguillage `tracer` ; M1′ sur la seule branche grand livre × export ; la mutation
    de `AppState::trace()` déclarée équivalente en 32-3 ; M1-témoin (C-32-3-18).
  - **F2/R12** — `Lecture::pool()` en `pub(in crate::services)` ; AC10 (iii) étendu à tout `src/` hors `services/` et
    `lib.rs`, AC10 (v) neuf contre l'élargissement, M11 ; phrase du *Hors périmètre* réécrite (elle affirmait une
    fermeture que l'AC10 (iii) ne faisait pas) (C-32-3-17).
  - **R2/F11** — la lecture de `load_pdf_context` passe en service (`rapports::identite_societe`, exigence
    `{ Consultation, None }`), exécutée par l'AC9 (a) ; la mise en forme reste au handler ; AC11 (3) : « établi par la
    lecture statique, confirmé sur les branches des fixtures » (C-32-3-17, qui révise C-32-3-10).
  - **R4/F9** — actions et types d'entité **littéraux** aux sept `NewAuditLogEntry::user(`, aucune entrée à
    `SITES_INDIRECTS` ; G4-ter ; `audit_label_registry`, `textes_coherents`, `admin_pat_denied_e2e`,
    `audit_route_registry` nommés à l'AC7 et au gate ciblé de l'AC13 (C-32-3-19).
  - **R5** — doc-comment de l'AC3, Origine, Story, AC10 (iv) et *Hors périmètre* : la fabrication d'une `Lecture` en
    lecture seule (forme et lieu) est décidée par Guy (Q11) et réalisée par la 32-6 ; la 32-3 ne suppose qu'une poignée
    distincte (C-32-3-22).
  - LOW : **R6/F4/F5** fait 9 et Risques recomptés (trois tests sur le bilan, un lit le contenu ; 17 handlers sur 18,
    7 chemins de trace sur 8) ; **R7/F7** ce qu'expose le `Rapport` (période résolue, bornes, `as_of` calculé une fois,
    mode ; `user_id` porté), balance âgée en CSV seul (C-32-3-23) ; **R8/F8** AC7 : neuf fichiers relevés, sept qui
    frappent une route (F8 se trompait sur `letterings_e2e` : `letterable` est lu par `open_items_e2e`, l. 701 ; R8
    comptait six, sept au site), neuf comptes, neuf binaires au gate ciblé ; **R9** les huit `details_json` en
    littéral, `as_of` lu sur le `Rapport` (minuit UTC) ; **R10/F10** #729 aux sites, assertion `…_729` ; **R11** garde
    `get_or_create` étendue à `services/` et `helpers.rs` ; **R13/F6** « une relation d'ordre, deux sites » ; **F12**
    manuel vérifié — `admin-manual.tex:1464` dit du rôle Consultation « aucune mutation, à une exception près » alors
    que chaque consultation de rapport écrit au journal d'audit : écart réel, antérieur, **issue P3 demandée à
    l'orchestrateur**, inscrit au *Hors périmètre* *(annoté en P2 : ouverte, **#730**, et complétée — constat incomplet,
    cf. Validation P2, R2)* ; **F13** `content_type`/`extension` suivent `FormatExport`, assumé
    (C-32-3-21) ; recette visuelle sans objet (AC14). Ajout de la remédiation : M12 (équivalente, déclarée).
  - Propagation grepée sur la fiche, le registre et la clé : `seize`, `issue demandée`, `pool de lecture`,
    `pub(crate) fn pool`, `une seule règle`, `deviendra`, `load_pdf_context(&Lecture`. Prompt de la passe P2 :
    `323-validate-prompt-p2.md` (complète, Opus, R et F).
- **2026-10-24 — Validation P2** (Opus, deux lentilles en contexte frais ; rapports `kesh-gate-logs/323-validate-p2-R.md`
  et `-F.md`) : **R 0/0/3/8, F 0/0/3/9** (CRITICAL/HIGH/MEDIUM/LOW). Recoupements : R1 = F1 (MEDIUM), R4 = F4, R5 = F5,
  R7 = F10, R8 = F7, R9 = F9, R10 = F6 ; propres à R : R2, R3 (MEDIUM), R6, R11 ; propres à F : F2, F3 (MEDIUM), F8,
  F11, F12 — soit **cinq défauts MEDIUM distincts** (inventaire des poignées, manuel, refus avant validation, traces
  des dix-huit couples, usage du handler) et onze LOW distincts. ⚠️ **Signal de recyclage déclaré au Project Lead** :
  R1 = F1 est une **forme recyclée** du MEDIUM F2 de la P1 (le pool d'une poignée atteignable hors de la couche) — un
  défaut revenu sous une autre forme après son correctif, que la P1 avait traité par une énumération de formes.
  Remède par **inventaire clos**, non par une énumération plus longue. Le périmètre reste à quatre modules (F, axe 2) :
  pas de découpage proposé ; l'arbitrage reste au Project Lead. Sort de chacun (registre C-32-3-24 à C-32-3-30) :
  - **R1/F1** — AC10 (v) refondu en cinq inventaires clos, listes écrites (v-a huit lignes, v-b deux en-têtes d'`impl`,
    v-c quatre `fn`, v-d deux `::new(` au câblage de `lib.rs`, v-e deux définitions), sur `services/` ou tout `src/` ;
    champs des poignées privés, fonctions d'audit libres ; anciens (iii)-(iv) absorbés par (v-d), qui couvre
    `services/` ; M11 gardée, M11′ et M11″ ajoutées ; les phrases « interdit d'élargir sans rougir » (AC10, *Hors
    périmètre*) réécrites en « attrape » et « reste à la revue » (formes non lexicales) ; *Le motif*, AC3, R4, R8,
    esquisse alignés (C-32-3-24).
  - **R2** — *Hors périmètre* et AC14 : **#730** citée, deux sites du manuel (`admin-manual.tex:1464`, `:2897`), trois
    écritures omises (rapports, `exports.global`, `touch_last_used`) ; #730 complétée par l'orchestrateur ; en-tête de
    la fiche ; entrée P1 annotée (non réécrite).
  - **R3** — AC8 (c) scindé en (c-val) et (c-lec), une assertion chacun par service ; M3′ ; R7 des règles (C-32-3-26).
  - **F2** — AC9 (b) tabulaire sur les dix-huit couples, `details_json` en littéral par ligne, filtre de journal non
    nul, filtre de comptes, `cumulative`, deux formats ; M1, M1′, M4, M10 ajustés ; fait 9 et Risques (C-32-3-25).
  - **F3** — AC10 (vii) (`.exporter(`/`.afficher(` par corps de handler, neuf de chaque) et AC9 (b′) (la même table
    par la route) ; M13 (C-32-3-25).
  - LOW : **R4/F4** T5 (M1 à M13, primes, M1-témoin) ; **R5/F5** R3 complétée (aide partagée : le plus bas des rôles),
    cas « société inexistante » et M2″ (C-32-3-27) ; **R6** mutation de `AppState::trace()` dite non exprimable,
    à la 32-6 ; **R7/F10** mesures du `span` dans `rendre`, validation sous le `span` assumée (C-32-3-28) ; **R8/F7**
    définition des commentaires pour tous les volets, commentaire de fin de ligne, interdiction des `/* */` retirée
    (C-32-3-30) ; **R9/F9** fixture de TVA : facture validée à ligne taxée, écriture sur `1000`,
    `seed_accounting_company` retenu, sans recopier l'`INSERT` de réglages (F9 se trompait sur le compte d'impôt
    préalable, configuré à `1000`) ; **R10/F6** M12 située dans `tracer`, « une seule variable `as_of` » ; **R11**
    `invoice_settlements` à l'AC11 (3) ; **F8** huit gardes de sources nommées (dont `test_schema_guard`, que la P2
    n'avait pas relevée, atteignable par le fichier de test neuf), motifs d'exclusion des neuf autres lecteurs de
    fichiers ; **F11** `ContexteAcces` à champs privés, résidu `CurrentUser` écrit (C-32-3-29) ; **F12** dérives et
    visibilité de `FormatExport`.
  - Propagation grepée sur la fiche, le registre et la clé, par valeur : `P3 demandée`, `issue demandée`, `élargir`,
    `M1 à M1`, `huit assertions`, `huit chemins`, `quatre gardes`, `AppState::trace()`, `/* `, `Trace::new(`,
    `Lecture::new(`, `le permet`, `pub(in crate::services) MySqlPool`, `méthodes de .Trace.`, `pub company_id` ;
    survivances laissées : relevés datés (Change Log P1, registre C-32-3-17/18/19/20 annotés). Prompt de la passe P3 :
    `323-validate-prompt-p3.md` (complète, Sonnet, R et F).
