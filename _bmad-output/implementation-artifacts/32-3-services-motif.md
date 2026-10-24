# Story 32.3 : la couche de services — le motif, sur deux modules pilotes (rapports, plan comptable)

Status: ready-for-dev

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-32-3`, branche `story/32-3` partie de la spike
`275e9b97`. Registre : `epic-32-choix-autonomes-323.md` (C-32-3-1 à C-32-3-16). Clé courte : `323`. Issue de l'epic :
#723 (CR « Serveur MCP intégré à Kesh »), en `refs` seulement. Les numéros de ligne cités sont ceux de `275e9b97`.*

## Story

En tant que Guy, qui va brancher un agent sur ses livres réels par le serveur MCP de la 0.16.0, je veux que l'API et le
futur MCP lisent les livres **par les mêmes services**, qui portent eux-mêmes le contrôle d'accès et séparent la lecture
des données de l'écriture de la trace, afin que le MCP ne puisse ni contourner une règle d'accès écrite ailleurs, ni
perdre en silence la ligne d'audit d'un rapport lorsque sa connexion de lecture deviendra une connexion en lecture seule
(32-6).

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
  attente du Project Lead. Cette story n'en dépend pas** : le service reçoit une poignée `Lecture` ; **sa fabrication**
  — pool en lecture seule, utilisateur `SELECT`, contrôle au démarrage — **reste à la 32-6**. En 32-3, `Lecture` et
  `Trace` enveloppent **le même** pool de production (`state.pool`) ; seul le test (AC9) fabrique une `Lecture` en
  mode de session `READ ONLY`, pour prouver la séparation — ce n'est **pas** la preuve de la lecture seule de
  production, qui appartient à la 32-6 (C-32-3-7).

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
   changerait le comportement, ce que cette story s'interdit : **issue demandée à l'orchestrateur** (C-32-3-8).
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
9. **Couverture actuelle de la trace** : un seul test HTTP lit la ligne d'audit d'un rapport —
   `report_generated_audit_emitted_on_success` (`tests/reports_e2e.rs:955`, bilan seulement) — plus son pendant
   négatif (l. 1027). Aucun test ne lit la trace d'un export, du grand livre, de la balance âgée ni d'un rapport de
   projet (`grep -n "audit_log" crates/kesh-api/tests/{reports_export_e2e,vat_report_e2e,project_reports_e2e}.rs` → rien).
   Une trace perdue sur ces seize chemins serait **muette** aujourd'hui.
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
  **en lecture** aux dépôts et générateurs (`pub(crate) fn pool(&self) -> &MySqlPool`) ; `Trace` **n'expose aucun
  pool** : seulement les méthodes d'écriture de la trace. En production, fabriqués **uniquement** par
  `AppState::lecture()` et `AppState::trace()` ; en 32-3, tous deux sur `state.pool`.
- **`Rapport<T, U>`** : ce que rend un service de rapport. Le contenu `T` **ne s'obtient que** par
  `Rapport<T, Affichage>::afficher(self, &Trace) -> T` (trace de consultation) ou
  `Rapport<T, Export>::exporter(self, &Trace, rendre) -> Result<Vec<u8>, AppError>` (rendu, **puis** trace d'export).
  Un handler ne peut donc pas obtenir le rapport sans écrire la trace, ni tracer un export dont le rendu a échoué
  (C-32-3-3). Les métadonnées nécessaires au nom de fichier (les bornes d'un rapport de projet) sont exposées en
  lecture seule, sans donner le contenu.

## Critères d'acceptation

### A — Le module et ses types

1. **AC1 — module `services`.** `crates/kesh-api/src/services/` neuf, déclaré `pub mod services;` dans `lib.rs` :
   `mod.rs` (doc-comment de module : **le motif écrit pour le rollout**, les règles R1 à R8 des Dev Notes, et ce qui
   reste à la 32-6 — la fabrication de `Lecture`), `acces.rs` (`ContexteAcces`, `Canal`, `Portee`, `Exigence`),
   `poignees.rs` (`Lecture`, `Trace`), `rapports.rs` (services des rapports, `Rapport<T, U>`, `Affichage`, `Export`,
   `FormatExport`, la trace des rapports), `plan_comptable.rs` (lecture du plan comptable). **Aucun type d'Axum** ni
   `AppState` dans `services/` : `grep -rnE "axum|AppState|CurrentUser" crates/kesh-api/src/services` ne rend que
   `ContexteAcces::depuis_api(&CurrentUser)` (C-32-3-14). Chaque élément public porte son doc-comment.
2. **AC2 — contrôle d'accès dans le service, en tête.** Chaque fonction de service publique commence par
   `acces.exiger(…)?` **avant toute lecture** — y compris avant la validation de la demande. Les exigences sont
   celles du tableau de l'AC6 ; pour le canal API, elles **répliquent** le RBAC actuel des routes (fait 8) : le
   comportement HTTP ne change pas, le middleware refusant toujours le premier.
3. **AC3 — poignées typées, fabriquées en un lieu.** `Lecture::new(pool)` et `Trace::new(pool)` publics (la 32-6 et les
   tests en ont besoin) ; `AppState::lecture()` et `AppState::trace()` dans `lib.rs` (méthodes, **aucun champ ajouté**
   à `AppState`, fait 10), doc-comment : « en 0.16.0, la 32-6 fait de `lecture()` une connexion en lecture seule ;
   jusque-là, les deux poignées enveloppent `pool` ». Aucune conversion `Lecture → Trace` ni `Trace → Lecture`, aucun
   accesseur de pool sur `Trace`.

### B — Les pilotes

4. **AC4 — neuf services de rapport.** `rapports::{bilan, compte_de_resultat, balance, journaux, tva, grand_livre,
   balance_agee, depenses_projet, rendement_projet}`, chacun générique sur l'usage (`Affichage` ou `Export(FormatExport)`),
   recevant `(&ContexteAcces, &Lecture, demande, usage)` et rendant `Result<Rapport<T, U>, AppError>`. La validation de
   la demande passe dans le service **dans l'ordre exact d'aujourd'hui** (`validate_fiscal_year_id` puis
   `ReportPeriod::resolve` ; `LedgerPeriod::new` puis `parse_ledger_account_ids` ; pour les projets, le corps de
   `resolve_project_report` l. 746-793 ; la date `as_of` de la balance âgée, `Utc::now().naive_utc().date()`, calculée
   dans le service). Le grand livre garde ses deux régimes : affichage `offset` lu et `limit` à
   `Some(limit.unwrap_or(MAX_LEDGER_LIMIT))`, export `offset 0` et `limit None` (l. 295-301, 342-348). La validation
   du **format** d'export (`validate_format`, `validate_aged_export_format`) reste au handler, **en premier**, comme
   aujourd'hui. `ExportFormat` passe dans `services::rapports` sous le nom `FormatExport` avec ses trois méthodes, et
   `routes/reports.rs` l'importe sous son nom actuel (`use crate::services::rapports::FormatExport as ExportFormat;`),
   de sorte que le `mod tests` de `reports.rs` (l. 1587-1648) compile **sans modification** (C-32-3-11).
5. **AC5 — un service du plan comptable.** `plan_comptable::lister(&ContexteAcces, &Lecture, inclure_archives) ->
   Result<Vec<CompteDuPlan>, AppError>` (`CompteDuPlan { compte: Account, lettrable: bool }`), exigence
   `{ Consultation, Some(BooksRead) }`, pas de trace (aujourd'hui aucune). Il reprend `list_accounts` à l'identique :
   société lue d'abord (même erreur `Internal`, même message), puis `list_by_company`, puis
   `letterable_account_ids` sur une connexion acquise de la `Lecture`. Pour ne pas dupliquer le message, `helpers.rs`
   gagne `company_for_ids(pool, company_id, user_id)`, à laquelle `get_company_for` délègue (C-32-3-12). Le handler
   `list_accounts` construit `AccountResponse::new(compte, lettrable)` ; les handlers mutants ne changent pas.
6. **AC6 — handlers minces, comportement inchangé.** Chaque handler de lecture des pilotes appelle son service et ne
   fait plus que : extraire la requête, valider le format d'export, ouvrir le `span` d'export, charger le contexte PDF
   (`load_pdf_context`, qui reçoit désormais `&Lecture`), rendre, construire la réponse. Statuts, corps, en-têtes,
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
   JSON, message et champs du `warn!` — sont **recopiés à l'identique** ; seule la source du pool change.

### C — Les tests

7. **AC7 — les tests existants restent verts, sans modification.** Aucun fichier de test existant n'est modifié,
   renommé ni supprimé ; le `mod tests` de `routes/reports.rs` non plus. Contrôle au dernier commit :
   `git diff --diff-filter=MDR --name-only $(git merge-base HEAD origin/main) HEAD -- crates/kesh-api/tests` → **vide**,
   et `diff <(git show $(git merge-base HEAD origin/main):crates/kesh-api/src/routes/reports.rs | sed -n
   '/^#\[cfg(test)\]/,$p') <(sed -n '/^#\[cfg(test)\]/,$p' crates/kesh-api/src/routes/reports.rs)` → **vide** (le
   bloc `#[cfg(test)] mod tests`, l. 1587 à la fin, identique octet pour octet). Fichiers qui doivent rester verts —
   l'ensemble des tests HTTP qui frappent une route de l'AC6, relevé le 2026-10-24 par
   `grep -rlE "/api/v1/reports|/api/v1/accounts" crates/kesh-api/tests` (**9** fichiers) : `accounts_e2e.rs`,
   `idor_multi_tenant_e2e.rs`, `letterings_e2e.rs`, `opening_balances_e2e.rs`, `open_items_e2e.rs`,
   `project_reports_e2e.rs`, `reports_e2e.rs`, `reports_export_e2e.rs`, `vat_report_e2e.rs` (attributs `sqlx::test` :
   39 dans `reports_e2e.rs`, 21 dans `reports_export_e2e.rs`, 11 dans `vat_report_e2e.rs`, 8 dans
   `project_reports_e2e.rs`, 19 dans `accounts_e2e.rs`, par `grep -c "sqlx::test"`). Le développement relance ce
   `grep` au dernier commit et recopie la liste et les comptes réellement exécutés au Dev Agent Record.
