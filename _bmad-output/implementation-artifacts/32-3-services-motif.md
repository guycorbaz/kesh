# Story 32.3 : la couche de services — le motif, sur deux modules pilotes (rapports, plan comptable)

Status: review

*Fiche rédigée le 2026-10-24, en autonomie (Opus 5.5), worktree `kesh-32-3`, branche `story/32-3` partie de la spike
`275e9b97`. Registre : `epic-32-choix-autonomes-323.md` (C-32-3-1 à C-32-3-33). Clé courte : `323`. Issue de l'epic :
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
   C'est le défaut de **#431** (« le traçage par repository perd l'attribution par clé API »), sur sept sites que
   #431 ne nommait pas — son fil couvre des dépôts de `kesh-db` et des sites de route (la dévalidation `unvalidate`,
   Story 15-8b), non ces sept fonctions d'audit : **tracé sur #431** par un commentaire du 2026-10-24 qui les nomme
   (C-32-3-31, C-32-3-32).
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
  dépôts, aux générateurs et à `helpers::company_for_ids`, celui de la `Trace` est **destiné** aux seules fonctions
  d'audit de `services/rapports.rs`. Hors de `services`, le compilateur refuse `lecture.pool()` (C-32-3-17) — c'est
  la **garde principale** ; dans `services`, `#![cfg_attr(not(test), forbid(unsafe_code))]` ferme par le
  compilateur tout `unsafe`, `transmute` d'une poignée compris (AC1, C-32-3-32 ; établi par M14 au développement). **Dans** `services`, rien ne prouve qu'un service n'emploie pas le pool d'une poignée à
  autre chose qu'à son usage : la fiche ne promet **aucune preuve lexicale** de cette inaccessibilité (C-32-3-31) ;
  l'AC10 (v) pose deux gardes simples, l'AC9 prouve le comportement à l'exécution, et le paragraphe *Ce que les gardes
  ne voient pas* (AC10) dit le reste, que tiennent la revue de code puis la 32-6. En production, fabriqués
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
   **exécutée** par le test de l'AC10 (vi) (C-32-3-20). Une ligne de code qui porte un commentaire **de fin de ligne**
   reste une ligne de code et compte (le jeton y est écrit en code ou à côté du code : rouge voulu, à ne pas « corriger »
   dans la garde). Un commentaire de bloc `/* … */` n'est **pas** retiré par la règle : il est lu comme du code et ne
   peut donc produire, **dans un volet d'absence ou d'égalité**, qu'un **faux rouge**, jamais masquer une ligne — d'où
   l'absence d'interdiction (C-32-3-30). Le seul volet de **présence**, l'AC10 (vii), ôte au contraire les commentaires
   avant de chercher (C-32-3-31). Chaque élément public porte son doc-comment. **`services/mod.rs` porte, parmi ses
   attributs internes de tête, `#![cfg_attr(not(test), forbid(unsafe_code))]`** (C-32-3-32) : il s'applique au module
   et à ses sous-modules, donc à tout `services/`, dans la compilation de la lib que lient le binaire et les tests
   d'intégration ; un `unsafe` dans `services/` — `std::mem::transmute::<Lecture, Trace>` compris — **ne compile
   pas** — raisonné sur la référence du langage, non compilé : la mutation M14 l'établit au développement. Un `forbid` ne se lève par aucun
   `allow` plus bas. L'attribut ne restreint pas les macros qu'un service peut employer : l'effet du lint sur
   l'expansion d'une macro d'une autre crate (`tokio::join!`, dont l'expansion porte un `unsafe`) n'est pas établi par
   cette fiche, et un précédent publié le contredit (C-32-3-33). **Le crate entier** : l'extension de l'attribut à
   `lib.rs` est **à essayer au développement** (T1) ; les `unsafe` de `config.rs` sont tous dans des modules
   `#[cfg(test)]` (`tests`, l. 1665-1666 et suivantes ; `smtp_config_tests`, l. 3293-3596, `#[cfg(test)]` l. 3292) et n'y feraient pas
   obstacle sous `cfg_attr(not(test), …)` — relevé par lecture, non compilé.
2. **AC2 — contrôle d'accès dans le service, en tête.** Chaque fonction de service publique commence par
   `acces.exiger(…)?` **avant toute lecture** — y compris avant la validation de la demande. Les exigences sont
   celles du tableau de l'AC6 ; pour le canal API, elles **répliquent** le RBAC actuel des routes (fait 8) : le
   comportement HTTP ne change pas, le middleware refusant toujours le premier.
