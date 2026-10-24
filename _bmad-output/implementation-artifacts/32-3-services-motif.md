# Story 32.3 : la couche de services — le motif, sur deux modules pilotes (rapports, plan comptable)

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-32-3`, branche `story/32-3` partie de la spike
`275e9b97`. Registre : `epic-32-choix-autonomes-323.md` (C-32-3-1 à C-32-3-23). Clé courte : `323`. Issue de l'epic :
#723 (CR « Serveur MCP intégré à Kesh ») ; issue du défaut conservé : #729 (export du grand livre tracé en consultation) ;
toutes deux en `refs` seulement. Les numéros de ligne cités sont ceux de `275e9b97`.*

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
   donc **muette** aujourd'hui sur **dix-sept des dix-huit handlers** — sur **sept des huit chemins de trace** (AC9 (b)),
   le seul couvert étant « période × affichage », et pour le seul bilan.
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

- **`ContexteAcces`** : société, utilisateur, **canal** (`Api { role }` ou `Mcp { portees }`). Construit par
  `ContexteAcces::depuis_api(&CurrentUser)` pour l'API ; `ContexteAcces::mcp(company_id, user_id, portees)` existe pour
  les tests et la 32-6.
- **`Exigence`** : `{ role_api: Role, portee_mcp: Option<Portee> }` — `None` : jamais servi au MCP. `acces.exiger(e)`
  rend `Err(AppError::Forbidden)` si `Api { role }` et `role < e.role_api`, ou si `Mcp { portees }` et que
  `e.portee_mcp` est `None` ou absente de `portees`.
- **`Portee`** : `BooksRead`, `InvoicesRead` en 32-3 (les deux dont les pilotes ont besoin, C-32-0-7) ; la 32-5 ajoute
  les autres et leur persistance (C-32-3-4).
- **`Lecture`** et **`Trace`** : deux types distincts, sans conversion de l'un à l'autre. `Lecture` expose son pool
  **à la seule couche de services** (`pub(in crate::services) fn pool(&self) -> &MySqlPool`), qui le passe aux dépôts,
  aux générateurs et à `helpers::company_for_ids` ; **aucun handler ne peut l'atteindre** — le compilateur le refuse
  (C-32-3-17). `Trace` **n'expose aucun pool hors de `services`** : l'écriture de la trace (les sept fonctions d'audit)
  vit dans `services/rapports.rs`, et c'est là seulement que le pool de la `Trace` est lu. En production, fabriqués
  **uniquement** par `AppState::lecture()` et `AppState::trace()` ; en 32-3, tous deux sur `state.pool`.
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
   **exécutée** par le test de l'AC10 (vi) (C-32-3-20). Aucun commentaire de bloc `/* … */` dans `services/` (la
   commande ne les voit pas). Chaque élément public porte son doc-comment.
2. **AC2 — contrôle d'accès dans le service, en tête.** Chaque fonction de service publique commence par
   `acces.exiger(…)?` **avant toute lecture** — y compris avant la validation de la demande. Les exigences sont
   celles du tableau de l'AC6 ; pour le canal API, elles **répliquent** le RBAC actuel des routes (fait 8) : le
   comportement HTTP ne change pas, le middleware refusant toujours le premier.
3. **AC3 — poignées typées, fabriquées en un lieu.** `Lecture::new(pool)` et `Trace::new(pool)` publics (la 32-6 et les
   tests en ont besoin) ; `#[derive(Clone)]` sur les deux (un clone de pool, sans coût) ; `Lecture::pool()` en
   `pub(in crate::services)`, seul accesseur de pool des deux types, celui de `Trace` étant un champ lu par les
   fonctions d'audit de `services/rapports.rs` (`pub(in crate::services)` lui aussi, ou un accesseur de même
   visibilité) ; `AppState::lecture()` et `AppState::trace()` dans `lib.rs` (méthodes, **aucun champ ajouté** à
   `AppState`, fait 10), doc-comment **sans promesse sur Q11** : « la fabrication d'une `Lecture` en lecture seule
   (mode de session, utilisateur `SELECT`, ou les deux, et son lieu) est décidée par Guy (Q11) et réalisée par la
   32-6 ; en 32-3, les deux poignées enveloppent `pool` » (C-32-3-22). Aucune conversion `Lecture → Trace` ni
   `Trace → Lecture`.

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
   de `load_pdf_context` l. 1229-1234 recopié avec son `map_db_error`), exigence `{ Consultation, None }` (le contexte
   PDF ne sert qu'aux exports, jamais au MCP) ; la mise en forme (`PdfContext`, `map_language_to_bcp47`) reste au
   handler (C-32-3-17). Le grand livre garde ses deux régimes : affichage `offset` lu et `limit` à
   `Some(limit.unwrap_or(MAX_LEDGER_LIMIT))`, export `offset 0` et `limit None` (l. 295-301, 342-348). La validation
   du **format** d'export (`validate_format`, `validate_aged_export_format`) reste au handler, **en premier**, comme
   aujourd'hui. `ExportFormat` passe dans `services::rapports` sous le nom `FormatExport` avec ses trois méthodes, et
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
   ordre des erreurs, **action et `details_json` de chaque trace** : identiques. Tableau des 18 + 1 chemins (ensemble
   clos, fait 1 et fait 7) :

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

   Les sept fonctions d'audit **quittent** `routes/reports.rs` pour `services/rapports.rs`, en méthodes de `Trace`
   (ou fonctions recevant `&Trace`) ; leurs corps — action, `entity_type`, `AUDIT_ENTITY_ID_NONE`, clés et valeurs du
   JSON, message et champs du `warn!` — sont **recopiés à l'identique** ; seule la source du pool change. ⚠️ **L'action
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
   Dev Agent Record. S'y ajoutent les **gardes de sources** qui balaient `crates/*/src` et verront `services/` :
   `audit_label_registry.rs` (actions littérales, AC6), `textes_coherents.rs` (G4-ter), `admin_pat_denied_e2e.rs`
   (`the_admin_guards_have_no_alias_and_no_second_consumer_crate_wide`, l. 439, qui compte
   `require_admin_role`/`require_not_pat` dans tout `src/`), `audit_route_registry.rs` (lit `lib.rs`,
   que la story touche).
8. **AC8 — un test de refus par service, et son contrôle positif.** Fichier neuf `crates/kesh-api/tests/services_motif.rs`
   (`#[sqlx::test(migrations = "../kesh-db/test-schema")]`, fixtures **synthétiques** :
   `kesh_db::test_fixtures::seed_accounting_company`, plus une facture ouverte et un projet posés en SQL pour la balance
   âgée et les rapports de projet). Pour **chacun des dix services** (neuf rapports, `lister`), une assertion nommée
   par cas — et pour l'aide `identite_societe` (AC4) : MCP avec toutes les portées → `Forbidden`, API `Consultation`
   → `Ok` :
   - (a) canal **MCP sans la portée** exigée (portées vides, et — pour `balance_agee` — `{BooksRead}` seule ; pour les
     neuf autres — `{InvoicesRead}` seule) → `Err(AppError::Forbidden)` ;
   - (b) **contrôle positif** : canal MCP **avec** la portée exigée → `Ok` (sans quoi (a) pourrait rougir pour une autre
     raison) ;
   - (c) **le refus précède la lecture et la validation** : canal MCP sans portée, demande qui échouerait sans le
     contrôle — `fiscalYearId` **inexistant** pour les cinq rapports à période (échec à la lecture de
     `ReportPeriod::resolve`), période inversée (`from > to`) pour le grand livre (échec à `LedgerPeriod::new`),
     `projectId` inexistant pour les deux rapports de projet (échec à `resolve_scope`), société inexistante pour
     `lister` (échec `Internal`) → `Forbidden`, et non l'erreur attendue sans contrôle. **Sans objet pour
     `balance_agee`**, dont la demande ne porte rien qui puisse échouer : le contrôle y reste couvert par (a) et (b),
     et le Dev Agent Record le dit ;
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
     exercent**, que leur chemin de lecture n'écrit rien (AC11). Les fixtures portent au moins une facture ouverte
     (balance âgée non vide) et une écriture sur un compte de TVA si `seed_accounting_company` le permet ; une branche
     de générateur que les fixtures n'atteignent pas est **nommée** au Dev Agent Record, non revendiquée ;
   - (b) pour **chacun des huit chemins de trace** (période, grand livre, balance âgée, projet) × (affichage, export),
     après `afficher`/`exporter`, **exactement une** ligne neuve dans `audit_log` pour l'utilisateur, avec l'`action`,
     l'`entity_type` `"report"`, l'`entity_id` `0` et un `details_json` **égal, objet entier**, à celui qu'écrit le code
     d'aujourd'hui (`routes/reports.rs:599-615, 626-645, 1372-1400, 1407-1435, 1463-1495, 1514-1538, 1548-1573`) — une
     assertion nommée par chemin. Les huit attendus, en littéral (dates `"AAAA-MM-JJ"`, valeurs de la demande du test) :

     | Chemin | `action` | `details_json` |
     |---|---|---|
     | période × affichage | `report.generated` | `{"report_type": <"balance-sheet"…"vat">, "fiscal_year_id": n, "period_start": d, "period_end": d, "journal_filter": null}` (`journal_filter` : le code du journal pour `journals` filtré, `null` sinon — **clé toujours présente**) |
     | période × export | `report.exported` | `{"report_type": …, "format": "pdf"\|"csv", "fiscal_year_id": n, "period_start": d, "period_end": d, "journal_filter": …}` |
     | grand livre × affichage | `report.generated` | `{"report_type": "general-ledger", "period_start": d, "period_end": d, "account_ids": [..]\|null}` (`null` sans filtre — **clé toujours présente**) |
     | grand livre × export | **`report.generated`** | **le même objet, sans `format`** — comportement conservé (#729) : l'assertion se nomme `…_export_grand_livre_trace_en_consultation_729` et son commentaire renvoie à #729, que son correctif modifiera en connaissance de cause |
     | balance âgée × affichage | `report.generated` | `{"report_type": "aged-receivables", "as_of": d}` |
     | balance âgée × export | `report.exported` | `{"report_type": "aged-receivables", "format": "csv", "as_of": d}` |
     | projet × affichage | `report.generated` | `{"report_type": "project-expenses"\|"project-return", "project_id": n, "mode": "fiscal_year"\|"cumulative"}` |
     | projet × export | `report.exported` | `{"report_type": …, "format": "pdf"\|"csv", "project_id": n, "mode": …}` |

     `period_start`/`period_end` d'un rapport à période sont ceux de la `ReportPeriod` **résolue**, `fiscal_year_id`
     celui de la demande. Pour la balance âgée, l'`as_of` attendu est **celui qu'expose le `Rapport`**, lu avant
     `afficher`/`exporter` — le test ne recalcule pas la date, et ne peut donc pas franchir minuit UTC entre l'attendu
     et l'écrit ; une assertion de plus vérifie que cette date est celle du contenu (`AgedReceivables.as_of`)
     (C-32-3-23) ;
   - (c) `exporter` avec un rendu qui échoue → `Err`, et **aucune** ligne d'audit (ordre du fait 4) ;
   - (d) un service qui échoue (période hors exercice) → aucune ligne (pendant de `reports_e2e.rs:1027`, ici au niveau
     du service).
10. **AC10 — garde lexicale de la frontière.** Dans `services_motif.rs`, un test qui lit les sources (comme
    `textes_coherents.rs` ou `audit_route_registry.rs`) et exige : (i) `crates/kesh-api/src/routes/reports.rs` ne
    contient plus ni `state.pool` ni `MySqlPool` ; (ii) le corps de `list_accounts` (de `pub async fn list_accounts` au
    `pub async fn` suivant) ne contient pas `state.pool` ; (iii) **aucun fichier de `crates/kesh-api/src/`, hors
    `services/` et `lib.rs`**, ne contient `Lecture::new(` ni `Trace::new(` (balayage récursif, `main.rs`, `exports/`,
    `mail/`, `middleware/`, `auth/` compris) ; (iv) en 32-3, `crates/kesh-api/src/lib.rs` en contient **exactement un**
    de chaque (dans `lecture()` et `trace()`) — ce volet décrit l'état de la 32-3, non un lieu imposé à la 32-6, qui
    l'amende avec (iii) en connaissance de cause si Q11 fait fabriquer la `Lecture` du MCP ailleurs (C-32-3-22) ;
    (v) dans `services/poignees.rs`, hors commentaires, aucune ligne ne contient `pub(crate)`, ni `pub fn pool`, ni
    `pub MySqlPool` : l'accès au pool d'une poignée reste borné à `pub(in crate::services)` — le compilateur interdit
    alors `lecture.pool()` dans un handler, et ce volet interdit d'élargir la visibilité sans rougir (C-32-3-17) ;
    (vi) la règle de l'AC1 (lignes de code de `services/` portant `axum`, `AppState` ou `CurrentUser` : exactement
    les deux lignes attendues d'`acces.rs`). Chaque volet rend le fichier et la ligne fautifs. Contrôle positif écrit
    dans le test : chaque motif cherché est d'abord trouvé dans un texte témoin, pour qu'une faute de frappe dans le
    motif ne rende pas la garde vide ; le balayage de (iii) affirme un nombre plancher de fichiers lus (comme
    `admin_pat_denied_e2e.rs`, l. 480-485), pour qu'un chemin faux ne la rende pas vide.

### D — Inventaire, mutations, gates

11. **AC11 — inventaire clos des écritures sur les chemins de lecture des pilotes.** Ensemble clos, écrit au
    doc-comment de `services/mod.rs` et au Dev Agent Record : (1) les **sept** fonctions d'audit, **18** sites
    d'appel, toutes déplacées sur `Trace` (AC6) ; (2) `api_keys::touch_last_used` (`auth/api_key.rs:185`), écrite par
    le **middleware** sur `state.pool` pour une requête par PAT — **hors service**, inchangée ; une clé MCP aura sa
    propre validation, dont la 32-5 dira qu'elle écrit sa dernière utilisation sur `Trace` (spike § 4.4) ; (3) aucune
    autre : générateurs de `kesh-report`, `ReportPeriod::resolve`, `resolve_scope`, `identite_societe` (le `SELECT` de
    l'ancien `load_pdf_context`), `get_company_for`/`company_for_ids`, `list_by_company`, `letterable_account_ids` ne
    font que lire — **établi par la lecture statique** (fait 3 ; les dépôts appelés relus : `fiscal_years`, `projects`,
    `accounts`, `letterings`, `companies`, sans écriture ni verrou) **et confirmé, sur les branches que les fixtures
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
      `&Trace::new(self.lecture_m1.pool().clone())` au lieu de la `Trace` reçue → **les huit assertions de l'AC9 (b)
      rougissent** (l'`INSERT` échoue en 1792 sur la `Lecture` de test, avalé par le `warn!` : la ligne manque).
      **M1′** : même échafaudage, substitution au seul appel de `exporter`, gardée par
      `matches!(details, DetailsTrace::GrandLivre { .. })` → sa seule assertion (`…_729`) rougit. Le diff de l'échafaudage et de la mutation est recopié au Dev Agent Record.
      ⚠️ **Ce que M1 ne couvre pas, et pourquoi** : en 32-3, `AppState::lecture()` et `AppState::trace()` enveloppent
      le même pool ; muter `trace()` pour qu'il enveloppe la `Lecture` n'a **aucun effet observable** (même pool, au
      niveau HTTP comme au niveau du service) — c'est une mutation **équivalente**, déclarée comme telle ; le lieu de
      fabrication est tenu par l'AC10 (iii)-(iv), et la preuve de production revient à la 32-6. **M1-témoin** (contrôle
      du test, non une mutation de production) : bâtir dans le test la `Trace` sur le pool en lecture seule → les huit
      assertions rougissent ; consigné pour montrer que l'AC9 (b) voit une trace perdue.
    - **M2 — contrôle d'accès retiré du service** : supprimer l'`acces.exiger(…)?` de `rapports::bilan` → AC8 (a), (c),
      (d) de `bilan` rougissent ; **M2′** : le même retrait dans `plan_comptable::lister` → AC8 (a), (c) de `lister`.
    - **M3 — contrôle d'accès déplacé après la lecture** (dans `rapports::journaux`) → AC8 (c) de `journaux` rougit,
      (a) restant vert : c'est ce qui prouve que (c) mesure l'ordre.
    - **M4 — trace supprimée** : retirer l'`insert_in_tx` du chemin « balance âgée, export » → son assertion AC9 (b).
    - **M5 — mauvaise portée** : `balance_agee` exige `BooksRead` → AC8 (a) avec `{BooksRead}` seule rougit.
    - **M6 — export servi au MCP** : l'exigence d'export porte `Some(BooksRead)` → AC8 (d).
    - **M7 — rôle API perdu** : l'export de `balance_agee` exige `Consultation` → AC8 (e) rougit, alors que
      `aged_receivables_export_forbidden_for_consultation` (`reports_e2e.rs:1663`) reste **vert** (le middleware refuse
      d'abord) — à consigner : c'est la défense en profondeur que le test du service est seul à voir.
    - **M8 — handler revenu au pool** : dans `get_balance_sheet`, appeler `generate_balance_sheet(&state.pool, …)` →
      AC10 (i).
    - **M9 — trace avant rendu** : dans `exporter`, tracer avant `rendre` → AC9 (c).
    - **M10 — clé de détail changée** : renommer `fiscal_year_id` en `fiscalYearId` dans la trace de période →
      `report_generated_audit_emitted_on_success` (`reports_e2e.rs:955`) **et** AC9 (b) rougissent.
    - **M11 — pool rendu atteignable des handlers** : `Lecture::pool()` en `pub(crate)` → AC10 (v) rougit (le
      compilateur, lui, ne dit rien : l'élargissement compile).
    - **M12 — `as_of` recalculé** : dans `balance_agee`, la trace reçoit `Utc::now()…` relu au moment de tracer au lieu
      de la date du contenu → mutation **équivalente hors minuit**, déclarée ; l'AC9 (b) la verrait seulement au
      passage de minuit UTC. Elle est consignée pour dire que la règle « calculée une fois » (C-32-3-23) tient par la
      structure (un seul champ `as_of`), non par le test.
    Une mutation qui ne rougit rien est **déclarée** au Dev Agent Record avec la raison (équivalente, ou test manquant
    ajouté puis mutation rejouée).
13. **AC13 — gates au dernier commit de code.** `kesh-db` n'est **pas** touché (aucun dépôt, aucune migration,
    `post_restore.rs` non plus) : en boucle de revue, gate ciblé (`fmt` et `clippy` sur le workspace, `nextest` sur
    `services_motif`, les neuf binaires de l'AC7 — `reports_e2e`, `reports_export_e2e`, `vat_report_e2e`,
    `project_reports_e2e`, `accounts_e2e`, `opening_balances_e2e`, `open_items_e2e`, `idor_multi_tenant_e2e`,
    `letterings_e2e` —, les quatre gardes de sources — `audit_label_registry`, `textes_coherents`,
    `admin_pat_denied_e2e`, `audit_route_registry` —, et la lib de `kesh-api`) ; au **dernier commit de code** : **gate complet** (`scripts/test-fast.sh` sous verrou, base
    remise à zéro d'abord) **et E2E Playwright complet** (recette de `docs/testing.md`, jugé fichier par fichier contre
    § *Les échecs attendus*), les quatre gates frontend inclus (aucun fichier frontend ne change, mais l'E2E les exige
    par la règle du dernier commit). Ne déclarer que ce qui a tourné, avec les chiffres.
14. **AC14 — documentation.** Aucun changement visible : ni manuel, ni `docs/api-external.md`, ni CHANGELOG, ni
    README (C-32-3-13) — le contrôle se fait quand même (une passe vérifie qu'aucun texte ne décrit ce qui bouge).
    Aucun écran ne change : la recette visuelle est sans objet. L'écart antérieur du manuel sur le rôle Consultation
    (« aucune mutation ») n'est pas corrigé ici (*Hors périmètre*). Les
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
- [ ] **T5 (AC12)** — mutations M1 à M10 (M1′, M2′), chacune tuée ou déclarée.
- [ ] **T6 (AC7, AC13, AC14)** — contrôles de l'AC7, gate complet et E2E complet au dernier commit de code ; Dev Agent
  Record : chiffres réellement exécutés, liste exacte des fichiers de l'AC7, inventaire de l'AC11.

## Dev Notes

### Les règles du motif (à recopier au doc-comment de `services/mod.rs`, pour la 32-4)

- **R1** — Un service ne reçoit ni `AppState`, ni type d'Axum : `(&ContexteAcces, &Lecture, demande[, usage])`, plus
  `&Trace` là où il trace. Le MCP l'appellera tel quel.
- **R2** — Première instruction : `acces.exiger(EXIGENCE)?` — avant la validation, avant toute lecture.
- **R3** — Une `Exigence` par opération, constante nommée à côté du service ; pour l'API, le rôle **égal** au RBAC de la
  route (le middleware reste ; le service ne l'assouplit jamais) ; pour le MCP, une portée, ou `None`.
- **R4** — Toute lecture passe par la `Lecture` ; **aucune écriture** sur un chemin de lecture : ce qui doit écrire
  (trace, dernière utilisation) passe par la `Trace`. Le pool d'une poignée ne sort pas de `services`
  (`pub(in crate::services)`) ; ce qu'un handler doit lire pour sa présentation (le contexte PDF) passe par une
  fonction de service. L'action d'audit reste un **littéral** à chaque `NewAuditLogEntry::user(` (garde
  `audit_label_registry`).
- **R5** — Une trace qu'un handler pourrait oublier est une trace muette : le contenu ne se rend qu'à travers la méthode
  qui trace (`Rapport::afficher`/`exporter`). Une lecture sans trace (le plan comptable) rend directement.
- **R6** — La validation de la demande vit dans le service, **dans l'ordre d'aujourd'hui** ; la présentation (format
  d'export, rendu, en-têtes, nom de fichier) reste au handler.
- **R7** — Chaque service a son test de refus (portée absente, canal refusé, refus avant lecture) **et** son contrôle
  positif ; et un test qui l'exécute sur une `Lecture` en lecture seule.
- **R8** — La garde lexicale (AC10) s'étend aux handlers migrés.

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
pub struct ContexteAcces { pub company_id: i64, pub user_id: i64, pub canal: Canal }
pub struct Exigence { pub role_api: Role, pub portee_mcp: Option<Portee> }
impl ContexteAcces {
    pub fn depuis_api(u: &CurrentUser) -> Self;
    pub fn mcp(company_id: i64, user_id: i64, portees: BTreeSet<Portee>) -> Self;
    pub fn exiger(&self, e: Exigence) -> Result<(), AppError>; // Err(AppError::Forbidden)
}
// services/poignees.rs
#[derive(Clone)] pub struct Lecture(MySqlPool);
impl Lecture { pub fn new(p: MySqlPool) -> Self; pub(in crate::services) fn pool(&self) -> &MySqlPool; }
#[derive(Clone)] pub struct Trace(pub(in crate::services) MySqlPool);   // lu par les fonctions d'audit seules
impl Trace   { pub fn new(p: MySqlPool) -> Self; }
// services/rapports.rs
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

- **La fabrication d'une `Lecture` en lecture seule** — décidée par Guy (Q11), réalisée par la 32-6. En 32-3, un
  handler ne **peut pas** écrire sur le pool d'une `Lecture` : `Lecture::pool()` est `pub(in crate::services)`, et le
  compilateur refuse `lecture.pool()` hors de la couche ; l'AC10 (v) interdit d'élargir cette visibilité, l'AC10 (iii)
  de fabriquer une poignée hors de `services/` et `lib.rs`. Ce qui reste **non observable** en 32-3 : un handler qui
  écrirait par un autre pool (`state.pool`) — fermé dans `routes/reports.rs` et `list_accounts` par l'AC10 (i)-(ii),
  **pas** dans les modules que la 32-4 n'a pas encore migrés ; et une `AppState::trace()` qui envelopperait le pool de
  la `Lecture` — équivalente tant que les deux partagent `state.pool` (M1, AC12). Au niveau du service, l'AC9.
- **L'export du grand livre tracé en consultation** (fait 5) — conservé ; **#729** (C-32-3-8).
- **Le manuel d'administration** (`admin-manual.tex:1464`) dit du rôle Consultation « aucune mutation, à une exception
  près » (le premier téléchargement du PDF d'une facture, qui « l'inscrit au journal d'audit »), alors que chaque
  consultation de rapport inscrit une ligne `report.generated` (fait 1). Écart **antérieur** à la story, qu'elle ne
  déplace pas (AC14) : issue P3 demandée à l'orchestrateur.
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
  trace (fait 9) : l'AC9 (b) les fige avant le déplacement des sept fonctions. Écrire les tests (T2) **avant** de déplacer
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
    l'orchestrateur**, inscrit au *Hors périmètre* ; **F13** `content_type`/`extension` suivent `FormatExport`, assumé
    (C-32-3-21) ; recette visuelle sans objet (AC14). Ajout de la remédiation : M12 (équivalente, déclarée).
  - Propagation grepée sur la fiche, le registre et la clé : `seize`, `issue demandée`, `pool de lecture`,
    `pub(crate) fn pool`, `une seule règle`, `deviendra`, `load_pdf_context(&Lecture`. Prompt de la passe P2 :
    `323-validate-prompt-p2.md` (complète, Opus, R et F).