8. **AC8 — un test de refus par service, et son contrôle positif.** Fichier neuf `crates/kesh-api/tests/services_motif.rs`
   (`#[sqlx::test(migrations = "../kesh-db/test-schema")]`, fixtures **synthétiques** :
   `kesh_db::test_fixtures::seed_accounting_company`, plus une facture ouverte et un projet posés en SQL pour la balance
   âgée et les rapports de projet). Pour **chacun des dix services** (neuf rapports, `lister`), une assertion nommée
   par cas :
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
   - (a) les **neuf** services de rapport et `lister` **réussissent** sur cette `Lecture` (en affichage et, pour les
     rapports, en export) — preuve exécutée que leur chemin de lecture n'écrit rien (AC11) ;
   - (b) pour **chacun des huit chemins de trace** (période, grand livre, balance âgée, projet) × (affichage, export),
     après `afficher`/`exporter`, **exactement une** ligne neuve dans `audit_log` pour l'utilisateur, avec l'`action`,
     l'`entity_type` (`report`), l'`entity_id` (`0`) et un `details_json` **égal**, objet entier, à celui qu'écrit le
     code d'aujourd'hui (tableau de l'AC6, y compris `report.generated` sans format pour l'export du grand livre et
     `"format": "csv"` pour la balance âgée) — une assertion nommée par chemin ;
   - (c) `exporter` avec un rendu qui échoue → `Err`, et **aucune** ligne d'audit (ordre du fait 4) ;
   - (d) un service qui échoue (période hors exercice) → aucune ligne (pendant de `reports_e2e.rs:1027`, ici au niveau
     du service).