3. **AC3 — poignées typées, fabriquées en un lieu.** `Lecture::new(pool)` et `Trace::new(pool)` publics (la 32-6 et les
   tests en ont besoin) ; `#[derive(Clone)]` sur les deux (un clone de pool, sans coût) ; champ de tuple **privé** sur
   les deux ; `Lecture::pool()` et `Trace::pool()` en `pub(in crate::services)`, seuls accesseurs de pool, seules
   méthodes avec `new` — `Trace::pool()` n'ayant pour appelants **prévus** que les fonctions d'audit de
   `services/rapports.rs`. ⚠️ Sa visibilité ne peut pas être plus étroite : `pub(in …)` n'accepte qu'un module
   **ancêtre** de celui qui définit l'élément (erreur E0742), et `crate::services::rapports` n'est pas un ancêtre de
   `crate::services::poignees` — le plus étroit commun est `crate::services`. Qu'aucun autre service n'appelle
   `trace.pool()` n'est donc tenu **ni par le compilateur ni par une garde** : par la revue (AC10, *Ce que les gardes
   ne voient pas*, C-32-3-31) ; `AppState::lecture()` et `AppState::trace()` dans `lib.rs` (méthodes, **aucun champ ajouté** à
   `AppState`, fait 10), doc-comment **sans promesse sur Q11** : « la fabrication d'une `Lecture` en lecture seule
   (mode de session, utilisateur `SELECT`, ou les deux, et son lieu) est décidée par Guy (Q11) et réalisée par la
   32-6 ; en 32-3, les deux poignées enveloppent `pool` » (C-32-3-22). Aucune conversion `Lecture → Trace` ni
   `Trace → Lecture`, **où que ce soit** dans `src/`, `services/` compris ; aucun `impl` de trait sur les poignées.
   La garde (v-2) de l'AC10 ne laisse `Lecture::new(`/`Trace::new(` qu'au câblage de `lib.rs`, ce qui attrape la
   conversion par reconstruction ; un `transmute` dans `services/` ne compile pas (`forbid(unsafe_code)`, AC1,
   C-32-3-32 ; établi par M14) ; une conversion par un autre moyen (`impl From`, méthode de plus, `transmute` **hors** de `services/`)
   n'est vue que par la revue (C-32-3-31).

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
   l'API : dans les **six** exports qui ouvrent un `span`, la validation de la demande, aujourd'hui **avant**
   l'ouverture du `span` — `validate_fiscal_year_id` l. 462, 660, 970, 1034, 1144 (spans l. 464, 662, 972, 1036, 1146)
   et `LedgerPeriod::new` l. 330 (span l. 332, export du grand livre) —, passe dans le service, appelé sous le
   `span` ; une erreur de validation naît donc dans le `span`, sans qu'aucun champ n'y soit enregistré (comme toute
   erreur postérieure aujourd'hui). Les trois autres exports (`export_aged_receivables`, `export_project_expenses`,
   `export_project_return`) n'ouvrent aucun `span` et ne sont pas concernés. Tableau des 18 + 1 chemins
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
   recevant `&Trace`** — et non en méthodes : les seules méthodes des poignées restent `new` et `pool`, dans
   `poignees.rs` (C-32-3-24, C-32-3-31) ; leurs corps — action, `entity_type`, `AUDIT_ENTITY_ID_NONE`, clés et valeurs du
   JSON, message et champs du `warn!` — sont **recopiés à l'identique** ; seule la source du pool change
   (`trace.pool().begin()` au lieu de `pool.begin()`, aucun type `sqlx` nommé). Seul déplacement de texte autour des
   corps : le doc-comment de conventions (« Audit log `report.generated` — best-effort … Convention projet —
   `audit_log.details_json` JSON keys », `reports.rs:1343-1367`), aujourd'hui collé sans ligne vide au doc-comment
   propre de `emit_ledger_audit` (l. 1368-1371, « Trace la consultation du grand livre. » et son paragraphe), est
   rattaché à **`emit_report_audit`** (l. 1407, sans doc-comment aujourd'hui), à qui il appartient ;
   `emit_ledger_audit` garde ses quatre lignes. ⚠️ **L'action
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
     par couple et une assertion nommée par ligne (le nom porte le service et l'usage). ⚠️ **Chaque appel de service
     y reçoit la `Lecture` en lecture seule du chapeau** — jamais un `Lecture::new(pool)` sur le pool ordinaire — et
     `afficher`/`exporter` la `Trace` du pool ordinaire : c'est ce qui fait de M1 un rouge ; une `Lecture` ordinaire
     rendrait M1 vert, et la preuve de séparation tomberait sans rien signaler. Après `afficher`/`exporter`,
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
     que (b′) relit — **paramétrée** : ses cellules portent les **symboles** `fy`, `debut`, `fin`, `du`, `au`,
     `compte`, `pid`, `asof`, résolus à chaque jeu (en (b) sur le `Rapport`, en (b′) comme dit ci-dessous), et non des
     dates écrites en dur (C-32-3-32) ;
   - (b′) **la même table, par la route** : pour chacune des dix-huit lignes, la requête HTTP de la route du tableau de
     l'AC6 (routeur `kesh_api::build_router` sur `AppState::new_for_tests`, jeton JWT d'un utilisateur seedé, comme
     `reports_e2e.rs:54-80`, montage recopié dans `services_motif.rs`), puis **exactement une** ligne neuve dans
     `audit_log`, d'`action` et de `details_json` égaux à la ligne de la table — chaque ligne est jouée avec **la même
     demande** qu'en (b) : `debut`/`fin` égalent les bornes de la demande quand elle en porte, et, pour la ligne sans
     bornes, celles de l'exercice seedé (`2020-01-01`, `2030-12-31`, `crates/kesh-db/src/test_fixtures.rs:118`, dans
     `seed_accounting_company`) — les seuls littéraux de date, liés aux symboles `debut`/`fin` de cette ligne-là —,
     connues du test ; `asof`
     est accepté s'il égale la date UTC du jour lue **avant ou après** la requête (seul point où le test ne lit pas le
     `Rapport`). C'est le seul volet qui voie un **handler** appeler le mauvais **service** (un `report_type` faux) ; le
     mauvais **usage** (un export tracé `report.generated`, la forme de #729), il le voit avec l'AC10 (vii)
     (C-32-3-25) ;
   - (c) `exporter` avec un rendu qui échoue → `Err`, et **aucune** ligne d'audit (ordre du fait 4) ;
   - (d) un service qui échoue (période hors exercice) → aucune ligne (pendant de `reports_e2e.rs:1027`, ici au niveau
     du service).
   En (b), les bornes `debut`/`fin` sont lues sur le `Rapport` ; une demande sans bornes (résolues à celles de
   l'exercice) est employée pour au moins une ligne, afin que la résolution soit exercée — (b′) la rejoue telle
   quelle (ci-dessus).
10. **AC10 — garde lexicale de la frontière.** Dans `services_motif.rs`, un test qui lit les sources (comme
    `textes_coherents.rs` ou `audit_route_registry.rs`) et exige ce qui suit. **Tous les volets portent sur les lignes
    de code**, selon la définition de l'AC1 (est commentaire une ligne dont le premier caractère non blanc ouvre `//`,
    `///` et `//!` compris ; une ligne de code à commentaire de fin de ligne compte ; un `/* … */` est lu comme du code
    et ne peut produire qu'un faux rouge dans les volets d'absence ou d'égalité — (vii), seul volet de présence, ôte les
    commentaires, cf. ci-dessous). ⚠️ **Cette garde ne prouve pas que le pool d'une poignée est inaccessible** : elle
    rend un rouge nommé pour des fautes décidables, et dit en clair ce qu'elle ne voit pas (C-32-3-31) :
    - (i) `crates/kesh-api/src/routes/reports.rs` ne contient plus ni `state.pool` ni `MySqlPool` ;
    - (ii) le corps de `list_accounts` (de `pub async fn list_accounts` au `pub async fn` suivant) ne contient pas
      `state.pool` ;
    - (iii)-(iv) *(remplacés en P2 par un inventaire, lui-même retiré en P3 — C-32-3-24, C-32-3-31 ; la garde (v-2)
      en reprend la part décidable)* ;
    - (v) **deux gardes simples sur les poignées** (C-32-3-31 ; les inventaires (v-a) à (v-e) de la P2 sont
      **retirés**) :
      - **(v-1)** aucune ligne de code de `crates/kesh-api/src/` **hors** de `services/` (récursif) ne contient
        d'**appel ni de chemin** de l'accesseur `pool`, motif `\.\s*pool\s*\(|::\s*pool\b` (C-32-3-32) : la forme
        méthode (`lecture.pool()`, `. pool (`), l'appel qualifié (`Lecture::pool(&x)`, `<Lecture>::pool(&x)`) et le
        chemin passé comme fonction (`.map(Lecture::pool)`). Baseline vide au relevé du 2026-10-24 (`grep -rnE
        "\.\s*pool\s*\(|::\s*pool\b" crates/kesh-api/src` → rien ; `self.pool.clone()` et `state.pool` — le **champ**
        — n'y tombent pas, raison pour laquelle le motif n'est pas `\bpool\b`). Un chemin de module nommé `pool`
        (`sqlx::pool::…`) rougirait, de même qu'un futur accesseur `AppState::pool()` ou toute méthode `.pool(` d'un
        autre type : faux rouges d'un volet d'absence ; la conduite est d'**élargir la liste d'exceptions écrite** de la
        garde (vide en 32-3) (fichier, ligne réduite de ses blancs, motif écrit au test), jamais d'affaiblir le motif. Le
        compilateur refuse déjà ces appels hors de `services` ; la garde nomme le rouge si une visibilité élargie
        (M11, M11-bis) les rendait possibles d'un handler. Elle ne regarde **pas** l'intérieur de `services/` ;
      - **(v-2)** les lignes de code de tout `crates/kesh-api/src/` qui contiennent `\bLecture::new\(` ou
        `\bTrace::new\(` (frontière de mot : `DetailsTrace::new(` n'est pas compté) sont **exactement deux**, au câblage
        de `lib.rs` : le corps de `AppState::lecture()` (`Lecture::new(self.pool.clone())`, chemin éventuel compris) et
        celui de `AppState::trace()` (`Trace::new(self.pool.clone())`) — comparées en multiensemble de couples
        (fichier, ligne réduite de ses blancs), la mise en forme rustfmt étant recopiée au Dev Agent Record. Rougit :
        la conversion par reconstruction `Trace::new(lecture.pool().clone())`, dans `services/` comme ailleurs — celle
        que M1 simule. Ce volet décrit l'état de la 32-3, non un lieu imposé à la 32-6, qui l'amende en connaissance de
        cause si Q11 fait fabriquer la `Lecture` du MCP ailleurs (C-32-3-22).

      **Ce que les gardes ne voient pas** (écrit en clair, C-32-3-31) — chacune de ces formes **compile** et laisse
      l'AC10 vert :
      - un service de `services/` qui emploie `lecture.pool()` pour **écrire** (`begin()` puis `insert_in_tx`, un dépôt
        d'écriture) ;
      - un service qui emploie `trace.pool()` **hors** des fonctions d'audit — pour lire, ou pour écrire une donnée : la
        visibilité ne peut pas borner cet accesseur à `rapports.rs` (AC3, E0742) ;
      - un accesseur d'un autre nom, une visibilité élargie de `pool()` employée seulement dans `services/`, un
        `impl Deref`/`AsRef`/`From`/`Borrow` sur une poignée, une méthode de plus dans `poignees.rs` ;
      - un alias de type ou un ré-export (`type P = sqlx::MySqlPool;`, `pub use sqlx::MySqlPool as P;`), une macro ;
      - pour (v-1), une fois `pool()` élargi : un appel coupé **entre deux lignes** entre `.` ou `::` et `pool`
        (`lecture.` puis `pool()` à la ligne suivante), ou **entre `pool` et `(`** (`lecture.pool` puis `()` à la ligne
        suivante) — rustfmt n'écrit ni l'un ni l'autre, le compilateur les accepte —, l'identifiant brut
        (`lecture.r#pool()`, `Lecture::r#pool` : `::\s*pool` ne reconnaît pas `::r#pool`), un commentaire de bloc
        glissé dans l'appel (`lecture./* */pool()`), un identifiant fabriqué par une macro ;
      - `unsafe` et `std::mem::transmute::<Lecture, Trace>` **hors** de `services/` (un handler) : le
        `forbid(unsafe_code)` de l'AC1 ne couvre que `services/` (C-32-3-32), sauf si l'essai de T1 l'étend au crate
        (C-32-3-33). Dans `services/`, cette forme est fermée par le compilateur (M14).

      Ce que l'**exécution** en voit : l'AC9 (a) fait échouer (1792) toute écriture sur la `Lecture`, **sur les
      branches que les fixtures exercent** ; l'AC9 (b), M1 et M1-témoin montrent qu'une trace partie sur la `Lecture`
      est vue. ⚠️ **La garantie réelle contre une écriture sur la `Lecture` est celle de la 32-6** — la lecture seule
      imposée par la base, selon la décision de Guy sur Q11. D'ici là, et pour `trace.pool()` employé hors des
      fonctions d'audit — qu'aucune base ne refusera, la `Trace` ayant le droit d'écrire —, c'est la **revue de code**
      de la 32-3 et de chaque rollout qui lit chaque appel de `pool` de `services/` (`.pool(`, `::pool`) ;
    - (vi) la règle de l'AC1 (lignes de code de `services/` portant `axum`, `AppState` ou `CurrentUser` : exactement
      les deux lignes attendues d'`acces.rs`) ;
    - (vii) **le bon usage à chaque handler** (C-32-3-25) : dans `routes/reports.rs`, le corps de chaque
      `pub async fn export_…` (de sa ligne d'ouverture à la première ligne égale à `}`, colonne 0) contient
      `.exporter(` et ne contient pas `.afficher(` ; celui de chaque `pub async fn get_…` contient `.afficher(` et ne
      contient pas `.exporter(` ; le test affirme **neuf** corps de chaque sorte (un chemin faux ou un renommage ne
      rend pas le volet vide). La **présence** se cherche dans le corps **ôté de ses commentaires** — lignes de
      commentaire (AC1), partie de ligne qui suit `//`, texte entre `/*` et `*/` **y compris quand le bloc s'étend sur
      plusieurs lignes** (le retrait des blocs se fait sur le corps entier, non ligne à ligne, C-32-3-32) —, sans quoi
      un commentaire qui cite l'appel ferait un faux vert ; l'**absence** se cherche sur les lignes de code de l'AC1,
      où un commentaire ne peut produire qu'un faux rouge. Un `//` dans une chaîne littérale tronquerait la ligne et ne pourrait que masquer un
      appel réel, donc rougir (aucun corps de handler n'en porte au relevé du 2026-10-24 : `grep -n '"[^"]*//'
      crates/kesh-api/src/routes/reports.rs` → rien).

    Chaque volet rend le fichier et la ligne fautifs. Contrôle positif écrit dans le test : chaque motif cherché est
    d'abord trouvé dans un texte témoin (dont, pour (vii), un appel cité en commentaire, y compris dans un bloc
    `/* … */` de plusieurs lignes, qui doit **ne pas** compter ; pour (v-1), les trois formes `lecture.pool()`,
    `Lecture::pool(&l)` et `.map(Lecture::pool)`, qui doivent compter, et `state.pool`, qui ne doit pas),
    pour qu'une faute de frappe dans le motif ne rende pas la garde vide ; les balayages récursifs de `src/` affirment un nombre plancher
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
      (v-2) rougit aussi (un `Trace::new(` hors du câblage) : attendu, c'est la conversion que l'AC3 interdit ; le Dev
      Agent Record consigne **les deux** rouges, celui de l'AC9 (b) étant la preuve à l'exécution. L'AC9 (b′) reste
      **vert** sous M1 — par la route, la `Lecture` est le pool ordinaire et l'`INSERT` réussit —, ce qui est consigné.
      **M1′** : même échafaudage, substitution au seul appel de `exporter`, gardée par
      `matches!(details, DetailsTrace::GrandLivre { .. })` → la seule ligne 12 de l'AC9 (b) (`…_729`) rougit, les
      dix-sept autres restant vertes ; l'AC10 (v-2) rougit aussi, attendu (le même `Trace::new(` hors du câblage) —
      le rouge de l'AC9 (b) n'est donc pas le seul du gate, et ne doit pas être pris pour une régression. Le diff de l'échafaudage et de la mutation est recopié au Dev Agent Record.
      ⚠️ **Ce que M1 ne couvre pas, et pourquoi** : la mutation « `AppState::trace()` enveloppe la `Lecture` » n'est
      **pas exprimable** en 32-3 — `lib.rs` est hors de `crate::services` et ne peut pas lire `lecture.pool()`, si bien
      que la seule écriture possible de `trace()` est `Trace::new(self.pool.clone())`, c'est-à-dire le code lui-même.
      Elle le deviendra quand la 32-6 donnera à la `Lecture` un pool distinct ; la 32-6 la pose alors (C-32-3-30). Le
      lieu de fabrication est tenu par l'AC10 (v-2). **M1-témoin** (contrôle du test, non une mutation de production) :
      bâtir dans le test la `Trace` sur le pool en lecture seule → les dix-huit lignes de l'AC9 (b) rougissent ;
      consigné pour montrer que l'AC9 (b) voit une trace perdue.
    - **M2 — contrôle d'accès retiré du service** : supprimer l'`acces.exiger(…)?` de `rapports::bilan` → AC8 (a), (c),
      (d) de `bilan` rougissent ; **M2′** : le même retrait dans `plan_comptable::lister` → AC8 (a), (c-lec) de `lister` ;
      **M2″** : le même retrait dans `rapports::identite_societe` → ses deux assertions MCP de l'AC8 rougissent
      (C-32-3-27).
    - **M3 — contrôle d'accès déplacé après la lecture** (dans `rapports::journaux`) → AC8 (c-val) et (c-lec) de
      `journaux` rougissent, (a) restant vert : c'est ce qui prouve que (c) mesure l'ordre. ⚠️ **Forme du service que
      M3 et M3′ supposent** : `exiger`, la validation et la résolution sont **trois appels distincts** dans le corps de
      chaque service — deux lignes de validation et de résolution, que la règle DRY n'oblige pas à factoriser. Si le
      développement les factorise quand même en un auxiliaire commun aux cinq rapports à période, M3 et M3′ se posent
      **dans l'auxiliaire** (qui reçoit alors l'`acces`) et rougissent les cas (c) des **cinq** services, non du seul
      service nommé : le Dev Agent Record le consigne, avec la forme retenue.
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
    - **M10 — clé de détail changée** : renommer `fiscal_year_id` en `fiscalYearId` dans **les deux** fonctions de
      période, `emit_report_audit` et `emit_report_export_audit` (chacune écrit la clé) →
      `report_generated_audit_emitted_on_success` (`reports_e2e.rs:955`) **et** les lignes 1 à 10 de l'AC9 (b) et (b′)
      rougissent. Le renommage dans la seule `emit_report_audit` ne rougit que les lignes 1, 3, 5, 7, 9 (et le test de
      `reports_e2e.rs`), dans la seule `emit_report_export_audit` que les lignes 2, 4, 6, 8, 10 : à consigner si le
      développement les joue séparément.
    - **M11 — pool employé d'un handler** : `Lecture::pool()` en `pub(crate)` **et**, dans `get_balance_sheet`,
      `generate_balance_sheet(state.lecture().pool(), …)` → AC10 (v-1) rougit (une ligne `.pool()` hors de
      `services/`) ; le compilateur, lui, accepte les deux. L'élargissement **seul**, sans appel hors de `services/`,
      ne rougit rien : c'est une forme que les gardes ne voient pas (AC10), consignée comme telle. *(M11′ et M11″, qui
      ne visaient que les inventaires retirés en P3, sont retirées avec eux : `impl Deref` et l'accesseur d'un autre
      nom sont au paragraphe *Ce que les gardes ne voient pas*, C-32-3-31.)*
    - **M11-bis — pool appelé d'un handler par chemin qualifié** (C-32-3-32) : le même élargissement **et**, dans
      `get_balance_sheet`, `generate_balance_sheet(Lecture::pool(&state.lecture()), …)` → AC10 (v-1) rougit (une ligne
      `::pool` hors de `services/`) ; c'est la forme que le motif `\.pool\(\)` d'avant la P4 laissait verte.
    - **M14 — `unsafe` dans `services/`** (C-32-3-32) : dans `services/poignees.rs`, une fonction
      `fn m14(l: Lecture) -> Trace { unsafe { std::mem::transmute::<Lecture, Trace>(l) } }` → la lib de `kesh-api` **ne
      compile pas** (`forbid(unsafe_code)`, AC1) : gate `cargo build -p kesh-api --lib`, et le binaire `services_motif`
      ne se construit pas. Le Dev Agent Record recopie le message du compilateur ; c'est un rouge de compilation, non
      d'assertion, déclaré comme tel.
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
    README (C-32-3-13) — le contrôle se fait quand même (une passe vérifie qu'aucun texte ne décrit ce qui bouge),
    **y compris** `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (l. 152, 160, 244-273 : « Every handler validates company
    exists — `get_company_for()` », exemple en handler ; la 32-3 ajoute `company_for_ids` et un second lieu de
    vérification, le service — une ligne de renvoi au doc-comment de `services/mod.rs` si le texte le contredit) et
    `planning-artifacts/architecture.md` (arbre de `kesh-api/src`, l. 471-490, sans `services/`).
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

- [x] **T0** — Worktree `kesh-32-3`, branche `story/32-3` (déjà posés) ; bases `kesh_323`, `kesh_e2e_323` créées,
  migrées depuis les migrations du worktree et seedées ; port attribué par l'orchestrateur ;
  `CARGO_TARGET_DIR=/home/gcorbaz/devel/kesh-32-3/target` (`docs/agents-orchestration.md` § 4).
- [x] **T1 (AC1-AC3, AC15)** — `services/{mod,acces,poignees}.rs` ; `pub mod services;` ; `AppState::lecture()` et
  `trace()` ; doc-comments. **Essai du crate entier** (C-32-3-33) : poser `#![cfg_attr(not(test), forbid(unsafe_code))]`
  dans `lib.rs`, compiler (`cargo build -p kesh-api --all-targets`) ; s'il passe, le garder (l'attribut de
  `services/mod.rs` reste, M14 inchangée) ; sinon le retirer et consigner au Dev Agent Record la **cause réelle
  constatée** (fichier, ligne, message du compilateur).
- [x] **T2 (AC8, AC9, AC10 — rouge d'abord)** — `tests/services_motif.rs`, en **deux commits** : (i) le montage HTTP,
  la table et l'AC9 (b′) **seuls**, sans aucune référence à `kesh_api::services` — joués **verts sur le code d'avant
  T3** (c'est le relevé qui établit que la table est ce que le code fait aujourd'hui), résultat consigné au Dev Agent
  Record ; (ii) le reste — fixtures synthétiques, `Lecture` de test en lecture seule, AC8, AC9 (a)-(d), AC10,
  assertions nommées —, dont on constate qu'il ne compile pas ou rougit avant T3-T4.
- [x] **T3 (AC4, AC6, AC11)** — `services/rapports.rs` : `FormatExport`, `Affichage`, `Export`, `Rapport<T, U>`, les
  neuf services, `identite_societe`, `Rapport::nouveau`, l'aiguillage `tracer`, la trace (les sept corps recopiés, actions
  littérales ; le doc-comment de conventions rattaché à `emit_report_audit`, AC6) ; `routes/reports.rs` : handlers minces, `load_pdf_context(&ContexteAcces, &Lecture)` (mise en forme
  seule), import `FormatExport as ExportFormat` ; gate ciblé.
- [x] **T4 (AC5)** — `helpers.rs::company_for_ids` (et délégation de `get_company_for`) ; `services/plan_comptable.rs` ;
  `list_accounts` mince ; gate ciblé.
- [x] **T5 (AC12)** — mutations M1 à M14 (M1′, M2′, M2″, M3′, M11-bis) et M1-témoin, chacune tuée ou déclarée
  (M1 en échafaudage, M12 équivalente hors minuit).
- [x] **T6 (AC7, AC13, AC14)** — contrôles de l'AC7, gate complet et E2E complet au dernier commit de code ; Dev Agent
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
  (`pub(in crate::services)`, champ privé — le compilateur le tient ; aucun `unsafe` dans `services`,
  `forbid(unsafe_code)` en tête de `services/mod.rs`, C-32-3-32) ; **dans** `services`, on n'ouvre aucun autre
  chemin vers lui et on n'emploie `lecture.pool()` qu'à lire, `trace.pool()` qu'aux fonctions d'audit — ce qu'aucune
  garde ne prouve : la revue le lit, l'AC9 le vérifie à l'exécution, la 32-6 l'imposera par la base pour la
  `Lecture` (AC10, *Ce que les gardes ne voient pas*, C-32-3-31) ; ce qu'un handler doit lire pour sa présentation (le contexte PDF) passe par une
  fonction de service. L'action d'audit reste un **littéral** à chaque `NewAuditLogEntry::user(` (garde
  `audit_label_registry`).
- **R5** — Une trace qu'un handler pourrait oublier est une trace muette : le contenu ne se rend qu'à travers la méthode
  qui trace (`Rapport::afficher`/`exporter`). Une lecture sans trace (le plan comptable) rend directement.
- **R6** — La validation de la demande vit dans le service, **dans l'ordre d'aujourd'hui** ; la présentation (format
  d'export, rendu, en-têtes, nom de fichier) reste au handler.
- **R7** — Chaque service a son test de refus (portée absente, canal refusé, refus avant la validation **et** avant la
  lecture) **et** son contrôle positif ; un test qui l'exécute sur une `Lecture` en lecture seule ; et, s'il trace, sa
  ligne dans la table des traces, au service et par la route.
- **R8** — La garde lexicale (AC10) s'étend aux handlers migrés ((i)-(ii), (vii)) ; la liste de (v-2) s'amende en
  connaissance de cause, ligne par ligne, jamais par un motif élargi. Elle ne prouve pas l'inaccessibilité du pool : la
  revue de chaque rollout lit chaque appel de `pool` de `services/` (`.pool(`, `::pool`). Un module de services
  ajouté par la 32-4 hérite du `forbid(unsafe_code)` de `services/mod.rs`, qu'il ne doit pas déplacer.

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
// services/poignees.rs — deux types, quatre `fn` (deux `new`, deux `pool`), rien d'autre
#[derive(Clone)] pub struct Lecture(MySqlPool);                      // champ privé
impl Lecture { pub fn new(p: MySqlPool) -> Self; pub(in crate::services) fn pool(&self) -> &MySqlPool; }
#[derive(Clone)] pub struct Trace(MySqlPool);                        // champ privé
impl Trace   { pub fn new(p: MySqlPool) -> Self; pub(in crate::services) fn pool(&self) -> &MySqlPool; }
// (pool() de Trace : destiné aux seules fonctions d'audit, libres, de services/rapports.rs — tenu par la revue :
//  pub(in crate::services::rapports) n'est pas écrivable ici, rapports n'étant pas un ancêtre de poignees, E0742)
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
  compilateur refuse `lecture.pool()` hors de `services` (`pub(in crate::services)`, champ privé) : c'est la garde
  principale. La fiche **ne promet pas** de preuve lexicale que le pool d'une poignée est inaccessible **dans**
  `services` (C-32-3-31) : l'AC10 (v) n'attrape que deux fautes décidables — un appel ou un chemin de `pool` hors de
  `services/`, forme méthode ou qualifiée (v-1), un `Lecture::new(`/`Trace::new(` hors du câblage de `lib.rs` (v-2) ;
  le compilateur ferme `unsafe` dans `services/` (`forbid(unsafe_code)`, AC1, C-32-3-32). **Restent à la revue**,
  d'ici la 32-6 : un service qui écrirait par `lecture.pool()` (l'AC9 (a) le voit sur les branches exercées), un
  `trace.pool()` hors des fonctions d'audit, un autre accesseur, un `impl` de trait sur une poignée, un alias, un
  ré-export, une macro, `unsafe` **hors** de `services/` sauf si l'essai de T1 étend l'attribut au crate (AC10, *Ce
  que les gardes ne voient pas*, C-32-3-33). La garantie réelle contre une écriture sur la `Lecture` est la
  lecture seule imposée par la base en 32-6 (Q11). Reste **non observable** en 32-3 : un handler qui écrirait par un autre pool
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
- **L2** (`actor_type='user'` via PAT, fait 6) — conservé ; l'acteur d'une clé MCP est à la 32-5/32-6. Même défaut
  que **#431**, dont le fil couvre des dépôts et des sites de route (`unvalidate`), non les sept fonctions d'audit des
  rapports : un point qu'on ne traite pas se trace — **tracé sur #431** par un commentaire du 2026-10-24 qui les nomme
  (C-32-3-31, C-32-3-32) ; entrée de la fiche
  32-5/32-6, dont l'agent MCP laissera des `report.generated` au nom du créateur de sa clé.
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
  mêmes dix-huit par la route, joués verts avant le déplacement des sept fonctions (premier commit de T2). Écrire les
  tests (T2) **avant** de déplacer (T3).
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

Opus 5.5 (agent de développement, en autonomie, worktree `kesh-32-3`), 2026-10-24. Choix consignés : C-32-3-34 à
C-32-3-41 (registre `epic-32-choix-autonomes-323.md`).

### Étape 0 — recontrôle des points renvoyés par les validations P6 et P7 (ligne lue citée)

- **P6-2 / P7-3** — `config.rs` : `#[cfg(test)]` l. 1665 (`mod tests` l. 1666), l. 3134 (`mod cookie_secure_tests`
  l. 3135), l. 3292 (`mod smtp_config_tests` l. 3293), l. 3600 (`mod mot_de_passe_base_tests` l. 3601) ; le premier
  `unsafe` du fichier est l. 1691 (`grep -n "unsafe" config.rs`), donc **tous** dans des modules de test, dont
  `cookie_secure_tests`. L'énumération de l'AC1 est à compléter de ce module. **Confirmé par compilation** : la lib hors
  tests compile avec `forbid(unsafe_code)` (`323-t1-build.log`).
- **P6-3 / P7-4** — la liste d'exceptions de (v-1) est la constante `EXCEPTIONS_V1` de `tests/services_motif.rs`, vide.
- **P6-4 / P7-4** — l'essai T1 garde les deux attributs (`lib.rs:9`, `services/mod.rs:100`). M14 (`poignees.rs`) : la
  note du compilateur désigne `crates/kesh-api/src/services/mod.rs:100:31` ; M14-bis (un `unsafe` dans `helpers.rs`,
  hors de `services/`) désigne `crates/kesh-api/src/lib.rs:9:31` (`323-mut-M14.log`, `323-mut-M14bis.log`).
- **P7-1** — l'entrée « Validation P6 » attribue l'atténuation à l'AC1 et à *Le motif* ; le diff `26731b29` l'a portée
  sur deux annotations du Change Log. Relevé daté, laissé tel quel (annotation P7 déjà écrite).
- **P7-2** — « un précédent publié le contredit » (AC1) est lu comme « suggère » ; **tranché par compilation** à T1 : la
  lib, qui contient `tokio::join!` (`routes/invoices.rs:1156`), compile sous `forbid(unsafe_code)` — le lint ne se
  déclenche pas dans l'expansion de cette macro externe (C-32-3-36).

### Completion Notes List

- **T0** — bases `kesh_323` et `kesh_e2e_323` créées, migrées (migrations du worktree) et seedées, remises à zéro avant
  le gate complet ; `CARGO_TARGET_DIR` du worktree ; `node_modules` par `cp -al` ; port 3323.
- **T1** — `services/{mod,acces,poignees}.rs`, `pub mod services;`, `AppState::lecture()`/`trace()` (méthodes, aucun
  champ). **Essai du crate entier concluant** : `#![cfg_attr(not(test), forbid(unsafe_code))]` gardé en tête de
  `lib.rs` (C-32-3-36) ; `cargo build -p kesh-api --all-targets` vert (`323-t1-build.log`).
- **T2 (i)** — `tests/services_motif.rs` : montage HTTP, fixtures synthétiques, `TABLE_DES_TRACES` (constante unique,
  symboles `$fy`…`$asof`), AC9 (b′). **Joué vert sur le code d'avant T3** (commit `d720bb7e`) :
  `table_des_traces_par_la_route` 1 passed (`323-t2i-route.log`) — la table est ce que le code faisait.
- **T2 (ii)** — AC8, AC9 (a)-(d), AC10 : commit `2850a9e9`, **ne compilait pas** avant T3-T4 (`E0432` sur
  `services::rapports` et `services::plan_comptable`, constaté par `cargo check`).
- **T3-T4** — `services/rapports.rs`, `services/plan_comptable.rs`, `helpers::company_for_ids`, handlers minces
  (`be6043e6`). Forme retenue : `exiger` et la validation restent deux lignes distinctes dans chaque service, la
  résolution de période passe par l'auxiliaire `resoudre_periode` (C-32-3-37) — M3 et M3′ ne rougissent que le service
  nommé. Accesseurs du `Rapport` en `Option`, lus par `attendu(…)` (C-32-3-34). Le `mod tests` de `reports.rs` est
  identique octet pour octet (commande de l'AC7 → vide).
- **AC8, cas sans objet** : pas de (c-val) ni de (c-lec) pour `balance_agee` (sa demande ne porte rien) ; pas de
  (c-lec) pour le grand livre (aucune entité nommée par la demande n'est lue avant la génération). Chaque cas (c) porte
  aussi un **contrôle** : avec la portée, la même demande échoue bien (erreur non `Forbidden`).
- **AC9 (a), branches** : les fixtures atteignent une ligne de TVA au taux `8.10` (assertion « rows non vide ») et une
  ligne de balance âgée ; un témoin établit que la `Lecture` de test refuse l'écriture (`INSERT` → 1792,
  `ac9_temoin_la_lecture_de_test_refuse_l_ecriture`). **Non atteintes, non revendiquées** : les diminutions de
  contre-prestation du rapport de TVA (aucun solde d'escompte ni de perte), les sous-projets (`resolve_scope` d'une
  racine sans enfant), un projet borné (`start_date`/`end_date`), le grand livre avec `include_zero` et pagination
  au-delà du plafond, la balance âgée avec règlement partiel.
- **AC10, mise en forme rustfmt des deux lignes de (v-2)** : `services::poignees::Lecture::new(self.pool.clone())` et
  `services::poignees::Trace::new(self.pool.clone())` (`lib.rs:153`, `:162`). Plancher du balayage de `src/` : 80 (85
  fichiers lus) ; `services/` : 5 exactement. « Ligne réduite » = `trim()` (C-32-3-39).
  *Annotation du 2026-10-24, revue de code P1 (A1)* : « 85 fichiers lus » était faux à sa date. Recompté :
  `find crates/kesh-api/src -name '*.rs' | wc -l` → **87** à `be6043e6` (identique à `210c5808`, commit de
  documentation seule) ; `git ls-tree -r --name-only 08a6f1af crates/kesh-api/src | grep -c '\.rs$'` → **82** à la
  base, soit 82 + les cinq de `services/`. Le plancher de 80 reste valide (annotation jumelle à C-32-3-39).
- **AC11 — inventaire clos** (écrit aussi au doc-comment de `services/mod.rs`) : (1) les sept fonctions d'audit, dix-huit
  sites d'appel (un par handler, par l'aiguillage `tracer`), toutes sur `Trace` ; (2) `api_keys::touch_last_used`
  (middleware, hors service, inchangée) ; (3) aucune autre — générateurs de `kesh-report`, `ReportPeriod::resolve`,
  `resolve_scope`, `identite_societe`, `company_for_ids`/`get_company_for`, `list_by_company`,
  `letterable_account_ids` ne font que lire ; confirmé sur les branches exercées par l'AC9 (a).
  `grep -rn "get_or_create" crates/kesh-api/src/routes/reports.rs crates/kesh-api/src/routes/accounts.rs
  crates/kesh-api/src/services crates/kesh-api/src/helpers.rs` → rien (au dernier commit de code).
- **AC15** : `grep -rniE "transaction read only|tx_read_only" crates/kesh-api/src` → vide.
- **AC1** : `grep -rnE "axum|AppState|CurrentUser" crates/kesh-api/src/services | grep -vE '^[^:]+:[0-9]+:[[:space:]]*//'`
  → `acces.rs:15` (`use crate::middleware::auth::CurrentUser;`) et `acces.rs:69` (`pub fn depuis_api(u: &CurrentUser)
  -> Self {`), seules.
- **AC14** : aucun manuel, `docs/api-external.md`, CHANGELOG ni README touché (rien de visible ne change) ; une phrase
  de renvoi dans `docs/MULTI-TENANT-SCOPING-PATTERNS.md` (« Every handler validates company exists ») et la ligne
  `services/` dans l'arbre de `planning-artifacts/architecture.md` (C-32-3-41) ; doc-comments de module de
  `routes/reports.rs` et `routes/accounts.rs` renvoient à `services`.

#### Mutations (T5, AC12) — une à la fois, restauration `git checkout -- <f> && touch <f>`, diff résiduel nul

Campagne `323-campagne.log` (diffs `323-mut-<nom>.diff`, journaux `323-mut-<nom>.log`), gate `binary(services_motif)`
plus le binaire nommé :

| Mutation | Rouge constaté |
|---|---|
| M1 (échafaudage `lecture_m1` + `Trace::new(self.lecture_m1.pool().clone())` aux deux `tracer`) | AC9 (b) : les **18** lignes (« 0 trace neuve ») ; AC10 (v-2) ; AC9 (b′) **vert**, comme prévu |
| M1′ (même échafaudage, `exporter` seul, garde `GrandLivre`) | AC9 (b) : la seule ligne 12 (`…_729`) ; AC10 (v-2) (attendu) |
| M1-témoin (`Trace` du test sur le pool en lecture seule) | AC9 (b) : les 18 lignes |
| M2 (`exiger` retiré de `bilan`) | AC8 (a) ×2, (c-val), (c-lec), (d) de `Bilan` |
| M2′ (`exiger` retiré de `lister`) | AC8 (a) ×2, (c-lec) de `lister` |
| M2″ (`exiger` retiré d'`identite_societe`) | ses deux assertions MCP |
| M3 (`exiger` après la génération, `journaux`) | AC8 (c-val) et (c-lec) de `Journaux` ; (a) vert |
| M3′ (`exiger` entre validation et résolution, `bilan`) | AC8 (c-val) de `Bilan` seul ; (c-lec) vert |
| M4 (`insert_in_tx` retiré, balance âgée × export) | ligne 14 de l'AC9 (b) et de l'AC9 (b′) |
| M5 (`balance_agee` exige `BooksRead`) | AC8 (a) `{BooksRead}` ; et AC8 (b) |
| M6 (`EXPORT_SANS_MCP` porte `Some(BooksRead)`) | AC8 (d) des **huit** services qui la partagent (C-32-3-35) |
| M7 (export de la balance âgée à `Consultation`) | AC8 (e) ; `aged_receivables_export_forbidden_for_consultation` **vert** (le middleware refuse d'abord) |
| M8 (`generate_balance_sheet(&state.pool, …)` dans `get_balance_sheet`) | AC10 (i) |
| M9 (tracer avant `rendre`) | AC9 (c), neuf services |
| M10 (`fiscalYearId` dans les deux fonctions de période) | lignes 1 à 10 de l'AC9 (b) et (b′), et `report_generated_audit_emitted_on_success` |
| M11 (`Lecture::pool()` en `pub(crate)` + `state.lecture().pool()` dans `get_balance_sheet`) | AC10 (v-1) |
| M11, élargissement seul | **aucun rouge** — forme que les gardes ne voient pas, déclarée (AC10) |
| M11-bis (élargissement + `Lecture::pool(&state.lecture())`) | AC10 (v-1) |
| M12 (`as_of` recalculé dans les deux branches balance âgée de `tracer`) | **aucun rouge — équivalente hors minuit**, déclarée ; la règle tient par la structure (une seule variable `as_of`) |
| M13 (`export_trial_balance` par `Affichage` + `afficher`) | AC10 (vii) et ligne 6 de l'AC9 (b′) (`action` `report.generated`) ; AC9 (b) vert |
| M14 (`transmute::<Lecture, Trace>` dans `poignees.rs`) | **rouge de compilation** : `error: usage of an unsafe block`, `note: the lint level is defined here --> crates/kesh-api/src/services/mod.rs:100:31` |
| M14-bis (`unsafe` dans `helpers.rs`, contrôle de l'attribut de la lib) | rouge de compilation, niveau défini à `crates/kesh-api/src/lib.rs:9:31` |

#### Gates réellement exécutés

- **Gate ciblé** (au commit `be6043e6`, avant la campagne) : `fmt` et `clippy -D warnings` sur le workspace verts ;
  `nextest` sur `services_motif`, les neuf binaires de l'AC7, les huit gardes de sources et la lib de `kesh-api` :
  **808 passés, 0 échec** (`323-t3-cible.log`). Tests exécutés par binaire : `services_motif` 19, `reports_e2e` 38,
  `opening_balances_e2e` 31, `reports_export_e2e` 20, `accounts_e2e` 19, `idor_multi_tenant_e2e` 14,
  `letterings_e2e` 11, `vat_report_e2e` 10, `project_reports_e2e` 7, `open_items_e2e` 7 ; gardes :
  `configuration_transmise` 30, `textes_coherents` 18, `admin_pat_denied_e2e` 16, `audit_label_registry` 11,
  `audit_route_registry` 11, `bank_account_closed_inventory` 9, `test_schema_guard` 7, `letterings_lexical` 7 ;
  lib `kesh-api` 523. *Annotation du 2026-10-24, revue de code P1 (A2)* : `323-t3-cible.log` ne porte **que** le
  `nextest` — la ligne de `mem-guard`, la compilation de `kesh-api` et `kesh-db`, puis `Starting 808 tests across 19
  binaries` et `Summary … 808 tests run: 808 passed, 1 skipped` ; `grep -c "fmt\|clippy"` y rend **0**. Le `fmt` et le
  `clippy -D warnings` verts au même commit `be6043e6` sont attestés par `323-gate-complet.log` (étapes `▶ cargo fmt
  --check` et `▶ cargo clippy (all-targets, -D warnings)`), non par ce journal. ⚠️ Les comptes de la fiche (`grep -c "sqlx::test"`) dépassent d'une unité les tests exécutés
  pour `reports_e2e` (39), `reports_export_e2e` (21), `vat_report_e2e` (11), `project_reports_e2e` (8) : la mention
  `sqlx::test` de leur commentaire d'en-tête est comptée ; `grep -c '^#\[sqlx::test'` rend 38, 20, 10, 7.
- **AC7, relancé au dernier commit de code** : `git diff --diff-filter=MDR --name-only $(git merge-base HEAD
  origin/main) HEAD -- crates/kesh-api/tests` → vide ; `grep -rlE "/api/v1/reports|/api/v1/accounts"
  crates/kesh-api/tests` → les neuf fichiers de la fiche, plus `services_motif.rs` ; `grep -rlnE
  "include_str!|read_to_string|read_dir" crates/*/tests` → les dix-sept de la fiche, plus `services_motif.rs`.
- **Gate complet** au dernier commit de code `be6043e6`, bases remises à zéro d'abord : `scripts/test-fast.sh --ci`
  (garde des droits conforme, `fmt`, `clippy`, `nextest`) → **3990 passés, 7 ignorés, 0 échec**
  (`323-gate-complet.log`) — 3971 + 19 neufs.
- **Frontend** (même révision) : `npm run check` 0 erreur (29 avertissements préexistants), `lint-i18n-ownership` PASS,
  `test:unit` **1910 passés**, `build` vert (`323-frontend-*.log`).
- **E2E Playwright complet** (même révision, recette de `docs/testing.md`, port 3323, base `kesh_e2e_323`, dossiers
  `/tmp/kesh-e2e-323`) : **315 passés, 7 échecs, 17 ignorés** (`323-e2e.log`). Les sept échecs, jugés fichier par
  fichier contre § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
  `onboarding.spec.ts:57`, `:77`, `:150` — les sept de la **KF-029 (#97)** ; aucun autre.
- Tests ajoutés (de `08a6f1af` à `be6043e6`) : **19**, tous dans `tests/services_motif.rs` (13 `#[sqlx::test]`,
  6 `#[test]`) ; aucun fichier de test existant modifié.

### File List

- `crates/kesh-api/src/services/mod.rs` (neuf)
- `crates/kesh-api/src/services/acces.rs` (neuf)
- `crates/kesh-api/src/services/poignees.rs` (neuf)
- `crates/kesh-api/src/services/rapports.rs` (neuf)
- `crates/kesh-api/src/services/plan_comptable.rs` (neuf)
- `crates/kesh-api/src/lib.rs`
- `crates/kesh-api/src/routes/reports.rs`
- `crates/kesh-api/src/routes/accounts.rs`
- `crates/kesh-api/src/helpers.rs`
- `crates/kesh-api/tests/services_motif.rs` (neuf)
- `docs/MULTI-TENANT-SCOPING-PATTERNS.md`
- `_bmad-output/planning-artifacts/architecture.md`
- `_bmad-output/implementation-artifacts/32-3-services-motif.md`
- `_bmad-output/implementation-artifacts/epic-32-choix-autonomes-323.md`
- `_bmad-output/implementation-artifacts/sprint-status.yaml`
- `_bmad-output/implementation-artifacts/323-review-prompt-p1.md` (neuf)
- `_bmad-output/implementation-artifacts/323-review-prompt-p2.md` (neuf, revue de code P1)

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
    esquisse alignés (C-32-3-24). *(Annoté en P3 : les cinq inventaires, M11′ et M11″ sont retirés — C-32-3-31.)*
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
- **2026-10-24 — Validation P3** (Sonnet, deux lentilles en contexte frais ; rapports `kesh-gate-logs/323-validate-p3-R.md`
  et `-F.md`) : **R 0/0/1/8, F 0/0/0/10** (CRITICAL/HIGH/MEDIUM/LOW). Recoupements complets : R1 = F1 (R en MEDIUM, F en
  LOW : retenu MEDIUM), R3 = F4, R5 = F5, R6 = F2, R7 = F3 ; R8 et F7 se recoupent en partie (M1′ et (v-d)), comptés à
  part — soit **quatorze findings distincts** (19 rapportés − 5) : **un MEDIUM**, treize LOW. **Tendance des MEDIUM
  distincts : P1 6 → P2 5 → P3 1.** ⚠️ **R1 = F1 est la troisième forme recyclée** de « le pool d'une poignée est
  atteignable » (P1 F2, P2 R1 = F1, P3 R1 = F1) : chaque correction lexicale ouvrait une échappée nouvelle — ici
  `.pool()` lui-même, que les inventaires ne cherchaient pas. **Décision de l'orchestrateur : retrait** (registre
  C-32-3-31). La fiche **cesse de promettre une preuve lexicale d'inaccessibilité du pool** ; elle garde la visibilité
  Rust comme garde principale (champs privés, `pool()` en `pub(in crate::services)` — `pub(in crate::services::rapports)`
  pour `Trace::pool()`, proposé par F1, n'est pas écrivable : E0742, `rapports` n'est pas un ancêtre de `poignees`),
  deux gardes simples ((v-1) aucun `.pool()` hors de `services/` ; (v-2) `\bLecture::new\(`/`\bTrace::new\(` au seul
  câblage de `lib.rs`) avec (i), (ii), (vi), (vii), l'exécution (AC9 (b) sur la `Lecture` en lecture seule, dit
  explicitement ; M1 ; M1-témoin), et un paragraphe *Ce que les gardes ne voient pas* écrit en clair, qui renvoie la
  garantie contre une écriture sur la `Lecture` à la 32-6 (Q11) et à la revue d'ici là. Retirés : inventaires (v-a) à
  (v-e), M11′, M11″ ; M11 refondue (élargissement **et** appel d'un handler → (v-1)). Sort des LOW :
  - **R2** — `unsafe`/`transmute`, ré-export au paragraphe *Ce que les gardes ne voient pas* (et un `forbid` de module
    à `services/mod.rs` signalé à l'arbitrage, le crate entier ne pouvant le prendre). **R3/F4** — (vii) cherche la
    présence hors commentaires ; AC1 et chapeau de l'AC10 bornés aux volets d'absence. **R4** — `\b` dans (v-2) ;
    « multiensemble » dit. **R5/F5** — sans objet ((v-b) retiré). **R6/F2** — esquisse sans décompte. **R7/F3** — six
    validations sous le `span` (l. 330, 462, 660, 970, 1034, 1144), trois exports sans `span`. **R8** — (b′) : seul
    volet du mauvais **service** ; M1′ rougit aussi (v-2). **R9** — M10 dans les deux fonctions de période.
  - **F6** — T2 en deux commits, (b′) joué vert sur le code d'avant T3. **F7** — forme du service supposée par M3/M3′,
    cas de l'auxiliaire commun consigné ; M1′. **F8** — (b) passe la `Lecture` en lecture seule, dit ; (b′) rejoue la
    ligne sans bornes avec les bornes de l'exercice seedé. **F9** — L2 rattachée à #431 (qui ne couvre que les dépôts) :
    commentaire sur #431 ou issue neuve **demandé à l'orchestrateur** ; AC14 nomme `docs/MULTI-TENANT-SCOPING-
    PATTERNS.md` et `architecture.md`. **F10** — doc-comment de conventions rattaché à `emit_report_audit` (AC6, T3).
  - Propagation grepée sur la fiche, le registre et la clé, par valeur : `(v-a)` à `(v-e)`, `v-d`, `inventaire clos`,
    `M11′`, `M11″`, `huit lignes`, `tout chemin`, `attrape`, `Ce que (v)`, `n'étant appelé`, `seules fonctions d'audit` ;
    survivances laissées : relevés datés (Change Log P2, annoté ; registre C-32-3-24, annoté). Prompt de la passe P4 :
    `323-validate-prompt-p4.md` (**ciblée**, une lentille, Haiku, sur le seul diff de cette remédiation).
    *(Annoté en P4 : (v-1) ne cherchait que `.pool()` et laissait vert l'appel qualifié — élargie, M11-bis ; le
    `forbid(unsafe_code)` de module signalé à l'arbitrage est **retenu** ; F9 « #431 ne couvre que les dépôts » était
    trop étroit, et le traçage est fait — C-32-3-32.)*
- **2026-10-24 — Validation P4** (Haiku, passe **ciblée**, une lentille — regression hunter —, sur le seul diff de la
  remédiation P3, `1269b07a` ; rapport `kesh-gate-logs/323-validate-p4.md`) : **0/0/1/4** (CRITICAL/HIGH/MEDIUM/LOW),
  cinq findings, tous nés de la remédiation P3. Sort de chacun (registre C-32-3-32) :
  - **P4-1 (MEDIUM)** — (v-1) ne voyait que la forme méthode `.pool()` : l'appel qualifié `Lecture::pool(&x)` et le
    chemin `.map(Lecture::pool)` compilaient une fois `pool()` élargi et laissaient la garde verte, alors que la fiche
    affirmait qu'elle nommait le rouge. **Corrigé** : motif `\.\s*pool\s*\(|::\s*pool\b` hors commentaires, sur tout
    `src/` hors `services/`, baseline vide revérifiée ; témoins positifs des trois formes et témoin négatif `state.pool` ;
    mutation **M11-bis** ; formes restées hors d'atteinte (appel coupé entre deux lignes, commentaire de bloc dans
    l'appel, identifiant de macro) écrites à *Ce que les gardes ne voient pas* ; phrases de (v-1), R8, *Hors
    périmètre* et du paragraphe d'exécution alignées ; entrée P3 annotée (non réécrite).
  - **P4-2 (LOW), décision de l'orchestrateur** — « `forbid` sur le crate : impossible » était inexact (les `unsafe` de
    `config.rs` sont tous sous `#[cfg(test)] mod tests`). **Retenu** : `#![cfg_attr(not(test), forbid(unsafe_code))]`
    en tête de `services/mod.rs` (AC1, *Le motif*, AC3, R4, R8) ; mutation **M14** (un `transmute` dans `services/` ne
    compile pas). Le crate entier ne le prend pas : `routes/invoices.rs:1156` emploie `tokio::join!`, dont l'expansion
    porte un `unsafe` (tokio 1.52.3, `macros/join.rs:176`) — raisonné, non compilé. `unsafe`/`transmute` quittent *Ce
    que les gardes ne voient pas* pour `services/`, et y restent **hors** de `services/`.
  - **P4-3 (LOW)** — L2 : #431 couvre aussi des sites de route (`unvalidate`, Story 15-8b) ; le commentaire du
    2026-10-24 sur #431 existe : « traçage demandé à l'orchestrateur » devient « tracé sur #431 » (fait 6, *Hors
    périmètre*).
  - **P4-4 (LOW)** — (vii) : le retrait des `/* … */` se fait sur le corps entier, blocs de plusieurs lignes compris ;
    témoin positif ajouté.
  - **P4-5 (LOW)** — (b′) : chemin complet `crates/kesh-db/src/test_fixtures.rs:118` (`seed_accounting_company`) ; la
    table est paramétrée par des symboles (`fy`, `debut`, `fin`…), les deux dates de l'exercice seedé étant les seuls
    littéraux, ceux de la ligne sans bornes.
  - Propagation grepée sur la fiche, le registre et la clé, par valeur : `.pool()`, `\.pool\(`, `forbid(unsafe`,
    `transmute`, `unsafe`, `demandé à l'orchestrateur`, `ne couvre que les dépôts`, `ne nomme que les dépôts`,
    `test_fixtures.rs:118`, `M1 à M13`, `C-32-3-31)` en en-tête ; survivances laissées : relevés datés (Change Log P1 à
    P3, annotés ; corps de C-32-3-31, annoté). La boucle n'est pas close : un MEDIUM appelle une passe de plus. Prompt
    de la passe P5 : `323-validate-prompt-p5.md` (**ciblée**, une lentille — regression hunter —, **Sonnet**, sur le
    seul diff de cette remédiation).
    *(Annoté en P5 : la prémisse « l'expansion de `tokio::join!` porte un `unsafe` que le lint refuserait », écrite
    ci-dessus comme un fait, n'était pas établie — un seul précédent lu, non compilé, suggère que le lint ne se déclenche pas dans l'expansion d'une macro d'une autre
    crate, précédent `testcontainers-0.26.3` ; la contrainte « un service n'emploie pas `tokio::join!` » est retirée,
    l'extension au crate devient un essai de T1 ; P5-3 : les `unsafe` de `config.rs` sont aussi dans
    `smtp_config_tests` — C-32-3-33.)*
- **2026-10-24 — Validation P5** (Sonnet, passe **ciblée**, une lentille — regression hunter —, sur le seul diff de la
  remédiation P4, `a2eb9e6f` ; rapport `kesh-gate-logs/323-validate-p5.md`) : **0/0/1/2** (CRITICAL/HIGH/MEDIUM/LOW),
  trois findings, tous nés de la remédiation P4. Sort de chacun (registre C-32-3-33) :
  - **P5-1 (MEDIUM)** — la prémisse « `tokio::join!` porte un `unsafe` que `forbid(unsafe_code)` refuserait » n'est
    pas établie (un seul précédent lu, non compilé, suggère que le lint ne se déclenche pas dans l'expansion d'une macro externe ; `testcontainers-0.26.3` porte
    `#![forbid(unsafe_code)]` et emploie `tokio::select!` en code non-test). **Retrait** : la contrainte « un service
    n'emploie pas `tokio::join!` » quitte l'AC1 (seul site vivant) ; l'attribut reste au module `services/` ; son
    extension au crate est **à essayer au développement** (T1 : poser, compiler, garder s'il passe, sinon consigner la
    cause réelle constatée) ; ton harmonisé — l'effet de l'attribut est dit raisonné, établi par M14 (AC1, AC3, *Le
    motif*) ; *Ce que les gardes ne voient pas* et *Hors périmètre* nuancés ; entrée P4 et C-32-3-32 annotées (non
    réécrites).
  - **P5-2 (LOW)** — (v-1) : formes hors d'atteinte ajoutées (l'identifiant brut `r#pool`, la coupure entre `pool` et
    `(`) ; faux rouges futurs cités (un `AppState::pool()`, toute méthode `.pool(` d'un autre type) avec la conduite :
    élargir la liste d'exceptions écrite de la garde, jamais affaiblir le motif.
  - **P5-3 (LOW)** — `config.rs` : les `unsafe` sont aussi dans `smtp_config_tests` (l. 3293-3596), tous sous
    `#[cfg(test)]` ; désignation corrigée (AC1).
  - **Tendance des MEDIUM distincts : P1 6 → P2 5 → P3 1 → P4 1 → P5 1**, chacun des trois derniers né de la
    remédiation précédente — signal de recyclage **déclaré au Project Lead** (règle de splitting préventif, amendement
    du 2026-10-06) : la sévérité ne décroît plus, elle se déplace sur ce que chaque remédiation écrit.
  - Propagation grepée sur la fiche, le registre et la clé, par valeur : `join!`, `séquence`, `vraisemblablement`,
    `1665-1666`, `smtp_config_tests`, `r#pool`, `sqlx::pool`, `ne couvre que`, `forbid` ; survivances
    laissées : relevés datés (Change Log P4, annoté ; C-32-3-32, annoté). La boucle n'est pas close : un MEDIUM appelle
    une passe de plus. Prompt de la passe P6 : `323-validate-prompt-p6.md` (**ciblée**, une lentille — regression
    hunter —, **Haiku**, sur le seul diff de cette remédiation).

### Validation P6 (2026-10-24, passe ciblée, une lentille — regression hunter —, Haiku 4.5, sur le diff de la P5)

- **Résultat : 0 CRITICAL, 0 HIGH, 1 MEDIUM, 4 LOW.** Remédiation par **retrait pur** : aucune affirmation ajoutée,
  les affirmations non établies sont atténuées.
  - **P6-1 (MEDIUM)** — la P5 affirmait comme établi l'effet du lint `unsafe_code` sur l'expansion d'une macro d'une
    autre crate ; un seul précédent lu, non compilé. Atténué aux deux sites de la fiche (AC1, *Le motif*) et aux deux
    du registre : « un seul précédent lu, non compilé, suggère que le lint ne se déclenche pas » ; la mutation M14 et
    l'essai T1 l'établissent au développement.
  - **P6-2 (LOW)** — `config.rs` porte aussi des `unsafe` dans `cookie_secure_tests` (l. 3134-3286), sous
    `#[cfg(test)]` comme les autres : à relever au développement (T1), avec l'essai du `forbid` crate entier.
  - **P6-3 (LOW)** — la liste d'exceptions de (v-1), vide en 32-3, n'est pas ancrée à un nom de constante : le
    développement la nomme dans le test et la recopie au Dev Agent Record.
  - **P6-4 (LOW)** — si l'essai T1 garde à la fois l'attribut de `services/mod.rs` et celui de `lib.rs`, M14 se joue
    sur les deux : le Dev Agent Record dit lequel a rougi.
  - **P6-5 (LOW)** — `sprint-status.yaml` nommait la plage « C-32-3-1 à 16 » : portée à « C-32-3-1 à 33 ».
- **Tendance des MEDIUM distincts : P1 6 → P2 5 → P3 1 → P4 1 → P5 1 → P6 1**, chacun né de la remédiation
  précédente ; celle-ci ne fait que retirer. La boucle n'est pas close (un MEDIUM appelle une passe de plus) :
  prompt de la P7, `323-validate-prompt-p7.md` (**ciblée**, une lentille, **Sonnet**, sur le seul diff de cette
  remédiation).

### Validation P7 (2026-10-24, passe ciblée, une lentille — regression hunter —, Sonnet, sur `26731b29` ; rapport `kesh-gate-logs/323-validate-p7.md`)

- **Résultat : 0 CRITICAL, 0 HIGH, 0 MEDIUM, 4 LOW**, tous nés de la remédiation P6. **La boucle de validation est
  terminée** : P7 ne rend aucun finding au-dessus de LOW. Tendance des MEDIUM distincts : **P1 6 → P2 5 → P3 1 → P4 1
  → P5 1 → P6 1 → P7 0** (modèles : Sonnet, Opus, Sonnet, Haiku, Sonnet, Haiku, Sonnet).
- Les quatre LOW **ne sont pas appliqués au texte** — une remédiation écrite après la dernière passe ne serait relue
  par personne (`CLAUDE.md`, *Review Iteration Rule*) — et se **recontrôlent à l'étape 0 du développement**, ligne lue
  citée au Dev Agent Record :
  - **P7-1** — l'entrée « Validation P6 » ci-dessus dit avoir atténué « l'AC1 et *Le motif* » : le diff `26731b29`
    atténue en réalité deux annotations du Change Log (entrée P4 annotée et entrée P5). Relevé daté inexact à sa date,
    corrigé par cette annotation, non réécrit.
  - **P7-2** — l'AC1 garde « un précédent publié le contredit », plus affirmatif que « un seul précédent lu, non
    compilé, suggère » : **à lire au développement comme ce dernier** ; l'essai T1 et M14 tranchent.
  - **P7-3** — `config.rs` porte aussi `cookie_secure_tests` (l. 3134-3286, sous `#[cfg(test)]`) ; aucun `unsafe` hors
    des modules de test (relu par la lentille) : l'énumération de l'AC1 est à compléter de lui à T1.
  - **P7-4** — P6-3 (nommer la constante d'exceptions de la garde (v-1)) et P6-4 (M14 sur les deux attributs si
    l'essai T1 les garde) font partie des tâches T1 et M14, bien qu'écrits seulement ici.

### Développement (2026-10-24, Opus 5.5, agent de développement, en autonomie)

- T0 à T6 faits ; commits `7a2ac2f0` (T1), `d720bb7e` (T2 (i), vert avant T3), `64d957a5` (rustfmt), `2850a9e9`
  (T2 (ii), rouge de compilation), `be6043e6` (T3-T4, dernier commit de code), puis le commit de documentation de fin.
- Gates au dernier commit de code : complet **3990/3990** (7 ignorés), frontend 1910 Vitest, E2E **315 passés, 7
  échecs** (les sept de KF-029), détail au Dev Agent Record. Vingt-deux mutations jouées, toutes tuées sauf deux
  déclarées (M11 élargissement seul, M12 équivalente).
- Choix C-32-3-34 à C-32-3-41 ; essai T1 concluant (`forbid(unsafe_code)` sur la lib). Écart à la fiche : aucun sur le
  comportement ; les quatre comptes `sqlx::test` de l'AC7 comptaient une mention de commentaire (Dev Agent Record).
- Prompt de la revue de code P1 : `323-review-prompt-p1.md` (complète, **Sonnet**, lentilles B, E, A).

### Revue de code P1 (2026-10-24, complète, trois lentilles en contexte frais — B, E, A —, Sonnet ×3 ; rapports `kesh-gate-logs/323-review-p1-{B,E,A}.md`)

- **Résultat : B 0/0/0/5, E 0/0/1/3, A 0/0/0/5** (CRITICAL/HIGH/MEDIUM/LOW) — **un MEDIUM** (E1), quatorze LOW. Trend de
  la boucle de revue : **P1 1 MEDIUM** ; une passe P2 est due (`CLAUDE.md` : une boucle ne se clôt que sur une passe à
  0 MEDIUM+). Prompt versionné : `323-review-prompt-p2.md` (passe **ciblée**, une lentille — regression hunter —,
  **Opus**, sur le seul diff `210c5808..HEAD`).
- **Remédiation : tests et comptes rendus seulement, aucune ligne de production** (`git diff --stat 210c5808 HEAD` :
  `crates/kesh-api/tests/services_motif.rs` et trois artefacts de `_bmad-output/`). Commits `1d27f065` (test) et le
  commit de documentation qui porte cette entrée.
  - **E1 (MEDIUM) — appliqué.** Prémisse lue au code du générateur : `kesh-report/src/general_ledger.rs`, `generate`
    écrête `limit` à `[1, MAX_LEDGER_LIMIT]` (`options.limit.map(|l| l.clamp(1, MAX_LEDGER_LIMIT))`) et `fetch_lines`
    pagine **par compte** (`WHERE jel.account_id = ? … LIMIT ? OFFSET ?`, `None` → `i64::MAX`). Test neuf
    `grand_livre_les_deux_regimes` : une écriture synthétique de `MAX_LEDGER_LIMIT + 2` lignes, dont
    `MAX_LEDGER_LIMIT + 1` (501) sur le compte `1100` ; export sans paramètre → 501 lignes ; export avec `offset=1`,
    `limit=1` → les mêmes 501 ; affichage sans paramètre → les 500 premières ; affichage `offset=1`, `limit=1` → la
    deuxième seule ; `line_count` 501 partout. **Quatre mutations jouées, toutes tuées** (une à la fois, copie de
    sauvegarde, restauration `git checkout -- <f> && touch <f>`, diff résiduel nul et `cmp` à la copie vérifiés ;
    gate `binary(services_motif)`, campagne `323-p1rem-campagne.log`, diffs `323-p1rem-mut-<nom>.diff`, journaux
    `.log`) :

    | Mutation (`services/rapports.rs::grand_livre`) | Rouge constaté |
    |---|---|
    | (a) export `(0, Some(MAX_LEDGER_LIMIT))` | `grand_livre_les_deux_regimes` : « export : le livre entier (501 lignes), jamais tronqué au plafond 500 » (500 ≠ 501) |
    | (b) export `(demande.offset.unwrap_or(0), None)` | même test : « export : `offset` et `limit` sont ignorés » (lignes 2…501 ≠ 1…501) |
    | (c1) affichage `limit: demande.limit` (sans plafond par défaut) | même test : « affichage : le défaut est le plafond (500 lignes) » (501 ≠ 500) |
    | (c2) affichage `offset` à `0` (non lu) | même test : « affichage : `offset=1&limit=1` rend la deuxième ligne seule » (`[1]` ≠ `[2]`) |

  - **E2 (LOW) — appliqué, test court.** La ligne 8 de `TABLE_DES_TRACES` devient un export **filtré**
    (`journaux_export_csv_filtre_achats`, `journal: Some("Achats")`, `"journal_filter": "Achats"`) ; le `null` d'un
    export à période reste tenu par les lignes 2, 4, 6 et 10. Table toujours de dix-huit lignes. Mutation jouée
    (`journal_filter.as_deref()` → `None` dans la seule branche d'export de `tracer`) : **tuée** par
    `ac9_b_table_des_traces_au_service` (`service#8`) et `table_des_traces_par_la_route` (`route#8`).
  - **A1 (LOW) — appliqué par annotation datée** au Dev Agent Record et à C-32-3-39 du registre : 87 fichiers `.rs` à
    `be6043e6` (`find crates/kesh-api/src -name '*.rs' | wc -l`), 82 à `08a6f1af` (`git ls-tree`), jamais 85.
  - **A2 (LOW) — appliqué par annotation datée** au Dev Agent Record : `323-t3-cible.log` ne porte que le `nextest`
    (`grep -c "fmt\|clippy"` → 0) ; `fmt` et `clippy` verts au même commit sont attestés par `323-gate-complet.log`.
  - **Non traités, à verser à une issue par l'orchestrateur** : **B1** (`Rapport` sans `#[must_use]` : lecture sans
    trace possible si le `Rapport` est abandonné) ; **B2** (le contenu d'un export lisible dans la fermeture `rendre`
    qui rend `Err`, sans trace — doc de module plus fort que le type) ; **B3** (accesseurs `Option` + `attendu(…)` : un
    état impossible exprimé à l'exécution) ; **B4** (gardes lexicales non étanches : alias, `<Lecture>::new(`,
    déstructuration de `AppState`, `ContexteAcces::mcp(` non gardé) ; **B5** (doc-comment `/// GET
    /api/v1/reports/trial-balance` mal placé sur `get_general_ledger`, hérité) ; **E3** (l'ordre `project_id <= 0` →
    `resolve_scope` → `fiscalYearId` requis n'est figé par aucun test) ; **E4** (la branche « trace en échec, réponse
    intacte » n'est exercée nulle part) ; **A3** (diffs de M1/M1′ non recopiés au Dev Agent Record, renvoyés à
    `kesh-gate-logs/`) ; **A4** (texte de l'AC1 resté périmé sur trois points réglés à l'Étape 0) ; **A5** (AC10 (i)/(ii)
    sans témoin positif, (ii) sans mutation) ; **A6** (non vérifiable en lecture seule : remise à zéro des bases avant
    le gate, révision des journaux frontend, campagne non rejouée).
- **Gates réellement exécutés** (au commit de test `1d27f065`, dernier commit de code de la remédiation) :
  - **gate ciblé** avant la campagne : `cargo fmt --all -- --check` vert (`323-p1rem-fmt.log`, après `cargo fmt` du
    seul fichier de test), `cargo clippy --workspace --all-targets -- -D warnings` vert (`323-p1rem-clippy.log`),
    `nextest -E 'binary(services_motif)'` **20 passés** (`323-p1rem-cible.log`) ;
  - **gate complet** `scripts/test-fast.sh --ci`, bases `kesh_323` et `kesh_e2e_323` remises à zéro d'abord (drop,
    create, migrations du worktree, seed) : garde des droits conforme, `fmt`, `clippy`, **3991 passés, 7 ignorés,
    0 échec** (`323-p1rem-gate-complet.log`, première ligne `1d27f065`) — 3990 + 1 ;
  - **E2E complet** (recette de `CLAUDE.md` § E2E, port 3323, base `kesh_e2e_323`, dossiers `/tmp/kesh-e2e-323`,
    `KESH_INBOX_DIR`/`KESH_DOCUMENTS_DIR` passés aux deux côtés ; frontend `build` de `be6043e6`, aucun fichier de
    `frontend/` changé depuis) : **315 passés, 7 échecs, 17 ignorés** (`323-p1rem-e2e.log`, révision
    `323-p1rem-e2e-revision.txt` = `1d27f065`). Les sept échecs, jugés fichier par fichier contre `docs/testing.md`
    § *Les échecs attendus* : `mode-expert.spec.ts:26`, `:41`, `onboarding-path-b.spec.ts:65`, `:92`,
    `onboarding.spec.ts:57`, `:77`, `:150` — les sept de la **KF-029 (#97)** ; aucun autre. Backend arrêté par PID.
  - Gates frontend non rejoués : aucun fichier de `frontend/` touché par la remédiation.
- **Décompte** (de `210c5808` à `1d27f065`) : **un test ajouté**, `grand_livre_les_deux_regimes` ; `services_motif.rs`
  passe de 19 à **20** (`grep -c '#\[sqlx::test'` → 14, `grep -cE '^#\[test\]'` → 6) ; une ligne de la table modifiée.
- Statut inchangé : `review`.