10. **AC10 — garde lexicale de la frontière.** Dans `services_motif.rs`, un test qui lit les sources (comme
    `textes_coherents.rs` ou `audit_route_registry.rs`) et exige : (i) `crates/kesh-api/src/routes/reports.rs` ne
    contient plus ni `state.pool` ni `MySqlPool` ; (ii) le corps de `list_accounts` (de `pub async fn list_accounts` au
    `pub async fn` suivant) ne contient pas `state.pool` ; (iii) aucun fichier de `crates/kesh-api/src/routes/` ne
    contient `Lecture::new(` ni `Trace::new(` ; (iv) `crates/kesh-api/src/lib.rs` en contient **un** de chaque (dans
    `lecture()` et `trace()`). Chaque volet rend le fichier et la ligne fautifs. Contrôle positif écrit dans le test :
    chaque motif cherché est d'abord trouvé dans un texte témoin, pour qu'une faute de frappe dans le motif ne rende pas
    la garde vide.

### D — Inventaire, mutations, gates

11. **AC11 — inventaire clos des écritures sur les chemins de lecture des pilotes.** Ensemble clos, écrit au
    doc-comment de `services/mod.rs` et au Dev Agent Record : (1) les **sept** fonctions d'audit, **18** sites
    d'appel, toutes déplacées sur `Trace` (AC6) ; (2) `api_keys::touch_last_used` (`auth/api_key.rs:185`), écrite par
    le **middleware** sur `state.pool` pour une requête par PAT — **hors service**, inchangée ; une clé MCP aura sa
    propre validation, dont la 32-5 dira qu'elle écrit sa dernière utilisation sur `Trace` (spike § 4.4) ; (3) aucune
    autre : générateurs de `kesh-report` (fait 3), `ReportPeriod::resolve`, `resolve_scope`, `load_pdf_context`,
    `get_company_for`/`company_for_ids`, `list_by_company`, `letterable_account_ids` ne font que lire — **établi par
    l'AC9 (a)**, qui les exécute sur une connexion en lecture seule. Les `get_or_create_default` du § 4.4 de la spike
    ne sont sur aucun chemin de ces pilotes (`grep -n "get_or_create" crates/kesh-api/src/routes/reports.rs
    crates/kesh-api/src/routes/accounts.rs` → rien).
12. **AC12 — mutations qui doivent rougir** (une à la fois, restauration `git checkout -- <f> && touch <f>`, gate
    ciblé `cargo nextest run -E 'binary(services_motif)'`, plus le binaire nommé) ; chacune doit rougir **au moins une
    assertion nommée** :
    - **M1 — la trace part sur `Lecture`** : dans le point commun d'écriture de la trace des rapports, écrire sur le
      pool de la `Lecture` (gardée par le `Rapport`) au lieu de celui de la `Trace` → les huit assertions de l'AC9 (b)
      rougissent (l'`INSERT` échoue en 1792, avalé par le `warn!` : la ligne manque). **M1′** : la même mutation sur le
      seul chemin « export du grand livre » → sa seule assertion rougit.
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
    Une mutation qui ne rougit rien est **déclarée** au Dev Agent Record avec la raison (équivalente, ou test manquant
    ajouté puis mutation rejouée).
13. **AC13 — gates au dernier commit de code.** `kesh-db` n'est **pas** touché (aucun dépôt, aucune migration,
    `post_restore.rs` non plus) : en boucle de revue, gate ciblé (`fmt` et `clippy` sur le workspace, `nextest` sur
    `services_motif`, `reports_e2e`, `reports_export_e2e`, `vat_report_e2e`, `project_reports_e2e`, `accounts_e2e`,
    `kesh-api` lib) ; au **dernier commit de code** : **gate complet** (`scripts/test-fast.sh` sous verrou, base
    remise à zéro d'abord) **et E2E Playwright complet** (recette de `docs/testing.md`, jugé fichier par fichier contre
    § *Les échecs attendus*), les quatre gates frontend inclus (aucun fichier frontend ne change, mais l'E2E les exige
    par la règle du dernier commit). Ne déclarer que ce qui a tourné, avec les chiffres.
14. **AC14 — documentation.** Aucun changement visible : ni manuel, ni `docs/api-external.md`, ni CHANGELOG, ni
    README (C-32-3-13) — le contrôle se fait quand même (une passe vérifie qu'aucun texte ne décrit ce qui bouge). Les
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
  neuf services, la trace (les sept corps recopiés) ; `routes/reports.rs` : handlers minces, `load_pdf_context(&Lecture,
  …)`, import `FormatExport as ExportFormat` ; gate ciblé.
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
  (trace, dernière utilisation) passe par la `Trace`.
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
- `crates/kesh-api/src/middleware/rbac.rs:19-28` — la règle `role < min_role`, que `ContexteAcces::exiger` reprend
  par la même comparaison sur `Role` (`Ord` par `Role::level`, `kesh-db/src/entities/user.rs:42-60`) : une seule règle,
  pas de copie à tenir d'accord.
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
pub struct Lecture(MySqlPool);  impl Lecture { pub fn new(p: MySqlPool) -> Self; pub(crate) fn pool(&self) -> &MySqlPool; }
pub struct Trace(MySqlPool);    impl Trace   { pub fn new(p: MySqlPool) -> Self; /* aucun accesseur de pool */ }
// services/rapports.rs
pub struct Affichage;  pub struct Export(pub FormatExport);
pub struct Rapport<T, U> { contenu: T, trace: DetailsTrace, usage: U }      // champs privés
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

- **La fabrication d'une `Lecture` en lecture seule** — 32-6, selon Q11. En 32-3, une mutation qui ferait écrire la
  trace sur la `Lecture` **dans un handler** n'est pas observable au niveau HTTP (même pool) : c'est la garde de l'AC10
  (iii) qui la ferme ; au niveau du service, l'AC9.
- **L'export du grand livre tracé en consultation** (fait 5) — conservé ; issue demandée (C-32-3-8).
- **L2** (`actor_type='user'` via PAT, fait 6) — conservé ; l'acteur d'une clé MCP est à la 32-5/32-6.
- **Les autres modules de lecture** (écritures, postes ouverts, contacts, factures) — 32-4a/32-4b.
- **`touch_last_used`** — middleware, hors service (AC11 (2)).
- **L'ajout d'un champ à `AppState`** (le pool de lecture de la 32-6) touchera les tests qui construisent `AppState`
  par littéral (fait 10) : à prévoir par la 32-6, signalé ici.

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

- **Moyen — une trace perdue est muette.** Seize des dix-huit chemins n'ont aujourd'hui aucun test qui lise leur trace
  (fait 9) : l'AC9 (b) les fige avant le déplacement des sept fonctions. Écrire les tests (T2) **avant** de déplacer
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
